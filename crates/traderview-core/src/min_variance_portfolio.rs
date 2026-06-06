//! Markowitz minimum-variance and maximum-Sharpe (tangency) portfolios
//! — closed-form weights from the inverse covariance matrix.
//!
//!   min-variance:  w = Σ⁻¹·1 / (1ᵀ·Σ⁻¹·1)
//!   tangency:      w = Σ⁻¹·μ / (1ᵀ·Σ⁻¹·μ)
//!
//! where μ = expected-excess-return vector. The tangency portfolio is
//! the highest-Sharpe combination on the efficient frontier; min-
//! variance is the lowest-variance feasible portfolio.
//!
//! These are the canonical optimization-free MVO weights — valid when
//! short-selling is allowed. For long-only constraints use a quadratic
//! programmer (out of scope).
//!
//! Pure compute. Companion to `risk_parity_weights`, `hierarchical_risk_parity`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MvReport {
    pub min_variance_weights: Vec<f64>,
    pub min_variance_portfolio_volatility: f64,
    pub tangency_weights: Vec<f64>,
    pub tangency_expected_return: f64,
    pub tangency_volatility: f64,
    pub tangency_sharpe: f64,
}

pub fn solve(covariance: &[Vec<f64>], expected_excess_returns: &[f64]) -> Option<MvReport> {
    let n = covariance.len();
    if n < 2 || covariance.iter().any(|r| r.len() != n) {
        return None;
    }
    if expected_excess_returns.len() != n {
        return None;
    }
    if covariance.iter().any(|r| r.iter().any(|c| !c.is_finite())) {
        return None;
    }
    if expected_excess_returns.iter().any(|x| !x.is_finite()) {
        return None;
    }
    let sigma_inv = invert(covariance)?;
    let ones = vec![1.0_f64; n];
    let sigma_inv_ones = matvec(&sigma_inv, &ones);
    let denom_mv: f64 = ones
        .iter()
        .zip(sigma_inv_ones.iter())
        .map(|(a, b)| a * b)
        .sum();
    if denom_mv.abs() < 1e-18 {
        return None;
    }
    let mv_weights: Vec<f64> = sigma_inv_ones.iter().map(|x| x / denom_mv).collect();
    let mv_var: f64 = mv_weights
        .iter()
        .zip(matvec(covariance, &mv_weights).iter())
        .map(|(w, sw)| w * sw)
        .sum();
    let mv_vol = mv_var.max(0.0).sqrt();
    let sigma_inv_mu = matvec(&sigma_inv, expected_excess_returns);
    let denom_tan: f64 = ones
        .iter()
        .zip(sigma_inv_mu.iter())
        .map(|(a, b)| a * b)
        .sum();
    if denom_tan.abs() < 1e-18 {
        return None;
    }
    let tan_weights: Vec<f64> = sigma_inv_mu.iter().map(|x| x / denom_tan).collect();
    let tan_var: f64 = tan_weights
        .iter()
        .zip(matvec(covariance, &tan_weights).iter())
        .map(|(w, sw)| w * sw)
        .sum();
    let tan_vol = tan_var.max(0.0).sqrt();
    let tan_ret: f64 = tan_weights
        .iter()
        .zip(expected_excess_returns.iter())
        .map(|(w, r)| w * r)
        .sum();
    let tan_sharpe = if tan_vol > 0.0 {
        tan_ret / tan_vol
    } else {
        0.0
    };
    Some(MvReport {
        min_variance_weights: mv_weights,
        min_variance_portfolio_volatility: mv_vol,
        tangency_weights: tan_weights,
        tangency_expected_return: tan_ret,
        tangency_volatility: tan_vol,
        tangency_sharpe: tan_sharpe,
    })
}

fn matvec(m: &[Vec<f64>], v: &[f64]) -> Vec<f64> {
    m.iter()
        .map(|r| r.iter().zip(v.iter()).map(|(a, b)| a * b).sum())
        .collect()
}

