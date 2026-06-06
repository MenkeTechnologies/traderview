//! Ljung-Box (1978) test for serial autocorrelation up to lag h.
//!
//!   Q = n · (n+2) · Σ_{k=1}^{h} ρ̂_k² / (n − k)
//!
//! Under H₀ (no autocorrelation through lag h), Q ~ χ²(h). Reject when
//! Q exceeds the χ²(h) critical value. Used to:
//!   - Validate that residuals from a model (ARMA, GARCH) are white noise
//!   - Test return-series independence / momentum effects
//!   - Detect any leftover predictability the model missed
//!
//! Pure compute. Caller compares the Q statistic against tabulated
//! χ²(h) thresholds (5% level: h=10 → 18.31, h=20 → 31.41).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LjungBoxReport {
    pub q_statistic: f64,
    pub autocorrelations: Vec<f64>,
    pub n_observations: usize,
    pub lags: usize,
}

pub fn test(series: &[f64], lags: usize) -> Option<LjungBoxReport> {
    let n = series.len();
    if n < lags + 2 || lags == 0 {
        return None;
    }
    let clean: Vec<f64> = series.iter().copied().filter(|x| x.is_finite()).collect();
    let n_clean = clean.len();
    if n_clean < lags + 2 {
        return None;
    }
    let mean = clean.iter().sum::<f64>() / n_clean as f64;
    let var: f64 = clean.iter().map(|x| (x - mean).powi(2)).sum();
    if var <= 0.0 {
        return None;
    }
    let mut acf = vec![0.0_f64; lags];
    for k in 1..=lags {
        let mut cov = 0.0_f64;
        for t in k..n_clean {
            cov += (clean[t] - mean) * (clean[t - k] - mean);
        }
        acf[k - 1] = cov / var;
    }
    let n_f = n_clean as f64;
    let q: f64 = acf
        .iter()
        .enumerate()
        .map(|(idx, rho)| rho * rho / (n_f - (idx + 1) as f64))
        .sum::<f64>()
        * n_f
        * (n_f + 2.0);
    Some(LjungBoxReport {
        q_statistic: q,
        autocorrelations: acf,
        n_observations: n_clean,
        lags,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn too_short_returns_none() {
        assert!(test(&[1.0; 5], 10).is_none());
    }

    #[test]
    fn zero_lags_returns_none() {
        assert!(test(&[1.0; 50], 0).is_none());
    }

    #[test]
    fn flat_series_returns_none() {
        assert!(test(&[1.0; 100], 5).is_none());
    }

    #[test]
    fn iid_gaussian_yields_small_q() {
        let n = 1_000;
        let mut state: u64 = 42;
        let mut x = Vec::with_capacity(n);
        for _ in 0..n / 2 {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let u1 = ((state >> 32) as f64 / u32::MAX as f64).max(1e-12);
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let u2 = (state >> 32) as f64 / u32::MAX as f64;
            let z1 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
            let z2 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).sin();
            x.push(z1);
            x.push(z2);
        }
        let r = test(&x, 10).unwrap();
        // Under H₀ Q ~ χ²(10) → mean = 10, 95th percentile ≈ 18.3. Be lenient
        // for one-sample sampling noise.
        assert!(
            r.q_statistic < 40.0,
            "iid series should NOT have huge Q, got {}",
            r.q_statistic
        );
    }

    #[test]
    fn ar1_series_yields_large_q() {
        // x_t = 0.7 · x_{t−1} + noise — strong lag-1 autocorrelation.
        let n = 1_000;
        let mut state: u64 = 999;
        let mut x = vec![0.0_f64; n];
        for t in 1..n {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let noise = ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 0.5;
            x[t] = 0.7 * x[t - 1] + noise;
        }
        let r = test(&x, 10).unwrap();
        // Should massively reject H₀.
        assert!(
            r.q_statistic > 50.0,
            "AR(1) series should produce large Q, got {}",
            r.q_statistic
        );
    }

    #[test]
    fn lag1_autocorrelation_approximately_phi() {
        let n = 5_000;
        let mut state: u64 = 7;
        let mut x = vec![0.0_f64; n];
        for t in 1..n {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let noise = ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 0.5;
            x[t] = 0.5 * x[t - 1] + noise;
        }
        let r = test(&x, 5).unwrap();
        // ρ_1 should be ≈ 0.5 for AR(1) with φ = 0.5.
        assert!((r.autocorrelations[0] - 0.5).abs() < 0.1);
    }

    #[test]
    fn nan_inputs_skipped() {
        let mut x: Vec<f64> = (0..50).map(|i| (i as f64 * 0.1).sin()).collect();
        x[20] = f64::NAN;
        let r = test(&x, 5).unwrap();
        assert_eq!(r.n_observations, 49);
    }
}
