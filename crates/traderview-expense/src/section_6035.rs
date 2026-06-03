//! IRC § 6035 Basis Information to Persons Acquiring
//! Property From Decedent + § 1014(f) Basis Consistency
//! Requirement — pure-compute compliance check for executor
//! reporting on Form 8971 + Schedule A and beneficiary basis
//! consistency with final estate tax value.
//!
//! Enacted by the Surface Transportation and Veterans Health
//! Care Choice Improvement Act of 2015 (Public Law 114-41,
//! 129 Stat. 443), signed by President Barack Obama on **JULY
//! 31, 2015**. Section 2004 of the 2015 Act added new IRC
//! sections **§ 1014(f)** (substantive basis consistency rule)
//! and **§ 6035** (reporting requirements on executors), plus
//! companion penalty provisions at **§ 6662(k)** (inconsistent
//! estate basis accuracy-related penalty), **§ 6721**
//! (information return failure), and **§ 6722** (payee
//! statement failure). Applies to estates required under § 6018
//! to file a federal estate tax return AFTER July 31, 2015.
//! Final regulations published in the Federal Register on
//! **SEPTEMBER 17, 2024**.
//!
//! Web research (verified 2026-06-03):
//! - **Enactment**: Surface Transportation and Veterans Health Care Choice Improvement Act of 2015, **Public Law 114-41**, 129 Stat. 443; signed by President Obama on **JULY 31, 2015**; Section 2004 of the Act added IRC §§ 1014(f) and 6035 ([Burr & Forman — Wait a Minute: Congress Passes Law Requiring Consistent Basis Reporting](https://www.burr.com/tax-law-insights/wait-a-minute-congress-passes-law-requiring-consistent-basis-reporting-between-estates-and-beneficiaries-for-estate-tax-purposes); [Schneider Downs — IRS Releases Form 8971](https://www.schneiderdowns.com/our-thoughts-on/irs-releases-final-version-form-8971); [ABA RPTE — Congress Passes New Reporting Requirements and Basis Consistency Rules Under IRC §§ 6035 and 1014(f)](https://www.americanbar.org/groups/real_property_trust_estate/publications/probate-property-magazine/2016/september_october_2016/2016_aba_rpte_pp_v30_5_article_hunt_congress_passes_new_requirements_and_consistency_rules_irc_6035_1014f/); [Bloomberg Tax — IRC § 6035 Basis Information To Persons Acquiring Property From Decedent](https://irc.bloombergtax.com/public/uscode/doc/irc/section_6035); [Federal Register — Consistent Basis Reporting Between Estate and Person Acquiring Property From Decedent (March 4, 2016 Proposed Rules)](https://www.federalregister.gov/documents/2016/03/04/2016-04718/consistent-basis-reporting-between-estate-and-person-acquiring-property-from-decedent); [Federal Register — Consistent Basis Reporting Between Estate and Person Acquiring Property From Decedent (September 17, 2024 Final Rules)](https://www.federalregister.gov/documents/2024/09/17/2024-20429/consistent-basis-reporting-between-estate-and-person-acquiring-property-from-decedent); [IRS Notice 2016-19 — Part III Administrative, Procedural, and Miscellaneous](https://www.irs.gov/pub/irs-drop/n-16-19.pdf); [IRS Instructions for Form 8971 and Schedule A (Rev. August 2025)](https://www.irs.gov/instructions/i8971); [Williams Mullen — Consistent Basis Reporting Update: Treasury Issues Proposed Rules on Portability Returns, Final Values, and After-Discovered Property](https://www.williamsmullen.com/insights/news/legal-news/consistent-basis-reporting-update-treasury-issues-proposed-rules); [Wealth Management — Basis Consistency and Reporting for Property Acquired from a Decedent](https://www.wealthmanagement.com/estate-planning/basis-consistency-and-reporting-property-acquired-decedent); [Wealth Management — Treasury Department Finalizes Basis Consistency Regulations](https://www.wealthmanagement.com/estate-planning/treasury-department-finalizes-basis-consistency-regulations); [Kitces — New IRS Form 8971 Rules To Report Beneficiary Cost Basis](https://www.kitces.com/blog/irs-form-8971-and-required-valuation-reporting-from-executors-for-beneficiary-step-up-in-cost-basis/); [ABA RPTE — Treasury Finalizes Long-Awaited Basis Consistency and Reporting Regulations (May / June 2025)](https://www.americanbar.org/groups/real_property_trust_estate/resources/probate-property/2025-may-june/treasury-finalizes-basis-consistency-reporting-regulations/)).
//! - **Effective Date Scope**: § 6035 applies to estates required to file under § 6018 when such return is filed AFTER **JULY 31, 2015**.
//! - **Form 8971 + Schedule A**: executor of estate required to file Form 706 must (a) file Form 8971 with IRS and (b) furnish each beneficiary acquiring an interest with the beneficiary's Schedule A, both within **30 DAYS** after the earlier of the date the estate tax return is required to be filed (including extensions) or the date it is actually filed.
//! - **Supplemental Form 8971 + Schedule A**: due within **30 DAYS** after the FINAL VALUE of property is determined OR the executor discovers the previously reported information is incorrect or incomplete OR a supplemental Federal estate tax return is filed.
//! - **§ 1014(f) Basis Consistency Rule**: heir's basis in inherited property cannot exceed the FINAL VALUE as determined for federal estate tax purposes; eliminates basis-step-up "abuse" where estate reports low value for estate tax minimization but heir claims high basis for income tax minimization.
//! - **§ 6662(k) Inconsistent Estate Basis Penalty**: 20 % accuracy-related penalty on any portion of an underpayment attributable to failure to comply with § 1014(f) consistent basis requirement.
//! - **§ 6721 Information Return Failure Penalty**: applies to Form 8971 filed with IRS — base penalty of $250 per failure (2015 enacted; periodically inflation-adjusted to higher tiers under § 6721(a)(1)).
//! - **§ 6722 Payee Statement Failure Penalty**: applies to Schedule A furnished to beneficiary — base penalty of $250 per failure (2015 enacted; periodically inflation-adjusted).
//! - **Final Regulations Federal Register Publication Date**: **SEPTEMBER 17, 2024** (89 FR final action) — key taxpayer-friendly changes included **(1) ELIMINATING the zero-basis rule** for property unreported on Form 8971 / Schedule A; **(2)** modifying definition of "acquiring" for § 6035(a)(1) timing of reporting in cases where beneficiary has not acquired before estate tax return due date; **(3) ELIMINATING the subsequent transfer reporting requirement** for all beneficiaries other than trustees; **(4)** excepting additional property types (cash, IRD, tangible personal property under § 6018(b) threshold) from the consistent basis and reporting regimes.
//! - **Notice 2015-57**: initial executor reporting transition relief (originally extended deadlines for Forms 8971 from August 31, 2015 to February 29, 2016 then to March 31, 2016 then to June 30, 2016).
//! - **Notice 2016-19**: further executor reporting transition relief and adjustments to § 6035 deadlines.
//! - **§ 6018 Estate Tax Return Required Threshold**: estate must file Form 706 if gross estate exceeds the applicable exclusion amount (e.g., **$13,990,000** for 2025 under post-TCJA basic exclusion; subject to sunset under § 2010(c)(3)(C) absent extension). § 6035 reporting is keyed to whether the estate is REQUIRED to file Form 706 — not whether it actually files voluntarily for portability election under § 2010(c)(5).

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const IRC_6035_ENACTMENT_DATE_YEAR: u32 = 2015;
pub const IRC_6035_ENACTMENT_DATE_MONTH: u32 = 7;
pub const IRC_6035_ENACTMENT_DATE_DAY: u32 = 31;
pub const IRC_6035_PUBLIC_LAW_CONGRESS_NUMBER: u32 = 114;
pub const IRC_6035_PUBLIC_LAW_ENACTMENT_NUMBER: u32 = 41;
pub const IRC_6035_STAT_VOLUME: u32 = 129;
pub const IRC_6035_STAT_PAGE: u32 = 443;
pub const IRC_6035_REPORTING_DEADLINE_DAYS_AFTER_ESTATE_TAX_RETURN_FILED: u32 = 30;
pub const IRC_6035_SUPPLEMENTAL_REPORTING_DEADLINE_DAYS: u32 = 30;
pub const IRC_6035_FINAL_REGULATIONS_FEDERAL_REGISTER_PUBLICATION_DATE_YEAR: u32 = 2024;
pub const IRC_6035_FINAL_REGULATIONS_FEDERAL_REGISTER_PUBLICATION_DATE_MONTH: u32 = 9;
pub const IRC_6035_FINAL_REGULATIONS_FEDERAL_REGISTER_PUBLICATION_DATE_DAY: u32 = 17;
pub const IRC_6035_ENABLING_ACT_SECTION_NUMBER: u32 = 2004;
pub const IRC_6662_K_INCONSISTENT_ESTATE_BASIS_ACCURACY_PENALTY_RATE_BPS: u64 = 2_000;
pub const IRC_6721_INFORMATION_RETURN_BASE_PENALTY_DOLLARS: u64 = 250;
pub const IRC_6722_PAYEE_STATEMENT_BASE_PENALTY_DOLLARS: u64 = 250;
pub const IRC_6035_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EstateFilingStatus {
    EstateRequiredToFileForm706UnderSection6018,
    EstateNotRequiredToFileButFilesVoluntarilyForPortabilityUnderSection2010C5,
    EstateNotRequiredToFileAndDoesNotFile,
    EstateTaxReturnFiledOnOrBeforeJuly31_2015PreEnactment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyInclusionStatus {
    PropertyIncludedInGrossEstateSubjectToReporting,
    PropertyExcludedFromGrossEstate,
    PropertyExceptedFromReportingByFinalRegulations,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReportingComplianceAspect {
    Form8971AndScheduleAInitialFiling,
    SupplementalForm8971AfterFinalValueOrCorrection,
    BeneficiaryBasisConsistencyWithFinalEstateTaxValueUnderSection1014F,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FilingTimelinessStatus {
    FiledWithinThirtyDayWindow,
    FiledAfterThirtyDayWindow,
    NotFiledAtAll,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BeneficiaryBasisStatus {
    BeneficiaryReportedBasisEqualToFinalEstateTaxValue,
    BeneficiaryReportedBasisBelowFinalEstateTaxValue,
    BeneficiaryReportedBasisExceedingFinalEstateTaxValueInconsistent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section6035Mode {
    NotApplicableEstateNotRequiredToFileForm706,
    NotApplicableEstateTaxReturnFiledOnOrBeforeJuly31_2015PreEnactment,
    NotApplicablePropertyExcludedFromGrossEstate,
    NotApplicablePropertyExceptedByFinalRegulations,
    CompliantForm8971AndScheduleAFiledWithinThirtyDays,
    CompliantSupplementalForm8971FiledWithinThirtyDaysOfFinalValueDetermination,
    CompliantBeneficiaryBasisConsistentWithFinalEstateTaxValue,
    CompliantBeneficiaryBasisBelowFinalEstateTaxValueDownwardConsistent,
    ViolationFailureToFileForm8971WithIrs,
    ViolationFailureToFurnishScheduleAToBeneficiary,
    ViolationLateFilingOfForm8971PastThirtyDayDeadline,
    ViolationFailureToFileSupplementalAfterFinalValueDetermined,
    ViolationLateSupplementalFilingPastThirtyDayDeadline,
    ViolationInconsistentEstateBasisBeneficiaryClaimingHigherBasisThanFinalEstateTaxValue,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub estate_filing_status: EstateFilingStatus,
    pub property_inclusion_status: PropertyInclusionStatus,
    pub reporting_compliance_aspect: ReportingComplianceAspect,
    pub initial_filing_timeliness: FilingTimelinessStatus,
    pub supplemental_filing_timeliness: FilingTimelinessStatus,
    pub schedule_a_furnished_to_beneficiary: bool,
    pub beneficiary_basis_status: BeneficiaryBasisStatus,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section6035Mode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
    pub information_return_penalty_per_failure_dollars: u64,
    pub payee_statement_penalty_per_failure_dollars: u64,
    pub inconsistent_estate_basis_accuracy_penalty_rate_bps: u64,
}

pub type Section6035Input = Input;
pub type Section6035Output = Output;
pub type Section6035Result = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "Surface Transportation and Veterans Health Care Choice Improvement Act of 2015 (Public Law 114-41), 129 Stat. 443, signed by President Barack Obama on July 31, 2015; Section 2004 of the Act added IRC §§ 1014(f) and 6035 with companion penalty provisions at § 6662(k), § 6721, and § 6722".to_string(),
        "IRC § 6035(a)(1) — executor of estate required to file estate tax return under § 6018 must furnish to the Secretary and to each person acquiring any interest in property included in the decedent's gross estate a statement (Form 8971 + Schedule A) identifying the value of each interest as reported on the estate tax return".to_string(),
        "IRC § 6035(a)(3) — reporting due within 30 days after the EARLIER of (i) the date the estate tax return is required to be filed (with extensions) or (ii) the date the estate tax return is actually filed".to_string(),
        "IRC § 6035(a)(3)(B) — supplemental Form 8971 + Schedule A due within 30 days after the FINAL VALUE of property is determined OR executor discovers previously reported information is incorrect or incomplete OR a supplemental Federal estate tax return is filed".to_string(),
        "IRC § 1014(f) Basis Consistency Rule — heir's basis in inherited property CANNOT EXCEED the FINAL VALUE as determined for federal estate tax purposes (the 'consistent basis' requirement)".to_string(),
        "IRC § 6662(k) Inconsistent Estate Basis Penalty — 20 % accuracy-related penalty on any portion of an underpayment attributable to failure to comply with § 1014(f) consistent basis requirement; defined as 'inconsistent estate basis'".to_string(),
        "IRC § 6721 Information Return Failure Penalty — base penalty $250 per failure for Form 8971 filed with IRS (subject to inflation adjustment under § 6721(a)(1); tiered penalty structure escalates for delays beyond 30 days, beyond August 1, or intentional disregard)".to_string(),
        "IRC § 6722 Payee Statement Failure Penalty — base penalty $250 per failure for Schedule A furnished to beneficiary (subject to inflation adjustment; tiered penalty structure escalates parallel to § 6721)".to_string(),
        "IRS Form 8971 + Schedule A — Information Regarding Beneficiaries Acquiring Property from a Decedent; final August 2025 instructions available at irs.gov/instructions/i8971".to_string(),
        "Federal Register Final Regulations published September 17, 2024 (89 FR final action) — taxpayer-friendly modifications: (1) ELIMINATING the zero-basis rule for unreported property; (2) modifying definition of 'acquiring' for § 6035(a)(1) timing; (3) ELIMINATING subsequent transfer reporting requirement for all beneficiaries other than trustees; (4) excepting additional property types (cash; income in respect of decedent; tangible personal property under § 6018(b) threshold) from consistent basis and reporting requirements".to_string(),
        "IRS Notice 2015-57 — initial executor reporting transition relief delaying Form 8971 deadlines through June 30, 2016".to_string(),
        "IRS Notice 2016-19 — further executor reporting transition relief and adjustments to § 6035 deadlines".to_string(),
        "Effective Date Scope — § 6035 applies only to estates required under § 6018 to file estate tax return when that return is filed AFTER JULY 31, 2015".to_string(),
        "Portability-Only Filings under § 2010(c)(5) — estate not REQUIRED to file Form 706 but filing voluntarily for portability election does NOT trigger § 6035 reporting (only required filings under § 6018 trigger reporting)".to_string(),
        "ABA RPTE September / October 2016 — Congress Passes New Reporting Requirements and Basis Consistency Rules Under IRC §§ 6035 and 1014(f) — practitioner overview".to_string(),
        "ABA RPTE May / June 2025 — Treasury Finalizes Long-Awaited Basis Consistency and Reporting Regulations — final-regs practitioner guide".to_string(),
    ];

    if input.estate_filing_status == EstateFilingStatus::EstateTaxReturnFiledOnOrBeforeJuly31_2015PreEnactment {
        return Output {
            mode: Section6035Mode::NotApplicableEstateTaxReturnFiledOnOrBeforeJuly31_2015PreEnactment,
            statutory_basis: "Public Law 114-41 § 2004(d) effective date — § 6035 applies only to estates filing under § 6018 after July 31, 2015".to_string(),
            notes: "NOT APPLICABLE: estate tax return filed on or before July 31, 2015 (pre-enactment of Public Law 114-41); § 6035 and § 1014(f) basis consistency do not apply; Form 8971 reporting not required.".to_string(),
            citations,
            information_return_penalty_per_failure_dollars: 0,
            payee_statement_penalty_per_failure_dollars: 0,
            inconsistent_estate_basis_accuracy_penalty_rate_bps: 0,
        };
    }

    if matches!(
        input.estate_filing_status,
        EstateFilingStatus::EstateNotRequiredToFileButFilesVoluntarilyForPortabilityUnderSection2010C5
            | EstateFilingStatus::EstateNotRequiredToFileAndDoesNotFile
    ) {
        return Output {
            mode: Section6035Mode::NotApplicableEstateNotRequiredToFileForm706,
            statutory_basis: "IRC § 6035(a)(1) keyed to § 6018 mandatory filing — not portability-only voluntary filings under § 2010(c)(5)".to_string(),
            notes: "NOT APPLICABLE: estate not required to file Form 706 under § 6018; portability-only voluntary filings under § 2010(c)(5) do not trigger § 6035 reporting; Form 8971 not required.".to_string(),
            citations,
            information_return_penalty_per_failure_dollars: 0,
            payee_statement_penalty_per_failure_dollars: 0,
            inconsistent_estate_basis_accuracy_penalty_rate_bps: 0,
        };
    }

    if input.property_inclusion_status == PropertyInclusionStatus::PropertyExcludedFromGrossEstate {
        return Output {
            mode: Section6035Mode::NotApplicablePropertyExcludedFromGrossEstate,
            statutory_basis: "IRC § 6035(a)(1) — only property INCLUDED in decedent's gross estate is subject to reporting".to_string(),
            notes: "NOT APPLICABLE: property excluded from decedent's gross estate (e.g., property held under non-includible trust; pre-decedent gifts; non-probate transfers outside § 2031); § 6035 reporting does not attach.".to_string(),
            citations,
            information_return_penalty_per_failure_dollars: 0,
            payee_statement_penalty_per_failure_dollars: 0,
            inconsistent_estate_basis_accuracy_penalty_rate_bps: 0,
        };
    }

    if input.property_inclusion_status == PropertyInclusionStatus::PropertyExceptedFromReportingByFinalRegulations {
        return Output {
            mode: Section6035Mode::NotApplicablePropertyExceptedByFinalRegulations,
            statutory_basis: "Treas. Reg. § 1.6035-1 (September 17, 2024 final regulations) — exceptions for cash; income in respect of decedent (IRD); tangible personal property under § 6018(b) threshold".to_string(),
            notes: "NOT APPLICABLE: property type excepted from consistent basis and reporting requirements by final September 17, 2024 regulations (cash; income in respect of decedent under § 691; tangible personal property under § 6018(b) threshold; certain non-includible interests).".to_string(),
            citations,
            information_return_penalty_per_failure_dollars: 0,
            payee_statement_penalty_per_failure_dollars: 0,
            inconsistent_estate_basis_accuracy_penalty_rate_bps: 0,
        };
    }

    match input.reporting_compliance_aspect {
        ReportingComplianceAspect::Form8971AndScheduleAInitialFiling => match input.initial_filing_timeliness {
            FilingTimelinessStatus::NotFiledAtAll => Output {
                mode: Section6035Mode::ViolationFailureToFileForm8971WithIrs,
                statutory_basis: "IRC § 6035(a)(1) + § 6721 — failure to file Form 8971 with IRS triggers information return penalty".to_string(),
                notes: "VIOLATION: executor failed to file Form 8971 with IRS at all; § 6721 information return failure penalty applies; base $250 per failure with escalation under tiered penalty structure for intentional disregard.".to_string(),
                citations,
                information_return_penalty_per_failure_dollars: IRC_6721_INFORMATION_RETURN_BASE_PENALTY_DOLLARS,
                payee_statement_penalty_per_failure_dollars: IRC_6722_PAYEE_STATEMENT_BASE_PENALTY_DOLLARS,
                inconsistent_estate_basis_accuracy_penalty_rate_bps: 0,
            },
            FilingTimelinessStatus::FiledAfterThirtyDayWindow => {
                if !input.schedule_a_furnished_to_beneficiary {
                    Output {
                        mode: Section6035Mode::ViolationFailureToFurnishScheduleAToBeneficiary,
                        statutory_basis: "IRC § 6035(a)(1) + § 6722 — failure to furnish Schedule A to beneficiary triggers payee statement penalty".to_string(),
                        notes: "VIOLATION: executor filed Form 8971 with IRS but failed to furnish Schedule A to beneficiary; § 6722 payee statement failure penalty applies; base $250 per failure.".to_string(),
                        citations,
                        information_return_penalty_per_failure_dollars: IRC_6721_INFORMATION_RETURN_BASE_PENALTY_DOLLARS,
                        payee_statement_penalty_per_failure_dollars: IRC_6722_PAYEE_STATEMENT_BASE_PENALTY_DOLLARS,
                        inconsistent_estate_basis_accuracy_penalty_rate_bps: 0,
                    }
                } else {
                    Output {
                        mode: Section6035Mode::ViolationLateFilingOfForm8971PastThirtyDayDeadline,
                        statutory_basis: "IRC § 6035(a)(3) + § 6721 — Form 8971 filed past 30-day window triggers late-filing penalty".to_string(),
                        notes: "VIOLATION: executor filed Form 8971 with IRS but past the 30-day window after the earlier of estate tax return due date or actual filing date; § 6721 tiered late-filing penalty applies (escalating with delay).".to_string(),
                        citations,
                        information_return_penalty_per_failure_dollars: IRC_6721_INFORMATION_RETURN_BASE_PENALTY_DOLLARS,
                        payee_statement_penalty_per_failure_dollars: IRC_6722_PAYEE_STATEMENT_BASE_PENALTY_DOLLARS,
                        inconsistent_estate_basis_accuracy_penalty_rate_bps: 0,
                    }
                }
            }
            FilingTimelinessStatus::FiledWithinThirtyDayWindow => {
                if !input.schedule_a_furnished_to_beneficiary {
                    Output {
                        mode: Section6035Mode::ViolationFailureToFurnishScheduleAToBeneficiary,
                        statutory_basis: "IRC § 6035(a)(1) + § 6722 — failure to furnish Schedule A to beneficiary".to_string(),
                        notes: "VIOLATION: executor timely filed Form 8971 with IRS but failed to furnish Schedule A to beneficiary; § 6722 payee statement failure penalty applies; base $250 per failure.".to_string(),
                        citations,
                        information_return_penalty_per_failure_dollars: 0,
                        payee_statement_penalty_per_failure_dollars: IRC_6722_PAYEE_STATEMENT_BASE_PENALTY_DOLLARS,
                        inconsistent_estate_basis_accuracy_penalty_rate_bps: 0,
                    }
                } else {
                    Output {
                        mode: Section6035Mode::CompliantForm8971AndScheduleAFiledWithinThirtyDays,
                        statutory_basis: "IRC § 6035(a)(1) + (a)(3) — Form 8971 and Schedule A filed within statutory 30-day window".to_string(),
                        notes: "COMPLIANT: executor filed Form 8971 with IRS and furnished Schedule A to beneficiary within 30 days after the earlier of estate tax return due date or actual filing date.".to_string(),
                        citations,
                        information_return_penalty_per_failure_dollars: 0,
                        payee_statement_penalty_per_failure_dollars: 0,
                        inconsistent_estate_basis_accuracy_penalty_rate_bps: 0,
                    }
                }
            }
        },
        ReportingComplianceAspect::SupplementalForm8971AfterFinalValueOrCorrection => {
            match input.supplemental_filing_timeliness {
                FilingTimelinessStatus::NotFiledAtAll => Output {
                    mode: Section6035Mode::ViolationFailureToFileSupplementalAfterFinalValueDetermined,
                    statutory_basis: "IRC § 6035(a)(3)(B) + § 6721 — failure to file supplemental Form 8971 after final value determined".to_string(),
                    notes: "VIOLATION: final value of property was determined or executor discovered previously reported information incorrect or incomplete but executor did not file supplemental Form 8971; § 6035(a)(3)(B) requires supplemental filing within 30 days; § 6721 penalty applies.".to_string(),
                    citations,
                    information_return_penalty_per_failure_dollars: IRC_6721_INFORMATION_RETURN_BASE_PENALTY_DOLLARS,
                    payee_statement_penalty_per_failure_dollars: IRC_6722_PAYEE_STATEMENT_BASE_PENALTY_DOLLARS,
                    inconsistent_estate_basis_accuracy_penalty_rate_bps: 0,
                },
                FilingTimelinessStatus::FiledAfterThirtyDayWindow => Output {
                    mode: Section6035Mode::ViolationLateSupplementalFilingPastThirtyDayDeadline,
                    statutory_basis: "IRC § 6035(a)(3)(B) + § 6721 — supplemental Form 8971 filed past 30-day window".to_string(),
                    notes: "VIOLATION: executor filed supplemental Form 8971 but past the 30-day window after final value determination or discovery of error; § 6721 tiered late-filing penalty applies.".to_string(),
                    citations,
                    information_return_penalty_per_failure_dollars: IRC_6721_INFORMATION_RETURN_BASE_PENALTY_DOLLARS,
                    payee_statement_penalty_per_failure_dollars: IRC_6722_PAYEE_STATEMENT_BASE_PENALTY_DOLLARS,
                    inconsistent_estate_basis_accuracy_penalty_rate_bps: 0,
                },
                FilingTimelinessStatus::FiledWithinThirtyDayWindow => Output {
                    mode: Section6035Mode::CompliantSupplementalForm8971FiledWithinThirtyDaysOfFinalValueDetermination,
                    statutory_basis: "IRC § 6035(a)(3)(B) — supplemental Form 8971 filed within 30 days of final value or correction".to_string(),
                    notes: "COMPLIANT: executor timely filed supplemental Form 8971 within 30 days after final value of property was determined or after discovery that previously reported information was incorrect or incomplete.".to_string(),
                    citations,
                    information_return_penalty_per_failure_dollars: 0,
                    payee_statement_penalty_per_failure_dollars: 0,
                    inconsistent_estate_basis_accuracy_penalty_rate_bps: 0,
                },
            }
        }
        ReportingComplianceAspect::BeneficiaryBasisConsistencyWithFinalEstateTaxValueUnderSection1014F => match input.beneficiary_basis_status {
            BeneficiaryBasisStatus::BeneficiaryReportedBasisEqualToFinalEstateTaxValue => Output {
                mode: Section6035Mode::CompliantBeneficiaryBasisConsistentWithFinalEstateTaxValue,
                statutory_basis: "IRC § 1014(f) — beneficiary basis equal to final estate tax value satisfies consistent basis rule".to_string(),
                notes: "COMPLIANT: beneficiary's reported basis equals the final value as determined for federal estate tax purposes; § 1014(f) consistent basis rule satisfied; no § 6662(k) inconsistent estate basis penalty exposure.".to_string(),
                citations,
                information_return_penalty_per_failure_dollars: 0,
                payee_statement_penalty_per_failure_dollars: 0,
                inconsistent_estate_basis_accuracy_penalty_rate_bps: 0,
            },
            BeneficiaryBasisStatus::BeneficiaryReportedBasisBelowFinalEstateTaxValue => Output {
                mode: Section6035Mode::CompliantBeneficiaryBasisBelowFinalEstateTaxValueDownwardConsistent,
                statutory_basis: "IRC § 1014(f) — beneficiary basis below final estate tax value satisfies one-directional consistency rule".to_string(),
                notes: "COMPLIANT: beneficiary's reported basis is below (not above) final estate tax value; § 1014(f) rule is one-directional ('cannot exceed') so downward basis claims do not trigger inconsistency; suboptimal for beneficiary but not penalized.".to_string(),
                citations,
                information_return_penalty_per_failure_dollars: 0,
                payee_statement_penalty_per_failure_dollars: 0,
                inconsistent_estate_basis_accuracy_penalty_rate_bps: 0,
            },
            BeneficiaryBasisStatus::BeneficiaryReportedBasisExceedingFinalEstateTaxValueInconsistent => Output {
                mode: Section6035Mode::ViolationInconsistentEstateBasisBeneficiaryClaimingHigherBasisThanFinalEstateTaxValue,
                statutory_basis: "IRC § 1014(f) + § 6662(k) — beneficiary basis exceeding final estate tax value violates consistent basis rule".to_string(),
                notes: "VIOLATION: beneficiary's reported basis EXCEEDS the final value as determined for federal estate tax purposes; § 1014(f) consistent basis rule violated; § 6662(k) 20 % accuracy-related inconsistent estate basis penalty applies on underpayment.".to_string(),
                citations,
                information_return_penalty_per_failure_dollars: 0,
                payee_statement_penalty_per_failure_dollars: 0,
                inconsistent_estate_basis_accuracy_penalty_rate_bps: IRC_6662_K_INCONSISTENT_ESTATE_BASIS_ACCURACY_PENALTY_RATE_BPS,
            },
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_input() -> Input {
        Input {
            estate_filing_status: EstateFilingStatus::EstateRequiredToFileForm706UnderSection6018,
            property_inclusion_status: PropertyInclusionStatus::PropertyIncludedInGrossEstateSubjectToReporting,
            reporting_compliance_aspect: ReportingComplianceAspect::Form8971AndScheduleAInitialFiling,
            initial_filing_timeliness: FilingTimelinessStatus::FiledWithinThirtyDayWindow,
            supplemental_filing_timeliness: FilingTimelinessStatus::FiledWithinThirtyDayWindow,
            schedule_a_furnished_to_beneficiary: true,
            beneficiary_basis_status: BeneficiaryBasisStatus::BeneficiaryReportedBasisEqualToFinalEstateTaxValue,
        }
    }

    #[test]
    fn pre_enactment_estate_return_not_applicable() {
        let mut input = baseline_input();
        input.estate_filing_status = EstateFilingStatus::EstateTaxReturnFiledOnOrBeforeJuly31_2015PreEnactment;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section6035Mode::NotApplicableEstateTaxReturnFiledOnOrBeforeJuly31_2015PreEnactment
        );
    }

    #[test]
    fn estate_not_required_to_file_not_applicable() {
        let mut input = baseline_input();
        input.estate_filing_status = EstateFilingStatus::EstateNotRequiredToFileAndDoesNotFile;
        let output = check(&input);
        assert_eq!(output.mode, Section6035Mode::NotApplicableEstateNotRequiredToFileForm706);
    }

    #[test]
    fn portability_only_voluntary_filing_not_applicable() {
        let mut input = baseline_input();
        input.estate_filing_status =
            EstateFilingStatus::EstateNotRequiredToFileButFilesVoluntarilyForPortabilityUnderSection2010C5;
        let output = check(&input);
        assert_eq!(output.mode, Section6035Mode::NotApplicableEstateNotRequiredToFileForm706);
    }

    #[test]
    fn property_excluded_from_gross_estate_not_applicable() {
        let mut input = baseline_input();
        input.property_inclusion_status = PropertyInclusionStatus::PropertyExcludedFromGrossEstate;
        let output = check(&input);
        assert_eq!(output.mode, Section6035Mode::NotApplicablePropertyExcludedFromGrossEstate);
    }

    #[test]
    fn property_excepted_by_final_regulations_not_applicable() {
        let mut input = baseline_input();
        input.property_inclusion_status =
            PropertyInclusionStatus::PropertyExceptedFromReportingByFinalRegulations;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section6035Mode::NotApplicablePropertyExceptedByFinalRegulations
        );
    }

    #[test]
    fn initial_filing_within_thirty_days_compliant() {
        let output = check(&baseline_input());
        assert_eq!(
            output.mode,
            Section6035Mode::CompliantForm8971AndScheduleAFiledWithinThirtyDays
        );
    }

    #[test]
    fn initial_filing_not_filed_violation() {
        let mut input = baseline_input();
        input.initial_filing_timeliness = FilingTimelinessStatus::NotFiledAtAll;
        let output = check(&input);
        assert_eq!(output.mode, Section6035Mode::ViolationFailureToFileForm8971WithIrs);
        assert_eq!(
            output.information_return_penalty_per_failure_dollars,
            IRC_6721_INFORMATION_RETURN_BASE_PENALTY_DOLLARS
        );
    }

    #[test]
    fn initial_filing_late_past_thirty_day_violation() {
        let mut input = baseline_input();
        input.initial_filing_timeliness = FilingTimelinessStatus::FiledAfterThirtyDayWindow;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section6035Mode::ViolationLateFilingOfForm8971PastThirtyDayDeadline
        );
    }

    #[test]
    fn schedule_a_not_furnished_violation_when_form_filed_timely() {
        let mut input = baseline_input();
        input.schedule_a_furnished_to_beneficiary = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section6035Mode::ViolationFailureToFurnishScheduleAToBeneficiary
        );
        assert_eq!(
            output.payee_statement_penalty_per_failure_dollars,
            IRC_6722_PAYEE_STATEMENT_BASE_PENALTY_DOLLARS
        );
    }

    #[test]
    fn supplemental_filing_within_thirty_days_compliant() {
        let mut input = baseline_input();
        input.reporting_compliance_aspect =
            ReportingComplianceAspect::SupplementalForm8971AfterFinalValueOrCorrection;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section6035Mode::CompliantSupplementalForm8971FiledWithinThirtyDaysOfFinalValueDetermination
        );
    }

    #[test]
    fn supplemental_filing_not_filed_violation() {
        let mut input = baseline_input();
        input.reporting_compliance_aspect =
            ReportingComplianceAspect::SupplementalForm8971AfterFinalValueOrCorrection;
        input.supplemental_filing_timeliness = FilingTimelinessStatus::NotFiledAtAll;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section6035Mode::ViolationFailureToFileSupplementalAfterFinalValueDetermined
        );
    }

    #[test]
    fn supplemental_filing_late_past_thirty_days_violation() {
        let mut input = baseline_input();
        input.reporting_compliance_aspect =
            ReportingComplianceAspect::SupplementalForm8971AfterFinalValueOrCorrection;
        input.supplemental_filing_timeliness = FilingTimelinessStatus::FiledAfterThirtyDayWindow;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section6035Mode::ViolationLateSupplementalFilingPastThirtyDayDeadline
        );
    }

    #[test]
    fn beneficiary_basis_equal_to_final_estate_tax_value_compliant() {
        let mut input = baseline_input();
        input.reporting_compliance_aspect =
            ReportingComplianceAspect::BeneficiaryBasisConsistencyWithFinalEstateTaxValueUnderSection1014F;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section6035Mode::CompliantBeneficiaryBasisConsistentWithFinalEstateTaxValue
        );
    }

    #[test]
    fn beneficiary_basis_below_final_estate_tax_value_downward_consistent() {
        let mut input = baseline_input();
        input.reporting_compliance_aspect =
            ReportingComplianceAspect::BeneficiaryBasisConsistencyWithFinalEstateTaxValueUnderSection1014F;
        input.beneficiary_basis_status =
            BeneficiaryBasisStatus::BeneficiaryReportedBasisBelowFinalEstateTaxValue;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section6035Mode::CompliantBeneficiaryBasisBelowFinalEstateTaxValueDownwardConsistent
        );
    }

    #[test]
    fn beneficiary_basis_exceeding_final_estate_tax_value_violation_triggers_section_6662k_penalty() {
        let mut input = baseline_input();
        input.reporting_compliance_aspect =
            ReportingComplianceAspect::BeneficiaryBasisConsistencyWithFinalEstateTaxValueUnderSection1014F;
        input.beneficiary_basis_status =
            BeneficiaryBasisStatus::BeneficiaryReportedBasisExceedingFinalEstateTaxValueInconsistent;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section6035Mode::ViolationInconsistentEstateBasisBeneficiaryClaimingHigherBasisThanFinalEstateTaxValue
        );
        assert_eq!(
            output.inconsistent_estate_basis_accuracy_penalty_rate_bps,
            IRC_6662_K_INCONSISTENT_ESTATE_BASIS_ACCURACY_PENALTY_RATE_BPS
        );
        assert_eq!(
            output.inconsistent_estate_basis_accuracy_penalty_rate_bps,
            2_000
        );
    }

    #[test]
    fn constants_pin_statutory_facts() {
        assert_eq!(IRC_6035_ENACTMENT_DATE_YEAR, 2015);
        assert_eq!(IRC_6035_ENACTMENT_DATE_MONTH, 7);
        assert_eq!(IRC_6035_ENACTMENT_DATE_DAY, 31);
        assert_eq!(IRC_6035_PUBLIC_LAW_CONGRESS_NUMBER, 114);
        assert_eq!(IRC_6035_PUBLIC_LAW_ENACTMENT_NUMBER, 41);
        assert_eq!(IRC_6035_STAT_VOLUME, 129);
        assert_eq!(IRC_6035_STAT_PAGE, 443);
        assert_eq!(
            IRC_6035_REPORTING_DEADLINE_DAYS_AFTER_ESTATE_TAX_RETURN_FILED,
            30
        );
        assert_eq!(IRC_6035_SUPPLEMENTAL_REPORTING_DEADLINE_DAYS, 30);
        assert_eq!(IRC_6035_FINAL_REGULATIONS_FEDERAL_REGISTER_PUBLICATION_DATE_YEAR, 2024);
        assert_eq!(IRC_6035_FINAL_REGULATIONS_FEDERAL_REGISTER_PUBLICATION_DATE_MONTH, 9);
        assert_eq!(IRC_6035_FINAL_REGULATIONS_FEDERAL_REGISTER_PUBLICATION_DATE_DAY, 17);
        assert_eq!(IRC_6035_ENABLING_ACT_SECTION_NUMBER, 2004);
        assert_eq!(
            IRC_6662_K_INCONSISTENT_ESTATE_BASIS_ACCURACY_PENALTY_RATE_BPS,
            2_000
        );
        assert_eq!(IRC_6721_INFORMATION_RETURN_BASE_PENALTY_DOLLARS, 250);
        assert_eq!(IRC_6722_PAYEE_STATEMENT_BASE_PENALTY_DOLLARS, 250);
        assert_eq!(IRC_6035_BASIS_POINT_DENOMINATOR, 10_000);
    }

    #[test]
    fn citations_contain_landmarks() {
        let output = check(&baseline_input());
        let joined = output.citations.join("\n");
        assert!(joined.contains("Surface Transportation and Veterans Health Care Choice Improvement Act of 2015"));
        assert!(joined.contains("Public Law 114-41"));
        assert!(joined.contains("129 Stat. 443"));
        assert!(joined.contains("July 31, 2015"));
        assert!(joined.contains("§ 1014(f)"));
        assert!(joined.contains("§ 6035"));
        assert!(joined.contains("§ 6018"));
        assert!(joined.contains("§ 6662(k)"));
        assert!(joined.contains("§ 6721"));
        assert!(joined.contains("§ 6722"));
        assert!(joined.contains("Form 8971"));
        assert!(joined.contains("Schedule A"));
        assert!(joined.contains("30 days"));
        assert!(joined.contains("September 17, 2024"));
        assert!(joined.contains("Notice 2015-57"));
        assert!(joined.contains("Notice 2016-19"));
        assert!(joined.contains("§ 2010(c)(5)"));
        assert!(joined.contains("zero-basis rule"));
    }
}