fn invert(m: &[Vec<f64>]) -> Option<Vec<Vec<f64>>> {
    let n = m.len();
    if n == 0 || m.iter().any(|r| r.len() != n) {
        return None;
    }
    let mut aug = vec![vec![0.0_f64; 2 * n]; n];
    for i in 0..n {
        for j in 0..n {
            aug[i][j] = m[i][j];
            aug[i][n + j] = if i == j { 1.0 } else { 0.0 };
        }
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
    Some(aug.into_iter().map(|r| r[n..].to_vec()).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_or_malformed_returns_none() {
        assert!(solve(&[], &[]).is_none());
        assert!(solve(&[vec![1.0]], &[0.05]).is_none());
        let cov = vec![vec![0.04, 0.0], vec![0.0, 0.04]];
        assert!(solve(&cov, &[0.05]).is_none());
    }

    #[test]
    fn nan_input_returns_none() {
        let cov = vec![vec![0.04, f64::NAN], vec![f64::NAN, 0.04]];
        assert!(solve(&cov, &[0.05, 0.05]).is_none());
    }

    #[test]
    fn singular_covariance_returns_none() {
        // Rank-deficient covariance.
        let cov = vec![vec![1.0, 1.0], vec![1.0, 1.0]];
        assert!(solve(&cov, &[0.05, 0.05]).is_none());
    }

    #[test]
    fn equal_variance_uncorrelated_yields_equal_mv_weights() {
        let cov = vec![
            vec![0.04, 0.0, 0.0],
            vec![0.0, 0.04, 0.0],
            vec![0.0, 0.0, 0.04],
        ];
        let mu = vec![0.05, 0.05, 0.05];
        let r = solve(&cov, &mu).unwrap();
        for w in &r.min_variance_weights {
            assert!((w - 1.0 / 3.0).abs() < 1e-9);
        }
    }

    #[test]
    fn weights_sum_to_one() {
        let cov = vec![
            vec![0.04, 0.01, 0.005],
            vec![0.01, 0.09, 0.02],
            vec![0.005, 0.02, 0.16],
        ];
        let mu = vec![0.05, 0.08, 0.12];
        let r = solve(&cov, &mu).unwrap();
        let sum_mv: f64 = r.min_variance_weights.iter().sum();
        let sum_tan: f64 = r.tangency_weights.iter().sum();
        assert!((sum_mv - 1.0).abs() < 1e-9);
        assert!((sum_tan - 1.0).abs() < 1e-9);
    }

    #[test]
    fn min_variance_volatility_le_any_single_asset() {
        let cov = vec![vec![0.04, 0.0], vec![0.0, 0.09]];
        let mu = vec![0.05, 0.07];
        let r = solve(&cov, &mu).unwrap();
        // MV portfolio variance should be ≤ min individual variance (0.04).
        assert!(r.min_variance_portfolio_volatility.powi(2) <= 0.04 + 1e-9);
    }

    #[test]
    fn tangency_sharpe_at_least_min_var_sharpe() {
        // By construction tangency maximizes Sharpe.
        let cov = vec![vec![0.04, 0.01], vec![0.01, 0.09]];
        let mu = vec![0.10, 0.05];
        let r = solve(&cov, &mu).unwrap();
        // Compute MV portfolio Sharpe for comparison.
        let mv_ret: f64 = r
            .min_variance_weights
            .iter()
            .zip(mu.iter())
            .map(|(w, m)| w * m)
            .sum();
        let mv_sharpe = mv_ret / r.min_variance_portfolio_volatility;
        assert!(r.tangency_sharpe >= mv_sharpe - 1e-9);
    }

    #[test]
    fn higher_expected_return_increases_tangency_weight() {
        // Asset 0 gets higher μ → tangency weight should rise.
        let cov = vec![vec![0.04, 0.0], vec![0.0, 0.04]];
        let mu_eq = vec![0.05, 0.05];
        let mu_skew = vec![0.10, 0.05];
        let r_eq = solve(&cov, &mu_eq).unwrap();
        let r_skew = solve(&cov, &mu_skew).unwrap();
        assert!(r_skew.tangency_weights[0] > r_eq.tangency_weights[0]);
    }
}
