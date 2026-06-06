//! Backtest parameter sweep — grid-search a preset over multiple
//! parameter combinations and emit each combination's headline stats.
//!
//! Lets the trader answer "what was the best SMA fast/slow combo on
//! AAPL over the last 5 years?" without writing a one-off script per
//! question.
//!
//! The orchestrator runs the existing `crate::backtest::run` for every
//! combo and returns sorted results. Currently supports SmaCross and
//! BollingerBreakout grids (the two presets with two-axis parameters);
//! RsiReversion and MacdCross can be added the same way later.

use crate::backtest::{self, BtResult, Preset};
use crate::models::PriceBar;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmaCrossGrid {
    pub fasts: Vec<usize>,
    pub slows: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BbBreakoutGrid {
    pub periods: Vec<usize>,
    pub ks: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SweepRow {
    pub label: String,
    pub trades: usize,
    pub win_rate: f64,
    pub profit_factor: f64,
    pub total_return_pct: f64,
    pub max_drawdown_pct: f64,
    pub sharpe_daily: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SweepReport {
    pub rows: Vec<SweepRow>,
    /// Index into `rows` of the highest-return combo. None when no combo ran.
    pub best_by_return: Option<usize>,
    /// Same for highest sharpe (the more risk-adjusted "best").
    pub best_by_sharpe: Option<usize>,
}

pub fn sweep_sma_cross(
    bars: &[PriceBar],
    grid: &SmaCrossGrid,
    initial_capital: f64,
    fee_per_trade: f64,
) -> SweepReport {
    let mut report = SweepReport::default();
    if bars.is_empty() {
        return report;
    }
    for &fast in &grid.fasts {
        for &slow in &grid.slows {
            // Skip nonsensical combos (fast >= slow, zero periods).
            if fast == 0 || slow == 0 || fast >= slow {
                continue;
            }
            let preset = Preset::SmaCross { fast, slow };
            let res = backtest::run(bars, preset, initial_capital, fee_per_trade);
            report
                .rows
                .push(row_from(&format!("sma {fast}/{slow}"), &res));
        }
    }
    finalize(&mut report);
    report
}

pub fn sweep_bb_breakout(
    bars: &[PriceBar],
    grid: &BbBreakoutGrid,
    initial_capital: f64,
    fee_per_trade: f64,
) -> SweepReport {
    let mut report = SweepReport::default();
    if bars.is_empty() {
        return report;
    }
    for &period in &grid.periods {
        for &k in &grid.ks {
            if period == 0 || !k.is_finite() || k <= 0.0 {
                continue;
            }
            let preset = Preset::BollingerBreakout { period, k };
            let res = backtest::run(bars, preset, initial_capital, fee_per_trade);
            report
                .rows
                .push(row_from(&format!("bb {period}/{k:.1}"), &res));
        }
    }
    finalize(&mut report);
    report
}

fn row_from(label: &str, res: &BtResult) -> SweepRow {
    SweepRow {
        label: label.into(),
        trades: res.summary.trades,
        win_rate: res.summary.win_rate,
        profit_factor: res.summary.profit_factor,
        total_return_pct: res.summary.total_return_pct,
        max_drawdown_pct: res.summary.max_drawdown_pct,
        sharpe_daily: res.summary.sharpe_daily,
    }
}

fn finalize(report: &mut SweepReport) {
    if report.rows.is_empty() {
        return;
    }
    // Best by return.
    let mut best_i = 0;
    for i in 1..report.rows.len() {
        if report.rows[i].total_return_pct > report.rows[best_i].total_return_pct {
            best_i = i;
        }
    }
    report.best_by_return = Some(best_i);
    // Best by sharpe (skipping NaN — should already be filtered by the
    // backtester's `stdev > 0` guard but defensive).
    let mut best_s = 0;
    for i in 1..report.rows.len() {
        let s_i = report.rows[i].sharpe_daily;
        let s_b = report.rows[best_s].sharpe_daily;
        if s_i.is_finite() && (!s_b.is_finite() || s_i > s_b) {
            best_s = i;
        }
    }
    report.best_by_sharpe = Some(best_s);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::BarInterval;
    use chrono::{TimeZone, Utc};
    use rust_decimal::Decimal;

    fn bar(open: f64, high: f64, low: f64, close: f64, ts: i64) -> PriceBar {
        PriceBar {
            symbol: "X".into(),
            interval: BarInterval::D1,
            bar_time: Utc.timestamp_opt(ts, 0).unwrap(),
            open: Decimal::try_from(open).unwrap(),
            high: Decimal::try_from(high).unwrap(),
            low: Decimal::try_from(low).unwrap(),
            close: Decimal::try_from(close).unwrap(),
            volume: Decimal::from(1_000_000),
            source: "test".into(),
        }
    }

    fn build_trending_bars(n: usize) -> Vec<PriceBar> {
        (1..=n)
            .map(|i| {
                let c = 100.0 + i as f64 * 0.1;
                bar(c - 0.2, c + 0.2, c - 0.4, c, i as i64)
            })
            .collect()
    }

    #[test]
    fn empty_input_returns_empty_report() {
        let g = SmaCrossGrid {
            fasts: vec![5, 10],
            slows: vec![20, 30],
        };
        let r = sweep_sma_cross(&[], &g, 10_000.0, 1.0);
        assert!(r.rows.is_empty());
        assert!(r.best_by_return.is_none());
    }

    #[test]
    fn invalid_combos_filtered() {
        let bars = build_trending_bars(60);
        let g = SmaCrossGrid {
            fasts: vec![0, 50, 5], // 0 invalid, 50 >= 50 invalid
            slows: vec![50],
        };
        let r = sweep_sma_cross(&bars, &g, 10_000.0, 1.0);
        assert_eq!(r.rows.len(), 1, "only fast=5/slow=50 should survive");
    }

    #[test]
    fn sweep_emits_one_row_per_valid_combo() {
        let bars = build_trending_bars(120);
        let g = SmaCrossGrid {
            fasts: vec![5, 10],
            slows: vec![20, 30],
        };
        let r = sweep_sma_cross(&bars, &g, 10_000.0, 1.0);
        // 2 × 2 = 4 valid combos.
        assert_eq!(r.rows.len(), 4);
        assert!(r.best_by_return.is_some());
        assert!(r.best_by_sharpe.is_some());
    }

    #[test]
    fn best_by_return_index_points_to_highest_return_row() {
        let bars = build_trending_bars(120);
        let g = SmaCrossGrid {
            fasts: vec![5, 10],
            slows: vec![20, 30],
        };
        let r = sweep_sma_cross(&bars, &g, 10_000.0, 1.0);
        let best = r.best_by_return.unwrap();
        let max_ret = r
            .rows
            .iter()
            .map(|x| x.total_return_pct)
            .fold(f64::NEG_INFINITY, f64::max);
        assert!((r.rows[best].total_return_pct - max_ret).abs() < 1e-9);
    }

    #[test]
    fn bb_breakout_grid_runs_each_combo() {
        let bars = build_trending_bars(120);
        let g = BbBreakoutGrid {
            periods: vec![10, 20],
            ks: vec![1.5, 2.0, 2.5],
        };
        let r = sweep_bb_breakout(&bars, &g, 10_000.0, 1.0);
        assert_eq!(r.rows.len(), 6);
    }

    #[test]
    fn invalid_k_filtered() {
        let bars = build_trending_bars(120);
        let g = BbBreakoutGrid {
            periods: vec![20],
            ks: vec![0.0, -1.0, f64::NAN, 2.0],
        };
        let r = sweep_bb_breakout(&bars, &g, 10_000.0, 1.0);
        assert_eq!(r.rows.len(), 1, "only k=2.0 should survive");
    }
}
