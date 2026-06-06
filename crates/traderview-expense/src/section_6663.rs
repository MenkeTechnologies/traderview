//! IRC § 6663 — Civil fraud penalty. 75% of the portion of any
//! underpayment attributable to fraud. The harshest civil tax
//! penalty in the Code (excluding § 6672 TFRP's 100% personal
//! liability). Natural sibling to `section_6664` (reasonable
//! cause + good faith defense), `section_6501` (§ 6501(c)(1)
//! unlimited ASED for fraud), `section_6502` (CSED collection
//! framework), `section_6662` (accuracy-related penalty —
//! NON-STACKABLE with § 6663 on same understatement), and
//! `section_6672` (TFRP — separate trust-fund-tax track).
//!
//! **§ 6663(a) — 75% penalty rate**. If any portion of an
//! underpayment is attributable to fraud, the penalty is 75% of
//! that portion. Stack against the underpayment itself + § 6601
//! interest from original due date.
//!
//! **§ 6663(b) — Burden-shift rule**. If the Secretary
//! establishes that ANY PORTION of an underpayment is
//! attributable to fraud, the ENTIRE underpayment shall be
//! treated as attributable to fraud, EXCEPT with respect to any
//! portion which the taxpayer establishes (by PREPONDERANCE OF
//! EVIDENCE) is NOT attributable to fraud. Once IRS proves
//! fraud on $1 of underpayment, ALL underpayment is treated as
//! fraud unless taxpayer carves out specific portions.
//!
//! **§ 6663(c) — Joint return innocent spouse exception**. On a
//! joint return, the § 6663 penalty does NOT apply to a spouse
//! UNLESS some part of the underpayment is attributable to the
//! fraud of THAT spouse specifically. Cross-references § 6015
//! innocent spouse relief framework.
//!
//! **IRS burden of proof — CLEAR AND CONVINCING EVIDENCE**.
//! Heightened standard (greater than preponderance, less than
//! beyond reasonable doubt). § 7454(a) imposes burden of proof
//! on IRS for fraud determinations. Direct evidence rare; IRS
//! relies on circumstantial "badges of fraud" (Spies v. United
//! States, 317 U.S. 492 (1943)) to infer fraudulent intent.
//!
//! **Badges of fraud** (non-exhaustive, from Spies + later
//! case law + IRM 25.1.6):
//! - Unreported income
//! - Failure to keep adequate books and records
//! - Concealment of books or records
//! - Failure to cooperate with IRS investigation
//! - Misleading or false statements to IRS
//! - Consistent pattern of understatement
//! - Dealing in cash to avoid reporting
//! - False documents, records, or invoices
//! - Implausible or inconsistent explanations of conduct
//!
//! **Non-stacking with § 6662**. § 6662(b)(7) prevents the §
//! 6662 accuracy-related penalty from applying to any portion
//! of underpayment to which § 6663 fraud penalty applies. The
//! penalties are MUTUALLY EXCLUSIVE on the same dollar.
//!
//! **§ 6664(c)(1) defense theoretically applies** — reasonable
//! cause + good faith defense theoretically available against
//! § 6663 civil fraud, but if fraud is established by clear and
//! convincing evidence, the defense rarely succeeds. See
//! `section_6664`.
//!
//! **Cross-references for trader-tax exposure**:
//! - § 6501(c)(1) — UNLIMITED ASED when fraud established
//!   (no statute of limitations on assessment)
//! - § 6651(f) — 75% failure-to-file penalty when fraud
//! - 11 U.S.C. § 523(a)(1)(C) — nondischargeable in personal
//!   bankruptcy when fraud established
//! - § 7201 + § 7206 — criminal prosecution may parallel civil
//!   fraud determination (Spies-Daly doctrine permits parallel
//!   civil + criminal)
//!
//! Citations: IRC § 6663(a) 75% penalty rate; § 6663(b)
//! burden-shift rule; § 6663(c) joint return innocent spouse
//! exception; § 6662(b)(7) non-stacking with accuracy penalty;
//! § 6664(c)(1) reasonable-cause defense (theoretical);
//! § 6501(c)(1) unlimited ASED for fraud; § 6651(f) 75%
//! failure-to-file penalty for fraud; § 7454(a) burden of
//! proof on IRS; 11 U.S.C. § 523(a)(1)(C) nondischargeable in
//! bankruptcy; Spies v. United States, 317 U.S. 492 (1943)
//! badges of fraud doctrine; IRM 25.1.6 Civil Fraud procedural
//! manual.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Section6663Input {
    /// Whether the IRS has met its CLEAR AND CONVINCING burden of
    /// proof on fraud (§ 7454(a) imposes burden on IRS).
    pub irs_clear_and_convincing_burden_met: bool,
    /// Total underpayment in cents.
    pub total_underpayment_cents: i64,
    /// Portion of underpayment for which IRS has proven fraud
    /// (by clear and convincing evidence), in cents.
    pub portion_proven_fraud_cents: i64,
    /// Portion that the taxpayer has rebutted as NOT attributable
    /// to fraud (by preponderance of evidence under § 6663(b)
    /// burden-shift rule), in cents.
    pub portion_taxpayer_rebutted_as_non_fraud_cents: i64,
    /// Whether the return is a joint return (§ 6663(c) innocent
    /// spouse exception only applies to joint returns).
    pub joint_return: bool,
    /// Whether the underpayment is attributable to the alleged
    /// spouse's fraud (vs the other spouse).
    pub fraud_attributable_to_alleged_spouse: bool,
    /// Whether the innocent spouse participated in the fraud.
    pub innocent_spouse_participated_in_fraud: bool,

    // Badges of fraud (Spies v. United States, 317 U.S. 492 (1943))
    pub badge_unreported_income: bool,
    pub badge_failure_to_keep_adequate_books: bool,
    pub badge_concealment_of_books_or_records: bool,
    pub badge_failure_to_cooperate_with_irs: bool,
    pub badge_misleading_or_false_statements: bool,
    pub badge_consistent_pattern_of_understatement: bool,
    pub badge_dealing_in_cash_to_avoid_reporting: bool,
    pub badge_false_documents_or_records: bool,
    pub badge_implausible_inconsistent_explanations: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section6663Result {
    /// Whether the § 6663 civil fraud penalty is imposable.
    pub penalty_imposable: bool,
    /// 75% civil fraud penalty amount in cents (saturating math).
    pub penalty_amount_cents: i64,
    /// Effective fraud-attributable portion in cents AFTER
    /// burden-shift and taxpayer rebuttal.
    pub effective_fraud_portion_cents: i64,
    /// Whether § 6663(b) burden-shift expanded the fraud portion
    /// from IRS-proven portion to entire underpayment.
    pub burden_shift_engaged: bool,
    /// Whether § 6663(c) joint return innocent spouse carveout is
    /// engaged.
    pub joint_return_innocent_spouse_carveout_engaged: bool,
    /// Count of badges of fraud present (circumstantial support
    /// for IRS clear-and-convincing burden).
    pub badges_of_fraud_count: u32,
    /// Whether the § 6662 accuracy penalty is barred by § 6662(b)(7)
    /// non-stacking rule.
    pub section_6662_accuracy_penalty_barred: bool,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section6663Input) -> Section6663Result {
    let mut notes: Vec<String> = Vec::new();

    let badges = count_badges(input);

    notes.push(
        "§ 6663(a) — 75% penalty on portion of underpayment attributable to fraud; stacks against the underpayment itself plus § 6601 interest from original due date"
            .to_string(),
    );

    notes.push(format!(
        "Spies v. United States, 317 U.S. 492 (1943) — {} of 9 enumerated badges of fraud present (circumstantial support for IRS clear-and-convincing burden)",
        badges
    ));

    if !input.irs_clear_and_convincing_burden_met {
        notes.push(
            "§ 7454(a) — IRS bears CLEAR AND CONVINCING EVIDENCE burden of proof on fraud (heightened standard; greater than preponderance, less than beyond reasonable doubt); burden NOT yet met"
                .to_string(),
        );
        return Section6663Result {
            penalty_imposable: false,
            penalty_amount_cents: 0,
            effective_fraud_portion_cents: 0,
            burden_shift_engaged: false,
            joint_return_innocent_spouse_carveout_engaged: false,
            badges_of_fraud_count: badges,
            section_6662_accuracy_penalty_barred: false,
            citation: citation(),
            notes,
        };
    }

    if input.joint_return
        && !input.fraud_attributable_to_alleged_spouse
        && !input.innocent_spouse_participated_in_fraud
    {
        notes.push(
            "§ 6663(c) — joint return innocent spouse exception engaged; § 6663 penalty does NOT apply to spouse whose conduct did not contribute to the fraud; cross-reference § 6015 innocent spouse relief"
                .to_string(),
        );
        return Section6663Result {
            penalty_imposable: false,
            penalty_amount_cents: 0,
            effective_fraud_portion_cents: 0,
            burden_shift_engaged: false,
            joint_return_innocent_spouse_carveout_engaged: true,
            badges_of_fraud_count: badges,
            section_6662_accuracy_penalty_barred: false,
            citation: citation(),
            notes,
        };
    }

    // § 6663(b) burden-shift: once IRS proves any portion as fraud,
    // entire underpayment treated as fraud unless taxpayer carves out.
    let burden_shift_engaged = input.portion_proven_fraud_cents > 0
        && input.portion_proven_fraud_cents < input.total_underpayment_cents;

    let effective_fraud_portion = if burden_shift_engaged {
        let rebutted = input.portion_taxpayer_rebutted_as_non_fraud_cents.max(0);
        let total = input.total_underpayment_cents.max(0);
        total.saturating_sub(rebutted).max(0)
    } else {
        input.portion_proven_fraud_cents.max(0)
    };

    notes.push(
        "§ 6663(b) — burden-shift rule: once IRS proves any portion of underpayment as fraud, ENTIRE underpayment is treated as fraud UNLESS taxpayer establishes (preponderance) that specific portions are NOT fraud"
            .to_string(),
    );

    if burden_shift_engaged {
        notes.push(format!(
            "§ 6663(b) burden-shift ENGAGED — IRS-proven portion (${}) less than total underpayment (${}); entire underpayment minus taxpayer-rebutted portion (${}) treated as fraud",
            input.portion_proven_fraud_cents / 100,
            input.total_underpayment_cents / 100,
            input.portion_taxpayer_rebutted_as_non_fraud_cents / 100
        ));
    }

    let penalty_amount = effective_fraud_portion
        .saturating_mul(75)
        .saturating_div(100)
        .max(0);

    notes.push(
        "§ 6662(b)(7) — § 6662 accuracy-related penalty BARRED on any portion to which § 6663 fraud penalty applies; penalties are mutually exclusive on the same dollar"
            .to_string(),
    );

    notes.push(
        "§ 6664(c)(1) reasonable-cause + good-faith defense theoretically applies but rarely succeeds where fraud established by clear and convincing evidence; see section_6664"
            .to_string(),
    );

    notes.push(
        "§ 6501(c)(1) — UNLIMITED ASED when fraud established (no assessment statute of limitations); § 6651(f) 75% failure-to-file penalty parallel when fraud"
            .to_string(),
    );

    notes.push(
        "11 U.S.C. § 523(a)(1)(C) — civil fraud tax liability NONDISCHARGEABLE in personal bankruptcy; Spies-Daly doctrine permits parallel civil + criminal prosecution under § 7201 / § 7206"
            .to_string(),
    );

    notes.push(
        "IRM 25.1.6 — Civil Fraud procedural manual governs IRS examination + appeals process for fraud determinations"
            .to_string(),
    );

    Section6663Result {
        penalty_imposable: effective_fraud_portion > 0,
        penalty_amount_cents: penalty_amount,
        effective_fraud_portion_cents: effective_fraud_portion,
        burden_shift_engaged,
        joint_return_innocent_spouse_carveout_engaged: false,
        badges_of_fraud_count: badges,
        section_6662_accuracy_penalty_barred: effective_fraud_portion > 0,
        citation: citation(),
        notes,
    }
}

