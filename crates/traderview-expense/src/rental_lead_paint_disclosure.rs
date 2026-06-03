//! Federal Title X § 1018 Residential Lead-Based Paint Hazard Reduction
//! Act of 1992 pre-lease lead-based paint disclosure module
//! (42 U.S.C. § 4852d; 24 C.F.R. Part 35 Subpart A; 40 C.F.R. Part 745
//! Subpart F).
//!
//! Universal federal landlord obligation: BEFORE any lease for "target
//! housing" (residential housing constructed before 1978-01-01) becomes
//! binding, the landlord, real-estate agent, and property manager are
//! jointly and severally obligated to satisfy FIVE concrete disclosure
//! elements:
//!
//! 1. Insert the federal "Lead Warning Statement" verbatim into the
//!    lease.
//! 2. Disclose ANY known lead-based paint (LBP) and/or LBP-hazards in
//!    the target housing unit, OR affirmatively state "no knowledge."
//! 3. Provide all available LBP records and reports in the landlord's
//!    possession.
//! 4. Provide the EPA "Protect Your Family From Lead in Your Home"
//!    pamphlet (EPA-747-K-12-001).
//! 5. Obtain a signed acknowledgment from the tenant.
//!
//! Civil penalty under 42 U.S.C. § 4852d(b)(5): UP TO $22,263 per
//! violation (per-tenant, per-property) as of the 2025 inflation
//! adjustment, AND jointly-and-severally liable for treble actual
//! damages to the lessee under 42 U.S.C. § 4852d(b)(3).
//!
//! Exemptions (per 24 C.F.R. § 35.82(b)):
//!
//! - Zero-bedroom dwellings (efficiencies, lofts, dormitories) UNLESS a
//!   child under 6 is expected to reside.
//! - Short-term leases of 100 days or less where no renewal/extension
//!   can occur.
//! - Housing specifically designated for the elderly or persons with
//!   disabilities UNLESS a child under 6 is expected to reside.
//! - Post-1977 housing (built on or after 1978-01-01) — not "target
//!   housing" by definition.
//! - Foreclosure sales.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[allow(dead_code)]
pub const PRE_1978_TARGET_HOUSING_THRESHOLD_YEAR: u32 = 1978;
#[allow(dead_code)]
pub const SHORT_TERM_LEASE_EXEMPTION_DAYS: u32 = 100;
#[allow(dead_code)]
pub const CHILD_AGE_TRIGGER_BELOW_YEARS: u32 = 6;
#[allow(dead_code)]
pub const TITLE_X_SECTION_1018_CIVIL_PENALTY_MAX_CENTS_2025: u64 = 2_226_300;
#[allow(dead_code)]
pub const TREBLE_DAMAGES_MULTIPLIER: u64 = 3;
#[allow(dead_code)]
pub const TOTAL_DISCLOSURE_ELEMENTS_REQUIRED: u32 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    DisclosureCompleteAllFiveElementsLandlordCompliant,
    DisclosureIncompleteMissingEpaPamphletViolation,
    DisclosureIncompleteMissingKnownLbpDisclosureViolation,
    DisclosureIncompleteMissingLeadWarningStatementInLeaseViolation,
    DisclosureIncompleteMissingSignedAcknowledgmentViolation,
    DisclosureIncompleteMissingAvailableReportsViolation,
    DisclosureIncompleteMissingMultipleElementsAggravatedViolation,
    ExemptZeroBedroomNoChildExpected,
    ExemptShortTermLeaseUnder100Days,
    ExemptElderlyOrDisabledHousingNoChildExpected,
    ExemptPost1977Housing,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub year_unit_built: u32,
    pub bedrooms: u32,
    pub elderly_only_or_disabled_only_housing: bool,
    pub child_under_six_expected: bool,
    pub lease_term_days: u32,
    pub lease_renewal_or_extension_possible: bool,
    pub lead_warning_statement_in_lease: bool,
    pub known_lbp_disclosed_or_no_knowledge_stated: bool,
    pub available_reports_provided: bool,
    pub epa_pamphlet_provided: bool,
    pub tenant_signed_acknowledgment: bool,
    pub actual_damages_to_tenant_cents: u64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Output {
    pub severity: Severity,
    pub disclosure_elements_satisfied: u32,
    pub maximum_civil_penalty_cents: u64,
    pub treble_damages_exposure_cents: u64,
    pub total_landlord_exposure_cents: u64,
    pub notes: Vec<String>,
    pub citations: Vec<String>,
}

