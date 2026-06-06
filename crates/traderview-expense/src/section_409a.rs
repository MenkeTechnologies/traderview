//! IRC §409A — Nonqualified deferred compensation compliance.
//!
//! Major executive exposure for NQDC plans (top-hat plans,
//! supplemental executive retirement plans, deferred bonus
//! arrangements, equity compensation deferral, etc.). Noncompliance
//! triggers three layered penalties:
//!
//!   1. **§409A(a)(1)(A) immediate income inclusion** — ALL vested
//!      deferred amounts become taxable in the year of failure (not the
//!      year of distribution).
//!   2. **§409A(a)(1)(B) 20% additional tax** — separate 20% tax on
//!      the same amount.
//!   3. **§409A(a)(1)(C) "premium interest tax"** — interest at the
//!      IRS underpayment rate **+ 1%** from the year of initial
//!      deferral (or first vesting if later) to the year of failure.
//!
//! Combined effect on a $1M deferral with 5 years of deferral at a 9%
//! premium rate (8% IRS + 1%): ordinary income tax + 20% extra +
//! roughly 45% interest = staggering total burden.
//!
//! **§409A(a)(2)(A) permitted distribution events** (deferred comp may
//! only be distributed on):
//!   - Separation from service (with 6-month delay for specified
//!     employees of public companies — §409A(a)(2)(B)(i))
//!   - Disability
//!   - Death
//!   - Time or schedule specified at deferral
//!   - Change in control
//!   - Unforeseeable emergency
//!
//! **§409A(a)(2)(B)(i) specified-employee 6-month delay**: for "key
//! employees" of public companies (≈ top 50 paid + 1% owners + 5%
//! owners + certain officers per §416(i)(1)), distributions on
//! account of separation from service may not be made during the
//! first 6 months after separation. Private companies are exempt.
//!
//! **§409A(a)(3) anti-acceleration rule**: once a distribution time
//! is fixed, it may NOT be accelerated. Limited exceptions (de minimis
//! cash-outs, conflict-of-interest divestiture, plan termination
//! within 12 months of change in control) are caller-side determined.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DistributionEvent {
    SeparationFromService,
    Disability,
    Death,
    SpecifiedTimeOrSchedule,
    ChangeInControl,
    UnforeseeableEmergency,
    /// Any other trigger that doesn't fall in the permitted list —
    /// drives a §409A(a)(2) violation.
    OtherImpermissible,
}

