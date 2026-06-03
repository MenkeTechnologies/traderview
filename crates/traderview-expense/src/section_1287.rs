//! IRC § 1287 — Denial of Capital Gain Treatment for Gains on
//! Certain Obligations Not in Registered Form / Anti-Bearer
//! Bond Rule Module.
//!
//! Pure-compute check for IRC § 1287 anti-bearer-bond rule
//! that converts capital gain on disposition of registration-
//! required obligations not in registered form into ORDINARY
//! INCOME. Enacted as part of TEFRA (Tax Equity and Fiscal
//! Responsibility Act of 1982) effective for obligations
//! issued after December 31, 1982. Trader-relevant for family
//! offices and international portfolio operators holding
//! legacy bearer bonds, off-shore-issued bonds, or other
//! non-registered debt instruments — § 1287 converts what
//! would otherwise be preferential-rate LTCG into ordinary
//! income, while the issuer faces § 4701 tax of 1 % per year
//! on the face amount.
//!
//! Web research (verified 2026-06-03):
//! - **IRC § 1287(a) General Rule**: if any **registration-
//!   required obligation** is not in registered form, any
//!   gain on the sale or other disposition of such obligation
//!   shall be treated as **ORDINARY INCOME** (unless the
//!   issuance of such obligation was subject to tax under
//!   § 4701) ([Cornell LII 26 USC § 1287 / Bloomberg Tax Sec.
//!   1287](https://irc.bloombergtax.com/public/uscode/doc/irc/section_1287);
//!   [26 CFR § 1.1287-1](https://www.ecfr.gov/current/title-26/chapter-I/subchapter-A/part-1/subject-group-ECFR56edfa33b27e3cf/section-1.1287-1)).
//! - **IRC § 1287(b)(1) "Registration-Required Obligation"
//!   Definition**: has the meaning given by **§ 163(f)(2)** —
//!   any obligation EXCEPT (i) obligations with maturity (at
//!   issue) of **NOT MORE THAN 1 YEAR**; (ii) obligations not
//!   of a type offered to public; (iii) certain foreign-
//!   targeted obligations meeting Treasury foreign-targeting
//!   conditions ([26 USC § 163(f)(2) definition](https://www.law.cornell.edu/definitions/uscode.php?width=840&height=800&iframe=true&def_id=26-USC-650521995-1977781534&term_occur=999)).
//! - **IRC § 1287(b)(2) "Registered Form" Definition**: has
//!   the same meaning as when used in § 163(f) — generally
//!   means issued in form that requires transfer through book
//!   entry or other system that records the holder's identity
//!   on the books of the issuer or its agent.
//! - **§ 4701 Issuer Tax**: separate excise tax on issuer of
//!   registration-required obligation not in registered form
//!   — **1 % per year** of the obligation face amount (for
//!   number of complete calendar years between issue date and
//!   maturity); § 1287(a) does not apply if § 4701 tax has
//!   been paid by issuer.
//! - **Effective Date**: § 1287 applies to obligations
//!   **ISSUED AFTER DECEMBER 31, 1982** (TEFRA enactment;
//!   Public Law 97-248 § 310).
//! - **Treas. Reg. § 1.165-12(c) Holder Exception**: holder
//!   will NOT be subject to § 1287(a) if holder meets the
//!   conditions of Treas. Reg. § 1.165-12(c) — generally
//!   requires holder to demonstrate (a) obligation was held in
//!   registered form, OR (b) holder had no actual knowledge of
//!   the registration failure and exercised reasonable care.
//! - **September 2017 Treasury Proposed Regulations**:
//!   updated the definition of "registered form" to address
//!   modern bond market practices (Notice 2012-20 and Federal
//!   Register 82 FR 43773 of September 19, 2017) ([Federal
//!   Register — Guidance on the Definition of Registered
//!   Form](https://www.federalregister.gov/documents/2017/09/19/2017-19753/guidance-on-the-definition-of-registered-form);
//!   [IRS Notice 2012-20](https://www.irs.gov/pub/irs-drop/n-12-20.pdf)).
//! - **Cross-Reference with § 1276 Market Discount**: § 1287
//!   ordinary income treatment runs in addition to (not in
//!   lieu of) § 1276 accrued market discount conversion to
//!   ordinary income; whichever is more taxpayer-adverse may
//!   apply depending on facts.
//! - **Pre-TEFRA Bearer Bond Legacy**: obligations issued
//!   before January 1, 1983 are GRANDFATHERED outside § 1287
//!   regardless of registration status; many family-office
//!   legacy holdings are pre-TEFRA bearer bonds that escape
//!   § 1287 application.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const IRC_1287_TEFRA_EFFECTIVE_DATE_YEAR: u32 = 1982;
pub const IRC_1287_TEFRA_EFFECTIVE_DATE_MONTH: u32 = 12;
pub const IRC_1287_TEFRA_EFFECTIVE_DATE_DAY: u32 = 31;
pub const IRC_4701_ISSUER_TAX_RATE_BASIS_POINTS: u64 = 100;
pub const IRC_4701_BASIS_POINT_DENOMINATOR: u64 = 10_000;
pub const IRC_163_F_2_SHORT_TERM_OBLIGATION_MAX_DAYS: u32 = 365;
pub const TREAS_REG_REGISTERED_FORM_2017_UPDATE_YEAR: u32 = 2017;
pub const TREAS_REG_REGISTERED_FORM_2017_UPDATE_MONTH: u32 = 9;
pub const TREAS_REG_REGISTERED_FORM_2017_UPDATE_DAY: u32 = 19;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ObligationClassification {
    RegistrationRequiredObligationUnderSection163F2,
    ShortTermObligationMaturityAtOrUnder1YearExempt,
    NotOfferedToPublicExempt,
    ForeignTargetedObligationMeetingTreasuryConditionsExempt,
    NotADebtObligationAtAll,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RegistrationStatus {
    HeldInRegisteredFormUnderSection163F,
    NotInRegisteredFormBearerOrUnregisteredBookEntry,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IssuanceDateStatus {
    IssuedAfterDecember31_1982TefraApplies,
    IssuedOnOrBeforeDecember31_1982GrandfatheredPreTefra,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section4701TaxStatus {
    Section4701TaxPaidByIssuerSection1287AInapplicable,
    Section4701TaxNotPaidByIssuer,
    NotApplicableObligationInRegisteredForm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HolderExceptionStatus {
    HolderQualifiesUnderTreasReg1_165_12C,
    HolderHadActualKnowledgeOfRegistrationFailureNoException,
    HolderExceptionNotClaimedOrNotApplicable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section1287Mode {
    NotApplicableNotARegistrationRequiredObligation,
    NotApplicableShortTermObligationUnder1YearMaturity,
    NotApplicablePreTefraGrandfatheredObligation,
    NotApplicableObligationHeldInRegisteredForm,
    NotApplicableSection4701IssuerTaxPaid,
    NotApplicableHolderExceptionUnderTreasReg1_165_12C,
    CompliantNoDispositionGainOrLoss,
    CompliantPreservedLtcgTreatmentRegisteredOrExempt,
    ViolationSection1287AConvertsGainToOrdinaryIncome,
    ViolationSection1287AClaimedAsAvoidedButHolderHadActualKnowledge,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub obligation_classification: ObligationClassification,
    pub registration_status: RegistrationStatus,
    pub issuance_date_status: IssuanceDateStatus,
    pub section_4701_tax_status: Section4701TaxStatus,
    pub holder_exception_status: HolderExceptionStatus,
    pub gain_on_disposition_dollars: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section1287Mode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
    pub gain_recharacterized_as_ordinary_income_dollars: u64,
    pub gain_preserved_as_capital_dollars: u64,
}

pub type Section1287Input = Input;
pub type Section1287Output = Output;
pub type Section1287Result = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "IRC § 1287(a) — if any registration-required obligation is NOT in registered form, any gain on the sale or other disposition of such obligation shall be treated as ORDINARY INCOME (unless the issuance of such obligation was subject to tax under § 4701)".to_string(),
        "IRC § 1287(b)(1) — 'Registration-Required Obligation' has the meaning given by § 163(f)(2) — any obligation EXCEPT (i) obligations with maturity at issue of NOT MORE THAN 1 YEAR; (ii) obligations not of a type offered to public; (iii) certain foreign-targeted obligations meeting Treasury foreign-targeting conditions".to_string(),
        "IRC § 1287(b)(2) — 'Registered Form' has the same meaning as when used in § 163(f) — generally means issued in form that requires transfer through book entry or other system that records holder's identity on books of issuer or its agent".to_string(),
        "IRC § 4701 — separate excise tax on issuer of registration-required obligation not in registered form — 1 % per year of obligation face amount (for number of complete calendar years between issue date and maturity); § 1287(a) does not apply if § 4701 tax has been paid by issuer".to_string(),
        "Effective Date — § 1287 applies to obligations ISSUED AFTER DECEMBER 31, 1982 (TEFRA enactment; Public Law 97-248 § 310)".to_string(),
        "Treas. Reg. § 1.165-12(c) Holder Exception — holder will NOT be subject to § 1287(a) if holder meets conditions of Treas. Reg. § 1.165-12(c); generally requires holder to demonstrate (a) obligation was held in registered form, OR (b) holder had no actual knowledge of registration failure and exercised reasonable care".to_string(),
        "September 2017 Treasury Proposed Regulations — updated the definition of 'registered form' to address modern bond market practices (Notice 2012-20 and Federal Register 82 FR 43773 of September 19, 2017)".to_string(),
        "Cross-Reference with § 1276 Market Discount — § 1287 ordinary income treatment runs in addition to (not in lieu of) § 1276 accrued market discount conversion to ordinary income; whichever is more taxpayer-adverse may apply depending on facts".to_string(),
        "Pre-TEFRA Bearer Bond Legacy — obligations issued before January 1, 1983 are GRANDFATHERED outside § 1287 regardless of registration status; many family-office legacy holdings are pre-TEFRA bearer bonds that escape § 1287 application".to_string(),
        "26 CFR § 1.1287-1 — implementing regulation for § 1287(a) anti-bearer-bond rule".to_string(),
        "TEFRA (Tax Equity and Fiscal Responsibility Act of 1982; Public Law 97-248) — original enactment of § 1287 + § 4701 as part of broader effort to eliminate bearer bond market".to_string(),
        "Cornell LII 26 USC § 163(f) — registered form definition cross-reference".to_string(),
        "Bloomberg Tax Sec. 1287 — comprehensive code commentary".to_string(),
        "Federal Register 82 FR 43773 (September 19, 2017) — Guidance on the Definition of Registered Form".to_string(),
    ];

    if input.obligation_classification == ObligationClassification::NotADebtObligationAtAll {
        return Output {
            mode: Section1287Mode::NotApplicableNotARegistrationRequiredObligation,
            statutory_basis: "IRC § 1287 — applies only to debt obligations".to_string(),
            notes: "NOT APPLICABLE: instrument is not a debt obligation; § 1287 anti-bearer-bond rule does not apply.".to_string(),
            citations,
            gain_recharacterized_as_ordinary_income_dollars: 0,
            gain_preserved_as_capital_dollars: input.gain_on_disposition_dollars,
        };
    }

    if input.obligation_classification
        == ObligationClassification::ShortTermObligationMaturityAtOrUnder1YearExempt
    {
        return Output {
            mode: Section1287Mode::NotApplicableShortTermObligationUnder1YearMaturity,
            statutory_basis: "IRC § 163(f)(2) — short-term obligation ≤ 1 year exempt from registration".to_string(),
            notes: "NOT APPLICABLE: obligation maturity at issue ≤ 1 year (365 days); short-term obligation exempt from registration requirement under § 163(f)(2)(A); § 1287 inapplicable; gain preserves capital character.".to_string(),
            citations,
            gain_recharacterized_as_ordinary_income_dollars: 0,
            gain_preserved_as_capital_dollars: input.gain_on_disposition_dollars,
        };
    }

    if matches!(
        input.obligation_classification,
        ObligationClassification::NotOfferedToPublicExempt
            | ObligationClassification::ForeignTargetedObligationMeetingTreasuryConditionsExempt
    ) {
        return Output {
            mode: Section1287Mode::NotApplicableNotARegistrationRequiredObligation,
            statutory_basis: "IRC § 163(f)(2) — obligation not within registration-required category".to_string(),
            notes: format!(
                "NOT APPLICABLE: obligation classification ({:?}) is not a registration-required obligation under § 163(f)(2); § 1287 inapplicable; gain preserves capital character.",
                input.obligation_classification
            ),
            citations,
            gain_recharacterized_as_ordinary_income_dollars: 0,
            gain_preserved_as_capital_dollars: input.gain_on_disposition_dollars,
        };
    }

    if input.issuance_date_status
        == IssuanceDateStatus::IssuedOnOrBeforeDecember31_1982GrandfatheredPreTefra
    {
        return Output {
            mode: Section1287Mode::NotApplicablePreTefraGrandfatheredObligation,
            statutory_basis: "TEFRA effective date — § 1287 applies only to obligations issued after December 31, 1982".to_string(),
            notes: "NOT APPLICABLE: obligation issued on or before December 31, 1982; pre-TEFRA grandfathered obligation; § 1287 does not apply regardless of registration status; gain preserves capital character.".to_string(),
            citations,
            gain_recharacterized_as_ordinary_income_dollars: 0,
            gain_preserved_as_capital_dollars: input.gain_on_disposition_dollars,
        };
    }

    if input.registration_status == RegistrationStatus::HeldInRegisteredFormUnderSection163F {
        return Output {
            mode: Section1287Mode::NotApplicableObligationHeldInRegisteredForm,
            statutory_basis: "IRC § 1287(a) — applies only to obligations NOT in registered form".to_string(),
            notes: "NOT APPLICABLE: obligation is held in registered form under § 163(f); § 1287(a) ordinary income recharacterization does not apply; gain preserves capital character.".to_string(),
            citations,
            gain_recharacterized_as_ordinary_income_dollars: 0,
            gain_preserved_as_capital_dollars: input.gain_on_disposition_dollars,
        };
    }

    if input.section_4701_tax_status
        == Section4701TaxStatus::Section4701TaxPaidByIssuerSection1287AInapplicable
    {
        return Output {
            mode: Section1287Mode::NotApplicableSection4701IssuerTaxPaid,
            statutory_basis: "IRC § 1287(a) parenthetical — exception when § 4701 tax paid".to_string(),
            notes: "NOT APPLICABLE: issuer paid § 4701 tax (1 % per year of face amount) on registration failure; § 1287(a) parenthetical exception applies; gain preserves capital character.".to_string(),
            citations,
            gain_recharacterized_as_ordinary_income_dollars: 0,
            gain_preserved_as_capital_dollars: input.gain_on_disposition_dollars,
        };
    }

    if input.holder_exception_status
        == HolderExceptionStatus::HolderHadActualKnowledgeOfRegistrationFailureNoException
    {
        return Output {
            mode: Section1287Mode::ViolationSection1287AClaimedAsAvoidedButHolderHadActualKnowledge,
            statutory_basis: "Treas. Reg. § 1.165-12(c) — holder exception requires no actual knowledge".to_string(),
            notes: format!(
                "VIOLATION: holder had actual knowledge of registration failure and cannot invoke Treas. Reg. § 1.165-12(c) holder exception; § 1287(a) converts ${} of gain to ordinary income.",
                input.gain_on_disposition_dollars
            ),
            citations,
            gain_recharacterized_as_ordinary_income_dollars: input.gain_on_disposition_dollars,
            gain_preserved_as_capital_dollars: 0,
        };
    }

    if input.holder_exception_status == HolderExceptionStatus::HolderQualifiesUnderTreasReg1_165_12C
    {
        return Output {
            mode: Section1287Mode::NotApplicableHolderExceptionUnderTreasReg1_165_12C,
            statutory_basis: "Treas. Reg. § 1.165-12(c) — holder exception applies".to_string(),
            notes: "NOT APPLICABLE: holder qualifies for Treas. Reg. § 1.165-12(c) exception (held in registered form or no actual knowledge with reasonable care); § 1287(a) recharacterization does not apply.".to_string(),
            citations,
            gain_recharacterized_as_ordinary_income_dollars: 0,
            gain_preserved_as_capital_dollars: input.gain_on_disposition_dollars,
        };
    }

    if input.gain_on_disposition_dollars == 0 {
        return Output {
            mode: Section1287Mode::CompliantNoDispositionGainOrLoss,
            statutory_basis: "IRC § 1287(a) — no gain on disposition to recharacterize".to_string(),
            notes: "COMPLIANT: no gain on disposition; § 1287(a) recharacterization rule moot.".to_string(),
            citations,
            gain_recharacterized_as_ordinary_income_dollars: 0,
            gain_preserved_as_capital_dollars: 0,
        };
    }

    Output {
        mode: Section1287Mode::ViolationSection1287AConvertsGainToOrdinaryIncome,
        statutory_basis: "IRC § 1287(a) — gain on disposition of registration-required obligation not in registered form converted to ordinary income".to_string(),
        notes: format!(
            "VIOLATION: § 1287(a) converts ${} of capital gain to ordinary income on disposition of registration-required obligation not in registered form; issuer did not pay § 4701 tax; holder does not qualify for Treas. Reg. § 1.165-12(c) exception.",
            input.gain_on_disposition_dollars
        ),
        citations,
        gain_recharacterized_as_ordinary_income_dollars: input.gain_on_disposition_dollars,
        gain_preserved_as_capital_dollars: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_violation_unregistered_obligation() -> Input {
        Input {
            obligation_classification:
                ObligationClassification::RegistrationRequiredObligationUnderSection163F2,
            registration_status: RegistrationStatus::NotInRegisteredFormBearerOrUnregisteredBookEntry,
            issuance_date_status: IssuanceDateStatus::IssuedAfterDecember31_1982TefraApplies,
            section_4701_tax_status: Section4701TaxStatus::Section4701TaxNotPaidByIssuer,
            holder_exception_status: HolderExceptionStatus::HolderExceptionNotClaimedOrNotApplicable,
            gain_on_disposition_dollars: 10_000,
        }
    }

    #[test]
    fn not_a_debt_obligation_not_applicable() {
        let input = Input {
            obligation_classification: ObligationClassification::NotADebtObligationAtAll,
            ..baseline_violation_unregistered_obligation()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section1287Mode::NotApplicableNotARegistrationRequiredObligation
        );
    }

    #[test]
    fn short_term_obligation_under_1_year_not_applicable() {
        let input = Input {
            obligation_classification:
                ObligationClassification::ShortTermObligationMaturityAtOrUnder1YearExempt,
            ..baseline_violation_unregistered_obligation()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section1287Mode::NotApplicableShortTermObligationUnder1YearMaturity
        );
        assert_eq!(result.gain_preserved_as_capital_dollars, 10_000);
    }

    #[test]
    fn not_offered_to_public_not_applicable() {
        let input = Input {
            obligation_classification: ObligationClassification::NotOfferedToPublicExempt,
            ..baseline_violation_unregistered_obligation()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section1287Mode::NotApplicableNotARegistrationRequiredObligation
        );
    }

    #[test]
    fn foreign_targeted_exemption_not_applicable() {
        let input = Input {
            obligation_classification:
                ObligationClassification::ForeignTargetedObligationMeetingTreasuryConditionsExempt,
            ..baseline_violation_unregistered_obligation()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section1287Mode::NotApplicableNotARegistrationRequiredObligation
        );
    }

    #[test]
    fn pre_tefra_grandfathered_not_applicable() {
        let input = Input {
            issuance_date_status:
                IssuanceDateStatus::IssuedOnOrBeforeDecember31_1982GrandfatheredPreTefra,
            ..baseline_violation_unregistered_obligation()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section1287Mode::NotApplicablePreTefraGrandfatheredObligation
        );
        assert_eq!(result.gain_preserved_as_capital_dollars, 10_000);
    }

    #[test]
    fn obligation_in_registered_form_not_applicable() {
        let input = Input {
            registration_status: RegistrationStatus::HeldInRegisteredFormUnderSection163F,
            ..baseline_violation_unregistered_obligation()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section1287Mode::NotApplicableObligationHeldInRegisteredForm
        );
        assert_eq!(result.gain_preserved_as_capital_dollars, 10_000);
    }

    #[test]
    fn section_4701_tax_paid_by_issuer_not_applicable() {
        let input = Input {
            section_4701_tax_status:
                Section4701TaxStatus::Section4701TaxPaidByIssuerSection1287AInapplicable,
            ..baseline_violation_unregistered_obligation()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section1287Mode::NotApplicableSection4701IssuerTaxPaid
        );
        assert_eq!(result.gain_preserved_as_capital_dollars, 10_000);
    }

    #[test]
    fn holder_exception_under_treas_reg_165_12_c_not_applicable() {
        let input = Input {
            holder_exception_status: HolderExceptionStatus::HolderQualifiesUnderTreasReg1_165_12C,
            ..baseline_violation_unregistered_obligation()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section1287Mode::NotApplicableHolderExceptionUnderTreasReg1_165_12C
        );
    }

    #[test]
    fn holder_actual_knowledge_violation() {
        let input = Input {
            holder_exception_status:
                HolderExceptionStatus::HolderHadActualKnowledgeOfRegistrationFailureNoException,
            ..baseline_violation_unregistered_obligation()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section1287Mode::ViolationSection1287AClaimedAsAvoidedButHolderHadActualKnowledge
        );
        assert_eq!(result.gain_recharacterized_as_ordinary_income_dollars, 10_000);
    }

    #[test]
    fn baseline_violation_converts_to_ordinary_income() {
        let result = check(&baseline_violation_unregistered_obligation());
        assert_eq!(
            result.mode,
            Section1287Mode::ViolationSection1287AConvertsGainToOrdinaryIncome
        );
        assert_eq!(result.gain_recharacterized_as_ordinary_income_dollars, 10_000);
        assert_eq!(result.gain_preserved_as_capital_dollars, 0);
    }

    #[test]
    fn no_gain_on_disposition_compliant() {
        let input = Input {
            gain_on_disposition_dollars: 0,
            ..baseline_violation_unregistered_obligation()
        };
        let result = check(&input);
        assert_eq!(result.mode, Section1287Mode::CompliantNoDispositionGainOrLoss);
    }

    #[test]
    fn citations_pin_section_1287_subsections_and_tefra() {
        let result = check(&baseline_violation_unregistered_obligation());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("IRC § 1287(a)"));
        assert!(joined.contains("IRC § 1287(b)(1)"));
        assert!(joined.contains("IRC § 1287(b)(2)"));
        assert!(joined.contains("§ 163(f)"));
        assert!(joined.contains("§ 163(f)(2)"));
        assert!(joined.contains("IRC § 4701"));
        assert!(joined.contains("ORDINARY INCOME"));
        assert!(joined.contains("registration-required obligation"));
        assert!(joined.contains("NOT MORE THAN 1 YEAR"));
        assert!(joined.contains("DECEMBER 31, 1982"));
        assert!(joined.contains("TEFRA"));
        assert!(joined.contains("Public Law 97-248"));
        assert!(joined.contains("Treas. Reg. § 1.165-12(c)"));
        assert!(joined.contains("26 CFR § 1.1287-1"));
        assert!(joined.contains("Federal Register 82 FR 43773"));
        assert!(joined.contains("September 19, 2017"));
        assert!(joined.contains("§ 1276 Market Discount"));
        assert!(joined.contains("GRANDFATHERED"));
    }

    #[test]
    fn constant_pin_tefra_dates_and_4701_rate() {
        assert_eq!(IRC_1287_TEFRA_EFFECTIVE_DATE_YEAR, 1982);
        assert_eq!(IRC_1287_TEFRA_EFFECTIVE_DATE_MONTH, 12);
        assert_eq!(IRC_1287_TEFRA_EFFECTIVE_DATE_DAY, 31);
        assert_eq!(IRC_4701_ISSUER_TAX_RATE_BASIS_POINTS, 100);
        assert_eq!(IRC_4701_BASIS_POINT_DENOMINATOR, 10_000);
        assert_eq!(IRC_163_F_2_SHORT_TERM_OBLIGATION_MAX_DAYS, 365);
        assert_eq!(TREAS_REG_REGISTERED_FORM_2017_UPDATE_YEAR, 2017);
        assert_eq!(TREAS_REG_REGISTERED_FORM_2017_UPDATE_MONTH, 9);
        assert_eq!(TREAS_REG_REGISTERED_FORM_2017_UPDATE_DAY, 19);
    }
}
