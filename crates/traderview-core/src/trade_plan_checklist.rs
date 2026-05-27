//! Trade-plan checklist enforcer.
//!
//! A planned trade should clear basic discipline gates BEFORE the order
//! goes through:
//!   - Has a written thesis (≥ N words)
//!   - Has a stop loss
//!   - Has a target / R-multiple
//!   - Position size doesn't exceed max % per trade
//!   - R-multiple ≥ minimum (e.g., 1.5R)
//!
//! Each gate returns pass/fail with a human-readable reason. Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedTrade {
    pub thesis: String,
    pub entry_price: f64,
    pub stop_price: Option<f64>,
    pub target_price: Option<f64>,
    /// Notional dollars to risk on this trade.
    pub risk_dollars: f64,
    pub account_equity: f64,
    pub is_long: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct ChecklistConfig {
    pub min_thesis_words: usize,
    pub min_r_multiple: f64,
    pub max_risk_pct_per_trade: f64,
}

impl Default for ChecklistConfig {
    fn default() -> Self {
        Self {
            min_thesis_words: 10,
            min_r_multiple: 1.5,
            max_risk_pct_per_trade: 0.02,    // 2%
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateResult {
    pub gate: String,
    pub passed: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChecklistReport {
    pub gates: Vec<GateResult>,
    pub all_passed: bool,
    pub computed_r_multiple: Option<f64>,
    pub risk_pct: f64,
}

pub fn evaluate(plan: &PlannedTrade, cfg: &ChecklistConfig) -> ChecklistReport {
    let mut report = ChecklistReport::default();
    let mut emit = |gate: &str, passed: bool, reason: String| {
        report.gates.push(GateResult {
            gate: gate.into(),
            passed,
            reason,
        });
    };
    // Thesis check.
    let word_count = plan.thesis.split_whitespace().count();
    emit("thesis_present",
        word_count >= cfg.min_thesis_words,
        format!("{} words (minimum {})", word_count, cfg.min_thesis_words));
    // Stop loss check.
    let has_stop = plan.stop_price.is_some();
    emit("stop_loss_set", has_stop,
        if has_stop { "stop is set".into() }
        else        { "no stop loss defined — naked trade".into() });
    // Target check.
    let has_target = plan.target_price.is_some();
    emit("target_set", has_target,
        if has_target { "target is set".into() }
        else          { "no target — exit discipline missing".into() });
    // R-multiple check (requires both stop AND target).
    if let (Some(stop), Some(target)) = (plan.stop_price, plan.target_price) {
        let risk = (plan.entry_price - stop).abs();
        let reward = (target - plan.entry_price).abs();
        let r = if risk > 0.0 { reward / risk } else { 0.0 };
        report.computed_r_multiple = Some(r);
        emit("r_multiple_meets_minimum",
            r >= cfg.min_r_multiple,
            format!("R = {:.2} (min {:.2})", r, cfg.min_r_multiple));
        // Direction sanity.
        let target_in_direction = if plan.is_long { target > plan.entry_price }
            else                                   { target < plan.entry_price };
        emit("target_in_direction",
            target_in_direction,
            if target_in_direction { "target on profitable side of entry".into() }
            else                    { "target on WRONG side of entry — direction bug".into() });
        let stop_in_direction = if plan.is_long { stop < plan.entry_price }
            else                                 { stop > plan.entry_price };
        emit("stop_in_direction",
            stop_in_direction,
            if stop_in_direction { "stop on loss side of entry".into() }
            else                  { "stop on WRONG side of entry".into() });
    }
    // Risk size check.
    let risk_pct = if plan.account_equity > 0.0 {
        plan.risk_dollars / plan.account_equity
    } else { 0.0 };
    report.risk_pct = risk_pct;
    emit("risk_within_max",
        risk_pct <= cfg.max_risk_pct_per_trade,
        format!("risking {:.2}% (max {:.2}%)",
            risk_pct * 100.0, cfg.max_risk_pct_per_trade * 100.0));
    report.all_passed = report.gates.iter().all(|g| g.passed);
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn long_plan() -> PlannedTrade {
        PlannedTrade {
            thesis: "Breakout above prior month high on heavy volume with sector confirmation.".into(),
            entry_price: 100.0,
            stop_price: Some(98.0),
            target_price: Some(106.0),    // 3R target
            risk_dollars: 200.0,
            account_equity: 50_000.0,
            is_long: true,
        }
    }

    #[test]
    fn good_plan_all_gates_pass() {
        let r = evaluate(&long_plan(), &ChecklistConfig::default());
        assert!(r.all_passed, "complete plan should pass all gates: {:?}", r.gates);
        assert_eq!(r.computed_r_multiple, Some(3.0));
    }

    #[test]
    fn missing_stop_fails() {
        let plan = PlannedTrade { stop_price: None, ..long_plan() };
        let r = evaluate(&plan, &ChecklistConfig::default());
        assert!(!r.all_passed);
        let stop = r.gates.iter().find(|g| g.gate == "stop_loss_set").unwrap();
        assert!(!stop.passed);
    }

    #[test]
    fn missing_target_fails() {
        let plan = PlannedTrade { target_price: None, ..long_plan() };
        let r = evaluate(&plan, &ChecklistConfig::default());
        assert!(!r.all_passed);
    }

    #[test]
    fn short_thesis_fails() {
        let plan = PlannedTrade { thesis: "yolo".into(), ..long_plan() };
        let r = evaluate(&plan, &ChecklistConfig::default());
        let thesis = r.gates.iter().find(|g| g.gate == "thesis_present").unwrap();
        assert!(!thesis.passed);
    }

    #[test]
    fn under_min_r_multiple_fails() {
        // 1R target instead of 3R.
        let plan = PlannedTrade {
            entry_price: 100.0,
            stop_price: Some(98.0),
            target_price: Some(102.0),    // 1R
            ..long_plan()
        };
        let r = evaluate(&plan, &ChecklistConfig::default());
        let rm = r.gates.iter().find(|g| g.gate == "r_multiple_meets_minimum").unwrap();
        assert!(!rm.passed);
    }

    #[test]
    fn oversize_risk_fails() {
        // Risk $2000 on $50k account = 4% > 2% max.
        let plan = PlannedTrade { risk_dollars: 2_000.0, ..long_plan() };
        let r = evaluate(&plan, &ChecklistConfig::default());
        let risk_gate = r.gates.iter().find(|g| g.gate == "risk_within_max").unwrap();
        assert!(!risk_gate.passed);
    }

    #[test]
    fn target_on_wrong_side_for_long_fails() {
        // Long with target BELOW entry — direction bug.
        let plan = PlannedTrade {
            target_price: Some(95.0),
            ..long_plan()
        };
        let r = evaluate(&plan, &ChecklistConfig::default());
        let dir = r.gates.iter().find(|g| g.gate == "target_in_direction").unwrap();
        assert!(!dir.passed);
    }

    #[test]
    fn stop_on_wrong_side_for_long_fails() {
        let plan = PlannedTrade {
            stop_price: Some(105.0),    // above entry for a long? bug.
            ..long_plan()
        };
        let r = evaluate(&plan, &ChecklistConfig::default());
        let dir = r.gates.iter().find(|g| g.gate == "stop_in_direction").unwrap();
        assert!(!dir.passed);
    }

    #[test]
    fn short_trade_uses_flipped_direction_checks() {
        let plan = PlannedTrade {
            entry_price: 100.0,
            stop_price: Some(102.0),     // stop above entry for short — correct
            target_price: Some(94.0),    // target below entry for short — correct
            is_long: false,
            ..long_plan()
        };
        let r = evaluate(&plan, &ChecklistConfig::default());
        let target_dir = r.gates.iter().find(|g| g.gate == "target_in_direction").unwrap();
        let stop_dir = r.gates.iter().find(|g| g.gate == "stop_in_direction").unwrap();
        assert!(target_dir.passed);
        assert!(stop_dir.passed);
    }

    #[test]
    fn custom_config_changes_thresholds() {
        let strict = ChecklistConfig {
            min_thesis_words: 50,    // very high bar
            min_r_multiple: 5.0,
            max_risk_pct_per_trade: 0.001,
        };
        let r = evaluate(&long_plan(), &strict);
        assert!(!r.all_passed, "strict config should fail decent plan");
    }
}
