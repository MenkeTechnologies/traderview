//! Walk-forward validation for algo strategies — the overfit detector
//! the grid optimizer needs.
//!
//! Rolling windows: optimize the grid on the IN-SAMPLE slice, take the
//! best entry_rules, then backtest them UNSEEN on the OUT-OF-SAMPLE
//! slice that follows. Roll forward and repeat. The summary statistic
//! is walk-forward efficiency = avg OOS score / avg IS score — a
//! config that only earns in-sample collapses here (the classic
//! threshold reading: WFE below ~0.5 means the optimizer curve-fit).

use crate::algo_backtest::{self, AlgoBtSummary, BacktestConfig};
use crate::algo_optimize::{self, OptimizeError, OptimizeMetric};
use crate::algo_strategies::{from_kind, Sizing};
use crate::models::PriceBar;
use serde::Serialize;

/// Window-count backstop: combinations × windows is the real cost.
pub const MAX_WINDOWS: usize = 50;

#[derive(Debug, Clone, Serialize)]
pub struct WfWindow {
    pub window: usize,
    /// Bar-index ranges [start, end) into the input series.
    pub is_range: (usize, usize),
    pub oos_range: (usize, usize),
    /// Winning override set from the in-sample optimization.
    pub best_overrides: serde_json::Value,
    pub is_score: f64,
    pub oos_score: f64,
    pub oos_summary: AlgoBtSummary,
}

#[derive(Debug, Clone, Serialize)]
pub struct AlgoWfResult {
    pub strategy_kind: String,
    pub metric: OptimizeMetric,
    pub windows: Vec<WfWindow>,
    pub avg_is_score: f64,
    pub avg_oos_score: f64,
    /// avg OOS / avg IS. None when avg IS isn't a positive finite
    /// number — a ratio against a zero or negative base is noise.
    pub wf_efficiency: Option<f64>,
    pub oos_total_trades: usize,
}

/// Window layout: (start, is_end, oos_end) triples — pure, pinned.
pub fn layout(n: usize, is_bars: usize, oos_bars: usize, step: usize) -> Vec<(usize, usize, usize)> {
    let mut out = Vec::new();
    if is_bars == 0 || oos_bars == 0 || step == 0 {
        return out;
    }
    let mut start = 0usize;
    while start + is_bars + oos_bars <= n && out.len() < MAX_WINDOWS {
        out.push((start, start + is_bars, start + is_bars + oos_bars));
        start += step;
    }
    out
}

/// avg OOS / avg IS, defined only against a positive finite base.
pub fn efficiency(avg_is: f64, avg_oos: f64) -> Option<f64> {
    (avg_is.is_finite() && avg_oos.is_finite() && avg_is > 0.0).then(|| avg_oos / avg_is)
}

