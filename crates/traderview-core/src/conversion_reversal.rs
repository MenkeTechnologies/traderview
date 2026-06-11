//! Conversion / Reversal — put-call parity arbitrage detector.
//!
//! A conversion locks in a riskless payoff by pairing stock with a
//! synthetic short position at the same strike/expiry:
//!
//!   Conversion: + stock  + put(K)  − call(K)   → worth exactly K at expiry
//!   Reversal:   − stock  − put(K)  + call(K)   → owes exactly K at expiry
//!
//! Net cost of the conversion today = S + P − C − PV(dividends).
//! Implied financing rate (continuous):
//!
//!   r_conv = ln(K / cost) / T
//!
//! r_conv above the market rate → the conversion lends at a premium
//! (do the conversion); below → the reversal borrows cheap (do the
//! reversal). Deviations beyond the threshold flag actionable arb.
//!
//! Pure compute. Companion to `box_spread`, `jelly_roll_arbitrage`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConversionReversalReport {
    pub strike: f64,
    /// S + P − C − PV(dividends): outlay that becomes K at expiry.
    pub conversion_cost: f64,
    pub implied_continuous_rate: f64,
    pub market_rate: f64,
    pub deviation_basis_points: f64,
    /// "conversion", "reversal", or "none".
    pub arbitrage_side: &'static str,
    /// P/L per share at expiry if financed at the market rate:
    /// K·e^(−rT) − cost (positive favors the conversion).
    pub edge_per_share: f64,
    pub is_arbitrage_opportunity: bool,
}