pub type LeadPaintDisclosureInput = Input;
pub type LeadPaintDisclosureOutput = Output;
pub type LeadPaintDisclosureResult = Output;

pub fn check(input: &Input) -> Output {
    let mut notes: Vec<String> = Vec::new();
    let citations: Vec<String> = vec![
        "42 U.S.C. § 4852d".to_string(),
        "Title X § 1018 of the Residential Lead-Based Paint Hazard Reduction Act of 1992 (Pub. L. 102-550)".to_string(),
        "24 C.F.R. Part 35 Subpart A (HUD Lead Disclosure Rule)".to_string(),
        "40 C.F.R. Part 745 Subpart F (EPA Lead Disclosure Rule)".to_string(),
        "EPA-747-K-12-001 — Protect Your Family From Lead in Your Home pamphlet".to_string(),
        "24 C.F.R. § 35.82(b) (exemptions)".to_string(),
        "42 U.S.C. § 4852d(b)(3) (treble damages)".to_string(),
        "42 U.S.C. § 4852d(b)(5) (civil penalty up to $22,263 per violation, 2025 inflation-adjusted)".to_string(),
    ];

    if input.year_unit_built >= PRE_1978_TARGET_HOUSING_THRESHOLD_YEAR {
        notes.push(format!(
            "Built {} — not pre-1978 target housing; Title X § 1018 disclosure rule does not apply.",
            input.year_unit_built
        ));
        return Output {
            severity: Severity::ExemptPost1977Housing,
            disclosure_elements_satisfied: 0,
            maximum_civil_penalty_cents: 0,
            treble_damages_exposure_cents: 0,
            total_landlord_exposure_cents: 0,
            notes,
            citations,
        };
    }

    if input.lease_term_days <= SHORT_TERM_LEASE_EXEMPTION_DAYS
        && !input.lease_renewal_or_extension_possible
    {
        notes.push(format!(
            "Short-term lease {} days with no renewal/extension — exempt per 24 C.F.R. § 35.82(b)(3).",
            input.lease_term_days
        ));
        return Output {
            severity: Severity::ExemptShortTermLeaseUnder100Days,
            disclosure_elements_satisfied: 0,
            maximum_civil_penalty_cents: 0,
            treble_damages_exposure_cents: 0,
            total_landlord_exposure_cents: 0,
            notes,
            citations,
        };
    }

    if input.bedrooms == 0 && !input.child_under_six_expected {
        notes.push("Zero-bedroom dwelling with no child under 6 expected — exempt per 24 C.F.R. § 35.82(b)(1).".to_string());
        return Output {
            severity: Severity::ExemptZeroBedroomNoChildExpected,
            disclosure_elements_satisfied: 0,
            maximum_civil_penalty_cents: 0,
            treble_damages_exposure_cents: 0,
            total_landlord_exposure_cents: 0,
            notes,
            citations,
        };
    }

    if input.elderly_only_or_disabled_only_housing && !input.child_under_six_expected {
        notes.push("Housing designated for elderly or persons with disabilities; no child under 6 expected — exempt per 24 C.F.R. § 35.82(b)(2).".to_string());
        return Output {
            severity: Severity::ExemptElderlyOrDisabledHousingNoChildExpected,
            disclosure_elements_satisfied: 0,
            maximum_civil_penalty_cents: 0,
            treble_damages_exposure_cents: 0,
            total_landlord_exposure_cents: 0,
            notes,
            citations,
        };
    }

    let satisfied: u32 = [
        input.lead_warning_statement_in_lease,
        input.known_lbp_disclosed_or_no_knowledge_stated,
        input.available_reports_provided,
        input.epa_pamphlet_provided,
        input.tenant_signed_acknowledgment,
    ]
    .iter()
    .filter(|x| **x)
    .count() as u32;

    let missing = TOTAL_DISCLOSURE_ELEMENTS_REQUIRED - satisfied;

    if satisfied == TOTAL_DISCLOSURE_ELEMENTS_REQUIRED {
        notes.push("All five Title X § 1018 disclosure elements satisfied: Lead Warning Statement in lease, known LBP disclosed (or no-knowledge stated), available reports provided, EPA pamphlet delivered, tenant signed acknowledgment. Landlord compliant.".to_string());
        return Output {
            severity: Severity::DisclosureCompleteAllFiveElementsLandlordCompliant,
            disclosure_elements_satisfied: satisfied,
            maximum_civil_penalty_cents: 0,
            treble_damages_exposure_cents: 0,
            total_landlord_exposure_cents: 0,
            notes,
            citations,
        };
    }

    let treble_exposure = input
        .actual_damages_to_tenant_cents
        .saturating_mul(TREBLE_DAMAGES_MULTIPLIER);
    let total_exposure = TITLE_X_SECTION_1018_CIVIL_PENALTY_MAX_CENTS_2025
        .saturating_add(treble_exposure);

    if missing >= 2 {
        notes.push(format!(
            "Aggravated violation: {} of {} disclosure elements missing. Civil penalty up to ${} + treble damages.",
            missing,
            TOTAL_DISCLOSURE_ELEMENTS_REQUIRED,
            TITLE_X_SECTION_1018_CIVIL_PENALTY_MAX_CENTS_2025 / 100
        ));
        return Output {
            severity: Severity::DisclosureIncompleteMissingMultipleElementsAggravatedViolation,
            disclosure_elements_satisfied: satisfied,
            maximum_civil_penalty_cents: TITLE_X_SECTION_1018_CIVIL_PENALTY_MAX_CENTS_2025,
            treble_damages_exposure_cents: treble_exposure,
            total_landlord_exposure_cents: total_exposure,
            notes,
            citations,
        };
    }

    let severity = if !input.lead_warning_statement_in_lease {
        notes.push("Missing federal Lead Warning Statement verbatim in lease — per se Title X § 1018 violation.".to_string());
        Severity::DisclosureIncompleteMissingLeadWarningStatementInLeaseViolation
    } else if !input.known_lbp_disclosed_or_no_knowledge_stated {
        notes.push("Failed to disclose known LBP/LBP-hazards or affirmatively state no knowledge — per se Title X § 1018 violation.".to_string());
        Severity::DisclosureIncompleteMissingKnownLbpDisclosureViolation
    } else if !input.available_reports_provided {
        notes.push("Failed to provide available LBP records/reports — per se Title X § 1018 violation.".to_string());
        Severity::DisclosureIncompleteMissingAvailableReportsViolation
    } else if !input.epa_pamphlet_provided {
        notes.push("Failed to provide EPA Protect Your Family From Lead in Your Home pamphlet (EPA-747-K-12-001) — per se Title X § 1018 violation.".to_string());
        Severity::DisclosureIncompleteMissingEpaPamphletViolation
    } else {
        notes.push("Missing tenant signed acknowledgment of disclosure — per se Title X § 1018 violation.".to_string());
        Severity::DisclosureIncompleteMissingSignedAcknowledgmentViolation
    };

    Output {
        severity,
        disclosure_elements_satisfied: satisfied,
        maximum_civil_penalty_cents: TITLE_X_SECTION_1018_CIVIL_PENALTY_MAX_CENTS_2025,
        treble_damages_exposure_cents: treble_exposure,
        total_landlord_exposure_cents: total_exposure,
        notes,
        citations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_pre_1978_compliant() -> Input {
        Input {
            year_unit_built: 1955,
            bedrooms: 2,
            elderly_only_or_disabled_only_housing: false,
            child_under_six_expected: true,
            lease_term_days: 365,
            lease_renewal_or_extension_possible: true,
            lead_warning_statement_in_lease: true,
            known_lbp_disclosed_or_no_knowledge_stated: true,
            available_reports_provided: true,
            epa_pamphlet_provided: true,
            tenant_signed_acknowledgment: true,
            actual_damages_to_tenant_cents: 0,
        }
    }

    #[test]
    fn all_five_disclosure_elements_satisfied_is_compliant() {
        let out = check(&base_pre_1978_compliant());
        assert_eq!(
            out.severity,
            Severity::DisclosureCompleteAllFiveElementsLandlordCompliant
        );
        assert_eq!(out.disclosure_elements_satisfied, 5);
        assert_eq!(out.maximum_civil_penalty_cents, 0);
        assert_eq!(out.treble_damages_exposure_cents, 0);
    }

    #[test]
    fn post_1977_housing_exempt_no_disclosure_required() {
        let mut i = base_pre_1978_compliant();
        i.year_unit_built = 1980;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExemptPost1977Housing);
        assert!(out.notes.iter().any(|n| n.contains("1980")));
    }

    #[test]
    fn unit_built_in_1978_itself_exempt_threshold_boundary() {
        let mut i = base_pre_1978_compliant();
        i.year_unit_built = 1978;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExemptPost1977Housing);
    }

    #[test]
    fn unit_built_in_1977_target_housing_disclosure_required() {
        let mut i = base_pre_1978_compliant();
        i.year_unit_built = 1977;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::DisclosureCompleteAllFiveElementsLandlordCompliant
        );
    }

    #[test]
    fn short_term_lease_under_100_days_no_renewal_exempt() {
        let mut i = base_pre_1978_compliant();
        i.lease_term_days = 60;
        i.lease_renewal_or_extension_possible = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExemptShortTermLeaseUnder100Days);
    }

    #[test]
    fn short_term_lease_with_renewal_possible_not_exempt() {
        let mut i = base_pre_1978_compliant();
        i.lease_term_days = 60;
        i.lease_renewal_or_extension_possible = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::DisclosureCompleteAllFiveElementsLandlordCompliant
        );
    }

    #[test]
    fn zero_bedroom_no_child_expected_exempt() {
        let mut i = base_pre_1978_compliant();
        i.bedrooms = 0;
        i.child_under_six_expected = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExemptZeroBedroomNoChildExpected);
    }

    #[test]
    fn zero_bedroom_with_child_expected_disclosure_required() {
        let mut i = base_pre_1978_compliant();
        i.bedrooms = 0;
        i.child_under_six_expected = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::DisclosureCompleteAllFiveElementsLandlordCompliant
        );
    }

    #[test]
    fn elderly_only_housing_no_child_expected_exempt() {
        let mut i = base_pre_1978_compliant();
        i.elderly_only_or_disabled_only_housing = true;
        i.child_under_six_expected = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ExemptElderlyOrDisabledHousingNoChildExpected
        );
    }

    #[test]
    fn elderly_only_with_child_expected_disclosure_required() {
        let mut i = base_pre_1978_compliant();
        i.elderly_only_or_disabled_only_housing = true;
        i.child_under_six_expected = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::DisclosureCompleteAllFiveElementsLandlordCompliant
        );
    }

    #[test]
    fn missing_lead_warning_statement_only_per_se_violation() {
        let mut i = base_pre_1978_compliant();
        i.lead_warning_statement_in_lease = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::DisclosureIncompleteMissingLeadWarningStatementInLeaseViolation
        );
        assert_eq!(out.disclosure_elements_satisfied, 4);
        assert_eq!(out.maximum_civil_penalty_cents, 2_226_300);
    }

    #[test]
    fn missing_known_lbp_disclosure_only_per_se_violation() {
        let mut i = base_pre_1978_compliant();
        i.known_lbp_disclosed_or_no_knowledge_stated = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::DisclosureIncompleteMissingKnownLbpDisclosureViolation
        );
    }

    #[test]
    fn missing_epa_pamphlet_only_per_se_violation() {
        let mut i = base_pre_1978_compliant();
        i.epa_pamphlet_provided = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::DisclosureIncompleteMissingEpaPamphletViolation
        );
        assert!(out.notes.iter().any(|n| n.contains("EPA")));
    }

    #[test]
    fn missing_signed_acknowledgment_only_per_se_violation() {
        let mut i = base_pre_1978_compliant();
        i.tenant_signed_acknowledgment = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::DisclosureIncompleteMissingSignedAcknowledgmentViolation
        );
    }

    #[test]
    fn missing_two_elements_aggravated_violation() {
        let mut i = base_pre_1978_compliant();
        i.epa_pamphlet_provided = false;
        i.tenant_signed_acknowledgment = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::DisclosureIncompleteMissingMultipleElementsAggravatedViolation
        );
        assert_eq!(out.disclosure_elements_satisfied, 3);
    }

    #[test]
    fn treble_damages_compute_on_actual_damages() {
        let mut i = base_pre_1978_compliant();
        i.epa_pamphlet_provided = false;
        i.actual_damages_to_tenant_cents = 1_000_000;
        let out = check(&i);
        assert_eq!(out.treble_damages_exposure_cents, 3_000_000);
        assert_eq!(out.total_landlord_exposure_cents, 5_226_300);
    }

    #[test]
    fn citations_pin_42_usc_4852d_title_x_24_cfr_40_cfr() {
        let out = check(&base_pre_1978_compliant());
        assert!(out.citations.iter().any(|c| c.contains("42 U.S.C. § 4852d")));
        assert!(out.citations.iter().any(|c| c.contains("Title X § 1018")));
        assert!(out.citations.iter().any(|c| c.contains("24 C.F.R. Part 35")));
        assert!(out.citations.iter().any(|c| c.contains("40 C.F.R. Part 745")));
    }

    #[test]
    fn citations_pin_treble_damages_and_civil_penalty_subsections() {
        let out = check(&base_pre_1978_compliant());
        assert!(out.citations.iter().any(|c| c.contains("4852d(b)(3)")));
        assert!(out.citations.iter().any(|c| c.contains("4852d(b)(5)")));
    }

    #[test]
    fn constant_pin_civil_penalty_max_22263_dollars() {
        assert_eq!(TITLE_X_SECTION_1018_CIVIL_PENALTY_MAX_CENTS_2025, 2_226_300);
    }

    #[test]
    fn constant_pin_100_day_short_term_lease_threshold() {
        assert_eq!(SHORT_TERM_LEASE_EXEMPTION_DAYS, 100);
    }

    #[test]
    fn constant_pin_child_under_6_trigger() {
        assert_eq!(CHILD_AGE_TRIGGER_BELOW_YEARS, 6);
    }

    #[test]
    fn constant_pin_pre_1978_target_housing_threshold() {
        assert_eq!(PRE_1978_TARGET_HOUSING_THRESHOLD_YEAR, 1978);
    }

    #[test]
    fn constant_pin_treble_damages_multiplier() {
        assert_eq!(TREBLE_DAMAGES_MULTIPLIER, 3);
    }

    #[test]
    fn very_large_damages_saturating_does_not_overflow() {
        let mut i = base_pre_1978_compliant();
        i.epa_pamphlet_provided = false;
        i.actual_damages_to_tenant_cents = u64::MAX;
        let out = check(&i);
        assert_eq!(out.treble_damages_exposure_cents, u64::MAX);
        assert_eq!(out.total_landlord_exposure_cents, u64::MAX);
    }

    #[test]
    fn pre_1978_lease_100_days_no_renewal_exemption_works_with_child() {
        let mut i = base_pre_1978_compliant();
        i.lease_term_days = 100;
        i.lease_renewal_or_extension_possible = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExemptShortTermLeaseUnder100Days);
    }

    #[test]
    fn pre_1978_lease_101_days_no_renewal_disclosure_required() {
        let mut i = base_pre_1978_compliant();
        i.lease_term_days = 101;
        i.lease_renewal_or_extension_possible = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::DisclosureCompleteAllFiveElementsLandlordCompliant
        );
    }
}
