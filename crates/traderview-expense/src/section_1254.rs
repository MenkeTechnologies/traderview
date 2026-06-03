//! IRC § 1254 — Gain from Disposition of Interest in Oil, Gas,
//! Geothermal, or Other Mineral Properties (Natural Resource
//! Recapture).
//!
//! Pure-compute recharacterization rule. When a taxpayer disposes
//! of § 1254 property (any property under § 614 whose adjusted
//! basis includes § 611 depletion adjustments), gain recognized
//! is recharacterized as ORDINARY INCOME to the extent of the
//! "§ 1254 costs" previously deducted — intangible drilling and
//! development costs (IDC) under § 263(c), mine development costs
//! under § 616, mine exploration costs under § 617, and depletion
//! under § 611 that reduced the basis. Parallel to § 1245 (personal
//! property recapture) and § 1250 (real property recapture) for
//! natural resource property.
//!
//! Statute (verbatim mapping):
//! - § 1254(a)(1) — GENERAL RULE: any gain realized on the
//!   disposition of § 1254 property shall be treated as ordinary
//!   income to the extent of the lesser of (A) the aggregate
//!   amount of expenditures and deductions described in clauses
//!   (i) through (iii) of subparagraph (B) with respect to the
//!   property (the "§ 1254 costs"), OR (B) the excess of (i) the
//!   amount realized over (ii) the adjusted basis of the property.
//! - § 1254(a)(1)(B) clause-by-clause — POST-1986 PROPERTY:
//!   includes (i) IDC under § 263(c) and § 616 development; (ii)
//!   mine exploration costs under § 617; (iii) depletion under
//!   § 611 reducing adjusted basis. (For property placed in
//!   service before Jan 1, 1987, only IDCs under § 263(c) reduced
//!   by depletion that would have been allowable if IDCs had been
//!   capitalized.)
//! - § 1254(a)(3) — ELECTION FOR PARTNERSHIPS / TRUSTS: in case of
//!   a partnership or trust, application of § 1254(a)(1) at entity
//!   level + allocation to partners under § 1.1254-5 regulations.
//! - § 1254(b) — SPECIAL RULES: (1) carryover basis transferee
//!   treated as holding for periods transferor held; (2) certain
//!   § 351 transfers + § 721 partnership contributions tack basis.
//! - § 1254(c) — DEFINITION: § 1254 property = any property within
//!   § 614 whose adjusted basis includes adjustments for § 611
//!   depletion deductions.
//! - § 1254(d) — PARTIAL DISPOSITION: in the case of disposition
//!   of portion of § 1254 property (other than undivided interest),
//!   the entire amount of § 1254 costs treated as allocable to
//!   that portion to the extent of the gain to which § 1254(a)(1)
//!   applies.
//! - Treas. Reg. § 1.1254-1 — implementing regulations; final regs
//!   reorganized 1995 / updated 2008.
//! - Treas. Reg. § 1.1254-5 — special rules for partnerships and
//!   partners; allocation method for distributive share.
//! - **Nonproductive wells exception** under Treas. Reg.
//!   § 1.1254-2(b): § 1254 costs attributable to nonproductive
//!   wells NOT recapturable, except in certain limited-risk
//!   situations (taxpayer has agreement to reimburse drilling
//!   costs from operating partner regardless of well success).
//!
//! Web research (verified 2026-06-03):
//! - Cornell LII § 1254 + Treas. Reg. § 1.1254-1 confirm
//!   statutory text.
//! - IRS Form 4797 Instructions confirm § 1254 reporting on Part
//!   III line 28 for natural resource recapture.
//! - IRS Pub 5652 "Oil & Gas Audit Technique Guide" provides
//!   examples of recapture computation.
//! - Tax Notes IRC § 1254 confirms partial-disposition allocation
//!   rules.
//! - 26 CFR § 1.1254-5 confirms partnership allocation.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const SECTION_1254_POST_1986_EFFECTIVE_DATE_YEAR: u32 = 1987;
pub const SECTION_1254_POST_1986_EFFECTIVE_DATE_MONTH: u32 = 1;
pub const SECTION_1254_POST_1986_EFFECTIVE_DATE_DAY: u32 = 1;
pub const SECTION_1254_TREAS_REG_FINAL_REGS_REORGANIZED_YEAR: u32 = 1995;
pub const SECTION_1254_TREAS_REG_UPDATED_YEAR: u32 = 2008;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section1254PropertyCategory {
    PostJan1_1987PropertyAllRecapture,
    PreJan1_1987PropertyDepletionOffset,
    NonproductiveWellLimitedRiskExcluded,
    OverridingRoyaltyInterest,
    WorkingInterest,
    NetProfitsInterest,
    NotSection1254Property,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GainCharacterAsReported {
    CapitalLongTerm,
    CapitalShortTerm,
    Section1231Gain,
    OrdinaryOther,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DispositionType {
    EntireProperty,
    PartialDispositionNotUndividedInterest,
    UndividedInterest,
    CarryoverBasisSection351Or721,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section1254Mode {
    NotApplicableNoGainRecognized,
    NotApplicableNotSection1254Property,
    NotApplicableNonproductiveWellExclusion,
    CompliantGainRecharacterizedAsOrdinary,
    CompliantPartialDispositionAllocationApplied,
    CompliantPre1987PropertyDepletionOffsetApplied,
    CompliantCarryoverBasisTransfereeTreatedAsHoldingTransferorPeriod,
    ViolationGainReportedAsCapitalDespiteSection1254,
    ViolationPartialDispositionAllocationOmitted,
    ViolationPre1987DepletionOffsetNotComputed,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub property_category: Section1254PropertyCategory,
    pub disposition_type: DispositionType,
    pub gain_recognized_dollars: u64,
    pub aggregate_section_1254_costs_dollars: u64,
    pub depletion_offset_dollars_pre_1987: u64,
    pub partial_disposition_allocable_costs_dollars: u64,
    pub gain_character_as_reported: GainCharacterAsReported,
    pub partial_disposition_allocation_performed: bool,
    pub depletion_offset_computed_for_pre_1987: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section1254Mode,
    pub ordinary_income_recharacterized_dollars: u64,
    pub remaining_capital_or_1231_gain_dollars: u64,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type Section1254Input = Input;
pub type Section1254Output = Output;
pub type Section1254Result = Output;

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "26 U.S.C. § 1254(a)(1) — gain recharacterized as ordinary income to extent of lesser of aggregate § 1254 costs OR gain recognized".to_string(),
        "26 U.S.C. § 1254(a)(1)(B) — post-1986: aggregate of IDC § 263(c) + mine development § 616 + mine exploration § 617 + § 611 depletion reducing basis".to_string(),
        "26 U.S.C. § 1254(a)(3) — partnership/trust election; allocate to partners under Treas. Reg. § 1.1254-5".to_string(),
        "26 U.S.C. § 1254(b) — carryover basis transferee treated as holding for transferor's periods; § 351 and § 721 transfers tack basis".to_string(),
        "26 U.S.C. § 1254(c) — § 1254 property: any property within § 614 whose adjusted basis includes § 611 depletion adjustments".to_string(),
        "26 U.S.C. § 1254(d) — partial disposition: entire § 1254 costs allocable to disposed portion to extent of gain".to_string(),
        "Treas. Reg. § 1.1254-1 — natural resource recapture property; final regs reorganized 1995, updated 2008".to_string(),
        "Treas. Reg. § 1.1254-2(b) — nonproductive wells exception: NOT recapturable except limited-risk reimbursement situations".to_string(),
        "Treas. Reg. § 1.1254-5 — partnerships and partners; § 1254 cost allocation".to_string(),
        "IRS Form 4797 Instructions Part III line 28 — § 1254 recapture reporting".to_string(),
        "Cross-reference siblings: § 1245 personal property recapture; § 1250 real property recapture; § 1252 farm land recapture; § 1255 § 126 property recapture".to_string(),
    ];

    if input.gain_recognized_dollars == 0 {
        return Output {
            mode: Section1254Mode::NotApplicableNoGainRecognized,
            ordinary_income_recharacterized_dollars: 0,
            remaining_capital_or_1231_gain_dollars: 0,
            statutory_basis: "§ 1254(a)(1) — no gain recognized; rule inapplicable".to_string(),
            notes: "No gain on disposition of § 1254 property; recapture rule inapplicable.".to_string(),
            citations,
        };
    }

    if input.property_category == Section1254PropertyCategory::NotSection1254Property {
        return Output {
            mode: Section1254Mode::NotApplicableNotSection1254Property,
            ordinary_income_recharacterized_dollars: 0,
            remaining_capital_or_1231_gain_dollars: input.gain_recognized_dollars,
            statutory_basis: "§ 1254(c) — property does not satisfy § 1254 property definition".to_string(),
            notes: "Property's adjusted basis does not include § 611 depletion adjustments; not § 1254 property; original gain character preserved.".to_string(),
            citations,
        };
    }

    if input.property_category == Section1254PropertyCategory::NonproductiveWellLimitedRiskExcluded {
        return Output {
            mode: Section1254Mode::NotApplicableNonproductiveWellExclusion,
            ordinary_income_recharacterized_dollars: 0,
            remaining_capital_or_1231_gain_dollars: input.gain_recognized_dollars,
            statutory_basis: "Treas. Reg. § 1.1254-2(b) — nonproductive well exclusion".to_string(),
            notes: "§ 1254 costs attributable to nonproductive wells NOT recapturable (no limited-risk reimbursement situation); original gain character preserved.".to_string(),
            citations,
        };
    }

    if input.disposition_type == DispositionType::CarryoverBasisSection351Or721 {
        return Output {
            mode: Section1254Mode::CompliantCarryoverBasisTransfereeTreatedAsHoldingTransferorPeriod,
            ordinary_income_recharacterized_dollars: 0,
            remaining_capital_or_1231_gain_dollars: input.gain_recognized_dollars,
            statutory_basis: "§ 1254(b) — § 351/§ 721 carryover basis; transferee tacks transferor's § 1254 costs".to_string(),
            notes: "Carryover basis transfer; transferee treated as holding for transferor's periods. No § 1254 recapture at transferor on contribution; recapture deferred until transferee's future disposition.".to_string(),
            citations,
        };
    }

    if input.property_category == Section1254PropertyCategory::PreJan1_1987PropertyDepletionOffset
        && !input.depletion_offset_computed_for_pre_1987
    {
        return Output {
            mode: Section1254Mode::ViolationPre1987DepletionOffsetNotComputed,
            ordinary_income_recharacterized_dollars: input
                .gain_recognized_dollars
                .min(input.aggregate_section_1254_costs_dollars),
            remaining_capital_or_1231_gain_dollars: input
                .gain_recognized_dollars
                .saturating_sub(input.aggregate_section_1254_costs_dollars),
            statutory_basis: "§ 1254(a)(1)(A) pre-1987 — depletion offset to IDC must be computed".to_string(),
            notes: "VIOLATION: pre-Jan 1, 1987 property requires reduction of IDC by depletion that would have been allowable if IDCs had been capitalized; offset not computed.".to_string(),
            citations,
        };
    }

    if input.property_category == Section1254PropertyCategory::PreJan1_1987PropertyDepletionOffset
        && input.depletion_offset_computed_for_pre_1987
    {
        let net_section_1254_costs = input
            .aggregate_section_1254_costs_dollars
            .saturating_sub(input.depletion_offset_dollars_pre_1987);
        let ordinary = net_section_1254_costs.min(input.gain_recognized_dollars);
        let remaining = input.gain_recognized_dollars.saturating_sub(ordinary);
        return Output {
            mode: Section1254Mode::CompliantPre1987PropertyDepletionOffsetApplied,
            ordinary_income_recharacterized_dollars: ordinary,
            remaining_capital_or_1231_gain_dollars: remaining,
            statutory_basis: "§ 1254(a)(1)(A) pre-1987 — IDC reduced by hypothetical capitalized depletion".to_string(),
            notes: format!(
                "COMPLIANT pre-1987: gross § 1254 costs ${} − depletion offset ${} = net § 1254 costs ${}; recharacterized ordinary income = lesser of net costs and gain ${}; residual capital/§ 1231 gain ${}.",
                input.aggregate_section_1254_costs_dollars,
                input.depletion_offset_dollars_pre_1987,
                net_section_1254_costs,
                ordinary,
                remaining
            ),
            citations,
        };
    }

    if input.disposition_type == DispositionType::PartialDispositionNotUndividedInterest
        && !input.partial_disposition_allocation_performed
    {
        return Output {
            mode: Section1254Mode::ViolationPartialDispositionAllocationOmitted,
            ordinary_income_recharacterized_dollars: input
                .gain_recognized_dollars
                .min(input.aggregate_section_1254_costs_dollars),
            remaining_capital_or_1231_gain_dollars: input
                .gain_recognized_dollars
                .saturating_sub(input.aggregate_section_1254_costs_dollars),
            statutory_basis: "§ 1254(d) — partial disposition allocation required".to_string(),
            notes: "VIOLATION: partial disposition of § 1254 property requires § 1254 costs allocable to the disposed portion to be computed under § 1254(d); allocation not performed.".to_string(),
            citations,
        };
    }

    if input.disposition_type == DispositionType::PartialDispositionNotUndividedInterest
        && input.partial_disposition_allocation_performed
    {
        let allocable = input
            .partial_disposition_allocable_costs_dollars
            .min(input.aggregate_section_1254_costs_dollars);
        let ordinary = allocable.min(input.gain_recognized_dollars);
        let remaining = input.gain_recognized_dollars.saturating_sub(ordinary);
        return Output {
            mode: Section1254Mode::CompliantPartialDispositionAllocationApplied,
            ordinary_income_recharacterized_dollars: ordinary,
            remaining_capital_or_1231_gain_dollars: remaining,
            statutory_basis: "§ 1254(d) — partial disposition allocation applied".to_string(),
            notes: format!(
                "COMPLIANT partial disposition: allocable § 1254 costs ${} (subset of aggregate ${}); recharacterized ordinary income = lesser of allocable costs and gain = ${}; residual gain = ${}.",
                allocable, input.aggregate_section_1254_costs_dollars, ordinary, remaining
            ),
            citations,
        };
    }

    let ordinary = input
        .aggregate_section_1254_costs_dollars
        .min(input.gain_recognized_dollars);
    let remaining = input.gain_recognized_dollars.saturating_sub(ordinary);

    if matches!(
        input.gain_character_as_reported,
        GainCharacterAsReported::CapitalLongTerm
            | GainCharacterAsReported::CapitalShortTerm
            | GainCharacterAsReported::Section1231Gain
    ) && ordinary > 0
        && input.disposition_type != DispositionType::UndividedInterest
    {
        return Output {
            mode: Section1254Mode::ViolationGainReportedAsCapitalDespiteSection1254,
            ordinary_income_recharacterized_dollars: ordinary,
            remaining_capital_or_1231_gain_dollars: remaining,
            statutory_basis: "§ 1254(a)(1) — gain reported as capital/§ 1231 despite § 1254 costs".to_string(),
            notes: format!(
                "VIOLATION § 1254(a)(1): gain of ${} reported as {:?} but § 1254 costs of ${} require ${} ordinary income recharacterization; residual gain = ${}.",
                input.gain_recognized_dollars,
                input.gain_character_as_reported,
                input.aggregate_section_1254_costs_dollars,
                ordinary,
                remaining
            ),
            citations,
        };
    }

    Output {
        mode: Section1254Mode::CompliantGainRecharacterizedAsOrdinary,
        ordinary_income_recharacterized_dollars: ordinary,
        remaining_capital_or_1231_gain_dollars: remaining,
        statutory_basis: "§ 1254(a)(1) — gain recharacterized as ordinary income to extent of § 1254 costs".to_string(),
        notes: format!(
            "COMPLIANT § 1254: gain ${}; aggregate § 1254 costs ${}; recharacterized ordinary income = ${}; residual capital/§ 1231 gain = ${}.",
            input.gain_recognized_dollars,
            input.aggregate_section_1254_costs_dollars,
            ordinary,
            remaining
        ),
        citations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_post_1987_working_interest_sale() -> Input {
        Input {
            property_category: Section1254PropertyCategory::PostJan1_1987PropertyAllRecapture,
            disposition_type: DispositionType::EntireProperty,
            gain_recognized_dollars: 1_000_000,
            aggregate_section_1254_costs_dollars: 600_000,
            depletion_offset_dollars_pre_1987: 0,
            partial_disposition_allocable_costs_dollars: 0,
            gain_character_as_reported: GainCharacterAsReported::OrdinaryOther,
            partial_disposition_allocation_performed: false,
            depletion_offset_computed_for_pre_1987: false,
        }
    }

    #[test]
    fn no_gain_not_applicable() {
        let input = Input {
            gain_recognized_dollars: 0,
            ..baseline_post_1987_working_interest_sale()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1254Mode::NotApplicableNoGainRecognized);
    }

    #[test]
    fn not_section_1254_property_not_applicable() {
        let input = Input {
            property_category: Section1254PropertyCategory::NotSection1254Property,
            ..baseline_post_1987_working_interest_sale()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1254Mode::NotApplicableNotSection1254Property);
    }

    #[test]
    fn nonproductive_well_excluded_not_applicable() {
        let input = Input {
            property_category: Section1254PropertyCategory::NonproductiveWellLimitedRiskExcluded,
            ..baseline_post_1987_working_interest_sale()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1254Mode::NotApplicableNonproductiveWellExclusion);
    }

    #[test]
    fn post_1987_working_interest_compliant_full_recapture() {
        let result = compute(&baseline_post_1987_working_interest_sale());
        assert_eq!(result.mode, Section1254Mode::CompliantGainRecharacterizedAsOrdinary);
        assert_eq!(result.ordinary_income_recharacterized_dollars, 600_000);
        assert_eq!(result.remaining_capital_or_1231_gain_dollars, 400_000);
    }

    #[test]
    fn post_1987_costs_exceed_gain_capped_at_gain() {
        let input = Input {
            aggregate_section_1254_costs_dollars: 1_500_000,
            ..baseline_post_1987_working_interest_sale()
        };
        let result = compute(&input);
        assert_eq!(result.ordinary_income_recharacterized_dollars, 1_000_000);
        assert_eq!(result.remaining_capital_or_1231_gain_dollars, 0);
    }

    #[test]
    fn capital_long_term_misreport_violation() {
        let input = Input {
            gain_character_as_reported: GainCharacterAsReported::CapitalLongTerm,
            ..baseline_post_1987_working_interest_sale()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1254Mode::ViolationGainReportedAsCapitalDespiteSection1254);
    }

    #[test]
    fn section_1231_misreport_violation() {
        let input = Input {
            gain_character_as_reported: GainCharacterAsReported::Section1231Gain,
            ..baseline_post_1987_working_interest_sale()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1254Mode::ViolationGainReportedAsCapitalDespiteSection1254);
    }

    #[test]
    fn pre_1987_property_depletion_offset_compliant() {
        let input = Input {
            property_category: Section1254PropertyCategory::PreJan1_1987PropertyDepletionOffset,
            aggregate_section_1254_costs_dollars: 800_000,
            depletion_offset_dollars_pre_1987: 200_000,
            depletion_offset_computed_for_pre_1987: true,
            ..baseline_post_1987_working_interest_sale()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1254Mode::CompliantPre1987PropertyDepletionOffsetApplied);
        assert_eq!(result.ordinary_income_recharacterized_dollars, 600_000);
        assert_eq!(result.remaining_capital_or_1231_gain_dollars, 400_000);
    }

    #[test]
    fn pre_1987_depletion_offset_omitted_violation() {
        let input = Input {
            property_category: Section1254PropertyCategory::PreJan1_1987PropertyDepletionOffset,
            depletion_offset_computed_for_pre_1987: false,
            ..baseline_post_1987_working_interest_sale()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1254Mode::ViolationPre1987DepletionOffsetNotComputed);
    }

    #[test]
    fn partial_disposition_allocation_compliant() {
        let input = Input {
            disposition_type: DispositionType::PartialDispositionNotUndividedInterest,
            partial_disposition_allocation_performed: true,
            partial_disposition_allocable_costs_dollars: 300_000,
            ..baseline_post_1987_working_interest_sale()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1254Mode::CompliantPartialDispositionAllocationApplied);
        assert_eq!(result.ordinary_income_recharacterized_dollars, 300_000);
        assert_eq!(result.remaining_capital_or_1231_gain_dollars, 700_000);
    }

    #[test]
    fn partial_disposition_allocation_omitted_violation() {
        let input = Input {
            disposition_type: DispositionType::PartialDispositionNotUndividedInterest,
            partial_disposition_allocation_performed: false,
            ..baseline_post_1987_working_interest_sale()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1254Mode::ViolationPartialDispositionAllocationOmitted);
    }

    #[test]
    fn carryover_basis_section_351_compliant_no_recapture() {
        let input = Input {
            disposition_type: DispositionType::CarryoverBasisSection351Or721,
            ..baseline_post_1987_working_interest_sale()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1254Mode::CompliantCarryoverBasisTransfereeTreatedAsHoldingTransferorPeriod);
        assert_eq!(result.ordinary_income_recharacterized_dollars, 0);
    }

    #[test]
    fn overriding_royalty_interest_compliant() {
        let input = Input {
            property_category: Section1254PropertyCategory::OverridingRoyaltyInterest,
            ..baseline_post_1987_working_interest_sale()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1254Mode::CompliantGainRecharacterizedAsOrdinary);
    }

    #[test]
    fn net_profits_interest_compliant() {
        let input = Input {
            property_category: Section1254PropertyCategory::NetProfitsInterest,
            ..baseline_post_1987_working_interest_sale()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1254Mode::CompliantGainRecharacterizedAsOrdinary);
    }

    #[test]
    fn undivided_interest_not_partial_disposition() {
        let input = Input {
            disposition_type: DispositionType::UndividedInterest,
            ..baseline_post_1987_working_interest_sale()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1254Mode::CompliantGainRecharacterizedAsOrdinary);
    }

    #[test]
    fn citations_pin_section_1254_subsections_and_treas_regs() {
        let result = compute(&baseline_post_1987_working_interest_sale());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("§ 1254(a)(1)"));
        assert!(joined.contains("§ 1254(a)(1)(B)"));
        assert!(joined.contains("§ 1254(a)(3)"));
        assert!(joined.contains("§ 1254(b)"));
        assert!(joined.contains("§ 1254(c)"));
        assert!(joined.contains("§ 1254(d)"));
        assert!(joined.contains("§ 1.1254-1"));
        assert!(joined.contains("§ 1.1254-2(b)"));
        assert!(joined.contains("§ 1.1254-5"));
        assert!(joined.contains("Form 4797 Instructions"));
        assert!(joined.contains("§ 1245"));
        assert!(joined.contains("§ 1250"));
        assert!(joined.contains("§ 1252"));
        assert!(joined.contains("§ 1255"));
    }

    #[test]
    fn constant_pin_post_1986_effective_date() {
        assert_eq!(SECTION_1254_POST_1986_EFFECTIVE_DATE_YEAR, 1987);
        assert_eq!(SECTION_1254_POST_1986_EFFECTIVE_DATE_MONTH, 1);
        assert_eq!(SECTION_1254_POST_1986_EFFECTIVE_DATE_DAY, 1);
        assert_eq!(SECTION_1254_TREAS_REG_FINAL_REGS_REORGANIZED_YEAR, 1995);
        assert_eq!(SECTION_1254_TREAS_REG_UPDATED_YEAR, 2008);
    }

    #[test]
    fn saturating_overflow_defense_extreme_gain() {
        let input = Input {
            gain_recognized_dollars: u64::MAX,
            aggregate_section_1254_costs_dollars: u64::MAX,
            ..baseline_post_1987_working_interest_sale()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1254Mode::CompliantGainRecharacterizedAsOrdinary);
    }
}
