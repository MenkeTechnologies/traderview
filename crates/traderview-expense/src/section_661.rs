//! IRC § 661 — Deduction for Estates and Trusts Accumulating
//! Income or Distributing Corpus (Complex Trust Distribution
//! Deduction).
//!
//! Pure-compute complex-trust distribution deduction computation.
//! Trader-critical because the deduction shifts taxable income
//! from the compressed-bracket trust (37 % at ~$15,650 for 2025)
//! to the beneficiary at potentially much lower individual
//! brackets. § 661 + § 662 together form the conduit principle:
//! trust receives a deduction; beneficiary includes corresponding
//! income with character preservation.
//!
//! Statute (verbatim mapping):
//! - § 661(a) — GENERAL RULE: a complex trust or estate is
//!   allowed a deduction equal to the sum of (1) any amount of
//!   INCOME REQUIRED TO BE DISTRIBUTED CURRENTLY (tier-1
//!   mandatory) and (2) any OTHER AMOUNTS PROPERLY PAID,
//!   CREDITED, OR REQUIRED TO BE DISTRIBUTED (tier-2
//!   discretionary). The total deduction is **LIMITED TO DNI**
//!   under § 661(c) cross-reference to § 643(a).
//! - § 661(b) — CHARACTER OF AMOUNTS: the amount allowed as a
//!   deduction under § 661(a) shall be treated as consisting of
//!   the SAME PROPORTION of each class of items entering into
//!   the computation of DNI as the total of each class bears to
//!   the total DNI. Character preservation — ordinary income,
//!   qualified dividends, LTCG, tax-exempt interest all flow
//!   proportionally to beneficiaries.
//! - § 661(c) — COORDINATION WITH § 642(c) CHARITABLE
//!   CONTRIBUTION DEDUCTION: § 661(a) deduction is reduced by the
//!   amount of § 642(c) charitable contribution deduction
//!   attributable to current-year DNI.
//! - § 662(a)(1) — TIER-1 BENEFICIARY INCLUSION: beneficiary
//!   includes amount of income required to be distributed,
//!   limited to DNI.
//! - § 662(a)(2) — TIER-2 BENEFICIARY INCLUSION: beneficiary
//!   includes other amounts properly distributed, limited to
//!   REMAINING DNI after tier-1.
//! - § 662(b) — CHARACTER: same proportional allocation as
//!   § 661(b).
//! - § 663(a) — EXCLUSIONS: gifts, bequests, and devises of
//!   specific sums or property under § 663(a)(1) are NOT
//!   distributions for § 661 / § 662 purposes; § 663(a)(2)
//!   capital gains allocated to corpus are NOT distributions.
//! - § 663(b) — 65-DAY RULE ELECTION: distributions made within
//!   first 65 days of next taxable year may be ELECTED to be
//!   treated as made on the last day of preceding taxable year.
//! - § 663(c) — SEPARATE SHARE RULE: substantially separate and
//!   independent shares of different beneficiaries of a SINGLE
//!   trust shall be treated as separate trusts for purposes of
//!   § 661 / § 662 / § 663(b) (DNI computation, distribution
//!   deduction, beneficiary inclusion, and 65-day election).
//! - Treas. Reg. § 1.661(c)-2 — separate share regulations.
//! - Treas. Reg. § 1.663(c)-1 to § 1.663(c)-5 — separate share
//!   detailed application.
//!
//! Web research (verified 2026-06-03):
//! - Cornell LII § 661 confirms statutory text.
//! - IRS Form 1041 Schedule B implements DNI ceiling computation
//!   for distribution deduction.
//! - IRS Form 1041 Instructions (2025) confirm tier-1 / tier-2
//!   reporting structure on Schedule K-1.
//! - Forvis Mazars US "65-Day Distribution Election" confirms
//!   § 663(b) eleventh-hour planning opportunity.
//! - Greenleaf Trust DNI explainer + Miami Law Heckerling
//!   Demystifying DNI confirm conduit principle.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const SECTION_663_B_65_DAY_RULE_DAYS: u32 = 65;
pub const SECTION_663_A_1_SPECIFIC_BEQUEST_EXCLUSION: bool = true;
pub const SECTION_663_A_2_CAPITAL_GAINS_ALLOCATED_TO_CORPUS_EXCLUSION: bool = true;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TrustOrEstateType {
    DomesticComplexTrust,
    ForeignComplexTrust,
    EstateProbate,
    DomesticSimpleTrustWrongStatute,
    DomesticGrantorTrustWrongStatute,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DistributionScenario {
    NoDistributionMade,
    Tier1MandatoryOnly,
    Tier1AndTier2Mixed,
    DniNotFullyDistributed,
    MultipleBeneficiariesSeparateShares,
    SpecificBequestSection663a1,
    CapitalGainsAllocatedToCorpusSection663a2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DniCharacterClass {
    OrdinaryIncomeOnly,
    OrdinaryAndQualifiedDividendsMixed,
    OrdinaryAndLongTermCapitalGainMixed,
    OrdinaryAndTaxExemptInterestMixed,
    FourClassDniMixed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section661Mode {
    NotApplicableNoDistribution,
    NotApplicableSimpleTrustUseSection651Instead,
    NotApplicableGrantorTrustPassThrough,
    CompliantTier1MandatoryDistributionsWithinDni,
    CompliantTier2DiscretionaryDistributionsWithinDni,
    CompliantSeparateShareRuleAppliedTreasReg1_663C,
    Compliant65DayRuleElectionMadeSection663b,
    CompliantCharacterProportionatelyAllocatedSection661b,
    CompliantSection663a1SpecificBequestExcluded,
    CompliantSection663a2CapitalGainsCorpusExcluded,
    ViolationDistributionDeductionExceedsDniCeiling,
    ViolationSeparateShareRuleNotAppliedDespiteMultipleBeneficiaries,
    ViolationTier1Tier2PriorityIgnored,
    ViolationCharacterNotProportionatelyAllocated,
    ViolationSection642cCharitableCoordinationOmitted,
    Violation65DayRuleElectionDistributionAfter65Days,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub trust_or_estate_type: TrustOrEstateType,
    pub distribution_scenario: DistributionScenario,
    pub dni_dollars: u64,
    pub tier_1_mandatory_distributions_dollars: u64,
    pub tier_2_discretionary_distributions_dollars: u64,
    pub section_642c_charitable_deduction_attributable_to_dni_dollars: u64,
    pub multiple_beneficiaries_with_separate_shares: bool,
    pub separate_share_rule_applied: bool,
    pub character_proportionately_allocated_per_section_661b: bool,
    pub section_663b_65_day_rule_elected: bool,
    pub days_distribution_after_year_end: u32,
    pub dni_character_class: DniCharacterClass,
    pub tier_priority_respected: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section661Mode,
    pub distribution_deduction_dollars: u64,
    pub tier_1_allowed_dollars: u64,
    pub tier_2_allowed_dollars: u64,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type Section661Input = Input;
pub type Section661Output = Output;
pub type Section661Result = Output;

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "26 U.S.C. § 661(a)(1) — tier-1 income required to be distributed currently (mandatory)".to_string(),
        "26 U.S.C. § 661(a)(2) — tier-2 other amounts properly paid, credited, or required to be distributed (discretionary)".to_string(),
        "26 U.S.C. § 661(b) — character preservation: distribution deduction consists of same proportion of each DNI class as that class bears to total DNI".to_string(),
        "26 U.S.C. § 661(c) — coordination with § 642(c) charitable contribution deduction".to_string(),
        "26 U.S.C. § 662(a)(1) — tier-1 beneficiary inclusion (mandatory distributions limited to DNI)".to_string(),
        "26 U.S.C. § 662(a)(2) — tier-2 beneficiary inclusion (discretionary distributions limited to REMAINING DNI after tier-1)".to_string(),
        "26 U.S.C. § 662(b) — beneficiary character allocation parallels § 661(b)".to_string(),
        "26 U.S.C. § 663(a)(1) — specific sums or property gifts/bequests/devises NOT distributions for § 661/§ 662 purposes".to_string(),
        "26 U.S.C. § 663(a)(2) — capital gains allocated to corpus NOT distributions".to_string(),
        "26 U.S.C. § 663(b) — 65-day rule election: distributions within first 65 days of next taxable year may be elected as made on last day of prior year".to_string(),
        "26 U.S.C. § 663(c) — separate share rule: substantially separate and independent shares treated as separate trusts for § 661 / § 662 / § 663(b)".to_string(),
        "Treas. Reg. § 1.661(c)-2 — separate share regulations".to_string(),
        "Treas. Reg. § 1.663(c)-1 to § 1.663(c)-5 — separate share detailed application".to_string(),
        "26 U.S.C. § 643(a) — DNI ceiling on distribution deduction".to_string(),
        "IRS Form 1041 Schedule B — distribution deduction worksheet; Schedule K-1 — beneficiary character allocation".to_string(),
    ];

    match input.trust_or_estate_type {
        TrustOrEstateType::DomesticSimpleTrustWrongStatute => {
            return Output {
                mode: Section661Mode::NotApplicableSimpleTrustUseSection651Instead,
                distribution_deduction_dollars: 0,
                tier_1_allowed_dollars: 0,
                tier_2_allowed_dollars: 0,
                statutory_basis: "Simple trust uses § 651 distribution deduction".to_string(),
                notes: "Simple trust required to distribute all current income; uses § 651 distribution deduction, not § 661.".to_string(),
                citations,
            };
        }
        TrustOrEstateType::DomesticGrantorTrustWrongStatute => {
            return Output {
                mode: Section661Mode::NotApplicableGrantorTrustPassThrough,
                distribution_deduction_dollars: 0,
                tier_1_allowed_dollars: 0,
                tier_2_allowed_dollars: 0,
                statutory_basis:
                    "Subpart E grantor trust passthrough; no § 661 deduction at trust level"
                        .to_string(),
                notes:
                    "Grantor trust passthrough; all income flows to grantor; § 661 inapplicable."
                        .to_string(),
                citations,
            };
        }
        _ => {}
    }

    if input.distribution_scenario == DistributionScenario::NoDistributionMade {
        return Output {
            mode: Section661Mode::NotApplicableNoDistribution,
            distribution_deduction_dollars: 0,
            tier_1_allowed_dollars: 0,
            tier_2_allowed_dollars: 0,
            statutory_basis: "No distribution made; § 661 inapplicable".to_string(),
            notes: "No distribution made during taxable year; complex trust retains all income at trust level (taxed at 37 % top bracket above $15,650 for 2025).".to_string(),
            citations,
        };
    }

    if input.distribution_scenario == DistributionScenario::SpecificBequestSection663a1 {
        return Output {
            mode: Section661Mode::CompliantSection663a1SpecificBequestExcluded,
            distribution_deduction_dollars: 0,
            tier_1_allowed_dollars: 0,
            tier_2_allowed_dollars: 0,
            statutory_basis: "§ 663(a)(1) — specific bequest excluded from § 661 distribution deduction".to_string(),
            notes: "COMPLIANT § 663(a)(1): specific sum or property bequest is NOT a distribution for § 661 / § 662 purposes; no distribution deduction; no beneficiary income inclusion.".to_string(),
            citations,
        };
    }

    if input.distribution_scenario
        == DistributionScenario::CapitalGainsAllocatedToCorpusSection663a2
    {
        return Output {
            mode: Section661Mode::CompliantSection663a2CapitalGainsCorpusExcluded,
            distribution_deduction_dollars: 0,
            tier_1_allowed_dollars: 0,
            tier_2_allowed_dollars: 0,
            statutory_basis: "§ 663(a)(2) — capital gains allocated to corpus NOT distributions".to_string(),
            notes: "COMPLIANT § 663(a)(2): capital gains allocated to corpus and not distributed are not § 661 distributions; trust pays tax at trust rates.".to_string(),
            citations,
        };
    }

    if input.multiple_beneficiaries_with_separate_shares && !input.separate_share_rule_applied {
        return Output {
            mode: Section661Mode::ViolationSeparateShareRuleNotAppliedDespiteMultipleBeneficiaries,
            distribution_deduction_dollars: 0,
            tier_1_allowed_dollars: 0,
            tier_2_allowed_dollars: 0,
            statutory_basis: "§ 663(c) + Treas. Reg. § 1.663(c)-2 — separate share rule required".to_string(),
            notes: "VIOLATION § 663(c): multiple beneficiaries with substantially separate and independent shares require separate share rule application; failure to apply may cause one beneficiary's distribution to inappropriately carry out another beneficiary's DNI.".to_string(),
            citations,
        };
    }

    let total_distributions = input
        .tier_1_mandatory_distributions_dollars
        .saturating_add(input.tier_2_discretionary_distributions_dollars);
    let dni_after_charitable = input
        .dni_dollars
        .saturating_sub(input.section_642c_charitable_deduction_attributable_to_dni_dollars);

    if total_distributions > input.dni_dollars
        && input.tier_2_discretionary_distributions_dollars > 0
        && !input.tier_priority_respected
    {
        return Output {
            mode: Section661Mode::ViolationTier1Tier2PriorityIgnored,
            distribution_deduction_dollars: dni_after_charitable,
            tier_1_allowed_dollars: 0,
            tier_2_allowed_dollars: 0,
            statutory_basis: "§ 662(a)(1) + § 662(a)(2) — tier-1 mandatory has priority over tier-2 discretionary".to_string(),
            notes: format!(
                "VIOLATION: tier-1 mandatory distributions of ${} must be deducted FIRST, with tier-2 limited to remaining DNI after tier-1; taxpayer ignored priority ordering.",
                input.tier_1_mandatory_distributions_dollars
            ),
            citations,
        };
    }

    if input.tier_2_discretionary_distributions_dollars > 0
        && input.section_663b_65_day_rule_elected
        && input.days_distribution_after_year_end > SECTION_663_B_65_DAY_RULE_DAYS
    {
        return Output {
            mode: Section661Mode::Violation65DayRuleElectionDistributionAfter65Days,
            distribution_deduction_dollars: 0,
            tier_1_allowed_dollars: 0,
            tier_2_allowed_dollars: 0,
            statutory_basis: "§ 663(b) — 65-day rule election limited to distributions within 65 days of year end".to_string(),
            notes: format!(
                "VIOLATION § 663(b): 65-day rule elected but distribution made {} days after year end (> 65-day window).",
                input.days_distribution_after_year_end
            ),
            citations,
        };
    }

    if dni_after_charitable < total_distributions {
        let allowed_tier_1 = input
            .tier_1_mandatory_distributions_dollars
            .min(dni_after_charitable);
        let allowed_tier_2 = dni_after_charitable.saturating_sub(allowed_tier_1);
        return Output {
            mode: Section661Mode::ViolationDistributionDeductionExceedsDniCeiling,
            distribution_deduction_dollars: dni_after_charitable,
            tier_1_allowed_dollars: allowed_tier_1,
            tier_2_allowed_dollars: allowed_tier_2,
            statutory_basis: "§ 643(a) + § 661(c) — distribution deduction CAPPED at DNI (less § 642(c) charitable allocable to DNI)".to_string(),
            notes: format!(
                "VIOLATION § 643(a)/§ 661(c): total distributions ${} exceed DNI ceiling ${} (after § 642(c) charitable ${}); deduction capped; excess of ${} is tax-free return of corpus to beneficiary.",
                total_distributions,
                dni_after_charitable,
                input.section_642c_charitable_deduction_attributable_to_dni_dollars,
                total_distributions.saturating_sub(dni_after_charitable)
            ),
            citations,
        };
    }

    if !input.character_proportionately_allocated_per_section_661b {
        return Output {
            mode: Section661Mode::ViolationCharacterNotProportionatelyAllocated,
            distribution_deduction_dollars: total_distributions,
            tier_1_allowed_dollars: input.tier_1_mandatory_distributions_dollars,
            tier_2_allowed_dollars: input.tier_2_discretionary_distributions_dollars,
            statutory_basis: "§ 661(b) + § 662(b) — character allocation must be proportional to DNI class composition".to_string(),
            notes: format!(
                "VIOLATION § 661(b)/§ 662(b): distribution deduction not proportionately allocated across DNI character classes (DNI class = {:?}); each class must flow to beneficiary in proportion to total DNI.",
                input.dni_character_class
            ),
            citations,
        };
    }

    if input.section_663b_65_day_rule_elected {
        return Output {
            mode: Section661Mode::Compliant65DayRuleElectionMadeSection663b,
            distribution_deduction_dollars: total_distributions,
            tier_1_allowed_dollars: input.tier_1_mandatory_distributions_dollars,
            tier_2_allowed_dollars: input.tier_2_discretionary_distributions_dollars,
            statutory_basis: "§ 663(b) — 65-day rule election applied".to_string(),
            notes: format!(
                "COMPLIANT § 663(b): 65-day rule elected; distribution made {} days after year end (≤ 65) deemed made on last day of preceding taxable year. Deduction = ${}.",
                input.days_distribution_after_year_end, total_distributions
            ),
            citations,
        };
    }

    if input.separate_share_rule_applied {
        return Output {
            mode: Section661Mode::CompliantSeparateShareRuleAppliedTreasReg1_663C,
            distribution_deduction_dollars: total_distributions,
            tier_1_allowed_dollars: input.tier_1_mandatory_distributions_dollars,
            tier_2_allowed_dollars: input.tier_2_discretionary_distributions_dollars,
            statutory_basis: "§ 663(c) + Treas. Reg. § 1.663(c) — separate share rule applied".to_string(),
            notes: format!(
                "COMPLIANT § 663(c): separate share rule applied; each beneficiary's substantially separate and independent share treated as separate trust for DNI ceiling. Total deduction = ${}.",
                total_distributions
            ),
            citations,
        };
    }

    if input.tier_2_discretionary_distributions_dollars > 0 {
        Output {
            mode: Section661Mode::CompliantTier2DiscretionaryDistributionsWithinDni,
            distribution_deduction_dollars: total_distributions,
            tier_1_allowed_dollars: input.tier_1_mandatory_distributions_dollars,
            tier_2_allowed_dollars: input.tier_2_discretionary_distributions_dollars,
            statutory_basis: "§ 661(a)(2) — tier-2 discretionary distributions within DNI ceiling".to_string(),
            notes: format!(
                "COMPLIANT § 661(a)(2): tier-1 ${} + tier-2 ${} = total deduction ${} within DNI ${}; character proportional under § 661(b).",
                input.tier_1_mandatory_distributions_dollars,
                input.tier_2_discretionary_distributions_dollars,
                total_distributions,
                dni_after_charitable
            ),
            citations,
        }
    } else {
        Output {
            mode: Section661Mode::CompliantTier1MandatoryDistributionsWithinDni,
            distribution_deduction_dollars: total_distributions,
            tier_1_allowed_dollars: input.tier_1_mandatory_distributions_dollars,
            tier_2_allowed_dollars: 0,
            statutory_basis: "§ 661(a)(1) — tier-1 mandatory distributions within DNI ceiling".to_string(),
            notes: format!(
                "COMPLIANT § 661(a)(1): tier-1 mandatory distributions ${} within DNI ${}; character proportional under § 661(b).",
                input.tier_1_mandatory_distributions_dollars, dni_after_charitable
            ),
            citations,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_complex_trust_tier_1_compliant() -> Input {
        Input {
            trust_or_estate_type: TrustOrEstateType::DomesticComplexTrust,
            distribution_scenario: DistributionScenario::Tier1MandatoryOnly,
            dni_dollars: 100_000,
            tier_1_mandatory_distributions_dollars: 60_000,
            tier_2_discretionary_distributions_dollars: 0,
            section_642c_charitable_deduction_attributable_to_dni_dollars: 0,
            multiple_beneficiaries_with_separate_shares: false,
            separate_share_rule_applied: false,
            character_proportionately_allocated_per_section_661b: true,
            section_663b_65_day_rule_elected: false,
            days_distribution_after_year_end: 0,
            dni_character_class: DniCharacterClass::OrdinaryIncomeOnly,
            tier_priority_respected: true,
        }
    }

    #[test]
    fn simple_trust_wrong_statute_not_applicable() {
        let input = Input {
            trust_or_estate_type: TrustOrEstateType::DomesticSimpleTrustWrongStatute,
            ..baseline_complex_trust_tier_1_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section661Mode::NotApplicableSimpleTrustUseSection651Instead
        );
    }

    #[test]
    fn grantor_trust_passthrough_not_applicable() {
        let input = Input {
            trust_or_estate_type: TrustOrEstateType::DomesticGrantorTrustWrongStatute,
            ..baseline_complex_trust_tier_1_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section661Mode::NotApplicableGrantorTrustPassThrough
        );
    }

    #[test]
    fn no_distribution_not_applicable() {
        let input = Input {
            distribution_scenario: DistributionScenario::NoDistributionMade,
            ..baseline_complex_trust_tier_1_compliant()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section661Mode::NotApplicableNoDistribution);
    }

    #[test]
    fn tier_1_mandatory_within_dni_compliant() {
        let result = compute(&baseline_complex_trust_tier_1_compliant());
        assert_eq!(
            result.mode,
            Section661Mode::CompliantTier1MandatoryDistributionsWithinDni
        );
        assert_eq!(result.distribution_deduction_dollars, 60_000);
        assert_eq!(result.tier_1_allowed_dollars, 60_000);
    }

    #[test]
    fn tier_2_discretionary_within_dni_compliant() {
        let input = Input {
            distribution_scenario: DistributionScenario::Tier1AndTier2Mixed,
            tier_1_mandatory_distributions_dollars: 40_000,
            tier_2_discretionary_distributions_dollars: 30_000,
            ..baseline_complex_trust_tier_1_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section661Mode::CompliantTier2DiscretionaryDistributionsWithinDni
        );
        assert_eq!(result.distribution_deduction_dollars, 70_000);
    }

    #[test]
    fn distribution_exceeds_dni_violation_capped_at_dni() {
        let input = Input {
            distribution_scenario: DistributionScenario::Tier1AndTier2Mixed,
            tier_1_mandatory_distributions_dollars: 60_000,
            tier_2_discretionary_distributions_dollars: 80_000,
            ..baseline_complex_trust_tier_1_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section661Mode::ViolationDistributionDeductionExceedsDniCeiling
        );
        assert_eq!(result.distribution_deduction_dollars, 100_000);
        assert_eq!(result.tier_1_allowed_dollars, 60_000);
        assert_eq!(result.tier_2_allowed_dollars, 40_000);
    }

    #[test]
    fn separate_share_rule_applied_compliant() {
        let input = Input {
            distribution_scenario: DistributionScenario::MultipleBeneficiariesSeparateShares,
            multiple_beneficiaries_with_separate_shares: true,
            separate_share_rule_applied: true,
            ..baseline_complex_trust_tier_1_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section661Mode::CompliantSeparateShareRuleAppliedTreasReg1_663C
        );
    }

    #[test]
    fn separate_share_rule_not_applied_violation() {
        let input = Input {
            distribution_scenario: DistributionScenario::MultipleBeneficiariesSeparateShares,
            multiple_beneficiaries_with_separate_shares: true,
            separate_share_rule_applied: false,
            ..baseline_complex_trust_tier_1_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section661Mode::ViolationSeparateShareRuleNotAppliedDespiteMultipleBeneficiaries
        );
    }

    #[test]
    fn sixty_five_day_rule_election_within_window_compliant() {
        let input = Input {
            section_663b_65_day_rule_elected: true,
            days_distribution_after_year_end: 60,
            ..baseline_complex_trust_tier_1_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section661Mode::Compliant65DayRuleElectionMadeSection663b
        );
    }

    #[test]
    fn sixty_five_day_rule_at_exactly_65_days_compliant() {
        let input = Input {
            section_663b_65_day_rule_elected: true,
            days_distribution_after_year_end: 65,
            tier_2_discretionary_distributions_dollars: 10_000,
            distribution_scenario: DistributionScenario::Tier1AndTier2Mixed,
            ..baseline_complex_trust_tier_1_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section661Mode::Compliant65DayRuleElectionMadeSection663b
        );
    }

    #[test]
    fn sixty_five_day_rule_at_66_days_violation() {
        let input = Input {
            section_663b_65_day_rule_elected: true,
            days_distribution_after_year_end: 66,
            tier_2_discretionary_distributions_dollars: 10_000,
            distribution_scenario: DistributionScenario::Tier1AndTier2Mixed,
            ..baseline_complex_trust_tier_1_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section661Mode::Violation65DayRuleElectionDistributionAfter65Days
        );
    }

    #[test]
    fn specific_bequest_section_663_a_1_excluded() {
        let input = Input {
            distribution_scenario: DistributionScenario::SpecificBequestSection663a1,
            ..baseline_complex_trust_tier_1_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section661Mode::CompliantSection663a1SpecificBequestExcluded
        );
    }

    #[test]
    fn capital_gains_corpus_section_663_a_2_excluded() {
        let input = Input {
            distribution_scenario: DistributionScenario::CapitalGainsAllocatedToCorpusSection663a2,
            ..baseline_complex_trust_tier_1_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section661Mode::CompliantSection663a2CapitalGainsCorpusExcluded
        );
    }

    #[test]
    fn character_not_proportionately_allocated_violation() {
        let input = Input {
            character_proportionately_allocated_per_section_661b: false,
            dni_character_class: DniCharacterClass::FourClassDniMixed,
            ..baseline_complex_trust_tier_1_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section661Mode::ViolationCharacterNotProportionatelyAllocated
        );
    }

    #[test]
    fn tier_1_tier_2_priority_ignored_violation() {
        let input = Input {
            tier_1_mandatory_distributions_dollars: 50_000,
            tier_2_discretionary_distributions_dollars: 100_000,
            tier_priority_respected: false,
            distribution_scenario: DistributionScenario::Tier1AndTier2Mixed,
            ..baseline_complex_trust_tier_1_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section661Mode::ViolationTier1Tier2PriorityIgnored
        );
    }

    #[test]
    fn charitable_section_642c_reduces_dni_ceiling() {
        let input = Input {
            tier_1_mandatory_distributions_dollars: 70_000,
            section_642c_charitable_deduction_attributable_to_dni_dollars: 40_000,
            ..baseline_complex_trust_tier_1_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section661Mode::ViolationDistributionDeductionExceedsDniCeiling
        );
        assert_eq!(result.distribution_deduction_dollars, 60_000);
    }

    #[test]
    fn estate_probate_compliant() {
        let input = Input {
            trust_or_estate_type: TrustOrEstateType::EstateProbate,
            ..baseline_complex_trust_tier_1_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section661Mode::CompliantTier1MandatoryDistributionsWithinDni
        );
    }

    #[test]
    fn foreign_complex_trust_compliant() {
        let input = Input {
            trust_or_estate_type: TrustOrEstateType::ForeignComplexTrust,
            ..baseline_complex_trust_tier_1_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section661Mode::CompliantTier1MandatoryDistributionsWithinDni
        );
    }

    #[test]
    fn citations_pin_section_661_subsections_and_treas_regs() {
        let result = compute(&baseline_complex_trust_tier_1_compliant());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("§ 661(a)(1)"));
        assert!(joined.contains("§ 661(a)(2)"));
        assert!(joined.contains("§ 661(b)"));
        assert!(joined.contains("§ 661(c)"));
        assert!(joined.contains("§ 662(a)(1)"));
        assert!(joined.contains("§ 662(a)(2)"));
        assert!(joined.contains("§ 662(b)"));
        assert!(joined.contains("§ 663(a)(1)"));
        assert!(joined.contains("§ 663(a)(2)"));
        assert!(joined.contains("§ 663(b)"));
        assert!(joined.contains("§ 663(c)"));
        assert!(joined.contains("§ 1.661(c)-2"));
        assert!(joined.contains("§ 1.663(c)-1"));
        assert!(joined.contains("§ 1.663(c)-5"));
        assert!(joined.contains("§ 643(a)"));
        assert!(joined.contains("Form 1041 Schedule B"));
        assert!(joined.contains("Schedule K-1"));
    }

    #[test]
    fn constant_pin_65_day_rule() {
        assert_eq!(SECTION_663_B_65_DAY_RULE_DAYS, 65);
        assert!(SECTION_663_A_1_SPECIFIC_BEQUEST_EXCLUSION);
        assert!(SECTION_663_A_2_CAPITAL_GAINS_ALLOCATED_TO_CORPUS_EXCLUSION);
    }
}
