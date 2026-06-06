//! Marginal / Component VaR — risk budgeting for a portfolio.
//!
//! Given a covariance matrix `Σ` (k×k) and a weight vector `w` (k×1):
//!
//!   portfolio_variance = wᵀ Σ w
//!   portfolio_vol      = √portfolio_variance
//!   portfolio_var(α)   = portfolio_vol · z_α
//!   marginal_var_i     = z_α · (Σ w)_i / portfolio_vol
//!   component_var_i    = w_i · marginal_var_i
//!   pct_contribution_i = component_var_i / portfolio_var
//!
//! Used to identify which positions contribute most to portfolio
//! tail risk — required for risk-budgeting trade decisions.
//!
//! Pure compute. Caller supplies the covariance matrix in row-major
//! `Vec<Vec<f64>>` form and matching weights. z-score for confidence α
//! supplied directly (1.645 for 95%, 2.326 for 99%).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Portfolio {
    pub weights: Vec<f64>,
    /// k×k covariance matrix (variance of returns), row-major.
    pub covariance: Vec<Vec<f64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VarReport {
    pub portfolio_var: f64,
    pub portfolio_vol: f64,
    pub marginal_var: Vec<f64>,
    pub component_var: Vec<f64>,
    pub pct_contribution: Vec<f64>,
}

pub fn analyze(port: &Portfolio, z_alpha: f64) -> Option<VarReport> {
    let k = port.weights.len();
    if k == 0 || port.covariance.len() != k {
        return None;
    }
    if port.covariance.iter().any(|row| row.len() != k) {
        return None;
    }
    if !z_alpha.is_finite() || z_alpha <= 0.0 {
        return None;
    }
    if port.weights.iter().any(|w| !w.is_finite())
        || port
            .covariance
            .iter()
            .any(|row| row.iter().any(|c| !c.is_finite()))
    {
        return None;
    }
    // Σw
    let sigma_w: Vec<f64> = port
        .covariance
        .iter()
        .map(|row| {
            row.iter()
                .zip(port.weights.iter())
                .map(|(c, w)| c * w)
                .sum()
        })
        .collect();
    // wᵀ Σ w
    let port_variance: f64 = port
        .weights
        .iter()
        .zip(sigma_w.iter())
        .map(|(w, sw)| w * sw)
        .sum();
    if !port_variance.is_finite() || port_variance < 0.0 {
        return None;
    }
    let port_vol = port_variance.sqrt();
    if port_vol == 0.0 {
        // Pure cash / fully-hedged portfolio: every contribution is 0.
        return Some(VarReport {
            portfolio_var: 0.0,
            portfolio_vol: 0.0,
            marginal_var: vec![0.0; k],
            component_var: vec![0.0; k],
            pct_contribution: vec![0.0; k],
        });
    }
    let port_var_at_alpha = port_vol * z_alpha;
    let marginal: Vec<f64> = sigma_w.iter().map(|sw| z_alpha * sw / port_vol).collect();
    let component: Vec<f64> = port
        .weights
        .iter()
        .zip(marginal.iter())
        .map(|(w, m)| w * m)
        .collect();
    let pct: Vec<f64> = component
        .iter()
        .map(|c| c / port_var_at_alpha * 100.0)
        .collect();
    Some(VarReport {
        portfolio_var: port_var_at_alpha,
        portfolio_vol: port_vol,
        marginal_var: marginal,
        component_var: component,
        pct_contribution: pct,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_none() {
        let p = Portfolio {
            weights: vec![],
            covariance: vec![],
        };
        assert!(analyze(&p, 1.645).is_none());
    }

    #[test]
    fn dim_mismatch_returns_none() {
        let p = Portfolio {
            weights: vec![0.5, 0.5],
            covariance: vec![vec![1.0; 2]],
        };
        assert!(analyze(&p, 1.645).is_none());
        let p = Portfolio {
            weights: vec![0.5, 0.5],
            covariance: vec![vec![1.0; 3], vec![1.0; 3]],
        };
        assert!(analyze(&p, 1.645).is_none());
    }

    #[test]
    fn invalid_z_returns_none() {
        let p = Portfolio {
            weights: vec![1.0],
            covariance: vec![vec![0.04]],
        };
        assert!(analyze(&p, 0.0).is_none());
        assert!(analyze(&p, -1.0).is_none());
        assert!(analyze(&p, f64::NAN).is_none());
    }

    #[test]
    fn nan_input_returns_none() {
        let p = Portfolio {
            weights: vec![f64::NAN],
            covariance: vec![vec![0.04]],
        };
        assert!(analyze(&p, 1.645).is_none());
        let p = Portfolio {
            weights: vec![1.0],
            covariance: vec![vec![f64::NAN]],
        };
        assert!(analyze(&p, 1.645).is_none());
    }

    #[test]
    fn single_asset_var_matches_simple_formula() {
        // σ² = 0.04 → σ = 0.2. VaR(95%) = 0.2 · 1.645 = 0.329.
        let p = Portfolio {
            weights: vec![1.0],
            covariance: vec![vec![0.04]],
        };
        let r = analyze(&p, 1.645).unwrap();
        assert!((r.portfolio_var - 0.329).abs() < 1e-3);
        assert!((r.pct_contribution[0] - 100.0).abs() < 1e-9);
    }

    #[test]
    fn uncorrelated_50_50_split_contributions_match_variances() {
        // w = [0.5, 0.5], Σ = diag(0.04, 0.09)
        // Var(p) = 0.25·0.04 + 0.25·0.09 = 0.0325; vol = 0.180278.
        // Component_i = w_i · z · (Σw)_i / vol.
        let p = Portfolio {
            weights: vec![0.5, 0.5],
            covariance: vec![vec![0.04, 0.0], vec![0.0, 0.09]],
        };
        let r = analyze(&p, 1.645).unwrap();
        // Components must sum to total VaR.
        let sum: f64 = r.component_var.iter().sum();
        assert!((sum - r.portfolio_var).abs() < 1e-9);
        // Higher-vol asset should contribute more.
        assert!(r.pct_contribution[1] > r.pct_contribution[0]);
    }

    #[test]
    fn perfectly_hedged_portfolio_has_zero_var() {
        // Equal long-short on identically-distributed assets with corr=1.
        let p = Portfolio {
            weights: vec![1.0, -1.0],
            covariance: vec![vec![0.04, 0.04], vec![0.04, 0.04]],
        };
        let r = analyze(&p, 1.645).unwrap();
        assert!(r.portfolio_var.abs() < 1e-12);
        assert!(r.pct_contribution.iter().all(|x| x.abs() < 1e-12));
    }

    #[test]
    fn component_var_sums_to_total_var_identity() {
        let p = Portfolio {
            weights: vec![0.3, 0.4, 0.3],
            covariance: vec![
                vec![0.04, 0.01, 0.005],
                vec![0.01, 0.09, 0.02],
                vec![0.005, 0.02, 0.16],
            ],
        };
        let r = analyze(&p, 2.326).unwrap();
        let sum_components: f64 = r.component_var.iter().sum();
        assert!((sum_components - r.portfolio_var).abs() < 1e-9);
        let sum_pct: f64 = r.pct_contribution.iter().sum();
        assert!((sum_pct - 100.0).abs() < 1e-9);
    }
}
