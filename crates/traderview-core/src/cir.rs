//! Cox-Ingersoll-Ross (1985) short-rate model — zero-coupon bond.
//!
//!   dr_t = κ · (θ − r_t) · dt + σ · √r_t · dW_t
//!
//! Unlike Vasicek's Gaussian-r model, CIR's √r diffusion keeps r > 0
//! as long as the Feller condition 2κθ ≥ σ² holds. Otherwise the
//! process can touch zero with positive probability.
//!
//! Closed-form ZCB price:
//!   γ = √(κ² + 2σ²)
//!   B(T) = 2·(e^{γT} − 1) / ((γ + κ)·(e^{γT} − 1) + 2γ)
//!   A(T) = [2γ·e^{(κ+γ)T/2} / ((γ + κ)·(e^{γT} − 1) + 2γ)]^{2κθ/σ²}
//!   P(0, T) = A(T) · exp(−B(T)·r_0)
//!
//! Companion to `vasicek` — both produce zero-coupon prices, but CIR
//! guarantees non-negative rates and yields fatter-tailed bond prices.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CirParams {
    pub kappa: f64,
    pub theta: f64,
    pub sigma: f64,
    pub r0: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CirZcb {
    pub bond_price: f64,
    pub zero_rate: f64,
    pub a_factor: f64,
    pub b_factor: f64,
    pub feller_condition_satisfied: bool,
}

pub fn zero_coupon_bond(p: &CirParams, tenor: f64) -> Option<CirZcb> {
    if !p.kappa.is_finite() || p.kappa <= 0.0
        || !p.theta.is_finite() || p.theta < 0.0
        || !p.sigma.is_finite() || p.sigma < 0.0
        || !p.r0.is_finite() || p.r0 < 0.0
        || !tenor.is_finite() || tenor <= 0.0
    {
        return None;
    }
    let k = p.kappa;
    let theta = p.theta;
    let sigma = p.sigma;
    let r0 = p.r0;
    let sigma2 = sigma * sigma;
    let gamma = (k * k + 2.0 * sigma2).sqrt();
    let e_gt = (gamma * tenor).exp();
    let denom = (gamma + k) * (e_gt - 1.0) + 2.0 * gamma;
    if !denom.is_finite() || denom == 0.0 { return None; }
    let b = 2.0 * (e_gt - 1.0) / denom;
    // A is positive for valid parameters.
    let a_num = 2.0 * gamma * ((k + gamma) * tenor / 2.0).exp();
    let a_base = a_num / denom;
    if !a_base.is_finite() || a_base <= 0.0 { return None; }
    let a_exp = if sigma2 > 0.0 { 2.0 * k * theta / sigma2 } else { 0.0 };
    let a = a_base.powf(a_exp);
    if !a.is_finite() || a <= 0.0 { return None; }
    let price = a * (-b * r0).exp();
    if !price.is_finite() || price <= 0.0 { return None; }
    let zero_rate = -price.ln() / tenor;
    let feller = 2.0 * k * theta >= sigma2;
    Some(CirZcb {
        bond_price: price,
        zero_rate,
        a_factor: a,
        b_factor: b,
        feller_condition_satisfied: feller,
    })
}

