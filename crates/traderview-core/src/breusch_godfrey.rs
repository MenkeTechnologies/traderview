//! Breusch-Godfrey Serial Correlation LM Test (Breusch 1978, Godfrey 1978).
//!
//! Tests whether OLS residuals exhibit serial correlation up to order p,
//! allowing for lagged dependent variables in the regression (unlike
//! the Durbin-Watson test which is biased when regressors include
//! lags).
//!
//! Procedure (after fitting y = α + β·x + ε):
//!
//!   1. Compute residuals ε̂_t.
//!   2. Regress ε̂_t on (intercept, x_t, ε̂_{t-1}, …, ε̂_{t-p}).
//!   3. LM = n · R² ~ χ²(p) under H0 (no serial correlation).
//!
//! Use cases:
//!   - Diagnostic for regression residuals after fitting α + β·x models
//!   - Validate IID assumption for VaR-style risk models
//!   - Catch missing-lag-structure in pair-trading mean-reversion fits
//!
//! Pure compute. Univariate predictor only (intercept + slope on x).
//! For multivariate diagnostics, run separately on residual series.
//!
//! Companion to `ljung_box`, `arch_lm_test`, `cusum`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BreuschGodfreyReport {
    pub lm_statistic: f64,
    pub p_value: f64,
    pub r_squared_auxiliary: f64,
    pub lag_order: usize,
    pub n_observations: usize,
    pub reject_at_5pct: bool,
}

pub fn test(
    x: &[f64],
    y: &[f64],
    lag_order: usize,
) -> Option<BreuschGodfreyReport> {
    let n = x.len();
    if n < lag_order + 8 || y.len() != n || lag_order == 0 { return None; }
    if x.iter().any(|v| !v.is_finite()) || y.iter().any(|v| !v.is_finite()) {
        return None;
    }
    // Step 1: OLS of y on (1, x).
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
    let resid: Vec<f64> = (0..n).map(|i| y[i] - alpha - beta * x[i]).collect();
    // Step 2: regress resid_t on (1, x_t, resid_{t-1}, …, resid_{t-p}).
    let p = 2 + lag_order;    // intercept + x + p lagged residuals
    let n_aux = n - lag_order;
    if n_aux < p + 2 { return None; }
    let mut xtx = vec![vec![0.0_f64; p]; p];
    let mut xty = vec![0.0_f64; p];
    let mut sum_y = 0.0_f64;
    let mut sum_y_sq = 0.0_f64;
    for t in lag_order..n {
        let mut row = vec![0.0_f64; p];
        row[0] = 1.0;
        row[1] = x[t];
        for l in 0..lag_order { row[2 + l] = resid[t - 1 - l]; }
        let yt = resid[t];
        sum_y += yt;
        sum_y_sq += yt * yt;
        for j in 0..p {
            xty[j] += row[j] * yt;
            for k in 0..p { xtx[j][k] += row[j] * row[k]; }
        }
    }
    let coef = solve_linear(&xtx, &xty)?;
    let mean_y = sum_y / n_aux as f64;
    let tss = sum_y_sq - n_aux as f64 * mean_y.powi(2);
    let mut ssr = 0.0_f64;
    for t in lag_order..n {
        let mut yhat = coef[0] + coef[1] * x[t];
        for l in 0..lag_order { yhat += coef[2 + l] * resid[t - 1 - l]; }
        ssr += (resid[t] - yhat).powi(2);
    }
    let r_sq = if tss > 1e-18 { 1.0 - ssr / tss } else { 0.0 };
    let lm = n_aux as f64 * r_sq.max(0.0);
    // χ²(p) asymptotic p-value via series approximation (sufficient for
    // small p typical of B-G: p = 1..10).
    let p_value = chi_squared_upper_tail(lm, lag_order as f64);
    // 5% critical value of χ²(p).
    let crit = chi_squared_5pct_critical(lag_order);
    Some(BreuschGodfreyReport {
        lm_statistic: lm,
        p_value,
        r_squared_auxiliary: r_sq,
        lag_order,
        n_observations: n,
        reject_at_5pct: lm > crit,
    })
}

