//! Chooser option — Rubinstein (1991) simple chooser closed form.
//!
//! At intermediate time t_choose ≤ T, the holder picks between a call
//! and a put on the same underlying (both struck at K, expiring at T).
//! At t_choose the holder picks the more valuable side, so the chooser
//! value at t=0 collapses to:
//!
//!   chooser = c(S, K, T) + p(S, K · e^{−(r−q)(T − t_choose)}, t_choose)
//!
//! where c, p are vanilla Black-Scholes calls and puts. The put is
//! struck at the discounted forward strike at t_choose, reflecting
//! that the holder waits until t_choose to decide and during that time
//! S can drift.
//!
//! Pure compute. Useful for retail "double-no-touch" style products
//! and as a building block for synthetic straddles with delayed
//! commitment.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ChooserReport {
    pub price: f64,
    pub call_component: f64,
    pub put_component: f64,
    pub effective_put_strike: f64,
}

#[allow(clippy::too_many_arguments)]
pub fn price(
    spot: f64,
    strike: f64,
    time_to_choice: f64,
    time_to_expiry: f64,
    risk_free: f64,
    dividend_yield: f64,
    sigma: f64,
) -> Option<ChooserReport> {
    if !spot.is_finite()
        || spot <= 0.0
        || !strike.is_finite()
        || strike <= 0.0
        || !time_to_choice.is_finite()
        || time_to_choice <= 0.0
        || !time_to_expiry.is_finite()
        || time_to_expiry < time_to_choice
        || !risk_free.is_finite()
        || !dividend_yield.is_finite()
        || !sigma.is_finite()
        || sigma <= 0.0
    {
        return None;
    }
    // Call leg: full-maturity vanilla call at strike K.
    let call = bs_price(
        spot,
        strike,
        time_to_expiry,
        risk_free,
        dividend_yield,
        sigma,
        true,
    );
    // Put leg: maturity = t_choose, struck at K · e^{−(r−q)·(T − t_choose)}.
    let effective_strike =
        strike * (-(risk_free - dividend_yield) * (time_to_expiry - time_to_choice)).exp();
    let put = bs_price(
        spot,
        effective_strike,
        time_to_choice,
        risk_free,
        dividend_yield,
        sigma,
        false,
    );
    let total = call + put;
    if !total.is_finite() {
        return None;
    }
    Some(ChooserReport {
        price: total.max(0.0),
        call_component: call,
        put_component: put,
        effective_put_strike: effective_strike,
    })
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
            assert!(price(bad, 100.0, 0.25, 0.5, 0.05, 0.0, 0.2).is_none());
            assert!(price(100.0, bad, 0.25, 0.5, 0.05, 0.0, 0.2).is_none());
            assert!(price(100.0, 100.0, bad, 0.5, 0.05, 0.0, 0.2).is_none());
            assert!(price(100.0, 100.0, 0.25, 0.5, 0.05, 0.0, bad).is_none());
        }
    }

    #[test]
    fn t_choose_after_t_expiry_rejected() {
        assert!(price(100.0, 100.0, 1.0, 0.5, 0.05, 0.0, 0.2).is_none());
    }

    #[test]
    fn choice_at_expiry_collapses_to_straddle() {
        // When t_choose == T the chooser becomes a straddle (call + put
        // at K both expiring at T — no waiting period).
        let s = 100.0;
        let k = 100.0;
        let t = 0.5;
        let r = 0.05;
        let q = 0.0;
        let v = 0.20;
        let ch = price(s, k, t, t, r, q, v).unwrap();
        let vanilla_call = bs_price(s, k, t, r, q, v, true);
        let vanilla_put = bs_price(s, k, t, r, q, v, false);
        let straddle = vanilla_call + vanilla_put;
        // Effective put strike at t_choose = T should equal K exactly.
        assert!((ch.effective_put_strike - k).abs() < 1e-9);
        assert!((ch.price - straddle).abs() < 1e-6);
    }

    #[test]
    fn chooser_bounded_below_by_call_alone() {
        // The chooser is at least as valuable as a standalone call
        // (since one option is always picking the better of call/put).
        let ch = price(100.0, 100.0, 0.25, 0.5, 0.05, 0.0, 0.20).unwrap();
        let call = bs_price(100.0, 100.0, 0.5, 0.05, 0.0, 0.20, true);
        assert!(ch.price >= call - 1e-9);
    }

    #[test]
    fn chooser_bounded_above_by_straddle() {
        // And bounded above by the equivalent straddle (chooser at T).
        let ch = price(100.0, 100.0, 0.25, 0.5, 0.05, 0.0, 0.20).unwrap();
        let call = bs_price(100.0, 100.0, 0.5, 0.05, 0.0, 0.20, true);
        let put = bs_price(100.0, 100.0, 0.5, 0.05, 0.0, 0.20, false);
        assert!(ch.price <= call + put + 1e-9);
    }

    #[test]
    fn earlier_choice_lowers_chooser_value() {
        // Less waiting time = less optionality.
        let early = price(100.0, 100.0, 0.05, 0.5, 0.05, 0.0, 0.20).unwrap();
        let late = price(100.0, 100.0, 0.45, 0.5, 0.05, 0.0, 0.20).unwrap();
        assert!(late.price > early.price);
    }

    #[test]
    fn higher_vol_inflates_chooser_price() {
        let r_low = price(100.0, 100.0, 0.25, 0.5, 0.05, 0.0, 0.10).unwrap();
        let r_high = price(100.0, 100.0, 0.25, 0.5, 0.05, 0.0, 0.40).unwrap();
        assert!(r_high.price > r_low.price);
    }

    #[test]
    fn components_sum_to_price() {
        let r = price(100.0, 100.0, 0.25, 0.5, 0.05, 0.02, 0.20).unwrap();
        assert!((r.call_component + r.put_component - r.price).abs() < 1e-9);
    }
}
