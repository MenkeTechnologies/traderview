//! Strategy-portfolio analysis — which strategies DIVERSIFY each other.
//!
//! The tournament ranks strategies individually; this runs a chosen set
//! over the same bars, computes the pairwise Pearson correlation of
//! their per-bar equity returns, and builds the equal-weight combined
//! curve. Two mediocre strategies with low correlation can beat either
//! alone — the diversification_benefit field (combined Sharpe minus
//! the average individual Sharpe) measures exactly that.
//!
//! Sharpe here is the same un-annualized bar-Sharpe (mean/std of bar
//! returns) the backtester reports, so legs and combined are
//! comparable. Pearson reuses the shared correlation core.

use crate::algo_backtest::{run, BacktestConfig};
use crate::algo_strategies::{from_kind, Sizing};
use crate::correlation::pearson;
use crate::models::PriceBar;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct CurveStats {
    pub total_return_pct: f64,
    pub max_drawdown_pct: f64,
    pub sharpe: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PortfolioLeg {
    pub kind: String,
    pub trades: usize,
    pub stats: CurveStats,
}

#[derive(Debug, Clone, Serialize)]
pub struct PortfolioResult {
    pub legs: Vec<PortfolioLeg>,
    /// Symmetric matrix in leg order; diagonal 1.0; None where a leg's
    /// returns are degenerate (never traded → zero variance).
    pub correlation: Vec<Vec<Option<f64>>>,
    pub combined: CurveStats,
    pub avg_individual_sharpe: f64,
    /// Combined Sharpe − average individual Sharpe.
    pub diversification_benefit: f64,
    /// Buy-and-hold over the same bars — the passive baseline the
    /// combination must clear.
    pub benchmark: CurveStats,
}

/// Per-bar simple returns from an equity series.
pub fn bar_returns(equity: &[f64]) -> Vec<f64> {
    equity
        .windows(2)
        .map(|w| if w[0] > 0.0 { w[1] / w[0] - 1.0 } else { 0.0 })
        .collect()
}

/// Equal-weight combination: mean of each bar's leg returns. Legs must
/// be equal length (the backtester emits one point per bar).
pub fn combine_equal_weight(legs: &[Vec<f64>]) -> Vec<f64> {
    let Some(n) = legs.iter().map(|l| l.len()).min() else {
        return Vec::new();
    };
    (0..n)
        .map(|t| legs.iter().map(|l| l[t]).sum::<f64>() / legs.len() as f64)
        .collect()
}

/// Total return / max drawdown / bar-Sharpe from a return series —
/// same Sharpe convention as the backtester (mean/std, un-annualized).
pub fn curve_stats(returns: &[f64]) -> CurveStats {
    let mut equity = 1.0_f64;
    let mut peak = 1.0_f64;
    let mut max_dd = 0.0_f64;
    for r in returns {
        equity *= 1.0 + r;
        if equity > peak {
            peak = equity;
        }
        let dd = if peak > 0.0 { (peak - equity) / peak } else { 0.0 };
        if dd > max_dd {
            max_dd = dd;
        }
    }
    let sharpe = if returns.len() > 1 {
        let mean = returns.iter().sum::<f64>() / returns.len() as f64;
        let var = returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / returns.len() as f64;
        let std = var.sqrt();
        if std > 0.0 {
            mean / std
        } else {
            0.0
        }
    } else {
        0.0
    };
    CurveStats {
        total_return_pct: (equity - 1.0) * 100.0,
        max_drawdown_pct: max_dd * 100.0,
        sharpe,
    }
}

/// Run the chosen kinds over the same bars and analyze the set.
pub fn analyze(
    bars: &[PriceBar],
    kinds: &[String],
    sizing: &Sizing,
    cfg: &BacktestConfig,
) -> Result<PortfolioResult, String> {
    if kinds.len() < 2 {
        return Err("portfolio analysis needs at least 2 strategies".into());
    }
    let mut legs = Vec::with_capacity(kinds.len());
    let mut leg_returns: Vec<Vec<f64>> = Vec::with_capacity(kinds.len());
    for kind in kinds {
        let strat = from_kind(kind, &serde_json::json!({})).map_err(|e| e.to_string())?;
        if strat.required_symbols().is_some() {
            return Err(format!("{kind} is multi-symbol — not portfolioable on one bar set"));
        }
        let bt = run(bars, strat.as_ref(), sizing, *cfg);
        let equity: Vec<f64> = bt.equity.iter().map(|p| p.equity).collect();
        let returns = bar_returns(&equity);
        legs.push(PortfolioLeg {
            kind: kind.clone(),
            trades: bt.summary.trades,
            stats: curve_stats(&returns),
        });
        leg_returns.push(returns);
    }
    let n = legs.len();
    let mut correlation = vec![vec![None; n]; n];
    for i in 0..n {
        correlation[i][i] = Some(1.0);
        for j in (i + 1)..n {
            let c = pearson(&leg_returns[i], &leg_returns[j]);
            correlation[i][j] = c;
            correlation[j][i] = c;
        }
    }
    let combined = curve_stats(&combine_equal_weight(&leg_returns));
    let avg_individual_sharpe =
        legs.iter().map(|l| l.stats.sharpe).sum::<f64>() / n as f64;
    Ok(PortfolioResult {
        diversification_benefit: combined.sharpe - avg_individual_sharpe,
        legs,
        correlation,
        combined,
        avg_individual_sharpe,
        benchmark: crate::algo_tournament::buy_and_hold(bars),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bar_returns_pin_simple_math() {
        let r = bar_returns(&[100.0, 110.0, 99.0]);
        assert!((r[0] - 0.10).abs() < 1e-12);
        assert!((r[1] + 0.10).abs() < 1e-12);
        // Non-positive base yields 0, not a NaN that poisons everything.
        assert_eq!(bar_returns(&[0.0, 50.0]), vec![0.0]);
    }

    #[test]
    fn equal_weight_combination_is_the_per_bar_mean() {
        let legs = vec![vec![0.02, -0.01], vec![0.00, 0.03]];
        let c = combine_equal_weight(&legs);
        assert!((c[0] - 0.01).abs() < 1e-12);
        assert!((c[1] - 0.01).abs() < 1e-12);
    }

    #[test]
    fn curve_stats_pin_return_and_drawdown() {
        // +10% then −20%: total = 1.1 × 0.8 − 1 = −12%; max DD = 20%.
        let s = curve_stats(&[0.10, -0.20]);
        assert!((s.total_return_pct + 12.0).abs() < 1e-9);
        assert!((s.max_drawdown_pct - 20.0).abs() < 1e-9);
    }

    #[test]
    fn anticorrelated_legs_diversify_the_drawdown_away() {
        // Two volatile legs that exactly offset: each swings ±10% with
        // brutal drawdowns; the equal-weight combination is flat.
        let a = vec![0.10, -0.10, 0.10, -0.10];
        let b = vec![-0.10, 0.10, -0.10, 0.10];
        let combined = curve_stats(&combine_equal_weight(&[a.clone(), b.clone()]));
        assert_eq!(combined.max_drawdown_pct, 0.0);
        // And the correlation that explains it is exactly −1.
        let c = pearson(&a, &b).unwrap();
        assert!((c + 1.0).abs() < 1e-12);
    }

    #[test]
    fn fewer_than_two_kinds_is_an_error() {
        let bars: Vec<PriceBar> = Vec::new();
        assert!(analyze(
            &bars,
            &["momentum".to_string()],
            &Sizing::default(),
            &BacktestConfig {
                initial_equity: 100_000.0,
                fee_per_trade: 1.0,
                slippage_bps: 5.0,
                side_mode: crate::algo_strategies::SideMode::Both,
            },
        )
        .is_err());
    }
}
