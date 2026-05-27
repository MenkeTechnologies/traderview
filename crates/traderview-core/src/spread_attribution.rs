//! Spread-trade P&L attribution.
//!
//! For a long-short pair, decompose total P&L into:
//!   - **Long-leg contribution**: long_pnl alone (long_exit - long_entry).
//!   - **Short-leg contribution**: short_pnl alone (short_entry - short_exit).
//!   - **Spread shrink/widen**: change in (long_price - short_price) over time.
//!
//! Pair traders care about the spread component — that's the alpha;
//! the leg contributions are largely market-neutral noise.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairTrade {
    pub long_entry: f64,
    pub long_exit: f64,
    pub long_qty: f64,
    pub short_entry: f64,
    pub short_exit: f64,
    pub short_qty: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AttributionReport {
    pub long_leg_pnl: f64,
    pub short_leg_pnl: f64,
    pub total_pnl: f64,
    pub spread_entry: f64,
    pub spread_exit: f64,
    pub spread_change: f64,
    /// Per-share-equivalent spread change × min qty.
    pub spread_contribution: f64,
    /// The "noise" piece — what's left after spread is accounted for.
    pub directional_contribution: f64,
}

pub fn attribute(t: &PairTrade) -> AttributionReport {
    let long_pnl = (t.long_exit - t.long_entry) * t.long_qty;
    let short_pnl = (t.short_entry - t.short_exit) * t.short_qty;
    let total = long_pnl + short_pnl;
    let spread_entry = t.long_entry - t.short_entry;
    let spread_exit = t.long_exit - t.short_exit;
    let spread_change = spread_exit - spread_entry;
    let min_qty = t.long_qty.min(t.short_qty);
    let spread_contribution = spread_change * min_qty;
    let directional = total - spread_contribution;
    AttributionReport {
        long_leg_pnl: long_pnl,
        short_leg_pnl: short_pnl,
        total_pnl: total,
        spread_entry,
        spread_exit,
        spread_change,
        spread_contribution,
        directional_contribution: directional,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pure_spread_widening_attributes_to_spread() {
        // Long entered at $100, exited at $110 (+$10).
        // Short entered at $90, exited at $90 (+$0).
        // Spread widened from $10 to $20 → +$10.
        // Total PnL = $10 (long) + $0 (short) = $10.
        // All of it attributable to spread movement.
        let t = PairTrade {
            long_entry: 100.0,
            long_exit: 110.0,
            long_qty: 1.0,
            short_entry: 90.0,
            short_exit: 90.0,
            short_qty: 1.0,
        };
        let r = attribute(&t);
        assert_eq!(r.long_leg_pnl, 10.0);
        assert_eq!(r.short_leg_pnl, 0.0);
        assert_eq!(r.total_pnl, 10.0);
        assert_eq!(r.spread_contribution, 10.0);
        assert_eq!(r.directional_contribution, 0.0);
    }

    #[test]
    fn pure_directional_no_spread_change_attributes_to_directional() {
        // Both go up by $10 → spread unchanged.
        // Long +$10, Short -$10. Net = 0.
        let t = PairTrade {
            long_entry: 100.0,
            long_exit: 110.0,
            long_qty: 1.0,
            short_entry: 90.0,
            short_exit: 100.0,
            short_qty: 1.0,
        };
        let r = attribute(&t);
        assert_eq!(r.total_pnl, 0.0);
        assert_eq!(r.spread_change, 0.0);
        assert_eq!(r.spread_contribution, 0.0);
        assert_eq!(r.directional_contribution, 0.0);
    }

    #[test]
    fn short_leg_negative_pnl_when_short_price_rises() {
        let t = PairTrade {
            long_entry: 100.0,
            long_exit: 100.0,
            long_qty: 1.0,
            short_entry: 90.0,
            short_exit: 100.0,
            short_qty: 1.0,
        };
        let r = attribute(&t);
        assert_eq!(r.short_leg_pnl, -10.0);
    }

    #[test]
    fn short_leg_positive_pnl_when_short_price_falls() {
        let t = PairTrade {
            long_entry: 100.0,
            long_exit: 100.0,
            long_qty: 1.0,
            short_entry: 100.0,
            short_exit: 90.0,
            short_qty: 1.0,
        };
        let r = attribute(&t);
        assert_eq!(r.short_leg_pnl, 10.0);
    }

    #[test]
    fn quantity_scales_pnl_linearly() {
        let one_share = PairTrade {
            long_entry: 100.0,
            long_exit: 110.0,
            long_qty: 1.0,
            short_entry: 90.0,
            short_exit: 85.0,
            short_qty: 1.0,
        };
        let ten_share = PairTrade {
            long_qty: 10.0,
            short_qty: 10.0,
            ..one_share.clone()
        };
        let a = attribute(&one_share);
        let b = attribute(&ten_share);
        assert_eq!(b.total_pnl, a.total_pnl * 10.0);
    }

    #[test]
    fn min_qty_used_for_spread_contribution() {
        // Asymmetric position size — long 10, short 5. Spread × min(10,5)=5.
        let t = PairTrade {
            long_entry: 100.0,
            long_exit: 110.0,
            long_qty: 10.0,
            short_entry: 100.0,
            short_exit: 105.0,
            short_qty: 5.0,
        };
        let r = attribute(&t);
        // Spread change = (110-105) - (100-100) = 5. Spread contrib = 5 × 5 = 25.
        assert_eq!(r.spread_contribution, 25.0);
    }

    #[test]
    fn spread_shrink_attributes_negative_spread_contribution() {
        // Long $100 → $95 (-5). Short $90 → $90 (0). Spread $10 → $5 (shrink -5).
        let t = PairTrade {
            long_entry: 100.0,
            long_exit: 95.0,
            long_qty: 1.0,
            short_entry: 90.0,
            short_exit: 90.0,
            short_qty: 1.0,
        };
        let r = attribute(&t);
        assert_eq!(r.spread_change, -5.0);
        assert_eq!(r.spread_contribution, -5.0);
    }
}
