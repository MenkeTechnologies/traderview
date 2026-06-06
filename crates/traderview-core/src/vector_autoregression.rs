//! Vector Autoregression VAR(p) — equation-by-equation OLS estimation
//! and one-step-ahead forecast.
//!
//!   y_t = c + A_1 · y_{t-1} + … + A_p · y_{t-p} + ε_t
//!
//! For each of the k component series, run OLS with regressors
//! [1, y₁_{t−1}, …, y_k_{t−1}, …, y₁_{t−p}, …, y_k_{t−p}]. The
//! one-step-ahead forecast Y_{T+1} is then computed by stacking the
//! per-equation predictions.
//!
//! Pure compute. Caller supplies series as `time × k` (each row = one
//! observation; row vector of k components).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VarReport {
    /// Per-equation coefficients: rows = equations (k), columns = [intercept, lag1_var1, …, lagp_vark].
    pub coefficients: Vec<Vec<f64>>,
    pub one_step_forecast: Vec<f64>,
    pub residual_variance: Vec<f64>,
    pub n_observations: usize,
    pub n_variables: usize,
    pub lags: usize,
}

pub fn estimate(series: &[Vec<f64>], lags: usize) -> Option<VarReport> {
    let t = series.len();
    if t < lags + 4 || lags == 0 {
        return None;
    }
    let k = series[0].len();
    if k == 0 {
        return None;
    }
    if series.iter().any(|row| row.len() != k) {
        return None;
    }
    if series.iter().any(|row| row.iter().any(|v| !v.is_finite())) {
        return None;
    }
    // Number of regressors per equation: 1 (intercept) + k · p
    let p_regressors = 1 + k * lags;
    let m = t - lags; // effective observations after lag-truncation
    if m < p_regressors + 1 {
        return None;
    }
    // Build the design matrix X (m × p_regressors) — shared across all equations.
    let mut x = vec![vec![0.0_f64; p_regressors]; m];
    for (i, row) in x.iter_mut().enumerate() {
        let obs_idx = lags + i;
        row[0] = 1.0;
        let mut col = 1;
        for lag in 1..=lags {
            let lagged = &series[obs_idx - lag];
            for &val in lagged.iter().take(k) {
                row[col] = val;
                col += 1;
            }
        }
    }
    let mut coefficients = vec![vec![0.0_f64; p_regressors]; k];
    let mut residual_variance = vec![0.0_f64; k];
    for eq in 0..k {
        let y: Vec<f64> = (lags..t).map(|i| series[i][eq]).collect();
        let beta = ols(&x, &y)?;
        // Residual variance.
        let mut ss_res = 0.0_f64;
        for i in 0..m {
            let yhat: f64 = (0..p_regressors).map(|j| beta[j] * x[i][j]).sum();
            ss_res += (y[i] - yhat).powi(2);
        }
        residual_variance[eq] = ss_res / (m as f64 - p_regressors as f64).max(1.0);
        coefficients[eq] = beta;
    }
    // Build the one-step-ahead forecast.
    let mut forecast_x = vec![0.0_f64; p_regressors];
    forecast_x[0] = 1.0;
    let mut col = 1;
    for lag in 1..=lags {
        let row = &series[t - lag];
        for &val in row.iter().take(k) {
            forecast_x[col] = val;
            col += 1;
        }
    }
    let one_step_forecast: Vec<f64> = coefficients
        .iter()
        .map(|beta| (0..p_regressors).map(|j| beta[j] * forecast_x[j]).sum())
        .collect();
    Some(VarReport {
        coefficients,
        one_step_forecast,
        residual_variance,
        n_observations: m,
        n_variables: k,
        lags,
    })
}

