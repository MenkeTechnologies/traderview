//! Implementation Shortfall — the canonical Almgren-style execution-cost
//! decomposition. Used in institutional TCA (TradeStation, IBKR Best
//! Execution, Bloomberg AIM).
//!
//! Total execution cost is broken into four components:
//!
//!   1. **Spread cost**: half the bid/ask at decision time, paid as the
//!      cost of immediacy.
//!   2. **Market impact**: realized price minus arrival mid, after the
//!      spread allocation. Captures how much the trader moved the market.
//!   3. **Timing cost**: arrival-mid-vs-decision-mid drift while the order
//!      sat unfilled. Captures opportunity loss from delayed execution.
//!   4. **Opportunity cost**: unfilled quantity × final-mid-minus-decision-mid.
//!      The cost of NOT trading the rest.
//!
//! All components in dollars (signed: positive = the trader gave up money,
//! negative = trader captured liquidity). Sum = total implementation shortfall.
//!
//! Pure compute. Sized by trade direction: long position pays positive
//! costs when the market moves UP between the decision and the fill.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TradeDirection {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortfallInput {
    pub direction: TradeDirection,
    /// Mid-price at the moment the order was decided.
    pub decision_mid: f64,
    /// Mid-price when the order first arrived at the market.
    pub arrival_mid: f64,
    /// Volume-weighted average fill price (over the filled quantity).
    pub vwap_fill: f64,
    /// Mid-price at the END of the order's life (post-cancellation or
    /// completion). Used for opportunity-cost calculation.
    pub final_mid: f64,
    /// Half-spread at decision time (e.g. (ask - bid) / 2). Allocated as
    /// the immediacy premium.
    pub half_spread_at_decision: f64,
    /// Shares the trader INTENDED to fill.
    pub intended_qty: f64,
    /// Shares actually filled. <= intended_qty.
    pub filled_qty: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ShortfallReport {
    pub spread_cost: f64,
    pub timing_cost: f64,
    pub impact_cost: f64,
    pub opportunity_cost: f64,
    pub total_dollars: f64,
    /// Total / (intended_qty × decision_mid) — basis-point yardstick.
    pub total_bps: f64,
    pub note: String,
}

