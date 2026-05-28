//! Granger (1969) causality F-test — "does past Y help predict X?"
//!
//! Two nested regressions over (T - lags) observations:
//!
//!   Restricted:   X_t = α + Σ φ_i · X_{t−i}                     + ε_t
//!   Unrestricted: X_t = α + Σ φ_i · X_{t−i} + Σ ψ_j · Y_{t−j} + u_t
//!
//! F-statistic:
//!   F = ((RSS_r − RSS_u) / lags) / (RSS_u / (n − 2·lags − 1))
//!
//! Under the null "Y does not Granger-cause X", F is F-distributed
//! with (lags, n − 2·lags − 1) degrees of freedom. We do NOT compute
//! the exact p-value (no F distribution in the dep tree); instead
//! we surface the F-statistic and let the caller compare against
//! tabulated critical values (5%: ~ 2.5 for typical n, lags).
//!
//! Pure compute. Test holds in BOTH directions — call twice swapping
//! x and y to test the reverse direction.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GrangerReport {
    pub f_statistic: f64,
    pub rss_restricted: f64,
    pub rss_unrestricted: f64,
    pub n_observations: usize,
    pub lags: usize,
}

pub fn test(x: &[f64], y: &[f64], lags: usize) -> Option<GrangerReport> {
    let n = x.len();
    if y.len() != n || n < 4 * lags + 2 || lags == 0 {
        return None;
    }
    if x.iter().any(|v| !v.is_finite()) || y.iter().any(|v| !v.is_finite()) {
        return None;
    }
    // Build design matrices.
    let start = lags;
    let m = n - start;
    if m < 2 * lags + 2 { return None; }
    let mut y_target = Vec::with_capacity(m);    // dependent variable = X_t
    let intercept = vec![1.0_f64; m];
    let mut x_lags: Vec<Vec<f64>> = (0..lags).map(|_| Vec::with_capacity(m)).collect();
    let mut y_lags: Vec<Vec<f64>> = (0..lags).map(|_| Vec::with_capacity(m)).collect();
    for t in start..n {
        y_target.push(x[t]);
        for i in 0..lags {
            x_lags[i].push(x[t - 1 - i]);
            y_lags[i].push(y[t - 1 - i]);
        }
    }
    let restricted_cols: Vec<Vec<f64>> =
        std::iter::once(intercept.clone()).chain(x_lags.clone()).collect();
    let rss_r = ols_rss(&restricted_cols, &y_target)?;
    let mut unrestricted_cols = Vec::with_capacity(2 * lags + 1);
    unrestricted_cols.append(&mut intercept.clone().iter().map(|x| vec![*x]).collect());
    // Build properly: intercept + x_lags + y_lags.
    let mut unrestricted_cols: Vec<Vec<f64>> = Vec::with_capacity(2 * lags + 1);
    unrestricted_cols.push(intercept);
    unrestricted_cols.extend(x_lags);
    unrestricted_cols.extend(y_lags);
    let rss_u = ols_rss(&unrestricted_cols, &y_target)?;
    let dof_u = m as isize - 2 * lags as isize - 1;
    if dof_u <= 0 { return None; }
    let f_stat = ((rss_r - rss_u) / lags as f64) / (rss_u / dof_u as f64);
    if !f_stat.is_finite() || f_stat.is_nan() { return None; }
    Some(GrangerReport {
        f_statistic: f_stat,
        rss_restricted: rss_r,
        rss_unrestricted: rss_u,
        n_observations: m,
        lags,
    })
}

