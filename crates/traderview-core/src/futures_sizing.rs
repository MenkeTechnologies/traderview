//! Futures position sizing — contracts from dollar risk, tick math,
//! and the margin cap.
//!
//!   $/point           = tick_value / tick_size
//!   risk per contract = |entry − stop| · $/point
//!   by risk           = ⌊account · risk% / risk_per_contract⌋
//!   by margin         = ⌊account · margin_cap% / initial_margin⌋
//!   contracts         = min(by risk, by margin)
//!
//! The margin cap matters: on vol spikes exchanges raise initial
//! margin and the risk-based size stops being attainable — the binding
//! constraint is reported so the trader knows which one to argue with.
//!
//! Pure compute. Companion to `risk_of_ruin`, `vol_targeting_sizer`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct FuturesSizingInput {
    pub account: f64,
    /// Risk per trade, % of account.
    pub risk_pct: f64,
    pub entry: f64,
    pub stop: f64,
    /// Minimum price increment (ES: 0.25).
    pub tick_size: f64,
    /// Dollar value of one tick (ES: 12.50).
    pub tick_value: f64,
    /// Initial margin per contract, $.
    pub initial_margin: f64,
    /// Max share of the account in margin, % (default 50).
    #[serde(default = "default_margin_cap")]
    pub margin_cap_pct: f64,
}

fn default_margin_cap() -> f64 {
    50.0
}

#[derive(Debug, Clone, Serialize)]
pub struct FuturesSizingReport {
    pub dollars_per_point: f64,
    pub stop_distance_points: f64,
    pub risk_per_contract: f64,
    pub contracts_by_risk: u32,
    pub contracts_by_margin: u32,
    pub contracts: u32,
    /// "risk", "margin", or "none" (zero contracts fit).
    pub binding_constraint: &'static str,
    pub total_risk: f64,
    pub margin_used: f64,
    pub margin_utilization_pct: f64,
    pub notional: f64,
}

pub fn compute(inp: &FuturesSizingInput) -> Option<FuturesSizingReport> {
    if ![
        inp.account,
        inp.risk_pct,
        inp.entry,
        inp.stop,
        inp.tick_size,
        inp.tick_value,
        inp.initial_margin,
        inp.margin_cap_pct,
    ]
    .iter()
    .all(|v| v.is_finite())
        || inp.account <= 0.0
        || inp.risk_pct <= 0.0
        || inp.risk_pct > 100.0
        || inp.entry <= 0.0
        || inp.stop <= 0.0
        || inp.entry == inp.stop
        || inp.tick_size <= 0.0
        || inp.tick_value <= 0.0
        || inp.initial_margin <= 0.0
        || inp.margin_cap_pct <= 0.0
        || inp.margin_cap_pct > 100.0
    {
        return None;
    }
    let per_point = inp.tick_value / inp.tick_size;
    let stop_pts = (inp.entry - inp.stop).abs();
    let risk_per = stop_pts * per_point;
    let by_risk = (inp.account * inp.risk_pct / 100.0 / risk_per).floor() as u32;
    let by_margin = (inp.account * inp.margin_cap_pct / 100.0 / inp.initial_margin).floor() as u32;
    let contracts = by_risk.min(by_margin);
    let binding = if contracts == 0 {
        "none"
    } else if by_risk <= by_margin {
        "risk"
    } else {
        "margin"
    };
    let margin_used = contracts as f64 * inp.initial_margin;
    Some(FuturesSizingReport {
        dollars_per_point: per_point,
        stop_distance_points: stop_pts,
        risk_per_contract: risk_per,
        contracts_by_risk: by_risk,
        contracts_by_margin: by_margin,
        contracts,
        binding_constraint: binding,
        total_risk: contracts as f64 * risk_per,
        margin_used,
        margin_utilization_pct: margin_used / inp.account * 100.0,
        notional: contracts as f64 * inp.entry * per_point,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn es() -> FuturesSizingInput {
        FuturesSizingInput {
            account: 100_000.0,
            risk_pct: 1.0,
            entry: 5000.0,
            stop: 4990.0,
            tick_size: 0.25,
            tick_value: 12.50,
            initial_margin: 15_000.0,
            margin_cap_pct: 50.0,
        }
    }

    #[test]
    fn es_hand_walk_risk_binds() {
        // $50/pt, 10-pt stop = $500/contract; $1000 budget ⇒ 2 by
        // risk; margin allows 3 ⇒ risk binds at 2.
        let r = compute(&es()).unwrap();
        assert!((r.dollars_per_point - 50.0).abs() < 1e-12);
        assert!((r.risk_per_contract - 500.0).abs() < 1e-12);
        assert_eq!(r.contracts_by_risk, 2);
        assert_eq!(r.contracts_by_margin, 3);
        assert_eq!(r.contracts, 2);
        assert_eq!(r.binding_constraint, "risk");
        assert!((r.total_risk - 1000.0).abs() < 1e-12);
        assert!((r.margin_used - 30_000.0).abs() < 1e-12);
        assert!((r.margin_utilization_pct - 30.0).abs() < 1e-12);
        assert!((r.notional - 2.0 * 5000.0 * 50.0).abs() < 1e-12);
    }

    #[test]
    fn margin_spike_becomes_the_binding_constraint() {
        // Exchange hikes initial margin to $30k: only 1 contract fits
        // the 50% cap even though risk allows 2.
        let mut inp = es();
        inp.initial_margin = 30_000.0;
        let r = compute(&inp).unwrap();
        assert_eq!(r.contracts_by_risk, 2);
        assert_eq!(r.contracts_by_margin, 1);
        assert_eq!(r.contracts, 1);
        assert_eq!(r.binding_constraint, "margin");
    }

    #[test]
    fn too_wide_a_stop_fits_zero_contracts() {
        // 100-pt stop = $5000/contract against a $1000 budget.
        let mut inp = es();
        inp.stop = 4900.0;
        let r = compute(&inp).unwrap();
        assert_eq!(r.contracts, 0);
        assert_eq!(r.binding_constraint, "none");
        assert_eq!(r.total_risk, 0.0);
    }

    #[test]
    fn short_side_is_symmetric() {
        let mut inp = es();
        inp.entry = 4990.0;
        inp.stop = 5000.0;
        let r = compute(&inp).unwrap();
        assert_eq!(r.contracts, 2);
        assert!((r.stop_distance_points - 10.0).abs() < 1e-12);
    }

    #[test]
    fn hostile_inputs_return_none() {
        let mut bad = es();
        bad.stop = 5000.0; // zero stop distance
        assert!(compute(&bad).is_none());
        let mut bad = es();
        bad.tick_size = 0.0;
        assert!(compute(&bad).is_none());
        let mut bad = es();
        bad.risk_pct = 0.0;
        assert!(compute(&bad).is_none());
        let mut bad = es();
        bad.account = f64::NAN;
        assert!(compute(&bad).is_none());
    }
}
