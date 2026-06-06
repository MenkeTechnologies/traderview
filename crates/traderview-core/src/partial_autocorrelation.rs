//! Partial Autocorrelation Function (PACF) — Yule-Walker recursion.
//!
//! Where the ACF measures the total correlation at lag k (including
//! correlation transmitted through intermediate lags), the PACF
//! measures the DIRECT correlation at lag k with intermediate lags
//! partialled out:
//!
//!   φ_kk = corr(x_t, x_{t−k} | x_{t−1}, …, x_{t−k+1})
//!
//! Computed via the Levinson-Durbin recursion from the autocorrelation
//! function. For an AR(p) process, the PACF cuts off after lag p.
//!
//! Used in: model-order selection for AR(p) models (PACF cutoff
//! pattern), distinguishing AR vs MA structure (MA has cutting ACF,
//! AR has cutting PACF).
//!
//! Pure compute. Companion to `autocorrelation_function`, `arima_111`,
//! `vector_autoregression`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PacfReport {
    pub lags: Vec<usize>,
    pub partial_autocorrelations: Vec<f64>,
    /// 95% Bartlett confidence-band half-width (±this value).
    pub confidence_band: f64,
    pub significant_lags: Vec<usize>,
    pub n_observations: usize,
}

pub fn compute(series: &[f64], max_lag: usize) -> Option<PacfReport> {
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
    // Sample autocorrelations needed by Levinson-Durbin.
    let acf: Vec<f64> = (0..=max_lag)
        .map(|k| {
            let num: f64 = (k..n)
                .map(|t| (series[t] - mean) * (series[t - k] - mean))
                .sum();
            num / denom
        })
        .collect();
    // Levinson-Durbin recursion. φ_kk = PACF at lag k.
    let mut pacf = vec![0.0_f64; max_lag + 1];
    pacf[0] = 1.0;
    if max_lag >= 1 {
        pacf[1] = acf[1];
    }
    let mut phi_prev: Vec<f64> = vec![acf[1]];
    let mut error_prev = 1.0 - acf[1].powi(2);
    for k in 2..=max_lag {
        if error_prev.abs() < 1e-18 {
            break;
        }
        let mut num = acf[k];
        for j in 0..(k - 1) {
            num -= phi_prev[j] * acf[k - 1 - j];
        }
        let phi_kk = num / error_prev;
        pacf[k] = phi_kk;
        // Update φ_k,j for j < k.
        let mut phi_curr = vec![0.0_f64; k];
        for j in 0..(k - 1) {
            phi_curr[j] = phi_prev[j] - phi_kk * phi_prev[k - 2 - j];
        }
        phi_curr[k - 1] = phi_kk;
        error_prev *= 1.0 - phi_kk * phi_kk;
        phi_prev = phi_curr;
    }
    let band = 1.96 / n_f.sqrt();
    let significant: Vec<usize> = pacf
        .iter()
        .enumerate()
        .filter(|(k, v)| *k > 0 && v.abs() > band)
        .map(|(k, _)| k)
        .collect();
    Some(PacfReport {
        lags: (0..=max_lag).collect(),
        partial_autocorrelations: pacf,
        confidence_band: band,
        significant_lags: significant,
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
    fn lag_zero_pacf_is_one() {
        let s: Vec<f64> = (0..50).map(|i| (i as f64 * 0.1).sin() * 5.0).collect();
        let r = compute(&s, 5).unwrap();
        assert!((r.partial_autocorrelations[0] - 1.0).abs() < 1e-12);
    }

    #[test]
    fn ar1_pacf_cuts_off_after_lag_one() {
        // AR(1): PACF should be ~φ at lag 1 and ~0 at all higher lags.
        let mut state: u64 = 11;
        let phi = 0.7_f64;
        let mut s = vec![0.0_f64; 1000];
        for i in 1..1000 {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let eps = ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 0.5;
            s[i] = phi * s[i - 1] + eps;
        }
        let r = compute(&s, 10).unwrap();
        assert!(
            (r.partial_autocorrelations[1] - phi).abs() < 0.05,
            "AR(1) PACF[1] expected ~0.7, got {}",
            r.partial_autocorrelations[1]
        );
        // Higher lags should be near zero.
        let max_higher = r.partial_autocorrelations[2..]
            .iter()
            .cloned()
            .map(f64::abs)
            .fold(0.0_f64, f64::max);
        assert!(
            max_higher < 0.15,
            "AR(1) PACF should cut off after lag 1, max |higher| = {max_higher}"
        );
    }

    #[test]
    fn white_noise_pacf_mostly_inside_band() {
        let mut state: u64 = 99;
        let s: Vec<f64> = (0..500)
            .map(|_| {
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 2.0
            })
            .collect();
        let r = compute(&s, 10).unwrap();
        let outside: usize = (1..=10)
            .filter(|k| r.partial_autocorrelations[*k].abs() > r.confidence_band)
            .count();
        assert!(
            outside <= 3,
            "{outside} PACF lags outside 95% band on white noise: {:?}",
            &r.partial_autocorrelations[1..=10]
        );
    }

    #[test]
    fn output_lengths_match() {
        let s: Vec<f64> = (0..50).map(|i| i as f64).collect();
        let r = compute(&s, 5).unwrap();
        assert_eq!(r.lags.len(), 6);
        assert_eq!(r.partial_autocorrelations.len(), 6);
    }
}
