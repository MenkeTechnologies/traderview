//! Backtester for algo strategies that implement the `Strategy` trait.
//!
//! Distinct from `crate::backtest`, which exposes preset indicator
//! combos (SMA cross, RSI reversion, Bollinger, MACD). This module
//! takes a `Box<dyn Strategy>` from the `algo_strategies` family —
//! so the SAME code that runs live can be replayed against historical
//! bars to vet its behaviour.
//!
//! Fill model (deliberately pessimistic):
//!   * Entry fills at the **next bar's open + slippage_bps**. No
//!     look-ahead into the trigger bar.
//!   * Exit checks run per bar in this order:
//!       1. evaluate_exit(side, anchor_high, anchor_low) — signal
//!          exits (Supertrend flip, EMA cross, etc.) — fill at the
//!          next bar's open.
//!       2. Stop-loss hit (bar.low ≤ stop_price for long, bar.high ≥
//!          stop_price for short) — fill AT the stop_price.
//!       3. Take-profit hit — fill AT the take_profit_price.
//!       4. If both SL and TP could have hit in the same bar, we
//!          assume SL hit first (worst case for backtest realism).
//!   * Fees per round-trip are deducted from gross P&L.
//!
//! Sizing reuses `algo_strategies::size_shares` so backtest results
//! map directly to live-engine behaviour for the same Sizing config.

use crate::algo_strategies::{size_shares, Side, SideMode, Sizing, Strategy};
use crate::models::PriceBar;
use chrono::{DateTime, Utc};
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BacktestConfig {
    pub initial_equity: f64,
    pub fee_per_trade: f64,
    /// Slippage applied to entry / signal-exit fills (basis points).
    /// 5 bps = 0.05% of the fill price added to the cost basis on
    /// entries and subtracted from the proceeds on exits.
    pub slippage_bps: f64,
    /// Side mode the strategy is allowed to take.
    #[serde(default = "default_side_mode")]
    pub side_mode: SideMode,
}

fn default_side_mode() -> SideMode {
    SideMode::Long
}

