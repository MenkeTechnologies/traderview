//! IRC § 6039 corporate reporting of ISO exercises and ESPP transfers.
//!
//! Companion to `section_422` (Incentive Stock Options — ISO) and
//! `section_423` (Employee Stock Purchase Plan — ESPP). Where § 422
//! and § 423 govern the employee tax treatment of statutory options,
//! § 6039 imposes the CORPORATE reporting obligation that drives
//! employee basis tracking via:
//!
//! - **Form 3921** — one return per ISO exercise (§ 6039(a)(1)).
//! - **Form 3922** — one return per first transfer of legal title of
//!   ESPP-acquired shares where the purchase price was less than the
//!   FMV at grant or was not fixed/determinable at grant
//!   (§ 6039(a)(2)).
//!
//! Deadlines:
//!
//! - Employee statement (Copy B): **January 31** of the year following
//!   the exercise/transfer year (§ 6039(b)).
//! - IRS filing: **February 28** if paper, **March 31** if filed
//!   electronically (per Treas. Reg. § 1.6039-2 + IRS Pub. 1220).
//!
//! Electronic-filing threshold (Treas. Reg. § 301.6011-2 as amended by
//! T.D. 9972, eff. tax-year 2023 / filings made in 2024 and later):
//! **10 or more aggregate information returns** of any qualifying type
//! (W-2, W-2G, 1042-S, 1094 series, 1095-B, 1095-C, 1097-BTC, 1098,
//! 1098-C, 1098-E, 1098-Q, 1098-T, 1099 series, 3921, 3922, 5498
//! series, 8027) triggers mandatory electronic filing.
//!
//! Penalties under § 6721 (information-return-penalty default schedule
//! cross-referenced by § 6039(c)) — 2025 amounts:
//!
//! - Late ≤ 30 days: $60 per return.
//! - Late 31 days through August 1: $120 per return.
//! - After August 1 or complete failure: $310 per return.
//! - Intentional disregard: no maximum penalty under § 6721(e).

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[allow(dead_code)]
pub const EMPLOYEE_COPY_B_DEADLINE_DAY_OF_JANUARY: u32 = 31;
#[allow(dead_code)]
pub const IRS_PAPER_FILING_DEADLINE_DAY_OF_FEBRUARY: u32 = 28;
#[allow(dead_code)]
pub const IRS_ELECTRONIC_FILING_DEADLINE_DAY_OF_MARCH: u32 = 31;
#[allow(dead_code)]
pub const ELECTRONIC_FILING_AGGREGATE_THRESHOLD_RETURNS: u32 = 10;
#[allow(dead_code)]
pub const PENALTY_LATE_WITHIN_30_DAYS_CENTS_PER_FORM: u64 = 6_000;
#[allow(dead_code)]
pub const PENALTY_LATE_31_DAYS_TO_AUGUST_1_CENTS_PER_FORM: u64 = 12_000;
#[allow(dead_code)]
pub const PENALTY_LATE_AFTER_AUGUST_1_OR_COMPLETE_FAILURE_CENTS_PER_FORM: u64 = 31_000;
#[allow(dead_code)]
pub const INTENTIONAL_DISREGARD_MIN_PER_FORM_FLOOR_CENTS: u64 = 63_000;
#[allow(dead_code)]
pub const LATE_30_DAY_BOUNDARY: u32 = 30;
#[allow(dead_code)]
pub const LATE_AUGUST_1_BOUNDARY_DAYS_FROM_FEB_28: u32 = 154;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    ComplianceTimelyFiledWithEmployeeCopyAndIrsCopy,
    ViolationEmployeeCopyBNotProvidedByJan31,
    ViolationLateFiledWithin30Days,
    ViolationLateFiledBetween31DaysAndAugust1,
    ViolationLateFiledAfterAugust1OrCompleteFailure,
    ViolationElectronicFilingRequiredButPaperFiled,
    ViolationIntentionalDisregardNoMaximumPenalty,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub iso_exercises_count: u32,
    pub espp_first_transfers_count: u32,
    pub total_aggregate_info_returns_filed: u32,
    pub employee_copy_b_provided_by_jan_31: bool,
    pub irs_copy_a_filed: bool,
    pub filed_electronically: bool,
    pub days_late_filing: u32,
    pub intentional_disregard: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Output {
    pub severity: Severity,
    pub total_forms_required: u32,
    pub electronic_filing_required: bool,
    pub per_form_penalty_cents: u64,
    pub total_penalty_cents: u64,
    pub notes: Vec<String>,
    pub citations: Vec<String>,
}

pub type Section6039Input = Input;
pub type Section6039Output = Output;
pub type Section6039Result = Output;

