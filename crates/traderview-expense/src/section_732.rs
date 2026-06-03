//! IRC § 732 basis of distributed property other than money.
//!
//! Foundational distributee-partner basis-determination provision
//! that pairs with `section_731` (distribution nonrecognition rules)
//! and feeds into `section_734` (iter 578 distributee basis
//! adjustment) and `section_755` (iter 586 allocation methodology).
//!
//! **§ 732(a) current (nonliquidating) distribution**:
//!
//! - **§ 732(a)(1) general rule**: distributee takes the partnership's
//!   adjusted basis in the property immediately before distribution
//!   (carryover basis).
//! - **§ 732(a)(2) limitation**: basis to distributee CANNOT EXCEED
//!   the partner's outside basis in the partnership interest, reduced
//!   by any money distributed in the same transaction.
//!
//! **§ 732(b) liquidating distribution**: distributee takes a
//! "substituted basis" equal to the partner's outside basis in the
//! partnership interest minus any money distributed in the same
//! transaction. The substituted basis is the partner's investment in
//! the partnership "translated" into the distributed property.
//!
//! **§ 732(c) basis allocation among multiple properties**: when
//! more than one property is distributed, the available basis is
//! allocated in this order:
//!
//! - **§ 732(c)(1)(A)**: first to § 751(c) unrealized receivables and
//!   § 751(d) inventory items at the partnership's adjusted basis in
//!   each item.
//! - **§ 732(c)(1)(B)**: any remaining basis is allocated to other
//!   distributed property. If basis increase needed, allocated first
//!   to appreciated property up to FMV. If basis decrease needed,
//!   allocated first to depreciated property up to FMV. Then
//!   proportionally based on adjusted basis.
//!
//! **§ 732(d) special rule for transfers without § 754 election**:
//! a partner who acquired all or part of their interest by transfer
//! when § 754 was NOT in effect, and who receives a distribution
//! within **2 years** after such transfer, may ELECT to treat the
//! adjusted partnership basis of distributed property as if the
//! § 743(b) adjustment were in effect.
//!
//! **§ 732(d) mandatory application** (Treas. Reg. § 1.732-1(d)(4)):
//! special rule MUST be applied regardless of 2-year window if at
//! time of the original transfer ALL THREE conditions held: (i) FMV
//! of all partnership property exceeded **110%** of its adjusted
//! basis to the partnership; (ii) a § 732(c) allocation upon
//! liquidation would have resulted in a basis shift from non-
//! depreciable to depreciable property; (iii) a § 743(b) adjustment
//! would change the basis of the property actually distributed.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[allow(dead_code)]
pub const SECTION_732D_TWO_YEAR_WINDOW_MONTHS: u32 = 24;
#[allow(dead_code)]
pub const SECTION_732D_FMV_OVER_BASIS_THRESHOLD_PERCENT: u32 = 110;

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
    CurrentDistributionCarryoverBasis,
    CurrentDistributionLimitedByOutsideBasis,
    LiquidatingDistributionSubstitutedBasis,
    Section732cAllocationToHotAssetsThenOtherProperty,
    Section732dSpecialRuleElectiveBasisAdjustment,
    Section732dSpecialRuleMandatoryFmv110PercentTrigger,
    ViolationFailedToApplyMandatorySection732d,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub distribution_type: DistributionType,
    pub partnership_basis_in_property_cents: u64,
    pub partner_outside_basis_cents: u64,
    pub money_distributed_same_transaction_cents: u64,
    pub hot_assets_751c_751d_basis_cents: u64,
    pub other_property_basis_cents: u64,
    pub partner_acquired_via_transfer: bool,
    pub months_since_partner_transfer: u32,
    pub section_754_election_in_effect_at_transfer: bool,
    pub fmv_of_partnership_property_at_transfer_cents: u64,
    pub partnership_basis_at_transfer_cents: u64,
    pub section_732c_would_shift_basis_to_depreciable: bool,
    pub section_743b_would_change_distributed_property_basis: bool,
    pub taxpayer_applied_section_732d: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Output {
    pub severity: Severity,
    pub distributee_basis_in_property_cents: u64,
    pub hot_assets_allocation_cents: u64,
    pub other_property_allocation_cents: u64,
    pub section_732d_applies_mandatory: bool,
    pub section_732d_elective_available: bool,
    pub notes: Vec<String>,
    pub citations: Vec<String>,
}