fn ols_rss(x: &[Vec<f64>], y: &[f64]) -> Option<f64> {
    let p = x.len();
    let n = y.len();
    if p == 0 || n == 0 || x.iter().any(|c| c.len() != n) { return None; }
    let mut xtx = vec![vec![0.0_f64; p]; p];
    let mut xty = vec![0.0_f64; p];
    for i in 0..p {
        for j in 0..p {
            xtx[i][j] = x[i].iter().zip(x[j].iter()).map(|(a, b)| a * b).sum();
        }
        xty[i] = x[i].iter().zip(y.iter()).map(|(a, b)| a * b).sum();
    }
    let mut aug = vec![vec![0.0_f64; p + 1]; p];
    for i in 0..p {
        for j in 0..p { aug[i][j] = xtx[i][j]; }
        aug[i][p] = xty[i];
    }
    // Gauss-Jordan with partial pivoting.
    for i in 0..p {
        let mut pivot = i;
        for r in (i + 1)..p {
            if aug[r][i].abs() > aug[pivot][i].abs() { pivot = r; }
        }
        if aug[pivot][i].abs() < 1e-18 { return None; }
        aug.swap(i, pivot);
        let div = aug[i][i];
        for v in aug[i].iter_mut() { *v /= div; }
        for r in 0..p {
            if r == i { continue; }
            let f = aug[r][i];
            if f == 0.0 { continue; }
            let pivot_row = aug[i].clone();
            for (j, v) in aug[r].iter_mut().enumerate() { *v -= f * pivot_row[j]; }
        }
    }
    let beta: Vec<f64> = (0..p).map(|i| aug[i][p]).collect();
    let mut rss = 0.0_f64;
    for k in 0..n {
        let yh: f64 = (0..p).map(|i| beta[i] * x[i][k]).sum();
        rss += (y[k] - yh).powi(2);
    }
    Some(rss)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dim_mismatch_returns_none() {
        assert!(test(&[1.0; 50], &[1.0; 25], 2).is_none());
    }

    #[test]
    fn too_short_returns_none() {
        assert!(test(&[1.0; 5], &[1.0; 5], 3).is_none());
    }

    #[test]
    fn zero_lags_returns_none() {
        assert!(test(&[1.0; 50], &[1.0; 50], 0).is_none());
    }

    #[test]
    fn nan_inputs_return_none() {
        let mut x = vec![1.0; 50];
        let y = vec![1.0; 50];
        x[10] = f64::NAN;
        assert!(test(&x, &y, 2).is_none());
    }

    #[test]
    fn y_causes_x_yields_large_f_stat() {
        // Build x_t = 0.5 · y_{t−1} + noise — Y strongly Granger-causes X.
        let n = 500;
        let mut state: u64 = 42;
        let mut y = vec![0.0_f64; n];
        for slot in y.iter_mut() {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            *slot = ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 2.0;
        }
        let mut x = vec![0.0_f64; n];
        for i in 1..n {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let noise = ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 0.1;
            x[i] = 0.5 * y[i - 1] + noise;
        }
        let r = test(&x, &y, 2).unwrap();
        assert!(r.f_statistic > 3.0,
            "expected large F (Y causes X), got {}", r.f_statistic);
    }

    #[test]
    fn independent_series_yield_modest_f_stat() {
        let n = 500;
        let mut state: u64 = 999;
        let mut x = vec![0.0; n];
        let mut y = vec![0.0; n];
        for (xs, ys) in x.iter_mut().zip(y.iter_mut()) {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            *xs = (state >> 32) as f64 / u32::MAX as f64 - 0.5;
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            *ys = (state >> 32) as f64 / u32::MAX as f64 - 0.5;
        }
        let r = test(&x, &y, 2).unwrap();
        // Under the null, F has expected value 1 for large n; samples
        // can deviate but the value should not be in the highly-significant
        // tail.
        assert!(r.f_statistic < 10.0,
            "independent series shouldn't show large F, got {}", r.f_statistic);
    }

    #[test]
    fn restricted_rss_at_least_as_large_as_unrestricted() {
        // RSS_r ≥ RSS_u always (nested models — adding regressors can't
        // increase RSS).
        let n = 200;
        let x: Vec<f64> = (0..n).map(|i| (i as f64 * 0.1).sin()).collect();
        let y: Vec<f64> = (0..n).map(|i| (i as f64 * 0.13).cos()).collect();
        let r = test(&x, &y, 3).unwrap();
        assert!(r.rss_restricted >= r.rss_unrestricted - 1e-9);
    }
}
