//! IRC §754 partnership election + §743(b) inside-basis adjustment.
//!
//! Critical for any trader investing in LP / LLC partnerships
//! (hedge funds, PE funds, real-estate syndications) where there's a
//! gap between inside basis (partnership's adjusted basis in its
//! assets) and outside basis (partner's basis in their partnership
//! interest).
//!
//! **§754 election** (partnership-level, irrevocable except by IRS
//! consent) unlocks two basis-adjustment mechanisms:
//! - **§743(b)** — adjusts inside basis when an interest is
//!   TRANSFERRED (sale or exchange, or death of partner)
//! - **§734(b)** — adjusts inside basis on certain DISTRIBUTIONS
//!
//! This module handles §743(b) (the more common trader path: buying
//! a hedge-fund interest from an exiting LP, inheriting an interest,
//! etc.) with the mandatory-adjustment substantial-built-in-loss
//! override.
//!
//! **§743(b) adjustment formula**:
//!
//! ```text
//! §743(b) adjustment = transferee outside basis − transferee
//!                      proportionate share of partnership's
//!                      adjusted inside basis
//! ```
//!
//! - Positive → STEP-UP allocated to appreciated assets under §755
//! - Negative → STEP-DOWN allocated to depreciated assets under §755
//! - Adjustment is PERSONAL to the transferee; other partners are
//!   unaffected
//!
//! **§743(d) mandatory adjustment** (TCJA expanded — post-2017):
//! adjustment is REQUIRED even WITHOUT a §754 election if EITHER of
//! these substantial-built-in-loss tests is satisfied:
//!
//! 1. **Partnership-level test (§743(d)(1)(A))**: partnership's
//!    adjusted inside basis EXCEEDS partnership-level FMV by more
//!    than $250,000.
//! 2. **Transferee-level test (§743(d)(1)(B), TCJA addition)**: the
//!    transferee partner would be allocated a loss greater than
//!    $250,000 if all partnership assets were sold for cash equal to
//!    FMV immediately after the transfer.
//!
//! When either test fires, §743(b) MUST be applied even though no
//! §754 election is on file. Pre-2017 only test 1 applied; TCJA's
//! addition of test 2 closes the "loss-trafficking transferee"
//! loophole.
//!
//! **Without §754 election AND no mandatory adjustment**: §743(b)
//! does NOT apply. Transferee inherits their seller's inside-basis
//! share unchanged — and bears phantom income equal to the basis
//! gap when the partnership sells appreciated assets.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdjustmentDirection {
    StepUp,
    StepDown,
    NoAdjustment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransferType {
    /// Sale or exchange of partnership interest (most common).
    SaleOrExchange,
    /// Death of partner — interest transfers to estate / heir, takes
    /// FMV basis under §1014.
    DeathOfPartner,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section754Input {
    pub transfer_type: TransferType,
    pub election_in_effect: bool,
    pub transferee_outside_basis: Decimal,
    pub transferee_share_of_inside_basis: Decimal,
    pub partnership_total_inside_basis: Decimal,
    pub partnership_total_fmv: Decimal,
    /// Loss the transferee would be allocated if partnership assets
    /// were immediately sold at FMV after the transfer. Used for
    /// the §743(d)(1)(B) transferee-level mandatory test.
    pub transferee_hypothetical_loss_on_immediate_sale: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section754Result {
    pub adjustment_amount: Decimal,
    pub adjustment_direction: AdjustmentDirection,
    /// Partnership-level: inside basis − FMV > $250k.
    pub mandatory_test_a_partnership_bil: bool,
    /// Transferee-level: hypothetical loss > $250k.
    pub mandatory_test_b_transferee_loss: bool,
    pub mandatory_under_substantial_bil: bool,
    pub election_required_for_adjustment: bool,
    pub adjustment_applies: bool,
    pub citation: String,
    pub note: String,
}

/// §743(d) substantial-built-in-loss threshold: the test fires when
/// the relevant amount EXCEEDS $250,000 (strict greater-than).
const SUBSTANTIAL_BIL_THRESHOLD_DOLLARS: i64 = 250_000;

pub fn compute(input: &Section754Input) -> Section754Result {
    let threshold = Decimal::from(SUBSTANTIAL_BIL_THRESHOLD_DOLLARS);
    let raw_adjustment =
        input.transferee_outside_basis - input.transferee_share_of_inside_basis;

    // §743(d) mandatory adjustment tests.
    let partnership_bil_excess =
        input.partnership_total_inside_basis - input.partnership_total_fmv;
    let test_a = partnership_bil_excess > threshold;
    let test_b = input.transferee_hypothetical_loss_on_immediate_sale > threshold;
    let mandatory = test_a || test_b;

    // Adjustment applies IF §754 election in effect OR mandatory.
    let adjustment_applies = input.election_in_effect || mandatory;
    let election_required = !mandatory && !input.election_in_effect;

    let (adjustment, direction) = if !adjustment_applies {
        (Decimal::ZERO, AdjustmentDirection::NoAdjustment)
    } else if raw_adjustment > Decimal::ZERO {
        (raw_adjustment, AdjustmentDirection::StepUp)
    } else if raw_adjustment < Decimal::ZERO {
        (raw_adjustment, AdjustmentDirection::StepDown)
    } else {
        (Decimal::ZERO, AdjustmentDirection::NoAdjustment)
    };

    let mandatory_basis = if mandatory {
        let mut reasons: Vec<String> = Vec::new();
        if test_a {
            reasons.push(format!(
                "§743(d)(1)(A) partnership BIL excess ${} > $250k",
                partnership_bil_excess.round_dp(2)
            ));
        }
        if test_b {
            reasons.push(format!(
                "§743(d)(1)(B) transferee loss ${} > $250k",
                input.transferee_hypothetical_loss_on_immediate_sale.round_dp(2)
            ));
        }
        format!(" MANDATORY: {}", reasons.join("; "))
    } else {
        String::new()
    };

    let path_label = match (input.election_in_effect, mandatory) {
        (true, _) => "§754 election in effect",
        (false, true) => "§754 election NOT in effect but §743(d) substantial BIL forces mandatory adjustment",
        (false, false) => "§754 election NOT in effect and no §743(d) trigger — §743(b) does NOT apply",
    };

    let transfer_label = match input.transfer_type {
        TransferType::SaleOrExchange => "sale/exchange",
        TransferType::DeathOfPartner => "death of partner (heir takes §1014 FMV outside basis)",
    };

    let note = format!(
        "{}; transferee outside basis ${}, share of inside basis ${}, raw §743(b) adjustment ${} ({:?}).{} Path: {}.",
        transfer_label,
        input.transferee_outside_basis.round_dp(2),
        input.transferee_share_of_inside_basis.round_dp(2),
        raw_adjustment.round_dp(2),
        direction,
        mandatory_basis,
        path_label,
    );

    Section754Result {
        adjustment_amount: adjustment,
        adjustment_direction: direction,
        mandatory_test_a_partnership_bil: test_a,
        mandatory_test_b_transferee_loss: test_b,
        mandatory_under_substantial_bil: mandatory,
        election_required_for_adjustment: election_required,
        adjustment_applies,
        citation:
            "IRC §754 partnership election; §743(b) inside basis adjustment formula; §743(d)(1)(A) partnership-level BIL test + §743(d)(1)(B) transferee-level loss test (TCJA addition); §755 allocation rules; §1014 FMV outside basis on death"
                .to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn base() -> Section754Input {
        Section754Input {
            transfer_type: TransferType::SaleOrExchange,
            election_in_effect: true,
            transferee_outside_basis: dec!(1_000_000),
            transferee_share_of_inside_basis: dec!(600_000),
            partnership_total_inside_basis: dec!(6_000_000),
            partnership_total_fmv: dec!(10_000_000),
            transferee_hypothetical_loss_on_immediate_sale: Decimal::ZERO,
        }
    }

    #[test]
    fn step_up_baseline_election_in_effect() {
        // Outside $1M − inside $600k = $400k step-up.
        let r = compute(&base());
        assert_eq!(r.adjustment_amount, dec!(400_000));
        assert_eq!(r.adjustment_direction, AdjustmentDirection::StepUp);
        assert!(r.adjustment_applies);
        assert!(!r.election_required_for_adjustment);
    }

    #[test]
    fn step_down_when_outside_below_inside() {
        // Outside $400k − inside $600k = −$200k step-down.
        let mut i = base();
        i.transferee_outside_basis = dec!(400_000);
        let r = compute(&i);
        assert_eq!(r.adjustment_amount, dec!(-200_000));
        assert_eq!(r.adjustment_direction, AdjustmentDirection::StepDown);
    }

    #[test]
    fn no_election_no_mandatory_no_adjustment() {
        let mut i = base();
        i.election_in_effect = false;
        let r = compute(&i);
        assert_eq!(r.adjustment_amount, Decimal::ZERO);
        assert_eq!(r.adjustment_direction, AdjustmentDirection::NoAdjustment);
        assert!(!r.adjustment_applies);
        assert!(r.election_required_for_adjustment);
    }

    #[test]
    fn mandatory_partnership_bil_test_a_triggers_at_250k_threshold() {
        // Inside $6M − FMV $5.5M = $500k BIL > $250k → test A fires.
        let mut i = base();
        i.election_in_effect = false;
        i.partnership_total_inside_basis = dec!(6_000_000);
        i.partnership_total_fmv = dec!(5_500_000);
        let r = compute(&i);
        assert!(r.mandatory_test_a_partnership_bil);
        assert!(r.mandatory_under_substantial_bil);
        assert!(r.adjustment_applies);
        assert!(!r.election_required_for_adjustment);
    }

    #[test]
    fn mandatory_test_a_exact_boundary_250k_does_not_fire() {
        // Test requires "more than $250k" — exactly $250k does NOT fire.
        let mut i = base();
        i.election_in_effect = false;
        i.partnership_total_inside_basis = dec!(6_000_000);
        i.partnership_total_fmv = dec!(5_750_000);
        let r = compute(&i);
        assert!(!r.mandatory_test_a_partnership_bil);
        assert!(!r.mandatory_under_substantial_bil);
    }

    #[test]
    fn mandatory_test_a_at_250_001_fires() {
        let mut i = base();
        i.election_in_effect = false;
        i.partnership_total_inside_basis = dec!(6_000_000);
        i.partnership_total_fmv = dec!(5_749_999);
        let r = compute(&i);
        assert!(r.mandatory_test_a_partnership_bil);
    }

    #[test]
    fn mandatory_transferee_loss_test_b_triggers() {
        // Transferee hypothetical loss $400k > $250k → test B fires
        // (TCJA addition). Test A might not fire even when B does.
        let mut i = base();
        i.election_in_effect = false;
        i.partnership_total_inside_basis = dec!(6_000_000);
        i.partnership_total_fmv = dec!(8_000_000); // No partnership BIL
        i.transferee_hypothetical_loss_on_immediate_sale = dec!(400_000);
        let r = compute(&i);
        assert!(!r.mandatory_test_a_partnership_bil);
        assert!(r.mandatory_test_b_transferee_loss);
        assert!(r.mandatory_under_substantial_bil);
        assert!(r.adjustment_applies);
    }

    #[test]
    fn mandatory_test_b_at_exact_250k_does_not_fire() {
        let mut i = base();
        i.election_in_effect = false;
        i.transferee_hypothetical_loss_on_immediate_sale = dec!(250_000);
        let r = compute(&i);
        assert!(!r.mandatory_test_b_transferee_loss);
    }

    #[test]
    fn both_tests_a_and_b_fire_together() {
        let mut i = base();
        i.election_in_effect = false;
        i.partnership_total_inside_basis = dec!(6_000_000);
        i.partnership_total_fmv = dec!(5_500_000);
        i.transferee_hypothetical_loss_on_immediate_sale = dec!(400_000);
        let r = compute(&i);
        assert!(r.mandatory_test_a_partnership_bil);
        assert!(r.mandatory_test_b_transferee_loss);
        assert!(r.mandatory_under_substantial_bil);
    }

    #[test]
    fn death_of_partner_transfer_type_path() {
        // Heir takes outside basis at FMV per §1014; §743(b) still applies.
        let mut i = base();
        i.transfer_type = TransferType::DeathOfPartner;
        let r = compute(&i);
        assert_eq!(r.adjustment_amount, dec!(400_000));
        assert!(r.note.contains("death of partner"));
        assert!(r.note.contains("§1014"));
    }

    #[test]
    fn election_in_effect_makes_election_required_false() {
        let r = compute(&base());
        assert!(!r.election_required_for_adjustment);
    }

    #[test]
    fn zero_basis_difference_no_adjustment_no_direction() {
        let mut i = base();
        i.transferee_outside_basis = dec!(600_000);
        i.transferee_share_of_inside_basis = dec!(600_000);
        let r = compute(&i);
        assert_eq!(r.adjustment_amount, Decimal::ZERO);
        assert_eq!(r.adjustment_direction, AdjustmentDirection::NoAdjustment);
    }

    #[test]
    fn step_up_election_required_when_no_election_and_no_mandatory() {
        // Big step-up potential ($400k) but no election + no mandatory
        // trigger → no adjustment applies; election would be required.
        let mut i = base();
        i.election_in_effect = false;
        let r = compute(&i);
        assert!(r.adjustment_amount == Decimal::ZERO);
        assert!(r.election_required_for_adjustment);
        assert!(r.note.contains("does NOT apply"));
    }

    #[test]
    fn very_large_step_up_path() {
        // $100M outside − $20M inside share = $80M step-up.
        let mut i = base();
        i.transferee_outside_basis = dec!(100_000_000);
        i.transferee_share_of_inside_basis = dec!(20_000_000);
        let r = compute(&i);
        assert_eq!(r.adjustment_amount, dec!(80_000_000));
    }

    #[test]
    fn mandatory_with_election_off_still_applies_step_down() {
        // Partnership BIL forces mandatory adjustment; transferee
        // outside $400k < inside $600k → step-down $200k applied even
        // though no §754 election on file.
        let mut i = base();
        i.election_in_effect = false;
        i.transferee_outside_basis = dec!(400_000);
        i.transferee_share_of_inside_basis = dec!(600_000);
        i.partnership_total_inside_basis = dec!(6_000_000);
        i.partnership_total_fmv = dec!(5_500_000);
        let r = compute(&i);
        assert!(r.mandatory_under_substantial_bil);
        assert_eq!(r.adjustment_amount, dec!(-200_000));
        assert_eq!(r.adjustment_direction, AdjustmentDirection::StepDown);
    }

    #[test]
    fn note_describes_mandatory_test_a_reason() {
        let mut i = base();
        i.election_in_effect = false;
        i.partnership_total_inside_basis = dec!(6_000_000);
        i.partnership_total_fmv = dec!(5_500_000);
        let r = compute(&i);
        assert!(r.note.contains("§743(d)(1)(A)"));
        assert!(r.note.contains("MANDATORY"));
    }

    #[test]
    fn note_describes_mandatory_test_b_reason() {
        let mut i = base();
        i.election_in_effect = false;
        i.transferee_hypothetical_loss_on_immediate_sale = dec!(400_000);
        let r = compute(&i);
        assert!(r.note.contains("§743(d)(1)(B)"));
    }

    #[test]
    fn note_describes_election_in_effect_path() {
        let r = compute(&base());
        assert!(r.note.contains("§754 election in effect"));
    }

    #[test]
    fn note_describes_no_election_no_mandatory_path() {
        let mut i = base();
        i.election_in_effect = false;
        let r = compute(&i);
        assert!(r.note.contains("§754 election NOT in effect"));
        assert!(r.note.contains("§743(b) does NOT apply"));
    }

    #[test]
    fn note_describes_no_election_but_mandatory_path() {
        let mut i = base();
        i.election_in_effect = false;
        i.partnership_total_inside_basis = dec!(6_000_000);
        i.partnership_total_fmv = dec!(5_500_000);
        let r = compute(&i);
        assert!(r.note.contains("§754 election NOT in effect but §743(d) substantial BIL forces"));
    }

    #[test]
    fn fmv_greater_than_inside_no_test_a_fire() {
        // Partnership has built-in GAIN, not LOSS → test A never fires.
        let mut i = base();
        i.election_in_effect = false;
        i.partnership_total_inside_basis = dec!(6_000_000);
        i.partnership_total_fmv = dec!(10_000_000);
        let r = compute(&i);
        assert!(!r.mandatory_test_a_partnership_bil);
    }

    #[test]
    fn zero_transferee_loss_test_b_does_not_fire() {
        let mut i = base();
        i.transferee_hypothetical_loss_on_immediate_sale = Decimal::ZERO;
        let r = compute(&i);
        assert!(!r.mandatory_test_b_transferee_loss);
    }
}