fn ols(x: &[Vec<f64>], y: &[f64]) -> Option<Vec<f64>> {
    let p = x.first().map(|r| r.len()).unwrap_or(0);
    let n = y.len();
    if p == 0 || n == 0 || x.iter().any(|c| c.len() != p) {
        return None;
    }
    let mut xtx = vec![vec![0.0_f64; p]; p];
    let mut xty = vec![0.0_f64; p];
    for i in 0..p {
        for j in 0..p {
            xtx[i][j] = x.iter().map(|row| row[i] * row[j]).sum();
        }
        xty[i] = x.iter().zip(y.iter()).map(|(row, yv)| row[i] * yv).sum();
    }
    let mut aug = vec![vec![0.0_f64; p + 1]; p];
    for i in 0..p {
        for j in 0..p {
            aug[i][j] = xtx[i][j];
        }
        aug[i][p] = xty[i];
    }
    for i in 0..p {
        let mut pivot = i;
        for r in (i + 1)..p {
            if aug[r][i].abs() > aug[pivot][i].abs() {
                pivot = r;
            }
        }
        if aug[pivot][i].abs() < 1e-18 {
            return None;
        }
        aug.swap(i, pivot);
        let div = aug[i][i];
        for v in aug[i].iter_mut() {
            *v /= div;
        }
        for r in 0..p {
            if r == i {
                continue;
            }
            let f = aug[r][i];
            if f == 0.0 {
                continue;
            }
            let pivot_row = aug[i].clone();
            for (j, v) in aug[r].iter_mut().enumerate() {
                *v -= f * pivot_row[j];
            }
        }
    }
    Some((0..p).map(|i| aug[i][p]).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_none() {
        assert!(estimate(&[], 1).is_none());
    }

    #[test]
    fn too_short_returns_none() {
        let s = vec![vec![1.0, 2.0]; 3];
        assert!(estimate(&s, 2).is_none());
    }

    #[test]
    fn zero_lags_returns_none() {
        let s = vec![vec![1.0, 2.0]; 50];
        assert!(estimate(&s, 0).is_none());
    }

    #[test]
    fn dim_mismatch_returns_none() {
        let s = vec![vec![1.0, 2.0], vec![1.0], vec![1.0, 2.0]];
        assert!(estimate(&s, 1).is_none());
    }

    #[test]
    fn nan_input_returns_none() {
        let mut s: Vec<Vec<f64>> = (0..50).map(|i| vec![i as f64, (i as f64).sin()]).collect();
        s[10][1] = f64::NAN;
        assert!(estimate(&s, 1).is_none());
    }

    #[test]
    fn synthetic_ar1_var_recovers_coefficients() {
        // Build a true 2-variable VAR(1): y_t = c + A · y_{t-1} + ε.
        // c = [0.1, -0.05], A = [[0.6, 0.2], [0.1, 0.5]].
        let n = 1_000;
        let mut state: u64 = 42;
        let mut y = vec![vec![0.0_f64, 0.0]; n];
        for t in 1..n {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let e1 = ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 0.02;
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let e2 = ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 0.02;
            y[t][0] = 0.1 + 0.6 * y[t - 1][0] + 0.2 * y[t - 1][1] + e1;
            y[t][1] = -0.05 + 0.1 * y[t - 1][0] + 0.5 * y[t - 1][1] + e2;
        }
        let r = estimate(&y, 1).unwrap();
        assert_eq!(r.n_variables, 2);
        assert_eq!(r.lags, 1);
        // Coefficient layout: [intercept, lag1_var0, lag1_var1] per equation.
        let eq0 = &r.coefficients[0];
        let eq1 = &r.coefficients[1];
        assert!((eq0[0] - 0.1).abs() < 0.02);
        assert!((eq0[1] - 0.6).abs() < 0.05);
        assert!((eq0[2] - 0.2).abs() < 0.05);
        assert!((eq1[0] - (-0.05)).abs() < 0.02);
        assert!((eq1[1] - 0.1).abs() < 0.05);
        assert!((eq1[2] - 0.5).abs() < 0.05);
    }

    #[test]
    fn forecast_dimension_matches_n_variables() {
        let n = 100;
        let s: Vec<Vec<f64>> = (0..n)
            .map(|i| {
                vec![
                    (i as f64 * 0.1).sin(),
                    (i as f64 * 0.13).cos(),
                    (i as f64 * 0.07).sin(),
                ]
            })
            .collect();
        let r = estimate(&s, 2).unwrap();
        assert_eq!(r.one_step_forecast.len(), 3);
        assert_eq!(r.coefficients.len(), 3);
        assert_eq!(r.coefficients[0].len(), 1 + 3 * 2);
    }

    #[test]
    fn residual_variance_nonnegative() {
        let n = 100;
        let s: Vec<Vec<f64>> = (0..n)
            .map(|i| vec![(i as f64 * 0.1).sin(), (i as f64 * 0.13).cos()])
            .collect();
        let r = estimate(&s, 1).unwrap();
        for v in &r.residual_variance {
            assert!(*v >= 0.0);
        }
    }
}
