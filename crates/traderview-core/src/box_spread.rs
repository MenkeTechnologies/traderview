//! Box Spread — synthetic risk-free rate / arbitrage detector.
//!
//! A box spread combines four option positions on the same underlying
//! and expiry, locking in a guaranteed payoff equal to the difference
//! between the two strikes:
//!
//!   Long box at strikes K1 < K2:
//!     + long call(K1)
//!     − short call(K2)
//!     + long put(K2)
//!     − short put(K1)
//!
//!   Locked payoff at expiry = K2 − K1.
//!
//! Box premium (debit) = call(K1) − call(K2) + put(K2) − put(K1).
//! Box implied risk-free rate (continuous):
//!
//!   r_box = ln((K2 − K1) / premium) / T
//!
//! Used as:
//!   - Synthetic financing rate vs LIBOR / SOFR for arbitrage
//!   - Detect mispriced option chains (r_box significantly off market rate)
//!   - Cash-management proxy for tax-advantaged accounts
//!
//! Pure compute. Companion to `iron_condor`, `iron_butterfly`,
//! `cross_currency_basis`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BoxSpreadReport {
    pub strike_low: f64,
    pub strike_high: f64,
    pub locked_payoff: f64,
    pub net_premium_debit: f64,
    pub implied_continuous_rate: f64,
    pub market_rate: f64,
    pub arbitrage_basis_points: f64,
    pub is_arbitrage_opportunity: bool,
}

#[allow(clippy::too_many_arguments)]
pub fn compute(
    strike_low: f64,
    strike_high: f64,
    call_low_price: f64,
    call_high_price: f64,
    put_low_price: f64,
    put_high_price: f64,
    time_to_expiry_years: f64,
    market_risk_free_rate: f64,
    arbitrage_threshold_bps: f64,
) -> Option<BoxSpreadReport> {
    if !strike_low.is_finite() || !strike_high.is_finite()
        || strike_high <= strike_low
        || ![call_low_price, call_high_price, put_low_price, put_high_price]
            .iter().all(|p| p.is_finite() && *p >= 0.0)
        || !time_to_expiry_years.is_finite() || time_to_expiry_years <= 0.0
        || !market_risk_free_rate.is_finite()
        || !arbitrage_threshold_bps.is_finite() || arbitrage_threshold_bps < 0.0
    {
        return None;
    }
    let locked = strike_high - strike_low;
    // Long box = +call(low) − call(high) + put(high) − put(low). Premium
    // outlay (debit) for the long box.
    let premium = call_low_price - call_high_price + put_high_price - put_low_price;
    if premium <= 0.0 {
        // Free or negative-cost box → infinite implied rate; arbitrage.
        return Some(BoxSpreadReport {
            strike_low,
            strike_high,
            locked_payoff: locked,
            net_premium_debit: premium,
            implied_continuous_rate: f64::INFINITY,
            market_rate: market_risk_free_rate,
            arbitrage_basis_points: f64::INFINITY,
            is_arbitrage_opportunity: true,
        });
    }
    let implied_rate = (locked / premium).ln() / time_to_expiry_years;
    let arb_bps = (implied_rate - market_risk_free_rate) * 10_000.0;
    let is_arb = arb_bps.abs() > arbitrage_threshold_bps;
    Some(BoxSpreadReport {
        strike_low,
        strike_high,
        locked_payoff: locked,
        net_premium_debit: premium,
        implied_continuous_rate: implied_rate,
        market_rate: market_risk_free_rate,
        arbitrage_basis_points: arb_bps,
        is_arbitrage_opportunity: is_arb,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_strikes_return_none() {
        // strike_high must exceed strike_low.
        assert!(compute(100.0, 100.0, 5.0, 4.0, 4.0, 5.0, 0.25, 0.05, 50.0).is_none());
        assert!(compute(100.0, 90.0, 5.0, 4.0, 4.0, 5.0, 0.25, 0.05, 50.0).is_none());
    }

    #[test]
    fn invalid_prices_return_none() {
        assert!(compute(100.0, 110.0, -1.0, 4.0, 4.0, 5.0, 0.25, 0.05, 50.0).is_none());
        assert!(compute(100.0, 110.0, f64::NAN, 4.0, 4.0, 5.0, 0.25, 0.05, 50.0).is_none());
    }

    #[test]
    fn invalid_time_or_rate_return_none() {
        assert!(compute(100.0, 110.0, 5.0, 4.0, 4.0, 5.0, 0.0, 0.05, 50.0).is_none());
        assert!(compute(100.0, 110.0, 5.0, 4.0, 4.0, 5.0, 0.25, f64::NAN, 50.0).is_none());
    }

    #[test]
    fn fair_priced_box_yields_market_rate() {
        // Box payoff = 10; market rate = 5%; T = 0.25y → fair premium ≈ 10·e^(-0.05·0.25) = 9.876.
        // Build call_low - call_high + put_high - put_low = 9.876.
        let fair_premium = 10.0_f64 * (-0.05_f64 * 0.25).exp();
        // Pick a decomposition: call_low=8.0, call_high=2.0, put_high=5.876, put_low=2.0 → 9.876.
        let cl = 8.0;
        let ch = 2.0;
        let pl = 2.0;
        let ph = fair_premium - cl + ch + pl;
        let r = compute(100.0, 110.0, cl, ch, pl, ph, 0.25, 0.05, 50.0).unwrap();
        assert!((r.implied_continuous_rate - 0.05).abs() < 1e-9);
        assert!(!r.is_arbitrage_opportunity);
    }

    #[test]
    fn cheap_box_creates_arbitrage() {
        // Premium = 9.0; payoff 10; implied rate ≈ ln(10/9) / 0.25 ≈ 42%.
        let r = compute(100.0, 110.0, 5.0, 1.0, 1.0, 4.0, 0.25, 0.05, 50.0).unwrap();
        assert!(r.implied_continuous_rate > 0.10);
        assert!(r.is_arbitrage_opportunity);
        assert!(r.arbitrage_basis_points > 50.0);
    }

    #[test]
    fn negative_premium_yields_infinite_rate() {
        // Premium < 0 → free money / risk-free arbitrage.
        let r = compute(100.0, 110.0, 1.0, 5.0, 5.0, 1.0, 0.25, 0.05, 50.0).unwrap();
        assert!(r.implied_continuous_rate.is_infinite());
        assert!(r.is_arbitrage_opportunity);
    }

    #[test]
    fn arbitrage_threshold_respected() {
        // Build a box at exactly market rate, then put threshold high.
        let fair_premium = 10.0_f64 * (-0.05_f64 * 0.25).exp();
        let r = compute(100.0, 110.0, 8.0, 2.0, 2.0,
            fair_premium - 8.0 + 2.0 + 2.0, 0.25, 0.05, 1000.0).unwrap();
        assert!(!r.is_arbitrage_opportunity);
    }

    #[test]
    fn locked_payoff_equals_strike_diff() {
        let r = compute(100.0, 110.0, 5.0, 1.0, 1.0, 4.0, 0.25, 0.05, 50.0).unwrap();
        assert_eq!(r.locked_payoff, 10.0);
    }
}
