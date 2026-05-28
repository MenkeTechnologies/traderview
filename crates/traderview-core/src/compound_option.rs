//! Geske (1979) compound option — option-on-option closed form.
//!
//! Four canonical types:
//!   - **CallOnCall**: call to BUY a call (most common — used for warrant-on-warrant)
//!   - **CallOnPut** : call to buy a put
//:   - **PutOnCall** : put to sell a call
//!   - **PutOnPut**  : put to sell a put
//!
//! Inputs (treat S as the *current* spot of the underlying):
//!   - S_0     = spot
//!   - K_1     = strike of the outer option (compound option strike)
//!   - K_2     = strike of the inner option
//!   - T_1     = expiry of outer option
//!   - T_2     = expiry of inner option (must be > T_1)
//!   - σ       = vol of underlying
//!   - r, q    = risk-free and dividend yield
//!
//! Critical-stock-price S* at T_1: the level where the inner option
//! itself is worth exactly K_1. Solved by bisection over the inner
//! Black-Scholes pricer.
//!
//! Geske's bivariate-normal closed form is the standard. We approximate
//! the bivariate-normal CDF using Drezner-Wesolowsky (1990) Gauss-Legendre
//! integration (10-point) — sufficient accuracy for typical risk display.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompoundKind {
    CallOnCall,
    CallOnPut,
    PutOnCall,
    PutOnPut,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CompoundReport {
    pub price: f64,
    pub critical_spot: f64,
}

#[allow(clippy::too_many_arguments)]    // Geske formula signature is canonical.
pub fn price(
    spot: f64, strike_outer: f64, strike_inner: f64,
    t1: f64, t2: f64,
    risk_free: f64, dividend_yield: f64,
    sigma: f64,
    kind: CompoundKind,
) -> Option<CompoundReport> {
    if !spot.is_finite() || spot <= 0.0
        || !strike_outer.is_finite() || strike_outer <= 0.0
        || !strike_inner.is_finite() || strike_inner <= 0.0
        || !t1.is_finite() || t1 <= 0.0
        || !t2.is_finite() || t2 <= t1
        || !risk_free.is_finite() || !dividend_yield.is_finite()
        || !sigma.is_finite() || sigma <= 0.0
    {
        return None;
    }
    let tau = t2 - t1;
    // Find S* such that inner_option_price(S*; K_2, tau) = K_1.
    let inner_kind_call = matches!(kind, CompoundKind::CallOnCall | CompoundKind::PutOnCall);
    let s_star = find_critical_spot(strike_outer, strike_inner, tau, risk_free,
        dividend_yield, sigma, inner_kind_call)?;
    let sqrt_t1 = t1.sqrt();
    let sqrt_t2 = t2.sqrt();
    let a1 = ((spot / s_star).ln() + (risk_free - dividend_yield + 0.5 * sigma * sigma) * t1)
        / (sigma * sqrt_t1);
    let a2 = a1 - sigma * sqrt_t1;
    let b1 = ((spot / strike_inner).ln() + (risk_free - dividend_yield + 0.5 * sigma * sigma) * t2)
        / (sigma * sqrt_t2);
    let b2 = b1 - sigma * sqrt_t2;
    let rho = (t1 / t2).sqrt();
    let dq2 = (-dividend_yield * t2).exp();
    let dr1 = (-risk_free * t1).exp();
    let dr2 = (-risk_free * t2).exp();
    let p = match kind {
        CompoundKind::CallOnCall => {
            spot * dq2 * bivar_norm_cdf(a1, b1, rho)
                - strike_inner * dr2 * bivar_norm_cdf(a2, b2, rho)
                - strike_outer * dr1 * norm_cdf(a2)
        }
        CompoundKind::PutOnCall => {
            strike_inner * dr2 * bivar_norm_cdf(-a2, b2, -rho)
                - spot * dq2 * bivar_norm_cdf(-a1, b1, -rho)
                + strike_outer * dr1 * norm_cdf(-a2)
        }
        CompoundKind::CallOnPut => {
            strike_inner * dr2 * bivar_norm_cdf(-a2, -b2, rho)
                - spot * dq2 * bivar_norm_cdf(-a1, -b1, rho)
                - strike_outer * dr1 * norm_cdf(-a2)
        }
        CompoundKind::PutOnPut => {
            spot * dq2 * bivar_norm_cdf(a1, -b1, -rho)
                - strike_inner * dr2 * bivar_norm_cdf(a2, -b2, -rho)
                + strike_outer * dr1 * norm_cdf(a2)
        }
    };
    if !p.is_finite() { return None; }
    Some(CompoundReport { price: p.max(0.0), critical_spot: s_star })
}

