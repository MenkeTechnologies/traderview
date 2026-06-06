//! IRC § 6700 — Promoting abusive tax shelters, etc.
//!
//! Third member of the preparer + promoter penalty cluster after
//! § 6694 (preparer substantive position, iter 254) and § 6695
//! (preparer procedural failures, iter 256). § 6700 targets the
//! PROMOTER side — the organizer/seller of abusive tax shelter
//! plans or arrangements, not the preparer of the underlying
//! return. Effective since January 1, 1990; substantially
//! amended by the American Jobs Creation Act of 2004 (AJCA)
//! which raised the false-statement penalty from $1,000 to 50%
//! of gross income.
//!
//! § 6700(a) — TWO-PRONG STRUCTURE:
//!   Prong 1 (§ 6700(a)(1)): Person organizes, assists in the
//!     organization of, or participates (directly or indirectly)
//!     in the sale of any interest in a partnership, investment
//!     plan, arrangement, or other entity;
//!   Prong 2 (§ 6700(a)(2)): EITHER:
//!     (A) Makes or furnishes (or causes another to make or
//!         furnish) a FALSE OR FRAUDULENT STATEMENT about any
//!         material matter; OR
//!     (B) Makes or furnishes a GROSS VALUATION OVERSTATEMENT.
//!
//! § 6700(b)(1) — GROSS VALUATION OVERSTATEMENT defined: any
//! statement as to the value of property or services where the
//! stated value EXCEEDS 200 PERCENT of the correct value, AND
//! the value is directly related to the amount of any deduction
//! or credit allowable to a participant.
//!
//! § 6700 SCIENTER REQUIREMENT (false-statement prong only):
//! Promoter must KNOW or have REASON TO KNOW the statement is
//! false. No scienter requirement for gross valuation
//! overstatement under § 6700(b).
//!
//! § 6700 PENALTY STRUCTURE (post-AJCA 2004):
//!   § 6700(a) FALSE/FRAUDULENT STATEMENT: penalty = 50% of
//!     gross income derived (or to be derived) from the activity.
//!     No cap.
//!   § 6700(b) GROSS VALUATION OVERSTATEMENT: penalty = $1,000
//!     per activity, OR (if promoter establishes that it is
//!     LESSER) 100% of gross income from the activity. Effective
//!     cap of $1,000 when gross income is high; reduced floor
//!     when gross income is low.
//!
//! § 6700 ENGAGEMENT — penalty applies REGARDLESS of (a) whether
//! a participant relies on the plan or (b) whether the
//! participant actually underreports their tax. The promoter is
//! liable for the conduct of promotion itself.
//!
//! Citations: 26 U.S.C. § 6700 (general); 26 U.S.C. § 6700(a)(1)
//! (organizer/seller prong); 26 U.S.C. § 6700(a)(2)(A) (false
//! or fraudulent statement prong); 26 U.S.C. § 6700(a)(2)(B)
//! (gross valuation overstatement prong); 26 U.S.C. § 6700(a)
//! flush (50% gross income penalty for false statements);
//! 26 U.S.C. § 6700(b)(1) (gross valuation overstatement
//! definition — 200% threshold); 26 U.S.C. § 6700(b)(1) flush
//! ($1,000 or 100% gross income penalty); American Jobs
//! Creation Act of 2004, Pub. L. 108-357 § 818 (50% gross
//! income amendment effective for activities after October 22,
//! 2004); Treas. Reg. § 301.6700-1 (formerly proposed
//! regulations); IRS Form 14242 (promoter referral). Sibling
//! preparer + promoter penalty cluster: § 6694 (preparer
//! substantive position, iter 254); § 6695 (preparer procedural
//! failures, iter 256); § 6701 (aiding and abetting
//! understatement of tax liability).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Section6700Input {
    /// § 6700(a)(1) — true if person organized, assisted in
    /// organizing, or participated in the sale of an interest
    /// in a partnership, plan, arrangement, or other entity.
    pub organized_or_promoted_plan: bool,
    /// § 6700(a)(2)(A) — true if person made or furnished a
    /// false or fraudulent statement about any material matter.
    pub made_false_or_fraudulent_statement: bool,
    /// § 6700 scienter — true if promoter knew or had reason to
    /// know the statement was false. Required for (a)(2)(A)
    /// engagement.
    pub knew_or_should_have_known_statement_false: bool,
    /// § 6700(a)(2)(B) — true if person made or furnished a
    /// gross valuation overstatement.
    pub made_gross_valuation_overstatement: bool,
    /// § 6700(b)(1) — stated valuation of property or services
    /// (cents).
    pub stated_value_cents: i64,
    /// § 6700(b)(1) — correct valuation as determined (cents).
    pub correct_value_cents: i64,
    /// § 6700(b)(1) — true if the stated value is directly
    /// related to the amount of any deduction or credit
    /// allowable to a participant.
    pub value_directly_related_to_deduction_or_credit: bool,
    /// Gross income derived (or to be derived) from the
    /// promotion activity (cents).
    pub gross_income_from_activity_cents: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section6700Result {
    /// True if § 6700(a)(1) promoter status engaged.
    pub promoter_status_engaged: bool,
    /// True if § 6700(a)(2)(A) false/fraudulent statement
    /// violation engages (statement + scienter).
    pub false_statement_violation: bool,
    /// True if § 6700(a)(2)(B) + § 6700(b)(1) gross valuation
    /// overstatement violation engages (200% threshold exceeded
    /// + direct deduction/credit relationship).
    pub gross_valuation_overstatement_violation: bool,
    /// Ratio of stated value to correct value (basis points).
    /// 20,000 = 200% (statutory threshold); higher = overstatement.
    pub valuation_overstatement_ratio_bps: i64,
    /// § 6700(a) penalty for false/fraudulent statement (cents).
    pub section_6700a_penalty_cents: i64,
    /// § 6700(b) penalty for gross valuation overstatement (cents).
    pub section_6700b_penalty_cents: i64,
    pub total_penalty_cents: i64,
    pub compliant: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

/// § 6700(a) false-statement penalty rate (basis points). 50%.
pub const FALSE_STATEMENT_PENALTY_BPS: i64 = 5000;
/// § 6700(b) gross valuation overstatement default penalty (cents).
pub const VALUATION_OVERSTATEMENT_FLOOR_CENTS: i64 = 100_000;
/// § 6700(b)(1) statutory threshold — 200% in basis points.
pub const VALUATION_OVERSTATEMENT_THRESHOLD_BPS: i64 = 20_000;
/// Basis-point denominator.
pub const BPS_DENOMINATOR: i64 = 10_000;

pub fn compute(input: &Section6700Input) -> Section6700Result {
    let mut notes: Vec<String> = Vec::new();
    let mut violations: Vec<String> = Vec::new();

    let gross_income = input.gross_income_from_activity_cents.max(0);
    let stated = input.stated_value_cents.max(0);
    let correct = input.correct_value_cents.max(0);

    // Promoter status (§ 6700(a)(1)).
    let promoter_status_engaged = input.organized_or_promoted_plan;

    // § 6700(a)(2)(A) false-statement violation — scienter required.
    let false_statement_violation = promoter_status_engaged
        && input.made_false_or_fraudulent_statement
        && input.knew_or_should_have_known_statement_false;

    // § 6700(b)(1) gross valuation overstatement.
    // Exact-integer test: stated > 200% × correct ≡ stated > 2 × correct.
    // Use saturating math to avoid overflow.
    let valuation_overstatement_ratio_bps = if correct > 0 {
        stated.saturating_mul(BPS_DENOMINATOR) / correct
    } else if stated > 0 {
        i64::MAX // infinite — stated valuation with zero correct
    } else {
        0
    };
    let valuation_exceeds_200_percent = correct > 0 && stated > correct.saturating_mul(2);
    let gross_valuation_overstatement_violation = promoter_status_engaged
        && input.made_gross_valuation_overstatement
        && valuation_exceeds_200_percent
        && input.value_directly_related_to_deduction_or_credit;

    // § 6700(a) penalty — 50% of gross income.
    let section_6700a_penalty_cents = if false_statement_violation {
        gross_income.saturating_mul(FALSE_STATEMENT_PENALTY_BPS) / BPS_DENOMINATOR
    } else {
        0
    };

    // § 6700(b) penalty — $1,000 OR (if promoter establishes lesser) 100% of gross income.
    // Practically: penalty = min($1,000, 100% × gross income) when both are computable;
    // most of the time penalty = $1,000 floor; if gross_income < $1,000, penalty drops.
    let section_6700b_penalty_cents = if gross_valuation_overstatement_violation {
        VALUATION_OVERSTATEMENT_FLOOR_CENTS.min(gross_income)
    } else {
        0
    };

    let total_penalty_cents =
        section_6700a_penalty_cents.saturating_add(section_6700b_penalty_cents);

    if false_statement_violation {
        violations.push(format!(
            "§ 6700(a)(2)(A) — false or fraudulent statement violation. Promoter \
             organized/sold plan AND made false statement AND knew or had reason to know \
             of falsity (scienter). Penalty: {} cents = 50% × {} cents gross income from \
             activity. Post-AJCA 2004 amount (raised from $1,000 by Pub. L. 108-357 \
             § 818).",
            section_6700a_penalty_cents, gross_income,
        ));
    }
    if gross_valuation_overstatement_violation {
        violations.push(format!(
            "§ 6700(a)(2)(B) + § 6700(b)(1) — gross valuation overstatement violation. \
             Stated value {} cents exceeds 200% of correct value {} cents \
             (overstatement ratio: {}% × 100 bps). Value directly related to participant \
             deduction/credit. Penalty: {} cents (min of $1,000 floor and 100% × gross \
             income {} cents).",
            stated,
            correct,
            valuation_overstatement_ratio_bps / 100,
            section_6700b_penalty_cents,
            gross_income,
        ));
    }

    // Notes.
    if !promoter_status_engaged {
        notes.push(
            "§ 6700(a)(1) promoter-status prong NOT engaged — person did not organize, \
             assist in organizing, or participate in the sale of any interest in a \
             partnership, plan, arrangement, or entity. No § 6700 exposure regardless \
             of other conduct."
                .to_string(),
        );
    } else {
        notes.push(
            "§ 6700(a)(1) PROMOTER STATUS engaged — person organized, assisted in \
             organizing, or participated in the sale of an interest in a partnership, \
             plan, arrangement, or entity. § 6700(a)(2) prong (A) or (B) must also \
             engage for penalty to attach."
                .to_string(),
        );
    }

    if input.made_false_or_fraudulent_statement && !input.knew_or_should_have_known_statement_false
    {
        notes.push(
            "§ 6700(a)(2)(A) false statement made BUT scienter (knowledge or reason to \
             know) NOT established. No § 6700(a) penalty without scienter. Consider \
             § 6694 (preparer substantive position) for preparer-side liability without \
             scienter requirement."
                .to_string(),
        );
    }

    if input.made_gross_valuation_overstatement && !valuation_exceeds_200_percent {
        notes.push(format!(
            "§ 6700(b)(1) gross valuation overstatement claimed BUT 200% threshold NOT \
             exceeded — stated value {} cents is only {}% of correct value {} cents. \
             § 6700(b)(1) threshold is STRICT EXCEEDS 200%, not ≥ 200%.",
            stated,
            valuation_overstatement_ratio_bps / 100,
            correct,
        ));
    }

    if input.made_gross_valuation_overstatement
        && valuation_exceeds_200_percent
        && !input.value_directly_related_to_deduction_or_credit
    {
        notes.push(
            "§ 6700(b)(1) — stated value exceeds 200% threshold but value NOT directly \
             related to participant deduction/credit. § 6700(b)(1) requires both prongs \
             (200% overstatement AND direct relationship). No § 6700(b) penalty."
                .to_string(),
        );
    }

    if gross_valuation_overstatement_violation && gross_income < VALUATION_OVERSTATEMENT_FLOOR_CENTS
    {
        notes.push(format!(
            "§ 6700(b)(1) — gross income {} cents is LESS than $1,000 floor. Penalty \
             reduced to 100% × gross income = {} cents (statute permits LESSER amount \
             when promoter establishes gross income is below $1,000 floor).",
            gross_income, section_6700b_penalty_cents,
        ));
    }

    notes.push(
        "§ 6700 ENGAGEMENT — penalty applies REGARDLESS of (a) whether a participant \
         relied on the plan or (b) whether the participant actually underreported tax. \
         The promoter is liable for the conduct of promotion itself. § 6700 penalty \
         survives even when no taxpayer claims the deduction/credit at issue."
            .to_string(),
    );

    notes.push(
        "Sibling preparer + promoter penalty cluster: § 6694 (preparer substantive \
         position — unreasonable position or willful/reckless conduct, iter 254); \
         § 6695 (preparer procedural failures — copy/signature/PTIN/retention/info \
         return + due diligence on credits, iter 256); § 6701 (aiding and abetting \
         understatement of tax liability — flat penalty per document). Taxpayer-side \
         companions: § 6011 (taxpayer disclosure Form 8886), § 6662 (accuracy), \
         § 6662A (reportable-transaction-understatement), § 6707A (taxpayer disclosure \
         failure). § 6700 + § 6701 + § 7408 (injunction) form the principal anti-\
         promoter enforcement arsenal."
            .to_string(),
    );

    let compliant = violations.is_empty();

    Section6700Result {
        promoter_status_engaged,
        false_statement_violation,
        gross_valuation_overstatement_violation,
        valuation_overstatement_ratio_bps,
        section_6700a_penalty_cents,
        section_6700b_penalty_cents,
        total_penalty_cents,
        compliant,
        violations,
        citation: "26 U.S.C. § 6700 (general); 26 U.S.C. § 6700(a)(1) (organizer/seller \
                   prong); 26 U.S.C. § 6700(a)(2)(A) (false or fraudulent statement \
                   prong); 26 U.S.C. § 6700(a)(2)(B) (gross valuation overstatement \
                   prong); 26 U.S.C. § 6700(a) flush (50% gross income penalty); \
                   26 U.S.C. § 6700(b)(1) (gross valuation overstatement definition — \
                   200% threshold); 26 U.S.C. § 6700(b)(1) flush ($1,000 or 100% gross \
                   income penalty); American Jobs Creation Act of 2004, Pub. L. 108-357 \
                   § 818 (50% gross income amendment effective for activities after \
                   October 22, 2004); IRS Form 14242 (promoter referral); 26 U.S.C. \
                   § 7408 (injunction remedy for § 6700 + § 6701 conduct)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> Section6700Input {
        Section6700Input {
            organized_or_promoted_plan: false,
            made_false_or_fraudulent_statement: false,
            knew_or_should_have_known_statement_false: false,
            made_gross_valuation_overstatement: false,
            stated_value_cents: 0,
            correct_value_cents: 0,
            value_directly_related_to_deduction_or_credit: false,
            gross_income_from_activity_cents: 0,
        }
    }

    // ── Promoter status threshold ─────────────────────────────

    #[test]
    fn no_promoter_status_no_penalty() {
        let mut b = input();
        // Even with false statement + scienter, no promoter status = no penalty.
        b.made_false_or_fraudulent_statement = true;
        b.knew_or_should_have_known_statement_false = true;
        b.gross_income_from_activity_cents = 10_000_000; // $100K
        let r = compute(&b);
        assert!(!r.promoter_status_engaged);
        assert!(!r.false_statement_violation);
        assert_eq!(r.total_penalty_cents, 0);
        assert!(r.compliant);
    }

    #[test]
    fn promoter_status_alone_no_penalty() {
        let mut b = input();
        b.organized_or_promoted_plan = true;
        let r = compute(&b);
        assert!(r.promoter_status_engaged);
        assert_eq!(r.total_penalty_cents, 0);
        // No violation prong engaged yet.
        assert!(r.compliant);
    }

    // ── § 6700(a)(2)(A) false/fraudulent statement ────────────

    #[test]
    fn false_statement_with_scienter_50_percent_penalty() {
        let mut b = input();
        b.organized_or_promoted_plan = true;
        b.made_false_or_fraudulent_statement = true;
        b.knew_or_should_have_known_statement_false = true;
        b.gross_income_from_activity_cents = 100_000_000; // $1M
        let r = compute(&b);
        assert!(r.false_statement_violation);
        // 50% of $1M = $500K.
        assert_eq!(r.section_6700a_penalty_cents, 50_000_000);
    }

    #[test]
    fn false_statement_without_scienter_no_penalty() {
        let mut b = input();
        b.organized_or_promoted_plan = true;
        b.made_false_or_fraudulent_statement = true;
        b.knew_or_should_have_known_statement_false = false; // no scienter
        b.gross_income_from_activity_cents = 100_000_000;
        let r = compute(&b);
        assert!(!r.false_statement_violation);
        assert_eq!(r.section_6700a_penalty_cents, 0);
    }

    #[test]
    fn false_statement_zero_gross_income_zero_penalty() {
        let mut b = input();
        b.organized_or_promoted_plan = true;
        b.made_false_or_fraudulent_statement = true;
        b.knew_or_should_have_known_statement_false = true;
        b.gross_income_from_activity_cents = 0;
        let r = compute(&b);
        assert!(r.false_statement_violation);
        // 50% of 0 = 0. No statutory floor for false-statement prong.
        assert_eq!(r.section_6700a_penalty_cents, 0);
        // Still flags as violation.
        assert!(!r.compliant);
    }

    // ── § 6700(b)(1) gross valuation overstatement ────────────

    #[test]
    fn valuation_at_exactly_200_percent_not_engaged() {
        let mut b = input();
        b.organized_or_promoted_plan = true;
        b.made_gross_valuation_overstatement = true;
        b.stated_value_cents = 2_000_000; // 200% of $10K
        b.correct_value_cents = 1_000_000;
        b.value_directly_related_to_deduction_or_credit = true;
        let r = compute(&b);
        // Statute reads "EXCEEDS 200%" — exact 200% does not engage.
        assert!(!r.gross_valuation_overstatement_violation);
        assert_eq!(r.section_6700b_penalty_cents, 0);
    }

    #[test]
    fn valuation_one_cent_above_200_percent_engaged() {
        let mut b = input();
        b.organized_or_promoted_plan = true;
        b.made_gross_valuation_overstatement = true;
        b.stated_value_cents = 2_000_001;
        b.correct_value_cents = 1_000_000;
        b.value_directly_related_to_deduction_or_credit = true;
        b.gross_income_from_activity_cents = 10_000_000;
        let r = compute(&b);
        assert!(r.gross_valuation_overstatement_violation);
    }

    #[test]
    fn valuation_300_percent_engages_with_1000_floor() {
        let mut b = input();
        b.organized_or_promoted_plan = true;
        b.made_gross_valuation_overstatement = true;
        b.stated_value_cents = 3_000_000;
        b.correct_value_cents = 1_000_000;
        b.value_directly_related_to_deduction_or_credit = true;
        b.gross_income_from_activity_cents = 10_000_000;
        let r = compute(&b);
        // Penalty = min($1,000 = 100,000 cents, 100% × $100K = 10,000,000) = 100,000.
        assert_eq!(
            r.section_6700b_penalty_cents,
            VALUATION_OVERSTATEMENT_FLOOR_CENTS
        );
    }

    #[test]
    fn valuation_overstatement_low_gross_income_reduces_penalty() {
        let mut b = input();
        b.organized_or_promoted_plan = true;
        b.made_gross_valuation_overstatement = true;
        b.stated_value_cents = 3_000_000;
        b.correct_value_cents = 1_000_000;
        b.value_directly_related_to_deduction_or_credit = true;
        b.gross_income_from_activity_cents = 50_000; // $500 — below $1,000 floor
        let r = compute(&b);
        // 100% × $500 = $500 < $1,000 floor → use lesser.
        assert_eq!(r.section_6700b_penalty_cents, 50_000);
    }

    #[test]
    fn valuation_overstatement_no_direct_deduction_no_violation() {
        let mut b = input();
        b.organized_or_promoted_plan = true;
        b.made_gross_valuation_overstatement = true;
        b.stated_value_cents = 3_000_000;
        b.correct_value_cents = 1_000_000;
        b.value_directly_related_to_deduction_or_credit = false; // critical
        b.gross_income_from_activity_cents = 10_000_000;
        let r = compute(&b);
        assert!(!r.gross_valuation_overstatement_violation);
    }

    // ── Combined violations ──────────────────────────────────

    #[test]
    fn both_prongs_penalties_sum() {
        let mut b = input();
        b.organized_or_promoted_plan = true;
        b.made_false_or_fraudulent_statement = true;
        b.knew_or_should_have_known_statement_false = true;
        b.made_gross_valuation_overstatement = true;
        b.stated_value_cents = 3_000_000;
        b.correct_value_cents = 1_000_000;
        b.value_directly_related_to_deduction_or_credit = true;
        b.gross_income_from_activity_cents = 100_000_000; // $1M
        let r = compute(&b);
        // (a) 50% × $1M = $500K; (b) min($1K, 100% × $1M) = $1K.
        assert_eq!(r.section_6700a_penalty_cents, 50_000_000);
        assert_eq!(
            r.section_6700b_penalty_cents,
            VALUATION_OVERSTATEMENT_FLOOR_CENTS
        );
        assert_eq!(r.total_penalty_cents, 50_000_000 + 100_000);
    }

    // ── Valuation ratio reporting ────────────────────────────

    #[test]
    fn valuation_ratio_in_bps_correctly_reported() {
        let mut b = input();
        b.organized_or_promoted_plan = true;
        b.stated_value_cents = 2_500_000;
        b.correct_value_cents = 1_000_000;
        let r = compute(&b);
        // 250% = 25000 bps.
        assert_eq!(r.valuation_overstatement_ratio_bps, 25_000);
    }

    #[test]
    fn valuation_ratio_zero_correct_max() {
        let mut b = input();
        b.stated_value_cents = 1_000_000;
        b.correct_value_cents = 0;
        let r = compute(&b);
        // Division by zero → reported as i64::MAX.
        assert_eq!(r.valuation_overstatement_ratio_bps, i64::MAX);
    }

    // ── Multi-regime invariants ──────────────────────────────

    #[test]
    fn promoter_status_required_for_both_prongs_invariant() {
        // Both violation flags require promoter_status_engaged.
        let mut b = input();
        b.organized_or_promoted_plan = false;
        b.made_false_or_fraudulent_statement = true;
        b.knew_or_should_have_known_statement_false = true;
        b.made_gross_valuation_overstatement = true;
        b.stated_value_cents = 3_000_000;
        b.correct_value_cents = 1_000_000;
        b.value_directly_related_to_deduction_or_credit = true;
        let r = compute(&b);
        assert!(!r.false_statement_violation);
        assert!(!r.gross_valuation_overstatement_violation);
    }

    #[test]
    fn scienter_required_for_false_statement_only_invariant() {
        // Scienter required for (a)(2)(A); NOT required for (b)(1) gross
        // valuation overstatement.
        let mut b = input();
        b.organized_or_promoted_plan = true;
        b.made_false_or_fraudulent_statement = true;
        b.knew_or_should_have_known_statement_false = false; // no scienter
        b.made_gross_valuation_overstatement = true;
        b.stated_value_cents = 3_000_000;
        b.correct_value_cents = 1_000_000;
        b.value_directly_related_to_deduction_or_credit = true;
        b.gross_income_from_activity_cents = 10_000_000;
        let r = compute(&b);
        // False-statement prong NOT engaged (no scienter).
        assert!(!r.false_statement_violation);
        // Valuation overstatement engaged regardless of scienter.
        assert!(r.gross_valuation_overstatement_violation);
    }

    #[test]
    fn valuation_threshold_boundary_truth_table() {
        // 200% threshold: stated > 2 × correct.
        let cells = [
            (1_000_000, 1_000_000, false), // 100% — not engaged
            (1_999_999, 1_000_000, false), // 199.99% — not engaged
            (2_000_000, 1_000_000, false), // exactly 200% — not engaged (strict exceeds)
            (2_000_001, 1_000_000, true),  // 200.0001% — engaged
            (5_000_000, 1_000_000, true),  // 500% — engaged
        ];
        for (stated, correct, expected) in cells.iter() {
            let mut b = input();
            b.organized_or_promoted_plan = true;
            b.made_gross_valuation_overstatement = true;
            b.stated_value_cents = *stated;
            b.correct_value_cents = *correct;
            b.value_directly_related_to_deduction_or_credit = true;
            let r = compute(&b);
            assert_eq!(
                r.gross_valuation_overstatement_violation, *expected,
                "stated={} correct={}",
                stated, correct
            );
        }
    }

    #[test]
    fn penalty_constants_invariant() {
        assert_eq!(FALSE_STATEMENT_PENALTY_BPS, 5000); // 50%
        assert_eq!(VALUATION_OVERSTATEMENT_FLOOR_CENTS, 100_000); // $1,000
        assert_eq!(VALUATION_OVERSTATEMENT_THRESHOLD_BPS, 20_000); // 200%
    }

    #[test]
    fn citation_pins_all_subsections() {
        let r = compute(&input());
        assert!(r.citation.contains("§ 6700"));
        assert!(r.citation.contains("§ 6700(a)(1)"));
        assert!(r.citation.contains("§ 6700(a)(2)(A)"));
        assert!(r.citation.contains("§ 6700(a)(2)(B)"));
        assert!(r.citation.contains("§ 6700(a) flush"));
        assert!(r.citation.contains("§ 6700(b)(1)"));
        assert!(r.citation.contains("Pub. L. 108-357 § 818"));
        assert!(r.citation.contains("October 22, 2004"));
        assert!(r.citation.contains("Form 14242"));
        assert!(r.citation.contains("§ 7408"));
    }

    #[test]
    fn sibling_cluster_note_present() {
        let r = compute(&input());
        assert!(
            r.notes.iter().any(|n| n.contains("§ 6694")
                && n.contains("§ 6695")
                && n.contains("§ 6701")
                && n.contains("§ 6011")
                && n.contains("§ 6662")
                && n.contains("§ 6662A")
                && n.contains("§ 6707A")
                && n.contains("§ 7408")),
            "sibling cluster note must reference preparer + promoter + taxpayer-side + injunction cluster"
        );
    }

    // ── Defensive input clamping ──────────────────────────────

    #[test]
    fn defensive_negative_gross_income_clamped() {
        let mut b = input();
        b.organized_or_promoted_plan = true;
        b.made_false_or_fraudulent_statement = true;
        b.knew_or_should_have_known_statement_false = true;
        b.gross_income_from_activity_cents = -1_000_000;
        let r = compute(&b);
        // Negative → 0 → no penalty.
        assert_eq!(r.section_6700a_penalty_cents, 0);
    }

    #[test]
    fn defensive_negative_valuation_clamped() {
        let mut b = input();
        b.organized_or_promoted_plan = true;
        b.made_gross_valuation_overstatement = true;
        b.stated_value_cents = -1_000_000;
        b.correct_value_cents = -500_000;
        let r = compute(&b);
        // Both clamped to 0; 0 > 0 is false → no engagement.
        assert!(!r.gross_valuation_overstatement_violation);
    }

    #[test]
    fn defensive_extreme_gross_income_no_overflow() {
        let mut b = input();
        b.organized_or_promoted_plan = true;
        b.made_false_or_fraudulent_statement = true;
        b.knew_or_should_have_known_statement_false = true;
        b.gross_income_from_activity_cents = 100_000_000_000; // $1B
        let r = compute(&b);
        // 50% × $1B = $500M.
        assert_eq!(r.section_6700a_penalty_cents, 50_000_000_000);
    }

    #[test]
    fn scienter_note_when_statement_made_without_knowledge() {
        let mut b = input();
        b.organized_or_promoted_plan = true;
        b.made_false_or_fraudulent_statement = true;
        b.knew_or_should_have_known_statement_false = false;
        let r = compute(&b);
        assert!(r.notes.iter().any(|n| n.contains("scienter")));
    }

    #[test]
    fn engagement_regardless_of_participant_reliance_note() {
        let mut b = input();
        b.organized_or_promoted_plan = true;
        b.made_false_or_fraudulent_statement = true;
        b.knew_or_should_have_known_statement_false = true;
        b.gross_income_from_activity_cents = 10_000_000;
        let r = compute(&b);
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("REGARDLESS of") && n.contains("participant relied")),
            "engagement-regardless note must be present"
        );
    }
}
