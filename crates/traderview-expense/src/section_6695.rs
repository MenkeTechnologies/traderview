//! IRC § 6695 — Other assessable penalties with respect to the
//! preparation of tax returns for other persons.
//!
//! Direct sibling to § 6694 (preparer substantive-position
//! penalty, iter 254). Where § 6694 punishes UNREASONABLE
//! POSITIONS on the return, § 6695 punishes PROCEDURAL FAILURES
//! by the preparer — failure to sign, failure to furnish copy,
//! failure to retain records, failure to exercise due diligence
//! on credit eligibility determinations. Together § 6694 +
//! § 6695 cover both the substantive and procedural sides of
//! preparer liability. Sibling cluster: § 6695 + § 6700
//! (promoter penalties) + § 6701 (aiding and abetting
//! understatement).
//!
//! Per-failure penalties of $60 each (2025; inflation-adjusted
//! from $50 original), max $31,500 per preparer per calendar
//! year per subsection (2025; inflation-adjusted):
//!
//!   § 6695(a) — FAILURE TO FURNISH COPY to taxpayer of the
//!     return or claim for refund.
//!   § 6695(b) — FAILURE TO SIGN return or claim for refund.
//!   § 6695(c) — FAILURE TO FURNISH IDENTIFYING NUMBER (PTIN)
//!     on the return or claim.
//!   § 6695(d) — FAILURE TO RETAIN COPY or list of taxpayers for
//!     three years.
//!   § 6695(e) — FAILURE TO FILE CORRECT INFORMATION RETURNS
//!     (Forms 1099 series for preparer payments to others).
//!
//! Higher-tier per-failure penalties:
//!
//!   § 6695(f) — NEGOTIATION OF REFUND CHECK by preparer. $635
//!     per check (2025; inflation-adjusted from $500 original).
//!     No annual maximum cap.
//!   § 6695(g) — FAILURE TO EXERCISE DUE DILIGENCE in
//!     determining eligibility for certain credits / filing
//!     status. $635 per failure (2025). FOUR independent
//!     credits/statuses generate separate failures:
//!       (1) EITC (Earned Income Tax Credit);
//!       (2) CTC / ACTC / ODC (Child Tax Credit / Additional
//!           Child Tax Credit / Credit for Other Dependents);
//!       (3) AOTC (American Opportunity Tax Credit);
//!       (4) HOH (Head of Household filing status).
//!     Maximum combined § 6695(g) per return = $2,540 ($635 × 4).
//!
//! Treas. Reg. § 1.6695-2(b) — DUE DILIGENCE REQUIREMENTS for
//! § 6695(g): preparer must (1) complete and submit Form 8867
//! (Paid Preparer's Earned Income Credit Checklist); (2)
//! compute the credit using applicable worksheet; (3) NOT know
//! (and have no reason to know) that information relied upon is
//! incorrect; (4) retain records for 3 years.
//!
//! Citations: 26 U.S.C. § 6695 (general); 26 U.S.C. § 6695(a)
//! (copy to taxpayer); 26 U.S.C. § 6695(b) (signing return);
//! 26 U.S.C. § 6695(c) (PTIN identifying number); 26 U.S.C.
//! § 6695(d) (retain copy or list); 26 U.S.C. § 6695(e)
//! (information return); 26 U.S.C. § 6695(f) (refund check
//! negotiation); 26 U.S.C. § 6695(g) (due diligence on
//! credits/HOH); Treas. Reg. § 1.6695-1 (general regulations);
//! Treas. Reg. § 1.6695-2 (due diligence regulations); Form 8867
//! (Paid Preparer's Earned Income Credit Checklist); Rev. Proc.
//! 2024-40 (2025 inflation adjustments). Sibling preparer +
//! promoter cluster: § 6694 (preparer substantive position),
//! § 6700 (promoter penalties), § 6701 (aiding and abetting).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Section6695Input {
    /// § 6695(a) — number of returns where copy was not
    /// furnished to taxpayer.
    pub returns_with_a_failures: i64,
    /// § 6695(b) — number of returns not signed by preparer.
    pub returns_with_b_failures: i64,
    /// § 6695(c) — number of returns missing preparer's
    /// identifying number (PTIN).
    pub returns_with_c_failures: i64,
    /// § 6695(d) — number of returns for which preparer failed
    /// to retain copy or list.
    pub returns_with_d_failures: i64,
    /// § 6695(e) — number of correct information returns
    /// (Forms 1099 series) preparer failed to file.
    pub returns_with_e_failures: i64,
    /// § 6695(f) — number of refund checks endorsed/negotiated
    /// by preparer.
    pub refund_checks_negotiated: i64,
    /// § 6695(g) — number of due diligence failures relating to
    /// EITC.
    pub due_diligence_eitc_failures: i64,
    /// § 6695(g) — number of due diligence failures relating to
    /// CTC / ACTC / ODC.
    pub due_diligence_ctc_failures: i64,
    /// § 6695(g) — number of due diligence failures relating to
    /// AOTC.
    pub due_diligence_aotc_failures: i64,
    /// § 6695(g) — number of due diligence failures relating to
    /// Head of Household filing status.
    pub due_diligence_hoh_failures: i64,
    /// Per-failure penalty for § 6695(a)/(b)/(c)/(d)/(e) — 2025
    /// default $60 = 6,000 cents; parameterized for prior years.
    pub per_failure_penalty_cents: i64,
    /// Annual maximum cap per subsection (a)/(b)/(c)/(d)/(e) —
    /// 2025 default $31,500 = 3,150,000 cents.
    pub annual_max_cap_cents: i64,
    /// § 6695(f)/(g) per-failure penalty — 2025 default $635 =
    /// 63,500 cents.
    pub higher_tier_penalty_cents: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section6695Result {
    pub section_6695a_penalty_cents: i64,
    pub section_6695b_penalty_cents: i64,
    pub section_6695c_penalty_cents: i64,
    pub section_6695d_penalty_cents: i64,
    pub section_6695e_penalty_cents: i64,
    pub section_6695f_penalty_cents: i64,
    pub section_6695g_penalty_cents: i64,
    pub total_penalty_cents: i64,
    /// True if any subsection penalty was capped at the annual
    /// maximum.
    pub any_subsection_capped: bool,
    pub compliant: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

/// 2025 § 6695(a)/(b)/(c)/(d)/(e) per-failure penalty (cents).
pub const PER_FAILURE_PENALTY_2025_CENTS: i64 = 6_000;
/// 2025 annual max cap per subsection (cents).
pub const ANNUAL_MAX_CAP_2025_CENTS: i64 = 3_150_000;
/// 2025 § 6695(f)/(g) per-failure penalty (cents).
pub const HIGHER_TIER_PENALTY_2025_CENTS: i64 = 63_500;
/// § 6695(g) four-credit maximum per return ($635 × 4 = $2,540).
pub const SECTION_6695G_MAX_PER_RETURN_CENTS: i64 = 254_000;

pub fn compute(input: &Section6695Input) -> Section6695Result {
    let mut notes: Vec<String> = Vec::new();
    let mut violations: Vec<String> = Vec::new();

    let per_failure = input.per_failure_penalty_cents.max(0);
    let annual_cap = input.annual_max_cap_cents.max(0);
    let higher_tier = input.higher_tier_penalty_cents.max(0);

    // Helper to compute capped per-failure penalty.
    let compute_capped = |failures: i64| -> (i64, bool) {
        let count = failures.max(0);
        let raw = count.saturating_mul(per_failure);
        if raw > annual_cap {
            (annual_cap, true)
        } else {
            (raw, false)
        }
    };

    let (section_6695a_penalty_cents, a_capped) = compute_capped(input.returns_with_a_failures);
    let (section_6695b_penalty_cents, b_capped) = compute_capped(input.returns_with_b_failures);
    let (section_6695c_penalty_cents, c_capped) = compute_capped(input.returns_with_c_failures);
    let (section_6695d_penalty_cents, d_capped) = compute_capped(input.returns_with_d_failures);
    let (section_6695e_penalty_cents, e_capped) = compute_capped(input.returns_with_e_failures);

    let section_6695f_penalty_cents = input
        .refund_checks_negotiated
        .max(0)
        .saturating_mul(higher_tier);

    // § 6695(g) — sum of four credit categories, each at higher-tier rate.
    let g_eitc = input
        .due_diligence_eitc_failures
        .max(0)
        .saturating_mul(higher_tier);
    let g_ctc = input
        .due_diligence_ctc_failures
        .max(0)
        .saturating_mul(higher_tier);
    let g_aotc = input
        .due_diligence_aotc_failures
        .max(0)
        .saturating_mul(higher_tier);
    let g_hoh = input
        .due_diligence_hoh_failures
        .max(0)
        .saturating_mul(higher_tier);
    let section_6695g_penalty_cents = g_eitc
        .saturating_add(g_ctc)
        .saturating_add(g_aotc)
        .saturating_add(g_hoh);

    let total_penalty_cents = section_6695a_penalty_cents
        .saturating_add(section_6695b_penalty_cents)
        .saturating_add(section_6695c_penalty_cents)
        .saturating_add(section_6695d_penalty_cents)
        .saturating_add(section_6695e_penalty_cents)
        .saturating_add(section_6695f_penalty_cents)
        .saturating_add(section_6695g_penalty_cents);

    let any_subsection_capped = a_capped || b_capped || c_capped || d_capped || e_capped;

    // Violations.
    let subsections = [
        (
            section_6695a_penalty_cents,
            "§ 6695(a) failure to furnish copy to taxpayer",
        ),
        (
            section_6695b_penalty_cents,
            "§ 6695(b) failure to sign return",
        ),
        (
            section_6695c_penalty_cents,
            "§ 6695(c) failure to furnish PTIN identifying number",
        ),
        (
            section_6695d_penalty_cents,
            "§ 6695(d) failure to retain copy or list",
        ),
        (
            section_6695e_penalty_cents,
            "§ 6695(e) failure to file correct information returns",
        ),
        (
            section_6695f_penalty_cents,
            "§ 6695(f) negotiation of refund check",
        ),
        (
            section_6695g_penalty_cents,
            "§ 6695(g) failure to exercise due diligence on credits/HOH",
        ),
    ];
    for (penalty, label) in subsections.iter() {
        if *penalty > 0 {
            violations.push(format!("{}: {} cents.", label, penalty));
        }
    }

    if any_subsection_capped {
        notes.push(format!(
            "Annual maximum cap of {} cents ($31,500 for 2025) engaged on at least one \
             subsection. Cap applies per preparer per calendar year per subsection.",
            annual_cap,
        ));
    }

    if section_6695g_penalty_cents > 0 {
        notes.push(format!(
            "§ 6695(g) due-diligence penalty engaged. Four independent credit/status \
             categories generate separate failures: EITC ({} cents), CTC/ACTC/ODC \
             ({} cents), AOTC ({} cents), HOH ({} cents). Maximum combined per return \
             = {} cents ($2,540 = $635 × 4). Treas. Reg. § 1.6695-2 requires Form 8867 \
             completion + worksheet computation + knowledge requirement + 3-year \
             retention.",
            g_eitc, g_ctc, g_aotc, g_hoh, SECTION_6695G_MAX_PER_RETURN_CENTS,
        ));
    }

    notes.push(format!(
        "Total § 6695 penalty exposure: {} cents ($60 base × {} failures + § 6695(f)/(g) \
         × {} count at higher tier $635). 2025 inflation-adjusted amounts per Rev. Proc. \
         2024-40.",
        total_penalty_cents,
        input.returns_with_a_failures.max(0)
            + input.returns_with_b_failures.max(0)
            + input.returns_with_c_failures.max(0)
            + input.returns_with_d_failures.max(0)
            + input.returns_with_e_failures.max(0),
        input.refund_checks_negotiated.max(0)
            + input.due_diligence_eitc_failures.max(0)
            + input.due_diligence_ctc_failures.max(0)
            + input.due_diligence_aotc_failures.max(0)
            + input.due_diligence_hoh_failures.max(0),
    ));

    notes.push(
        "Sibling preparer + promoter penalty cluster: § 6694 (preparer SUBSTANTIVE \
         position — unreasonable position or willful/reckless conduct, iter 254); \
         § 6700 (promoter penalties for abusive tax shelter promotion); § 6701 (aiding \
         and abetting understatement of tax liability). Together § 6694 + § 6695 cover \
         both substantive and procedural preparer liability. Taxpayer-side companions: \
         § 6662 + § 6662A + § 6707A. Form 8867 + applicable worksheet are the principal \
         compliance documents for § 6695(g) due diligence."
            .to_string(),
    );

    let compliant = violations.is_empty();

    Section6695Result {
        section_6695a_penalty_cents,
        section_6695b_penalty_cents,
        section_6695c_penalty_cents,
        section_6695d_penalty_cents,
        section_6695e_penalty_cents,
        section_6695f_penalty_cents,
        section_6695g_penalty_cents,
        total_penalty_cents,
        any_subsection_capped,
        compliant,
        violations,
        citation: "26 U.S.C. § 6695 (general); 26 U.S.C. § 6695(a) (copy to taxpayer); \
                   26 U.S.C. § 6695(b) (signing return); 26 U.S.C. § 6695(c) (PTIN \
                   identifying number); 26 U.S.C. § 6695(d) (retain copy or list); \
                   26 U.S.C. § 6695(e) (information return); 26 U.S.C. § 6695(f) (refund \
                   check negotiation); 26 U.S.C. § 6695(g) (due diligence on credits/HOH); \
                   Treas. Reg. § 1.6695-1 (general regulations); Treas. Reg. § 1.6695-2 \
                   (due diligence regulations); Form 8867 (Paid Preparer's Earned Income \
                   Credit Checklist); Rev. Proc. 2024-40 (2025 inflation adjustments)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> Section6695Input {
        Section6695Input {
            returns_with_a_failures: 0,
            returns_with_b_failures: 0,
            returns_with_c_failures: 0,
            returns_with_d_failures: 0,
            returns_with_e_failures: 0,
            refund_checks_negotiated: 0,
            due_diligence_eitc_failures: 0,
            due_diligence_ctc_failures: 0,
            due_diligence_aotc_failures: 0,
            due_diligence_hoh_failures: 0,
            per_failure_penalty_cents: PER_FAILURE_PENALTY_2025_CENTS,
            annual_max_cap_cents: ANNUAL_MAX_CAP_2025_CENTS,
            higher_tier_penalty_cents: HIGHER_TIER_PENALTY_2025_CENTS,
        }
    }

    // ── No failures → no penalty ───────────────────────────────

    #[test]
    fn no_failures_no_penalty() {
        let r = compute(&input());
        assert_eq!(r.total_penalty_cents, 0);
        assert!(r.compliant);
    }

    // ── § 6695(a)/(b)/(c)/(d)/(e) per-failure penalties ──────

    #[test]
    fn single_a_failure_60_dollars() {
        let mut b = input();
        b.returns_with_a_failures = 1;
        let r = compute(&b);
        assert_eq!(r.section_6695a_penalty_cents, 6_000);
        assert_eq!(r.total_penalty_cents, 6_000);
        assert!(!r.compliant);
    }

    #[test]
    fn single_b_failure_60_dollars() {
        let mut b = input();
        b.returns_with_b_failures = 1;
        let r = compute(&b);
        assert_eq!(r.section_6695b_penalty_cents, 6_000);
    }

    #[test]
    fn single_c_failure_60_dollars() {
        let mut b = input();
        b.returns_with_c_failures = 1;
        let r = compute(&b);
        assert_eq!(r.section_6695c_penalty_cents, 6_000);
    }

    #[test]
    fn single_d_failure_60_dollars() {
        let mut b = input();
        b.returns_with_d_failures = 1;
        let r = compute(&b);
        assert_eq!(r.section_6695d_penalty_cents, 6_000);
    }

    #[test]
    fn single_e_failure_60_dollars() {
        let mut b = input();
        b.returns_with_e_failures = 1;
        let r = compute(&b);
        assert_eq!(r.section_6695e_penalty_cents, 6_000);
    }

    // ── Annual cap ────────────────────────────────────────────

    #[test]
    fn cap_525_a_failures_at_31500() {
        let mut b = input();
        // 525 × $60 = $31,500 — exactly at cap.
        b.returns_with_a_failures = 525;
        let r = compute(&b);
        assert_eq!(r.section_6695a_penalty_cents, 3_150_000);
        // At exact cap, raw == cap so no clamping engaged; flag reports
        // engagement only when penalty was actually capped down.
        assert!(!r.any_subsection_capped);
    }

    #[test]
    fn cap_1000_a_failures_at_max() {
        let mut b = input();
        b.returns_with_a_failures = 1000;
        // 1000 × $60 = $60,000 > $31,500 → caps.
        let r = compute(&b);
        assert_eq!(r.section_6695a_penalty_cents, ANNUAL_MAX_CAP_2025_CENTS);
        assert!(r.any_subsection_capped);
    }

    #[test]
    fn each_subsection_caps_independently() {
        let mut b = input();
        b.returns_with_a_failures = 1000;
        b.returns_with_b_failures = 1000;
        b.returns_with_c_failures = 1000;
        let r = compute(&b);
        // Each of three caps at $31,500 → $94,500 total.
        assert_eq!(
            r.section_6695a_penalty_cents
                + r.section_6695b_penalty_cents
                + r.section_6695c_penalty_cents,
            3 * ANNUAL_MAX_CAP_2025_CENTS
        );
    }

    // ── § 6695(f) refund check negotiation ────────────────────

    #[test]
    fn single_refund_check_negotiation_635_dollars() {
        let mut b = input();
        b.refund_checks_negotiated = 1;
        let r = compute(&b);
        assert_eq!(r.section_6695f_penalty_cents, 63_500);
    }

    #[test]
    fn refund_check_no_annual_cap() {
        let mut b = input();
        b.refund_checks_negotiated = 100;
        // 100 × $635 = $63,500. No annual cap on § 6695(f).
        let r = compute(&b);
        assert_eq!(r.section_6695f_penalty_cents, 6_350_000);
    }

    // ── § 6695(g) due diligence ───────────────────────────────

    #[test]
    fn single_eitc_due_diligence_failure_635() {
        let mut b = input();
        b.due_diligence_eitc_failures = 1;
        let r = compute(&b);
        assert_eq!(r.section_6695g_penalty_cents, 63_500);
    }

    #[test]
    fn all_four_categories_on_same_return_2540_max() {
        let mut b = input();
        b.due_diligence_eitc_failures = 1;
        b.due_diligence_ctc_failures = 1;
        b.due_diligence_aotc_failures = 1;
        b.due_diligence_hoh_failures = 1;
        let r = compute(&b);
        // 4 × $635 = $2,540.
        assert_eq!(r.section_6695g_penalty_cents, 254_000);
        assert_eq!(SECTION_6695G_MAX_PER_RETURN_CENTS, 254_000);
    }

    #[test]
    fn ctc_failure_only_engages_g() {
        let mut b = input();
        b.due_diligence_ctc_failures = 2;
        let r = compute(&b);
        assert_eq!(r.section_6695g_penalty_cents, 127_000);
    }

    #[test]
    fn aotc_failure_only_engages_g() {
        let mut b = input();
        b.due_diligence_aotc_failures = 1;
        let r = compute(&b);
        assert_eq!(r.section_6695g_penalty_cents, 63_500);
    }

    #[test]
    fn hoh_failure_only_engages_g() {
        let mut b = input();
        b.due_diligence_hoh_failures = 1;
        let r = compute(&b);
        assert_eq!(r.section_6695g_penalty_cents, 63_500);
    }

    // ── Combined penalty calculation ──────────────────────────

    #[test]
    fn combined_penalty_sums_all_subsections() {
        let mut b = input();
        b.returns_with_a_failures = 1; // $60
        b.returns_with_b_failures = 2; // $120
        b.refund_checks_negotiated = 1; // $635
        b.due_diligence_eitc_failures = 1; // $635
        let r = compute(&b);
        // $60 + $120 + $635 + $635 = $1,450.
        assert_eq!(r.total_penalty_cents, 6_000 + 12_000 + 63_500 + 63_500);
    }

    // ── Parameterization for non-2025 years ───────────────────

    #[test]
    fn parameterized_2024_amounts() {
        // 2024 § 6695(a)-(e) penalty was $60 each, max $30,000.
        let mut b = input();
        b.returns_with_a_failures = 1;
        b.per_failure_penalty_cents = 6_000;
        b.annual_max_cap_cents = 3_000_000; // $30,000 2024 cap
        let r = compute(&b);
        assert_eq!(r.section_6695a_penalty_cents, 6_000);
    }

    #[test]
    fn parameterized_2024_higher_tier() {
        // 2024 § 6695(f)/(g) was $600 per failure.
        let mut b = input();
        b.due_diligence_eitc_failures = 1;
        b.higher_tier_penalty_cents = 60_000; // $600
        let r = compute(&b);
        assert_eq!(r.section_6695g_penalty_cents, 60_000);
    }

    // ── Multi-regime invariants ───────────────────────────────

    #[test]
    fn higher_tier_exceeds_per_failure_invariant() {
        // $635 > $60 — higher-tier penalties are roughly 10× the
        // per-failure rate.
        assert!(HIGHER_TIER_PENALTY_2025_CENTS > PER_FAILURE_PENALTY_2025_CENTS);
        assert_eq!(
            HIGHER_TIER_PENALTY_2025_CENTS / PER_FAILURE_PENALTY_2025_CENTS,
            10
        );
    }

    #[test]
    fn section_6695g_max_equals_4x_higher_tier_invariant() {
        // 4 credit/status categories × $635 = $2,540.
        assert_eq!(
            SECTION_6695G_MAX_PER_RETURN_CENTS,
            4 * HIGHER_TIER_PENALTY_2025_CENTS
        );
    }

    #[test]
    fn five_per_failure_subsections_truth_table() {
        // Each of (a)/(b)/(c)/(d)/(e) — single failure → $60.
        for subsection in 0..5 {
            let mut b = input();
            match subsection {
                0 => b.returns_with_a_failures = 1,
                1 => b.returns_with_b_failures = 1,
                2 => b.returns_with_c_failures = 1,
                3 => b.returns_with_d_failures = 1,
                _ => b.returns_with_e_failures = 1,
            }
            let r = compute(&b);
            assert_eq!(r.total_penalty_cents, 6_000, "subsection {}", subsection);
        }
    }

    #[test]
    fn constants_2025_invariant() {
        assert_eq!(PER_FAILURE_PENALTY_2025_CENTS, 6_000); // $60
        assert_eq!(ANNUAL_MAX_CAP_2025_CENTS, 3_150_000); // $31,500
        assert_eq!(HIGHER_TIER_PENALTY_2025_CENTS, 63_500); // $635
        assert_eq!(SECTION_6695G_MAX_PER_RETURN_CENTS, 254_000); // $2,540
    }

    #[test]
    fn citation_pins_all_subsections() {
        let r = compute(&input());
        assert!(r.citation.contains("§ 6695"));
        assert!(r.citation.contains("§ 6695(a)"));
        assert!(r.citation.contains("§ 6695(b)"));
        assert!(r.citation.contains("§ 6695(c)"));
        assert!(r.citation.contains("§ 6695(d)"));
        assert!(r.citation.contains("§ 6695(e)"));
        assert!(r.citation.contains("§ 6695(f)"));
        assert!(r.citation.contains("§ 6695(g)"));
        assert!(r.citation.contains("§ 1.6695-1"));
        assert!(r.citation.contains("§ 1.6695-2"));
        assert!(r.citation.contains("Form 8867"));
        assert!(r.citation.contains("Rev. Proc. 2024-40"));
    }

    #[test]
    fn sibling_cluster_note_present() {
        let mut b = input();
        b.returns_with_a_failures = 1;
        let r = compute(&b);
        assert!(
            r.notes.iter().any(|n| n.contains("§ 6694")
                && n.contains("§ 6700")
                && n.contains("§ 6701")
                && n.contains("§ 6662")
                && n.contains("§ 6662A")
                && n.contains("§ 6707A")
                && n.contains("Form 8867")),
            "sibling cluster note must reference preparer + promoter + taxpayer-side companions"
        );
    }

    // ── Defensive input clamping ──────────────────────────────

    #[test]
    fn defensive_negative_counts_clamped() {
        let mut b = input();
        b.returns_with_a_failures = -10;
        b.refund_checks_negotiated = -5;
        b.due_diligence_eitc_failures = -3;
        let r = compute(&b);
        assert_eq!(r.total_penalty_cents, 0);
    }

    #[test]
    fn defensive_negative_per_failure_clamped() {
        let mut b = input();
        b.returns_with_a_failures = 5;
        b.per_failure_penalty_cents = -1_000;
        let r = compute(&b);
        // Per-failure clamps to 0 → no penalty.
        assert_eq!(r.section_6695a_penalty_cents, 0);
    }

    #[test]
    fn extreme_count_no_overflow() {
        let mut b = input();
        b.refund_checks_negotiated = 100_000;
        let r = compute(&b);
        // 100K × $635 = $63.5M.
        assert_eq!(r.section_6695f_penalty_cents, 6_350_000_000);
    }
}
