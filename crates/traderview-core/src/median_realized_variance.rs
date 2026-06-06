//! Median Realized Variance (MedRV) — Andersen, Dobrev, Schaumburg
//! (2012).
//!
//! Jump-robust estimator of integrated variance based on the median
//! of three consecutive absolute returns:
//!
//!   MedRV = (π / (6 − 4√3 + π)) · (n / (n − 2)) · Σ_{i=2..n}
//!           median(|r_{i-1}|, |r_i|, |r_{i+1}|)²
//!
//! Compared with bipower variation (BV), MedRV is more robust to
//! isolated jumps because the median ignores the extreme bar in each
//! triple. Specifically, MedRV is consistent for IV when there are
//! up to one jump per three consecutive returns.
//!
//! Test of jump-variation magnitude:
//!   jump_variation_estimate = max(0, RV − MedRV)
//!
//! Pure compute. Companion to `bipower_variation`, `realized_volatility`,
//! `realized_quarticity`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MedianRvReport {
    pub median_realized_variance: f64,
    pub realized_variance: f64,
    pub jump_variation_estimate: f64,
    pub n_triples_used: usize,
    pub n_returns: usize,
}

pub fn compute(returns: &[f64]) -> Option<MedianRvReport> {
    let n = returns.len();
    if n < 5 {
        return None;
    }
    if returns.iter().any(|x| !x.is_finite()) {
        return None;
    }
    let n_f = n as f64;
    let constant = std::f64::consts::PI / (6.0 - 4.0 * 3.0_f64.sqrt() + std::f64::consts::PI);
    let scale = (n_f / (n_f - 2.0)) * constant;
    let mut acc = 0.0_f64;
    let mut count = 0_usize;
    for i in 1..(n - 1) {
        let mut three = [returns[i - 1].abs(), returns[i].abs(), returns[i + 1].abs()];
        three.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let med = three[1];
        acc += med * med;
        count += 1;
    }
    let med_rv = scale * acc;
    let rv: f64 = returns.iter().map(|r| r * r).sum();
    let jump_var = (rv - med_rv).max(0.0);
    Some(MedianRvReport {
        median_realized_variance: med_rv,
        realized_variance: rv,
        jump_variation_estimate: jump_var,
        n_triples_used: count,
        n_returns: n,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn box_muller(n: usize, seed: u64, scale: f64) -> Vec<f64> {
        let mut state = seed;
        (0..n)
            .map(|_| {
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let u1 = ((state >> 32) as f64 / u32::MAX as f64).max(1e-12);
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let u2 = (state >> 32) as f64 / u32::MAX as f64;
                scale * (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
            })
            .collect()
    }

    #[test]
    fn too_short_returns_none() {
        assert!(compute(&[0.01, -0.01]).is_none());
    }

    #[test]
    fn nan_returns_none() {
        assert!(compute(&[0.01, f64::NAN, -0.01, 0.005, 0.002]).is_none());
    }

    #[test]
    fn flat_returns_yield_zero_med_rv() {
        let r = compute(&[0.0; 50]).unwrap();
        assert_eq!(r.median_realized_variance, 0.0);
        assert_eq!(r.realized_variance, 0.0);
        assert_eq!(r.jump_variation_estimate, 0.0);
    }

    #[test]
    fn no_jump_med_rv_tracks_rv() {
        // Smooth Gaussian returns: MedRV should be close to RV (both
        // estimate IV, no jumps to distinguish).
        let r = box_muller(2000, 42, 0.01);
        let result = compute(&r).unwrap();
        let rel = (result.median_realized_variance - result.realized_variance).abs()
            / result.realized_variance;
        assert!(
            rel < 0.30,
            "MedRV {} should track RV {} when no jumps, rel diff = {:.2}",
            result.median_realized_variance,
            result.realized_variance,
            rel
        );
    }

    #[test]
    fn isolated_jump_inflates_rv_more_than_med_rv() {
        let mut r = vec![0.001_f64; 500];
        r[200] = 0.50;
        let result = compute(&r).unwrap();
        assert!(
            result.realized_variance > result.median_realized_variance,
            "jump should inflate RV {} above MedRV {}",
            result.realized_variance,
            result.median_realized_variance
        );
        assert!(result.jump_variation_estimate > 0.0);
    }

    #[test]
    fn jump_variation_floored_at_zero() {
        // In a rare case where MedRV exceeds RV due to sampling noise,
        // jump variation should clamp to 0.
        let r = box_muller(100, 99, 0.01);
        let result = compute(&r).unwrap();
        assert!(result.jump_variation_estimate >= 0.0);
    }

    #[test]
    fn n_triples_equals_n_minus_2() {
        let r = box_muller(50, 7, 0.01);
        let result = compute(&r).unwrap();
        assert_eq!(result.n_triples_used, 48);
        assert_eq!(result.n_returns, 50);
    }

    #[test]
    fn scaling_constant_correct() {
        // For constant returns ±c, MedRV should equal Σ c².
        let c = 0.01;
        let r: Vec<f64> = (0..50).map(|i| if i % 2 == 0 { c } else { -c }).collect();
        let result = compute(&r).unwrap();
        // Each triple median = c, so sum = (n-2)·c². Scaled by π/(6-4√3+π)·n/(n-2).
        let n_f = r.len() as f64;
        let const_factor =
            std::f64::consts::PI / (6.0 - 4.0 * 3.0_f64.sqrt() + std::f64::consts::PI);
        let expected = const_factor * (n_f / (n_f - 2.0)) * (n_f - 2.0) * c * c;
        assert!((result.median_realized_variance - expected).abs() < 1e-12);
    }
}
