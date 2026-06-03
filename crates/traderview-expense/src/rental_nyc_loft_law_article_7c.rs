//! NYC Loft Law (NY Multiple Dwelling Law Article 7-C) compliance
//! for trader-landlords with NYC commercial/manufacturing loft
//! building inventory.
//!
//! Article 7-C of the NY Multiple Dwelling Law (MDL §§ 280-287)
//! was enacted in 1982 (NY Laws of 1982, c. 349) to address the
//! widespread residential occupation of commercial and
//! manufacturing loft buildings in SoHo, TriBeCa, NoHo, the Lower
//! East Side, Williamsburg, and Long Island City during the 1970s
//! and early 1980s. The statute creates a third building
//! classification — **Interim Multiple Dwelling (IMD)** — for
//! commercial/manufacturing buildings residentially occupied
//! during specified eligibility windows, and mandates legalization
//! through code compliance with Article 7-B safety and fire
//! protection standards.
//!
//! **MDL § 281 IMD coverage criteria**:
//!
//! - Building was originally commercial or manufacturing in use;
//! - Residentially occupied by 3 or more families living
//!   independently from one another for 12 consecutive months
//!   during a specified eligibility window;
//! - Not previously a legal residential building with Certificate
//!   of Occupancy.
//!
//! **Eligibility windows**:
//!
//! - **Original (1982 law)**: residential occupation April 1, 1980
//!   to December 1, 1981.
//! - **2010 expansion** (Laws of 2010, c. 135) — MDL § 281(5):
//!   residential occupation 12 consecutive months in 2008-2009.
//! - **2013 amendment** (Laws of 2013, c. 4): clarified coverage
//!   for additional building types.
//! - **2019 amendment**: expanded coverage criteria.
//!
//! **MDL § 284 code compliance timetable**:
//!
//! - **Stage 1**: file alteration application with NYC DOB within
//!   12 months of IMD designation.
//! - **Stage 2**: obtain alteration permit within 18 months.
//! - **Stage 3**: achieve Article 7-B safety and fire protection
//!   compliance within 36 months.
//! - **Stage 4**: obtain residential Certificate of Occupancy
//!   within 48 months.
//!
//! The NYC Loft Board may grant extensions for cause but generally
//! enforces strict compliance with the statutory schedule.
//!
//! **Narrative Statement Process** (29 RCNY § 2-01.1): document
//! describing in plain language the proposed alterations for each
//! unit (residential and non-residential), work the owner will
//! perform in such unit, and work in common areas. Once Loft Board
//! verifies the Narrative Statement process is complete, DOB plan
//! examiner can issue a building permit.
//!
//! **Protected Occupant** under MDL § 286: an occupant qualified
//! for protection under Article 7-C is a "Protected Occupant" and
//! CANNOT be evicted except for good cause; entitled to rent
//! stabilization after building legalization.
//!
//! **Civil penalties** under Loft Board Rules 29 RCNY § 2-12:
//! $500 minimum, $1,000+ maximum per violation per day. Cumulative
//! daily penalties for ongoing non-compliance with statutory
//! timetable.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[allow(dead_code)]
pub const LOFT_LAW_ORIGINAL_ENACTMENT_YEAR: u32 = 1982;
#[allow(dead_code)]
pub const LOFT_LAW_2010_AMENDMENT_YEAR: u32 = 2010;
#[allow(dead_code)]
pub const LOFT_LAW_2013_AMENDMENT_YEAR: u32 = 2013;
#[allow(dead_code)]
pub const LOFT_LAW_2019_AMENDMENT_YEAR: u32 = 2019;
#[allow(dead_code)]
pub const IMD_MIN_FAMILY_UNITS: u32 = 3;
#[allow(dead_code)]
pub const IMD_RESIDENTIAL_USE_CONSECUTIVE_MONTHS: u32 = 12;
#[allow(dead_code)]
pub const IMD_2008_2009_WINDOW_START_YEAR: u32 = 2008;
#[allow(dead_code)]
pub const IMD_2008_2009_WINDOW_END_YEAR: u32 = 2009;
#[allow(dead_code)]
pub const IMD_ORIGINAL_WINDOW_START_YEAR: u32 = 1980;
#[allow(dead_code)]
pub const IMD_ORIGINAL_WINDOW_END_YEAR: u32 = 1981;
#[allow(dead_code)]
pub const ALTERATION_APPLICATION_DEADLINE_MONTHS: u32 = 12;
#[allow(dead_code)]
pub const ALTERATION_PERMIT_DEADLINE_MONTHS: u32 = 18;
#[allow(dead_code)]
pub const ARTICLE_7B_COMPLIANCE_DEADLINE_MONTHS: u32 = 36;
#[allow(dead_code)]
pub const CERTIFICATE_OF_OCCUPANCY_DEADLINE_MONTHS: u32 = 48;
#[allow(dead_code)]
pub const LOFT_BOARD_CIVIL_PENALTY_MIN_CENTS_PER_DAY: u64 = 50_000;
#[allow(dead_code)]
pub const LOFT_BOARD_CIVIL_PENALTY_MAX_CENTS_PER_DAY: u64 = 100_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    ExemptResidentialFromInception,
    ExemptCommercialFullyVacant,
    CompliantImdRegisteredAndLegalizationOnSchedule,
    CompliantArticle7BComplianceMet,
    CompliantCertificateOfOccupancyObtainedRemovedFromLoftBoardJurisdiction,
    ViolationImdNotRegisteredWithLoftBoard,
    ViolationNarrativeStatementNotFiled,
    ViolationAlterationApplicationNotFiledWithin12Months,
    ViolationAlterationPermitNotObtainedWithin18Months,
    ViolationArticle7BComplianceNotMetWithin36Months,
    ViolationCertificateOfOccupancyNotObtainedWithin48Months,
    ViolationHarassmentOrIllegalEvictionOfProtectedOccupant,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub former_commercial_or_manufacturing_use: bool,
    pub consecutive_months_residential_use_in_window: u32,
    pub number_of_independent_family_units: u32,
    pub previously_legal_residential_with_cofo: bool,
    pub imd_registered_with_loft_board: bool,
    pub narrative_statement_filed: bool,
    pub months_since_imd_designation: u32,
    pub alteration_application_filed: bool,
    pub alteration_permit_obtained: bool,
    pub article_7b_compliance_met: bool,
    pub certificate_of_occupancy_obtained: bool,
    pub harassment_of_protected_occupant: bool,
    pub days_of_continuing_violation: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Output {
    pub severity: Severity,
    pub compliant: bool,
    pub imd_status: bool,
    pub daily_civil_penalty_min_cents: u64,
    pub aggregate_civil_penalty_min_cents: u64,
    pub code_compliance_deadline_months: u32,
    pub notes: Vec<String>,
    pub citations: Vec<String>,
}

