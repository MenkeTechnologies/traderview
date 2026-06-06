//! Sample Autocorrelation Function (ACF) + Bartlett Confidence Bands.
//!
//! For a time series {x_t}, the sample autocorrelation at lag k is:
//!
//!   ρ̂(k) = Σ_{t=k+1..n} (x_t − x̄)(x_{t−k} − x̄)  /  Σ_{t=1..n} (x_t − x̄)²
//!
//! Bartlett (1946) showed that under the null of white noise, the
//! sample autocorrelations are approximately distributed as:
//!
//!   ρ̂(k) ~ N(0, 1/n)
//!
//! so 95% confidence bands are ±1.96/√n. A lag whose ACF lies outside
//! these bands is "significant" — evidence against white noise.
//!
//! Used in: residual diagnostics of fitted models, model-order
//! selection for MA(q) processes (PACF for AR(p)), random-walk tests.
//!
//! Pure compute. Companion to `partial_autocorrelation`, `ljung_box`,
//! `arima_111`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AcfReport {
    pub lags: Vec<usize>,
    pub autocorrelations: Vec<f64>,
    /// 95% Bartlett confidence-band half-width (±this value).
    pub confidence_band: f64,
    /// Indices into `lags` where ACF lies outside the band.
    pub significant_lags: Vec<usize>,
    pub n_observations: usize,
}

pub fn compute(series: &[f64], max_lag: usize) -> Option<AcfReport> {
    let n = series.len();
    if n < 5 || max_lag == 0 || max_lag >= n {
        return None;
    }
    if series.iter().any(|x| !x.is_finite()) {
        return None;
    }
    let n_f = n as f64;
    let mean: f64 = series.iter().sum::<f64>() / n_f;
    let denom: f64 = series.iter().map(|x| (x - mean).powi(2)).sum();
    if denom <= 0.0 {
        return None;
    }
    let mut acfs = Vec::with_capacity(max_lag + 1);
    let mut lags = Vec::with_capacity(max_lag + 1);
    for k in 0..=max_lag {
        let num: f64 = (k..n)
            .map(|t| (series[t] - mean) * (series[t - k] - mean))
            .sum();
        acfs.push(num / denom);
        lags.push(k);
    }
    let band = 1.96 / n_f.sqrt();
    let significant_lags: Vec<usize> = acfs
        .iter()
        .enumerate()
        .filter(|(k, v)| *k > 0 && v.abs() > band)
        .map(|(k, _)| k)
        .collect();
    Some(AcfReport {
        lags,
        autocorrelations: acfs,
        confidence_band: band,
        significant_lags,
        n_observations: n,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn too_short_returns_none() {
        let s = vec![0.01_f64; 3];
        assert!(compute(&s, 1).is_none());
    }

    #[test]
    fn zero_lag_returns_none() {
        let s: Vec<f64> = (0..50).map(|i| i as f64).collect();
        assert!(compute(&s, 0).is_none());
    }

    #[test]
    fn nan_input_returns_none() {
        let mut s: Vec<f64> = (0..50).map(|i| i as f64).collect();
        s[10] = f64::NAN;
        assert!(compute(&s, 5).is_none());
    }

    #[test]
    fn flat_series_returns_none() {
        let s = vec![100.0_f64; 50];
        assert!(compute(&s, 5).is_none());
    }

    #[test]
    fn lag_zero_acf_is_one() {
        let s: Vec<f64> = (0..50).map(|i| (i as f64 * 0.1).sin() * 5.0).collect();
        let r = compute(&s, 10).unwrap();
        assert!((r.autocorrelations[0] - 1.0).abs() < 1e-12);
    }

    #[test]
    fn random_walk_yields_high_acf_at_low_lags() {
        let mut state: u64 = 42;
        let mut s = vec![0.0_f64; 200];
        for i in 1..200 {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let step = ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 2.0;
            s[i] = s[i - 1] + step;
        }
        let r = compute(&s, 10).unwrap();
        assert!(
            r.autocorrelations[1] > 0.8,
            "RW lag-1 ACF should be near 1, got {}",
            r.autocorrelations[1]
        );
    }

    #[test]
    fn white_noise_yields_small_acf_at_low_lags() {
        let mut state: u64 = 11;
        let s: Vec<f64> = (0..500)
            .map(|_| {
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 2.0
            })
            .collect();
        let r = compute(&s, 10).unwrap();
        // Most lag-1..10 ACFs should be inside the Bartlett band for white noise.
        let outside: usize = (1..=10)
            .filter(|k| r.autocorrelations[*k].abs() > r.confidence_band)
            .count();
        assert!(
            outside <= 3,
            "{outside} lags outside 95% band on white noise: {:?}",
            &r.autocorrelations[1..=10]
        );
    }

    #[test]
    fn ar1_process_acf_decays_geometrically() {
        // x_t = 0.8 · x_{t-1} + ε. ACF should decay roughly as 0.8^k.
        let mut state: u64 = 7;
        let phi = 0.8_f64;
        let mut s = vec![0.0_f64; 1000];
        for i in 1..1000 {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let eps = ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 0.5;
            s[i] = phi * s[i - 1] + eps;
        }
        let r = compute(&s, 5).unwrap();
        // Lag-1 ACF should be close to φ = 0.8.
        assert!(
            (r.autocorrelations[1] - phi).abs() < 0.05,
            "AR(1) lag-1 ACF expected ~0.8, got {}",
            r.autocorrelations[1]
        );
    }

    #[test]
    fn bartlett_band_scales_inversely_with_sqrt_n() {
        let s50: Vec<f64> = (0..50).map(|i| i as f64).collect();
        let s500: Vec<f64> = (0..500).map(|i| i as f64).collect();
        let r50 = compute(&s50, 5).unwrap();
        let r500 = compute(&s500, 5).unwrap();
        let ratio = r50.confidence_band / r500.confidence_band;
        let expected = (500.0_f64 / 50.0).sqrt();
        assert!(
            (ratio - expected).abs() < 0.01,
            "Bartlett ratio {}, expected {}",
            ratio,
            expected
        );
    }
}
