//! IRC § 1288 — Treatment of Original Issue Discount on
//! Tax-Exempt Obligations / Muni-Bond OID Module.
//!
//! Pure-compute check for IRC § 1288 treatment of OID on
//! tax-exempt obligations — the "muni-bond" rule that
//! ensures OID on tax-exempt bonds (municipal bonds + other
//! § 103-exempt instruments) accrues into the holder's
//! adjusted basis WITHOUT being included in gross income.
//! § 1288 is the closing companion to the OID statutory
//! cluster (§§ 1271-1275 + § 1286 + § 1287) and the
//! foundational anti-abuse rule preventing muni-bond holders
//! from claiming basis step-up without accruing OID through
//! adjusted basis.
//!
//! Web research (verified 2026-06-03):
//! - **IRC § 1288(a) General Rule for Tax-Exempt Obligation OID**: original issue discount on any tax-exempt obligation shall be treated as accruing in the manner provided by § 1272(a) FOR PURPOSES OF DETERMINING THE ADJUSTED BASIS of the holder, with the adjustments described in § 1272(a)(7). The same accrual method applies for interest-deduction purposes BUT without the § 1272(a)(7) adjustments. The effect is that OID accrues into the adjusted basis of a tax-exempt bond holder EVEN THOUGH the interest on tax-exempt obligations remains EXCLUDED from gross income under § 103 ([Cornell LII 26 USC § 1288](https://www.law.cornell.edu/uscode/text/26/1288); [Bloomberg Tax Sec. 1288](https://irc.bloombergtax.com/public/uscode/doc/irc/section_1288); [Tax Notes IRC 1288](https://www.taxnotes.com/research/federal/usc26/1288); [CCH AnswerConnect § 1288 Treatment of OID on Tax-Exempt Obligations](https://answerconnect.cch.com/document/arp1209013e2c83dc661e/federal/irc/current/treatment-of-original-issue-discount-on-tax-exempt-obligations); [IRS Publication 1212 — Guide to Original Issue Discount (OID) Instruments (12/2025)](https://www.irs.gov/publications/p1212)).
//! - **IRC § 1288(b)(1) OID Definition Without De Minimis Rule**: OID on a tax-exempt obligation is determined under § 1273(a) WITHOUT REGARD TO PARAGRAPH (3) — the § 1273(a)(3) DE MINIMIS OID RULE (0.25 % × SRPM × number of complete years to maturity) DOES NOT APPLY to tax-exempt obligations. Every dollar of OID on a tax-exempt bond accrues into adjusted basis, no matter how small. This is the most-litigated divergence between taxable-bond and tax-exempt-bond OID treatment.
//! - **IRC § 1288(b)(2) Adjusted Federal Rate Adjustments**: in applying § 483 and § 1274 to tax-exempt obligations, the Secretary shall prescribe regulations making appropriate adjustments to the **APPLICABLE FEDERAL RATE** to take into account the tax-exemption benefit of the interest on the obligation. The resulting **TAX-EXEMPT AFR** is typically lower than the standard taxable AFR by a factor reflecting the after-tax-equivalent rate to a high-bracket taxpayer.
//! - **IRC § 1288(b)(3) Tax-Exempt Obligation Definition**: tax-exempt obligation has the meaning given by § 1275(a)(3), which cross-references § 103 (interest on certain state and local bonds exempt from gross income).
//! - **IRC § 1288(b)(4) Short-Term Obligations**: in the case of obligations maturing within ONE YEAR or less, rules similar to the rules of **§ 1283(b)** apply — short-term tax-exempt obligations follow the short-term-obligation accrual framework rather than the long-term OID framework.
//! - **Effect: OID Excluded from Gross Income But Included in Basis**: the central mechanic of § 1288 is to RECONCILE two competing tax principles: (1) interest on tax-exempt obligations is excluded from gross income under § 103; AND (2) OID accruals must adjust the holder's basis so that disposition gain/loss is properly computed. § 1288 achieves this reconciliation by treating OID accruals as basis adjustments only (not gross-income inclusions). Without § 1288, a holder of a tax-exempt bond could claim basis step-up without accruing OID through adjusted basis, generating artificial losses on disposition.
//! - **Effective Date**: § 1288 was added to the IRC by **Public Law 98-369 § 41(c) (Deficit Reduction Act of 1984)** on **July 18, 1984**. Applies to taxable years ending after July 18, 1984, but only with respect to obligations ISSUED AFTER **SEPTEMBER 3, 1982** AND ACQUIRED AFTER **MARCH 1, 1984**. The Tax Reform Act of 1986 confirmed and clarified the § 1288 effective date through subsequent amendments.
//! - **Companion Provisions**: § 1271 (retirement of debt instrument); § 1272 (current OID inclusion); § 1273 (general OID determination — § 1288(b)(1) references § 1273(a) WITHOUT (a)(3)); § 1274 (issue price for debt-for-property exchanges — built iter 678; § 1288(b)(2) AFR adjustment cross-reference); § 1275 (other definitions — built iter 680; § 1288(b)(3) tax-exempt-obligation definition cross-reference); § 1276 (market discount accrual); § 1277 / § 1278 (market discount deferred deductions); § 1281 (current inclusion on short-term obligations); § 1282 (deferred deductions on short-term obligation holding costs); § 1283 (short-term obligation definitions — § 1288(b)(4) cross-reference); § 1286 (built iter 672 — stripped bonds; § 1286(c)/(d) tax-exempt-stripped-obligation rules added by Tax Reform Act of 1986); § 1287 (anti-bearer-bond rule); § 103 (interest on certain state and local bonds — tax exemption foundation); § 6049 (information reporting on Form 1099-OID even for tax-exempt OID).
//! - **Trader / Municipal-Bond-Desk Significance**: § 1288 is the operational anchor for every municipal bond trader who holds an OID muni — the trader cannot ignore OID accruals (despite the gross-income exclusion under § 103) because failure to accrue OID into basis generates artificial disposition losses that the IRS will challenge. The no-de-minimis rule under § 1288(b)(1) traps practitioners who default to the standard § 1273(a)(3) de minimis check on taxable bonds. The tax-exempt AFR adjustment under § 1288(b)(2) creates the most-litigated valuation issue for muni-bond seller-financed transactions and below-market loan analyses.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const IRC_1288_ENACTMENT_YEAR: u32 = 1984;
pub const IRC_1288_ENACTMENT_MONTH: u32 = 7;
pub const IRC_1288_ENACTMENT_DAY: u32 = 18;
pub const IRC_1288_OBLIGATION_ISSUANCE_CUTOFF_YEAR: u32 = 1982;
pub const IRC_1288_OBLIGATION_ISSUANCE_CUTOFF_MONTH: u32 = 9;
pub const IRC_1288_OBLIGATION_ISSUANCE_CUTOFF_DAY: u32 = 3;
pub const IRC_1288_OBLIGATION_ACQUISITION_CUTOFF_YEAR: u32 = 1984;
pub const IRC_1288_OBLIGATION_ACQUISITION_CUTOFF_MONTH: u32 = 3;
pub const IRC_1288_OBLIGATION_ACQUISITION_CUTOFF_DAY: u32 = 1;
pub const IRC_1288_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ObligationClassification {
    TaxExemptObligationUnderSection1275A3CrossReferencingSection103,
    TaxableObligationOutsideSection1288Scope,
    ShortTermTaxExemptObligationAtOrUnderOneYearUsesSection1283bRules,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ObligationIssuanceDateStatus {
    IssuedAfterSeptember3_1982CoveredBySection1288,
    IssuedOnOrBeforeSeptember3_1982PreEffectiveDateGrandfathered,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ObligationAcquisitionDateStatus {
    AcquiredAfterMarch1_1984CoveredBySection1288,
    AcquiredOnOrBeforeMarch1_1984PreEffectiveDateGrandfathered,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAspect {
    OidAccrualIntoAdjustedBasisExcludedFromGrossIncome,
    AdjustedFederalRateAdjustmentForTaxExemptionBenefit,
    NoDeMinimisOidUnderSection1288B1,
    ShortTermObligationUsesSection1283bRules,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DeMinimisRuleApplicationStatus {
    DeMinimisRuleProperlyExcludedFromTaxExemptObligationAccrual,
    DeMinimisRuleImproperlyAppliedToTaxExemptObligation,
    NotApplicableNonTaxExemptOrShortTerm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GrossIncomeAndBasisStatus {
    OidExcludedFromGrossIncomeAndAccruedIntoAdjustedBasisCompliant,
    OidNotAccruedIntoAdjustedBasisViolation,
    OidImproperlyIncludedInGrossIncomeViolation,
    NotApplicable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section1288Mode {
    NotApplicableNotATaxExemptObligation,
    NotApplicableShortTermTaxExemptObligationUsesSection1283bRules,
    NotApplicableObligationIssuedOnOrBeforeSeptember3_1982PreEffectiveDate,
    NotApplicableObligationAcquiredOnOrBeforeMarch1_1984PreEffectiveDate,
    CompliantTaxExemptOidAccruedForBasisAdjustmentExcludedFromGrossIncome,
    CompliantAdjustedFederalRateAdjustedForTaxExemptionBenefit,
    CompliantNoDeMinimisOidUnderSection1288B1,
    CompliantShortTermObligationUsesSection1283bRules,
    ViolationDeMinimisOidRuleImproperlyAppliedToTaxExemptObligation,
    ViolationOidNotAccruedForBasisAdjustmentPurposes,
    ViolationOidImproperlyIncludedInGrossIncome,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub obligation_classification: ObligationClassification,
    pub obligation_issuance_date_status: ObligationIssuanceDateStatus,
    pub obligation_acquisition_date_status: ObligationAcquisitionDateStatus,
    pub compliance_aspect: ComplianceAspect,
    pub de_minimis_rule_application_status: DeMinimisRuleApplicationStatus,
    pub gross_income_and_basis_status: GrossIncomeAndBasisStatus,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section1288Mode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
    pub de_minimis_rule_applies_to_obligation: bool,
}

pub type Section1288Input = Input;
pub type Section1288Output = Output;
pub type Section1288Result = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "IRC § 1288(a) General Rule — OID on tax-exempt obligations treated as accruing in manner provided by § 1272(a) FOR PURPOSES OF DETERMINING ADJUSTED BASIS of holder (with § 1272(a)(7) adjustments); same accrual method applies for interest-deduction purposes BUT without § 1272(a)(7) adjustments; OID accrues into adjusted basis EVEN THOUGH interest on tax-exempt obligations remains EXCLUDED from gross income under § 103".to_string(),
        "IRC § 1288(b)(1) OID Definition Without De Minimis Rule — OID on tax-exempt obligation determined under § 1273(a) WITHOUT REGARD TO PARAGRAPH (3); § 1273(a)(3) DE MINIMIS OID RULE (0.25 % × SRPM × number of complete years to maturity) DOES NOT APPLY to tax-exempt obligations; every dollar of OID accrues into adjusted basis no matter how small".to_string(),
        "IRC § 1288(b)(2) Adjusted Federal Rate Adjustments — in applying § 483 and § 1274 to tax-exempt obligations, Secretary prescribes regulations making appropriate adjustments to APPLICABLE FEDERAL RATE to take into account tax-exemption benefit; resulting TAX-EXEMPT AFR typically lower than standard taxable AFR by factor reflecting after-tax-equivalent rate to high-bracket taxpayer".to_string(),
        "IRC § 1288(b)(3) Tax-Exempt Obligation Definition — meaning given by § 1275(a)(3) which cross-references § 103 (interest on certain state and local bonds exempt from gross income)".to_string(),
        "IRC § 1288(b)(4) Short-Term Obligations — obligations maturing within ONE YEAR or less subject to rules similar to § 1283(b); short-term tax-exempt obligations follow short-term-obligation accrual framework rather than long-term OID framework".to_string(),
        "Effect: OID Excluded from Gross Income But Included in Basis — § 1288 RECONCILES two competing tax principles: (1) interest on tax-exempt obligations excluded from gross income under § 103; AND (2) OID accruals must adjust holder's basis so disposition gain/loss properly computed; § 1288 treats OID accruals as basis adjustments only (not gross-income inclusions); without § 1288, tax-exempt bond holders could claim basis step-up without accruing OID through adjusted basis, generating artificial losses on disposition".to_string(),
        "Effective Date — § 1288 added to IRC by Public Law 98-369 § 41(c) (Deficit Reduction Act of 1984) on July 18, 1984; applies to taxable years ending after July 18, 1984, but only with respect to obligations ISSUED AFTER September 3, 1982 AND ACQUIRED AFTER March 1, 1984; Tax Reform Act of 1986 confirmed and clarified § 1288 effective date through subsequent amendments".to_string(),
        "Companion Provisions — § 1271 (retirement of debt instrument); § 1272 (current OID inclusion); § 1273 (general OID determination — § 1288(b)(1) references § 1273(a) WITHOUT (a)(3)); § 1274 (built iter 678 — issue price for debt-for-property; § 1288(b)(2) AFR adjustment cross-reference); § 1275 (built iter 680 — other definitions; § 1288(b)(3) tax-exempt-obligation definition cross-reference); § 1276 (market discount accrual); § 1277 / § 1278 (market discount deferred deductions); § 1281 (current inclusion on short-term obligations); § 1282 (deferred deductions on short-term obligation holding costs); § 1283 (short-term obligation definitions — § 1288(b)(4) cross-reference); § 1286 (built iter 672 — stripped bonds; § 1286(c)/(d) tax-exempt-stripped-obligation rules); § 1287 (anti-bearer-bond rule); § 103 (interest on certain state and local bonds — tax exemption foundation); § 6049 (information reporting on Form 1099-OID even for tax-exempt OID)".to_string(),
        "Cornell LII 26 USC § 1288 — primary statutory text".to_string(),
        "Bloomberg Tax Sec. 1288 — comprehensive code commentary".to_string(),
        "Tax Notes IRC 1288 — Internal Revenue Code Section 1288 reference".to_string(),
        "CCH AnswerConnect § 1288 — Treatment of OID on Tax-Exempt Obligations practitioner guide".to_string(),
        "IRS Publication 1212 (12/2025) — Guide to Original Issue Discount (OID) Instruments".to_string(),
    ];

    if input.obligation_classification
        == ObligationClassification::TaxableObligationOutsideSection1288Scope
    {
        return Output {
            mode: Section1288Mode::NotApplicableNotATaxExemptObligation,
            statutory_basis: "IRC § 1288(b)(3) — applies only to tax-exempt obligations as defined in § 1275(a)(3) (cross-references § 103)".to_string(),
            notes: "NOT APPLICABLE: obligation is not a tax-exempt obligation under § 1275(a)(3) (does not qualify for § 103 interest exclusion); § 1288 does not apply; standard taxable OID rules under §§ 1271-1287 govern.".to_string(),
            citations,
            de_minimis_rule_applies_to_obligation: true,
        };
    }

    if input.obligation_classification
        == ObligationClassification::ShortTermTaxExemptObligationAtOrUnderOneYearUsesSection1283bRules
    {
        return Output {
            mode: Section1288Mode::NotApplicableShortTermTaxExemptObligationUsesSection1283bRules,
            statutory_basis: "IRC § 1288(b)(4) — short-term tax-exempt obligations subject to rules similar to § 1283(b)".to_string(),
            notes: "NOT APPLICABLE TO LONG-TERM OID RULES: short-term tax-exempt obligation (maturity at or under 1 year); § 1288(b)(4) directs application of rules similar to § 1283(b); long-term OID framework does not apply.".to_string(),
            citations,
            de_minimis_rule_applies_to_obligation: false,
        };
    }

    if input.obligation_issuance_date_status
        == ObligationIssuanceDateStatus::IssuedOnOrBeforeSeptember3_1982PreEffectiveDateGrandfathered
    {
        return Output {
            mode: Section1288Mode::NotApplicableObligationIssuedOnOrBeforeSeptember3_1982PreEffectiveDate,
            statutory_basis: "Public Law 98-369 § 41(c) effective date — § 1288 applies only to obligations issued after September 3, 1982".to_string(),
            notes: "NOT APPLICABLE: obligation issued on or before September 3, 1982; pre-effective-date grandfathered obligation; § 1288 does not apply.".to_string(),
            citations,
            de_minimis_rule_applies_to_obligation: false,
        };
    }

    if input.obligation_acquisition_date_status
        == ObligationAcquisitionDateStatus::AcquiredOnOrBeforeMarch1_1984PreEffectiveDateGrandfathered
    {
        return Output {
            mode: Section1288Mode::NotApplicableObligationAcquiredOnOrBeforeMarch1_1984PreEffectiveDate,
            statutory_basis: "Public Law 98-369 § 41(c) effective date — § 1288 applies only to obligations acquired after March 1, 1984".to_string(),
            notes: "NOT APPLICABLE: obligation acquired on or before March 1, 1984; pre-effective-date grandfathered acquisition; § 1288 does not apply.".to_string(),
            citations,
            de_minimis_rule_applies_to_obligation: false,
        };
    }

    match input.compliance_aspect {
        ComplianceAspect::NoDeMinimisOidUnderSection1288B1 => match input.de_minimis_rule_application_status {
            DeMinimisRuleApplicationStatus::DeMinimisRuleProperlyExcludedFromTaxExemptObligationAccrual => Output {
                mode: Section1288Mode::CompliantNoDeMinimisOidUnderSection1288B1,
                statutory_basis: "IRC § 1288(b)(1) — § 1273(a)(3) de minimis OID rule DOES NOT APPLY to tax-exempt obligations".to_string(),
                notes: "COMPLIANT: de minimis OID rule under § 1273(a)(3) properly EXCLUDED from accrual computation on tax-exempt obligation; every dollar of OID accrues into adjusted basis no matter how small.".to_string(),
                citations,
                de_minimis_rule_applies_to_obligation: false,
            },
            DeMinimisRuleApplicationStatus::DeMinimisRuleImproperlyAppliedToTaxExemptObligation => Output {
                mode: Section1288Mode::ViolationDeMinimisOidRuleImproperlyAppliedToTaxExemptObligation,
                statutory_basis: "IRC § 1288(b)(1) — § 1273(a)(3) de minimis rule may NOT be applied to tax-exempt obligations".to_string(),
                notes: "VIOLATION: § 1273(a)(3) de minimis OID rule (0.25 % × SRPM × years to maturity) improperly applied to tax-exempt obligation; § 1288(b)(1) requires OID determination WITHOUT regard to § 1273(a)(3); de minimis exclusion is unavailable for tax-exempt obligations; recompute accrual without de minimis threshold.".to_string(),
                citations,
                de_minimis_rule_applies_to_obligation: false,
            },
            DeMinimisRuleApplicationStatus::NotApplicableNonTaxExemptOrShortTerm => Output {
                mode: Section1288Mode::CompliantNoDeMinimisOidUnderSection1288B1,
                statutory_basis: "IRC § 1288(b)(1) — § 1273(a)(3) de minimis OID rule DOES NOT APPLY to tax-exempt obligations".to_string(),
                notes: "COMPLIANT: de minimis rule status not applicable to this obligation type; § 1288(b)(1) exclusion of § 1273(a)(3) de minimis rule from tax-exempt obligations confirmed.".to_string(),
                citations,
                de_minimis_rule_applies_to_obligation: false,
            },
        },
        ComplianceAspect::OidAccrualIntoAdjustedBasisExcludedFromGrossIncome => match input.gross_income_and_basis_status {
            GrossIncomeAndBasisStatus::OidExcludedFromGrossIncomeAndAccruedIntoAdjustedBasisCompliant => Output {
                mode: Section1288Mode::CompliantTaxExemptOidAccruedForBasisAdjustmentExcludedFromGrossIncome,
                statutory_basis: "IRC § 1288(a) + § 103 — OID accrues for basis adjustment under § 1272(a) with § 1272(a)(7) adjustments while interest remains excluded from gross income under § 103".to_string(),
                notes: "COMPLIANT: OID on tax-exempt obligation properly EXCLUDED from gross income (consistent with § 103 interest exclusion) AND ACCRUED into adjusted basis (consistent with § 1288(a) and § 1272(a) accrual mechanics with § 1272(a)(7) adjustments); disposition gain/loss will be properly computed using adjusted basis.".to_string(),
                citations,
                de_minimis_rule_applies_to_obligation: false,
            },
            GrossIncomeAndBasisStatus::OidNotAccruedIntoAdjustedBasisViolation => Output {
                mode: Section1288Mode::ViolationOidNotAccruedForBasisAdjustmentPurposes,
                statutory_basis: "IRC § 1288(a) — OID must accrue into adjusted basis even though excluded from gross income".to_string(),
                notes: "VIOLATION: OID on tax-exempt obligation NOT properly accrued into adjusted basis; § 1288(a) requires accrual under § 1272(a) with § 1272(a)(7) adjustments for basis-determination purposes regardless of § 103 gross-income exclusion; failure produces artificial disposition loss on sale of bond.".to_string(),
                citations,
                de_minimis_rule_applies_to_obligation: false,
            },
            GrossIncomeAndBasisStatus::OidImproperlyIncludedInGrossIncomeViolation => Output {
                mode: Section1288Mode::ViolationOidImproperlyIncludedInGrossIncome,
                statutory_basis: "§ 103 + IRC § 1288(a) — OID on tax-exempt obligation excluded from gross income; only basis adjustment effected".to_string(),
                notes: "VIOLATION: OID on tax-exempt obligation improperly INCLUDED in gross income; § 103 excludes interest on tax-exempt obligations from gross income; § 1288(a) effects basis adjustment only; recompute return excluding the OID from gross income.".to_string(),
                citations,
                de_minimis_rule_applies_to_obligation: false,
            },
            GrossIncomeAndBasisStatus::NotApplicable => Output {
                mode: Section1288Mode::CompliantTaxExemptOidAccruedForBasisAdjustmentExcludedFromGrossIncome,
                statutory_basis: "IRC § 1288(a) — basis adjustment mechanic for tax-exempt OID".to_string(),
                notes: "COMPLIANT: gross-income/basis status not applicable for this aspect; § 1288(a) basis adjustment mechanic in effect.".to_string(),
                citations,
                de_minimis_rule_applies_to_obligation: false,
            },
        },
        ComplianceAspect::AdjustedFederalRateAdjustmentForTaxExemptionBenefit => Output {
            mode: Section1288Mode::CompliantAdjustedFederalRateAdjustedForTaxExemptionBenefit,
            statutory_basis: "IRC § 1288(b)(2) — applicable Federal rate under §§ 483 and 1274 adjusted for tax-exemption benefit".to_string(),
            notes: "COMPLIANT: applicable Federal rate under § 483 (unstated interest) and § 1274 (issue price for debt-for-property) properly adjusted to take into account the tax-exemption benefit of interest on the obligation; tax-exempt AFR computed per Secretary's regulations is typically lower than standard taxable AFR by a factor reflecting after-tax-equivalent rate to a high-bracket taxpayer.".to_string(),
            citations,
            de_minimis_rule_applies_to_obligation: false,
        },
        ComplianceAspect::ShortTermObligationUsesSection1283bRules => Output {
            mode: Section1288Mode::CompliantShortTermObligationUsesSection1283bRules,
            statutory_basis: "IRC § 1288(b)(4) — short-term tax-exempt obligations follow rules similar to § 1283(b)".to_string(),
            notes: "COMPLIANT: short-term tax-exempt obligation (maturity at or under 1 year) properly subject to rules similar to § 1283(b); long-term OID framework not applied.".to_string(),
            citations,
            de_minimis_rule_applies_to_obligation: false,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_input() -> Input {
        Input {
            obligation_classification:
                ObligationClassification::TaxExemptObligationUnderSection1275A3CrossReferencingSection103,
            obligation_issuance_date_status:
                ObligationIssuanceDateStatus::IssuedAfterSeptember3_1982CoveredBySection1288,
            obligation_acquisition_date_status:
                ObligationAcquisitionDateStatus::AcquiredAfterMarch1_1984CoveredBySection1288,
            compliance_aspect: ComplianceAspect::OidAccrualIntoAdjustedBasisExcludedFromGrossIncome,
            de_minimis_rule_application_status:
                DeMinimisRuleApplicationStatus::NotApplicableNonTaxExemptOrShortTerm,
            gross_income_and_basis_status:
                GrossIncomeAndBasisStatus::OidExcludedFromGrossIncomeAndAccruedIntoAdjustedBasisCompliant,
        }
    }

    #[test]
    fn taxable_obligation_outside_section_1288_scope_not_applicable() {
        let mut input = baseline_input();
        input.obligation_classification =
            ObligationClassification::TaxableObligationOutsideSection1288Scope;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1288Mode::NotApplicableNotATaxExemptObligation
        );
        assert!(output.de_minimis_rule_applies_to_obligation);
    }

    #[test]
    fn short_term_tax_exempt_under_1_year_uses_section_1283b() {
        let mut input = baseline_input();
        input.obligation_classification =
            ObligationClassification::ShortTermTaxExemptObligationAtOrUnderOneYearUsesSection1283bRules;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1288Mode::NotApplicableShortTermTaxExemptObligationUsesSection1283bRules
        );
    }

    #[test]
    fn obligation_issued_on_or_before_september_3_1982_grandfathered() {
        let mut input = baseline_input();
        input.obligation_issuance_date_status =
            ObligationIssuanceDateStatus::IssuedOnOrBeforeSeptember3_1982PreEffectiveDateGrandfathered;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1288Mode::NotApplicableObligationIssuedOnOrBeforeSeptember3_1982PreEffectiveDate
        );
    }

    #[test]
    fn obligation_acquired_on_or_before_march_1_1984_grandfathered() {
        let mut input = baseline_input();
        input.obligation_acquisition_date_status =
            ObligationAcquisitionDateStatus::AcquiredOnOrBeforeMarch1_1984PreEffectiveDateGrandfathered;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1288Mode::NotApplicableObligationAcquiredOnOrBeforeMarch1_1984PreEffectiveDate
        );
    }

    #[test]
    fn tax_exempt_oid_accrued_into_basis_excluded_from_gross_income_compliant() {
        let output = check(&baseline_input());
        assert_eq!(
            output.mode,
            Section1288Mode::CompliantTaxExemptOidAccruedForBasisAdjustmentExcludedFromGrossIncome
        );
        assert!(!output.de_minimis_rule_applies_to_obligation);
    }

    #[test]
    fn oid_not_accrued_into_basis_violation() {
        let mut input = baseline_input();
        input.gross_income_and_basis_status =
            GrossIncomeAndBasisStatus::OidNotAccruedIntoAdjustedBasisViolation;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1288Mode::ViolationOidNotAccruedForBasisAdjustmentPurposes
        );
    }

    #[test]
    fn oid_improperly_included_in_gross_income_violation() {
        let mut input = baseline_input();
        input.gross_income_and_basis_status =
            GrossIncomeAndBasisStatus::OidImproperlyIncludedInGrossIncomeViolation;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1288Mode::ViolationOidImproperlyIncludedInGrossIncome
        );
    }

    #[test]
    fn no_de_minimis_rule_under_section_1288_b_1_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::NoDeMinimisOidUnderSection1288B1;
        input.de_minimis_rule_application_status =
            DeMinimisRuleApplicationStatus::DeMinimisRuleProperlyExcludedFromTaxExemptObligationAccrual;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1288Mode::CompliantNoDeMinimisOidUnderSection1288B1
        );
    }

    #[test]
    fn de_minimis_rule_improperly_applied_to_tax_exempt_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::NoDeMinimisOidUnderSection1288B1;
        input.de_minimis_rule_application_status =
            DeMinimisRuleApplicationStatus::DeMinimisRuleImproperlyAppliedToTaxExemptObligation;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1288Mode::ViolationDeMinimisOidRuleImproperlyAppliedToTaxExemptObligation
        );
    }

    #[test]
    fn adjusted_federal_rate_adjustment_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::AdjustedFederalRateAdjustmentForTaxExemptionBenefit;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1288Mode::CompliantAdjustedFederalRateAdjustedForTaxExemptionBenefit
        );
    }

    #[test]
    fn short_term_obligation_uses_section_1283b_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ShortTermObligationUsesSection1283bRules;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1288Mode::CompliantShortTermObligationUsesSection1283bRules
        );
    }

    #[test]
    fn constants_pin_statutory_facts() {
        assert_eq!(IRC_1288_ENACTMENT_YEAR, 1984);
        assert_eq!(IRC_1288_ENACTMENT_MONTH, 7);
        assert_eq!(IRC_1288_ENACTMENT_DAY, 18);
        assert_eq!(IRC_1288_OBLIGATION_ISSUANCE_CUTOFF_YEAR, 1982);
        assert_eq!(IRC_1288_OBLIGATION_ISSUANCE_CUTOFF_MONTH, 9);
        assert_eq!(IRC_1288_OBLIGATION_ISSUANCE_CUTOFF_DAY, 3);
        assert_eq!(IRC_1288_OBLIGATION_ACQUISITION_CUTOFF_YEAR, 1984);
        assert_eq!(IRC_1288_OBLIGATION_ACQUISITION_CUTOFF_MONTH, 3);
        assert_eq!(IRC_1288_OBLIGATION_ACQUISITION_CUTOFF_DAY, 1);
        assert_eq!(IRC_1288_BASIS_POINT_DENOMINATOR, 10_000);
    }

    #[test]
    fn citation_contains_landmarks() {
        let output = check(&baseline_input());
        let joined = output.citations.join("\n");
        assert!(joined.contains("§ 1288(a)"));
        assert!(joined.contains("§ 1288(b)(1)"));
        assert!(joined.contains("§ 1288(b)(2)"));
        assert!(joined.contains("§ 1288(b)(3)"));
        assert!(joined.contains("§ 1288(b)(4)"));
        assert!(joined.contains("§ 1273(a)(3)"));
        assert!(joined.contains("§ 1272(a)(7)"));
        assert!(joined.contains("§ 1275(a)(3)"));
        assert!(joined.contains("§ 1283(b)"));
        assert!(joined.contains("§ 103"));
        assert!(joined.contains("Public Law 98-369"));
        assert!(joined.contains("July 18, 1984"));
        assert!(joined.contains("September 3, 1982"));
        assert!(joined.contains("March 1, 1984"));
        assert!(joined.contains("DE MINIMIS OID RULE"));
    }
}
