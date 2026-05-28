//! Component Expected Shortfall (ES) — Euler-allocation decomposition
//! of portfolio ES into per-position contributions.
//!
//! ES (a.k.a. CVaR / conditional VaR) is the expected loss given that
//! the loss exceeds VaR. Under the Gaussian assumption:
//!
//!   ES = (φ(z) / (1 − α)) · σ_p
//!
//! where φ(z) is the standard-normal density at z = Φ⁻¹(α).
//!
//! Per-position contribution (Tasche 2002 / Acerbi-Tasche 2002):
//!
//!   MES_i = (φ(z) / (1 − α)) · (Σ·w)_i / σ_p
//!   CES_i = w_i · MES_i
//!   PctES_i = CES_i / ES
//!
//!   Σ_i CES_i = ES                  (exact Euler decomposition)
//!
//! Unlike component VaR, ES is coherent (subadditive), so component ES
//! is a more theoretically defensible risk-budget allocation.
//!
//! Pure compute. Companion to `component_var`, `conditional_var`,
//! `marginal_var`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComponentEsReport {
    pub portfolio_volatility: f64,
    pub portfolio_var: f64,
    pub portfolio_es: f64,
    /// Per-position component ES; sums to portfolio_es.
    pub component_es: Vec<f64>,
    pub marginal_es: Vec<f64>,
    pub percent_es: Vec<f64>,
}

pub fn compute(
    weights: &[f64],
    covariance: &[Vec<f64>],
    confidence: f64,
) -> Option<ComponentEsReport> {
    let n = weights.len();
    if n == 0 || covariance.len() != n { return None; }
    if covariance.iter().any(|r| r.len() != n) { return None; }
    if weights.iter().any(|x| !x.is_finite()) { return None; }
    if covariance.iter().any(|r| r.iter().any(|c| !c.is_finite())) { return None; }
    if !confidence.is_finite() || !(0.5..1.0).contains(&confidence) { return None; }
    let sigma_w = matvec(covariance, weights);
    let port_var: f64 = weights.iter().zip(sigma_w.iter()).map(|(a, b)| a * b).sum();
    if port_var <= 0.0 { return None; }
    let port_vol = port_var.sqrt();
    let z = normal_inv_cdf(confidence);
    let var = z * port_vol;
    let phi_z = standard_normal_pdf(z);
    let es_multiplier = phi_z / (1.0 - confidence);
    let es = es_multiplier * port_vol;
    let mes: Vec<f64> = sigma_w.iter().map(|s| es_multiplier * s / port_vol).collect();
    let ces: Vec<f64> = weights.iter().zip(mes.iter()).map(|(w, m)| w * m).collect();
    let pct: Vec<f64> = ces.iter()
        .map(|c| if es.abs() > 1e-18 { c / es } else { 0.0 }).collect();
    Some(ComponentEsReport {
        portfolio_volatility: port_vol,
        portfolio_var: var,
        portfolio_es: es,
        component_es: ces,
        marginal_es: mes,
        percent_es: pct,
    })
}

fn matvec(m: &[Vec<f64>], v: &[f64]) -> Vec<f64> {
    m.iter().map(|r| r.iter().zip(v.iter()).map(|(a, b)| a * b).sum()).collect()
}

fn standard_normal_pdf(z: f64) -> f64 {
    (-0.5 * z * z).exp() / (2.0 * std::f64::consts::PI).sqrt()
}

