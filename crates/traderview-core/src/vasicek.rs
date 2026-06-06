//! Vasicek (1977) short-rate model — zero-coupon bond pricing.
//!
//!   dr_t = κ · (θ − r_t) · dt + σ · dW_t
//!
//! Closed-form zero-coupon bond price for tenor T:
//!
//!   B(T) = (1 − e^{−κT}) / κ
//!   A(T) = (B(T) − T) · (κ²·θ − σ²/2) / κ² − σ²·B(T)² / (4κ)
//!   P(0, T) = exp(A(T) − B(T) · r_0)
//!
//! Zero rate: y(T) = −ln(P) / T.
//!
//! Vasicek allows NEGATIVE rates (a feature in some regimes, a bug in
//! others — CIR model in `cir` would fix that). Returns the bond price,
//! the zero rate, and the A/B factors for caller diagnostics.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct VasicekParams {
    pub kappa: f64, // mean-reversion speed
    pub theta: f64, // long-run mean rate
    pub sigma: f64, // volatility of short rate
    pub r0: f64,    // current short rate
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct VasicekZcb {
    pub bond_price: f64,
    pub zero_rate: f64,
    pub a_factor: f64,
    pub b_factor: f64,
}

pub fn zero_coupon_bond(params: &VasicekParams, tenor: f64) -> Option<VasicekZcb> {
    if !params.kappa.is_finite()
        || params.kappa <= 0.0
        || !params.theta.is_finite()
        || !params.sigma.is_finite()
        || params.sigma < 0.0
        || !params.r0.is_finite()
        || !tenor.is_finite()
        || tenor <= 0.0
    {
        return None;
    }
    let k = params.kappa;
    let theta = params.theta;
    let sigma = params.sigma;
    let r0 = params.r0;
    let b = (1.0 - (-k * tenor).exp()) / k;
    let sigma2 = sigma * sigma;
    let a = (b - tenor) * (k * k * theta - sigma2 / 2.0) / (k * k) - sigma2 * b * b / (4.0 * k);
    let price = (a - b * r0).exp();
    if !price.is_finite() || price <= 0.0 {
        return None;
    }
    let zero_rate = -price.ln() / tenor;
    Some(VasicekZcb {
        bond_price: price,
        zero_rate,
        a_factor: a,
        b_factor: b,
    })
}

/// Generate a full zero-curve over a vector of tenors. Skips tenors
/// that fail validation (returns None at those positions).
pub fn zero_curve(params: &VasicekParams, tenors: &[f64]) -> Vec<Option<VasicekZcb>> {
    tenors
        .iter()
        .map(|t| zero_coupon_bond(params, *t))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(k: f64, theta: f64, sigma: f64, r0: f64) -> VasicekParams {
        VasicekParams {
            kappa: k,
            theta,
            sigma,
            r0,
        }
    }

    #[test]
    fn invalid_inputs_return_none() {
        assert!(zero_coupon_bond(&p(0.0, 0.05, 0.01, 0.03), 1.0).is_none());
        assert!(zero_coupon_bond(&p(-1.0, 0.05, 0.01, 0.03), 1.0).is_none());
        assert!(zero_coupon_bond(&p(0.5, 0.05, -0.01, 0.03), 1.0).is_none());
        assert!(zero_coupon_bond(&p(0.5, 0.05, 0.01, 0.03), 0.0).is_none());
        assert!(zero_coupon_bond(&p(0.5, 0.05, 0.01, 0.03), -1.0).is_none());
        assert!(zero_coupon_bond(&p(f64::NAN, 0.05, 0.01, 0.03), 1.0).is_none());
    }

    #[test]
    fn zero_volatility_yields_constant_rate_curve() {
        // σ = 0 → no variance correction → rate = r0 at all tenors (limit).
        // With κ > 0 and θ = r0, all tenors should give zero_rate ≈ r0.
        let r = zero_coupon_bond(&p(0.5, 0.03, 0.0, 0.03), 5.0).unwrap();
        assert!((r.zero_rate - 0.03).abs() < 1e-9);
    }

    #[test]
    fn bond_price_less_than_one_for_positive_rates() {
        let r = zero_coupon_bond(&p(0.5, 0.04, 0.01, 0.03), 1.0).unwrap();
        assert!(r.bond_price < 1.0 && r.bond_price > 0.0);
    }

    #[test]
    fn longer_tenor_yields_lower_bond_price_under_positive_rates() {
        let p1 = zero_coupon_bond(&p(0.5, 0.04, 0.01, 0.03), 1.0).unwrap();
        let p5 = zero_coupon_bond(&p(0.5, 0.04, 0.01, 0.03), 5.0).unwrap();
        assert!(p5.bond_price < p1.bond_price);
    }

    #[test]
    fn rate_converges_toward_theta_at_long_tenor() {
        // Without σ correction, as T → ∞, y(T) → θ − σ²/(2κ²).
        let pp = p(0.5, 0.05, 0.0, 0.03);
        let r = zero_coupon_bond(&pp, 100.0).unwrap();
        assert!((r.zero_rate - pp.theta).abs() < 0.01);
    }

    #[test]
    fn convexity_correction_reduces_long_rate() {
        // With σ > 0 the long rate should be BELOW θ by σ²/(2κ²).
        let pp_no_vol = p(0.5, 0.05, 0.0, 0.05);
        let pp_vol = p(0.5, 0.05, 0.02, 0.05);
        let no_vol = zero_coupon_bond(&pp_no_vol, 30.0).unwrap();
        let with_vol = zero_coupon_bond(&pp_vol, 30.0).unwrap();
        assert!(with_vol.zero_rate < no_vol.zero_rate);
    }

    #[test]
    fn b_factor_increases_with_tenor() {
        let b1 = zero_coupon_bond(&p(0.5, 0.04, 0.01, 0.03), 1.0)
            .unwrap()
            .b_factor;
        let b5 = zero_coupon_bond(&p(0.5, 0.04, 0.01, 0.03), 5.0)
            .unwrap()
            .b_factor;
        assert!(b5 > b1);
    }

    #[test]
    fn zero_curve_returns_vector_of_same_length() {
        let pp = p(0.5, 0.04, 0.01, 0.03);
        let tenors = vec![0.5, 1.0, 2.0, 5.0, 10.0];
        let curve = zero_curve(&pp, &tenors);
        assert_eq!(curve.len(), 5);
        assert!(curve.iter().all(|x| x.is_some()));
    }

    #[test]
    fn zero_curve_skips_invalid_tenors() {
        let pp = p(0.5, 0.04, 0.01, 0.03);
        let tenors = vec![1.0, -1.0, 5.0];
        let curve = zero_curve(&pp, &tenors);
        assert!(curve[0].is_some());
        assert!(curve[1].is_none());
        assert!(curve[2].is_some());
    }
}
