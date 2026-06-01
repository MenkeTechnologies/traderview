//! IRC § 752 — Treatment of certain partnership liabilities.
//!
//! Completes the partnership-tax cluster: § 721 (contribution
//! non-recognition, iter 264) + § 731 (distribution gain/loss,
//! iter 266) + § 752 (THIS MODULE — liability allocation
//! affecting outside basis). § 752 is structurally CENTRAL to
//! partnership taxation because partner's share of partnership
//! liabilities is treated as MONEY CONTRIBUTED OR DISTRIBUTED
//! for purposes of outside basis calculation — directly affects
//! § 731 distribution gain/loss math and § 704(d) basis-limited
//! loss deductions. Trader-critical for real estate
//! partnerships with mortgage debt, hedge funds with margin
//! debt, and any leveraged partnership structure.
//!
//! § 752(a) INCREASE TREATED AS CONTRIBUTION — any increase in
//! a partner's share of partnership liabilities (or increase in
//! partner's individual liabilities by reason of the partner's
//! assumption of partnership liabilities) is treated as a
//! CONTRIBUTION OF MONEY by the partner to the partnership.
//! Increases outside basis dollar-for-dollar.
//!
//! § 752(b) DECREASE TREATED AS DISTRIBUTION — any decrease in
//! a partner's share of partnership liabilities (or decrease in
//! partner's individual liabilities by reason of the
//! partnership's assumption of partner's individual liabilities)
//! is treated as a DISTRIBUTION OF MONEY by the partnership to
//! the partner. Decreases outside basis dollar-for-dollar; may
//! trigger § 731(a)(1) gain if cumulative deemed-money exceeds
//! basis.
//!
//! § 752(c) PROPERTY-SUBJECT-TO-LIABILITY — a liability to which
//! property is subject is treated as a liability of the
//! partnership (and considered nonrecourse) to the extent of
//! the property's fair market value. Distinct from recourse
//! liabilities where partner bears economic risk of loss.
//!
//! § 752 NETTING RULE — if a single transaction results in both
//! an increase AND a decrease in partner's share of liabilities,
//! only the NET change is recognized. Net increase → deemed
//! contribution; net decrease → deemed distribution.
//!
//! RECOURSE vs NONRECOURSE allocation:
//!
//! Treas. Reg. § 1.752-2 RECOURSE — partner who bears the
//! ECONOMIC RISK OF LOSS (EROL) bears the recourse liability
//! share. Determined via constructive liquidation analysis.
//! Final Regulations (TD 10014, December 2, 2024) clarified
//! multi-partner risk-sharing, tiered partnerships, related-
//! party rules.
//!
//! Treas. Reg. § 1.752-3 NONRECOURSE — THREE-TIER ALLOCATION:
//!   TIER 1 — Partner's share of partnership minimum gain under
//!     § 704(b) Reg. § 1.704-2.
//!   TIER 2 — Taxable gain allocated to partner under § 704(c)
//!     if partnership disposed of all property subject to
//!     nonrecourse debt for no consideration other than full
//!     satisfaction of the debt.
//!   TIER 3 — Remaining excess nonrecourse liability —
//!     partner's share of partnership profits (alternative
//!     allocation methods available).
//!
//! Citations: 26 U.S.C. § 752 (general); 26 U.S.C. § 752(a)
//! (increase treated as contribution); 26 U.S.C. § 752(b)
//! (decrease treated as distribution); 26 U.S.C. § 752(c)
//! (property-subject-to-liability rule); 26 U.S.C. § 752(d)
//! (sale or exchange treatment); 26 CFR § 1.752-1 (general
//! liability treatment + netting rule); 26 CFR § 1.752-2
//! (recourse allocation — economic risk of loss); 26 CFR
//! § 1.752-3 (nonrecourse three-tier allocation); 26 CFR
//! § 1.704-2 (minimum gain — tier 1 input); 26 CFR § 1.704-3
//! (§ 704(c) gain — tier 2 input); TD 10014 (December 2, 2024
//! final recourse regulations); 26 U.S.C. § 721 (contribution
//! non-recognition — companion); 26 U.S.C. § 731 (distribution
//! gain/loss — companion; deemed distribution under § 752(b)
//! may trigger § 731(a)(1) gain); 26 U.S.C. § 704(b) (minimum
//! gain framework); 26 U.S.C. § 704(c) (built-in gain
//! allocation); 26 U.S.C. § 704(d) (basis-limited loss);
//! 26 U.S.C. § 705 (partner basis adjustments).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Section752Input {
    /// Partner's share of partnership liabilities BEFORE the
    /// transaction or year-end determination (cents).
    pub partner_share_liabilities_before_cents: i64,
    /// Partner's share of partnership liabilities AFTER the
    /// transaction or year-end determination (cents).
    pub partner_share_liabilities_after_cents: i64,
    /// Partner's outside basis BEFORE applying § 752 deemed
    /// contribution/distribution (cents).
    pub partner_outside_basis_before_cents: i64,
    /// Tier 1 — partner's share of partnership minimum gain
    /// under § 704(b) (cents). Used for nonrecourse 3-tier
    /// allocation reporting.
    pub tier1_minimum_gain_share_cents: i64,
    /// Tier 2 — § 704(c) gain that would be allocated to
    /// partner on hypothetical disposition for full debt
    /// satisfaction (cents).
    pub tier2_section_704c_gain_share_cents: i64,
    /// Tier 3 — partner's share of excess nonrecourse
    /// liability based on profit share (cents).
    pub tier3_excess_nonrecourse_share_cents: i64,
    /// True if partner bears the economic risk of loss
    /// (constructive liquidation analysis) — engages recourse
    /// allocation under Treas. Reg. § 1.752-2.
    pub bears_economic_risk_of_loss: bool,
    /// True if a single transaction produced both increases and
    /// decreases in partner's share, triggering the netting
    /// rule under Treas. Reg. § 1.752-1.
    pub single_transaction_netting_applied: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section752Result {
    /// Net change in partner's share of liabilities
    /// (positive = increase; negative = decrease).
    pub net_liability_change_cents: i64,
    /// § 752(a) deemed contribution (cents). Positive when
    /// partner's share increases.
    pub deemed_contribution_cents: i64,
    /// § 752(b) deemed distribution (cents). Positive when
    /// partner's share decreases.
    pub deemed_distribution_cents: i64,
    /// § 731(a)(1) gain triggered when deemed distribution
    /// exceeds outside basis (cents).
    pub section_731_gain_recognized_cents: i64,
    /// Partner's outside basis AFTER applying § 752 deemed
    /// contribution/distribution (cents).
    pub partner_outside_basis_after_cents: i64,
    /// Sum of three-tier nonrecourse allocation
    /// (tier1 + tier2 + tier3).
    pub nonrecourse_three_tier_total_cents: i64,
    /// True if recourse allocation applies (partner bears
    /// economic risk of loss).
    pub recourse_allocation_engaged: bool,
    /// True if § 731(a)(1) gain triggered by § 752(b) deemed
    /// distribution.
    pub deemed_distribution_triggered_section_731_gain: bool,
    pub compliant: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn compute(input: &Section752Input) -> Section752Result {
    let mut notes: Vec<String> = Vec::new();
    let mut violations: Vec<String> = Vec::new();

    let liabilities_before = input.partner_share_liabilities_before_cents.max(0);
    let liabilities_after = input.partner_share_liabilities_after_cents.max(0);
    let outside_basis_before = input.partner_outside_basis_before_cents.max(0);
    let tier1 = input.tier1_minimum_gain_share_cents.max(0);
    let tier2 = input.tier2_section_704c_gain_share_cents.max(0);
    let tier3 = input.tier3_excess_nonrecourse_share_cents.max(0);

    // Net change in liabilities.
    let net_liability_change_cents = liabilities_after - liabilities_before;

    // § 752(a)/(b) treatment.
    let deemed_contribution_cents = net_liability_change_cents.max(0);
    let deemed_distribution_cents = (-net_liability_change_cents).max(0);

    // Apply § 731(a)(1) — distribution exceeding basis triggers gain.
    let section_731_gain_recognized_cents =
        (deemed_distribution_cents - outside_basis_before).max(0);

    // Basis adjustment.
    let partner_outside_basis_after_cents = if deemed_contribution_cents > 0 {
        outside_basis_before.saturating_add(deemed_contribution_cents)
    } else {
        outside_basis_before.saturating_sub(deemed_distribution_cents).max(0)
    };

    // Nonrecourse three-tier total.
    let nonrecourse_three_tier_total_cents = tier1
        .saturating_add(tier2)
        .saturating_add(tier3);

    let recourse_allocation_engaged = input.bears_economic_risk_of_loss;
    let deemed_distribution_triggered_section_731_gain = section_731_gain_recognized_cents > 0;

    if deemed_distribution_triggered_section_731_gain {
        violations.push(format!(
            "§ 752(b) deemed distribution ({} cents) exceeds partner's outside basis \
             ({} cents) → § 731(a)(1) gain recognition of {} cents. Gain treated as gain \
             from sale or exchange of partnership interest under § 741 (typically \
             capital gain). Trader-critical: refinancing or assumption events can \
             trigger this gain without any actual cash distribution.",
            deemed_distribution_cents,
            outside_basis_before,
            section_731_gain_recognized_cents,
        ));
    }

    // Notes.
    if deemed_contribution_cents > 0 {
        notes.push(format!(
            "§ 752(a) — partner's share of partnership liabilities INCREASED by {} cents \
             ({} cents before → {} cents after). Treated as money contribution; outside \
             basis increases dollar-for-dollar. New outside basis: {} cents.",
            deemed_contribution_cents,
            liabilities_before,
            liabilities_after,
            partner_outside_basis_after_cents,
        ));
    } else if deemed_distribution_cents > 0 {
        notes.push(format!(
            "§ 752(b) — partner's share of partnership liabilities DECREASED by {} cents \
             ({} cents before → {} cents after). Treated as money distribution; outside \
             basis decreases dollar-for-dollar (floored at $0). New outside basis: \
             {} cents. § 731(a)(1) gain if cumulative deemed-money exceeds basis: {}.",
            deemed_distribution_cents,
            liabilities_before,
            liabilities_after,
            partner_outside_basis_after_cents,
            deemed_distribution_triggered_section_731_gain,
        ));
    } else {
        notes.push(
            "§ 752 — no net change in partner's share of partnership liabilities. No \
             deemed contribution or distribution; outside basis unchanged by § 752."
                .to_string(),
        );
    }

    if input.single_transaction_netting_applied {
        notes.push(
            "Treas. Reg. § 1.752-1 NETTING RULE engaged — single transaction produced \
             both increases AND decreases in partner's share of liabilities. Only the \
             NET change is recognized as § 752(a) contribution or § 752(b) distribution. \
             Gross amounts collapsed to net for § 752 purposes."
                .to_string(),
        );
    }

    // Recourse vs nonrecourse allocation notes.
    if recourse_allocation_engaged {
        notes.push(
            "Treas. Reg. § 1.752-2 RECOURSE allocation — partner bears the economic risk \
             of loss (EROL) and receives full share of the recourse liability. \
             Constructive liquidation analysis determines the partner's allocation. \
             Final Regulations (TD 10014, December 2, 2024) clarified multi-partner \
             risk-sharing, tiered partnerships, and related-party rules."
                .to_string(),
        );
    } else {
        notes.push(format!(
            "Treas. Reg. § 1.752-3 NONRECOURSE 3-TIER ALLOCATION (no economic risk of \
             loss assumed). Tier 1 (§ 1.704-2 minimum gain): {} cents. Tier 2 (§ 704(c) \
             hypothetical-disposition gain): {} cents. Tier 3 (excess nonrecourse — \
             profit share): {} cents. Total nonrecourse allocation: {} cents.",
            tier1,
            tier2,
            tier3,
            nonrecourse_three_tier_total_cents,
        ));
    }

    notes.push(
        "Sibling partnership cluster: § 721 (contribution non-recognition — iter 264; \
         partner's initial outside basis); § 731 (distribution gain/loss — iter 266; \
         § 752(b) deemed distribution feeds § 731(a)(1) gain math); § 704(b) (substantial \
         economic effect — minimum gain framework for tier 1); § 704(c) (built-in gain \
         allocation — tier 2); § 704(d) (basis-limited loss — uses § 752-adjusted \
         outside basis); § 705 (partner basis adjustments aggregating § 752 + § 704 + \
         distributions). § 752 is structurally central — liability allocation feeds \
         every other partnership computation."
            .to_string(),
    );

    let compliant = violations.is_empty();

    Section752Result {
        net_liability_change_cents,
        deemed_contribution_cents,
        deemed_distribution_cents,
        section_731_gain_recognized_cents,
        partner_outside_basis_after_cents,
        nonrecourse_three_tier_total_cents,
        recourse_allocation_engaged,
        deemed_distribution_triggered_section_731_gain,
        compliant,
        violations,
        citation: "26 U.S.C. § 752 (general); 26 U.S.C. § 752(a) (increase = \
                   contribution); 26 U.S.C. § 752(b) (decrease = distribution); \
                   26 U.S.C. § 752(c) (property-subject-to-liability rule); 26 U.S.C. \
                   § 752(d) (sale or exchange treatment); 26 CFR § 1.752-1 (general + \
                   netting rule); 26 CFR § 1.752-2 (recourse — economic risk of loss); \
                   26 CFR § 1.752-3 (nonrecourse three-tier allocation); 26 CFR \
                   § 1.704-2 (minimum gain framework — tier 1 input); 26 CFR § 1.704-3 \
                   (§ 704(c) gain — tier 2 input); TD 10014 (December 2, 2024 final \
                   recourse regulations); 26 U.S.C. § 721 (contribution non-recognition \
                   companion); 26 U.S.C. § 731 (distribution gain/loss companion); \
                   26 U.S.C. § 704(b) (substantial economic effect); 26 U.S.C. § 704(c) \
                   (built-in gain allocation); 26 U.S.C. § 704(d) (basis-limited loss); \
                   26 U.S.C. § 705 (partner basis adjustments)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> Section752Input {
        Section752Input {
            partner_share_liabilities_before_cents: 10_000_000, // $100K
            partner_share_liabilities_after_cents: 10_000_000,
            partner_outside_basis_before_cents: 20_000_000, // $200K
            tier1_minimum_gain_share_cents: 0,
            tier2_section_704c_gain_share_cents: 0,
            tier3_excess_nonrecourse_share_cents: 0,
            bears_economic_risk_of_loss: false,
            single_transaction_netting_applied: false,
        }
    }

    // ── § 752(a) increase = contribution ──────────────────────

    #[test]
    fn liability_increase_treated_as_contribution() {
        let mut b = input();
        b.partner_share_liabilities_after_cents = 15_000_000; // +$50K
        let r = compute(&b);
        assert_eq!(r.net_liability_change_cents, 5_000_000);
        assert_eq!(r.deemed_contribution_cents, 5_000_000);
        assert_eq!(r.deemed_distribution_cents, 0);
        // Outside basis: $200K + $50K = $250K.
        assert_eq!(r.partner_outside_basis_after_cents, 25_000_000);
        assert!(r.compliant);
    }

    #[test]
    fn liability_zero_change_no_deemed_event() {
        let r = compute(&input());
        assert_eq!(r.net_liability_change_cents, 0);
        assert_eq!(r.deemed_contribution_cents, 0);
        assert_eq!(r.deemed_distribution_cents, 0);
        // Outside basis unchanged.
        assert_eq!(r.partner_outside_basis_after_cents, 20_000_000);
        assert!(r.compliant);
    }

    // ── § 752(b) decrease = distribution ──────────────────────

    #[test]
    fn liability_decrease_treated_as_distribution_within_basis() {
        let mut b = input();
        b.partner_share_liabilities_after_cents = 7_000_000; // -$30K
        let r = compute(&b);
        assert_eq!(r.net_liability_change_cents, -3_000_000);
        assert_eq!(r.deemed_distribution_cents, 3_000_000);
        // Outside basis: $200K - $30K = $170K. No § 731 gain.
        assert_eq!(r.partner_outside_basis_after_cents, 17_000_000);
        assert!(!r.deemed_distribution_triggered_section_731_gain);
        assert_eq!(r.section_731_gain_recognized_cents, 0);
    }

    #[test]
    fn liability_decrease_exceeds_basis_triggers_731_gain() {
        let mut b = input();
        b.partner_outside_basis_before_cents = 5_000_000;     // $50K basis
        b.partner_share_liabilities_after_cents = 0;          // -$100K (full payoff)
        let r = compute(&b);
        assert_eq!(r.deemed_distribution_cents, 10_000_000);
        // Distribution $100K > basis $50K → $50K gain.
        assert_eq!(r.section_731_gain_recognized_cents, 5_000_000);
        assert!(r.deemed_distribution_triggered_section_731_gain);
        assert!(!r.compliant);
        assert_eq!(r.partner_outside_basis_after_cents, 0); // basis floored
        assert!(r.violations.iter().any(|v| v.contains("§ 731(a)(1)")));
    }

    #[test]
    fn liability_decrease_exactly_equals_basis_no_gain() {
        let mut b = input();
        b.partner_outside_basis_before_cents = 10_000_000;    // $100K
        b.partner_share_liabilities_after_cents = 0;          // -$100K
        let r = compute(&b);
        assert_eq!(r.section_731_gain_recognized_cents, 0);
        assert_eq!(r.partner_outside_basis_after_cents, 0);
        assert!(r.compliant);
    }

    // ── Three-tier nonrecourse allocation ────────────────────

    #[test]
    fn nonrecourse_tier1_only_minimum_gain() {
        let mut b = input();
        b.tier1_minimum_gain_share_cents = 5_000_000;
        let r = compute(&b);
        assert_eq!(r.nonrecourse_three_tier_total_cents, 5_000_000);
        assert!(!r.recourse_allocation_engaged);
    }

    #[test]
    fn nonrecourse_tier2_only_704c_gain() {
        let mut b = input();
        b.tier2_section_704c_gain_share_cents = 3_000_000;
        let r = compute(&b);
        assert_eq!(r.nonrecourse_three_tier_total_cents, 3_000_000);
    }

    #[test]
    fn nonrecourse_tier3_only_excess() {
        let mut b = input();
        b.tier3_excess_nonrecourse_share_cents = 2_000_000;
        let r = compute(&b);
        assert_eq!(r.nonrecourse_three_tier_total_cents, 2_000_000);
    }

    #[test]
    fn nonrecourse_all_three_tiers_summed() {
        let mut b = input();
        b.tier1_minimum_gain_share_cents = 5_000_000;
        b.tier2_section_704c_gain_share_cents = 3_000_000;
        b.tier3_excess_nonrecourse_share_cents = 2_000_000;
        let r = compute(&b);
        assert_eq!(r.nonrecourse_three_tier_total_cents, 10_000_000);
    }

    // ── Recourse vs nonrecourse allocation ──────────────────

    #[test]
    fn recourse_allocation_when_bears_erol() {
        let mut b = input();
        b.bears_economic_risk_of_loss = true;
        let r = compute(&b);
        assert!(r.recourse_allocation_engaged);
        assert!(r.notes.iter().any(|n| n.contains("RECOURSE allocation")));
    }

    #[test]
    fn nonrecourse_allocation_when_no_erol() {
        let r = compute(&input());
        assert!(!r.recourse_allocation_engaged);
        assert!(r.notes.iter().any(|n| n.contains("NONRECOURSE 3-TIER ALLOCATION")));
    }

    // ── § 1.752-1 netting rule ───────────────────────────────

    #[test]
    fn single_transaction_netting_note_present() {
        let mut b = input();
        b.single_transaction_netting_applied = true;
        let r = compute(&b);
        assert!(r.notes.iter().any(|n| n.contains("NETTING RULE engaged")));
    }

    // ── Multi-regime invariants ──────────────────────────────

    #[test]
    fn liability_change_directly_translates_to_basis_change_invariant() {
        // 4-cell sweep: liability change × basis change should equal liability change.
        let cells = [
            (10_000_000, 15_000_000, 5_000_000),     // +$50K liability → +$50K basis
            (15_000_000, 10_000_000, -5_000_000),    // -$50K liability → -$50K basis
            (10_000_000, 10_000_000, 0),              // no change
            (5_000_000, 20_000_000, 15_000_000),     // +$150K liability → +$150K basis
        ];
        for (before, after, expected_change) in cells.iter() {
            let mut b = input();
            b.partner_share_liabilities_before_cents = *before;
            b.partner_share_liabilities_after_cents = *after;
            b.partner_outside_basis_before_cents = 50_000_000; // huge basis to avoid § 731 gain
            let r = compute(&b);
            let basis_change = r.partner_outside_basis_after_cents - 50_000_000;
            assert_eq!(basis_change, *expected_change, "before={} after={}", before, after);
        }
    }

    #[test]
    fn deemed_contribution_and_distribution_mutually_exclusive_invariant() {
        // For any liability change, only one of contribution or
        // distribution is positive.
        let cells = [(5_000_000, 15_000_000), (15_000_000, 5_000_000), (10_000_000, 10_000_000)];
        for (before, after) in cells.iter() {
            let mut b = input();
            b.partner_share_liabilities_before_cents = *before;
            b.partner_share_liabilities_after_cents = *after;
            let r = compute(&b);
            assert!(
                !(r.deemed_contribution_cents > 0 && r.deemed_distribution_cents > 0),
                "before={} after={}",
                before,
                after
            );
        }
    }

    #[test]
    fn section_731_gain_only_when_distribution_exceeds_basis_invariant() {
        // 4-cell sweep over basis × distribution.
        let cells = [
            (20_000_000, 5_000_000, 0),     // basis $200K > $50K dist → no gain
            (5_000_000, 5_000_000, 0),      // exact match → no gain
            (3_000_000, 5_000_000, 2_000_000),  // basis $30K, dist $50K → $20K gain
            (0, 10_000_000, 10_000_000),    // zero basis → full distribution = gain
        ];
        for (basis, decrease, expected_gain) in cells.iter() {
            let mut b = input();
            b.partner_outside_basis_before_cents = *basis;
            b.partner_share_liabilities_before_cents = *decrease;
            b.partner_share_liabilities_after_cents = 0;
            let r = compute(&b);
            assert_eq!(
                r.section_731_gain_recognized_cents,
                *expected_gain,
                "basis={} decrease={}",
                basis,
                decrease
            );
        }
    }

    #[test]
    fn three_tier_nonrecourse_sum_invariant() {
        // 4-cell sweep — tier sum across multiple combinations.
        let cells = [
            (0, 0, 0, 0),
            (1_000_000, 2_000_000, 3_000_000, 6_000_000),
            (5_000_000, 0, 0, 5_000_000),
            (0, 0, 7_000_000, 7_000_000),
        ];
        for (t1, t2, t3, expected_total) in cells.iter() {
            let mut b = input();
            b.tier1_minimum_gain_share_cents = *t1;
            b.tier2_section_704c_gain_share_cents = *t2;
            b.tier3_excess_nonrecourse_share_cents = *t3;
            let r = compute(&b);
            assert_eq!(
                r.nonrecourse_three_tier_total_cents,
                *expected_total,
                "t1={} t2={} t3={}",
                t1,
                t2,
                t3
            );
        }
    }

    // ── Citation + sibling note ──────────────────────────────

    #[test]
    fn citation_pins_all_subsections() {
        let r = compute(&input());
        assert!(r.citation.contains("§ 752"));
        assert!(r.citation.contains("§ 752(a)"));
        assert!(r.citation.contains("§ 752(b)"));
        assert!(r.citation.contains("§ 752(c)"));
        assert!(r.citation.contains("§ 752(d)"));
        assert!(r.citation.contains("§ 1.752-1"));
        assert!(r.citation.contains("§ 1.752-2"));
        assert!(r.citation.contains("§ 1.752-3"));
        assert!(r.citation.contains("§ 1.704-2"));
        assert!(r.citation.contains("§ 1.704-3"));
        assert!(r.citation.contains("TD 10014"));
        assert!(r.citation.contains("December 2, 2024"));
        assert!(r.citation.contains("§ 721"));
        assert!(r.citation.contains("§ 731"));
        assert!(r.citation.contains("§ 704(b)"));
        assert!(r.citation.contains("§ 704(c)"));
        assert!(r.citation.contains("§ 704(d)"));
        assert!(r.citation.contains("§ 705"));
    }

    #[test]
    fn sibling_cluster_note_present() {
        let r = compute(&input());
        assert!(
            r.notes.iter().any(|n| n.contains("§ 721")
                && n.contains("§ 731")
                && n.contains("§ 704(b)")
                && n.contains("§ 704(c)")
                && n.contains("§ 704(d)")
                && n.contains("§ 705")
                && n.contains("structurally central")),
            "sibling cluster note must reference partnership cluster + § 752 centrality"
        );
    }

    // ── Defensive input clamping ──────────────────────────────

    #[test]
    fn defensive_negative_basis_clamped() {
        let mut b = input();
        b.partner_outside_basis_before_cents = -1_000_000;
        b.partner_share_liabilities_after_cents = 15_000_000; // +$50K
        let r = compute(&b);
        // Negative basis → 0; new basis = 0 + $50K = $50K.
        assert_eq!(r.partner_outside_basis_after_cents, 5_000_000);
    }

    #[test]
    fn defensive_negative_tier_clamped() {
        let mut b = input();
        b.tier1_minimum_gain_share_cents = -1_000_000;
        b.tier2_section_704c_gain_share_cents = -2_000_000;
        b.tier3_excess_nonrecourse_share_cents = -3_000_000;
        let r = compute(&b);
        // All clamped to 0.
        assert_eq!(r.nonrecourse_three_tier_total_cents, 0);
    }

    #[test]
    fn extreme_amounts_no_overflow() {
        let mut b = input();
        b.partner_share_liabilities_before_cents = 0;
        b.partner_share_liabilities_after_cents = 100_000_000_000; // $1B
        b.partner_outside_basis_before_cents = 0;
        let r = compute(&b);
        // $1B contribution → $1B new basis.
        assert_eq!(r.partner_outside_basis_after_cents, 100_000_000_000);
    }
}
