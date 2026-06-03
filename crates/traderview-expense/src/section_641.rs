//! IRC § 641 — Imposition of Tax on Trusts and Estates (Fiduciary
//! Income Taxation).
//!
//! Pure-compute fiduciary income tax computation under § 641 with
//! the compressed four-bracket schedule for trusts and estates.
//! Trader-critical because **2025 trust top marginal bracket = 37 %
//! at approximately $15,650** of taxable income (40× more
//! compressed than individual single filer at $626,350 for 2025).
//!
//! Statute (verbatim mapping):
//! - § 641(a) — IMPOSITION OF TAX: a tax shall be imposed on the
//!   taxable income of estates, trusts, and bankruptcy estates,
//!   computed and paid by the fiduciary on Form 1041.
//! - § 641(b) — COMPUTATION: taxable income of estate or trust
//!   computed in same manner as for an INDIVIDUAL except as
//!   otherwise provided — subject to § 642 special rules, § 643
//!   DNI ceiling, and § 651 (simple trust) / § 661 (complex
//!   trust) distribution deductions.
//! - § 641(c) — ELECTING SMALL BUSINESS TRUST (ESBT): S corporation
//!   portion of an ESBT shall be treated as a separate trust and
//!   taxed at the highest § 1(e) rate applicable to estates and
//!   trusts (37 % currently); no § 651 / § 661 distribution
//!   deduction permitted on the S corp portion.
//! - § 641(d) — ESBT INTEREST ALLOCABLE BETWEEN S-CORP AND NON-
//!   S-CORP PORTIONS: § 641(c) governs S-corp portion; non-S-corp
//!   portion taxed under normal trust rules.
//! - § 1(e) — RATE SCHEDULE FOR ESTATES AND TRUSTS (compressed
//!   four-bracket structure post-TCJA, made permanent by P.L. 119-
//!   21 OBBBA 2025):
//!   - 10 % on income up to first bracket ceiling
//!   - 24 % on income above first ceiling up to second ceiling
//!   - 35 % on income above second ceiling up to top bracket
//!   - 37 % on income above top bracket ceiling
//! - **2025 brackets** (IRS Rev. Proc. 2024-40):
//!   - 10 %: $0 – $3,150
//!   - 24 %: $3,150 – $11,450
//!   - 35 %: $11,450 – $15,650
//!   - 37 %: above $15,650
//! - **2026 brackets** (estimated IRS inflation adjustment) — top
//!   bracket starts at approximately **$16,000**.
//!
//! Cross-references:
//! - § 642(b) — personal exemption ($600 estate / $300 complex
//!   trust / $100 simple trust, NOT indexed for inflation).
//! - § 642(c) — charitable contribution deduction.
//! - § 643(a) — DNI definition (controls distribution deduction
//!   ceiling).
//! - § 651 — simple trust distribution deduction (mandatory
//!   distributions only, no corpus distributions, no charitable
//!   distributions).
//! - § 661 — complex trust distribution deduction (mandatory +
//!   discretionary; corpus distributions; tier-1 / tier-2 priority).
//! - § 1411 — NIIT 3.8 % surtax applies to trust at top bracket
//!   threshold (same compressed level — undistributed net
//!   investment income above $15,650 for 2025).
//! - § 67(g) — TCJA misc itemized deduction suspension at
//!   compressed trust brackets makes investment-expense
//!   deductibility critical.
//! - § 1(j) — TCJA 2017 rate structure made permanent by OBBBA
//!   2025.
//!
//! Web research (verified 2026-06-03):
//! - IRS Form 1041-ES 2026 confirms compressed four-bracket
//!   structure.
//! - IRS Rev. Proc. 2024-40 confirms 2025 trust bracket
//!   thresholds.
//! - SmartAsset Trust Tax Rates 2026 confirms 37 % at ~$16,000 for
//!   2026.
//! - Tax Foundation 2026 Tax Brackets confirms individual single
//!   37 % at $640,600 (≈ 40× compression vs trust).

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const TRUST_2025_BRACKET_10_CEILING_DOLLARS: u64 = 3_150;
pub const TRUST_2025_BRACKET_24_CEILING_DOLLARS: u64 = 11_450;
pub const TRUST_2025_BRACKET_35_CEILING_DOLLARS: u64 = 15_650;
pub const TRUST_2026_TOP_BRACKET_THRESHOLD_DOLLARS_APPROX: u64 = 16_000;
pub const TRUST_BRACKET_10_RATE_BASIS_POINTS: u64 = 1_000;
pub const TRUST_BRACKET_24_RATE_BASIS_POINTS: u64 = 2_400;
pub const TRUST_BRACKET_35_RATE_BASIS_POINTS: u64 = 3_500;
pub const TRUST_BRACKET_37_RATE_BASIS_POINTS: u64 = 3_700;
pub const SECTION_641_BASIS_POINT_DENOMINATOR: u64 = 10_000;
pub const SECTION_642_B_ESTATE_EXEMPTION_DOLLARS: u64 = 600;
pub const SECTION_642_B_COMPLEX_TRUST_EXEMPTION_DOLLARS: u64 = 300;
pub const SECTION_642_B_SIMPLE_TRUST_EXEMPTION_DOLLARS: u64 = 100;
pub const SECTION_1411_NIIT_RATE_BASIS_POINTS: u64 = 380;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FiduciaryEntityType {
    DomesticGrantorTrustPassThrough,
    DomesticSimpleTrust,
    DomesticComplexTrust,
    ElectingSmallBusinessTrust,
    DomesticEstateInProbate,
    ForeignTrust,
    BankruptcyEstate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EsbtPortion {
    NotApplicable,
    SCorpPortion,
    NonSCorpPortion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section641Mode {
    NotApplicableGrantorTrustPassThroughNoTaxAtTrustLevel,
    NotApplicableNoTaxableIncome,
    CompliantTaxComputedAtBracket10,
    CompliantTaxComputedAtBracket24,
    CompliantTaxComputedAtBracket35,
    CompliantTaxComputedAtBracket37TopBracket,
    CompliantEsbtSCorpPortionTaxedAtHighestRate,
    CompliantDistributionDeductionAppliedSimpleTrust,
    CompliantDistributionDeductionAppliedComplexTrust,
    CompliantNiitAppliedAtTopBracketThreshold,
    ViolationIndividualBracketsUsedInsteadOfTrustCompressed,
    ViolationEsbtSCorpPortionImproperlyDistributionDeducted,
    ViolationDniCeilingExceededByDistributionDeduction,
    ViolationNiitOmittedAtTopBracket,
    ViolationSection642bExemptionOverclaimed,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub entity_type: FiduciaryEntityType,
    pub esbt_portion: EsbtPortion,
    pub taxable_income_before_exemption_dollars: u64,
    pub section_642b_personal_exemption_claimed_dollars: u64,
    pub distribution_deduction_claimed_dollars: u64,
    pub dni_dollars: u64,
    pub undistributed_net_investment_income_dollars: u64,
    pub used_individual_brackets_instead: bool,
    pub niit_applied_at_top_bracket_threshold: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section641Mode,
    pub taxable_income_after_exemption_dollars: u64,
    pub computed_tax_dollars: u64,
    pub niit_dollars: u64,
    pub total_tax_dollars: u64,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type Section641Input = Input;
pub type Section641Output = Output;
pub type Section641Result = Output;

fn correct_exemption_for(entity: FiduciaryEntityType) -> u64 {
    match entity {
        FiduciaryEntityType::DomesticEstateInProbate => SECTION_642_B_ESTATE_EXEMPTION_DOLLARS,
        FiduciaryEntityType::DomesticComplexTrust | FiduciaryEntityType::ElectingSmallBusinessTrust => {
            SECTION_642_B_COMPLEX_TRUST_EXEMPTION_DOLLARS
        }
        FiduciaryEntityType::DomesticSimpleTrust => SECTION_642_B_SIMPLE_TRUST_EXEMPTION_DOLLARS,
        FiduciaryEntityType::ForeignTrust | FiduciaryEntityType::BankruptcyEstate => 0,
        FiduciaryEntityType::DomesticGrantorTrustPassThrough => 0,
    }
}

fn apply_rate(amount: u64, rate_bp: u64) -> u64 {
    (amount as u128)
        .saturating_mul(rate_bp as u128)
        .checked_div(SECTION_641_BASIS_POINT_DENOMINATOR as u128)
        .unwrap_or(0) as u64
}

fn compute_bracketed_tax(taxable_income: u64) -> u64 {
    let mut tax: u64 = 0;
    let bracket1 = taxable_income.min(TRUST_2025_BRACKET_10_CEILING_DOLLARS);
    tax = tax.saturating_add(apply_rate(bracket1, TRUST_BRACKET_10_RATE_BASIS_POINTS));
    if taxable_income > TRUST_2025_BRACKET_10_CEILING_DOLLARS {
        let bracket2 = taxable_income
            .min(TRUST_2025_BRACKET_24_CEILING_DOLLARS)
            .saturating_sub(TRUST_2025_BRACKET_10_CEILING_DOLLARS);
        tax = tax.saturating_add(apply_rate(bracket2, TRUST_BRACKET_24_RATE_BASIS_POINTS));
    }
    if taxable_income > TRUST_2025_BRACKET_24_CEILING_DOLLARS {
        let bracket3 = taxable_income
            .min(TRUST_2025_BRACKET_35_CEILING_DOLLARS)
            .saturating_sub(TRUST_2025_BRACKET_24_CEILING_DOLLARS);
        tax = tax.saturating_add(apply_rate(bracket3, TRUST_BRACKET_35_RATE_BASIS_POINTS));
    }
    if taxable_income > TRUST_2025_BRACKET_35_CEILING_DOLLARS {
        let bracket4 = taxable_income.saturating_sub(TRUST_2025_BRACKET_35_CEILING_DOLLARS);
        tax = tax.saturating_add(apply_rate(bracket4, TRUST_BRACKET_37_RATE_BASIS_POINTS));
    }
    tax
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "26 U.S.C. § 641(a) — tax imposed on taxable income of estates, trusts, bankruptcy estates; paid by fiduciary on Form 1041".to_string(),
        "26 U.S.C. § 641(b) — taxable income computed as for individual subject to § 642 + § 643 DNI + § 651 / § 661 distribution deductions".to_string(),
        "26 U.S.C. § 641(c) — ESBT: S corp portion taxed at highest § 1(e) rate (37 %) with NO § 651 / § 661 distribution deduction".to_string(),
        "26 U.S.C. § 641(d) — ESBT non-S-corp portion taxed under normal trust rules".to_string(),
        "26 U.S.C. § 1(e) — rate schedule for estates and trusts (compressed four-bracket post-TCJA, made permanent by OBBBA 2025 § 70411)".to_string(),
        "IRS Rev. Proc. 2024-40 — 2025 trust brackets: 10 % $0-$3,150; 24 % $3,150-$11,450; 35 % $11,450-$15,650; 37 % above $15,650".to_string(),
        "26 U.S.C. § 642(b) — personal exemption: $600 estate, $300 complex trust, $100 simple trust (NOT inflation-indexed)".to_string(),
        "26 U.S.C. § 642(c) — charitable contribution deduction (only from gross income; no AGI floor)".to_string(),
        "26 U.S.C. § 643(a) — DNI ceiling controls maximum § 651 / § 661 distribution deduction".to_string(),
        "26 U.S.C. § 651 — simple trust distribution deduction: mandatory income distributions only".to_string(),
        "26 U.S.C. § 661 — complex trust distribution deduction: mandatory + discretionary; corpus permitted; tier-1 / tier-2 priority".to_string(),
        "26 U.S.C. § 1411(a)(2) — NIIT 3.8 % on undistributed net investment income above top trust bracket threshold ($15,650 for 2025)".to_string(),
        "Compressed brackets — 2025 trust top bracket $15,650 vs individual single $626,350 = ≈ 40× compression".to_string(),
        "IRS Form 1041 — U.S. Income Tax Return for Estates and Trusts; Form 1041-ES — estimated payments".to_string(),
    ];

    if input.entity_type == FiduciaryEntityType::DomesticGrantorTrustPassThrough {
        return Output {
            mode: Section641Mode::NotApplicableGrantorTrustPassThroughNoTaxAtTrustLevel,
            taxable_income_after_exemption_dollars: 0,
            computed_tax_dollars: 0,
            niit_dollars: 0,
            total_tax_dollars: 0,
            statutory_basis: "Subpart E grantor trust rules; no tax at trust level".to_string(),
            notes: "Grantor trust under subpart E; all income passed through to grantor; § 641 inapplicable at trust level.".to_string(),
            citations,
        };
    }

    if input.taxable_income_before_exemption_dollars == 0 {
        return Output {
            mode: Section641Mode::NotApplicableNoTaxableIncome,
            taxable_income_after_exemption_dollars: 0,
            computed_tax_dollars: 0,
            niit_dollars: 0,
            total_tax_dollars: 0,
            statutory_basis: "§ 641 inapplicable absent taxable income".to_string(),
            notes: "Trust or estate has no taxable income; § 641 tax = 0.".to_string(),
            citations,
        };
    }

    if input.used_individual_brackets_instead {
        return Output {
            mode: Section641Mode::ViolationIndividualBracketsUsedInsteadOfTrustCompressed,
            taxable_income_after_exemption_dollars: input.taxable_income_before_exemption_dollars,
            computed_tax_dollars: 0,
            niit_dollars: 0,
            total_tax_dollars: 0,
            statutory_basis: "§ 1(e) — trust brackets are COMPRESSED relative to individual § 1(j) brackets".to_string(),
            notes: "VIOLATION § 1(e): fiduciary used individual brackets instead of compressed trust brackets; recompute under § 1(e) (top 37 % at $15,650 for 2025).".to_string(),
            citations,
        };
    }

    let correct_exemption = correct_exemption_for(input.entity_type);
    if input.section_642b_personal_exemption_claimed_dollars > correct_exemption {
        return Output {
            mode: Section641Mode::ViolationSection642bExemptionOverclaimed,
            taxable_income_after_exemption_dollars: input.taxable_income_before_exemption_dollars,
            computed_tax_dollars: 0,
            niit_dollars: 0,
            total_tax_dollars: 0,
            statutory_basis: "§ 642(b) personal exemption capped per entity type".to_string(),
            notes: format!(
                "VIOLATION § 642(b): claimed exemption ${} exceeds statutory cap of ${} for entity type {:?}.",
                input.section_642b_personal_exemption_claimed_dollars, correct_exemption, input.entity_type
            ),
            citations,
        };
    }

    if input.entity_type == FiduciaryEntityType::ElectingSmallBusinessTrust
        && input.esbt_portion == EsbtPortion::SCorpPortion
        && input.distribution_deduction_claimed_dollars > 0
    {
        return Output {
            mode: Section641Mode::ViolationEsbtSCorpPortionImproperlyDistributionDeducted,
            taxable_income_after_exemption_dollars: input.taxable_income_before_exemption_dollars,
            computed_tax_dollars: apply_rate(
                input.taxable_income_before_exemption_dollars,
                TRUST_BRACKET_37_RATE_BASIS_POINTS,
            ),
            niit_dollars: 0,
            total_tax_dollars: 0,
            statutory_basis: "§ 641(c) — ESBT S-corp portion: no § 651 / § 661 distribution deduction".to_string(),
            notes: format!(
                "VIOLATION § 641(c): ESBT S-corp portion claimed ${} distribution deduction; not permitted. S-corp portion taxed at top 37 % rate on entire taxable income.",
                input.distribution_deduction_claimed_dollars
            ),
            citations,
        };
    }

    if input.distribution_deduction_claimed_dollars > input.dni_dollars {
        return Output {
            mode: Section641Mode::ViolationDniCeilingExceededByDistributionDeduction,
            taxable_income_after_exemption_dollars: input.taxable_income_before_exemption_dollars,
            computed_tax_dollars: 0,
            niit_dollars: 0,
            total_tax_dollars: 0,
            statutory_basis: "§ 643(a) DNI ceiling on distribution deduction".to_string(),
            notes: format!(
                "VIOLATION § 643(a): distribution deduction ${} exceeds DNI ceiling ${}.",
                input.distribution_deduction_claimed_dollars, input.dni_dollars
            ),
            citations,
        };
    }

    if input.entity_type == FiduciaryEntityType::ElectingSmallBusinessTrust
        && input.esbt_portion == EsbtPortion::SCorpPortion
    {
        let taxable_after_exemption = input
            .taxable_income_before_exemption_dollars
            .saturating_sub(input.section_642b_personal_exemption_claimed_dollars);
        let tax = apply_rate(taxable_after_exemption, TRUST_BRACKET_37_RATE_BASIS_POINTS);
        return Output {
            mode: Section641Mode::CompliantEsbtSCorpPortionTaxedAtHighestRate,
            taxable_income_after_exemption_dollars: taxable_after_exemption,
            computed_tax_dollars: tax,
            niit_dollars: 0,
            total_tax_dollars: tax,
            statutory_basis: "§ 641(c) — ESBT S-corp portion at top 37 % rate".to_string(),
            notes: format!(
                "COMPLIANT § 641(c): ESBT S-corp portion of ${} taxed at 37 % = ${}.",
                taxable_after_exemption, tax
            ),
            citations,
        };
    }

    let taxable_after_deductions = input
        .taxable_income_before_exemption_dollars
        .saturating_sub(input.section_642b_personal_exemption_claimed_dollars)
        .saturating_sub(input.distribution_deduction_claimed_dollars);

    let tax = compute_bracketed_tax(taxable_after_deductions);

    let niit = if taxable_after_deductions > TRUST_2025_BRACKET_35_CEILING_DOLLARS {
        let niit_base = input
            .undistributed_net_investment_income_dollars
            .min(taxable_after_deductions.saturating_sub(TRUST_2025_BRACKET_35_CEILING_DOLLARS));
        apply_rate(niit_base, SECTION_1411_NIIT_RATE_BASIS_POINTS)
    } else {
        0
    };

    if niit > 0 && !input.niit_applied_at_top_bracket_threshold {
        return Output {
            mode: Section641Mode::ViolationNiitOmittedAtTopBracket,
            taxable_income_after_exemption_dollars: taxable_after_deductions,
            computed_tax_dollars: tax,
            niit_dollars: niit,
            total_tax_dollars: tax,
            statutory_basis: "§ 1411(a)(2) — NIIT 3.8 % on undistributed net investment income above top bracket threshold".to_string(),
            notes: format!(
                "VIOLATION § 1411(a)(2): NIIT of ${} not applied; trust taxable income ${} exceeds top bracket threshold ${} with undistributed NII of ${}.",
                niit,
                taxable_after_deductions,
                TRUST_2025_BRACKET_35_CEILING_DOLLARS,
                input.undistributed_net_investment_income_dollars
            ),
            citations,
        };
    }

    if input.distribution_deduction_claimed_dollars > 0 {
        let mode = match input.entity_type {
            FiduciaryEntityType::DomesticSimpleTrust => {
                Section641Mode::CompliantDistributionDeductionAppliedSimpleTrust
            }
            _ => Section641Mode::CompliantDistributionDeductionAppliedComplexTrust,
        };
        return Output {
            mode,
            taxable_income_after_exemption_dollars: taxable_after_deductions,
            computed_tax_dollars: tax,
            niit_dollars: niit,
            total_tax_dollars: tax.saturating_add(niit),
            statutory_basis: format!(
                "§ 641(b) + § 651 / § 661 distribution deduction ${} applied within DNI ceiling ${}",
                input.distribution_deduction_claimed_dollars, input.dni_dollars
            ),
            notes: format!(
                "COMPLIANT: distribution deduction reduces trust-level taxable income to ${}; tax ${}; NIIT ${}; total ${}.",
                taxable_after_deductions,
                tax,
                niit,
                tax.saturating_add(niit)
            ),
            citations,
        };
    }

    let mode = if taxable_after_deductions > TRUST_2025_BRACKET_35_CEILING_DOLLARS {
        Section641Mode::CompliantTaxComputedAtBracket37TopBracket
    } else if taxable_after_deductions > TRUST_2025_BRACKET_24_CEILING_DOLLARS {
        Section641Mode::CompliantTaxComputedAtBracket35
    } else if taxable_after_deductions > TRUST_2025_BRACKET_10_CEILING_DOLLARS {
        Section641Mode::CompliantTaxComputedAtBracket24
    } else {
        Section641Mode::CompliantTaxComputedAtBracket10
    };

    let final_mode = if niit > 0 { Section641Mode::CompliantNiitAppliedAtTopBracketThreshold } else { mode };

    Output {
        mode: final_mode,
        taxable_income_after_exemption_dollars: taxable_after_deductions,
        computed_tax_dollars: tax,
        niit_dollars: niit,
        total_tax_dollars: tax.saturating_add(niit),
        statutory_basis: format!(
            "§ 641(b) + § 1(e) compressed brackets; income ${}, tax ${}, NIIT ${}, total ${}",
            taxable_after_deductions,
            tax,
            niit,
            tax.saturating_add(niit)
        ),
        notes: format!(
            "COMPLIANT § 641: taxable income after deductions ${}; bracketed tax ${}; NIIT ${}; total tax ${}.",
            taxable_after_deductions,
            tax,
            niit,
            tax.saturating_add(niit)
        ),
        citations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_complex_trust_top_bracket() -> Input {
        Input {
            entity_type: FiduciaryEntityType::DomesticComplexTrust,
            esbt_portion: EsbtPortion::NotApplicable,
            taxable_income_before_exemption_dollars: 100_000,
            section_642b_personal_exemption_claimed_dollars: 300,
            distribution_deduction_claimed_dollars: 0,
            dni_dollars: 100_000,
            undistributed_net_investment_income_dollars: 0,
            used_individual_brackets_instead: false,
            niit_applied_at_top_bracket_threshold: true,
        }
    }

    #[test]
    fn grantor_trust_passthrough_not_applicable() {
        let input = Input {
            entity_type: FiduciaryEntityType::DomesticGrantorTrustPassThrough,
            ..baseline_complex_trust_top_bracket()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section641Mode::NotApplicableGrantorTrustPassThroughNoTaxAtTrustLevel);
    }

    #[test]
    fn zero_taxable_income_not_applicable() {
        let input = Input {
            taxable_income_before_exemption_dollars: 0,
            ..baseline_complex_trust_top_bracket()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section641Mode::NotApplicableNoTaxableIncome);
    }

    #[test]
    fn bracket_10_compliant() {
        let input = Input {
            taxable_income_before_exemption_dollars: 3_000,
            section_642b_personal_exemption_claimed_dollars: 0,
            ..baseline_complex_trust_top_bracket()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section641Mode::CompliantTaxComputedAtBracket10);
        assert_eq!(result.computed_tax_dollars, 300);
    }

    #[test]
    fn bracket_24_compliant() {
        let input = Input {
            taxable_income_before_exemption_dollars: 10_000,
            section_642b_personal_exemption_claimed_dollars: 0,
            ..baseline_complex_trust_top_bracket()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section641Mode::CompliantTaxComputedAtBracket24);
        // 3150 × 10 % + 6850 × 24 % = 315 + 1644 = 1959
        assert_eq!(result.computed_tax_dollars, 1_959);
    }

    #[test]
    fn bracket_35_compliant() {
        let input = Input {
            taxable_income_before_exemption_dollars: 14_000,
            section_642b_personal_exemption_claimed_dollars: 0,
            ..baseline_complex_trust_top_bracket()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section641Mode::CompliantTaxComputedAtBracket35);
    }

    #[test]
    fn bracket_37_top_compliant() {
        let input = Input {
            taxable_income_before_exemption_dollars: 100_000,
            section_642b_personal_exemption_claimed_dollars: 0,
            ..baseline_complex_trust_top_bracket()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section641Mode::CompliantTaxComputedAtBracket37TopBracket);
    }

    #[test]
    fn at_exactly_15_650_top_bracket_threshold() {
        let input = Input {
            taxable_income_before_exemption_dollars: 15_650,
            section_642b_personal_exemption_claimed_dollars: 0,
            ..baseline_complex_trust_top_bracket()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section641Mode::CompliantTaxComputedAtBracket35);
    }

    #[test]
    fn at_15_651_enters_37_bracket() {
        let input = Input {
            taxable_income_before_exemption_dollars: 15_651,
            section_642b_personal_exemption_claimed_dollars: 0,
            ..baseline_complex_trust_top_bracket()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section641Mode::CompliantTaxComputedAtBracket37TopBracket);
    }

    #[test]
    fn individual_brackets_misused_violation() {
        let input = Input {
            used_individual_brackets_instead: true,
            ..baseline_complex_trust_top_bracket()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section641Mode::ViolationIndividualBracketsUsedInsteadOfTrustCompressed);
    }

    #[test]
    fn esbt_s_corp_portion_top_rate_compliant() {
        let input = Input {
            entity_type: FiduciaryEntityType::ElectingSmallBusinessTrust,
            esbt_portion: EsbtPortion::SCorpPortion,
            taxable_income_before_exemption_dollars: 50_000,
            section_642b_personal_exemption_claimed_dollars: 0,
            distribution_deduction_claimed_dollars: 0,
            ..baseline_complex_trust_top_bracket()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section641Mode::CompliantEsbtSCorpPortionTaxedAtHighestRate);
        assert_eq!(result.computed_tax_dollars, 18_500);
    }

    #[test]
    fn esbt_s_corp_distribution_deduction_violation() {
        let input = Input {
            entity_type: FiduciaryEntityType::ElectingSmallBusinessTrust,
            esbt_portion: EsbtPortion::SCorpPortion,
            distribution_deduction_claimed_dollars: 10_000,
            ..baseline_complex_trust_top_bracket()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section641Mode::ViolationEsbtSCorpPortionImproperlyDistributionDeducted);
    }

    #[test]
    fn distribution_deduction_exceeds_dni_violation() {
        let input = Input {
            distribution_deduction_claimed_dollars: 80_000,
            dni_dollars: 60_000,
            ..baseline_complex_trust_top_bracket()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section641Mode::ViolationDniCeilingExceededByDistributionDeduction);
    }

    #[test]
    fn distribution_deduction_within_dni_compliant() {
        let input = Input {
            distribution_deduction_claimed_dollars: 50_000,
            dni_dollars: 60_000,
            section_642b_personal_exemption_claimed_dollars: 100,
            entity_type: FiduciaryEntityType::DomesticSimpleTrust,
            ..baseline_complex_trust_top_bracket()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section641Mode::CompliantDistributionDeductionAppliedSimpleTrust);
    }

    #[test]
    fn niit_applied_at_top_bracket_compliant() {
        let input = Input {
            taxable_income_before_exemption_dollars: 20_000,
            section_642b_personal_exemption_claimed_dollars: 0,
            undistributed_net_investment_income_dollars: 4_000,
            niit_applied_at_top_bracket_threshold: true,
            ..baseline_complex_trust_top_bracket()
        };
        let result = compute(&input);
        assert!(matches!(
            result.mode,
            Section641Mode::CompliantNiitAppliedAtTopBracketThreshold
                | Section641Mode::CompliantTaxComputedAtBracket37TopBracket
        ));
        assert!(result.niit_dollars > 0);
    }

    #[test]
    fn niit_omitted_at_top_bracket_violation() {
        let input = Input {
            taxable_income_before_exemption_dollars: 20_000,
            section_642b_personal_exemption_claimed_dollars: 0,
            undistributed_net_investment_income_dollars: 4_000,
            niit_applied_at_top_bracket_threshold: false,
            ..baseline_complex_trust_top_bracket()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section641Mode::ViolationNiitOmittedAtTopBracket);
    }

    #[test]
    fn section_642b_exemption_overclaimed_simple_trust_violation() {
        let input = Input {
            entity_type: FiduciaryEntityType::DomesticSimpleTrust,
            section_642b_personal_exemption_claimed_dollars: 200,
            ..baseline_complex_trust_top_bracket()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section641Mode::ViolationSection642bExemptionOverclaimed);
    }

    #[test]
    fn section_642b_estate_exemption_600_compliant() {
        let input = Input {
            entity_type: FiduciaryEntityType::DomesticEstateInProbate,
            section_642b_personal_exemption_claimed_dollars: 600,
            ..baseline_complex_trust_top_bracket()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section641Mode::CompliantTaxComputedAtBracket37TopBracket);
    }

    #[test]
    fn citations_pin_section_641_subsections_and_brackets() {
        let result = compute(&baseline_complex_trust_top_bracket());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("§ 641(a)"));
        assert!(joined.contains("§ 641(b)"));
        assert!(joined.contains("§ 641(c)"));
        assert!(joined.contains("§ 641(d)"));
        assert!(joined.contains("§ 1(e)"));
        assert!(joined.contains("Rev. Proc. 2024-40"));
        assert!(joined.contains("§ 642(b)"));
        assert!(joined.contains("§ 642(c)"));
        assert!(joined.contains("§ 643(a)"));
        assert!(joined.contains("§ 651"));
        assert!(joined.contains("§ 661"));
        assert!(joined.contains("§ 1411(a)(2)"));
        assert!(joined.contains("$15,650"));
        assert!(joined.contains("40×"));
        assert!(joined.contains("Form 1041"));
    }

    #[test]
    fn constant_pin_brackets_and_rates() {
        assert_eq!(TRUST_2025_BRACKET_10_CEILING_DOLLARS, 3_150);
        assert_eq!(TRUST_2025_BRACKET_24_CEILING_DOLLARS, 11_450);
        assert_eq!(TRUST_2025_BRACKET_35_CEILING_DOLLARS, 15_650);
        assert_eq!(TRUST_2026_TOP_BRACKET_THRESHOLD_DOLLARS_APPROX, 16_000);
        assert_eq!(TRUST_BRACKET_10_RATE_BASIS_POINTS, 1_000);
        assert_eq!(TRUST_BRACKET_24_RATE_BASIS_POINTS, 2_400);
        assert_eq!(TRUST_BRACKET_35_RATE_BASIS_POINTS, 3_500);
        assert_eq!(TRUST_BRACKET_37_RATE_BASIS_POINTS, 3_700);
        assert_eq!(SECTION_642_B_ESTATE_EXEMPTION_DOLLARS, 600);
        assert_eq!(SECTION_642_B_COMPLEX_TRUST_EXEMPTION_DOLLARS, 300);
        assert_eq!(SECTION_642_B_SIMPLE_TRUST_EXEMPTION_DOLLARS, 100);
        assert_eq!(SECTION_1411_NIIT_RATE_BASIS_POINTS, 380);
    }

    #[test]
    fn saturating_overflow_defense_extreme_income() {
        let input = Input {
            taxable_income_before_exemption_dollars: u64::MAX,
            section_642b_personal_exemption_claimed_dollars: 0,
            ..baseline_complex_trust_top_bracket()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section641Mode::CompliantTaxComputedAtBracket37TopBracket);
    }
}
