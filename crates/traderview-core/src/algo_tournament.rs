//! Strategy tournament — run EVERY single-symbol registry strategy
//! (default rules) over the same bars and rank them.
//!
//! Purpose: the registry holds 20+ strategies; the tournament answers
//! "which of them actually earns on THIS symbol and period" in one
//! call instead of twenty manual backtests. Multi-symbol strategies
//! (pairs) are skipped and reported as skipped — silence would read as
//! "covered".
//!
//! Ranking: strategies that never traded sink below ANY strategy with
//! trades regardless of metric — a strategy that never fired has no
//! measured edge, just an unblemished record.

use crate::algo_backtest::{run, AlgoBtSummary, BacktestConfig};
use crate::algo_strategies::{from_kind, Sizing, StrategyKind};
use crate::models::PriceBar;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RankMetric {
    #[default]
    Sharpe,
    TotalReturn,
    ProfitFactor,
}

#[derive(Debug, Clone, Serialize)]
pub struct TournamentRow {
    pub kind: &'static str,
    pub summary: AlgoBtSummary,
}

#[derive(Debug, Clone, Serialize)]
pub struct TournamentResult {
    /// Ranked best-first by the requested metric.
    pub rows: Vec<TournamentRow>,
    /// Strategies not run (multi-symbol) — reported, never silent.
    pub skipped: Vec<&'static str>,
    pub rank_by: RankMetric,
    /// Buy-and-hold over the same bars — the bar every row must clear.
    /// A 20% strategy on a 30% tape is negative alpha, not a win.
    pub benchmark: crate::algo_strategy_portfolio::CurveStats,
}

fn metric_value(s: &AlgoBtSummary, m: RankMetric) -> f64 {
    let v = match m {
        RankMetric::Sharpe => s.sharpe,
        RankMetric::TotalReturn => s.total_return_pct,
        RankMetric::ProfitFactor => s.profit_factor,
    };
    if v.is_finite() {
        v
    } else {
        f64::NEG_INFINITY
    }
}

/// Ranking: traded strategies first (by metric desc), zero-trade
/// strategies last. Pure — pinned independently of the backtester.
pub fn sort_rows(rows: &mut [TournamentRow], metric: RankMetric) {
    rows.sort_by(|a, b| {
        let a_traded = a.summary.trades > 0;
        let b_traded = b.summary.trades > 0;
        b_traded.cmp(&a_traded).then(
            metric_value(&b.summary, metric).total_cmp(&metric_value(&a.summary, metric)),
        )
    });
}

/// Run the tournament over one symbol's bars.
pub fn tournament(
    bars: &[PriceBar],
    sizing: &Sizing,
    cfg: &BacktestConfig,
    rank_by: RankMetric,
) -> TournamentResult {
    let mut rows = Vec::new();
    let mut skipped = Vec::new();
    for kind in StrategyKind::all() {
        let slug = kind.as_str();
        let Ok(strat) = from_kind(slug, &serde_json::json!({})) else {
            skipped.push(slug);
            continue;
        };
        if strat.required_symbols().is_some() {
            skipped.push(slug);
            continue;
        }
        let result = run(bars, strat.as_ref(), sizing, cfg.clone());
        rows.push(TournamentRow {
            kind: slug,
            summary: result.summary,
        });
    }
    sort_rows(&mut rows, rank_by);
    TournamentResult {
        rows,
        skipped,
        rank_by,
        benchmark: buy_and_hold(bars),
    }
}

/// The registry's single-symbol kinds, in stable registry order —
/// the row axis of the matrix.
pub fn single_symbol_kinds() -> Vec<&'static str> {
    StrategyKind::all()
        .iter()
        .filter_map(|k| {
            let slug = k.as_str();
            let strat = from_kind(slug, &serde_json::json!({})).ok()?;
            strat.required_symbols().is_none().then_some(slug)
        })
        .collect()
}

#[derive(Debug, Clone, Serialize)]
pub struct SymbolColumn {
    pub symbol: String,
    /// Metric score per kind, aligned with `MatrixResult::kinds`;
    /// None = the strategy never traded this symbol.
    pub scores: Vec<Option<f64>>,
    pub best_kind: Option<&'static str>,
    pub benchmark: crate::algo_strategy_portfolio::CurveStats,
}

#[derive(Debug, Clone, Serialize)]
pub struct MatrixResult {
    pub kinds: Vec<&'static str>,
    pub columns: Vec<SymbolColumn>,
    pub rank_by: RankMetric,
}

