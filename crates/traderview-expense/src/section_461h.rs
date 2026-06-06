//! IRC § 461(h) economic performance rule — the timing provision
//! determining when accrued liabilities become deductible under the
//! accrual method of accounting.
//!
//! Enacted by Deficit Reduction Act of 1984 (Pub. L. 98-369) to
//! prevent accrual-method taxpayers from deducting liabilities
//! before any cash outflow or actual provision of services/property.
//! Before § 461(h), accrual-method taxpayers could deduct liabilities
//! as soon as the all-events test was satisfied — even if the
//! liability would not be paid for years (or never). § 461(h) added
//! the third prong: economic performance must also occur before
//! deduction is permitted.
//!
//! **Three-prong all-events test under § 461(h)(1) and Treas. Reg.
//! § 1.461-1(a)(2)**: a liability is deductible by an accrual-method
//! taxpayer when (i) all events have occurred to establish the fact
//! of liability, (ii) amount can be determined with reasonable
//! accuracy, AND (iii) economic performance has occurred.
//!
//! **§ 461(h)(2) when economic performance occurs**:
//!
//! - **§ 461(h)(2)(A)(i) services provided to taxpayer**: as
//!   services are provided.
//! - **§ 461(h)(2)(A)(ii) property provided to taxpayer**: as
//!   property is provided.
//! - **§ 461(h)(2)(A)(iii) use of property**: as property is used.
//! - **§ 461(h)(2)(B) services or property provided BY taxpayer**:
//!   as taxpayer provides such services or property.
//! - **§ 461(h)(2)(C) workers compensation, tort, breach of
//!   contract, or violation of law**: as payments to such person
//!   are made.
//! - **§ 461(h)(2)(D) other liabilities**: at time determined under
//!   regulations prescribed by Secretary.
//!
//! **§ 461(h)(3) recurring item exception**: taxpayer may deduct
//! an accrued expense at end of taxable year IN ADVANCE of
//! economic performance if FOUR conditions satisfied:
//!
//! 1. As of end of taxable year, all events have occurred that
//!    establish the fact of liability and amount can be determined
//!    with reasonable accuracy.
//! 2. Economic performance occurs within **8.5 months** after the
//!    close of the taxable year, OR by date of timely filing of
//!    return for that year (whichever is earlier).
//! 3. Item is recurring + taxpayer consistently treats similar
//!    items as incurred in the tax year.
//! 4. Either (a) item is NOT MATERIAL, or (b) accrual of the item
//!    for that taxable year results in BETTER MATCHING of the item
//!    with the income to which it relates (compared with accruing
//!    the liability in the year of economic performance).
//!
//! **§ 461(h)(4) reserves for estimated expenses prohibited**: a
//! taxpayer may not, in the absence of express statutory or
//! regulatory authority, deduct an estimated amount of expenses or
//! losses for the year.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[allow(dead_code)]
pub const RECURRING_ITEM_EXCEPTION_MONTHS_X_10: u32 = 85;
#[allow(dead_code)]
pub const DRA_1984_ENACTMENT_YEAR: u32 = 1984;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LiabilityType {
    NotApplicable,
    ServicesProvidedToTaxpayer461h2Ai,
    PropertyProvidedToTaxpayer461h2Aii,
    UseOfProperty461h2Aiii,
    ServicesOrPropertyProvidedByTaxpayer461h2B,
    WorkersCompTortBreachViolationOfLaw461h2C,
    OtherLiability461h2D,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeductionTimingYear {
    NotApplicable,
    CurrentYearEconomicPerformanceOccurred,
    RecurringItemExceptionUnder461h3,
    DeferredToEconomicPerformanceYear,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    CompliantServicesOrPropertyProvidedToTaxpayerUnder461h2A,
    CompliantServicesOrPropertyProvidedByTaxpayerUnder461h2B,
    CompliantPaymentLiabilitiesWorkersCompTortBreachUnder461h2C,
    CompliantOtherLiabilityRegulatoryDeterminationUnder461h2D,
    CompliantRecurringItemExceptionUnder461h3,
    ViolationLiabilityDeductedBeforeEconomicPerformance,
    ViolationAllEventsTestNotMet,
    ViolationReservesForEstimatedExpensesUnder461h4,
    ViolationRecurringItemMissedEightAndHalfMonthDeadline,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub liability_type: LiabilityType,
    pub all_events_test_satisfied: bool,
    pub amount_determinable_with_reasonable_accuracy: bool,
    pub economic_performance_occurred: bool,
    pub recurring_item_exception_claimed: bool,
    pub months_to_economic_performance_after_taxable_year_x_10: u32,
    pub item_is_recurring: bool,
    pub consistent_treatment_of_similar_items: bool,
    pub item_immaterial_or_matching_improved: bool,
    pub reserves_for_estimated_expenses_claimed: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Output {
    pub severity: Severity,
    pub deduction_allowable: bool,
    pub deduction_timing_year: DeductionTimingYear,
    pub notes: Vec<String>,
    pub citations: Vec<String>,
}

pub type Section461hInput = Input;
pub type Section461hOutput = Output;
pub type Section461hResult = Output;

pub fn check(input: &Input) -> Output {
    let mut notes: Vec<String> = Vec::new();
    let citations: Vec<String> = vec![
        "IRC § 461(h)(1) (economic performance general rule)".to_string(),
        "IRC § 461(h)(2)(A)(i) (services provided to taxpayer)".to_string(),
        "IRC § 461(h)(2)(A)(ii) (property provided to taxpayer)".to_string(),
        "IRC § 461(h)(2)(A)(iii) (use of property)".to_string(),
        "IRC § 461(h)(2)(B) (services or property provided by taxpayer)".to_string(),
        "IRC § 461(h)(2)(C) (workers comp, tort, breach, violation of law — payment liability)"
            .to_string(),
        "IRC § 461(h)(2)(D) (other liabilities — regulatory determination)".to_string(),
        "IRC § 461(h)(3) (recurring item exception — 8.5 month rule)".to_string(),
        "IRC § 461(h)(4) (reserves for estimated expenses prohibited)".to_string(),
        "IRC § 446(c)(2) (accrual method cross-reference)".to_string(),
        "Treas. Reg. § 1.461-1(a)(2) (all-events test three prongs)".to_string(),
        "Treas. Reg. § 1.461-4 (economic performance — implementing regulations)".to_string(),
        "Treas. Reg. § 1.461-5 (recurring item exception)".to_string(),
        "Deficit Reduction Act of 1984 (Pub. L. 98-369) — § 461(h) enactment".to_string(),
    ];

    if matches!(input.liability_type, LiabilityType::NotApplicable) {
        notes.push("No accrual-method liability under § 461(h) analysis recorded.".to_string());
        return Output {
            severity: Severity::NotApplicable,
            deduction_allowable: false,
            deduction_timing_year: DeductionTimingYear::NotApplicable,
            notes,
            citations,
        };
    }

    if input.reserves_for_estimated_expenses_claimed {
        notes.push("§ 461(h)(4) prohibits deduction of reserves for estimated expenses absent express statutory authority — per se violation.".to_string());
        return Output {
            severity: Severity::ViolationReservesForEstimatedExpensesUnder461h4,
            deduction_allowable: false,
            deduction_timing_year: DeductionTimingYear::NotApplicable,
            notes,
            citations,
        };
    }

    if !input.all_events_test_satisfied || !input.amount_determinable_with_reasonable_accuracy {
        notes.push("All-events test not satisfied (fact of liability or determinable amount missing) — Treas. Reg. § 1.461-1(a)(2) violation; deduction disallowed.".to_string());
        return Output {
            severity: Severity::ViolationAllEventsTestNotMet,
            deduction_allowable: false,
            deduction_timing_year: DeductionTimingYear::DeferredToEconomicPerformanceYear,
            notes,
            citations,
        };
    }

    if input.recurring_item_exception_claimed {
        let within_eight_and_half_months = input
            .months_to_economic_performance_after_taxable_year_x_10
            <= RECURRING_ITEM_EXCEPTION_MONTHS_X_10;
        if !within_eight_and_half_months {
            notes.push(format!(
                "Recurring item exception claimed but economic performance {} months after close of taxable year exceeds 8.5-month limit — § 461(h)(3) violation.",
                input.months_to_economic_performance_after_taxable_year_x_10 / 10
            ));
            return Output {
                severity: Severity::ViolationRecurringItemMissedEightAndHalfMonthDeadline,
                deduction_allowable: false,
                deduction_timing_year: DeductionTimingYear::DeferredToEconomicPerformanceYear,
                notes,
                citations,
            };
        }
        if !input.item_is_recurring
            || !input.consistent_treatment_of_similar_items
            || !input.item_immaterial_or_matching_improved
        {
            notes.push("Recurring item exception claimed but one or more § 461(h)(3) requirements unmet (recurrence + consistency + immaterial-or-better-matching).".to_string());
            return Output {
                severity: Severity::ViolationLiabilityDeductedBeforeEconomicPerformance,
                deduction_allowable: false,
                deduction_timing_year: DeductionTimingYear::DeferredToEconomicPerformanceYear,
                notes,
                citations,
            };
        }
        notes.push("§ 461(h)(3) recurring item exception applied: all four conditions met (all-events test + economic performance within 8.5 months + recurring + immaterial or better matching).".to_string());
        return Output {
            severity: Severity::CompliantRecurringItemExceptionUnder461h3,
            deduction_allowable: true,
            deduction_timing_year: DeductionTimingYear::RecurringItemExceptionUnder461h3,
            notes,
            citations,
        };
    }

    if !input.economic_performance_occurred {
        notes.push("Liability accrued but economic performance has NOT occurred and recurring item exception not claimed — deduction deferred to year economic performance occurs.".to_string());
        return Output {
            severity: Severity::ViolationLiabilityDeductedBeforeEconomicPerformance,
            deduction_allowable: false,
            deduction_timing_year: DeductionTimingYear::DeferredToEconomicPerformanceYear,
            notes,
            citations,
        };
    }

    let severity = match input.liability_type {
        LiabilityType::ServicesProvidedToTaxpayer461h2Ai
        | LiabilityType::PropertyProvidedToTaxpayer461h2Aii
        | LiabilityType::UseOfProperty461h2Aiii => {
            notes.push("§ 461(h)(2)(A): economic performance occurs as services/property provided to taxpayer or property used; deduction allowable in year economic performance occurs.".to_string());
            Severity::CompliantServicesOrPropertyProvidedToTaxpayerUnder461h2A
        }
        LiabilityType::ServicesOrPropertyProvidedByTaxpayer461h2B => {
            notes.push("§ 461(h)(2)(B): economic performance occurs as taxpayer provides services or property; deduction allowable in year economic performance occurs.".to_string());
            Severity::CompliantServicesOrPropertyProvidedByTaxpayerUnder461h2B
        }
        LiabilityType::WorkersCompTortBreachViolationOfLaw461h2C => {
            notes.push("§ 461(h)(2)(C) payment-liability rule: workers compensation, tort, breach, or violation of law — economic performance occurs as payments to such person are made; deduction allowable in year of payment.".to_string());
            Severity::CompliantPaymentLiabilitiesWorkersCompTortBreachUnder461h2C
        }
        LiabilityType::OtherLiability461h2D => {
            notes.push("§ 461(h)(2)(D): other liability — economic performance occurs at time determined under regulations (typically Treas. Reg. § 1.461-4).".to_string());
            Severity::CompliantOtherLiabilityRegulatoryDeterminationUnder461h2D
        }
        LiabilityType::NotApplicable => unreachable!(),
    };

    Output {
        severity,
        deduction_allowable: true,
        deduction_timing_year: DeductionTimingYear::CurrentYearEconomicPerformanceOccurred,
        notes,
        citations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_services_to_taxpayer() -> Input {
        Input {
            liability_type: LiabilityType::ServicesProvidedToTaxpayer461h2Ai,
            all_events_test_satisfied: true,
            amount_determinable_with_reasonable_accuracy: true,
            economic_performance_occurred: true,
            recurring_item_exception_claimed: false,
            months_to_economic_performance_after_taxable_year_x_10: 0,
            item_is_recurring: false,
            consistent_treatment_of_similar_items: false,
            item_immaterial_or_matching_improved: false,
            reserves_for_estimated_expenses_claimed: false,
        }
    }

    #[test]
    fn services_to_taxpayer_compliant_461h2a() {
        let out = check(&base_services_to_taxpayer());
        assert_eq!(
            out.severity,
            Severity::CompliantServicesOrPropertyProvidedToTaxpayerUnder461h2A
        );
        assert!(out.deduction_allowable);
    }

    #[test]
    fn property_to_taxpayer_compliant_461h2aii() {
        let mut i = base_services_to_taxpayer();
        i.liability_type = LiabilityType::PropertyProvidedToTaxpayer461h2Aii;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::CompliantServicesOrPropertyProvidedToTaxpayerUnder461h2A
        );
    }

    #[test]
    fn use_of_property_compliant_461h2aiii() {
        let mut i = base_services_to_taxpayer();
        i.liability_type = LiabilityType::UseOfProperty461h2Aiii;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::CompliantServicesOrPropertyProvidedToTaxpayerUnder461h2A
        );
    }

    #[test]
    fn services_by_taxpayer_compliant_461h2b() {
        let mut i = base_services_to_taxpayer();
        i.liability_type = LiabilityType::ServicesOrPropertyProvidedByTaxpayer461h2B;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::CompliantServicesOrPropertyProvidedByTaxpayerUnder461h2B
        );
    }

    #[test]
    fn workers_comp_tort_breach_compliant_461h2c() {
        let mut i = base_services_to_taxpayer();
        i.liability_type = LiabilityType::WorkersCompTortBreachViolationOfLaw461h2C;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::CompliantPaymentLiabilitiesWorkersCompTortBreachUnder461h2C
        );
    }

    #[test]
    fn other_liability_compliant_461h2d() {
        let mut i = base_services_to_taxpayer();
        i.liability_type = LiabilityType::OtherLiability461h2D;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::CompliantOtherLiabilityRegulatoryDeterminationUnder461h2D
        );
    }

    #[test]
    fn liability_deducted_before_economic_performance_violation() {
        let mut i = base_services_to_taxpayer();
        i.economic_performance_occurred = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationLiabilityDeductedBeforeEconomicPerformance
        );
    }

    #[test]
    fn all_events_test_not_met_violation() {
        let mut i = base_services_to_taxpayer();
        i.all_events_test_satisfied = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ViolationAllEventsTestNotMet);
    }

    #[test]
    fn amount_not_determinable_violation() {
        let mut i = base_services_to_taxpayer();
        i.amount_determinable_with_reasonable_accuracy = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ViolationAllEventsTestNotMet);
    }

    #[test]
    fn reserves_for_estimated_expenses_violation() {
        let mut i = base_services_to_taxpayer();
        i.reserves_for_estimated_expenses_claimed = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationReservesForEstimatedExpensesUnder461h4
        );
    }

    #[test]
    fn recurring_item_exception_within_85_months_compliant() {
        let mut i = base_services_to_taxpayer();
        i.economic_performance_occurred = false;
        i.recurring_item_exception_claimed = true;
        i.months_to_economic_performance_after_taxable_year_x_10 = 85;
        i.item_is_recurring = true;
        i.consistent_treatment_of_similar_items = true;
        i.item_immaterial_or_matching_improved = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::CompliantRecurringItemExceptionUnder461h3
        );
    }

    #[test]
    fn recurring_item_exception_at_exactly_85_months_compliant() {
        let mut i = base_services_to_taxpayer();
        i.economic_performance_occurred = false;
        i.recurring_item_exception_claimed = true;
        i.months_to_economic_performance_after_taxable_year_x_10 = 85;
        i.item_is_recurring = true;
        i.consistent_treatment_of_similar_items = true;
        i.item_immaterial_or_matching_improved = true;
        let out = check(&i);
        assert!(out.deduction_allowable);
    }

    #[test]
    fn recurring_item_exception_at_86_months_violation() {
        let mut i = base_services_to_taxpayer();
        i.economic_performance_occurred = false;
        i.recurring_item_exception_claimed = true;
        i.months_to_economic_performance_after_taxable_year_x_10 = 86;
        i.item_is_recurring = true;
        i.consistent_treatment_of_similar_items = true;
        i.item_immaterial_or_matching_improved = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationRecurringItemMissedEightAndHalfMonthDeadline
        );
    }

    #[test]
    fn recurring_item_not_recurring_violation() {
        let mut i = base_services_to_taxpayer();
        i.economic_performance_occurred = false;
        i.recurring_item_exception_claimed = true;
        i.months_to_economic_performance_after_taxable_year_x_10 = 50;
        i.item_is_recurring = false;
        i.consistent_treatment_of_similar_items = true;
        i.item_immaterial_or_matching_improved = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationLiabilityDeductedBeforeEconomicPerformance
        );
    }

    #[test]
    fn recurring_item_inconsistent_treatment_violation() {
        let mut i = base_services_to_taxpayer();
        i.economic_performance_occurred = false;
        i.recurring_item_exception_claimed = true;
        i.months_to_economic_performance_after_taxable_year_x_10 = 50;
        i.item_is_recurring = true;
        i.consistent_treatment_of_similar_items = false;
        i.item_immaterial_or_matching_improved = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationLiabilityDeductedBeforeEconomicPerformance
        );
    }

    #[test]
    fn recurring_item_not_immaterial_or_better_matching_violation() {
        let mut i = base_services_to_taxpayer();
        i.economic_performance_occurred = false;
        i.recurring_item_exception_claimed = true;
        i.months_to_economic_performance_after_taxable_year_x_10 = 50;
        i.item_is_recurring = true;
        i.consistent_treatment_of_similar_items = true;
        i.item_immaterial_or_matching_improved = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationLiabilityDeductedBeforeEconomicPerformance
        );
    }

    #[test]
    fn not_applicable_returns_default() {
        let mut i = base_services_to_taxpayer();
        i.liability_type = LiabilityType::NotApplicable;
        let out = check(&i);
        assert_eq!(out.severity, Severity::NotApplicable);
    }

    #[test]
    fn citations_pin_461h_subsections() {
        let out = check(&base_services_to_taxpayer());
        assert!(out.citations.iter().any(|c| c.contains("§ 461(h)(1)")));
        assert!(out
            .citations
            .iter()
            .any(|c| c.contains("§ 461(h)(2)(A)(i)")));
        assert!(out
            .citations
            .iter()
            .any(|c| c.contains("§ 461(h)(2)(A)(ii)")));
        assert!(out
            .citations
            .iter()
            .any(|c| c.contains("§ 461(h)(2)(A)(iii)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 461(h)(2)(B)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 461(h)(2)(C)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 461(h)(2)(D)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 461(h)(3)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 461(h)(4)")));
    }

    #[test]
    fn citations_pin_446c2_treas_reg_dra_1984() {
        let out = check(&base_services_to_taxpayer());
        assert!(out.citations.iter().any(|c| c.contains("§ 446(c)(2)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 1.461-1(a)(2)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 1.461-4")));
        assert!(out.citations.iter().any(|c| c.contains("§ 1.461-5")));
        assert!(out
            .citations
            .iter()
            .any(|c| c.contains("Deficit Reduction Act of 1984")));
    }

    #[test]
    fn constant_pin_85_x_10_eight_and_half_months() {
        assert_eq!(RECURRING_ITEM_EXCEPTION_MONTHS_X_10, 85);
    }

    #[test]
    fn constant_pin_dra_1984_year() {
        assert_eq!(DRA_1984_ENACTMENT_YEAR, 1984);
    }
}
