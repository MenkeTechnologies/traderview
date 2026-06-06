//! Hull-White (1990) single-factor extended-Vasicek model — zero-coupon
//! bond pricing.
//!
//!   dr_t = [θ(t) − a · r_t] · dt + σ · dW_t
//!
//! Unlike vanilla Vasicek (constant θ), Hull-White's time-dependent
//! θ(t) lets the model fit the initial yield curve exactly. For a flat
//! observed forward curve f(0, T) ≡ f₀ — the simplest calibration —
//! θ(t) reduces to:
//!
//!   θ(t) = a · f₀ + σ²/(2a) · (1 − e^{−2at})
//!
//! Closed-form ZCB price (with P^M(0,t) denoting the observed market
//! discount curve):
//!
//!   B(t, T) = (1 − e^{−a(T−t)}) / a
//!   ln A(t,T) = ln(P^M(0,T)/P^M(0,t))
//!             + B(t,T) · f^M(0,t)
//!             − σ²/(4a) · (1 − e^{−2at}) · B(t,T)²
//!   P(t, T)   = A(t, T) · exp(−B(t, T) · r_t)
//!
//! Caller supplies the calibrated forward curve via a closure (or a
//! pre-tabulated `market_curve_log_p` callback that returns ln P^M(0,τ)).
//!
//! Pure compute. Companion to `vasicek` (constant-coefficient cousin)
//! and `cir` (positive-rate cousin).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct HullWhiteParams {
    pub mean_reversion: f64, // a (typical 0.05–0.15)
    pub sigma: f64,          // σ (volatility of short rate)
    pub r_t: f64,            // current short rate at time t
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct HullWhiteZcb {
    pub bond_price: f64,
    pub zero_rate: f64,
    pub a_factor: f64,
    pub b_factor: f64,
}