/// Strategies × symbols: every single-symbol kind backtested (default
/// rules) on every symbol's bars. Column order follows the input;
/// row order is the stable registry order, NOT per-column ranking, so
/// rows align across columns.
pub fn matrix(
    per_symbol: &[(String, Vec<PriceBar>)],
    sizing: &Sizing,
    cfg: &BacktestConfig,
    rank_by: RankMetric,
) -> MatrixResult {
    let kinds = single_symbol_kinds();
    let columns = per_symbol
        .iter()
        .map(|(symbol, bars)| {
            let mut scores = Vec::with_capacity(kinds.len());
            let mut best: Option<(&'static str, f64)> = None;
            for slug in &kinds {
                let score = from_kind(slug, &serde_json::json!({}))
                    .ok()
                    .map(|strat| run(bars, strat.as_ref(), sizing, cfg.clone()))
                    .filter(|bt| bt.summary.trades > 0)
                    .map(|bt| metric_value(&bt.summary, rank_by));
                if let Some(v) = score {
                    if v.is_finite() && best.map_or(true, |(_, b)| v > b) {
                        best = Some((slug, v));
                    }
                }
                scores.push(score);
            }
            SymbolColumn {
                symbol: symbol.clone(),
                scores,
                best_kind: best.map(|(k, _)| k),
                benchmark: buy_and_hold(bars),
            }
        })
        .collect();
    MatrixResult {
        kinds,
        columns,
        rank_by,
    }
}

/// Buy-and-hold stats over the bars' closes — the passive baseline.
pub fn buy_and_hold(bars: &[PriceBar]) -> crate::algo_strategy_portfolio::CurveStats {
    use rust_decimal::prelude::ToPrimitive;
    let closes: Vec<f64> = bars
        .iter()
        .map(|b| b.close.to_f64().unwrap_or(0.0))
        .collect();
    crate::algo_strategy_portfolio::curve_stats(&crate::algo_strategy_portfolio::bar_returns(
        &closes,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::BarInterval;
    use chrono::{TimeZone, Utc};
    use rust_decimal::Decimal;
    use std::str::FromStr;

    fn summary(trades: usize, sharpe: f64, ret: f64) -> AlgoBtSummary {
        AlgoBtSummary {
            trades,
            wins: 0,
            losses: 0,
            win_rate: 0.0,
            avg_win: 0.0,
            avg_loss: 0.0,
            avg_r: 0.0,
            profit_factor: 0.0,
            total_return_pct: ret,
            max_drawdown_pct: 0.0,
            final_equity: 0.0,
            sharpe,
            bars_in_market_pct: 0.0,
            exits_by_stop: 0,
            exits_by_tp: 0,
            exits_by_signal: 0,
            exits_by_time_stop: 0,
            exits_by_eod: 0,
        }
    }

    #[test]
    fn zero_trade_strategies_rank_below_any_traded_one() {
        // A never-fired strategy with a pristine 0.0 Sharpe must NOT
        // outrank a traded strategy with a negative Sharpe.
        let mut rows = vec![
            TournamentRow { kind: "idle", summary: summary(0, 0.0, 0.0) },
            TournamentRow { kind: "loser", summary: summary(12, -0.4, -3.0) },
            TournamentRow { kind: "winner", summary: summary(9, 1.2, 18.0) },
        ];
        sort_rows(&mut rows, RankMetric::Sharpe);
        assert_eq!(
            rows.iter().map(|r| r.kind).collect::<Vec<_>>(),
            vec!["winner", "loser", "idle"]
        );
    }

    #[test]
    fn metric_switch_reorders_traded_rows() {
        // High Sharpe / low return vs low Sharpe / high return.
        let mut rows = vec![
            TournamentRow { kind: "steady", summary: summary(20, 2.0, 8.0) },
            TournamentRow { kind: "swinger", summary: summary(20, 0.5, 40.0) },
        ];
        sort_rows(&mut rows, RankMetric::Sharpe);
        assert_eq!(rows[0].kind, "steady");
        sort_rows(&mut rows, RankMetric::TotalReturn);
        assert_eq!(rows[0].kind, "swinger");
    }

    #[test]
    fn non_finite_metrics_sink_within_traded_group() {
        // A NaN profit factor (no losses yet) must not poison the sort.
        let mut rows = vec![
            TournamentRow { kind: "nan", summary: summary(3, f64::NAN, 1.0) },
            TournamentRow { kind: "real", summary: summary(3, 0.3, 1.0) },
        ];
        sort_rows(&mut rows, RankMetric::Sharpe);
        assert_eq!(rows[0].kind, "real");
    }

    fn bar(t: i64, p: f64) -> PriceBar {
        PriceBar {
            symbol: "TEST".into(),
            interval: BarInterval::D1,
            bar_time: Utc.timestamp_opt(t, 0).unwrap(),
            open: Decimal::from_str(&format!("{p:.4}")).unwrap(),
            high: Decimal::from_str(&format!("{:.4}", p + 0.5)).unwrap(),
            low: Decimal::from_str(&format!("{:.4}", p - 0.5)).unwrap(),
            close: Decimal::from_str(&format!("{p:.4}")).unwrap(),
            volume: Decimal::from(1_000_000u64),
            source: "test".into(),
        }
    }

    #[test]
    fn matrix_rows_align_across_columns_and_best_is_argmax() {
        let mk = |seed: f64| -> Vec<PriceBar> {
            (0..120)
                .map(|i| {
                    bar(
                        1_700_000_000 + i * 86_400,
                        100.0 + ((i as f64 * 0.3) + seed).sin() * 6.0 + i as f64 * 0.05,
                    )
                })
                .collect()
        };
        let per_symbol = vec![("AAA".to_string(), mk(0.0)), ("BBB".to_string(), mk(1.5))];
        let cfg = BacktestConfig {
            initial_equity: 100_000.0,
            fee_per_trade: 1.0,
            slippage_bps: 5.0,
            side_mode: crate::algo_strategies::SideMode::Both,
        };
        let m = matrix(&per_symbol, &Sizing::default(), &cfg, RankMetric::Sharpe);
        // Row axis is the registry's single-symbol kinds; every column
        // aligns with it exactly.
        assert_eq!(m.kinds, single_symbol_kinds());
        assert!(!m.kinds.contains(&"pairs"));
        for col in &m.columns {
            assert_eq!(col.scores.len(), m.kinds.len());
            // best_kind is the argmax over the column's Some scores.
            if let Some(best) = col.best_kind {
                let bi = m.kinds.iter().position(|k| *k == best).unwrap();
                let bv = col.scores[bi].unwrap();
                assert!(col
                    .scores
                    .iter()
                    .flatten()
                    .filter(|v| v.is_finite())
                    .all(|v| *v <= bv));
            } else {
                assert!(col.scores.iter().all(|s| s.is_none()));
            }
        }
    }

    #[test]
    fn benchmark_pins_buy_and_hold_math() {
        // 100 → 110 → 99: total −1%, max drawdown (110−99)/110 = 10%.
        let bars = vec![
            bar(1_700_000_000, 100.0),
            bar(1_700_086_400, 110.0),
            bar(1_700_172_800, 99.0),
        ];
        let b = buy_and_hold(&bars);
        assert!((b.total_return_pct + 1.0).abs() < 1e-9);
        assert!((b.max_drawdown_pct - 10.0).abs() < 1e-9);
    }

    #[test]
    fn tournament_covers_the_whole_registry() {
        // Every registry kind is either ranked or explicitly skipped —
        // the count is derived from StrategyKind::all(), never typed.
        let bars: Vec<PriceBar> = (0..120)
            .map(|i| bar(1_700_000_000 + i * 86_400, 100.0 + (i as f64 * 0.3).sin() * 5.0))
            .collect();
        let cfg = BacktestConfig {
            initial_equity: 100_000.0,
            fee_per_trade: 1.0,
            slippage_bps: 5.0,
            side_mode: crate::algo_strategies::SideMode::Both,
        };
        let r = tournament(&bars, &Sizing::default(), &cfg, RankMetric::Sharpe);
        assert_eq!(
            r.rows.len() + r.skipped.len(),
            StrategyKind::all().len(),
            "every kind must be accounted for"
        );
        // Pairs is multi-symbol and must be in the skipped list.
        assert!(r.skipped.contains(&"pairs"));
        // No traded row ranks below a zero-trade row.
        let first_idle = r.rows.iter().position(|x| x.summary.trades == 0);
        if let Some(idx) = first_idle {
            assert!(r.rows[idx..].iter().all(|x| x.summary.trades == 0));
        }
    }
}
