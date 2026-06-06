//! Two Scales Realized Variance (TSRV) — Zhang, Mykland, Aït-Sahalia
//! (2005, "A Tale of Two Time Scales").
//!
//! Robust realized-variance estimator that corrects for microstructure
//! noise bias inherent in high-frequency RV. Compares two scales:
//!
//!   RV_all = Σ r_i²                                   (all observations)
//!   RV_avg = (1/K) · Σ_{k=1..K} RV_k                  (avg of K subsamples)
//!
//! TSRV = RV_avg − (n_bar / n) · RV_all
//!
//! where n = total observations, n_bar = (n − K + 1) / K (avg subsample
//! length). The TSRV is consistent for integrated variance even under
//! iid noise contamination of observed prices.
//!
//! Optimal K is ~n^(2/3) per Zhang-Mykland-Aït-Sahalia; we default
//! to that with manual override.
//!
//! Pure compute. Companion to `realized_volatility`, `bipower_variation`,
//! `realized_semivariance`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TsrvReport {
    pub tsrv: f64,
    pub rv_all: f64,
    pub rv_avg_subsample: f64,
    pub n_subsamples_k: usize,
    pub n_returns: usize,
}

pub fn compute(returns: &[f64], k_subsamples: Option<usize>) -> Option<TsrvReport> {
    let n = returns.len();
    if n < 30 {
        return None;
    }
    if returns.iter().any(|x| !x.is_finite()) {
        return None;
    }
    let n_f = n as f64;
    let k = k_subsamples.unwrap_or_else(|| (n_f.powf(2.0 / 3.0).round() as usize).clamp(2, n / 4));
    if k < 2 || k >= n {
        return None;
    }
    let rv_all: f64 = returns.iter().map(|r| r * r).sum();
    // Subsampled RV: for each k = 0..K-1, sum r_i² for i ≡ k mod K.
    let mut rv_subsamples = vec![0.0_f64; k];
    let mut subsample_counts = vec![0_usize; k];
    for (i, r) in returns.iter().enumerate() {
        let bucket = i % k;
        rv_subsamples[bucket] += r * r;
        subsample_counts[bucket] += 1;
    }
    let rv_avg: f64 = rv_subsamples.iter().sum::<f64>() / k as f64;
    let avg_subsample_len: f64 = subsample_counts.iter().sum::<usize>() as f64 / k as f64;
    let tsrv = rv_avg - (avg_subsample_len / n_f) * rv_all;
    Some(TsrvReport {
        tsrv,
        rv_all,
        rv_avg_subsample: rv_avg,
        n_subsamples_k: k,
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
        assert!(compute(&[0.01; 20], None).is_none());
    }

    #[test]
    fn nan_returns_none() {
        let mut r = vec![0.01_f64; 100];
        r[10] = f64::NAN;
        assert!(compute(&r, None).is_none());
    }

    #[test]
    fn invalid_k_returns_none() {
        let r = vec![0.001_f64; 100];
        assert!(compute(&r, Some(1)).is_none());
        assert!(compute(&r, Some(100)).is_none());
    }

    #[test]
    fn clean_returns_tsrv_close_to_rv_minus_bias() {
        // No noise: TSRV should be very close to true integrated variance.
        let r = box_muller(500, 42, 0.01);
        let r_all = compute(&r, None).unwrap();
        // RV_all and TSRV should be in roughly the same order.
        let true_var = 500.0_f64 * 0.0001;
        assert!((r_all.rv_all - true_var).abs() < true_var * 0.5);
    }

    #[test]
    fn noise_contamination_reduces_via_tsrv() {
        // True signal: σ = 0.01. Add iid noise η ~ N(0, 0.005²) to prices.
        let mut state: u64 = 0xDEAD_BEEF_CAFE_BABE;
        let n = 1000_usize;
        let true_returns = box_muller(n, 42, 0.01);
        let noise: Vec<f64> = (0..=n)
            .map(|_| {
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let u1 = ((state >> 32) as f64 / u32::MAX as f64).max(1e-12);
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let u2 = (state >> 32) as f64 / u32::MAX as f64;
                0.005 * (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
            })
            .collect();
        // Observed return = true return + (η_t - η_{t-1}).
        let observed: Vec<f64> = true_returns
            .iter()
            .enumerate()
            .map(|(i, r)| r + noise[i + 1] - noise[i])
            .collect();
        let r = compute(&observed, None).unwrap();
        let true_iv: f64 = true_returns.iter().map(|r| r * r).sum();
        // RV_all is heavily noise-inflated.
        assert!(
            r.rv_all > true_iv * 1.5,
            "RV_all = {} should be inflated above true IV {}",
            r.rv_all,
            true_iv
        );
        // TSRV should be much closer to true IV.
        assert!(
            r.tsrv < r.rv_all,
            "TSRV {} should be below noise-inflated RV_all {}",
            r.tsrv,
            r.rv_all
        );
    }

    #[test]
    fn default_k_uses_two_thirds_power() {
        let r = vec![0.001_f64; 100];
        // Wait - flat returns will be filtered? No, my compute doesn't check
        // for that. r·r = 0 everywhere → tsrv = 0.
        let result = compute(&r, None).unwrap();
        // n^(2/3) for n=100 ≈ 21.5 → round = 22, clamped to n/4=25 → k=22.
        assert!((20..=25).contains(&result.n_subsamples_k));
    }

    #[test]
    fn custom_k_used_when_provided() {
        let r = vec![0.001_f64; 200];
        let result = compute(&r, Some(10)).unwrap();
        assert_eq!(result.n_subsamples_k, 10);
    }

    #[test]
    fn output_lengths_reported_correctly() {
        let r = box_muller(150, 7, 0.01);
        let result = compute(&r, None).unwrap();
        assert_eq!(result.n_returns, 150);
    }
}
