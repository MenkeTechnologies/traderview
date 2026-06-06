//! Bootstrap P&L Confidence Intervals.
//!
//! Resamples per-trade or per-period P&L with replacement to construct
//! a non-parametric distribution of the strategy's total or annualized
//! return. Useful when normality / iid assumptions don't hold.
//!
//! Output:
//!   - per-resample total P&L distribution (sorted)
//!   - mean / median P&L
//!   - 5th / 95th percentile (90% CI)
//!   - 2.5th / 97.5th (95% CI)
//!   - probability of positive P&L
//!
//! Pure compute. Distinct from `block_bootstrap` which preserves
//! serial dependence; this is the simple iid bootstrap appropriate
//! for trade-level P&L (where trades are assumed iid).
//!
//! Companion to `block_bootstrap`, `expectancy_per_trade`,
//! `monte_carlo_var`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BootstrapPnlReport {
    pub mean_total_pnl: f64,
    pub median_total_pnl: f64,
    pub pnl_5th_percentile: f64,
    pub pnl_95th_percentile: f64,
    pub pnl_2_5th_percentile: f64,
    pub pnl_97_5th_percentile: f64,
    pub probability_positive: f64,
    pub n_resamples: usize,
    pub n_trades: usize,
}

pub fn bootstrap(trade_pnls: &[f64], n_resamples: usize, seed: u64) -> Option<BootstrapPnlReport> {
    let n = trade_pnls.len();
    if n < 5 || n_resamples < 100 {
        return None;
    }
    if trade_pnls.iter().any(|x| !x.is_finite()) {
        return None;
    }
    let mut state = seed;
    let mut resampled_totals = Vec::with_capacity(n_resamples);
    for _ in 0..n_resamples {
        let mut total = 0.0_f64;
        for _ in 0..n {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let idx = ((state >> 32) as usize) % n;
            total += trade_pnls[idx];
        }
        resampled_totals.push(total);
    }
    resampled_totals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n_f = n_resamples as f64;
    let mean: f64 = resampled_totals.iter().sum::<f64>() / n_f;
    let q = |p: f64| {
        let idx = ((p * n_f).floor() as usize).min(n_resamples - 1);
        resampled_totals[idx]
    };
    let prob_pos = resampled_totals.iter().filter(|x| **x > 0.0).count() as f64 / n_f;
    Some(BootstrapPnlReport {
        mean_total_pnl: mean,
        median_total_pnl: q(0.50),
        pnl_5th_percentile: q(0.05),
        pnl_95th_percentile: q(0.95),
        pnl_2_5th_percentile: q(0.025),
        pnl_97_5th_percentile: q(0.975),
        probability_positive: prob_pos,
        n_resamples,
        n_trades: n,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn too_short_or_few_resamples_returns_none() {
        assert!(bootstrap(&[1.0; 4], 1000, 42).is_none());
        assert!(bootstrap(&[1.0; 10], 50, 42).is_none());
    }

    #[test]
    fn nan_returns_none() {
        let mut t = vec![1.0_f64; 20];
        t[5] = f64::NAN;
        assert!(bootstrap(&t, 1000, 42).is_none());
    }

    #[test]
    fn deterministic_for_fixed_seed() {
        let trades = vec![10.0, -5.0, 15.0, -10.0, 20.0, -8.0, 12.0, 25.0];
        let r1 = bootstrap(&trades, 500, 42).unwrap();
        let r2 = bootstrap(&trades, 500, 42).unwrap();
        assert_eq!(r1.mean_total_pnl, r2.mean_total_pnl);
    }

    #[test]
    fn mean_resampled_close_to_n_times_input_mean() {
        let trades = vec![10.0, -5.0, 15.0, -10.0, 20.0, -8.0, 12.0, 25.0];
        let input_mean: f64 = trades.iter().sum::<f64>() / trades.len() as f64;
        let expected_total = input_mean * trades.len() as f64;
        let r = bootstrap(&trades, 5000, 42).unwrap();
        let rel = (r.mean_total_pnl - expected_total).abs() / expected_total.abs();
        assert!(
            rel < 0.15,
            "mean {} vs expected {}",
            r.mean_total_pnl,
            expected_total
        );
    }

    #[test]
    fn quantiles_ordered_correctly() {
        let trades = vec![10.0, -5.0, 15.0, -10.0, 20.0, -8.0];
        let r = bootstrap(&trades, 2000, 7).unwrap();
        assert!(r.pnl_2_5th_percentile <= r.pnl_5th_percentile);
        assert!(r.pnl_5th_percentile <= r.median_total_pnl);
        assert!(r.median_total_pnl <= r.pnl_95th_percentile);
        assert!(r.pnl_95th_percentile <= r.pnl_97_5th_percentile);
    }

    #[test]
    fn all_positive_trades_yield_high_probability_positive() {
        let trades = vec![10.0, 5.0, 20.0, 8.0, 15.0];
        let r = bootstrap(&trades, 1000, 42).unwrap();
        // Every resample sum > 0.
        assert!((r.probability_positive - 1.0).abs() < 1e-12);
    }

    #[test]
    fn all_negative_trades_yield_zero_probability_positive() {
        let trades = vec![-10.0, -5.0, -20.0, -8.0, -15.0];
        let r = bootstrap(&trades, 1000, 42).unwrap();
        assert!(r.probability_positive.abs() < 1e-12);
    }

    #[test]
    fn n_resamples_and_trades_reported() {
        let trades = vec![1.0; 25];
        let r = bootstrap(&trades, 500, 42).unwrap();
        assert_eq!(r.n_resamples, 500);
        assert_eq!(r.n_trades, 25);
    }
}
