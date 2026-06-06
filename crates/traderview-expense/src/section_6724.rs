//! IRC § 6724 — Waiver / Definitions / Special Rules
//! Reasonable Cause Defense Module.
//!
//! Pure-compute check for IRC § 6724 reasonable cause defense
//! and information return penalty waiver framework. § 6724
//! completes the **§ 6721 / § 6722 / § 6723 / § 6724
//! information-return penalty quartet** by providing the
//! reasonable-cause waiver standard (§ 6724(a)), de minimis
//! failure exception (§ 6724(c)), and statutory definitions
//! (§ 6724(d)) of "information return" and "payee statement"
//! referenced by all three preceding penalty sections.
//! Trader-critical for: defending § 6721 / § 6722 / § 6723
//! penalty assessments through reasonable-cause demonstration;
//! qualifying for de minimis exception (§ 6724(c) greater of
//! 10 or 0.5 % of total returns); building documented
//! compliance history under Treas. Reg. § 301.6724-1.
//!
//! Web research (verified 2026-06-03):
//! - **IRC § 6724(a) Reasonable Cause Waiver**: no penalty
//!   shall be imposed under §§ 6721, 6722, or 6723 with
//!   respect to any failure if it is shown that **such
//!   failure is due to REASONABLE CAUSE and NOT to WILLFUL
//!   NEGLECT** ([Cornell LII 26 USC § 6724](https://www.law.cornell.edu/uscode/text/26/6724);
//!   [Bloomberg Tax Sec. 6724](https://irc.bloombergtax.com/public/uscode/doc/irc/section_6724);
//!   [Meadows Collier — Information Reporting Penalty and
//!   the Reasonable Cause Defense](https://www.meadowscollier.com/information-reporting-penalty-and-the-reasonable-cause-defense)).
//! - **Treas. Reg. § 301.6724-1 Reasonable Cause Test**:
//!   filer must establish BOTH (1) **either** (a) significant
//!   mitigating factors with respect to failure OR (b)
//!   failure arose from **events beyond filer's control**
//!   (an "impediment"); AND (2) filer **acted in a
//!   responsible manner** BOTH before and after the failure
//!   occurred ([26 CFR § 301.6724-1](https://www.ecfr.gov/current/title-26/chapter-I/subchapter-F/part-301/subpart-ECFRe7a848e7ecebb4b/subject-group-ECFR90240e9b8fcd266/section-301.6724-1)).
//! - **Examples of Impediments** (Treas. Reg. § 301.6724-1):
//!   natural disaster (hurricane, earthquake, flood); IRS
//!   systems failure preventing e-filing; death or serious
//!   illness of person responsible for filing; fire or
//!   casualty destroying records; unavailability of necessary
//!   business records due to circumstances beyond control.
//! - **Examples of Mitigating Factors**: established history
//!   of compliance; isolated incident; prompt correction
//!   immediately upon discovery; immateriality of underlying
//!   error.
//! - **"Acted in Responsible Manner"** requires: established
//!   prior compliance history; documented internal controls
//!   for information return preparation; prompt correction
//!   action immediately after discovery; cooperation with
//!   IRS examination.
//! - **IRC § 6724(b) Payment of Penalty**: any penalty
//!   imposed by §§ 6721, 6722, or 6723 shall be paid on
//!   notice and demand by the Secretary and in the same
//!   manner as tax.
//! - **IRC § 6724(c) De Minimis Failure Exception**: § 6721
//!   and § 6722 penalties NOT imposed on failures corrected
//!   by August 1 if number of such failures does NOT exceed
//!   the **GREATER OF (i) 10 OR (ii) ONE-HALF OF ONE PERCENT
//!   (0.5 %) of the total number** of information returns /
//!   payee statements required to be filed during the
//!   calendar year. De minimis exception applies AFTER
//!   reasonable cause analysis; NOT available for § 6723
//!   penalties.
//! - **IRC § 6724(d)(1) "Information Return" Definition**:
//!   any return or statement required to be filed under
//!   enumerated sections — Forms 1098 (mortgage interest),
//!   1099 (NEC/MISC/DIV/INT/B/K/etc.), 3921 (incentive stock
//!   option exercise), 3922 (employee stock purchase plan
//!   transfer), 5498 (IRA contributions), W-2G (gambling
//!   winnings), 1097 (bond tax credit allocation), W-2 (wage
//!   and tax statement), W-3 (transmittal of W-2), Form 5471
//!   (foreign corporation), Form 8865 (foreign partnership).
//! - **IRC § 6724(d)(2) "Payee Statement" Definition**: any
//!   statement required to be furnished to payee under
//!   enumerated sections — corresponds to recipient copies
//!   of forms in § 6724(d)(1).
//! - **IRC § 6724(d)(3) "Specified Information Reporting
//!   Requirement"**: covered by § 6723 — payee TIN
//!   furnishing requirements (W-9, W-4); magnetic media
//!   filing; ancillary statements.
//! - **IRS Publication 1586** ("Reasonable Cause Regulations
//!   and Requirements for Missing and Incorrect"): operational
//!   IRS guidance for reasonable cause documentation.
//! - **Revenue Procedure 2025-22** (IR Bulletin 2025-30):
//!   IRS guidance on TIN solicitation and reasonable cause
//!   procedures for 2026 information returns.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const IRC_6724_DE_MINIMIS_FIXED_FLOOR_COUNT: u64 = 10;
pub const IRC_6724_DE_MINIMIS_PCT_BASIS_POINTS: u64 = 50;
pub const IRC_6724_DE_MINIMIS_PCT_BASIS_POINT_DENOMINATOR: u64 = 10_000;
pub const IRC_6724_D_1_INFORMATION_RETURN_FORM_COUNT: u32 = 11;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PenaltyUnderlyingSection {
    Section6721FailureToFileInformationReturn,
    Section6722FailureToFurnishPayeeStatement,
    Section6723FailureToComplyOtherInformationReporting,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ImpedimentCategory {
    NaturalDisasterHurricaneEarthquakeFloodFire,
    IrsSystemsFailurePreventingEFiling,
    DeathOrSeriousIllnessOfResponsiblePerson,
    UnavailabilityOfBusinessRecordsBeyondControl,
    NoImpedimentCausedFailure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MitigatingFactorCategory {
    EstablishedHistoryOfCompliance,
    IsolatedIncidentOtherwiseComplete,
    PromptCorrectionImmediatelyUponDiscovery,
    ImmaterialityOfUnderlyingError,
    NoMitigatingFactor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponsibleMannerStatus {
    EstablishedPriorComplianceWithDocumentedControlsAndPromptCorrection,
    NoResponsibleManner,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section6724Mode {
    NotApplicableNoUnderlyingPenalty,
    CompliantReasonableCauseWaiverGrantedWillfulNeglectNegated,
    CompliantDeMinimisExceptionUnderSection6724C,
    CompliantBothReasonableCauseAndDeMinimisApply,
    ViolationReasonableCauseClaimedButNoImpedimentOrMitigatingFactor,
    ViolationReasonableCauseClaimedButFilerDidNotActResponsibly,
    ViolationDeMinimisExceptionClaimedButFailuresExceedGreaterOf10OrHalfPercent,
    ViolationDeMinimisClaimedForSection6723ButOnlyAppliesToSection6721And6722,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub penalty_underlying_section: PenaltyUnderlyingSection,
    pub impediment_category: ImpedimentCategory,
    pub mitigating_factor_category: MitigatingFactorCategory,
    pub responsible_manner_status: ResponsibleMannerStatus,
    pub willful_neglect_present: bool,
    pub claiming_de_minimis_exception: bool,
    pub number_of_failures_corrected_by_august_1: u64,
    pub total_returns_or_statements_required_during_calendar_year: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section6724Mode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
    pub de_minimis_threshold_count: u64,
}

pub type Section6724Input = Input;
pub type Section6724Output = Output;
pub type Section6724Result = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "IRC § 6724(a) — reasonable cause waiver: no penalty under §§ 6721, 6722, or 6723 if failure due to REASONABLE CAUSE and NOT to WILLFUL NEGLECT".to_string(),
        "Treas. Reg. § 301.6724-1 — reasonable cause two-prong test: (1) significant mitigating factors OR events beyond filer's control (impediment); AND (2) filer acted in responsible manner BOTH before and after failure".to_string(),
        "Examples of Impediments — natural disaster (hurricane, earthquake, flood); IRS systems failure preventing e-filing; death or serious illness of person responsible for filing; fire or casualty destroying records; unavailability of business records due to circumstances beyond control".to_string(),
        "Examples of Mitigating Factors — established history of compliance; isolated incident; prompt correction immediately upon discovery; immateriality of underlying error".to_string(),
        "'Acted in Responsible Manner' requires — established prior compliance history; documented internal controls for information return preparation; prompt correction action immediately after discovery; cooperation with IRS examination".to_string(),
        "IRC § 6724(b) — payment of penalty: any penalty imposed by §§ 6721, 6722, or 6723 shall be paid on notice and demand by the Secretary and in the same manner as tax".to_string(),
        "IRC § 6724(c) — DE MINIMIS FAILURE EXCEPTION: § 6721 and § 6722 penalties NOT imposed on failures corrected by August 1 if number of failures does NOT exceed GREATER OF (i) 10 OR (ii) ONE-HALF OF ONE PERCENT (0.5 %) of total information returns / payee statements required to be filed during calendar year; applies AFTER reasonable cause analysis; NOT available for § 6723 penalties".to_string(),
        "IRC § 6724(d)(1) 'Information Return' Definition — Forms 1098, 1099, 3921, 3922, 5498, W-2G, 1097, W-2, W-3, Form 5471 (foreign corporation), Form 8865 (foreign partnership)".to_string(),
        "IRC § 6724(d)(2) 'Payee Statement' Definition — any statement required to be furnished to payee; corresponds to recipient copies of forms in § 6724(d)(1)".to_string(),
        "IRC § 6724(d)(3) 'Specified Information Reporting Requirement' — covered by § 6723; payee TIN furnishing requirements (W-9, W-4); magnetic media filing; ancillary statements".to_string(),
        "IRS Publication 1586 — Reasonable Cause Regulations and Requirements for Missing and Incorrect; operational IRS guidance for reasonable cause documentation".to_string(),
        "Revenue Procedure 2025-22 (IR Bulletin 2025-30) — IRS guidance on TIN solicitation and reasonable cause procedures for 2026 information returns".to_string(),
        "IRS IRM 20.1.7 — Information Return Penalties operational guidance".to_string(),
        "Cornell LII 26 USC § 6724 — primary statutory text".to_string(),
        "Bloomberg Tax Sec. 6724 — comprehensive code commentary".to_string(),
        "Meadows Collier — Information Reporting Penalty and the Reasonable Cause Defense — practitioner guide".to_string(),
    ];

    let de_minimis_pct_count = input
        .total_returns_or_statements_required_during_calendar_year
        .saturating_mul(IRC_6724_DE_MINIMIS_PCT_BASIS_POINTS)
        / IRC_6724_DE_MINIMIS_PCT_BASIS_POINT_DENOMINATOR;
    let de_minimis_threshold_count =
        IRC_6724_DE_MINIMIS_FIXED_FLOOR_COUNT.max(de_minimis_pct_count);

    if input.willful_neglect_present {
        return Output {
            mode: Section6724Mode::ViolationReasonableCauseClaimedButNoImpedimentOrMitigatingFactor,
            statutory_basis: "IRC § 6724(a) — reasonable cause unavailable when willful neglect present".to_string(),
            notes: "VIOLATION: reasonable cause waiver claimed but willful neglect present; § 6724(a) requires failure NOT due to willful neglect — waiver unavailable.".to_string(),
            citations,
            de_minimis_threshold_count,
        };
    }

    let has_impediment_or_mitigating = !matches!(
        input.impediment_category,
        ImpedimentCategory::NoImpedimentCausedFailure
    ) || !matches!(
        input.mitigating_factor_category,
        MitigatingFactorCategory::NoMitigatingFactor
    );

    let acted_responsibly = matches!(
        input.responsible_manner_status,
        ResponsibleMannerStatus::EstablishedPriorComplianceWithDocumentedControlsAndPromptCorrection
    );

    let de_minimis_qualified = input.claiming_de_minimis_exception
        && matches!(
            input.penalty_underlying_section,
            PenaltyUnderlyingSection::Section6721FailureToFileInformationReturn
                | PenaltyUnderlyingSection::Section6722FailureToFurnishPayeeStatement
        )
        && input.number_of_failures_corrected_by_august_1 <= de_minimis_threshold_count;

    if input.claiming_de_minimis_exception
        && input.penalty_underlying_section
            == PenaltyUnderlyingSection::Section6723FailureToComplyOtherInformationReporting
    {
        return Output {
            mode: Section6724Mode::ViolationDeMinimisClaimedForSection6723ButOnlyAppliesToSection6721And6722,
            statutory_basis: "IRC § 6724(c) — de minimis exception applies only to § 6721 and § 6722, NOT § 6723".to_string(),
            notes: "VIOLATION: de minimis exception claimed for § 6723 penalty but § 6724(c) limits the exception to § 6721 and § 6722 only; § 6723 catch-all penalties not eligible.".to_string(),
            citations,
            de_minimis_threshold_count,
        };
    }

    if input.claiming_de_minimis_exception
        && !de_minimis_qualified
        && matches!(
            input.penalty_underlying_section,
            PenaltyUnderlyingSection::Section6721FailureToFileInformationReturn
                | PenaltyUnderlyingSection::Section6722FailureToFurnishPayeeStatement
        )
    {
        return Output {
            mode: Section6724Mode::ViolationDeMinimisExceptionClaimedButFailuresExceedGreaterOf10OrHalfPercent,
            statutory_basis: "IRC § 6724(c) — de minimis exception requires failures ≤ greater of 10 or 0.5 % of total".to_string(),
            notes: format!(
                "VIOLATION: de minimis exception claimed but {} failures corrected by August 1 exceeds greater of {} (fixed floor 10) or {} (0.5 % of {} total returns) = {} statutory threshold.",
                input.number_of_failures_corrected_by_august_1,
                IRC_6724_DE_MINIMIS_FIXED_FLOOR_COUNT,
                de_minimis_pct_count,
                input.total_returns_or_statements_required_during_calendar_year,
                de_minimis_threshold_count
            ),
            citations,
            de_minimis_threshold_count,
        };
    }

    if !has_impediment_or_mitigating && !de_minimis_qualified {
        return Output {
            mode: Section6724Mode::ViolationReasonableCauseClaimedButNoImpedimentOrMitigatingFactor,
            statutory_basis: "Treas. Reg. § 301.6724-1 — reasonable cause requires impediment OR mitigating factor".to_string(),
            notes: "VIOLATION: reasonable cause waiver claimed but neither impediment (events beyond filer's control) nor significant mitigating factor present; § 6724(a) waiver unavailable.".to_string(),
            citations,
            de_minimis_threshold_count,
        };
    }

    if has_impediment_or_mitigating && !acted_responsibly {
        return Output {
            mode: Section6724Mode::ViolationReasonableCauseClaimedButFilerDidNotActResponsibly,
            statutory_basis: "Treas. Reg. § 301.6724-1 — filer must act in responsible manner before AND after failure".to_string(),
            notes: "VIOLATION: impediment or mitigating factor present but filer did not act in responsible manner before and after failure (e.g., no prior compliance history, no internal controls, no prompt correction action); § 6724(a) waiver unavailable.".to_string(),
            citations,
            de_minimis_threshold_count,
        };
    }

    let reasonable_cause_qualified = has_impediment_or_mitigating && acted_responsibly;

    if reasonable_cause_qualified && de_minimis_qualified {
        return Output {
            mode: Section6724Mode::CompliantBothReasonableCauseAndDeMinimisApply,
            statutory_basis: "IRC § 6724(a) + § 6724(c) — both reasonable cause and de minimis apply".to_string(),
            notes: format!(
                "COMPLIANT: both reasonable cause waiver under § 6724(a) AND de minimis exception under § 6724(c) apply; {} failures within threshold of {} (greater of 10 or 0.5 %); penalty waived.",
                input.number_of_failures_corrected_by_august_1, de_minimis_threshold_count
            ),
            citations,
            de_minimis_threshold_count,
        };
    }

    if reasonable_cause_qualified {
        return Output {
            mode: Section6724Mode::CompliantReasonableCauseWaiverGrantedWillfulNeglectNegated,
            statutory_basis: "IRC § 6724(a) — reasonable cause waiver granted".to_string(),
            notes: "COMPLIANT: § 6724(a) reasonable cause waiver granted; impediment or mitigating factor present; filer acted in responsible manner before and after failure; willful neglect negated; § 6721/§ 6722/§ 6723 penalty waived.".to_string(),
            citations,
            de_minimis_threshold_count,
        };
    }

    Output {
        mode: Section6724Mode::CompliantDeMinimisExceptionUnderSection6724C,
        statutory_basis: "IRC § 6724(c) — de minimis exception applies".to_string(),
        notes: format!(
            "COMPLIANT: § 6724(c) de minimis exception applies; {} failures corrected by August 1 ≤ statutory threshold of {} (greater of 10 or 0.5 % of {} total returns); § 6721 or § 6722 penalty waived.",
            input.number_of_failures_corrected_by_august_1,
            de_minimis_threshold_count,
            input.total_returns_or_statements_required_during_calendar_year
        ),
        citations,
        de_minimis_threshold_count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_reasonable_cause_qualified() -> Input {
        Input {
            penalty_underlying_section: PenaltyUnderlyingSection::Section6721FailureToFileInformationReturn,
            impediment_category: ImpedimentCategory::NaturalDisasterHurricaneEarthquakeFloodFire,
            mitigating_factor_category: MitigatingFactorCategory::EstablishedHistoryOfCompliance,
            responsible_manner_status:
                ResponsibleMannerStatus::EstablishedPriorComplianceWithDocumentedControlsAndPromptCorrection,
            willful_neglect_present: false,
            claiming_de_minimis_exception: false,
            number_of_failures_corrected_by_august_1: 0,
            total_returns_or_statements_required_during_calendar_year: 1_000,
        }
    }

    #[test]
    fn willful_neglect_blocks_reasonable_cause() {
        let input = Input {
            willful_neglect_present: true,
            ..baseline_reasonable_cause_qualified()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section6724Mode::ViolationReasonableCauseClaimedButNoImpedimentOrMitigatingFactor
        );
    }

    #[test]
    fn reasonable_cause_natural_disaster_compliant() {
        let result = check(&baseline_reasonable_cause_qualified());
        assert_eq!(
            result.mode,
            Section6724Mode::CompliantReasonableCauseWaiverGrantedWillfulNeglectNegated
        );
    }

    #[test]
    fn reasonable_cause_irs_systems_failure_compliant() {
        let input = Input {
            impediment_category: ImpedimentCategory::IrsSystemsFailurePreventingEFiling,
            ..baseline_reasonable_cause_qualified()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section6724Mode::CompliantReasonableCauseWaiverGrantedWillfulNeglectNegated
        );
    }

    #[test]
    fn reasonable_cause_death_serious_illness_compliant() {
        let input = Input {
            impediment_category: ImpedimentCategory::DeathOrSeriousIllnessOfResponsiblePerson,
            ..baseline_reasonable_cause_qualified()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section6724Mode::CompliantReasonableCauseWaiverGrantedWillfulNeglectNegated
        );
    }

    #[test]
    fn no_impediment_no_mitigating_factor_violation() {
        let input = Input {
            impediment_category: ImpedimentCategory::NoImpedimentCausedFailure,
            mitigating_factor_category: MitigatingFactorCategory::NoMitigatingFactor,
            ..baseline_reasonable_cause_qualified()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section6724Mode::ViolationReasonableCauseClaimedButNoImpedimentOrMitigatingFactor
        );
    }

    #[test]
    fn mitigating_factor_only_no_impediment_compliant() {
        let input = Input {
            impediment_category: ImpedimentCategory::NoImpedimentCausedFailure,
            mitigating_factor_category: MitigatingFactorCategory::EstablishedHistoryOfCompliance,
            ..baseline_reasonable_cause_qualified()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section6724Mode::CompliantReasonableCauseWaiverGrantedWillfulNeglectNegated
        );
    }

    #[test]
    fn did_not_act_responsibly_violation() {
        let input = Input {
            responsible_manner_status: ResponsibleMannerStatus::NoResponsibleManner,
            ..baseline_reasonable_cause_qualified()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section6724Mode::ViolationReasonableCauseClaimedButFilerDidNotActResponsibly
        );
    }

    #[test]
    fn de_minimis_within_10_floor_compliant() {
        let input = Input {
            impediment_category: ImpedimentCategory::NoImpedimentCausedFailure,
            mitigating_factor_category: MitigatingFactorCategory::NoMitigatingFactor,
            claiming_de_minimis_exception: true,
            number_of_failures_corrected_by_august_1: 5,
            total_returns_or_statements_required_during_calendar_year: 100,
            ..baseline_reasonable_cause_qualified()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section6724Mode::CompliantDeMinimisExceptionUnderSection6724C
        );
    }

    #[test]
    fn de_minimis_within_half_percent_compliant() {
        let input = Input {
            impediment_category: ImpedimentCategory::NoImpedimentCausedFailure,
            mitigating_factor_category: MitigatingFactorCategory::NoMitigatingFactor,
            claiming_de_minimis_exception: true,
            number_of_failures_corrected_by_august_1: 49,
            total_returns_or_statements_required_during_calendar_year: 10_000,
            ..baseline_reasonable_cause_qualified()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section6724Mode::CompliantDeMinimisExceptionUnderSection6724C
        );
    }

    #[test]
    fn de_minimis_at_exactly_half_percent_compliant() {
        let input = Input {
            impediment_category: ImpedimentCategory::NoImpedimentCausedFailure,
            mitigating_factor_category: MitigatingFactorCategory::NoMitigatingFactor,
            claiming_de_minimis_exception: true,
            number_of_failures_corrected_by_august_1: 50,
            total_returns_or_statements_required_during_calendar_year: 10_000,
            ..baseline_reasonable_cause_qualified()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section6724Mode::CompliantDeMinimisExceptionUnderSection6724C
        );
    }

    #[test]
    fn de_minimis_exceeds_half_percent_violation() {
        let input = Input {
            impediment_category: ImpedimentCategory::NoImpedimentCausedFailure,
            mitigating_factor_category: MitigatingFactorCategory::NoMitigatingFactor,
            claiming_de_minimis_exception: true,
            number_of_failures_corrected_by_august_1: 51,
            total_returns_or_statements_required_during_calendar_year: 10_000,
            ..baseline_reasonable_cause_qualified()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section6724Mode::ViolationDeMinimisExceptionClaimedButFailuresExceedGreaterOf10OrHalfPercent
        );
    }

    #[test]
    fn de_minimis_for_section_6723_not_available_violation() {
        let input = Input {
            penalty_underlying_section:
                PenaltyUnderlyingSection::Section6723FailureToComplyOtherInformationReporting,
            claiming_de_minimis_exception: true,
            number_of_failures_corrected_by_august_1: 5,
            ..baseline_reasonable_cause_qualified()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section6724Mode::ViolationDeMinimisClaimedForSection6723ButOnlyAppliesToSection6721And6722
        );
    }

    #[test]
    fn both_reasonable_cause_and_de_minimis_compliant() {
        let input = Input {
            claiming_de_minimis_exception: true,
            number_of_failures_corrected_by_august_1: 5,
            ..baseline_reasonable_cause_qualified()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section6724Mode::CompliantBothReasonableCauseAndDeMinimisApply
        );
    }

    #[test]
    fn citations_pin_section_6724_subsections_and_definitions() {
        let result = check(&baseline_reasonable_cause_qualified());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("IRC § 6724(a)"));
        assert!(joined.contains("IRC § 6724(b)"));
        assert!(joined.contains("IRC § 6724(c)"));
        assert!(joined.contains("IRC § 6724(d)(1)"));
        assert!(joined.contains("IRC § 6724(d)(2)"));
        assert!(joined.contains("IRC § 6724(d)(3)"));
        assert!(joined.contains("REASONABLE CAUSE"));
        assert!(joined.contains("WILLFUL NEGLECT"));
        assert!(joined.contains("DE MINIMIS FAILURE EXCEPTION"));
        assert!(joined.contains("GREATER OF"));
        assert!(joined.contains("ONE-HALF OF ONE PERCENT"));
        assert!(joined.contains("0.5 %"));
        assert!(joined.contains("Treas. Reg. § 301.6724-1"));
        assert!(joined.contains("hurricane"));
        assert!(joined.contains("Form 5471"));
        assert!(joined.contains("Form 8865"));
        assert!(joined.contains("IRS Publication 1586"));
        assert!(joined.contains("Revenue Procedure 2025-22"));
        assert!(joined.contains("IRS IRM 20.1.7"));
        assert!(joined.contains("Meadows Collier"));
    }

    #[test]
    fn constant_pin_de_minimis_floor_and_pct() {
        assert_eq!(IRC_6724_DE_MINIMIS_FIXED_FLOOR_COUNT, 10);
        assert_eq!(IRC_6724_DE_MINIMIS_PCT_BASIS_POINTS, 50);
        assert_eq!(IRC_6724_DE_MINIMIS_PCT_BASIS_POINT_DENOMINATOR, 10_000);
        assert_eq!(IRC_6724_D_1_INFORMATION_RETURN_FORM_COUNT, 11);
    }

    #[test]
    fn de_minimis_threshold_calculation() {
        let input = Input {
            total_returns_or_statements_required_during_calendar_year: 10_000,
            ..baseline_reasonable_cause_qualified()
        };
        let result = check(&input);
        assert_eq!(result.de_minimis_threshold_count, 50);
    }

    #[test]
    fn de_minimis_threshold_uses_10_floor_for_small_filers() {
        let input = Input {
            total_returns_or_statements_required_during_calendar_year: 500,
            ..baseline_reasonable_cause_qualified()
        };
        let result = check(&input);
        assert_eq!(result.de_minimis_threshold_count, 10);
    }

    #[test]
    fn saturating_overflow_defense_extreme_returns() {
        let input = Input {
            impediment_category: ImpedimentCategory::NoImpedimentCausedFailure,
            mitigating_factor_category: MitigatingFactorCategory::NoMitigatingFactor,
            claiming_de_minimis_exception: true,
            number_of_failures_corrected_by_august_1: 1,
            total_returns_or_statements_required_during_calendar_year: u64::MAX,
            ..baseline_reasonable_cause_qualified()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section6724Mode::CompliantDeMinimisExceptionUnderSection6724C
        );
    }
}
