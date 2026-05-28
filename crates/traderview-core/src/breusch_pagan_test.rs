//! Breusch-Pagan Test for Heteroskedasticity (Breusch & Pagan 1979).
//!
//! Tests whether OLS residual variance depends on the regressors.
//! Procedure:
//!
//!   1. Fit y = α + β · x → residuals ê_t.
//!   2. Auxiliary regression: ê_t² = γ_0 + γ_1 · x_t + u_t.
//!   3. LM = n · R²_aux ~ χ²(p) under H0 (homoskedasticity).
//!
//! For univariate x, p = 1 → critical value 3.841 at 5%.
//!
//! Use cases:
//!   - Diagnose OLS residuals before computing standard errors
//!   - Decide whether to use heteroskedasticity-robust SEs (White)
//!   - Detect volatility-clustering proxy in residuals
//!
//! Pure compute (univariate predictor only). Companion to
//! `breusch_godfrey`, `arch_lm_test`, `ljung_box`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BreuschPaganReport {
    pub lm_statistic: f64,
    pub p_value: f64,
    pub r_squared_auxiliary: f64,
    pub n_observations: usize,
    pub reject_at_5pct: bool,
    pub reject_at_1pct: bool,
}

pub fn test(x: &[f64], y: &[f64]) -> Option<BreuschPaganReport> {
    let n = x.len();
    if n < 10 || y.len() != n { return None; }
    if x.iter().any(|v| !v.is_finite()) || y.iter().any(|v| !v.is_finite()) {
        return None;
    }
    let n_f = n as f64;
    let x_mean: f64 = x.iter().sum::<f64>() / n_f;
    let y_mean: f64 = y.iter().sum::<f64>() / n_f;
    let mut sxx = 0.0_f64;
    let mut sxy = 0.0_f64;
    for i in 0..n {
        sxx += (x[i] - x_mean).powi(2);
        sxy += (x[i] - x_mean) * (y[i] - y_mean);
    }
    if sxx <= 0.0 { return None; }
    let beta = sxy / sxx;
    let alpha = y_mean - beta * x_mean;
    let resid_sq: Vec<f64> = (0..n).map(|i| (y[i] - alpha - beta * x[i]).powi(2)).collect();
    // Auxiliary regression: ê² on (1, x).
    let rs_mean: f64 = resid_sq.iter().sum::<f64>() / n_f;
    let mut s_xx_aux = 0.0_f64;
    let mut s_xy_aux = 0.0_f64;
    for i in 0..n {
        s_xx_aux += (x[i] - x_mean).powi(2);
        s_xy_aux += (x[i] - x_mean) * (resid_sq[i] - rs_mean);
    }
    if s_xx_aux <= 0.0 { return None; }
    let gamma1 = s_xy_aux / s_xx_aux;
    let gamma0 = rs_mean - gamma1 * x_mean;
    let tss: f64 = resid_sq.iter().map(|r| (r - rs_mean).powi(2)).sum();
    let ssr: f64 = (0..n).map(|i| (resid_sq[i] - gamma0 - gamma1 * x[i]).powi(2)).sum();
    let r_sq = if tss > 1e-18 { 1.0 - ssr / tss } else { 0.0 };
    let lm = n_f * r_sq.max(0.0);
    let p_value = chi_squared_upper_tail(lm, 1.0);
    Some(BreuschPaganReport {
        lm_statistic: lm,
        p_value,
        r_squared_auxiliary: r_sq,
        n_observations: n,
        reject_at_5pct: lm > 3.841,
        reject_at_1pct: lm > 6.635,
    })
}

fn chi_squared_upper_tail(x: f64, k: f64) -> f64 {
    if x <= 0.0 || k <= 0.0 { return 1.0; }
    // Wilson-Hilferty cube-root approximation, ~3% accuracy for k ≥ 1.
    let z = ((x / k).powf(1.0 / 3.0) - (1.0 - 2.0 / (9.0 * k))) / (2.0 / (9.0 * k)).sqrt();
    1.0 - standard_normal_cdf(z)
}

fn standard_normal_cdf(z: f64) -> f64 {
    0.5 * (1.0 + erf(z / std::f64::consts::SQRT_2))
}

fn erf(x: f64) -> f64 {
    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs();
    let t = 1.0 / (1.0 + 0.327_591_1 * x);
    let y = 1.0 - (((((1.061_405_429 * t - 1.453_152_027) * t)
        + 1.421_413_741) * t - 0.284_496_736) * t + 0.254_829_592) * t * (-x * x).exp();
    sign * y
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn too_short_returns_none() {
        let x = vec![1.0; 5];
        let y = vec![1.0; 5];
        assert!(test(&x, &y).is_none());
    }

    #[test]
    fn mismatched_returns_none() {
        let x = vec![1.0; 20];
        let y = vec![1.0; 10];
        assert!(test(&x, &y).is_none());
    }

    #[test]
    fn nan_returns_none() {
        let x = vec![1.0_f64; 30];
        let mut y = vec![1.0_f64; 30];
        y[5] = f64::NAN;
        assert!(test(&x, &y).is_none());
    }

    #[test]
    fn flat_predictor_returns_none() {
        let x = vec![1.0_f64; 30];
        let y: Vec<f64> = (0..30).map(|i| i as f64).collect();
        assert!(test(&x, &y).is_none());
    }

    #[test]
    fn homoskedastic_does_not_reject() {
        // Residuals iid with constant variance.
        let mut state: u64 = 42;
        let x: Vec<f64> = (0..300).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|xi| {
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let eps = ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 1.0;
            2.0 * xi + eps
        }).collect();
        let r = test(&x, &y).unwrap();
        assert!(!r.reject_at_5pct,
            "homoskedastic shouldn't reject, LM = {}, p = {}", r.lm_statistic, r.p_value);
    }

    #[test]
    fn variance_increasing_in_x_rejects() {
        // Variance proportional to x → strong heteroskedasticity.
        let mut state: u64 = 11;
        let x: Vec<f64> = (1..=300).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|xi| {
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let eps = ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * (xi / 30.0);
            2.0 * xi + eps
        }).collect();
        let r = test(&x, &y).unwrap();
        assert!(r.reject_at_5pct,
            "heteroskedastic should reject, LM = {}, p = {}", r.lm_statistic, r.p_value);
    }

    #[test]
    fn p_value_in_unit_range() {
        let mut state: u64 = 99;
        let x: Vec<f64> = (0..100).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|xi| {
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let eps = ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 0.5;
            2.0 * xi + eps
        }).collect();
        let r = test(&x, &y).unwrap();
        assert!((0.0..=1.0).contains(&r.p_value));
    }
}
