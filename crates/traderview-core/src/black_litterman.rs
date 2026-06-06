//! Black-Litterman portfolio model — combines equilibrium implied
//! returns with subjective investor views to yield a posterior return
//! vector for mean-variance optimization.
//!
//! Inputs:
//!   - **Σ** (n×n covariance matrix)
//!   - **π** (n-vector of equilibrium "market-implied" returns)
//!   - **P** (k×n view-loading matrix; each row selects/blends assets)
//!   - **Q** (k-vector of view returns)
//!   - **Ω** (k×k confidence matrix on views; diagonal in practice)
//!   - **τ** (scalar prior weight, typically 0.025–0.05)
//!
//! Posterior mean (He-Litterman 1999 form):
//!   μ_bl = [(τΣ)⁻¹ + Pᵀ Ω⁻¹ P]⁻¹ · [(τΣ)⁻¹ π + Pᵀ Ω⁻¹ Q]
//!
//! Posterior covariance:
//!   Σ_bl = Σ + [(τΣ)⁻¹ + Pᵀ Ω⁻¹ P]⁻¹
//!
//! Pure compute. Caller supplies all matrices already estimated.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlackLittermanInputs {
    pub covariance: Vec<Vec<f64>>,
    pub equilibrium_returns: Vec<f64>,
    /// k×n matrix; each row is one view's loading on assets.
    pub view_loadings: Vec<Vec<f64>>,
    pub view_returns: Vec<f64>,
    /// k×k confidence matrix; pass diagonal Ω in full square form.
    pub view_confidence: Vec<Vec<f64>>,
    pub tau: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BlackLittermanReport {
    pub posterior_returns: Vec<f64>,
    pub posterior_covariance: Vec<Vec<f64>>,
}

pub fn solve(inputs: &BlackLittermanInputs) -> Option<BlackLittermanReport> {
    let n = inputs.equilibrium_returns.len();
    let k = inputs.view_returns.len();
    if n == 0
        || inputs.covariance.len() != n
        || inputs.covariance.iter().any(|r| r.len() != n)
        || inputs.equilibrium_returns.iter().any(|v| !v.is_finite())
        || inputs
            .covariance
            .iter()
            .any(|r| r.iter().any(|c| !c.is_finite()))
        || !inputs.tau.is_finite()
        || inputs.tau <= 0.0
    {
        return None;
    }
    if k > 0
        && (inputs.view_loadings.len() != k
            || inputs.view_loadings.iter().any(|r| r.len() != n)
            || inputs.view_confidence.len() != k
            || inputs.view_confidence.iter().any(|r| r.len() != k)
            || inputs.view_returns.iter().any(|v| !v.is_finite())
            || inputs
                .view_loadings
                .iter()
                .any(|r| r.iter().any(|c| !c.is_finite()))
            || inputs
                .view_confidence
                .iter()
                .any(|r| r.iter().any(|c| !c.is_finite())))
    {
        return None;
    }
    // No views → posterior == prior.
    if k == 0 {
        return Some(BlackLittermanReport {
            posterior_returns: inputs.equilibrium_returns.clone(),
            posterior_covariance: inputs.covariance.clone(),
        });
    }
    // (τΣ)⁻¹
    let tau_sigma = scale(&inputs.covariance, inputs.tau);
    let tau_sigma_inv = invert(&tau_sigma)?;
    let omega_inv = invert(&inputs.view_confidence)?;
    // PᵀΩ⁻¹P
    let pt = transpose(&inputs.view_loadings);
    let pt_omega_inv = matmul(&pt, &omega_inv);
    let pt_omega_inv_p = matmul(&pt_omega_inv, &inputs.view_loadings);
    // Posterior precision: A = (τΣ)⁻¹ + PᵀΩ⁻¹P
    let a = matadd(&tau_sigma_inv, &pt_omega_inv_p);
    let a_inv = invert(&a)?;
    // (τΣ)⁻¹π
    let tau_sigma_inv_pi = matvec(&tau_sigma_inv, &inputs.equilibrium_returns);
    // PᵀΩ⁻¹Q
    let pt_omega_inv_q = matvec(&pt_omega_inv, &inputs.view_returns);
    let rhs: Vec<f64> = tau_sigma_inv_pi
        .iter()
        .zip(pt_omega_inv_q.iter())
        .map(|(a, b)| a + b)
        .collect();
    let posterior_returns = matvec(&a_inv, &rhs);
    // Posterior covariance = Σ + A⁻¹
    let posterior_cov = matadd(&inputs.covariance, &a_inv);
    Some(BlackLittermanReport {
        posterior_returns,
        posterior_covariance: posterior_cov,
    })
}

fn scale(m: &[Vec<f64>], s: f64) -> Vec<Vec<f64>> {
    m.iter()
        .map(|r| r.iter().map(|x| x * s).collect())
        .collect()
}

