//! Jarque-Bera Normality Test (Jarque & Bera 1980).
//!
//! Tests the null hypothesis that the sample has skewness 0 and
//! excess kurtosis 0 (the moment characterization of a normal
//! distribution):
//!
//!   JB = n/6 · (S² + (K − 3)²/4)
//!
//! Under H0 (normality), JB ~ χ²(2) asymptotically.
//!
//! Critical values:
//!   - α = 0.10 → χ²(2) = 4.605
//!   - α = 0.05 → χ²(2) = 5.991
//!   - α = 0.01 → χ²(2) = 9.210
//!
//! Asymptotic p-value: p = exp(−JB / 2) (using χ²(2) = Exp(1/2)).
//!
//! Use cases:
//!   - Residual diagnostic for fitted models (regression, GARCH, ARIMA)
//!   - Pre-test for parametric VaR validity
//!   - Strategy return-distribution sanity check
//!
//! Pure compute. Companion to `cornish_fisher`, `realized_higher_moments`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JarqueBeraReport {
    pub statistic: f64,
    pub skewness: f64,
    pub excess_kurtosis: f64,
    pub p_value: f64,
    pub reject_at_5pct: bool,
    pub reject_at_1pct: bool,
    pub n_observations: usize,
}

pub fn test(sample: &[f64]) -> Option<JarqueBeraReport> {
    if sample.len() < 8 {
        return None;
    }
    if sample.iter().any(|x| !x.is_finite()) {
        return None;
    }
    let n = sample.len();
    let n_f = n as f64;
    let mean: f64 = sample.iter().sum::<f64>() / n_f;
    let m2: f64 = sample.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n_f;
    let m3: f64 = sample.iter().map(|x| (x - mean).powi(3)).sum::<f64>() / n_f;
    let m4: f64 = sample.iter().map(|x| (x - mean).powi(4)).sum::<f64>() / n_f;
    if m2 <= 0.0 {
        return None;
    }
    let sd = m2.sqrt();
    let skewness = m3 / sd.powi(3);
    let kurt = m4 / m2.powi(2);
    let excess_k = kurt - 3.0;
    let jb = n_f / 6.0 * (skewness.powi(2) + excess_k.powi(2) / 4.0);
    let p_value = (-jb / 2.0).exp().clamp(0.0, 1.0);
    Some(JarqueBeraReport {
        statistic: jb,
        skewness,
        excess_kurtosis: excess_k,
        p_value,
        reject_at_5pct: jb > 5.991,
        reject_at_1pct: jb > 9.210,
        n_observations: n,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn box_muller(n: usize, seed: u64) -> Vec<f64> {
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
                (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
            })
            .collect()
    }

    #[test]
    fn too_short_returns_none() {
        assert!(test(&[0.0; 5]).is_none());
    }

    #[test]
    fn nan_input_returns_none() {
        assert!(test(&[0.0, f64::NAN, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0]).is_none());
    }

    #[test]
    fn flat_input_returns_none() {
        assert!(test(&[1.0; 50]).is_none());
    }

    #[test]
    fn normal_sample_does_not_reject() {
        let s = box_muller(5000, 42);
        let r = test(&s).unwrap();
        // Under H0 with n=5000, we should rarely reject at 1%.
        assert!(
            !r.reject_at_1pct,
            "Gaussian sample shouldn't reject at 1%, JB={}, p={}",
            r.statistic, r.p_value
        );
    }

    #[test]
    fn left_skewed_sample_rejects() {
        // Heavy left tail: chi-squared-1 negated.
        let z = box_muller(2000, 7);
        let s: Vec<f64> = z.iter().map(|x| -(x * x)).collect();
        let r = test(&s).unwrap();
        assert!(
            r.reject_at_1pct,
            "skewed sample should reject at 1%, JB={}, skew={}",
            r.statistic, r.skewness
        );
        assert!(r.skewness < 0.0);
    }

    #[test]
    fn heavy_tailed_sample_rejects() {
        // Gaussian mixture with a heavy contamination component: 90%
        // N(0,1) + 10% N(0,25). Produces clear excess kurtosis without
        // the undefined-moment pitfalls of t(3).
        let mut state: u64 = 1234567;
        let s: Vec<f64> = (0..3000)
            .map(|_| {
                let u = {
                    state = state
                        .wrapping_mul(6364136223846793005)
                        .wrapping_add(1442695040888963407);
                    (state >> 32) as f64 / u32::MAX as f64
                };
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let u1 = ((state >> 32) as f64 / u32::MAX as f64).max(1e-12);
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let u2 = (state >> 32) as f64 / u32::MAX as f64;
                let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
                if u < 0.1 {
                    z * 5.0
                } else {
                    z
                }
            })
            .collect();
        let r = test(&s).unwrap();
        assert!(
            r.reject_at_1pct,
            "mixture should reject normality, JB={}, kurt={}",
            r.statistic, r.excess_kurtosis
        );
        assert!(r.excess_kurtosis > 1.0);
    }

    #[test]
    fn p_value_in_unit_range() {
        let s = box_muller(500, 99);
        let r = test(&s).unwrap();
        assert!((0.0..=1.0).contains(&r.p_value));
    }

    #[test]
    fn n_reported_correctly() {
        let s = box_muller(100, 3);
        let r = test(&s).unwrap();
        assert_eq!(r.n_observations, 100);
    }
}
