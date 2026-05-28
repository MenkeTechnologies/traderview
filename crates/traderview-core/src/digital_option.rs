//! Digital (binary) options — cash-or-nothing + asset-or-nothing,
//! European, Black-Scholes closed form.
//!
//! Cash-or-nothing call: pays `cash` if S_T > K, else 0.
//! Cash-or-nothing put : pays `cash` if S_T < K, else 0.
//! Asset-or-nothing call: pays S_T if S_T > K, else 0.
//! Asset-or-nothing put : pays S_T if S_T < K, else 0.
//!
//! Classic relationship: a vanilla call = asset-or-nothing call
//! − cash-or-nothing call (with cash = K).
//!
//! Pricing:
//!   d1 = [ln(S/K) + (r − q + 0.5σ²)·T] / (σ·√T)
//!   d2 = d1 − σ·√T
//!   cash_or_nothing_call = cash · e^{−rT} · N(d2)
//!   cash_or_nothing_put  = cash · e^{−rT} · N(−d2)
//!   asset_or_nothing_call = S · e^{−qT} · N(d1)
//!   asset_or_nothing_put  = S · e^{−qT} · N(−d1)
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DigitalKind {
    CashOrNothingCall,
    CashOrNothingPut,
    AssetOrNothingCall,
    AssetOrNothingPut,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DigitalReport {
    pub price: f64,
    pub d1: f64,
    pub d2: f64,
}

#[allow(clippy::too_many_arguments)]    // Canonical BS-style signature; struct-wrapping adds construction noise.
pub fn price(
    spot: f64, strike: f64,
    time_to_expiry: f64,
    risk_free: f64, dividend_yield: f64,
    sigma: f64,
    cash: f64,
    kind: DigitalKind,
) -> Option<DigitalReport> {
    if !spot.is_finite() || spot <= 0.0
        || !strike.is_finite() || strike <= 0.0
        || !time_to_expiry.is_finite() || time_to_expiry <= 0.0
        || !risk_free.is_finite() || !dividend_yield.is_finite()
        || !sigma.is_finite() || sigma <= 0.0
        || !cash.is_finite() || cash < 0.0
    {
        return None;
    }
    let sqrt_t = time_to_expiry.sqrt();
    let d1 = ((spot / strike).ln()
        + (risk_free - dividend_yield + 0.5 * sigma * sigma) * time_to_expiry)
        / (sigma * sqrt_t);
    let d2 = d1 - sigma * sqrt_t;
    let dr = (-risk_free * time_to_expiry).exp();
    let dq = (-dividend_yield * time_to_expiry).exp();
    let p = match kind {
        DigitalKind::CashOrNothingCall => cash * dr * norm_cdf(d2),
        DigitalKind::CashOrNothingPut  => cash * dr * norm_cdf(-d2),
        DigitalKind::AssetOrNothingCall => spot * dq * norm_cdf(d1),
        DigitalKind::AssetOrNothingPut  => spot * dq * norm_cdf(-d1),
    };
    if !p.is_finite() { return None; }
    Some(DigitalReport { price: p.max(0.0), d1, d2 })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_none() {
        for bad in [0.0, -1.0, f64::NAN] {
            assert!(price(bad, 100.0, 0.5, 0.05, 0.0, 0.2, 1.0,
                DigitalKind::CashOrNothingCall).is_none());
            assert!(price(100.0, bad, 0.5, 0.05, 0.0, 0.2, 1.0,
                DigitalKind::CashOrNothingCall).is_none());
            assert!(price(100.0, 100.0, bad, 0.05, 0.0, 0.2, 1.0,
                DigitalKind::CashOrNothingCall).is_none());
            assert!(price(100.0, 100.0, 0.5, 0.05, 0.0, bad, 1.0,
                DigitalKind::CashOrNothingCall).is_none());
        }
        assert!(price(100.0, 100.0, 0.5, 0.05, 0.0, 0.2, -1.0,
            DigitalKind::CashOrNothingCall).is_none());
    }

    #[test]
    fn cash_or_nothing_call_put_sum_to_discounted_cash() {
        // CN call + CN put = cash · e^{−rT} (option pays the cash either way).
        let s = 100.0; let k = 105.0; let t = 0.5; let r = 0.05; let q = 0.0; let v = 0.20;
        let cash = 1.0;
        let c = price(s, k, t, r, q, v, cash, DigitalKind::CashOrNothingCall).unwrap();
        let p = price(s, k, t, r, q, v, cash, DigitalKind::CashOrNothingPut).unwrap();
        let parity = cash * (-r * t).exp();
        assert!((c.price + p.price - parity).abs() < 1e-9);
    }

    #[test]
    fn asset_or_nothing_call_put_sum_to_discounted_spot() {
        // AN call + AN put = S · e^{−qT}.
        let s = 100.0; let k = 105.0; let t = 0.5; let r = 0.05; let q = 0.02; let v = 0.20;
        let c = price(s, k, t, r, q, v, 0.0, DigitalKind::AssetOrNothingCall).unwrap();
        let p = price(s, k, t, r, q, v, 0.0, DigitalKind::AssetOrNothingPut).unwrap();
        let parity = s * (-q * t).exp();
        assert!((c.price + p.price - parity).abs() < 1e-9);
    }

    #[test]
    fn vanilla_call_replication_holds() {
        // Vanilla call = asset-or-nothing call − cash-or-nothing call (cash = K).
        let s = 100.0; let k = 95.0; let t = 0.5; let r = 0.05; let q = 0.0; let v = 0.20;
        let an_c = price(s, k, t, r, q, v, 0.0, DigitalKind::AssetOrNothingCall).unwrap();
        let cn_c = price(s, k, t, r, q, v, k, DigitalKind::CashOrNothingCall).unwrap();
        let synth = an_c.price - cn_c.price;
        // Vanilla call via Black-Scholes for comparison.
        let sqrt_t = t.sqrt();
        let d1 = ((s / k).ln() + (r - q + 0.5 * v * v) * t) / (v * sqrt_t);
        let d2 = d1 - v * sqrt_t;
        let bs_call = s * (-q * t).exp() * norm_cdf(d1) - k * (-r * t).exp() * norm_cdf(d2);
        assert!((synth - bs_call).abs() < 1e-9,
            "synth call {synth} should match BS {bs_call}");
    }

    #[test]
    fn deep_itm_cash_or_nothing_call_approaches_discounted_cash() {
        // S >> K → N(d2) ≈ 1 → CN call ≈ cash · e^{−rT}.
        let s = 200.0; let k = 100.0; let t = 0.25; let r = 0.05; let q = 0.0; let v = 0.20;
        let r1 = price(s, k, t, r, q, v, 1.0, DigitalKind::CashOrNothingCall).unwrap();
        let max_value = 1.0 * (-r * t).exp();
        assert!((r1.price - max_value).abs() < 0.001);
    }

    #[test]
    fn deep_otm_cash_or_nothing_call_approaches_zero() {
        let r = price(50.0, 100.0, 0.25, 0.05, 0.0, 0.20, 1.0,
            DigitalKind::CashOrNothingCall).unwrap();
        assert!(r.price < 0.001);
    }

    #[test]
    fn cash_zero_yields_zero_cash_or_nothing_price() {
        let r = price(100.0, 100.0, 0.5, 0.05, 0.0, 0.20, 0.0,
            DigitalKind::CashOrNothingCall).unwrap();
        assert_eq!(r.price, 0.0);
    }
}
