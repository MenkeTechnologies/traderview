//! Subsampled Realized Variance.
//!
//! For each starting offset s ∈ [0, K), forms a thinned subseries of
//! returns aggregated at K-bar intervals, then averages the K
//! single-scale RV estimates:
//!
//!   r_k^{(s)} = Σ_{i in k-th K-block starting at s} r_i      (aggregated return)
//!   RV^{(s)}  = Σ_k (r_k^{(s)})²                             (single-scale RV)
//!   RV_sub    = (1/K) · Σ_{s=0..K-1} RV^{(s)}
//!
//! This reduces (but does not eliminate) microstructure-noise bias by
//! using slower sampling, while averaging across K offsets to reduce
//! the variance of the estimator vs a single arbitrary offset.
//!
//! Distinct from `two_scales_realized_variance` — TSRV is the formal
//! bias-corrected combination; this is the raw subsample-averaged RV
//! that forms TSRV's first piece.
//!
//! Pure compute. Companion to `two_scales_realized_variance`,
//! `realized_volatility`, `bipower_variation`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SubsampledRvReport {
    pub subsampled_rv: f64,
    pub per_offset_rv: Vec<f64>,
    pub k: usize,
    pub n_returns: usize,
}

pub fn compute(returns: &[f64], k: usize) -> Option<SubsampledRvReport> {
    if k < 2 || returns.len() < 2 * k { return None; }
    if returns.iter().any(|x| !x.is_finite()) { return None; }
    let n = returns.len();
    let mut per_offset = Vec::with_capacity(k);
    for s in 0..k {
        // Aggregate returns in blocks of K starting at offset s.
        let mut rv_s = 0.0_f64;
        let mut block_acc = 0.0_f64;
        let mut block_count = 0_usize;
        for (i, r) in returns.iter().enumerate().skip(s) {
            block_acc += r;
            block_count += 1;
            if block_count == k {
                rv_s += block_acc * block_acc;
                block_acc = 0.0;
                block_count = 0;
            }
            let _ = i;
        }
        per_offset.push(rv_s);
    }
    let avg: f64 = per_offset.iter().sum::<f64>() / k as f64;
    Some(SubsampledRvReport {
        subsampled_rv: avg,
        per_offset_rv: per_offset,
        k,
        n_returns: n,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn box_muller(n: usize, seed: u64, scale: f64) -> Vec<f64> {
        let mut state = seed;
        (0..n).map(|_| {
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let u1 = ((state >> 32) as f64 / u32::MAX as f64).max(1e-12);
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let u2 = (state >> 32) as f64 / u32::MAX as f64;
            scale * (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
        }).collect()
    }

    #[test]
    fn invalid_k_or_too_short_returns_none() {
        assert!(compute(&[0.01; 100], 1).is_none());
        assert!(compute(&[0.01; 5], 4).is_none());
    }

    #[test]
    fn nan_returns_none() {
        let mut r = vec![0.01_f64; 100];
        r[5] = f64::NAN;
        assert!(compute(&r, 4).is_none());
    }

    #[test]
    fn k_one_returns_none() {
        let r = vec![0.001_f64; 100];
        assert!(compute(&r, 1).is_none());
    }

    #[test]
    fn per_offset_length_equals_k() {
        let r = box_muller(200, 42, 0.01);
        let result = compute(&r, 5).unwrap();
        assert_eq!(result.per_offset_rv.len(), 5);
        assert_eq!(result.k, 5);
    }

    #[test]
    fn average_is_mean_of_per_offset() {
        let r = box_muller(200, 42, 0.01);
        let result = compute(&r, 5).unwrap();
        let avg: f64 = result.per_offset_rv.iter().sum::<f64>() / 5.0;
        assert!((result.subsampled_rv - avg).abs() < 1e-12);
    }

    #[test]
    fn subsampled_rv_smaller_than_full_rv_under_noise() {
        // Noisy returns: subsampled RV (slower sampling) should be smaller
        // than full RV because noise contribution per aggregated bar is
        // smaller relative to signal.
        let mut state: u64 = 0x0FEE_DFAC_EBAD_F00D;
        let n = 1000_usize;
        let true_returns = box_muller(n, 42, 0.01);
        let noise: Vec<f64> = (0..=n).map(|_| {
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let u1 = ((state >> 32) as f64 / u32::MAX as f64).max(1e-12);
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let u2 = (state >> 32) as f64 / u32::MAX as f64;
            0.005 * (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
        }).collect();
        let observed: Vec<f64> = true_returns.iter().enumerate()
            .map(|(i, r)| r + noise[i + 1] - noise[i]).collect();
        let rv_full: f64 = observed.iter().map(|r| r * r).sum();
        let r = compute(&observed, 10).unwrap();
        assert!(r.subsampled_rv < rv_full,
            "subsampled RV {} should be less than full RV {} under noise",
            r.subsampled_rv, rv_full);
    }
}
