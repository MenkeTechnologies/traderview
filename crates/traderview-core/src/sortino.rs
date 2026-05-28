//! Sortino ratio + downside-deviation calculator.
//!
//! Sortino is the "real" Sharpe for asymmetric distributions — it
//! penalizes only DOWNSIDE volatility, not upside:
//!
//!   sortino = (mean_return - mar) / downside_deviation × sqrt(annualization)
//!
//! Where downside_deviation =
//!   sqrt( mean( min(r - mar, 0)^2 ) )
//!
//! `mar` is the Minimum Acceptable Return — typically 0 (any loss
//! counts as bad) or the risk-free rate.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SortinoReport {
    pub mean_return: f64,
    pub mar: f64,
    pub downside_deviation: f64,
    pub sortino_ratio: f64,
    /// Number of below-MAR observations (the "downside count").
    pub downside_obs: usize,
    /// Total observations.
    pub n: usize,
}

pub fn compute(returns: &[f64], mar: f64, annualization: f64) -> SortinoReport {
    let n = returns.len();
    if n < 2 {
        return SortinoReport {
            mean_return: returns.first().copied().unwrap_or(0.0),
            mar,
            n,
            ..Default::default()
        };
    }
    let sum: f64 = returns.iter().sum();
    let mean = sum / n as f64;
    let downside_sum: f64 = returns
        .iter()
        .map(|r| {
            let d = r - mar;
            if d < 0.0 {
                d * d
            } else {
                0.0
            }
        })
        .sum();
    let downside_dev = (downside_sum / n as f64).sqrt();
    let downside_obs = returns.iter().filter(|r| **r < mar).count();
    // Floor annualization at 0 so a negative JSON value doesn't produce
    // NaN via sqrt(-x) and silently poison the ratio.
    let ann_sqrt = annualization.max(0.0).sqrt();
    let sortino = if downside_dev == 0.0 {
        if mean > mar {
            f64::INFINITY
        } else {
            0.0
        }
    } else {
        (mean - mar) / downside_dev * ann_sqrt
    };
    SortinoReport {
        mean_return: mean,
        mar,
        downside_deviation: downside_dev,
        sortino_ratio: sortino,
        downside_obs,
        n,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_default_with_zero_n() {
        let r = compute(&[], 0.0, 252.0);
        assert_eq!(r.n, 0);
        assert_eq!(r.sortino_ratio, 0.0);
    }

    #[test]
    fn single_observation_returns_default() {
        let r = compute(&[1.0], 0.0, 252.0);
        assert_eq!(r.n, 1);
        assert_eq!(
            r.sortino_ratio, 0.0,
            "need at least 2 obs for a meaningful sortino"
        );
    }

    #[test]
    fn all_positive_returns_infinite_sortino() {
        // No below-MAR obs → downside dev = 0 AND mean > MAR → infinite.
        let r = compute(&[1.0, 2.0, 3.0], 0.0, 252.0);
        assert_eq!(r.downside_deviation, 0.0);
        assert!(r.sortino_ratio.is_infinite());
        assert!(r.sortino_ratio > 0.0);
        assert_eq!(r.downside_obs, 0);
    }

    #[test]
    fn all_returns_at_mar_zero_sortino() {
        // No deviation, but no excess return either → 0 (not infinite).
        let r = compute(&[0.0, 0.0, 0.0], 0.0, 252.0);
        assert_eq!(r.sortino_ratio, 0.0);
    }

    #[test]
    fn symmetric_distribution_around_mar_zero_sortino() {
        // Returns symmetric around 0 → mean=0, sortino = 0/x = 0.
        let r = compute(&[-1.0, 0.0, 1.0], 0.0, 252.0);
        assert!((r.mean_return - 0.0).abs() < 1e-12);
        assert_eq!(r.sortino_ratio, 0.0);
        assert_eq!(r.downside_obs, 1);
    }

    #[test]
    fn positive_skew_yields_positive_sortino() {
        // Mostly winners, occasional small loss → positive sortino.
        let r = compute(&[1.0, 1.0, 1.0, -0.5], 0.0, 252.0);
        assert!(r.sortino_ratio > 0.0);
    }

    #[test]
    fn downside_obs_counts_strictly_below_mar() {
        // r == mar is NOT downside (strict <).
        let r = compute(&[0.0, -1.0, -2.0], 0.0, 252.0);
        assert_eq!(r.downside_obs, 2);
    }

    #[test]
    fn non_zero_mar_treats_below_mar_as_downside() {
        // MAR = 1. r = 0.5 is BELOW MAR → counts as downside.
        let r = compute(&[2.0, 0.5, 3.0], 1.0, 252.0);
        assert_eq!(r.downside_obs, 1);
    }

    #[test]
    fn sortino_higher_than_sharpe_when_distribution_skewed_positive() {
        // Compare against textbook sharpe for the same series. Sortino
        // ignores upside vol, so on a positively-skewed series it's higher.
        let returns = vec![5.0, 5.0, 5.0, -1.0];
        let sortino = compute(&returns, 0.0, 1.0).sortino_ratio;
        // Naive sharpe with stdev based on full distribution:
        let mean = returns.iter().sum::<f64>() / returns.len() as f64;
        let var: f64 =
            returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / returns.len() as f64;
        let stdev = var.sqrt();
        let sharpe = mean / stdev;
        assert!(sortino > sharpe,
            "positive-skew series should have higher sortino than sharpe: got sortino={}, sharpe={}",
            sortino, sharpe);
    }

    #[test]
    fn annualization_scales_with_sqrt() {
        // Same series at 4 vs 16 annualization → sortino at 16 = 2× at 4
        // (sqrt(16)/sqrt(4) = 2).
        let returns = vec![1.0, 2.0, -1.0, 3.0];
        let r4 = compute(&returns, 0.0, 4.0).sortino_ratio;
        let r16 = compute(&returns, 0.0, 16.0).sortino_ratio;
        assert!((r16 / r4 - 2.0).abs() < 1e-9);
    }
}
