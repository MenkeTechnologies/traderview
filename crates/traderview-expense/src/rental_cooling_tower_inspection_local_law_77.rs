//! NYC Local Law 77 of 2015 cooling tower registration, inspection,
//! and Legionella testing compliance for trader-landlords with
//! commercial or large-multifamily NYC inventory.
//!
//! Enacted in August 2015 in response to the South Bronx
//! Legionnaires' disease outbreak (July-August 2015) that killed 8
//! people and infected 138 — traced to contaminated cooling-tower
//! water at the Opera House Hotel and several other buildings in
//! the Mott Haven / Hunts Point area. Local Law 77 is the most
//! aggressive municipal cooling-tower regulation in the United
//! States and the first to require automatic biocide treatment
//! plus quarterly Legionella culture testing for ALL registered
//! cooling towers.
//!
//! Statutory chain:
//!
//! - **NYC Admin Code § 17-194.1** (statutory basis)
//! - **Chapter 8 of the Rules of the City of New York** (operative
//!   detail — registration, MMP, sampling, certification)
//! - **NYS 10 NYCRR Subpart 4-1** (parallel state cooling-tower
//!   regulation, also enacted post-2015 outbreak)
//!
//! **Operative obligations**:
//!
//! 1. Register cooling tower with NYC DOHMH within 30 days of
//!    installation.
//! 2. Develop and implement a written **Maintenance Management Plan
//!    (MMP)** by a Qualified Person.
//! 3. Inspect tower every **90 days** while in operation.
//! 4. Bacteriological sampling for Legionella pneumophila culture +
//!    heterotrophic plate count quarterly (March, June, September,
//!    December).
//! 5. Submit lab results to DOHMH within **5 days** of receipt.
//! 6. File annual certification with DOHMH by **November 1** of
//!    each year.
//!
//! **Qualified Person definition** (Chapter 8 § 8-04): (a) New York
//! State licensed PE or RA; (b) certified water technologist with
//! MPP-development training; (c) environmental consultant with at
//! least 2 years operational experience in water-management
//! planning.
//!
//! **Penalty schedule** (NYC Admin Code § 17-194.1(g)):
//!
//! - First violation: up to **$2,000**.
//! - Second or subsequent violation: up to **$5,000**.
//! - Violation accompanied by or resulting in fatality or serious
//!   injury: up to **$10,000**.
//! - No MPP at all: $1,000.
//! - Incomplete MPP or not on-site: ~$500.
//! - Late or missing annual certification: up to $10,000.
//!
//! **NYC Local Law 159 (2024) upcoming change**: effective May 7,
//! 2026, monthly Legionella testing replaces the quarterly 90-day
//! cycle.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[allow(dead_code)]
pub const LL77_ENACTMENT_YEAR: u32 = 2015;
#[allow(dead_code)]
pub const LL77_INSPECTION_INTERVAL_DAYS: u32 = 90;
#[allow(dead_code)]
pub const LL77_LEGIONELLA_TESTING_INTERVAL_DAYS: u32 = 90;
#[allow(dead_code)]
pub const LL77_LAB_RESULT_SUBMISSION_DAYS: u32 = 5;
#[allow(dead_code)]
pub const LL77_REGISTRATION_DAYS_FROM_INSTALL: u32 = 30;
#[allow(dead_code)]
pub const LL77_ANNUAL_CERTIFICATION_NOV_1_DAY: u32 = 1;
#[allow(dead_code)]
pub const LL77_FIRST_VIOLATION_MAX_PENALTY_CENTS: u64 = 200_000;
#[allow(dead_code)]
pub const LL77_SUBSEQUENT_VIOLATION_MAX_PENALTY_CENTS: u64 = 500_000;
#[allow(dead_code)]
pub const LL77_FATALITY_OR_SERIOUS_INJURY_MAX_PENALTY_CENTS: u64 = 1_000_000;
#[allow(dead_code)]
pub const LL77_NO_MPP_PENALTY_CENTS: u64 = 100_000;
#[allow(dead_code)]
pub const LL77_INCOMPLETE_MPP_PENALTY_CENTS: u64 = 50_000;
#[allow(dead_code)]
pub const LL77_LATE_ANNUAL_CERT_MAX_PENALTY_CENTS: u64 = 1_000_000;
#[allow(dead_code)]
pub const LL159_MONTHLY_TESTING_EFFECTIVE_YEAR: u32 = 2026;
#[allow(dead_code)]
pub const LL77_QUALIFIED_PERSON_ENV_CONSULTANT_MIN_EXPERIENCE_YEARS: u32 = 2;
#[allow(dead_code)]
pub const SOUTH_BRONX_2015_LEGIONNAIRES_DEATHS: u32 = 8;
#[allow(dead_code)]
pub const SOUTH_BRONX_2015_LEGIONNAIRES_CASES: u32 = 138;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicableNoCoolingTower,
    CompliantInspectionAndCertificationOnTime,
    ViolationCoolingTowerNotRegistered,
    ViolationMmpAbsentNoPlanAtAll,
    ViolationMmpIncompleteOrNotOnSite,
    ViolationQuarterlyInspectionMissed,
    ViolationLegionellaTestingMissed,
    ViolationLabResultsNotSubmittedWithin5Days,
    ViolationAnnualCertificationLate,
    ViolationLl159MonthlyTestingRequirementMissedPostMay2026,
    ViolationQualifiedPersonNotEligible,
    AggravatedViolationFatalityOrSeriousInjury,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub has_cooling_tower: bool,
    pub registered_with_dohmh: bool,
    pub mmp_in_place: bool,
    pub mmp_complete_and_on_site: bool,
    pub days_since_last_inspection: u32,
    pub days_since_last_legionella_test: u32,
    pub days_to_submit_lab_results: u32,
    pub annual_certification_filed_by_nov_1: bool,
    pub months_late_annual_certification: u32,
    pub current_year: u32,
    pub qualified_person_meets_definition: bool,
    pub violation_count_history: u32,
    pub fatality_or_serious_injury_from_violation: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Output {
    pub severity: Severity,
    pub compliant: bool,
    pub total_penalty_cents: u64,
    pub monthly_testing_required_post_ll159: bool,
    pub notes: Vec<String>,
    pub citations: Vec<String>,
}