#[allow(clippy::too_many_arguments)]
pub fn run(
    bars: &[PriceBar],
    strategy_kind: &str,
    baseline_rules: &serde_json::Value,
    grid: &serde_json::Map<String, serde_json::Value>,
    sizing: &Sizing,
    cfg: BacktestConfig,
    metric: OptimizeMetric,
    is_bars: usize,
    oos_bars: usize,
    step: Option<usize>,
    gates: crate::algo_backtest::BtGates,
) -> Result<AlgoWfResult, OptimizeError> {
    let step = step.unwrap_or(oos_bars).max(1);
    let windows_layout = layout(bars.len(), is_bars, oos_bars, step);
    if windows_layout.is_empty() {
        return Err(OptimizeError::StrategyBuild(format!(
            "{} bars can't fit one {is_bars}+{oos_bars} window",
            bars.len()
        )));
    }
    let mut windows = Vec::with_capacity(windows_layout.len());
    let mut oos_total_trades = 0usize;
    for (w, (start, is_end, oos_end)) in windows_layout.into_iter().enumerate() {
        // Optimize on the in-sample slice only.
        let opt = algo_optimize::run(
            &bars[start..is_end],
            strategy_kind,
            baseline_rules,
            grid,
            sizing,
            cfg,
            metric,
            1,
            gates.clone(),
        )?;
        let best = opt
            .top
            .first()
            .ok_or_else(|| OptimizeError::StrategyBuild("optimizer returned no results".into()))?;
        // Validate the winner on bars it has never seen.
        let strat = from_kind(strategy_kind, &best.entry_rules)
            .map_err(|e| OptimizeError::StrategyBuild(e.to_string()))?;
        let oos_bt = algo_backtest::run_with_gates(
            &bars[is_end..oos_end],
            strat.as_ref(),
            sizing,
            cfg,
            gates.clone(),
        );
        oos_total_trades += oos_bt.summary.trades;
        windows.push(WfWindow {
            window: w,
            is_range: (start, is_end),
            oos_range: (is_end, oos_end),
            best_overrides: best.overrides.clone(),
            is_score: best.metric_score,
            oos_score: metric.score(&oos_bt),
            oos_summary: oos_bt.summary,
        });
    }
    let n = windows.len() as f64;
    let avg_is = windows.iter().map(|w| w.is_score).sum::<f64>() / n;
    let avg_oos = windows.iter().map(|w| w.oos_score).sum::<f64>() / n;
    Ok(AlgoWfResult {
        strategy_kind: strategy_kind.to_string(),
        metric,
        windows,
        avg_is_score: avg_is,
        avg_oos_score: avg_oos,
        wf_efficiency: efficiency(avg_is, avg_oos),
        oos_total_trades,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algo_strategies::SideMode;
    use crate::models::BarInterval;
    use chrono::{TimeZone, Utc};
    use rust_decimal::Decimal;
    use std::str::FromStr;

    #[test]
    fn layout_pins_window_arithmetic() {
        // 300 bars, IS 100, OOS 50, step 50 → starts 0,50,100,150.
        let w = layout(300, 100, 50, 50);
        assert_eq!(
            w,
            vec![(0, 100, 150), (50, 150, 200), (100, 200, 250), (150, 250, 300)]
        );
        // Exactly one window when n == is + oos.
        assert_eq!(layout(150, 100, 50, 50), vec![(0, 100, 150)]);
        // Too short → none.
        assert!(layout(149, 100, 50, 50).is_empty());
        // Degenerate params → none, never a panic.
        assert!(layout(300, 0, 50, 50).is_empty());
        assert!(layout(300, 100, 50, 0).is_empty());
    }

    #[test]
    fn efficiency_defined_only_against_positive_base() {
        assert_eq!(efficiency(2.0, 1.0), Some(0.5));
        // Negative or zero IS base: a ratio there reads as nonsense.
        assert_eq!(efficiency(0.0, 1.0), None);
        assert_eq!(efficiency(-1.0, -0.5), None);
        assert_eq!(efficiency(f64::NAN, 1.0), None);
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
    fn run_produces_one_row_per_window_with_unseen_oos() {
        // Wavy series so the momentum strategy has something to chew on.
        let bars: Vec<PriceBar> = (0..260)
            .map(|i| bar(1_700_000_000 + i * 86_400, 100.0 + (i as f64 * 0.2).sin() * 8.0 + i as f64 * 0.05))
            .collect();
        let mut grid = serde_json::Map::new();
        grid.insert("roc_period".into(), serde_json::json!([5, 10]));
        let cfg = BacktestConfig {
            initial_equity: 100_000.0,
            fee_per_trade: 1.0,
            slippage_bps: 5.0,
            side_mode: SideMode::Both,
        };
        let r = run(
            &bars,
            "momentum",
            &serde_json::json!({}),
            &grid,
            &Sizing::default(),
            cfg,
            OptimizeMetric::Sharpe,
            120,
            60,
            None,
            crate::algo_backtest::BtGates::default(),
        )
        .expect("walk-forward must run");
        // layout(260, 120, 60, 60) → starts 0, 60 → 2 windows.
        assert_eq!(r.windows.len(), 2);
        for w in &r.windows {
            // OOS slice begins exactly where IS ends — nothing seen twice.
            assert_eq!(w.is_range.1, w.oos_range.0);
            assert_eq!(w.oos_range.1 - w.oos_range.0, 60);
            // The winner carries the swept key.
            assert!(w.best_overrides.get("roc_period").is_some());
        }
    }

    #[test]
    fn too_few_bars_is_an_error_not_a_silent_empty() {
        let bars: Vec<PriceBar> = (0..50)
            .map(|i| bar(1_700_000_000 + i * 86_400, 100.0))
            .collect();
        let mut grid = serde_json::Map::new();
        grid.insert("roc_period".into(), serde_json::json!([5]));
        let cfg = BacktestConfig {
            initial_equity: 100_000.0,
            fee_per_trade: 1.0,
            slippage_bps: 5.0,
            side_mode: SideMode::Both,
        };
        assert!(run(
            &bars,
            "momentum",
            &serde_json::json!({}),
            &grid,
            &Sizing::default(),
            cfg,
            OptimizeMetric::Sharpe,
            100,
            50,
            None,
            crate::algo_backtest::BtGates::default(),
        )
        .is_err());
    }
}
