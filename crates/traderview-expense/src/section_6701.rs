//! IRC § 6701 — Penalties for aiding and abetting understatement
//! of tax liability.
//!
//! Fourth and final member of the preparer + promoter penalty
//! cluster after § 6694 (preparer substantive position, iter
//! 254), § 6695 (preparer procedural failures, iter 256), and
//! § 6700 (promoter penalties, iter 258). § 6701 captures the
//! broadest range of conduct — any person who aids, assists,
//! procures, or advises with respect to preparation of a return
//! or other document that they KNOW would result in an
//! understatement of another person's tax liability. Wider net
//! than § 6694 (preparer-specific) and § 6700 (promoter-
//! specific).
//!
//! § 6701(a) THREE-ELEMENT TEST (all required):
//!   (1) Person aids/assists/procures/advises with respect to
//!       preparation or presentation of any portion of a return,
//!       affidavit, claim, or other document;
//!   (2) Person KNOWS or HAS REASON TO BELIEVE the portion will
//!       be used in connection with any MATERIAL MATTER arising
//!       under the internal revenue laws;
//!   (3) Person KNOWS that the portion (if so used) would result
//!       in an UNDERSTATEMENT of the liability for tax of
//!       ANOTHER person.
//!
//! § 6701(b)(1) PENALTY AMOUNTS:
//!   - $1,000 per document — when document relates to tax
//!     liability of a non-corporate (natural person, partnership,
//!     trust, estate) taxpayer;
//!   - $10,000 per document — when document relates to tax
//!     liability of a CORPORATE taxpayer.
//!
//! § 6701(b)(2) ONE-PENALTY-PER-TAXPAYER-PER-PERIOD: If a person
//! is subject to a § 6701(a) penalty for any document relating
//! to a particular taxpayer for a particular taxable period,
//! that person shall NOT be subject to additional § 6701(a)
//! penalties with respect to other documents relating to the
//! SAME taxpayer for the SAME taxable period. Limits geometric
//! penalty growth when a single aider prepares multiple
//! documents for the same taxpayer.
//!
//! § 6701(f) COORDINATION WITH § 6694: No penalty shall be
//! assessed under § 6694(a) or § 6694(b) on any person with
//! respect to any document for which a § 6701(a) penalty is
//! assessed on such person. § 6701 SUPERSEDES § 6694 on the
//! same document; § 6701 is the broader and more serious
//! liability.
//!
//! Citations: 26 U.S.C. § 6701 (general); 26 U.S.C. § 6701(a)(1)
//! (aid/assist/procure/advise element); 26 U.S.C. § 6701(a)(2)
//! (material-matter knowledge element); 26 U.S.C. § 6701(a)(3)
//! (understatement-knowledge element); 26 U.S.C. § 6701(b)(1)(A)
//! ($1,000 non-corporate penalty); 26 U.S.C. § 6701(b)(1)(B)
//! ($10,000 corporate penalty); 26 U.S.C. § 6701(b)(2) (one-
//! penalty-per-taxpayer-per-period rule); 26 U.S.C. § 6701(f)
//! (§ 6694 coordination — § 6701 supersedes); 26 U.S.C.
//! § 6700 (companion promoter penalty); 26 U.S.C. § 7408
//! (injunction remedy for § 6700 + § 6701 conduct); IRM 20.1.6
//! (preparer and promoter penalty examination); IRS Chief
//! Counsel Memorandum 201531015 (per-document, not per-
//! taxpayer, penalty assessment).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Section6701Input {
    /// § 6701(a)(1) — true if person aided, assisted, procured,
    /// or advised with respect to preparation or presentation of
    /// any portion of a return, affidavit, claim, or document.
    pub aided_or_assisted_preparation: bool,
    /// § 6701(a)(2) — true if person knew or had reason to
    /// believe the portion would be used in connection with any
    /// material matter arising under the internal revenue laws.
    pub knew_or_should_have_known_material_matter: bool,
    /// § 6701(a)(3) — true if person knew that the portion (if
    /// so used) would result in an understatement of liability
    /// for tax of another person.
    pub knew_understatement_would_result: bool,
    /// § 6701(b)(1)(B) — true if any document relates to the
    /// tax liability of a corporate taxpayer ($10,000 per
    /// document rate applies for that subset).
    pub relates_to_corporate_taxpayer: bool,
    /// Total number of documents the person aided or assisted
    /// in preparing/presenting.
    pub number_of_documents: i64,
    /// Number of distinct taxpayers across all the documents.
    /// Used to apply § 6701(b)(2) one-per-taxpayer-per-period
    /// rule.
    pub number_of_distinct_taxpayers: i64,
    /// True if all documents relate to the SAME taxable period
    /// for each respective taxpayer. When false, multiple
    /// periods may produce multiple penalties per taxpayer.
    pub all_documents_same_taxable_period: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section6701Result {
    /// True if all three § 6701(a) elements engage.
    pub aiding_abetting_engaged: bool,
    /// § 6701(b)(1) per-document penalty rate (cents).
    pub per_document_penalty_cents: i64,
    /// Number of penalties assessable after § 6701(b)(2)
    /// one-per-taxpayer-per-period limit.
    pub number_of_penalties_assessable: i64,
    /// Total penalty exposure (cents).
    pub total_penalty_cents: i64,
    /// True if § 6701 engagement supersedes § 6694(a)/(b) under
    /// the § 6701(f) coordination rule.
    pub supersedes_section_6694: bool,
    pub compliant: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

/// § 6701(b)(1)(A) — non-corporate taxpayer per-document penalty.
pub const NONCORPORATE_PENALTY_CENTS: i64 = 100_000;
/// § 6701(b)(1)(B) — corporate taxpayer per-document penalty.
pub const CORPORATE_PENALTY_CENTS: i64 = 1_000_000;

pub fn compute(input: &Section6701Input) -> Section6701Result {
    let mut notes: Vec<String> = Vec::new();
    let mut violations: Vec<String> = Vec::new();

    let documents = input.number_of_documents.max(0);
    let distinct_taxpayers = input.number_of_distinct_taxpayers.max(0);

    // § 6701(a) three-element engagement.
    let aiding_abetting_engaged = input.aided_or_assisted_preparation
        && input.knew_or_should_have_known_material_matter
        && input.knew_understatement_would_result;

    // § 6701(b)(1) per-document penalty rate.
    let per_document_penalty_cents = if input.relates_to_corporate_taxpayer {
        CORPORATE_PENALTY_CENTS
    } else {
        NONCORPORATE_PENALTY_CENTS
    };

    // § 6701(b)(2) one-per-taxpayer-per-period limit.
    // When all documents are in the same taxable period:
    //   number of penalties = min(documents, distinct_taxpayers)
    // When documents span multiple periods (rare in practice):
    //   number of penalties = number of documents
    let number_of_penalties_assessable = if !aiding_abetting_engaged {
        0
    } else if input.all_documents_same_taxable_period {
        documents.min(distinct_taxpayers.max(1))
    } else {
        documents
    };

    let total_penalty_cents =
        number_of_penalties_assessable.saturating_mul(per_document_penalty_cents);

    // § 6701(f) coordination — § 6701 supersedes § 6694.
    let supersedes_section_6694 = aiding_abetting_engaged;

    if aiding_abetting_engaged {
        violations.push(format!(
            "§ 6701(a) — all three elements engaged: (1) aid/assist/procure/advise; \
             (2) knew or had reason to believe portion used in material matter; \
             (3) knew portion would result in understatement of another person's \
             liability. § 6701(b)(1)({}) penalty rate: {} cents per document × {} \
             penalties assessable = {} cents total. § 6701(f) — supersedes § 6694(a)/(b) \
             on the same document.",
            if input.relates_to_corporate_taxpayer {
                "B) ($10,000 corporate"
            } else {
                "A) ($1,000 non-corporate"
            },
            per_document_penalty_cents,
            number_of_penalties_assessable,
            total_penalty_cents,
        ));
    }

    // Engagement-element notes.
    if !input.aided_or_assisted_preparation {
        notes.push(
            "§ 6701(a)(1) — person did NOT aid, assist, procure, or advise with respect \
             to preparation or presentation. No § 6701 exposure regardless of other \
             elements."
                .to_string(),
        );
    } else if !input.knew_or_should_have_known_material_matter {
        notes.push(
            "§ 6701(a)(2) — material-matter knowledge element NOT engaged. Person did \
             not know (and had no reason to believe) that the portion would be used in \
             connection with a material matter arising under the internal revenue laws."
                .to_string(),
        );
    } else if !input.knew_understatement_would_result {
        notes.push(
            "§ 6701(a)(3) — understatement-knowledge element NOT engaged. Person did \
             not know that the portion (if so used) would result in an understatement \
             of another person's tax liability. § 6701 requires KNOWLEDGE of \
             understatement — distinguishes from § 6694's broader scienter standards."
                .to_string(),
        );
    }

    if aiding_abetting_engaged && input.all_documents_same_taxable_period && distinct_taxpayers > 0
    {
        notes.push(format!(
            "§ 6701(b)(2) — one-per-taxpayer-per-period rule engaged. {} documents \
             prepared for {} distinct taxpayers in same taxable period → max {} \
             penalties assessable (min of documents and distinct-taxpayers). Multiple \
             documents for the same taxpayer in the same period count as ONE penalty.",
            documents, distinct_taxpayers, number_of_penalties_assessable,
        ));
    }

    if aiding_abetting_engaged && !input.all_documents_same_taxable_period {
        notes.push(format!(
            "§ 6701(b)(2) — documents span multiple taxable periods. One-per-taxpayer-\
             per-period limit applies independently to each period; module computes \
             {} penalties = {} documents (each in a distinct period for assessment).",
            number_of_penalties_assessable, documents,
        ));
    }

    if input.relates_to_corporate_taxpayer {
        notes.push(format!(
            "§ 6701(b)(1)(B) — corporate-taxpayer rate engaged ($10,000 per document = \
             {} cents). 10× the non-corporate rate under § 6701(b)(1)(A) ($1,000 / \
             {} cents).",
            CORPORATE_PENALTY_CENTS, NONCORPORATE_PENALTY_CENTS,
        ));
    }

    notes.push(
        "Sibling preparer + promoter penalty cluster — § 6701 is the BROADEST member: \
         § 6694 (preparer substantive position — preparer-specific; iter 254); § 6695 \
         (preparer procedural failures — preparer-specific; iter 256); § 6700 \
         (promoter penalties — organizer/seller-specific; iter 258); § 6701 (aiding \
         and abetting — captures anyone who aids regardless of preparer or promoter \
         status). Taxpayer-side companions: § 6011 + § 6662 + § 6662A + § 6707A. \
         § 7408 provides injunction remedy for § 6700 + § 6701 conduct. § 6701(f) \
         coordination — when § 6701 engages, § 6694(a)/(b) does NOT engage on the same \
         document (§ 6701 is the broader and more serious liability)."
            .to_string(),
    );

    let compliant = violations.is_empty();

    Section6701Result {
        aiding_abetting_engaged,
        per_document_penalty_cents,
        number_of_penalties_assessable,
        total_penalty_cents,
        supersedes_section_6694,
        compliant,
        violations,
        citation: "26 U.S.C. § 6701 (general); 26 U.S.C. § 6701(a)(1) (aid/assist/\
                   procure/advise element); 26 U.S.C. § 6701(a)(2) (material-matter \
                   knowledge element); 26 U.S.C. § 6701(a)(3) (understatement-knowledge \
                   element); 26 U.S.C. § 6701(b)(1)(A) ($1,000 non-corporate penalty); \
                   26 U.S.C. § 6701(b)(1)(B) ($10,000 corporate penalty); 26 U.S.C. \
                   § 6701(b)(2) (one-penalty-per-taxpayer-per-period rule); 26 U.S.C. \
                   § 6701(f) (§ 6694 coordination — § 6701 supersedes); 26 U.S.C. \
                   § 6700 (companion promoter penalty); 26 U.S.C. § 7408 (injunction \
                   remedy); IRM 20.1.6 (preparer and promoter penalty examination); \
                   IRS Chief Counsel Memorandum 201531015 (per-document, not per-\
                   taxpayer, penalty assessment)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> Section6701Input {
        Section6701Input {
            aided_or_assisted_preparation: false,
            knew_or_should_have_known_material_matter: false,
            knew_understatement_would_result: false,
            relates_to_corporate_taxpayer: false,
            number_of_documents: 1,
            number_of_distinct_taxpayers: 1,
            all_documents_same_taxable_period: true,
        }
    }

    // ── No engagement ────────────────────────────────────────

    #[test]
    fn no_aid_no_engagement() {
        let r = compute(&input());
        assert!(!r.aiding_abetting_engaged);
        assert_eq!(r.total_penalty_cents, 0);
        assert!(r.compliant);
    }

    #[test]
    fn aid_without_material_matter_knowledge_no_engagement() {
        let mut b = input();
        b.aided_or_assisted_preparation = true;
        b.knew_or_should_have_known_material_matter = false;
        b.knew_understatement_would_result = true;
        let r = compute(&b);
        assert!(!r.aiding_abetting_engaged);
    }

    #[test]
    fn aid_without_understatement_knowledge_no_engagement() {
        let mut b = input();
        b.aided_or_assisted_preparation = true;
        b.knew_or_should_have_known_material_matter = true;
        b.knew_understatement_would_result = false;
        let r = compute(&b);
        assert!(!r.aiding_abetting_engaged);
    }

    // ── All three elements engaged ──────────────────────────

    #[test]
    fn all_three_elements_engages_non_corporate_1000() {
        let mut b = input();
        b.aided_or_assisted_preparation = true;
        b.knew_or_should_have_known_material_matter = true;
        b.knew_understatement_would_result = true;
        let r = compute(&b);
        assert!(r.aiding_abetting_engaged);
        assert_eq!(r.per_document_penalty_cents, NONCORPORATE_PENALTY_CENTS);
        assert_eq!(r.total_penalty_cents, 100_000);
        assert!(r.supersedes_section_6694);
    }

    #[test]
    fn all_three_elements_engages_corporate_10000() {
        let mut b = input();
        b.aided_or_assisted_preparation = true;
        b.knew_or_should_have_known_material_matter = true;
        b.knew_understatement_would_result = true;
        b.relates_to_corporate_taxpayer = true;
        let r = compute(&b);
        assert_eq!(r.per_document_penalty_cents, CORPORATE_PENALTY_CENTS);
        assert_eq!(r.total_penalty_cents, 1_000_000);
    }

    // ── § 6701(b)(2) one-per-taxpayer-per-period rule ──────

    #[test]
    fn ten_documents_same_taxpayer_one_period_one_penalty() {
        let mut b = input();
        b.aided_or_assisted_preparation = true;
        b.knew_or_should_have_known_material_matter = true;
        b.knew_understatement_would_result = true;
        b.number_of_documents = 10;
        b.number_of_distinct_taxpayers = 1;
        b.all_documents_same_taxable_period = true;
        let r = compute(&b);
        // 10 docs × 1 taxpayer same period → min(10, 1) = 1 penalty.
        assert_eq!(r.number_of_penalties_assessable, 1);
        assert_eq!(r.total_penalty_cents, NONCORPORATE_PENALTY_CENTS);
    }

    #[test]
    fn ten_documents_ten_taxpayers_ten_penalties() {
        let mut b = input();
        b.aided_or_assisted_preparation = true;
        b.knew_or_should_have_known_material_matter = true;
        b.knew_understatement_would_result = true;
        b.number_of_documents = 10;
        b.number_of_distinct_taxpayers = 10;
        b.all_documents_same_taxable_period = true;
        let r = compute(&b);
        // 10 docs × 10 taxpayers → min(10, 10) = 10 penalties × $1K = $10K.
        assert_eq!(r.number_of_penalties_assessable, 10);
        assert_eq!(r.total_penalty_cents, 10 * NONCORPORATE_PENALTY_CENTS);
    }

    #[test]
    fn ten_documents_ten_taxpayers_corporate_100k() {
        let mut b = input();
        b.aided_or_assisted_preparation = true;
        b.knew_or_should_have_known_material_matter = true;
        b.knew_understatement_would_result = true;
        b.relates_to_corporate_taxpayer = true;
        b.number_of_documents = 10;
        b.number_of_distinct_taxpayers = 10;
        let r = compute(&b);
        // 10 × $10K = $100K.
        assert_eq!(r.total_penalty_cents, 10 * CORPORATE_PENALTY_CENTS);
    }

    #[test]
    fn five_documents_three_taxpayers_three_penalties() {
        let mut b = input();
        b.aided_or_assisted_preparation = true;
        b.knew_or_should_have_known_material_matter = true;
        b.knew_understatement_would_result = true;
        b.number_of_documents = 5;
        b.number_of_distinct_taxpayers = 3;
        let r = compute(&b);
        // 5 docs × 3 taxpayers same period → min(5, 3) = 3 penalties.
        assert_eq!(r.number_of_penalties_assessable, 3);
    }

    #[test]
    fn multi_period_documents_each_count() {
        let mut b = input();
        b.aided_or_assisted_preparation = true;
        b.knew_or_should_have_known_material_matter = true;
        b.knew_understatement_would_result = true;
        b.number_of_documents = 5;
        b.number_of_distinct_taxpayers = 1;
        b.all_documents_same_taxable_period = false; // multi-period
        let r = compute(&b);
        // Multi-period bypasses the one-per-taxpayer cap.
        assert_eq!(r.number_of_penalties_assessable, 5);
        assert_eq!(r.total_penalty_cents, 5 * NONCORPORATE_PENALTY_CENTS);
    }

    // ── § 6701(f) coordination ──────────────────────────────

    #[test]
    fn supersedes_section_6694_when_engaged() {
        let mut b = input();
        b.aided_or_assisted_preparation = true;
        b.knew_or_should_have_known_material_matter = true;
        b.knew_understatement_would_result = true;
        let r = compute(&b);
        assert!(r.supersedes_section_6694);
    }

    #[test]
    fn does_not_supersede_when_not_engaged() {
        let r = compute(&input());
        assert!(!r.supersedes_section_6694);
    }

    // ── Corporate vs non-corporate rate ─────────────────────

    #[test]
    fn corporate_rate_10x_noncorporate_invariant() {
        // $10,000 = 10 × $1,000.
        assert_eq!(CORPORATE_PENALTY_CENTS, 10 * NONCORPORATE_PENALTY_CENTS);
    }

    #[test]
    fn corporate_taxpayer_uses_higher_rate() {
        let mut b = input();
        b.aided_or_assisted_preparation = true;
        b.knew_or_should_have_known_material_matter = true;
        b.knew_understatement_would_result = true;
        b.relates_to_corporate_taxpayer = true;
        let r = compute(&b);
        assert_eq!(r.per_document_penalty_cents, CORPORATE_PENALTY_CENTS);
    }

    #[test]
    fn non_corporate_uses_lower_rate() {
        let mut b = input();
        b.aided_or_assisted_preparation = true;
        b.knew_or_should_have_known_material_matter = true;
        b.knew_understatement_would_result = true;
        b.relates_to_corporate_taxpayer = false;
        let r = compute(&b);
        assert_eq!(r.per_document_penalty_cents, NONCORPORATE_PENALTY_CENTS);
    }

    // ── Multi-regime invariants ─────────────────────────────

    #[test]
    fn three_element_truth_table() {
        // 8-cell sweep: aid × material_matter × understatement.
        let cells = [
            (false, false, false, false),
            (true, false, false, false),
            (false, true, false, false),
            (false, false, true, false),
            (true, true, false, false),
            (true, false, true, false),
            (false, true, true, false),
            (true, true, true, true), // only all-three engages
        ];
        for (aid, mat, under, expected) in cells.iter() {
            let mut b = input();
            b.aided_or_assisted_preparation = *aid;
            b.knew_or_should_have_known_material_matter = *mat;
            b.knew_understatement_would_result = *under;
            let r = compute(&b);
            assert_eq!(
                r.aiding_abetting_engaged, *expected,
                "aid={} mat={} under={}",
                aid, mat, under
            );
        }
    }

    #[test]
    fn penalty_constants_invariant() {
        assert_eq!(NONCORPORATE_PENALTY_CENTS, 100_000); // $1,000
        assert_eq!(CORPORATE_PENALTY_CENTS, 1_000_000); // $10,000
    }

    #[test]
    fn citation_pins_all_subsections() {
        let r = compute(&input());
        assert!(r.citation.contains("§ 6701"));
        assert!(r.citation.contains("§ 6701(a)(1)"));
        assert!(r.citation.contains("§ 6701(a)(2)"));
        assert!(r.citation.contains("§ 6701(a)(3)"));
        assert!(r.citation.contains("§ 6701(b)(1)(A)"));
        assert!(r.citation.contains("§ 6701(b)(1)(B)"));
        assert!(r.citation.contains("§ 6701(b)(2)"));
        assert!(r.citation.contains("§ 6701(f)"));
        assert!(r.citation.contains("§ 6700"));
        assert!(r.citation.contains("§ 7408"));
        assert!(r.citation.contains("IRM 20.1.6"));
        assert!(r.citation.contains("201531015"));
    }

    #[test]
    fn sibling_cluster_note_present() {
        let r = compute(&input());
        assert!(
            r.notes.iter().any(|n| n.contains("§ 6694")
                && n.contains("§ 6695")
                && n.contains("§ 6700")
                && n.contains("§ 6701")
                && n.contains("§ 6011")
                && n.contains("§ 6662")
                && n.contains("§ 6662A")
                && n.contains("§ 6707A")
                && n.contains("§ 7408")
                && n.contains("BROADEST member")),
            "sibling cluster note must reference full preparer + promoter + taxpayer-side cluster + § 6701 broadest position"
        );
    }

    // ── Defensive input clamping ────────────────────────────

    #[test]
    fn defensive_negative_documents_clamped() {
        let mut b = input();
        b.aided_or_assisted_preparation = true;
        b.knew_or_should_have_known_material_matter = true;
        b.knew_understatement_would_result = true;
        b.number_of_documents = -5;
        let r = compute(&b);
        assert_eq!(r.number_of_penalties_assessable, 0);
        assert_eq!(r.total_penalty_cents, 0);
    }

    #[test]
    fn defensive_negative_taxpayers_clamped_to_floor_at_one() {
        let mut b = input();
        b.aided_or_assisted_preparation = true;
        b.knew_or_should_have_known_material_matter = true;
        b.knew_understatement_would_result = true;
        b.number_of_distinct_taxpayers = -2;
        let r = compute(&b);
        // Negative clamped to 0; max(0, 1) floor = 1; min(1 doc, 1) = 1 penalty.
        assert_eq!(r.number_of_penalties_assessable, 1);
    }

    #[test]
    fn extreme_documents_no_overflow() {
        let mut b = input();
        b.aided_or_assisted_preparation = true;
        b.knew_or_should_have_known_material_matter = true;
        b.knew_understatement_would_result = true;
        b.relates_to_corporate_taxpayer = true;
        b.number_of_documents = 100_000;
        b.number_of_distinct_taxpayers = 100_000;
        let r = compute(&b);
        // 100K × $10K = $1B = 100B cents.
        assert_eq!(r.total_penalty_cents, 100_000_000_000);
    }

    #[test]
    fn no_engagement_no_supersession() {
        let mut b = input();
        b.aided_or_assisted_preparation = true;
        b.knew_or_should_have_known_material_matter = true;
        b.knew_understatement_would_result = false; // last element missing
        let r = compute(&b);
        assert!(!r.aiding_abetting_engaged);
        assert!(!r.supersedes_section_6694);
        assert_eq!(r.total_penalty_cents, 0);
    }

    #[test]
    fn missing_element_specific_notes() {
        // Missing element 1 (aid).
        let r1 = compute(&input());
        assert!(r1.notes.iter().any(|n| n.contains("§ 6701(a)(1)")));

        // Missing element 2 (material matter).
        let mut b2 = input();
        b2.aided_or_assisted_preparation = true;
        let r2 = compute(&b2);
        assert!(r2.notes.iter().any(|n| n.contains("§ 6701(a)(2)")));

        // Missing element 3 (understatement).
        let mut b3 = input();
        b3.aided_or_assisted_preparation = true;
        b3.knew_or_should_have_known_material_matter = true;
        let r3 = compute(&b3);
        assert!(r3.notes.iter().any(|n| n.contains("§ 6701(a)(3)")));
    }
}
