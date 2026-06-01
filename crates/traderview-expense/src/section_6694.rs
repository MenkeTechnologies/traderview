//! IRC § 6694 — Understatement of taxpayer's liability by tax
//! return preparer.
//!
//! Trader-critical because trader-CPAs commonly take aggressive
//! positions on Schedule D, mark-to-market § 475(f) elections,
//! wash-sale aggregation under § 1091, and partnership K-1
//! allocations. § 6694 creates direct preparer liability on top
//! of taxpayer penalties under § 6662 / § 6662A / § 6707A.
//! Sibling to the preparer + promoter penalty cluster: § 6695
//! (preparer information return penalties), § 6700 (promoter
//! penalties), § 6701 (aiding and abetting understatement).
//!
//! § 6694(a) — UNDERSTATEMENT DUE TO UNREASONABLE POSITION.
//! Penalty: greater of $1,000 OR 50% of the preparer's income
//! derived from preparation of the return or claim for refund.
//!
//! § 6694(a)(2) — POSITION IS UNREASONABLE if any of three
//! prongs:
//!   (A) No substantial authority for the position AND position
//!       not disclosed under § 6662(d)(2)(B)(ii)(I);
//!   (B) Position disclosed but no reasonable basis;
//!   (C) Position is with respect to a tax shelter or § 6662A
//!       reportable transaction AND was NOT reasonable to
//!       believe the position would be more likely than not
//!       sustained on the merits.
//!
//! § 6694(a)(3) — REASONABLE-CAUSE EXCEPTION: No § 6694(a)
//! penalty if it is shown that, considering all facts and
//! circumstances, the understatement was due to reasonable
//! cause AND the preparer acted in good faith.
//!
//! § 6694(b) — UNDERSTATEMENT DUE TO WILLFUL OR RECKLESS
//! CONDUCT. Penalty: greater of $5,000 OR 75% of preparer's
//! income from the return. Engages when preparer:
//!   (1) willfully understates the liability; OR
//!   (2) recklessly or intentionally disregards rules or
//!       regulations.
//! No reasonable-cause exception for § 6694(b) penalties.
//!
//! § 6694(b)(3) — COORDINATION: The § 6694(b) penalty is
//! REDUCED by the amount of any § 6694(a) penalty imposed on
//! the same return. Effectively (b) replaces (a) when both
//! trigger (no stacking).
//!
//! Standards of authority (for § 6694(a)(2)(A)/(B)/(C)):
//!   - "Substantial authority" — generally 35-40% probability
//!     of sustaining the position;
//!   - "Reasonable basis" — generally ≥ 20% probability (lower
//!     than substantial authority);
//!   - "More likely than not" — > 50% probability (highest
//!     standard; required for tax-shelter / reportable
//!     transaction positions).
//!
//! Citations: 26 U.S.C. § 6694 (general); 26 U.S.C. § 6694(a)
//! (unreasonable-position penalty); 26 U.S.C. § 6694(a)(2)(A)
//! (no-substantial-authority + non-disclosure prong);
//! 26 U.S.C. § 6694(a)(2)(B) (disclosed-without-reasonable-
//! basis prong); 26 U.S.C. § 6694(a)(2)(C) (tax-shelter / § 6662A
//! more-likely-than-not prong); 26 U.S.C. § 6694(a)(3)
//! (reasonable-cause exception); 26 U.S.C. § 6694(b) (willful/
//! reckless penalty); 26 U.S.C. § 6694(b)(2)(A) (willful);
//! 26 U.S.C. § 6694(b)(2)(B) (reckless/intentional disregard);
//! 26 U.S.C. § 6694(b)(3) (no-stacking coordination); Treas. Reg.
//! § 1.6694-1 (general regulations); Treas. Reg. § 1.6694-2
//! (penalty for understatement due to unreasonable position);
//! Treas. Reg. § 1.6694-3 (penalty for willful/reckless
//! conduct); 26 U.S.C. § 6662(d)(2)(B) (substantial-authority
//! standard); 26 U.S.C. § 6662A (reportable-transaction
//! understatement — cross-reference); 26 U.S.C. § 7701(a)(36)
//! (tax return preparer definition).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Section6694Input {
    /// Preparer's income (fee) derived from preparation of the
    /// return or claim for refund (cents).
    pub preparer_fee_cents: i64,
    /// True if the position has substantial authority (generally
    /// 35-40% probability of sustaining on the merits).
    pub position_has_substantial_authority: bool,
    /// True if the position was disclosed on the return under
    /// § 6662(d)(2)(B)(ii)(I).
    pub position_was_disclosed: bool,
    /// True if the position has a reasonable basis (≥ 20%
    /// probability) — required for disclosed positions under
    /// § 6694(a)(2)(B).
    pub position_has_reasonable_basis: bool,
    /// True if the position relates to a tax shelter OR a
    /// § 6662A reportable transaction.
    pub position_is_tax_shelter_or_reportable: bool,
    /// True if it was reasonable to believe the position would
    /// be more likely than not sustained (> 50% probability).
    /// Required for tax-shelter cases under § 6694(a)(2)(C).
    pub position_more_likely_than_not_sustained: bool,
    /// True if an understatement of taxpayer's liability
    /// actually occurred.
    pub understatement_occurred: bool,
    /// True if the preparer advised the taxpayer of the
    /// applicable penalty standards (relevant to good-faith
    /// determination).
    pub preparer_advised_taxpayer_of_penalty_standards: bool,
    /// True if § 6694(a)(3) reasonable-cause + good-faith
    /// exception applies.
    pub reasonable_cause_and_good_faith: bool,
    /// True if the preparer willfully understated under
    /// § 6694(b)(2)(A).
    pub willful_understatement: bool,
    /// True if the preparer recklessly or intentionally
    /// disregarded rules/regulations under § 6694(b)(2)(B).
    pub reckless_or_intentional_disregard: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section6694Result {
    /// True if § 6694(a) unreasonable-position trigger engages
    /// (before reasonable-cause exception).
    pub unreasonable_position_engaged: bool,
    /// True if § 6694(a)(3) reasonable-cause + good-faith
    /// exception applies.
    pub reasonable_cause_excused_6694a: bool,
    /// True if § 6694(b) willful/reckless trigger engages.
    pub willful_reckless_engaged: bool,
    /// § 6694(a) penalty before § 6694(b)(3) offset (cents).
    pub section_6694a_penalty_cents: i64,
    /// § 6694(b) penalty before § 6694(b)(3) offset (cents).
    pub section_6694b_penalty_cents: i64,
    /// Total preparer penalty after § 6694(b)(3) no-stacking
    /// coordination (cents).
    pub total_preparer_penalty_cents: i64,
    pub compliant: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

/// § 6694(a) — minimum penalty floor (cents). $1,000.
pub const SECTION_6694A_MIN_CENTS: i64 = 100_000;
/// § 6694(a) — percentage of preparer fee (basis points). 50%.
pub const SECTION_6694A_PERCENT_BPS: i64 = 5000;
/// § 6694(b) — minimum penalty floor (cents). $5,000.
pub const SECTION_6694B_MIN_CENTS: i64 = 500_000;
/// § 6694(b) — percentage of preparer fee (basis points). 75%.
pub const SECTION_6694B_PERCENT_BPS: i64 = 7500;
/// Basis-point denominator.
pub const BPS_DENOMINATOR: i64 = 10_000;

pub fn compute(input: &Section6694Input) -> Section6694Result {
    let mut notes: Vec<String> = Vec::new();
    let mut violations: Vec<String> = Vec::new();

    let preparer_fee = input.preparer_fee_cents.max(0);

    // § 6694(a)(2) — unreasonable position determination.
    let unreasonable_position_engaged = if !input.understatement_occurred {
        false
    } else if input.position_is_tax_shelter_or_reportable {
        // § 6694(a)(2)(C) — shelter/reportable requires more-likely-than-not.
        !input.position_more_likely_than_not_sustained
    } else if !input.position_was_disclosed {
        // § 6694(a)(2)(A) — undisclosed + no substantial authority.
        !input.position_has_substantial_authority
    } else {
        // § 6694(a)(2)(B) — disclosed but no reasonable basis.
        !input.position_has_reasonable_basis
    };

    // § 6694(a)(3) — reasonable-cause exception.
    let reasonable_cause_excused_6694a = unreasonable_position_engaged
        && input.reasonable_cause_and_good_faith;

    // § 6694(a) penalty calc.
    let section_6694a_penalty_cents = if unreasonable_position_engaged && !reasonable_cause_excused_6694a {
        let fee_based = preparer_fee.saturating_mul(SECTION_6694A_PERCENT_BPS)
            / BPS_DENOMINATOR;
        fee_based.max(SECTION_6694A_MIN_CENTS)
    } else {
        0
    };

    // § 6694(b) — willful or reckless conduct.
    let willful_reckless_engaged = input.understatement_occurred
        && (input.willful_understatement || input.reckless_or_intentional_disregard);

    let section_6694b_penalty_cents = if willful_reckless_engaged {
        let fee_based = preparer_fee.saturating_mul(SECTION_6694B_PERCENT_BPS)
            / BPS_DENOMINATOR;
        fee_based.max(SECTION_6694B_MIN_CENTS)
    } else {
        0
    };

    // § 6694(b)(3) coordination — no stacking; (b) replaces (a) when both apply.
    let total_preparer_penalty_cents = if willful_reckless_engaged {
        // (b) penalty reduced by (a) imposed on same return.
        // Practically: total = max of the two, but at least (b) - any (a) already imposed.
        section_6694b_penalty_cents.max(section_6694a_penalty_cents)
    } else {
        section_6694a_penalty_cents
    };

    // Violations.
    if section_6694a_penalty_cents > 0 {
        violations.push(format!(
            "§ 6694(a) — unreasonable-position penalty: {} cents (greater of $1,000 OR \
             50% of preparer fee {} cents = {} cents). § 6694(a)(3) reasonable-cause \
             exception {} engaged.",
            section_6694a_penalty_cents,
            preparer_fee,
            preparer_fee.saturating_mul(SECTION_6694A_PERCENT_BPS) / BPS_DENOMINATOR,
            if reasonable_cause_excused_6694a { "ENGAGED but failed to defeat trigger" } else { "NOT" },
        ));
    }
    if section_6694b_penalty_cents > 0 {
        violations.push(format!(
            "§ 6694(b) — willful or reckless conduct penalty: {} cents (greater of $5,000 \
             OR 75% of preparer fee {} cents = {} cents). No reasonable-cause exception \
             available under § 6694(b).",
            section_6694b_penalty_cents,
            preparer_fee,
            preparer_fee.saturating_mul(SECTION_6694B_PERCENT_BPS) / BPS_DENOMINATOR,
        ));
    }

    // Position-prong notes.
    if input.understatement_occurred {
        if input.position_is_tax_shelter_or_reportable {
            notes.push(format!(
                "§ 6694(a)(2)(C) tax-shelter / § 6662A reportable-transaction path. \
                 Required standard: MORE LIKELY THAN NOT sustained on the merits. \
                 Standard {}: position {} qualify.",
                if input.position_more_likely_than_not_sustained { "MET" } else { "NOT MET" },
                if input.position_more_likely_than_not_sustained { "DOES" } else { "does NOT" },
            ));
        } else if !input.position_was_disclosed {
            notes.push(format!(
                "§ 6694(a)(2)(A) — non-disclosed position path. Required: SUBSTANTIAL \
                 AUTHORITY (35-40% probability). Substantial authority {}: position {} \
                 qualify.",
                if input.position_has_substantial_authority { "PRESENT" } else { "ABSENT" },
                if input.position_has_substantial_authority { "DOES" } else { "does NOT" },
            ));
        } else {
            notes.push(format!(
                "§ 6694(a)(2)(B) — disclosed-position path. Required: REASONABLE BASIS \
                 (≥ 20% probability). Reasonable basis {}: position {} qualify.",
                if input.position_has_reasonable_basis { "PRESENT" } else { "ABSENT" },
                if input.position_has_reasonable_basis { "DOES" } else { "does NOT" },
            ));
        }
    } else {
        notes.push(
            "No understatement occurred — § 6694(a) and § 6694(b) triggers do not engage. \
             Preparer remains subject to other obligations (e.g., § 6695 information \
             return requirements, § 6107 copy retention)."
                .to_string(),
        );
    }

    if input.preparer_advised_taxpayer_of_penalty_standards {
        notes.push(
            "Preparer advised taxpayer of applicable penalty standards — relevant to \
             § 6694(a)(3) good-faith determination but does NOT defeat the underlying \
             unreasonable-position trigger; only reasonable cause + good faith excuses \
             the (a) penalty."
                .to_string(),
        );
    }

    if section_6694b_penalty_cents > 0 && section_6694a_penalty_cents > 0 {
        notes.push(format!(
            "§ 6694(b)(3) NO-STACKING COORDINATION engaged. Both § 6694(a) ({} cents) and \
             § 6694(b) ({} cents) trigger; total penalty = max of the two = {} cents.",
            section_6694a_penalty_cents,
            section_6694b_penalty_cents,
            total_preparer_penalty_cents,
        ));
    }

    notes.push(
        "Sibling preparer + promoter penalty cluster: § 6695 (preparer information return \
         penalties — signature, identification, due diligence); § 6700 (promoter \
         penalties — abusive tax shelter promotion); § 6701 (aiding and abetting \
         understatement of tax liability). Related taxpayer-side penalties: § 6662 \
         (accuracy penalty), § 6662A (reportable-transaction-understatement), § 6707A \
         (disclosure failure). § 7701(a)(36) defines 'tax return preparer'. Trader-CPAs \
         commonly face § 6694 exposure on Schedule D positions, § 475(f) mark-to-market \
         elections, § 1091 wash-sale aggregation, and partnership K-1 allocations."
            .to_string(),
    );

    let compliant = violations.is_empty();

    Section6694Result {
        unreasonable_position_engaged,
        reasonable_cause_excused_6694a,
        willful_reckless_engaged,
        section_6694a_penalty_cents,
        section_6694b_penalty_cents,
        total_preparer_penalty_cents,
        compliant,
        violations,
        citation: "26 U.S.C. § 6694 (general); 26 U.S.C. § 6694(a) (unreasonable-position \
                   penalty); 26 U.S.C. § 6694(a)(2)(A) (no-substantial-authority + \
                   non-disclosure prong); 26 U.S.C. § 6694(a)(2)(B) (disclosed-without-\
                   reasonable-basis prong); 26 U.S.C. § 6694(a)(2)(C) (tax-shelter / \
                   § 6662A more-likely-than-not prong); 26 U.S.C. § 6694(a)(3) \
                   (reasonable-cause + good-faith exception); 26 U.S.C. § 6694(b) \
                   (willful/reckless penalty); 26 U.S.C. § 6694(b)(2)(A) (willful); \
                   26 U.S.C. § 6694(b)(2)(B) (reckless/intentional disregard); \
                   26 U.S.C. § 6694(b)(3) (no-stacking coordination); Treas. Reg. \
                   § 1.6694-1 (general regulations); Treas. Reg. § 1.6694-2 (penalty \
                   for unreasonable position); Treas. Reg. § 1.6694-3 (penalty for \
                   willful/reckless); 26 U.S.C. § 6662(d)(2)(B) (substantial-authority \
                   standard); 26 U.S.C. § 6662A (reportable-transaction-understatement); \
                   26 U.S.C. § 7701(a)(36) (preparer definition)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> Section6694Input {
        Section6694Input {
            preparer_fee_cents: 100_000, // $1,000 fee
            position_has_substantial_authority: true,
            position_was_disclosed: false,
            position_has_reasonable_basis: true,
            position_is_tax_shelter_or_reportable: false,
            position_more_likely_than_not_sustained: false,
            understatement_occurred: true,
            preparer_advised_taxpayer_of_penalty_standards: true,
            reasonable_cause_and_good_faith: false,
            willful_understatement: false,
            reckless_or_intentional_disregard: false,
        }
    }

    // ── No understatement → no penalty ─────────────────────────

    #[test]
    fn no_understatement_no_penalty() {
        let mut b = input();
        b.understatement_occurred = false;
        let r = compute(&b);
        assert!(!r.unreasonable_position_engaged);
        assert!(!r.willful_reckless_engaged);
        assert_eq!(r.total_preparer_penalty_cents, 0);
        assert!(r.compliant);
    }

    // ── § 6694(a)(2)(A) — undisclosed without substantial authority ─

    #[test]
    fn undisclosed_with_substantial_authority_no_penalty() {
        let r = compute(&input());
        assert!(!r.unreasonable_position_engaged);
        assert_eq!(r.section_6694a_penalty_cents, 0);
    }

    #[test]
    fn undisclosed_without_substantial_authority_penalty() {
        let mut b = input();
        b.position_has_substantial_authority = false;
        let r = compute(&b);
        assert!(r.unreasonable_position_engaged);
        // Fee $1K × 50% = $500 → floor $1,000 wins.
        assert_eq!(r.section_6694a_penalty_cents, SECTION_6694A_MIN_CENTS);
    }

    // ── § 6694(a)(2)(B) — disclosed without reasonable basis ──

    #[test]
    fn disclosed_with_reasonable_basis_no_penalty() {
        let mut b = input();
        b.position_was_disclosed = true;
        b.position_has_reasonable_basis = true;
        b.position_has_substantial_authority = false; // irrelevant when disclosed
        let r = compute(&b);
        assert!(!r.unreasonable_position_engaged);
    }

    #[test]
    fn disclosed_without_reasonable_basis_penalty() {
        let mut b = input();
        b.position_was_disclosed = true;
        b.position_has_reasonable_basis = false;
        let r = compute(&b);
        assert!(r.unreasonable_position_engaged);
        assert_eq!(r.section_6694a_penalty_cents, SECTION_6694A_MIN_CENTS);
    }

    // ── § 6694(a)(2)(C) — tax shelter / § 6662A ──────────────

    #[test]
    fn tax_shelter_more_likely_than_not_no_penalty() {
        let mut b = input();
        b.position_is_tax_shelter_or_reportable = true;
        b.position_more_likely_than_not_sustained = true;
        let r = compute(&b);
        assert!(!r.unreasonable_position_engaged);
    }

    #[test]
    fn tax_shelter_without_more_likely_than_not_penalty() {
        let mut b = input();
        b.position_is_tax_shelter_or_reportable = true;
        b.position_more_likely_than_not_sustained = false;
        // Substantial authority alone is NOT enough for shelter.
        b.position_has_substantial_authority = true;
        let r = compute(&b);
        assert!(r.unreasonable_position_engaged);
    }

    // ── § 6694(a)(3) reasonable-cause exception ──────────────

    #[test]
    fn reasonable_cause_excuses_6694a() {
        let mut b = input();
        b.position_has_substantial_authority = false;
        b.reasonable_cause_and_good_faith = true;
        let r = compute(&b);
        assert!(r.unreasonable_position_engaged);
        assert!(r.reasonable_cause_excused_6694a);
        assert_eq!(r.section_6694a_penalty_cents, 0);
    }

    // ── § 6694(a) penalty math — fee-based ─────────────────────

    #[test]
    fn fee_based_50_percent_exceeds_minimum() {
        let mut b = input();
        b.position_has_substantial_authority = false;
        b.preparer_fee_cents = 1_000_000; // $10,000 fee → 50% = $5,000
        let r = compute(&b);
        // 50% of $10K = $5K > $1K floor → use $5K.
        assert_eq!(r.section_6694a_penalty_cents, 500_000);
    }

    #[test]
    fn fee_based_50_percent_low_fee_floor_applies() {
        let mut b = input();
        b.position_has_substantial_authority = false;
        b.preparer_fee_cents = 50_000; // $500 fee → 50% = $250
        let r = compute(&b);
        // 50% of $500 = $250 < $1,000 floor → use floor.
        assert_eq!(r.section_6694a_penalty_cents, SECTION_6694A_MIN_CENTS);
    }

    // ── § 6694(b) willful or reckless ─────────────────────────

    #[test]
    fn willful_understatement_engages_6694b() {
        let mut b = input();
        b.willful_understatement = true;
        let r = compute(&b);
        assert!(r.willful_reckless_engaged);
        // $1K fee × 75% = $750 → floor $5,000.
        assert_eq!(r.section_6694b_penalty_cents, SECTION_6694B_MIN_CENTS);
    }

    #[test]
    fn reckless_disregard_engages_6694b() {
        let mut b = input();
        b.reckless_or_intentional_disregard = true;
        let r = compute(&b);
        assert!(r.willful_reckless_engaged);
        assert_eq!(r.section_6694b_penalty_cents, SECTION_6694B_MIN_CENTS);
    }

    #[test]
    fn no_understatement_no_6694b_even_if_willful() {
        let mut b = input();
        b.understatement_occurred = false;
        b.willful_understatement = true;
        let r = compute(&b);
        assert!(!r.willful_reckless_engaged);
        assert_eq!(r.section_6694b_penalty_cents, 0);
    }

    // ── § 6694(b) math ────────────────────────────────────────

    #[test]
    fn fee_based_75_percent_exceeds_minimum() {
        let mut b = input();
        b.willful_understatement = true;
        b.preparer_fee_cents = 1_000_000; // $10K fee → 75% = $7,500
        let r = compute(&b);
        assert_eq!(r.section_6694b_penalty_cents, 750_000);
    }

    #[test]
    fn fee_based_75_percent_low_fee_floor_applies() {
        let mut b = input();
        b.willful_understatement = true;
        b.preparer_fee_cents = 100_000; // $1K fee → 75% = $750
        let r = compute(&b);
        // Floor $5,000 wins.
        assert_eq!(r.section_6694b_penalty_cents, SECTION_6694B_MIN_CENTS);
    }

    // ── § 6694(b)(3) no-stacking coordination ────────────────

    #[test]
    fn both_a_and_b_trigger_no_stacking() {
        let mut b = input();
        b.position_has_substantial_authority = false; // engages (a)
        b.willful_understatement = true; // engages (b)
        b.preparer_fee_cents = 10_000_000; // $100K fee
        // (a) = $100K × 50% = $50K; (b) = $100K × 75% = $75K.
        // Total = max = $75K.
        let r = compute(&b);
        assert_eq!(r.section_6694a_penalty_cents, 5_000_000);
        assert_eq!(r.section_6694b_penalty_cents, 7_500_000);
        assert_eq!(r.total_preparer_penalty_cents, 7_500_000);
    }

    #[test]
    fn only_a_no_stacking_total_equals_a() {
        let mut b = input();
        b.position_has_substantial_authority = false;
        let r = compute(&b);
        assert_eq!(r.total_preparer_penalty_cents, r.section_6694a_penalty_cents);
    }

    #[test]
    fn only_b_no_stacking_total_equals_b() {
        let mut b = input();
        b.willful_understatement = true;
        let r = compute(&b);
        assert_eq!(r.total_preparer_penalty_cents, r.section_6694b_penalty_cents);
    }

    // ── Multi-regime invariants ───────────────────────────────

    #[test]
    fn b_penalty_strictly_higher_than_a_invariant() {
        // (b) floor + (b) percent both strictly higher than (a).
        assert!(SECTION_6694B_MIN_CENTS > SECTION_6694A_MIN_CENTS);
        assert!(SECTION_6694B_PERCENT_BPS > SECTION_6694A_PERCENT_BPS);
        assert_eq!(SECTION_6694B_MIN_CENTS, 5 * SECTION_6694A_MIN_CENTS);
    }

    #[test]
    fn three_unreasonable_position_paths_truth_table() {
        // Path (A): undisclosed + no substantial authority.
        let mut path_a = input();
        path_a.position_was_disclosed = false;
        path_a.position_has_substantial_authority = false;
        path_a.position_is_tax_shelter_or_reportable = false;
        assert!(compute(&path_a).unreasonable_position_engaged);

        // Path (B): disclosed + no reasonable basis.
        let mut path_b = input();
        path_b.position_was_disclosed = true;
        path_b.position_has_reasonable_basis = false;
        path_b.position_is_tax_shelter_or_reportable = false;
        assert!(compute(&path_b).unreasonable_position_engaged);

        // Path (C): tax shelter + no more-likely-than-not.
        let mut path_c = input();
        path_c.position_is_tax_shelter_or_reportable = true;
        path_c.position_more_likely_than_not_sustained = false;
        assert!(compute(&path_c).unreasonable_position_engaged);

        // None: baseline (undisclosed + substantial authority).
        assert!(!compute(&input()).unreasonable_position_engaged);
    }

    #[test]
    fn shelter_supersedes_substantial_authority_invariant() {
        // Even with substantial authority, tax-shelter path requires
        // more-likely-than-not standard.
        let mut b = input();
        b.position_is_tax_shelter_or_reportable = true;
        b.position_more_likely_than_not_sustained = false;
        b.position_has_substantial_authority = true; // not enough for shelter
        let r = compute(&b);
        assert!(r.unreasonable_position_engaged);
    }

    #[test]
    fn penalty_constants_invariant() {
        assert_eq!(SECTION_6694A_MIN_CENTS, 100_000); // $1,000
        assert_eq!(SECTION_6694A_PERCENT_BPS, 5000); // 50%
        assert_eq!(SECTION_6694B_MIN_CENTS, 500_000); // $5,000
        assert_eq!(SECTION_6694B_PERCENT_BPS, 7500); // 75%
    }

    #[test]
    fn citation_pins_all_subsections() {
        let r = compute(&input());
        assert!(r.citation.contains("§ 6694"));
        assert!(r.citation.contains("§ 6694(a)"));
        assert!(r.citation.contains("§ 6694(a)(2)(A)"));
        assert!(r.citation.contains("§ 6694(a)(2)(B)"));
        assert!(r.citation.contains("§ 6694(a)(2)(C)"));
        assert!(r.citation.contains("§ 6694(a)(3)"));
        assert!(r.citation.contains("§ 6694(b)"));
        assert!(r.citation.contains("§ 6694(b)(2)(A)"));
        assert!(r.citation.contains("§ 6694(b)(2)(B)"));
        assert!(r.citation.contains("§ 6694(b)(3)"));
        assert!(r.citation.contains("§ 1.6694-1"));
        assert!(r.citation.contains("§ 1.6694-2"));
        assert!(r.citation.contains("§ 1.6694-3"));
        assert!(r.citation.contains("§ 6662(d)(2)(B)"));
        assert!(r.citation.contains("§ 6662A"));
        assert!(r.citation.contains("§ 7701(a)(36)"));
    }

    #[test]
    fn sibling_cluster_note_present() {
        let r = compute(&input());
        assert!(
            r.notes.iter().any(|n| n.contains("§ 6695")
                && n.contains("§ 6700")
                && n.contains("§ 6701")
                && n.contains("§ 6662A")
                && n.contains("§ 6707A")
                && n.contains("§ 475(f)")
                && n.contains("§ 1091")),
            "sibling cluster note must reference preparer + promoter + trader-specific positions"
        );
    }

    // ── Defensive input clamping ──────────────────────────────

    #[test]
    fn defensive_negative_preparer_fee_clamped() {
        let mut b = input();
        b.position_has_substantial_authority = false;
        b.preparer_fee_cents = -1_000_000;
        let r = compute(&b);
        // Negative fee clamped to 0; floor $1K applies.
        assert_eq!(r.section_6694a_penalty_cents, SECTION_6694A_MIN_CENTS);
    }

    #[test]
    fn zero_fee_floor_still_applies() {
        let mut b = input();
        b.position_has_substantial_authority = false;
        b.preparer_fee_cents = 0;
        let r = compute(&b);
        // 50% of 0 = 0 → floor $1,000.
        assert_eq!(r.section_6694a_penalty_cents, SECTION_6694A_MIN_CENTS);
    }

    #[test]
    fn defensive_huge_fee_no_overflow() {
        let mut b = input();
        b.position_has_substantial_authority = false;
        b.preparer_fee_cents = 10_000_000_000; // $100M fee
        let r = compute(&b);
        // 50% = $50M → 5,000,000,000 cents. No overflow.
        assert_eq!(r.section_6694a_penalty_cents, 5_000_000_000);
    }
}
