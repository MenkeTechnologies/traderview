//! IRC § 269A personal service corporations formed or availed of to avoid or evade tax.
//!
//! § 269A gives Treasury authority to ALLOCATE income, deductions, credits, exclusions,
//! and other allowances between a personal service corporation (PSC) and its
//! employee-owner(s) when the corporation is formed or used to avoid or evade federal
//! income tax. The provision targets the classic "incorporate-and-zero-out" tactic where
//! an individual provides services to a single client, forms a wholly-owned PSC to
//! receive the income, then zeroes out the PSC's taxable income via fringe benefits,
//! deferred compensation, and qualified retirement contributions that the individual
//! could not claim if reporting the income directly on Schedule C.
//!
//! § 269A(a) STATUTORY TEST — both conditions must be satisfied:
//!   (1) SUBSTANTIALLY ALL of the PSC's services are performed for ONE other
//!       corporation, partnership, or entity, AND
//!   (2) The principal purpose of forming or availing of the PSC is to AVOID or EVADE
//!       federal income tax by securing the benefit of any expense, deduction, credit,
//!       exclusion, or other allowance the employee-owner could not otherwise claim.
//!
//! § 269A(b)(1) "PERSONAL SERVICE CORPORATION" DEFINITION: corporation whose principal
//! activity is the performance of personal services AND those services are substantially
//! performed by employee-owners.
//!
//! § 269A(b)(2) "EMPLOYEE-OWNER" DEFINITION: any employee who owns, on any day during
//! the taxable year, more than 10% of the outstanding stock of the PSC.
//!
//! § 269A(b)(3) RELATED-PERSONS aggregation: all related persons within the meaning of
//! § 144(a)(3) are treated as one entity for the "one entity" test under § 269A(a)(1).
//!
//! Effective for taxable years beginning after December 31, 1982.
//!
//! Citations (verified per WebSearch 2026-06-02):
//! - law.cornell.edu/uscode/text/26/269A
//! - irc.bloombergtax.com/public/uscode/doc/irc/section_269a
//! - taxnotes.com/research/federal/usc26/269A
//! - uscode.house.gov/view.xhtml?req=granuleid:USC-prelim-title26-section269A&num=0&edition=prelim

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CorporationActivityType {
    /// Principal activity is performance of personal services substantially performed
    /// by employee-owners. Meets § 269A(b)(1) PSC definition.
    PersonalServiceCorporationSection269AB1,
    /// Corporation engaged in non-personal-service business (manufacturing, sales,
    /// rental real estate) — § 269A inapplicable.
    NonPersonalServiceBusiness,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientConcentrationStatus {
    /// "Substantially all" services performed for ONE other entity (typically
    /// understood as 80%+ of revenue). § 269A(a)(1) condition satisfied.
    SubstantiallyAllServicesForOneEntity,
    /// Services performed for MULTIPLE unrelated clients — fails one-entity test.
    ServicesPerformedForMultipleUnrelatedClients,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrincipalPurposeStatus {
    /// Principal purpose was tax avoidance/evasion (securing employee-only benefit
    /// not otherwise available).
    PrincipalPurposeIsTaxAvoidanceOrEvasion,
    /// Principal purpose was bona-fide business (asset protection, liability shield,
    /// state-law professional-corporation requirement, partner buy-in mechanic).
    PrincipalPurposeIsBonaFideBusiness,
}

/// Percentage stock ownership of the employee-owner (any day during taxable year).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EmployeeOwnerThresholdStatus {
    /// Owns more than 10% of outstanding stock — § 269A(b)(2) employee-owner.
    EmployeeOwnsMoreThanTenPercent,
    /// Owns 10% or less — not an employee-owner.
    EmployeeOwnsTenPercentOrLess,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    NotPersonalServiceCorporationSection269AInapplicable,
    MultipleClientsFailsOneEntityTestNoAllocation,
    BonaFideBusinessPurposeNoAllocation,
    NoEmployeeOwnerThresholdSatisfiedNoAllocation,
    Section269AAllocationApplied,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub corporation_activity_type: CorporationActivityType,
    pub client_concentration_status: ClientConcentrationStatus,
    pub principal_purpose_status: PrincipalPurposeStatus,
    pub employee_owner_threshold_status: EmployeeOwnerThresholdStatus,
    pub psc_taxable_income_cents: u64,
    pub psc_claimed_benefit_amount_cents: u64,
}

pub type Section269APersonalServiceCorpAllocationInput = Input;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub severity: Severity,
    pub allocated_to_employee_owner_cents: u64,
    pub allocated_to_psc_cents: u64,
    pub note: String,
}

