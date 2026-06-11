//! Parameter grid-search optimizer for algo strategies.
//!
//! Takes a baseline `entry_rules` JSON + a grid of override paths and
//! candidate values, expands the Cartesian product, runs the
//! backtester for each combination, and returns the top-N results
//! ranked by a chosen metric.
//!
//! Grid format (callable from the REST layer with arbitrary JSON):
//! ```json
//! {
//!   "fast_period":  [5, 9, 13],
//!   "slow_period":  [21, 34, 55],
//!   "adx_min":      [20.0, 25.0, 30.0]
//! }
//! ```
//! The product above produces 3 × 3 × 3 = 27 backtests. Each
//! combination's `entry_rules` is the baseline with the listed keys
//! overwritten.
//!
//! Reject grids exceeding `MAX_COMBINATIONS` so a runaway sweep
//! can't pin the CPU for hours on the desktop process.

use crate::algo_backtest::{self, AlgoBtResult, BacktestConfig};
use crate::algo_strategies::{from_kind, Sizing};
use crate::models::PriceBar;
use serde::{Deserialize, Serialize};

pub const MAX_COMBINATIONS: usize = 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OptimizeMetric {
    Sharpe,
    TotalReturn,
    ProfitFactor,
    AvgR,
    /// Total return penalized by max drawdown: total_return - max_dd.
    /// Discourages high-leverage gradient hunters with brutal DDs.
    ReturnMinusDd,
}

