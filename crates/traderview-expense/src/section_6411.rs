//! IRC § 6411 — Tentative Carryback and Refund Adjustments
//! / Form 1139 (Corporation) and Form 1045 (Individual)
//! Quick Refund Module.
//!
//! Pure-compute check for IRC § 6411 tentative carryback
//! adjustment procedure — the procedural mechanism by which
//! taxpayers claim accelerated refunds for net operating
//! losses (§ 172), net capital losses (§ 1212), or unused
//! business credits (§ 39) carried back to a prior taxable
//! year. § 6411 is the companion to § 6425 (corporate quick
//! refund for estimated tax overpayments — built iter 684)
//! and follows a similar 90-day IRS processing model. Form
//! 1139 (Corporation Application for Tentative Refund) is
//! the operative form for corporations; Form 1045
//! (Application for Tentative Refund) is the analog for
//! individuals, estates, and trusts. § 6411(d) added by 1978
//! amendment extends the tentative refund procedure to
//! § 1341 claim-of-right adjustments.
//!
//! Web research (verified 2026-06-03):
//! - **IRC § 6411(a) Right to Tentative Adjustment**: a taxpayer may file an application for a tentative carryback adjustment of tax for any taxable year affected by a NOL carryback (§ 172) or net capital loss carryback (§ 1212) from a subsequent taxable year, or by an unused business credit carryback (§ 39). For corporations the application is filed on **FORM 1139 ("Corporation Application for Tentative Refund")**; for individuals, estates, and trusts the analog is **FORM 1045 ("Application for Tentative Refund")** ([Cornell LII 26 USC § 6411](https://www.law.cornell.edu/uscode/text/26/6411); [Bloomberg Tax Sec. 6411](https://irc.bloombergtax.com/public/uscode/doc/irc/section_6411); [House Office of Law Revision Counsel — 26 USC § 6411 prelim](https://uscode.house.gov/view.xhtml?req=(title:26+section:6411+edition:prelim)); [IRS Instructions for Form 1139 (Rev. December 2025)](https://www.irs.gov/pub/irs-pdf/i1139.pdf); [IRS Notice 2020-26 — Extension of Time to File Form 1139 / 1045](https://www.irs.gov/pub/irs-drop/n-20-26.pdf); [Journal of Accountancy — Guidance on NOL Carryback and Tentative Carryback Adjustments under CARES Act](https://www.journalofaccountancy.com/news/2020/apr/guidance-nol-carryback-tentative-adjustments-coronavirus-cares-act/); [Journal of Accountancy — IRS Finalizes Regs on Tentative Carryback Adjustments](https://www.journalofaccountancy.com/news/2010/aug/20103246.html); [Foley & Lardner — IRS Issues Guidance Regarding NOL Carryback Waivers and Refunds Under the CARES Act](https://www.foley.com/insights/publications/2020/04/irs-guidance-carryback-waivers-refunds-cares-act/); [Akerman — IRS Issues Guidance on the NOL Carryback Provisions of the CARES Act](https://www.akerman.com/en/perspectives/irs-issues-guidance-on-the-nol-carryback-provisions-of-the-cares-act.html); [IRM 21.5.9 Carrybacks](https://www.irs.gov/irm/part21/irm_21-005-009r); [eCFR — 26 CFR 5.6411-1 Tentative Refund Under Claim of Right Adjustment](https://www.ecfr.gov/current/title-26/chapter-I/subchapter-A/part-5/section-5.6411-1)).
//! - **IRC § 6411(a) Filing Deadline**: the application must be filed within **12 MONTHS after the end of the taxable year** in which the NOL, net capital loss, or unused business credit AROSE. The taxpayer must also have FILED its income tax return for the loss / unused-credit taxable year NO LATER THAN the date the Form 1139 / Form 1045 application is filed.
//! - **IRC § 6411(b) IRS Examination Window — 90 DAYS**: within **90 DAYS** from the date on which the application for tentative carryback adjustment is filed, the Secretary shall (1) make a LIMITED EXAMINATION of the application to discover omissions and material errors of computation; (2) determine the amount of the decrease in tax attributable to the carryback / unused credit; (3) apply any decrease against unpaid amounts then due from the taxpayer; (4) refund any remainder to the taxpayer within the 90-day window. The 90-day window is shorter than the § 6425 corporate-quick-refund procedure (which is 45 days).
//! - **IRC § 6411(c) Consolidated Return Special Rules**: special limitations apply when corporations file consolidated returns for either the loss year or the affected prior year. These rules ensure that the tentative carryback adjustment properly accounts for intercompany transactions and consolidated taxable income / loss aggregations.
//! - **IRC § 6411(d) Claim of Right Tentative Refund**: added by Public Law 95-628 of 1978; permits tentative refund under § 1341(b)(1) for amounts repaid under a claim of right within **12 MONTHS** from the last day of such taxable year. This sub-provision allows accelerated refund for the year-of-repayment portion of a § 1341 claim-of-right adjustment without waiting for the standard refund claim cycle.
//! - **NOT a Refund Claim under § 6511**: an application under § 6411 SHALL NOT constitute a claim for credit or refund under § 6511. The § 6411 tentative refund process is distinct from the standard § 6511 refund-claim procedure, but the corporation retains the right to file a normal refund claim if the tentative refund is disallowed or insufficient.
//! - **Categories Eligible for Tentative Carryback**: (1) **§ 172 Net Operating Loss** carried back to one or more prior years (default 2-year carryback for most NOLs under pre-TCJA law; TCJA eliminated NOL carrybacks for losses arising in taxable years beginning after December 31, 2017, except for certain farming losses and § 1212 net capital losses retained 3-year carryback); (2) **§ 1212 Net Capital Loss** — corporations may carry back 3 years and forward 5 years; (3) **§ 39 Unused Business Credit** — 1-year carryback and 20-year carryforward; (4) **§ 1341(b)(1) Claim of Right Adjustment** under § 6411(d).
//! - **Subsequent Disallowance Risk**: if the IRS subsequently disallows the tentative carryback adjustment, the excessive refund is treated under **§ 6213(b)(3)** (special rule for tentative carryback and refund adjustments) as a deficiency that may be assessed without the normal § 6213(a) Tax Court petition right; interest at the § 6601 underpayment rate accrues from the original refund date. This creates a high-stakes due-diligence requirement when filing Form 1139 / Form 1045 because the procedural protections of standard deficiency procedure are not available.
//! - **CARES Act 2020 Updates** (Public Law 116-136, March 27, 2020): temporarily allowed corporations to carry back NOLs arising in taxable years beginning in 2018, 2019, or 2020 to each of the FIVE preceding taxable years; IRS Notice 2020-26 extended Form 1139 / Form 1045 filing deadlines for tax-year-2018 NOLs by 6 months due to COVID-19 administrative delays; these CARES Act provisions sunset for losses arising in 2021 and later.
//! - **Penalty Considerations**: filing a fraudulent Form 1139 or Form 1045 may trigger **§ 7206 criminal penalties** for fraud and false statements; substantial understatements may trigger **§ 6662 accuracy-related penalties** on the underlying tax.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const IRC_6411_FILING_DEADLINE_MONTHS_AFTER_TAXABLE_YEAR_END: u32 = 12;
pub const IRC_6411_IRS_EXAMINATION_WINDOW_DAYS: u32 = 90;
pub const IRC_6411_CLAIM_OF_RIGHT_DEADLINE_MONTHS: u32 = 12;
pub const IRC_6411_1978_AMENDMENT_YEAR: u32 = 1978;
pub const IRC_6411_CARES_ACT_NOL_CARRYBACK_YEARS: u32 = 5;
pub const IRC_6411_CARES_ACT_TAX_YEAR_START_YEAR: u32 = 2018;
pub const IRC_6411_CARES_ACT_TAX_YEAR_END_YEAR: u32 = 2020;
pub const IRC_6411_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TaxpayerType {
    CorporationFilesForm1139,
    IndividualFilesForm1045,
    EstateFilesForm1045,
    TrustFilesForm1045,
    OtherEntityNotEligibleForSection6411,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ApplicationFormStatus {
    Form1139CorporationApplicationFiled,
    Form1045IndividualEstateOrTrustApplicationFiled,
    OtherFormOrNoApplicationFiled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CarrybackCategory {
    NetOperatingLossCarrybackUnderSection172,
    NetCapitalLossCarrybackUnderSection1212,
    UnusedBusinessCreditCarrybackUnderSection39,
    ClaimOfRightAdjustmentUnderSection1341BSection6411D,
    NoCarrybackAmount,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IncomeTaxReturnStatus {
    IncomeTaxReturnFiledBeforeOrConcurrentlyWithForm1139OrForm1045,
    IncomeTaxReturnFiledAfterForm1139OrForm1045Violation,
    IncomeTaxReturnNotYetFiledViolation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAspect {
    TentativeCarrybackEligibilityAndProcedureCheck,
    TwelveMonthFilingDeadlineCheck,
    IrsExamination90DayWindowCheck,
    ConsolidatedReturnSpecialRulesUnderSection6411C,
    ClaimOfRightTentativeRefundUnderSection6411D,
    SubsequentDisallowanceRiskAnalysis,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section6411Mode {
    NotApplicableNotEligibleEntity,
    NotApplicableNoCarrybackAmount,
    NotApplicableForm1139Or1045NotFiled,
    NotApplicableIncomeTaxReturnNotFiledBeforeForm,
    NotApplicableConsolidatedReturnSpecialRulesApply,
    CompliantQuickRefundApplicationFiledTimelyOnForm1139OrForm1045,
    CompliantNetOperatingLossCarrybackUnderSection172,
    CompliantNetCapitalLossCarrybackUnderSection1212,
    CompliantUnusedBusinessCreditCarrybackUnderSection39,
    CompliantClaimOfRightAdjustmentUnderSection6411DAndSection1341B1,
    CompliantIrsProcessedWithin90Days,
    CompliantNotConstrutedAsRefundClaimUnderSection6511,
    ViolationApplicationFiledAfter12MonthDeadline,
    ViolationIncomeTaxReturnFiledAfterApplication,
    ViolationIrsProcessingExceeded90Days,
    ViolationSubsequentlyDisallowedTriggeringSection6213B3DeficiencyProcedure,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub taxpayer_type: TaxpayerType,
    pub application_form_status: ApplicationFormStatus,
    pub carryback_category: CarrybackCategory,
    pub application_filing_months_after_taxable_year_end: u32,
    pub income_tax_return_status: IncomeTaxReturnStatus,
    pub irs_processing_days: u32,
    pub consolidated_return: bool,
    pub subsequently_disallowed: bool,
    pub compliance_aspect: ComplianceAspect,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section6411Mode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type Section6411Input = Input;
pub type Section6411Output = Output;
pub type Section6411Result = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "IRC § 6411(a) Right to Tentative Adjustment — taxpayer may file application for tentative carryback adjustment of tax for any taxable year affected by NOL carryback (§ 172), net capital loss carryback (§ 1212), or unused business credit carryback (§ 39); corporations file FORM 1139 (Corporation Application for Tentative Refund); individuals, estates, trusts file FORM 1045 (Application for Tentative Refund)".to_string(),
        "IRC § 6411(a) Filing Deadline — application must be filed within 12 MONTHS after end of taxable year in which NOL, net capital loss, or unused business credit AROSE; taxpayer must have FILED income tax return for loss/credit year NO LATER THAN date Form 1139 / Form 1045 application is filed".to_string(),
        "IRC § 6411(b) IRS Examination Window — within 90 DAYS from date application filed, Secretary shall (1) make LIMITED EXAMINATION for omissions and computational errors; (2) determine amount of decrease in tax attributable to carryback / unused credit; (3) apply decrease against unpaid amounts due; (4) refund remainder within 90-day window".to_string(),
        "IRC § 6411(c) Consolidated Return Special Rules — special limitations apply when corporations file consolidated returns for either loss year or affected prior year".to_string(),
        "IRC § 6411(d) Claim of Right Tentative Refund — added by Public Law 95-628 of 1978; permits tentative refund under § 1341(b)(1) for amounts repaid under claim of right within 12 MONTHS from last day of such taxable year".to_string(),
        "NOT a Refund Claim under § 6511 — application under § 6411 SHALL NOT constitute claim for credit or refund under § 6511; § 6411 tentative refund process distinct from standard § 6511 refund-claim procedure".to_string(),
        "Categories Eligible — (1) § 172 NOL; (2) § 1212 Net Capital Loss (corporations: 3-year carryback, 5-year carryforward); (3) § 39 Unused Business Credit (1-year carryback, 20-year carryforward); (4) § 1341(b)(1) Claim of Right Adjustment under § 6411(d)".to_string(),
        "Subsequent Disallowance Risk — if IRS subsequently disallows tentative carryback adjustment, excessive refund treated under § 6213(b)(3) as deficiency that may be assessed WITHOUT normal § 6213(a) Tax Court petition right; interest at § 6601 underpayment rate accrues from original refund date".to_string(),
        "CARES Act 2020 Updates — Public Law 116-136 (March 27, 2020) temporarily allowed corporations to carry back NOLs arising in taxable years beginning in 2018, 2019, or 2020 to each of FIVE preceding taxable years; IRS Notice 2020-26 extended Form 1139 / Form 1045 deadlines for tax-year-2018 NOLs by 6 months; CARES Act provisions sunset for losses arising in 2021 and later".to_string(),
        "Penalty Considerations — filing fraudulent Form 1139 or Form 1045 may trigger § 7206 criminal penalties for fraud and false statements; substantial understatements may trigger § 6662 accuracy-related penalties".to_string(),
        "Companion Provisions — § 6425 (built iter 684; corporate estimated tax quick refund — 45-day window vs § 6411's 90-day window); § 6601 (built earlier; underpayment interest); § 6611 (overpayment interest); § 6621 (built iter 674; underpayment rate); § 6655 (built iter 676; corporate estimated tax penalty); § 6511 (refund claim limitations — DISTINCT from § 6411); § 6213(b)(3) (special rule for tentative carryback assessments); § 172 (NOL); § 1212 (net capital loss); § 39 (unused business credit); § 1341 (claim of right); § 6662 (accuracy-related penalty); § 7206 (fraud and false statements)".to_string(),
        "Cornell LII 26 USC § 6411 — primary statutory text".to_string(),
        "Bloomberg Tax Sec. 6411 — comprehensive code commentary".to_string(),
        "IRS Instructions for Form 1139 (Rev. December 2025) — corporate quick refund instructions".to_string(),
        "IRS Notice 2020-26 — Extension of Time to File Form 1139 / 1045 under CARES Act".to_string(),
        "IRM 21.5.9 — Carrybacks (IRS examiner manual)".to_string(),
        "eCFR — 26 CFR 5.6411-1 Tentative Refund Under Claim of Right Adjustment".to_string(),
    ];

    if input.taxpayer_type == TaxpayerType::OtherEntityNotEligibleForSection6411 {
        return Output {
            mode: Section6411Mode::NotApplicableNotEligibleEntity,
            statutory_basis: "IRC § 6411 — applies only to corporations (Form 1139), individuals, estates, and trusts (Form 1045)".to_string(),
            notes: "NOT APPLICABLE: taxpayer is not an entity eligible for § 6411 tentative carryback adjustment procedure; partnerships and S corporations generally pass losses through to partners / shareholders who file their own Form 1045s.".to_string(),
            citations,
        };
    }

    if input.carryback_category == CarrybackCategory::NoCarrybackAmount {
        return Output {
            mode: Section6411Mode::NotApplicableNoCarrybackAmount,
            statutory_basis: "IRC § 6411(a) — applies only to NOL / net capital loss / unused business credit carrybacks or § 1341 claim-of-right adjustments".to_string(),
            notes: "NOT APPLICABLE: no carryback amount to apply; § 6411 procedure unavailable.".to_string(),
            citations,
        };
    }

    if input.application_form_status == ApplicationFormStatus::OtherFormOrNoApplicationFiled {
        return Output {
            mode: Section6411Mode::NotApplicableForm1139Or1045NotFiled,
            statutory_basis: "IRC § 6411(a) — application required on Form 1139 (corporation) or Form 1045 (individual, estate, trust)".to_string(),
            notes: "NOT APPLICABLE: no Form 1139 / Form 1045 application filed; § 6411 tentative carryback procedure not invoked; taxpayer must use standard amended return / § 6511 refund claim procedure.".to_string(),
            citations,
        };
    }

    match input.compliance_aspect {
        ComplianceAspect::TentativeCarrybackEligibilityAndProcedureCheck => {
            if input.application_filing_months_after_taxable_year_end
                > IRC_6411_FILING_DEADLINE_MONTHS_AFTER_TAXABLE_YEAR_END
            {
                return Output {
                    mode: Section6411Mode::ViolationApplicationFiledAfter12MonthDeadline,
                    statutory_basis: "IRC § 6411(a) — application must be filed within 12 months after end of taxable year of loss / credit".to_string(),
                    notes: format!(
                        "VIOLATION: application filed {} months after end of taxable year of loss / unused credit; § 6411(a) requires filing within 12 months; tentative carryback unavailable; taxpayer must file standard refund claim under § 6511.",
                        input.application_filing_months_after_taxable_year_end
                    ),
                    citations,
                };
            }
            if input.income_tax_return_status
                != IncomeTaxReturnStatus::IncomeTaxReturnFiledBeforeOrConcurrentlyWithForm1139OrForm1045
            {
                return Output {
                    mode: Section6411Mode::ViolationIncomeTaxReturnFiledAfterApplication,
                    statutory_basis: "IRC § 6411(a) — taxpayer must file income tax return for loss/credit year no later than Form 1139 / Form 1045 filing date".to_string(),
                    notes: "VIOLATION: income tax return for loss / credit taxable year NOT filed before or concurrently with Form 1139 / Form 1045 application; § 6411(a) sequencing requirement violated; tentative carryback unavailable.".to_string(),
                    citations,
                };
            }
            if input.consolidated_return {
                return Output {
                    mode: Section6411Mode::NotApplicableConsolidatedReturnSpecialRulesApply,
                    statutory_basis: "IRC § 6411(c) — special limitations apply to corporations filing consolidated returns".to_string(),
                    notes: "CONSOLIDATED RETURN: § 6411(c) special limitations apply for consolidated returns; standard tentative carryback procedure modified; specific intercompany / consolidated computation rules under Treas. Reg. § 1.1502-78 govern.".to_string(),
                    citations,
                };
            }
            match input.carryback_category {
                CarrybackCategory::NetOperatingLossCarrybackUnderSection172 => Output {
                    mode: Section6411Mode::CompliantNetOperatingLossCarrybackUnderSection172,
                    statutory_basis: "IRC § 6411(a) + § 172 — NOL carryback eligible for tentative refund".to_string(),
                    notes: format!(
                        "COMPLIANT: § 172 NOL carryback Form {} application filed within 12-month deadline ({} months after end of loss year); income tax return for loss year filed before or concurrently; tentative refund will be processed within 90 days under § 6411(b).",
                        match input.application_form_status {
                            ApplicationFormStatus::Form1139CorporationApplicationFiled => "1139",
                            ApplicationFormStatus::Form1045IndividualEstateOrTrustApplicationFiled => "1045",
                            ApplicationFormStatus::OtherFormOrNoApplicationFiled => "(none)",
                        },
                        input.application_filing_months_after_taxable_year_end
                    ),
                    citations,
                },
                CarrybackCategory::NetCapitalLossCarrybackUnderSection1212 => Output {
                    mode: Section6411Mode::CompliantNetCapitalLossCarrybackUnderSection1212,
                    statutory_basis: "IRC § 6411(a) + § 1212 — net capital loss carryback eligible for tentative refund (corporations: 3-year carryback)".to_string(),
                    notes: "COMPLIANT: § 1212 net capital loss carryback application filed timely; corporations carry back 3 years and forward 5 years; tentative refund will be processed within 90 days.".to_string(),
                    citations,
                },
                CarrybackCategory::UnusedBusinessCreditCarrybackUnderSection39 => Output {
                    mode: Section6411Mode::CompliantUnusedBusinessCreditCarrybackUnderSection39,
                    statutory_basis: "IRC § 6411(a) + § 39 — unused business credit carryback eligible for tentative refund (1-year carryback)".to_string(),
                    notes: "COMPLIANT: § 39 unused business credit carryback application filed timely; 1-year carryback and 20-year carryforward; tentative refund will be processed within 90 days.".to_string(),
                    citations,
                },
                CarrybackCategory::ClaimOfRightAdjustmentUnderSection1341BSection6411D => Output {
                    mode: Section6411Mode::CompliantClaimOfRightAdjustmentUnderSection6411DAndSection1341B1,
                    statutory_basis: "IRC § 6411(d) + § 1341(b)(1) — claim-of-right tentative refund (added by 1978 amendment)".to_string(),
                    notes: "COMPLIANT: § 1341 claim-of-right adjustment tentative refund application filed within 12 months from last day of taxable year of repayment; § 6411(d) procedure available.".to_string(),
                    citations,
                },
                CarrybackCategory::NoCarrybackAmount => unreachable!(),
            }
        }
        ComplianceAspect::TwelveMonthFilingDeadlineCheck => {
            if input.application_filing_months_after_taxable_year_end
                > IRC_6411_FILING_DEADLINE_MONTHS_AFTER_TAXABLE_YEAR_END
            {
                Output {
                    mode: Section6411Mode::ViolationApplicationFiledAfter12MonthDeadline,
                    statutory_basis: "IRC § 6411(a) — 12-month filing deadline".to_string(),
                    notes: format!(
                        "VIOLATION: application filed {} months after end of taxable year of loss / credit; exceeds § 6411(a) 12-month deadline.",
                        input.application_filing_months_after_taxable_year_end
                    ),
                    citations,
                }
            } else {
                Output {
                    mode: Section6411Mode::CompliantQuickRefundApplicationFiledTimelyOnForm1139OrForm1045,
                    statutory_basis: "IRC § 6411(a) — application filed within 12 months after end of loss / credit taxable year".to_string(),
                    notes: format!(
                        "COMPLIANT: Form 1139 / Form 1045 application filed {} months after end of loss / credit taxable year (≤ 12-month deadline).",
                        input.application_filing_months_after_taxable_year_end
                    ),
                    citations,
                }
            }
        }
        ComplianceAspect::IrsExamination90DayWindowCheck => {
            if input.irs_processing_days > IRC_6411_IRS_EXAMINATION_WINDOW_DAYS {
                Output {
                    mode: Section6411Mode::ViolationIrsProcessingExceeded90Days,
                    statutory_basis: "IRC § 6411(b) — IRS must process application within 90 days".to_string(),
                    notes: format!(
                        "VIOLATION OF AGENCY DEADLINE: IRS processing took {} days; § 6411(b) requires limited examination, adjustment determination, credit / refund within 90 days; taxpayer may pursue Taxpayer Advocate Service or formal complaint.",
                        input.irs_processing_days
                    ),
                    citations,
                }
            } else {
                Output {
                    mode: Section6411Mode::CompliantIrsProcessedWithin90Days,
                    statutory_basis: "IRC § 6411(b) — IRS processed application within 90 days".to_string(),
                    notes: format!(
                        "COMPLIANT: IRS processed Form 1139 / Form 1045 application within {} days (≤ 90-day statutory window).",
                        input.irs_processing_days
                    ),
                    citations,
                }
            }
        }
        ComplianceAspect::ConsolidatedReturnSpecialRulesUnderSection6411C => {
            if input.consolidated_return {
                Output {
                    mode: Section6411Mode::NotApplicableConsolidatedReturnSpecialRulesApply,
                    statutory_basis: "IRC § 6411(c) + Treas. Reg. § 1.1502-78 — consolidated return special limitations".to_string(),
                    notes: "CONSOLIDATED RETURN: § 6411(c) special limitations apply; consolidated tentative carryback adjustment must account for intercompany transactions and consolidated taxable income / loss aggregations under Treas. Reg. § 1.1502-78.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: Section6411Mode::CompliantQuickRefundApplicationFiledTimelyOnForm1139OrForm1045,
                    statutory_basis: "IRC § 6411 — non-consolidated return; standard procedure applies".to_string(),
                    notes: "NOT TRIGGERED: taxpayer does not file consolidated return; § 6411(c) special limitations do not apply; standard § 6411(a) / (b) tentative carryback procedure applies.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::ClaimOfRightTentativeRefundUnderSection6411D => Output {
            mode: Section6411Mode::CompliantClaimOfRightAdjustmentUnderSection6411DAndSection1341B1,
            statutory_basis: "IRC § 6411(d) + § 1341(b)(1) — claim-of-right tentative refund".to_string(),
            notes: "COMPLIANT: § 6411(d) claim-of-right tentative refund procedure invoked for § 1341(b)(1) adjustment; application must be filed within 12 months from last day of taxable year of repayment; added by Public Law 95-628 of 1978.".to_string(),
            citations,
        },
        ComplianceAspect::SubsequentDisallowanceRiskAnalysis => {
            if input.subsequently_disallowed {
                Output {
                    mode: Section6411Mode::ViolationSubsequentlyDisallowedTriggeringSection6213B3DeficiencyProcedure,
                    statutory_basis: "IRC § 6213(b)(3) — special rule for tentative carryback adjustments; excessive tentative refund treated as deficiency assessable without normal § 6213(a) Tax Court petition right".to_string(),
                    notes: "VIOLATION OF SAFE-REFUND TREATMENT: tentative carryback refund subsequently disallowed by IRS examination; § 6213(b)(3) treats excessive refund as deficiency that may be assessed without normal § 6213(a) Tax Court petition right; interest at § 6601 underpayment rate accrues from original refund date; taxpayer may file refund claim under § 6511 if disallowance is final.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: Section6411Mode::CompliantNotConstrutedAsRefundClaimUnderSection6511,
                    statutory_basis: "IRC § 6411 — application not a refund claim under § 6511; distinct procedural track".to_string(),
                    notes: "COMPLIANT: § 6411 tentative refund processed without subsequent disallowance; application not constituted as refund claim under § 6511; distinct procedural track preserved.".to_string(),
                    citations,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_input() -> Input {
        Input {
            taxpayer_type: TaxpayerType::CorporationFilesForm1139,
            application_form_status: ApplicationFormStatus::Form1139CorporationApplicationFiled,
            carryback_category: CarrybackCategory::NetOperatingLossCarrybackUnderSection172,
            application_filing_months_after_taxable_year_end: 6,
            income_tax_return_status:
                IncomeTaxReturnStatus::IncomeTaxReturnFiledBeforeOrConcurrentlyWithForm1139OrForm1045,
            irs_processing_days: 60,
            consolidated_return: false,
            subsequently_disallowed: false,
            compliance_aspect: ComplianceAspect::TentativeCarrybackEligibilityAndProcedureCheck,
        }
    }

    #[test]
    fn not_eligible_entity_not_applicable() {
        let mut input = baseline_input();
        input.taxpayer_type = TaxpayerType::OtherEntityNotEligibleForSection6411;
        let output = check(&input);
        assert_eq!(output.mode, Section6411Mode::NotApplicableNotEligibleEntity);
    }

    #[test]
    fn no_carryback_amount_not_applicable() {
        let mut input = baseline_input();
        input.carryback_category = CarrybackCategory::NoCarrybackAmount;
        let output = check(&input);
        assert_eq!(output.mode, Section6411Mode::NotApplicableNoCarrybackAmount);
    }

    #[test]
    fn form_not_filed_not_applicable() {
        let mut input = baseline_input();
        input.application_form_status = ApplicationFormStatus::OtherFormOrNoApplicationFiled;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section6411Mode::NotApplicableForm1139Or1045NotFiled
        );
    }

    #[test]
    fn corporation_nol_carryback_form_1139_compliant() {
        let output = check(&baseline_input());
        assert_eq!(
            output.mode,
            Section6411Mode::CompliantNetOperatingLossCarrybackUnderSection172
        );
    }

    #[test]
    fn individual_nol_carryback_form_1045_compliant() {
        let mut input = baseline_input();
        input.taxpayer_type = TaxpayerType::IndividualFilesForm1045;
        input.application_form_status =
            ApplicationFormStatus::Form1045IndividualEstateOrTrustApplicationFiled;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section6411Mode::CompliantNetOperatingLossCarrybackUnderSection172
        );
    }

    #[test]
    fn application_after_12_month_deadline_violation() {
        let mut input = baseline_input();
        input.application_filing_months_after_taxable_year_end = 15;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section6411Mode::ViolationApplicationFiledAfter12MonthDeadline
        );
    }

    #[test]
    fn application_at_exactly_12_months_compliant_boundary() {
        let mut input = baseline_input();
        input.application_filing_months_after_taxable_year_end = 12;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section6411Mode::CompliantNetOperatingLossCarrybackUnderSection172
        );
    }

    #[test]
    fn application_at_13_months_violation_boundary() {
        let mut input = baseline_input();
        input.application_filing_months_after_taxable_year_end = 13;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section6411Mode::ViolationApplicationFiledAfter12MonthDeadline
        );
    }

    #[test]
    fn income_tax_return_filed_after_application_violation() {
        let mut input = baseline_input();
        input.income_tax_return_status =
            IncomeTaxReturnStatus::IncomeTaxReturnFiledAfterForm1139OrForm1045Violation;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section6411Mode::ViolationIncomeTaxReturnFiledAfterApplication
        );
    }

    #[test]
    fn consolidated_return_special_rules_applied() {
        let mut input = baseline_input();
        input.consolidated_return = true;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section6411Mode::NotApplicableConsolidatedReturnSpecialRulesApply
        );
    }

    #[test]
    fn net_capital_loss_carryback_compliant() {
        let mut input = baseline_input();
        input.carryback_category = CarrybackCategory::NetCapitalLossCarrybackUnderSection1212;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section6411Mode::CompliantNetCapitalLossCarrybackUnderSection1212
        );
    }

    #[test]
    fn unused_business_credit_carryback_compliant() {
        let mut input = baseline_input();
        input.carryback_category = CarrybackCategory::UnusedBusinessCreditCarrybackUnderSection39;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section6411Mode::CompliantUnusedBusinessCreditCarrybackUnderSection39
        );
    }

    #[test]
    fn claim_of_right_adjustment_section_6411d_compliant() {
        let mut input = baseline_input();
        input.carryback_category =
            CarrybackCategory::ClaimOfRightAdjustmentUnderSection1341BSection6411D;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section6411Mode::CompliantClaimOfRightAdjustmentUnderSection6411DAndSection1341B1
        );
    }

    #[test]
    fn irs_processed_within_90_days_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::IrsExamination90DayWindowCheck;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section6411Mode::CompliantIrsProcessedWithin90Days
        );
    }

    #[test]
    fn irs_processed_at_exactly_90_days_compliant_boundary() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::IrsExamination90DayWindowCheck;
        input.irs_processing_days = 90;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section6411Mode::CompliantIrsProcessedWithin90Days
        );
    }

    #[test]
    fn irs_processing_exceeds_90_days_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::IrsExamination90DayWindowCheck;
        input.irs_processing_days = 120;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section6411Mode::ViolationIrsProcessingExceeded90Days
        );
    }

    #[test]
    fn subsequently_disallowed_section_6213b3_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SubsequentDisallowanceRiskAnalysis;
        input.subsequently_disallowed = true;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section6411Mode::ViolationSubsequentlyDisallowedTriggeringSection6213B3DeficiencyProcedure
        );
    }

    #[test]
    fn not_disallowed_not_refund_claim_under_section_6511_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SubsequentDisallowanceRiskAnalysis;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section6411Mode::CompliantNotConstrutedAsRefundClaimUnderSection6511
        );
    }

    #[test]
    fn constants_pin_statutory_facts() {
        assert_eq!(IRC_6411_FILING_DEADLINE_MONTHS_AFTER_TAXABLE_YEAR_END, 12);
        assert_eq!(IRC_6411_IRS_EXAMINATION_WINDOW_DAYS, 90);
        assert_eq!(IRC_6411_CLAIM_OF_RIGHT_DEADLINE_MONTHS, 12);
        assert_eq!(IRC_6411_1978_AMENDMENT_YEAR, 1978);
        assert_eq!(IRC_6411_CARES_ACT_NOL_CARRYBACK_YEARS, 5);
        assert_eq!(IRC_6411_CARES_ACT_TAX_YEAR_START_YEAR, 2018);
        assert_eq!(IRC_6411_CARES_ACT_TAX_YEAR_END_YEAR, 2020);
        assert_eq!(IRC_6411_BASIS_POINT_DENOMINATOR, 10_000);
    }

    #[test]
    fn citation_contains_landmarks() {
        let output = check(&baseline_input());
        let joined = output.citations.join("\n");
        assert!(joined.contains("§ 6411(a)"));
        assert!(joined.contains("§ 6411(b)"));
        assert!(joined.contains("§ 6411(c)"));
        assert!(joined.contains("§ 6411(d)"));
        assert!(joined.contains("FORM 1139"));
        assert!(joined.contains("FORM 1045"));
        assert!(joined.contains("12 MONTHS"));
        assert!(joined.contains("90 DAYS"));
        assert!(joined.contains("§ 172"));
        assert!(joined.contains("§ 1212"));
        assert!(joined.contains("§ 39"));
        assert!(joined.contains("§ 1341"));
        assert!(joined.contains("§ 6213(b)(3)"));
        assert!(joined.contains("§ 6511"));
        assert!(joined.contains("CARES Act"));
        assert!(joined.contains("1978"));
    }
}