pub type Section269APersonalServiceCorpAllocationOutput = Output;
pub type Section269APersonalServiceCorpAllocationResult = Output;

const SECTION_269AB2_EMPLOYEE_OWNER_THRESHOLD_PERCENT: u32 = 10;
const SECTION_269A_EFFECTIVE_YEAR: u32 = 1983;

#[must_use]
pub fn check(input: &Input) -> Output {
    if matches!(
        input.corporation_activity_type,
        CorporationActivityType::NonPersonalServiceBusiness
    ) {
        return Output {
            severity: Severity::NotPersonalServiceCorporationSection269AInapplicable,
            allocated_to_employee_owner_cents: 0,
            allocated_to_psc_cents: input.psc_taxable_income_cents,
            note: format!(
                "§ 269A inapplicable: corporation is not a personal service corporation \
                 under § 269A(b)(1). PSC requires principal activity is performance of \
                 personal services AND services substantially performed by employee-owners. \
                 Non-PSC corporations (manufacturing, sales, rental real estate, investment \
                 holding) are NOT subject to § 269A even if same employee-owner / single- \
                 client facts are present. § 482 transfer-pricing reallocation may still \
                 apply between related entities; § 162 reasonable-compensation doctrine \
                 may also apply to outsized executive compensation. PSC taxable income (${}) \
                 remains with PSC.",
                input.psc_taxable_income_cents / 100
            ),
        };
    }

    if matches!(
        input.client_concentration_status,
        ClientConcentrationStatus::ServicesPerformedForMultipleUnrelatedClients
    ) {
        return Output {
            severity: Severity::MultipleClientsFailsOneEntityTestNoAllocation,
            allocated_to_employee_owner_cents: 0,
            allocated_to_psc_cents: input.psc_taxable_income_cents,
            note: format!(
                "§ 269A inapplicable: services performed for MULTIPLE unrelated clients, \
                 failing the § 269A(a)(1) 'substantially all services for one other entity' \
                 condition. § 269A(b)(3) related-persons aggregation under § 144(a)(3) treats \
                 related entities as one — verify all client relationships for related-party \
                 status (controlled group, common parent, family-attribution chain). PSC \
                 taxable income (${}) remains with PSC.",
                input.psc_taxable_income_cents / 100
            ),
        };
    }

    if matches!(
        input.principal_purpose_status,
        PrincipalPurposeStatus::PrincipalPurposeIsBonaFideBusiness
    ) {
        return Output {
            severity: Severity::BonaFideBusinessPurposeNoAllocation,
            allocated_to_employee_owner_cents: 0,
            allocated_to_psc_cents: input.psc_taxable_income_cents,
            note: format!(
                "§ 269A inapplicable: principal purpose of forming or availing of the PSC \
                 was bona-fide business (asset protection, professional-liability shield, \
                 state-law professional-corporation mandate, partner buy-in mechanic, \
                 succession planning, retirement-plan optimization that the law expressly \
                 permits) — NOT tax avoidance or evasion. § 269A(a)(2) requires the \
                 evasion-or-avoidance purpose EXCEEDS IN IMPORTANCE any other purpose. \
                 Document business rationale: incorporation memorandum, professional- \
                 liability-insurance carrier requirements, asset-protection planning \
                 documents, state-bar professional-corporation registration. PSC taxable \
                 income (${}) remains with PSC.",
                input.psc_taxable_income_cents / 100
            ),
        };
    }

    if matches!(
        input.employee_owner_threshold_status,
        EmployeeOwnerThresholdStatus::EmployeeOwnsTenPercentOrLess
    ) {
        return Output {
            severity: Severity::NoEmployeeOwnerThresholdSatisfiedNoAllocation,
            allocated_to_employee_owner_cents: 0,
            allocated_to_psc_cents: input.psc_taxable_income_cents,
            note: format!(
                "§ 269A inapplicable: no employee-owner satisfies the § 269A(b)(2) more-than- \
                 {SECTION_269AB2_EMPLOYEE_OWNER_THRESHOLD_PERCENT}% stock ownership threshold. \
                 An 'employee-owner' is any employee who owns, on any day during the taxable \
                 year, more than 10% of the outstanding stock of the PSC. If no qualifying \
                 employee-owner exists, the § 269A allocation mechanism has no target. PSC \
                 taxable income (${}) remains with PSC.",
                input.psc_taxable_income_cents / 100
            ),
        };
    }

    Output {
        severity: Severity::Section269AAllocationApplied,
        allocated_to_employee_owner_cents: input.psc_taxable_income_cents,
        allocated_to_psc_cents: 0,
        note: format!(
            "§ 269A allocation applied. All four conditions satisfied: (1) PSC under \
             § 269A(b)(1); (2) substantially all services for one other entity under \
             § 269A(a)(1); (3) principal purpose is tax avoidance / evasion under \
             § 269A(a)(2); (4) employee-owner satisfies § 269A(b)(2) more-than-10% threshold. \
             Treasury may ALLOCATE all income, deductions, credits, exclusions, and other \
             allowances between PSC and employee-owner — typical effect is to recharacterize \
             PSC income (${}) as direct income to the employee-owner, eliminating the \
             benefit of the PSC structure for retirement contributions, fringe benefits, \
             and deferred compensation that exceeded what the individual could claim \
             directly. Claimed PSC benefit (${}) potentially recaptured. Section effective \
             for taxable years beginning after Dec 31, {SECTION_269A_EFFECTIVE_YEAR}. \
             Coordinates with § 269 (iter 536 — broader anti-tax-avoidance acquisition \
             disallowance), § 482 (related-party transfer pricing), § 162 (reasonable \
             compensation reallocation), § 444 (PSC fiscal-year election restrictions), \
             § 280H (PSC accumulated earnings + minimum distribution rules), § 199A \
             (Specified Service Trade or Business limitation parallel).",
            input.psc_taxable_income_cents / 100,
            input.psc_claimed_benefit_amount_cents / 100
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Input {
        Input {
            corporation_activity_type:
                CorporationActivityType::PersonalServiceCorporationSection269AB1,
            client_concentration_status:
                ClientConcentrationStatus::SubstantiallyAllServicesForOneEntity,
            principal_purpose_status:
                PrincipalPurposeStatus::PrincipalPurposeIsTaxAvoidanceOrEvasion,
            employee_owner_threshold_status:
                EmployeeOwnerThresholdStatus::EmployeeOwnsMoreThanTenPercent,
            psc_taxable_income_cents: 500_000_00,
            psc_claimed_benefit_amount_cents: 100_000_00,
        }
    }

    #[test]
    fn non_psc_corporation_section_269a_inapplicable() {
        let mut input = base();
        input.corporation_activity_type = CorporationActivityType::NonPersonalServiceBusiness;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::NotPersonalServiceCorporationSection269AInapplicable
        );
        assert_eq!(output.allocated_to_psc_cents, 500_000_00);
        assert!(output.note.contains("§ 482"));
        assert!(output.note.contains("§ 162"));
    }

    #[test]
    fn multiple_clients_fails_one_entity_test_no_allocation() {
        let mut input = base();
        input.client_concentration_status =
            ClientConcentrationStatus::ServicesPerformedForMultipleUnrelatedClients;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::MultipleClientsFailsOneEntityTestNoAllocation
        );
        assert_eq!(output.allocated_to_psc_cents, 500_000_00);
        assert!(output.note.contains("§ 269A(a)(1)"));
        assert!(output.note.contains("§ 144(a)(3)"));
    }

    #[test]
    fn bona_fide_business_purpose_no_allocation() {
        let mut input = base();
        input.principal_purpose_status = PrincipalPurposeStatus::PrincipalPurposeIsBonaFideBusiness;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::BonaFideBusinessPurposeNoAllocation
        );
        assert_eq!(output.allocated_to_psc_cents, 500_000_00);
        assert!(output.note.contains("EXCEEDS IN IMPORTANCE"));
        assert!(output.note.contains("professional-liability"));
    }

    #[test]
    fn no_employee_owner_threshold_no_allocation() {
        let mut input = base();
        input.employee_owner_threshold_status =
            EmployeeOwnerThresholdStatus::EmployeeOwnsTenPercentOrLess;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::NoEmployeeOwnerThresholdSatisfiedNoAllocation
        );
        assert_eq!(output.allocated_to_psc_cents, 500_000_00);
        assert!(output.note.contains("§ 269A(b)(2)"));
        assert!(output.note.contains("more-than-"));
        assert!(output.note.contains("10%"));
    }

    #[test]
    fn all_four_conditions_satisfied_section_269a_allocation_applied() {
        let input = base();
        let output = check(&input);
        assert_eq!(output.severity, Severity::Section269AAllocationApplied);
        assert_eq!(output.allocated_to_employee_owner_cents, 500_000_00);
        assert_eq!(output.allocated_to_psc_cents, 0);
        assert!(output.note.contains("§ 269A(a)(1)"));
        assert!(output.note.contains("§ 269A(a)(2)"));
        assert!(output.note.contains("§ 269A(b)(1)"));
        assert!(output.note.contains("§ 269A(b)(2)"));
    }

    #[test]
    fn employee_owner_threshold_constant_pins_10_percent() {
        assert_eq!(SECTION_269AB2_EMPLOYEE_OWNER_THRESHOLD_PERCENT, 10);
    }

    #[test]
    fn section_269a_effective_year_constant_pins_1983() {
        assert_eq!(SECTION_269A_EFFECTIVE_YEAR, 1983);
    }

    #[test]
    fn note_pins_section_269_companion() {
        let input = base();
        let output = check(&input);
        assert!(output.note.contains("§ 269"));
    }

    #[test]
    fn note_pins_section_482_related_party_transfer_pricing() {
        let mut input = base();
        input.corporation_activity_type = CorporationActivityType::NonPersonalServiceBusiness;
        let output = check(&input);
        assert!(output.note.contains("§ 482"));
    }

    #[test]
    fn note_pins_section_280h_psc_accumulated_earnings() {
        let input = base();
        let output = check(&input);
        assert!(output.note.contains("§ 280H"));
    }

    #[test]
    fn note_pins_section_199a_specified_service_trade_or_business() {
        let input = base();
        let output = check(&input);
        assert!(output.note.contains("§ 199A"));
    }

    #[test]
    fn very_large_psc_income_no_overflow() {
        let mut input = base();
        input.psc_taxable_income_cents = u64::MAX;
        let output = check(&input);
        assert_eq!(output.allocated_to_employee_owner_cents, u64::MAX);
    }

    #[test]
    fn zero_income_zero_allocation_no_panic() {
        let mut input = base();
        input.psc_taxable_income_cents = 0;
        let output = check(&input);
        assert_eq!(output.allocated_to_employee_owner_cents, 0);
        assert_eq!(output.allocated_to_psc_cents, 0);
    }

    #[test]
    fn non_psc_takes_priority_over_all_other_conditions() {
        let mut input = base();
        input.corporation_activity_type = CorporationActivityType::NonPersonalServiceBusiness;
        // All other conditions satisfied, but non-PSC takes priority
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::NotPersonalServiceCorporationSection269AInapplicable
        );
    }

    #[test]
    fn multiple_clients_takes_priority_over_purpose_and_owner() {
        let mut input = base();
        input.client_concentration_status =
            ClientConcentrationStatus::ServicesPerformedForMultipleUnrelatedClients;
        let output = check(&input);
        // Multiple-clients failure dispositive
        assert_eq!(
            output.severity,
            Severity::MultipleClientsFailsOneEntityTestNoAllocation
        );
    }

    #[test]
    fn bona_fide_business_purpose_takes_priority_over_employee_owner_threshold() {
        let mut input = base();
        input.principal_purpose_status = PrincipalPurposeStatus::PrincipalPurposeIsBonaFideBusiness;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::BonaFideBusinessPurposeNoAllocation
        );
    }

    #[test]
    fn note_describes_classic_zero_out_tactic() {
        let input = base();
        let output = check(&input);
        assert!(output.note.contains("recharacterize"));
        assert!(output.note.contains("benefit"));
    }
}