impl OptimizeMetric {
    pub fn score(&self, r: &AlgoBtResult) -> f64 {
        match self {
            Self::Sharpe => r.summary.sharpe,
            Self::TotalReturn => r.summary.total_return_pct,
            Self::ProfitFactor => {
                let pf = r.summary.profit_factor;
                if pf.is_finite() {
                    pf
                } else {
                    0.0
                }
            }
            Self::AvgR => r.summary.avg_r,
            Self::ReturnMinusDd => r.summary.total_return_pct - r.summary.max_drawdown_pct,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum OptimizeError {
    #[error("empty grid: at least one override key/value required")]
    EmptyGrid,
    #[error("grid expands to {0} combinations; max allowed is {MAX_COMBINATIONS}")]
    TooManyCombinations(usize),
    #[error("strategy build: {0}")]
    StrategyBuild(String),
}

#[derive(Debug, Clone, Serialize)]
pub struct OptimizeRunResult {
    /// The entry_rules JSON for this combination (baseline + overrides).
    pub entry_rules: serde_json::Value,
    /// Only the OVERRIDE values for THIS run — handy for the UI to
    /// render a compact "fast=9, slow=34, adx_min=25" badge instead of
    /// the full rules JSON.
    pub overrides: serde_json::Value,
    pub metric_score: f64,
    pub summary: crate::algo_backtest::AlgoBtSummary,
}

#[derive(Debug, Clone, Serialize)]
pub struct OptimizeResult {
    pub strategy_kind: String,
    pub metric: OptimizeMetric,
    pub combinations_evaluated: usize,
    pub top: Vec<OptimizeRunResult>,
}

/// Runs the optimizer.
///
/// * `strategy_kind` — discriminator (matches `algo_strategies::from_kind`).
/// * `baseline_rules` — the JSON used when a key isn't in the grid.
/// * `grid` — map of override-key → candidate-values. Each key whose
///   value is an array becomes a sweep dimension. Other JSON shapes
///   are passed through as scalars (1 candidate).
/// * `sizing` + `cfg` — same Sizing + BacktestConfig the
///   backtester takes; held constant across every combination so the
///   optimizer only sweeps STRATEGY params.
/// * `top_n` — how many results to keep, sorted descending by
///   `metric.score`.
pub fn run(
    bars: &[PriceBar],
    strategy_kind: &str,
    baseline_rules: &serde_json::Value,
    grid: &serde_json::Map<String, serde_json::Value>,
    sizing: &Sizing,
    cfg: BacktestConfig,
    metric: OptimizeMetric,
    top_n: usize,
) -> Result<OptimizeResult, OptimizeError> {
    if grid.is_empty() {
        return Err(OptimizeError::EmptyGrid);
    }
    // Materialize each dimension's candidate values.
    let mut dimensions: Vec<(String, Vec<serde_json::Value>)> = Vec::new();
    for (key, val) in grid {
        let candidates = match val {
            serde_json::Value::Array(arr) if !arr.is_empty() => arr.clone(),
            other => vec![other.clone()],
        };
        dimensions.push((key.clone(), candidates));
    }
    let total_combos: usize = dimensions.iter().map(|(_, v)| v.len()).product();
    if total_combos > MAX_COMBINATIONS {
        return Err(OptimizeError::TooManyCombinations(total_combos));
    }

    let mut results: Vec<OptimizeRunResult> = Vec::with_capacity(total_combos);
    // Walk the Cartesian product via base-N counter math; avoids
    // recursion + lets us reserve `results` up front.
    let mut indices = vec![0usize; dimensions.len()];
    let mut done = false;
    while !done {
        let mut overrides = serde_json::Map::new();
        for (dim_i, (key, candidates)) in dimensions.iter().enumerate() {
            overrides.insert(key.clone(), candidates[indices[dim_i]].clone());
        }
        let mut entry_rules = baseline_rules.clone();
        if let serde_json::Value::Object(ref mut map) = entry_rules {
            for (k, v) in overrides.iter() {
                map.insert(k.clone(), v.clone());
            }
        } else {
            // baseline was null / scalar — start from the overrides.
            entry_rules = serde_json::Value::Object(overrides.clone());
        }
        let strat = from_kind(strategy_kind, &entry_rules)
            .map_err(|e| OptimizeError::StrategyBuild(e.to_string()))?;
        let bt = algo_backtest::run(bars, strat.as_ref(), sizing, cfg);
        let score = metric.score(&bt);
        results.push(OptimizeRunResult {
            entry_rules,
            overrides: serde_json::Value::Object(overrides),
            metric_score: score,
            summary: bt.summary,
        });

        // Advance indices base-N
        let mut carry = 1usize;
        for dim_i in 0..dimensions.len() {
            indices[dim_i] += carry;
            if indices[dim_i] >= dimensions[dim_i].1.len() {
                indices[dim_i] = 0;
                carry = 1;
            } else {
                carry = 0;
                break;
            }
        }
        if carry == 1 {
            done = true;
        }
    }

    // Sort descending. NaN/-inf scores sink to the bottom.
    results.sort_by(|a, b| {
        b.metric_score
            .partial_cmp(&a.metric_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let top = results.into_iter().take(top_n.max(1)).collect();

    Ok(OptimizeResult {
        strategy_kind: strategy_kind.to_string(),
        metric,
        combinations_evaluated: total_combos,
        top,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::BarInterval;
    use chrono::{TimeZone, Utc};
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
    fn cartesian_product_size_is_correct() {
        let bars = uptrend_window();
        let baseline = serde_json::json!({});
        let mut grid = serde_json::Map::new();
        grid.insert("atr_period".into(), serde_json::json!([10, 14]));
        grid.insert("multiplier".into(), serde_json::json!([2.0, 3.0, 4.0]));
        let res = run(
            &bars,
            "supertrend",
            &baseline,
            &grid,
            &Sizing::default(),
            BacktestConfig::default(),
            OptimizeMetric::Sharpe,
            10,
        )
        .expect("optimize");
        assert_eq!(res.combinations_evaluated, 6, "2 × 3 must equal 6");
        assert_eq!(res.top.len(), 6);
    }

    #[test]
    fn top_n_truncates_and_sorts_descending() {
        let bars = uptrend_window();
        let baseline = serde_json::json!({});
        let mut grid = serde_json::Map::new();
        grid.insert("atr_period".into(), serde_json::json!([8, 10, 12, 14]));
        grid.insert("multiplier".into(), serde_json::json!([2.0, 3.0, 4.0]));
        let res = run(
            &bars,
            "supertrend",
            &baseline,
            &grid,
            &Sizing::default(),
            BacktestConfig::default(),
            OptimizeMetric::Sharpe,
            3,
        )
        .expect("optimize");
        assert_eq!(res.combinations_evaluated, 12);
        assert_eq!(res.top.len(), 3);
        for w in res.top.windows(2) {
            assert!(
                w[0].metric_score >= w[1].metric_score,
                "scores must be descending: {} vs {}",
                w[0].metric_score,
                w[1].metric_score
            );
        }
    }

    #[test]
    fn empty_grid_rejected() {
        let bars = uptrend_window();
        let res = run(
            &bars,
            "supertrend",
            &serde_json::json!({}),
            &serde_json::Map::new(),
            &Sizing::default(),
            BacktestConfig::default(),
            OptimizeMetric::Sharpe,
            5,
        );
        assert!(matches!(res, Err(OptimizeError::EmptyGrid)));
    }

    #[test]
    fn oversized_grid_rejected() {
        let bars = uptrend_window();
        // 12 × 12 × 12 = 1728 > MAX_COMBINATIONS.
        let large: Vec<serde_json::Value> = (1..=12).map(serde_json::Value::from).collect();
        let mut grid = serde_json::Map::new();
        grid.insert("a".into(), serde_json::json!(large.clone()));
        grid.insert("b".into(), serde_json::json!(large.clone()));
        grid.insert("c".into(), serde_json::json!(large));
        let res = run(
            &bars,
            "supertrend",
            &serde_json::json!({}),
            &grid,
            &Sizing::default(),
            BacktestConfig::default(),
            OptimizeMetric::Sharpe,
            5,
        );
        assert!(matches!(res, Err(OptimizeError::TooManyCombinations(1728))));
    }

    #[test]
    fn overrides_apply_to_entry_rules() {
        let bars = uptrend_window();
        // Baseline carries a key the grid DOESN'T override; the result
        // must still see that key (the override merge is non-destructive
        // for keys outside the grid).
        let baseline = serde_json::json!({ "atr_take_profit_mult": 5.0 });
        let mut grid = serde_json::Map::new();
        grid.insert("atr_period".into(), serde_json::json!([10, 14]));
        let res = run(
            &bars,
            "supertrend",
            &baseline,
            &grid,
            &Sizing::default(),
            BacktestConfig::default(),
            OptimizeMetric::Sharpe,
            5,
        )
        .expect("optimize");
        for entry in &res.top {
            assert_eq!(
                entry
                    .entry_rules
                    .get("atr_take_profit_mult")
                    .and_then(|v| v.as_f64()),
                Some(5.0),
                "baseline key must survive the merge"
            );
            assert!(entry.entry_rules.get("atr_period").is_some());
        }
    }

    #[test]
    fn unknown_strategy_kind_surfaces_build_error() {
        let bars = uptrend_window();
        let mut grid = serde_json::Map::new();
        grid.insert("anything".into(), serde_json::json!([1, 2]));
        let res = run(
            &bars,
            "this_strategy_does_not_exist",
            &serde_json::json!({}),
            &grid,
            &Sizing::default(),
            BacktestConfig::default(),
            OptimizeMetric::Sharpe,
            5,
        );
        match res {
            Err(OptimizeError::StrategyBuild(msg)) => {
                assert!(
                    msg.contains("this_strategy_does_not_exist"),
                    "error should name the bad kind, got: {msg}"
                );
            }
            other => panic!("expected StrategyBuild, got {other:?}"),
        }
    }

    #[test]
    fn metric_score_uses_correct_summary_field() {
        // Build a synthetic AlgoBtResult and verify every metric pulls
        // from the right summary field. No backtester run needed.
        use crate::algo_backtest::{AlgoBtResult, AlgoBtSummary};
        let summary = AlgoBtSummary {
            trades: 10,
            wins: 6,
            losses: 4,
            win_rate: 0.6,
            avg_win: 50.0,
            avg_loss: -25.0,
            avg_r: 1.5,
            profit_factor: 3.0,
            total_return_pct: 25.0,
            max_drawdown_pct: 8.0,
            final_equity: 125_000.0,
            sharpe: 1.7,
            bars_in_market_pct: 30.0,
            exits_by_stop: 4,
            exits_by_tp: 5,
            exits_by_signal: 1,
            exits_by_eod: 0,
        };
        let result = AlgoBtResult {
            gate_skips: crate::algo_backtest::GateSkips::default(),
            strategy_kind: "synthetic",
            trades: vec![],
            equity: vec![],
            summary,
        };
        assert!((OptimizeMetric::Sharpe.score(&result) - 1.7).abs() < 1e-9);
        assert!((OptimizeMetric::TotalReturn.score(&result) - 25.0).abs() < 1e-9);
        assert!((OptimizeMetric::ProfitFactor.score(&result) - 3.0).abs() < 1e-9);
        assert!((OptimizeMetric::AvgR.score(&result) - 1.5).abs() < 1e-9);
        // ReturnMinusDd = 25 − 8 = 17.
        assert!((OptimizeMetric::ReturnMinusDd.score(&result) - 17.0).abs() < 1e-9);
    }

    #[test]
    fn profit_factor_infinity_handled() {
        // pf=inf occurs when there are wins and no losses. score() must
        // coerce to 0.0 so the sort doesn't get poisoned by NaN/inf.
        use crate::algo_backtest::{AlgoBtResult, AlgoBtSummary};
        let mut summary = AlgoBtSummary {
            trades: 5,
            wins: 5,
            losses: 0,
            win_rate: 1.0,
            avg_win: 50.0,
            avg_loss: 0.0,
            avg_r: 2.0,
            profit_factor: f64::INFINITY,
            total_return_pct: 30.0,
            max_drawdown_pct: 0.0,
            final_equity: 130_000.0,
            sharpe: 2.5,
            bars_in_market_pct: 15.0,
            exits_by_stop: 0,
            exits_by_tp: 5,
            exits_by_signal: 0,
            exits_by_eod: 0,
        };
        let r = AlgoBtResult {
            gate_skips: crate::algo_backtest::GateSkips::default(),
            strategy_kind: "synthetic",
            trades: vec![],
            equity: vec![],
            summary: summary.clone(),
        };
        let s = OptimizeMetric::ProfitFactor.score(&r);
        assert!(s.is_finite(), "pf=inf must coerce, got {s}");
        assert_eq!(s, 0.0);

        // pf=0 (no wins, has losses) returns 0.0 cleanly too.
        summary.profit_factor = 0.0;
        let r2 = AlgoBtResult {
            gate_skips: crate::algo_backtest::GateSkips::default(),
            strategy_kind: "synthetic",
            trades: vec![],
            equity: vec![],
            summary,
        };
        assert_eq!(OptimizeMetric::ProfitFactor.score(&r2), 0.0);
    }
}
