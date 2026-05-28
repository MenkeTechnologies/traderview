//! Lower Partial Moments (LPM) — Bawa-Lindenberg (1977) downside-only
//! moments.
//!
//!   LPM_n(τ) = (1/N) · Σ max(τ − r_i, 0)^n
//!
//! Generalizes downside risk:
//!   - LPM_0: probability of falling below threshold τ (shortfall probability)
//!   - LPM_1: target shortfall (mean downside deviation)
//!   - LPM_2: target semi-variance (denominator of Sortino ratio)
//!   - LPM_3+: penalizes large downside moves more heavily
//!
//! Companion `upper_partial_moments` (UPM) covers the symmetric upside.
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct PartialMomentReport {
    pub lpm_0_shortfall_probability: f64,
    pub lpm_1_target_shortfall: f64,
    pub lpm_2_target_semi_variance: f64,
    pub upm_1_target_upside: f64,
    pub upm_2_target_semi_variance: f64,
    pub n_observations: usize,
    pub n_below_target: usize,
    pub n_above_target: usize,
    pub threshold: f64,
}

pub fn compute(returns: &[f64], threshold: f64) -> Option<PartialMomentReport> {
    if returns.is_empty() || !threshold.is_finite() { return None; }
    let mut n = 0_usize;
    let mut below = 0_usize;
    let mut above = 0_usize;
    let mut lpm_1 = 0.0_f64;
    let mut lpm_2 = 0.0_f64;
    let mut upm_1 = 0.0_f64;
    let mut upm_2 = 0.0_f64;
    for r in returns {
        if !r.is_finite() { continue; }
        n += 1;
        let downside = (threshold - r).max(0.0);
        let upside = (r - threshold).max(0.0);
        if downside > 0.0 { below += 1; }
        if upside > 0.0 { above += 1; }
        lpm_1 += downside;
        lpm_2 += downside * downside;
        upm_1 += upside;
        upm_2 += upside * upside;
    }
    if n == 0 { return None; }
    let n_f = n as f64;
    Some(PartialMomentReport {
        lpm_0_shortfall_probability: below as f64 / n_f,
        lpm_1_target_shortfall: lpm_1 / n_f,
        lpm_2_target_semi_variance: lpm_2 / n_f,
        upm_1_target_upside: upm_1 / n_f,
        upm_2_target_semi_variance: upm_2 / n_f,
        n_observations: n,
        n_below_target: below,
        n_above_target: above,
        threshold,
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
    fn nan_observations_skipped_safely() {
        let r = vec![0.01, f64::NAN, -0.02, 0.03];
        let report = compute(&r, 0.0).unwrap();
        assert_eq!(report.n_observations, 3);
    }

    #[test]
    fn all_above_threshold_yields_zero_lpm() {
        let r = vec![0.05; 10];
        let report = compute(&r, 0.0).unwrap();
        assert_eq!(report.lpm_0_shortfall_probability, 0.0);
        assert_eq!(report.lpm_1_target_shortfall, 0.0);
        assert_eq!(report.lpm_2_target_semi_variance, 0.0);
        assert!(report.upm_1_target_upside > 0.0);
    }

    #[test]
    fn all_below_threshold_yields_zero_upm() {
        let r = vec![-0.05; 10];
        let report = compute(&r, 0.0).unwrap();
        assert_eq!(report.lpm_0_shortfall_probability, 1.0);
        assert!(report.lpm_1_target_shortfall > 0.0);
        assert_eq!(report.upm_1_target_upside, 0.0);
    }

    #[test]
    fn symmetric_around_threshold_yields_equal_partial_moments() {
        let r = vec![-0.01, 0.01, -0.01, 0.01];
        let report = compute(&r, 0.0).unwrap();
        assert!((report.lpm_1_target_shortfall - report.upm_1_target_upside).abs() < 1e-12);
        assert!((report.lpm_2_target_semi_variance - report.upm_2_target_semi_variance).abs() < 1e-12);
    }

    #[test]
    fn larger_downside_increases_lpm2() {
        let small = vec![-0.01; 10];
        let large = vec![-0.10; 10];
        let r_small = compute(&small, 0.0).unwrap();
        let r_large = compute(&large, 0.0).unwrap();
        assert!(r_large.lpm_2_target_semi_variance > r_small.lpm_2_target_semi_variance);
    }

    #[test]
    fn shortfall_probability_matches_count() {
        let r = vec![-0.01, 0.02, -0.03, 0.04, -0.05];
        let report = compute(&r, 0.0).unwrap();
        // 3 of 5 below 0.
        assert!((report.lpm_0_shortfall_probability - 0.6).abs() < 1e-12);
        assert_eq!(report.n_below_target, 3);
        assert_eq!(report.n_above_target, 2);
    }

    #[test]
    fn threshold_shifts_classification() {
        let r = vec![0.01, 0.02, 0.03];
        let r_low = compute(&r, 0.0).unwrap();
        let r_mid = compute(&r, 0.02).unwrap();
        // At τ=0.02, one return is exactly at threshold (counted neither),
        // one above, one below.
        assert_eq!(r_low.n_above_target, 3);
        assert_eq!(r_mid.n_above_target, 1);
        assert_eq!(r_mid.n_below_target, 1);
    }
}
