//! Bachelier (1900) normal-Black-Scholes — European option on a NORMAL
//! (additive) underlying.
//!
//! Used canonically for:
//!   - Interest-rate options post-2020 (rates can go negative; lognormal
//!     model breaks down)
//!   - Swap rate options
//!   - Commodity options where price diffuses additively (e.g. spread
//!     option on two physically-similar grades)
//!
//! Formula:
//!   d = (F − K) / (σ · √T)
//!   call = e^{−r·T} · [(F − K) · N(d) + σ·√T · φ(d)]
//!   put  = e^{−r·T} · [(K − F) · N(−d) + σ·√T · φ(d)]
//!
//! σ here is the *normal* volatility (price units, not log-units).
//! Caller can convert from lognormal vol via σ_n ≈ F · σ_ln (good for
//! near-ATM, well-defined for any rate).
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OptionKind { Call, Put }

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BachelierReport {
    pub price: f64,
    pub d: f64,
    pub delta: f64,
    pub vega: f64,
}

#[allow(clippy::too_many_arguments)]
pub fn price(
    forward: f64, strike: f64,
    time_to_expiry: f64, risk_free: f64,
    normal_sigma: f64,
    kind: OptionKind,
) -> Option<BachelierReport> {
    if !forward.is_finite() || !strike.is_finite()
        || !time_to_expiry.is_finite() || time_to_expiry <= 0.0
        || !risk_free.is_finite()
        || !normal_sigma.is_finite() || normal_sigma <= 0.0
    {
        return None;
    }
    let sqrt_t = time_to_expiry.sqrt();
    let sigma_sqrt_t = normal_sigma * sqrt_t;
    let d = (forward - strike) / sigma_sqrt_t;
    let nd = norm_cdf(d);
    let pd = norm_pdf(d);
    let dr = (-risk_free * time_to_expiry).exp();
    let (intrinsic, sign) = match kind {
        OptionKind::Call => ((forward - strike) * nd, 1.0_f64),
        OptionKind::Put  => ((strike - forward) * norm_cdf(-d), -1.0),
    };
    let price = dr * (intrinsic + sigma_sqrt_t * pd);
    if !price.is_finite() { return None; }
    let delta = sign * dr * (match kind { OptionKind::Call => nd, OptionKind::Put => norm_cdf(-d) });
    let vega = dr * sqrt_t * pd;
    Some(BachelierReport { price: price.max(0.0), d, delta, vega })
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

fn norm_pdf(x: f64) -> f64 {
    (-0.5 * x * x).exp() / (2.0 * std::f64::consts::PI).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_none() {
        for bad in [0.0, -1.0, f64::NAN] {
            assert!(price(100.0, 100.0, bad, 0.05, 1.0, OptionKind::Call).is_none());
            assert!(price(100.0, 100.0, 0.5, 0.05, bad, OptionKind::Call).is_none());
        }
        assert!(price(f64::NAN, 100.0, 0.5, 0.05, 1.0, OptionKind::Call).is_none());
        assert!(price(100.0, f64::NAN, 0.5, 0.05, 1.0, OptionKind::Call).is_none());
    }

    #[test]
    fn handles_negative_forward_rate() {
        // Distinctive Bachelier capability — vanilla Black-Scholes can't.
        let r = price(-0.005, 0.0, 0.5, 0.0, 0.01, OptionKind::Call).unwrap();
        assert!(r.price >= 0.0 && r.price.is_finite());
    }

    #[test]
    fn at_the_money_call_equals_sigma_sqrt_t_over_sqrt_2pi() {
        // d = 0 → call = e^{−rT} · σ·√T · φ(0) = σ·√T / √(2π) (when r=0).
        let f = 100.0; let k = 100.0; let t = 1.0; let sigma = 5.0;
        let r = price(f, k, t, 0.0, sigma, OptionKind::Call).unwrap();
        let expected = sigma * t.sqrt() / (2.0 * std::f64::consts::PI).sqrt();
        assert!((r.price - expected).abs() < 1e-9);
    }

    #[test]
    fn put_call_parity_holds() {
        // Bachelier parity: c − p = (F − K) · e^{−rT}.
        let f = 100.0; let k = 95.0; let t = 0.5; let r = 0.05; let sigma = 5.0;
        let c = price(f, k, t, r, sigma, OptionKind::Call).unwrap();
        let p = price(f, k, t, r, sigma, OptionKind::Put).unwrap();
        let parity = (f - k) * (-r * t).exp();
        assert!((c.price - p.price - parity).abs() < 1e-9);
    }

    #[test]
    fn higher_normal_vol_inflates_atm_call() {
        let r_low = price(100.0, 100.0, 1.0, 0.05, 1.0, OptionKind::Call).unwrap();
        let r_high = price(100.0, 100.0, 1.0, 0.05, 10.0, OptionKind::Call).unwrap();
        assert!(r_high.price > r_low.price);
    }

    #[test]
    fn deep_itm_call_approaches_intrinsic_discounted() {
        let f = 200.0; let k = 100.0; let t = 0.25; let r = 0.05; let sigma = 1.0;
        let res = price(f, k, t, r, sigma, OptionKind::Call).unwrap();
        let intrinsic = (f - k) * (-r * t).exp();
        assert!((res.price - intrinsic).abs() < 0.5);
    }

    #[test]
    fn vega_matches_finite_difference() {
        let f = 100.0; let k = 100.0; let t = 0.5; let r = 0.05; let sigma = 5.0;
        let analytic = price(f, k, t, r, sigma, OptionKind::Call).unwrap().vega;
        let bump = 1e-4;
        let up = price(f, k, t, r, sigma + bump, OptionKind::Call).unwrap().price;
        let dn = price(f, k, t, r, sigma - bump, OptionKind::Call).unwrap().price;
        let fd = (up - dn) / (2.0 * bump);
        assert!((analytic - fd).abs() / analytic.abs() < 1e-4);
    }

    #[test]
    fn put_delta_negative() {
        let r = price(100.0, 100.0, 0.5, 0.05, 5.0, OptionKind::Put).unwrap();
        assert!(r.delta < 0.0 && r.delta > -1.0);
    }
}
