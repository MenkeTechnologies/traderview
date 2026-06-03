//! IRC § 6723 — Failure to Comply with Other Information
//! Reporting Requirements / $50-Per-Failure Catch-All Module.
//!
//! Pure-compute check for IRC § 6723 penalty for failure to
//! comply with **OTHER specified information reporting
//! requirements** not covered by § 6721 (failure to file
//! information return — iter 658) or § 6722 (failure to
//! furnish payee statement — iter 660). § 6723 is the simpler
//! catch-all: **$50 per failure** with **$100,000 annual
//! maximum**, NO TIER STRUCTURE, NO INFLATION ADJUSTMENT, NO
//! SMALL BUSINESS EXCEPTION, NO SAFE HARBOR. Examples: failure
//! to provide correct TIN on Form W-4; failure to file in
//! magnetic media when required; failure to provide W-9 to
//! payor on request. Completes the § 6109/§ 6721/§ 6722/
//! § 6723 information-return compliance set.
//!
//! Web research (verified 2026-06-03):
//! - **IRC § 6723 General Rule**: in the case of a failure by
//!   any person to comply with a **SPECIFIED INFORMATION
//!   REPORTING REQUIREMENT** on or before the time prescribed
//!   therefor, such person shall pay a penalty of **$50 for
//!   each such failure**, but the total amount imposed on such
//!   person for all such failures during any calendar year
//!   shall not exceed **$100,000** ([Cornell LII 26 USC §
//!   6723](https://www.law.cornell.edu/uscode/text/26/6723);
//!   [Bloomberg Tax Sec. 6723](https://irc.bloombergtax.com/public/uscode/doc/irc/section_6723)).
//! - **No Inflationary Adjustments**: penalties under § 6723
//!   are **NOT** subject to annual inflationary adjustments
//!   under § 1(f)(3) or Rev. Proc. cycle — the $50 per failure
//!   and $100,000 annual maximum remain fixed.
//! - **No Safe Harbor / De Minimis Exception / Small Business
//!   Exception**: unlike § 6721 and § 6722, there is **NO**
//!   safe harbor, de minimis exception, tier structure, or
//!   small business reduction available for § 6723 penalties.
//! - **"Specified Information Reporting Requirement"** under
//!   Treas. Reg. § 301.6723-1: includes (1) any TIN-furnishing
//!   requirement under § 6109(a); (2) filing/notice requirements
//!   under §§ 6038A, 6038B (Forms 5471, 8865 ancillary
//!   reporting); (3) § 6041A direct-sales notice requirements;
//!   (4) § 6042(c)(2) listing of corporate shareholders
//!   requirement; (5) magnetic media filing requirements where
//!   imposed by the Code or regulations ([26 CFR § 301.6723-1](https://www.ecfr.gov/current/title-26/chapter-I/subchapter-F/part-301/subpart-ECFRe7a848e7ecebb4b/subject-group-ECFR90240e9b8fcd266/section-301.6723-1)).
//! - **Common Triggers**: (a) failure to provide correct TIN
//!   on Form W-4 (Employee's Withholding Allowance Certificate);
//!   (b) failure of payee to provide W-9 to payor on request;
//!   (c) failure to file information return in magnetic media
//!   when required (currently 10+ returns triggers magnetic
//!   media requirement); (d) failure of taxpayer to provide
//!   TIN to corporation for § 6042(c)(2) compliance.
//! - **IRC § 6724 Reasonable Cause Waiver**: penalty waived if
//!   payer can show failure was due to reasonable cause AND
//!   not willful neglect — same reasonable-cause framework as
//!   § 6721 and § 6722.
//! - **Distinction from § 6721 / § 6722**: § 6721 covers
//!   payer's failure to FILE information return WITH IRS;
//!   § 6722 covers payer's failure to FURNISH payee statement
//!   TO PAYEE; § 6723 covers OTHER specified information
//!   reporting failures (payee-side TIN furnishing, magnetic
//!   media filing, ancillary statements) not within § 6721
//!   or § 6722 ([IRS IRM 20.1.7 Information Return
//!   Penalties](https://www.irs.gov/irm/part20/irm_20-001-007r)).

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const IRC_6723_PER_FAILURE_DOLLARS: u64 = 50;
pub const IRC_6723_ANNUAL_MAX_DOLLARS: u64 = 100_000;
pub const IRC_6723_NO_INFLATION_ADJUSTMENT: bool = true;
pub const IRC_6723_NO_SMALL_BUSINESS_EXCEPTION: bool = true;
pub const IRC_6723_NO_TIER_STRUCTURE: bool = true;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SpecifiedReportingRequirement {
    TinFurnishingUnderSection6109,
    Section6038AOrSection6038BAncillaryReporting,
    Section6041ADirectSalesNotice,
    Section6042C2ListingOfCorporateShareholders,
    MagneticMediaFilingRequirement,
    FormW9OrW4TinProvision,
    OtherSpecifiedReportingRequirementUnderTreasReg301_6723_1,
    NoSpecifiedReportingRequirementAtIssue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureStatus {
    NoFailureCompliedTimely,
    FailureToComplyTimely,
    FailureToIncludeCorrectSpecifiedInformation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section6723Mode {
    NotApplicableNoSpecifiedReportingRequirement,
    NotApplicableNoFailureCompliedTimely,
    CompliantSection6724ReasonableCauseWaiverGranted,
    ViolationSection6723PenaltyAccruesWithinAnnualMax,
    ViolationSection6723PenaltyAt100kAnnualMaxCeilingExactlyAllowed,
    ViolationSection6723PenaltyAtAnnualMaxCap,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub specified_reporting_requirement: SpecifiedReportingRequirement,
    pub failure_status: FailureStatus,
    pub number_of_failures: u64,
    pub reasonable_cause_waiver_granted: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section6723Mode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
    pub raw_penalty_dollars: u64,
    pub capped_penalty_dollars: u64,
}

pub type Section6723Input = Input;
pub type Section6723Output = Output;
pub type Section6723Result = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "IRC § 6723 — failure to comply with specified information reporting requirement on or before time prescribed; $50 per failure; total maximum $100,000 per calendar year".to_string(),
        "Treas. Reg. § 301.6723-1 — 'specified information reporting requirement' includes: (1) any TIN-furnishing requirement under § 6109(a); (2) filing/notice requirements under §§ 6038A, 6038B (Forms 5471, 8865 ancillary); (3) § 6041A direct-sales notice; (4) § 6042(c)(2) listing of corporate shareholders; (5) magnetic media filing requirements".to_string(),
        "No Inflationary Adjustments — penalties under § 6723 are NOT subject to annual inflationary adjustments; $50 per failure and $100,000 annual maximum remain fixed".to_string(),
        "No Safe Harbor / De Minimis Exception / Small Business Exception — unlike § 6721 and § 6722, NO safe harbor, de minimis exception, tier structure, or small business reduction available for § 6723 penalties".to_string(),
        "Common Triggers — (a) failure to provide correct TIN on Form W-4; (b) failure of payee to provide Form W-9 to payor on request; (c) failure to file information return in magnetic media when required (currently 10+ returns triggers magnetic media requirement); (d) failure of taxpayer to provide TIN to corporation for § 6042(c)(2) compliance".to_string(),
        "IRC § 6724 — reasonable cause waiver; penalty waived if payer shows failure due to events beyond control AND acted in responsible manner; same reasonable-cause framework as § 6721 and § 6722".to_string(),
        "Distinction from § 6721 / § 6722 — § 6721 covers payer's failure to FILE information return WITH IRS; § 6722 covers payer's failure to FURNISH payee statement TO PAYEE; § 6723 covers OTHER specified information reporting failures (payee-side TIN furnishing, magnetic media filing, ancillary statements) not within § 6721 or § 6722".to_string(),
        "Treas. Reg. § 301.6723-1 — implementing regulation".to_string(),
        "IRS IRM 20.1.7 — Information Return Penalties operational guidance".to_string(),
        "Cornell LII 26 USC § 6723 — primary statutory text".to_string(),
        "Bloomberg Tax Sec. 6723 — comprehensive code commentary".to_string(),
        "The Tax Adviser — Information return penalties: How to avoid or contest them — practitioner guide".to_string(),
    ];

    if input.specified_reporting_requirement
        == SpecifiedReportingRequirement::NoSpecifiedReportingRequirementAtIssue
    {
        return Output {
            mode: Section6723Mode::NotApplicableNoSpecifiedReportingRequirement,
            statutory_basis: "IRC § 6723 — applies only to specified information reporting requirements".to_string(),
            notes: "NOT APPLICABLE: no specified information reporting requirement at issue; § 6723 does not apply.".to_string(),
            citations,
            raw_penalty_dollars: 0,
            capped_penalty_dollars: 0,
        };
    }

    if input.failure_status == FailureStatus::NoFailureCompliedTimely {
        return Output {
            mode: Section6723Mode::NotApplicableNoFailureCompliedTimely,
            statutory_basis: "IRC § 6723 — no failure to comply".to_string(),
            notes: "NOT APPLICABLE: no failure to comply with specified information reporting requirement; § 6723 penalty does not apply.".to_string(),
            citations,
            raw_penalty_dollars: 0,
            capped_penalty_dollars: 0,
        };
    }

    if input.reasonable_cause_waiver_granted {
        return Output {
            mode: Section6723Mode::CompliantSection6724ReasonableCauseWaiverGranted,
            statutory_basis: "IRC § 6724 — reasonable cause waiver granted".to_string(),
            notes: "COMPLIANT: § 6724 reasonable cause waiver granted; payer established failure was due to events beyond control and acted in responsible manner; § 6723 penalty waived.".to_string(),
            citations,
            raw_penalty_dollars: 0,
            capped_penalty_dollars: 0,
        };
    }

    let raw_penalty_dollars = input
        .number_of_failures
        .saturating_mul(IRC_6723_PER_FAILURE_DOLLARS);
    let capped_penalty_dollars = raw_penalty_dollars.min(IRC_6723_ANNUAL_MAX_DOLLARS);

    if raw_penalty_dollars > IRC_6723_ANNUAL_MAX_DOLLARS {
        return Output {
            mode: Section6723Mode::ViolationSection6723PenaltyAtAnnualMaxCap,
            statutory_basis: "IRC § 6723 — $100,000 annual maximum reached".to_string(),
            notes: format!(
                "VIOLATION: {} failures × $50 per failure = ${} raw penalty; capped at $100,000 annual maximum under § 6723.",
                input.number_of_failures, raw_penalty_dollars
            ),
            citations,
            raw_penalty_dollars,
            capped_penalty_dollars,
        };
    }

    if raw_penalty_dollars == IRC_6723_ANNUAL_MAX_DOLLARS {
        return Output {
            mode: Section6723Mode::ViolationSection6723PenaltyAt100kAnnualMaxCeilingExactlyAllowed,
            statutory_basis: "IRC § 6723 — penalty at exactly $100,000 annual maximum".to_string(),
            notes: format!(
                "VIOLATION: {} failures × $50 per failure = ${} = exactly $100,000 annual maximum under § 6723.",
                input.number_of_failures, raw_penalty_dollars
            ),
            citations,
            raw_penalty_dollars,
            capped_penalty_dollars,
        };
    }

    Output {
        mode: Section6723Mode::ViolationSection6723PenaltyAccruesWithinAnnualMax,
        statutory_basis: "IRC § 6723 — $50 per failure penalty accrues within annual maximum".to_string(),
        notes: format!(
            "VIOLATION: {} failures × $50 per failure = ${} penalty; below $100,000 annual maximum under § 6723.",
            input.number_of_failures, raw_penalty_dollars
        ),
        citations,
        raw_penalty_dollars,
        capped_penalty_dollars,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_violation_10_failures() -> Input {
        Input {
            specified_reporting_requirement:
                SpecifiedReportingRequirement::TinFurnishingUnderSection6109,
            failure_status: FailureStatus::FailureToComplyTimely,
            number_of_failures: 10,
            reasonable_cause_waiver_granted: false,
        }
    }

    #[test]
    fn no_specified_reporting_requirement_not_applicable() {
        let input = Input {
            specified_reporting_requirement:
                SpecifiedReportingRequirement::NoSpecifiedReportingRequirementAtIssue,
            ..baseline_violation_10_failures()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section6723Mode::NotApplicableNoSpecifiedReportingRequirement
        );
    }

    #[test]
    fn no_failure_not_applicable() {
        let input = Input {
            failure_status: FailureStatus::NoFailureCompliedTimely,
            ..baseline_violation_10_failures()
        };
        let result = check(&input);
        assert_eq!(result.mode, Section6723Mode::NotApplicableNoFailureCompliedTimely);
    }

    #[test]
    fn reasonable_cause_waiver_compliant() {
        let input = Input {
            reasonable_cause_waiver_granted: true,
            ..baseline_violation_10_failures()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section6723Mode::CompliantSection6724ReasonableCauseWaiverGranted
        );
    }

    #[test]
    fn ten_failures_violation_within_annual_max() {
        let result = check(&baseline_violation_10_failures());
        assert_eq!(
            result.mode,
            Section6723Mode::ViolationSection6723PenaltyAccruesWithinAnnualMax
        );
        assert_eq!(result.raw_penalty_dollars, 500);
        assert_eq!(result.capped_penalty_dollars, 500);
    }

    #[test]
    fn at_exactly_2000_failures_equals_100k_max() {
        let input = Input {
            number_of_failures: 2_000,
            ..baseline_violation_10_failures()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section6723Mode::ViolationSection6723PenaltyAt100kAnnualMaxCeilingExactlyAllowed
        );
        assert_eq!(result.raw_penalty_dollars, 100_000);
        assert_eq!(result.capped_penalty_dollars, 100_000);
    }

    #[test]
    fn above_2000_failures_capped_at_100k() {
        let input = Input {
            number_of_failures: 3_000,
            ..baseline_violation_10_failures()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section6723Mode::ViolationSection6723PenaltyAtAnnualMaxCap
        );
        assert_eq!(result.raw_penalty_dollars, 150_000);
        assert_eq!(result.capped_penalty_dollars, 100_000);
    }

    #[test]
    fn very_large_failures_capped_at_100k() {
        let input = Input {
            number_of_failures: 1_000_000,
            ..baseline_violation_10_failures()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section6723Mode::ViolationSection6723PenaltyAtAnnualMaxCap
        );
        assert_eq!(result.capped_penalty_dollars, 100_000);
    }

    #[test]
    fn magnetic_media_filing_requirement_violation() {
        let input = Input {
            specified_reporting_requirement:
                SpecifiedReportingRequirement::MagneticMediaFilingRequirement,
            number_of_failures: 50,
            ..baseline_violation_10_failures()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section6723Mode::ViolationSection6723PenaltyAccruesWithinAnnualMax
        );
        assert_eq!(result.raw_penalty_dollars, 2_500);
    }

    #[test]
    fn form_w9_or_w4_tin_provision_violation() {
        let input = Input {
            specified_reporting_requirement: SpecifiedReportingRequirement::FormW9OrW4TinProvision,
            number_of_failures: 100,
            ..baseline_violation_10_failures()
        };
        let result = check(&input);
        assert_eq!(result.raw_penalty_dollars, 5_000);
    }

    #[test]
    fn section_6038a_ancillary_reporting_violation() {
        let input = Input {
            specified_reporting_requirement:
                SpecifiedReportingRequirement::Section6038AOrSection6038BAncillaryReporting,
            number_of_failures: 20,
            ..baseline_violation_10_failures()
        };
        let result = check(&input);
        assert_eq!(result.raw_penalty_dollars, 1_000);
    }

    #[test]
    fn failure_to_include_correct_information_violation() {
        let input = Input {
            failure_status: FailureStatus::FailureToIncludeCorrectSpecifiedInformation,
            number_of_failures: 200,
            ..baseline_violation_10_failures()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section6723Mode::ViolationSection6723PenaltyAccruesWithinAnnualMax
        );
        assert_eq!(result.raw_penalty_dollars, 10_000);
    }

    #[test]
    fn citations_pin_section_6723_and_distinctions() {
        let result = check(&baseline_violation_10_failures());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("IRC § 6723"));
        assert!(joined.contains("$50 per failure"));
        assert!(joined.contains("$100,000"));
        assert!(joined.contains("specified information reporting requirement"));
        assert!(joined.contains("Treas. Reg. § 301.6723-1"));
        assert!(joined.contains("§ 6109(a)"));
        assert!(joined.contains("§§ 6038A, 6038B"));
        assert!(joined.contains("§ 6041A"));
        assert!(joined.contains("§ 6042(c)(2)"));
        assert!(joined.contains("magnetic media"));
        assert!(joined.contains("No Inflationary Adjustments"));
        assert!(joined.contains("No Safe Harbor"));
        assert!(joined.contains("Form W-4"));
        assert!(joined.contains("Form W-9"));
        assert!(joined.contains("10+ returns"));
        assert!(joined.contains("IRC § 6724"));
        assert!(joined.contains("§ 6721 covers payer's failure to FILE"));
        assert!(joined.contains("§ 6722 covers payer's failure to FURNISH"));
        assert!(joined.contains("IRS IRM 20.1.7"));
    }

    #[test]
    fn constant_pin_per_failure_and_annual_max() {
        assert_eq!(IRC_6723_PER_FAILURE_DOLLARS, 50);
        assert_eq!(IRC_6723_ANNUAL_MAX_DOLLARS, 100_000);
        assert!(IRC_6723_NO_INFLATION_ADJUSTMENT);
        assert!(IRC_6723_NO_SMALL_BUSINESS_EXCEPTION);
        assert!(IRC_6723_NO_TIER_STRUCTURE);
    }

    #[test]
    fn saturating_overflow_defense_extreme_failures() {
        let input = Input {
            number_of_failures: u64::MAX,
            ..baseline_violation_10_failures()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section6723Mode::ViolationSection6723PenaltyAtAnnualMaxCap
        );
        assert_eq!(result.capped_penalty_dollars, 100_000);
    }
}
