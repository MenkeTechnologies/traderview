//! IRC § 6662A — Accuracy-related penalty on understatements
//! with respect to reportable transactions.
//!
//! Direct sibling to § 6011 (taxpayer disclosure — Form 8886),
//! § 6111 (advisor disclosure — Form 8918), § 6707 (advisor
//! penalty), and § 6707A (taxpayer disclosure penalty). § 6662A
//! is the UNDERSTATEMENT-OF-TAX penalty layered on top of the
//! disclosure regime — it taxes the substantive position taken
//! on the return, not just the failure to file Form 8886.
//!
//! § 6662A(a) PENALTY RATE: 20% of the reportable transaction
//! understatement.
//!
//! § 6662A(c) ENHANCED RATE FOR NONDISCLOSED TRANSACTIONS:
//! Substitute "30 percent" for "20 percent" with respect to any
//! portion of a reportable transaction understatement for which
//! the § 6664(d)(3)(A) requirement (adequate disclosure per
//! § 6011 regulations) is not met. Nondisclosure → 30% rate;
//! adequate Form 8886 disclosure → 20% rate.
//!
//! § 6662A(b)(1) REPORTABLE TRANSACTION UNDERSTATEMENT:
//!   = (increase in taxable income × highest tax rate)
//!     + (decrease in aggregate credits under subtitle A)
//!
//! § 6664(d) REASONABLE CAUSE EXCEPTION — no penalty if BOTH
//! reasonable cause AND good faith are shown WITH RESPECT TO
//! each portion of the understatement, AND THREE adequate-
//! protection prongs are all satisfied:
//!   § 6664(d)(3)(A) — facts adequately disclosed per § 6011 regs;
//!   § 6664(d)(3)(B) — substantial authority for the treatment;
//!   § 6664(d)(3)(C) — taxpayer reasonably believed the treatment
//!     was more likely than not the proper treatment.
//! ALL three prongs required. Missing any prong = no reasonable-
//! cause defense.
//!
//! Citations: 26 U.S.C. § 6662A (general); 26 U.S.C. § 6662A(a)
//! (20% rate); 26 U.S.C. § 6662A(b)(1) (understatement definition
//! — income increase × highest rate + credit decrease);
//! 26 U.S.C. § 6662A(b)(2) (reportable transaction definition —
//! cross to § 6011); 26 U.S.C. § 6662A(c) (30% nondisclosure
//! enhanced rate); 26 U.S.C. § 6664(d) (reasonable cause
//! exception); 26 U.S.C. § 6664(d)(3)(A)/(B)/(C) (three-prong
//! adequate-protection test); 26 CFR § 1.6664-4 (reasonable cause
//! regulations); Notice 2005-12 (transitional rules); Notice
//! 2005-22 (initial guidance on § 6662A application).
//! Sibling modules: § 6011 (taxpayer Form 8886); § 6111 (advisor
//! Form 8918); § 6707 (advisor penalty); § 6707A (taxpayer
//! disclosure penalty); § 6662 (general accuracy penalty — not
//! stacked with § 6662A per § 6662A(e)(2)(A) coordination rule).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DisclosureStatus {
    /// Form 8886 timely filed under § 6011 regulations. Triggers
    /// § 6662A(a) 20% rate.
    AdequatelyDisclosed,
    /// Form 8886 not filed or filed late. Triggers § 6662A(c) 30%
    /// enhanced rate.
    NotDisclosed,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section6662AInput {
    pub disclosure_status: DisclosureStatus,
    /// § 6662A(b)(1)(A) — increase in taxable income resulting
    /// from difference between proper tax treatment and
    /// taxpayer's treatment of the reportable-transaction item
    /// (cents).
    pub taxable_income_increase_cents: i64,
    /// § 6662A(b)(1)(B) — decrease in aggregate credits under
    /// subtitle A resulting from the reportable transaction
    /// (cents).
    pub credit_decrease_cents: i64,
    /// Highest tax rate in basis points (e.g., 3700 = 37%).
    /// Multiplied against taxable income increase per
    /// § 6662A(b)(1)(A).
    pub highest_tax_rate_bps: i64,
    /// § 6664(d) — taxpayer asserts reasonable cause + good
    /// faith defense.
    pub reasonable_cause_claimed: bool,
    /// § 6664(d)(3)(A) — facts adequately disclosed per § 6011
    /// regulations. Required prong of reasonable-cause defense.
    pub facts_adequately_disclosed: bool,
    /// § 6664(d)(3)(B) — substantial authority for the treatment.
    /// Required prong of reasonable-cause defense.
    pub substantial_authority_exists: bool,
    /// § 6664(d)(3)(C) — taxpayer reasonably believed the
    /// treatment was more likely than not the proper treatment.
    /// Required prong of reasonable-cause defense.
    pub more_likely_than_not_belief: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section6662AResult {
    /// § 6662A(b)(1) computed understatement (cents).
    pub reportable_transaction_understatement_cents: i64,
    /// Applicable penalty rate in basis points. 2000 = 20% (§
    /// 6662A(a)); 3000 = 30% (§ 6662A(c)).
    pub penalty_rate_bps: i64,
    /// Raw § 6662A penalty before § 6664(d) reasonable-cause
    /// reduction (cents).
    pub penalty_before_reasonable_cause_cents: i64,
    /// Final penalty after § 6664(d) reasonable-cause exception
    /// (cents). Zero if all three § 6664(d)(3)(A)/(B)/(C) prongs
    /// satisfied and reasonable cause + good faith demonstrated.
    pub penalty_cents: i64,
    /// True if § 6664(d) reasonable-cause exception engaged
    /// (all three prongs satisfied + reasonable cause claimed).
    pub reasonable_cause_excused: bool,
    pub compliant: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

/// § 6662A(a) — baseline penalty rate (basis points).
pub const PENALTY_RATE_DISCLOSED_BPS: i64 = 2000;
/// § 6662A(c) — enhanced penalty rate for nondisclosed (bps).
pub const PENALTY_RATE_NOT_DISCLOSED_BPS: i64 = 3000;
/// Basis-point denominator.
pub const BPS_DENOMINATOR: i64 = 10_000;

pub fn compute(input: &Section6662AInput) -> Section6662AResult {
    let mut notes: Vec<String> = Vec::new();
    let mut violations: Vec<String> = Vec::new();

    let income_increase = input.taxable_income_increase_cents.max(0);
    let credit_decrease = input.credit_decrease_cents.max(0);
    let tax_rate_bps = input.highest_tax_rate_bps.clamp(0, BPS_DENOMINATOR);

    // § 6662A(b)(1)(A) — income increase × highest tax rate.
    let tax_from_income_increase = income_increase
        .saturating_mul(tax_rate_bps)
        / BPS_DENOMINATOR;
    // § 6662A(b)(1) — sum of (A) + (B).
    let understatement = tax_from_income_increase.saturating_add(credit_decrease);

    // § 6662A(a) vs § 6662A(c) rate selection.
    let penalty_rate_bps = match input.disclosure_status {
        DisclosureStatus::AdequatelyDisclosed => PENALTY_RATE_DISCLOSED_BPS,
        DisclosureStatus::NotDisclosed => PENALTY_RATE_NOT_DISCLOSED_BPS,
    };

    let penalty_before = understatement
        .saturating_mul(penalty_rate_bps)
        / BPS_DENOMINATOR;

    // § 6664(d) reasonable-cause + good-faith exception.
    let reasonable_cause_excused = input.reasonable_cause_claimed
        && input.facts_adequately_disclosed
        && input.substantial_authority_exists
        && input.more_likely_than_not_belief;

    let penalty = if reasonable_cause_excused {
        0
    } else {
        penalty_before
    };

    if penalty > 0 {
        violations.push(format!(
            "§ 6662A({}) — reportable transaction understatement penalty: {} cents at {}% \
             rate ({}). Adequate disclosure under § 6011: {}.",
            match input.disclosure_status {
                DisclosureStatus::AdequatelyDisclosed => "a",
                DisclosureStatus::NotDisclosed => "c",
            },
            penalty,
            penalty_rate_bps / 100,
            match input.disclosure_status {
                DisclosureStatus::AdequatelyDisclosed => "20% baseline",
                DisclosureStatus::NotDisclosed => "30% enhanced — nondisclosure",
            },
            matches!(input.disclosure_status, DisclosureStatus::AdequatelyDisclosed),
        ));
    }

    notes.push(format!(
        "§ 6662A(b)(1) understatement = {} cents = (income increase {} cents × {}% highest \
         rate) {} cents + credit decrease {} cents.",
        understatement,
        income_increase,
        tax_rate_bps / 100,
        tax_from_income_increase,
        credit_decrease,
    ));

    notes.push(format!(
        "Applicable rate: § 6662A({}) → {}% ({}).",
        match input.disclosure_status {
            DisclosureStatus::AdequatelyDisclosed => "a",
            DisclosureStatus::NotDisclosed => "c",
        },
        penalty_rate_bps / 100,
        match input.disclosure_status {
            DisclosureStatus::AdequatelyDisclosed => "Form 8886 timely filed per § 6011 — \
                                                       baseline 20%",
            DisclosureStatus::NotDisclosed => "No Form 8886 or untimely — enhanced 30% \
                                                under § 6662A(c)",
        },
    ));

    if input.reasonable_cause_claimed {
        let missing_prongs: Vec<&str> = [
            (input.facts_adequately_disclosed, "§ 6664(d)(3)(A) facts adequately disclosed"),
            (
                input.substantial_authority_exists,
                "§ 6664(d)(3)(B) substantial authority",
            ),
            (
                input.more_likely_than_not_belief,
                "§ 6664(d)(3)(C) more-likely-than-not belief",
            ),
        ]
        .iter()
        .filter_map(|(satisfied, label)| if *satisfied { None } else { Some(*label) })
        .collect();

        if reasonable_cause_excused {
            notes.push(
                "§ 6664(d) reasonable-cause exception ENGAGED — all three prongs satisfied: \
                 (A) facts adequately disclosed; (B) substantial authority; (C) more-likely-\
                 than-not belief. Penalty reduced to zero."
                    .to_string(),
            );
        } else {
            notes.push(format!(
                "§ 6664(d) reasonable-cause defense FAILED — missing prong(s): {}. ALL \
                 THREE § 6664(d)(3)(A)/(B)/(C) prongs are mandatory; missing any prong \
                 forfeits the defense.",
                missing_prongs.join("; "),
            ));
        }
    }

    notes.push(
        "Sibling modules: § 6011 (taxpayer-side Form 8886 disclosure); § 6111 (advisor-\
         side Form 8918 disclosure); § 6707 (advisor failure-to-disclose penalty); \
         § 6707A (taxpayer failure-to-disclose penalty — separate from § 6662A \
         understatement penalty); § 6662 (general accuracy penalty — coordination under \
         § 6662A(e)(2)(A) prevents stacking on the same understatement). § 6662A \
         specifically targets the SUBSTANTIVE tax position on the return, not the \
         disclosure failure standalone."
            .to_string(),
    );

    Section6662AResult {
        reportable_transaction_understatement_cents: understatement,
        penalty_rate_bps,
        penalty_before_reasonable_cause_cents: penalty_before,
        penalty_cents: penalty,
        reasonable_cause_excused,
        compliant: violations.is_empty(),
        violations,
        citation: "26 U.S.C. § 6662A (accuracy-related penalty on reportable-transaction \
                   understatements); 26 U.S.C. § 6662A(a) (20% baseline rate); 26 U.S.C. \
                   § 6662A(b)(1) (understatement definition — income increase × highest \
                   rate + credit decrease); 26 U.S.C. § 6662A(b)(2) (reportable transaction \
                   cross-reference to § 6011); 26 U.S.C. § 6662A(c) (30% enhanced rate for \
                   nondisclosed transactions); 26 U.S.C. § 6662A(e)(2)(A) (coordination \
                   with § 6662 to prevent stacking); 26 U.S.C. § 6664(d) (reasonable-cause \
                   + good-faith exception); 26 U.S.C. § 6664(d)(3)(A) (adequate disclosure \
                   prong); 26 U.S.C. § 6664(d)(3)(B) (substantial authority prong); \
                   26 U.S.C. § 6664(d)(3)(C) (more-likely-than-not belief prong); \
                   26 CFR § 1.6664-4 (reasonable cause regulations); Notice 2005-12 \
                   (transitional rules); Notice 2005-22 (initial guidance)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        disclosure: DisclosureStatus,
        income_increase: i64,
        credit_decrease: i64,
        tax_rate_bps: i64,
    ) -> Section6662AInput {
        Section6662AInput {
            disclosure_status: disclosure,
            taxable_income_increase_cents: income_increase,
            credit_decrease_cents: credit_decrease,
            highest_tax_rate_bps: tax_rate_bps,
            reasonable_cause_claimed: false,
            facts_adequately_disclosed: false,
            substantial_authority_exists: false,
            more_likely_than_not_belief: false,
        }
    }

    // ── Penalty rate — disclosed vs not disclosed ─────────────

    #[test]
    fn disclosed_20_percent_baseline_rate() {
        let r = compute(&input(
            DisclosureStatus::AdequatelyDisclosed,
            10_000_000,
            0,
            3700,
        ));
        assert_eq!(r.penalty_rate_bps, 2000);
    }

    #[test]
    fn not_disclosed_30_percent_enhanced_rate() {
        let r = compute(&input(
            DisclosureStatus::NotDisclosed,
            10_000_000,
            0,
            3700,
        ));
        assert_eq!(r.penalty_rate_bps, 3000);
    }

    // ── Understatement math — income increase component ────────

    #[test]
    fn income_increase_times_37_percent_rate() {
        // $100K income increase × 37% = $37K understatement.
        let r = compute(&input(
            DisclosureStatus::AdequatelyDisclosed,
            10_000_000,
            0,
            3700,
        ));
        assert_eq!(r.reportable_transaction_understatement_cents, 3_700_000);
        // 20% of $37K = $7.4K penalty.
        assert_eq!(r.penalty_cents, 740_000);
    }

    #[test]
    fn income_increase_times_24_percent_rate() {
        // $50K income increase × 24% = $12K understatement.
        let r = compute(&input(
            DisclosureStatus::AdequatelyDisclosed,
            5_000_000,
            0,
            2400,
        ));
        assert_eq!(r.reportable_transaction_understatement_cents, 1_200_000);
    }

    // ── Understatement math — credit decrease component ────────

    #[test]
    fn credit_decrease_direct_pass_through() {
        // $50K credit decrease → $50K understatement (no rate multiplier).
        let r = compute(&input(
            DisclosureStatus::AdequatelyDisclosed,
            0,
            5_000_000,
            3700,
        ));
        assert_eq!(r.reportable_transaction_understatement_cents, 5_000_000);
        // 20% of $50K = $10K penalty.
        assert_eq!(r.penalty_cents, 1_000_000);
    }

    #[test]
    fn combined_income_and_credit_components() {
        // $100K income × 37% = $37K + $20K credit = $57K understatement.
        let r = compute(&input(
            DisclosureStatus::AdequatelyDisclosed,
            10_000_000,
            2_000_000,
            3700,
        ));
        assert_eq!(r.reportable_transaction_understatement_cents, 5_700_000);
        // 20% of $57K = $11.4K penalty.
        assert_eq!(r.penalty_cents, 1_140_000);
    }

    // ── 30% rate calculation ───────────────────────────────────

    #[test]
    fn not_disclosed_30_percent_higher_penalty() {
        // Same understatement, 30% rate.
        let disclosed = compute(&input(
            DisclosureStatus::AdequatelyDisclosed,
            10_000_000,
            0,
            3700,
        ));
        let not_disclosed = compute(&input(
            DisclosureStatus::NotDisclosed,
            10_000_000,
            0,
            3700,
        ));
        // 30% / 20% = 1.5× ratio.
        assert_eq!(not_disclosed.penalty_cents, 1_110_000);
        assert_eq!(disclosed.penalty_cents, 740_000);
        assert!(not_disclosed.penalty_cents > disclosed.penalty_cents);
    }

    // ── § 6664(d) reasonable-cause exception ──────────────────

    #[test]
    fn reasonable_cause_all_three_prongs_excuses_penalty() {
        let mut b = input(DisclosureStatus::AdequatelyDisclosed, 10_000_000, 0, 3700);
        b.reasonable_cause_claimed = true;
        b.facts_adequately_disclosed = true;
        b.substantial_authority_exists = true;
        b.more_likely_than_not_belief = true;
        let r = compute(&b);
        assert!(r.reasonable_cause_excused);
        assert_eq!(r.penalty_cents, 0);
        assert!(r.compliant);
    }

    #[test]
    fn reasonable_cause_missing_disclosure_prong_fails() {
        let mut b = input(DisclosureStatus::AdequatelyDisclosed, 10_000_000, 0, 3700);
        b.reasonable_cause_claimed = true;
        b.facts_adequately_disclosed = false;
        b.substantial_authority_exists = true;
        b.more_likely_than_not_belief = true;
        let r = compute(&b);
        assert!(!r.reasonable_cause_excused);
        assert_eq!(r.penalty_cents, 740_000);
    }

    #[test]
    fn reasonable_cause_missing_substantial_authority_fails() {
        let mut b = input(DisclosureStatus::AdequatelyDisclosed, 10_000_000, 0, 3700);
        b.reasonable_cause_claimed = true;
        b.facts_adequately_disclosed = true;
        b.substantial_authority_exists = false;
        b.more_likely_than_not_belief = true;
        let r = compute(&b);
        assert!(!r.reasonable_cause_excused);
        assert!(r.penalty_cents > 0);
    }

    #[test]
    fn reasonable_cause_missing_more_likely_than_not_fails() {
        let mut b = input(DisclosureStatus::AdequatelyDisclosed, 10_000_000, 0, 3700);
        b.reasonable_cause_claimed = true;
        b.facts_adequately_disclosed = true;
        b.substantial_authority_exists = true;
        b.more_likely_than_not_belief = false;
        let r = compute(&b);
        assert!(!r.reasonable_cause_excused);
        assert!(r.penalty_cents > 0);
    }

    #[test]
    fn reasonable_cause_not_claimed_no_exception() {
        let mut b = input(DisclosureStatus::AdequatelyDisclosed, 10_000_000, 0, 3700);
        b.reasonable_cause_claimed = false;
        b.facts_adequately_disclosed = true;
        b.substantial_authority_exists = true;
        b.more_likely_than_not_belief = true;
        let r = compute(&b);
        assert!(!r.reasonable_cause_excused);
        assert!(r.penalty_cents > 0);
    }

    // ── Multi-regime invariants ───────────────────────────────

    #[test]
    fn enhanced_rate_strictly_higher_than_baseline_invariant() {
        assert!(PENALTY_RATE_NOT_DISCLOSED_BPS > PENALTY_RATE_DISCLOSED_BPS);
        assert_eq!(PENALTY_RATE_NOT_DISCLOSED_BPS, 3000);
        assert_eq!(PENALTY_RATE_DISCLOSED_BPS, 2000);
        // 30% is exactly 1.5× 20%.
        assert_eq!(PENALTY_RATE_NOT_DISCLOSED_BPS * 2, PENALTY_RATE_DISCLOSED_BPS * 3);
    }

    #[test]
    fn reasonable_cause_requires_all_three_prongs_truth_table() {
        // 8-cell truth table for prong combinations. Reasonable
        // cause excuses ONLY when all three are true.
        for facts in [false, true] {
            for authority in [false, true] {
                for belief in [false, true] {
                    let mut b =
                        input(DisclosureStatus::AdequatelyDisclosed, 10_000_000, 0, 3700);
                    b.reasonable_cause_claimed = true;
                    b.facts_adequately_disclosed = facts;
                    b.substantial_authority_exists = authority;
                    b.more_likely_than_not_belief = belief;
                    let r = compute(&b);
                    let expected_excused = facts && authority && belief;
                    assert_eq!(
                        r.reasonable_cause_excused, expected_excused,
                        "facts={} authority={} belief={}",
                        facts, authority, belief
                    );
                }
            }
        }
    }

    #[test]
    fn rate_switch_depends_only_on_disclosure_invariant() {
        // 2-cell sweep — same understatement, only disclosure varies.
        let disclosed = compute(&input(
            DisclosureStatus::AdequatelyDisclosed,
            10_000_000,
            0,
            3700,
        ));
        let not_disclosed = compute(&input(
            DisclosureStatus::NotDisclosed,
            10_000_000,
            0,
            3700,
        ));
        assert_eq!(disclosed.reportable_transaction_understatement_cents, not_disclosed.reportable_transaction_understatement_cents);
        assert_ne!(disclosed.penalty_rate_bps, not_disclosed.penalty_rate_bps);
        assert!(not_disclosed.penalty_cents > disclosed.penalty_cents);
    }

    #[test]
    fn zero_understatement_zero_penalty() {
        let r = compute(&input(DisclosureStatus::NotDisclosed, 0, 0, 3700));
        assert_eq!(r.reportable_transaction_understatement_cents, 0);
        assert_eq!(r.penalty_cents, 0);
    }

    #[test]
    fn citation_pins_all_subsections() {
        let r = compute(&input(DisclosureStatus::NotDisclosed, 10_000_000, 0, 3700));
        assert!(r.citation.contains("§ 6662A"));
        assert!(r.citation.contains("§ 6662A(a)"));
        assert!(r.citation.contains("§ 6662A(b)(1)"));
        assert!(r.citation.contains("§ 6662A(b)(2)"));
        assert!(r.citation.contains("§ 6662A(c)"));
        assert!(r.citation.contains("§ 6662A(e)(2)(A)"));
        assert!(r.citation.contains("§ 6664(d)"));
        assert!(r.citation.contains("§ 6664(d)(3)(A)"));
        assert!(r.citation.contains("§ 6664(d)(3)(B)"));
        assert!(r.citation.contains("§ 6664(d)(3)(C)"));
        assert!(r.citation.contains("§ 1.6664-4"));
        assert!(r.citation.contains("Notice 2005-12"));
        assert!(r.citation.contains("Notice 2005-22"));
    }

    #[test]
    fn sibling_modules_note_present() {
        let r = compute(&input(DisclosureStatus::NotDisclosed, 10_000_000, 0, 3700));
        assert!(
            r.notes.iter().any(|n| n.contains("§ 6011")
                && n.contains("§ 6111")
                && n.contains("§ 6707")
                && n.contains("§ 6707A")
                && n.contains("§ 6662")),
            "sibling-modules cluster note must reference § 6011 + § 6111 + § 6707 + § 6707A + § 6662"
        );
    }

    #[test]
    fn defensive_negative_income_increase_clamped() {
        let r = compute(&input(
            DisclosureStatus::NotDisclosed,
            -1_000_000,
            0,
            3700,
        ));
        assert_eq!(r.reportable_transaction_understatement_cents, 0);
        assert_eq!(r.penalty_cents, 0);
    }

    #[test]
    fn defensive_negative_credit_decrease_clamped() {
        let r = compute(&input(
            DisclosureStatus::NotDisclosed,
            0,
            -1_000_000,
            3700,
        ));
        assert_eq!(r.reportable_transaction_understatement_cents, 0);
    }

    #[test]
    fn defensive_tax_rate_above_100_percent_clamped() {
        let r = compute(&input(
            DisclosureStatus::NotDisclosed,
            10_000_000,
            0,
            15_000, // 150% — nonsensical
        ));
        // Should clamp to 100% (10,000 bps).
        assert_eq!(r.reportable_transaction_understatement_cents, 10_000_000);
    }

    #[test]
    fn defensive_negative_tax_rate_clamped_to_zero() {
        let r = compute(&input(
            DisclosureStatus::NotDisclosed,
            10_000_000,
            0,
            -100,
        ));
        // Negative rate → 0 contribution from income component;
        // credit-decrease still passes through (0 here).
        assert_eq!(r.reportable_transaction_understatement_cents, 0);
    }

    #[test]
    fn boundary_understatement_one_cent_one_percent_rate() {
        let r = compute(&input(
            DisclosureStatus::AdequatelyDisclosed,
            10_000_000,
            0,
            100, // 1%
        ));
        assert_eq!(r.reportable_transaction_understatement_cents, 100_000);
        // 20% of $1000 = $200.
        assert_eq!(r.penalty_cents, 20_000);
    }

    #[test]
    fn large_understatement_no_overflow() {
        // $1B income × 37% = $370M understatement; 30% = $111M.
        let r = compute(&input(
            DisclosureStatus::NotDisclosed,
            100_000_000_000,
            0,
            3700,
        ));
        assert_eq!(r.reportable_transaction_understatement_cents, 37_000_000_000);
        assert_eq!(r.penalty_cents, 11_100_000_000);
    }
}
