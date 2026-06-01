//! IRC § 332 — Complete liquidations of subsidiaries.
//!
//! § 332(a) provides that NO GAIN OR LOSS is recognized to a PARENT
//! corporation on the receipt of property in complete liquidation of a
//! subsidiary IF the parent meets the 80% ownership requirement. Pairs
//! with § 337(a) (no gain/loss to the liquidating subsidiary on the
//! distribution) and § 334(b) (parent takes CARRYOVER basis in the
//! distributed property).
//!
//! § 332(b) requirements:
//!
//! (1) Parent must own at least **80% of the total voting power** of
//!     the subsidiary's stock; AND
//! (2) Parent must own at least **80% of the total value** of the
//!     subsidiary's stock (this is the § 1504(a)(2) controlled-group
//!     test); AND
//! (3) The 80% ownership must exist on the **date of adoption of the
//!     plan of liquidation** AND be MAINTAINED until all property has
//!     been distributed (continuous-ownership requirement); AND
//! (4) The distribution must be in **COMPLETE LIQUIDATION** of the
//!     subsidiary (all property distributed, all stock cancelled).
//!
//! Failing ANY of the four prongs → § 332 does NOT apply, and the
//! liquidation is treated as a standard distribution under § 331 with
//! FMV recognition by both parties.
//!
//! § 334(b) consequence — when § 332 applies, the parent takes
//! CARRYOVER BASIS in the property (same as the subsidiary's basis,
//! NOT FMV). This preserves the subsidiary's tax attributes including
//! NOLs (§ 381) and built-in gain/loss.
//!
//! § 337(a) parallel — when § 332(a) applies on the parent side, no
//! gain/loss recognized on the SUBSIDIARY side either on distribution
//! to the parent (§ 337(a) "subsidiary non-recognition").
//!
//! Citations: 26 U.S.C. § 332(a) (parent non-recognition); § 332(b)(1)
//! (complete-liquidation requirement); § 332(b)(2) (80% voting +
//! value); § 334(b)(1) (carryover basis); § 337(a) (subsidiary non-
//! recognition); § 1504(a)(2) (definition of 80% voting + value).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Section332Input {
    /// Percent of total voting power of the subsidiary's stock owned by
    /// the parent on the date of adoption of the plan of liquidation
    /// (basis points × 100; e.g., 8000 = 80.00%).
    pub voting_power_owned_bp: u32,
    /// Percent of total value of the subsidiary's stock owned by the
    /// parent on the date of adoption of the plan of liquidation
    /// (basis points × 100).
    pub value_owned_bp: u32,
    /// Whether the parent maintained at least 80% voting AND value
    /// ownership continuously from the plan-adoption date through the
    /// final distribution of property. Required by § 332(b)(3).
    pub continuous_80_pct_ownership_maintained: bool,
    /// Whether the distribution is in COMPLETE liquidation — all
    /// property distributed and all subsidiary stock cancelled.
    pub complete_liquidation: bool,
    /// FMV of property distributed by subsidiary to parent. Used to
    /// compute hypothetical gain/loss if § 332 fails (§ 331/§ 336 path).
    pub fmv_of_property_distributed_cents: i64,
    /// Subsidiary's adjusted basis in the distributed property. Used
    /// for § 334(b) carryover basis to parent if § 332 applies.
    pub subsidiary_adjusted_basis_cents: i64,
    /// Parent's basis in the subsidiary stock surrendered. Used to
    /// compute hypothetical gain/loss if § 332 fails (parent's gain
    /// under § 331 = FMV property received − basis in stock).
    pub parent_basis_in_subsidiary_stock_cents: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section332Result {
    pub section_332_applies: bool,
    pub voting_test_satisfied: bool,
    pub value_test_satisfied: bool,
    pub continuous_ownership_satisfied: bool,
    pub complete_liquidation_satisfied: bool,
    /// Parent's gain/loss recognized. Zero if § 332 applies; otherwise
    /// § 331 FMV-minus-stock-basis amount.
    pub parent_gain_or_loss_recognized_cents: i64,
    /// Subsidiary's gain/loss on distribution. Zero if § 332(a) applies
    /// (§ 337(a) parallel non-recognition); otherwise § 336 FMV-minus-
    /// basis amount.
    pub subsidiary_gain_or_loss_recognized_cents: i64,
    /// Parent's basis in the distributed property: carryover (§ 334(b))
    /// if § 332 applies; FMV if § 332 fails.
    pub parent_basis_in_property_cents: i64,
    pub citation: &'static str,
    pub note: String,
}

