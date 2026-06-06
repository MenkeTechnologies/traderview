//! IRC § 4701 — Tax on Issuer of Registration-Required
//! Obligation Not in Registered Form / TEFRA Issuer Excise
//! Tax Module.
//!
//! Pure-compute check for IRC § 4701 issuer-side excise tax
//! companion to § 1287 (holder-side ordinary income
//! recharacterization — built iter 666). Together § 1287 and
//! § 4701 form the TEFRA anti-bearer-bond duo enacted by
//! Public Law 97-248 § 310 effective for obligations issued
//! after December 31, 1982. § 4701(a) imposes **1 PERCENT of
//! principal amount × number of calendar years** (or portion
//! thereof) during the period from issue date through
//! maturity. § 4701(b) registration-required obligation
//! definition matches § 163(f)(2): excepts individual
//! issuers, non-public obligations, short-term ≤ 1 year, and
//! foreign-targeted Eurobond / TEFRA D obligations.
//!
//! Web research (verified 2026-06-03):
//! - **IRC § 4701(a) Imposition of Tax**: tax is imposed on
//!   any person who issues a registration-required obligation
//!   which is **NOT IN REGISTERED FORM**; tax equal to the
//!   product of **1 PERCENT × principal amount × number of
//!   calendar years (or portions thereof)** during the
//!   period beginning on the date of issuance and ending on
//!   the date of maturity ([Cornell LII 26 USC § 4701](https://www.law.cornell.edu/uscode/text/26/4701);
//!   [Bloomberg Tax Sec. 4701](https://irc.bloombergtax.com/public/uscode/doc/irc/section_4701)).
//! - **IRC § 4701(b) Registration-Required Obligation
//!   Definition**: any debt obligation EXCEPT:
//!   (1) obligation issued by an **INDIVIDUAL**;
//!   (2) obligation **NOT OF A TYPE OFFERED TO THE PUBLIC**;
//!   (3) obligation with **MATURITY OF NOT MORE THAN 1 YEAR**;
//!   (4) obligation **TARGETED TO NON-UNITED STATES PERSONS**
//!   on issuance pursuant to provision frequently referred to
//!   as the **"EUROBOND EXCEPTION"** / **TEFRA D EXCEPTION**
//!   under Treasury Regulations § 1.163-5(c)(2)(i)(D).
//! - **Treas. Reg. § 46.4701-1**: implementing regulation for
//!   § 4701 imposition of tax on issuer ([26 CFR § 46.4701-1](https://www.ecfr.gov/current/title-26/chapter-I/subchapter-D/part-46/subpart-D/section-46.4701-1)).
//! - **Effective Date**: § 4701 applies to obligations
//!   **ISSUED AFTER DECEMBER 31, 1982** (TEFRA enactment;
//!   Public Law 97-248 § 310); exception under § 310(d)(3)
//!   for obligations issued on exercise of warrant or
//!   conversion of convertible obligation if warrant or
//!   obligation was offered or sold outside US without
//!   registration under Securities Act of 1933 and issued
//!   before August 10, 1982.
//! - **HIRE Act of 2010 Amendments**: Hiring Incentives to
//!   Restore Employment Act (Pub. L. 111-147) substantially
//!   narrowed the Eurobond / TEFRA D exception for
//!   obligations issued after **March 18, 2012** — most
//!   foreign-targeted bearer obligations now subject to
//!   § 4701 except for very narrow class of qualifying
//!   obligations ([NYSBA Report on Registered Debt Following
//!   the HIRE Act](https://nysba.org/wp-content/uploads/2025/03/1250-Report.pdf)).
//! - **Codification**: § 4701 is in **Subtitle D Chapter 39**
//!   (Registration-Required Obligations).
//! - **Companion Provisions**: § 1287(a) imposes parallel
//!   ordinary-income recharacterization on holder (built
//!   iter 666); § 1287(a) parenthetical exception applies if
//!   § 4701 tax has been paid by issuer (avoids double
//!   penalty).

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const IRC_4701_TAX_RATE_BASIS_POINTS_PER_YEAR: u64 = 100;
pub const IRC_4701_BASIS_POINT_DENOMINATOR: u64 = 10_000;
pub const IRC_4701_TEFRA_EFFECTIVE_DATE_YEAR: u32 = 1982;
pub const IRC_4701_TEFRA_EFFECTIVE_DATE_MONTH: u32 = 12;
pub const IRC_4701_TEFRA_EFFECTIVE_DATE_DAY: u32 = 31;
pub const IRC_4701_HIRE_ACT_EFFECTIVE_DATE_YEAR: u32 = 2012;
pub const IRC_4701_HIRE_ACT_EFFECTIVE_DATE_MONTH: u32 = 3;
pub const IRC_4701_HIRE_ACT_EFFECTIVE_DATE_DAY: u32 = 18;
pub const IRC_4701_SHORT_TERM_OBLIGATION_MAX_DAYS: u32 = 365;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IssuerType {
    Corporation,
    Partnership,
    FamilyOfficeOrTrust,
    Individual,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ObligationExceptionStatus {
    NoExceptionAppliesRegistrationRequiredObligation,
    IndividualIssuerExceptionUnderSection4701B1,
    NotOfferedToPublicExceptionUnderSection4701B2,
    ShortTermMaturityAtOrUnder1YearExceptionUnderSection4701B3,
    EurobondTefraDForeignTargetedExceptionPreHireAct,
    EurobondTefraDForeignTargetedExceptionPostHireActQualifying,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IssuanceDateStatus {
    IssuedAfterMarch18_2012HireActAmendmentsApply,
    IssuedBetweenJanuary1_1983AndMarch18_2012OriginalTefra,
    IssuedOnOrBeforeDecember31_1982GrandfatheredPreTefra,
    IssuedBeforeAugust10_1982WarrantOrConvertibleExceptionUnderSection310D3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RegistrationStatus {
    IssuedInRegisteredFormUnderSection163F,
    IssuedNotInRegisteredFormBearerOrUnregisteredBookEntry,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section4701Mode {
    NotApplicableObligationInRegisteredForm,
    NotApplicableIndividualIssuerExceptionUnderSection4701B1,
    NotApplicableNotOfferedToPublicExceptionUnderSection4701B2,
    NotApplicableShortTermMaturityExceptionUnderSection4701B3,
    NotApplicableEurobondTefraDExceptionPreHireAct,
    NotApplicableEurobondTefraDExceptionPostHireActQualifying,
    NotApplicablePreTefraGrandfatheredObligation,
    NotApplicableSection310D3WarrantOrConvertibleException,
    ViolationSection4701AIssuerTaxOwedAt1PctPerYearOfPrincipal,
    CompliantSection4701TaxPaidByIssuerAvoidingSection1287AHolderTax,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub issuer_type: IssuerType,
    pub obligation_exception_status: ObligationExceptionStatus,
    pub issuance_date_status: IssuanceDateStatus,
    pub registration_status: RegistrationStatus,
    pub principal_amount_dollars: u64,
    pub years_or_portions_from_issue_to_maturity: u32,
    pub section_4701_tax_paid_by_issuer: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section4701Mode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
    pub section_4701_tax_owed_dollars: u64,
}

pub type Section4701Input = Input;
pub type Section4701Output = Output;
pub type Section4701Result = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "IRC § 4701(a) — tax on any person who issues registration-required obligation NOT IN REGISTERED FORM equal to product of 1 PERCENT × principal amount × number of calendar years (or portions thereof) during period beginning issue date and ending maturity date".to_string(),
        "IRC § 4701(b) Registration-Required Obligation Definition — any debt obligation EXCEPT: (1) issued by INDIVIDUAL; (2) NOT OF A TYPE OFFERED TO THE PUBLIC; (3) MATURITY OF NOT MORE THAN 1 YEAR; (4) TARGETED TO NON-UNITED STATES PERSONS on issuance ('EUROBOND EXCEPTION' / 'TEFRA D EXCEPTION' under Treas. Reg. § 1.163-5(c)(2)(i)(D))".to_string(),
        "Treas. Reg. § 46.4701-1 — implementing regulation for § 4701 imposition of tax on issuer".to_string(),
        "Effective Date — § 4701 applies to obligations ISSUED AFTER DECEMBER 31, 1982 (TEFRA enactment; Public Law 97-248 § 310); exception under § 310(d)(3) for obligations issued on exercise of warrant or conversion of convertible obligation if warrant or obligation was offered or sold outside US without registration under Securities Act of 1933 and issued before August 10, 1982".to_string(),
        "HIRE Act of 2010 Amendments — Hiring Incentives to Restore Employment Act (Pub. L. 111-147) substantially narrowed Eurobond / TEFRA D exception for obligations issued after March 18, 2012; most foreign-targeted bearer obligations now subject to § 4701 except very narrow class of qualifying obligations".to_string(),
        "Codification — § 4701 is in Subtitle D Chapter 39 (Registration-Required Obligations)".to_string(),
        "Companion Provisions — § 1287(a) imposes parallel ordinary-income recharacterization on holder (built iter 666); § 1287(a) parenthetical exception applies if § 4701 tax has been paid by issuer (avoids double penalty)".to_string(),
        "TEFRA (Tax Equity and Fiscal Responsibility Act of 1982; Public Law 97-248) — original enactment of § 4701 + § 1287 as part of broader effort to eliminate bearer bond market".to_string(),
        "NYSBA Report on Registered Debt Following the HIRE Act — comprehensive analysis of post-2012 TEFRA D exception scope".to_string(),
        "Cornell LII 26 USC § 4701 — primary statutory text".to_string(),
        "Bloomberg Tax Sec. 4701 — comprehensive code commentary".to_string(),
        "Federal Register 82 FR 43773 (September 19, 2017) — Guidance on the Definition of Registered Form".to_string(),
    ];

    if input.registration_status == RegistrationStatus::IssuedInRegisteredFormUnderSection163F {
        return Output {
            mode: Section4701Mode::NotApplicableObligationInRegisteredForm,
            statutory_basis: "IRC § 4701(a) — applies only to obligations NOT in registered form".to_string(),
            notes: "NOT APPLICABLE: obligation issued in registered form under § 163(f); § 4701(a) issuer excise tax does not apply.".to_string(),
            citations,
            section_4701_tax_owed_dollars: 0,
        };
    }

    if matches!(
        input.issuance_date_status,
        IssuanceDateStatus::IssuedOnOrBeforeDecember31_1982GrandfatheredPreTefra
    ) {
        return Output {
            mode: Section4701Mode::NotApplicablePreTefraGrandfatheredObligation,
            statutory_basis: "TEFRA effective date — § 4701 applies only to obligations issued after December 31, 1982".to_string(),
            notes: "NOT APPLICABLE: obligation issued on or before December 31, 1982; pre-TEFRA grandfathered obligation; § 4701 does not apply.".to_string(),
            citations,
            section_4701_tax_owed_dollars: 0,
        };
    }

    if matches!(
        input.issuance_date_status,
        IssuanceDateStatus::IssuedBeforeAugust10_1982WarrantOrConvertibleExceptionUnderSection310D3
    ) {
        return Output {
            mode: Section4701Mode::NotApplicableSection310D3WarrantOrConvertibleException,
            statutory_basis: "TEFRA § 310(d)(3) — warrant or convertible obligation exception".to_string(),
            notes: "NOT APPLICABLE: obligation issued on exercise of warrant or conversion of convertible obligation; warrant or original obligation issued before August 10, 1982 outside US without 1933 Act registration; TEFRA § 310(d)(3) exception applies.".to_string(),
            citations,
            section_4701_tax_owed_dollars: 0,
        };
    }

    match input.obligation_exception_status {
        ObligationExceptionStatus::IndividualIssuerExceptionUnderSection4701B1 => {
            return Output {
                mode: Section4701Mode::NotApplicableIndividualIssuerExceptionUnderSection4701B1,
                statutory_basis: "IRC § 4701(b)(1) — individual issuer exception".to_string(),
                notes: "NOT APPLICABLE: obligation issued by individual; § 4701(b)(1) individual issuer exception applies; no issuer tax owed.".to_string(),
                citations,
                section_4701_tax_owed_dollars: 0,
            };
        }
        ObligationExceptionStatus::NotOfferedToPublicExceptionUnderSection4701B2 => {
            return Output {
                mode: Section4701Mode::NotApplicableNotOfferedToPublicExceptionUnderSection4701B2,
                statutory_basis: "IRC § 4701(b)(2) — not offered to public exception".to_string(),
                notes: "NOT APPLICABLE: obligation not of a type offered to public; § 4701(b)(2) exception applies; no issuer tax owed.".to_string(),
                citations,
                section_4701_tax_owed_dollars: 0,
            };
        }
        ObligationExceptionStatus::ShortTermMaturityAtOrUnder1YearExceptionUnderSection4701B3 => {
            return Output {
                mode: Section4701Mode::NotApplicableShortTermMaturityExceptionUnderSection4701B3,
                statutory_basis: "IRC § 4701(b)(3) — short-term maturity ≤ 1 year exception".to_string(),
                notes: "NOT APPLICABLE: obligation maturity ≤ 1 year (365 days); § 4701(b)(3) short-term exception applies; no issuer tax owed.".to_string(),
                citations,
                section_4701_tax_owed_dollars: 0,
            };
        }
        ObligationExceptionStatus::EurobondTefraDForeignTargetedExceptionPreHireAct => {
            if input.issuance_date_status
                == IssuanceDateStatus::IssuedAfterMarch18_2012HireActAmendmentsApply
            {
                // HIRE Act narrowed Eurobond exception; pre-HIRE Eurobond claim post-HIRE invalid
                // Fall through to compute tax
            } else {
                return Output {
                    mode: Section4701Mode::NotApplicableEurobondTefraDExceptionPreHireAct,
                    statutory_basis: "IRC § 4701(b)(4) + Treas. Reg. § 1.163-5(c)(2)(i)(D) — pre-HIRE Eurobond / TEFRA D exception".to_string(),
                    notes: "NOT APPLICABLE: obligation issued before March 18, 2012 with pre-HIRE Eurobond / TEFRA D foreign-targeted exception under Treas. Reg. § 1.163-5(c)(2)(i)(D); no issuer tax owed.".to_string(),
                    citations,
                    section_4701_tax_owed_dollars: 0,
                };
            }
        }
        ObligationExceptionStatus::EurobondTefraDForeignTargetedExceptionPostHireActQualifying => {
            return Output {
                mode: Section4701Mode::NotApplicableEurobondTefraDExceptionPostHireActQualifying,
                statutory_basis: "IRC § 4701(b)(4) + HIRE Act narrowed Eurobond / TEFRA D exception".to_string(),
                notes: "NOT APPLICABLE: obligation qualifies for narrowed post-HIRE Act Eurobond / TEFRA D exception under § 4701(b)(4); narrow class of qualifying obligations exempt; no issuer tax owed.".to_string(),
                citations,
                section_4701_tax_owed_dollars: 0,
            };
        }
        ObligationExceptionStatus::NoExceptionAppliesRegistrationRequiredObligation => {}
    }

    if input.issuer_type == IssuerType::Individual {
        return Output {
            mode: Section4701Mode::NotApplicableIndividualIssuerExceptionUnderSection4701B1,
            statutory_basis: "IRC § 4701(b)(1) — individual issuer exception".to_string(),
            notes: "NOT APPLICABLE: issuer is an individual; § 4701(b)(1) individual issuer exception applies regardless of exception status code; no issuer tax owed.".to_string(),
            citations,
            section_4701_tax_owed_dollars: 0,
        };
    }

    let section_4701_tax_owed_dollars = input
        .principal_amount_dollars
        .saturating_mul(IRC_4701_TAX_RATE_BASIS_POINTS_PER_YEAR)
        .saturating_mul(u64::from(input.years_or_portions_from_issue_to_maturity))
        / IRC_4701_BASIS_POINT_DENOMINATOR;

    if input.section_4701_tax_paid_by_issuer {
        return Output {
            mode: Section4701Mode::CompliantSection4701TaxPaidByIssuerAvoidingSection1287AHolderTax,
            statutory_basis: "IRC § 4701(a) tax paid + § 1287(a) parenthetical holder exception".to_string(),
            notes: format!(
                "COMPLIANT: issuer paid § 4701 tax of ${} (1 % × ${} × {} years); per § 1287(a) parenthetical, holder gain on disposition retains capital character despite obligation being in unregistered form.",
                section_4701_tax_owed_dollars,
                input.principal_amount_dollars,
                input.years_or_portions_from_issue_to_maturity
            ),
            citations,
            section_4701_tax_owed_dollars,
        };
    }

    Output {
        mode: Section4701Mode::ViolationSection4701AIssuerTaxOwedAt1PctPerYearOfPrincipal,
        statutory_basis: "IRC § 4701(a) — issuer tax owed at 1 % per year of principal × years from issue to maturity".to_string(),
        notes: format!(
            "VIOLATION: § 4701(a) imposes issuer excise tax of ${} (1 % × ${} principal × {} years from issue to maturity) on registration-required obligation not in registered form; tax not yet paid by issuer.",
            section_4701_tax_owed_dollars,
            input.principal_amount_dollars,
            input.years_or_portions_from_issue_to_maturity
        ),
        citations,
        section_4701_tax_owed_dollars,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_violation_corporate_issuer() -> Input {
        Input {
            issuer_type: IssuerType::Corporation,
            obligation_exception_status:
                ObligationExceptionStatus::NoExceptionAppliesRegistrationRequiredObligation,
            issuance_date_status: IssuanceDateStatus::IssuedAfterMarch18_2012HireActAmendmentsApply,
            registration_status:
                RegistrationStatus::IssuedNotInRegisteredFormBearerOrUnregisteredBookEntry,
            principal_amount_dollars: 1_000_000,
            years_or_portions_from_issue_to_maturity: 5,
            section_4701_tax_paid_by_issuer: false,
        }
    }

    #[test]
    fn obligation_in_registered_form_not_applicable() {
        let input = Input {
            registration_status: RegistrationStatus::IssuedInRegisteredFormUnderSection163F,
            ..baseline_violation_corporate_issuer()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section4701Mode::NotApplicableObligationInRegisteredForm
        );
        assert_eq!(result.section_4701_tax_owed_dollars, 0);
    }

    #[test]
    fn pre_tefra_grandfathered_not_applicable() {
        let input = Input {
            issuance_date_status:
                IssuanceDateStatus::IssuedOnOrBeforeDecember31_1982GrandfatheredPreTefra,
            ..baseline_violation_corporate_issuer()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section4701Mode::NotApplicablePreTefraGrandfatheredObligation
        );
    }

    #[test]
    fn section_310_d3_warrant_convertible_exception_not_applicable() {
        let input = Input {
            issuance_date_status:
                IssuanceDateStatus::IssuedBeforeAugust10_1982WarrantOrConvertibleExceptionUnderSection310D3,
            ..baseline_violation_corporate_issuer()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section4701Mode::NotApplicableSection310D3WarrantOrConvertibleException
        );
    }

    #[test]
    fn individual_issuer_exception_not_applicable() {
        let input = Input {
            obligation_exception_status:
                ObligationExceptionStatus::IndividualIssuerExceptionUnderSection4701B1,
            ..baseline_violation_corporate_issuer()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section4701Mode::NotApplicableIndividualIssuerExceptionUnderSection4701B1
        );
    }

    #[test]
    fn individual_issuer_type_not_applicable() {
        let input = Input {
            issuer_type: IssuerType::Individual,
            ..baseline_violation_corporate_issuer()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section4701Mode::NotApplicableIndividualIssuerExceptionUnderSection4701B1
        );
    }

    #[test]
    fn not_offered_to_public_exception_not_applicable() {
        let input = Input {
            obligation_exception_status:
                ObligationExceptionStatus::NotOfferedToPublicExceptionUnderSection4701B2,
            ..baseline_violation_corporate_issuer()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section4701Mode::NotApplicableNotOfferedToPublicExceptionUnderSection4701B2
        );
    }

    #[test]
    fn short_term_maturity_exception_not_applicable() {
        let input = Input {
            obligation_exception_status:
                ObligationExceptionStatus::ShortTermMaturityAtOrUnder1YearExceptionUnderSection4701B3,
            ..baseline_violation_corporate_issuer()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section4701Mode::NotApplicableShortTermMaturityExceptionUnderSection4701B3
        );
    }

    #[test]
    fn eurobond_tefra_d_pre_hire_act_not_applicable() {
        let input = Input {
            obligation_exception_status:
                ObligationExceptionStatus::EurobondTefraDForeignTargetedExceptionPreHireAct,
            issuance_date_status:
                IssuanceDateStatus::IssuedBetweenJanuary1_1983AndMarch18_2012OriginalTefra,
            ..baseline_violation_corporate_issuer()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section4701Mode::NotApplicableEurobondTefraDExceptionPreHireAct
        );
    }

    #[test]
    fn eurobond_pre_hire_claimed_post_hire_act_violation() {
        let input = Input {
            obligation_exception_status:
                ObligationExceptionStatus::EurobondTefraDForeignTargetedExceptionPreHireAct,
            issuance_date_status: IssuanceDateStatus::IssuedAfterMarch18_2012HireActAmendmentsApply,
            ..baseline_violation_corporate_issuer()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section4701Mode::ViolationSection4701AIssuerTaxOwedAt1PctPerYearOfPrincipal
        );
    }

    #[test]
    fn eurobond_tefra_d_post_hire_act_qualifying_not_applicable() {
        let input = Input {
            obligation_exception_status:
                ObligationExceptionStatus::EurobondTefraDForeignTargetedExceptionPostHireActQualifying,
            ..baseline_violation_corporate_issuer()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section4701Mode::NotApplicableEurobondTefraDExceptionPostHireActQualifying
        );
    }

    #[test]
    fn baseline_corporate_5_year_violation() {
        let result = check(&baseline_violation_corporate_issuer());
        assert_eq!(
            result.mode,
            Section4701Mode::ViolationSection4701AIssuerTaxOwedAt1PctPerYearOfPrincipal
        );
        assert_eq!(result.section_4701_tax_owed_dollars, 50_000);
    }

    #[test]
    fn corporate_10_year_tax_calculation() {
        let input = Input {
            years_or_portions_from_issue_to_maturity: 10,
            ..baseline_violation_corporate_issuer()
        };
        let result = check(&input);
        assert_eq!(result.section_4701_tax_owed_dollars, 100_000);
    }

    #[test]
    fn corporate_2_year_tax_calculation() {
        let input = Input {
            years_or_portions_from_issue_to_maturity: 2,
            principal_amount_dollars: 5_000_000,
            ..baseline_violation_corporate_issuer()
        };
        let result = check(&input);
        assert_eq!(result.section_4701_tax_owed_dollars, 100_000);
    }

    #[test]
    fn section_4701_tax_paid_compliant_avoids_section_1287_a() {
        let input = Input {
            section_4701_tax_paid_by_issuer: true,
            ..baseline_violation_corporate_issuer()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section4701Mode::CompliantSection4701TaxPaidByIssuerAvoidingSection1287AHolderTax
        );
        assert_eq!(result.section_4701_tax_owed_dollars, 50_000);
    }

    #[test]
    fn partnership_issuer_violation() {
        let input = Input {
            issuer_type: IssuerType::Partnership,
            ..baseline_violation_corporate_issuer()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section4701Mode::ViolationSection4701AIssuerTaxOwedAt1PctPerYearOfPrincipal
        );
    }

    #[test]
    fn family_office_or_trust_issuer_violation() {
        let input = Input {
            issuer_type: IssuerType::FamilyOfficeOrTrust,
            ..baseline_violation_corporate_issuer()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section4701Mode::ViolationSection4701AIssuerTaxOwedAt1PctPerYearOfPrincipal
        );
    }

    #[test]
    fn citations_pin_section_4701_subsections_and_tefra_hire() {
        let result = check(&baseline_violation_corporate_issuer());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("IRC § 4701(a)"));
        assert!(joined.contains("IRC § 4701(b)"));
        assert!(joined.contains("1 PERCENT"));
        assert!(joined.contains("principal amount"));
        assert!(joined.contains("calendar years"));
        assert!(joined.contains("NOT IN REGISTERED FORM"));
        assert!(joined.contains("INDIVIDUAL"));
        assert!(joined.contains("NOT OF A TYPE OFFERED TO THE PUBLIC"));
        assert!(joined.contains("MATURITY OF NOT MORE THAN 1 YEAR"));
        assert!(joined.contains("EUROBOND EXCEPTION"));
        assert!(joined.contains("TEFRA D EXCEPTION"));
        assert!(joined.contains("Treas. Reg. § 46.4701-1"));
        assert!(joined.contains("DECEMBER 31, 1982"));
        assert!(joined.contains("Public Law 97-248"));
        assert!(joined.contains("§ 310(d)(3)"));
        assert!(joined.contains("August 10, 1982"));
        assert!(joined.contains("HIRE Act"));
        assert!(joined.contains("Pub. L. 111-147"));
        assert!(joined.contains("March 18, 2012"));
        assert!(joined.contains("§ 1287(a)"));
        assert!(joined.contains("Subtitle D Chapter 39"));
        assert!(joined.contains("Federal Register 82 FR 43773"));
    }

    #[test]
    fn constant_pin_tefra_hire_dates_and_tax_rate() {
        assert_eq!(IRC_4701_TAX_RATE_BASIS_POINTS_PER_YEAR, 100);
        assert_eq!(IRC_4701_BASIS_POINT_DENOMINATOR, 10_000);
        assert_eq!(IRC_4701_TEFRA_EFFECTIVE_DATE_YEAR, 1982);
        assert_eq!(IRC_4701_TEFRA_EFFECTIVE_DATE_MONTH, 12);
        assert_eq!(IRC_4701_TEFRA_EFFECTIVE_DATE_DAY, 31);
        assert_eq!(IRC_4701_HIRE_ACT_EFFECTIVE_DATE_YEAR, 2012);
        assert_eq!(IRC_4701_HIRE_ACT_EFFECTIVE_DATE_MONTH, 3);
        assert_eq!(IRC_4701_HIRE_ACT_EFFECTIVE_DATE_DAY, 18);
        assert_eq!(IRC_4701_SHORT_TERM_OBLIGATION_MAX_DAYS, 365);
    }

    #[test]
    fn saturating_overflow_defense_extreme_principal() {
        let input = Input {
            principal_amount_dollars: u64::MAX,
            years_or_portions_from_issue_to_maturity: u32::MAX,
            ..baseline_violation_corporate_issuer()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section4701Mode::ViolationSection4701AIssuerTaxOwedAt1PctPerYearOfPrincipal
        );
    }
}