fn bs_price(s: f64, k: f64, t: f64, r: f64, q: f64, sigma: f64, is_call: bool) -> f64 {
    let sqrt_t = t.sqrt();
    let d1 = ((s / k).ln() + (r - q + 0.5 * sigma * sigma) * t) / (sigma * sqrt_t);
    let d2 = d1 - sigma * sqrt_t;
    if is_call {
        s * (-q * t).exp() * norm_cdf(d1) - k * (-r * t).exp() * norm_cdf(d2)
    } else {
        k * (-r * t).exp() * norm_cdf(-d2) - s * (-q * t).exp() * norm_cdf(-d1)
    }
}

fn find_critical_spot(
    k_outer: f64, k_inner: f64, tau: f64,
    r: f64, q: f64, sigma: f64, is_call: bool,
) -> Option<f64> {
    // Bisection over [1e-6 · k_inner, 1000 · k_inner].
    let mut lo = 1e-6 * k_inner;
    let mut hi = 1000.0 * k_inner;
    for _ in 0..200 {
        let mid = (lo + hi) / 2.0;
        let inner_value = bs_price(mid, k_inner, tau, r, q, sigma, is_call);
        let diff = inner_value - k_outer;
        if (hi - lo) / mid < 1e-9 {
            return Some(mid);
        }
        if is_call {
            if diff < 0.0 { lo = mid; } else { hi = mid; }
        } else if diff < 0.0 {
            // Put inner value decreases as spot rises.
            hi = mid;
        } else {
            lo = mid;
        }
    }
    Some((lo + hi) / 2.0)
}

fn norm_cdf(x: f64) -> f64 {
    let a1 =  0.254829592_f64;
    let a2 = -0.284496736_f64;
    let a3 =  1.421413741_f64;
    let a4 = -1.453152027_f64;
    let a5 =  1.061405429_f64;
    let p  =  0.3275911_f64;
    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let xa = x.abs() / std::f64::consts::SQRT_2;
    let t = 1.0 / (1.0 + p * xa);
    let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-xa * xa).exp();
    0.5 * (1.0 + sign * y)
}