pub fn zero_curve(p: &CirParams, tenors: &[f64]) -> Vec<Option<CirZcb>> {
    tenors.iter().map(|t| zero_coupon_bond(p, *t)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(k: f64, theta: f64, sigma: f64, r0: f64) -> CirParams {
        CirParams { kappa: k, theta, sigma, r0 }
    }

    #[test]
    fn invalid_inputs_return_none() {
        assert!(zero_coupon_bond(&p(0.0, 0.05, 0.05, 0.03), 1.0).is_none());
        assert!(zero_coupon_bond(&p(-1.0, 0.05, 0.05, 0.03), 1.0).is_none());
        assert!(zero_coupon_bond(&p(0.5, -0.05, 0.05, 0.03), 1.0).is_none());
        assert!(zero_coupon_bond(&p(0.5, 0.05, -0.05, 0.03), 1.0).is_none());
        assert!(zero_coupon_bond(&p(0.5, 0.05, 0.05, -0.03), 1.0).is_none());
        assert!(zero_coupon_bond(&p(0.5, 0.05, 0.05, 0.03), 0.0).is_none());
        assert!(zero_coupon_bond(&p(f64::NAN, 0.05, 0.05, 0.03), 1.0).is_none());
    }

    #[test]
    fn zero_volatility_collapses_to_vasicek_no_vol_form() {
        // σ = 0 → γ = κ → b = (e^{κT} − 1) / (κ·e^{κT}) = (1 − e^{−κT})/κ.
        // Matches Vasicek B factor exactly.
        let pp = p(0.5, 0.04, 0.0, 0.03);
        let r = zero_coupon_bond(&pp, 5.0).unwrap();
        let expected_b = (1.0 - (-0.5_f64 * 5.0).exp()) / 0.5;
        assert!((r.b_factor - expected_b).abs() < 1e-9);
    }

    #[test]
    fn bond_price_between_zero_and_one_for_positive_rates() {
        let r = zero_coupon_bond(&p(0.5, 0.04, 0.05, 0.03), 1.0).unwrap();
        assert!(r.bond_price > 0.0 && r.bond_price < 1.0);
    }

    #[test]
    fn longer_tenor_yields_lower_bond_price() {
        let p1 = zero_coupon_bond(&p(0.5, 0.04, 0.05, 0.03), 1.0).unwrap();
        let p5 = zero_coupon_bond(&p(0.5, 0.04, 0.05, 0.03), 5.0).unwrap();
        assert!(p5.bond_price < p1.bond_price);
    }

    #[test]
    fn feller_condition_flagged_correctly() {
        // 2κθ < σ² → violated.
        let r = zero_coupon_bond(&p(0.1, 0.01, 0.20, 0.03), 1.0).unwrap();
        assert!(!r.feller_condition_satisfied);
        // 2κθ ≥ σ² → satisfied.
        let r = zero_coupon_bond(&p(1.0, 0.10, 0.05, 0.03), 1.0).unwrap();
        assert!(r.feller_condition_satisfied);
    }

    #[test]
    fn higher_initial_rate_reduces_bond_price() {
        let r_low = zero_coupon_bond(&p(0.5, 0.04, 0.05, 0.01), 1.0).unwrap();
        let r_high = zero_coupon_bond(&p(0.5, 0.04, 0.05, 0.10), 1.0).unwrap();
        assert!(r_high.bond_price < r_low.bond_price);
    }

    #[test]
    fn long_run_yield_bounded_above_by_2_kappa_theta_over_gamma_plus_kappa() {
        // CIR long-run yield → 2κθ / (γ + κ) where γ = √(κ² + 2σ²).
        // This is strictly below θ when σ > 0 (convexity adjustment),
        // BUT the convergence is slow and depends on r₀. Just verify
        // the asymptotic bound holds at very long T.
        let pp = p(0.5, 0.05, 0.10, 0.05);
        let kappa = pp.kappa;
        let sigma = pp.sigma;
        let theta = pp.theta;
        let gamma = (kappa * kappa + 2.0 * sigma * sigma).sqrt();
        let bound = 2.0 * kappa * theta / (gamma + kappa);
        let r = zero_coupon_bond(&pp, 100.0).unwrap();
        // At T=100 with κ=0.5 the rate should be close to the bound.
        assert!((r.zero_rate - bound).abs() < 0.01,
            "long-run rate {} should be close to bound {}", r.zero_rate, bound);
        // And the bound itself should be below θ (convexity-adjusted).
        assert!(bound < pp.theta);
    }

    #[test]
    fn zero_curve_returns_vec_of_same_length() {
        let pp = p(0.5, 0.04, 0.05, 0.03);
        let tenors = vec![0.5, 1.0, 2.0, 5.0, 10.0];
        let curve = zero_curve(&pp, &tenors);
        assert_eq!(curve.len(), 5);
        assert!(curve.iter().all(|x| x.is_some()));
    }
}