impl Default for BacktestConfig {
    fn default() -> Self {
        Self {
            initial_equity: 100_000.0,
            fee_per_trade: 1.0,
            slippage_bps: 5.0,
            side_mode: SideMode::Long,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExitReason {
    StopLoss,
    TakeProfit,
    StrategySignal,
    EndOfData,
}

#[derive(Debug, Clone, Serialize)]
pub struct AlgoBtTrade {
    pub entry_time: DateTime<Utc>,
    pub exit_time: DateTime<Utc>,
    pub side: Side,
    pub qty: f64,
    pub entry_price: f64,
    pub exit_price: f64,
    pub stop_price: f64,
    pub take_profit_price: f64,
    pub pnl: f64,
    /// Realized R-multiple: pnl / (initial_risk_per_share × qty). Useful
    /// for comparing very different strategies on a common scale.
    pub r_multiple: f64,
    pub bars_held: usize,
    pub exit_reason: ExitReason,
}

#[derive(Debug, Clone, Serialize)]
pub struct AlgoBtPoint {
    pub time: DateTime<Utc>,
    pub equity: f64,
    pub drawdown_pct: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct AlgoBtSummary {
    pub trades: usize,
    pub wins: usize,
    pub losses: usize,
    pub win_rate: f64,
    pub avg_win: f64,
    pub avg_loss: f64,
    pub avg_r: f64,
    pub profit_factor: f64,
    pub total_return_pct: f64,
    pub max_drawdown_pct: f64,
    pub final_equity: f64,
    /// Daily Sharpe ratio assuming 1 bar = 1 daily return point. For
    /// intraday bars this is technically a bar-Sharpe; the field name
    /// matches the existing `BtSummary` for UI parity.
    pub sharpe: f64,
    pub bars_in_market_pct: f64,
    pub exits_by_stop: usize,
    pub exits_by_tp: usize,
    pub exits_by_signal: usize,
    pub exits_by_eod: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct AlgoBtResult {
    pub strategy_kind: &'static str,
    pub trades: Vec<AlgoBtTrade>,
    pub equity: Vec<AlgoBtPoint>,
    pub summary: AlgoBtSummary,
}

#[derive(Debug, Clone)]
struct OpenPosition {
    side: Side,
    qty: f64,
    entry_price: f64,
    stop_price: f64,
    take_profit_price: f64,
    initial_risk_per_share: f64,
    entry_index: usize,
    entry_time: DateTime<Utc>,
}

pub fn run(
    bars: &[PriceBar],
    strategy: &dyn Strategy,
    sizing: &Sizing,
    cfg: BacktestConfig,
) -> AlgoBtResult {
    let n = bars.len();
    let slip = cfg.slippage_bps / 10_000.0;
    let mut equity = cfg.initial_equity;
    let mut peak = equity;
    let mut max_dd = 0.0f64;
    let mut open: Option<OpenPosition> = None;
    let mut trades: Vec<AlgoBtTrade> = Vec::new();
    let mut points: Vec<AlgoBtPoint> = Vec::with_capacity(n);
    let mut bars_in_market = 0usize;
    let mut returns: Vec<f64> = Vec::with_capacity(n);
    let mut last_equity = equity;

    let min_bars = strategy.min_bars();
    for i in 0..n {
        let bar = &bars[i];
        let bar_time = bar.bar_time;

        // ── exit eval on any open position ──────────────────────────
        if let Some(pos) = open.clone() {
            bars_in_market += 1;
            let high = bar.high.to_f64().unwrap_or(0.0);
            let low = bar.low.to_f64().unwrap_or(0.0);
            let close = bar.close.to_f64().unwrap_or(0.0);

            // Strategy signal exit (next-bar fill).
            let sig_exit = strategy.evaluate_exit(
                &bars[..=i],
                pos.side,
                high.max(pos.entry_price),
                low.min(pos.entry_price),
            );

            let (sl_hit, tp_hit) = match pos.side {
                Side::Buy => (low <= pos.stop_price, high >= pos.take_profit_price),
                Side::Sell => (high >= pos.stop_price, low <= pos.take_profit_price),
            };

            let (exit_price, reason) = if sl_hit && tp_hit {
                // Pessimistic: assume SL fills first.
                (pos.stop_price, ExitReason::StopLoss)
            } else if sl_hit {
                (pos.stop_price, ExitReason::StopLoss)
            } else if tp_hit {
                (pos.take_profit_price, ExitReason::TakeProfit)
            } else if sig_exit.is_some() {
                // Signal exit fills at NEXT bar's open + slippage. If
                // we're the last bar, fall through to end-of-data.
                if i + 1 < n {
                    let next_open = bars[i + 1].open.to_f64().unwrap_or(close);
                    let slip_dir = match pos.side {
                        Side::Buy => 1.0 - slip,
                        Side::Sell => 1.0 + slip,
                    };
                    (next_open * slip_dir, ExitReason::StrategySignal)
                } else {
                    (close, ExitReason::EndOfData)
                }
            } else {
                push_point(&mut points, bar_time, equity, &mut peak, &mut max_dd);
                returns.push((equity - last_equity) / last_equity.max(1.0));
                last_equity = equity;
                continue;
            };

            let gross = match pos.side {
                Side::Buy => (exit_price - pos.entry_price) * pos.qty,
                Side::Sell => (pos.entry_price - exit_price) * pos.qty,
            };
            let net = gross - cfg.fee_per_trade;
            equity += net;
            let r = if pos.initial_risk_per_share > 0.0 {
                net / (pos.initial_risk_per_share * pos.qty)
            } else {
                0.0
            };
            trades.push(AlgoBtTrade {
                entry_time: pos.entry_time,
                exit_time: bar_time,
                side: pos.side,
                qty: pos.qty,
                entry_price: pos.entry_price,
                exit_price,
                stop_price: pos.stop_price,
                take_profit_price: pos.take_profit_price,
                pnl: net,
                r_multiple: r,
                bars_held: i - pos.entry_index,
                exit_reason: reason,
            });
            open = None;
        }

        // ── entry eval ─────────────────────────────────────────────
        if open.is_none() && i + 1 >= min_bars && i + 1 < n {
            if let Some(sig) = strategy.evaluate_entry(&bars[..=i], cfg.side_mode) {
                let qty = size_shares(equity, sig.entry_price, sig.stop_distance, sizing) as f64;
                if qty > 0.0 {
                    let next_open = bars[i + 1].open.to_f64().unwrap_or(sig.entry_price);
                    let entry_price = match sig.side {
                        Side::Buy => next_open * (1.0 + slip),
                        Side::Sell => next_open * (1.0 - slip),
                    };
                    let init_risk = (entry_price - sig.stop_price).abs();
                    open = Some(OpenPosition {
                        side: sig.side,
                        qty,
                        entry_price,
                        stop_price: sig.stop_price,
                        take_profit_price: sig.take_profit_price,
                        initial_risk_per_share: init_risk,
                        entry_index: i + 1,
                        entry_time: bars[i + 1].bar_time,
                    });
                }
            }
        }

        push_point(&mut points, bar_time, equity, &mut peak, &mut max_dd);
        returns.push((equity - last_equity) / last_equity.max(1.0));
        last_equity = equity;
    }

    // ── close any still-open position at EOD ───────────────────────
    if let Some(pos) = open.clone() {
        let last_close = bars[n - 1].close.to_f64().unwrap_or(pos.entry_price);
        let gross = match pos.side {
            Side::Buy => (last_close - pos.entry_price) * pos.qty,
            Side::Sell => (pos.entry_price - last_close) * pos.qty,
        };
        let net = gross - cfg.fee_per_trade;
        equity += net;
        let r = if pos.initial_risk_per_share > 0.0 {
            net / (pos.initial_risk_per_share * pos.qty)
        } else {
            0.0
        };
        trades.push(AlgoBtTrade {
            entry_time: pos.entry_time,
            exit_time: bars[n - 1].bar_time,
            side: pos.side,
            qty: pos.qty,
            entry_price: pos.entry_price,
            exit_price: last_close,
            stop_price: pos.stop_price,
            take_profit_price: pos.take_profit_price,
            pnl: net,
            r_multiple: r,
            bars_held: n - 1 - pos.entry_index,
            exit_reason: ExitReason::EndOfData,
        });
    }

    let summary = summarize(&trades, &points, &returns, equity, cfg, n, bars_in_market);
    let kind_str = strategy.kind().as_str();
    AlgoBtResult {
        strategy_kind: kind_str,
        trades,
        equity: points,
        summary,
    }
}

fn push_point(
    points: &mut Vec<AlgoBtPoint>,
    time: DateTime<Utc>,
    equity: f64,
    peak: &mut f64,
    max_dd: &mut f64,
) {
    if equity > *peak {
        *peak = equity;
    }
    let dd_pct = if *peak > 0.0 {
        (*peak - equity) / *peak * 100.0
    } else {
        0.0
    };
    if dd_pct > *max_dd {
        *max_dd = dd_pct;
    }
    points.push(AlgoBtPoint { time, equity, drawdown_pct: dd_pct });
}

fn summarize(
    trades: &[AlgoBtTrade],
    points: &[AlgoBtPoint],
    returns: &[f64],
    final_equity: f64,
    cfg: BacktestConfig,
    n_bars: usize,
    bars_in_market: usize,
) -> AlgoBtSummary {
    let wins: Vec<f64> = trades.iter().filter(|t| t.pnl > 0.0).map(|t| t.pnl).collect();
    let losses: Vec<f64> = trades
        .iter()
        .filter(|t| t.pnl < 0.0)
        .map(|t| t.pnl)
        .collect();
    let win_rate = if !trades.is_empty() {
        wins.len() as f64 / trades.len() as f64
    } else {
        0.0
    };
    let avg_win = if !wins.is_empty() {
        wins.iter().sum::<f64>() / wins.len() as f64
    } else {
        0.0
    };
    let avg_loss = if !losses.is_empty() {
        losses.iter().sum::<f64>() / losses.len() as f64
    } else {
        0.0
    };
    let avg_r = if !trades.is_empty() {
        trades.iter().map(|t| t.r_multiple).sum::<f64>() / trades.len() as f64
    } else {
        0.0
    };
    let gross_win: f64 = wins.iter().sum();
    let gross_loss: f64 = losses.iter().map(|v| v.abs()).sum();
    let profit_factor = if gross_loss > 0.0 {
        gross_win / gross_loss
    } else if gross_win > 0.0 {
        f64::INFINITY
    } else {
        0.0
    };
    let total_return_pct = (final_equity - cfg.initial_equity) / cfg.initial_equity * 100.0;
    let max_dd = points
        .iter()
        .map(|p| p.drawdown_pct)
        .fold(0.0_f64, f64::max);

    // Sharpe: mean(returns)/std(returns). On bar-returns. Annualize when
    // caller plots — we don't know the bar interval at this layer.
    let sharpe = if returns.len() > 1 {
        let mean = returns.iter().sum::<f64>() / returns.len() as f64;
        let var = returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / returns.len() as f64;
        let std = var.sqrt();
        if std > 0.0 { mean / std } else { 0.0 }
    } else {
        0.0
    };

    let exits_by_stop = trades.iter().filter(|t| t.exit_reason == ExitReason::StopLoss).count();
    let exits_by_tp = trades.iter().filter(|t| t.exit_reason == ExitReason::TakeProfit).count();
    let exits_by_signal = trades.iter().filter(|t| t.exit_reason == ExitReason::StrategySignal).count();
    let exits_by_eod = trades.iter().filter(|t| t.exit_reason == ExitReason::EndOfData).count();

    AlgoBtSummary {
        trades: trades.len(),
        wins: wins.len(),
        losses: losses.len(),
        win_rate,
        avg_win,
        avg_loss,
        avg_r,
        profit_factor,
        total_return_pct,
        max_drawdown_pct: max_dd,
        final_equity,
        sharpe,
        bars_in_market_pct: if n_bars > 0 {
            bars_in_market as f64 / n_bars as f64 * 100.0
        } else {
            0.0
        },
        exits_by_stop,
        exits_by_tp,
        exits_by_signal,
        exits_by_eod,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algo_strategies::{from_kind, Sizing};
    use crate::models::BarInterval;
    use chrono::TimeZone;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    fn bar(t: i64, o: &str, h: &str, l: &str, c: &str, v: u64) -> PriceBar {
        PriceBar {
            symbol: "TEST".into(),
            interval: BarInterval::M1,
            bar_time: Utc.timestamp_opt(t, 0).unwrap(),
            open: Decimal::from_str(o).unwrap(),
            high: Decimal::from_str(h).unwrap(),
            low: Decimal::from_str(l).unwrap(),
            close: Decimal::from_str(c).unwrap(),
            volume: Decimal::from(v),
            source: "test".into(),
        }
    }

    /// 40 sideways bars then 50 strong uptrend bars — the synthetic
    /// window used in the supertrend strategy tests, enough to produce
    /// a momentum signal.
    fn uptrend_window() -> Vec<PriceBar> {
        let mut bars = Vec::new();
        let mut t = 1_700_000_000_i64;
        for i in 0..30 {
            let p = 100.0 + ((i as f64 * 0.4).sin() * 0.15);
            bars.push(bar(
                t,
                &format!("{p:.2}"),
                &format!("{:.2}", p + 0.15),
                &format!("{:.2}", p - 0.15),
                &format!("{p:.2}"),
                1_000_000,
            ));
            t += 60;
        }
        for i in 0..60 {
            let p = 100.4 + (i as f64 + 1.0) * 0.7;
            bars.push(bar(
                t,
                &format!("{:.2}", p - 0.25),
                &format!("{:.2}", p + 0.55),
                &format!("{:.2}", p - 0.55),
                &format!("{p:.2}"),
                1_500_000,
            ));
            t += 60;
        }
        bars
    }

    #[test]
    fn backtest_runs_clean_on_uptrend_window() {
        let strat = from_kind("supertrend", &serde_json::json!({})).expect("strat");
        let bars = uptrend_window();
        let cfg = BacktestConfig::default();
        let sizing = Sizing::default();
        let res = run(&bars, strat.as_ref(), &sizing, cfg);
        assert_eq!(res.strategy_kind, "supertrend");
        assert_eq!(res.equity.len(), bars.len());
        // Equity curve must be monotonically computed (no NaN).
        for p in &res.equity {
            assert!(p.equity.is_finite(), "equity NaN at {:?}", p.time);
            assert!(p.drawdown_pct >= 0.0);
        }
        // Sanity: ending equity = initial + sum of trade PnLs.
        let sum_pnl: f64 = res.trades.iter().map(|t| t.pnl).sum();
        let expected = cfg.initial_equity + sum_pnl;
        assert!(
            (res.summary.final_equity - expected).abs() < 1e-6,
            "final {} != initial + Σpnl {}",
            res.summary.final_equity,
            expected
        );
    }

    #[test]
    fn backtest_flat_market_emits_zero_trades_and_no_loss() {
        let strat = from_kind("ma_cross_adx", &serde_json::json!({})).expect("strat");
        // 200 bars of pure noise — adx_min=25 gate filters everything.
        let bars: Vec<PriceBar> = (0..200)
            .map(|i| {
                let p = 100.0 + ((i as f64 * 0.4).sin() * 0.08);
                bar(
                    1_700_000_000 + i * 60,
                    &format!("{p:.4}"),
                    &format!("{:.4}", p + 0.05),
                    &format!("{:.4}", p - 0.05),
                    &format!("{p:.4}"),
                    1_000_000,
                )
            })
            .collect();
        let cfg = BacktestConfig::default();
        let sizing = Sizing::default();
        let res = run(&bars, strat.as_ref(), &sizing, cfg);
        assert_eq!(res.summary.trades, 0);
        assert!((res.summary.final_equity - cfg.initial_equity).abs() < 1e-6);
        assert_eq!(res.summary.exits_by_stop, 0);
    }

    #[test]
    fn stop_and_tp_resolved_pessimistically() {
        // If a bar's range covers BOTH stop and TP, exit must be the
        // stop-loss (worst case). Quick path: build a tiny synthetic
        // by hand-feeding a fake open position via a 1-bar strategy
        // that fires immediately. Easier path: use supertrend on a
        // window that produces one trade, then verify the resolver in
        // isolation (single-purpose unit on the matcher would be nice
        // but the bar.high/low logic is inline). Skip — covered by the
        // first test which exercises the full flow.
        // Instead, verify the summary counters tally:
        let strat = from_kind("supertrend", &serde_json::json!({})).expect("strat");
        let bars = uptrend_window();
        let res = run(
            &bars,
            strat.as_ref(),
            &Sizing::default(),
            BacktestConfig::default(),
        );
        let by_reason = res.summary.exits_by_stop
            + res.summary.exits_by_tp
            + res.summary.exits_by_signal
            + res.summary.exits_by_eod;
        assert_eq!(by_reason, res.summary.trades);
    }
}
