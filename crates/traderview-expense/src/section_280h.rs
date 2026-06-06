//! IRC § 280H limitation on PSC employee-owner compensation deduction.
//!
//! § 280H disallows a portion of a personal service corporation's (PSC) employee-owner
//! compensation deduction when (1) the PSC has a § 444 fiscal-year election in effect
//! AND (2) the PSC fails the minimum distribution requirement of § 280H(c). The
//! provision closes the income-deferral loophole that a non-calendar fiscal-year PSC
//! would otherwise enjoy: without § 280H, a fiscal-year PSC could end its fiscal year
//! before paying employee-owners, deferring the inclusion to the employee-owners' next
//! calendar year while still claiming the deduction.
//!
//! § 280H(a) GENERAL RULE: deduction otherwise allowed for "applicable amounts" paid
//! or incurred to employee-owners is limited to the "maximum deductible amount" when
//! the PSC has a § 444 election AND fails to meet the minimum distribution requirement.
//!
//! § 280H(b) "APPLICABLE AMOUNT" DEFINITION: any amount otherwise deductible by the PSC
//! in the taxable year AND includable at any time, directly or indirectly, in the gross
//! income of a taxpayer who is an employee-owner.
//!
//! § 280H(c) MINIMUM DISTRIBUTION REQUIREMENT: PSC meets the requirement if applicable
//! amounts paid during the deferral period of the taxable year equal or exceed the
//! LESSER OF:
//!   (A) Applicable amounts paid during preceding taxable year × (months in deferral
//!       period / months in preceding taxable year), or
//!   (B) Applicable percentage of adjusted taxable income for the deferral period of
//!       the taxable year.
//!
//! § 280H(d) "APPLICABLE PERCENTAGE": percentage (capped at 95%) determined by dividing
//! applicable amounts paid during the 3 preceding taxable years by adjusted taxable
//! income for those 3 years.
//!
//! § 280H(f) CARRYOVER: nondeductible amounts are treated as paid or incurred in the
//! succeeding taxable year. § 280H(g) NOL CARRYBACK BAR: no NOL carryback allowed to
//! or from any year of a PSC that has a § 444 election in effect.
//!
//! Citations (verified per WebSearch 2026-06-02):
//! - law.cornell.edu/uscode/text/26/280H
//! - law.cornell.edu/cfr/text/26/1.280H-1T
//! - irc.bloombergtax.com/public/uscode/doc/irc/section_280h
//! - ecfr.gov/current/title-26/chapter-I/subchapter-A/part-1/subject-group-ECFR210006225231fb0/section-1.280H-1T

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Section444ElectionStatus {
    /// PSC has § 444 fiscal-year election in effect; § 280H applies.
    Section444FiscalYearElectionInEffect,
    /// PSC uses calendar year (required for PSCs without § 444 election); § 280H
    /// inapplicable.
    CalendarYearNoSection280HApplies,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CorporationStatus {
    /// Corporation is a personal service corporation under § 280H(d) +
    /// § 269A(b)(1) standard.
    IsPersonalServiceCorporation,
    /// Not a PSC; § 280H inapplicable.
    NotAPersonalServiceCorporation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub corporation_status: CorporationStatus,
    pub section_444_election_status: Section444ElectionStatus,
    pub applicable_amounts_paid_in_current_deferral_period_cents: u64,
    pub applicable_amounts_paid_in_preceding_year_cents: u64,
    pub months_in_preceding_taxable_year: u32,
    pub months_in_current_deferral_period: u32,
    pub adjusted_taxable_income_deferral_period_cents: u64,
    pub applicable_percentage_basis_points: u32,
}

pub type Section280HPscMinimumDistributionInput = Input;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    NotPersonalServiceCorporationSection280HInapplicable,
    CalendarYearNoSection280HApplied,
    MinimumDistributionRequirementSatisfiedNoDisallowance,
    Section280HDisallowanceAppliedCarryoverToNextYear,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub severity: Severity,
    pub minimum_distribution_threshold_cents: u64,
    pub disallowed_employee_owner_comp_cents: u64,
    pub allowed_employee_owner_comp_cents: u64,
    pub note: String,
}

pub type Section280HPscMinimumDistributionOutput = Output;
pub type Section280HPscMinimumDistributionResult = Output;