/// Peter Acklam's inverse normal CDF, ~1.15e-9 abs error across (0, 1).
fn normal_inv_cdf(p: f64) -> f64 {
    const A: [f64; 6] = [
        -3.969_683_028_665_376e1,
         2.209_460_984_245_205e2,
        -2.759_285_104_469_687e2,
         1.383_577_518_672_69e2,
        -3.066_479_806_614_716e1,
         2.506_628_277_459_239,
    ];
    const B: [f64; 5] = [
        -5.447_609_879_822_406e1,
         1.615_858_368_580_409e2,
        -1.556_989_798_598_866e2,
         6.680_131_188_771_972e1,
        -1.328_068_155_288_572e1,
    ];
    const C: [f64; 6] = [
        -7.784_894_002_430_293e-3,
        -3.223_964_580_411_365e-1,
        -2.400_758_277_161_838,
        -2.549_732_539_343_734,
         4.374_664_141_464_968,
         2.938_163_982_698_783,
    ];
    const D: [f64; 4] = [
         7.784_695_709_041_462e-3,
         3.224_671_290_700_398e-1,
         2.445_134_137_142_996,
         3.754_408_661_907_416,
    ];
    let p_low = 0.02425;
    let p_high = 1.0 - p_low;
    if p < p_low {
        let q = (-2.0 * p.ln()).sqrt();
        (((((C[0]*q+C[1])*q+C[2])*q+C[3])*q+C[4])*q+C[5])
            / ((((D[0]*q+D[1])*q+D[2])*q+D[3])*q+1.0)
    } else if p <= p_high {
        let q = p - 0.5;
        let r = q * q;
        (((((A[0]*r+A[1])*r+A[2])*r+A[3])*r+A[4])*r+A[5]) * q
            / (((((B[0]*r+B[1])*r+B[2])*r+B[3])*r+B[4])*r+1.0)
    } else {
        let q = (-2.0 * (1.0 - p).ln()).sqrt();
        -(((((C[0]*q+C[1])*q+C[2])*q+C[3])*q+C[4])*q+C[5])
            / ((((D[0]*q+D[1])*q+D[2])*q+D[3])*q+1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_dimensions_return_none() {
        let w = vec![0.5, 0.5];
        assert!(compute(&[], &[], 0.95).is_none());
        assert!(compute(&w, &[vec![0.04]], 0.95).is_none());
        let non_square = vec![vec![0.04, 0.01], vec![0.01]];
        assert!(compute(&w, &non_square, 0.95).is_none());
    }

    #[test]
    fn invalid_confidence_returns_none() {
        let w = vec![0.5, 0.5];
        let cov = vec![vec![0.04, 0.01], vec![0.01, 0.04]];
        assert!(compute(&w, &cov, 0.4).is_none());
        assert!(compute(&w, &cov, 1.0).is_none());
        assert!(compute(&w, &cov, f64::NAN).is_none());
    }

    #[test]
    fn nan_inputs_return_none() {
        let w = vec![0.5, f64::NAN];
        let cov = vec![vec![0.04, 0.01], vec![0.01, 0.04]];
        assert!(compute(&w, &cov, 0.95).is_none());
    }

    #[test]
    fn portfolio_es_exceeds_var_under_gaussian() {
        let w = vec![0.4, 0.3, 0.3];
        let cov = vec![
            vec![0.04, 0.01, 0.005],
            vec![0.01, 0.09, 0.02],
            vec![0.005, 0.02, 0.16],
        ];
        let r = compute(&w, &cov, 0.95).unwrap();
        // ES > VaR always (E[L | L > VaR] > VaR).
        assert!(r.portfolio_es > r.portfolio_var,
            "ES {} should exceed VaR {}", r.portfolio_es, r.portfolio_var);
    }

    #[test]
    fn components_sum_to_portfolio_es() {
        let w = vec![0.4, 0.3, 0.3];
        let cov = vec![
            vec![0.04, 0.01, 0.005],
            vec![0.01, 0.09, 0.02],
            vec![0.005, 0.02, 0.16],
        ];
        let r = compute(&w, &cov, 0.95).unwrap();
        let sum: f64 = r.component_es.iter().sum();
        assert!((sum - r.portfolio_es).abs() < 1e-9,
            "components {} should sum to ES {}", sum, r.portfolio_es);
    }

    #[test]
    fn percentages_sum_to_one() {
        let w = vec![0.5, 0.5];
        let cov = vec![vec![0.04, 0.01], vec![0.01, 0.09]];
        let r = compute(&w, &cov, 0.95).unwrap();
        let sum: f64 = r.percent_es.iter().sum();
        assert!((sum - 1.0).abs() < 1e-9);
    }

    #[test]
    fn higher_confidence_yields_higher_es() {
        let w = vec![0.5, 0.5];
        let cov = vec![vec![0.04, 0.01], vec![0.01, 0.09]];
        let r95 = compute(&w, &cov, 0.95).unwrap();
        let r99 = compute(&w, &cov, 0.99).unwrap();
        assert!(r99.portfolio_es > r95.portfolio_es);
    }

    #[test]
    fn larger_position_drives_higher_component_es() {
        let w = vec![0.8, 0.2];
        let cov = vec![vec![0.04, 0.0], vec![0.0, 0.04]];
        let r = compute(&w, &cov, 0.95).unwrap();
        assert!(r.component_es[0] > r.component_es[1]);
    }

    #[test]
    fn short_hedge_can_have_negative_component_es() {
        let w = vec![1.0, -0.5];
        let cov = vec![vec![0.04, 0.035], vec![0.035, 0.04]];
        let r = compute(&w, &cov, 0.95).unwrap();
        assert!(r.component_es[1] < 0.0,
            "short hedge component ES should be negative, got {}", r.component_es[1]);
    }
}
