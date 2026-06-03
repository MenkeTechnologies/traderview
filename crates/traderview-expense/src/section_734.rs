//! IRC § 734 adjustment to basis of undistributed partnership
//! property where § 754 election in effect or substantial basis
//! reduction.
//!
//! **§ 734(a) general rule**: no adjustment to partnership property
//! on distribution.
//!
//! **§ 734(b) exception**: adjustment REQUIRED if EITHER (1) a § 754
//! election is in effect for the partnership OR (2) the partnership
//! has a "substantial basis reduction" with respect to the
//! distribution.
//!
//! **§ 734(b)(1) INCREASE** remaining partnership property basis by
//! the sum of:
//! - (A) gain recognized by the distributee partner under § 731(a)(1)
//!   (cash + marketable securities distribution in excess of outside
//!   basis), PLUS
//! - (B) excess of partnership's adjusted basis in distributed
//!   property over the distributee's basis in that property as
//!   determined under § 732 (step-up at distributee level requires
//!   matching step-up at partnership level).
//!
//! **§ 734(b)(2) DECREASE** remaining partnership property basis by
//! the sum of:
//! - (A) loss recognized by the distributee partner under § 731(a)(2)
//!   (liquidating distribution of cash + receivables + inventory only,
//!   below outside basis), PLUS
//! - (B) excess of distributee's basis in distributed property as
//!   determined under § 732 over the partnership's adjusted basis in
//!   that property (step-down at distributee level requires matching
//!   step-down at partnership level).
//!
//! **§ 734(d) substantial basis reduction** (TCJA 2017 broadened
//! scope, applicable to distributions after Dec. 31, 2017): the sum
//! of amounts described in § 734(b)(2)(A) and (B) exceeds **$250,000**.
//! If substantial basis reduction exists, the § 734(b) adjustment
//! is MANDATORY even without a § 754 election in effect.
//!
//! Pairs with `section_754` (election mechanics), `section_743`
//! (transferee partner basis adjustment — the transfer-side companion
//! to § 734's distribution-side rule), `section_755` (allocation of
//! adjustments among partnership properties), `section_731`
//! (distribution gain/loss recognition rules), `section_732`
//! (distributee partner basis in distributed property).

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[allow(dead_code)]
pub const SUBSTANTIAL_BASIS_REDUCTION_THRESHOLD_CENTS: u64 = 25_000_000;
#[allow(dead_code)]
pub const TCJA_2017_AMENDMENT_EFFECTIVE_AFTER_YEAR: u32 = 2017;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DistributionType {
    NotApplicable,
    CurrentDistribution,
    LiquidatingDistribution,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    DistributionWithoutAdjustmentNo754NoSubstantialBasisReduction,
    Section754PositiveIncreaseGainOrBasisStepUp,
    Section754NegativeDecreaseLossOrBasisStepDown,
    Section754NoNetAdjustment,
    SubstantialBasisReductionMandatoryAdjustmentEvenWithout754,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub distribution_type: DistributionType,
    pub section_754_election_in_effect: bool,
    pub distribution_year: u32,
    pub distributee_gain_recognized_under_731a1_cents: u64,
    pub distributee_loss_recognized_under_731a2_cents: u64,
    pub partnership_adjusted_basis_in_distributed_property_cents: u64,
    pub distributee_basis_in_distributed_property_under_732_cents: u64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Output {
    pub severity: Severity,
    pub adjustment_required: bool,
    pub adjustment_cents: i128,
    pub increase_under_734b1_cents: u64,
    pub decrease_under_734b2_cents: u64,
    pub substantial_basis_reduction_triggered: bool,
    pub notes: Vec<String>,
    pub citations: Vec<String>,
}

pub type Section734Input = Input;
pub type Section734Output = Output;
pub type Section734Result = Output;

pub fn check(input: &Input) -> Output {
    let mut notes: Vec<String> = Vec::new();
    let citations: Vec<String> = vec![
        "IRC § 734(a) (general rule — no basis adjustment)".to_string(),
        "IRC § 734(b) (exception — § 754 election OR substantial basis reduction)".to_string(),
        "IRC § 734(b)(1) (increase = gain + property basis step-up)".to_string(),
        "IRC § 734(b)(2) (decrease = loss + property basis step-down)".to_string(),
        "IRC § 734(d) (substantial basis reduction — $250,000 threshold)".to_string(),
        "IRC § 731(a)(1) (distributee gain recognition)".to_string(),
        "IRC § 731(a)(2) (distributee loss recognition)".to_string(),
        "IRC § 732 (distributee basis in distributed property)".to_string(),
        "IRC § 754 (election mechanics)".to_string(),
        "IRC § 755 (allocation among partnership properties)".to_string(),
        "Treas. Reg. § 1.734-1".to_string(),
        "AJCA 2004 § 833 (added § 734(d) substantial-basis-reduction prong)".to_string(),
    ];

    if matches!(input.distribution_type, DistributionType::NotApplicable) {
        notes.push("No partnership distribution recorded.".to_string());
        return Output {
            severity: Severity::NotApplicable,
            adjustment_required: false,
            adjustment_cents: 0,
            increase_under_734b1_cents: 0,
            decrease_under_734b2_cents: 0,
            substantial_basis_reduction_triggered: false,
            notes,
            citations,
        };
    }

    let property_step_up = input
        .partnership_adjusted_basis_in_distributed_property_cents
        .saturating_sub(input.distributee_basis_in_distributed_property_under_732_cents);
    let property_step_down = input
        .distributee_basis_in_distributed_property_under_732_cents
        .saturating_sub(input.partnership_adjusted_basis_in_distributed_property_cents);

    let increase_b1 = input
        .distributee_gain_recognized_under_731a1_cents
        .saturating_add(property_step_up);
    let decrease_b2 = input
        .distributee_loss_recognized_under_731a2_cents
        .saturating_add(property_step_down);

    let substantial_basis_reduction = decrease_b2 > SUBSTANTIAL_BASIS_REDUCTION_THRESHOLD_CENTS;

    if !input.section_754_election_in_effect && !substantial_basis_reduction {
        notes.push("No § 754 election in effect and no substantial basis reduction — § 734(a) default applies; no basis adjustment to remaining partnership property.".to_string());
        return Output {
            severity: Severity::DistributionWithoutAdjustmentNo754NoSubstantialBasisReduction,
            adjustment_required: false,
            adjustment_cents: 0,
            increase_under_734b1_cents: increase_b1,
            decrease_under_734b2_cents: decrease_b2,
            substantial_basis_reduction_triggered: false,
            notes,
            citations,
        };
    }

    let net_adjustment: i128 = (increase_b1 as i128) - (decrease_b2 as i128);

    if substantial_basis_reduction && !input.section_754_election_in_effect {
        notes.push(format!(
            "§ 734(d) substantial basis reduction triggered: § 734(b)(2) decrease ${} > ${} threshold. Mandatory adjustment even without § 754 election.",
            decrease_b2 / 100,
            SUBSTANTIAL_BASIS_REDUCTION_THRESHOLD_CENTS / 100
        ));
        return Output {
            severity: Severity::SubstantialBasisReductionMandatoryAdjustmentEvenWithout754,
            adjustment_required: true,
            adjustment_cents: net_adjustment,
            increase_under_734b1_cents: increase_b1,
            decrease_under_734b2_cents: decrease_b2,
            substantial_basis_reduction_triggered: true,
            notes,
            citations,
        };
    }

    let severity = if net_adjustment > 0 {
        notes.push(format!(
            "§ 754 election + § 734(b)(1) net positive: increase remaining partnership property basis by ${} (gain ${} + property step-up ${}).",
            net_adjustment / 100,
            input.distributee_gain_recognized_under_731a1_cents / 100,
            property_step_up / 100
        ));
        Severity::Section754PositiveIncreaseGainOrBasisStepUp
    } else if net_adjustment < 0 {
        notes.push(format!(
            "§ 754 election + § 734(b)(2) net negative: decrease remaining partnership property basis by ${} (loss ${} + property step-down ${}).",
            (-net_adjustment) / 100,
            input.distributee_loss_recognized_under_731a2_cents / 100,
            property_step_down / 100
        ));
        Severity::Section754NegativeDecreaseLossOrBasisStepDown
    } else {
        notes.push("§ 754 election in effect but no net § 734(b) adjustment (increase and decrease offset).".to_string());
        Severity::Section754NoNetAdjustment
    };

    Output {
        severity,
        adjustment_required: net_adjustment != 0,
        adjustment_cents: net_adjustment,
        increase_under_734b1_cents: increase_b1,
        decrease_under_734b2_cents: decrease_b2,
        substantial_basis_reduction_triggered: substantial_basis_reduction,
        notes,
        citations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_754_current() -> Input {
        Input {
            distribution_type: DistributionType::CurrentDistribution,
            section_754_election_in_effect: true,
            distribution_year: 2026,
            distributee_gain_recognized_under_731a1_cents: 1_000_000,
            distributee_loss_recognized_under_731a2_cents: 0,
            partnership_adjusted_basis_in_distributed_property_cents: 5_000_000,
            distributee_basis_in_distributed_property_under_732_cents: 3_000_000,
        }
    }

    #[test]
    fn current_distribution_with_gain_and_step_up_increase() {
        let out = check(&base_754_current());
        assert_eq!(
            out.severity,
            Severity::Section754PositiveIncreaseGainOrBasisStepUp
        );
        assert_eq!(out.increase_under_734b1_cents, 3_000_000);
        assert_eq!(out.adjustment_cents, 3_000_000);
        assert!(out.adjustment_required);
    }

    #[test]
    fn distribution_with_loss_and_step_down_decrease() {
        let mut i = base_754_current();
        i.distribution_type = DistributionType::LiquidatingDistribution;
        i.distributee_gain_recognized_under_731a1_cents = 0;
        i.distributee_loss_recognized_under_731a2_cents = 500_000;
        i.partnership_adjusted_basis_in_distributed_property_cents = 2_000_000;
        i.distributee_basis_in_distributed_property_under_732_cents = 4_000_000;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::Section754NegativeDecreaseLossOrBasisStepDown
        );
        assert_eq!(out.decrease_under_734b2_cents, 2_500_000);
        assert_eq!(out.adjustment_cents, -2_500_000);
    }

    #[test]
    fn distribution_with_no_gain_no_loss_no_basis_diff_no_net_adjustment() {
        let mut i = base_754_current();
        i.distributee_gain_recognized_under_731a1_cents = 0;
        i.distributee_loss_recognized_under_731a2_cents = 0;
        i.partnership_adjusted_basis_in_distributed_property_cents = 3_000_000;
        i.distributee_basis_in_distributed_property_under_732_cents = 3_000_000;
        let out = check(&i);
        assert_eq!(out.severity, Severity::Section754NoNetAdjustment);
        assert_eq!(out.adjustment_cents, 0);
        assert!(!out.adjustment_required);
    }

    #[test]
    fn distribution_without_754_no_substantial_reduction_no_adjustment() {
        let mut i = base_754_current();
        i.section_754_election_in_effect = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::DistributionWithoutAdjustmentNo754NoSubstantialBasisReduction
        );
        assert!(!out.adjustment_required);
        assert_eq!(out.adjustment_cents, 0);
    }

    #[test]
    fn substantial_basis_reduction_above_250k_mandatory_adjustment() {
        let mut i = base_754_current();
        i.section_754_election_in_effect = false;
        i.distributee_loss_recognized_under_731a2_cents = 30_000_000;
        let out = check(&i);
        assert!(out.substantial_basis_reduction_triggered);
        assert_eq!(
            out.severity,
            Severity::SubstantialBasisReductionMandatoryAdjustmentEvenWithout754
        );
        assert!(out.adjustment_required);
    }

    #[test]
    fn substantial_basis_reduction_boundary_exactly_250k_not_triggered() {
        let mut i = base_754_current();
        i.section_754_election_in_effect = false;
        i.distributee_gain_recognized_under_731a1_cents = 0;
        i.distributee_loss_recognized_under_731a2_cents = 25_000_000;
        i.partnership_adjusted_basis_in_distributed_property_cents = 0;
        i.distributee_basis_in_distributed_property_under_732_cents = 0;
        let out = check(&i);
        assert!(!out.substantial_basis_reduction_triggered);
        assert_eq!(
            out.severity,
            Severity::DistributionWithoutAdjustmentNo754NoSubstantialBasisReduction
        );
    }

    #[test]
    fn substantial_basis_reduction_boundary_250k_plus_one_cent_triggered() {
        let mut i = base_754_current();
        i.section_754_election_in_effect = false;
        i.distributee_gain_recognized_under_731a1_cents = 0;
        i.distributee_loss_recognized_under_731a2_cents = 25_000_001;
        i.partnership_adjusted_basis_in_distributed_property_cents = 0;
        i.distributee_basis_in_distributed_property_under_732_cents = 0;
        let out = check(&i);
        assert!(out.substantial_basis_reduction_triggered);
    }

    #[test]
    fn substantial_basis_reduction_via_property_step_down_alone() {
        let mut i = base_754_current();
        i.section_754_election_in_effect = false;
        i.distributee_gain_recognized_under_731a1_cents = 0;
        i.distributee_loss_recognized_under_731a2_cents = 0;
        i.partnership_adjusted_basis_in_distributed_property_cents = 0;
        i.distributee_basis_in_distributed_property_under_732_cents = 30_000_000;
        let out = check(&i);
        assert!(out.substantial_basis_reduction_triggered);
    }

    #[test]
    fn substantial_basis_reduction_via_sum_of_loss_and_step_down() {
        let mut i = base_754_current();
        i.section_754_election_in_effect = false;
        i.distributee_loss_recognized_under_731a2_cents = 15_000_000;
        i.partnership_adjusted_basis_in_distributed_property_cents = 0;
        i.distributee_basis_in_distributed_property_under_732_cents = 20_000_000;
        let out = check(&i);
        assert!(out.substantial_basis_reduction_triggered);
        assert_eq!(out.decrease_under_734b2_cents, 35_000_000);
    }

    #[test]
    fn not_applicable_returns_zero() {
        let mut i = base_754_current();
        i.distribution_type = DistributionType::NotApplicable;
        let out = check(&i);
        assert_eq!(out.severity, Severity::NotApplicable);
        assert_eq!(out.adjustment_cents, 0);
    }

    #[test]
    fn citations_pin_734a_734b_734b1_734b2_734d() {
        let out = check(&base_754_current());
        assert!(out.citations.iter().any(|c| c.contains("§ 734(a)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 734(b)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 734(b)(1)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 734(b)(2)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 734(d)")));
    }

    #[test]
    fn citations_pin_731a1_731a2_732_754_755() {
        let out = check(&base_754_current());
        assert!(out.citations.iter().any(|c| c.contains("§ 731(a)(1)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 731(a)(2)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 732")));
        assert!(out.citations.iter().any(|c| c.contains("§ 754")));
        assert!(out.citations.iter().any(|c| c.contains("§ 755")));
    }

    #[test]
    fn citations_pin_treas_reg_1_734_1_and_ajca_2004() {
        let out = check(&base_754_current());
        assert!(out.citations.iter().any(|c| c.contains("§ 1.734-1")));
        assert!(out.citations.iter().any(|c| c.contains("AJCA 2004")));
        assert!(out.citations.iter().any(|c| c.contains("§ 833")));
    }

    #[test]
    fn constant_pin_substantial_basis_reduction_250k() {
        assert_eq!(SUBSTANTIAL_BASIS_REDUCTION_THRESHOLD_CENTS, 25_000_000);
    }

    #[test]
    fn very_large_gain_saturating_no_overflow() {
        let mut i = base_754_current();
        i.distributee_gain_recognized_under_731a1_cents = u64::MAX;
        let out = check(&i);
        assert_eq!(out.increase_under_734b1_cents, u64::MAX);
    }

    #[test]
    fn liquidating_distribution_treated_same_as_current_for_734() {
        let mut i = base_754_current();
        i.distribution_type = DistributionType::LiquidatingDistribution;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::Section754PositiveIncreaseGainOrBasisStepUp
        );
    }

    #[test]
    fn gain_only_no_property_basis_difference_increase_equals_gain() {
        let mut i = base_754_current();
        i.partnership_adjusted_basis_in_distributed_property_cents = 3_000_000;
        i.distributee_basis_in_distributed_property_under_732_cents = 3_000_000;
        let out = check(&i);
        assert_eq!(out.increase_under_734b1_cents, 1_000_000);
        assert_eq!(out.adjustment_cents, 1_000_000);
    }

    #[test]
    fn step_up_only_no_distributee_gain_increase_equals_step_up() {
        let mut i = base_754_current();
        i.distributee_gain_recognized_under_731a1_cents = 0;
        let out = check(&i);
        assert_eq!(out.increase_under_734b1_cents, 2_000_000);
    }
}
