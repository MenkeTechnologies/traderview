//! Emergency-fund readiness calculator.
//!
//! Standard personal-finance rule (Bach / Dave Ramsey / Bogleheads):
//! hold 3, 6, 9, or 12 months of essential monthly expenses in liquid
//! cash. Given current fund + monthly expenses + target months +
//! monthly contribution, computes:
//!
//!   - months_covered_now = current_fund / monthly_expenses
//!   - target_amount_usd = monthly_expenses × target_months
//!   - gap_usd = max(0, target_amount - current_fund)
//!   - months_to_target = gap / monthly_contribution (None if gap > 0
//!     AND monthly_contribution ≤ 0 — never reached)
//!   - status = "complete" | "on-track" | "underfunded"
//!   - scenarios = 3m / 6m / 9m / 12m sensitivity at the same
//!     contribution rate so the user sees how far from each preset
//!     they are
//!
//! Pure compute — no DB I/O, no clock reads, no randomness.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct EmergencyFundInput {
    pub monthly_expenses_usd: f64,
    pub current_fund_usd: f64,
    pub target_months: f64,
    pub monthly_contribution_usd: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct EmergencyFundInputEcho {
    pub monthly_expenses_usd: f64,
    pub current_fund_usd: f64,
    pub target_months: f64,
    pub monthly_contribution_usd: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScenarioCell {
    pub target_months: f64,
    pub target_amount_usd: f64,
    pub gap_usd: f64,
    pub months_to_target: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EmergencyFundReport {
    pub input: EmergencyFundInputEcho,
    pub months_covered_now: f64,
    pub target_amount_usd: f64,
    pub gap_usd: f64,
    pub months_to_target: Option<f64>,
    pub status: String,
    pub scenarios: Vec<ScenarioCell>,
}

// ─── Pure compute ─────────────────────────────────────────────────────────

pub fn months_covered(current_fund: f64, monthly_expenses: f64) -> f64 {
    if monthly_expenses <= 0.0 {
        return 0.0;
    }
    current_fund / monthly_expenses
}

pub fn target_amount(monthly_expenses: f64, target_months: f64) -> f64 {
    if monthly_expenses <= 0.0 || target_months <= 0.0 {
        return 0.0;
    }
    monthly_expenses * target_months
}

pub fn months_to_target(gap_usd: f64, monthly_contribution: f64) -> Option<f64> {
    if gap_usd <= 0.0 {
        return Some(0.0);
    }
    if monthly_contribution <= 0.0 {
        return None;
    }
    Some(gap_usd / monthly_contribution)
}

pub fn status_label(months_now: f64, target_months: f64) -> &'static str {
    if target_months <= 0.0 {
        return "underfunded";
    }
    if months_now >= target_months {
        "complete"
    } else if months_now >= target_months * 0.5 {
        "on-track"
    } else {
        "underfunded"
    }
}

pub fn compute(input: &EmergencyFundInput) -> EmergencyFundReport {
    let months_now = months_covered(input.current_fund_usd, input.monthly_expenses_usd);
    let target = target_amount(input.monthly_expenses_usd, input.target_months);
    let gap = (target - input.current_fund_usd).max(0.0);
    let to_target = months_to_target(gap, input.monthly_contribution_usd);
    let status = status_label(months_now, input.target_months).to_string();
    let mut scenarios: Vec<ScenarioCell> = Vec::with_capacity(4);
    for m in &[3.0_f64, 6.0, 9.0, 12.0] {
        let amt = target_amount(input.monthly_expenses_usd, *m);
        let g = (amt - input.current_fund_usd).max(0.0);
        let mt = months_to_target(g, input.monthly_contribution_usd);
        scenarios.push(ScenarioCell {
            target_months: *m,
            target_amount_usd: amt,
            gap_usd: g,
            months_to_target: mt,
        });
    }
    EmergencyFundReport {
        input: EmergencyFundInputEcho {
            monthly_expenses_usd: input.monthly_expenses_usd,
            current_fund_usd: input.current_fund_usd,
            target_months: input.target_months,
            monthly_contribution_usd: input.monthly_contribution_usd,
        },
        months_covered_now: months_now,
        target_amount_usd: target,
        gap_usd: gap,
        months_to_target: to_target,
        status,
        scenarios,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn months_covered_zero_expenses_returns_zero() {
        assert_eq!(months_covered(10_000.0, 0.0), 0.0);
    }

    #[test]
    fn months_covered_basic() {
        assert_eq!(months_covered(15_000.0, 3_000.0), 5.0);
    }

    #[test]
    fn target_amount_six_months_basic() {
        assert_eq!(target_amount(3_000.0, 6.0), 18_000.0);
    }

    #[test]
    fn target_amount_zero_inputs_return_zero() {
        assert_eq!(target_amount(0.0, 6.0), 0.0);
        assert_eq!(target_amount(3_000.0, 0.0), 0.0);
    }

    #[test]
    fn months_to_target_zero_gap_is_done() {
        assert_eq!(months_to_target(0.0, 500.0), Some(0.0));
    }

    #[test]
    fn months_to_target_basic_division() {
        assert_eq!(months_to_target(3_000.0, 500.0), Some(6.0));
    }

    #[test]
    fn months_to_target_no_contribution_with_gap_is_none() {
        assert_eq!(months_to_target(3_000.0, 0.0), None);
    }

    #[test]
    fn status_complete_at_or_above_target() {
        assert_eq!(status_label(6.5, 6.0), "complete");
        assert_eq!(status_label(6.0, 6.0), "complete");
    }

    #[test]
    fn status_on_track_between_half_and_full() {
        assert_eq!(status_label(3.0, 6.0), "on-track");
        assert_eq!(status_label(5.5, 6.0), "on-track");
    }

    #[test]
    fn status_underfunded_below_half() {
        assert_eq!(status_label(1.0, 6.0), "underfunded");
        assert_eq!(status_label(2.99, 6.0), "underfunded");
    }

    #[test]
    fn compute_full_report_24_months_to_six_target() {
        let r = compute(&EmergencyFundInput {
            monthly_expenses_usd: 3_000.0,
            current_fund_usd: 6_000.0,
            target_months: 6.0,
            monthly_contribution_usd: 500.0,
        });
        assert_eq!(r.target_amount_usd, 18_000.0);
        assert_eq!(r.gap_usd, 12_000.0);
        assert_eq!(r.months_to_target, Some(24.0));
        assert_eq!(r.scenarios.len(), 4);
        assert_eq!(r.scenarios[0].target_amount_usd, 9_000.0);
        assert_eq!(r.scenarios[3].target_amount_usd, 36_000.0);
        assert_eq!(r.status, "underfunded");
    }

    #[test]
    fn compute_already_funded_zero_gap_complete_status() {
        let r = compute(&EmergencyFundInput {
            monthly_expenses_usd: 3_000.0,
            current_fund_usd: 30_000.0,
            target_months: 6.0,
            monthly_contribution_usd: 500.0,
        });
        assert_eq!(r.gap_usd, 0.0);
        assert_eq!(r.months_to_target, Some(0.0));
        assert_eq!(r.status, "complete");
    }

    #[test]
    fn compute_no_contribution_with_gap_is_none() {
        let r = compute(&EmergencyFundInput {
            monthly_expenses_usd: 3_000.0,
            current_fund_usd: 1_000.0,
            target_months: 6.0,
            monthly_contribution_usd: 0.0,
        });
        assert!(r.gap_usd > 0.0);
        assert_eq!(r.months_to_target, None);
    }
}