pub fn check(input: &Input) -> Output {
    let mut notes: Vec<String> = Vec::new();
    let citations: Vec<String> = vec![
        "IRC § 6039(a)(1) (Form 3921 — ISO exercise return)".to_string(),
        "IRC § 6039(a)(2) (Form 3922 — ESPP transfer return)".to_string(),
        "IRC § 6039(b) (employee information statement)".to_string(),
        "IRC § 6039(c) (penalty cross-reference to § 6721)".to_string(),
        "IRC § 6721 (failure to file information return penalty schedule)".to_string(),
        "IRC § 6721(e) (intentional disregard — no maximum penalty)".to_string(),
        "Treas. Reg. § 1.6039-1 (Form 3921 reporting)".to_string(),
        "Treas. Reg. § 1.6039-2 (Form 3922 reporting)".to_string(),
        "Treas. Reg. § 301.6011-2 (electronic filing threshold)".to_string(),
        "T.D. 9972 (10-return aggregate e-file threshold, eff. filings in 2024)".to_string(),
        "IRS Pub. 1220 (electronic information-return specifications)".to_string(),
    ];

    let total_forms = input
        .iso_exercises_count
        .saturating_add(input.espp_first_transfers_count);

    if total_forms == 0 {
        notes.push("No ISO exercises and no ESPP first transfers — § 6039 reporting not triggered for this period.".to_string());
        return Output {
            severity: Severity::NotApplicable,
            total_forms_required: 0,
            electronic_filing_required: false,
            per_form_penalty_cents: 0,
            total_penalty_cents: 0,
            notes,
            citations,
        };
    }

    let electronic_required =
        input.total_aggregate_info_returns_filed >= ELECTRONIC_FILING_AGGREGATE_THRESHOLD_RETURNS;

    if input.intentional_disregard {
        let per_form = INTENTIONAL_DISREGARD_MIN_PER_FORM_FLOOR_CENTS;
        let total = per_form.saturating_mul(total_forms as u64);
        notes.push(format!(
            "Intentional disregard under § 6721(e): no maximum penalty; per-form floor ${}/form × {} forms = ${}+ minimum, IRS may assess higher with no cap.",
            per_form / 100,
            total_forms,
            total / 100
        ));
        return Output {
            severity: Severity::ViolationIntentionalDisregardNoMaximumPenalty,
            total_forms_required: total_forms,
            electronic_filing_required: electronic_required,
            per_form_penalty_cents: per_form,
            total_penalty_cents: total,
            notes,
            citations,
        };
    }

    if electronic_required && !input.filed_electronically && input.irs_copy_a_filed {
        notes.push(format!(
            "Aggregate info-return count {} ≥ {} triggers mandatory electronic filing per Treas. Reg. § 301.6011-2 + T.D. 9972; paper filing is a per se § 6721 failure-to-file in the required manner.",
            input.total_aggregate_info_returns_filed,
            ELECTRONIC_FILING_AGGREGATE_THRESHOLD_RETURNS
        ));
        let per_form = PENALTY_LATE_AFTER_AUGUST_1_OR_COMPLETE_FAILURE_CENTS_PER_FORM;
        let total = per_form.saturating_mul(total_forms as u64);
        return Output {
            severity: Severity::ViolationElectronicFilingRequiredButPaperFiled,
            total_forms_required: total_forms,
            electronic_filing_required: electronic_required,
            per_form_penalty_cents: per_form,
            total_penalty_cents: total,
            notes,
            citations,
        };
    }

    if !input.employee_copy_b_provided_by_jan_31 {
        notes.push(format!(
            "Employee Copy B not provided by January {}: per se § 6722 employee-statement failure.",
            EMPLOYEE_COPY_B_DEADLINE_DAY_OF_JANUARY
        ));
        let per_form = PENALTY_LATE_WITHIN_30_DAYS_CENTS_PER_FORM;
        let total = per_form.saturating_mul(total_forms as u64);
        return Output {
            severity: Severity::ViolationEmployeeCopyBNotProvidedByJan31,
            total_forms_required: total_forms,
            electronic_filing_required: electronic_required,
            per_form_penalty_cents: per_form,
            total_penalty_cents: total,
            notes,
            citations,
        };
    }

    if !input.irs_copy_a_filed {
        notes.push(
            "Complete failure to file IRS Copy A: § 6721 maximum-rate penalty applies.".to_string(),
        );
        let per_form = PENALTY_LATE_AFTER_AUGUST_1_OR_COMPLETE_FAILURE_CENTS_PER_FORM;
        let total = per_form.saturating_mul(total_forms as u64);
        return Output {
            severity: Severity::ViolationLateFiledAfterAugust1OrCompleteFailure,
            total_forms_required: total_forms,
            electronic_filing_required: electronic_required,
            per_form_penalty_cents: per_form,
            total_penalty_cents: total,
            notes,
            citations,
        };
    }

    if input.days_late_filing == 0 {
        notes.push(format!(
            "Timely § 6039 compliance: Form 3921 ({} ISO exercises) + Form 3922 ({} ESPP transfers) = {} forms filed on time with employee Copy B by Jan {} and IRS Copy A by {}.",
            input.iso_exercises_count,
            input.espp_first_transfers_count,
            total_forms,
            EMPLOYEE_COPY_B_DEADLINE_DAY_OF_JANUARY,
            if input.filed_electronically { "Mar 31 (electronic)" } else { "Feb 28 (paper)" }
        ));
        return Output {
            severity: Severity::ComplianceTimelyFiledWithEmployeeCopyAndIrsCopy,
            total_forms_required: total_forms,
            electronic_filing_required: electronic_required,
            per_form_penalty_cents: 0,
            total_penalty_cents: 0,
            notes,
            citations,
        };
    }

    let (severity, per_form) = if input.days_late_filing <= LATE_30_DAY_BOUNDARY {
        (
            Severity::ViolationLateFiledWithin30Days,
            PENALTY_LATE_WITHIN_30_DAYS_CENTS_PER_FORM,
        )
    } else if input.days_late_filing <= LATE_AUGUST_1_BOUNDARY_DAYS_FROM_FEB_28 {
        (
            Severity::ViolationLateFiledBetween31DaysAndAugust1,
            PENALTY_LATE_31_DAYS_TO_AUGUST_1_CENTS_PER_FORM,
        )
    } else {
        (
            Severity::ViolationLateFiledAfterAugust1OrCompleteFailure,
            PENALTY_LATE_AFTER_AUGUST_1_OR_COMPLETE_FAILURE_CENTS_PER_FORM,
        )
    };

    let total = per_form.saturating_mul(total_forms as u64);
    notes.push(format!(
        "Late filing {} days past deadline: ${}/form × {} forms = ${} total § 6721 penalty.",
        input.days_late_filing,
        per_form / 100,
        total_forms,
        total / 100
    ));

    Output {
        severity,
        total_forms_required: total_forms,
        electronic_filing_required: electronic_required,
        per_form_penalty_cents: per_form,
        total_penalty_cents: total,
        notes,
        citations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_timely() -> Input {
        Input {
            iso_exercises_count: 3,
            espp_first_transfers_count: 5,
            total_aggregate_info_returns_filed: 100,
            employee_copy_b_provided_by_jan_31: true,
            irs_copy_a_filed: true,
            filed_electronically: true,
            days_late_filing: 0,
            intentional_disregard: false,
        }
    }

    #[test]
    fn timely_filing_with_e_file_is_compliant() {
        let out = check(&base_timely());
        assert_eq!(
            out.severity,
            Severity::ComplianceTimelyFiledWithEmployeeCopyAndIrsCopy
        );
        assert_eq!(out.total_forms_required, 8);
        assert!(out.electronic_filing_required);
        assert_eq!(out.total_penalty_cents, 0);
    }

    #[test]
    fn no_iso_no_espp_is_not_applicable() {
        let mut i = base_timely();
        i.iso_exercises_count = 0;
        i.espp_first_transfers_count = 0;
        let out = check(&i);
        assert_eq!(out.severity, Severity::NotApplicable);
        assert_eq!(out.total_forms_required, 0);
    }

    #[test]
    fn late_within_30_days_60_per_form() {
        let mut i = base_timely();
        i.days_late_filing = 15;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ViolationLateFiledWithin30Days);
        assert_eq!(out.per_form_penalty_cents, 6_000);
        assert_eq!(out.total_penalty_cents, 48_000);
    }

    #[test]
    fn late_31_to_aug_1_120_per_form() {
        let mut i = base_timely();
        i.days_late_filing = 60;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationLateFiledBetween31DaysAndAugust1
        );
        assert_eq!(out.per_form_penalty_cents, 12_000);
    }

    #[test]
    fn late_after_aug_1_310_per_form() {
        let mut i = base_timely();
        i.days_late_filing = 200;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationLateFiledAfterAugust1OrCompleteFailure
        );
        assert_eq!(out.per_form_penalty_cents, 31_000);
    }

    #[test]
    fn complete_failure_to_file_max_rate() {
        let mut i = base_timely();
        i.irs_copy_a_filed = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationLateFiledAfterAugust1OrCompleteFailure
        );
        assert_eq!(out.per_form_penalty_cents, 31_000);
    }

    #[test]
    fn employee_copy_b_not_provided_is_per_se_violation() {
        let mut i = base_timely();
        i.employee_copy_b_provided_by_jan_31 = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationEmployeeCopyBNotProvidedByJan31
        );
        assert!(out.notes.iter().any(|n| n.contains("January 31")));
    }

    #[test]
    fn aggregate_threshold_10_or_more_forces_electronic() {
        let mut i = base_timely();
        i.total_aggregate_info_returns_filed = 10;
        i.filed_electronically = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationElectronicFilingRequiredButPaperFiled
        );
        assert!(out.electronic_filing_required);
    }

    #[test]
    fn aggregate_under_10_paper_filing_acceptable() {
        let mut i = base_timely();
        i.total_aggregate_info_returns_filed = 9;
        i.filed_electronically = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ComplianceTimelyFiledWithEmployeeCopyAndIrsCopy
        );
        assert!(!out.electronic_filing_required);
    }

    #[test]
    fn aggregate_threshold_boundary_9_returns_not_electronic_required() {
        let mut i = base_timely();
        i.total_aggregate_info_returns_filed = 9;
        let out = check(&i);
        assert!(!out.electronic_filing_required);
    }

    #[test]
    fn intentional_disregard_no_maximum_penalty() {
        let mut i = base_timely();
        i.intentional_disregard = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationIntentionalDisregardNoMaximumPenalty
        );
        assert_eq!(
            out.per_form_penalty_cents,
            INTENTIONAL_DISREGARD_MIN_PER_FORM_FLOOR_CENTS
        );
        assert!(out.notes.iter().any(|n| n.contains("no maximum")));
    }

    #[test]
    fn late_30_day_boundary_within_30_window() {
        let mut i = base_timely();
        i.days_late_filing = 30;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ViolationLateFiledWithin30Days);
    }

    #[test]
    fn late_31_day_boundary_steps_to_120_per_form() {
        let mut i = base_timely();
        i.days_late_filing = 31;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationLateFiledBetween31DaysAndAugust1
        );
        assert_eq!(out.per_form_penalty_cents, 12_000);
    }

    #[test]
    fn citations_pin_section_6039_subsections() {
        let out = check(&base_timely());
        assert!(out.citations.iter().any(|c| c.contains("§ 6039(a)(1)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 6039(a)(2)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 6039(b)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 6039(c)")));
    }

    #[test]
    fn citations_pin_6721_penalty_schedule_and_e_file_td_9972() {
        let out = check(&base_timely());
        assert!(out.citations.iter().any(|c| c.contains("§ 6721")));
        assert!(out.citations.iter().any(|c| c.contains("§ 6721(e)")));
        assert!(out.citations.iter().any(|c| c.contains("T.D. 9972")));
    }

    #[test]
    fn citations_pin_treas_reg_1_6039_1_and_2() {
        let out = check(&base_timely());
        assert!(out.citations.iter().any(|c| c.contains("§ 1.6039-1")));
        assert!(out.citations.iter().any(|c| c.contains("§ 1.6039-2")));
    }

    #[test]
    fn constant_pin_electronic_filing_threshold_10_returns() {
        assert_eq!(ELECTRONIC_FILING_AGGREGATE_THRESHOLD_RETURNS, 10);
    }

    #[test]
    fn constant_pin_employee_copy_b_jan_31_deadline() {
        assert_eq!(EMPLOYEE_COPY_B_DEADLINE_DAY_OF_JANUARY, 31);
    }

    #[test]
    fn constant_pin_paper_irs_feb_28_deadline() {
        assert_eq!(IRS_PAPER_FILING_DEADLINE_DAY_OF_FEBRUARY, 28);
    }

    #[test]
    fn constant_pin_electronic_irs_mar_31_deadline() {
        assert_eq!(IRS_ELECTRONIC_FILING_DEADLINE_DAY_OF_MARCH, 31);
    }

    #[test]
    fn constant_pin_60_per_form_late_within_30_days() {
        assert_eq!(PENALTY_LATE_WITHIN_30_DAYS_CENTS_PER_FORM, 6_000);
    }

    #[test]
    fn constant_pin_120_per_form_late_31_days_to_aug_1() {
        assert_eq!(PENALTY_LATE_31_DAYS_TO_AUGUST_1_CENTS_PER_FORM, 12_000);
    }

    #[test]
    fn constant_pin_310_per_form_after_aug_1_or_no_file() {
        assert_eq!(
            PENALTY_LATE_AFTER_AUGUST_1_OR_COMPLETE_FAILURE_CENTS_PER_FORM,
            31_000
        );
    }

    #[test]
    fn very_large_form_count_saturating_does_not_overflow() {
        let mut i = base_timely();
        i.iso_exercises_count = u32::MAX;
        i.days_late_filing = 200;
        let out = check(&i);
        assert_eq!(out.total_forms_required, u32::MAX);
        assert!(out.total_penalty_cents > 0);
    }
}
