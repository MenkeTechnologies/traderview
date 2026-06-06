//! Omega Ratio — Keating & Shadwick (2002).
//!
//!   Ω(τ) = Σ max(r_i − τ, 0) / Σ max(τ − r_i, 0)
//!
//! where τ is a target return threshold (often 0 for raw gain/loss).
//! Generalizes Sharpe in that it uses the full return distribution
//! rather than just first two moments. Ω > 1 means the gains above τ
//! outweigh losses below τ; higher is better.
//!
//! Returns: ratio plus the numerator/denominator separately so the
//! caller can detect the degenerate denominator-zero case (no losses
//! below threshold → Ω = +∞, often a backtest red flag for overfit).
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct OmegaReport {
    pub ratio: f64,
    pub gains_above_threshold: f64,
    pub losses_below_threshold: f64,
    pub n_gains: usize,
    pub n_losses: usize,
}

pub fn compute(returns: &[f64], threshold: f64) -> Option<OmegaReport> {
    if returns.is_empty() || !threshold.is_finite() {
        return None;
    }
    let mut gains = 0.0_f64;
    let mut losses = 0.0_f64;
    let mut n_gain = 0usize;
    let mut n_loss = 0usize;
    let mut any_valid = false;
    for r in returns {
        if !r.is_finite() {
            continue;
        }
        any_valid = true;
        let diff = r - threshold;
        if diff > 0.0 {
            gains += diff;
            n_gain += 1;
        } else if diff < 0.0 {
            losses += -diff;
            n_loss += 1;
        }
    }
    if !any_valid {
        return None;
    }
    let ratio = if losses == 0.0 {
        f64::INFINITY
    } else {
        gains / losses
    };
    Some(OmegaReport {
        ratio,
        gains_above_threshold: gains,
        losses_below_threshold: losses,
        n_gains: n_gain,
        n_losses: n_loss,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_none() {
        assert!(compute(&[], 0.0).is_none());
    }

    #[test]
    fn nan_threshold_returns_none() {
        assert!(compute(&[0.01, 0.02], f64::NAN).is_none());
    }

    #[test]
    fn all_nan_returns_none() {
        let r = vec![f64::NAN; 10];
        assert!(compute(&r, 0.0).is_none());
    }

    #[test]
    fn flat_returns_equal_threshold_yields_zero_over_zero_finite() {
        // All returns exactly at threshold → gains = losses = 0 → ratio = ∞.
        let r = vec![0.0; 10];
        let out = compute(&r, 0.0).unwrap();
        assert_eq!(out.gains_above_threshold, 0.0);
        assert_eq!(out.losses_below_threshold, 0.0);
        assert!(out.ratio.is_infinite());
    }

    #[test]
    fn no_losses_returns_infinity() {
        let r = vec![0.01, 0.02, 0.03];
        let out = compute(&r, 0.0).unwrap();
        assert!(out.ratio.is_infinite());
        assert_eq!(out.n_losses, 0);
    }

    #[test]
    fn no_gains_returns_zero() {
        let r = vec![-0.01, -0.02, -0.03];
        let out = compute(&r, 0.0).unwrap();
        assert_eq!(out.ratio, 0.0);
        assert_eq!(out.n_gains, 0);
    }

    #[test]
    fn balanced_symmetric_returns_yields_omega_one() {
        let r = vec![0.01, -0.01, 0.01, -0.01];
        let out = compute(&r, 0.0).unwrap();
        assert!((out.ratio - 1.0).abs() < 1e-9);
        assert_eq!(out.n_gains, 2);
        assert_eq!(out.n_losses, 2);
    }

    #[test]
    fn threshold_shifts_classification() {
        // Returns: 0.005, 0.015. τ=0 → both gains. τ=0.01 → one each.
        let r = vec![0.005, 0.015];
        let zero_t = compute(&r, 0.0).unwrap();
        assert_eq!(zero_t.n_losses, 0);
        let one_t = compute(&r, 0.01).unwrap();
        assert_eq!(one_t.n_gains, 1);
        assert_eq!(one_t.n_losses, 1);
    }

    #[test]
    fn nan_observations_skipped_safely() {
        let r = vec![0.01, f64::NAN, -0.01];
        let out = compute(&r, 0.0).unwrap();
        // NaN dropped — 1 gain, 1 loss.
        assert_eq!(out.n_gains, 1);
        assert_eq!(out.n_losses, 1);
        assert!((out.ratio - 1.0).abs() < 1e-9);
    }
}
