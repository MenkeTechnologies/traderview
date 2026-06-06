//! Augmented Dickey-Fuller (ADF) unit-root test (standalone).
//!
//! Tests H₀: the series has a unit root (non-stationary) vs
//! H₁: the series is stationary.
//!
//! Regression:
//!   Δy_t = α + γ · y_{t−1} + Σ_{i=1..p} φ_i · Δy_{t−i} + ε_t
//!
//! Test statistic: t-stat on γ. Reject H₀ when t < critical (more
//! negative). Critical values (Fuller 1976, regression with constant):
//!   1%:  −3.43,  5%: −2.86,  10%: −2.57   (large-sample asymptotic)
//!
//! Companion module to `cointegration` (which uses ADF on residuals
//! for Engle-Granger pair testing). This module exposes ADF as a
//! first-class general-purpose stationarity test.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdfSignificance {
    Pct1,
    Pct5,
    Pct10,
    Insignificant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdfReport {
    pub t_statistic: f64,
    pub gamma: f64,
    pub gamma_se: f64,
    pub significance: AdfSignificance,
    pub n_observations: usize,
    pub lags: usize,
}

pub fn test(series: &[f64], lags: usize) -> Option<AdfReport> {
    let n = series.len();
    if n < 3 * lags + 4 {
        return None;
    }
    if series.iter().any(|x| !x.is_finite()) {
        return None;
    }
    let mut diffs = vec![0.0_f64; n];
    for i in 1..n {
        diffs[i] = series[i] - series[i - 1];
    }
    let start = lags + 1;
    if n <= start {
        return None;
    }
    let m = n - start;
    if m < 2 * lags + 2 {
        return None;
    }
    let p_cols = 2 + lags;
    let mut x: Vec<Vec<f64>> = (0..p_cols).map(|_| Vec::with_capacity(m)).collect();
    let mut y_vec: Vec<f64> = Vec::with_capacity(m);
    for i in start..n {
        x[0].push(1.0);
        x[1].push(series[i - 1]);
        for k in 0..lags {
            x[2 + k].push(diffs[i - 1 - k]);
        }
        y_vec.push(diffs[i]);
    }
    let (beta, se) = ols_with_se(&x, &y_vec)?;
    if beta.len() != p_cols || se.len() != p_cols {
        return None;
    }
    let gamma = beta[1];
    let gamma_se = se[1];
    if gamma_se <= 0.0 {
        return None;
    }
    let t_stat = gamma / gamma_se;
    let sig = if t_stat < -3.43 {
        AdfSignificance::Pct1
    } else if t_stat < -2.86 {
        AdfSignificance::Pct5
    } else if t_stat < -2.57 {
        AdfSignificance::Pct10
    } else {
        AdfSignificance::Insignificant
    };
    Some(AdfReport {
        t_statistic: t_stat,
        gamma,
        gamma_se,
        significance: sig,
        n_observations: m,
        lags,
    })
}

fn ols_with_se(x: &[Vec<f64>], y: &[f64]) -> Option<(Vec<f64>, Vec<f64>)> {
    let p = x.len();
    let n = y.len();
    if p == 0 || n == 0 || x.iter().any(|c| c.len() != n) {
        return None;
    }
    let mut xtx = vec![vec![0.0_f64; p]; p];
    let mut xty = vec![0.0_f64; p];
    for i in 0..p {
        for j in 0..p {
            xtx[i][j] = x[i].iter().zip(x[j].iter()).map(|(a, b)| a * b).sum();
        }
        xty[i] = x[i].iter().zip(y.iter()).map(|(a, b)| a * b).sum();
    }
    let mut aug = vec![vec![0.0_f64; 2 * p + 1]; p];
    for i in 0..p {
        for j in 0..p {
            aug[i][j] = xtx[i][j];
            aug[i][p + j] = if i == j { 1.0 } else { 0.0 };
        }
        aug[i][2 * p] = xty[i];
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
    let beta: Vec<f64> = (0..p).map(|i| aug[i][2 * p]).collect();
    let mut residuals = Vec::with_capacity(n);
    for k in 0..n {
        let yhat: f64 = (0..p).map(|i| beta[i] * x[i][k]).sum();
        residuals.push(y[k] - yhat);
    }
    let ss_res: f64 = residuals.iter().map(|r| r * r).sum();
    let dof = (n as isize - p as isize).max(1) as f64;
    let sigma2 = ss_res / dof;
    let mut se = vec![0.0_f64; p];
    for i in 0..p {
        let var = sigma2 * aug[i][p + i];
        se[i] = if var > 0.0 { var.sqrt() } else { 0.0 };
    }
    Some((beta, se))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn too_short_returns_none() {
        assert!(test(&[1.0; 5], 2).is_none());
    }

    #[test]
    fn nan_returns_none() {
        let mut s = vec![1.0; 50];
        s[5] = f64::NAN;
        assert!(test(&s, 1).is_none());
    }

    #[test]
    fn random_walk_fails_to_reject_h0() {
        // Pure random walk has a unit root → ADF should NOT reject (high t).
        let mut state: u64 = 42;
        let n = 500;
        let mut s = vec![0.0_f64; n];
        for i in 1..n {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let u = (state >> 32) as f64 / u32::MAX as f64 - 0.5;
            s[i] = s[i - 1] + u;
        }
        let r = test(&s, 1).unwrap();
        // ADF on random walk: t typically in [-1, 0], rarely below -2.86.
        assert!(matches!(
            r.significance,
            AdfSignificance::Insignificant | AdfSignificance::Pct10
        ));
    }

    #[test]
    fn strongly_mean_reverting_rejects_h0() {
        // AR(1) with φ = 0.3 → highly stationary → ADF strongly rejects.
        let mut state: u64 = 999;
        let n = 500;
        let mut s = vec![0.0_f64; n];
        for i in 1..n {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let u = (state >> 32) as f64 / u32::MAX as f64 - 0.5;
            s[i] = 0.3 * s[i - 1] + u;
        }
        let r = test(&s, 2).unwrap();
        assert!(
            r.t_statistic < -2.86,
            "expected significant rejection, got t={}",
            r.t_statistic
        );
        assert!(!matches!(r.significance, AdfSignificance::Insignificant));
    }

    #[test]
    fn flat_series_collinear_yields_none_or_zero_se() {
        // Constant series → diffs all zero → singular → None.
        let s = vec![100.0; 50];
        let r = test(&s, 1);
        assert!(r.is_none() || r.unwrap().gamma_se == 0.0);
    }

    #[test]
    fn zero_lags_runs_simple_df_test() {
        // Lags=0 → Dickey-Fuller (non-augmented). Just verify it runs.
        let n = 300;
        let mut s = vec![0.0_f64; n];
        let mut state: u64 = 7;
        for i in 1..n {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let u = (state >> 32) as f64 / u32::MAX as f64 - 0.5;
            s[i] = 0.5 * s[i - 1] + u;
        }
        let r = test(&s, 0).unwrap();
        assert!(r.t_statistic.is_finite());
        assert_eq!(r.lags, 0);
    }
}