fn solve_linear(m: &[Vec<f64>], y: &[f64]) -> Option<Vec<f64>> {
    let n = m.len();
    if n == 0 || y.len() != n { return None; }
    let mut aug = vec![vec![0.0_f64; n + 1]; n];
    for (i, row) in aug.iter_mut().enumerate() {
        for (j, slot) in row.iter_mut().enumerate().take(n) { *slot = m[i][j]; }
        row[n] = y[i];
    }
    for i in 0..n {
        let mut pivot = i;
        for r in (i + 1)..n {
            if aug[r][i].abs() > aug[pivot][i].abs() { pivot = r; }
        }
        if aug[pivot][i].abs() < 1e-18 { return None; }
        aug.swap(i, pivot);
        let div = aug[i][i];
        for v in aug[i].iter_mut() { *v /= div; }
        for r in 0..n {
            if r == i { continue; }
            let f = aug[r][i];
            if f == 0.0 { continue; }
            let pivot_row = aug[i].clone();
            for (j, v) in aug[r].iter_mut().enumerate() { *v -= f * pivot_row[j]; }
        }
    }
    Some((0..n).map(|i| aug[i][n]).collect())
}

fn chi_squared_upper_tail(x: f64, k: f64) -> f64 {
    if x <= 0.0 || k <= 0.0 { return 1.0; }
    // 1 - F(x; k) via Wilson-Hilferty approximation; good to ~3% for k ≥ 1.
    let z = ((x / k).powf(1.0 / 3.0) - (1.0 - 2.0 / (9.0 * k))) / (2.0 / (9.0 * k)).sqrt();
    1.0 - standard_normal_cdf(z)
}

fn chi_squared_5pct_critical(k: usize) -> f64 {
    match k {
        1 => 3.841, 2 => 5.991, 3 => 7.815, 4 => 9.488, 5 => 11.070,
        6 => 12.592, 7 => 14.067, 8 => 15.507, 9 => 16.919, 10 => 18.307,
        _ => (k as f64) + 2.0 * ((2.0 * k as f64).sqrt()),    // large-k approx
    }
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
        assert!(test(&x, &y, 2).is_none());
    }

    #[test]
    fn zero_lag_returns_none() {
        let x: Vec<f64> = (0..30).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|xi| 2.0 * xi).collect();
        assert!(test(&x, &y, 0).is_none());
    }

    #[test]
    fn mismatched_returns_none() {
        let x: Vec<f64> = (0..30).map(|i| i as f64).collect();
        let y = vec![1.0; 10];
        assert!(test(&x, &y, 2).is_none());
    }

    #[test]
    fn nan_returns_none() {
        let x: Vec<f64> = (0..30).map(|i| i as f64).collect();
        let mut y: Vec<f64> = x.iter().map(|xi| 2.0 * xi).collect();
        y[10] = f64::NAN;
        assert!(test(&x, &y, 2).is_none());
    }

    #[test]
    fn iid_residuals_do_not_reject() {
        let mut state: u64 = 42;
        let x: Vec<f64> = (0..200).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|xi| {
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let eps = ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 0.5;
            2.0 * xi + eps
        }).collect();
        let r = test(&x, &y, 4).unwrap();
        assert!(!r.reject_at_5pct,
            "IID residuals shouldn't reject, LM = {}, p = {}", r.lm_statistic, r.p_value);
    }

    #[test]
    fn ar1_residuals_reject() {
        // y = 2x + e, where e_t = 0.8·e_{t-1} + η (strong serial correlation).
        let mut state: u64 = 11;
        let x: Vec<f64> = (0..300).map(|i| i as f64).collect();
        let mut e = vec![0.0_f64; 300];
        for i in 1..300 {
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let eta = ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 5.0;
            e[i] = 0.8 * e[i - 1] + eta;
        }
        let y: Vec<f64> = x.iter().zip(e.iter()).map(|(xi, ei)| 2.0 * xi + ei).collect();
        let r = test(&x, &y, 2).unwrap();
        assert!(r.reject_at_5pct,
            "AR(1) residuals should reject, LM = {}", r.lm_statistic);
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
        let r = test(&x, &y, 3).unwrap();
        assert!((0.0..=1.0).contains(&r.p_value));
    }
}