/// Price a ZCB at time `t` with maturity `t + tau` under Hull-White,
/// given the *flat* market forward rate `flat_forward_rate`. This is
/// the canonical demo / sanity-check calibration; for a real-curve
/// calibration, override with the full A(t,T) formula yourself.
pub fn zero_coupon_bond_flat_forward(
    params: &HullWhiteParams,
    t: f64,
    tau: f64,
    flat_forward_rate: f64,
) -> Option<HullWhiteZcb> {
    if !params.mean_reversion.is_finite()
        || params.mean_reversion <= 0.0
        || !params.sigma.is_finite()
        || params.sigma < 0.0
        || !params.r_t.is_finite()
        || !t.is_finite()
        || t < 0.0
        || !tau.is_finite()
        || tau <= 0.0
        || !flat_forward_rate.is_finite()
    {
        return None;
    }
    let a = params.mean_reversion;
    let sigma = params.sigma;
    let r = params.r_t;
    let f = flat_forward_rate;
    let big_t = t + tau;
    let b = (1.0 - (-a * tau).exp()) / a;
    // For flat forward curve: ln P^M(0, T) = −f · T.
    let log_p_now = -f * t;
    let log_p_maturity = -f * big_t;
    let f_at_t = f;
    // Hull-White A factor (from the closed-form ln A above).
    let convexity = sigma * sigma / (4.0 * a) * (1.0 - (-2.0 * a * t).exp()) * b * b;
    let ln_a = log_p_maturity - log_p_now + b * f_at_t - convexity;
    let a_factor = ln_a.exp();
    if !a_factor.is_finite() || a_factor <= 0.0 {
        return None;
    }
    let price = a_factor * (-b * r).exp();
    if !price.is_finite() || price <= 0.0 {
        return None;
    }
    let zero_rate = -price.ln() / tau;
    Some(HullWhiteZcb {
        bond_price: price,
        zero_rate,
        a_factor,
        b_factor: b,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(a: f64, sigma: f64, r: f64) -> HullWhiteParams {
        HullWhiteParams {
            mean_reversion: a,
            sigma,
            r_t: r,
        }
    }

    #[test]
    fn invalid_inputs_return_none() {
        assert!(zero_coupon_bond_flat_forward(&p(0.0, 0.01, 0.03), 0.0, 1.0, 0.03).is_none());
        assert!(zero_coupon_bond_flat_forward(&p(-0.1, 0.01, 0.03), 0.0, 1.0, 0.03).is_none());
        assert!(zero_coupon_bond_flat_forward(&p(0.1, -0.01, 0.03), 0.0, 1.0, 0.03).is_none());
        assert!(zero_coupon_bond_flat_forward(&p(0.1, 0.01, 0.03), 0.0, 0.0, 0.03).is_none());
        assert!(zero_coupon_bond_flat_forward(&p(0.1, 0.01, 0.03), 0.0, -1.0, 0.03).is_none());
        assert!(zero_coupon_bond_flat_forward(&p(f64::NAN, 0.01, 0.03), 0.0, 1.0, 0.03).is_none());
    }

    #[test]
    fn t_zero_initial_curve_matches_market() {
        // At t=0 the model bond price equals the market bond price (calibrated).
        // Flat forward f → P^M(0, T) = exp(−fT). Given r_t = f, the model
        // returns exactly that.
        let f = 0.04;
        let r = zero_coupon_bond_flat_forward(&p(0.1, 0.01, f), 0.0, 5.0, f).unwrap();
        let market = (-f * 5.0_f64).exp();
        assert!(
            (r.bond_price - market).abs() < 1e-9,
            "HW at t=0 should match market: model={} market={}",
            r.bond_price,
            market
        );
    }

    #[test]
    fn t_zero_zero_rate_equals_forward_rate_for_flat_curve() {
        let f = 0.05;
        let r = zero_coupon_bond_flat_forward(&p(0.1, 0.01, f), 0.0, 3.0, f).unwrap();
        assert!((r.zero_rate - f).abs() < 1e-9);
    }

    #[test]
    fn b_factor_increases_with_tenor() {
        let b1 = zero_coupon_bond_flat_forward(&p(0.1, 0.01, 0.03), 0.0, 1.0, 0.03)
            .unwrap()
            .b_factor;
        let b5 = zero_coupon_bond_flat_forward(&p(0.1, 0.01, 0.03), 0.0, 5.0, 0.03)
            .unwrap()
            .b_factor;
        assert!(b5 > b1);
    }

    #[test]
    fn longer_tenor_yields_lower_bond_price_under_positive_rate() {
        let p1 = zero_coupon_bond_flat_forward(&p(0.1, 0.01, 0.03), 0.0, 1.0, 0.03).unwrap();
        let p5 = zero_coupon_bond_flat_forward(&p(0.1, 0.01, 0.03), 0.0, 5.0, 0.03).unwrap();
        assert!(p5.bond_price < p1.bond_price);
    }

    #[test]
    fn higher_rate_lowers_bond_price_at_same_tenor() {
        let f = 0.03;
        let r_low = zero_coupon_bond_flat_forward(&p(0.1, 0.01, 0.02), 0.0, 5.0, f).unwrap();
        let r_high = zero_coupon_bond_flat_forward(&p(0.1, 0.01, 0.06), 0.0, 5.0, f).unwrap();
        assert!(r_high.bond_price < r_low.bond_price);
    }

    #[test]
    fn higher_mean_reversion_dampens_b_factor() {
        let r_slow = zero_coupon_bond_flat_forward(&p(0.05, 0.01, 0.03), 0.0, 5.0, 0.03).unwrap();
        let r_fast = zero_coupon_bond_flat_forward(&p(0.50, 0.01, 0.03), 0.0, 5.0, 0.03).unwrap();
        // Larger a → smaller B(t, T).
        assert!(r_fast.b_factor < r_slow.b_factor);
    }

    #[test]
    fn t_positive_with_zero_vol_collapses_to_vasicek_form() {
        // σ = 0 → no convexity correction → A(t, T) = P^M(0,T)/P^M(0,t) · exp(B·f(0,t)).
        // For flat f and matching r_t = f → bond should price at the spot-T discount.
        let f = 0.04;
        let r = zero_coupon_bond_flat_forward(&p(0.1, 0.0, f), 2.0, 3.0, f).unwrap();
        assert!((r.bond_price - (-f * 3.0_f64).exp()).abs() < 1e-9);
    }
}
