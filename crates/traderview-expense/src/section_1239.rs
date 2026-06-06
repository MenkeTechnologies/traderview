//! IRC § 1239 — Gain from Sale of Depreciable Property Between
//! Certain Related Taxpayers.
//!
//! Pure-compute recharacterization rule. When a taxpayer sells or
//! exchanges property to a RELATED PERSON, and the property is —
//! IN THE HANDS OF THE TRANSFEREE — of a character subject to the
//! depreciation allowance of § 167, ALL gain recognized by the
//! transferor is treated as ORDINARY INCOME (not capital gain, not
//! § 1231 gain). The statute exists to defeat the tax arbitrage of
//! repeatedly selling depreciable property between related parties
//! to step up basis and re-depreciate while paying only capital-
//! gains rates on the inter-party transfer.
//!
//! Statute (verbatim mapping):
//! - § 1239(a) — GENERAL RULE: in the case of a sale or exchange
//!   of property, directly or indirectly, between related persons,
//!   any gain recognized to the transferor shall be treated as
//!   ordinary income if such property is, in the hands of the
//!   transferee, of a character which is subject to the allowance
//!   for depreciation provided in § 167.
//! - § 1239(b) — RELATED PERSONS: (1) a person and all entities
//!   which are CONTROLLED ENTITIES with respect to such person; (2)
//!   a taxpayer and any TRUST in which such taxpayer (or his spouse)
//!   is a BENEFICIARY, unless such beneficiary's interest in the
//!   trust is a REMOTE CONTINGENT INTEREST within the meaning of
//!   § 318(a)(3)(B)(i); (3) except in the case of a sale or exchange
//!   in satisfaction of a pecuniary bequest, an EXECUTOR of an
//!   estate and a BENEFICIARY of such estate.
//! - § 1239(c)(1) — CONTROLLED ENTITY defined: (A) a corporation
//!   more than 50 percent of the value of the outstanding stock of
//!   which is owned (directly or indirectly) by or for the person;
//!   (B) a partnership more than 50 percent of the capital interest
//!   or profits interest in which is owned (directly or indirectly)
//!   by or for the person; (C) any entity which is a related person
//!   to such person under paragraph (3), (10), (11), or (12) of
//!   § 267(b).
//! - § 1239(c)(2) — CONSTRUCTIVE OWNERSHIP: § 267(c) constructive
//!   ownership rules apply for purposes of determining whether a
//!   person owns more than 50 percent (treats family members,
//!   partner-of-partner, etc. as constructively owning).
//! - § 1239(d) — EMPLOYER/EMPLOYEE / RELATED PENSION PLAN: an
//!   employee benefit plan covering an owner-employee is treated
//!   as a related person to the employer under § 1239(b)(1).
//! - Treas. Reg. § 1.1239-1 — sale or exchange of depreciable
//!   property between certain related taxpayers after Oct 4, 1976.
//! - Note: depreciable-character determination is made AT TIME OF
//!   SALE AND IN THE HANDS OF THE TRANSFEREE, regardless of
//!   whether the property was depreciable to the transferor.
//!
//! Web research (verified 2026-06-03):
//! - Cornell LII § 1239 full statutory text confirms structure.
//! - NCBarBlog: § 1239 discourages related taxpayers from taking
//!   advantage of arbitrage between depreciation deductions that
//!   offset ordinary income and more favorable capital gains rates.
//! - eCFR § 1.1239-1 confirms 50 %-by-value (not 50 %-by-count)
//!   threshold and § 267(c) constructive ownership.
//!
//! Trader-landlord critical for: rental real estate sales to
//! family members, controlled S-corp, controlled LLC, family LP;
//! depreciable equipment sales to trading-shop entities; building
//! sales to controlled REIT; sales to grantor trust where trader
//! or spouse is beneficiary; estate-administration depreciable
//! property distributions to non-pecuniary-bequest beneficiaries.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const SECTION_1239_CONTROLLED_ENTITY_THRESHOLD_BASIS_POINTS: u64 = 5_000;
pub const SECTION_1239_BASIS_POINT_DENOMINATOR: u64 = 10_000;
pub const SECTION_1239_TREAS_REG_EFFECTIVE_DATE_YEAR: u32 = 1976;
pub const SECTION_1239_TREAS_REG_EFFECTIVE_DATE_MONTH: u32 = 10;
pub const SECTION_1239_TREAS_REG_EFFECTIVE_DATE_DAY: u32 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RelatedPersonCategory {
    NotRelated,
    ControlledCorporationOver50PctByValue,
    ControlledPartnershipOver50PctCapitalOrProfits,
    TrustBeneficiaryNonRemoteContingent,
    TrustBeneficiaryRemoteContingent,
    ExecutorBeneficiaryNonPecuniaryBequest,
    ExecutorBeneficiaryPecuniaryBequestSatisfaction,
    Section267BRelatedEntity,
    EmployerControlledPensionPlan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyDepreciableToTransferee {
    DepreciableUnderSection167InTransfereeHands,
    NotDepreciableInTransfereeHands,
    AmortizableUnderSection197InTransfereeHands,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GainCharacter {
    CapitalLongTerm,
    CapitalShortTerm,
    Section1231Gain,
    Section1245OrdinaryRecapture,
    Section1250OrdinaryRecapture,
    OrdinaryOther,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section1239Mode {
    NotApplicableNoGainRecognized,
    NotApplicableUnrelatedParties,
    NotApplicablePropertyNotDepreciableToTransferee,
    CompliantGainRecharacterizedAsOrdinaryUnderSection1239,
    CompliantPropertyAlreadyOrdinaryUnderSection1245Or1250,
    CompliantTrustBeneficiaryRemoteContingentNotRelated,
    CompliantPecuniaryBequestSatisfactionNotRelated,
    ViolationGainReportedAsCapitalDespiteSection1239,
    ViolationConstructiveOwnershipNotAppliedToSection267C,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub gain_recognized_cents: u64,
    pub related_person_category: RelatedPersonCategory,
    pub direct_ownership_percentage_basis_points: u64,
    pub constructive_ownership_percentage_basis_points: u64,
    pub property_depreciable_to_transferee: PropertyDepreciableToTransferee,
    pub gain_character_as_reported: GainCharacter,
    pub constructive_ownership_test_performed: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section1239Mode,
    pub recharacterized_ordinary_income_cents: u64,
    pub correct_gain_character: GainCharacter,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type Section1239Input = Input;
pub type Section1239Output = Output;
pub type Section1239Result = Output;

fn category_makes_related(c: RelatedPersonCategory, effective_ownership_bp: u64) -> bool {
    match c {
        RelatedPersonCategory::NotRelated => false,
        RelatedPersonCategory::ControlledCorporationOver50PctByValue
        | RelatedPersonCategory::ControlledPartnershipOver50PctCapitalOrProfits => {
            effective_ownership_bp > SECTION_1239_CONTROLLED_ENTITY_THRESHOLD_BASIS_POINTS
        }
        RelatedPersonCategory::TrustBeneficiaryNonRemoteContingent => true,
        RelatedPersonCategory::TrustBeneficiaryRemoteContingent => false,
        RelatedPersonCategory::ExecutorBeneficiaryNonPecuniaryBequest => true,
        RelatedPersonCategory::ExecutorBeneficiaryPecuniaryBequestSatisfaction => false,
        RelatedPersonCategory::Section267BRelatedEntity => true,
        RelatedPersonCategory::EmployerControlledPensionPlan => true,
    }
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "26 U.S.C. § 1239(a) — gain on sale/exchange of depreciable-to-transferee property between related persons treated as ordinary income".to_string(),
        "26 U.S.C. § 1239(b)(1) — related persons: person and all controlled entities".to_string(),
        "26 U.S.C. § 1239(b)(2) — related persons: taxpayer + trust where taxpayer/spouse is beneficiary (unless remote contingent)".to_string(),
        "26 U.S.C. § 1239(b)(3) — related persons: executor + beneficiary (except pecuniary bequest satisfaction)".to_string(),
        "26 U.S.C. § 1239(c)(1)(A) — controlled corporation: > 50 % of value of outstanding stock owned directly or indirectly".to_string(),
        "26 U.S.C. § 1239(c)(1)(B) — controlled partnership: > 50 % capital interest or profits interest owned directly or indirectly".to_string(),
        "26 U.S.C. § 1239(c)(1)(C) — controlled entity: § 267(b)(3)/(10)/(11)/(12) related entities".to_string(),
        "26 U.S.C. § 1239(c)(2) — constructive ownership rules of § 267(c) apply".to_string(),
        "26 U.S.C. § 1239(d) — employer + employee benefit plan covering owner-employee is related person".to_string(),
        "Treas. Reg. § 1.1239-1 — sale or exchange of depreciable property between related taxpayers after October 4, 1976".to_string(),
        "Treas. Reg. § 1.1239-2 — same rule for sales on or before October 4, 1976".to_string(),
    ];

    if input.gain_recognized_cents == 0 {
        return Output {
            mode: Section1239Mode::NotApplicableNoGainRecognized,
            recharacterized_ordinary_income_cents: 0,
            correct_gain_character: input.gain_character_as_reported,
            notes: "No gain recognized; § 1239 inapplicable.".to_string(),
            citations,
        };
    }

    if matches!(
        input.gain_character_as_reported,
        GainCharacter::Section1245OrdinaryRecapture | GainCharacter::Section1250OrdinaryRecapture
    ) {
        return Output {
            mode: Section1239Mode::CompliantPropertyAlreadyOrdinaryUnderSection1245Or1250,
            recharacterized_ordinary_income_cents: input.gain_recognized_cents,
            correct_gain_character: GainCharacter::OrdinaryOther,
            notes: format!(
                "Gain of {} cents already characterized as ordinary recapture under § 1245/§ 1250; § 1239 recharacterization redundant but not in conflict.",
                input.gain_recognized_cents
            ),
            citations,
        };
    }

    match input.related_person_category {
        RelatedPersonCategory::TrustBeneficiaryRemoteContingent => {
            return Output {
                mode: Section1239Mode::CompliantTrustBeneficiaryRemoteContingentNotRelated,
                recharacterized_ordinary_income_cents: 0,
                correct_gain_character: input.gain_character_as_reported,
                notes: "Trust beneficiary's interest is remote contingent under § 318(a)(3)(B)(i); not a related person under § 1239(b)(2). No recharacterization.".to_string(),
                citations,
            };
        }
        RelatedPersonCategory::ExecutorBeneficiaryPecuniaryBequestSatisfaction => {
            return Output {
                mode: Section1239Mode::CompliantPecuniaryBequestSatisfactionNotRelated,
                recharacterized_ordinary_income_cents: 0,
                correct_gain_character: input.gain_character_as_reported,
                notes: "Sale or exchange is in satisfaction of a pecuniary bequest; § 1239(b)(3) carve-out applies. No recharacterization.".to_string(),
                citations,
            };
        }
        _ => {}
    }

    let effective_ownership_bp = input
        .direct_ownership_percentage_basis_points
        .max(input.constructive_ownership_percentage_basis_points);

    if matches!(
        input.related_person_category,
        RelatedPersonCategory::ControlledCorporationOver50PctByValue
            | RelatedPersonCategory::ControlledPartnershipOver50PctCapitalOrProfits
    ) && input.direct_ownership_percentage_basis_points
        <= SECTION_1239_CONTROLLED_ENTITY_THRESHOLD_BASIS_POINTS
        && input.constructive_ownership_percentage_basis_points
            > SECTION_1239_CONTROLLED_ENTITY_THRESHOLD_BASIS_POINTS
        && !input.constructive_ownership_test_performed
    {
        return Output {
            mode: Section1239Mode::ViolationConstructiveOwnershipNotAppliedToSection267C,
            recharacterized_ordinary_income_cents: input.gain_recognized_cents,
            correct_gain_character: GainCharacter::OrdinaryOther,
            notes: format!(
                "VIOLATION § 1239(c)(2): direct ownership = {} basis points ≤ 50 %; constructive ownership under § 267(c) = {} basis points > 50 %; but taxpayer failed to perform constructive-ownership test. § 1239 applies; gain of {} cents must be recharacterized as ordinary.",
                input.direct_ownership_percentage_basis_points,
                input.constructive_ownership_percentage_basis_points,
                input.gain_recognized_cents
            ),
            citations,
        };
    }

    let related = category_makes_related(input.related_person_category, effective_ownership_bp);

    if !related {
        return Output {
            mode: Section1239Mode::NotApplicableUnrelatedParties,
            recharacterized_ordinary_income_cents: 0,
            correct_gain_character: input.gain_character_as_reported,
            notes: format!(
                "Parties are not related under § 1239(b). Category = {:?}; effective ownership = {} basis points. § 1239 inapplicable; original gain character {:?} preserved.",
                input.related_person_category, effective_ownership_bp, input.gain_character_as_reported
            ),
            citations,
        };
    }

    if input.property_depreciable_to_transferee
        != PropertyDepreciableToTransferee::DepreciableUnderSection167InTransfereeHands
    {
        return Output {
            mode: Section1239Mode::NotApplicablePropertyNotDepreciableToTransferee,
            recharacterized_ordinary_income_cents: 0,
            correct_gain_character: input.gain_character_as_reported,
            notes: format!(
                "Property is {:?} in transferee's hands. § 1239 requires depreciability under § 167 in the hands of the TRANSFEREE (not the transferor). § 1239 inapplicable; original gain character {:?} preserved.",
                input.property_depreciable_to_transferee, input.gain_character_as_reported
            ),
            citations,
        };
    }

    let is_already_ordinary = matches!(
        input.gain_character_as_reported,
        GainCharacter::OrdinaryOther
            | GainCharacter::Section1245OrdinaryRecapture
            | GainCharacter::Section1250OrdinaryRecapture
    );

    if is_already_ordinary {
        return Output {
            mode: Section1239Mode::CompliantGainRecharacterizedAsOrdinaryUnderSection1239,
            recharacterized_ordinary_income_cents: input.gain_recognized_cents,
            correct_gain_character: GainCharacter::OrdinaryOther,
            notes: format!(
                "COMPLIANT § 1239: related parties + depreciable to transferee. Gain of {} cents already reported as ordinary; § 1239 reaches same result.",
                input.gain_recognized_cents
            ),
            citations,
        };
    }

    Output {
        mode: Section1239Mode::ViolationGainReportedAsCapitalDespiteSection1239,
        recharacterized_ordinary_income_cents: input.gain_recognized_cents,
        correct_gain_character: GainCharacter::OrdinaryOther,
        notes: format!(
            "VIOLATION § 1239(a): related-party sale of depreciable-to-transferee property; transferor reported gain of {} cents as {:?} but ALL gain must be recharacterized as ORDINARY INCOME. Effective ownership = {} basis points; related person category = {:?}.",
            input.gain_recognized_cents,
            input.gain_character_as_reported,
            effective_ownership_bp,
            input.related_person_category
        ),
        citations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_controlled_corp_capital_misreport() -> Input {
        Input {
            gain_recognized_cents: 5_000_000,
            related_person_category: RelatedPersonCategory::ControlledCorporationOver50PctByValue,
            direct_ownership_percentage_basis_points: 8_000,
            constructive_ownership_percentage_basis_points: 8_000,
            property_depreciable_to_transferee:
                PropertyDepreciableToTransferee::DepreciableUnderSection167InTransfereeHands,
            gain_character_as_reported: GainCharacter::CapitalLongTerm,
            constructive_ownership_test_performed: true,
        }
    }

    #[test]
    fn no_gain_recognized_not_applicable() {
        let mut input = baseline_controlled_corp_capital_misreport();
        input.gain_recognized_cents = 0;
        let result = compute(&input);
        assert_eq!(result.mode, Section1239Mode::NotApplicableNoGainRecognized);
    }

    #[test]
    fn controlled_corp_capital_gain_violation_recharacterized() {
        let result = compute(&baseline_controlled_corp_capital_misreport());
        assert_eq!(
            result.mode,
            Section1239Mode::ViolationGainReportedAsCapitalDespiteSection1239
        );
        assert_eq!(result.recharacterized_ordinary_income_cents, 5_000_000);
        assert_eq!(result.correct_gain_character, GainCharacter::OrdinaryOther);
    }

    #[test]
    fn controlled_partnership_section_1231_misreport_violation() {
        let input = Input {
            related_person_category:
                RelatedPersonCategory::ControlledPartnershipOver50PctCapitalOrProfits,
            gain_character_as_reported: GainCharacter::Section1231Gain,
            ..baseline_controlled_corp_capital_misreport()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section1239Mode::ViolationGainReportedAsCapitalDespiteSection1239
        );
        assert_eq!(result.correct_gain_character, GainCharacter::OrdinaryOther);
    }

    #[test]
    fn controlled_corp_exactly_50_pct_not_related() {
        let input = Input {
            direct_ownership_percentage_basis_points: 5_000,
            constructive_ownership_percentage_basis_points: 5_000,
            ..baseline_controlled_corp_capital_misreport()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1239Mode::NotApplicableUnrelatedParties);
        assert!(result.notes.contains("not related"));
    }

    #[test]
    fn controlled_corp_50_pct_plus_1bp_related() {
        let input = Input {
            direct_ownership_percentage_basis_points: 5_001,
            constructive_ownership_percentage_basis_points: 5_001,
            ..baseline_controlled_corp_capital_misreport()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section1239Mode::ViolationGainReportedAsCapitalDespiteSection1239
        );
    }

    #[test]
    fn constructive_ownership_not_tested_violation() {
        let input = Input {
            direct_ownership_percentage_basis_points: 3_000,
            constructive_ownership_percentage_basis_points: 7_000,
            constructive_ownership_test_performed: false,
            ..baseline_controlled_corp_capital_misreport()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section1239Mode::ViolationConstructiveOwnershipNotAppliedToSection267C
        );
        assert_eq!(result.recharacterized_ordinary_income_cents, 5_000_000);
    }

    #[test]
    fn property_not_depreciable_to_transferee_not_applicable() {
        let input = Input {
            property_depreciable_to_transferee:
                PropertyDepreciableToTransferee::NotDepreciableInTransfereeHands,
            ..baseline_controlled_corp_capital_misreport()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section1239Mode::NotApplicablePropertyNotDepreciableToTransferee
        );
    }

    #[test]
    fn property_amortizable_section_197_not_applicable() {
        let input = Input {
            property_depreciable_to_transferee:
                PropertyDepreciableToTransferee::AmortizableUnderSection197InTransfereeHands,
            ..baseline_controlled_corp_capital_misreport()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section1239Mode::NotApplicablePropertyNotDepreciableToTransferee
        );
    }

    #[test]
    fn unrelated_parties_not_applicable() {
        let input = Input {
            related_person_category: RelatedPersonCategory::NotRelated,
            direct_ownership_percentage_basis_points: 0,
            constructive_ownership_percentage_basis_points: 0,
            ..baseline_controlled_corp_capital_misreport()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1239Mode::NotApplicableUnrelatedParties);
    }

    #[test]
    fn trust_beneficiary_non_remote_related_violation() {
        let input = Input {
            related_person_category: RelatedPersonCategory::TrustBeneficiaryNonRemoteContingent,
            ..baseline_controlled_corp_capital_misreport()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section1239Mode::ViolationGainReportedAsCapitalDespiteSection1239
        );
    }

    #[test]
    fn trust_beneficiary_remote_contingent_not_related() {
        let input = Input {
            related_person_category: RelatedPersonCategory::TrustBeneficiaryRemoteContingent,
            ..baseline_controlled_corp_capital_misreport()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section1239Mode::CompliantTrustBeneficiaryRemoteContingentNotRelated
        );
        assert_eq!(result.recharacterized_ordinary_income_cents, 0);
    }

    #[test]
    fn executor_non_pecuniary_beneficiary_related_violation() {
        let input = Input {
            related_person_category: RelatedPersonCategory::ExecutorBeneficiaryNonPecuniaryBequest,
            ..baseline_controlled_corp_capital_misreport()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section1239Mode::ViolationGainReportedAsCapitalDespiteSection1239
        );
    }

    #[test]
    fn executor_pecuniary_bequest_satisfaction_not_related() {
        let input = Input {
            related_person_category:
                RelatedPersonCategory::ExecutorBeneficiaryPecuniaryBequestSatisfaction,
            ..baseline_controlled_corp_capital_misreport()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section1239Mode::CompliantPecuniaryBequestSatisfactionNotRelated
        );
        assert_eq!(result.recharacterized_ordinary_income_cents, 0);
    }

    #[test]
    fn employer_controlled_pension_plan_related_violation() {
        let input = Input {
            related_person_category: RelatedPersonCategory::EmployerControlledPensionPlan,
            ..baseline_controlled_corp_capital_misreport()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section1239Mode::ViolationGainReportedAsCapitalDespiteSection1239
        );
    }

    #[test]
    fn section_1245_ordinary_recapture_already_ordinary_compliant() {
        let input = Input {
            gain_character_as_reported: GainCharacter::Section1245OrdinaryRecapture,
            ..baseline_controlled_corp_capital_misreport()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section1239Mode::CompliantPropertyAlreadyOrdinaryUnderSection1245Or1250
        );
    }

    #[test]
    fn section_1250_ordinary_recapture_already_ordinary_compliant() {
        let input = Input {
            gain_character_as_reported: GainCharacter::Section1250OrdinaryRecapture,
            ..baseline_controlled_corp_capital_misreport()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section1239Mode::CompliantPropertyAlreadyOrdinaryUnderSection1245Or1250
        );
    }

    #[test]
    fn ordinary_other_compliant_no_recharacterization_needed() {
        let input = Input {
            gain_character_as_reported: GainCharacter::OrdinaryOther,
            ..baseline_controlled_corp_capital_misreport()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section1239Mode::CompliantGainRecharacterizedAsOrdinaryUnderSection1239
        );
    }

    #[test]
    fn capital_short_term_misreport_violation() {
        let input = Input {
            gain_character_as_reported: GainCharacter::CapitalShortTerm,
            ..baseline_controlled_corp_capital_misreport()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section1239Mode::ViolationGainReportedAsCapitalDespiteSection1239
        );
    }

    #[test]
    fn citations_pin_section_1239_subsections() {
        let result = compute(&baseline_controlled_corp_capital_misreport());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("§ 1239(a)"));
        assert!(joined.contains("§ 1239(b)(1)"));
        assert!(joined.contains("§ 1239(b)(2)"));
        assert!(joined.contains("§ 1239(b)(3)"));
        assert!(joined.contains("§ 1239(c)(1)(A)"));
        assert!(joined.contains("§ 1239(c)(1)(B)"));
        assert!(joined.contains("§ 1239(c)(1)(C)"));
        assert!(joined.contains("§ 1239(c)(2)"));
        assert!(joined.contains("§ 1239(d)"));
        assert!(joined.contains("§ 1.1239-1"));
        assert!(joined.contains("§ 1.1239-2"));
    }

    #[test]
    fn constant_pin_50_pct_threshold_and_1976_treas_reg_date() {
        assert_eq!(SECTION_1239_CONTROLLED_ENTITY_THRESHOLD_BASIS_POINTS, 5_000);
        assert_eq!(SECTION_1239_BASIS_POINT_DENOMINATOR, 10_000);
        assert_eq!(SECTION_1239_TREAS_REG_EFFECTIVE_DATE_YEAR, 1976);
        assert_eq!(SECTION_1239_TREAS_REG_EFFECTIVE_DATE_MONTH, 10);
        assert_eq!(SECTION_1239_TREAS_REG_EFFECTIVE_DATE_DAY, 4);
    }

    #[test]
    fn saturating_overflow_defense_extreme_gain() {
        let input = Input {
            gain_recognized_cents: u64::MAX,
            ..baseline_controlled_corp_capital_misreport()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section1239Mode::ViolationGainReportedAsCapitalDespiteSection1239
        );
        assert_eq!(result.recharacterized_ordinary_income_cents, u64::MAX);
    }
}