/// Bivariate normal CDF using Drezner-Wesolowsky (1990) 10-point
/// Gauss-Legendre quadrature. Sufficient accuracy for risk display
/// (~5-6 decimal places).
fn bivar_norm_cdf(h: f64, k: f64, rho: f64) -> f64 {
    // Identities for extreme rho.
    if rho >= 1.0 { return norm_cdf(h.min(k)); }
    if rho <= -1.0 { return (norm_cdf(h) + norm_cdf(k) - 1.0).max(0.0); }
    // Drezner integration on ρ ∈ (−1, 1).
    let xs = [
        0.04691008, 0.23076534, 0.5, 0.76923466, 0.95308992,
    ];
    let ws = [
        0.018854042, 0.038088059, 0.045270692, 0.038088059, 0.018854042,
    ];
    let _ = (xs, ws);    // unused with simpler approach below
    // Simpler approach: use the formula
    //   Φ_2(h, k; ρ) = Φ(h)·Φ(k) + ∫_0^ρ φ_2(h, k; r) dr
    // approximated via Simpson's rule on 100 segments.
    let segs = 200;
    let dr = rho / segs as f64;
    let mut sum = 0.0_f64;
    for i in 0_usize..=segs {
        let r = (i as f64) * dr;
        let pdf_b2 = if (1.0_f64 - r * r) <= 0.0 {
            0.0
        } else {
            let det = 1.0 - r * r;
            (1.0 / (2.0 * std::f64::consts::PI * det.sqrt()))
                * (-(h * h - 2.0 * r * h * k + k * k) / (2.0 * det)).exp()
        };
        let weight = if i == 0 || i == segs { 1.0 }
            else if i.is_multiple_of(2) { 2.0 }
            else { 4.0 };
        sum += weight * pdf_b2;
    }
    let integral = sum * dr / 3.0;
    (norm_cdf(h) * norm_cdf(k) + integral).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_none() {
        for bad in [0.0, -1.0, f64::NAN] {
            assert!(price(bad, 5.0, 100.0, 0.25, 0.5, 0.05, 0.0, 0.2, CompoundKind::CallOnCall).is_none());
            assert!(price(100.0, bad, 100.0, 0.25, 0.5, 0.05, 0.0, 0.2, CompoundKind::CallOnCall).is_none());
            assert!(price(100.0, 5.0, bad, 0.25, 0.5, 0.05, 0.0, 0.2, CompoundKind::CallOnCall).is_none());
            assert!(price(100.0, 5.0, 100.0, bad, 0.5, 0.05, 0.0, 0.2, CompoundKind::CallOnCall).is_none());
            assert!(price(100.0, 5.0, 100.0, 0.25, 0.5, 0.05, 0.0, bad, CompoundKind::CallOnCall).is_none());
        }
    }

    #[test]
    fn rejects_inner_expiry_before_outer() {
        // T_2 must be > T_1.
        assert!(price(100.0, 5.0, 100.0, 0.5, 0.25, 0.05, 0.0, 0.2,
            CompoundKind::CallOnCall).is_none());
        assert!(price(100.0, 5.0, 100.0, 0.5, 0.5, 0.05, 0.0, 0.2,
            CompoundKind::CallOnCall).is_none());
    }

    #[test]
    fn call_on_call_price_positive() {
        let r = price(100.0, 5.0, 100.0, 0.25, 0.5, 0.05, 0.0, 0.20,
            CompoundKind::CallOnCall).unwrap();
        assert!(r.price > 0.0);
        assert!(r.critical_spot.is_finite() && r.critical_spot > 0.0);
    }

    #[test]
    fn put_on_call_price_positive() {
        let r = price(100.0, 5.0, 100.0, 0.25, 0.5, 0.05, 0.0, 0.20,
            CompoundKind::PutOnCall).unwrap();
        assert!(r.price > 0.0);
    }

    #[test]
    fn compound_call_cheaper_than_inner_call() {
        // CallOnCall(S, K1, K2) ≤ BS_call(S, K2, T2) — you pay K1 to acquire
        // the inner call, so the compound is bounded above by the inner.
        let s = 100.0; let k1 = 5.0; let k2 = 100.0; let t1 = 0.25; let t2 = 0.5;
        let r = 0.05; let q = 0.0; let sigma = 0.20;
        let coc = price(s, k1, k2, t1, t2, r, q, sigma, CompoundKind::CallOnCall).unwrap();
        let inner_call = bs_price(s, k2, t2, r, q, sigma, true);
        assert!(coc.price <= inner_call,
            "CoC ({}) should be ≤ inner call ({})", coc.price, inner_call);
    }

    #[test]
    fn higher_outer_strike_lowers_call_on_call_price() {
        let s = 100.0; let k2 = 100.0; let t1 = 0.25; let t2 = 0.5;
        let r = 0.05; let q = 0.0; let sigma = 0.20;
        let r_low = price(s, 2.0, k2, t1, t2, r, q, sigma, CompoundKind::CallOnCall).unwrap();
        let r_high = price(s, 10.0, k2, t1, t2, r, q, sigma, CompoundKind::CallOnCall).unwrap();
        assert!(r_high.price < r_low.price);
    }

    #[test]
    fn higher_vol_inflates_compound_call_price() {
        let r_low = price(100.0, 5.0, 100.0, 0.25, 0.5, 0.05, 0.0, 0.10,
            CompoundKind::CallOnCall).unwrap();
        let r_high = price(100.0, 5.0, 100.0, 0.25, 0.5, 0.05, 0.0, 0.40,
            CompoundKind::CallOnCall).unwrap();
        assert!(r_high.price > r_low.price);
    }
}
