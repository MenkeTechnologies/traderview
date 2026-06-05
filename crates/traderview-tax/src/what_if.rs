//! Tax "what-if" delta engine.
//!
//! Given a baseline `TaxReturn` and a single field mutation, compute
//! the baseline result, the mutated result, and the delta. Used by the
//! tax wizard's "what-if" panel to answer:
//!   * "What if I contribute another $5k to my Traditional IRA?"
//!   * "What if my W-2 wages were $10k higher?"
//!   * "What if I had 1 more qualifying child?"
//!
//! Each scenario is a (path, value) pair — `path` names the field via
//! a string slug (e.g. `"ira_deduction"`), `value` is the new amount.
//! The engine clones the baseline, applies the mutation, and runs the
//! tax pipeline twice (baseline + mutated) so the UI can show:
//!   delta_refund   = mutated.refund_due - baseline.refund_due
//!   delta_owed     = mutated.tax_owed   - baseline.tax_owed
//!   delta_agi      = mutated.agi - baseline.agi
//!   delta_taxable  = mutated.taxable_income - baseline.taxable_income
//!
//! Why a path-based mutator instead of taking two `TaxReturn`s? The
//! caller (frontend) only needs to send the small `(field, value)`
//! pair — the heavy `TaxReturn` already lives in `tax_returns.data`
//! server-side. Keeps the API minimal and prevents the frontend from
//! drifting from the canonical struct shape.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::engine::{compute, TaxResult, TaxReturn};

/// A single scenario — the field to change and its new value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    /// Field slug. Recognized values match `TaxReturn` field names:
    ///   - `w2_box_1_wages_total` (overwrites the SUM across all W-2s)
    ///   - `interest_income`
    ///   - `ordinary_dividends`
    ///   - `qualified_dividends`
    ///   - `net_long_term_capital_gain`
    ///   - `schedule_c_net_profit` (overwrites net; clears gross/exp)
    ///   - `schedule_e_net_income`
    ///   - `other_income`
    ///   - `hsa_deduction`
    ///   - `ira_deduction`
    ///   - `student_loan_interest`
    ///   - `other_adjustments`
    ///   - `estimated_tax_payments`
    ///   - `qualifying_children_under_17`  (interpreted as integer)
    ///   - `other_dependents`              (integer)
    pub path: String,
    /// The new value. For integer-valued paths (kid counts), the
    /// decimal is rounded down via `to_u32`.
    pub value: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatIfResult {
    pub baseline: TaxResult,
    pub scenario: TaxResult,
    pub scenario_input: Scenario,
    pub delta_refund_due: Decimal,
    pub delta_tax_owed: Decimal,
    pub delta_agi: Decimal,
    pub delta_taxable_income: Decimal,
    pub delta_total_tax: Decimal,
    /// Did the scenario produce a NET benefit (more refund OR less owed)?
    /// Sums refund delta and owed delta (with sign flip on owed) so the
    /// UI can colorize a single "you save / you pay" indicator.
    pub net_dollar_change_in_pocket: Decimal,
}

/// Compute the what-if. Returns `None` when `path` doesn't match any
/// known field — caller should surface a "field not supported" message.
pub fn compute_what_if(baseline: &TaxReturn, scenario: Scenario) -> Option<WhatIfResult> {
    let baseline_result = compute(baseline);

    let mut mutated = baseline.clone();
    if !apply(&mut mutated, &scenario) {
        return None;
    }
    let scenario_result = compute(&mutated);

    let delta_refund_due  = scenario_result.refund_due  - baseline_result.refund_due;
    let delta_tax_owed    = scenario_result.tax_owed    - baseline_result.tax_owed;
    let delta_agi         = scenario_result.agi         - baseline_result.agi;
    let delta_taxable     = scenario_result.taxable_income - baseline_result.taxable_income;
    let delta_total_tax   = scenario_result.tax_after_credits - baseline_result.tax_after_credits;

    // In-pocket = (refund_up - owed_up). Refund increase is good
    // (positive); owed increase is bad (negative). Subtract the
    // signed delta of owed.
    let net_dollar_change_in_pocket = delta_refund_due - delta_tax_owed;

    Some(WhatIfResult {
        baseline: baseline_result,
        scenario: scenario_result,
        scenario_input: scenario,
        delta_refund_due,
        delta_tax_owed,
        delta_agi,
        delta_taxable_income: delta_taxable,
        delta_total_tax,
        net_dollar_change_in_pocket,
    })
}

