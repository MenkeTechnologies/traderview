//! NYC Local Law 87 of 2009 Energy Audits & Retro-Commissioning
//! compliance for trader-landlords with NYC commercial or large
//! multifamily inventory.
//!
//! Local Law 87 of 2009 is part of the **Greener, Greater Buildings
//! Plan (GGBP)** along with Local Law 84 (annual benchmarking) and
//! Local Law 97 (emissions cap). Codified at NYC Admin Code
//! § 28-308. Requires owners of covered buildings to undergo an
//! **ASHRAE Level II energy audit** and **retro-commissioning**
//! (RCx) study of base building systems every **10 years** and
//! submit the results in an **Energy Efficiency Report (EER)** to
//! NYC DOB.
//!
//! **Coverage**: buildings with gross floor area **greater than
//! 50,000 square feet** ("covered buildings"). Excludes one- and
//! two-family residential (R-3 occupancy).
//!
//! **Filing schedule based on tax block last digit**: owner must
//! submit an EER in the calendar year in which the last digit of
//! the year coincides with the last digit of the building's tax
//! block number. E.g., tax block ending in "5" → file in 2015,
//! 2025, 2035, etc.
//!
//! **2025 filing extension**: NYC DOB extended the 2025 EER filing
//! deadline to **March 31, 2026** (one-time extension for 2025
//! filing year only). Subsequent filing years revert to standard
//! calendar-year deadline.
//!
//! **Qualified Professional requirement**: all individuals
//! performing or supervising Energy Audit and Retro-commissioning
//! studies for inclusion in an EER must be a **registered design
//! professional** (NY-licensed Professional Engineer or Registered
//! Architect) with appropriate qualifications, AND cannot be on
//! the staff of the building being audited or retro-commissioned.
//!
//! **ASHRAE Level II Energy Audit**: more detailed than Level I —
//! includes savings calculations, payback period analysis, and
//! identifies retrofit opportunities at the system/component level.
//!
//! **Retro-Commissioning (RCx)**: process to ensure existing
//! energy systems are installed as per design intentions,
//! functionally tested, and capable of being operated and
//! maintained according to owner's operational needs.
//!
//! **Civil penalties** (NYC Admin Code § 28-308.7): **$3,000** for
//! the first year of non-submittal; **$5,000** for each additional
//! year. NYC DOB will NOT accept any outstanding EER submission if
//! outstanding penalties are not paid in full.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[allow(dead_code)]
pub const LL87_ENACTMENT_YEAR: u32 = 2009;
#[allow(dead_code)]
pub const LL87_BUILDING_SIZE_THRESHOLD_SQFT: u32 = 50_000;
#[allow(dead_code)]
pub const LL87_REPORTING_CYCLE_YEARS: u32 = 10;
#[allow(dead_code)]
pub const LL87_FIRST_YEAR_CIVIL_PENALTY_CENTS: u64 = 300_000;
#[allow(dead_code)]
pub const LL87_SUBSEQUENT_YEAR_CIVIL_PENALTY_CENTS: u64 = 500_000;
#[allow(dead_code)]
pub const LL87_2025_FILING_EXTENSION_DEADLINE_YEAR: u32 = 2026;
#[allow(dead_code)]
pub const LL87_2025_FILING_EXTENSION_DEADLINE_MONTH: u32 = 3;
#[allow(dead_code)]
pub const LL87_2025_FILING_EXTENSION_DEADLINE_DAY: u32 = 31;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    ExemptBuildingUnder50000Sqft,
    CompliantEerFiledOnTimeForAssignedYear,
    CompliantExtensionGrantedAndFiledByMarch31_2026,
    ViolationEerNotFiledByAssignedDeadline,
    ViolationQualifiedProfessionalNotLicensedPeOrRa,
    ViolationQualifiedProfessionalIsBuildingStaffMember,
    ViolationAuditNotAshraeLevelII,
    AggravatedViolationCivilPenaltyAccumulatingMultipleYears,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub gross_floor_area_sqft: u32,
    pub building_tax_block_last_digit: u32,
    pub current_year: u32,
    pub eer_filed_in_correct_year: bool,
    pub extension_granted_for_2025_filing: bool,
    pub eer_filed_by_extension_deadline_march_31_2026: bool,
    pub audit_is_ashrae_level_ii: bool,
    pub qualified_professional_pe_or_ra: bool,
    pub qualified_professional_is_building_staff: bool,
    pub years_of_continuing_non_submittal: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Output {
    pub severity: Severity,
    pub compliant: bool,
    pub total_penalty_cents: u64,
    pub assigned_filing_year: u32,
    pub next_filing_year: u32,
    pub notes: Vec<String>,
    pub citations: Vec<String>,
}