fn count_badges(i: &Section6663Input) -> u32 {
    [
        i.badge_unreported_income,
        i.badge_failure_to_keep_adequate_books,
        i.badge_concealment_of_books_or_records,
        i.badge_failure_to_cooperate_with_irs,
        i.badge_misleading_or_false_statements,
        i.badge_consistent_pattern_of_understatement,
        i.badge_dealing_in_cash_to_avoid_reporting,
        i.badge_false_documents_or_records,
        i.badge_implausible_inconsistent_explanations,
    ]
    .iter()
    .filter(|&&b| b)
    .count() as u32
}

fn citation() -> &'static str {
    "IRC §§ 6663(a), 6663(b), 6663(c), 6662(b)(7), 6664(c)(1), 6501(c)(1), 6651(f), 7454(a); 11 U.S.C. § 523(a)(1)(C); Spies v. United States, 317 U.S. 492 (1943); IRM 25.1.6"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Section6663Input {
        Section6663Input {
            irs_clear_and_convincing_burden_met: true,
            total_underpayment_cents: 100_000_00,
            portion_proven_fraud_cents: 100_000_00,
            portion_taxpayer_rebutted_as_non_fraud_cents: 0,
            joint_return: false,
            fraud_attributable_to_alleged_spouse: true,
            innocent_spouse_participated_in_fraud: false,
            badge_unreported_income: false,
            badge_failure_to_keep_adequate_books: false,
            badge_concealment_of_books_or_records: false,
            badge_failure_to_cooperate_with_irs: false,
            badge_misleading_or_false_statements: false,
            badge_consistent_pattern_of_understatement: false,
            badge_dealing_in_cash_to_avoid_reporting: false,
            badge_false_documents_or_records: false,
            badge_implausible_inconsistent_explanations: false,
        }
    }

    #[test]
    fn full_underpayment_fraud_75_percent_penalty() {
        let r = check(&base());
        assert!(r.penalty_imposable);
        assert_eq!(r.penalty_amount_cents, 7_500_000);
        assert_eq!(r.effective_fraud_portion_cents, 10_000_000);
    }

    #[test]
    fn irs_burden_not_met_no_penalty() {
        let mut i = base();
        i.irs_clear_and_convincing_burden_met = false;
        let r = check(&i);
        assert!(!r.penalty_imposable);
        assert_eq!(r.penalty_amount_cents, 0);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 7454(a)") && n.contains("CLEAR AND CONVINCING")));
    }

    #[test]
    fn burden_shift_engaged_when_portion_proven_less_than_total() {
        let mut i = base();
        i.total_underpayment_cents = 10_000_000;
        i.portion_proven_fraud_cents = 1_000_000;
        let r = check(&i);
        assert!(r.burden_shift_engaged);
        assert_eq!(r.effective_fraud_portion_cents, 10_000_000);
        assert_eq!(r.penalty_amount_cents, 7_500_000);
    }

    #[test]
    fn burden_shift_with_taxpayer_rebuttal_carves_out() {
        let mut i = base();
        i.total_underpayment_cents = 10_000_000;
        i.portion_proven_fraud_cents = 1_000_000;
        i.portion_taxpayer_rebutted_as_non_fraud_cents = 4_000_000;
        let r = check(&i);
        assert!(r.burden_shift_engaged);
        assert_eq!(r.effective_fraud_portion_cents, 6_000_000);
        assert_eq!(r.penalty_amount_cents, 4_500_000);
    }

    #[test]
    fn no_burden_shift_when_full_amount_proven() {
        let r = check(&base());
        assert!(!r.burden_shift_engaged);
        assert_eq!(r.effective_fraud_portion_cents, 10_000_000);
    }

    #[test]
    fn joint_return_innocent_spouse_carveout_engaged() {
        let mut i = base();
        i.joint_return = true;
        i.fraud_attributable_to_alleged_spouse = false;
        i.innocent_spouse_participated_in_fraud = false;
        let r = check(&i);
        assert!(r.joint_return_innocent_spouse_carveout_engaged);
        assert!(!r.penalty_imposable);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6663(c)") && n.contains("§ 6015")));
    }

    #[test]
    fn joint_return_innocent_spouse_participated_no_carveout() {
        let mut i = base();
        i.joint_return = true;
        i.fraud_attributable_to_alleged_spouse = false;
        i.innocent_spouse_participated_in_fraud = true;
        let r = check(&i);
        assert!(!r.joint_return_innocent_spouse_carveout_engaged);
        assert!(r.penalty_imposable);
    }

    #[test]
    fn joint_return_with_own_fraud_no_carveout() {
        let mut i = base();
        i.joint_return = true;
        i.fraud_attributable_to_alleged_spouse = true;
        let r = check(&i);
        assert!(!r.joint_return_innocent_spouse_carveout_engaged);
        assert!(r.penalty_imposable);
    }

    #[test]
    fn non_joint_return_no_carveout_possible() {
        let mut i = base();
        i.joint_return = false;
        i.fraud_attributable_to_alleged_spouse = false;
        i.innocent_spouse_participated_in_fraud = false;
        let r = check(&i);
        assert!(!r.joint_return_innocent_spouse_carveout_engaged);
        assert!(r.penalty_imposable);
    }

    #[test]
    fn section_6662_accuracy_penalty_barred_when_6663_imposable() {
        let r = check(&base());
        assert!(r.section_6662_accuracy_penalty_barred);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6662(b)(7)") && n.contains("BARRED")));
    }

    #[test]
    fn section_6662_accuracy_penalty_not_barred_when_6663_not_imposable() {
        let mut i = base();
        i.irs_clear_and_convincing_burden_met = false;
        let r = check(&i);
        assert!(!r.section_6662_accuracy_penalty_barred);
    }

    #[test]
    fn badges_count_zero_when_no_badges() {
        let r = check(&base());
        assert_eq!(r.badges_of_fraud_count, 0);
    }

    #[test]
    fn badges_count_increments_per_badge() {
        let mut i = base();
        i.badge_unreported_income = true;
        i.badge_failure_to_keep_adequate_books = true;
        i.badge_concealment_of_books_or_records = true;
        let r = check(&i);
        assert_eq!(r.badges_of_fraud_count, 3);
    }

    #[test]
    fn all_nine_badges_present() {
        let mut i = base();
        i.badge_unreported_income = true;
        i.badge_failure_to_keep_adequate_books = true;
        i.badge_concealment_of_books_or_records = true;
        i.badge_failure_to_cooperate_with_irs = true;
        i.badge_misleading_or_false_statements = true;
        i.badge_consistent_pattern_of_understatement = true;
        i.badge_dealing_in_cash_to_avoid_reporting = true;
        i.badge_false_documents_or_records = true;
        i.badge_implausible_inconsistent_explanations = true;
        let r = check(&i);
        assert_eq!(r.badges_of_fraud_count, 9);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Spies v. United States") && n.contains("9 of 9")));
    }

    #[test]
    fn spies_case_always_cited_in_notes() {
        let r = check(&base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Spies v. United States, 317 U.S. 492 (1943)")));
    }

    #[test]
    fn unlimited_ased_cross_reference_note() {
        let r = check(&base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6501(c)(1)") && n.contains("UNLIMITED ASED")));
    }

    #[test]
    fn parallel_failure_to_file_75_percent_note() {
        let r = check(&base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6651(f)") && n.contains("75%")));
    }

    #[test]
    fn nondischargeable_bankruptcy_note() {
        let r = check(&base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 523(a)(1)(C)") && n.contains("NONDISCHARGEABLE")));
    }

    #[test]
    fn criminal_parallel_prosecution_note() {
        let r = check(&base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Spies-Daly") && n.contains("§ 7201") && n.contains("§ 7206")));
    }

    #[test]
    fn section_6664_defense_cross_reference_note() {
        let r = check(&base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6664(c)(1)") && n.contains("rarely succeeds")));
    }

    #[test]
    fn irm_25_1_6_note_present() {
        let r = check(&base());
        assert!(r.notes.iter().any(|n| n.contains("IRM 25.1.6")));
    }

    #[test]
    fn citation_pins_all_subsections_and_supporting_authorities() {
        let r = check(&base());
        assert!(r.citation.contains("§§ 6663(a), 6663(b), 6663(c)"));
        assert!(r.citation.contains("6662(b)(7)"));
        assert!(r.citation.contains("6664(c)(1)"));
        assert!(r.citation.contains("6501(c)(1)"));
        assert!(r.citation.contains("6651(f)"));
        assert!(r.citation.contains("7454(a)"));
        assert!(r.citation.contains("§ 523(a)(1)(C)"));
        assert!(r.citation.contains("Spies v. United States"));
        assert!(r.citation.contains("IRM 25.1.6"));
    }

    #[test]
    fn negative_underpayment_clamped() {
        let mut i = base();
        i.total_underpayment_cents = -1_000_000;
        i.portion_proven_fraud_cents = -500_000;
        let r = check(&i);
        assert_eq!(r.effective_fraud_portion_cents, 0);
        assert_eq!(r.penalty_amount_cents, 0);
        assert!(!r.penalty_imposable);
    }

    #[test]
    fn i64_max_saturating_no_overflow() {
        let mut i = base();
        i.total_underpayment_cents = i64::MAX;
        i.portion_proven_fraud_cents = i64::MAX;
        let r = check(&i);
        assert!(r.penalty_amount_cents > 0);
    }

    #[test]
    fn taxpayer_rebuttal_excess_clamped_to_zero_floor() {
        let mut i = base();
        i.total_underpayment_cents = 10_000_000;
        i.portion_proven_fraud_cents = 1_000_000;
        i.portion_taxpayer_rebutted_as_non_fraud_cents = 50_000_000;
        let r = check(&i);
        assert!(r.burden_shift_engaged);
        assert_eq!(r.effective_fraud_portion_cents, 0);
        assert_eq!(r.penalty_amount_cents, 0);
    }

    #[test]
    fn burden_shift_only_when_partial_proof() {
        let mut i_full = base();
        i_full.portion_proven_fraud_cents = i_full.total_underpayment_cents;
        assert!(!check(&i_full).burden_shift_engaged);

        let mut i_partial = base();
        i_partial.portion_proven_fraud_cents = 1_000_000;
        i_partial.total_underpayment_cents = 10_000_000;
        assert!(check(&i_partial).burden_shift_engaged);

        let mut i_zero = base();
        i_zero.portion_proven_fraud_cents = 0;
        assert!(!check(&i_zero).burden_shift_engaged);
    }

    #[test]
    fn innocent_spouse_carveout_only_on_joint_return() {
        let mut i_non_joint = base();
        i_non_joint.joint_return = false;
        i_non_joint.fraud_attributable_to_alleged_spouse = false;
        i_non_joint.innocent_spouse_participated_in_fraud = false;
        let r = check(&i_non_joint);
        assert!(!r.joint_return_innocent_spouse_carveout_engaged);

        let mut i_joint = base();
        i_joint.joint_return = true;
        i_joint.fraud_attributable_to_alleged_spouse = false;
        i_joint.innocent_spouse_participated_in_fraud = false;
        let r_joint = check(&i_joint);
        assert!(r_joint.joint_return_innocent_spouse_carveout_engaged);
    }

    #[test]
    fn penalty_is_75_percent_invariant_across_amounts() {
        for amount_cents in [100_00i64, 10_000_00i64, 1_000_000_00i64] {
            let mut i = base();
            i.total_underpayment_cents = amount_cents;
            i.portion_proven_fraud_cents = amount_cents;
            let r = check(&i);
            let expected = amount_cents.saturating_mul(75).saturating_div(100);
            assert_eq!(r.penalty_amount_cents, expected);
        }
    }

    #[test]
    fn zero_underpayment_zero_penalty() {
        let mut i = base();
        i.total_underpayment_cents = 0;
        i.portion_proven_fraud_cents = 0;
        let r = check(&i);
        assert_eq!(r.penalty_amount_cents, 0);
        assert!(!r.penalty_imposable);
    }
}