pub fn compute(input: &Section332Input) -> Section332Result {
    let voting_test = input.voting_power_owned_bp >= 8000;
    let value_test = input.value_owned_bp >= 8000;
    let continuous = input.continuous_80_pct_ownership_maintained;
    let complete = input.complete_liquidation;

    let section_332_applies = voting_test && value_test && continuous && complete;

    let fmv = input.fmv_of_property_distributed_cents.max(0);
    let sub_basis = input.subsidiary_adjusted_basis_cents.max(0);
    let stock_basis = input.parent_basis_in_subsidiary_stock_cents.max(0);

    let (parent_gain, sub_gain, parent_property_basis, citation) = if section_332_applies {
        // Both sides have non-recognition; parent takes carryover basis
        // in the distributed property under § 334(b).
        (
            0_i64,
            0_i64,
            sub_basis,
            "26 U.S.C. § 332(a) — parent non-recognition; § 337(a) subsidiary non-recognition; § 334(b)(1) carryover basis to parent",
        )
    } else {
        // § 332 fails. Parent recognizes under § 331 = FMV property
        // received − basis in subsidiary stock surrendered. Subsidiary
        // recognizes under § 336 = FMV − adjusted basis. Parent takes
        // FMV basis in the property.
        let parent_gain = fmv - stock_basis;
        let sub_gain = fmv - sub_basis;
        (
            parent_gain,
            sub_gain,
            fmv,
            "26 U.S.C. § 331 + § 336 — § 332 inapplicable; parent recognizes gain/loss under § 331; subsidiary recognizes under § 336; parent takes FMV basis in property",
        )
    };

    let note = format!(
        "§ 332 four-prong test: voting {} ≥ 80%? {} | value {} ≥ 80%? {} | continuous 80% ownership? {} | complete liquidation? {}. § 332 applies: {}. Parent gain/loss = {} cents. Subsidiary gain/loss = {} cents. Parent basis in distributed property = {} cents ({}).",
        input.voting_power_owned_bp,
        voting_test,
        input.value_owned_bp,
        value_test,
        continuous,
        complete,
        section_332_applies,
        parent_gain,
        sub_gain,
        parent_property_basis,
        if section_332_applies { "carryover under § 334(b)" } else { "FMV under § 331" },
    );

    Section332Result {
        section_332_applies,
        voting_test_satisfied: voting_test,
        value_test_satisfied: value_test,
        continuous_ownership_satisfied: continuous,
        complete_liquidation_satisfied: complete,
        parent_gain_or_loss_recognized_cents: parent_gain,
        subsidiary_gain_or_loss_recognized_cents: sub_gain,
        parent_basis_in_property_cents: parent_property_basis,
        citation,
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        voting_bp: u32,
        value_bp: u32,
        continuous: bool,
        complete: bool,
        fmv: i64,
        sub_basis: i64,
        stock_basis: i64,
    ) -> Section332Input {
        Section332Input {
            voting_power_owned_bp: voting_bp,
            value_owned_bp: value_bp,
            continuous_80_pct_ownership_maintained: continuous,
            complete_liquidation: complete,
            fmv_of_property_distributed_cents: fmv,
            subsidiary_adjusted_basis_cents: sub_basis,
            parent_basis_in_subsidiary_stock_cents: stock_basis,
        }
    }

    #[test]
    fn full_compliance_80_pct_both_complete_continuous() {
        // 100% owned, complete liquidation, continuous ownership.
        let r = compute(&input(
            10000,
            10000,
            true,
            true,
            500_000_00,
            300_000_00,
            200_000_00,
        ));
        assert!(r.section_332_applies);
        assert_eq!(r.parent_gain_or_loss_recognized_cents, 0);
        assert_eq!(r.subsidiary_gain_or_loss_recognized_cents, 0);
        // Carryover basis = subsidiary's adjusted basis.
        assert_eq!(r.parent_basis_in_property_cents, 300_000_00);
    }

    #[test]
    fn at_80_pct_voting_value_boundary_applies() {
        // Exactly 80% voting + 80% value → meets § 332(b)(2).
        let r = compute(&input(
            8000,
            8000,
            true,
            true,
            500_000_00,
            300_000_00,
            200_000_00,
        ));
        assert!(r.section_332_applies);
    }

    #[test]
    fn at_79_99_pct_voting_below_threshold_fails() {
        let r = compute(&input(
            7999,
            8000,
            true,
            true,
            500_000_00,
            300_000_00,
            200_000_00,
        ));
        assert!(!r.section_332_applies);
        assert!(!r.voting_test_satisfied);
        // Parent recognizes under § 331: $500K - $200K = $300K gain.
        assert_eq!(r.parent_gain_or_loss_recognized_cents, 300_000_00);
        // Subsidiary recognizes under § 336: $500K - $300K = $200K gain.
        assert_eq!(r.subsidiary_gain_or_loss_recognized_cents, 200_000_00);
        // Parent takes FMV basis.
        assert_eq!(r.parent_basis_in_property_cents, 500_000_00);
    }

    #[test]
    fn at_79_99_pct_value_below_threshold_fails() {
        let r = compute(&input(
            8000,
            7999,
            true,
            true,
            500_000_00,
            300_000_00,
            200_000_00,
        ));
        assert!(!r.section_332_applies);
        assert!(!r.value_test_satisfied);
    }

    #[test]
    fn continuous_ownership_not_maintained_fails() {
        let r = compute(&input(
            10000,
            10000,
            false,
            true,
            500_000_00,
            300_000_00,
            200_000_00,
        ));
        assert!(!r.section_332_applies);
        assert!(!r.continuous_ownership_satisfied);
    }

    #[test]
    fn incomplete_liquidation_fails() {
        let r = compute(&input(
            10000,
            10000,
            true,
            false,
            500_000_00,
            300_000_00,
            200_000_00,
        ));
        assert!(!r.section_332_applies);
        assert!(!r.complete_liquidation_satisfied);
    }

    #[test]
    fn all_four_prongs_individually_required() {
        // Test each prong individually fails the whole test.
        // Voting < 80%.
        let r_v = compute(&input(
            7900,
            10000,
            true,
            true,
            100_000_00,
            50_000_00,
            10_000_00,
        ));
        assert!(!r_v.section_332_applies);
        // Value < 80%.
        let r_val = compute(&input(
            10000,
            7900,
            true,
            true,
            100_000_00,
            50_000_00,
            10_000_00,
        ));
        assert!(!r_val.section_332_applies);
        // Not continuous.
        let r_c = compute(&input(
            10000,
            10000,
            false,
            true,
            100_000_00,
            50_000_00,
            10_000_00,
        ));
        assert!(!r_c.section_332_applies);
        // Not complete.
        let r_complete = compute(&input(
            10000,
            10000,
            true,
            false,
            100_000_00,
            50_000_00,
            10_000_00,
        ));
        assert!(!r_complete.section_332_applies);
    }

    #[test]
    fn carryover_basis_under_334_b() {
        // § 334(b): parent takes subsidiary's adjusted basis, NOT FMV.
        let r = compute(&input(
            10000,
            10000,
            true,
            true,
            1_000_000_00,
            200_000_00, // subsidiary basis
            300_000_00,
        ));
        assert!(r.section_332_applies);
        assert_eq!(r.parent_basis_in_property_cents, 200_000_00);
    }

    #[test]
    fn fmv_basis_when_332_fails() {
        // § 332 fails → parent takes FMV basis (not carryover).
        let r = compute(&input(
            7900,
            7900,
            true,
            true,
            1_000_000_00,
            200_000_00,
            300_000_00,
        ));
        assert!(!r.section_332_applies);
        assert_eq!(r.parent_basis_in_property_cents, 1_000_000_00);
    }

    #[test]
    fn section_337a_parallel_subsidiary_non_recognition() {
        // When § 332(a) applies on parent side, § 337(a) gives parallel
        // non-recognition on subsidiary side regardless of built-in gain.
        let r = compute(&input(
            10000,
            10000,
            true,
            true,
            1_000_000_00,
            100_000_00, // subsidiary has $900K built-in gain
            500_000_00,
        ));
        assert!(r.section_332_applies);
        assert_eq!(r.subsidiary_gain_or_loss_recognized_cents, 0);
    }

    #[test]
    fn parent_loss_when_332_fails_and_fmv_below_stock_basis() {
        // FMV $100K, stock basis $300K → $200K LOSS on stock surrender.
        let r = compute(&input(
            7000,
            7000,
            true,
            true,
            100_000_00,
            50_000_00,
            300_000_00,
        ));
        assert!(!r.section_332_applies);
        assert_eq!(r.parent_gain_or_loss_recognized_cents, -200_000_00);
    }

    #[test]
    fn citations_pin_correct_authorities() {
        let r_applies = compute(&input(
            10000,
            10000,
            true,
            true,
            100_000_00,
            50_000_00,
            10_000_00,
        ));
        assert!(r_applies.citation.contains("§ 332(a)"));
        assert!(r_applies.citation.contains("§ 337(a)"));
        assert!(r_applies.citation.contains("§ 334(b)"));

        let r_fails = compute(&input(
            7000,
            7000,
            true,
            true,
            100_000_00,
            50_000_00,
            10_000_00,
        ));
        assert!(r_fails.citation.contains("§ 331"));
        assert!(r_fails.citation.contains("§ 336"));
    }

    #[test]
    fn negative_inputs_clamped() {
        let r = compute(&input(10000, 10000, true, true, -1, -1, -1));
        assert_eq!(r.parent_basis_in_property_cents, 0);
    }

    #[test]
    fn voting_at_exactly_80_value_at_exactly_80_applies() {
        let r = compute(&input(
            8000, 8000, true, true, 100_000_00, 50_000_00, 10_000_00,
        ));
        assert!(r.section_332_applies);
    }

    #[test]
    fn voting_above_80_value_below_80_fails_value_test() {
        let r = compute(&input(
            9500, 7500, true, true, 100_000_00, 50_000_00, 10_000_00,
        ));
        assert!(!r.section_332_applies);
        assert!(r.voting_test_satisfied);
        assert!(!r.value_test_satisfied);
    }

    #[test]
    fn worked_example_clean_parent_sub_liquidation() {
        // Parent owns 100% of Sub. Sub has $1M FMV property with $400K
        // basis. Parent's stock basis $200K. § 332 applies →
        // (1) Parent no gain; subsidiary no gain.
        // (2) Parent takes $400K carryover basis (NOT $1M FMV).
        let r = compute(&input(
            10000, 10000, true, true, 1_000_000_00, 400_000_00, 200_000_00,
        ));
        assert_eq!(r.parent_gain_or_loss_recognized_cents, 0);
        assert_eq!(r.subsidiary_gain_or_loss_recognized_cents, 0);
        assert_eq!(r.parent_basis_in_property_cents, 400_000_00);
    }

    #[test]
    fn worked_example_failed_liquidation_recognizes_everything() {
        // Parent owns only 75% — § 332 fails. Same fact pattern.
        let r = compute(&input(
            7500, 7500, true, true, 1_000_000_00, 400_000_00, 200_000_00,
        ));
        // Parent: $1M FMV − $200K stock basis = $800K gain.
        assert_eq!(r.parent_gain_or_loss_recognized_cents, 800_000_00);
        // Subsidiary: $1M FMV − $400K basis = $600K gain.
        assert_eq!(r.subsidiary_gain_or_loss_recognized_cents, 600_000_00);
        // Parent basis = FMV.
        assert_eq!(r.parent_basis_in_property_cents, 1_000_000_00);
    }

    #[test]
    fn all_four_test_flags_returned() {
        let r = compute(&input(
            10000, 10000, true, true, 100_000_00, 50_000_00, 10_000_00,
        ));
        assert!(r.voting_test_satisfied);
        assert!(r.value_test_satisfied);
        assert!(r.continuous_ownership_satisfied);
        assert!(r.complete_liquidation_satisfied);
        assert!(r.section_332_applies);
    }
}