pub type LocalLaw87EnergyAuditRetroCommissioningInput = Input;
pub type LocalLaw87EnergyAuditRetroCommissioningOutput = Output;
pub type LocalLaw87EnergyAuditRetroCommissioningResult = Output;

pub fn check(input: &Input) -> Output {
    let mut notes: Vec<String> = Vec::new();
    let citations: Vec<String> = vec![
        "NYC Local Law 87 of 2009 (Energy Audits & Retro-Commissioning)".to_string(),
        "NYC Admin Code § 28-308 (statutory codification)".to_string(),
        "NYC Admin Code § 28-308.7 (penalty schedule: $3K first / $5K subsequent year)".to_string(),
        "Greener, Greater Buildings Plan (GGBP)".to_string(),
        "ASHRAE Standard 90.1 (energy efficiency reference)".to_string(),
        "ASHRAE Audit Level II (commercial building energy audit methodology)".to_string(),
        "NYC DOB EER2 — Application for Extension to File an Energy Efficiency Report".to_string(),
        "NYC DOB Energy Efficiency Report (EER) submission requirements".to_string(),
        "NYC DOB 2025 Filing Extension Service Notice (March 31, 2026 deadline)".to_string(),
    ];

    let assigned_filing_year =
        compute_assigned_filing_year(input.building_tax_block_last_digit, input.current_year);
    let next_filing_year = assigned_filing_year + LL87_REPORTING_CYCLE_YEARS;

    if input.gross_floor_area_sqft <= LL87_BUILDING_SIZE_THRESHOLD_SQFT {
        notes.push(format!(
            "Building gross floor area {} sqft ≤ {} sqft threshold — exempt from LL 87.",
            input.gross_floor_area_sqft, LL87_BUILDING_SIZE_THRESHOLD_SQFT
        ));
        return Output {
            severity: Severity::ExemptBuildingUnder50000Sqft,
            compliant: true,
            total_penalty_cents: 0,
            assigned_filing_year,
            next_filing_year,
            notes,
            citations,
        };
    }

    if !input.audit_is_ashrae_level_ii {
        notes.push("Audit performed below ASHRAE Level II standard — LL 87 requires ASHRAE Level II energy audit; EER non-compliant.".to_string());
        return Output {
            severity: Severity::ViolationAuditNotAshraeLevelII,
            compliant: false,
            total_penalty_cents: LL87_FIRST_YEAR_CIVIL_PENALTY_CENTS,
            assigned_filing_year,
            next_filing_year,
            notes,
            citations,
        };
    }

    if !input.qualified_professional_pe_or_ra {
        notes.push("Qualified Professional must be NY-licensed Professional Engineer or Registered Architect — LL 87 procedural violation.".to_string());
        return Output {
            severity: Severity::ViolationQualifiedProfessionalNotLicensedPeOrRa,
            compliant: false,
            total_penalty_cents: LL87_FIRST_YEAR_CIVIL_PENALTY_CENTS,
            assigned_filing_year,
            next_filing_year,
            notes,
            citations,
        };
    }

    if input.qualified_professional_is_building_staff {
        notes.push("Qualified Professional is on staff of building being audited — LL 87 prohibits staff QP; independence requirement violated.".to_string());
        return Output {
            severity: Severity::ViolationQualifiedProfessionalIsBuildingStaffMember,
            compliant: false,
            total_penalty_cents: LL87_FIRST_YEAR_CIVIL_PENALTY_CENTS,
            assigned_filing_year,
            next_filing_year,
            notes,
            citations,
        };
    }

    if input.eer_filed_in_correct_year {
        notes.push(format!(
            "EER filed on time for assigned year {} (tax block last digit {}).",
            assigned_filing_year, input.building_tax_block_last_digit
        ));
        return Output {
            severity: Severity::CompliantEerFiledOnTimeForAssignedYear,
            compliant: true,
            total_penalty_cents: 0,
            assigned_filing_year,
            next_filing_year,
            notes,
            citations,
        };
    }

    if assigned_filing_year == 2025
        && input.extension_granted_for_2025_filing
        && input.eer_filed_by_extension_deadline_march_31_2026
    {
        notes.push("EER filed under 2025 extension by March 31, 2026 deadline.".to_string());
        return Output {
            severity: Severity::CompliantExtensionGrantedAndFiledByMarch31_2026,
            compliant: true,
            total_penalty_cents: 0,
            assigned_filing_year,
            next_filing_year,
            notes,
            citations,
        };
    }

    let penalty = LL87_FIRST_YEAR_CIVIL_PENALTY_CENTS.saturating_add(
        LL87_SUBSEQUENT_YEAR_CIVIL_PENALTY_CENTS
            .saturating_mul(input.years_of_continuing_non_submittal.saturating_sub(1) as u64),
    );

    if input.years_of_continuing_non_submittal > 1 {
        notes.push(format!(
            "EER not filed for {} years past assigned deadline — § 28-308.7 cumulative penalties: $3,000 first year + $5,000 each additional year = ${}.",
            input.years_of_continuing_non_submittal,
            penalty / 100
        ));
        return Output {
            severity: Severity::AggravatedViolationCivilPenaltyAccumulatingMultipleYears,
            compliant: false,
            total_penalty_cents: penalty,
            assigned_filing_year,
            next_filing_year,
            notes,
            citations,
        };
    }

    notes.push(format!(
        "EER not filed by assigned deadline for year {} — first-year ${} civil penalty.",
        assigned_filing_year,
        LL87_FIRST_YEAR_CIVIL_PENALTY_CENTS / 100
    ));
    Output {
        severity: Severity::ViolationEerNotFiledByAssignedDeadline,
        compliant: false,
        total_penalty_cents: LL87_FIRST_YEAR_CIVIL_PENALTY_CENTS,
        assigned_filing_year,
        next_filing_year,
        notes,
        citations,
    }
}

