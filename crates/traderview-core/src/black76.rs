//! Black-76 pricing for European options on futures / forwards.
//!
//! Unlike Black-Scholes (on spot with dividend yield), Black-76 prices
//! options whose underlying is a forward/futures contract F:
//!
//!   d1 = [ln(F/K) + 0.5·σ²·T] / (σ·√T)
//!   d2 = d1 − σ·√T
//!   call = e^{−rT} · [F · N(d1) − K · N(d2)]
//!   put  = e^{−rT} · [K · N(−d2) − F · N(−d1)]
//!
//! No dividend yield (the futures price already reflects the cost of
//! carry). Used canonically for: bond options, swaption pricing
//! (re-cast as Black-76 with annuity numeraire), interest-rate caps/
//! floors, energy options, agricultural options.
//!
//! Pure compute. Returns price + greeks (delta, gamma, vega, theta).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OptionKind { Call, Put }

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Black76Output {
    pub price: f64,
    pub delta: f64,
    pub gamma: f64,
    pub vega: f64,
    pub theta: f64,
}

pub fn price(forward: f64, strike: f64, time_to_expiry: f64, risk_free: f64,
    sigma: f64, kind: OptionKind) -> Option<Black76Output> {
    if !forward.is_finite() || forward <= 0.0
        || !strike.is_finite() || strike <= 0.0
        || !time_to_expiry.is_finite() || time_to_expiry <= 0.0
        || !risk_free.is_finite()
        || !sigma.is_finite() || sigma <= 0.0
    {
        return None;
    }
    let sqrt_t = time_to_expiry.sqrt();
    let d1 = ((forward / strike).ln() + 0.5 * sigma * sigma * time_to_expiry) / (sigma * sqrt_t);
    let d2 = d1 - sigma * sqrt_t;
    let nd1 = norm_cdf(d1);
    let nd2 = norm_cdf(d2);
    let pdf_d1 = (-0.5 * d1 * d1).exp() / (2.0 * std::f64::consts::PI).sqrt();
    let dr = (-risk_free * time_to_expiry).exp();
    let (price, delta) = match kind {
        OptionKind::Call => (
            dr * (forward * nd1 - strike * nd2),
            dr * nd1,
        ),
        OptionKind::Put => (
            dr * (strike * (1.0 - nd2) - forward * (1.0 - nd1)),
            -dr * (1.0 - nd1),
        ),
    };
    let gamma = dr * pdf_d1 / (forward * sigma * sqrt_t);
    let vega = dr * forward * pdf_d1 * sqrt_t;
    let theta = match kind {
        OptionKind::Call => -dr * forward * pdf_d1 * sigma / (2.0 * sqrt_t)
            - risk_free * (-(price - dr * forward * nd1 + dr * strike * nd2)),
        OptionKind::Put => -dr * forward * pdf_d1 * sigma / (2.0 * sqrt_t)
            + risk_free * (dr * (strike * (1.0 - nd2) - forward * (1.0 - nd1))),
    };
    if [price, delta, gamma, vega, theta].iter().all(|x| x.is_finite()) {
        Some(Black76Output { price, delta, gamma, vega, theta })
    } else {
        None
    }
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
        for bad in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert!(price(bad, 100.0, 0.25, 0.05, 0.2, OptionKind::Call).is_none(), "F={bad}");
            assert!(price(100.0, bad, 0.25, 0.05, 0.2, OptionKind::Call).is_none(), "K={bad}");
            assert!(price(100.0, 100.0, bad, 0.05, 0.2, OptionKind::Call).is_none(), "T={bad}");
            assert!(price(100.0, 100.0, 0.25, 0.05, bad, OptionKind::Call).is_none(), "σ={bad}");
        }
    }

    #[test]
    fn put_call_parity_holds() {
        // Black-76 parity: call − put = e^{−rT} · (F − K).
        let f = 100.0; let k = 95.0; let t = 0.5; let r = 0.05; let sigma = 0.25;
        let c = price(f, k, t, r, sigma, OptionKind::Call).unwrap();
        let p = price(f, k, t, r, sigma, OptionKind::Put).unwrap();
        let parity = (-r * t).exp() * (f - k);
        assert!((c.price - p.price - parity).abs() < 1e-9);
    }

    #[test]
    fn at_the_money_call_price_positive() {
        let r = price(100.0, 100.0, 0.25, 0.05, 0.20, OptionKind::Call).unwrap();
        assert!(r.price > 0.0);
        assert!(r.delta > 0.0 && r.delta < 1.0);
        assert!(r.gamma > 0.0);
        assert!(r.vega > 0.0);
    }

    #[test]
    fn deep_itm_call_delta_close_to_one_with_discount() {
        // Deep ITM (F >> K) → delta ≈ e^{-rT} · 1.
        let r = price(200.0, 100.0, 0.25, 0.05, 0.20, OptionKind::Call).unwrap();
        let expected_delta = (-0.05_f64 * 0.25).exp();
        assert!((r.delta - expected_delta).abs() < 0.01);
    }

    #[test]
    fn put_delta_negative() {
        let r = price(100.0, 100.0, 0.25, 0.05, 0.20, OptionKind::Put).unwrap();
        assert!(r.delta < 0.0 && r.delta > -1.0);
    }

    #[test]
    fn longer_time_yields_higher_atm_call_price() {
        let r_short = price(100.0, 100.0, 0.10, 0.05, 0.20, OptionKind::Call).unwrap();
        let r_long  = price(100.0, 100.0, 1.00, 0.05, 0.20, OptionKind::Call).unwrap();
        assert!(r_long.price > r_short.price);
    }

    #[test]
    fn higher_vol_yields_higher_atm_call_price() {
        let r_low  = price(100.0, 100.0, 0.25, 0.05, 0.10, OptionKind::Call).unwrap();
        let r_high = price(100.0, 100.0, 0.25, 0.05, 0.40, OptionKind::Call).unwrap();
        assert!(r_high.price > r_low.price);
    }

    #[test]
    fn vega_matches_finite_difference() {
        // vega = ∂price/∂σ — verify with central difference.
        let (f, k, t, r, sigma) = (100.0, 100.0, 0.25, 0.05, 0.20);
        let analytic = price(f, k, t, r, sigma, OptionKind::Call).unwrap().vega;
        let bump = 1e-4;
        let up = price(f, k, t, r, sigma + bump, OptionKind::Call).unwrap().price;
        let dn = price(f, k, t, r, sigma - bump, OptionKind::Call).unwrap().price;
        let fd = (up - dn) / (2.0 * bump);
        assert!((analytic - fd).abs() / analytic.abs() < 1e-4,
            "vega mismatch: analytic={analytic}, fd={fd}");
    }
}
