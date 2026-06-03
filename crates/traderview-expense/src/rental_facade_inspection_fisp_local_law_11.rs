//! NYC Facade Inspection Safety Program (FISP) / Local Law 11 of
//! 1998 compliance for trader-landlords with NYC high-rise inventory.
//!
//! Statutory chain:
//!
//! - **Local Law 10 of 1980** — original NYC facade inspection law
//!   enacted after the May 16, 1979 facade-collapse fatality at 115
//!   Madison Avenue (Grace Gold killed by falling masonry).
//! - **Local Law 11 of 1998** — current Facade Inspection Safety
//!   Program, codified at 1 RCNY 103-04, expanding scope to include
//!   appurtenances and parapets.
//! - **Local Law 38 of 2007** — instituted the 5-year inspection
//!   cycle (formerly 5-year cycle had been variable subcycles).
//!
//! Operative rule (1 RCNY 103-04): exterior walls of ALL buildings
//! greater than 6 stories (i.e., 7+ stories) must be inspected by a
//! **Qualified Exterior Wall Inspector (QEWI)** every 5 years and a
//! technical facade report filed with NYC Department of Buildings
//! (DOB). Single-family homes and buildings 6 stories or fewer are
//! exempt.
//!
//! **QEWI qualification** (1 RCNY 103-04(c)(1)): must be a New York
//! State licensed Professional Engineer (PE) or Registered Architect
//! (RA) with at least **7 years** of relevant experience specifically
//! with facades over 6 stories, AND must complete a separate DOB
//! approval process before filing FISP reports.
//!
//! Three reporting classifications:
//!
//! - **SAFE** — no work required; clean compliance.
//! - **SWARMP** — Safe With a Repair and Maintenance Program; facade
//!   safe at time of inspection but requires repairs/maintenance
//!   within the next 5-year cycle to prevent deterioration into an
//!   unsafe condition.
//! - **UNSAFE** — must be repaired within **90 days**; QEWI must
//!   notify owner and recommend sidewalk shed or other public-
//!   protection measures; owner must immediately erect sidewalk shed
//!   pending repair.
//!
//! Current cycle: **Cycle 10** commenced February 21, 2025 — runs
//! through February 21, 2030 with subcycles 10A/10B/10C (each
//! covering 2-year filing windows).
//!
//! Penalties (DOB administrative tribunal):
//!
//! - Late initial report filing: **$1,000 per month** until filed.
//! - Failure to file: **$5,000 per year**.
//! - Failure to correct SWARMP conditions before next cycle:
//!   **$2,000** per condition.
//! - Failure to address unsafe conditions: additional civil penalties
//!   under NYC Admin Code § 28-301.1.1 and potential criminal
//!   referral if a fatality results.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[allow(dead_code)]
pub const LL11_BUILDING_STORY_THRESHOLD_OVER: u32 = 6;
#[allow(dead_code)]
pub const LL11_INSPECTION_CYCLE_YEARS: u32 = 5;
#[allow(dead_code)]
pub const LL11_CYCLE_10_START_YEAR: u32 = 2025;
#[allow(dead_code)]
pub const LL11_CYCLE_10_END_YEAR: u32 = 2030;
#[allow(dead_code)]
pub const LL11_UNSAFE_REPAIR_DAYS: u32 = 90;
#[allow(dead_code)]
pub const LL11_LATE_FILING_PENALTY_CENTS_PER_MONTH: u64 = 100_000;
#[allow(dead_code)]
pub const LL11_FAILURE_TO_FILE_PENALTY_CENTS_PER_YEAR: u64 = 500_000;
#[allow(dead_code)]
pub const LL11_SWARMP_NOT_CORRECTED_PENALTY_CENTS: u64 = 200_000;
#[allow(dead_code)]
pub const QEWI_MIN_EXPERIENCE_YEARS: u32 = 7;
#[allow(dead_code)]
pub const LL11_ENACTMENT_YEAR: u32 = 1998;
#[allow(dead_code)]
pub const LL10_ORIGINAL_ENACTMENT_YEAR: u32 = 1980;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReportClassification {
    Safe,
    Swarmp,
    Unsafe,
    NotFiled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    ExemptBuildingSixStoriesOrFewer,
    CompliantSafeFiledOnTime,
    CompliantSwarmpFiledOnTimeRepairsInProgress,
    ViolationSwarmpRepairsNotMadeByNextCycle,
    ViolationUnsafeNotRepairedWithin90Days,
    ViolationUnsafeNoSidewalkShedPublicSafetyExposure,
    ViolationLateInitialReportFiling,
    ViolationFailureToFile,
    ViolationQewiNotQualified,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub story_count: u32,
    pub current_year: u32,
    pub report_classification: ReportClassification,
    pub swarmp_repairs_completed_by_next_cycle: bool,
    pub unsafe_repair_days_elapsed: u32,
    pub sidewalk_shed_in_place_if_unsafe: bool,
    pub qewi_is_licensed_pe_or_ra: bool,
    pub qewi_experience_years: u32,
    pub qewi_dob_approval: bool,
    pub months_late_initial_report_filing: u32,
    pub years_failure_to_file: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Output {
    pub severity: Severity,
    pub compliant: bool,
    pub total_penalty_cents: u64,
    pub inspection_required: bool,
    pub current_cycle_number: u32,
    pub notes: Vec<String>,
    pub citations: Vec<String>,
}

pub type FacadeInspectionFispInput = Input;
pub type FacadeInspectionFispOutput = Output;
pub type FacadeInspectionFispResult = Output;

pub fn check(input: &Input) -> Output {
    let mut notes: Vec<String> = Vec::new();
    let citations: Vec<String> = vec![
        "NYC Local Law 11 of 1998 (Facade Inspection Safety Program)".to_string(),
        "NYC Local Law 10 of 1980 (original facade inspection law)".to_string(),
        "NYC Local Law 38 of 2007 (5-year inspection cycle)".to_string(),
        "1 RCNY § 103-04 (FISP regulation)".to_string(),
        "1 RCNY § 103-04(c)(1) (QEWI qualification)".to_string(),
        "NYC Admin Code § 28-301.1.1 (facade compliance)".to_string(),
        "NYC DOB Facade & Local Law page".to_string(),
        "Grace Gold incident (May 16, 1979) — 115 Madison Avenue — original impetus for LL 10/1980".to_string(),
        "Erica Tishman incident (Dec. 17, 2019) — 729 7th Avenue — enforcement intensification".to_string(),
    ];

    if input.story_count <= LL11_BUILDING_STORY_THRESHOLD_OVER {
        notes.push(format!(
            "Building {} stories ≤ {} threshold — exempt from LL 11 FISP.",
            input.story_count, LL11_BUILDING_STORY_THRESHOLD_OVER
        ));
        return Output {
            severity: Severity::ExemptBuildingSixStoriesOrFewer,
            compliant: true,
            total_penalty_cents: 0,
            inspection_required: false,
            current_cycle_number: 10,
            notes,
            citations,
        };
    }

    if !input.qewi_is_licensed_pe_or_ra
        || input.qewi_experience_years < QEWI_MIN_EXPERIENCE_YEARS
        || !input.qewi_dob_approval
    {
        notes.push(format!(
            "QEWI fails 1 RCNY § 103-04(c)(1) qualification: PE/RA = {}, {} years experience (< {} required), DOB approval = {}.",
            input.qewi_is_licensed_pe_or_ra,
            input.qewi_experience_years,
            QEWI_MIN_EXPERIENCE_YEARS,
            input.qewi_dob_approval
        ));
        return Output {
            severity: Severity::ViolationQewiNotQualified,
            compliant: false,
            total_penalty_cents: 0,
            inspection_required: true,
            current_cycle_number: 10,
            notes,
            citations,
        };
    }

    if input.years_failure_to_file > 0 {
        let penalty = LL11_FAILURE_TO_FILE_PENALTY_CENTS_PER_YEAR
            .saturating_mul(input.years_failure_to_file as u64);
        notes.push(format!(
            "Failure to file FISP report for {} year(s): ${} total DOB civil penalty.",
            input.years_failure_to_file,
            penalty / 100
        ));
        return Output {
            severity: Severity::ViolationFailureToFile,
            compliant: false,
            total_penalty_cents: penalty,
            inspection_required: true,
            current_cycle_number: 10,
            notes,
            citations,
        };
    }

    if input.months_late_initial_report_filing > 0 {
        let penalty = LL11_LATE_FILING_PENALTY_CENTS_PER_MONTH
            .saturating_mul(input.months_late_initial_report_filing as u64);
        notes.push(format!(
            "Late initial report filing {} month(s): ${} total DOB civil penalty at ${}/month.",
            input.months_late_initial_report_filing,
            penalty / 100,
            LL11_LATE_FILING_PENALTY_CENTS_PER_MONTH / 100
        ));
        return Output {
            severity: Severity::ViolationLateInitialReportFiling,
            compliant: false,
            total_penalty_cents: penalty,
            inspection_required: true,
            current_cycle_number: 10,
            notes,
            citations,
        };
    }

    match input.report_classification {
        ReportClassification::Safe => {
            notes.push("FISP SAFE classification filed on time; clean compliance.".to_string());
            Output {
                severity: Severity::CompliantSafeFiledOnTime,
                compliant: true,
                total_penalty_cents: 0,
                inspection_required: true,
                current_cycle_number: 10,
                notes,
                citations,
            }
        }
        ReportClassification::Swarmp => {
            if input.swarmp_repairs_completed_by_next_cycle {
                notes.push("FISP SWARMP filed on time; repairs in progress and on track for completion before next cycle.".to_string());
                Output {
                    severity: Severity::CompliantSwarmpFiledOnTimeRepairsInProgress,
                    compliant: true,
                    total_penalty_cents: 0,
                    inspection_required: true,
                    current_cycle_number: 10,
                    notes,
                    citations,
                }
            } else {
                notes.push(format!(
                    "SWARMP repairs not completed by next cycle: ${} per uncorrected condition civil penalty.",
                    LL11_SWARMP_NOT_CORRECTED_PENALTY_CENTS / 100
                ));
                Output {
                    severity: Severity::ViolationSwarmpRepairsNotMadeByNextCycle,
                    compliant: false,
                    total_penalty_cents: LL11_SWARMP_NOT_CORRECTED_PENALTY_CENTS,
                    inspection_required: true,
                    current_cycle_number: 10,
                    notes,
                    citations,
                }
            }
        }
        ReportClassification::Unsafe => {
            if !input.sidewalk_shed_in_place_if_unsafe {
                notes.push("UNSAFE classification + no sidewalk shed = public-safety exposure; immediate sidewalk-shed erection required + DOB unsafe-building civil penalty + criminal referral risk if fatality.".to_string());
                return Output {
                    severity: Severity::ViolationUnsafeNoSidewalkShedPublicSafetyExposure,
                    compliant: false,
                    total_penalty_cents: 0,
                    inspection_required: true,
                    current_cycle_number: 10,
                    notes,
                    citations,
                };
            }
            if input.unsafe_repair_days_elapsed > LL11_UNSAFE_REPAIR_DAYS {
                notes.push(format!(
                    "UNSAFE condition not repaired within {} days ({} elapsed); DOB civil penalty + unsafe-building violation.",
                    LL11_UNSAFE_REPAIR_DAYS, input.unsafe_repair_days_elapsed
                ));
                return Output {
                    severity: Severity::ViolationUnsafeNotRepairedWithin90Days,
                    compliant: false,
                    total_penalty_cents: 0,
                    inspection_required: true,
                    current_cycle_number: 10,
                    notes,
                    citations,
                };
            }
            notes.push(format!(
                "UNSAFE classification + sidewalk shed in place + within {}-day repair window. Conditional compliance pending repair completion.",
                LL11_UNSAFE_REPAIR_DAYS
            ));
            Output {
                severity: Severity::CompliantSwarmpFiledOnTimeRepairsInProgress,
                compliant: true,
                total_penalty_cents: 0,
                inspection_required: true,
                current_cycle_number: 10,
                notes,
                citations,
            }
        }
        ReportClassification::NotFiled => {
            notes.push("No FISP report filed at all — per se failure-to-file violation.".to_string());
            Output {
                severity: Severity::ViolationFailureToFile,
                compliant: false,
                total_penalty_cents: LL11_FAILURE_TO_FILE_PENALTY_CENTS_PER_YEAR,
                inspection_required: true,
                current_cycle_number: 10,
                notes,
                citations,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_compliant_safe() -> Input {
        Input {
            story_count: 10,
            current_year: 2026,
            report_classification: ReportClassification::Safe,
            swarmp_repairs_completed_by_next_cycle: false,
            unsafe_repair_days_elapsed: 0,
            sidewalk_shed_in_place_if_unsafe: false,
            qewi_is_licensed_pe_or_ra: true,
            qewi_experience_years: 10,
            qewi_dob_approval: true,
            months_late_initial_report_filing: 0,
            years_failure_to_file: 0,
        }
    }

    #[test]
    fn ten_story_safe_filed_on_time_compliant() {
        let out = check(&base_compliant_safe());
        assert_eq!(out.severity, Severity::CompliantSafeFiledOnTime);
        assert!(out.compliant);
        assert_eq!(out.total_penalty_cents, 0);
    }

    #[test]
    fn six_story_building_exempt_at_threshold() {
        let mut i = base_compliant_safe();
        i.story_count = 6;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExemptBuildingSixStoriesOrFewer);
    }

    #[test]
    fn seven_story_building_not_exempt() {
        let mut i = base_compliant_safe();
        i.story_count = 7;
        let out = check(&i);
        assert_eq!(out.severity, Severity::CompliantSafeFiledOnTime);
    }

    #[test]
    fn swarmp_with_repairs_in_progress_compliant() {
        let mut i = base_compliant_safe();
        i.report_classification = ReportClassification::Swarmp;
        i.swarmp_repairs_completed_by_next_cycle = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::CompliantSwarmpFiledOnTimeRepairsInProgress
        );
    }

    #[test]
    fn swarmp_repairs_not_completed_violation() {
        let mut i = base_compliant_safe();
        i.report_classification = ReportClassification::Swarmp;
        i.swarmp_repairs_completed_by_next_cycle = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationSwarmpRepairsNotMadeByNextCycle
        );
        assert_eq!(out.total_penalty_cents, 200_000);
    }

    #[test]
    fn unsafe_no_sidewalk_shed_immediate_violation() {
        let mut i = base_compliant_safe();
        i.report_classification = ReportClassification::Unsafe;
        i.sidewalk_shed_in_place_if_unsafe = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationUnsafeNoSidewalkShedPublicSafetyExposure
        );
    }

    #[test]
    fn unsafe_with_shed_within_90_days_conditional_compliance() {
        let mut i = base_compliant_safe();
        i.report_classification = ReportClassification::Unsafe;
        i.sidewalk_shed_in_place_if_unsafe = true;
        i.unsafe_repair_days_elapsed = 30;
        let out = check(&i);
        assert!(out.compliant);
    }

    #[test]
    fn unsafe_with_shed_past_90_days_violation() {
        let mut i = base_compliant_safe();
        i.report_classification = ReportClassification::Unsafe;
        i.sidewalk_shed_in_place_if_unsafe = true;
        i.unsafe_repair_days_elapsed = 100;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationUnsafeNotRepairedWithin90Days
        );
    }

    #[test]
    fn unsafe_90_day_boundary_at_exactly_90_compliant() {
        let mut i = base_compliant_safe();
        i.report_classification = ReportClassification::Unsafe;
        i.sidewalk_shed_in_place_if_unsafe = true;
        i.unsafe_repair_days_elapsed = 90;
        let out = check(&i);
        assert!(out.compliant);
    }

    #[test]
    fn unsafe_91_days_just_over_threshold_violation() {
        let mut i = base_compliant_safe();
        i.report_classification = ReportClassification::Unsafe;
        i.sidewalk_shed_in_place_if_unsafe = true;
        i.unsafe_repair_days_elapsed = 91;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationUnsafeNotRepairedWithin90Days
        );
    }

    #[test]
    fn late_initial_report_three_months_penalty_3000() {
        let mut i = base_compliant_safe();
        i.months_late_initial_report_filing = 3;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ViolationLateInitialReportFiling);
        assert_eq!(out.total_penalty_cents, 300_000);
    }

    #[test]
    fn failure_to_file_two_years_penalty_10000() {
        let mut i = base_compliant_safe();
        i.years_failure_to_file = 2;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ViolationFailureToFile);
        assert_eq!(out.total_penalty_cents, 1_000_000);
    }

    #[test]
    fn not_filed_classification_per_se_violation() {
        let mut i = base_compliant_safe();
        i.report_classification = ReportClassification::NotFiled;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ViolationFailureToFile);
    }

    #[test]
    fn qewi_not_pe_or_ra_violation() {
        let mut i = base_compliant_safe();
        i.qewi_is_licensed_pe_or_ra = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ViolationQewiNotQualified);
    }

    #[test]
    fn qewi_under_7_year_experience_violation() {
        let mut i = base_compliant_safe();
        i.qewi_experience_years = 5;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ViolationQewiNotQualified);
    }

    #[test]
    fn qewi_exactly_7_years_qualifies() {
        let mut i = base_compliant_safe();
        i.qewi_experience_years = 7;
        let out = check(&i);
        assert_eq!(out.severity, Severity::CompliantSafeFiledOnTime);
    }

    #[test]
    fn qewi_no_dob_approval_violation() {
        let mut i = base_compliant_safe();
        i.qewi_dob_approval = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ViolationQewiNotQualified);
    }

    #[test]
    fn citations_pin_ll11_1998_ll10_1980_ll38_2007() {
        let out = check(&base_compliant_safe());
        assert!(out.citations.iter().any(|c| c.contains("Local Law 11 of 1998")));
        assert!(out.citations.iter().any(|c| c.contains("Local Law 10 of 1980")));
        assert!(out.citations.iter().any(|c| c.contains("Local Law 38 of 2007")));
    }

    #[test]
    fn citations_pin_rcny_103_04_and_admin_code_28_301() {
        let out = check(&base_compliant_safe());
        assert!(out.citations.iter().any(|c| c.contains("1 RCNY § 103-04")));
        assert!(out.citations.iter().any(|c| c.contains("§ 28-301.1.1")));
    }

    #[test]
    fn citations_pin_grace_gold_and_erica_tishman_incidents() {
        let out = check(&base_compliant_safe());
        assert!(out.citations.iter().any(|c| c.contains("Grace Gold")));
        assert!(out.citations.iter().any(|c| c.contains("Erica Tishman")));
    }

    #[test]
    fn constant_pin_6_story_threshold() {
        assert_eq!(LL11_BUILDING_STORY_THRESHOLD_OVER, 6);
    }

    #[test]
    fn constant_pin_5_year_cycle() {
        assert_eq!(LL11_INSPECTION_CYCLE_YEARS, 5);
    }

    #[test]
    fn constant_pin_90_day_unsafe_repair_window() {
        assert_eq!(LL11_UNSAFE_REPAIR_DAYS, 90);
    }

    #[test]
    fn constant_pin_qewi_7_year_experience() {
        assert_eq!(QEWI_MIN_EXPERIENCE_YEARS, 7);
    }

    #[test]
    fn constant_pin_1000_per_month_late_filing() {
        assert_eq!(LL11_LATE_FILING_PENALTY_CENTS_PER_MONTH, 100_000);
    }

    #[test]
    fn constant_pin_5000_per_year_failure_to_file() {
        assert_eq!(LL11_FAILURE_TO_FILE_PENALTY_CENTS_PER_YEAR, 500_000);
    }

    #[test]
    fn constant_pin_2000_swarmp_uncorrected() {
        assert_eq!(LL11_SWARMP_NOT_CORRECTED_PENALTY_CENTS, 200_000);
    }

    #[test]
    fn very_large_months_late_saturating_no_overflow() {
        let mut i = base_compliant_safe();
        i.months_late_initial_report_filing = u32::MAX;
        let out = check(&i);
        assert!(out.total_penalty_cents > 0);
    }
}
