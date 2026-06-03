//! IRC § 451(c) Special Rule for Advance Payments —
//! pure-compute compliance check for accrual-method
//! taxpayer treatment of advance payments under the
//! Tax Cuts and Jobs Act of 2017 reform.
//!
//! Originally enacted by **Section 13221 of the Tax Cuts
//! and Jobs Act of 2017 (Public Law 115-97)**, signed by
//! President Donald Trump on **December 22, 2017**, with
//! effective date for tax years beginning after December
//! 31, 2017. § 451(c) replaced prior Rev. Proc. 2004-34 +
//! Notice 2018-35 with statutory codification of the
//! one-year deferral method for advance payments. Treas.
//! Reg. § 1.451-8 — proposed September 9, 2019 (84 FR
//! 47175); finalized January 6, 2021 (T.D. 9941; 86 FR
//! 810); applies to tax years beginning on or after
//! January 1, 2021. Rev. Proc. 2021-34 (August 2021)
//! provides change-of-accounting-method procedures.
//!
//! Trader/business-critical for any accrual-method
//! taxpayer with subscription / prepaid / gift card /
//! warranty / membership / loyalty-program revenue
//! recognition — § 451(c) sets the maximum tax-deferral
//! horizon at ONE YEAR after year of receipt for
//! payments included in AFS revenue in a subsequent
//! year.
//!
//! Web research (verified 2026-06-03):
//! - **Enactment**: IRC § 451(c) added by **Section 13221 of the Tax Cuts and Jobs Act of 2017 (Public Law 115-97, 131 Stat. 2054)**; signed by President Donald Trump on **December 22, 2017**; effective for tax years beginning after **December 31, 2017** ([Cornell LII — 26 CFR § 1.451-8](https://www.law.cornell.edu/cfr/text/26/1.451-8); [BDO — Final Section 451 Regulations Provide Additional Clarity and Limited Relief for Accrual Method Taxpayers](https://www.bdo.com/insights/tax/final-section-451-regulations-provide-additional-clarity-and-limited-relief-for-accrual-method-taxpa); [LegalClarity — Section 451 and the One-Year Deferral for Deferred Revenue](https://legalclarity.org/section-451-and-the-one-year-deferral-for-deferred-revenue/); [IRS — Revenue Procedure 2019-37 (Administrative, Procedural, and Miscellaneous)](https://www.irs.gov/pub/irs-drop/rp-19-37.pdf); [LegalClarity — Revenue Recognition for Tax Purposes](https://legalclarity.org/revenue-recognition-for-tax-purposes/); [BDO — Final Regulations Remove Two-Year Deferral of Advance Payments for Inventoriable Goods Under Section 1.451-5](https://www.bdo.com/insights/tax/final-regulations-remove-two-year-deferral-of-advance-payments-for-inventoriable-goods-under-section); [BDO — Proposed Removal of Section 451 Regs Resulting From Changes Made Under Tax Reform](https://www.bdo.com/insights/tax/proposed-removal-of-section-451-regs-resulting-from-changes-made-under-tax-reform,-to-limit-advance); [Accounting Insights — Section 451 and the Timing of Income Recognition](https://accountinginsights.org/section-451-and-the-timing-of-income-recognition/); [Thomson Reuters — Transitional Guidance Issued on the Treatment of Advance Payments Under TCJA](https://tax.thomsonreuters.com/news/transitional-guidance-issued-on-the-treatment-of-advance-payments-under-tcja/); [The Tax Adviser — Sec. 451 Regulations Offer Ways to Reduce Revenue Acceleration](https://www.thetaxadviser.com/issues/2022/jul/sec-451-regulations-reduce-revenue-acceleration/); [Journal of Accountancy — Final Rules Govern Income Inclusion and Advanced Payments](https://www.journalofaccountancy.com/news/2021/jan/irs-rules-income-inclusion-advanced-payments/); [Grant Thornton — Final Regulations Help Taxpayers Compute Revenue](https://grantthornton.com/insights/alerts/tax/2021/insights/final-regulations-help-taxpayers-compute-revenue); [KPMG — Final Regulations on Changes to Income (January 6, 2021)](https://kpmg.com/kpmg-us/content/dam/kpmg/taxnewsflash/pdf/2021/01/tnf-451-final-regs-jan6-2021.pdf); [Civic Research Institute — Final Regulations Under Section 451 Help Taxpayers Compute Revenue](https://www.civicresearchinstitute.com/online/PDF/JTI-3902-01-Reiris-451.pdf); [Federal Register — Taxable Year of Income Inclusion Under an Accrual Method of Accounting and Advance Payments for Goods, Services, and Other Items (January 6, 2021)](https://www.federalregister.gov/documents/2021/01/06/2020-28653/taxable-year-of-income-inclusion-under-an-accrual-method-of-accounting-and-advance-payments-for); [EY Tax News — Final Section 451 Regulations Provide New Rules for Timing of Income Recognition and Treatment of Advance Payments](https://taxnews.ey.com/news/2021-0062-final-section-451-regulations-provide-new-rules-for-timing-of-income-recognition-and-treatment-of-advance-payments); [Federal Register — Advance Payments for Goods, Services, and Other Items (September 9, 2019 Proposed)](https://www.federalregister.gov/documents/2019/09/09/2019-19197/advance-payments-for-goods-services-and-other-items); [The Tax Adviser — Advance Payments for Goods and Services (February 2020)](https://www.thetaxadviser.com/issues/2020/feb/advance-payments-goods-services/); [Federal Register — Regulations Regarding Advance Payments for Goods and Long-Term Contracts (July 15, 2019)](https://www.federalregister.gov/documents/2019/07/15/2019-14947/regulations-regarding-advance-payments-for-goods-and-long-term-contracts); [The Tax Adviser — Acceleration of Deferred Revenue in M&As](https://www.thetaxadviser.com/issues/2021/jul/acceleration-deferred-revenue-mergers-acquisitions/); [IRS — Notice 2018-35](https://www.irs.gov/pub/irs-drop/n-18-35.pdf); [GovInfo — 26 CFR § 1.451-8 (CFR-2022-title26-vol8)](https://www.govinfo.gov/content/pkg/CFR-2022-title26-vol8/pdf/CFR-2022-title26-vol8-sec1-451-8.pdf)).
//! - **§ 451(c)(1)(A) General Rule**: an accrual method taxpayer must include any **ADVANCE PAYMENT** in income in the **TAXABLE YEAR OF RECEIPT**.
//! - **§ 451(c)(1)(B) AFS Deferral Method Election**: taxpayer may elect to include in income the portion of the advance payment included in AFS revenue for the taxable year of receipt + include the **REMAINDER in the SUCCEEDING TAXABLE YEAR** (maximum 1-YEAR deferral).
//! - **§ 451(c)(4)(A) Advance Payment Definition**: payment that satisfies all three conditions — (i) full inclusion in income in year of receipt is a permissible method; (ii) at least a portion is included in revenue in AFS for a subsequent taxable year (or, absent AFS, earned by the taxpayer in a subsequent taxable year); (iii) payment is for **GOODS + SERVICES + USE OF INTELLECTUAL PROPERTY + SOFTWARE + GIFT CARDS + CERTAIN SUBSCRIPTIONS + CERTAIN WARRANTY CONTRACTS + MEMBERSHIPS + USE OF PROPERTY ANCILLARY TO SERVICES + REWARD AND LOYALTY PROGRAMS** (per Treas. Reg. § 1.451-8(a)(1)).
//! - **§ 451(b)(3) Applicable Financial Statement Definition**: GAAP-basis financial statement filed with SEC (Form 10-K + audited consolidated statement of financial position) + certified audited financial statement using GAAP + similar tiered statements.
//! - **Treas. Reg. § 1.451-8 Final Regulations**: proposed **September 9, 2019** (84 FR 47175); finalized **JANUARY 6, 2021** (T.D. 9941; 86 FR 810); applies to tax years beginning on or after **JANUARY 1, 2021**.
//! - **Rev. Proc. 2021-34 Method Change Procedures**: released August 2021; provides automatic consent procedures for taxpayers to change methods of accounting to comply with § 451(c) amendments and Treas. Reg. § 1.451-8 final regulations.
//! - **Obsoleted Authorities**: Rev. Proc. 2004-34 + Notice 2018-35 obsoleted for tax years beginning on or after January 1, 2021 (replaced by Treas. Reg. § 1.451-8 codification of one-year deferral method).
//! - **Acceleration on Sale / Liquidation**: deferred advance payment balance is accelerated into income upon (a) cessation of taxpayer's trade or business; (b) bankruptcy; (c) certain M&A transactions per Treas. Reg. § 1.451-8(f).

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const IRC_451C_TCJA_ENACTMENT_DATE_YEAR: u32 = 2017;
pub const IRC_451C_TCJA_ENACTMENT_DATE_MONTH: u32 = 12;
pub const IRC_451C_TCJA_ENACTMENT_DATE_DAY: u32 = 22;
pub const IRC_451C_TCJA_PUBLIC_LAW_CONGRESS: u32 = 115;
pub const IRC_451C_TCJA_PUBLIC_LAW_ENACTMENT: u32 = 97;
pub const IRC_451C_TCJA_ENABLING_SECTION: u32 = 13221;
pub const IRC_451C_EFFECTIVE_FOR_TAX_YEARS_BEGINNING_AFTER_DATE_YEAR: u32 = 2017;
pub const IRC_451C_EFFECTIVE_FOR_TAX_YEARS_BEGINNING_AFTER_DATE_MONTH: u32 = 12;
pub const IRC_451C_EFFECTIVE_FOR_TAX_YEARS_BEGINNING_AFTER_DATE_DAY: u32 = 31;
pub const IRC_451C_MAXIMUM_DEFERRAL_YEARS: u32 = 1;
pub const IRC_451C_FINAL_REGS_TD_NUMBER: u32 = 9941;
pub const IRC_451C_FINAL_REGS_FEDERAL_REGISTER_PUBLICATION_DATE_YEAR: u32 = 2021;
pub const IRC_451C_FINAL_REGS_FEDERAL_REGISTER_PUBLICATION_DATE_MONTH: u32 = 1;
pub const IRC_451C_FINAL_REGS_FEDERAL_REGISTER_PUBLICATION_DATE_DAY: u32 = 6;
pub const IRC_451C_FINAL_REGS_EFFECTIVE_DATE_YEAR: u32 = 2021;
pub const IRC_451C_FINAL_REGS_EFFECTIVE_DATE_MONTH: u32 = 1;
pub const IRC_451C_FINAL_REGS_EFFECTIVE_DATE_DAY: u32 = 1;
pub const IRC_451C_FINAL_REGS_FEDERAL_REGISTER_VOLUME: u32 = 86;
pub const IRC_451C_FINAL_REGS_FEDERAL_REGISTER_PAGE: u32 = 810;
pub const IRC_451C_PROPOSED_REGS_FEDERAL_REGISTER_VOLUME: u32 = 84;
pub const IRC_451C_PROPOSED_REGS_FEDERAL_REGISTER_PAGE: u32 = 47_175;
pub const IRC_451C_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountingMethod {
    AccrualMethodWithApplicableFinancialStatement,
    AccrualMethodWithoutApplicableFinancialStatement,
    CashMethodNotSubjectToSection451CRule,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentCategory {
    GoodsSale,
    Services,
    UseOfIntellectualProperty,
    SoftwareLicense,
    GiftCard,
    CertainSubscriptionsOrMemberships,
    CertainWarrantyContracts,
    UseOfPropertyAncillaryToServices,
    RewardAndLoyaltyProgram,
    NotEligibleAdvancePaymentCategory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAspect {
    FullInclusionInYearOfReceiptUnderSection451C1A,
    OneYearDeferralElectionUnderSection451C1B,
    EligibleAdvancePaymentCategoryUnderTreasReg1_451_8A1,
    MethodChangeUnderRevProc2021_34,
    AccelerationOnCessationOrMergerUnderTreasReg1_451_8F,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section451CMode {
    NotApplicableCashMethodTaxpayerNotSubjectToSection451CRule,
    NotApplicableNotAnAdvancePayment,
    NotApplicableTaxYearBeginningOnOrBeforeDecember31_2017PreTcja,
    CompliantFullInclusionInYearOfReceiptUnderSection451C1A,
    CompliantOneYearDeferralElectionAppliedCorrectlyUnderSection451C1B,
    CompliantPartialAfsRecognitionPlusOneYearDeferralOfRemainder,
    CompliantEligibleAdvancePaymentCategoryUnderTreasReg1_451_8A1,
    CompliantMethodChangeFiledUnderRevProc2021_34,
    CompliantAccelerationOnCessationOrMergerHandled,
    ViolationDeferralBeyondOneYearMaximum,
    ViolationNonEligibleAdvancePaymentCategoryDeferred,
    ViolationAfsAmountNotIncludedInYearOfReceiptUnderAfsDeferralMethod,
    ViolationContinuedUseOfObsoleteRevProc2004_34PostJanuary1_2021,
    ViolationFailureToFileRequiredMethodChangeUnderRevProc2021_34,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub accounting_method: AccountingMethod,
    pub payment_category: PaymentCategory,
    pub compliance_aspect: ComplianceAspect,
    pub tax_year_begins_after_december_31_2017: bool,
    pub tax_year_begins_on_or_after_january_1_2021: bool,
    pub afs_amount_included_in_year_of_receipt: bool,
    pub deferral_years_after_receipt: u32,
    pub method_change_filed_under_rev_proc_2021_34: bool,
    pub deferred_balance_accelerated_on_cessation_or_merger: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section451CMode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type Section451CInput = Input;
pub type Section451COutput = Output;
pub type Section451CResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "IRC § 451(c) added by Section 13221 of the Tax Cuts and Jobs Act of 2017 (Public Law 115-97, 131 Stat. 2054); signed by President Donald Trump on December 22, 2017; effective for tax years beginning after December 31, 2017".to_string(),
        "IRC § 451(c)(1)(A) General Rule — an accrual method taxpayer must include any ADVANCE PAYMENT in income in the TAXABLE YEAR OF RECEIPT".to_string(),
        "IRC § 451(c)(1)(B) AFS Deferral Method Election — taxpayer may elect to include in income the portion of the advance payment included in AFS revenue for the taxable year of receipt + include the REMAINDER in the SUCCEEDING TAXABLE YEAR (maximum 1-YEAR deferral)".to_string(),
        "IRC § 451(c)(4)(A) Advance Payment Definition — payment that satisfies all three conditions: (i) full inclusion in income in year of receipt is a permissible method; (ii) at least a portion is included in revenue in AFS for a subsequent taxable year (or, absent AFS, earned by the taxpayer in a subsequent taxable year); (iii) payment is for GOODS + SERVICES + USE OF INTELLECTUAL PROPERTY + SOFTWARE + GIFT CARDS + CERTAIN SUBSCRIPTIONS + CERTAIN WARRANTY CONTRACTS + MEMBERSHIPS + USE OF PROPERTY ANCILLARY TO SERVICES + REWARD AND LOYALTY PROGRAMS".to_string(),
        "IRC § 451(b)(3) Applicable Financial Statement Definition — GAAP-basis financial statement filed with SEC (Form 10-K + audited consolidated statement of financial position) + certified audited financial statement using GAAP + similar tiered statements".to_string(),
        "Treas. Reg. § 1.451-8 Final Regulations — proposed September 9, 2019 (84 FR 47175); finalized JANUARY 6, 2021 (T.D. 9941; 86 FR 810); applies to tax years beginning on or after JANUARY 1, 2021".to_string(),
        "Treas. Reg. § 1.451-8(a)(1) — categorizes eligible advance payments and establishes the framework for accrual-method taxpayer deferral based on applicable financial statement reporting".to_string(),
        "Rev. Proc. 2021-34 Method Change Procedures — released August 2021; provides automatic consent procedures for taxpayers to change methods of accounting to comply with § 451(c) amendments and Treas. Reg. § 1.451-8 final regulations".to_string(),
        "Obsoleted Authorities — Rev. Proc. 2004-34 + Notice 2018-35 obsoleted for tax years beginning on or after January 1, 2021 (replaced by Treas. Reg. § 1.451-8 codification of one-year deferral method)".to_string(),
        "Treas. Reg. § 1.451-8(f) Acceleration on Cessation or Merger — deferred advance payment balance is accelerated into income upon (a) cessation of taxpayer's trade or business; (b) bankruptcy; (c) certain M&A transactions".to_string(),
        "Federal Register — Taxable Year of Income Inclusion Under an Accrual Method of Accounting and Advance Payments for Goods, Services, and Other Items (January 6, 2021) — official final-rule publication".to_string(),
        "BDO + Grant Thornton + KPMG + EY + Journal of Accountancy + The Tax Adviser + Civic Research Institute — practitioner overview of § 451(c) and Treas. Reg. § 1.451-8 final regs".to_string(),
    ];

    if input.accounting_method == AccountingMethod::CashMethodNotSubjectToSection451CRule {
        return Output {
            mode: Section451CMode::NotApplicableCashMethodTaxpayerNotSubjectToSection451CRule,
            statutory_basis: "IRC § 451(c) — § 451(c) applies only to accrual method taxpayers".to_string(),
            notes: "NOT APPLICABLE: taxpayer uses cash method of accounting; advance payments are includible in income when actually or constructively received under § 451(a); § 451(c) and Treas. Reg. § 1.451-8 do not apply.".to_string(),
            citations,
        };
    }

    if !input.tax_year_begins_after_december_31_2017 {
        return Output {
            mode: Section451CMode::NotApplicableTaxYearBeginningOnOrBeforeDecember31_2017PreTcja,
            statutory_basis: "TCJA 2017 § 13221 effective date — § 451(c) effective for tax years beginning after December 31, 2017".to_string(),
            notes: "NOT APPLICABLE: tax year begins on or before December 31, 2017 (pre-TCJA); § 451(c) does not yet apply; prior Rev. Proc. 2004-34 / Rev. Proc. 2013-39 one-year deferral methods govern.".to_string(),
            citations,
        };
    }

    if input.payment_category == PaymentCategory::NotEligibleAdvancePaymentCategory {
        return Output {
            mode: Section451CMode::NotApplicableNotAnAdvancePayment,
            statutory_basis: "Treas. Reg. § 1.451-8(a)(1) — eligible advance payment categories listed exhaustively; non-listed categories not subject to § 451(c) deferral".to_string(),
            notes: "NOT APPLICABLE: payment is not within an eligible advance payment category under Treas. Reg. § 1.451-8(a)(1); § 451(c) deferral method unavailable; general accrual method timing under § 451(b) applies.".to_string(),
            citations,
        };
    }

    match input.compliance_aspect {
        ComplianceAspect::FullInclusionInYearOfReceiptUnderSection451C1A => Output {
            mode: Section451CMode::CompliantFullInclusionInYearOfReceiptUnderSection451C1A,
            statutory_basis: "IRC § 451(c)(1)(A) — full inclusion of advance payment in income in year of receipt".to_string(),
            notes: "COMPLIANT: accrual method taxpayer included full advance payment amount in gross income in the taxable year of receipt under § 451(c)(1)(A) general rule.".to_string(),
            citations,
        },
        ComplianceAspect::OneYearDeferralElectionUnderSection451C1B => {
            if input.deferral_years_after_receipt <= IRC_451C_MAXIMUM_DEFERRAL_YEARS
                && input.afs_amount_included_in_year_of_receipt
            {
                Output {
                    mode: Section451CMode::CompliantPartialAfsRecognitionPlusOneYearDeferralOfRemainder,
                    statutory_basis: "IRC § 451(c)(1)(B) — AFS deferral method election applied correctly with 1-year deferral".to_string(),
                    notes: "COMPLIANT: accrual method taxpayer elected AFS deferral method under § 451(c)(1)(B); included AFS revenue amount in year of receipt and deferred remaining portion to the succeeding taxable year (within 1-year maximum deferral).".to_string(),
                    citations,
                }
            } else if input.deferral_years_after_receipt > IRC_451C_MAXIMUM_DEFERRAL_YEARS {
                Output {
                    mode: Section451CMode::ViolationDeferralBeyondOneYearMaximum,
                    statutory_basis: "IRC § 451(c)(1)(B) — deferral exceeds 1-year maximum".to_string(),
                    notes: "VIOLATION: taxpayer deferred advance payment beyond the 1-year statutory maximum under § 451(c)(1)(B); deferral capped at SUCCEEDING TAXABLE YEAR only; balance must be accelerated to year following receipt.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: Section451CMode::ViolationAfsAmountNotIncludedInYearOfReceiptUnderAfsDeferralMethod,
                    statutory_basis: "IRC § 451(c)(1)(B) — AFS amount must be included in year of receipt".to_string(),
                    notes: "VIOLATION: taxpayer elected AFS deferral method but failed to include AFS-recognized portion of advance payment in income in year of receipt; § 451(c)(1)(B) requires AFS revenue amount in year of receipt + deferral of remainder only.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::EligibleAdvancePaymentCategoryUnderTreasReg1_451_8A1 => Output {
            mode: Section451CMode::CompliantEligibleAdvancePaymentCategoryUnderTreasReg1_451_8A1,
            statutory_basis: "Treas. Reg. § 1.451-8(a)(1) — payment within an eligible advance payment category".to_string(),
            notes: "COMPLIANT: payment is within an eligible advance payment category under Treas. Reg. § 1.451-8(a)(1) (goods / services / IP use / software / gift cards / subscriptions / warranties / memberships / property use ancillary to services / loyalty programs).".to_string(),
            citations,
        },
        ComplianceAspect::MethodChangeUnderRevProc2021_34 => {
            if input.tax_year_begins_on_or_after_january_1_2021 {
                if input.method_change_filed_under_rev_proc_2021_34 {
                    Output {
                        mode: Section451CMode::CompliantMethodChangeFiledUnderRevProc2021_34,
                        statutory_basis: "Rev. Proc. 2021-34 — method change filed under automatic consent procedures".to_string(),
                        notes: "COMPLIANT: taxpayer filed method change under Rev. Proc. 2021-34 automatic consent procedures to comply with § 451(c) amendments and Treas. Reg. § 1.451-8 final regulations.".to_string(),
                        citations,
                    }
                } else {
                    Output {
                        mode: Section451CMode::ViolationContinuedUseOfObsoleteRevProc2004_34PostJanuary1_2021,
                        statutory_basis: "Rev. Proc. 2021-34 + Rev. Proc. 2004-34 obsoleted — method change required for tax years beginning on or after January 1, 2021".to_string(),
                        notes: "VIOLATION: taxpayer continued using obsoleted Rev. Proc. 2004-34 method for tax years beginning on or after January 1, 2021; Treas. Reg. § 1.451-8 codification of one-year deferral method has replaced prior authority; method change under Rev. Proc. 2021-34 required.".to_string(),
                        citations,
                    }
                }
            } else if !input.method_change_filed_under_rev_proc_2021_34 {
                Output {
                    mode: Section451CMode::ViolationFailureToFileRequiredMethodChangeUnderRevProc2021_34,
                    statutory_basis: "Rev. Proc. 2021-34 — failure to file required method change".to_string(),
                    notes: "VIOLATION: taxpayer failed to file required method change under Rev. Proc. 2021-34 to transition into Treas. Reg. § 1.451-8 framework; § 481(a) adjustment exposure.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: Section451CMode::CompliantMethodChangeFiledUnderRevProc2021_34,
                    statutory_basis: "Rev. Proc. 2021-34 — method change filed timely".to_string(),
                    notes: "COMPLIANT: taxpayer filed method change under Rev. Proc. 2021-34 in advance of January 1, 2021 final-regs effective date.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::AccelerationOnCessationOrMergerUnderTreasReg1_451_8F => {
            if input.deferred_balance_accelerated_on_cessation_or_merger {
                Output {
                    mode: Section451CMode::CompliantAccelerationOnCessationOrMergerHandled,
                    statutory_basis: "Treas. Reg. § 1.451-8(f) — deferred balance accelerated correctly on cessation or M&A".to_string(),
                    notes: "COMPLIANT: taxpayer accelerated deferred advance payment balance into income upon cessation of trade or business / bankruptcy / certain M&A transactions per Treas. Reg. § 1.451-8(f).".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: Section451CMode::ViolationDeferralBeyondOneYearMaximum,
                    statutory_basis: "Treas. Reg. § 1.451-8(f) — deferred balance not accelerated on cessation or M&A".to_string(),
                    notes: "VIOLATION: deferred advance payment balance not accelerated into income upon cessation of trade or business / bankruptcy / certain M&A transactions; Treas. Reg. § 1.451-8(f) requires acceleration; § 481(a) adjustment + accuracy-related penalty exposure.".to_string(),
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
            accounting_method: AccountingMethod::AccrualMethodWithApplicableFinancialStatement,
            payment_category: PaymentCategory::Services,
            compliance_aspect: ComplianceAspect::OneYearDeferralElectionUnderSection451C1B,
            tax_year_begins_after_december_31_2017: true,
            tax_year_begins_on_or_after_january_1_2021: true,
            afs_amount_included_in_year_of_receipt: true,
            deferral_years_after_receipt: 1,
            method_change_filed_under_rev_proc_2021_34: true,
            deferred_balance_accelerated_on_cessation_or_merger: true,
        }
    }

    #[test]
    fn cash_method_not_applicable() {
        let mut input = baseline_input();
        input.accounting_method = AccountingMethod::CashMethodNotSubjectToSection451CRule;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section451CMode::NotApplicableCashMethodTaxpayerNotSubjectToSection451CRule
        );
    }

    #[test]
    fn pre_tcja_tax_year_not_applicable() {
        let mut input = baseline_input();
        input.tax_year_begins_after_december_31_2017 = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section451CMode::NotApplicableTaxYearBeginningOnOrBeforeDecember31_2017PreTcja
        );
    }

    #[test]
    fn non_eligible_payment_category_not_applicable() {
        let mut input = baseline_input();
        input.payment_category = PaymentCategory::NotEligibleAdvancePaymentCategory;
        let output = check(&input);
        assert_eq!(output.mode, Section451CMode::NotApplicableNotAnAdvancePayment);
    }

    #[test]
    fn full_inclusion_in_year_of_receipt_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::FullInclusionInYearOfReceiptUnderSection451C1A;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section451CMode::CompliantFullInclusionInYearOfReceiptUnderSection451C1A
        );
    }

    #[test]
    fn one_year_deferral_election_at_one_year_compliant() {
        let output = check(&baseline_input());
        assert_eq!(
            output.mode,
            Section451CMode::CompliantPartialAfsRecognitionPlusOneYearDeferralOfRemainder
        );
    }

    #[test]
    fn one_year_deferral_at_exactly_one_year_boundary_compliant() {
        let mut input = baseline_input();
        input.deferral_years_after_receipt = 1;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section451CMode::CompliantPartialAfsRecognitionPlusOneYearDeferralOfRemainder
        );
    }

    #[test]
    fn deferral_beyond_one_year_violation() {
        let mut input = baseline_input();
        input.deferral_years_after_receipt = 2;
        let output = check(&input);
        assert_eq!(output.mode, Section451CMode::ViolationDeferralBeyondOneYearMaximum);
    }

    #[test]
    fn afs_amount_not_included_in_year_of_receipt_violation() {
        let mut input = baseline_input();
        input.afs_amount_included_in_year_of_receipt = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section451CMode::ViolationAfsAmountNotIncludedInYearOfReceiptUnderAfsDeferralMethod
        );
    }

    #[test]
    fn eligible_advance_payment_category_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EligibleAdvancePaymentCategoryUnderTreasReg1_451_8A1;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section451CMode::CompliantEligibleAdvancePaymentCategoryUnderTreasReg1_451_8A1
        );
    }

    #[test]
    fn method_change_filed_under_rev_proc_2021_34_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::MethodChangeUnderRevProc2021_34;
        let output = check(&input);
        assert_eq!(output.mode, Section451CMode::CompliantMethodChangeFiledUnderRevProc2021_34);
    }

    #[test]
    fn continued_use_of_obsoleted_rev_proc_2004_34_post_2021_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::MethodChangeUnderRevProc2021_34;
        input.method_change_filed_under_rev_proc_2021_34 = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section451CMode::ViolationContinuedUseOfObsoleteRevProc2004_34PostJanuary1_2021
        );
    }

    #[test]
    fn acceleration_on_cessation_or_merger_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::AccelerationOnCessationOrMergerUnderTreasReg1_451_8F;
        let output = check(&input);
        assert_eq!(output.mode, Section451CMode::CompliantAccelerationOnCessationOrMergerHandled);
    }

    #[test]
    fn failure_to_accelerate_on_cessation_or_merger_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::AccelerationOnCessationOrMergerUnderTreasReg1_451_8F;
        input.deferred_balance_accelerated_on_cessation_or_merger = false;
        let output = check(&input);
        assert_eq!(output.mode, Section451CMode::ViolationDeferralBeyondOneYearMaximum);
    }

    #[test]
    fn accrual_method_without_afs_compliant_baseline() {
        let mut input = baseline_input();
        input.accounting_method = AccountingMethod::AccrualMethodWithoutApplicableFinancialStatement;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section451CMode::CompliantPartialAfsRecognitionPlusOneYearDeferralOfRemainder
        );
    }

    #[test]
    fn constants_pin_statutory_facts() {
        assert_eq!(IRC_451C_TCJA_ENACTMENT_DATE_YEAR, 2017);
        assert_eq!(IRC_451C_TCJA_ENACTMENT_DATE_MONTH, 12);
        assert_eq!(IRC_451C_TCJA_ENACTMENT_DATE_DAY, 22);
        assert_eq!(IRC_451C_TCJA_PUBLIC_LAW_CONGRESS, 115);
        assert_eq!(IRC_451C_TCJA_PUBLIC_LAW_ENACTMENT, 97);
        assert_eq!(IRC_451C_TCJA_ENABLING_SECTION, 13221);
        assert_eq!(IRC_451C_EFFECTIVE_FOR_TAX_YEARS_BEGINNING_AFTER_DATE_YEAR, 2017);
        assert_eq!(IRC_451C_EFFECTIVE_FOR_TAX_YEARS_BEGINNING_AFTER_DATE_MONTH, 12);
        assert_eq!(IRC_451C_EFFECTIVE_FOR_TAX_YEARS_BEGINNING_AFTER_DATE_DAY, 31);
        assert_eq!(IRC_451C_MAXIMUM_DEFERRAL_YEARS, 1);
        assert_eq!(IRC_451C_FINAL_REGS_TD_NUMBER, 9941);
        assert_eq!(IRC_451C_FINAL_REGS_FEDERAL_REGISTER_PUBLICATION_DATE_YEAR, 2021);
        assert_eq!(IRC_451C_FINAL_REGS_FEDERAL_REGISTER_PUBLICATION_DATE_MONTH, 1);
        assert_eq!(IRC_451C_FINAL_REGS_FEDERAL_REGISTER_PUBLICATION_DATE_DAY, 6);
        assert_eq!(IRC_451C_FINAL_REGS_EFFECTIVE_DATE_YEAR, 2021);
        assert_eq!(IRC_451C_FINAL_REGS_EFFECTIVE_DATE_MONTH, 1);
        assert_eq!(IRC_451C_FINAL_REGS_EFFECTIVE_DATE_DAY, 1);
        assert_eq!(IRC_451C_FINAL_REGS_FEDERAL_REGISTER_VOLUME, 86);
        assert_eq!(IRC_451C_FINAL_REGS_FEDERAL_REGISTER_PAGE, 810);
        assert_eq!(IRC_451C_PROPOSED_REGS_FEDERAL_REGISTER_VOLUME, 84);
        assert_eq!(IRC_451C_PROPOSED_REGS_FEDERAL_REGISTER_PAGE, 47_175);
        assert_eq!(IRC_451C_BASIS_POINT_DENOMINATOR, 10_000);
    }

    #[test]
    fn citations_contain_landmarks() {
        let output = check(&baseline_input());
        let joined = output.citations.join("\n");
        assert!(joined.contains("IRC § 451(c)"));
        assert!(joined.contains("Section 13221 of the Tax Cuts and Jobs Act of 2017"));
        assert!(joined.contains("Public Law 115-97"));
        assert!(joined.contains("131 Stat. 2054"));
        assert!(joined.contains("December 22, 2017"));
        assert!(joined.contains("December 31, 2017"));
        assert!(joined.contains("§ 451(c)(1)(A)"));
        assert!(joined.contains("§ 451(c)(1)(B)"));
        assert!(joined.contains("§ 451(c)(4)(A)"));
        assert!(joined.contains("§ 451(b)(3)"));
        assert!(joined.contains("TAXABLE YEAR OF RECEIPT"));
        assert!(joined.contains("SUCCEEDING TAXABLE YEAR"));
        assert!(joined.contains("1-YEAR deferral"));
        assert!(joined.contains("GOODS"));
        assert!(joined.contains("SERVICES"));
        assert!(joined.contains("SOFTWARE"));
        assert!(joined.contains("GIFT CARDS"));
        assert!(joined.contains("WARRANTY"));
        assert!(joined.contains("MEMBERSHIPS"));
        assert!(joined.contains("LOYALTY"));
        assert!(joined.contains("Applicable Financial Statement"));
        assert!(joined.contains("Treas. Reg. § 1.451-8"));
        assert!(joined.contains("September 9, 2019"));
        assert!(joined.contains("JANUARY 6, 2021"));
        assert!(joined.contains("T.D. 9941"));
        assert!(joined.contains("86 FR 810"));
        assert!(joined.contains("84 FR 47175"));
        assert!(joined.contains("Rev. Proc. 2021-34"));
        assert!(joined.contains("Rev. Proc. 2004-34"));
        assert!(joined.contains("Notice 2018-35"));
        assert!(joined.contains("Treas. Reg. § 1.451-8(f)"));
        assert!(joined.contains("cessation"));
        assert!(joined.contains("bankruptcy"));
        assert!(joined.contains("M&A"));
    }
}