pub type Section732Input = Input;
pub type Section732Output = Output;
pub type Section732Result = Output;

pub fn check(input: &Input) -> Output {
    let mut notes: Vec<String> = Vec::new();
    let citations: Vec<String> = vec![
        "IRC § 732(a)(1) (carryover basis general rule for current distributions)".to_string(),
        "IRC § 732(a)(2) (outside-basis limitation on current distributions)".to_string(),
        "IRC § 732(b) (substituted basis for liquidating distributions)".to_string(),
        "IRC § 732(c) (basis allocation among multiple properties)".to_string(),
        "IRC § 732(c)(1)(A) (first allocation to § 751(c)/(d) hot assets)".to_string(),
        "IRC § 732(c)(1)(B) (remaining allocation to other property)".to_string(),
        "IRC § 732(d) (special rule for distributions within 2 years of transfer without § 754)".to_string(),
        "IRC § 732(f) (corporate-partner distribution rule, post-2015)".to_string(),
        "IRC § 751(c) (unrealized receivables definition cross-reference)".to_string(),
        "IRC § 751(d) (inventory items definition cross-reference)".to_string(),
        "IRC § 743(b) (transferee basis adjustment cross-reference)".to_string(),
        "IRC § 754 (election cross-reference)".to_string(),
        "Treas. Reg. § 1.732-1 (general rules)".to_string(),
        "Treas. Reg. § 1.732-1(d)(4) (mandatory § 732(d) application — 110% trigger)".to_string(),
    ];

    if matches!(input.distribution_type, DistributionType::NotApplicable) {
        notes.push("No partnership distribution recorded.".to_string());
        return Output {
            severity: Severity::NotApplicable,
            distributee_basis_in_property_cents: 0,
            hot_assets_allocation_cents: 0,
            other_property_allocation_cents: 0,
            section_732d_applies_mandatory: false,
            section_732d_elective_available: false,
            notes,
            citations,
        };
    }

    let fmv_over_basis_pct = input
        .fmv_of_partnership_property_at_transfer_cents
        .saturating_mul(100)
        .checked_div(input.partnership_basis_at_transfer_cents)
        .unwrap_or(100);
    let mandatory_732d = input.partner_acquired_via_transfer
        && !input.section_754_election_in_effect_at_transfer
        && fmv_over_basis_pct as u32 > SECTION_732D_FMV_OVER_BASIS_THRESHOLD_PERCENT
        && input.section_732c_would_shift_basis_to_depreciable
        && input.section_743b_would_change_distributed_property_basis;
    let elective_732d = input.partner_acquired_via_transfer
        && !input.section_754_election_in_effect_at_transfer
        && input.months_since_partner_transfer <= SECTION_732D_TWO_YEAR_WINDOW_MONTHS;

    if mandatory_732d && !input.taxpayer_applied_section_732d {
        notes.push(format!(
            "Mandatory § 732(d) trigger: FMV-over-basis {}% exceeds {}% threshold + § 732(c) basis shift + § 743(b) would change distributed property basis. Taxpayer failed to apply.",
            fmv_over_basis_pct, SECTION_732D_FMV_OVER_BASIS_THRESHOLD_PERCENT
        ));
        return Output {
            severity: Severity::ViolationFailedToApplyMandatorySection732d,
            distributee_basis_in_property_cents: 0,
            hot_assets_allocation_cents: 0,
            other_property_allocation_cents: 0,
            section_732d_applies_mandatory: true,
            section_732d_elective_available: elective_732d,
            notes,
            citations,
        };
    }

    if mandatory_732d {
        notes.push(format!(
            "Mandatory § 732(d) applied: hypothetical § 743(b) adjustment imputed; FMV-over-basis {}% > {}% threshold satisfied.",
            fmv_over_basis_pct, SECTION_732D_FMV_OVER_BASIS_THRESHOLD_PERCENT
        ));
        return Output {
            severity: Severity::Section732dSpecialRuleMandatoryFmv110PercentTrigger,
            distributee_basis_in_property_cents: input.partnership_basis_in_property_cents,
            hot_assets_allocation_cents: input.hot_assets_751c_751d_basis_cents,
            other_property_allocation_cents: input.other_property_basis_cents,
            section_732d_applies_mandatory: true,
            section_732d_elective_available: elective_732d,
            notes,
            citations,
        };
    }

    if elective_732d && input.taxpayer_applied_section_732d {
        notes.push(format!(
            "Elective § 732(d) applied: distribution within {}-month window of transfer without § 754; hypothetical § 743(b) adjustment imputed.",
            SECTION_732D_TWO_YEAR_WINDOW_MONTHS
        ));
        return Output {
            severity: Severity::Section732dSpecialRuleElectiveBasisAdjustment,
            distributee_basis_in_property_cents: input.partnership_basis_in_property_cents,
            hot_assets_allocation_cents: input.hot_assets_751c_751d_basis_cents,
            other_property_allocation_cents: input.other_property_basis_cents,
            section_732d_applies_mandatory: false,
            section_732d_elective_available: true,
            notes,
            citations,
        };
    }

    let has_multiple_property_classes = input.hot_assets_751c_751d_basis_cents > 0
        && input.other_property_basis_cents > 0;

    if has_multiple_property_classes {
        let available_basis = match input.distribution_type {
            DistributionType::LiquidatingDistribution => input
                .partner_outside_basis_cents
                .saturating_sub(input.money_distributed_same_transaction_cents),
            _ => input.partner_outside_basis_cents.min(
                input
                    .partnership_basis_in_property_cents
                    .saturating_sub(input.money_distributed_same_transaction_cents),
            ),
        };
        let hot_allocation = input.hot_assets_751c_751d_basis_cents.min(available_basis);
        let other_allocation = available_basis.saturating_sub(hot_allocation);
        notes.push(format!(
            "§ 732(c) two-step allocation: ${} to hot assets first; ${} remaining allocated to other distributed property.",
            hot_allocation / 100,
            other_allocation / 100
        ));
        return Output {
            severity: Severity::Section732cAllocationToHotAssetsThenOtherProperty,
            distributee_basis_in_property_cents: hot_allocation.saturating_add(other_allocation),
            hot_assets_allocation_cents: hot_allocation,
            other_property_allocation_cents: other_allocation,
            section_732d_applies_mandatory: false,
            section_732d_elective_available: elective_732d,
            notes,
            citations,
        };
    }

    if matches!(input.distribution_type, DistributionType::LiquidatingDistribution) {
        let substituted_basis = input
            .partner_outside_basis_cents
            .saturating_sub(input.money_distributed_same_transaction_cents);
        notes.push(format!(
            "§ 732(b) liquidating distribution: substituted basis = outside basis ${} − money ${} = ${}.",
            input.partner_outside_basis_cents / 100,
            input.money_distributed_same_transaction_cents / 100,
            substituted_basis / 100
        ));
        return Output {
            severity: Severity::LiquidatingDistributionSubstitutedBasis,
            distributee_basis_in_property_cents: substituted_basis,
            hot_assets_allocation_cents: 0,
            other_property_allocation_cents: substituted_basis,
            section_732d_applies_mandatory: false,
            section_732d_elective_available: elective_732d,
            notes,
            citations,
        };
    }

    let outside_basis_minus_money = input
        .partner_outside_basis_cents
        .saturating_sub(input.money_distributed_same_transaction_cents);
    if input.partnership_basis_in_property_cents > outside_basis_minus_money {
        notes.push(format!(
            "§ 732(a)(2) limitation: partnership basis ${} exceeds outside basis less money ${}; distributee basis capped at ${}.",
            input.partnership_basis_in_property_cents / 100,
            outside_basis_minus_money / 100,
            outside_basis_minus_money / 100
        ));
        return Output {
            severity: Severity::CurrentDistributionLimitedByOutsideBasis,
            distributee_basis_in_property_cents: outside_basis_minus_money,
            hot_assets_allocation_cents: 0,
            other_property_allocation_cents: outside_basis_minus_money,
            section_732d_applies_mandatory: false,
            section_732d_elective_available: elective_732d,
            notes,
            citations,
        };
    }

    notes.push(format!(
        "§ 732(a)(1) carryover basis: distributee takes partnership's ${} adjusted basis; outside basis ${} not exceeded.",
        input.partnership_basis_in_property_cents / 100,
        input.partner_outside_basis_cents / 100
    ));
    Output {
        severity: Severity::CurrentDistributionCarryoverBasis,
        distributee_basis_in_property_cents: input.partnership_basis_in_property_cents,
        hot_assets_allocation_cents: 0,
        other_property_allocation_cents: input.partnership_basis_in_property_cents,
        section_732d_applies_mandatory: false,
        section_732d_elective_available: elective_732d,
        notes,
        citations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_current_carryover() -> Input {
        Input {
            distribution_type: DistributionType::CurrentDistribution,
            partnership_basis_in_property_cents: 3_000_000,
            partner_outside_basis_cents: 10_000_000,
            money_distributed_same_transaction_cents: 0,
            hot_assets_751c_751d_basis_cents: 0,
            other_property_basis_cents: 3_000_000,
            partner_acquired_via_transfer: false,
            months_since_partner_transfer: 0,
            section_754_election_in_effect_at_transfer: false,
            fmv_of_partnership_property_at_transfer_cents: 0,
            partnership_basis_at_transfer_cents: 0,
            section_732c_would_shift_basis_to_depreciable: false,
            section_743b_would_change_distributed_property_basis: false,
            taxpayer_applied_section_732d: false,
        }
    }

    #[test]
    fn current_distribution_carryover_when_outside_basis_sufficient() {
        let out = check(&base_current_carryover());
        assert_eq!(out.severity, Severity::CurrentDistributionCarryoverBasis);
        assert_eq!(out.distributee_basis_in_property_cents, 3_000_000);
    }

    #[test]
    fn current_distribution_limited_by_outside_basis() {
        let mut i = base_current_carryover();
        i.partner_outside_basis_cents = 2_000_000;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::CurrentDistributionLimitedByOutsideBasis
        );
        assert_eq!(out.distributee_basis_in_property_cents, 2_000_000);
    }

    #[test]
    fn current_distribution_outside_basis_reduced_by_money() {
        let mut i = base_current_carryover();
        i.partner_outside_basis_cents = 4_000_000;
        i.money_distributed_same_transaction_cents = 2_000_000;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::CurrentDistributionLimitedByOutsideBasis
        );
        assert_eq!(out.distributee_basis_in_property_cents, 2_000_000);
    }

    #[test]
    fn liquidating_distribution_substituted_basis() {
        let mut i = base_current_carryover();
        i.distribution_type = DistributionType::LiquidatingDistribution;
        i.partner_outside_basis_cents = 5_000_000;
        i.money_distributed_same_transaction_cents = 1_000_000;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::LiquidatingDistributionSubstitutedBasis
        );
        assert_eq!(out.distributee_basis_in_property_cents, 4_000_000);
    }

    #[test]
    fn liquidating_distribution_no_money_uses_full_outside_basis() {
        let mut i = base_current_carryover();
        i.distribution_type = DistributionType::LiquidatingDistribution;
        i.partner_outside_basis_cents = 7_000_000;
        i.money_distributed_same_transaction_cents = 0;
        let out = check(&i);
        assert_eq!(out.distributee_basis_in_property_cents, 7_000_000);
    }

    #[test]
    fn section_732c_allocation_hot_assets_first_then_other() {
        let mut i = base_current_carryover();
        i.distribution_type = DistributionType::LiquidatingDistribution;
        i.hot_assets_751c_751d_basis_cents = 1_500_000;
        i.other_property_basis_cents = 2_000_000;
        i.partner_outside_basis_cents = 4_000_000;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::Section732cAllocationToHotAssetsThenOtherProperty
        );
        assert_eq!(out.hot_assets_allocation_cents, 1_500_000);
        assert_eq!(out.other_property_allocation_cents, 2_500_000);
    }

    #[test]
    fn section_732d_mandatory_when_110_pct_fmv_trigger_met() {
        let mut i = base_current_carryover();
        i.partner_acquired_via_transfer = true;
        i.section_754_election_in_effect_at_transfer = false;
        i.fmv_of_partnership_property_at_transfer_cents = 15_000_000;
        i.partnership_basis_at_transfer_cents = 10_000_000;
        i.section_732c_would_shift_basis_to_depreciable = true;
        i.section_743b_would_change_distributed_property_basis = true;
        i.taxpayer_applied_section_732d = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::Section732dSpecialRuleMandatoryFmv110PercentTrigger
        );
        assert!(out.section_732d_applies_mandatory);
    }

    #[test]
    fn section_732d_mandatory_violation_when_taxpayer_did_not_apply() {
        let mut i = base_current_carryover();
        i.partner_acquired_via_transfer = true;
        i.section_754_election_in_effect_at_transfer = false;
        i.fmv_of_partnership_property_at_transfer_cents = 15_000_000;
        i.partnership_basis_at_transfer_cents = 10_000_000;
        i.section_732c_would_shift_basis_to_depreciable = true;
        i.section_743b_would_change_distributed_property_basis = true;
        i.taxpayer_applied_section_732d = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationFailedToApplyMandatorySection732d
        );
    }

    #[test]
    fn section_732d_elective_when_within_2_year_window() {
        let mut i = base_current_carryover();
        i.partner_acquired_via_transfer = true;
        i.section_754_election_in_effect_at_transfer = false;
        i.months_since_partner_transfer = 18;
        i.fmv_of_partnership_property_at_transfer_cents = 11_000_000;
        i.partnership_basis_at_transfer_cents = 10_000_000;
        i.taxpayer_applied_section_732d = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::Section732dSpecialRuleElectiveBasisAdjustment
        );
        assert!(out.section_732d_elective_available);
    }

    #[test]
    fn section_732d_24_month_boundary_in_window() {
        let mut i = base_current_carryover();
        i.partner_acquired_via_transfer = true;
        i.section_754_election_in_effect_at_transfer = false;
        i.months_since_partner_transfer = 24;
        let out = check(&i);
        assert!(out.section_732d_elective_available);
    }

    #[test]
    fn section_732d_25_month_boundary_outside_window() {
        let mut i = base_current_carryover();
        i.partner_acquired_via_transfer = true;
        i.section_754_election_in_effect_at_transfer = false;
        i.months_since_partner_transfer = 25;
        let out = check(&i);
        assert!(!out.section_732d_elective_available);
    }

    #[test]
    fn section_732d_110_pct_boundary_at_exactly_110_not_mandatory() {
        let mut i = base_current_carryover();
        i.partner_acquired_via_transfer = true;
        i.section_754_election_in_effect_at_transfer = false;
        i.fmv_of_partnership_property_at_transfer_cents = 11_000_000;
        i.partnership_basis_at_transfer_cents = 10_000_000;
        i.section_732c_would_shift_basis_to_depreciable = true;
        i.section_743b_would_change_distributed_property_basis = true;
        let out = check(&i);
        assert!(!out.section_732d_applies_mandatory);
    }

    #[test]
    fn section_732d_111_pct_just_over_threshold_mandatory() {
        let mut i = base_current_carryover();
        i.partner_acquired_via_transfer = true;
        i.section_754_election_in_effect_at_transfer = false;
        i.fmv_of_partnership_property_at_transfer_cents = 11_100_000;
        i.partnership_basis_at_transfer_cents = 10_000_000;
        i.section_732c_would_shift_basis_to_depreciable = true;
        i.section_743b_would_change_distributed_property_basis = true;
        i.taxpayer_applied_section_732d = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::Section732dSpecialRuleMandatoryFmv110PercentTrigger
        );
    }

    #[test]
    fn not_applicable_returns_default() {
        let mut i = base_current_carryover();
        i.distribution_type = DistributionType::NotApplicable;
        let out = check(&i);
        assert_eq!(out.severity, Severity::NotApplicable);
    }

    #[test]
    fn citations_pin_732_subsections() {
        let out = check(&base_current_carryover());
        assert!(out.citations.iter().any(|c| c.contains("§ 732(a)(1)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 732(a)(2)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 732(b)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 732(c)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 732(c)(1)(A)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 732(c)(1)(B)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 732(d)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 732(f)")));
    }

    #[test]
    fn citations_pin_cross_refs_751_743_754_treas_reg() {
        let out = check(&base_current_carryover());
        assert!(out.citations.iter().any(|c| c.contains("§ 751(c)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 751(d)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 743(b)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 754")));
        assert!(out.citations.iter().any(|c| c.contains("§ 1.732-1")));
        assert!(out.citations.iter().any(|c| c.contains("§ 1.732-1(d)(4)")));
    }

    #[test]
    fn constant_pin_24_month_732d_window() {
        assert_eq!(SECTION_732D_TWO_YEAR_WINDOW_MONTHS, 24);
    }

    #[test]
    fn constant_pin_110_pct_732d_trigger() {
        assert_eq!(SECTION_732D_FMV_OVER_BASIS_THRESHOLD_PERCENT, 110);
    }

    #[test]
    fn very_large_outside_basis_no_overflow() {
        let mut i = base_current_carryover();
        i.distribution_type = DistributionType::LiquidatingDistribution;
        i.partner_outside_basis_cents = u64::MAX;
        let out = check(&i);
        assert_eq!(out.distributee_basis_in_property_cents, u64::MAX);
    }
}