#[allow(clippy::too_many_arguments)]
pub fn compute(
    spot: f64,
    strike: f64,
    call_price: f64,
    put_price: f64,
    pv_dividends: f64,
    time_to_expiry_years: f64,
    market_risk_free_rate: f64,
    arbitrage_threshold_bps: f64,
) -> Option<ConversionReversalReport> {
    if ![spot, strike, call_price, put_price, pv_dividends].iter().all(|v| v.is_finite())
        || spot <= 0.0
        || strike <= 0.0
        || call_price < 0.0
        || put_price < 0.0
        || pv_dividends < 0.0
        || !time_to_expiry_years.is_finite()
        || time_to_expiry_years <= 0.0
        || !market_risk_free_rate.is_finite()
        || !arbitrage_threshold_bps.is_finite()
        || arbitrage_threshold_bps < 0.0
    {
        return None;
    }
    let cost = spot + put_price - call_price - pv_dividends;
    let pv_strike = strike * (-market_risk_free_rate * time_to_expiry_years).exp();
    if cost <= 0.0 {
        // Paid to put the conversion on → free money.
        return Some(ConversionReversalReport {
            strike,
            conversion_cost: cost,
            implied_continuous_rate: f64::INFINITY,
            market_rate: market_risk_free_rate,
            deviation_basis_points: f64::INFINITY,
            arbitrage_side: "conversion",
            edge_per_share: pv_strike - cost,
            is_arbitrage_opportunity: true,
        });
    }
    let implied = (strike / cost).ln() / time_to_expiry_years;
    let dev_bps = (implied - market_risk_free_rate) * 10_000.0;
    let is_arb = dev_bps.abs() > arbitrage_threshold_bps;
    let side = if !is_arb {
        "none"
    } else if dev_bps > 0.0 {
        "conversion"
    } else {
        "reversal"
    };
    Some(ConversionReversalReport {
        strike,
        conversion_cost: cost,
        implied_continuous_rate: implied,
        market_rate: market_risk_free_rate,
        deviation_basis_points: dev_bps,
        arbitrage_side: side,
        // PV(K) at the market rate minus what the conversion costs.
        edge_per_share: pv_strike - cost,
        is_arbitrage_opportunity: is_arb,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parity_priced_chain_implies_market_rate() {
        // S=100, K=100, r=5%, T=0.25, no divs: C − P = S − K·e^(−rT).
        let (s, k, t, r) = (100.0_f64, 100.0_f64, 0.25_f64, 0.05_f64);
        let p = 3.0;
        let c = p + s - k * (-r * t).exp();
        let rep = compute(s, k, c, p, 0.0, t, r, 25.0).unwrap();
        assert!((rep.implied_continuous_rate - r).abs() < 1e-9, "{}", rep.implied_continuous_rate);
        assert_eq!(rep.arbitrage_side, "none");
        assert!(!rep.is_arbitrage_opportunity);
        assert!(rep.edge_per_share.abs() < 1e-9);
    }

    #[test]
    fn overpriced_call_flags_conversion() {
        // Rich call makes the conversion cheap → implied rate above
        // market → conversion side, positive edge.
        let (s, k, t, r) = (100.0_f64, 100.0_f64, 0.25_f64, 0.05_f64);
        let p = 3.0;
        let fair_c = p + s - k * (-r * t).exp();
        let rep = compute(s, k, fair_c + 0.50, p, 0.0, t, r, 25.0).unwrap();
        assert_eq!(rep.arbitrage_side, "conversion");
        assert!(rep.deviation_basis_points > 25.0);
        assert!((rep.edge_per_share - 0.50).abs() < 1e-9);
    }

    #[test]
    fn overpriced_put_flags_reversal() {
        let (s, k, t, r) = (100.0_f64, 100.0_f64, 0.25_f64, 0.05_f64);
        let p = 3.0;
        let fair_c = p + s - k * (-r * t).exp();
        // Cheap call (equivalently rich put) → reversal side.
        let rep = compute(s, k, fair_c - 0.50, p, 0.0, t, r, 25.0).unwrap();
        assert_eq!(rep.arbitrage_side, "reversal");
        assert!(rep.deviation_basis_points < -25.0);
        assert!((rep.edge_per_share + 0.50).abs() < 1e-9);
    }

    #[test]
    fn dividends_reduce_conversion_cost() {
        let (s, k, t, r) = (100.0_f64, 100.0_f64, 0.25_f64, 0.05_f64);
        let p = 3.0;
        let fair_c = p + s - k * (-r * t).exp();
        let no_div = compute(s, k, fair_c, p, 0.0, t, r, 25.0).unwrap();
        let with_div = compute(s, k, fair_c, p, 1.0, t, r, 25.0).unwrap();
        assert!((no_div.conversion_cost - with_div.conversion_cost - 1.0).abs() < 1e-12);
        assert!(with_div.implied_continuous_rate > no_div.implied_continuous_rate);
    }

    #[test]
    fn threshold_gates_the_flag() {
        let (s, k, t, r) = (100.0_f64, 100.0_f64, 0.25_f64, 0.05_f64);
        let p = 3.0;
        let fair_c = p + s - k * (-r * t).exp();
        let rep = compute(s, k, fair_c + 0.50, p, 0.0, t, r, 100_000.0).unwrap();
        assert!(!rep.is_arbitrage_opportunity);
        assert_eq!(rep.arbitrage_side, "none");
    }

    #[test]
    fn negative_cost_is_infinite_rate_conversion() {
        // Call so rich the position is net-credit up front.
        let rep = compute(100.0, 100.0, 105.0, 3.0, 0.0, 0.25, 0.05, 25.0).unwrap();
        assert!(rep.implied_continuous_rate.is_infinite());
        assert_eq!(rep.arbitrage_side, "conversion");
        assert!(rep.is_arbitrage_opportunity);
    }

    #[test]
    fn hostile_inputs_return_none() {
        assert!(compute(0.0, 100.0, 4.0, 3.0, 0.0, 0.25, 0.05, 25.0).is_none());
        assert!(compute(100.0, -1.0, 4.0, 3.0, 0.0, 0.25, 0.05, 25.0).is_none());
        assert!(compute(100.0, 100.0, -4.0, 3.0, 0.0, 0.25, 0.05, 25.0).is_none());
        assert!(compute(100.0, 100.0, 4.0, 3.0, 0.0, 0.0, 0.05, 25.0).is_none());
        assert!(compute(100.0, 100.0, f64::NAN, 3.0, 0.0, 0.25, 0.05, 25.0).is_none());
        assert!(compute(100.0, 100.0, 4.0, 3.0, 0.0, 0.25, 0.05, -1.0).is_none());
    }
}
