//! IRC § 6418 — Transfer of Certain Credits
//! (Transferability of Clean Energy Tax Credits)
//! Compliance Module — pure-compute check for IRA 2022
//! § 13801 transferability monetization framework.
//!
//! **Inflation Reduction Act of 2022 enactment**: Section
//! 6418 was implemented through **§ 13801(b) of Public Law
//! 117-169** (136 Stat. 1818, 2009), commonly known as
//! the Inflation Reduction Act of 2022 (IRA), **enacted
//! August 16, 2022**. **Final regulations effective July
//! 1, 2024.**
//!
//! **Monetization bridge for the IRA 2022 clean-energy
//! cluster**: § 6418 allows certain eligible taxpayers to
//! elect to transfer **ALL OR A PORTION** of certain
//! clean energy tax credits to **UNRELATED TAXPAYERS FOR
//! CASH** rather than use the credits themselves. Direct
//! pairing with the entire IRA 2022 clean-energy cluster
//! (§§ 30C, 45, 45Q, 45U, 45V, 45W, 45X, 45Y, 45Z, 48,
//! 48E) — all 11 eligible credit categories.
//!
//! **Distinctive § 6418 features**: **11 ELIGIBLE CREDIT
//! CATEGORIES** under § 6418(f)(1); **CASH-ONLY** payment
//! requirement; **NOT INCLUDED IN TRANSFEROR'S GROSS
//! INCOME and NOT DEDUCTIBLE BY TRANSFEREE**; **ONE
//! TRANSFER ONLY** — irrevocable; **PRE-FILING
//! REGISTRATION** required (Form 7211); **20% EXCESSIVE
//! CREDIT TRANSFER PENALTY** under § 6418(g)(2) with
//! **REASONABLE CAUSE EXCEPTION**; **§ 469 PASSIVE
//! ACTIVITY RULES APPLY TO TRANSFEREE** (no carve-out
//! in final regulations).
//!
//! Web research (verified 2026-06-03):
//! - **Inflation Reduction Act of 2022 enactment**: § 13801(b) of Public Law 117-169 (136 Stat. 1818, 2009), commonly known as the Inflation Reduction Act of 2022 (IRA), enacted August 16, 2022 ([Cornell LII — 26 U.S. Code § 6418](https://www.law.cornell.edu/uscode/text/26/6418); [Bloomberg Tax — Sec. 6418 Transfer Of Certain Credits](https://irc.bloombergtax.com/public/uscode/doc/irc/section_6418); [Federal Register — Section 6418 Transfer of Certain Credits Final Rule April 30, 2024](https://www.federalregister.gov/documents/2024/04/30/2024-08926/transfer-of-certain-credits); [Federal Register — Section 6418 Transfer of Certain Credits Proposed Rule June 21, 2023](https://www.federalregister.gov/documents/2023/06/21/2023-12799/section-6418-transfer-of-certain-credits); [IRS — Elective Pay and Transferability Frequently Asked Questions: Transferability](https://www.irs.gov/credits-deductions/elective-pay-and-transferability-frequently-asked-questions-transferability); [IRS — Elective Pay and Transferability](https://www.irs.gov/credits-deductions/elective-pay-and-transferability); [Arnold & Porter — IRS and Treasury Final Rules on Transfers of Clean Energy Tax Credits June 2024](https://www.arnoldporter.com/en/perspectives/advisories/2024/06/final-rules-issued-on-transfers-of-clean-energy-tax-credits); [BDO — Treasury IRS Release Final Regulations Transfer of Certain Energy Tax Credits](https://www.bdo.com/insights/tax/treasury-irs-release-final-regulations-on-transfer-of-certain-energy-tax-credits); [RSM — IRS Treasury Issue Final Regulations Transfer Energy Credits](https://rsmus.com/insights/tax-alerts/2024/irs-treasury-issue-final-regulations-transfer-energy-credits.html); [Cherry Bekaert — IRC Section 6418 FAQ Transferring Energy Tax Credits](https://www.cbh.com/insights/articles/irc-section-6418-faq-transferring-energy-tax-credits/); [Sidley Austin — Final U.S. Regulations Issued on the Sale of Energy Tax Credits May 2024](https://www.sidley.com/en/insights/newsupdates/2024/05/final-us-regulations-issued-on-the-sale-of-energy-tax-credits); [Holland & Knight — Section 6418 What's New in the Final IRA Transferability Regulations May 2024](https://www.hklaw.com/en/insights/publications/2024/05/section-6418-whats-new-in-the-final-inflation-reduction-act); [Akin Gump — Clean Energy Tax Credit Transferability Rules Finalized](https://www.akingump.com/en/insights/alerts/clean-energy-tax-credit-transferability-rules-finalized); [Pillsbury Law — Treasury and IRS Issue Rules on Transferability of Tax Credits](https://www.pillsburylaw.com/en/news-and-insights/tax-credits-irc-6418.html); [Novogradac — Release of Final Regulations on Transferability Provide Guidance on New Financing Tool](https://www.novoco.com/notes-from-novogradac/the-release-of-final-regulations-on-transferability-provide-guidance-on-new-financing-tool); [Greenberg Traurig — Proposed Regulations under Section 6418 Transferability of Clean Energy Tax Credits](https://www.gtlaw.com/en/insights/2023/7/proposed-regulations-under-section-6418-transferability-of-clean-energy-tax-credits); [Womble Bond Dickinson — IRS Publishes Final Regulations for Transfer of Certain Credits](https://www.womblebonddickinson.com/us/insights/alerts/irs-publishes-final-regulations-transfer-certain-credits); [Paul Hastings — Treasury and IRS Issue Long-Awaited Guidance on Energy Tax Credit Transfers](https://www.paulhastings.com/insights/client-alerts/treasury-and-irs-issue-long-awaited-guidance-on-energy-tax-credit-transfers)).
//! - **§ 6418 Transferability Mechanism**: § 6418 allows certain **ELIGIBLE TAXPAYERS** to elect to transfer **ALL OR A PORTION** of certain clean energy tax credits to **UNRELATED TAXPAYERS FOR CASH** rather than use the credits themselves; this creates a monetization pathway for taxpayers without sufficient tax liability to fully utilize the credit.
//! - **§ 6418(f)(1) Eleven Eligible Credit Categories**: the eligible credits are: **(1) § 30C** Alternative Fuel Vehicle Refueling Property Credit; **(2) § 45** Renewable Electricity Production Credit (PTC); **(3) § 45Q** Carbon Oxide Sequestration Credit; **(4) § 45U** Zero-Emission Nuclear Power PTC; **(5) § 45V** Clean Hydrogen Production Credit; **(6) § 45W** Qualified Commercial Clean Vehicles Credit; **(7) § 45X** Advanced Manufacturing Production Credit; **(8) § 45Y** Clean Electricity Production Credit; **(9) § 45Z** Clean Fuel Production Credit; **(10) § 48** Energy Investment Credit (ITC); **(11) § 48E** Clean Electricity Investment Credit.
//! - **§ 6418(b)(1) Cash Payment Requirement**: the transfer must be paid with **CASH** and is **NOT INCLUDED IN THE TRANSFEROR'S GROSS INCOME** and **NOT DEDUCTIBLE BY THE TRANSFEREE** — a unique tax treatment that distinguishes credit transfers from ordinary asset sales.
//! - **§ 6418(e) One Transfer Only — Irrevocable**: a transfer election is **IRREVOCABLE**, and a credit can **ONLY BE TRANSFERRED ONCE** — the transferee cannot resell the credit to another party (preventing speculative secondary markets).
//! - **§ 6418(g)(1) Pre-Filing Registration Required**: the regulations describe rules related to a required **IRS PRE-FILING REGISTRATION PROCESS** — eligible taxpayers must obtain a registration number through the IRS portal **BEFORE** filing the return on which the transfer election is made; one registration number per eligible credit per tax year.
//! - **§ 6418(g)(2) Excessive Credit Transfer — 20% Penalty**: an **EXCESSIVE CREDIT TRANSFER** is defined as an amount equal to the excess of the amount of the eligible credit claimed by the transferee over the amount that would otherwise be allowable; the excess amount is imposed as a tax and subject to a **20% PENALTY**; the 20% addition to tax does **NOT APPLY** if the transferee taxpayer demonstrates to the satisfaction of the Secretary that the excessive credit transfer resulted from **REASONABLE CAUSE**.
//! - **§ 6418 + § 469 Passive Activity Limitations Apply**: the final regulations stated that there is **NO CARVE OUT** for IRC § 469 passive loss limitations in § 6418 transfers; transferred tax credits are treated as arising from a passive activity if an individual transferee taxpayer does not materially participate in the credit-generating activity; same treatment for § 49 at-risk limitations.
//! - **§ 6418(d) Filing Timing**: the eligible taxpayer must elect to transfer the eligible credit **NO LATER THAN THE DUE DATE** of the tax return for the tax year for which the credit is determined (including extensions); election made on Form 3800 General Business Credit.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const SECTION_6418_NUMBER: u32 = 6418;
pub const SECTION_6418_ELIGIBLE_CREDIT_CATEGORIES_COUNT: u32 = 11;
pub const SECTION_6418_EXCESSIVE_CREDIT_TRANSFER_PENALTY_PERCENT: u32 = 20;
pub const SECTION_6418_MAXIMUM_TRANSFERS_PER_CREDIT: u32 = 1;
pub const SECTION_6418_IRA_2022_ENACTMENT_YEAR: u32 = 2022;
pub const SECTION_6418_IRA_2022_PUBLIC_LAW_NUMBER: u32 = 117169;
pub const SECTION_6418_FINAL_REGULATIONS_EFFECTIVE_YEAR: u32 = 2024;
pub const SECTION_6418_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EligibleCreditCategory {
    Section30CAlternativeFuelVehicleRefuelingPropertyCredit,
    Section45RenewableElectricityProductionCredit,
    Section45QCarbonOxideSequestrationCredit,
    Section45UZeroEmissionNuclearPowerCredit,
    Section45VCleanHydrogenProductionCredit,
    Section45WQualifiedCommercialCleanVehiclesCredit,
    Section45XAdvancedManufacturingProductionCredit,
    Section45YCleanElectricityProductionCredit,
    Section45ZCleanFuelProductionCredit,
    Section48EnergyInvestmentCredit,
    Section48ECleanElectricityInvestmentCredit,
    NotEligibleForSection6418Transfer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TransferorEligibility {
    EligibleTaxpayerCanElectTransfer,
    NotEligibleTaxpayer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TransferorTransfereeRelationship {
    UnrelatedParties,
    RelatedParties,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CreditTransferCount {
    FirstTransfer,
    SecondOrSubsequentTransfer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExcessiveCreditTransferStatus {
    NoExcessiveTransfer,
    ExcessiveTransferWithoutReasonableCause,
    ExcessiveTransferWithReasonableCause,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PreFilingRegistrationStatus {
    RegistrationNumberObtainedBeforeReturnFiled,
    RegistrationNumberNotObtained,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentForm {
    CashPayment,
    NonCashPayment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TransferElectionTiming {
    ElectionMadeOnTimelyFiledReturn,
    ElectionMadeAfterReturnDueDate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAspect {
    EligibleCreditCategoryUnderSection6418F1,
    EligibleTaxpayerStatusUnderSection6418A,
    UnrelatedPartiesRequirementUnderSection6418A,
    OneTransferOnlyUnderSection6418E,
    PreFilingRegistrationUnderSection6418G1,
    CashPaymentRequirementUnderSection6418B1,
    ExcessiveCreditTransferUnderSection6418G2,
    TransferElectionTimingUnderSection6418D,
    PassiveActivityLimitationsUnderSection469AppliedToTransferee,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section6418Mode {
    NotApplicableCreditNotEligibleUnderSection6418F1,
    CompliantEligibleCreditCategoryUnderSection6418F1,
    CompliantEligibleTaxpayerStatusUnderSection6418A,
    CompliantUnrelatedPartiesRequirementSatisfied,
    CompliantOneTransferOnlyFirstTransferElection,
    CompliantPreFilingRegistrationNumberObtained,
    CompliantCashPaymentNotIncludedInGrossIncomeNotDeductible,
    CompliantNoExcessiveCreditTransfer,
    CompliantExcessiveTransferWithReasonableCauseNo20PercentPenalty,
    CompliantTransferElectionMadeOnTimelyFiledReturn,
    CompliantPassiveActivityLimitationsAppliedToTransferee,
    ViolationCreditNotEligibleUnderSection6418F1,
    ViolationTransferorNotEligibleTaxpayer,
    ViolationRelatedPartiesNotPermitted,
    ViolationSecondOrSubsequentTransferProhibitedUnderSection6418E,
    ViolationPreFilingRegistrationNumberNotObtained,
    ViolationNonCashPaymentForm,
    ViolationExcessiveCreditTransferWithout20PercentReasonableCausePenaltyApplies,
    ViolationTransferElectionMadeAfterReturnDueDate,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub eligible_credit_category: EligibleCreditCategory,
    pub transferor_eligibility: TransferorEligibility,
    pub transferor_transferee_relationship: TransferorTransfereeRelationship,
    pub credit_transfer_count: CreditTransferCount,
    pub excessive_credit_transfer_status: ExcessiveCreditTransferStatus,
    pub pre_filing_registration_status: PreFilingRegistrationStatus,
    pub payment_form: PaymentForm,
    pub transfer_election_timing: TransferElectionTiming,
    pub compliance_aspect: ComplianceAspect,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section6418Mode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type Section6418Input = Input;
pub type Section6418Output = Output;
pub type Section6418Result = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "Inflation Reduction Act of 2022 enactment — § 13801(b) of Public Law 117-169 (136 Stat. 1818, 2009), commonly known as the Inflation Reduction Act of 2022 (IRA), enacted August 16, 2022; final regulations effective July 1, 2024".to_string(),
        "IRC § 6418 Transferability Mechanism — § 6418 allows certain ELIGIBLE TAXPAYERS to elect to transfer ALL OR A PORTION of certain clean energy tax credits to UNRELATED TAXPAYERS FOR CASH rather than use the credits themselves".to_string(),
        "IRC § 6418(f)(1) Eleven Eligible Credit Categories — (1) § 30C Alternative Fuel Vehicle Refueling Property Credit; (2) § 45 Renewable Electricity Production Credit (PTC); (3) § 45Q Carbon Oxide Sequestration Credit; (4) § 45U Zero-Emission Nuclear Power PTC; (5) § 45V Clean Hydrogen Production Credit; (6) § 45W Qualified Commercial Clean Vehicles Credit; (7) § 45X Advanced Manufacturing Production Credit; (8) § 45Y Clean Electricity Production Credit; (9) § 45Z Clean Fuel Production Credit; (10) § 48 Energy Investment Credit (ITC); (11) § 48E Clean Electricity Investment Credit".to_string(),
        "IRC § 6418(b)(1) Cash Payment Requirement — the transfer must be paid with CASH and is NOT INCLUDED IN THE TRANSFEROR'S GROSS INCOME and NOT DEDUCTIBLE BY THE TRANSFEREE".to_string(),
        "IRC § 6418(e) One Transfer Only — Irrevocable — a transfer election is IRREVOCABLE, and a credit can ONLY BE TRANSFERRED ONCE; the transferee cannot resell the credit to another party".to_string(),
        "IRC § 6418(g)(1) Pre-Filing Registration Required — eligible taxpayers must obtain a registration number through the IRS PRE-FILING REGISTRATION PROCESS BEFORE filing the return on which the transfer election is made; one registration number per eligible credit per tax year".to_string(),
        "IRC § 6418(g)(2) Excessive Credit Transfer — 20% Penalty — an EXCESSIVE CREDIT TRANSFER is defined as an amount equal to the excess of the amount of the eligible credit claimed by the transferee over the amount that would otherwise be allowable; the excess amount is imposed as a tax and subject to a 20% PENALTY; the 20% addition to tax does NOT APPLY if the transferee taxpayer demonstrates REASONABLE CAUSE".to_string(),
        "IRC § 6418 + § 469 Passive Activity Limitations Apply — the final regulations stated that there is NO CARVE OUT for IRC § 469 passive loss limitations in § 6418 transfers; transferred tax credits are treated as arising from a passive activity if an individual transferee taxpayer does not materially participate".to_string(),
        "IRC § 6418(d) Filing Timing — the eligible taxpayer must elect to transfer the eligible credit NO LATER THAN THE DUE DATE of the tax return for the tax year for which the credit is determined (including extensions); election made on Form 3800 General Business Credit".to_string(),
        "Cornell LII + Bloomberg Tax + Federal Register + IRS + Arnold & Porter + BDO + RSM + Cherry Bekaert + Sidley Austin + Holland & Knight + Akin Gump + Pillsbury Law + Novogradac + Greenberg Traurig + Womble Bond Dickinson + Paul Hastings — practitioner overviews of IRC § 6418 transferability of clean energy tax credits".to_string(),
    ];

    if input.eligible_credit_category == EligibleCreditCategory::NotEligibleForSection6418Transfer
    {
        return Output {
            mode: Section6418Mode::NotApplicableCreditNotEligibleUnderSection6418F1,
            statutory_basis: "IRC § 6418(f)(1) — credit not within 11 eligible categories".to_string(),
            notes: "NOT APPLICABLE: credit not within the 11 eligible categories under § 6418(f)(1); transferability monetization unavailable.".to_string(),
            citations,
        };
    }

    match input.compliance_aspect {
        ComplianceAspect::EligibleCreditCategoryUnderSection6418F1 => Output {
            mode: Section6418Mode::CompliantEligibleCreditCategoryUnderSection6418F1,
            statutory_basis: format!(
                "IRC § 6418(f)(1) — {c:?} within 11 eligible categories",
                c = input.eligible_credit_category,
            ),
            notes: format!(
                "COMPLIANT: {c:?} within the 11 eligible credit categories under § 6418(f)(1); transferability monetization available.",
                c = input.eligible_credit_category,
            ),
            citations,
        },
        ComplianceAspect::EligibleTaxpayerStatusUnderSection6418A => {
            match input.transferor_eligibility {
                TransferorEligibility::EligibleTaxpayerCanElectTransfer => Output {
                    mode: Section6418Mode::CompliantEligibleTaxpayerStatusUnderSection6418A,
                    statutory_basis: "IRC § 6418(a) — transferor is eligible taxpayer".to_string(),
                    notes: "COMPLIANT: transferor qualifies as eligible taxpayer under § 6418(a); transferability election available.".to_string(),
                    citations,
                },
                TransferorEligibility::NotEligibleTaxpayer => Output {
                    mode: Section6418Mode::ViolationTransferorNotEligibleTaxpayer,
                    statutory_basis: "IRC § 6418(a) — transferor not eligible taxpayer".to_string(),
                    notes: "VIOLATION: transferor not eligible taxpayer under § 6418(a); transferability election unavailable.".to_string(),
                    citations,
                },
            }
        }
        ComplianceAspect::UnrelatedPartiesRequirementUnderSection6418A => {
            match input.transferor_transferee_relationship {
                TransferorTransfereeRelationship::UnrelatedParties => Output {
                    mode: Section6418Mode::CompliantUnrelatedPartiesRequirementSatisfied,
                    statutory_basis: "IRC § 6418(a) — transferor and transferee are unrelated parties".to_string(),
                    notes: "COMPLIANT: transferor and transferee are unrelated parties under § 6418(a); transferability election permitted.".to_string(),
                    citations,
                },
                TransferorTransfereeRelationship::RelatedParties => Output {
                    mode: Section6418Mode::ViolationRelatedPartiesNotPermitted,
                    statutory_basis: "IRC § 6418(a) — related parties not permitted".to_string(),
                    notes: "VIOLATION: related-party transfer prohibited under § 6418(a); transferor and transferee must be unrelated.".to_string(),
                    citations,
                },
            }
        }
        ComplianceAspect::OneTransferOnlyUnderSection6418E => {
            match input.credit_transfer_count {
                CreditTransferCount::FirstTransfer => Output {
                    mode: Section6418Mode::CompliantOneTransferOnlyFirstTransferElection,
                    statutory_basis: "IRC § 6418(e) — first transfer election is irrevocable but permitted".to_string(),
                    notes: "COMPLIANT: first transfer election under § 6418(e); election is IRREVOCABLE and credit can be transferred ONLY ONCE.".to_string(),
                    citations,
                },
                CreditTransferCount::SecondOrSubsequentTransfer => Output {
                    mode: Section6418Mode::ViolationSecondOrSubsequentTransferProhibitedUnderSection6418E,
                    statutory_basis: "IRC § 6418(e) — second or subsequent transfer prohibited".to_string(),
                    notes: "VIOLATION: second or subsequent transfer prohibited under § 6418(e); credit can ONLY BE TRANSFERRED ONCE; transferee cannot resell to another party.".to_string(),
                    citations,
                },
            }
        }
        ComplianceAspect::PreFilingRegistrationUnderSection6418G1 => {
            match input.pre_filing_registration_status {
                PreFilingRegistrationStatus::RegistrationNumberObtainedBeforeReturnFiled => Output {
                    mode: Section6418Mode::CompliantPreFilingRegistrationNumberObtained,
                    statutory_basis: "IRC § 6418(g)(1) — pre-filing registration number obtained".to_string(),
                    notes: "COMPLIANT: pre-filing registration number obtained through IRS portal BEFORE return filed under § 6418(g)(1).".to_string(),
                    citations,
                },
                PreFilingRegistrationStatus::RegistrationNumberNotObtained => Output {
                    mode: Section6418Mode::ViolationPreFilingRegistrationNumberNotObtained,
                    statutory_basis: "IRC § 6418(g)(1) — pre-filing registration number not obtained".to_string(),
                    notes: "VIOLATION: pre-filing registration number not obtained under § 6418(g)(1); transferability election cannot be perfected without registration number.".to_string(),
                    citations,
                },
            }
        }
        ComplianceAspect::CashPaymentRequirementUnderSection6418B1 => match input.payment_form {
            PaymentForm::CashPayment => Output {
                mode: Section6418Mode::CompliantCashPaymentNotIncludedInGrossIncomeNotDeductible,
                statutory_basis: "IRC § 6418(b)(1) — cash payment + non-includible + non-deductible treatment".to_string(),
                notes: "COMPLIANT: cash payment under § 6418(b)(1); payment is NOT INCLUDED IN TRANSFEROR'S GROSS INCOME and NOT DEDUCTIBLE BY TRANSFEREE.".to_string(),
                citations,
            },
            PaymentForm::NonCashPayment => Output {
                mode: Section6418Mode::ViolationNonCashPaymentForm,
                statutory_basis: "IRC § 6418(b)(1) — non-cash payment not permitted".to_string(),
                notes: "VIOLATION: non-cash payment not permitted under § 6418(b)(1); transferability monetization requires CASH payment only.".to_string(),
                citations,
            },
        },
        ComplianceAspect::ExcessiveCreditTransferUnderSection6418G2 => {
            match input.excessive_credit_transfer_status {
                ExcessiveCreditTransferStatus::NoExcessiveTransfer => Output {
                    mode: Section6418Mode::CompliantNoExcessiveCreditTransfer,
                    statutory_basis: "IRC § 6418(g)(2) — no excessive credit transfer".to_string(),
                    notes: "COMPLIANT: no excessive credit transfer under § 6418(g)(2); 20% penalty not triggered.".to_string(),
                    citations,
                },
                ExcessiveCreditTransferStatus::ExcessiveTransferWithReasonableCause => Output {
                    mode: Section6418Mode::CompliantExcessiveTransferWithReasonableCauseNo20PercentPenalty,
                    statutory_basis: "IRC § 6418(g)(2) — excessive transfer with reasonable cause exception".to_string(),
                    notes: "COMPLIANT: excessive credit transfer with REASONABLE CAUSE under § 6418(g)(2); 20% addition to tax does NOT APPLY; transferee demonstrated reasonable cause to satisfaction of Secretary.".to_string(),
                    citations,
                },
                ExcessiveCreditTransferStatus::ExcessiveTransferWithoutReasonableCause => Output {
                    mode: Section6418Mode::ViolationExcessiveCreditTransferWithout20PercentReasonableCausePenaltyApplies,
                    statutory_basis: "IRC § 6418(g)(2) — excessive transfer without reasonable cause; 20% penalty applies".to_string(),
                    notes: "VIOLATION: excessive credit transfer without reasonable cause under § 6418(g)(2); excess amount imposed as tax + 20% penalty addition to tax applies to transferee.".to_string(),
                    citations,
                },
            }
        }
        ComplianceAspect::TransferElectionTimingUnderSection6418D => {
            match input.transfer_election_timing {
                TransferElectionTiming::ElectionMadeOnTimelyFiledReturn => Output {
                    mode: Section6418Mode::CompliantTransferElectionMadeOnTimelyFiledReturn,
                    statutory_basis: "IRC § 6418(d) — election made on timely filed return".to_string(),
                    notes: "COMPLIANT: transfer election made on timely filed return under § 6418(d); election made no later than due date (including extensions) of return for tax year credit determined.".to_string(),
                    citations,
                },
                TransferElectionTiming::ElectionMadeAfterReturnDueDate => Output {
                    mode: Section6418Mode::ViolationTransferElectionMadeAfterReturnDueDate,
                    statutory_basis: "IRC § 6418(d) — election made after return due date".to_string(),
                    notes: "VIOLATION: transfer election made after return due date under § 6418(d); election must be made no later than due date (including extensions) of return for tax year credit determined.".to_string(),
                    citations,
                },
            }
        }
        ComplianceAspect::PassiveActivityLimitationsUnderSection469AppliedToTransferee => Output {
            mode: Section6418Mode::CompliantPassiveActivityLimitationsAppliedToTransferee,
            statutory_basis: "IRC § 6418 + § 469 — passive activity limitations apply to transferee".to_string(),
            notes: "COMPLIANT: § 469 passive activity limitations applied to transferee; no carve-out for passive loss limitations in § 6418 transfers; credit treated as arising from passive activity if individual transferee does not materially participate.".to_string(),
            citations,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_input() -> Input {
        Input {
            eligible_credit_category:
                EligibleCreditCategory::Section45YCleanElectricityProductionCredit,
            transferor_eligibility: TransferorEligibility::EligibleTaxpayerCanElectTransfer,
            transferor_transferee_relationship: TransferorTransfereeRelationship::UnrelatedParties,
            credit_transfer_count: CreditTransferCount::FirstTransfer,
            excessive_credit_transfer_status: ExcessiveCreditTransferStatus::NoExcessiveTransfer,
            pre_filing_registration_status:
                PreFilingRegistrationStatus::RegistrationNumberObtainedBeforeReturnFiled,
            payment_form: PaymentForm::CashPayment,
            transfer_election_timing: TransferElectionTiming::ElectionMadeOnTimelyFiledReturn,
            compliance_aspect: ComplianceAspect::EligibleCreditCategoryUnderSection6418F1,
        }
    }

    #[test]
    fn ineligible_credit_not_applicable() {
        let mut input = baseline_input();
        input.eligible_credit_category = EligibleCreditCategory::NotEligibleForSection6418Transfer;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section6418Mode::NotApplicableCreditNotEligibleUnderSection6418F1
        );
    }

    #[test]
    fn section_30c_eligible_credit_compliant() {
        let mut input = baseline_input();
        input.eligible_credit_category =
            EligibleCreditCategory::Section30CAlternativeFuelVehicleRefuelingPropertyCredit;
        input.compliance_aspect = ComplianceAspect::EligibleCreditCategoryUnderSection6418F1;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section6418Mode::CompliantEligibleCreditCategoryUnderSection6418F1
        );
    }

    #[test]
    fn section_45_renewable_ptc_eligible_compliant() {
        let mut input = baseline_input();
        input.eligible_credit_category =
            EligibleCreditCategory::Section45RenewableElectricityProductionCredit;
        input.compliance_aspect = ComplianceAspect::EligibleCreditCategoryUnderSection6418F1;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section6418Mode::CompliantEligibleCreditCategoryUnderSection6418F1
        );
    }

    #[test]
    fn section_45q_carbon_oxide_sequestration_eligible_compliant() {
        let mut input = baseline_input();
        input.eligible_credit_category =
            EligibleCreditCategory::Section45QCarbonOxideSequestrationCredit;
        input.compliance_aspect = ComplianceAspect::EligibleCreditCategoryUnderSection6418F1;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section6418Mode::CompliantEligibleCreditCategoryUnderSection6418F1
        );
    }

    #[test]
    fn section_45u_zero_emission_nuclear_eligible_compliant() {
        let mut input = baseline_input();
        input.eligible_credit_category =
            EligibleCreditCategory::Section45UZeroEmissionNuclearPowerCredit;
        input.compliance_aspect = ComplianceAspect::EligibleCreditCategoryUnderSection6418F1;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section6418Mode::CompliantEligibleCreditCategoryUnderSection6418F1
        );
    }

    #[test]
    fn section_45v_clean_hydrogen_eligible_compliant() {
        let mut input = baseline_input();
        input.eligible_credit_category =
            EligibleCreditCategory::Section45VCleanHydrogenProductionCredit;
        input.compliance_aspect = ComplianceAspect::EligibleCreditCategoryUnderSection6418F1;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section6418Mode::CompliantEligibleCreditCategoryUnderSection6418F1
        );
    }

    #[test]
    fn section_45w_commercial_clean_vehicles_eligible_compliant() {
        let mut input = baseline_input();
        input.eligible_credit_category =
            EligibleCreditCategory::Section45WQualifiedCommercialCleanVehiclesCredit;
        input.compliance_aspect = ComplianceAspect::EligibleCreditCategoryUnderSection6418F1;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section6418Mode::CompliantEligibleCreditCategoryUnderSection6418F1
        );
    }

    #[test]
    fn section_45x_advanced_manufacturing_eligible_compliant() {
        let mut input = baseline_input();
        input.eligible_credit_category =
            EligibleCreditCategory::Section45XAdvancedManufacturingProductionCredit;
        input.compliance_aspect = ComplianceAspect::EligibleCreditCategoryUnderSection6418F1;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section6418Mode::CompliantEligibleCreditCategoryUnderSection6418F1
        );
    }

    #[test]
    fn section_45y_clean_electricity_production_eligible_compliant() {
        let mut input = baseline_input();
        input.eligible_credit_category =
            EligibleCreditCategory::Section45YCleanElectricityProductionCredit;
        input.compliance_aspect = ComplianceAspect::EligibleCreditCategoryUnderSection6418F1;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section6418Mode::CompliantEligibleCreditCategoryUnderSection6418F1
        );
    }

    #[test]
    fn section_45z_clean_fuel_production_eligible_compliant() {
        let mut input = baseline_input();
        input.eligible_credit_category =
            EligibleCreditCategory::Section45ZCleanFuelProductionCredit;
        input.compliance_aspect = ComplianceAspect::EligibleCreditCategoryUnderSection6418F1;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section6418Mode::CompliantEligibleCreditCategoryUnderSection6418F1
        );
    }

    #[test]
    fn section_48_energy_itc_eligible_compliant() {
        let mut input = baseline_input();
        input.eligible_credit_category = EligibleCreditCategory::Section48EnergyInvestmentCredit;
        input.compliance_aspect = ComplianceAspect::EligibleCreditCategoryUnderSection6418F1;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section6418Mode::CompliantEligibleCreditCategoryUnderSection6418F1
        );
    }

    #[test]
    fn section_48e_clean_electricity_investment_eligible_compliant() {
        let mut input = baseline_input();
        input.eligible_credit_category =
            EligibleCreditCategory::Section48ECleanElectricityInvestmentCredit;
        input.compliance_aspect = ComplianceAspect::EligibleCreditCategoryUnderSection6418F1;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section6418Mode::CompliantEligibleCreditCategoryUnderSection6418F1
        );
    }

    #[test]
    fn eligible_taxpayer_status_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EligibleTaxpayerStatusUnderSection6418A;
        input.transferor_eligibility = TransferorEligibility::EligibleTaxpayerCanElectTransfer;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section6418Mode::CompliantEligibleTaxpayerStatusUnderSection6418A
        );
    }

    #[test]
    fn not_eligible_taxpayer_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EligibleTaxpayerStatusUnderSection6418A;
        input.transferor_eligibility = TransferorEligibility::NotEligibleTaxpayer;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section6418Mode::ViolationTransferorNotEligibleTaxpayer
        );
    }

    #[test]
    fn unrelated_parties_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::UnrelatedPartiesRequirementUnderSection6418A;
        input.transferor_transferee_relationship = TransferorTransfereeRelationship::UnrelatedParties;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section6418Mode::CompliantUnrelatedPartiesRequirementSatisfied
        );
    }

    #[test]
    fn related_parties_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::UnrelatedPartiesRequirementUnderSection6418A;
        input.transferor_transferee_relationship = TransferorTransfereeRelationship::RelatedParties;
        let out = check(&input);
        assert_eq!(out.mode, Section6418Mode::ViolationRelatedPartiesNotPermitted);
    }

    #[test]
    fn first_transfer_election_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::OneTransferOnlyUnderSection6418E;
        input.credit_transfer_count = CreditTransferCount::FirstTransfer;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section6418Mode::CompliantOneTransferOnlyFirstTransferElection
        );
    }

    #[test]
    fn second_or_subsequent_transfer_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::OneTransferOnlyUnderSection6418E;
        input.credit_transfer_count = CreditTransferCount::SecondOrSubsequentTransfer;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section6418Mode::ViolationSecondOrSubsequentTransferProhibitedUnderSection6418E
        );
    }

    #[test]
    fn pre_filing_registration_obtained_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::PreFilingRegistrationUnderSection6418G1;
        input.pre_filing_registration_status =
            PreFilingRegistrationStatus::RegistrationNumberObtainedBeforeReturnFiled;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section6418Mode::CompliantPreFilingRegistrationNumberObtained
        );
    }

    #[test]
    fn pre_filing_registration_not_obtained_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::PreFilingRegistrationUnderSection6418G1;
        input.pre_filing_registration_status =
            PreFilingRegistrationStatus::RegistrationNumberNotObtained;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section6418Mode::ViolationPreFilingRegistrationNumberNotObtained
        );
    }

    #[test]
    fn cash_payment_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::CashPaymentRequirementUnderSection6418B1;
        input.payment_form = PaymentForm::CashPayment;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section6418Mode::CompliantCashPaymentNotIncludedInGrossIncomeNotDeductible
        );
    }

    #[test]
    fn non_cash_payment_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::CashPaymentRequirementUnderSection6418B1;
        input.payment_form = PaymentForm::NonCashPayment;
        let out = check(&input);
        assert_eq!(out.mode, Section6418Mode::ViolationNonCashPaymentForm);
    }

    #[test]
    fn no_excessive_credit_transfer_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ExcessiveCreditTransferUnderSection6418G2;
        input.excessive_credit_transfer_status = ExcessiveCreditTransferStatus::NoExcessiveTransfer;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section6418Mode::CompliantNoExcessiveCreditTransfer
        );
    }

    #[test]
    fn excessive_transfer_with_reasonable_cause_no_penalty_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ExcessiveCreditTransferUnderSection6418G2;
        input.excessive_credit_transfer_status =
            ExcessiveCreditTransferStatus::ExcessiveTransferWithReasonableCause;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section6418Mode::CompliantExcessiveTransferWithReasonableCauseNo20PercentPenalty
        );
    }

    #[test]
    fn excessive_transfer_without_reasonable_cause_20_percent_penalty_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ExcessiveCreditTransferUnderSection6418G2;
        input.excessive_credit_transfer_status =
            ExcessiveCreditTransferStatus::ExcessiveTransferWithoutReasonableCause;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section6418Mode::ViolationExcessiveCreditTransferWithout20PercentReasonableCausePenaltyApplies
        );
    }

    #[test]
    fn timely_filed_election_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::TransferElectionTimingUnderSection6418D;
        input.transfer_election_timing = TransferElectionTiming::ElectionMadeOnTimelyFiledReturn;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section6418Mode::CompliantTransferElectionMadeOnTimelyFiledReturn
        );
    }

    #[test]
    fn late_election_after_due_date_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::TransferElectionTimingUnderSection6418D;
        input.transfer_election_timing = TransferElectionTiming::ElectionMadeAfterReturnDueDate;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section6418Mode::ViolationTransferElectionMadeAfterReturnDueDate
        );
    }

    #[test]
    fn passive_activity_limitations_applied_to_transferee_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::PassiveActivityLimitationsUnderSection469AppliedToTransferee;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section6418Mode::CompliantPassiveActivityLimitationsAppliedToTransferee
        );
    }

    #[test]
    fn constants_pin_section_6418_statutory_thresholds() {
        assert_eq!(SECTION_6418_NUMBER, 6418);
        assert_eq!(SECTION_6418_ELIGIBLE_CREDIT_CATEGORIES_COUNT, 11);
        assert_eq!(SECTION_6418_EXCESSIVE_CREDIT_TRANSFER_PENALTY_PERCENT, 20);
        assert_eq!(SECTION_6418_MAXIMUM_TRANSFERS_PER_CREDIT, 1);
        assert_eq!(SECTION_6418_IRA_2022_ENACTMENT_YEAR, 2022);
        assert_eq!(SECTION_6418_IRA_2022_PUBLIC_LAW_NUMBER, 117169);
        assert_eq!(SECTION_6418_FINAL_REGULATIONS_EFFECTIVE_YEAR, 2024);
        assert_eq!(SECTION_6418_BASIS_POINT_DENOMINATOR, 10_000);
    }

    #[test]
    fn citations_pin_section_6418_statutory_provisions() {
        let input = baseline_input();
        let out = check(&input);
        let joined = out.citations.join(" || ");
        assert!(joined.contains("Inflation Reduction Act of 2022"));
        assert!(joined.contains("§ 13801(b) of Public Law 117-169"));
        assert!(joined.contains("August 16, 2022"));
        assert!(joined.contains("final regulations effective July 1, 2024"));
        assert!(joined.contains("IRC § 6418"));
        assert!(joined.contains("ELIGIBLE TAXPAYERS"));
        assert!(joined.contains("ALL OR A PORTION"));
        assert!(joined.contains("UNRELATED TAXPAYERS FOR CASH"));
        assert!(joined.contains("IRC § 6418(f)(1)"));
        assert!(joined.contains("§ 30C"));
        assert!(joined.contains("§ 45"));
        assert!(joined.contains("§ 45Q"));
        assert!(joined.contains("§ 45U"));
        assert!(joined.contains("§ 45V"));
        assert!(joined.contains("§ 45W"));
        assert!(joined.contains("§ 45X"));
        assert!(joined.contains("§ 45Y"));
        assert!(joined.contains("§ 45Z"));
        assert!(joined.contains("§ 48"));
        assert!(joined.contains("§ 48E"));
        assert!(joined.contains("IRC § 6418(b)(1)"));
        assert!(joined.contains("CASH"));
        assert!(joined.contains("NOT INCLUDED IN THE TRANSFEROR'S GROSS INCOME"));
        assert!(joined.contains("NOT DEDUCTIBLE BY THE TRANSFEREE"));
        assert!(joined.contains("IRC § 6418(e)"));
        assert!(joined.contains("IRREVOCABLE"));
        assert!(joined.contains("ONLY BE TRANSFERRED ONCE"));
        assert!(joined.contains("IRC § 6418(g)(1)"));
        assert!(joined.contains("IRS PRE-FILING REGISTRATION PROCESS"));
        assert!(joined.contains("IRC § 6418(g)(2)"));
        assert!(joined.contains("EXCESSIVE CREDIT TRANSFER"));
        assert!(joined.contains("20% PENALTY"));
        assert!(joined.contains("REASONABLE CAUSE"));
        assert!(joined.contains("§ 469"));
        assert!(joined.contains("NO CARVE OUT"));
        assert!(joined.contains("IRC § 6418(d)"));
        assert!(joined.contains("NO LATER THAN THE DUE DATE"));
        assert!(joined.contains("Form 3800"));
    }
}
