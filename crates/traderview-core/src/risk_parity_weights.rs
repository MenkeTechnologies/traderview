//! Risk-parity weights solver — Maillard, Roncalli, Teïletche (2010).
//!
//! Find weights `w` such that every asset contributes equally to total
//! portfolio variance. Equivalently:
//!
//!   w_i · (Σ·w)_i = const     for all i
//!
//! Iterative solution via Spinu (2013) "Equal Risk Contributions"
//! algorithm (a fast, monotonically-convergent fixed point):
//!
//!   w_i^{k+1} = w_i^k / [(Σ·w^k)_i / σ_p^k]
//!   then normalize so Σ w_i = 1
//!
//! Returns the converged weights, marginal contributions, and the
//! convergence diagnostic (number of iterations, final max-deviation
//! of contributions). Reverts to None when the covariance matrix is
//! malformed, non-square, or non-positive-definite.
//!
//! Pure compute. Distinct from `risk_parity` (which is a simpler
//! inverse-volatility heuristic).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RiskParityReport {
    pub weights: Vec<f64>,
    pub risk_contributions: Vec<f64>,
    pub portfolio_volatility: f64,
    pub iterations: usize,
    pub max_contribution_deviation: f64,
    pub converged: bool,
}

pub fn solve(covariance: &[Vec<f64>], max_iter: usize, tolerance: f64) -> Option<RiskParityReport> {
    let n = covariance.len();
    if n < 2 || covariance.iter().any(|row| row.len() != n) {
        return None;
    }
    if covariance
        .iter()
        .any(|row| row.iter().any(|c| !c.is_finite()))
    {
        return None;
    }
    if !tolerance.is_finite() || tolerance <= 0.0 || max_iter == 0 {
        return None;
    }
    // Seed with inverse-volatility weights (a strictly better starting point
    // than 1/N for fast convergence).
    let mut w = vec![1.0 / n as f64; n];
    for (i, slot) in w.iter_mut().enumerate() {
        let sigma_i = covariance[i][i].max(0.0).sqrt();
        if sigma_i > 0.0 {
            *slot = 1.0 / sigma_i;
        }
    }
    normalize(&mut w);
    let mut iters = 0_usize;
    let mut max_dev = f64::INFINITY;
    for _ in 0..max_iter {
        iters += 1;
        // Σ · w
        let sigma_w = matvec(covariance, &w);
        // σ_p² = wᵀ · Σ · w
        let port_var: f64 = w.iter().zip(sigma_w.iter()).map(|(a, b)| a * b).sum();
        if port_var <= 0.0 {
            return None;
        }
        let port_vol = port_var.sqrt();
        // Spinu (2013) fixed-point: w_i ← b_i · σ_p / (Σw)_i, where
        // b_i is the target risk budget (1/n for equal RP). The earlier
        // form w_i · σ_p / (Σw)_i is WRONG — it scales the *current*
        // weight by the inverse marginal-risk ratio and diverges away
        // from the true equal-contribution fixed point on heterogeneous
        // covariance inputs.
        let target_budget = 1.0 / n as f64;
        let mut new_w = vec![0.0; n];
        for i in 0..n {
            if sigma_w[i] <= 0.0 {
                new_w[i] = w[i];
            } else {
                new_w[i] = target_budget * port_vol / sigma_w[i];
            }
        }
        normalize(&mut new_w);
        // Convergence check: max abs change in risk contribution.
        let new_sigma_w = matvec(covariance, &new_w);
        let new_port_vol: f64 = new_w
            .iter()
            .zip(new_sigma_w.iter())
            .map(|(a, b)| a * b)
            .sum::<f64>()
            .max(0.0)
            .sqrt();
        if new_port_vol <= 0.0 {
            return None;
        }
        let contributions: Vec<f64> = new_w
            .iter()
            .zip(new_sigma_w.iter())
            .map(|(a, b)| a * b / new_port_vol)
            .collect();
        let target = contributions.iter().sum::<f64>() / n as f64;
        max_dev = contributions
            .iter()
            .map(|c| (c - target).abs())
            .fold(0.0_f64, f64::max);
        w = new_w;
        if max_dev < tolerance {
            break;
        }
    }
    let sigma_w = matvec(covariance, &w);
    let port_var: f64 = w.iter().zip(sigma_w.iter()).map(|(a, b)| a * b).sum();
    let port_vol = port_var.max(0.0).sqrt();
    let contributions: Vec<f64> = w
        .iter()
        .zip(sigma_w.iter())
        .map(|(a, b)| {
            if port_vol > 0.0 {
                a * b / port_vol
            } else {
                0.0
            }
        })
        .collect();
    Some(RiskParityReport {
        weights: w,
        risk_contributions: contributions,
        portfolio_volatility: port_vol,
        iterations: iters,
        max_contribution_deviation: max_dev,
        converged: max_dev < tolerance,
    })
}

