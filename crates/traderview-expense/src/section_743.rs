//! IRC § 743 transferee partner basis adjustment on partnership-
//! interest transfer (sale, exchange, or death of partner).
//!
//! **§ 743(a) general rule**: no basis adjustment to partnership
//! property on transfer of partnership interest.
//!
//! **§ 743(b) exception**: basis adjustment REQUIRED if EITHER (1) a
//! § 754 election is in effect for the partnership OR (2) the
//! partnership has a "substantial built-in loss" immediately after
//! the transfer.
//!
//! **§ 743(b) adjustment math**:
//! - INCREASE inside basis by excess of transferee's OUTSIDE basis
//!   OVER transferee's proportionate share of INSIDE basis.
//! - DECREASE inside basis by excess of transferee's proportionate
//!   share of INSIDE basis OVER transferee's OUTSIDE basis.
//! - The adjustment is partner-specific (transferee only); does not
//!   affect non-transferee partners' allocations.
//!
//! **§ 743(d) substantial built-in loss** (TCJA 2017 amended to add
//! the transferee-specific prong, applicable to transfers after
//! December 31, 2017): substantial built-in loss exists if EITHER
//!
//! 1. partnership's adjusted basis in property EXCEEDS its fair
//!    market value by MORE THAN $250,000 (partnership-level prong); or
//! 2. the transferee partner would be allocated a loss of MORE THAN
//!    $250,000 if the partnership sold all assets at FMV immediately
//!    after the transfer (transferee-specific prong, TCJA 2017).
//!
//! Pairs with `section_754` (election mechanics), `section_734`
//! (distributee basis adjustment under § 754), `section_755`
//! (allocation of § 743(b)/§ 734 adjustments among partnership
//! properties).

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[allow(dead_code)]
pub const SUBSTANTIAL_BUILT_IN_LOSS_PARTNERSHIP_THRESHOLD_CENTS: u64 = 25_000_000;
#[allow(dead_code)]
pub const SUBSTANTIAL_BUILT_IN_LOSS_TRANSFEREE_THRESHOLD_CENTS: u64 = 25_000_000;
#[allow(dead_code)]
pub const TCJA_2017_AMENDMENT_TRANSFEREE_PRONG_EFFECTIVE_AFTER_YEAR: u32 = 2017;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransferEvent {
    NotApplicable,
    SaleOrExchange,
    DeathOfPartner,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    TransferWithoutAdjustmentNo754NoBuiltInLoss,
    Section754ElectionPositiveStepUpInsideBasisIncrease,
    Section754ElectionNegativeStepDownInsideBasisDecrease,
    Section754ElectionNoNetAdjustment,
    SubstantialBuiltInLossMandatoryAdjustmentEvenWithout754,
    TransferAtDeathSection754SteppedUpOutsideBasisIfElection,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub transfer_event: TransferEvent,
    pub section_754_election_in_effect: bool,
    pub transfer_year: u32,
    pub transferee_outside_basis_cents: u64,
    pub transferee_proportionate_share_of_inside_basis_cents: u64,
    pub partnership_aggregate_inside_basis_cents: u64,
    pub partnership_aggregate_fmv_cents: u64,
    pub transferee_allocable_loss_if_assets_sold_at_fmv_cents: u64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Output {
    pub severity: Severity,
    pub adjustment_required: bool,
    pub adjustment_cents: i128,
    pub substantial_built_in_loss_partnership_prong_met: bool,
    pub substantial_built_in_loss_transferee_prong_met: bool,
    pub notes: Vec<String>,
    pub citations: Vec<String>,
}

pub type Section743Input = Input;
pub type Section743Output = Output;
pub type Section743Result = Output;

pub fn check(input: &Input) -> Output {
    let mut notes: Vec<String> = Vec::new();
    let citations: Vec<String> = vec![
        "IRC § 743(a) (general rule — no basis adjustment)".to_string(),
        "IRC § 743(b) (exception — § 754 election OR substantial built-in loss)".to_string(),
        "IRC § 743(b)(1) (increase in basis)".to_string(),
        "IRC § 743(b)(2) (decrease in basis)".to_string(),
        "IRC § 743(d) (substantial built-in loss two-prong test)".to_string(),
        "IRC § 754 (election mechanics)".to_string(),
        "IRC § 755 (allocation among partnership properties)".to_string(),
        "Treas. Reg. § 1.743-1".to_string(),
        "TCJA 2017 § 13502 (added § 743(d)(1)(B) transferee prong, eff. transfers after Dec. 31, 2017)".to_string(),
        "IRS FAQs for IRC § 754 election and revocation".to_string(),
    ];

    if matches!(input.transfer_event, TransferEvent::NotApplicable) {
        notes.push("No partnership-interest transfer event recorded.".to_string());
        return Output {
            severity: Severity::NotApplicable,
            adjustment_required: false,
            adjustment_cents: 0,
            substantial_built_in_loss_partnership_prong_met: false,
            substantial_built_in_loss_transferee_prong_met: false,
            notes,
            citations,
        };
    }

    let partnership_built_in_loss = input
        .partnership_aggregate_inside_basis_cents
        .saturating_sub(input.partnership_aggregate_fmv_cents);
    let partnership_prong_met =
        partnership_built_in_loss > SUBSTANTIAL_BUILT_IN_LOSS_PARTNERSHIP_THRESHOLD_CENTS;
    let transferee_prong_met = input.transfer_year
        > TCJA_2017_AMENDMENT_TRANSFEREE_PRONG_EFFECTIVE_AFTER_YEAR
        && input.transferee_allocable_loss_if_assets_sold_at_fmv_cents
            > SUBSTANTIAL_BUILT_IN_LOSS_TRANSFEREE_THRESHOLD_CENTS;
    let substantial_built_in_loss = partnership_prong_met || transferee_prong_met;

    if !input.section_754_election_in_effect && !substantial_built_in_loss {
        notes.push("No § 754 election in effect and no substantial built-in loss — § 743(a) default applies; no basis adjustment.".to_string());
        return Output {
            severity: Severity::TransferWithoutAdjustmentNo754NoBuiltInLoss,
            adjustment_required: false,
            adjustment_cents: 0,
            substantial_built_in_loss_partnership_prong_met: partnership_prong_met,
            substantial_built_in_loss_transferee_prong_met: transferee_prong_met,
            notes,
            citations,
        };
    }

    let outside_basis = input.transferee_outside_basis_cents as i128;
    let inside_basis_share = input.transferee_proportionate_share_of_inside_basis_cents as i128;
    let adjustment = outside_basis - inside_basis_share;

    if substantial_built_in_loss && !input.section_754_election_in_effect {
        notes.push(format!(
            "Mandatory § 743(b) adjustment without § 754 election (substantial built-in loss): partnership-prong {} (excess ${}); transferee-prong {} (TCJA 2017). Adjustment ${} (decrease).",
            partnership_prong_met,
            partnership_built_in_loss / 100,
            transferee_prong_met,
            adjustment / 100
        ));
        return Output {
            severity: Severity::SubstantialBuiltInLossMandatoryAdjustmentEvenWithout754,
            adjustment_required: true,
            adjustment_cents: adjustment,
            substantial_built_in_loss_partnership_prong_met: partnership_prong_met,
            substantial_built_in_loss_transferee_prong_met: transferee_prong_met,
            notes,
            citations,
        };
    }

    if matches!(input.transfer_event, TransferEvent::DeathOfPartner) {
        notes.push(format!(
            "Death of partner + § 754 election: § 1014 outside-basis step-up to FMV; transferee inside-basis adjustment ${}.",
            adjustment / 100
        ));
        return Output {
            severity: Severity::TransferAtDeathSection754SteppedUpOutsideBasisIfElection,
            adjustment_required: true,
            adjustment_cents: adjustment,
            substantial_built_in_loss_partnership_prong_met: partnership_prong_met,
            substantial_built_in_loss_transferee_prong_met: transferee_prong_met,
            notes,
            citations,
        };
    }

    let severity = if adjustment > 0 {
        notes.push(format!(
            "§ 754 election in effect + transferee outside basis ${} > inside-basis share ${} = INCREASE by ${}.",
            outside_basis / 100,
            inside_basis_share / 100,
            adjustment / 100
        ));
        Severity::Section754ElectionPositiveStepUpInsideBasisIncrease
    } else if adjustment < 0 {
        notes.push(format!(
            "§ 754 election in effect + transferee outside basis ${} < inside-basis share ${} = DECREASE by ${}.",
            outside_basis / 100,
            inside_basis_share / 100,
            (-adjustment) / 100
        ));
        Severity::Section754ElectionNegativeStepDownInsideBasisDecrease
    } else {
        notes.push(
            "§ 754 election in effect but no net adjustment (outside basis = inside-basis share)."
                .to_string(),
        );
        Severity::Section754ElectionNoNetAdjustment
    };

    Output {
        severity,
        adjustment_required: adjustment != 0,
        adjustment_cents: adjustment,
        substantial_built_in_loss_partnership_prong_met: partnership_prong_met,
        substantial_built_in_loss_transferee_prong_met: transferee_prong_met,
        notes,
        citations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_754_sale() -> Input {
        Input {
            transfer_event: TransferEvent::SaleOrExchange,
            section_754_election_in_effect: true,
            transfer_year: 2026,
            transferee_outside_basis_cents: 10_000_000,
            transferee_proportionate_share_of_inside_basis_cents: 7_000_000,
            partnership_aggregate_inside_basis_cents: 20_000_000,
            partnership_aggregate_fmv_cents: 30_000_000,
            transferee_allocable_loss_if_assets_sold_at_fmv_cents: 0,
        }
    }

    #[test]
    fn sale_with_754_outside_above_inside_step_up() {
        let out = check(&base_754_sale());
        assert_eq!(
            out.severity,
            Severity::Section754ElectionPositiveStepUpInsideBasisIncrease
        );
        assert_eq!(out.adjustment_cents, 3_000_000);
        assert!(out.adjustment_required);
    }

    #[test]
    fn sale_with_754_outside_below_inside_step_down() {
        let mut i = base_754_sale();
        i.transferee_outside_basis_cents = 5_000_000;
        i.transferee_proportionate_share_of_inside_basis_cents = 7_000_000;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::Section754ElectionNegativeStepDownInsideBasisDecrease
        );
        assert_eq!(out.adjustment_cents, -2_000_000);
    }

    #[test]
    fn sale_with_754_outside_equal_inside_no_net_adjustment() {
        let mut i = base_754_sale();
        i.transferee_outside_basis_cents = 7_000_000;
        let out = check(&i);
        assert_eq!(out.severity, Severity::Section754ElectionNoNetAdjustment);
        assert_eq!(out.adjustment_cents, 0);
        assert!(!out.adjustment_required);
    }

    #[test]
    fn sale_without_754_no_built_in_loss_no_adjustment() {
        let mut i = base_754_sale();
        i.section_754_election_in_effect = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::TransferWithoutAdjustmentNo754NoBuiltInLoss
        );
        assert!(!out.adjustment_required);
    }

    #[test]
    fn substantial_built_in_loss_partnership_prong_mandatory_adjustment() {
        let mut i = base_754_sale();
        i.section_754_election_in_effect = false;
        i.partnership_aggregate_inside_basis_cents = 50_000_000;
        i.partnership_aggregate_fmv_cents = 10_000_000;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::SubstantialBuiltInLossMandatoryAdjustmentEvenWithout754
        );
        assert!(out.substantial_built_in_loss_partnership_prong_met);
        assert!(out.adjustment_required);
    }

    #[test]
    fn substantial_built_in_loss_partnership_boundary_exactly_250k_not_met() {
        let mut i = base_754_sale();
        i.section_754_election_in_effect = false;
        i.partnership_aggregate_inside_basis_cents = 35_000_000;
        i.partnership_aggregate_fmv_cents = 10_000_000;
        let out = check(&i);
        assert!(!out.substantial_built_in_loss_partnership_prong_met);
        assert_eq!(
            out.severity,
            Severity::TransferWithoutAdjustmentNo754NoBuiltInLoss
        );
    }

    #[test]
    fn substantial_built_in_loss_partnership_boundary_250k_plus_one_cent_met() {
        let mut i = base_754_sale();
        i.section_754_election_in_effect = false;
        i.partnership_aggregate_inside_basis_cents = 35_000_001;
        i.partnership_aggregate_fmv_cents = 10_000_000;
        let out = check(&i);
        assert!(out.substantial_built_in_loss_partnership_prong_met);
    }

    #[test]
    fn substantial_built_in_loss_transferee_prong_tcja_2017_post() {
        let mut i = base_754_sale();
        i.section_754_election_in_effect = false;
        i.transfer_year = 2026;
        i.transferee_allocable_loss_if_assets_sold_at_fmv_cents = 30_000_000;
        let out = check(&i);
        assert!(out.substantial_built_in_loss_transferee_prong_met);
        assert_eq!(
            out.severity,
            Severity::SubstantialBuiltInLossMandatoryAdjustmentEvenWithout754
        );
    }

    #[test]
    fn substantial_built_in_loss_transferee_prong_pre_tcja_2017_not_applicable() {
        let mut i = base_754_sale();
        i.section_754_election_in_effect = false;
        i.transfer_year = 2017;
        i.transferee_allocable_loss_if_assets_sold_at_fmv_cents = 30_000_000;
        let out = check(&i);
        assert!(!out.substantial_built_in_loss_transferee_prong_met);
    }

    #[test]
    fn death_of_partner_with_754_steps_up_outside_basis() {
        let mut i = base_754_sale();
        i.transfer_event = TransferEvent::DeathOfPartner;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::TransferAtDeathSection754SteppedUpOutsideBasisIfElection
        );
    }

    #[test]
    fn not_applicable_returns_zero_adjustment() {
        let mut i = base_754_sale();
        i.transfer_event = TransferEvent::NotApplicable;
        let out = check(&i);
        assert_eq!(out.severity, Severity::NotApplicable);
        assert_eq!(out.adjustment_cents, 0);
    }

    #[test]
    fn citations_pin_743a_743b_743d_754_755() {
        let out = check(&base_754_sale());
        assert!(out.citations.iter().any(|c| c.contains("§ 743(a)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 743(b)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 743(d)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 754")));
        assert!(out.citations.iter().any(|c| c.contains("§ 755")));
    }

    #[test]
    fn citations_pin_treas_reg_and_tcja_2017() {
        let out = check(&base_754_sale());
        assert!(out.citations.iter().any(|c| c.contains("§ 1.743-1")));
        assert!(out.citations.iter().any(|c| c.contains("TCJA 2017")));
        assert!(out.citations.iter().any(|c| c.contains("§ 13502")));
    }

    #[test]
    fn constant_pin_partnership_prong_250k_threshold() {
        assert_eq!(
            SUBSTANTIAL_BUILT_IN_LOSS_PARTNERSHIP_THRESHOLD_CENTS,
            25_000_000
        );
    }

    #[test]
    fn constant_pin_transferee_prong_250k_threshold() {
        assert_eq!(
            SUBSTANTIAL_BUILT_IN_LOSS_TRANSFEREE_THRESHOLD_CENTS,
            25_000_000
        );
    }

    #[test]
    fn constant_pin_tcja_2017_effective_year_cutoff() {
        assert_eq!(
            TCJA_2017_AMENDMENT_TRANSFEREE_PRONG_EFFECTIVE_AFTER_YEAR,
            2017
        );
    }

    #[test]
    fn very_large_outside_basis_no_overflow() {
        let mut i = base_754_sale();
        i.transferee_outside_basis_cents = u64::MAX / 2;
        let out = check(&i);
        assert!(out.adjustment_cents > 0);
    }

    #[test]
    fn both_substantial_built_in_loss_prongs_met_only_recorded_once() {
        let mut i = base_754_sale();
        i.section_754_election_in_effect = false;
        i.partnership_aggregate_inside_basis_cents = 50_000_000;
        i.partnership_aggregate_fmv_cents = 10_000_000;
        i.transferee_allocable_loss_if_assets_sold_at_fmv_cents = 30_000_000;
        let out = check(&i);
        assert!(out.substantial_built_in_loss_partnership_prong_met);
        assert!(out.substantial_built_in_loss_transferee_prong_met);
        assert_eq!(
            out.severity,
            Severity::SubstantialBuiltInLossMandatoryAdjustmentEvenWithout754
        );
    }

    #[test]
    fn death_with_754_step_down_still_records_negative_adjustment() {
        let mut i = base_754_sale();
        i.transfer_event = TransferEvent::DeathOfPartner;
        i.transferee_outside_basis_cents = 5_000_000;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::TransferAtDeathSection754SteppedUpOutsideBasisIfElection
        );
        assert_eq!(out.adjustment_cents, -2_000_000);
    }
}
