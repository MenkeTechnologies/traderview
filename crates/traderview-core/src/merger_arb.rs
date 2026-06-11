//! Merger arbitrage — cash-deal spread, annualization, and the
//! market-implied completion probability.
//!
//! With deal price D, break price B, and current price P:
//!
//!   P = p·D + (1 − p)·B   ⇒   p = (P − B)/(D − B)
//!
//! the risk-neutral completion probability the market is charging.
//! An edge exists when your estimate of p differs; the report prices
//! the expected value at the caller's estimate too.
//!
//! Pure compute. Companion to `probability_of_touch`, `risk_reward`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct MergerArbInput {
    pub current_price: f64,
    pub deal_price: f64,
    /// Estimated price if the deal breaks.
    pub break_price: f64,
    pub days_to_close: f64,
    /// Your completion-probability estimate (0–1), for the EV row.
    pub estimated_probability: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct MergerArbReport {
    pub gross_spread_pct: f64,
    pub annualized_spread_pct: f64,
    /// (P − B)/(D − B), clamped to [0, 1].
    pub implied_probability: f64,
    /// Expected price and return at the caller's probability.
    pub expected_price: f64,
    pub expected_return_pct: f64,
    /// Loss if the deal breaks today, %.
    pub downside_pct: f64,
    /// Reward:risk of the deal spread vs the break gap.
    pub reward_risk: Option<f64>,
}

pub fn compute(inp: &MergerArbInput) -> Option<MergerArbReport> {
    if ![inp.current_price, inp.deal_price, inp.break_price].iter().all(|v| v.is_finite() && *v > 0.0)
        || !inp.days_to_close.is_finite()
        || inp.days_to_close <= 0.0
        || !inp.estimated_probability.is_finite()
        || !(0.0..=1.0).contains(&inp.estimated_probability)
        || inp.deal_price <= inp.break_price
    {
        return None;
    }
    let (p, d, b) = (inp.current_price, inp.deal_price, inp.break_price);
    let gross = (d - p) / p * 100.0;
    let annualized = gross * 365.0 / inp.days_to_close;
    let implied = ((p - b) / (d - b)).clamp(0.0, 1.0);
    let est = inp.estimated_probability;
    let expected = est * d + (1.0 - est) * b;
    let upside = d - p;
    let downside = p - b;
    Some(MergerArbReport {
        gross_spread_pct: gross,
        annualized_spread_pct: annualized,
        implied_probability: implied,
        expected_price: expected,
        expected_return_pct: (expected / p - 1.0) * 100.0,
        downside_pct: -(downside / p * 100.0),
        reward_risk: (downside > 0.0).then(|| upside / downside),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> MergerArbInput {
        MergerArbInput {
            current_price: 98.0,
            deal_price: 100.0,
            break_price: 90.0,
            days_to_close: 73.0,
            estimated_probability: 0.9,
        }
    }

    #[test]
    fn spread_probability_and_ev_hand_walk() {
        let r = compute(&base()).unwrap();
        // Spread 2/98 ≈ 2.041%; 73 days = exactly 5 periods/year.
        assert!((r.gross_spread_pct - 2.0 / 98.0 * 100.0).abs() < 1e-12);
        assert!((r.annualized_spread_pct - r.gross_spread_pct * 5.0).abs() < 1e-9);
        // Implied p = (98 − 90)/(100 − 90) = 0.8.
        assert!((r.implied_probability - 0.8).abs() < 1e-12);
        // EV at p = 0.9: 0.9·100 + 0.1·90 = 99 ⇒ +1.0204%.
        assert!((r.expected_price - 99.0).abs() < 1e-12);
        assert!((r.expected_return_pct - (99.0 / 98.0 - 1.0) * 100.0).abs() < 1e-12);
        // Reward:risk = 2 : 8.
        assert!((r.reward_risk.unwrap() - 0.25).abs() < 1e-12);
        assert!((r.downside_pct + 8.0 / 98.0 * 100.0).abs() < 1e-12);
    }

    #[test]
    fn price_above_deal_implies_certainty_or_topping_bid() {
        let mut inp = base();
        inp.current_price = 101.0;
        let r = compute(&inp).unwrap();
        assert_eq!(r.implied_probability, 1.0); // clamped
        assert!(r.gross_spread_pct < 0.0); // negative spread
    }

    #[test]
    fn price_below_break_clamps_to_zero() {
        let mut inp = base();
        inp.current_price = 85.0;
        let r = compute(&inp).unwrap();
        assert_eq!(r.implied_probability, 0.0);
        assert!(r.reward_risk.is_none()); // no downside left to risk
    }

    #[test]
    fn hostile_inputs_return_none() {
        let mut bad = base();
        bad.break_price = 100.0; // D ≤ B
        assert!(compute(&bad).is_none());
        let mut bad = base();
        bad.days_to_close = 0.0;
        assert!(compute(&bad).is_none());
        let mut bad = base();
        bad.estimated_probability = 1.5;
        assert!(compute(&bad).is_none());
        let mut bad = base();
        bad.current_price = f64::NAN;
        assert!(compute(&bad).is_none());
    }
}
