//! IRC § 755 rules for allocation of basis adjustments.
//!
//! Downstream allocation regime that determines how § 743(b)
//! transferee-side basis adjustments (iter 576 module) and § 734(b)
//! distribution-side basis adjustments (iter 578 module) are spread
//! across partnership properties to preserve inside-outside basis
//! equality.
//!
//! **§ 755(a) general rule**: § 743(b) and § 734(b) basis
//! adjustments must be allocated among partnership properties in a
//! manner that takes into account the income, gain, or loss that
//! would have been recognized on a hypothetical taxable sale of
//! such properties, designed to preserve the partner's economic
//! position.
//!
//! **§ 755(b) two-class rule**: adjustments are first allocated
//! between two distinct classes of property — Class 1 Capital Gain
//! Property (§ 1221 capital assets + § 1231(b) depreciable real and
//! trade-or-business property held > 1 year); Class 2 Ordinary
//! Income Property (everything else, including § 751 hot assets,
//! § 1245 recapture property, and any other property that would
//! produce ordinary character on sale).
//!
//! Within each class, the allocation among individual properties is
//! based on unrealized appreciation or depreciation. A class with
//! zero net unrealized appreciation gets a proportional allocation
//! based on adjusted basis.
//!
//! **§ 743(b) class-attribution math** (Treas. Reg. § 1.755-1(b)):
//! amount allocated to the ORDINARY INCOME CLASS equals the total
//! ordinary income/gain/loss that would be allocated to the
//! transferee from a hypothetical sale of ALL ordinary income
//! property. The remainder is allocated to the capital gain class.
//!
//! **§ 734(b) class-attribution math** (Treas. Reg. § 1.755-1(c)):
//! follows the character of the gain or loss recognized by the
//! distributee partner under § 731 — capital-character recognition
//! flows the adjustment to the capital gain class.
//!
//! **TD 9059** (June 9, 2003) finalized coordination between § 755
//! and § 1060 (residual method for purchase price allocation),
//! adding specific class-allocation methodology for partnership
//! mergers and certain asset purchases.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[allow(dead_code)]
pub const CAPITAL_GAIN_PROPERTY_CLASS: u32 = 1;
#[allow(dead_code)]
pub const ORDINARY_INCOME_PROPERTY_CLASS: u32 = 2;
#[allow(dead_code)]
pub const TD_9059_FINALIZATION_YEAR: u32 = 2003;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdjustmentType {
    NotApplicable,
    Section743bTransferee,
    Section734bDistribution,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    AllocationToCapitalGainClassOnly,
    AllocationToOrdinaryIncomeClassOnly,
    AllocationBetweenBothClassesUnderTwoClassRule,
    ViolationAdjustmentMisallocatedAcrossClasses,
    ViolationAllocationExceedsClassNetUnrealizedAppreciation,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub adjustment_type: AdjustmentType,
    pub total_adjustment_cents: i128,
    pub ordinary_income_class_attributable_cents: i128,
    pub capital_gain_class_attributable_cents: i128,
    pub capital_class_net_unrealized_appreciation_cents: i128,
    pub ordinary_class_net_unrealized_appreciation_cents: i128,
    pub taxpayer_allocated_to_ordinary_class_cents: i128,
    pub taxpayer_allocated_to_capital_class_cents: i128,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Output {
    pub severity: Severity,
    pub ordinary_income_class_allocation_cents: i128,
    pub capital_gain_class_allocation_cents: i128,
    pub compliant_with_class_attribution: bool,
    pub notes: Vec<String>,
    pub citations: Vec<String>,
}

pub type Section755Input = Input;
pub type Section755Output = Output;
pub type Section755Result = Output;

pub fn check(input: &Input) -> Output {
    let mut notes: Vec<String> = Vec::new();
    let citations: Vec<String> = vec![
        "IRC § 755(a) (general allocation rule)".to_string(),
        "IRC § 755(b) (two-class rule: capital gain class + ordinary income class)".to_string(),
        "IRC § 1221 (capital asset definition cross-reference)".to_string(),
        "IRC § 1231(b) (depreciable trade-or-business property definition)".to_string(),
        "IRC § 751 (hot assets — ordinary income class composition)".to_string(),
        "IRC § 743(b) (transferee basis adjustment — § 755 allocation downstream)".to_string(),
        "IRC § 734(b) (distributee basis adjustment — § 755 allocation downstream)".to_string(),
        "IRC § 754 (election that triggers § 755 allocation)".to_string(),
        "Treas. Reg. § 1.755-1(a) (general allocation methodology)".to_string(),
        "Treas. Reg. § 1.755-1(b) (§ 743(b) class allocation)".to_string(),
        "Treas. Reg. § 1.755-1(c) (§ 734(b) class allocation)".to_string(),
        "TD 9059 (June 9, 2003) — coordination of § 755 and § 1060 residual method".to_string(),
    ];

    if matches!(input.adjustment_type, AdjustmentType::NotApplicable) {
        notes.push("No § 743(b) or § 734(b) adjustment recorded; § 755 allocation not triggered.".to_string());
        return Output {
            severity: Severity::NotApplicable,
            ordinary_income_class_allocation_cents: 0,
            capital_gain_class_allocation_cents: 0,
            compliant_with_class_attribution: true,
            notes,
            citations,
        };
    }

    let class_attribution_sum = input
        .ordinary_income_class_attributable_cents
        .saturating_add(input.capital_gain_class_attributable_cents);

    if class_attribution_sum != input.total_adjustment_cents {
        notes.push(format!(
            "Sum of class attributions (${}) does not equal total adjustment (${}) — § 755(b) misallocation violation.",
            class_attribution_sum / 100,
            input.total_adjustment_cents / 100
        ));
        return Output {
            severity: Severity::ViolationAdjustmentMisallocatedAcrossClasses,
            ordinary_income_class_allocation_cents: input.ordinary_income_class_attributable_cents,
            capital_gain_class_allocation_cents: input.capital_gain_class_attributable_cents,
            compliant_with_class_attribution: false,
            notes,
            citations,
        };
    }

    if input.ordinary_income_class_attributable_cents > 0
        && input.ordinary_income_class_attributable_cents
            > input.ordinary_class_net_unrealized_appreciation_cents
    {
        notes.push(format!(
            "Ordinary-income-class positive allocation ${} exceeds class net unrealized appreciation ${} — § 1.755-1(b)(2) cap violation.",
            input.ordinary_income_class_attributable_cents / 100,
            input.ordinary_class_net_unrealized_appreciation_cents / 100
        ));
        return Output {
            severity: Severity::ViolationAllocationExceedsClassNetUnrealizedAppreciation,
            ordinary_income_class_allocation_cents: input.ordinary_income_class_attributable_cents,
            capital_gain_class_allocation_cents: input.capital_gain_class_attributable_cents,
            compliant_with_class_attribution: false,
            notes,
            citations,
        };
    }

    if input.capital_gain_class_attributable_cents > 0
        && input.capital_gain_class_attributable_cents
            > input.capital_class_net_unrealized_appreciation_cents
    {
        notes.push(format!(
            "Capital-gain-class positive allocation ${} exceeds class net unrealized appreciation ${} — § 1.755-1(b)(2) cap violation.",
            input.capital_gain_class_attributable_cents / 100,
            input.capital_class_net_unrealized_appreciation_cents / 100
        ));
        return Output {
            severity: Severity::ViolationAllocationExceedsClassNetUnrealizedAppreciation,
            ordinary_income_class_allocation_cents: input.ordinary_income_class_attributable_cents,
            capital_gain_class_allocation_cents: input.capital_gain_class_attributable_cents,
            compliant_with_class_attribution: false,
            notes,
            citations,
        };
    }

    let ordinary_allocated = input.ordinary_income_class_attributable_cents != 0;
    let capital_allocated = input.capital_gain_class_attributable_cents != 0;

    let severity = if ordinary_allocated && capital_allocated {
        notes.push(format!(
            "§ 755(b) two-class allocation: ordinary-income class ${} + capital-gain class ${} = total ${}.",
            input.ordinary_income_class_attributable_cents / 100,
            input.capital_gain_class_attributable_cents / 100,
            input.total_adjustment_cents / 100
        ));
        Severity::AllocationBetweenBothClassesUnderTwoClassRule
    } else if ordinary_allocated {
        notes.push(format!(
            "§ 755(b): full adjustment ${} allocated to ordinary income class (no capital gain class attribution).",
            input.ordinary_income_class_attributable_cents / 100
        ));
        Severity::AllocationToOrdinaryIncomeClassOnly
    } else if capital_allocated {
        notes.push(format!(
            "§ 755(b): full adjustment ${} allocated to capital gain class (no ordinary income class attribution).",
            input.capital_gain_class_attributable_cents / 100
        ));
        Severity::AllocationToCapitalGainClassOnly
    } else {
        notes.push("Adjustment is zero or net-zero across classes; no allocation required.".to_string());
        Severity::NotApplicable
    };

    Output {
        severity,
        ordinary_income_class_allocation_cents: input.ordinary_income_class_attributable_cents,
        capital_gain_class_allocation_cents: input.capital_gain_class_attributable_cents,
        compliant_with_class_attribution: true,
        notes,
        citations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_743b() -> Input {
        Input {
            adjustment_type: AdjustmentType::Section743bTransferee,
            total_adjustment_cents: 5_000_000,
            ordinary_income_class_attributable_cents: 2_000_000,
            capital_gain_class_attributable_cents: 3_000_000,
            capital_class_net_unrealized_appreciation_cents: 10_000_000,
            ordinary_class_net_unrealized_appreciation_cents: 5_000_000,
            taxpayer_allocated_to_ordinary_class_cents: 2_000_000,
            taxpayer_allocated_to_capital_class_cents: 3_000_000,
        }
    }

    #[test]
    fn two_class_allocation_compliant() {
        let out = check(&base_743b());
        assert_eq!(
            out.severity,
            Severity::AllocationBetweenBothClassesUnderTwoClassRule
        );
        assert!(out.compliant_with_class_attribution);
    }

    #[test]
    fn ordinary_only_allocation_compliant() {
        let mut i = base_743b();
        i.ordinary_income_class_attributable_cents = 5_000_000;
        i.capital_gain_class_attributable_cents = 0;
        let out = check(&i);
        assert_eq!(out.severity, Severity::AllocationToOrdinaryIncomeClassOnly);
    }

    #[test]
    fn capital_only_allocation_compliant() {
        let mut i = base_743b();
        i.ordinary_income_class_attributable_cents = 0;
        i.capital_gain_class_attributable_cents = 5_000_000;
        let out = check(&i);
        assert_eq!(out.severity, Severity::AllocationToCapitalGainClassOnly);
    }

    #[test]
    fn misallocation_violation_when_sum_does_not_match_total() {
        let mut i = base_743b();
        i.ordinary_income_class_attributable_cents = 2_000_000;
        i.capital_gain_class_attributable_cents = 2_500_000;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationAdjustmentMisallocatedAcrossClasses
        );
        assert!(!out.compliant_with_class_attribution);
    }

    #[test]
    fn ordinary_class_allocation_exceeds_class_appreciation_violation() {
        let mut i = base_743b();
        i.ordinary_income_class_attributable_cents = 10_000_000;
        i.capital_gain_class_attributable_cents = -5_000_000;
        i.total_adjustment_cents = 5_000_000;
        i.ordinary_class_net_unrealized_appreciation_cents = 5_000_000;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationAllocationExceedsClassNetUnrealizedAppreciation
        );
    }

    #[test]
    fn capital_class_allocation_exceeds_class_appreciation_violation() {
        let mut i = base_743b();
        i.ordinary_income_class_attributable_cents = -7_000_000;
        i.capital_gain_class_attributable_cents = 12_000_000;
        i.total_adjustment_cents = 5_000_000;
        i.capital_class_net_unrealized_appreciation_cents = 10_000_000;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationAllocationExceedsClassNetUnrealizedAppreciation
        );
    }

    #[test]
    fn section_734b_two_class_allocation() {
        let mut i = base_743b();
        i.adjustment_type = AdjustmentType::Section734bDistribution;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::AllocationBetweenBothClassesUnderTwoClassRule
        );
    }

    #[test]
    fn negative_total_adjustment_with_two_class_decrease() {
        let mut i = base_743b();
        i.total_adjustment_cents = -3_000_000;
        i.ordinary_income_class_attributable_cents = -1_000_000;
        i.capital_gain_class_attributable_cents = -2_000_000;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::AllocationBetweenBothClassesUnderTwoClassRule
        );
    }

    #[test]
    fn zero_adjustment_returns_not_applicable() {
        let mut i = base_743b();
        i.total_adjustment_cents = 0;
        i.ordinary_income_class_attributable_cents = 0;
        i.capital_gain_class_attributable_cents = 0;
        let out = check(&i);
        assert_eq!(out.severity, Severity::NotApplicable);
    }

    #[test]
    fn not_applicable_type_returns_default() {
        let mut i = base_743b();
        i.adjustment_type = AdjustmentType::NotApplicable;
        let out = check(&i);
        assert_eq!(out.severity, Severity::NotApplicable);
    }

    #[test]
    fn citations_pin_755a_755b_treas_reg_subsections() {
        let out = check(&base_743b());
        assert!(out.citations.iter().any(|c| c.contains("§ 755(a)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 755(b)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 1.755-1(a)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 1.755-1(b)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 1.755-1(c)")));
    }

    #[test]
    fn citations_pin_1221_1231b_751_cross_refs() {
        let out = check(&base_743b());
        assert!(out.citations.iter().any(|c| c.contains("§ 1221")));
        assert!(out.citations.iter().any(|c| c.contains("§ 1231(b)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 751")));
    }

    #[test]
    fn citations_pin_743b_734b_754_cross_refs() {
        let out = check(&base_743b());
        assert!(out.citations.iter().any(|c| c.contains("§ 743(b)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 734(b)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 754")));
    }

    #[test]
    fn citations_pin_td_9059() {
        let out = check(&base_743b());
        assert!(out.citations.iter().any(|c| c.contains("TD 9059")));
        assert!(out.citations.iter().any(|c| c.contains("June 9, 2003")));
    }

    #[test]
    fn constant_pin_capital_gain_class_1() {
        assert_eq!(CAPITAL_GAIN_PROPERTY_CLASS, 1);
    }

    #[test]
    fn constant_pin_ordinary_income_class_2() {
        assert_eq!(ORDINARY_INCOME_PROPERTY_CLASS, 2);
    }

    #[test]
    fn constant_pin_td_9059_2003_year() {
        assert_eq!(TD_9059_FINALIZATION_YEAR, 2003);
    }

    #[test]
    fn ordinary_class_at_exactly_appreciation_cap_compliant() {
        let mut i = base_743b();
        i.ordinary_income_class_attributable_cents = 5_000_000;
        i.capital_gain_class_attributable_cents = 0;
        i.total_adjustment_cents = 5_000_000;
        i.ordinary_class_net_unrealized_appreciation_cents = 5_000_000;
        let out = check(&i);
        assert_eq!(out.severity, Severity::AllocationToOrdinaryIncomeClassOnly);
    }
}
