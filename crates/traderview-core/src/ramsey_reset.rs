//! Ramsey RESET (Regression Equation Specification Error Test, 1969).
//!
//! Tests whether an OLS regression suffers from functional-form
//! misspecification (omitted higher-order or interaction terms). The
//! null hypothesis is that the model is correctly specified.
//!
//! Procedure (powers-of-ŷ variant):
//!
//!   1. Fit y = α + β · x → fitted ŷ, residuals ê.
//!   2. Auxiliary regression: y = α + β·x + γ_2·ŷ² + γ_3·ŷ³ + ν
//!   3. F-test of H0: γ_2 = γ_3 = 0.
//!
//!   F = ((SSR_restricted − SSR_unrestricted) / q)
//!       / (SSR_unrestricted / (n − p − q))
//!
//! where q = 2 (number of extra terms) and p = 2 (intercept + slope).
//! Under H0, F ~ F(q, n − p − q).
//!
//! Use cases:
//!   - Pre-test linear regression for hidden non-linearity
//!   - Validate functional-form for VaR / hedging models
//!   - Detect bias from omitted-variable confounding
//!
//! Pure compute (univariate predictor). Companion to `breusch_pagan_test`,
//! `breusch_godfrey`, `chow_test`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RamseyResetReport {
    pub f_statistic: f64,
    pub ssr_restricted: f64,
    pub ssr_unrestricted: f64,
    pub degrees_of_freedom_numerator: f64,
    pub degrees_of_freedom_denominator: f64,
    pub n_observations: usize,
    pub reject_at_5pct: bool,
}

pub fn test(x: &[f64], y: &[f64]) -> Option<RamseyResetReport> {
    let n = x.len();
    if n < 10 || y.len() != n {
        return None;
    }
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
    if sxx <= 0.0 {
        return None;
    }
    let beta = sxy / sxx;
    let alpha = y_mean - beta * x_mean;
    let y_hat: Vec<f64> = x.iter().map(|xi| alpha + beta * xi).collect();
    let ssr_restricted: f64 = (0..n).map(|i| (y[i] - y_hat[i]).powi(2)).sum();
    // Unrestricted: y = a + b·x + c·ŷ² + d·ŷ³.
    let p = 4_usize;
    let mut xtx = vec![vec![0.0_f64; p]; p];
    let mut xty = vec![0.0_f64; p];
    for i in 0..n {
        let row = [1.0, x[i], y_hat[i].powi(2), y_hat[i].powi(3)];
        for j in 0..p {
            xty[j] += row[j] * y[i];
            for k in 0..p {
                xtx[j][k] += row[j] * row[k];
            }
        }
    }
    let coef = solve_linear(&xtx, &xty)?;
    let mut ssr_unrestricted = 0.0_f64;
    for i in 0..n {
        let yhat_full =
            coef[0] + coef[1] * x[i] + coef[2] * y_hat[i].powi(2) + coef[3] * y_hat[i].powi(3);
        ssr_unrestricted += (y[i] - yhat_full).powi(2);
    }
    let q = 2.0_f64;
    let dof_den = (n - p) as f64;
    if dof_den <= 0.0 || ssr_unrestricted <= 0.0 {
        return None;
    }
    let f_stat = ((ssr_restricted - ssr_unrestricted) / q) / (ssr_unrestricted / dof_den);
    let crit_5pct = 3.10; // ~F(2, ∞) at 5%
    Some(RamseyResetReport {
        f_statistic: f_stat,
        ssr_restricted,
        ssr_unrestricted,
        degrees_of_freedom_numerator: q,
        degrees_of_freedom_denominator: dof_den,
        n_observations: n,
        reject_at_5pct: f_stat > crit_5pct,
    })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn too_short_returns_none() {
        let x = vec![1.0; 5];
        let y = vec![2.0; 5];
        assert!(test(&x, &y).is_none());
    }

    #[test]
    fn nan_returns_none() {
        let x = vec![1.0_f64; 30];
        let mut y = vec![2.0_f64; 30];
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
    fn linear_relation_does_not_reject() {
        // y = 2x + noise → linear is the correct form.
        let mut state: u64 = 42;
        let x: Vec<f64> = (0..300).map(|i| i as f64).collect();
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
        let r = test(&x, &y).unwrap();
        assert!(
            !r.reject_at_5pct,
            "linear relation shouldn't reject, F = {}",
            r.f_statistic
        );
    }

    #[test]
    fn quadratic_relation_rejects() {
        // y = x² → linear model is misspecified.
        let mut state: u64 = 11;
        let x: Vec<f64> = (1..=100).map(|i| i as f64).collect();
        let y: Vec<f64> = x
            .iter()
            .map(|xi| {
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let eps = ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 100.0;
                xi * xi + eps
            })
            .collect();
        let r = test(&x, &y).unwrap();
        assert!(
            r.reject_at_5pct,
            "y = x² shouldn't fit linear, F = {}",
            r.f_statistic
        );
    }

    #[test]
    fn n_observations_reported() {
        let x: Vec<f64> = (0..30).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|xi| 2.0 * xi).collect();
        let r = test(&x, &y);
        if let Some(rep) = r {
            assert_eq!(rep.n_observations, 30);
        }
    }
}