pub type NycLoftLawArticle7CInput = Input;
pub type NycLoftLawArticle7COutput = Output;
pub type NycLoftLawArticle7CResult = Output;

pub fn check(input: &Input) -> Output {
    let mut notes: Vec<String> = Vec::new();
    let citations: Vec<String> = vec![
        "NY Multiple Dwelling Law Article 7-C (MDL §§ 280-287 — Loft Law)".to_string(),
        "MDL § 280 (legislative findings)".to_string(),
        "MDL § 281 (Interim Multiple Dwelling — IMD definition)".to_string(),
        "MDL § 281(5) (2010 expansion — 2008-2009 residential occupation window)".to_string(),
        "MDL § 284 (code compliance timetable — 12/18/36/48 months)".to_string(),
        "MDL § 286 (Protected Occupant — protection from eviction except good cause)".to_string(),
        "MDL § 287 (Loft Board jurisdiction)".to_string(),
        "29 RCNY Subchapter B (NYC Loft Board Rules)".to_string(),
        "29 RCNY § 2-01.1 (Narrative Statement process)".to_string(),
        "29 RCNY § 2-12 (civil penalties — $500-$1,000+ per violation per day)".to_string(),
        "NY Laws of 1982, c. 349 (Loft Law original enactment)".to_string(),
        "NY Laws of 2010, c. 135 (Loft Law 2010 expansion)".to_string(),
        "NY Laws of 2013, c. 4 (Loft Law 2013 amendment)".to_string(),
        "NY Laws of 2019 (Loft Law 2019 expansion)".to_string(),
        "Article 7-B of NY MDL (safety and fire protection standards)".to_string(),
    ];

    if !input.former_commercial_or_manufacturing_use {
        notes.push("Building was residential from inception — Article 7-C does not apply.".to_string());
        return Output {
            severity: Severity::ExemptResidentialFromInception,
            compliant: true,
            imd_status: false,
            daily_civil_penalty_min_cents: 0,
            aggregate_civil_penalty_min_cents: 0,
            code_compliance_deadline_months: 0,
            notes,
            citations,
        };
    }

    if input.previously_legal_residential_with_cofo {
        notes.push("Building previously had legal residential Certificate of Occupancy — outside MDL § 281 IMD coverage.".to_string());
        return Output {
            severity: Severity::ExemptResidentialFromInception,
            compliant: true,
            imd_status: false,
            daily_civil_penalty_min_cents: 0,
            aggregate_civil_penalty_min_cents: 0,
            code_compliance_deadline_months: 0,
            notes,
            citations,
        };
    }

    let imd_eligible = input.consecutive_months_residential_use_in_window
        >= IMD_RESIDENTIAL_USE_CONSECUTIVE_MONTHS
        && input.number_of_independent_family_units >= IMD_MIN_FAMILY_UNITS;

    if !imd_eligible {
        notes.push(format!(
            "IMD eligibility not met: {} consecutive months residential (need ≥ {}); {} independent family units (need ≥ {}).",
            input.consecutive_months_residential_use_in_window,
            IMD_RESIDENTIAL_USE_CONSECUTIVE_MONTHS,
            input.number_of_independent_family_units,
            IMD_MIN_FAMILY_UNITS
        ));
        return Output {
            severity: Severity::ExemptCommercialFullyVacant,
            compliant: true,
            imd_status: false,
            daily_civil_penalty_min_cents: 0,
            aggregate_civil_penalty_min_cents: 0,
            code_compliance_deadline_months: 0,
            notes,
            citations,
        };
    }

    if input.harassment_of_protected_occupant {
        notes.push("Owner harassment or illegal eviction of Protected Occupant — MDL § 286 violation; Loft Board may impose substantial civil penalties + tenant action for damages.".to_string());
        let penalty = LOFT_BOARD_CIVIL_PENALTY_MAX_CENTS_PER_DAY
            .saturating_mul(input.days_of_continuing_violation as u64);
        return Output {
            severity: Severity::ViolationHarassmentOrIllegalEvictionOfProtectedOccupant,
            compliant: false,
            imd_status: true,
            daily_civil_penalty_min_cents: LOFT_BOARD_CIVIL_PENALTY_MAX_CENTS_PER_DAY,
            aggregate_civil_penalty_min_cents: penalty,
            code_compliance_deadline_months: ALTERATION_APPLICATION_DEADLINE_MONTHS,
            notes,
            citations,
        };
    }

    if !input.imd_registered_with_loft_board {
        notes.push("IMD not registered with NYC Loft Board — 29 RCNY § 2-05 registration violation.".to_string());
        let penalty = LOFT_BOARD_CIVIL_PENALTY_MIN_CENTS_PER_DAY
            .saturating_mul(input.days_of_continuing_violation as u64);
        return Output {
            severity: Severity::ViolationImdNotRegisteredWithLoftBoard,
            compliant: false,
            imd_status: true,
            daily_civil_penalty_min_cents: LOFT_BOARD_CIVIL_PENALTY_MIN_CENTS_PER_DAY,
            aggregate_civil_penalty_min_cents: penalty,
            code_compliance_deadline_months: ALTERATION_APPLICATION_DEADLINE_MONTHS,
            notes,
            citations,
        };
    }

    if !input.narrative_statement_filed {
        notes.push("Narrative Statement not filed with Loft Board — 29 RCNY § 2-01.1 violation; DOB cannot issue alteration permit.".to_string());
        return Output {
            severity: Severity::ViolationNarrativeStatementNotFiled,
            compliant: false,
            imd_status: true,
            daily_civil_penalty_min_cents: LOFT_BOARD_CIVIL_PENALTY_MIN_CENTS_PER_DAY,
            aggregate_civil_penalty_min_cents: LOFT_BOARD_CIVIL_PENALTY_MIN_CENTS_PER_DAY
                .saturating_mul(input.days_of_continuing_violation as u64),
            code_compliance_deadline_months: ALTERATION_APPLICATION_DEADLINE_MONTHS,
            notes,
            citations,
        };
    }

    if input.months_since_imd_designation > ALTERATION_APPLICATION_DEADLINE_MONTHS
        && !input.alteration_application_filed
    {
        notes.push(format!(
            "Alteration application not filed within {} months of IMD designation ({} months elapsed) — MDL § 284 Stage 1 violation.",
            ALTERATION_APPLICATION_DEADLINE_MONTHS,
            input.months_since_imd_designation
        ));
        return Output {
            severity: Severity::ViolationAlterationApplicationNotFiledWithin12Months,
            compliant: false,
            imd_status: true,
            daily_civil_penalty_min_cents: LOFT_BOARD_CIVIL_PENALTY_MIN_CENTS_PER_DAY,
            aggregate_civil_penalty_min_cents: LOFT_BOARD_CIVIL_PENALTY_MIN_CENTS_PER_DAY
                .saturating_mul(input.days_of_continuing_violation as u64),
            code_compliance_deadline_months: ALTERATION_APPLICATION_DEADLINE_MONTHS,
            notes,
            citations,
        };
    }

    if input.months_since_imd_designation > ALTERATION_PERMIT_DEADLINE_MONTHS
        && !input.alteration_permit_obtained
    {
        notes.push(format!(
            "Alteration permit not obtained within {} months of IMD designation ({} months elapsed) — MDL § 284 Stage 2 violation.",
            ALTERATION_PERMIT_DEADLINE_MONTHS,
            input.months_since_imd_designation
        ));
        return Output {
            severity: Severity::ViolationAlterationPermitNotObtainedWithin18Months,
            compliant: false,
            imd_status: true,
            daily_civil_penalty_min_cents: LOFT_BOARD_CIVIL_PENALTY_MIN_CENTS_PER_DAY,
            aggregate_civil_penalty_min_cents: LOFT_BOARD_CIVIL_PENALTY_MIN_CENTS_PER_DAY
                .saturating_mul(input.days_of_continuing_violation as u64),
            code_compliance_deadline_months: ALTERATION_PERMIT_DEADLINE_MONTHS,
            notes,
            citations,
        };
    }

    if input.months_since_imd_designation > ARTICLE_7B_COMPLIANCE_DEADLINE_MONTHS
        && !input.article_7b_compliance_met
    {
        notes.push(format!(
            "Article 7-B safety and fire protection compliance not met within {} months of IMD designation ({} months elapsed) — MDL § 284 Stage 3 violation.",
            ARTICLE_7B_COMPLIANCE_DEADLINE_MONTHS,
            input.months_since_imd_designation
        ));
        return Output {
            severity: Severity::ViolationArticle7BComplianceNotMetWithin36Months,
            compliant: false,
            imd_status: true,
            daily_civil_penalty_min_cents: LOFT_BOARD_CIVIL_PENALTY_MIN_CENTS_PER_DAY,
            aggregate_civil_penalty_min_cents: LOFT_BOARD_CIVIL_PENALTY_MIN_CENTS_PER_DAY
                .saturating_mul(input.days_of_continuing_violation as u64),
            code_compliance_deadline_months: ARTICLE_7B_COMPLIANCE_DEADLINE_MONTHS,
            notes,
            citations,
        };
    }

    if input.months_since_imd_designation > CERTIFICATE_OF_OCCUPANCY_DEADLINE_MONTHS
        && !input.certificate_of_occupancy_obtained
    {
        notes.push(format!(
            "Residential Certificate of Occupancy not obtained within {} months of IMD designation ({} months elapsed) — MDL § 284 Stage 4 violation.",
            CERTIFICATE_OF_OCCUPANCY_DEADLINE_MONTHS,
            input.months_since_imd_designation
        ));
        return Output {
            severity: Severity::ViolationCertificateOfOccupancyNotObtainedWithin48Months,
            compliant: false,
            imd_status: true,
            daily_civil_penalty_min_cents: LOFT_BOARD_CIVIL_PENALTY_MIN_CENTS_PER_DAY,
            aggregate_civil_penalty_min_cents: LOFT_BOARD_CIVIL_PENALTY_MIN_CENTS_PER_DAY
                .saturating_mul(input.days_of_continuing_violation as u64),
            code_compliance_deadline_months: CERTIFICATE_OF_OCCUPANCY_DEADLINE_MONTHS,
            notes,
            citations,
        };
    }

    if input.certificate_of_occupancy_obtained {
        notes.push("Residential Certificate of Occupancy obtained — building removed from Loft Board jurisdiction; subject to NYC Rent Stabilization Law.".to_string());
        return Output {
            severity:
                Severity::CompliantCertificateOfOccupancyObtainedRemovedFromLoftBoardJurisdiction,
            compliant: true,
            imd_status: false,
            daily_civil_penalty_min_cents: 0,
            aggregate_civil_penalty_min_cents: 0,
            code_compliance_deadline_months: CERTIFICATE_OF_OCCUPANCY_DEADLINE_MONTHS,
            notes,
            citations,
        };
    }

    if input.article_7b_compliance_met {
        notes.push("Article 7-B safety and fire protection compliance met; pending Certificate of Occupancy.".to_string());
        return Output {
            severity: Severity::CompliantArticle7BComplianceMet,
            compliant: true,
            imd_status: true,
            daily_civil_penalty_min_cents: 0,
            aggregate_civil_penalty_min_cents: 0,
            code_compliance_deadline_months: CERTIFICATE_OF_OCCUPANCY_DEADLINE_MONTHS,
            notes,
            citations,
        };
    }

    notes.push("IMD registered with Loft Board; legalization on schedule per MDL § 284 timetable.".to_string());
    Output {
        severity: Severity::CompliantImdRegisteredAndLegalizationOnSchedule,
        compliant: true,
        imd_status: true,
        daily_civil_penalty_min_cents: 0,
        aggregate_civil_penalty_min_cents: 0,
        code_compliance_deadline_months: CERTIFICATE_OF_OCCUPANCY_DEADLINE_MONTHS,
        notes,
        citations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_compliant_imd() -> Input {
        Input {
            former_commercial_or_manufacturing_use: true,
            consecutive_months_residential_use_in_window: 18,
            number_of_independent_family_units: 5,
            previously_legal_residential_with_cofo: false,
            imd_registered_with_loft_board: true,
            narrative_statement_filed: true,
            months_since_imd_designation: 6,
            alteration_application_filed: true,
            alteration_permit_obtained: true,
            article_7b_compliance_met: false,
            certificate_of_occupancy_obtained: false,
            harassment_of_protected_occupant: false,
            days_of_continuing_violation: 0,
        }
    }

    #[test]
    fn imd_registered_legalization_on_schedule_compliant() {
        let out = check(&base_compliant_imd());
        assert_eq!(
            out.severity,
            Severity::CompliantImdRegisteredAndLegalizationOnSchedule
        );
        assert!(out.imd_status);
        assert!(out.compliant);
    }

    #[test]
    fn residential_from_inception_exempt() {
        let mut i = base_compliant_imd();
        i.former_commercial_or_manufacturing_use = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExemptResidentialFromInception);
    }

    #[test]
    fn previously_residential_cofo_exempt() {
        let mut i = base_compliant_imd();
        i.previously_legal_residential_with_cofo = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExemptResidentialFromInception);
    }

    #[test]
    fn under_12_consecutive_months_not_imd() {
        let mut i = base_compliant_imd();
        i.consecutive_months_residential_use_in_window = 11;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExemptCommercialFullyVacant);
        assert!(!out.imd_status);
    }

    #[test]
    fn exactly_12_consecutive_months_qualifies_imd() {
        let mut i = base_compliant_imd();
        i.consecutive_months_residential_use_in_window = 12;
        let out = check(&i);
        assert!(out.imd_status);
    }

    #[test]
    fn under_3_family_units_not_imd() {
        let mut i = base_compliant_imd();
        i.number_of_independent_family_units = 2;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExemptCommercialFullyVacant);
    }

    #[test]
    fn exactly_3_family_units_qualifies_imd() {
        let mut i = base_compliant_imd();
        i.number_of_independent_family_units = 3;
        let out = check(&i);
        assert!(out.imd_status);
    }

    #[test]
    fn imd_not_registered_violation() {
        let mut i = base_compliant_imd();
        i.imd_registered_with_loft_board = false;
        i.days_of_continuing_violation = 60;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationImdNotRegisteredWithLoftBoard
        );
        assert_eq!(out.aggregate_civil_penalty_min_cents, 3_000_000);
    }

    #[test]
    fn narrative_statement_not_filed_violation() {
        let mut i = base_compliant_imd();
        i.narrative_statement_filed = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ViolationNarrativeStatementNotFiled);
    }

    #[test]
    fn alteration_application_not_filed_within_12_months_violation() {
        let mut i = base_compliant_imd();
        i.alteration_application_filed = false;
        i.months_since_imd_designation = 18;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationAlterationApplicationNotFiledWithin12Months
        );
    }

    #[test]
    fn alteration_application_at_exactly_12_months_compliant() {
        let mut i = base_compliant_imd();
        i.alteration_application_filed = false;
        i.months_since_imd_designation = 12;
        let out = check(&i);
        assert!(out.compliant);
    }

    #[test]
    fn alteration_permit_not_obtained_within_18_months_violation() {
        let mut i = base_compliant_imd();
        i.alteration_permit_obtained = false;
        i.months_since_imd_designation = 24;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationAlterationPermitNotObtainedWithin18Months
        );
    }

    #[test]
    fn article_7b_compliance_not_met_within_36_months_violation() {
        let mut i = base_compliant_imd();
        i.article_7b_compliance_met = false;
        i.months_since_imd_designation = 40;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationArticle7BComplianceNotMetWithin36Months
        );
    }

    #[test]
    fn certificate_of_occupancy_not_obtained_within_48_months_violation() {
        let mut i = base_compliant_imd();
        i.article_7b_compliance_met = true;
        i.certificate_of_occupancy_obtained = false;
        i.months_since_imd_designation = 60;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationCertificateOfOccupancyNotObtainedWithin48Months
        );
    }

    #[test]
    fn article_7b_met_pending_cofo_compliant() {
        let mut i = base_compliant_imd();
        i.article_7b_compliance_met = true;
        i.certificate_of_occupancy_obtained = false;
        i.months_since_imd_designation = 40;
        let out = check(&i);
        assert_eq!(out.severity, Severity::CompliantArticle7BComplianceMet);
    }

    #[test]
    fn cofo_obtained_removed_from_loft_board() {
        let mut i = base_compliant_imd();
        i.article_7b_compliance_met = true;
        i.certificate_of_occupancy_obtained = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::CompliantCertificateOfOccupancyObtainedRemovedFromLoftBoardJurisdiction
        );
        assert!(!out.imd_status);
    }

    #[test]
    fn harassment_of_protected_occupant_violation() {
        let mut i = base_compliant_imd();
        i.harassment_of_protected_occupant = true;
        i.days_of_continuing_violation = 30;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationHarassmentOrIllegalEvictionOfProtectedOccupant
        );
        assert_eq!(out.daily_civil_penalty_min_cents, 100_000);
        assert_eq!(out.aggregate_civil_penalty_min_cents, 3_000_000);
    }

    #[test]
    fn citations_pin_mdl_280_281_284_286_287() {
        let out = check(&base_compliant_imd());
        assert!(out.citations.iter().any(|c| c.contains("MDL § 280")));
        assert!(out.citations.iter().any(|c| c.contains("MDL § 281")));
        assert!(out.citations.iter().any(|c| c.contains("MDL § 284")));
        assert!(out.citations.iter().any(|c| c.contains("MDL § 286")));
        assert!(out.citations.iter().any(|c| c.contains("MDL § 287")));
    }

    #[test]
    fn citations_pin_29_rcny_loft_board_rules() {
        let out = check(&base_compliant_imd());
        assert!(out.citations.iter().any(|c| c.contains("29 RCNY")));
        assert!(out.citations.iter().any(|c| c.contains("§ 2-01.1")));
        assert!(out.citations.iter().any(|c| c.contains("§ 2-12")));
    }

    #[test]
    fn citations_pin_ny_laws_1982_2010_2013_2019() {
        let out = check(&base_compliant_imd());
        assert!(out.citations.iter().any(|c| c.contains("Laws of 1982")));
        assert!(out.citations.iter().any(|c| c.contains("Laws of 2010")));
        assert!(out.citations.iter().any(|c| c.contains("Laws of 2013")));
        assert!(out.citations.iter().any(|c| c.contains("Laws of 2019")));
    }

    #[test]
    fn constant_pin_3_family_unit_threshold() {
        assert_eq!(IMD_MIN_FAMILY_UNITS, 3);
    }

    #[test]
    fn constant_pin_12_month_residential_threshold() {
        assert_eq!(IMD_RESIDENTIAL_USE_CONSECUTIVE_MONTHS, 12);
    }

    #[test]
    fn constant_pin_12_18_36_48_month_timetable() {
        assert_eq!(ALTERATION_APPLICATION_DEADLINE_MONTHS, 12);
        assert_eq!(ALTERATION_PERMIT_DEADLINE_MONTHS, 18);
        assert_eq!(ARTICLE_7B_COMPLIANCE_DEADLINE_MONTHS, 36);
        assert_eq!(CERTIFICATE_OF_OCCUPANCY_DEADLINE_MONTHS, 48);
    }

    #[test]
    fn constant_pin_500_min_1000_max_daily_civil_penalty() {
        assert_eq!(LOFT_BOARD_CIVIL_PENALTY_MIN_CENTS_PER_DAY, 50_000);
        assert_eq!(LOFT_BOARD_CIVIL_PENALTY_MAX_CENTS_PER_DAY, 100_000);
    }

    #[test]
    fn constant_pin_1982_original_enactment_year() {
        assert_eq!(LOFT_LAW_ORIGINAL_ENACTMENT_YEAR, 1982);
    }

    #[test]
    fn very_large_continuing_violation_days_no_overflow() {
        let mut i = base_compliant_imd();
        i.imd_registered_with_loft_board = false;
        i.days_of_continuing_violation = u32::MAX;
        let out = check(&i);
        assert!(out.aggregate_civil_penalty_min_cents > 0);
    }
}
