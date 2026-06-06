//! Equal-Risk-Contribution Portfolio — weights such that every asset
//! contributes the same marginal portfolio risk. Special case of risk
//! parity that doesn't require diagonal-cov; works on the full Σ.
//!
//! Per-asset risk contribution:
//!   RC_i = w_i · (Σ w)_i / σ_p,    σ_p = sqrt(wᵀ Σ w)
//!
//! ERC solves for w (all > 0, sum to 1) such that RC_i = σ_p / n for
//! every i. Closed-form unavailable; solved by Newton-style iteration
//! on the Lagrangian (Maillard, Roncalli, Teïletche 2010):
//!
//!   w_i ← w_i · (target_i / RC_i)^α
//!
//! followed by re-normalization to sum-to-one. Damping factor α = 0.5
//! gives stable convergence on well-conditioned cov matrices.
//!
//! Returns weights, per-asset risk contributions, and the resulting
//! portfolio variance. If any RC has more than 1% relative spread
//! from the average, `converged = false`.
//!
//! Pure compute. Companion to `risk_parity_weights`,
//! `hierarchical_risk_parity`, `min_variance_portfolio`.

#[derive(Debug)]
pub struct Report {
    pub weights: Vec<f64>,
    pub risk_contributions: Vec<f64>,
    pub portfolio_variance: f64,
    pub portfolio_stdev: f64,
    pub iterations: u32,
    pub converged: bool,
}

pub fn compute(cov: &[Vec<f64>], max_iter: u32) -> Option<Report> {
    let n = cov.len();
    if n < 2 || max_iter == 0 {
        return None;
    }
    if cov.iter().any(|row| row.len() != n) {
        return None;
    }
    if cov.iter().any(|row| row.iter().any(|v| !v.is_finite())) {
        return None;
    }
    if (0..n).any(|i| cov[i][i] <= 0.0) {
        return None;
    }
    // Initialize: inverse-volatility weights.
    let inv_vol: Vec<f64> = (0..n).map(|i| 1.0 / cov[i][i].sqrt()).collect();
    let denom_init: f64 = inv_vol.iter().sum();
    let mut w: Vec<f64> = inv_vol.iter().map(|v| v / denom_init).collect();
    let target_share = 1.0 / n as f64;
    let mut iters = 0;
    let mut converged = false;
    let alpha = 0.5_f64;
    for _ in 0..max_iter {
        iters += 1;
        let sigma_w = matvec(cov, &w);
        let var: f64 = w.iter().zip(sigma_w.iter()).map(|(a, b)| a * b).sum();
        if var <= 1e-18 {
            break;
        }
        let total_rc: f64 = w.iter().zip(sigma_w.iter()).map(|(a, b)| a * b).sum();
        let rc: Vec<f64> = w
            .iter()
            .zip(sigma_w.iter())
            .map(|(a, b)| (a * b) / total_rc)
            .collect();
        // Convergence check: max relative deviation < 0.5%.
        let max_dev = rc
            .iter()
            .map(|x| (x - target_share).abs() / target_share)
            .fold(0.0_f64, f64::max);
        if max_dev < 5e-3 {
            converged = true;
            break;
        }
        // Update.
        for i in 0..n {
            let ratio = (target_share / rc[i].max(1e-18)).powf(alpha);
            w[i] *= ratio;
        }
        let sum: f64 = w.iter().sum();
        for wi in w.iter_mut() {
            *wi /= sum;
        }
    }
    let sigma_w = matvec(cov, &w);
    let variance: f64 = w.iter().zip(sigma_w.iter()).map(|(a, b)| a * b).sum();
    let stdev = variance.max(0.0).sqrt();
    let total_rc: f64 = w.iter().zip(sigma_w.iter()).map(|(a, b)| a * b).sum();
    let rc: Vec<f64> = w
        .iter()
        .zip(sigma_w.iter())
        .map(|(a, b)| (a * b) / total_rc.max(1e-18))
        .collect();
    Some(Report {
        weights: w,
        risk_contributions: rc,
        portfolio_variance: variance,
        portfolio_stdev: stdev,
        iterations: iters,
        converged,
    })
}

fn matvec(m: &[Vec<f64>], v: &[f64]) -> Vec<f64> {
    let n = m.len();
    (0..n)
        .map(|i| {
            let mut s = 0.0;
            for j in 0..n {
                s += m[i][j] * v[j];
            }
            s
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn diag(d: &[f64]) -> Vec<Vec<f64>> {
        let n = d.len();
        let mut m = vec![vec![0.0_f64; n]; n];
        for (i, &v) in d.iter().enumerate() {
            m[i][i] = v;
        }
        m
    }

    #[test]
    fn invalid_inputs_return_none() {
        let m = diag(&[0.04, 0.04]);
        assert!(compute(&m, 0).is_none());
        assert!(compute(&[], 100).is_none());
        let bad_shape = vec![vec![1.0, 0.0], vec![0.0]];
        assert!(compute(&bad_shape, 100).is_none());
        let bad_diag = vec![vec![0.0, 0.0], vec![0.0, 1.0]];
        assert!(compute(&bad_diag, 100).is_none());
    }

    #[test]
    fn equal_variances_yield_equal_weights() {
        let m = diag(&[0.04, 0.04, 0.04, 0.04]);
        let r = compute(&m, 500).unwrap();
        for w in &r.weights {
            assert!((w - 0.25).abs() < 1e-4);
        }
        assert!(r.converged);
    }

    #[test]
    fn weights_sum_to_one() {
        let m = vec![
            vec![0.04, 0.01, 0.005],
            vec![0.01, 0.09, 0.02],
            vec![0.005, 0.02, 0.16],
        ];
        let r = compute(&m, 500).unwrap();
        assert!((r.weights.iter().sum::<f64>() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn risk_contributions_approximately_equal() {
        let m = vec![
            vec![0.04, 0.01, 0.005],
            vec![0.01, 0.09, 0.02],
            vec![0.005, 0.02, 0.16],
        ];
        let r = compute(&m, 1000).unwrap();
        let target = 1.0 / 3.0;
        for rc in &r.risk_contributions {
            assert!((rc - target).abs() < 0.005);
        }
    }

    #[test]
    fn high_vol_asset_gets_smaller_weight() {
        // Diagonal: σ² = 0.01, 0.04, 0.16. Inv-vol weights ≠ ERC but
        // ordering should match: low-vol → high weight.
        let m = diag(&[0.01, 0.04, 0.16]);
        let r = compute(&m, 500).unwrap();
        assert!(r.weights[0] > r.weights[1]);
        assert!(r.weights[1] > r.weights[2]);
    }

    #[test]
    fn portfolio_variance_finite_and_non_negative() {
        let m = vec![vec![0.04, 0.01], vec![0.01, 0.09]];
        let r = compute(&m, 500).unwrap();
        assert!(r.portfolio_variance >= 0.0);
        assert!(r.portfolio_variance.is_finite());
        assert!((r.portfolio_stdev * r.portfolio_stdev - r.portfolio_variance).abs() < 1e-12);
    }
}