fn matvec(m: &[Vec<f64>], v: &[f64]) -> Vec<f64> {
    m.iter()
        .map(|row| row.iter().zip(v.iter()).map(|(a, b)| a * b).sum())
        .collect()
}

fn normalize(w: &mut [f64]) {
    let sum: f64 = w.iter().sum();
    if sum > 0.0 {
        for x in w.iter_mut() {
            *x /= sum;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_or_malformed_returns_none() {
        assert!(solve(&[], 100, 1e-8).is_none());
        let bad = vec![vec![1.0, 0.0]];
        assert!(solve(&bad, 100, 1e-8).is_none());
        let nan = vec![vec![f64::NAN, 0.0], vec![0.0, 1.0]];
        assert!(solve(&nan, 100, 1e-8).is_none());
    }

    #[test]
    fn invalid_solver_params_return_none() {
        let cov = vec![vec![0.04, 0.0], vec![0.0, 0.09]];
        assert!(solve(&cov, 0, 1e-8).is_none());
        assert!(solve(&cov, 100, 0.0).is_none());
        assert!(solve(&cov, 100, f64::NAN).is_none());
    }

    #[test]
    fn equal_variance_uncorrelated_yields_equal_weights() {
        // Diagonal covariance with equal variances → equal weights.
        let cov = vec![
            vec![0.04, 0.0, 0.0],
            vec![0.0, 0.04, 0.0],
            vec![0.0, 0.0, 0.04],
        ];
        let r = solve(&cov, 500, 1e-10).unwrap();
        assert!(r.converged);
        for w in &r.weights {
            assert!((w - 1.0 / 3.0).abs() < 1e-6);
        }
    }

    #[test]
    fn high_vol_asset_gets_lower_weight() {
        // 2-asset: σ_A = 0.10, σ_B = 0.30 → A gets ~3× the weight of B.
        let cov = vec![vec![0.01, 0.0], vec![0.0, 0.09]];
        let r = solve(&cov, 500, 1e-12).unwrap();
        assert!(r.converged);
        assert!(r.weights[0] > r.weights[1]);
        let ratio = r.weights[0] / r.weights[1];
        assert!(
            (ratio - 3.0).abs() < 0.01,
            "expected ratio ≈ 3, got {ratio}"
        );
    }

    #[test]
    fn risk_contributions_equal_after_convergence() {
        let cov = vec![
            vec![0.04, 0.01, 0.005],
            vec![0.01, 0.09, 0.02],
            vec![0.005, 0.02, 0.16],
        ];
        let r = solve(&cov, 500, 1e-10).unwrap();
        assert!(r.converged);
        let target = 1.0 / r.risk_contributions.len() as f64 * r.portfolio_volatility;
        for c in &r.risk_contributions {
            assert!((c - target).abs() < 1e-6, "RC={c} target={target}");
        }
    }

    #[test]
    fn weights_sum_to_one() {
        let cov = vec![
            vec![0.04, 0.01, 0.005, 0.0],
            vec![0.01, 0.09, 0.02, 0.0],
            vec![0.005, 0.02, 0.16, 0.01],
            vec![0.0, 0.0, 0.01, 0.25],
        ];
        let r = solve(&cov, 500, 1e-10).unwrap();
        let sum: f64 = r.weights.iter().sum();
        assert!((sum - 1.0).abs() < 1e-9);
    }

    #[test]
    fn singular_matrix_returns_none_or_no_convergence() {
        // Zero covariance row → degenerate.
        let cov = vec![vec![0.0, 0.0], vec![0.0, 0.0]];
        let r = solve(&cov, 100, 1e-8);
        assert!(r.is_none());
    }
}
