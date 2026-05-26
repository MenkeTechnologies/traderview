//! Bar-by-bar strategy backtester.
//!
//! Each preset implements `Strategy::signal(idx, bars) -> Action`. The engine
//! iterates bars, opens/closes a single long position with a fixed capital
//! fraction, and emits trades + equity points + headline stats.
//!
//! Positions are always closed at next bar's open (no look-ahead), but for
//! simplicity we fill at the bar's close on the signal bar — note that this
//! is an optimistic assumption; users who care can subtract a slippage knob.

use crate::indicators::{closes, ema, highs, lows, rsi, sma};
use crate::models::PriceBar;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Preset {
    SmaCross    { fast: usize, slow: usize },
    RsiReversion { period: usize, oversold: f64, overbought: f64 },
    BollingerBreakout { period: usize, k: f64 },
    MacdCross,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Action { None, Buy, Sell }

#[derive(Debug, Clone, Serialize)]
pub struct BtTrade {
    pub entry_time: DateTime<Utc>,
    pub exit_time:  DateTime<Utc>,
    pub entry_price: f64,
    pub exit_price:  f64,
    pub qty: f64,
    pub pnl: f64,
    pub pnl_pct: f64,
    pub bars_held: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct BtPoint {
    pub time: DateTime<Utc>,
    pub equity: f64,
    pub drawdown_pct: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct BtSummary {
    pub trades: usize,
    pub wins: usize,
    pub losses: usize,
    pub win_rate: f64,
    pub avg_win: f64,
    pub avg_loss: f64,
    pub profit_factor: f64,
    pub total_return_pct: f64,
    pub max_drawdown_pct: f64,
    pub final_equity: f64,
    pub sharpe_daily: f64,
    pub bars_in_market_pct: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct BtResult {
    pub preset: Preset,
    pub trades: Vec<BtTrade>,
    pub equity: Vec<BtPoint>,
    pub summary: BtSummary,
}

pub fn run(bars: &[PriceBar], preset: Preset, initial_capital: f64, fee_per_trade: f64) -> BtResult {
    let n = bars.len();
    let c = closes(bars);
    let h = highs(bars);
    let l = lows(bars);
    let signals: Vec<Action> = match preset {
        Preset::SmaCross { fast, slow } => sma_cross(&c, fast, slow),
        Preset::RsiReversion { period, oversold, overbought } => rsi_rev(&c, period, oversold, overbought),
        Preset::BollingerBreakout { period, k } => bb_breakout(&c, period, k),
        Preset::MacdCross => macd_cross(&c),
    };
    let _ = (h, l);

    let mut cash = initial_capital;
    let mut position: Option<(usize, f64)> = None; // (entry idx, qty)
    let mut trades = Vec::new();
    let mut equity = Vec::with_capacity(n);
    let mut peak = initial_capital;
    let mut max_dd = 0.0_f64;
    let mut bars_in = 0usize;
    let mut daily_returns: Vec<f64> = Vec::new();
    let mut last_equity = initial_capital;

    for i in 0..n {
        let price = c[i];
        // Apply signal at this bar's close.
        match (signals[i], &position) {
            (Action::Buy, None) => {
                let qty = (cash * 0.95) / price; // 95% allocation, simple
                cash -= qty * price;
                cash -= fee_per_trade;
                position = Some((i, qty));
            }
            (Action::Sell, Some((entry_idx, qty))) => {
                let entry_idx = *entry_idx; let qty = *qty;
                cash += qty * price;
                cash -= fee_per_trade;
                let entry = c[entry_idx];
                let pnl = (price - entry) * qty - 2.0 * fee_per_trade;
                let pnl_pct = (price - entry) / entry * 100.0;
                trades.push(BtTrade {
                    entry_time: bars[entry_idx].bar_time,
                    exit_time:  bars[i].bar_time,
                    entry_price: entry,
                    exit_price:  price,
                    qty, pnl, pnl_pct,
                    bars_held: i - entry_idx,
                });
                position = None;
            }
            _ => {}
        }
        if position.is_some() { bars_in += 1; }

        let mark = cash + position.map(|(_, q)| q * price).unwrap_or(0.0);
        if mark > peak { peak = mark; }
        let dd = (mark - peak) / peak * 100.0;
        if dd < max_dd { max_dd = dd; }

        let day_ret = if last_equity > 0.0 { (mark - last_equity) / last_equity } else { 0.0 };
        daily_returns.push(day_ret);
        last_equity = mark;

        equity.push(BtPoint {
            time: bars[i].bar_time, equity: mark, drawdown_pct: dd,
        });
    }
    // Close any open position at last close.
    if let Some((entry_idx, qty)) = position {
        let price = c[n - 1];
        cash += qty * price;
        cash -= fee_per_trade;
        let entry = c[entry_idx];
        let pnl = (price - entry) * qty - 2.0 * fee_per_trade;
        let pnl_pct = (price - entry) / entry * 100.0;
        trades.push(BtTrade {
            entry_time: bars[entry_idx].bar_time,
            exit_time:  bars[n - 1].bar_time,
            entry_price: entry, exit_price: price, qty, pnl, pnl_pct,
            bars_held: n - 1 - entry_idx,
        });
    }

    // Stats.
    let wins = trades.iter().filter(|t| t.pnl > 0.0).count();
    let losses = trades.iter().filter(|t| t.pnl < 0.0).count();
    let total_win: f64 = trades.iter().filter(|t| t.pnl > 0.0).map(|t| t.pnl).sum();
    let total_loss: f64 = trades.iter().filter(|t| t.pnl < 0.0).map(|t| t.pnl).sum();
    let pf = if total_loss.abs() > 0.0 { total_win / total_loss.abs() } else { 0.0 };
    let final_eq = equity.last().map(|p| p.equity).unwrap_or(initial_capital);
    let total_return = (final_eq - initial_capital) / initial_capital * 100.0;
    let mean = daily_returns.iter().sum::<f64>() / daily_returns.len().max(1) as f64;
    let var = daily_returns.iter().map(|x| (x - mean).powi(2)).sum::<f64>()
        / daily_returns.len().max(1) as f64;
    let stdev = var.sqrt();
    let sharpe = if stdev > 0.0 { mean / stdev } else { 0.0 };

    BtResult {
        preset, trades: trades.clone(), equity,
        summary: BtSummary {
            trades: trades.len(),
            wins, losses,
            win_rate: if trades.is_empty() { 0.0 } else { wins as f64 / trades.len() as f64 },
            avg_win: if wins > 0 { total_win / wins as f64 } else { 0.0 },
            avg_loss: if losses > 0 { total_loss / losses as f64 } else { 0.0 },
            profit_factor: pf,
            total_return_pct: total_return,
            max_drawdown_pct: max_dd,
            final_equity: final_eq,
            sharpe_daily: sharpe,
            bars_in_market_pct: if n > 0 { bars_in as f64 / n as f64 * 100.0 } else { 0.0 },
        },
    }
}

// ===========================================================================
// Preset signal generators
// ===========================================================================

fn sma_cross(c: &[f64], fast: usize, slow: usize) -> Vec<Action> {
    let f = sma(c, fast);
    let s = sma(c, slow);
    let mut out = vec![Action::None; c.len()];
    for i in 1..c.len() {
        if let (Some(pf), Some(ps), Some(cf), Some(cs)) = (f[i-1], s[i-1], f[i], s[i]) {
            if pf <= ps && cf > cs { out[i] = Action::Buy; }
            else if pf >= ps && cf < cs { out[i] = Action::Sell; }
        }
    }
    out
}

fn rsi_rev(c: &[f64], period: usize, oversold: f64, overbought: f64) -> Vec<Action> {
    let r = rsi(c, period);
    let mut out = vec![Action::None; c.len()];
    for i in 0..c.len() {
        if let Some(v) = r[i] {
            if v <= oversold { out[i] = Action::Buy; }
            else if v >= overbought { out[i] = Action::Sell; }
        }
    }
    out
}

fn bb_breakout(c: &[f64], period: usize, k: f64) -> Vec<Action> {
    let bb = crate::indicators::bollinger(c, period, k);
    let mut out = vec![Action::None; c.len()];
    for i in 0..c.len() {
        if let (Some(u), Some(l)) = (bb.upper[i], bb.lower[i]) {
            if c[i] > u { out[i] = Action::Buy; }
            else if c[i] < l { out[i] = Action::Sell; }
        }
    }
    out
}

fn macd_cross(c: &[f64]) -> Vec<Action> {
    let e_fast = ema(c, 12);
    let e_slow = ema(c, 26);
    let mut line = vec![None; c.len()];
    for i in 0..c.len() {
        if let (Some(a), Some(b)) = (e_fast[i], e_slow[i]) {
            line[i] = Some(a - b);
        }
    }
    let line_compact: Vec<f64> = line.iter().filter_map(|x| *x).collect();
    let sig = ema(&line_compact, 9);
    let offset = c.len() - line_compact.len();
    let mut sig_full = vec![None; c.len()];
    for (i, v) in sig.iter().enumerate() { sig_full[offset + i] = *v; }
    let mut out = vec![Action::None; c.len()];
    for i in 1..c.len() {
        if let (Some(pl), Some(ps), Some(cl), Some(cs)) =
            (line[i-1], sig_full[i-1], line[i], sig_full[i])
        {
            if pl <= ps && cl > cs { out[i] = Action::Buy; }
            else if pl >= ps && cl < cs { out[i] = Action::Sell; }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::BarInterval;
    use chrono::{TimeZone, Utc};
    use rust_decimal::Decimal;

    fn bar(close: f64, ts: i64) -> PriceBar {
        PriceBar {
            symbol: "X".into(), interval: BarInterval::D1,
            bar_time: Utc.timestamp_opt(ts, 0).unwrap(),
            open: Decimal::try_from(close).unwrap(),
            high: Decimal::try_from(close).unwrap(),
            low:  Decimal::try_from(close).unwrap(),
            close: Decimal::try_from(close).unwrap(),
            volume: Decimal::from(1_000_000),
            source: "test".into(),
        }
    }

    #[test]
    fn sma_cross_runs() {
        // Sine wave guarantees multiple SMA(5) / SMA(20) crossovers.
        let mut bars = Vec::new();
        for i in 0..200 {
            let p = 100.0 + 20.0 * (i as f64 / 12.0).sin();
            bars.push(bar(p, i));
        }
        let r = run(&bars, Preset::SmaCross { fast: 5, slow: 20 }, 10_000.0, 0.0);
        assert!(r.summary.trades >= 1, "no trades fired");
        assert_eq!(r.equity.len(), bars.len());
    }
}
