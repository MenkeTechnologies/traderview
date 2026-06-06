//! Engle-Granger Two-Step Cointegration Test (Engle & Granger 1987).
//!
//! Tests whether two non-stationary series y_t and x_t share a common
//! stochastic trend (are cointegrated) via:
//!
//!   1. Run OLS y_t = α + β·x_t + ε_t → fitted residuals ê_t
//!   2. ADF test on ê_t (no intercept) → reject H0 of unit root
//!      ⇒ cointegration confirmed
//!
//! If cointegrated, β is the long-run equilibrium relationship and
//! ê_t is the mean-reverting spread useful for pair trading.
//!
//! Critical values for the ADF step use Engle-Granger-adjusted
//! values (more conservative than standard ADF since residuals are
//! pre-estimated):
//!   α = 0.10 → −3.04
//!   α = 0.05 → −3.34
//!   α = 0.01 → −3.90
//!
//! Distinct from existing `cointegration` module (which may use
//! Johansen or other tests); this is the canonical Engle-Granger 2-step.
//!
//! Pure compute. Companion to `cointegration`, `gonzalo_granger_decomposition`,
//! `adf_standalone`, `kpss_test`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EngleGrangerReport {
    pub cointegrating_alpha: f64,
    pub cointegrating_beta: f64,
    pub adf_statistic: f64,
    pub residual_stdev: f64,
    pub reject_no_cointegration_5pct: bool,
    pub reject_no_cointegration_1pct: bool,
    pub n_observations: usize,
    pub adf_lags: usize,
}

pub fn test(y: &[f64], x: &[f64], adf_lags: usize) -> Option<EngleGrangerReport> {
    let n = y.len();
    if n < 30 || x.len() != n {
        return None;
    }
    if y.iter().any(|v| !v.is_finite()) || x.iter().any(|v| !v.is_finite()) {
        return None;
    }
    // Step 1: OLS y = α + β·x.
    let n_f = n as f64;
    let x_mean: f64 = x.iter().sum::<f64>() / n_f;
    let y_mean: f64 = y.iter().sum::<f64>() / n_f;
    let mut sxx = 0.0_f64;
    let mut sxy = 0.0_f64;
    for i in 0..n {
        sxx += (x[i] - x_mean).powi(2);
        sxy += (x[i] - x_mean) * (y[i] - y_mean);
    }
    if sxx <= 0.0 {
        return None;
    }
    let beta = sxy / sxx;
    let alpha = y_mean - beta * x_mean;
    let resid: Vec<f64> = (0..n).map(|i| y[i] - alpha - beta * x[i]).collect();
    let resid_var: f64 = resid.iter().map(|r| r * r).sum::<f64>() / (n_f - 2.0);
    let resid_sd = resid_var.max(0.0).sqrt();
    // Step 2: ADF on residuals (no intercept, optional lags of Δê_t).
    let adf = augmented_dickey_fuller(&resid, adf_lags)?;
    Some(EngleGrangerReport {
        cointegrating_alpha: alpha,
        cointegrating_beta: beta,
        adf_statistic: adf,
        residual_stdev: resid_sd,
        // Engle-Granger critical values (no constant/trend in test reg).
        reject_no_cointegration_5pct: adf < -3.34,
        reject_no_cointegration_1pct: adf < -3.90,
        n_observations: n,
        adf_lags,
    })
}

fn augmented_dickey_fuller(series: &[f64], lags: usize) -> Option<f64> {
    let n = series.len();
    if n < lags + 5 {
        return None;
    }
    // Δε_t = γ · ε_{t-1} + Σ_{j=1..lags} φ_j · Δε_{t-j} + ν_t
    let mut delta = vec![0.0_f64; n];
    for i in 1..n {
        delta[i] = series[i] - series[i - 1];
    }
    let p = 1 + lags;
    let start = lags + 1;
    let n_obs = n - start;
    if n_obs < p + 2 {
        return None;
    }
    let mut xtx = vec![vec![0.0_f64; p]; p];
    let mut xty = vec![0.0_f64; p];
    for t in start..n {
        let mut row = vec![0.0_f64; p];
        row[0] = series[t - 1];
        for j in 1..=lags {
            row[j] = delta[t - j];
        }
        let y = delta[t];
        for j in 0..p {
            xty[j] += row[j] * y;
            for k in 0..p {
                xtx[j][k] += row[j] * row[k];
            }
        }
    }
    let coef = solve_linear(&xtx, &xty)?;
    let mut ssr = 0.0_f64;
    for t in start..n {
        let mut yhat = coef[0] * series[t - 1];
        for j in 1..=lags {
            yhat += coef[j] * delta[t - j];
        }
        ssr += (delta[t] - yhat).powi(2);
    }
    let dof = (n_obs - p) as f64;
    if dof <= 0.0 {
        return None;
    }
    let sigma2 = ssr / dof;
    // SE(γ) = √(σ² · (X'X)⁻¹_{00}).
    let inv00 = invert_cell_zero(&xtx)?;
    let se = (sigma2 * inv00).max(0.0).sqrt();
    if se <= 0.0 {
        return None;
    }
    Some(coef[0] / se)
}

