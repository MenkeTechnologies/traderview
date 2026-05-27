//! Margin-call runway estimator.
//!
//! For a Reg-T account with a single long position, the maintenance
//! margin requirement is 25% of position value (broker may be higher).
//! Account equity must stay > margin_req.
//!
//! Given:
//!   - account_equity (cash + position - debt)
//!   - position_value (gross long exposure)
//!   - maintenance_req_pct (e.g. 0.25 = 25%)
//!
//! Project the % price decline that triggers a margin call:
//!   margin_call_at_pct_decline ≈ 1 - (equity / position_value) / (1 - maint_req)
//!
//! This is the standard runway formula. Negative result = already in
//! margin call.
//!
//! Pure compute. Caller handles multi-position portfolios upstream by
//! aggregating notional + equity.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MarginRunwayReport {
    pub account_equity: f64,
    pub position_value: f64,
    pub maintenance_req_pct: f64,
    /// % decline from current price that triggers margin call.
    pub runway_pct: f64,
    pub already_in_margin_call: bool,
    pub equity_buffer_dollars: f64,
}

pub fn compute(
    account_equity: f64,
    position_value: f64,
    maintenance_req_pct: f64,
) -> MarginRunwayReport {
    if position_value <= 0.0 || maintenance_req_pct >= 1.0 {
        return MarginRunwayReport {
            account_equity,
            position_value,
            maintenance_req_pct,
            ..Default::default()
        };
    }
    let maint_req_dollars = position_value * maintenance_req_pct;
    let equity_buffer = account_equity - maint_req_dollars;
    let already_called = equity_buffer < 0.0;
    // After a price decline `d`: new position = position × (1-d).
    // New equity = old equity - position × d.
    // New maint = position × (1-d) × maint_pct.
    // Call when new_equity < new_maint:
    //   equity - position × d < position × (1-d) × maint_pct
    //   equity < position × d + position × maint_pct - position × d × maint_pct
    //   equity < position × (d (1-maint_pct) + maint_pct)
    //   equity - position × maint_pct < position × d × (1 - maint_pct)
    //   d > (equity - position × maint_pct) / (position × (1 - maint_pct))
    let runway = if already_called {
        0.0
    } else {
        (account_equity - maint_req_dollars) / (position_value * (1.0 - maintenance_req_pct))
    };
    MarginRunwayReport {
        account_equity,
        position_value,
        maintenance_req_pct,
        runway_pct: runway,
        already_in_margin_call: already_called,
        equity_buffer_dollars: equity_buffer,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_position_zero_runway() {
        let r = compute(10_000.0, 0.0, 0.25);
        assert_eq!(r.runway_pct, 0.0);
    }

    #[test]
    fn all_cash_no_position_safe() {
        // $50k equity, no position → no margin issue (runway zero by guard).
        let r = compute(50_000.0, 0.0, 0.25);
        assert!(!r.already_in_margin_call);
    }

    #[test]
    fn fully_funded_position_high_runway() {
        // $100k equity, $100k position at 25% maint. Buffer = $100k - $25k = $75k.
        // Runway = 75k / (100k × 0.75) = 1.0 → 100% price decline allowed.
        let r = compute(100_000.0, 100_000.0, 0.25);
        assert!((r.runway_pct - 1.0).abs() < 1e-9);
        assert!(!r.already_in_margin_call);
    }

    #[test]
    fn margined_position_lower_runway() {
        // $50k equity, $100k position at 25% maint. Buffer = $50k - $25k = $25k.
        // Runway = 25k / (100k × 0.75) = 0.333 → 33% decline triggers call.
        let r = compute(50_000.0, 100_000.0, 0.25);
        assert!((r.runway_pct - 1.0 / 3.0).abs() < 1e-9);
    }

    #[test]
    fn at_maintenance_zero_runway() {
        // Equity exactly equals maintenance req → no buffer left.
        let r = compute(25_000.0, 100_000.0, 0.25);
        assert_eq!(r.equity_buffer_dollars, 0.0);
        assert!(r.runway_pct.abs() < 1e-9);
    }

    #[test]
    fn under_maintenance_already_in_call() {
        let r = compute(20_000.0, 100_000.0, 0.25);
        assert!(r.already_in_margin_call);
        assert_eq!(r.runway_pct, 0.0);
    }

    #[test]
    fn higher_maint_req_lower_runway() {
        // Same equity + position; higher maint = less runway.
        let low_maint = compute(50_000.0, 100_000.0, 0.25);
        let high_maint = compute(50_000.0, 100_000.0, 0.40);
        assert!(high_maint.runway_pct < low_maint.runway_pct);
    }

    #[test]
    fn equity_buffer_correct() {
        // $40k equity - $25k maint = $15k buffer.
        let r = compute(40_000.0, 100_000.0, 0.25);
        assert_eq!(r.equity_buffer_dollars, 15_000.0);
    }
}