pub fn analyze(input: &ShortfallInput) -> ShortfallReport {
    if input.intended_qty <= 0.0 {
        return ShortfallReport {
            note: "intended_qty must be > 0".into(),
            ..Default::default()
        };
    }
    if !input.decision_mid.is_finite() || input.decision_mid <= 0.0 {
        return ShortfallReport {
            note: "decision_mid must be positive finite".into(),
            ..Default::default()
        };
    }
    // Sign convention: for a Buy, costs are POSITIVE when the market moves
    // up against the trader. For a Sell, costs are POSITIVE when the market
    // moves down.
    let sign = match input.direction {
        TradeDirection::Buy => 1.0,
        TradeDirection::Sell => -1.0,
    };
    let filled = input.filled_qty.max(0.0).min(input.intended_qty);
    let unfilled = input.intended_qty - filled;
    // Spread: pay half_spread on every filled share, sign-adjusted.
    let spread_cost = sign * input.half_spread_at_decision * filled;
    // Timing: arrival_mid - decision_mid on every intended share.
    let timing_cost = sign * (input.arrival_mid - input.decision_mid) * input.intended_qty;
    // Impact: vwap_fill - arrival_mid on filled shares, MINUS the spread
    // already counted (so impact and spread don't double-count immediacy).
    let raw_fill_premium = sign * (input.vwap_fill - input.arrival_mid) * filled;
    let impact_cost = raw_fill_premium - spread_cost;
    // Opportunity: final_mid - decision_mid on unfilled shares.
    let opportunity_cost = sign * (input.final_mid - input.decision_mid) * unfilled;
    let total = spread_cost + timing_cost + impact_cost + opportunity_cost;
    let notional = input.intended_qty * input.decision_mid;
    let total_bps = if notional > 0.0 {
        total / notional * 10_000.0
    } else {
        0.0
    };

    let note = if filled == 0.0 {
        format!("unfilled order — opportunity cost ${:.2}", opportunity_cost)
    } else if unfilled > 0.0 {
        format!(
            "partial fill ({:.0}%) — shortfall ${:.2} ({:.1} bps)",
            filled / input.intended_qty * 100.0,
            total,
            total_bps
        )
    } else {
        format!("full fill — shortfall ${:.2} ({:.1} bps)", total, total_bps)
    };
    ShortfallReport {
        spread_cost,
        timing_cost,
        impact_cost,
        opportunity_cost,
        total_dollars: total,
        total_bps,
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_intended_qty_returns_zero_with_note() {
        let r = analyze(&ShortfallInput {
            direction: TradeDirection::Buy,
            decision_mid: 100.0,
            arrival_mid: 100.0,
            vwap_fill: 100.0,
            final_mid: 100.0,
            half_spread_at_decision: 0.01,
            intended_qty: 0.0,
            filled_qty: 0.0,
        });
        assert_eq!(r.total_dollars, 0.0);
        assert!(r.note.contains("intended_qty"));
    }

    #[test]
    fn fill_at_full_ask_has_only_spread_cost() {
        // Trader buys 100 at $100 decision. Arrival = decision = $100, vwap
        // fill is exactly at the ask ($100 + $0.01 half-spread). No timing,
        // no opportunity, impact_cost should EXACTLY offset to zero — the
        // total is just the spread paid: $1.
        let r = analyze(&ShortfallInput {
            direction: TradeDirection::Buy,
            decision_mid: 100.0,
            arrival_mid: 100.0,
            vwap_fill: 100.01,
            final_mid: 100.0,
            half_spread_at_decision: 0.01,
            intended_qty: 100.0,
            filled_qty: 100.0,
        });
        assert!((r.spread_cost - 1.0).abs() < 1e-9);
        assert!(r.timing_cost.abs() < 1e-9);
        assert!(
            r.impact_cost.abs() < 1e-9,
            "no excess impact when fill = ask, got {}",
            r.impact_cost
        );
        assert!(r.opportunity_cost.abs() < 1e-9);
        assert!((r.total_dollars - 1.0).abs() < 1e-9);
    }

    #[test]
    fn midpoint_fill_decomposes_to_spread_minus_impact_credit() {
        // A midpoint fill (vwap = arrival_mid, no spread paid) decomposes
        // as: spread = $1 (the spread the trader WOULD have paid crossing),
        // impact = -$1 (the credit from getting a better-than-cross fill).
        // Total = $0 — trader paid nothing vs decision.
        let r = analyze(&ShortfallInput {
            direction: TradeDirection::Buy,
            decision_mid: 100.0,
            arrival_mid: 100.0,
            vwap_fill: 100.0,
            final_mid: 100.0,
            half_spread_at_decision: 0.01,
            intended_qty: 100.0,
            filled_qty: 100.0,
        });
        assert!((r.spread_cost - 1.0).abs() < 1e-9);
        assert!(
            (r.impact_cost - (-1.0)).abs() < 1e-9,
            "midpoint fill should give -spread impact credit, got {}",
            r.impact_cost
        );
        assert!(r.total_dollars.abs() < 1e-9);
    }

    #[test]
    fn market_drifted_up_before_fill_creates_timing_cost() {
        // Decision at $100, arrival at $100.50 → $0.50 × 100 shares = $50 timing.
        let r = analyze(&ShortfallInput {
            direction: TradeDirection::Buy,
            decision_mid: 100.0,
            arrival_mid: 100.5,
            vwap_fill: 100.5,
            final_mid: 100.5,
            half_spread_at_decision: 0.0,
            intended_qty: 100.0,
            filled_qty: 100.0,
        });
        assert!((r.timing_cost - 50.0).abs() < 1e-9);
        assert!(
            r.impact_cost.abs() < 1e-9,
            "no impact when fill = arrival_mid"
        );
    }

    #[test]
    fn unfilled_quantity_drives_opportunity_cost() {
        // Decision $100, half order filled at $100, market then drifts to
        // $102. Opportunity cost = $2 × 50 unfilled = $100.
        let r = analyze(&ShortfallInput {
            direction: TradeDirection::Buy,
            decision_mid: 100.0,
            arrival_mid: 100.0,
            vwap_fill: 100.0,
            final_mid: 102.0,
            half_spread_at_decision: 0.0,
            intended_qty: 100.0,
            filled_qty: 50.0,
        });
        assert!(
            (r.opportunity_cost - 100.0).abs() < 1e-9,
            "expected $100 opportunity cost, got {}",
            r.opportunity_cost
        );
        assert!(r.spread_cost.abs() < 1e-9, "no spread when half_spread=0");
    }

    #[test]
    fn sell_signs_invert_correctly() {
        // SELL with market drifting UP (against the seller) means costs are
        // NEGATIVE — the seller captured more than they expected.
        // Decision $100, arrival $101, no spread, all filled. Sell cost = -$100.
        let r = analyze(&ShortfallInput {
            direction: TradeDirection::Sell,
            decision_mid: 100.0,
            arrival_mid: 101.0,
            vwap_fill: 101.0,
            final_mid: 101.0,
            half_spread_at_decision: 0.0,
            intended_qty: 100.0,
            filled_qty: 100.0,
        });
        assert!(
            r.timing_cost < 0.0,
            "seller captured upside as negative cost"
        );
    }

    #[test]
    fn full_fill_at_ask_total_is_spread_only() {
        // Buy 1000 at decision=$50. Arrival = $50, fill at the ask = $50.02.
        // Total cost = $0.02 × 1000 = $20. Notional $50,000 → 4 bps.
        let r = analyze(&ShortfallInput {
            direction: TradeDirection::Buy,
            decision_mid: 50.0,
            arrival_mid: 50.0,
            vwap_fill: 50.02,
            final_mid: 50.0,
            half_spread_at_decision: 0.02,
            intended_qty: 1000.0,
            filled_qty: 1000.0,
        });
        assert!((r.total_dollars - 20.0).abs() < 1e-9);
        assert!((r.total_bps - 4.0).abs() < 1e-9);
    }
}