fn compute_assigned_filing_year(tax_block_last_digit: u32, current_year: u32) -> u32 {
    let decade_base = (current_year / 10) * 10;
    let candidate = decade_base + tax_block_last_digit;
    if candidate < current_year {
        candidate + 10
    } else {
        candidate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_compliant() -> Input {
        Input {
            gross_floor_area_sqft: 100_000,
            building_tax_block_last_digit: 5,
            current_year: 2025,
            eer_filed_in_correct_year: true,
            extension_granted_for_2025_filing: false,
            eer_filed_by_extension_deadline_march_31_2026: false,
            audit_is_ashrae_level_ii: true,
            qualified_professional_pe_or_ra: true,
            qualified_professional_is_building_staff: false,
            years_of_continuing_non_submittal: 0,
        }
    }

    #[test]
    fn fully_compliant_baseline() {
        let out = check(&base_compliant());
        assert_eq!(
            out.severity,
            Severity::CompliantEerFiledOnTimeForAssignedYear
        );
        assert!(out.compliant);
        assert_eq!(out.total_penalty_cents, 0);
    }

    #[test]
    fn building_under_50000_sqft_exempt() {
        let mut i = base_compliant();
        i.gross_floor_area_sqft = 49_999;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExemptBuildingUnder50000Sqft);
    }

    #[test]
    fn building_at_exactly_50000_sqft_exempt() {
        let mut i = base_compliant();
        i.gross_floor_area_sqft = 50_000;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExemptBuildingUnder50000Sqft);
    }

    #[test]
    fn building_at_50001_sqft_covered() {
        let mut i = base_compliant();
        i.gross_floor_area_sqft = 50_001;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::CompliantEerFiledOnTimeForAssignedYear
        );
    }

    #[test]
    fn audit_below_ashrae_level_ii_violation() {
        let mut i = base_compliant();
        i.audit_is_ashrae_level_ii = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ViolationAuditNotAshraeLevelII);
    }

    #[test]
    fn qualified_professional_not_pe_or_ra_violation() {
        let mut i = base_compliant();
        i.qualified_professional_pe_or_ra = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationQualifiedProfessionalNotLicensedPeOrRa
        );
    }

    #[test]
    fn qualified_professional_is_building_staff_violation() {
        let mut i = base_compliant();
        i.qualified_professional_is_building_staff = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationQualifiedProfessionalIsBuildingStaffMember
        );
    }

    #[test]
    fn eer_not_filed_by_assigned_deadline_3000_penalty() {
        let mut i = base_compliant();
        i.eer_filed_in_correct_year = false;
        i.years_of_continuing_non_submittal = 1;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationEerNotFiledByAssignedDeadline
        );
        assert_eq!(out.total_penalty_cents, 300_000);
    }

    #[test]
    fn eer_two_years_late_penalty_8000() {
        let mut i = base_compliant();
        i.eer_filed_in_correct_year = false;
        i.years_of_continuing_non_submittal = 2;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::AggravatedViolationCivilPenaltyAccumulatingMultipleYears
        );
        assert_eq!(out.total_penalty_cents, 800_000);
    }

    #[test]
    fn eer_three_years_late_penalty_13000() {
        let mut i = base_compliant();
        i.eer_filed_in_correct_year = false;
        i.years_of_continuing_non_submittal = 3;
        let out = check(&i);
        assert_eq!(out.total_penalty_cents, 1_300_000);
    }

    #[test]
    fn extension_granted_filed_by_march_31_2026_compliant() {
        let mut i = base_compliant();
        i.eer_filed_in_correct_year = false;
        i.extension_granted_for_2025_filing = true;
        i.eer_filed_by_extension_deadline_march_31_2026 = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::CompliantExtensionGrantedAndFiledByMarch31_2026
        );
    }

    #[test]
    fn tax_block_5_assigned_2025_in_year_2025() {
        let out = check(&base_compliant());
        assert_eq!(out.assigned_filing_year, 2025);
    }

    #[test]
    fn tax_block_3_assigned_2023_or_2033() {
        let mut i = base_compliant();
        i.building_tax_block_last_digit = 3;
        i.current_year = 2025;
        let out = check(&i);
        assert_eq!(out.assigned_filing_year, 2033);
    }

    #[test]
    fn tax_block_5_next_filing_year_2035() {
        let out = check(&base_compliant());
        assert_eq!(out.next_filing_year, 2035);
    }

    #[test]
    fn citations_pin_ll87_admin_code_28_308() {
        let out = check(&base_compliant());
        assert!(out
            .citations
            .iter()
            .any(|c| c.contains("Local Law 87 of 2009")));
        assert!(out.citations.iter().any(|c| c.contains("§ 28-308")));
        assert!(out.citations.iter().any(|c| c.contains("§ 28-308.7")));
    }

    #[test]
    fn citations_pin_ashrae_level_ii_ggbp_eer2() {
        let out = check(&base_compliant());
        assert!(out.citations.iter().any(|c| c.contains("ASHRAE")));
        assert!(out
            .citations
            .iter()
            .any(|c| c.contains("Greener, Greater Buildings Plan")));
        assert!(out.citations.iter().any(|c| c.contains("EER2")));
    }

    #[test]
    fn citations_pin_2025_extension_march_31_2026() {
        let out = check(&base_compliant());
        assert!(out
            .citations
            .iter()
            .any(|c| c.contains("2025 Filing Extension")));
        assert!(out.citations.iter().any(|c| c.contains("March 31, 2026")));
    }

    #[test]
    fn constant_pin_50000_sqft_threshold() {
        assert_eq!(LL87_BUILDING_SIZE_THRESHOLD_SQFT, 50_000);
    }

    #[test]
    fn constant_pin_10_year_reporting_cycle() {
        assert_eq!(LL87_REPORTING_CYCLE_YEARS, 10);
    }

    #[test]
    fn constant_pin_3000_first_year_penalty() {
        assert_eq!(LL87_FIRST_YEAR_CIVIL_PENALTY_CENTS, 300_000);
    }

    #[test]
    fn constant_pin_5000_subsequent_year_penalty() {
        assert_eq!(LL87_SUBSEQUENT_YEAR_CIVIL_PENALTY_CENTS, 500_000);
    }

    #[test]
    fn constant_pin_2025_extension_march_31_2026() {
        assert_eq!(LL87_2025_FILING_EXTENSION_DEADLINE_YEAR, 2026);
        assert_eq!(LL87_2025_FILING_EXTENSION_DEADLINE_MONTH, 3);
        assert_eq!(LL87_2025_FILING_EXTENSION_DEADLINE_DAY, 31);
    }

    #[test]
    fn very_large_continuing_non_submittal_no_overflow() {
        let mut i = base_compliant();
        i.eer_filed_in_correct_year = false;
        i.years_of_continuing_non_submittal = u32::MAX;
        let out = check(&i);
        assert!(out.total_penalty_cents > 0);
    }
}
