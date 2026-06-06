//! Walk-forward backtest validator.
//!
//! Splits the bar history into rolling (in-sample, out-of-sample) windows.
//! Runs the parameter sweep on the in-sample slice, picks the winning combo,
//! and reports how the SAME combo performs on the next out-of-sample slice.
//! Repeats forward through the series.
//!
//! Catches the most common backtest delusion: a parameter set that's
//! great in-sample but tracks like static out-of-sample (overfitting).
//! If out-of-sample returns drop to near-random levels, the strategy
//! has no edge.
//!
//! Pure compute. Currently SMA-cross only — same shape as the
//! `backtest_sweep` orchestrator.

use crate::backtest::{self, Preset};
use crate::backtest_sweep::SmaCrossGrid;
use crate::models::PriceBar;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalkForwardConfig {
    pub in_sample_bars: usize,
    pub out_of_sample_bars: usize,
    /// How many bars to advance the window between iterations. Setting
    /// this < out_of_sample_bars creates overlapping windows. Setting it
    /// = out_of_sample_bars creates non-overlapping "rolling" windows.
    pub step_bars: usize,
    pub initial_capital: f64,
    pub fee_per_trade: f64,
}

impl Default for WalkForwardConfig {
    fn default() -> Self {
        Self {
            in_sample_bars: 252,
            out_of_sample_bars: 63,
            step_bars: 63,
            initial_capital: 10_000.0,
            fee_per_trade: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalkRow {
    pub window_index: usize,
    pub in_sample_label: String,
    pub in_sample_return_pct: f64,
    pub out_of_sample_return_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WalkForwardReport {
    pub rows: Vec<WalkRow>,
    pub mean_in_sample_return: f64,
    pub mean_out_of_sample_return: f64,
    /// Out-of-sample / in-sample ratio; > 1 = OOS BETTER than IS (rare, very good).
    /// < 0.5 = significant overfitting; near 0 = no edge.
    pub stability_ratio: f64,
}

pub fn run_sma_cross(
    bars: &[PriceBar],
    grid: &SmaCrossGrid,
    cfg: &WalkForwardConfig,
) -> WalkForwardReport {
    let mut report = WalkForwardReport::default();
    if bars.is_empty()
        || cfg.in_sample_bars == 0
        || cfg.out_of_sample_bars == 0
        || cfg.step_bars == 0
    {
        return report;
    }
    let total = bars.len();
    let window_size = cfg.in_sample_bars + cfg.out_of_sample_bars;
    if total < window_size {
        return report;
    }
    let mut window_idx = 0usize;
    let mut start = 0usize;
    while start + window_size <= total {
        let is_end = start + cfg.in_sample_bars;
        let oos_end = is_end + cfg.out_of_sample_bars;
        let is_slice = &bars[start..is_end];
        let oos_slice = &bars[is_end..oos_end];

        // Run the sweep on IS slice; pick highest-return combo.
        let mut best: Option<(usize, usize, f64)> = None;
        for &fast in &grid.fasts {
            for &slow in &grid.slows {
                if fast == 0 || slow == 0 || fast >= slow {
                    continue;
                }
                let preset = Preset::SmaCross { fast, slow };
                let res = backtest::run(is_slice, preset, cfg.initial_capital, cfg.fee_per_trade);
                let ret = res.summary.total_return_pct;
                if best.is_none() || ret > best.unwrap().2 {
                    best = Some((fast, slow, ret));
                }
            }
        }
        if let Some((fast, slow, is_ret)) = best {
            let oos = backtest::run(
                oos_slice,
                Preset::SmaCross { fast, slow },
                cfg.initial_capital,
                cfg.fee_per_trade,
            );
            report.rows.push(WalkRow {
                window_index: window_idx,
                in_sample_label: format!("sma {fast}/{slow}"),
                in_sample_return_pct: is_ret,
                out_of_sample_return_pct: oos.summary.total_return_pct,
            });
        }

        window_idx += 1;
        start = start.saturating_add(cfg.step_bars);
    }
    if !report.rows.is_empty() {
        let n = report.rows.len() as f64;
        report.mean_in_sample_return = report
            .rows
            .iter()
            .map(|r| r.in_sample_return_pct)
            .sum::<f64>()
            / n;
        report.mean_out_of_sample_return = report
            .rows
            .iter()
            .map(|r| r.out_of_sample_return_pct)
            .sum::<f64>()
            / n;
        report.stability_ratio = if report.mean_in_sample_return.abs() > 1e-9 {
            report.mean_out_of_sample_return / report.mean_in_sample_return
        } else {
            0.0
        };
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::BarInterval;
    use chrono::{TimeZone, Utc};
    use rust_decimal::Decimal;

    fn bar(price: f64, ts: i64) -> PriceBar {
        PriceBar {
            symbol: "X".into(),
            interval: BarInterval::D1,
            bar_time: Utc.timestamp_opt(ts, 0).unwrap(),
            open: Decimal::try_from(price - 0.1).unwrap(),
            high: Decimal::try_from(price + 0.2).unwrap(),
            low: Decimal::try_from(price - 0.2).unwrap(),
            close: Decimal::try_from(price).unwrap(),
            volume: Decimal::from(1_000_000),
            source: "test".into(),
        }
    }

    fn trending(n: usize) -> Vec<PriceBar> {
        (1..=n)
            .map(|i| bar(100.0 + i as f64 * 0.1, i as i64))
            .collect()
    }

    #[test]
    fn empty_returns_default() {
        let g = SmaCrossGrid {
            fasts: vec![5],
            slows: vec![20],
        };
        let r = run_sma_cross(&[], &g, &WalkForwardConfig::default());
        assert!(r.rows.is_empty());
    }

    #[test]
    fn invalid_window_returns_default() {
        let bars = trending(100);
        let g = SmaCrossGrid {
            fasts: vec![5],
            slows: vec![20],
        };
        let cfg = WalkForwardConfig {
            in_sample_bars: 0,
            out_of_sample_bars: 10,
            step_bars: 10,
            ..Default::default()
        };
        assert!(run_sma_cross(&bars, &g, &cfg).rows.is_empty());
    }

    #[test]
    fn series_shorter_than_window_returns_default() {
        let bars = trending(50);
        let g = SmaCrossGrid {
            fasts: vec![5],
            slows: vec![20],
        };
        let cfg = WalkForwardConfig {
            in_sample_bars: 100,
            out_of_sample_bars: 30,
            step_bars: 30,
            ..Default::default()
        };
        assert!(run_sma_cross(&bars, &g, &cfg).rows.is_empty());
    }

    #[test]
    fn produces_one_row_per_window() {
        let bars = trending(500);
        let g = SmaCrossGrid {
            fasts: vec![5, 10],
            slows: vec![20, 30],
        };
        let cfg = WalkForwardConfig {
            in_sample_bars: 100,
            out_of_sample_bars: 50,
            step_bars: 50,
            initial_capital: 10_000.0,
            fee_per_trade: 1.0,
        };
        let r = run_sma_cross(&bars, &g, &cfg);
        // Windows: start at 0, advance by 50 until start + 150 > 500.
        // 500 - 150 = 350 / 50 + 1 = 8 windows.
        assert!(
            r.rows.len() >= 6 && r.rows.len() <= 8,
            "expected ~7 windows, got {}",
            r.rows.len()
        );
    }

    #[test]
    fn stability_ratio_finite() {
        let bars = trending(500);
        let g = SmaCrossGrid {
            fasts: vec![5],
            slows: vec![20],
        };
        let cfg = WalkForwardConfig {
            in_sample_bars: 100,
            out_of_sample_bars: 50,
            step_bars: 50,
            initial_capital: 10_000.0,
            fee_per_trade: 1.0,
        };
        let r = run_sma_cross(&bars, &g, &cfg);
        assert!(r.stability_ratio.is_finite());
    }

    #[test]
    fn invalid_grid_combos_filtered_out_no_panic() {
        let bars = trending(500);
        let g = SmaCrossGrid {
            fasts: vec![0, 20, 5],
            slows: vec![20, 20], // fast=20 vs slow=20 → invalid
        };
        let cfg = WalkForwardConfig {
            in_sample_bars: 100,
            out_of_sample_bars: 50,
            step_bars: 50,
            initial_capital: 10_000.0,
            fee_per_trade: 1.0,
        };
        let r = run_sma_cross(&bars, &g, &cfg);
        // Only fast=5/slow=20 survives; no panic.
        assert!(!r.rows.is_empty());
    }
}
