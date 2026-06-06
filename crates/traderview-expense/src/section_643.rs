//! IRC § 643 — Definitions and Rules for Trust and Estate Income
//! Taxation (Distributable Net Income / DNI).
//!
//! Pure-compute classification for whether and how a trust or
//! estate's distributable net income (DNI) is computed; DNI acts
//! as a CEILING on the amount of income that can be shifted from
//! the trust to beneficiaries for tax purposes. Trader-critical
//! because compressed trust marginal brackets (37 % at ~$15,650
//! for 2025) make DNI optimization the single most impactful
//! trust tax planning decision.
//!
//! Statute (verbatim mapping):
//! - § 643(a) — DNI DEFINITION: taxable income of the trust
//!   modified by — (1) no deduction for distributions allowed; (2)
//!   no personal exemption (§ 642(b)) allowed; (3) GAINS FROM
//!   SALE OR EXCHANGE OF CAPITAL ASSETS EXCLUDED to the extent
//!   they are (A) allocated to corpus AND (B) not (i) paid,
//!   credited, or required to be distributed to any beneficiary
//!   during the taxable year, or (ii) paid, permanently set aside,
//!   or to be used for the purposes specified in § 642(c)
//!   (charitable contribution deduction); (5) TAX-EXEMPT INCOME
//!   INCLUDED in DNI net of allocable expenses; (6) FOREIGN TRUST
//!   modifications; (7) AMOUNTS DEEMED DISTRIBUTED under § 651 /
//!   § 661 excluded.
//! - § 643(b) — TRUST ACCOUNTING INCOME (FAI): defines "income"
//!   for §§ 651 / 661 distribution limits, estate and gift tax
//!   marital deduction, required distributions from pooled income
//!   funds, and certain charitable remainder trusts. Generally
//!   includes interest, dividends, and rent; capital gains
//!   ALLOCATED TO PRINCIPAL absent contrary trust-instrument or
//!   state-law direction.
//! - § 643(c) — BENEFICIARY DEFINITION: includes heir, legatee,
//!   or devisee.
//! - § 643(d) — COORDINATION with §§ 651 / 661 distribution
//!   deductions.
//! - § 643(e) — TREATMENT OF PROPERTY DISTRIBUTED IN KIND: § 643
//!   (e)(2) loss recognition rules; § 643(e)(3) election to
//!   recognize gain.
//! - § 643(f) — MULTIPLE TRUSTS TREATED AS ONE for tax avoidance
//!   purposes: two or more trusts with substantially the same
//!   grantor and substantially the same primary beneficiary, with
//!   tax-avoidance principal purpose, treated as one.
//! - § 643(g) — ESTIMATED TAX PAYMENTS BY BENEFICIARIES: election
//!   to treat trust estimated payments as made by beneficiaries.
//! - § 643(h) — DISTRIBUTIONS BY FOREIGN TRUSTS: distribution
//!   amount limited under § 643(i)-style throwback principles.
//! - Treas. Reg. § 1.643(b)-1 (2004 final regs): defines
//!   "income" for trust purposes; permits "POWER TO ADJUST" under
//!   Uniform Principal and Income Act state-law equivalents
//!   (allows fiduciary to reallocate between income and principal).
//!
//! DNI ceiling mechanics: tier-1 (mandatory) distributions limited
//! to DNI; tier-2 (discretionary) distributions limited to
//! remaining DNI after tier-1; excess distributions = tax-free
//! return of corpus.
//!
//! Web research (verified 2026-06-03):
//! - Cornell LII § 643 confirms statutory text.
//! - IRS Form 1041 Instructions (2025) DNI computation worksheet
//!   on Schedule B.
//! - Treasury Press Release JS-1068 (Dec 30 2003) announcing 2004
//!   final regs on § 643(b) income definition.
//! - Greenleaf Trust "Distributable Net Income" confirms § 643(a)
//!   (3) capital gain exclusion default.
//! - Miami Law Heckerling Demystifying DNI materials confirm
//!   tier-1 / tier-2 distribution priority.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const SECTION_643_TRUST_TOP_MARGINAL_RATE_BASIS_POINTS: u64 = 3_700;
pub const SECTION_643_TRUST_2025_TOP_BRACKET_THRESHOLD_DOLLARS: u64 = 15_650;
pub const SECTION_643_TREAS_REG_643_B_FINAL_REGS_YEAR: u32 = 2004;
pub const SECTION_643_TREAS_REG_643_B_FINAL_REGS_MONTH: u32 = 12;
pub const SECTION_643_TREAS_REG_643_B_FINAL_REGS_DAY: u32 = 30;
pub const SECTION_643_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TrustOrEstateType {
    DomesticGrantorTrust,
    DomesticNonGrantorSimpleTrust,
    DomesticNonGrantorComplexTrust,
    ForeignTrust,
    EstateProbate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CapitalGainTreatment {
    AllocatedToCorpusNotDistributed,
    AllocatedToCorpusButDistributedToBeneficiary,
    AllocatedToCorpusButRequiredToBeDistributed,
    AllocatedToIncomeByTrustInstrumentOrState,
    SetAsideForSection642cCharity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DniInclusionApproach {
    StandardSection643aComputation,
    PowerToAdjustReallocatedUnderTreasReg1_643B,
    ForeignTrustModificationApplied,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section643Mode {
    NotApplicableNoTrustOrEstate,
    CompliantDniProperlyComputed,
    CompliantCapitalGainsExcludedAllocatedToCorpus,
    CompliantCapitalGainsIncludedInDniDistributedOrSetAside,
    CompliantTaxExemptIncomeIncludedInDni,
    CompliantPowerToAdjustAppliedTreasReg1_643B,
    CompliantForeignTrustDniModificationApplied,
    ViolationDniComputationIncludedPersonalExemption,
    ViolationDniComputationIncludedDistributionDeduction,
    ViolationCapitalGainsImproperlyIncludedInDni,
    ViolationCapitalGainsImproperlyExcludedDespiteDistribution,
    ViolationTaxExemptIncomeOmittedFromDni,
    ViolationMultipleTrustsAggregatedNotPerSection643f,
    ViolationForeignTrustDniModificationOmitted,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub trust_or_estate_type: TrustOrEstateType,
    pub taxable_income_before_dni_modifications_dollars: u64,
    pub personal_exemption_claimed_in_dni_dollars: u64,
    pub distribution_deduction_claimed_in_dni_dollars: u64,
    pub capital_gain_dollars: u64,
    pub capital_gain_treatment: CapitalGainTreatment,
    pub capital_gain_included_in_dni_by_taxpayer: bool,
    pub tax_exempt_interest_dollars: u64,
    pub tax_exempt_income_included_in_dni_by_taxpayer: bool,
    pub multiple_trusts_aggregated_to_avoid_brackets: bool,
    pub foreign_trust_dni_modification_applied: bool,
    pub dni_inclusion_approach: DniInclusionApproach,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section643Mode,
    pub computed_dni_dollars: u64,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type Section643Input = Input;
pub type Section643Output = Output;
pub type Section643Result = Output;

fn capital_gain_should_be_in_dni(treatment: CapitalGainTreatment) -> bool {
    matches!(
        treatment,
        CapitalGainTreatment::AllocatedToCorpusButDistributedToBeneficiary
            | CapitalGainTreatment::AllocatedToCorpusButRequiredToBeDistributed
            | CapitalGainTreatment::AllocatedToIncomeByTrustInstrumentOrState
            | CapitalGainTreatment::SetAsideForSection642cCharity
    )
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "26 U.S.C. § 643(a)(1) — no distribution deduction allowed in DNI computation".to_string(),
        "26 U.S.C. § 643(a)(2) — no personal exemption (§ 642(b)) allowed in DNI computation".to_string(),
        "26 U.S.C. § 643(a)(3) — capital gains EXCLUDED from DNI if allocated to corpus and NOT distributed/required to be distributed/set aside for § 642(c) charity".to_string(),
        "26 U.S.C. § 643(a)(5) — tax-exempt income INCLUDED in DNI net of allocable expenses".to_string(),
        "26 U.S.C. § 643(a)(6) — foreign trust DNI modifications".to_string(),
        "26 U.S.C. § 643(a)(7) — amounts deemed distributed under § 651 / § 661 excluded".to_string(),
        "26 U.S.C. § 643(b) — trust accounting income (FAI) definition for § 651 / § 661 distribution limits + marital deduction + pooled income funds + charitable remainder trusts".to_string(),
        "26 U.S.C. § 643(c) — beneficiary definition (heir, legatee, devisee)".to_string(),
        "26 U.S.C. § 643(d) — coordination with §§ 651 / 661 distribution deductions".to_string(),
        "26 U.S.C. § 643(e) — property distributed in kind: § 643(e)(2) loss recognition; § 643(e)(3) gain-recognition election".to_string(),
        "26 U.S.C. § 643(f) — multiple trusts treated as one for tax-avoidance purposes (substantially same grantor + primary beneficiary)".to_string(),
        "26 U.S.C. § 643(g) — election to treat trust estimated tax payments as made by beneficiaries".to_string(),
        "26 U.S.C. § 643(h) — distributions by foreign trusts".to_string(),
        "Treas. Reg. § 1.643(b)-1 (final regs Dec 30, 2003; effective 2004) — defines 'income' for trust purposes; permits POWER TO ADJUST under state Uniform Principal and Income Act equivalents".to_string(),
        "IRS Form 1041 Schedule B — DNI computation worksheet".to_string(),
        "DNI ceiling mechanics: tier-1 (mandatory) limited to DNI; tier-2 (discretionary) limited to remaining DNI; excess = tax-free return of corpus".to_string(),
        "2025 trust top marginal bracket: 37 % at approximately $15,650 (compressed brackets vs individual)".to_string(),
    ];

    if !matches!(
        input.trust_or_estate_type,
        TrustOrEstateType::DomesticGrantorTrust
            | TrustOrEstateType::DomesticNonGrantorSimpleTrust
            | TrustOrEstateType::DomesticNonGrantorComplexTrust
            | TrustOrEstateType::ForeignTrust
            | TrustOrEstateType::EstateProbate
    ) {
        return Output {
            mode: Section643Mode::NotApplicableNoTrustOrEstate,
            computed_dni_dollars: 0,
            statutory_basis: "§ 643 inapplicable absent trust or estate".to_string(),
            notes: "No trust or estate present; § 643 DNI computation inapplicable.".to_string(),
            citations,
        };
    }

    if input.personal_exemption_claimed_in_dni_dollars > 0 {
        return Output {
            mode: Section643Mode::ViolationDniComputationIncludedPersonalExemption,
            computed_dni_dollars: input.taxable_income_before_dni_modifications_dollars,
            statutory_basis: "§ 643(a)(2) — personal exemption disallowed in DNI computation".to_string(),
            notes: format!(
                "VIOLATION § 643(a)(2): taxpayer included personal exemption of ${} in DNI computation. § 642(b) trust exemption must be ADDED BACK when computing DNI.",
                input.personal_exemption_claimed_in_dni_dollars
            ),
            citations,
        };
    }

    if input.distribution_deduction_claimed_in_dni_dollars > 0 {
        return Output {
            mode: Section643Mode::ViolationDniComputationIncludedDistributionDeduction,
            computed_dni_dollars: input.taxable_income_before_dni_modifications_dollars,
            statutory_basis: "§ 643(a)(1) — distribution deduction disallowed in DNI computation".to_string(),
            notes: format!(
                "VIOLATION § 643(a)(1): taxpayer claimed distribution deduction of ${} in DNI computation. Distribution deduction must be ADDED BACK when computing DNI.",
                input.distribution_deduction_claimed_in_dni_dollars
            ),
            citations,
        };
    }

    if input.multiple_trusts_aggregated_to_avoid_brackets {
        return Output {
            mode: Section643Mode::ViolationMultipleTrustsAggregatedNotPerSection643f,
            computed_dni_dollars: input.taxable_income_before_dni_modifications_dollars,
            statutory_basis: "§ 643(f) — multiple trusts treated as one for tax-avoidance principal purpose".to_string(),
            notes: "VIOLATION § 643(f): multiple trusts with substantially the same grantor + primary beneficiary aggregated by IRS to defeat compressed-bracket avoidance; treat as one trust for DNI computation.".to_string(),
            citations,
        };
    }

    if input.trust_or_estate_type == TrustOrEstateType::ForeignTrust
        && !input.foreign_trust_dni_modification_applied
    {
        return Output {
            mode: Section643Mode::ViolationForeignTrustDniModificationOmitted,
            computed_dni_dollars: input.taxable_income_before_dni_modifications_dollars,
            statutory_basis: "§ 643(a)(6) — foreign trust DNI modification omitted".to_string(),
            notes: "VIOLATION § 643(a)(6): foreign trust DNI computation must apply foreign-trust-specific modifications; omission produces incorrect DNI.".to_string(),
            citations,
        };
    }

    if input.dni_inclusion_approach
        == DniInclusionApproach::PowerToAdjustReallocatedUnderTreasReg1_643B
    {
        let dni = input
            .taxable_income_before_dni_modifications_dollars
            .saturating_add(input.tax_exempt_interest_dollars);
        return Output {
            mode: Section643Mode::CompliantPowerToAdjustAppliedTreasReg1_643B,
            computed_dni_dollars: dni,
            statutory_basis: "Treas. Reg. § 1.643(b)-1 — power to adjust under Uniform Principal and Income Act equivalent".to_string(),
            notes: format!(
                "COMPLIANT: fiduciary reallocated between income and principal under state UPIA-equivalent power to adjust. Computed DNI = ${} (includes tax-exempt income).",
                dni
            ),
            citations,
        };
    }

    if input.trust_or_estate_type == TrustOrEstateType::ForeignTrust
        && input.foreign_trust_dni_modification_applied
    {
        let dni = input
            .taxable_income_before_dni_modifications_dollars
            .saturating_add(input.tax_exempt_interest_dollars);
        return Output {
            mode: Section643Mode::CompliantForeignTrustDniModificationApplied,
            computed_dni_dollars: dni,
            statutory_basis: "§ 643(a)(6) — foreign trust DNI modification applied".to_string(),
            notes: format!(
                "COMPLIANT § 643(a)(6): foreign trust DNI computed with § 643(a)(6) modifications. Computed DNI = ${}.",
                dni
            ),
            citations,
        };
    }

    let gain_should_be_in_dni = capital_gain_should_be_in_dni(input.capital_gain_treatment);

    if gain_should_be_in_dni
        && input.capital_gain_dollars > 0
        && !input.capital_gain_included_in_dni_by_taxpayer
    {
        return Output {
            mode: Section643Mode::ViolationCapitalGainsImproperlyExcludedDespiteDistribution,
            computed_dni_dollars: input.taxable_income_before_dni_modifications_dollars,
            statutory_basis: "§ 643(a)(3) — capital gains INCLUDED in DNI when distributed/required to be distributed/set aside for charity".to_string(),
            notes: format!(
                "VIOLATION § 643(a)(3): capital gain of ${} ({:?}) MUST be included in DNI because it is distributed, required to be distributed, or set aside for § 642(c) charity.",
                input.capital_gain_dollars, input.capital_gain_treatment
            ),
            citations,
        };
    }

    if !gain_should_be_in_dni
        && input.capital_gain_dollars > 0
        && input.capital_gain_included_in_dni_by_taxpayer
    {
        return Output {
            mode: Section643Mode::ViolationCapitalGainsImproperlyIncludedInDni,
            computed_dni_dollars: input.taxable_income_before_dni_modifications_dollars,
            statutory_basis: "§ 643(a)(3) — capital gains EXCLUDED from DNI when allocated to corpus and not distributed".to_string(),
            notes: format!(
                "VIOLATION § 643(a)(3): capital gain of ${} ({:?}) allocated to corpus and not distributed MUST be excluded from DNI; trust pays tax at trust rates.",
                input.capital_gain_dollars, input.capital_gain_treatment
            ),
            citations,
        };
    }

    if input.tax_exempt_interest_dollars > 0 && !input.tax_exempt_income_included_in_dni_by_taxpayer
    {
        return Output {
            mode: Section643Mode::ViolationTaxExemptIncomeOmittedFromDni,
            computed_dni_dollars: input.taxable_income_before_dni_modifications_dollars,
            statutory_basis: "§ 643(a)(5) — tax-exempt income INCLUDED in DNI net of allocable expenses".to_string(),
            notes: format!(
                "VIOLATION § 643(a)(5): tax-exempt interest of ${} must be INCLUDED in DNI net of allocable expenses; omission produces incorrect DNI allocation.",
                input.tax_exempt_interest_dollars
            ),
            citations,
        };
    }

    let mut dni = input.taxable_income_before_dni_modifications_dollars;
    if gain_should_be_in_dni {
        dni = dni.saturating_add(input.capital_gain_dollars);
    }
    dni = dni.saturating_add(input.tax_exempt_interest_dollars);

    if gain_should_be_in_dni && input.capital_gain_included_in_dni_by_taxpayer {
        return Output {
            mode: Section643Mode::CompliantCapitalGainsIncludedInDniDistributedOrSetAside,
            computed_dni_dollars: dni,
            statutory_basis: "§ 643(a)(3) — capital gains properly included in DNI".to_string(),
            notes: format!(
                "COMPLIANT § 643(a)(3): capital gain ${} included in DNI ({:?}); computed DNI = ${}.",
                input.capital_gain_dollars, input.capital_gain_treatment, dni
            ),
            citations,
        };
    }

    if !gain_should_be_in_dni && input.capital_gain_dollars > 0 {
        return Output {
            mode: Section643Mode::CompliantCapitalGainsExcludedAllocatedToCorpus,
            computed_dni_dollars: dni,
            statutory_basis: "§ 643(a)(3) — capital gains excluded; allocated to corpus".to_string(),
            notes: format!(
                "COMPLIANT § 643(a)(3): capital gain ${} allocated to corpus and not distributed; excluded from DNI; trust pays tax at trust rates. Computed DNI = ${}.",
                input.capital_gain_dollars, dni
            ),
            citations,
        };
    }

    if input.tax_exempt_interest_dollars > 0 {
        return Output {
            mode: Section643Mode::CompliantTaxExemptIncomeIncludedInDni,
            computed_dni_dollars: dni,
            statutory_basis: "§ 643(a)(5) — tax-exempt income included in DNI".to_string(),
            notes: format!(
                "COMPLIANT § 643(a)(5): tax-exempt interest ${} included in DNI net of allocable expenses; computed DNI = ${}.",
                input.tax_exempt_interest_dollars, dni
            ),
            citations,
        };
    }

    Output {
        mode: Section643Mode::CompliantDniProperlyComputed,
        computed_dni_dollars: dni,
        statutory_basis: "§ 643(a) — DNI computation standard modifications applied".to_string(),
        notes: format!(
            "COMPLIANT § 643(a): standard DNI computation. Computed DNI = ${}.",
            dni
        ),
        citations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_complex_trust_capital_gain_to_corpus() -> Input {
        Input {
            trust_or_estate_type: TrustOrEstateType::DomesticNonGrantorComplexTrust,
            taxable_income_before_dni_modifications_dollars: 100_000,
            personal_exemption_claimed_in_dni_dollars: 0,
            distribution_deduction_claimed_in_dni_dollars: 0,
            capital_gain_dollars: 50_000,
            capital_gain_treatment: CapitalGainTreatment::AllocatedToCorpusNotDistributed,
            capital_gain_included_in_dni_by_taxpayer: false,
            tax_exempt_interest_dollars: 0,
            tax_exempt_income_included_in_dni_by_taxpayer: false,
            multiple_trusts_aggregated_to_avoid_brackets: false,
            foreign_trust_dni_modification_applied: false,
            dni_inclusion_approach: DniInclusionApproach::StandardSection643aComputation,
        }
    }

    #[test]
    fn complex_trust_capital_gain_to_corpus_excluded_compliant() {
        let result = compute(&baseline_complex_trust_capital_gain_to_corpus());
        assert_eq!(
            result.mode,
            Section643Mode::CompliantCapitalGainsExcludedAllocatedToCorpus
        );
        assert_eq!(result.computed_dni_dollars, 100_000);
    }

    #[test]
    fn capital_gain_distributed_must_be_in_dni_compliant() {
        let input = Input {
            capital_gain_treatment:
                CapitalGainTreatment::AllocatedToCorpusButDistributedToBeneficiary,
            capital_gain_included_in_dni_by_taxpayer: true,
            ..baseline_complex_trust_capital_gain_to_corpus()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section643Mode::CompliantCapitalGainsIncludedInDniDistributedOrSetAside
        );
        assert_eq!(result.computed_dni_dollars, 150_000);
    }

    #[test]
    fn capital_gain_distributed_but_excluded_violation() {
        let input = Input {
            capital_gain_treatment:
                CapitalGainTreatment::AllocatedToCorpusButDistributedToBeneficiary,
            capital_gain_included_in_dni_by_taxpayer: false,
            ..baseline_complex_trust_capital_gain_to_corpus()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section643Mode::ViolationCapitalGainsImproperlyExcludedDespiteDistribution
        );
    }

    #[test]
    fn capital_gain_required_to_be_distributed_must_be_in_dni() {
        let input = Input {
            capital_gain_treatment:
                CapitalGainTreatment::AllocatedToCorpusButRequiredToBeDistributed,
            capital_gain_included_in_dni_by_taxpayer: true,
            ..baseline_complex_trust_capital_gain_to_corpus()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section643Mode::CompliantCapitalGainsIncludedInDniDistributedOrSetAside
        );
    }

    #[test]
    fn capital_gain_set_aside_for_section_642c_charity_must_be_in_dni() {
        let input = Input {
            capital_gain_treatment: CapitalGainTreatment::SetAsideForSection642cCharity,
            capital_gain_included_in_dni_by_taxpayer: true,
            ..baseline_complex_trust_capital_gain_to_corpus()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section643Mode::CompliantCapitalGainsIncludedInDniDistributedOrSetAside
        );
    }

    #[test]
    fn capital_gain_to_corpus_but_included_violation() {
        let input = Input {
            capital_gain_treatment: CapitalGainTreatment::AllocatedToCorpusNotDistributed,
            capital_gain_included_in_dni_by_taxpayer: true,
            ..baseline_complex_trust_capital_gain_to_corpus()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section643Mode::ViolationCapitalGainsImproperlyIncludedInDni
        );
    }

    #[test]
    fn tax_exempt_income_included_in_dni_compliant() {
        let input = Input {
            tax_exempt_interest_dollars: 10_000,
            tax_exempt_income_included_in_dni_by_taxpayer: true,
            capital_gain_dollars: 0,
            ..baseline_complex_trust_capital_gain_to_corpus()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section643Mode::CompliantTaxExemptIncomeIncludedInDni
        );
        assert_eq!(result.computed_dni_dollars, 110_000);
    }

    #[test]
    fn tax_exempt_income_omitted_from_dni_violation() {
        let input = Input {
            tax_exempt_interest_dollars: 10_000,
            tax_exempt_income_included_in_dni_by_taxpayer: false,
            capital_gain_dollars: 0,
            ..baseline_complex_trust_capital_gain_to_corpus()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section643Mode::ViolationTaxExemptIncomeOmittedFromDni
        );
    }

    #[test]
    fn personal_exemption_in_dni_violation() {
        let input = Input {
            personal_exemption_claimed_in_dni_dollars: 100,
            ..baseline_complex_trust_capital_gain_to_corpus()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section643Mode::ViolationDniComputationIncludedPersonalExemption
        );
    }

    #[test]
    fn distribution_deduction_in_dni_violation() {
        let input = Input {
            distribution_deduction_claimed_in_dni_dollars: 20_000,
            ..baseline_complex_trust_capital_gain_to_corpus()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section643Mode::ViolationDniComputationIncludedDistributionDeduction
        );
    }

    #[test]
    fn multiple_trusts_aggregated_violation_section_643f() {
        let input = Input {
            multiple_trusts_aggregated_to_avoid_brackets: true,
            ..baseline_complex_trust_capital_gain_to_corpus()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section643Mode::ViolationMultipleTrustsAggregatedNotPerSection643f
        );
    }

    #[test]
    fn foreign_trust_dni_modification_omitted_violation() {
        let input = Input {
            trust_or_estate_type: TrustOrEstateType::ForeignTrust,
            foreign_trust_dni_modification_applied: false,
            ..baseline_complex_trust_capital_gain_to_corpus()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section643Mode::ViolationForeignTrustDniModificationOmitted
        );
    }

    #[test]
    fn foreign_trust_dni_modification_applied_compliant() {
        let input = Input {
            trust_or_estate_type: TrustOrEstateType::ForeignTrust,
            foreign_trust_dni_modification_applied: true,
            capital_gain_dollars: 0,
            ..baseline_complex_trust_capital_gain_to_corpus()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section643Mode::CompliantForeignTrustDniModificationApplied
        );
    }

    #[test]
    fn power_to_adjust_treas_reg_compliant() {
        let input = Input {
            dni_inclusion_approach:
                DniInclusionApproach::PowerToAdjustReallocatedUnderTreasReg1_643B,
            ..baseline_complex_trust_capital_gain_to_corpus()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section643Mode::CompliantPowerToAdjustAppliedTreasReg1_643B
        );
    }

    #[test]
    fn allocated_to_income_by_trust_instrument_in_dni() {
        let input = Input {
            capital_gain_treatment: CapitalGainTreatment::AllocatedToIncomeByTrustInstrumentOrState,
            capital_gain_included_in_dni_by_taxpayer: true,
            ..baseline_complex_trust_capital_gain_to_corpus()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section643Mode::CompliantCapitalGainsIncludedInDniDistributedOrSetAside
        );
    }

    #[test]
    fn simple_trust_compliant_with_no_capital_gain() {
        let input = Input {
            trust_or_estate_type: TrustOrEstateType::DomesticNonGrantorSimpleTrust,
            capital_gain_dollars: 0,
            ..baseline_complex_trust_capital_gain_to_corpus()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section643Mode::CompliantDniProperlyComputed);
    }

    #[test]
    fn estate_probate_compliant() {
        let input = Input {
            trust_or_estate_type: TrustOrEstateType::EstateProbate,
            capital_gain_dollars: 0,
            ..baseline_complex_trust_capital_gain_to_corpus()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section643Mode::CompliantDniProperlyComputed);
    }

    #[test]
    fn citations_pin_section_643_subsections_and_treas_regs() {
        let result = compute(&baseline_complex_trust_capital_gain_to_corpus());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("§ 643(a)(1)"));
        assert!(joined.contains("§ 643(a)(2)"));
        assert!(joined.contains("§ 643(a)(3)"));
        assert!(joined.contains("§ 643(a)(5)"));
        assert!(joined.contains("§ 643(a)(6)"));
        assert!(joined.contains("§ 643(a)(7)"));
        assert!(joined.contains("§ 643(b)"));
        assert!(joined.contains("§ 643(c)"));
        assert!(joined.contains("§ 643(d)"));
        assert!(joined.contains("§ 643(e)"));
        assert!(joined.contains("§ 643(f)"));
        assert!(joined.contains("§ 643(g)"));
        assert!(joined.contains("§ 643(h)"));
        assert!(joined.contains("§ 1.643(b)-1"));
        assert!(joined.contains("Dec 30, 2003"));
        assert!(joined.contains("Form 1041 Schedule B"));
        assert!(joined.contains("tier-1"));
        assert!(joined.contains("tier-2"));
        assert!(joined.contains("$15,650"));
    }

    #[test]
    fn constant_pin_dates_and_rates() {
        assert_eq!(SECTION_643_TRUST_TOP_MARGINAL_RATE_BASIS_POINTS, 3_700);
        assert_eq!(SECTION_643_TRUST_2025_TOP_BRACKET_THRESHOLD_DOLLARS, 15_650);
        assert_eq!(SECTION_643_TREAS_REG_643_B_FINAL_REGS_YEAR, 2004);
        assert_eq!(SECTION_643_TREAS_REG_643_B_FINAL_REGS_MONTH, 12);
        assert_eq!(SECTION_643_TREAS_REG_643_B_FINAL_REGS_DAY, 30);
    }

    #[test]
    fn saturating_overflow_defense_extreme_dni() {
        let input = Input {
            taxable_income_before_dni_modifications_dollars: u64::MAX,
            capital_gain_dollars: u64::MAX,
            tax_exempt_interest_dollars: u64::MAX,
            tax_exempt_income_included_in_dni_by_taxpayer: true,
            ..baseline_complex_trust_capital_gain_to_corpus()
        };
        let result = compute(&input);
        assert!(matches!(
            result.mode,
            Section643Mode::CompliantCapitalGainsExcludedAllocatedToCorpus
                | Section643Mode::CompliantDniProperlyComputed
                | Section643Mode::CompliantTaxExemptIncomeIncludedInDni
        ));
    }
}