pub type CoolingTowerInspectionLocalLaw77Input = Input;
pub type CoolingTowerInspectionLocalLaw77Output = Output;
pub type CoolingTowerInspectionLocalLaw77Result = Output;

pub fn check(input: &Input) -> Output {
    let mut notes: Vec<String> = Vec::new();
    let citations: Vec<String> = vec![
        "NYC Local Law 77 of 2015 (cooling tower registration + inspection)".to_string(),
        "NYC Admin Code § 17-194.1 (statutory basis)".to_string(),
        "Chapter 8 of Rules of the City of New York (operative regulation)".to_string(),
        "NYS 10 NYCRR Subpart 4-1 (parallel state cooling-tower regulation)".to_string(),
        "NYC Local Law 76 (qualified person definition)".to_string(),
        "NYC Local Law 159 of 2024 (monthly testing eff. May 7, 2026)".to_string(),
        "ASHRAE Standard 188-2018 (Legionellosis: Risk Management)".to_string(),
        "South Bronx Legionnaires' outbreak (July-August 2015) — 8 deaths, 138 cases — precipitating incident for LL 77".to_string(),
        "NYC DOHMH Cooling Towers Maintenance Program and Plan guidance".to_string(),
    ];

    let monthly_testing_required = input.current_year >= LL159_MONTHLY_TESTING_EFFECTIVE_YEAR;

    if !input.has_cooling_tower {
        notes.push("No cooling tower at property; LL 77 inspection/testing regime not triggered.".to_string());
        return Output {
            severity: Severity::NotApplicableNoCoolingTower,
            compliant: true,
            total_penalty_cents: 0,
            monthly_testing_required_post_ll159: monthly_testing_required,
            notes,
            citations,
        };
    }

    if input.fatality_or_serious_injury_from_violation {
        notes.push(format!(
            "Aggravated violation accompanied by or resulting in fatality or serious injury — up to ${} penalty under NYC Admin Code § 17-194.1(g).",
            LL77_FATALITY_OR_SERIOUS_INJURY_MAX_PENALTY_CENTS / 100
        ));
        return Output {
            severity: Severity::AggravatedViolationFatalityOrSeriousInjury,
            compliant: false,
            total_penalty_cents: LL77_FATALITY_OR_SERIOUS_INJURY_MAX_PENALTY_CENTS,
            monthly_testing_required_post_ll159: monthly_testing_required,
            notes,
            citations,
        };
    }

    if !input.registered_with_dohmh {
        notes.push("Cooling tower not registered with DOHMH — per se LL 77 § 17-194.1 violation; registration required within 30 days of installation.".to_string());
        return Output {
            severity: Severity::ViolationCoolingTowerNotRegistered,
            compliant: false,
            total_penalty_cents: subsequent_or_first_penalty(input.violation_count_history),
            monthly_testing_required_post_ll159: monthly_testing_required,
            notes,
            citations,
        };
    }

    if !input.qualified_person_meets_definition {
        notes.push("Person designated as Qualified Person fails Chapter 8 § 8-04 definition (not PE/RA, not certified water technologist, or environmental consultant without 2+ years experience).".to_string());
        return Output {
            severity: Severity::ViolationQualifiedPersonNotEligible,
            compliant: false,
            total_penalty_cents: subsequent_or_first_penalty(input.violation_count_history),
            monthly_testing_required_post_ll159: monthly_testing_required,
            notes,
            citations,
        };
    }

    if !input.mmp_in_place {
        notes.push(format!(
            "No Maintenance Management Plan (MMP) at all — ${} penalty.",
            LL77_NO_MPP_PENALTY_CENTS / 100
        ));
        return Output {
            severity: Severity::ViolationMmpAbsentNoPlanAtAll,
            compliant: false,
            total_penalty_cents: LL77_NO_MPP_PENALTY_CENTS,
            monthly_testing_required_post_ll159: monthly_testing_required,
            notes,
            citations,
        };
    }

    if !input.mmp_complete_and_on_site {
        notes.push(format!(
            "MMP incomplete or not on-site — ${} penalty.",
            LL77_INCOMPLETE_MPP_PENALTY_CENTS / 100
        ));
        return Output {
            severity: Severity::ViolationMmpIncompleteOrNotOnSite,
            compliant: false,
            total_penalty_cents: LL77_INCOMPLETE_MPP_PENALTY_CENTS,
            monthly_testing_required_post_ll159: monthly_testing_required,
            notes,
            citations,
        };
    }

    if input.days_since_last_inspection > LL77_INSPECTION_INTERVAL_DAYS {
        notes.push(format!(
            "Quarterly inspection cycle exceeded: {} days since last inspection > {}-day interval.",
            input.days_since_last_inspection, LL77_INSPECTION_INTERVAL_DAYS
        ));
        return Output {
            severity: Severity::ViolationQuarterlyInspectionMissed,
            compliant: false,
            total_penalty_cents: subsequent_or_first_penalty(input.violation_count_history),
            monthly_testing_required_post_ll159: monthly_testing_required,
            notes,
            citations,
        };
    }

    let testing_interval = if monthly_testing_required {
        30
    } else {
        LL77_LEGIONELLA_TESTING_INTERVAL_DAYS
    };
    if input.days_since_last_legionella_test > testing_interval {
        let severity = if monthly_testing_required {
            Severity::ViolationLl159MonthlyTestingRequirementMissedPostMay2026
        } else {
            Severity::ViolationLegionellaTestingMissed
        };
        notes.push(format!(
            "Legionella testing cycle exceeded: {} days since last test > {}-day interval (LL 159 monthly = {}).",
            input.days_since_last_legionella_test, testing_interval, monthly_testing_required
        ));
        return Output {
            severity,
            compliant: false,
            total_penalty_cents: subsequent_or_first_penalty(input.violation_count_history),
            monthly_testing_required_post_ll159: monthly_testing_required,
            notes,
            citations,
        };
    }

    if input.days_to_submit_lab_results > LL77_LAB_RESULT_SUBMISSION_DAYS {
        notes.push(format!(
            "Lab results not submitted to DOHMH within {}-day window: {} days elapsed.",
            LL77_LAB_RESULT_SUBMISSION_DAYS, input.days_to_submit_lab_results
        ));
        return Output {
            severity: Severity::ViolationLabResultsNotSubmittedWithin5Days,
            compliant: false,
            total_penalty_cents: subsequent_or_first_penalty(input.violation_count_history),
            monthly_testing_required_post_ll159: monthly_testing_required,
            notes,
            citations,
        };
    }

    if !input.annual_certification_filed_by_nov_1 || input.months_late_annual_certification > 0 {
        notes.push(format!(
            "Annual certification not filed by November 1 (or {} months late) — up to ${} penalty.",
            input.months_late_annual_certification,
            LL77_LATE_ANNUAL_CERT_MAX_PENALTY_CENTS / 100
        ));
        return Output {
            severity: Severity::ViolationAnnualCertificationLate,
            compliant: false,
            total_penalty_cents: LL77_LATE_ANNUAL_CERT_MAX_PENALTY_CENTS,
            monthly_testing_required_post_ll159: monthly_testing_required,
            notes,
            citations,
        };
    }

    notes.push(format!(
        "LL 77 compliant: registered cooling tower with MMP in place, qualified person, 90-day inspection, {} testing, lab results submitted timely, annual certification filed by Nov 1.",
        if monthly_testing_required { "monthly Legionella" } else { "quarterly Legionella" }
    ));
    Output {
        severity: Severity::CompliantInspectionAndCertificationOnTime,
        compliant: true,
        total_penalty_cents: 0,
        monthly_testing_required_post_ll159: monthly_testing_required,
        notes,
        citations,
    }
}