fn solve_linear(m: &[Vec<f64>], y: &[f64]) -> Option<Vec<f64>> {
    let n = m.len();
    if n == 0 || y.len() != n {
        return None;
    }
    let mut aug = vec![vec![0.0_f64; n + 1]; n];
    for (i, row) in aug.iter_mut().enumerate() {
        for (j, slot) in row.iter_mut().enumerate().take(n) {
            *slot = m[i][j];
        }
        row[n] = y[i];
    }
    for i in 0..n {
        let mut pivot = i;
        for r in (i + 1)..n {
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
        for r in 0..n {
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
    Some((0..n).map(|i| aug[i][n]).collect())
}

/// Returns `(X'X)⁻¹\[0\]\[0\]` without computing the full inverse.
fn invert_cell_zero(m: &[Vec<f64>]) -> Option<f64> {
    let n = m.len();
    let mut e0 = vec![0.0_f64; n];
    e0[0] = 1.0;
    let inv_col0 = solve_linear(m, &e0)?;
    Some(inv_col0[0])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn random_walk(n: usize, seed: u64) -> Vec<f64> {
        let mut state = seed;
        let mut v = 0.0_f64;
        (0..n)
            .map(|_| {
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                v += ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 2.0;
                v
            })
            .collect()
    }

    #[test]
    fn too_short_or_mismatched_returns_none() {
        assert!(test(&[1.0; 20], &[1.0; 20], 1).is_none());
        assert!(test(&[1.0; 50], &[1.0; 10], 1).is_none());
    }

    #[test]
    fn nan_returns_none() {
        let y = vec![1.0_f64; 50];
        let mut x = vec![1.0_f64; 50];
        x[5] = f64::NAN;
        assert!(test(&y, &x, 1).is_none());
    }

    #[test]
    fn cointegrated_series_reject_no_cointegration() {
        // y = 2·x + stationary noise; ê = stationary → reject.
        let mut state: u64 = 42;
        let x = random_walk(500, 42);
        let y: Vec<f64> = x
            .iter()
            .map(|xi| {
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let eps = ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 0.5;
                2.0 * xi + eps
            })
            .collect();
        let r = test(&y, &x, 2).unwrap();
        assert!(
            (r.cointegrating_beta - 2.0).abs() < 0.2,
            "β ≈ 2.0 expected, got {}",
            r.cointegrating_beta
        );
        assert!(
            r.reject_no_cointegration_5pct,
            "ADF stat {}, expected to reject no-cointegration",
            r.adf_statistic
        );
    }

    #[test]
    fn independent_random_walks_do_not_reject() {
        let x = random_walk(500, 42);
        let y = random_walk(500, 1234567);
        let r = test(&y, &x, 2).unwrap();
        assert!(
            !r.reject_no_cointegration_1pct,
            "independent RWs shouldn't reject at 1%, ADF = {}",
            r.adf_statistic
        );
    }

    #[test]
    fn coefficients_match_ols() {
        // Perfect-fit y = 3x + 1 would yield zero residuals → ADF NaN.
        // Add small noise so residuals have nonzero variance.
        let mut state: u64 = 99;
        let x: Vec<f64> = (0..50).map(|i| i as f64).collect();
        let y: Vec<f64> = x
            .iter()
            .map(|xi| {
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let eps = ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 0.1;
                3.0 * xi + 1.0 + eps
            })
            .collect();
        let r = test(&y, &x, 0).unwrap();
        assert!((r.cointegrating_beta - 3.0).abs() < 0.05);
        assert!((r.cointegrating_alpha - 1.0).abs() < 1.0);
    }

    #[test]
    fn n_and_lags_reported() {
        let x: Vec<f64> = (0..100).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|xi| 2.0 * xi).collect();
        let r = test(&y, &x, 3);
        if let Some(rep) = r {
            assert_eq!(rep.n_observations, 100);
            assert_eq!(rep.adf_lags, 3);
        }
    }
}
