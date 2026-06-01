//! IRC § 731 — Extent of recognition of gain or loss on
//! partnership distribution.
//!
//! Direct sibling to § 721 (partnership contribution non-
//! recognition, iter 264) — completes the partnership
//! contribution/distribution cycle. Trader-critical for hedge
//! fund distributions, real estate JV distributions, fund-of-
//! fund redemptions, and any partnership-based investment
//! vehicle's exit math.
//!
//! § 731(a) PARTNER-LEVEL RULES:
//!   (1) GAIN — Gain recognized to a partner only to the extent
//!       that any MONEY distributed exceeds the adjusted basis
//!       of such partner's interest in the partnership
//!       immediately before the distribution. Applies to BOTH
//!       current and liquidating distributions.
//!   (2) LOSS — Loss recognized to a partner only on a
//!       LIQUIDATING distribution AND only if the partner
//!       receives nothing but (a) money, (b) unrealized
//!       receivables (§ 751 hot assets), and (c) inventory
//!       items. If partner receives ANY OTHER property, no loss
//!       recognized; built-in loss instead carries into the
//!       basis of the distributed property under § 732.
//!
//! § 731(b) PARTNERSHIP-LEVEL — No gain or loss recognized to
//! the partnership on a distribution of property. Partnership
//! distributions are non-events for the partnership entity.
//!
//! § 731(c) MARKETABLE SECURITIES TREATED AS MONEY — fair market
//! value of marketable securities distributed treated as money
//! for purposes of § 731(a)(1) gain calculation. Designed to
//! prevent partnerships from converting appreciated securities
//! into partnership interests and then distributing the
//! securities without gain recognition. Multiple exceptions
//! under § 731(c)(3) including investment partnership
//! distribution rule, contribution-rollover safe harbor (same
//! securities contributed by same partner returned), and the
//! reduction-of-net-gain provision preserving partner's
//! pro-rata share of unrealized gain. Treas. Reg. § 1.731-2
//! provides detailed implementation.
//!
//! § 731(d) CROSS-REFERENCE — basis adjustment rules under
//! § 732 (basis of distributed property) and § 733 (reduction
//! of partner's basis upon distribution).
//!
//! Citations: 26 U.S.C. § 731 (general); 26 U.S.C. § 731(a)(1)
//! (gain recognition — money exceeds basis); 26 U.S.C.
//! § 731(a)(2) (loss recognition — liquidating distribution
//! only, money + hot assets + inventory only); 26 U.S.C.
//! § 731(b) (partnership-level non-recognition); 26 U.S.C.
//! § 731(c) (marketable securities treated as money); 26 U.S.C.
//! § 731(c)(3) (marketable securities exceptions — investment
//! partnership / contribution rollover / reduction-of-net-gain);
//! 26 U.S.C. § 731(d) (cross-reference to § 732 + § 733);
//! 26 CFR § 1.731-1 (general regulations); 26 CFR § 1.731-2
//! (marketable securities detailed regulations); 26 U.S.C.
//! § 721 (contribution non-recognition — companion module);
//! 26 U.S.C. § 732 (basis of distributed property);
//! 26 U.S.C. § 733 (partner's basis reduction upon distribution);
//! 26 U.S.C. § 736 (retiring partner / deceased partner
//! distributions); 26 U.S.C. § 751 (unrealized receivables and
//! inventory — hot assets); 26 U.S.C. § 754 (basis adjustment
//! election); 26 U.S.C. § 707(c) (guaranteed payments —
//! distinct from § 731 distributions).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DistributionType {
    /// Distribution to partner who continues as partner.
    Current,
    /// Distribution that terminates partner's interest entirely.
    Liquidating,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section731Input {
    pub distribution_type: DistributionType,
    /// Money (cash) distributed to partner (cents).
    pub money_distributed_cents: i64,
    /// Fair market value of marketable securities distributed
    /// (treated as money under § 731(c) absent exception).
    pub marketable_securities_fmv_distributed_cents: i64,
    /// True if § 731(c)(3) exception applies (investment
    /// partnership, contribution rollover, etc.) — marketable
    /// securities NOT treated as money in that case.
    pub section_731c_exception_engaged: bool,
    /// Partner's adjusted basis in partnership interest
    /// immediately before distribution (outside basis).
    pub partner_outside_basis_cents: i64,
    /// True if partner received any property other than money,
    /// § 751 hot assets, or inventory (e.g., real estate,
    /// equipment, intangibles). Disables § 731(a)(2) loss
    /// recognition.
    pub other_property_received: bool,
    /// Partner's basis in § 751 unrealized receivables / hot
    /// assets received as part of liquidating distribution
    /// (cents).
    pub basis_in_hot_assets_received_cents: i64,
    /// Partner's basis in inventory items received as part of
    /// liquidating distribution (cents).
    pub basis_in_inventory_received_cents: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section731Result {
    /// § 731(a)(1) gain recognized to partner (cents).
    pub gain_recognized_cents: i64,
    /// § 731(a)(2) loss recognized to partner (liquidating
    /// only, no other property received) (cents).
    pub loss_recognized_cents: i64,
    /// True if § 731(c) marketable securities treated as money
    /// for the gain calculation.
    pub section_731c_treated_as_money: bool,
    /// Total amount treated as money for § 731(a)(1) gain test
    /// (cash + marketable securities unless exception).
    pub distribution_treated_as_money_cents: i64,
    /// § 731(b) partnership-level recognition — always false
    /// (no gain/loss recognized to partnership on distribution).
    pub partnership_level_gain_or_loss_recognized: bool,
    /// True if § 731(a)(2) loss recognition prerequisites
    /// satisfied (liquidating + money/hot/inventory only).
    pub loss_recognition_eligible: bool,
    pub compliant: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn compute(input: &Section731Input) -> Section731Result {
    let mut notes: Vec<String> = Vec::new();
    let mut violations: Vec<String> = Vec::new();

    let money = input.money_distributed_cents.max(0);
    let securities_fmv = input.marketable_securities_fmv_distributed_cents.max(0);
    let outside_basis = input.partner_outside_basis_cents.max(0);
    let hot_assets_basis = input.basis_in_hot_assets_received_cents.max(0);
    let inventory_basis = input.basis_in_inventory_received_cents.max(0);

    // § 731(c) — marketable securities treated as money unless exception.
    let section_731c_treated_as_money = securities_fmv > 0 && !input.section_731c_exception_engaged;

    let distribution_treated_as_money_cents = if section_731c_treated_as_money {
        money.saturating_add(securities_fmv)
    } else {
        money
    };

    // § 731(a)(1) gain — money exceeds partner's outside basis.
    let gain_recognized_cents =
        (distribution_treated_as_money_cents - outside_basis).max(0);

    // § 731(a)(2) loss — liquidating only + no other property received.
    let liquidating = matches!(input.distribution_type, DistributionType::Liquidating);
    let loss_recognition_eligible = liquidating && !input.other_property_received;

    let loss_recognized_cents = if loss_recognition_eligible {
        // Loss = outside_basis − (money + hot_assets_basis + inventory_basis),
        // floored at 0.
        let total_received_basis = distribution_treated_as_money_cents
            .saturating_add(hot_assets_basis)
            .saturating_add(inventory_basis);
        (outside_basis - total_received_basis).max(0)
    } else {
        0
    };

    if gain_recognized_cents > 0 {
        violations.push(format!(
            "§ 731(a)(1) — gain recognition of {} cents. Money distributed {} cents \
             ({}) exceeds partner's outside basis {} cents. Gain treated as gain from \
             sale or exchange of partnership interest under § 741 (typically capital \
             gain).",
            gain_recognized_cents,
            distribution_treated_as_money_cents,
            if section_731c_treated_as_money {
                "including marketable securities treated as money under § 731(c)"
            } else {
                "cash only"
            },
            outside_basis,
        ));
    }

    if loss_recognized_cents > 0 {
        violations.push(format!(
            "§ 731(a)(2) — LIQUIDATING DISTRIBUTION loss recognition of {} cents. \
             Partner's outside basis {} cents exceeds (money + § 751 hot-asset basis + \
             inventory basis = {} cents). No other property received — loss recognition \
             eligible.",
            loss_recognized_cents,
            outside_basis,
            distribution_treated_as_money_cents + hot_assets_basis + inventory_basis,
        ));
    }

    // Notes.
    if liquidating && input.other_property_received {
        notes.push(
            "§ 731(a)(2) — LIQUIDATING distribution but partner received OTHER PROPERTY \
             (non-money, non-§ 751 hot asset, non-inventory). Loss recognition NOT \
             eligible; built-in loss carries into basis of distributed property under \
             § 732. § 731(a)(2) loss path requires receiving ONLY money + hot assets + \
             inventory."
                .to_string(),
        );
    }

    if !liquidating {
        notes.push(
            "CURRENT distribution (partner remains in partnership). § 731(a)(2) LOSS \
             recognition does NOT apply — losses only on liquidating distributions. \
             Gain still possible under § 731(a)(1) if money distributed exceeds outside \
             basis."
                .to_string(),
        );
    }

    if securities_fmv > 0 {
        if input.section_731c_exception_engaged {
            notes.push(format!(
                "§ 731(c)(3) EXCEPTION engaged — marketable securities ({} cents FMV) \
                 NOT treated as money for gain calculation. Common exceptions: (a) \
                 distribution by investment partnership; (b) securities contributed by \
                 same partner (rollover safe harbor); (c) reduction-of-net-gain rule \
                 preserving partner's pro-rata share of unrealized gain. Treas. Reg. \
                 § 1.731-2 implementation details.",
                securities_fmv,
            ));
        } else {
            notes.push(format!(
                "§ 731(c) — marketable securities distribution ({} cents FMV) treated as \
                 MONEY for § 731(a)(1) gain calculation. Designed to prevent partnerships \
                 from converting appreciated securities into partnership interests and \
                 then distributing the securities without gain recognition. § 731(c)(3) \
                 exceptions may apply (investment partnership / rollover / reduction-of-\
                 net-gain).",
                securities_fmv,
            ));
        }
    }

    notes.push(
        "§ 731(b) — partnership-level non-recognition: no gain or loss recognized to the \
         partnership entity on a distribution of property. Partnership distributions are \
         non-events for the partnership. Subsequent partnership-level treatment governed \
         by § 754 basis-adjustment election (allowing inside-outside basis alignment) and \
         § 743(b) for purchase-of-interest scenarios."
            .to_string(),
    );

    notes.push(
        "Sibling partnership cluster: § 721 (contribution non-recognition — companion \
         module, iter 264); § 732 (basis of distributed property — when basis carries \
         from partnership inside basis vs. partner outside basis); § 733 (reduction of \
         partner's basis upon distribution); § 736 (retiring/deceased partner \
         distributions — distinct rules including § 736(a) payments characterized as \
         distributive share); § 751 (unrealized receivables + inventory — 'hot assets' \
         referenced in § 731(a)(2) loss path); § 754 (basis adjustment election); \
         § 707(c) (guaranteed payments — distinct from § 731 distributions; treated as \
         payments to non-partner under § 707(a) treatment). Trader-relevant for hedge \
         fund redemptions, real estate JV exits, fund-of-fund distributions."
            .to_string(),
    );

    let compliant = violations.is_empty();

    Section731Result {
        gain_recognized_cents,
        loss_recognized_cents,
        section_731c_treated_as_money,
        distribution_treated_as_money_cents,
        partnership_level_gain_or_loss_recognized: false,
        loss_recognition_eligible,
        compliant,
        violations,
        citation: "26 U.S.C. § 731 (general); 26 U.S.C. § 731(a)(1) (gain recognition — \
                   money exceeds basis); 26 U.S.C. § 731(a)(2) (loss recognition — \
                   liquidating distribution only, money + hot assets + inventory only); \
                   26 U.S.C. § 731(b) (partnership-level non-recognition); 26 U.S.C. \
                   § 731(c) (marketable securities treated as money); 26 U.S.C. \
                   § 731(c)(3) (marketable securities exceptions); 26 U.S.C. § 731(d) \
                   (cross-reference to § 732 + § 733); 26 CFR § 1.731-1 (general); \
                   26 CFR § 1.731-2 (marketable securities); 26 U.S.C. § 721 \
                   (contribution non-recognition companion); 26 U.S.C. § 732 (basis of \
                   distributed property); 26 U.S.C. § 733 (partner basis reduction); \
                   26 U.S.C. § 736 (retiring/deceased partner); 26 U.S.C. § 751 \
                   (unrealized receivables + inventory — hot assets); 26 U.S.C. § 754 \
                   (basis adjustment election); 26 U.S.C. § 707(c) (guaranteed payments)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(distribution_type: DistributionType) -> Section731Input {
        Section731Input {
            distribution_type,
            money_distributed_cents: 0,
            marketable_securities_fmv_distributed_cents: 0,
            section_731c_exception_engaged: false,
            partner_outside_basis_cents: 10_000_000, // $100K outside basis
            other_property_received: false,
            basis_in_hot_assets_received_cents: 0,
            basis_in_inventory_received_cents: 0,
        }
    }

    // ── § 731(a)(1) gain recognition ──────────────────────────

    #[test]
    fn current_money_within_basis_no_gain() {
        let mut b = input(DistributionType::Current);
        b.money_distributed_cents = 5_000_000; // $50K < $100K basis
        let r = compute(&b);
        assert_eq!(r.gain_recognized_cents, 0);
        assert!(r.compliant);
    }

    #[test]
    fn current_money_exceeds_basis_gain_recognized() {
        let mut b = input(DistributionType::Current);
        b.money_distributed_cents = 15_000_000; // $150K > $100K basis
        let r = compute(&b);
        assert_eq!(r.gain_recognized_cents, 5_000_000); // $50K gain
        assert!(!r.compliant);
    }

    #[test]
    fn liquidating_money_exceeds_basis_gain_recognized() {
        let mut b = input(DistributionType::Liquidating);
        b.money_distributed_cents = 12_000_000;
        let r = compute(&b);
        assert_eq!(r.gain_recognized_cents, 2_000_000); // $20K gain
    }

    #[test]
    fn money_exactly_equals_basis_no_gain() {
        let mut b = input(DistributionType::Current);
        b.money_distributed_cents = 10_000_000; // exactly $100K basis
        let r = compute(&b);
        assert_eq!(r.gain_recognized_cents, 0);
    }

    // ── § 731(a)(2) loss recognition ──────────────────────────

    #[test]
    fn current_distribution_no_loss_even_below_basis() {
        let mut b = input(DistributionType::Current);
        b.money_distributed_cents = 5_000_000; // $50K < $100K basis
        let r = compute(&b);
        // Current distribution — no loss recognition.
        assert_eq!(r.loss_recognized_cents, 0);
        assert!(!r.loss_recognition_eligible);
    }

    #[test]
    fn liquidating_money_below_basis_loss_recognized() {
        let mut b = input(DistributionType::Liquidating);
        b.money_distributed_cents = 6_000_000; // $60K < $100K basis
        let r = compute(&b);
        assert!(r.loss_recognition_eligible);
        assert_eq!(r.loss_recognized_cents, 4_000_000); // $40K loss
    }

    #[test]
    fn liquidating_other_property_received_no_loss() {
        let mut b = input(DistributionType::Liquidating);
        b.money_distributed_cents = 6_000_000;
        b.other_property_received = true; // disables loss
        let r = compute(&b);
        assert!(!r.loss_recognition_eligible);
        assert_eq!(r.loss_recognized_cents, 0);
        // Note about § 732 carryover.
        assert!(r.notes.iter().any(|n| n.contains("§ 732")));
    }

    #[test]
    fn liquidating_with_hot_assets_inventory_loss_reduced() {
        let mut b = input(DistributionType::Liquidating);
        b.money_distributed_cents = 4_000_000;
        b.basis_in_hot_assets_received_cents = 2_000_000;
        b.basis_in_inventory_received_cents = 1_000_000;
        let r = compute(&b);
        // Loss = $100K - ($40K money + $20K hot + $10K inventory) = $30K.
        assert_eq!(r.loss_recognized_cents, 3_000_000);
    }

    #[test]
    fn liquidating_total_received_exceeds_basis_no_loss() {
        let mut b = input(DistributionType::Liquidating);
        b.money_distributed_cents = 12_000_000; // exceeds basis
        let r = compute(&b);
        // Money $120K exceeds basis $100K → gain $20K, no loss.
        assert_eq!(r.gain_recognized_cents, 2_000_000);
        assert_eq!(r.loss_recognized_cents, 0);
    }

    // ── § 731(c) marketable securities ────────────────────────

    #[test]
    fn marketable_securities_treated_as_money_default() {
        let mut b = input(DistributionType::Current);
        b.marketable_securities_fmv_distributed_cents = 12_000_000;
        let r = compute(&b);
        assert!(r.section_731c_treated_as_money);
        assert_eq!(r.distribution_treated_as_money_cents, 12_000_000);
        // $120K securities > $100K basis → $20K gain.
        assert_eq!(r.gain_recognized_cents, 2_000_000);
    }

    #[test]
    fn marketable_securities_exception_not_money() {
        let mut b = input(DistributionType::Current);
        b.marketable_securities_fmv_distributed_cents = 12_000_000;
        b.section_731c_exception_engaged = true;
        let r = compute(&b);
        assert!(!r.section_731c_treated_as_money);
        // Securities not treated as money → no gain calculation.
        assert_eq!(r.distribution_treated_as_money_cents, 0);
        assert_eq!(r.gain_recognized_cents, 0);
    }

    #[test]
    fn cash_plus_marketable_securities_combined() {
        let mut b = input(DistributionType::Current);
        b.money_distributed_cents = 5_000_000;
        b.marketable_securities_fmv_distributed_cents = 8_000_000;
        let r = compute(&b);
        // Total money for § 731(a) = $130K > $100K basis → $30K gain.
        assert_eq!(r.distribution_treated_as_money_cents, 13_000_000);
        assert_eq!(r.gain_recognized_cents, 3_000_000);
    }

    // ── § 731(b) partnership-level ────────────────────────────

    #[test]
    fn partnership_level_no_recognition_always() {
        // Multiple scenarios — partnership never recognizes gain/loss.
        let scenarios = [
            DistributionType::Current,
            DistributionType::Liquidating,
        ];
        for distribution_type in scenarios.iter() {
            let mut b = input(*distribution_type);
            b.money_distributed_cents = 50_000_000; // huge distribution
            let r = compute(&b);
            assert!(!r.partnership_level_gain_or_loss_recognized, "{:?}", distribution_type);
        }
    }

    // ── Multi-regime invariants ──────────────────────────────

    #[test]
    fn gain_never_negative_invariant() {
        for money in [0, 5_000_000, 10_000_000, 15_000_000, 100_000_000] {
            let mut b = input(DistributionType::Current);
            b.money_distributed_cents = money;
            let r = compute(&b);
            assert!(r.gain_recognized_cents >= 0);
        }
    }

    #[test]
    fn loss_recognition_only_on_liquidating_invariant() {
        // 2-cell sweep: distribution type × low-basis path.
        let mut current = input(DistributionType::Current);
        current.money_distributed_cents = 4_000_000;
        let r_current = compute(&current);
        assert_eq!(r_current.loss_recognized_cents, 0);
        assert!(!r_current.loss_recognition_eligible);

        let mut liq = input(DistributionType::Liquidating);
        liq.money_distributed_cents = 4_000_000;
        let r_liq = compute(&liq);
        assert!(r_liq.loss_recognition_eligible);
        assert_eq!(r_liq.loss_recognized_cents, 6_000_000);
    }

    #[test]
    fn loss_recognition_disabled_by_other_property_invariant() {
        // Even on liquidating, other property disables loss.
        for other_property in [true, false] {
            let mut b = input(DistributionType::Liquidating);
            b.money_distributed_cents = 4_000_000;
            b.other_property_received = other_property;
            let r = compute(&b);
            assert_eq!(
                r.loss_recognition_eligible,
                !other_property,
                "other_property={}",
                other_property
            );
        }
    }

    #[test]
    fn section_731c_treated_as_money_only_when_no_exception_invariant() {
        for exception in [true, false] {
            let mut b = input(DistributionType::Current);
            b.marketable_securities_fmv_distributed_cents = 5_000_000;
            b.section_731c_exception_engaged = exception;
            let r = compute(&b);
            assert_eq!(r.section_731c_treated_as_money, !exception, "exception={}", exception);
        }
    }

    #[test]
    fn current_distribution_truth_table() {
        // 4-cell sweep: money × outside_basis combinations.
        let cells = [
            (5_000_000, 10_000_000, 0, 0),       // money $50K < basis $100K → no gain/loss
            (15_000_000, 10_000_000, 5_000_000, 0), // money $150K > basis $100K → $50K gain
            (10_000_000, 10_000_000, 0, 0),      // money = basis → 0
            (0, 10_000_000, 0, 0),                // no distribution → 0
        ];
        for (money, basis, expected_gain, expected_loss) in cells.iter() {
            let mut b = input(DistributionType::Current);
            b.money_distributed_cents = *money;
            b.partner_outside_basis_cents = *basis;
            let r = compute(&b);
            assert_eq!(r.gain_recognized_cents, *expected_gain, "money={} basis={}", money, basis);
            assert_eq!(r.loss_recognized_cents, *expected_loss, "money={} basis={}", money, basis);
        }
    }

    // ── Citation + sibling note ──────────────────────────────

    #[test]
    fn citation_pins_all_subsections() {
        let r = compute(&input(DistributionType::Current));
        assert!(r.citation.contains("§ 731"));
        assert!(r.citation.contains("§ 731(a)(1)"));
        assert!(r.citation.contains("§ 731(a)(2)"));
        assert!(r.citation.contains("§ 731(b)"));
        assert!(r.citation.contains("§ 731(c)"));
        assert!(r.citation.contains("§ 731(c)(3)"));
        assert!(r.citation.contains("§ 731(d)"));
        assert!(r.citation.contains("§ 1.731-1"));
        assert!(r.citation.contains("§ 1.731-2"));
        assert!(r.citation.contains("§ 721"));
        assert!(r.citation.contains("§ 732"));
        assert!(r.citation.contains("§ 733"));
        assert!(r.citation.contains("§ 736"));
        assert!(r.citation.contains("§ 751"));
        assert!(r.citation.contains("§ 754"));
        assert!(r.citation.contains("§ 707(c)"));
    }

    #[test]
    fn sibling_cluster_note_present() {
        let r = compute(&input(DistributionType::Current));
        assert!(
            r.notes.iter().any(|n| n.contains("§ 721")
                && n.contains("§ 732")
                && n.contains("§ 733")
                && n.contains("§ 736")
                && n.contains("§ 751")
                && n.contains("§ 754")
                && n.contains("§ 707(c)")
                && n.contains("hedge fund redemptions")),
            "sibling cluster note must reference full partnership cluster + trader-relevance"
        );
    }

    #[test]
    fn partnership_level_b_note_present() {
        let r = compute(&input(DistributionType::Current));
        assert!(
            r.notes.iter().any(|n| n.contains("§ 731(b)")
                && n.contains("partnership-level non-recognition")
                && n.contains("§ 754")
                && n.contains("§ 743(b)")),
            "§ 731(b) partnership-level note must reference companion partnership-level rules"
        );
    }

    // ── Defensive input clamping ──────────────────────────────

    #[test]
    fn defensive_negative_basis_clamped() {
        let mut b = input(DistributionType::Current);
        b.partner_outside_basis_cents = -1_000_000;
        b.money_distributed_cents = 5_000_000;
        let r = compute(&b);
        // Negative basis → 0; any money distribution → full gain.
        assert_eq!(r.gain_recognized_cents, 5_000_000);
    }

    #[test]
    fn defensive_negative_money_clamped() {
        let mut b = input(DistributionType::Current);
        b.money_distributed_cents = -1_000_000;
        let r = compute(&b);
        assert_eq!(r.gain_recognized_cents, 0);
    }

    #[test]
    fn extreme_amounts_no_overflow() {
        let mut b = input(DistributionType::Current);
        b.partner_outside_basis_cents = 1_000_000_000;     // $10M
        b.money_distributed_cents = 100_000_000_000;        // $1B
        let r = compute(&b);
        // $1B - $10M = $990M gain.
        assert_eq!(r.gain_recognized_cents, 99_000_000_000);
    }
}
