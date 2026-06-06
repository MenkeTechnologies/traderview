//! Geometric Asian option — closed-form European pricing.
//!
//! Asian options pay based on the AVERAGE underlying price over the
//! option's life. The geometric-average variant has a closed-form
//! solution; arithmetic-average requires Monte Carlo or
//! Curran/Vorst approximations (not implemented here).
//!
//! For a geometric Asian on a Black-Scholes underlying with averaging
//! from 0 to T (continuous averaging):
//!
//!   σ_avg = σ / √3
//!   q_avg = 0.5 · (r + q + σ²/6)
//!
//! Then price the option as a Black-Scholes vanilla using (σ_avg, q_avg):
//!
//!   d1 = [ln(S/K) + (r − q_avg + 0.5·σ_avg²)·T] / (σ_avg·√T)
//!   call = e^{−r·T} · [S · e^{(r − q_avg)·T} · N(d1) − K · N(d2)]
//!   put  = e^{−r·T} · [K · N(−d2) − S · e^{(r − q_avg)·T} · N(−d1)]
//!
//! Geometric Asians always price ≤ vanilla (averaging dampens vol).
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OptionKind {
    Call,
    Put,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AsianReport {
    pub price: f64,
    pub adjusted_sigma: f64,
    pub adjusted_dividend: f64,
    pub d1: f64,
    pub d2: f64,
}

pub fn price(
    spot: f64,
    strike: f64,
    time_to_expiry: f64,
    risk_free: f64,
    dividend_yield: f64,
    sigma: f64,
    kind: OptionKind,
) -> Option<AsianReport> {
    if !spot.is_finite()
        || spot <= 0.0
        || !strike.is_finite()
        || strike <= 0.0
        || !time_to_expiry.is_finite()
        || time_to_expiry <= 0.0
        || !risk_free.is_finite()
        || !dividend_yield.is_finite()
        || !sigma.is_finite()
        || sigma <= 0.0
    {
        return None;
    }
    let sigma_avg = sigma / 3.0_f64.sqrt();
    let q_avg = 0.5 * (risk_free + dividend_yield + sigma * sigma / 6.0);
    let sqrt_t = time_to_expiry.sqrt();
    let d1 = ((spot / strike).ln()
        + (risk_free - q_avg + 0.5 * sigma_avg * sigma_avg) * time_to_expiry)
        / (sigma_avg * sqrt_t);
    let d2 = d1 - sigma_avg * sqrt_t;
    let nd1 = norm_cdf(d1);
    let nd2 = norm_cdf(d2);
    let dr = (-risk_free * time_to_expiry).exp();
    let pv_factor = ((risk_free - q_avg) * time_to_expiry).exp();
    let price = match kind {
        OptionKind::Call => dr * (spot * pv_factor * nd1 - strike * nd2),
        OptionKind::Put => dr * (strike * (1.0 - nd2) - spot * pv_factor * (1.0 - nd1)),
    };
    if !price.is_finite() {
        return None;
    }
    Some(AsianReport {
        price: price.max(0.0),
        adjusted_sigma: sigma_avg,
        adjusted_dividend: q_avg,
        d1,
        d2,
    })
}

fn norm_cdf(x: f64) -> f64 {
    let a1 = 0.254829592_f64;
    let a2 = -0.284496736_f64;
    let a3 = 1.421413741_f64;
    let a4 = -1.453152027_f64;
    let a5 = 1.061405429_f64;
    let p = 0.3275911_f64;
    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let xa = x.abs() / std::f64::consts::SQRT_2;
    let t = 1.0 / (1.0 + p * xa);
    let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-xa * xa).exp();
    0.5 * (1.0 + sign * y)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vanilla_bs_call(s: f64, k: f64, t: f64, r: f64, q: f64, sigma: f64) -> f64 {
        let sqrt_t = t.sqrt();
        let d1 = ((s / k).ln() + (r - q + 0.5 * sigma * sigma) * t) / (sigma * sqrt_t);
        let d2 = d1 - sigma * sqrt_t;
        s * (-q * t).exp() * norm_cdf(d1) - k * (-r * t).exp() * norm_cdf(d2)
    }

    #[test]
    fn invalid_inputs_return_none() {
        for bad in [0.0, -1.0, f64::NAN] {
            assert!(price(bad, 100.0, 0.5, 0.05, 0.0, 0.2, OptionKind::Call).is_none());
            assert!(price(100.0, bad, 0.5, 0.05, 0.0, 0.2, OptionKind::Call).is_none());
            assert!(price(100.0, 100.0, bad, 0.05, 0.0, 0.2, OptionKind::Call).is_none());
            assert!(price(100.0, 100.0, 0.5, 0.05, 0.0, bad, OptionKind::Call).is_none());
        }
    }

    #[test]
    fn adjusted_sigma_is_sigma_over_sqrt3() {
        let r = price(100.0, 100.0, 0.5, 0.05, 0.0, 0.30, OptionKind::Call).unwrap();
        let expected = 0.30 / 3.0_f64.sqrt();
        assert!((r.adjusted_sigma - expected).abs() < 1e-12);
    }

    #[test]
    fn at_the_money_call_price_positive() {
        let r = price(100.0, 100.0, 0.5, 0.05, 0.0, 0.20, OptionKind::Call).unwrap();
        assert!(r.price > 0.0);
    }

    #[test]
    fn put_call_parity_holds_approximately() {
        // For geometric Asian: c − p = e^{−rT} · (S · e^{(r−q_avg)T} − K).
        let s = 100.0;
        let k = 100.0;
        let t = 0.5;
        let r = 0.05;
        let q = 0.02;
        let sigma = 0.25;
        let c = price(s, k, t, r, q, sigma, OptionKind::Call).unwrap().price;
        let p = price(s, k, t, r, q, sigma, OptionKind::Put).unwrap().price;
        let q_avg = 0.5 * (r + q + sigma * sigma / 6.0);
        let parity = (-r * t).exp() * (s * ((r - q_avg) * t).exp() - k);
        assert!((c - p - parity).abs() < 1e-9);
    }

    #[test]
    fn geometric_asian_priced_below_vanilla() {
        // Averaging always reduces vol → Asian < vanilla for ATM call (no skew).
        let s = 100.0;
        let k = 100.0;
        let t = 1.0;
        let r = 0.05;
        let q = 0.0;
        let sigma = 0.30;
        let asian = price(s, k, t, r, q, sigma, OptionKind::Call).unwrap().price;
        let vanilla = vanilla_bs_call(s, k, t, r, q, sigma);
        assert!(
            asian < vanilla,
            "Asian {asian} should be below vanilla {vanilla}"
        );
    }

    #[test]
    fn higher_vol_yields_higher_atm_asian_price() {
        let r_low = price(100.0, 100.0, 1.0, 0.05, 0.0, 0.10, OptionKind::Call).unwrap();
        let r_high = price(100.0, 100.0, 1.0, 0.05, 0.0, 0.40, OptionKind::Call).unwrap();
        assert!(r_high.price > r_low.price);
    }

    #[test]
    fn put_price_positive() {
        let r = price(100.0, 100.0, 0.5, 0.05, 0.0, 0.20, OptionKind::Put).unwrap();
        assert!(r.price >= 0.0);
    }

    #[test]
    fn longer_time_yields_higher_atm_call() {
        let r_short = price(100.0, 100.0, 0.10, 0.05, 0.0, 0.20, OptionKind::Call).unwrap();
        let r_long = price(100.0, 100.0, 1.00, 0.05, 0.0, 0.20, OptionKind::Call).unwrap();
        assert!(r_long.price > r_short.price);
    }
}