fn transpose(m: &[Vec<f64>]) -> Vec<Vec<f64>> {
    if m.is_empty() {
        return Vec::new();
    }
    let rows = m.len();
    let cols = m[0].len();
    let mut out = vec![vec![0.0; rows]; cols];
    for (i, row) in m.iter().enumerate() {
        for (j, &v) in row.iter().enumerate() {
            out[j][i] = v;
        }
    }
    out
}

fn matmul(a: &[Vec<f64>], b: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let ar = a.len();
    let ac = a.first().map(|r| r.len()).unwrap_or(0);
    let bc = b.first().map(|r| r.len()).unwrap_or(0);
    let mut out = vec![vec![0.0; bc]; ar];
    for (i, out_row) in out.iter_mut().enumerate() {
        for j in 0..bc {
            let mut s = 0.0;
            for kk in 0..ac {
                s += a[i][kk] * b[kk][j];
            }
            out_row[j] = s;
        }
    }
    out
}

fn matvec(m: &[Vec<f64>], v: &[f64]) -> Vec<f64> {
    m.iter()
        .map(|r| r.iter().zip(v.iter()).map(|(a, b)| a * b).sum())
        .collect()
}

fn matadd(a: &[Vec<f64>], b: &[Vec<f64>]) -> Vec<Vec<f64>> {
    a.iter()
        .zip(b.iter())
        .map(|(ra, rb)| ra.iter().zip(rb.iter()).map(|(x, y)| x + y).collect())
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

    fn inputs() -> BlackLittermanInputs {
        BlackLittermanInputs {
            covariance: vec![vec![0.04, 0.01], vec![0.01, 0.09]],
            equilibrium_returns: vec![0.05, 0.07],
            view_loadings: vec![vec![1.0, -1.0]], // view: asset 1 outperforms asset 2 by Q.
            view_returns: vec![0.02],
            view_confidence: vec![vec![0.001]],
            tau: 0.05,
        }
    }

    #[test]
    fn invalid_inputs_return_none() {
        let mut bad = inputs();
        bad.covariance = vec![];
        assert!(solve(&bad).is_none());
        let mut bad = inputs();
        bad.tau = 0.0;
        assert!(solve(&bad).is_none());
        let mut bad = inputs();
        bad.tau = f64::NAN;
        assert!(solve(&bad).is_none());
    }

    #[test]
    fn dim_mismatch_returns_none() {
        let mut bad = inputs();
        bad.equilibrium_returns = vec![0.05];
        assert!(solve(&bad).is_none());
        let mut bad = inputs();
        bad.view_loadings = vec![vec![1.0, -1.0, 0.0]]; // wrong dim
        assert!(solve(&bad).is_none());
    }

    #[test]
    fn no_views_yields_prior() {
        let mut no_views = inputs();
        no_views.view_loadings = vec![];
        no_views.view_returns = vec![];
        no_views.view_confidence = vec![];
        let r = solve(&no_views).unwrap();
        assert_eq!(r.posterior_returns, vec![0.05, 0.07]);
    }

    #[test]
    fn very_confident_view_pulls_posterior_toward_view() {
        // High confidence (small Ω) → posterior should align with view.
        let mut high_conf = inputs();
        high_conf.view_confidence = vec![vec![1e-8]];
        let r = solve(&high_conf).unwrap();
        // View was "asset 1 − asset 2 = 0.02". The posterior difference
        // should be close to that.
        let diff = r.posterior_returns[0] - r.posterior_returns[1];
        assert!(
            (diff - 0.02).abs() < 0.01,
            "expected ≈ 0.02 diff, got {diff}"
        );
    }

    #[test]
    fn low_confidence_view_leaves_posterior_close_to_prior() {
        let mut low_conf = inputs();
        low_conf.view_confidence = vec![vec![1e8]]; // very loose view
        let r = solve(&low_conf).unwrap();
        let diff0 = (r.posterior_returns[0] - 0.05).abs();
        let diff1 = (r.posterior_returns[1] - 0.07).abs();
        assert!(
            diff0 < 0.001 && diff1 < 0.001,
            "low-conf view should leave posterior ≈ prior: got {} {}",
            r.posterior_returns[0],
            r.posterior_returns[1]
        );
    }

    #[test]
    fn posterior_covariance_matrix_is_correct_dimension() {
        let r = solve(&inputs()).unwrap();
        assert_eq!(r.posterior_covariance.len(), 2);
        assert_eq!(r.posterior_covariance[0].len(), 2);
    }

    #[test]
    fn singular_omega_returns_none() {
        let mut bad = inputs();
        bad.view_confidence = vec![vec![0.0]];
        assert!(solve(&bad).is_none());
    }
}
