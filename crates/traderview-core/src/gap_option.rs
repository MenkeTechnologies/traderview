//! Gap option — Black-Scholes closed form with separate trigger and
//! settlement strikes.
//!
//! Pays (S_T − K_settle) if S_T > K_trigger (call) or
//!     (K_settle − S_T) if S_T < K_trigger (put).
//!
//! Unlike a vanilla option, the payoff is NOT bounded below by zero —
//! it can be negative when S_T crosses the trigger but is below the
//! settlement strike. Negative payoffs make gap options useful for
//! synthesizing range-bound bets.
//!
//! Closed form:
//!   d1 = [ln(S/K_trigger) + (r − q + 0.5·σ²)·T] / (σ·√T)
//!   d2 = d1 − σ·√T
//!   call = S · e^{−qT} · N(d1) − K_settle · e^{−rT} · N(d2)
//!   put  = K_settle · e^{−rT} · N(−d2) − S · e^{−qT} · N(−d1)
//!
//! Note: When K_trigger == K_settle the formula collapses to vanilla
//! Black-Scholes. Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OptionKind {
    Call,
    Put,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GapOptionReport {
    pub price: f64,
    pub d1: f64,
    pub d2: f64,
}

#[allow(clippy::too_many_arguments)] // canonical signature
pub fn price(
    spot: f64,
    strike_trigger: f64,
    strike_settlement: f64,
    time_to_expiry: f64,
    risk_free: f64,
    dividend_yield: f64,
    sigma: f64,
    kind: OptionKind,
) -> Option<GapOptionReport> {
    if !spot.is_finite()
        || spot <= 0.0
        || !strike_trigger.is_finite()
        || strike_trigger <= 0.0
        || !strike_settlement.is_finite()
        || strike_settlement <= 0.0
        || !time_to_expiry.is_finite()
        || time_to_expiry <= 0.0
        || !risk_free.is_finite()
        || !dividend_yield.is_finite()
        || !sigma.is_finite()
        || sigma <= 0.0
    {
        return None;
    }
    let sqrt_t = time_to_expiry.sqrt();
    let d1 = ((spot / strike_trigger).ln()
        + (risk_free - dividend_yield + 0.5 * sigma * sigma) * time_to_expiry)
        / (sigma * sqrt_t);
    let d2 = d1 - sigma * sqrt_t;
    let nd1 = norm_cdf(d1);
    let nd2 = norm_cdf(d2);
    let dr = (-risk_free * time_to_expiry).exp();
    let dq = (-dividend_yield * time_to_expiry).exp();
    let p = match kind {
        OptionKind::Call => spot * dq * nd1 - strike_settlement * dr * nd2,
        OptionKind::Put => strike_settlement * dr * (1.0 - nd2) - spot * dq * (1.0 - nd1),
    };
    if !p.is_finite() {
        return None;
    }
    // Gap option price may legitimately be NEGATIVE (e.g. ITM trigger
    // far above K_settle — payoff is bounded above but unbounded below
    // in the small region). Do NOT clamp to zero.
    Some(GapOptionReport { price: p, d1, d2 })
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

    #[test]
    fn invalid_inputs_return_none() {
        for bad in [0.0, -1.0, f64::NAN] {
            assert!(price(bad, 100.0, 100.0, 0.5, 0.05, 0.0, 0.2, OptionKind::Call).is_none());
            assert!(price(100.0, bad, 100.0, 0.5, 0.05, 0.0, 0.2, OptionKind::Call).is_none());
            assert!(price(100.0, 100.0, bad, 0.5, 0.05, 0.0, 0.2, OptionKind::Call).is_none());
            assert!(price(100.0, 100.0, 100.0, bad, 0.05, 0.0, 0.2, OptionKind::Call).is_none());
            assert!(price(100.0, 100.0, 100.0, 0.5, 0.05, 0.0, bad, OptionKind::Call).is_none());
        }
    }

    #[test]
    fn equal_strikes_collapse_to_black_scholes() {
        // K_trigger = K_settle = 100 → vanilla BS call.
        let s = 100.0;
        let k = 100.0;
        let t = 0.5;
        let r = 0.05;
        let q = 0.0;
        let v = 0.20;
        let r_gap = price(s, k, k, t, r, q, v, OptionKind::Call).unwrap();
        let sqrt_t = t.sqrt();
        let d1 = ((s / k).ln() + (r - q + 0.5 * v * v) * t) / (v * sqrt_t);
        let d2 = d1 - v * sqrt_t;
        let bs = s * (-q * t).exp() * norm_cdf(d1) - k * (-r * t).exp() * norm_cdf(d2);
        assert!((r_gap.price - bs).abs() < 1e-9);
    }

    #[test]
    fn higher_settlement_strike_lowers_call_price() {
        let r_low = price(100.0, 100.0, 95.0, 0.5, 0.05, 0.0, 0.2, OptionKind::Call).unwrap();
        let r_high = price(100.0, 100.0, 110.0, 0.5, 0.05, 0.0, 0.2, OptionKind::Call).unwrap();
        assert!(r_high.price < r_low.price);
    }

    #[test]
    fn gap_option_can_be_negative_with_high_settlement() {
        // Trigger 100, settlement 200 — pays (S − 200) if S > 100.
        // At spot 100, expected payoff at horizon is small positive prob ·
        // (S − 200), which is largely negative. Price should be negative.
        let r = price(100.0, 100.0, 200.0, 0.5, 0.05, 0.0, 0.20, OptionKind::Call).unwrap();
        assert!(
            r.price < 0.0,
            "expected negative gap call price, got {}",
            r.price
        );
    }

    #[test]
    fn put_call_parity_extension_holds() {
        // For gap options: c − p = S·e^{−qT} − K_settle·e^{−rT}
        // (independent of K_trigger; trigger only shapes the d1/d2 cutoff).
        let s = 100.0;
        let k_trig = 105.0;
        let k_set = 100.0;
        let t = 0.5;
        let r = 0.05;
        let q = 0.0;
        let v = 0.20;
        let c = price(s, k_trig, k_set, t, r, q, v, OptionKind::Call).unwrap();
        let p = price(s, k_trig, k_set, t, r, q, v, OptionKind::Put).unwrap();
        let parity = s * (-q * t).exp() - k_set * (-r * t).exp();
        assert!((c.price - p.price - parity).abs() < 1e-9);
    }

    #[test]
    fn higher_vol_inflates_atm_gap_call() {
        let r_low = price(100.0, 100.0, 100.0, 0.5, 0.05, 0.0, 0.10, OptionKind::Call).unwrap();
        let r_high = price(100.0, 100.0, 100.0, 0.5, 0.05, 0.0, 0.40, OptionKind::Call).unwrap();
        assert!(r_high.price > r_low.price);
    }
}
