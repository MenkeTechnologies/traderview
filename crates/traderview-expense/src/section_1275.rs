//! IRC § 1275 — Other Definitions and Special Rules /
//! OID Definitional Anchor + Adjusted Issue Price Mechanics
//! Module.
//!
//! Pure-compute check for IRC § 1275 definitional anchor for
//! the OID statutory cluster (§§ 163(e), 1271-1275 inclusive)
//! and its implementing regulations at Treas. Reg. §§ 1.1275-1
//! through 1.1275-7. § 1275 supplies the operative definitions
//! for "debt instrument," "issue date," "issue price," and
//! "qualified stated interest" that the rest of the OID
//! framework relies on, plus the §§ 1275(b)/(c)/(d) anti-
//! abuse and information-reporting rules. § 1275 is the
//! companion to § 1274 (issue price for debt-for-property
//! exchanges, built iter 678) and § 1273 (OID determination,
//! existing module) — together the three form the OID
//! definitional spine.
//!
//! Web research (verified 2026-06-03):
//! - **IRC § 1275(a)(1)(A) Debt Instrument Definition**: any instrument or contractual arrangement that constitutes indebtedness under general principles of Federal income tax law (including a certificate of deposit or a loan); § 1275 supplies the operative definitions for §§ 163(e) and 1271 through 1275 and the regulations thereunder ([Cornell LII 26 CFR § 1.1275-1](https://www.law.cornell.edu/cfr/text/26/1.1275-1); [Bloomberg Tax IRC § 1275](https://www.taxnotes.com/research/federal/usc26/1275); [IRS Rev. Rul. 2000-12 — § 1275 Other Definitions](https://www.irs.gov/pub/irs-drop/rr-00-12.pdf); [IRS Publication 1212 — Guide to Original Issue Discount (OID) Instruments (12/2025)](https://www.irs.gov/publications/p1212)).
//! - **IRC § 1275(a)(1)(B) Exclusions from Debt Instrument**: (i) **ANNUITY CONTRACTS** issued in transactions where some/all of payments depend on life expectancy of one or more individuals or held under a § 401(g)/§ 403(b)/§ 408 plan; (ii) **ANNUITY CONTRACTS issued by FOREIGN INSURERS** only if subject to tax under subchapter L with respect to income earned on the annuity contract. Disqualifying annuity provisions include cash surrender options, secured loan availability, minimum/maximum payout provisions (with limited exceptions), or decreasing payouts (except variable distributions tied to investment performance) ([Federal Register — Debt Instruments With Original Issue Discount; Annuity Contracts (REG-125237-00; May 7, 2002)](https://www.federalregister.gov/documents/2002/05/07/02-11035/debt-instruments-with-original-issue-discount-annuity-contracts)).
//! - **IRC § 1275(a)(2) Issue Date Definition**: the date of FIRST ISSUE of the obligation; for publicly offered debt the issue date is the first settlement date; for private debt the date of execution of the obligation by the issuer.
//! - **IRC § 1275(a)(3) Issue Price Definition Cross-Reference**: issue price determined under § 1273(b) for publicly offered / cash-sold / property debt and under § 1274 for debt instruments issued for property; § 1275 ties the OID statutory cluster together by referencing both § 1273 and § 1274 issue-price determinations.
//! - **IRC § 1275(b) Treatment of Borrower in Case of Certain Loans for Personal Use**: for any loan between natural persons that is NOT issued in connection with a trade or business of the lender, the original-issue-discount rules of §§ 1272 and 1273 do NOT apply to the borrower; the borrower's treatment is governed by the cash receipts and disbursements method of accounting. Personal-use property loan exception under § 1275(b)(1) prevents the borrower from being required to accrue OID on a debt instrument issued in exchange for personal-use property (e.g., a personal residence mortgage between family members at below-market rates).
//! - **IRC § 1275(c) Information Requirements**: (1) the SECRETARY shall require that all relevant information regarding any debt instrument issued at OID be sent to the holders of the obligation; (2) the secretary shall require that all relevant information regarding any TAX-EXEMPT OBLIGATION issued at OID be sent to the holders; (3) any person who issues a STRIPPED BOND must file information returns identifying the issuer and the obligation; § 1275(c)(4) cross-references § 6049 (information reporting on Form 1099-OID for OID and on Form 1099-INT for interest); failure to file triggers § 6721/§ 6722 information-return penalties.
//! - **IRC § 1275(d) Anti-Abuse Regulations**: the Secretary shall prescribe regulations to prevent the avoidance of these OID rules by means of changes in form (e.g., recharacterization of OID instruments as variable-rate, contingent, or convertible instruments); implementing regulations at Treas. Reg. §§ 1.1275-2 through 1.1275-7 address variable-rate debt (§ 1.1275-5), contingent-payment debt (§ 1.1275-4), integration of qualifying debt instruments with hedges (§ 1.1275-6), and inflation-indexed debt instruments (§ 1.1275-7).
//! - **§ 1.1275-1 Adjusted Issue Price Computation**: the **ADJUSTED ISSUE PRICE** equals the issue price of the debt instrument INCREASED by the amount of OID previously includible in the gross income of any holder AND DECREASED by the amount of any payment previously made on the debt instrument other than a payment of qualified stated interest. This running computation is the basis for accrual-period OID inclusions under § 1272.
//! - **OID Payment Allocation Rule (§ 1.1275-2(a))**: each payment under a debt instrument is treated FIRST as a payment of OID to the extent of the OID that has accrued as of the date the payment is due and has not been allocated to prior payments, and SECOND as a payment of principal. This "OID first, principal second" rule prevents tax-motivated repayment-ordering schemes.
//! - **Transition Rule (March 13, 2001)**: the current § 1.1275-1 definitions apply to debt instruments issued **ON OR AFTER MARCH 13, 2001**; pre-March-13-2001 instruments are subject to transitional rules under prior § 1.1275-1 versions.
//! - **Annuity Grandfather Date (April 7, 1995)**: annuity contracts purchased before April 7, 1995, or meeting the criteria of Notice FI-33-94, are exempt from certain annuity-related § 1275(a)(1)(B)(i) disqualifying provisions; the grandfather date is the most-litigated boundary for legacy annuity arrangements.
//! - **Companion Provisions**: § 163(e) issuer-side OID deduction (parallel to § 1272 holder-side inclusion); § 1271 retirement of debt instrument; § 1272 current OID inclusion; § 1273 OID determination (issue price + SRPM + de minimis); § 1274 issue price for debt-for-property exchanges (built iter 678); § 1276/§ 1277/§ 1278 market discount; § 1281/§ 1282/§ 1283 short-term obligations; § 1286 stripped bonds (built iter 672); § 1287 anti-bearer-bond rule; § 1288 anti-avoidance tax-exempt OID.
//! - **Treasury Regulation Authority**: § 1.1275-1 (definitions); § 1.1275-2 (special rules including OID payment allocation); § 1.1275-3 (information reporting); § 1.1275-4 (contingent payment debt instruments); § 1.1275-5 (variable rate debt instruments); § 1.1275-6 (integration of qualifying debt with hedges); § 1.1275-7 (inflation-indexed debt instruments such as TIPS).

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const IRC_1275_REGULATIONS_EFFECTIVE_DATE_YEAR: u32 = 2001;
pub const IRC_1275_REGULATIONS_EFFECTIVE_DATE_MONTH: u32 = 3;
pub const IRC_1275_REGULATIONS_EFFECTIVE_DATE_DAY: u32 = 13;
pub const IRC_1275_ANNUITY_GRANDFATHER_DATE_YEAR: u32 = 1995;
pub const IRC_1275_ANNUITY_GRANDFATHER_DATE_MONTH: u32 = 4;
pub const IRC_1275_ANNUITY_GRANDFATHER_DATE_DAY: u32 = 7;
pub const IRC_1275_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InstrumentType {
    DebtInstrumentUnderSection1275A1A,
    AnnuityContractExceptionSection1275A1Bi,
    ForeignInsurerAnnuityExceptionSection1275A1Bii,
    NotADebtInstrumentOutsideSection1275Scope,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAspect {
    DebtInstrumentClassification,
    OidPaymentAllocationOrder,
    AdjustedIssuePriceComputation,
    PersonalUsePropertyLoanExceptionUnderSection1275B,
    InformationReturnFilingUnderSection6049Section1275C,
    AnnuityContractExceptionQualification,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentAllocationOrder {
    OidFirstPrincipalSecondPerTreasReg12752,
    PrincipalFirstOidSecondViolation,
    NotApplicableNoPaymentMade,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AnnuityDisqualifyingProvisionStatus {
    NoDisqualifyingProvisionsPresent,
    CashSurrenderOptionPresent,
    SecuredLoanAvailabilityPresent,
    MaximumPayoutProvisionPresent,
    DecreasingPayoutsPresent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TransitionDateStatus {
    DebtInstrumentIssuedOnOrAfterMarch13_2001CurrentRegulationsApply,
    DebtInstrumentIssuedBeforeMarch13_2001PreRegulationTransitionalRulesApply,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section1275Mode {
    NotApplicableNotADebtInstrumentOutsideSection1275Scope,
    NotApplicableAnnuityContractExceptionSection1275A1Bi,
    NotApplicableForeignInsurerAnnuityExceptionSection1275A1Bii,
    NotApplicablePreMarch13_2001RegulationsTransitionalRule,
    CompliantDebtInstrumentClassifiedUnderSection1275A1A,
    CompliantOidPaymentAllocationOidFirstPrincipalSecondPerTreasReg12752,
    CompliantAdjustedIssuePriceComputationCorrect,
    CompliantPersonalUsePropertyLoanExceptionUnderSection1275BApplies,
    CompliantInformationReturnsFiledUnderSection6049Section1275C,
    CompliantAnnuityContractExceptionQualifiedNoDisqualifyingProvisions,
    ViolationOidPaymentAllocationPrincipalFirstOidSecond,
    ViolationAdjustedIssuePriceComputationIncorrect,
    ViolationInformationReturnNotFiledUnderSection6049,
    ViolationAnnuityContractClaimedExceptionButDisqualifyingProvisionPresent,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub instrument_type: InstrumentType,
    pub compliance_aspect: ComplianceAspect,
    pub transition_date_status: TransitionDateStatus,
    pub payment_allocation_order: PaymentAllocationOrder,
    pub adjusted_issue_price_computation_correct: bool,
    pub information_return_filed_under_section_6049: bool,
    pub annuity_disqualifying_provision_status: AnnuityDisqualifyingProvisionStatus,
    pub is_personal_use_property_loan_between_natural_persons: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section1275Mode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type Section1275Input = Input;
pub type Section1275Output = Output;
pub type Section1275Result = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "IRC § 1275(a)(1)(A) Debt Instrument Definition — any instrument or contractual arrangement that constitutes indebtedness under general principles of Federal income tax law (including certificate of deposit or loan); § 1275 supplies operative definitions for §§ 163(e) and 1271 through 1275 and regulations thereunder".to_string(),
        "IRC § 1275(a)(1)(B) Exclusions — (i) ANNUITY CONTRACTS depending on life expectancy or held under § 401(g)/§ 403(b)/§ 408 plan; (ii) ANNUITY CONTRACTS issued by FOREIGN INSURERS only if subject to tax under subchapter L; disqualifying provisions include cash surrender options, secured loan availability, minimum/maximum payout provisions, decreasing payouts (except variable distributions tied to investment performance)".to_string(),
        "IRC § 1275(a)(2) Issue Date Definition — date of FIRST ISSUE of obligation; for publicly offered debt = first settlement date; for private debt = date of execution by issuer".to_string(),
        "IRC § 1275(a)(3) Issue Price Definition Cross-Reference — issue price determined under § 1273(b) for publicly offered / cash-sold / property debt and under § 1274 for debt instruments issued for property".to_string(),
        "IRC § 1275(b) Personal-Use Loan Exception — for any loan between natural persons NOT issued in connection with trade/business of lender, OID rules of §§ 1272 and 1273 do NOT apply to borrower; borrower governed by cash receipts/disbursements method; personal-use property loan (e.g., personal residence mortgage between family members at below-market rates) prevents borrower from required OID accrual".to_string(),
        "IRC § 1275(c) Information Requirements — (1) Secretary requires all relevant OID information sent to holders; (2) tax-exempt obligation OID information also reported; (3) stripped bond issuer must file information returns identifying issuer and obligation; § 1275(c)(4) cross-references § 6049 (Form 1099-OID + Form 1099-INT); failure triggers § 6721/§ 6722 information-return penalties".to_string(),
        "IRC § 1275(d) Anti-Abuse Regulations — Secretary prescribes regulations to prevent avoidance by means of changes in form (variable-rate, contingent, convertible recharacterizations); implementing regs at Treas. Reg. §§ 1.1275-2 through 1.1275-7".to_string(),
        "Treas. Reg. § 1.1275-1 Adjusted Issue Price Computation — issue price INCREASED by OID previously includible in any holder's gross income AND DECREASED by payments previously made other than payments of qualified stated interest; running computation basis for accrual-period OID inclusions under § 1272".to_string(),
        "Treas. Reg. § 1.1275-2(a) OID Payment Allocation Rule — each payment treated FIRST as payment of OID to extent of OID that has accrued as of date payment is due and has not been allocated to prior payments, and SECOND as payment of principal; OID-first / principal-second rule prevents tax-motivated repayment-ordering schemes".to_string(),
        "Transition Rule (March 13, 2001) — current § 1.1275-1 definitions apply to debt instruments issued ON OR AFTER MARCH 13, 2001; pre-March-13-2001 instruments subject to transitional rules under prior § 1.1275-1 versions".to_string(),
        "Annuity Grandfather Date (April 7, 1995) — annuity contracts purchased before April 7, 1995 or meeting Notice FI-33-94 criteria exempt from certain annuity-related § 1275(a)(1)(B)(i) disqualifying provisions".to_string(),
        "Companion Provisions — § 163(e) issuer-side OID deduction (parallel to § 1272 holder-side inclusion); § 1271 retirement of debt instrument; § 1272 current OID inclusion; § 1273 OID determination; § 1274 issue price for debt-for-property exchanges (built iter 678); § 1276/§ 1277/§ 1278 market discount; § 1281/§ 1282/§ 1283 short-term obligations; § 1286 stripped bonds (built iter 672); § 1287 anti-bearer-bond rule; § 1288 anti-avoidance tax-exempt OID".to_string(),
        "Treasury Regulation Authority — § 1.1275-1 (definitions); § 1.1275-2 (special rules + OID payment allocation); § 1.1275-3 (information reporting); § 1.1275-4 (contingent payment debt instruments); § 1.1275-5 (variable rate debt instruments); § 1.1275-6 (integration of qualifying debt with hedges); § 1.1275-7 (inflation-indexed debt instruments such as TIPS)".to_string(),
        "Cornell LII 26 USC § 1275 — primary statutory text".to_string(),
        "Bloomberg Tax IRC § 1275 — comprehensive code commentary".to_string(),
        "IRS Rev. Rul. 2000-12 — Section 1275 Other Definitions".to_string(),
        "IRS Publication 1212 (12/2025) — Guide to Original Issue Discount (OID) Instruments".to_string(),
        "Federal Register REG-125237-00 (May 7, 2002) — Debt Instruments With Original Issue Discount; Annuity Contracts".to_string(),
    ];

    if input.instrument_type == InstrumentType::NotADebtInstrumentOutsideSection1275Scope {
        return Output {
            mode: Section1275Mode::NotApplicableNotADebtInstrumentOutsideSection1275Scope,
            statutory_basis: "IRC § 1275(a)(1)(A) — applies only to debt instruments constituting indebtedness under federal income tax principles".to_string(),
            notes: "NOT APPLICABLE: instrument is not a debt instrument within § 1275(a)(1)(A); § 1275 OID definitional anchor does not apply.".to_string(),
            citations,
        };
    }

    if input.instrument_type == InstrumentType::AnnuityContractExceptionSection1275A1Bi {
        // Annuity contract exception — check if disqualifying provisions present
        if input.annuity_disqualifying_provision_status
            != AnnuityDisqualifyingProvisionStatus::NoDisqualifyingProvisionsPresent
        {
            return Output {
                mode: Section1275Mode::ViolationAnnuityContractClaimedExceptionButDisqualifyingProvisionPresent,
                statutory_basis: "IRC § 1275(a)(1)(B)(i) + Treas. Reg. § 1.1275-1 — annuity contract exception disqualified by cash surrender option / secured loan availability / maximum payout / decreasing payouts".to_string(),
                notes: format!(
                    "VIOLATION: annuity contract claimed exception under § 1275(a)(1)(B)(i) but disqualifying provision present ({:?}); contract treated as debt instrument subject to OID rules under § 1272.",
                    input.annuity_disqualifying_provision_status
                ),
                citations,
            };
        }
        return Output {
            mode: Section1275Mode::NotApplicableAnnuityContractExceptionSection1275A1Bi,
            statutory_basis: "IRC § 1275(a)(1)(B)(i) — annuity contract exception with no disqualifying provisions".to_string(),
            notes: "NOT APPLICABLE: contract qualifies for § 1275(a)(1)(B)(i) annuity exception; no disqualifying provisions (cash surrender, secured loan, maximum/decreasing payouts) present; OID rules under §§ 1272/1273 do not apply.".to_string(),
            citations,
        };
    }

    if input.instrument_type == InstrumentType::ForeignInsurerAnnuityExceptionSection1275A1Bii {
        return Output {
            mode: Section1275Mode::NotApplicableForeignInsurerAnnuityExceptionSection1275A1Bii,
            statutory_basis: "IRC § 1275(a)(1)(B)(ii) — foreign insurer annuity exception (only if subject to tax under subchapter L)".to_string(),
            notes: "NOT APPLICABLE: annuity contract issued by foreign insurer subject to tax under subchapter L with respect to income earned on the annuity contract; § 1275(a)(1)(B)(ii) exception applies; OID rules under §§ 1272/1273 do not apply.".to_string(),
            citations,
        };
    }

    if input.transition_date_status
        == TransitionDateStatus::DebtInstrumentIssuedBeforeMarch13_2001PreRegulationTransitionalRulesApply
    {
        return Output {
            mode: Section1275Mode::NotApplicablePreMarch13_2001RegulationsTransitionalRule,
            statutory_basis: "Treas. Reg. § 1.1275-1 transition rule — current definitions apply to debt instruments issued on or after March 13, 2001".to_string(),
            notes: "NOT APPLICABLE: debt instrument issued before March 13, 2001; pre-March-13-2001 transitional rules under prior § 1.1275-1 versions apply; current § 1275 definitional anchor does not govern.".to_string(),
            citations,
        };
    }

    match input.compliance_aspect {
        ComplianceAspect::DebtInstrumentClassification => Output {
            mode: Section1275Mode::CompliantDebtInstrumentClassifiedUnderSection1275A1A,
            statutory_basis: "IRC § 1275(a)(1)(A) — debt instrument constituting indebtedness under federal income tax principles".to_string(),
            notes: "COMPLIANT: instrument properly classified as a debt instrument under § 1275(a)(1)(A); OID rules apply under §§ 163(e) (issuer side) and 1272/1273 (holder side).".to_string(),
            citations,
        },
        ComplianceAspect::OidPaymentAllocationOrder => match input.payment_allocation_order {
            PaymentAllocationOrder::OidFirstPrincipalSecondPerTreasReg12752 => Output {
                mode: Section1275Mode::CompliantOidPaymentAllocationOidFirstPrincipalSecondPerTreasReg12752,
                statutory_basis: "Treas. Reg. § 1.1275-2(a) — OID-first / principal-second payment allocation".to_string(),
                notes: "COMPLIANT: payment allocated under Treas. Reg. § 1.1275-2(a) — FIRST as payment of OID to extent of accrued OID not allocated to prior payments, SECOND as payment of principal.".to_string(),
                citations,
            },
            PaymentAllocationOrder::PrincipalFirstOidSecondViolation => Output {
                mode: Section1275Mode::ViolationOidPaymentAllocationPrincipalFirstOidSecond,
                statutory_basis: "Treas. Reg. § 1.1275-2(a) violated — principal-first allocation prohibited".to_string(),
                notes: "VIOLATION: payment allocated PRINCIPAL FIRST / OID SECOND in violation of Treas. Reg. § 1.1275-2(a); prevents tax-motivated repayment-ordering schemes; must reallocate OID first per the regulation.".to_string(),
                citations,
            },
            PaymentAllocationOrder::NotApplicableNoPaymentMade => Output {
                mode: Section1275Mode::CompliantOidPaymentAllocationOidFirstPrincipalSecondPerTreasReg12752,
                statutory_basis: "Treas. Reg. § 1.1275-2(a) — no payment made; allocation rule not triggered".to_string(),
                notes: "COMPLIANT: no payment made on the debt instrument during the period; Treas. Reg. § 1.1275-2(a) OID-first / principal-second allocation rule not yet triggered.".to_string(),
                citations,
            },
        },
        ComplianceAspect::AdjustedIssuePriceComputation => {
            if input.adjusted_issue_price_computation_correct {
                Output {
                    mode: Section1275Mode::CompliantAdjustedIssuePriceComputationCorrect,
                    statutory_basis: "Treas. Reg. § 1.1275-1 — adjusted issue price = issue price + OID previously included − payments other than qualified stated interest".to_string(),
                    notes: "COMPLIANT: adjusted issue price computation correct — issue price INCREASED by OID previously included in any holder's gross income AND DECREASED by payments previously made other than payments of qualified stated interest.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: Section1275Mode::ViolationAdjustedIssuePriceComputationIncorrect,
                    statutory_basis: "Treas. Reg. § 1.1275-1 — adjusted issue price formula not correctly applied".to_string(),
                    notes: "VIOLATION: adjusted issue price computation incorrect; running formula = (issue price + OID previously included) − (payments other than qualified stated interest) must be applied; recomputation required for accrual-period OID inclusions under § 1272.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::PersonalUsePropertyLoanExceptionUnderSection1275B => {
            if input.is_personal_use_property_loan_between_natural_persons {
                Output {
                    mode: Section1275Mode::CompliantPersonalUsePropertyLoanExceptionUnderSection1275BApplies,
                    statutory_basis: "IRC § 1275(b) — personal-use property loan between natural persons not in trade/business of lender".to_string(),
                    notes: "COMPLIANT: loan between natural persons NOT issued in connection with trade or business of lender; § 1275(b) personal-use property loan exception applies; OID rules of §§ 1272 and 1273 do NOT apply to borrower; borrower governed by cash receipts and disbursements method.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: Section1275Mode::CompliantDebtInstrumentClassifiedUnderSection1275A1A,
                    statutory_basis: "IRC § 1275(b) — personal-use property loan exception not satisfied; standard OID rules apply".to_string(),
                    notes: "NOT QUALIFIED: loan does not satisfy § 1275(b) personal-use property loan exception (not between natural persons OR issued in connection with trade/business of lender); standard OID rules under §§ 1272/1273 continue to apply.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::InformationReturnFilingUnderSection6049Section1275C => {
            if input.information_return_filed_under_section_6049 {
                Output {
                    mode: Section1275Mode::CompliantInformationReturnsFiledUnderSection6049Section1275C,
                    statutory_basis: "IRC § 1275(c) + § 6049 — information reporting on Form 1099-OID / Form 1099-INT".to_string(),
                    notes: "COMPLIANT: information returns properly filed under § 6049 (Form 1099-OID for OID and Form 1099-INT for interest); § 1275(c) reporting requirement satisfied.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: Section1275Mode::ViolationInformationReturnNotFiledUnderSection6049,
                    statutory_basis: "IRC § 1275(c) + § 6049 — information return required for OID instruments".to_string(),
                    notes: "VIOLATION: information return NOT filed under § 6049; § 1275(c) requires all relevant OID information be sent to holders + reported to IRS on Form 1099-OID / Form 1099-INT; failure triggers § 6721/§ 6722 information-return penalties.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::AnnuityContractExceptionQualification => {
            if input.annuity_disqualifying_provision_status
                == AnnuityDisqualifyingProvisionStatus::NoDisqualifyingProvisionsPresent
            {
                Output {
                    mode: Section1275Mode::CompliantAnnuityContractExceptionQualifiedNoDisqualifyingProvisions,
                    statutory_basis: "IRC § 1275(a)(1)(B)(i) — annuity contract exception qualified with no disqualifying provisions".to_string(),
                    notes: "COMPLIANT: annuity contract qualifies for § 1275(a)(1)(B)(i) exception; no disqualifying provisions present; treated as life-annuity contract outside the debt-instrument definition.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: Section1275Mode::ViolationAnnuityContractClaimedExceptionButDisqualifyingProvisionPresent,
                    statutory_basis: "IRC § 1275(a)(1)(B)(i) — annuity contract exception disqualified".to_string(),
                    notes: format!(
                        "VIOLATION: annuity contract claimed § 1275(a)(1)(B)(i) exception but disqualifying provision present ({:?}); reclassified as debt instrument subject to OID rules under § 1272.",
                        input.annuity_disqualifying_provision_status
                    ),
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
            instrument_type: InstrumentType::DebtInstrumentUnderSection1275A1A,
            compliance_aspect: ComplianceAspect::DebtInstrumentClassification,
            transition_date_status:
                TransitionDateStatus::DebtInstrumentIssuedOnOrAfterMarch13_2001CurrentRegulationsApply,
            payment_allocation_order: PaymentAllocationOrder::OidFirstPrincipalSecondPerTreasReg12752,
            adjusted_issue_price_computation_correct: true,
            information_return_filed_under_section_6049: true,
            annuity_disqualifying_provision_status:
                AnnuityDisqualifyingProvisionStatus::NoDisqualifyingProvisionsPresent,
            is_personal_use_property_loan_between_natural_persons: false,
        }
    }

    #[test]
    fn not_a_debt_instrument_not_applicable() {
        let mut input = baseline_input();
        input.instrument_type = InstrumentType::NotADebtInstrumentOutsideSection1275Scope;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1275Mode::NotApplicableNotADebtInstrumentOutsideSection1275Scope
        );
    }

    #[test]
    fn annuity_contract_no_disqualifying_provisions_exception() {
        let mut input = baseline_input();
        input.instrument_type = InstrumentType::AnnuityContractExceptionSection1275A1Bi;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1275Mode::NotApplicableAnnuityContractExceptionSection1275A1Bi
        );
    }

    #[test]
    fn annuity_contract_cash_surrender_option_violation() {
        let mut input = baseline_input();
        input.instrument_type = InstrumentType::AnnuityContractExceptionSection1275A1Bi;
        input.annuity_disqualifying_provision_status =
            AnnuityDisqualifyingProvisionStatus::CashSurrenderOptionPresent;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1275Mode::ViolationAnnuityContractClaimedExceptionButDisqualifyingProvisionPresent
        );
    }

    #[test]
    fn annuity_contract_secured_loan_violation() {
        let mut input = baseline_input();
        input.instrument_type = InstrumentType::AnnuityContractExceptionSection1275A1Bi;
        input.annuity_disqualifying_provision_status =
            AnnuityDisqualifyingProvisionStatus::SecuredLoanAvailabilityPresent;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1275Mode::ViolationAnnuityContractClaimedExceptionButDisqualifyingProvisionPresent
        );
    }

    #[test]
    fn annuity_contract_maximum_payout_violation() {
        let mut input = baseline_input();
        input.instrument_type = InstrumentType::AnnuityContractExceptionSection1275A1Bi;
        input.annuity_disqualifying_provision_status =
            AnnuityDisqualifyingProvisionStatus::MaximumPayoutProvisionPresent;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1275Mode::ViolationAnnuityContractClaimedExceptionButDisqualifyingProvisionPresent
        );
    }

    #[test]
    fn foreign_insurer_annuity_exception() {
        let mut input = baseline_input();
        input.instrument_type = InstrumentType::ForeignInsurerAnnuityExceptionSection1275A1Bii;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1275Mode::NotApplicableForeignInsurerAnnuityExceptionSection1275A1Bii
        );
    }

    #[test]
    fn pre_march_13_2001_transitional_rule_not_applicable() {
        let mut input = baseline_input();
        input.transition_date_status =
            TransitionDateStatus::DebtInstrumentIssuedBeforeMarch13_2001PreRegulationTransitionalRulesApply;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1275Mode::NotApplicablePreMarch13_2001RegulationsTransitionalRule
        );
    }

    #[test]
    fn debt_instrument_classification_compliant() {
        let output = check(&baseline_input());
        assert_eq!(
            output.mode,
            Section1275Mode::CompliantDebtInstrumentClassifiedUnderSection1275A1A
        );
    }

    #[test]
    fn oid_payment_allocation_oid_first_principal_second_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::OidPaymentAllocationOrder;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1275Mode::CompliantOidPaymentAllocationOidFirstPrincipalSecondPerTreasReg12752
        );
    }

    #[test]
    fn oid_payment_allocation_principal_first_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::OidPaymentAllocationOrder;
        input.payment_allocation_order = PaymentAllocationOrder::PrincipalFirstOidSecondViolation;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1275Mode::ViolationOidPaymentAllocationPrincipalFirstOidSecond
        );
    }

    #[test]
    fn oid_payment_allocation_no_payment_made_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::OidPaymentAllocationOrder;
        input.payment_allocation_order = PaymentAllocationOrder::NotApplicableNoPaymentMade;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1275Mode::CompliantOidPaymentAllocationOidFirstPrincipalSecondPerTreasReg12752
        );
    }

    #[test]
    fn adjusted_issue_price_computation_correct_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::AdjustedIssuePriceComputation;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1275Mode::CompliantAdjustedIssuePriceComputationCorrect
        );
    }

    #[test]
    fn adjusted_issue_price_computation_incorrect_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::AdjustedIssuePriceComputation;
        input.adjusted_issue_price_computation_correct = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1275Mode::ViolationAdjustedIssuePriceComputationIncorrect
        );
    }

    #[test]
    fn personal_use_property_loan_exception_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::PersonalUsePropertyLoanExceptionUnderSection1275B;
        input.is_personal_use_property_loan_between_natural_persons = true;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1275Mode::CompliantPersonalUsePropertyLoanExceptionUnderSection1275BApplies
        );
    }

    #[test]
    fn personal_use_property_loan_exception_not_satisfied() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::PersonalUsePropertyLoanExceptionUnderSection1275B;
        input.is_personal_use_property_loan_between_natural_persons = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1275Mode::CompliantDebtInstrumentClassifiedUnderSection1275A1A
        );
    }

    #[test]
    fn information_return_filed_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::InformationReturnFilingUnderSection6049Section1275C;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1275Mode::CompliantInformationReturnsFiledUnderSection6049Section1275C
        );
    }

    #[test]
    fn information_return_not_filed_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::InformationReturnFilingUnderSection6049Section1275C;
        input.information_return_filed_under_section_6049 = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1275Mode::ViolationInformationReturnNotFiledUnderSection6049
        );
    }

    #[test]
    fn annuity_contract_exception_qualified_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::AnnuityContractExceptionQualification;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1275Mode::CompliantAnnuityContractExceptionQualifiedNoDisqualifyingProvisions
        );
    }

    #[test]
    fn annuity_contract_exception_disqualified_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::AnnuityContractExceptionQualification;
        input.annuity_disqualifying_provision_status =
            AnnuityDisqualifyingProvisionStatus::DecreasingPayoutsPresent;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1275Mode::ViolationAnnuityContractClaimedExceptionButDisqualifyingProvisionPresent
        );
    }

    #[test]
    fn constants_pin_statutory_facts() {
        assert_eq!(IRC_1275_REGULATIONS_EFFECTIVE_DATE_YEAR, 2001);
        assert_eq!(IRC_1275_REGULATIONS_EFFECTIVE_DATE_MONTH, 3);
        assert_eq!(IRC_1275_REGULATIONS_EFFECTIVE_DATE_DAY, 13);
        assert_eq!(IRC_1275_ANNUITY_GRANDFATHER_DATE_YEAR, 1995);
        assert_eq!(IRC_1275_ANNUITY_GRANDFATHER_DATE_MONTH, 4);
        assert_eq!(IRC_1275_ANNUITY_GRANDFATHER_DATE_DAY, 7);
        assert_eq!(IRC_1275_BASIS_POINT_DENOMINATOR, 10_000);
    }

    #[test]
    fn citation_contains_landmarks() {
        let output = check(&baseline_input());
        let joined = output.citations.join("\n");
        assert!(joined.contains("§ 1275(a)(1)(A)"));
        assert!(joined.contains("§ 1275(a)(1)(B)"));
        assert!(joined.contains("§ 1275(b)"));
        assert!(joined.contains("§ 1275(c)"));
        assert!(joined.contains("§ 1275(d)"));
        assert!(joined.contains("§ 1.1275-1"));
        assert!(joined.contains("§ 1.1275-2"));
        assert!(joined.contains("MARCH 13, 2001"));
        assert!(joined.contains("April 7, 1995"));
        assert!(joined.contains("§ 6049"));
        assert!(joined.contains("§ 1273"));
        assert!(joined.contains("§ 1274"));
        assert!(joined.contains("Form 1099-OID"));
    }
}