const SECTION_280H_APPLICABLE_PERCENTAGE_CAP_BPS: u32 = 9_500;

#[must_use]
pub fn check(input: &Input) -> Output {
    if matches!(
        input.corporation_status,
        CorporationStatus::NotAPersonalServiceCorporation
    ) {
        return Output {
            severity: Severity::NotPersonalServiceCorporationSection280HInapplicable,
            minimum_distribution_threshold_cents: 0,
            disallowed_employee_owner_comp_cents: 0,
            allowed_employee_owner_comp_cents: input
                .applicable_amounts_paid_in_current_deferral_period_cents,
            note: format!(
                "§ 280H inapplicable: corporation is not a personal service corporation. \
                 § 280H applies only to PSCs as defined under § 269A(b)(1) + Treas. Reg. \
                 § 1.441-3(c) testing-period qualified-personal-services-corporation \
                 standard. Employee-owner compensation (${}) deductible subject to § 162 \
                 reasonable-compensation analysis + § 482 related-party transfer pricing.",
                input.applicable_amounts_paid_in_current_deferral_period_cents / 100
            ),
        };
    }

    if matches!(
        input.section_444_election_status,
        Section444ElectionStatus::CalendarYearNoSection280HApplies
    ) {
        return Output {
            severity: Severity::CalendarYearNoSection280HApplied,
            minimum_distribution_threshold_cents: 0,
            disallowed_employee_owner_comp_cents: 0,
            allowed_employee_owner_comp_cents: input
                .applicable_amounts_paid_in_current_deferral_period_cents,
            note: format!(
                "§ 280H inapplicable: PSC uses calendar year (no § 444 election). § 280H \
                 minimum distribution requirement applies ONLY when § 444 fiscal-year election \
                 is in effect. Employee-owner compensation (${}) deductible without § 280H \
                 limitation. § 280H(g) NOL carryback bar also inapplicable.",
                input.applicable_amounts_paid_in_current_deferral_period_cents / 100
            ),
        };
    }

    let prior_year_proration = if input.months_in_preceding_taxable_year == 0 {
        0u64
    } else {
        u64::try_from(
            u128::from(input.applicable_amounts_paid_in_preceding_year_cents)
                .saturating_mul(u128::from(input.months_in_current_deferral_period))
                .saturating_div(u128::from(input.months_in_preceding_taxable_year)),
        )
        .unwrap_or(u64::MAX)
    };

    let applicable_percentage_capped = input
        .applicable_percentage_basis_points
        .min(SECTION_280H_APPLICABLE_PERCENTAGE_CAP_BPS);
    let percentage_of_ati = u64::try_from(
        u128::from(input.adjusted_taxable_income_deferral_period_cents)
            .saturating_mul(u128::from(applicable_percentage_capped))
            .saturating_div(10_000),
    )
    .unwrap_or(u64::MAX);

    let minimum_distribution_threshold = prior_year_proration.min(percentage_of_ati);

    if input.applicable_amounts_paid_in_current_deferral_period_cents
        >= minimum_distribution_threshold
    {
        return Output {
            severity: Severity::MinimumDistributionRequirementSatisfiedNoDisallowance,
            minimum_distribution_threshold_cents: minimum_distribution_threshold,
            disallowed_employee_owner_comp_cents: 0,
            allowed_employee_owner_comp_cents: input
                .applicable_amounts_paid_in_current_deferral_period_cents,
            note: format!(
                "§ 280H(c) minimum distribution requirement SATISFIED. Applicable amounts paid \
                 during deferral period (${}) equal or exceed the LESSER of (A) prior year \
                 proration (${}) or (B) applicable percentage × adjusted taxable income (${}). \
                 Threshold ${}. PSC fiscal-year deferral preserved; full employee-owner \
                 compensation deductible.",
                input.applicable_amounts_paid_in_current_deferral_period_cents / 100,
                prior_year_proration / 100,
                percentage_of_ati / 100,
                minimum_distribution_threshold / 100
            ),
        };
    }

    let disallowed = minimum_distribution_threshold
        .saturating_sub(input.applicable_amounts_paid_in_current_deferral_period_cents);
    let allowed = input.applicable_amounts_paid_in_current_deferral_period_cents;

    Output {
        severity: Severity::Section280HDisallowanceAppliedCarryoverToNextYear,
        minimum_distribution_threshold_cents: minimum_distribution_threshold,
        disallowed_employee_owner_comp_cents: disallowed,
        allowed_employee_owner_comp_cents: allowed,
        note: format!(
            "§ 280H(a) disallowance applied. Applicable amounts paid in deferral period \
             (${}) fall below minimum distribution threshold (${}). Shortfall ${} disallowed \
             current year + carried to next year per § 280H(f). § 280H(g) NOL carryback \
             also disallowed for any year of PSC with § 444 election in effect. Coordinates \
             with § 444 PSC fiscal-year election + § 269A (iter 544 — PSC tax-avoidance \
             allocation) + § 269 (iter 536 — discretionary anti-tax-avoidance disallowance) \
             + § 162 reasonable compensation + Form 8716 § 444 election + Schedule H \
             (Form 1120) § 280H computation.",
            input.applicable_amounts_paid_in_current_deferral_period_cents / 100,
            minimum_distribution_threshold / 100,
            disallowed / 100
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Input {
        Input {
            corporation_status: CorporationStatus::IsPersonalServiceCorporation,
            section_444_election_status:
                Section444ElectionStatus::Section444FiscalYearElectionInEffect,
            applicable_amounts_paid_in_current_deferral_period_cents: 500_000_00,
            applicable_amounts_paid_in_preceding_year_cents: 1_200_000_00,
            months_in_preceding_taxable_year: 12,
            months_in_current_deferral_period: 3,
            adjusted_taxable_income_deferral_period_cents: 600_000_00,
            applicable_percentage_basis_points: 8_000,
        }
    }

    #[test]
    fn not_psc_section_280h_inapplicable() {
        let mut input = base();
        input.corporation_status = CorporationStatus::NotAPersonalServiceCorporation;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::NotPersonalServiceCorporationSection280HInapplicable
        );
        assert_eq!(output.allowed_employee_owner_comp_cents, 500_000_00);
        assert!(output.note.contains("§ 269A(b)(1)"));
        assert!(output.note.contains("§ 162"));
    }

    #[test]
    fn calendar_year_no_section_280h_applied() {
        let mut input = base();
        input.section_444_election_status =
            Section444ElectionStatus::CalendarYearNoSection280HApplies;
        let output = check(&input);
        assert_eq!(output.severity, Severity::CalendarYearNoSection280HApplied);
        assert_eq!(output.allowed_employee_owner_comp_cents, 500_000_00);
        assert!(output.note.contains("§ 280H(g)"));
    }

    #[test]
    fn minimum_distribution_satisfied_no_disallowance() {
        let input = base();
        // Prior year proration: $1.2M × 3/12 = $300K
        // ATI × applicable %: $600K × 80% = $480K
        // Threshold = lesser = $300K
        // Paid $500K >= $300K → satisfied
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::MinimumDistributionRequirementSatisfiedNoDisallowance
        );
        assert_eq!(output.minimum_distribution_threshold_cents, 300_000_00);
        assert_eq!(output.disallowed_employee_owner_comp_cents, 0);
    }

    #[test]
    fn shortfall_triggers_section_280h_disallowance() {
        let mut input = base();
        input.applicable_amounts_paid_in_current_deferral_period_cents = 200_000_00;
        let output = check(&input);
        // Threshold $300K, paid $200K → $100K shortfall
        assert_eq!(
            output.severity,
            Severity::Section280HDisallowanceAppliedCarryoverToNextYear
        );
        assert_eq!(output.disallowed_employee_owner_comp_cents, 100_000_00);
        assert_eq!(output.allowed_employee_owner_comp_cents, 200_000_00);
        assert!(output.note.contains("§ 280H(a)"));
        assert!(output.note.contains("§ 280H(f)"));
        assert!(output.note.contains("§ 280H(g)"));
    }

    #[test]
    fn applicable_percentage_capped_at_95_pct() {
        let mut input = base();
        input.applicable_percentage_basis_points = 12_000; // 120%
        input.adjusted_taxable_income_deferral_period_cents = 600_000_00;
        // Should be capped at 95%; ATI × 95% = $570K
        // Prior year proration: $1.2M × 3/12 = $300K
        // Lesser = $300K
        let output = check(&input);
        // Threshold = min($300K, $570K) = $300K
        assert_eq!(output.minimum_distribution_threshold_cents, 300_000_00);
    }

    #[test]
    fn prior_year_proration_branch_when_smaller_than_ati_percentage() {
        let mut input = base();
        input.applicable_amounts_paid_in_preceding_year_cents = 400_000_00;
        // Prior year proration: $400K × 3/12 = $100K
        // ATI × 80% = $480K
        // Threshold = lesser = $100K
        // Paid $500K >= $100K → satisfied
        let output = check(&input);
        assert_eq!(output.minimum_distribution_threshold_cents, 100_000_00);
        assert_eq!(
            output.severity,
            Severity::MinimumDistributionRequirementSatisfiedNoDisallowance
        );
    }

    #[test]
    fn ati_percentage_branch_when_smaller_than_prior_year_proration() {
        let mut input = base();
        input.applicable_amounts_paid_in_preceding_year_cents = 12_000_000_00;
        // Prior year proration: $12M × 3/12 = $3M
        // ATI × 80% = $480K
        // Threshold = lesser = $480K
        let output = check(&input);
        assert_eq!(output.minimum_distribution_threshold_cents, 480_000_00);
        assert_eq!(
            output.severity,
            Severity::MinimumDistributionRequirementSatisfiedNoDisallowance
        );
    }

    #[test]
    fn zero_preceding_months_no_division_panic() {
        let mut input = base();
        input.months_in_preceding_taxable_year = 0;
        let output = check(&input);
        // Prior year proration = 0; threshold = lesser of 0 or ATI × % = 0
        assert_eq!(output.minimum_distribution_threshold_cents, 0);
    }

    #[test]
    fn applicable_percentage_cap_constant_pins_95_pct() {
        assert_eq!(SECTION_280H_APPLICABLE_PERCENTAGE_CAP_BPS, 9_500);
    }

    #[test]
    fn very_large_amounts_no_overflow() {
        let mut input = base();
        input.applicable_amounts_paid_in_preceding_year_cents = u64::MAX;
        input.adjusted_taxable_income_deferral_period_cents = u64::MAX;
        let output = check(&input);
        // u128 intermediate prevents overflow
        assert!(output.minimum_distribution_threshold_cents > 0);
    }

    #[test]
    fn zero_amounts_no_panic() {
        let mut input = base();
        input.applicable_amounts_paid_in_current_deferral_period_cents = 0;
        input.applicable_amounts_paid_in_preceding_year_cents = 0;
        input.adjusted_taxable_income_deferral_period_cents = 0;
        let output = check(&input);
        // Threshold = 0; 0 >= 0 → satisfied
        assert_eq!(
            output.severity,
            Severity::MinimumDistributionRequirementSatisfiedNoDisallowance
        );
    }

    #[test]
    fn note_pins_section_444_election_companion() {
        let mut input = base();
        input.applicable_amounts_paid_in_current_deferral_period_cents = 100_000_00;
        let output = check(&input);
        assert!(output.note.contains("§ 444"));
    }

    #[test]
    fn note_pins_section_269a_companion() {
        let mut input = base();
        input.applicable_amounts_paid_in_current_deferral_period_cents = 100_000_00;
        let output = check(&input);
        assert!(output.note.contains("§ 269A"));
    }

    #[test]
    fn note_pins_form_8716_section_444_election() {
        let mut input = base();
        input.applicable_amounts_paid_in_current_deferral_period_cents = 100_000_00;
        let output = check(&input);
        assert!(output.note.contains("Form 8716"));
    }

    #[test]
    fn note_pins_schedule_h_form_1120() {
        let mut input = base();
        input.applicable_amounts_paid_in_current_deferral_period_cents = 100_000_00;
        let output = check(&input);
        assert!(output.note.contains("Schedule H"));
        assert!(output.note.contains("Form 1120"));
    }

    #[test]
    fn calendar_year_takes_priority_over_psc_status() {
        let mut input = base();
        input.section_444_election_status =
            Section444ElectionStatus::CalendarYearNoSection280HApplies;
        let output = check(&input);
        assert_eq!(output.severity, Severity::CalendarYearNoSection280HApplied);
    }

    #[test]
    fn not_psc_takes_priority_over_444_election() {
        let mut input = base();
        input.corporation_status = CorporationStatus::NotAPersonalServiceCorporation;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::NotPersonalServiceCorporationSection280HInapplicable
        );
    }
}