/// Apply a scenario to a draft. Returns `true` if the path was recognized.
fn apply(r: &mut TaxReturn, s: &Scenario) -> bool {
    use std::convert::TryFrom;
    match s.path.as_str() {
        "w2_box_1_wages_total" => {
            // Overwrite ALL W-2s with a single synthetic entry whose
            // wages = `value`. Withholding zeroed since we're modeling
            // "different wage level" not "different employer".
            r.w2s.clear();
            r.w2s.push(crate::engine::W2 {
                box_1_wages: s.value,
                ..Default::default()
            });
            true
        }
        "interest_income"               => { r.interest_income = s.value; true }
        "ordinary_dividends"            => { r.ordinary_dividends = s.value; true }
        "qualified_dividends"           => { r.qualified_dividends = s.value; true }
        "net_long_term_capital_gain"    => { r.net_long_term_capital_gain = s.value; true }
        "schedule_c_net_profit" => {
            // Caller is modeling net SE income directly. Zero out
            // gross/expenses so engine doesn't double-compute.
            r.schedule_c.gross_receipts = s.value;
            r.schedule_c.total_expenses = Decimal::ZERO;
            r.schedule_c.net_profit = s.value;
            true
        }
        "schedule_e_net_income" => {
            r.schedule_e.gross_rents = s.value;
            r.schedule_e.total_expenses = Decimal::ZERO;
            r.schedule_e.net_income = s.value;
            true
        }
        "other_income"                  => { r.other_income = s.value; true }
        "hsa_deduction"                 => { r.hsa_deduction = s.value; true }
        "ira_deduction"                 => { r.ira_deduction = s.value; true }
        "student_loan_interest"         => { r.student_loan_interest = s.value; true }
        "other_adjustments"             => { r.other_adjustments = s.value; true }
        "estimated_tax_payments"        => { r.estimated_tax_payments = s.value; true }
        "qualifying_children_under_17"  => {
            r.qualifying_children_under_17 = u32::try_from(s.value.trunc().mantissa()).unwrap_or(0);
            true
        }
        "other_dependents" => {
            r.other_dependents = u32::try_from(s.value.trunc().mantissa()).unwrap_or(0);
            true
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{ScheduleC, W2};
    use crate::FilingStatus;

    fn d(n: i64) -> Decimal { Decimal::from(n) }

    fn single_w2_baseline(wages: i64, withholding: i64) -> TaxReturn {
        TaxReturn {
            tax_year: 2025,
            status: FilingStatus::Single,
            w2s: vec![W2 {
                box_1_wages: d(wages),
                box_2_federal_income_tax_withheld: d(withholding),
                ..Default::default()
            }],
            ..Default::default()
        }
    }

    #[test]
    fn ira_deduction_reduces_taxable_income_dollar_for_dollar() {
        // $80k wages single. Bumping IRA contribution by $5k should
        // reduce taxable income by exactly $5k.
        let base = single_w2_baseline(80_000, 8_000);
        let scen = Scenario { path: "ira_deduction".into(), value: d(5_000) };
        let r = compute_what_if(&base, scen).expect("recognized");
        assert_eq!(r.delta_taxable_income, d(-5_000));
        // AGI also reduces by $5k since IRA is above-the-line.
        assert_eq!(r.delta_agi, d(-5_000));
    }

    #[test]
    fn ira_deduction_reduces_tax_by_marginal_rate() {
        // $80k wages, only $8k withheld → baseline is in OWED position
        // ($9,214 bracket tax - $8k withhold = $1,214 owed).
        // Adding $5k IRA → tax drops by 22% × $5k = $1,100.
        // Owed: $1,214 → $114 (delta -$1,100). Refund stays 0.
        // Net in pocket = +$1,100 (you keep more money).
        let base = single_w2_baseline(80_000, 8_000);
        let scen = Scenario { path: "ira_deduction".into(), value: d(5_000) };
        let r = compute_what_if(&base, scen).expect("recognized");
        assert_eq!(r.delta_tax_owed, d(-1_100),
            "owed drops by 22% × $5k = $1,100");
        assert_eq!(r.delta_refund_due, Decimal::ZERO,
            "still in owed position post-IRA; refund stays at 0");
        assert_eq!(r.net_dollar_change_in_pocket, d(1_100),
            "net = +$1,100 (owed dropped by $1,100)");
    }

    #[test]
    fn extra_kid_yields_2000_net_benefit() {
        // Single $80k, 0 kids → 1 kid. Hand-computed split:
        //   Baseline: owes $1,214 (no CTC).
        //   With kid: CTC $2k = $1,700 refundable + $300 nonref.
        //     tax_after_credits = $9,214 - $300 = $8,914.
        //     total_payments    = $8,000 + $1,700 = $9,700.
        //     refund            = $786.
        //   Delta refund = +$786, delta owed = -$1,214.
        //   Net in pocket = $786 - (-$1,214) = $2,000.
        let base = single_w2_baseline(80_000, 8_000);
        let scen = Scenario {
            path: "qualifying_children_under_17".into(),
            value: d(1),
        };
        let r = compute_what_if(&base, scen).expect("recognized");
        assert_eq!(r.delta_refund_due, d(786));
        assert_eq!(r.delta_tax_owed, d(-1_214));
        assert_eq!(r.net_dollar_change_in_pocket, d(2_000),
            "1 kid CTC = $2,000 net benefit");
    }

    #[test]
    fn higher_wages_in_a_new_bracket_increases_owed() {
        // $50k → $200k wages via the wages-overwrite path. Withholding
        // is zeroed by that path (the scenario models "different wage
        // level" not "different employer"), so we ALSO clear baseline
        // withholding to make refund/owed comparison apples-to-apples.
        let base = single_w2_baseline(50_000, 0);
        let scen = Scenario {
            path: "w2_box_1_wages_total".into(),
            value: d(200_000),
        };
        let r = compute_what_if(&base, scen).expect("recognized");
        // Baseline owes 3,961.50 (no withholding).
        assert_eq!(r.baseline.tax_owed, "3961.5".parse::<Decimal>().unwrap());
        assert_eq!(r.baseline.refund_due, Decimal::ZERO);
        // Scenario at $200k crosses into 32% bracket. Tax much higher.
        assert!(r.scenario.tax_owed > d(35_000),
            "$200k wages should owe ≥$35k with no withholding");
        // Net change in pocket is decidedly negative.
        assert!(r.net_dollar_change_in_pocket < d(-30_000));
    }

    #[test]
    fn schedule_c_net_increase_triggers_se_tax_in_delta() {
        // $80k wages baseline, no SE. Add $30k Schedule C net.
        let base = single_w2_baseline(80_000, 8_000);
        let scen = Scenario {
            path: "schedule_c_net_profit".into(),
            value: d(30_000),
        };
        let r = compute_what_if(&base, scen).expect("recognized");
        // SE tax in baseline = 0; scenario = positive.
        assert_eq!(r.baseline.se_tax.total, Decimal::ZERO);
        assert!(r.scenario.se_tax.total > d(4_000),
            "$30k SE net → SE tax > $4k");
        // Delta tax positive (more owed) — net in pocket negative.
        assert!(r.delta_total_tax > Decimal::ZERO);
        assert!(r.net_dollar_change_in_pocket < Decimal::ZERO);
    }

    #[test]
    fn estimated_payment_reduces_owed_dollar_for_dollar() {
        // SE filer who owes $5k can prepay $3k → owed drops by $3k.
        let mut base = TaxReturn {
            tax_year: 2025,
            status: FilingStatus::Single,
            schedule_c: ScheduleC {
                gross_receipts: d(50_000),
                total_expenses: Decimal::ZERO,
                net_profit: d(50_000),
            },
            ..Default::default()
        };
        // Baseline compute to ensure there's actual owed amount.
        let baseline_owed = compute(&base).tax_owed;
        assert!(baseline_owed > d(3_000), "test premise: baseline owes >$3k");
        let _ = baseline_owed;
        base.estimated_tax_payments = Decimal::ZERO;
        let scen = Scenario {
            path: "estimated_tax_payments".into(),
            value: d(3_000),
        };
        let r = compute_what_if(&base, scen).expect("recognized");
        assert_eq!(r.delta_tax_owed, d(-3_000));
        // Owed-down = good (positive net in pocket).
        assert!(r.net_dollar_change_in_pocket > Decimal::ZERO);
    }

    #[test]
    fn unrecognized_path_returns_none() {
        let base = single_w2_baseline(50_000, 5_000);
        let scen = Scenario { path: "garbage_field".into(), value: d(1_000) };
        assert!(compute_what_if(&base, scen).is_none());
    }

    #[test]
    fn hsa_deduction_reduces_agi() {
        // HSA is above-the-line — drops AGI dollar-for-dollar.
        let base = single_w2_baseline(80_000, 8_000);
        let scen = Scenario { path: "hsa_deduction".into(), value: d(4_150) };
        let r = compute_what_if(&base, scen).expect("recognized");
        assert_eq!(r.delta_agi, d(-4_150));
    }

    #[test]
    fn delta_refund_owed_signs_are_consistent() {
        // Bumping IRA on a refund-position return increases refund
        // (positive delta), never inflates owed.
        let base = single_w2_baseline(60_000, 8_000);
        let scen = Scenario { path: "ira_deduction".into(), value: d(2_000) };
        let r = compute_what_if(&base, scen).expect("recognized");
        assert!(r.delta_refund_due >= Decimal::ZERO);
        assert!(r.delta_tax_owed <= Decimal::ZERO);
    }
}