impl DistributionEvent {
    pub fn is_permitted(&self) -> bool {
        !matches!(self, DistributionEvent::OtherImpermissible)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section409aInput {
    /// Total NQDC amount vested as of the year being tested.
    pub deferred_amount_vested: Decimal,
    pub distribution_event: DistributionEvent,
    /// True if the recipient is a §416(i) "key employee" of a publicly-
    /// traded company subject to the 6-month delay.
    pub specified_employee_of_public_company: bool,
    /// Months elapsed between separation from service and distribution.
    /// `None` if the distribution is not on account of separation.
    pub months_since_separation: Option<u32>,
    /// True if the distribution was accelerated from the originally
    /// fixed schedule — triggers §409A(a)(3) violation unless caller
    /// confirms a regulatory exception applies.
    pub distribution_accelerated_from_original_schedule: bool,
    pub years_since_initial_deferral: u32,
    /// IRS underpayment rate in basis points (e.g., 800 = 8%) for the
    /// "premium interest" computation. §409A adds +1% (100bp) to this.
    pub irs_underpayment_rate_basis_points: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section409aResult {
    pub plan_compliant: bool,
    pub violation_details: Vec<String>,
    /// §409A(a)(1)(A) — vested amount included in current-year income.
    /// Equals `deferred_amount_vested` if any violation occurred.
    pub immediate_income_inclusion: Decimal,
    /// §409A(a)(1)(B) — 20% additional tax on the included amount.
    pub additional_20_percent_tax: Decimal,
    /// §409A(a)(1)(C) — premium interest tax at IRS underpayment rate
    /// plus 1%, compounded over `years_since_initial_deferral`.
    /// Simplified to simple interest for compute purposes; actual
    /// calculation is more complex per Treas. Reg. § 1.409A-4(b)(7).
    pub premium_interest_tax: Decimal,
    /// Sum of the 20% additional tax + premium interest.
    pub total_penalty: Decimal,
    /// True if §409A(a)(2)(B)(i) specified-employee 6-month delay
    /// applies to this distribution.
    pub specified_employee_delay_required: bool,
    /// `Some(true)` if the 6-month delay was satisfied; `Some(false)` if
    /// it applied and was not satisfied; `None` if the delay didn't
    /// apply.
    pub delay_period_satisfied: Option<bool>,
    pub note: String,
}

const SIX_MONTH_DELAY_MONTHS: u32 = 6;
const ADDITIONAL_TAX_BP: u32 = 2000; // 20%
const PREMIUM_RATE_ADDITION_BP: u32 = 100; // +1%

pub fn compute(input: &Section409aInput) -> Section409aResult {
    let mut violations: Vec<String> = Vec::new();

    // Check 1: Permitted distribution event.
    if !input.distribution_event.is_permitted() {
        violations.push(format!(
            "§409A(a)(2) — distribution event {:?} is not on the permitted list (separation, disability, death, specified time/schedule, change in control, unforeseeable emergency)",
            input.distribution_event
        ));
    }

    // Check 2: §409A(a)(2)(B)(i) specified-employee 6-month delay.
    let delay_required = input.specified_employee_of_public_company
        && matches!(
            input.distribution_event,
            DistributionEvent::SeparationFromService
        );
    let delay_satisfied = if delay_required {
        match input.months_since_separation {
            Some(months) => Some(months >= SIX_MONTH_DELAY_MONTHS),
            None => Some(false),
        }
    } else {
        None
    };
    if let Some(false) = delay_satisfied {
        violations.push(format!(
            "§409A(a)(2)(B)(i) — specified-employee of public company; 6-month delay after separation required; only {}m elapsed",
            input.months_since_separation.unwrap_or(0)
        ));
    }

    // Check 3: Anti-acceleration rule.
    if input.distribution_accelerated_from_original_schedule {
        violations.push(
            "§409A(a)(3) — distribution was accelerated from originally fixed schedule (no regulatory exception confirmed)".to_string(),
        );
    }

    let compliant = violations.is_empty();

    // Compute penalties only if non-compliant.
    let (income_inclusion, additional_tax, premium_interest, total_penalty) = if compliant {
        (Decimal::ZERO, Decimal::ZERO, Decimal::ZERO, Decimal::ZERO)
    } else {
        let income = input.deferred_amount_vested;
        let add_tax = income * Decimal::from(ADDITIONAL_TAX_BP) / Decimal::from(10_000);
        // Premium interest: IRS rate + 1% × years × amount (simple
        // interest approximation).
        let premium_rate_bp = input.irs_underpayment_rate_basis_points + PREMIUM_RATE_ADDITION_BP;
        let premium_interest = income * Decimal::from(premium_rate_bp) / Decimal::from(10_000)
            * Decimal::from(input.years_since_initial_deferral);
        let total = add_tax + premium_interest;
        (income, add_tax, premium_interest, total)
    };

    let note = if compliant {
        format!(
            "§409A compliant — distribution event permitted; specified-employee delay {}; anti-acceleration not triggered",
            match delay_satisfied {
                Some(true) => "satisfied",
                Some(false) => "FAILED",
                None => "n/a",
            }
        )
    } else {
        format!(
            "§409A NON-COMPLIANT — {} violation(s); ${} immediate income inclusion + ${} 20% additional tax + ${} premium interest = ${} total penalty",
            violations.len(),
            income_inclusion.round_dp(2),
            additional_tax.round_dp(2),
            premium_interest.round_dp(2),
            total_penalty.round_dp(2),
        )
    };

    Section409aResult {
        plan_compliant: compliant,
        violation_details: violations,
        immediate_income_inclusion: income_inclusion,
        additional_20_percent_tax: additional_tax,
        premium_interest_tax: premium_interest,
        total_penalty,
        specified_employee_delay_required: delay_required,
        delay_period_satisfied: delay_satisfied,
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn compliant_base() -> Section409aInput {
        Section409aInput {
            deferred_amount_vested: dec!(1_000_000),
            distribution_event: DistributionEvent::SpecifiedTimeOrSchedule,
            specified_employee_of_public_company: false,
            months_since_separation: None,
            distribution_accelerated_from_original_schedule: false,
            years_since_initial_deferral: 5,
            irs_underpayment_rate_basis_points: 800,
        }
    }

    #[test]
    fn compliant_baseline_no_penalty() {
        let r = compute(&compliant_base());
        assert!(r.plan_compliant);
        assert_eq!(r.immediate_income_inclusion, Decimal::ZERO);
        assert_eq!(r.additional_20_percent_tax, Decimal::ZERO);
        assert_eq!(r.premium_interest_tax, Decimal::ZERO);
        assert_eq!(r.total_penalty, Decimal::ZERO);
    }

    #[test]
    fn impermissible_distribution_event_triggers_violation() {
        let mut i = compliant_base();
        i.distribution_event = DistributionEvent::OtherImpermissible;
        let r = compute(&i);
        assert!(!r.plan_compliant);
        assert!(r
            .violation_details
            .iter()
            .any(|v| v.contains("§409A(a)(2)")));
    }

    #[test]
    fn specified_employee_separation_under_6_months_violates() {
        let mut i = compliant_base();
        i.distribution_event = DistributionEvent::SeparationFromService;
        i.specified_employee_of_public_company = true;
        i.months_since_separation = Some(3);
        let r = compute(&i);
        assert!(!r.plan_compliant);
        assert_eq!(r.delay_period_satisfied, Some(false));
        assert!(r
            .violation_details
            .iter()
            .any(|v| v.contains("§409A(a)(2)(B)(i)")));
    }

    #[test]
    fn specified_employee_separation_6_months_exact_complies() {
        let mut i = compliant_base();
        i.distribution_event = DistributionEvent::SeparationFromService;
        i.specified_employee_of_public_company = true;
        i.months_since_separation = Some(6);
        let r = compute(&i);
        assert!(r.plan_compliant);
        assert_eq!(r.delay_period_satisfied, Some(true));
    }

    #[test]
    fn specified_employee_separation_7_months_complies() {
        let mut i = compliant_base();
        i.distribution_event = DistributionEvent::SeparationFromService;
        i.specified_employee_of_public_company = true;
        i.months_since_separation = Some(7);
        let r = compute(&i);
        assert!(r.plan_compliant);
    }

    #[test]
    fn non_specified_employee_no_delay_required() {
        let mut i = compliant_base();
        i.distribution_event = DistributionEvent::SeparationFromService;
        i.specified_employee_of_public_company = false;
        i.months_since_separation = Some(0);
        let r = compute(&i);
        assert!(!r.specified_employee_delay_required);
        assert!(r.plan_compliant);
    }

    #[test]
    fn anti_acceleration_violation_triggers() {
        let mut i = compliant_base();
        i.distribution_accelerated_from_original_schedule = true;
        let r = compute(&i);
        assert!(!r.plan_compliant);
        assert!(r
            .violation_details
            .iter()
            .any(|v| v.contains("§409A(a)(3)")));
    }

    #[test]
    fn twenty_percent_additional_tax_on_violation() {
        // $1M deferral non-compliant → 20% × $1M = $200k additional tax.
        let mut i = compliant_base();
        i.distribution_event = DistributionEvent::OtherImpermissible;
        let r = compute(&i);
        assert_eq!(r.additional_20_percent_tax, dec!(200_000));
    }

    #[test]
    fn premium_interest_includes_one_percent_addition() {
        // IRS 8% + 1% = 9% per year × 5 years × $1M = $450k.
        let mut i = compliant_base();
        i.distribution_event = DistributionEvent::OtherImpermissible;
        i.irs_underpayment_rate_basis_points = 800;
        i.years_since_initial_deferral = 5;
        let r = compute(&i);
        assert_eq!(r.premium_interest_tax, dec!(450_000));
    }

    #[test]
    fn total_penalty_includes_additional_tax_plus_premium_interest() {
        // 20% tax $200k + premium interest $450k = $650k total.
        let mut i = compliant_base();
        i.distribution_event = DistributionEvent::OtherImpermissible;
        let r = compute(&i);
        assert_eq!(r.total_penalty, dec!(650_000));
    }

    #[test]
    fn immediate_income_inclusion_equals_vested_amount() {
        let mut i = compliant_base();
        i.distribution_event = DistributionEvent::OtherImpermissible;
        let r = compute(&i);
        assert_eq!(r.immediate_income_inclusion, dec!(1_000_000));
    }

    #[test]
    fn multiple_violations_stack_in_list() {
        // Impermissible event + acceleration + specified employee short
        // delay → 3 violations.
        let mut i = compliant_base();
        i.distribution_event = DistributionEvent::OtherImpermissible;
        i.specified_employee_of_public_company = true;
        i.distribution_accelerated_from_original_schedule = true;
        let r = compute(&i);
        // Only the impermissible-event + acceleration trigger; the
        // specified-employee delay only fires for separation events.
        assert_eq!(r.violation_details.len(), 2);
    }

    #[test]
    fn three_separate_violations_for_separation_path() {
        // Separation + short delay + acceleration = 2 violations
        // (anti-acceleration + specified-employee delay).
        let mut i = compliant_base();
        i.distribution_event = DistributionEvent::SeparationFromService;
        i.specified_employee_of_public_company = true;
        i.months_since_separation = Some(2);
        i.distribution_accelerated_from_original_schedule = true;
        let r = compute(&i);
        assert_eq!(r.violation_details.len(), 2);
    }

    #[test]
    fn all_permitted_events_compliant() {
        // Each permitted distribution event individually should comply
        // (with appropriate other inputs).
        let events = [
            DistributionEvent::SpecifiedTimeOrSchedule,
            DistributionEvent::Disability,
            DistributionEvent::Death,
            DistributionEvent::ChangeInControl,
            DistributionEvent::UnforeseeableEmergency,
        ];
        for event in events {
            let mut i = compliant_base();
            i.distribution_event = event;
            let r = compute(&i);
            assert!(r.plan_compliant, "{event:?} should comply");
        }
    }

    #[test]
    fn separation_event_alone_complies_when_no_specified_employee_status() {
        let mut i = compliant_base();
        i.distribution_event = DistributionEvent::SeparationFromService;
        i.months_since_separation = Some(0); // immediate separation OK for non-specified
        let r = compute(&i);
        assert!(r.plan_compliant);
    }

    #[test]
    fn distribution_event_helper_classifies_correctly() {
        assert!(DistributionEvent::SeparationFromService.is_permitted());
        assert!(DistributionEvent::Disability.is_permitted());
        assert!(DistributionEvent::Death.is_permitted());
        assert!(DistributionEvent::SpecifiedTimeOrSchedule.is_permitted());
        assert!(DistributionEvent::ChangeInControl.is_permitted());
        assert!(DistributionEvent::UnforeseeableEmergency.is_permitted());
        assert!(!DistributionEvent::OtherImpermissible.is_permitted());
    }

    #[test]
    fn zero_years_deferral_no_premium_interest() {
        let mut i = compliant_base();
        i.distribution_event = DistributionEvent::OtherImpermissible;
        i.years_since_initial_deferral = 0;
        let r = compute(&i);
        assert_eq!(r.premium_interest_tax, Decimal::ZERO);
        // 20% tax still applies.
        assert_eq!(r.additional_20_percent_tax, dec!(200_000));
    }

    #[test]
    fn zero_deferral_no_penalty_even_with_violations() {
        let mut i = compliant_base();
        i.deferred_amount_vested = Decimal::ZERO;
        i.distribution_event = DistributionEvent::OtherImpermissible;
        let r = compute(&i);
        // Plan flagged non-compliant but no dollars to tax.
        assert!(!r.plan_compliant);
        assert_eq!(r.total_penalty, Decimal::ZERO);
    }

    #[test]
    fn very_large_deferral_no_precision_loss() {
        let mut i = compliant_base();
        i.deferred_amount_vested = dec!(100_000_000); // $100M
        i.distribution_event = DistributionEvent::OtherImpermissible;
        i.irs_underpayment_rate_basis_points = 600; // 6%
        i.years_since_initial_deferral = 10;
        let r = compute(&i);
        assert_eq!(r.additional_20_percent_tax, dec!(20_000_000));
        // 6% + 1% = 7% × 10y × $100M = $70M
        assert_eq!(r.premium_interest_tax, dec!(70_000_000));
        assert_eq!(r.total_penalty, dec!(90_000_000));
    }

    #[test]
    fn note_describes_compliant_path() {
        let r = compute(&compliant_base());
        assert!(r.note.contains("§409A compliant"));
    }

    #[test]
    fn note_describes_non_compliant_path_with_violation_count_and_amounts() {
        let mut i = compliant_base();
        i.distribution_event = DistributionEvent::OtherImpermissible;
        let r = compute(&i);
        assert!(r.note.contains("NON-COMPLIANT"));
        assert!(r.note.contains("$1000000") || r.note.contains("1000000"));
        assert!(r.note.contains("$200000") || r.note.contains("200000"));
    }
}