fn subsequent_or_first_penalty(violation_count_history: u32) -> u64 {
    if violation_count_history >= 1 {
        LL77_SUBSEQUENT_VIOLATION_MAX_PENALTY_CENTS
    } else {
        LL77_FIRST_VIOLATION_MAX_PENALTY_CENTS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_compliant() -> Input {
        Input {
            has_cooling_tower: true,
            registered_with_dohmh: true,
            mmp_in_place: true,
            mmp_complete_and_on_site: true,
            days_since_last_inspection: 60,
            days_since_last_legionella_test: 60,
            days_to_submit_lab_results: 3,
            annual_certification_filed_by_nov_1: true,
            months_late_annual_certification: 0,
            current_year: 2025,
            qualified_person_meets_definition: true,
            violation_count_history: 0,
            fatality_or_serious_injury_from_violation: false,
        }
    }

    #[test]
    fn fully_compliant_baseline() {
        let out = check(&base_compliant());
        assert_eq!(
            out.severity,
            Severity::CompliantInspectionAndCertificationOnTime
        );
        assert!(out.compliant);
        assert_eq!(out.total_penalty_cents, 0);
    }

    #[test]
    fn no_cooling_tower_not_applicable() {
        let mut i = base_compliant();
        i.has_cooling_tower = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::NotApplicableNoCoolingTower);
    }

    #[test]
    fn fatality_or_serious_injury_aggravated_10000() {
        let mut i = base_compliant();
        i.fatality_or_serious_injury_from_violation = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::AggravatedViolationFatalityOrSeriousInjury
        );
        assert_eq!(out.total_penalty_cents, 1_000_000);
    }

    #[test]
    fn not_registered_first_violation_2000_penalty() {
        let mut i = base_compliant();
        i.registered_with_dohmh = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ViolationCoolingTowerNotRegistered);
        assert_eq!(out.total_penalty_cents, 200_000);
    }

    #[test]
    fn not_registered_subsequent_violation_5000_penalty() {
        let mut i = base_compliant();
        i.registered_with_dohmh = false;
        i.violation_count_history = 1;
        let out = check(&i);
        assert_eq!(out.total_penalty_cents, 500_000);
    }

    #[test]
    fn qualified_person_not_eligible_violation() {
        let mut i = base_compliant();
        i.qualified_person_meets_definition = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ViolationQualifiedPersonNotEligible);
    }

    #[test]
    fn no_mmp_at_all_1000_penalty() {
        let mut i = base_compliant();
        i.mmp_in_place = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ViolationMmpAbsentNoPlanAtAll);
        assert_eq!(out.total_penalty_cents, 100_000);
    }

    #[test]
    fn incomplete_mmp_500_penalty() {
        let mut i = base_compliant();
        i.mmp_complete_and_on_site = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ViolationMmpIncompleteOrNotOnSite);
        assert_eq!(out.total_penalty_cents, 50_000);
    }

    #[test]
    fn quarterly_inspection_missed_91_days_violation() {
        let mut i = base_compliant();
        i.days_since_last_inspection = 91;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ViolationQuarterlyInspectionMissed);
    }

    #[test]
    fn quarterly_inspection_exactly_90_days_compliant() {
        let mut i = base_compliant();
        i.days_since_last_inspection = 90;
        let out = check(&i);
        assert!(out.compliant);
    }

    #[test]
    fn legionella_testing_missed_91_days_violation_pre_ll159() {
        let mut i = base_compliant();
        i.days_since_last_legionella_test = 91;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ViolationLegionellaTestingMissed);
    }

    #[test]
    fn ll159_monthly_testing_post_2026_30_day_violation() {
        let mut i = base_compliant();
        i.current_year = 2026;
        i.days_since_last_legionella_test = 35;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationLl159MonthlyTestingRequirementMissedPostMay2026
        );
        assert!(out.monthly_testing_required_post_ll159);
    }

    #[test]
    fn ll159_2025_still_quarterly_testing_60_days_compliant() {
        let mut i = base_compliant();
        i.current_year = 2025;
        i.days_since_last_legionella_test = 60;
        let out = check(&i);
        assert!(out.compliant);
        assert!(!out.monthly_testing_required_post_ll159);
    }

    #[test]
    fn lab_results_not_submitted_within_5_days_violation() {
        let mut i = base_compliant();
        i.days_to_submit_lab_results = 6;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationLabResultsNotSubmittedWithin5Days
        );
    }

    #[test]
    fn lab_results_at_exactly_5_days_compliant() {
        let mut i = base_compliant();
        i.days_to_submit_lab_results = 5;
        let out = check(&i);
        assert!(out.compliant);
    }

    #[test]
    fn annual_certification_late_10000_penalty() {
        let mut i = base_compliant();
        i.annual_certification_filed_by_nov_1 = false;
        i.months_late_annual_certification = 2;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ViolationAnnualCertificationLate);
        assert_eq!(out.total_penalty_cents, 1_000_000);
    }

    #[test]
    fn citations_pin_ll77_admin_code_chapter_8_nyc() {
        let out = check(&base_compliant());
        assert!(out.citations.iter().any(|c| c.contains("Local Law 77 of 2015")));
        assert!(out.citations.iter().any(|c| c.contains("§ 17-194.1")));
        assert!(out.citations.iter().any(|c| c.contains("Chapter 8")));
        assert!(out.citations.iter().any(|c| c.contains("10 NYCRR Subpart 4-1")));
    }

    #[test]
    fn citations_pin_ll159_monthly_testing_2026() {
        let out = check(&base_compliant());
        assert!(out.citations.iter().any(|c| c.contains("Local Law 159 of 2024")));
        assert!(out.citations.iter().any(|c| c.contains("May 7, 2026")));
    }

    #[test]
    fn citations_pin_south_bronx_2015_outbreak() {
        let out = check(&base_compliant());
        assert!(out.citations.iter().any(|c| c.contains("South Bronx")));
        assert!(out.citations.iter().any(|c| c.contains("8 deaths")));
        assert!(out.citations.iter().any(|c| c.contains("138 cases")));
    }

    #[test]
    fn constant_pin_90_day_inspection() {
        assert_eq!(LL77_INSPECTION_INTERVAL_DAYS, 90);
    }

    #[test]
    fn constant_pin_5_day_lab_result_submission() {
        assert_eq!(LL77_LAB_RESULT_SUBMISSION_DAYS, 5);
    }

    #[test]
    fn constant_pin_2000_first_violation() {
        assert_eq!(LL77_FIRST_VIOLATION_MAX_PENALTY_CENTS, 200_000);
    }

    #[test]
    fn constant_pin_5000_subsequent_violation() {
        assert_eq!(LL77_SUBSEQUENT_VIOLATION_MAX_PENALTY_CENTS, 500_000);
    }

    #[test]
    fn constant_pin_10000_fatality_serious_injury() {
        assert_eq!(LL77_FATALITY_OR_SERIOUS_INJURY_MAX_PENALTY_CENTS, 1_000_000);
    }

    #[test]
    fn constant_pin_ll159_2026_effective_year() {
        assert_eq!(LL159_MONTHLY_TESTING_EFFECTIVE_YEAR, 2026);
    }

    #[test]
    fn constant_pin_south_bronx_8_deaths_138_cases() {
        assert_eq!(SOUTH_BRONX_2015_LEGIONNAIRES_DEATHS, 8);
        assert_eq!(SOUTH_BRONX_2015_LEGIONNAIRES_CASES, 138);
    }
}
