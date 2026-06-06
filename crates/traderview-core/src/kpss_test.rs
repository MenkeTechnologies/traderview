//! KPSS Stationarity Test — Kwiatkowski, Phillips, Schmidt, Shin (1992).
//!
//! Tests the null hypothesis that a series is (level- or trend-)
//! stationary against the alternative of a unit root. Note the
//! direction: KPSS H0 = stationary, ADF H0 = unit root. Joint use
//! disambiguates between the two:
//!
//!   ADF rejects + KPSS doesn't reject → stationary
//!   ADF doesn't reject + KPSS rejects → unit root
//!
//! Test statistic (level-stationarity variant):
//!
//!   e_t = y_t − μ̂
//!   S_t = Σ_{i=1..t} e_i
//!   KPSS = (1/n²) · Σ_t S_t² / s²(l)
//!   s²(l) = (1/n) · Σ e_t² + (2/n) · Σ_{s=1..l} w(s,l) · Σ_t e_t · e_{t−s}
//!
//! where w(s, l) = 1 − s/(l+1) is the Bartlett kernel and l is the
//! truncation lag (≈ √n for empirical use).
//!
//! Critical values for level-stationarity (Kwiatkowski et al. 1992):
//!   α = 0.10 → 0.347
//!   α = 0.05 → 0.463
//!   α = 0.01 → 0.739
//!
//! Pure compute. Companion to `adf_standalone`, `cusum`, `ljung_box`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KpssReport {
    pub kpss_statistic: f64,
    pub long_run_variance: f64,
    pub truncation_lag: usize,
    pub reject_at_5pct: bool,
    pub reject_at_1pct: bool,
    pub n_observations: usize,
}

pub fn test(series: &[f64], truncation_lag: Option<usize>) -> Option<KpssReport> {
    let n = series.len();
    if n < 20 {
        return None;
    }
    if series.iter().any(|x| !x.is_finite()) {
        return None;
    }
    let n_f = n as f64;
    let mean: f64 = series.iter().sum::<f64>() / n_f;
    let e: Vec<f64> = series.iter().map(|x| x - mean).collect();
    // Default truncation lag: floor(4 · (n/100)^(1/4)) per Kwiatkowski-Schmidt-Shin.
    let l = truncation_lag
        .unwrap_or_else(|| (4.0 * (n_f / 100.0).powf(0.25)).floor() as usize)
        .max(1)
        .min(n / 2);
    let gamma0: f64 = e.iter().map(|x| x * x).sum::<f64>() / n_f;
    let mut s2 = gamma0;
    for s in 1..=l {
        let w = 1.0 - s as f64 / (l as f64 + 1.0);
        let gamma_s: f64 = (s..n).map(|t| e[t] * e[t - s]).sum::<f64>() / n_f;
        s2 += 2.0 * w * gamma_s;
    }
    if s2 <= 0.0 {
        return None;
    }
    let mut cum = 0.0_f64;
    let mut sum_sq = 0.0_f64;
    for et in &e {
        cum += et;
        sum_sq += cum * cum;
    }
    let kpss = sum_sq / (n_f * n_f * s2);
    Some(KpssReport {
        kpss_statistic: kpss,
        long_run_variance: s2,
        truncation_lag: l,
        reject_at_5pct: kpss > 0.463,
        reject_at_1pct: kpss > 0.739,
        n_observations: n,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn too_short_returns_none() {
        assert!(test(&[1.0_f64; 10], None).is_none());
    }

    #[test]
    fn nan_returns_none() {
        let mut s = vec![1.0_f64; 50];
        s[10] = f64::NAN;
        assert!(test(&s, None).is_none());
    }

    #[test]
    fn flat_returns_none() {
        // Flat series → long-run variance = 0 → None.
        assert!(test(&[1.0_f64; 100], None).is_none());
    }

    #[test]
    fn iid_noise_does_not_reject_stationarity() {
        let mut state: u64 = 42;
        let s: Vec<f64> = (0..500)
            .map(|_| {
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 2.0
            })
            .collect();
        let r = test(&s, None).unwrap();
        assert!(
            !r.reject_at_5pct,
            "iid noise shouldn't reject KPSS (stationary), stat = {}",
            r.kpss_statistic
        );
    }

    #[test]
    fn random_walk_rejects_stationarity() {
        // Cumulative sum of iid → unit root → KPSS rejects.
        let mut state: u64 = 11;
        let mut s = vec![0.0_f64; 500];
        for i in 1..500 {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let step = ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 2.0;
            s[i] = s[i - 1] + step;
        }
        let r = test(&s, None).unwrap();
        assert!(
            r.reject_at_5pct,
            "random walk should reject stationarity, stat = {}",
            r.kpss_statistic
        );
    }

    #[test]
    fn custom_truncation_lag_used() {
        let mut state: u64 = 99;
        let s: Vec<f64> = (0..100)
            .map(|_| {
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                (state >> 32) as f64 / u32::MAX as f64 - 0.5
            })
            .collect();
        let r = test(&s, Some(7)).unwrap();
        assert_eq!(r.truncation_lag, 7);
    }

    #[test]
    fn n_observations_reported() {
        let s: Vec<f64> = (0..100).map(|i| (i as f64 * 0.1).sin()).collect();
        let r = test(&s, None).unwrap();
        assert_eq!(r.n_observations, 100);
    }
}
