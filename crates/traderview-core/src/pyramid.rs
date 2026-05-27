//! Pyramid / scale-in plan calculator.
//!
//! Two related entry-management styles:
//!   * **Pyramid up**: add to winners as price moves in your favor,
//!     each add at a tighter risk band. Used by trend-followers (Dan
//!     Zanger, Mark Minervini).
//!   * **Scale in (averaging down)**: add as price moves AGAINST you
//!     within a planned price ladder. Used by mean-reversion + LEAPs
//!     traders. Very dangerous without a hard total-risk cap.
//!
//! This module computes the size + cost-basis evolution for both. Pure
//! compute. Caller supplies the planned price ladder + tranche sizing.

use crate::models::TradeSide;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanKind {
    /// Each tranche fires when price moves IN your favor.
    PyramidUp,
    /// Each tranche fires when price moves AGAINST you (averaging down).
    ScaleIn,
}

/// One planned tranche.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tranche {
    /// Price at which this tranche fires.
    pub trigger_price: Decimal,
    /// Number of shares/contracts to add at this tranche.
    pub qty: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanInput {
    pub kind: PlanKind,
    pub side: TradeSide,
    /// First (already-filled) entry.
    pub initial_qty: Decimal,
    pub initial_entry: Decimal,
    pub tranches: Vec<Tranche>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CumulativeState {
    /// Tranche label ("entry", "add 1", "add 2", …)
    pub label: String,
    pub trigger_price: Decimal,
    pub added_qty: Decimal,
    pub total_qty: Decimal,
    pub avg_cost: Decimal,
    /// Cumulative dollar exposure at this state (avg_cost × total_qty).
    pub notional: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlanReport {
    pub states: Vec<CumulativeState>,
    /// Final total qty after all tranches fire.
    pub final_qty: Decimal,
    /// Volume-weighted avg cost across initial + all tranches.
    pub final_avg_cost: Decimal,
    /// Final total notional exposure.
    pub final_notional: Decimal,
    /// True if the plan's tranches violate the kind's direction rule
    /// (e.g., a PyramidUp tranche priced BELOW initial entry for a long).
    pub plan_misordered: bool,
}

pub fn build(input: &PlanInput) -> PlanReport {
    let mut total_qty = input.initial_qty;
    let mut total_cost = input.initial_entry * input.initial_qty;
    let mut states = vec![CumulativeState {
        label: "entry".into(),
        trigger_price: input.initial_entry,
        added_qty: input.initial_qty,
        total_qty,
        avg_cost: input.initial_entry,
        notional: total_cost,
    }];

    let mut misordered = false;
    for (i, t) in input.tranches.iter().enumerate() {
        // Direction check.
        let direction_ok = match (input.kind, input.side) {
            (PlanKind::PyramidUp,  TradeSide::Long)  => t.trigger_price > input.initial_entry,
            (PlanKind::PyramidUp,  TradeSide::Short) => t.trigger_price < input.initial_entry,
            (PlanKind::ScaleIn,    TradeSide::Long)  => t.trigger_price < input.initial_entry,
            (PlanKind::ScaleIn,    TradeSide::Short) => t.trigger_price > input.initial_entry,
        };
        if !direction_ok { misordered = true; }

        total_qty += t.qty;
        total_cost += t.trigger_price * t.qty;
        let avg = if total_qty.is_zero() { Decimal::ZERO } else { total_cost / total_qty };
        states.push(CumulativeState {
            label: format!("add {}", i + 1),
            trigger_price: t.trigger_price,
            added_qty: t.qty,
            total_qty,
            avg_cost: avg,
            notional: avg * total_qty,
        });
    }
    let final_avg = if total_qty.is_zero() { Decimal::ZERO } else { total_cost / total_qty };
    PlanReport {
        final_qty: total_qty,
        final_avg_cost: final_avg,
        final_notional: total_cost,
        states,
        plan_misordered: misordered,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn d(s: &str) -> Decimal { Decimal::from_str(s).unwrap() }

    fn pyramid_long() -> PlanInput {
        PlanInput {
            kind: PlanKind::PyramidUp,
            side: TradeSide::Long,
            initial_qty: d("100"),
            initial_entry: d("100"),
            tranches: vec![
                Tranche { trigger_price: d("105"), qty: d("50") },
                Tranche { trigger_price: d("110"), qty: d("25") },
            ],
        }
    }

    #[test]
    fn pyramid_long_increases_qty_and_avg_cost() {
        let r = build(&pyramid_long());
        // After tranche 1: 150 qty @ (100×100 + 50×105)/150 = 101.666...
        // After tranche 2: 175 qty @ (15250 + 25×110)/175 = (15250+2750)/175 = 102.857...
        assert_eq!(r.final_qty, d("175"));
        assert_eq!(r.final_avg_cost.round_dp(4), d("102.8571"));
        assert!(!r.plan_misordered);
    }

    #[test]
    fn scale_in_long_decreases_avg_cost() {
        // Same shape but averaging down.
        let p = PlanInput {
            kind: PlanKind::ScaleIn,
            side: TradeSide::Long,
            initial_qty: d("100"),
            initial_entry: d("100"),
            tranches: vec![
                Tranche { trigger_price: d("95"), qty: d("50") },
                Tranche { trigger_price: d("90"), qty: d("25") },
            ],
        };
        let r = build(&p);
        // After tranche 1: (100×100 + 50×95)/150 = 98.333...
        // After tranche 2: (14750 + 25×90)/175 = 17000/175 = 97.142...
        assert_eq!(r.final_avg_cost.round_dp(4), d("97.1429"));
    }

    #[test]
    fn pyramid_short_uses_lower_triggers() {
        let p = PlanInput {
            kind: PlanKind::PyramidUp,
            side: TradeSide::Short,
            initial_qty: d("100"),
            initial_entry: d("100"),
            tranches: vec![Tranche { trigger_price: d("95"), qty: d("50") }],
        };
        let r = build(&p);
        assert!(!r.plan_misordered);
    }

    #[test]
    fn misorder_flagged_when_pyramid_long_tranche_is_below_entry() {
        let mut p = pyramid_long();
        p.tranches.push(Tranche { trigger_price: d("90"), qty: d("10") });
        let r = build(&p);
        assert!(r.plan_misordered,
            "pyramid-up long add at a lower price violates direction");
    }

    #[test]
    fn misorder_flagged_when_scale_in_long_tranche_is_above_entry() {
        let p = PlanInput {
            kind: PlanKind::ScaleIn,
            side: TradeSide::Long,
            initial_qty: d("100"),
            initial_entry: d("100"),
            tranches: vec![Tranche { trigger_price: d("110"), qty: d("50") }],
        };
        let r = build(&p);
        assert!(r.plan_misordered);
    }

    #[test]
    fn states_capture_running_avg_at_each_tranche() {
        let r = build(&pyramid_long());
        assert_eq!(r.states.len(), 3);   // initial + 2 tranches
        // State 0: entry only — avg cost = entry.
        assert_eq!(r.states[0].avg_cost, d("100"));
        // State 1: after first add.
        assert_eq!(r.states[1].total_qty, d("150"));
        // State 2 = final.
        assert_eq!(r.states[2].total_qty, d("175"));
    }

    #[test]
    fn empty_tranches_yields_initial_only() {
        let p = PlanInput {
            kind: PlanKind::PyramidUp,
            side: TradeSide::Long,
            initial_qty: d("100"),
            initial_entry: d("100"),
            tranches: vec![],
        };
        let r = build(&p);
        assert_eq!(r.states.len(), 1);
        assert_eq!(r.final_qty, d("100"));
        assert_eq!(r.final_avg_cost, d("100"));
    }

    #[test]
    fn zero_qty_tranche_does_not_change_avg_cost() {
        let mut p = pyramid_long();
        p.tranches.push(Tranche { trigger_price: d("200"), qty: Decimal::ZERO });
        let r = build(&p);
        // Last tranche added 0 qty — avg cost shouldn't move.
        let prev_avg = r.states[r.states.len() - 2].avg_cost;
        assert_eq!(r.final_avg_cost, prev_avg);
    }
}
