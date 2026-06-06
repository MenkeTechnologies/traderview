//! IRC § 651 — Deduction for Trusts Distributing Current Income
//! Only (Simple Trust Distribution Deduction).
//!
//! Pure-compute simple-trust distribution deduction. Simple trust
//! is a trust whose terms require that ALL income be distributed
//! currently, do NOT permit § 642(c) charitable distributions, and
//! which did NOT distribute amounts other than current income (no
//! corpus distributions during the taxable year). Any failure of
//! these conditions converts the trust to a COMPLEX trust governed
//! by § 661 instead.
//!
//! Trader-critical because simple-trust setups are common in
//! family-office QPRT (Qualified Personal Residence Trust),
//! GRAT (Grantor Retained Annuity Trust) remainder, charitable
//! lead annuity trust (CLAT) post-charitable phase, and
//! traditional income-only family trusts. § 651 + § 652 together
//! form the conduit principle parallel to § 661 + § 662 but with
//! mandatory rather than discretionary distributions.
//!
//! Statute (verbatim mapping):
//! - § 651(a) — GENERAL RULE: in the case of a trust whose terms
//!   provide that all of its income is required to be distributed
//!   currently AND do not provide that any amounts are to be paid,
//!   permanently set aside, or used for the purposes specified in
//!   § 642(c) (relating to charitable contribution deduction), AND
//!   which did not distribute amounts other than income for the
//!   taxable year, a DEDUCTION is allowed in computing trust
//!   taxable income equal to the amount of the income required to
//!   be distributed currently.
//! - § 651(b) — LIMITATION ON AMOUNT DEDUCTIBLE: if the amount of
//!   income required to be distributed currently exceeds DNI of
//!   the trust for the taxable year, the deduction shall be
//!   LIMITED to the amount of DNI. For this purpose, DNI shall be
//!   computed without regard to any portion of DNI attributable to
//!   tax-exempt interest (§ 643(a)(5) modification).
//! - § 652(a) — BENEFICIARY INCLUSION: amount of income required
//!   to be distributed currently by a § 651 trust shall be
//!   INCLUDED in the gross income of the beneficiaries to whom
//!   such income is required to be distributed, WHETHER OR NOT
//!   ACTUALLY DISTRIBUTED. If such amount exceeds DNI, each
//!   beneficiary includes a RATABLE portion equal to (income
//!   required to be distributed to that beneficiary / total income
//!   required to be distributed) × DNI.
//! - § 652(b) — CHARACTER: character of amounts allocated to
//!   beneficiaries per Treas. Reg. § 1.652(b)-2 (same proportion
//!   of each DNI class).
//! - § 652(c) — DIFFERENT TAXABLE YEAR: beneficiary includes
//!   amounts on basis of trust's taxable year ending in the
//!   beneficiary's taxable year.
//! - Treas. Reg. § 1.651(a)-1 — simple trusts deduction in
//!   general.
//! - Treas. Reg. § 1.651(a)-2 — definition of income required to
//!   be distributed currently.
//! - Treas. Reg. § 1.651(b)-1 — deduction for distributions to
//!   beneficiaries.
//! - Treas. Reg. § 1.652(b)-2 — character of amounts.
//!
//! Web research (verified 2026-06-03):
//! - Cornell LII § 651 + Treas. Reg. § 1.651(a)-1 confirm three
//!   simple-trust requirements: all income current distribution +
//!   no § 642(c) provision + no corpus distribution this year.
//! - IRS Form 1041 Schedule B implements DNI ceiling.
//! - Greenleaf Trust + Miami Law Heckerling Demystifying DNI
//!   confirm simple-trust mandatory-only distribution scheme.
//! - Tax Notes IRC § 651 confirms § 651(b) DNI cap.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const SECTION_651_REQUIREMENTS_COUNT: u32 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TrustType {
    SimpleTrustQualifyingUnderSection651,
    ComplexTrustUseSection661,
    GrantorTrustPassThrough,
    EstateProbateUseSection661,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SimpleTrustRequirementsFailureCategory {
    AllRequirementsSatisfied,
    CorpusDistributionMadeInYear,
    Section642cCharitableProvisionInInstrument,
    IncomeNotRequiredToBeDistributedCurrently,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BeneficiaryInclusionStatus {
    BeneficiaryIncludedFullAmount,
    BeneficiaryIncludedRatablePortionAfterDniLimitation,
    BeneficiaryDidNotInclude,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section651Mode {
    NotApplicableNoIncomeRequiredToBeDistributed,
    NotApplicableTrustIsComplexNotSimpleUseSection661,
    NotApplicableGrantorTrustPassThrough,
    CompliantIncomeFullyDistributedAndIncludedByBeneficiary,
    CompliantDeductionCappedAtDniRatableInclusionToBeneficiaries,
    CompliantCharacterProportionatelyAllocatedSection652b,
    CompliantBeneficiaryDifferentTaxableYearSection652c,
    ViolationDeductionExceededDniCeiling,
    ViolationCorpusDistributionMadeTrustNotSimple,
    ViolationCharitableSection642cProvisionMakesTrustComplex,
    ViolationBeneficiaryDidNotIncludeIncomeRequiredToBeDistributed,
    ViolationBeneficiaryUsedIncorrectTaxableYearSection652c,
    ViolationCharacterNotProportionatelyAllocated,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub trust_type: TrustType,
    pub simple_trust_requirements_failure: SimpleTrustRequirementsFailureCategory,
    pub income_required_to_be_distributed_currently_dollars: u64,
    pub dni_dollars: u64,
    pub distribution_deduction_claimed_dollars: u64,
    pub beneficiary_inclusion_status: BeneficiaryInclusionStatus,
    pub beneficiary_used_correct_taxable_year_per_section_652c: bool,
    pub character_proportionately_allocated_per_section_652b: bool,
    pub beneficiary_taxable_year_different_from_trust: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section651Mode,
    pub distribution_deduction_dollars: u64,
    pub beneficiary_inclusion_dollars: u64,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type Section651Input = Input;
pub type Section651Output = Output;
pub type Section651Result = Output;

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "26 U.S.C. § 651(a) — simple trust deduction: amount of income required to be distributed currently if (1) all income required current distribution + (2) no § 642(c) charitable provision + (3) no corpus distribution this year".to_string(),
        "26 U.S.C. § 651(b) — deduction LIMITED to DNI; tax-exempt interest portion of DNI excluded".to_string(),
        "26 U.S.C. § 652(a) — beneficiary includes amount required to be distributed whether or not actually distributed; ratable allocation when income required > DNI".to_string(),
        "26 U.S.C. § 652(b) — character per Treas. Reg. § 1.652(b)-2: same proportion of each DNI class".to_string(),
        "26 U.S.C. § 652(c) — beneficiary includes amounts on basis of trust's taxable year ending in beneficiary's taxable year".to_string(),
        "Treas. Reg. § 1.651(a)-1 — simple trusts deduction in general; three requirements must be met for simple trust treatment".to_string(),
        "Treas. Reg. § 1.651(a)-2 — definition of 'income required to be distributed currently'".to_string(),
        "Treas. Reg. § 1.651(b)-1 — deduction for distributions to beneficiaries".to_string(),
        "Treas. Reg. § 1.652(b)-2 — character of amounts in beneficiary inclusion".to_string(),
        "26 U.S.C. § 643(a) — DNI ceiling (governs § 651(b) limitation)".to_string(),
        "IRS Form 1041 Schedule B — distribution deduction worksheet; Schedule K-1 — beneficiary character allocation".to_string(),
        "Simple vs complex trust: any corpus distribution OR § 642(c) charitable provision converts trust to § 661 complex trust".to_string(),
    ];

    if input.trust_type == TrustType::ComplexTrustUseSection661
        || input.trust_type == TrustType::EstateProbateUseSection661
    {
        return Output {
            mode: Section651Mode::NotApplicableTrustIsComplexNotSimpleUseSection661,
            distribution_deduction_dollars: 0,
            beneficiary_inclusion_dollars: 0,
            statutory_basis: "Complex trust / estate uses § 661 distribution deduction".to_string(),
            notes: "Trust does not satisfy § 651 simple-trust requirements; use § 661 complex-trust distribution deduction instead.".to_string(),
            citations,
        };
    }

    if input.trust_type == TrustType::GrantorTrustPassThrough {
        return Output {
            mode: Section651Mode::NotApplicableGrantorTrustPassThrough,
            distribution_deduction_dollars: 0,
            beneficiary_inclusion_dollars: 0,
            statutory_basis:
                "Subpart E grantor trust passthrough; no § 651 deduction at trust level".to_string(),
            notes: "Grantor trust passthrough; all income flows to grantor; § 651 inapplicable."
                .to_string(),
            citations,
        };
    }

    if input.income_required_to_be_distributed_currently_dollars == 0 {
        return Output {
            mode: Section651Mode::NotApplicableNoIncomeRequiredToBeDistributed,
            distribution_deduction_dollars: 0,
            beneficiary_inclusion_dollars: 0,
            statutory_basis:
                "§ 651 inapplicable absent income required to be distributed currently".to_string(),
            notes: "No income required to be distributed currently; § 651 deduction = 0."
                .to_string(),
            citations,
        };
    }

    match input.simple_trust_requirements_failure {
        SimpleTrustRequirementsFailureCategory::CorpusDistributionMadeInYear => {
            return Output {
                mode: Section651Mode::ViolationCorpusDistributionMadeTrustNotSimple,
                distribution_deduction_dollars: 0,
                beneficiary_inclusion_dollars: 0,
                statutory_basis: "§ 651(a) — simple trust requirement: NO corpus distribution this year".to_string(),
                notes: "VIOLATION § 651(a): trust distributed corpus during taxable year; not a simple trust; use § 661 complex trust deduction.".to_string(),
                citations,
            };
        }
        SimpleTrustRequirementsFailureCategory::Section642cCharitableProvisionInInstrument => {
            return Output {
                mode: Section651Mode::ViolationCharitableSection642cProvisionMakesTrustComplex,
                distribution_deduction_dollars: 0,
                beneficiary_inclusion_dollars: 0,
                statutory_basis: "§ 651(a) — simple trust requirement: NO § 642(c) charitable provision".to_string(),
                notes: "VIOLATION § 651(a): trust instrument contains § 642(c) charitable provision; trust is COMPLEX (not simple); use § 661.".to_string(),
                citations,
            };
        }
        SimpleTrustRequirementsFailureCategory::IncomeNotRequiredToBeDistributedCurrently => {
            return Output {
                mode: Section651Mode::ViolationCorpusDistributionMadeTrustNotSimple,
                distribution_deduction_dollars: 0,
                beneficiary_inclusion_dollars: 0,
                statutory_basis: "§ 651(a) — simple trust requirement: all income required to be distributed currently".to_string(),
                notes: "VIOLATION § 651(a): trust does not require all income to be distributed currently; trust is COMPLEX; use § 661.".to_string(),
                citations,
            };
        }
        SimpleTrustRequirementsFailureCategory::AllRequirementsSatisfied => {}
    }

    let deduction_capped_at_dni = input
        .income_required_to_be_distributed_currently_dollars
        .min(input.dni_dollars);

    if input.distribution_deduction_claimed_dollars > input.dni_dollars {
        return Output {
            mode: Section651Mode::ViolationDeductionExceededDniCeiling,
            distribution_deduction_dollars: deduction_capped_at_dni,
            beneficiary_inclusion_dollars: 0,
            statutory_basis: "§ 651(b) — deduction CAPPED at DNI".to_string(),
            notes: format!(
                "VIOLATION § 651(b): claimed deduction ${} exceeds DNI ceiling ${}; correct deduction = ${}.",
                input.distribution_deduction_claimed_dollars, input.dni_dollars, deduction_capped_at_dni
            ),
            citations,
        };
    }

    if input.beneficiary_taxable_year_different_from_trust
        && !input.beneficiary_used_correct_taxable_year_per_section_652c
    {
        return Output {
            mode: Section651Mode::ViolationBeneficiaryUsedIncorrectTaxableYearSection652c,
            distribution_deduction_dollars: deduction_capped_at_dni,
            beneficiary_inclusion_dollars: 0,
            statutory_basis: "§ 652(c) — beneficiary includes amounts on basis of trust's taxable year ending in beneficiary's taxable year".to_string(),
            notes: "VIOLATION § 652(c): beneficiary used incorrect taxable year for inclusion; must include based on trust's taxable year ending in beneficiary's taxable year.".to_string(),
            citations,
        };
    }

    if input.beneficiary_inclusion_status == BeneficiaryInclusionStatus::BeneficiaryDidNotInclude {
        return Output {
            mode: Section651Mode::ViolationBeneficiaryDidNotIncludeIncomeRequiredToBeDistributed,
            distribution_deduction_dollars: deduction_capped_at_dni,
            beneficiary_inclusion_dollars: 0,
            statutory_basis: "§ 652(a) — beneficiary includes income WHETHER OR NOT ACTUALLY DISTRIBUTED".to_string(),
            notes: "VIOLATION § 652(a): beneficiary did not include amount of income required to be distributed in gross income; statute requires inclusion whether or not actually distributed.".to_string(),
            citations,
        };
    }

    if !input.character_proportionately_allocated_per_section_652b {
        return Output {
            mode: Section651Mode::ViolationCharacterNotProportionatelyAllocated,
            distribution_deduction_dollars: deduction_capped_at_dni,
            beneficiary_inclusion_dollars: deduction_capped_at_dni,
            statutory_basis: "§ 652(b) + Treas. Reg. § 1.652(b)-2 — character must be proportional to DNI class composition".to_string(),
            notes: "VIOLATION § 652(b): distribution character not proportionately allocated across DNI classes (ordinary income / qualified dividends / LTCG / tax-exempt interest).".to_string(),
            citations,
        };
    }

    if input.income_required_to_be_distributed_currently_dollars > input.dni_dollars {
        return Output {
            mode: Section651Mode::CompliantDeductionCappedAtDniRatableInclusionToBeneficiaries,
            distribution_deduction_dollars: deduction_capped_at_dni,
            beneficiary_inclusion_dollars: deduction_capped_at_dni,
            statutory_basis: "§ 651(b) + § 652(a) — DNI cap with ratable beneficiary inclusion".to_string(),
            notes: format!(
                "COMPLIANT: income required ${} exceeds DNI ${}; deduction capped at DNI ${}; beneficiaries include ratable portions per § 652(a) second sentence.",
                input.income_required_to_be_distributed_currently_dollars,
                input.dni_dollars,
                deduction_capped_at_dni
            ),
            citations,
        };
    }

    if input.beneficiary_taxable_year_different_from_trust
        && input.beneficiary_used_correct_taxable_year_per_section_652c
    {
        return Output {
            mode: Section651Mode::CompliantBeneficiaryDifferentTaxableYearSection652c,
            distribution_deduction_dollars: deduction_capped_at_dni,
            beneficiary_inclusion_dollars: deduction_capped_at_dni,
            statutory_basis: "§ 652(c) — different taxable year rule properly applied".to_string(),
            notes: format!(
                "COMPLIANT § 652(c): beneficiary uses trust's taxable year ending in beneficiary's taxable year for inclusion; deduction = ${}.",
                deduction_capped_at_dni
            ),
            citations,
        };
    }

    Output {
        mode: Section651Mode::CompliantIncomeFullyDistributedAndIncludedByBeneficiary,
        distribution_deduction_dollars: deduction_capped_at_dni,
        beneficiary_inclusion_dollars: deduction_capped_at_dni,
        statutory_basis: "§ 651(a) + § 652(a) — simple trust distribution and inclusion satisfied".to_string(),
        notes: format!(
            "COMPLIANT § 651(a): simple trust deduction = ${} (income required to be distributed ${} within DNI ${}); beneficiary includes full amount under § 652(a); character proportional under § 652(b).",
            deduction_capped_at_dni,
            input.income_required_to_be_distributed_currently_dollars,
            input.dni_dollars
        ),
        citations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_simple_trust_compliant() -> Input {
        Input {
            trust_type: TrustType::SimpleTrustQualifyingUnderSection651,
            simple_trust_requirements_failure:
                SimpleTrustRequirementsFailureCategory::AllRequirementsSatisfied,
            income_required_to_be_distributed_currently_dollars: 60_000,
            dni_dollars: 100_000,
            distribution_deduction_claimed_dollars: 60_000,
            beneficiary_inclusion_status: BeneficiaryInclusionStatus::BeneficiaryIncludedFullAmount,
            beneficiary_used_correct_taxable_year_per_section_652c: true,
            character_proportionately_allocated_per_section_652b: true,
            beneficiary_taxable_year_different_from_trust: false,
        }
    }

    #[test]
    fn complex_trust_redirects_to_section_661() {
        let input = Input {
            trust_type: TrustType::ComplexTrustUseSection661,
            ..baseline_simple_trust_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section651Mode::NotApplicableTrustIsComplexNotSimpleUseSection661
        );
    }

    #[test]
    fn estate_probate_uses_section_661() {
        let input = Input {
            trust_type: TrustType::EstateProbateUseSection661,
            ..baseline_simple_trust_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section651Mode::NotApplicableTrustIsComplexNotSimpleUseSection661
        );
    }

    #[test]
    fn grantor_trust_passthrough_not_applicable() {
        let input = Input {
            trust_type: TrustType::GrantorTrustPassThrough,
            ..baseline_simple_trust_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section651Mode::NotApplicableGrantorTrustPassThrough
        );
    }

    #[test]
    fn no_income_required_not_applicable() {
        let input = Input {
            income_required_to_be_distributed_currently_dollars: 0,
            ..baseline_simple_trust_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section651Mode::NotApplicableNoIncomeRequiredToBeDistributed
        );
    }

    #[test]
    fn simple_trust_baseline_compliant() {
        let result = compute(&baseline_simple_trust_compliant());
        assert_eq!(
            result.mode,
            Section651Mode::CompliantIncomeFullyDistributedAndIncludedByBeneficiary
        );
        assert_eq!(result.distribution_deduction_dollars, 60_000);
        assert_eq!(result.beneficiary_inclusion_dollars, 60_000);
    }

    #[test]
    fn corpus_distribution_makes_trust_complex_violation() {
        let input = Input {
            simple_trust_requirements_failure:
                SimpleTrustRequirementsFailureCategory::CorpusDistributionMadeInYear,
            ..baseline_simple_trust_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section651Mode::ViolationCorpusDistributionMadeTrustNotSimple
        );
    }

    #[test]
    fn charitable_section_642c_provision_makes_trust_complex() {
        let input = Input {
            simple_trust_requirements_failure:
                SimpleTrustRequirementsFailureCategory::Section642cCharitableProvisionInInstrument,
            ..baseline_simple_trust_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section651Mode::ViolationCharitableSection642cProvisionMakesTrustComplex
        );
    }

    #[test]
    fn income_not_required_to_be_distributed_currently_violation() {
        let input = Input {
            simple_trust_requirements_failure:
                SimpleTrustRequirementsFailureCategory::IncomeNotRequiredToBeDistributedCurrently,
            ..baseline_simple_trust_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section651Mode::ViolationCorpusDistributionMadeTrustNotSimple
        );
    }

    #[test]
    fn deduction_exceeds_dni_violation_capped() {
        let input = Input {
            distribution_deduction_claimed_dollars: 150_000,
            ..baseline_simple_trust_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section651Mode::ViolationDeductionExceededDniCeiling
        );
        assert_eq!(result.distribution_deduction_dollars, 60_000);
    }

    #[test]
    fn income_required_exceeds_dni_capped_at_dni_compliant() {
        let input = Input {
            income_required_to_be_distributed_currently_dollars: 150_000,
            distribution_deduction_claimed_dollars: 100_000,
            ..baseline_simple_trust_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section651Mode::CompliantDeductionCappedAtDniRatableInclusionToBeneficiaries
        );
        assert_eq!(result.distribution_deduction_dollars, 100_000);
        assert_eq!(result.beneficiary_inclusion_dollars, 100_000);
    }

    #[test]
    fn beneficiary_did_not_include_violation() {
        let input = Input {
            beneficiary_inclusion_status: BeneficiaryInclusionStatus::BeneficiaryDidNotInclude,
            ..baseline_simple_trust_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section651Mode::ViolationBeneficiaryDidNotIncludeIncomeRequiredToBeDistributed
        );
    }

    #[test]
    fn beneficiary_includes_whether_or_not_actually_distributed_compliant() {
        let result = compute(&baseline_simple_trust_compliant());
        assert_eq!(result.beneficiary_inclusion_dollars, 60_000);
    }

    #[test]
    fn beneficiary_incorrect_taxable_year_violation() {
        let input = Input {
            beneficiary_taxable_year_different_from_trust: true,
            beneficiary_used_correct_taxable_year_per_section_652c: false,
            ..baseline_simple_trust_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section651Mode::ViolationBeneficiaryUsedIncorrectTaxableYearSection652c
        );
    }

    #[test]
    fn beneficiary_different_taxable_year_correctly_applied_compliant() {
        let input = Input {
            beneficiary_taxable_year_different_from_trust: true,
            beneficiary_used_correct_taxable_year_per_section_652c: true,
            ..baseline_simple_trust_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section651Mode::CompliantBeneficiaryDifferentTaxableYearSection652c
        );
    }

    #[test]
    fn character_not_proportionately_allocated_violation() {
        let input = Input {
            character_proportionately_allocated_per_section_652b: false,
            ..baseline_simple_trust_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section651Mode::ViolationCharacterNotProportionatelyAllocated
        );
    }

    #[test]
    fn ratable_inclusion_when_income_exceeds_dni_per_section_652a_second_sentence() {
        let input = Input {
            income_required_to_be_distributed_currently_dollars: 200_000,
            dni_dollars: 80_000,
            distribution_deduction_claimed_dollars: 80_000,
            beneficiary_inclusion_status:
                BeneficiaryInclusionStatus::BeneficiaryIncludedRatablePortionAfterDniLimitation,
            ..baseline_simple_trust_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section651Mode::CompliantDeductionCappedAtDniRatableInclusionToBeneficiaries
        );
        assert_eq!(result.distribution_deduction_dollars, 80_000);
        assert_eq!(result.beneficiary_inclusion_dollars, 80_000);
    }

    #[test]
    fn deduction_at_exactly_dni_compliant() {
        let input = Input {
            income_required_to_be_distributed_currently_dollars: 100_000,
            distribution_deduction_claimed_dollars: 100_000,
            ..baseline_simple_trust_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section651Mode::CompliantIncomeFullyDistributedAndIncludedByBeneficiary
        );
        assert_eq!(result.distribution_deduction_dollars, 100_000);
    }

    #[test]
    fn citations_pin_section_651_652_and_treas_regs() {
        let result = compute(&baseline_simple_trust_compliant());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("§ 651(a)"));
        assert!(joined.contains("§ 651(b)"));
        assert!(joined.contains("§ 652(a)"));
        assert!(joined.contains("§ 652(b)"));
        assert!(joined.contains("§ 652(c)"));
        assert!(joined.contains("§ 1.651(a)-1"));
        assert!(joined.contains("§ 1.651(a)-2"));
        assert!(joined.contains("§ 1.651(b)-1"));
        assert!(joined.contains("§ 1.652(b)-2"));
        assert!(joined.contains("§ 643(a)"));
        assert!(joined.contains("Form 1041 Schedule B"));
        assert!(joined.contains("Schedule K-1"));
    }

    #[test]
    fn constant_pin_three_requirements() {
        assert_eq!(SECTION_651_REQUIREMENTS_COUNT, 3);
    }

    #[test]
    fn saturating_overflow_defense_extreme_income() {
        let input = Input {
            income_required_to_be_distributed_currently_dollars: u64::MAX,
            dni_dollars: u64::MAX,
            distribution_deduction_claimed_dollars: u64::MAX,
            ..baseline_simple_trust_compliant()
        };
        let result = compute(&input);
        assert!(matches!(
            result.mode,
            Section651Mode::CompliantIncomeFullyDistributedAndIncludedByBeneficiary
                | Section651Mode::CompliantDeductionCappedAtDniRatableInclusionToBeneficiaries
        ));
    }
}
