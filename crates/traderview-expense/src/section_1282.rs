//! IRC § 1282 — Deferral of interest deduction allocable to
//! accrued acquisition discount on short-term obligations.
//!
//! Direct short-term-obligation companion to `section_1277` (long-
//! term market-discount interest deferral). Where § 1281 mandates
//! current inclusion of acquisition discount for specified holders,
//! § 1282 closes the parallel loophole: for holders NOT subject to
//! § 1281 current inclusion, § 1282 DEFERS the current-year
//! deduction of net direct interest expense on indebtedness
//! incurred to purchase or carry the short-term obligation —
//! preventing deduction of interest while deferring the offsetting
//! discount accrual.
//!
//! Operative subsections:
//!
//!   § 1282(a) — GENERAL RULE: Net direct interest expense (NDIE)
//!     on indebtedness incurred or continued to purchase or carry
//!     a short-term obligation is deductible in the current year
//!     ONLY to the extent it exceeds the daily portions of
//!     acquisition discount allocable to days held in the taxable
//!     year. Disallowed portion defers until the obligation is
//!     disposed of (parallels § 1277 for long-term obligations).
//!
//!   § 1282(b) — EXCEPTION FOR § 1281 HOLDERS: § 1282 does NOT
//!     apply where § 1281 (current inclusion of acquisition
//!     discount) is in effect — accrual-method taxpayers, dealers,
//!     banks, RICs, hedging-transaction holders, stripped-bond
//!     strippers, and pass-thru entities are exempt from § 1282
//!     deferral because they are already including the discount
//!     currently. § 1282(b)(2) provides an ELECTION to apply
//!     § 1281 to all short-term obligations held (which then
//!     triggers the § 1282(b) exception).
//!
//!   § 1282(c) — CROSS-REFERENCE: § 1277 (long-term market-discount
//!     interest deferral) rules apply by reference for
//!     computational parallels.
//!
//!   § 1282(d) — NONGOVERNMENTAL SHORT-TERM: § 1283(c) substitution
//!     of OID for acquisition discount applies for nongovernmental
//!     short-term obligations.
//!
//! Citations: 26 U.S.C. § 1282(a) (general deferral rule);
//! § 1282(b)(1) (exception for § 1281 holders); § 1282(b)(2)
//! (election to apply § 1281 to all short-term obligations —
//! triggers § 1282(b) exception); § 1282(c) (cross-reference to
//! § 1277 long-term rules); § 1282(d) (nongovernmental OID via
//! § 1283(c)); § 1281 (current inclusion mandate); § 1283
//! (definitions); § 1277 (long-term market-discount interest
//! deferral parallel).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HolderStatus {
    /// Holder subject to § 1281 current-inclusion mandate (accrual,
    /// dealer, bank, RIC, hedging, stripper, pass-thru). § 1282(b)
    /// exception applies — no interest deferral.
    Section1281Holder,
    /// Holder NOT subject to § 1281 current inclusion (cash-method
    /// individual). § 1282(a) deferral applies.
    OtherHolder,
    /// Holder has made § 1282(b)(2) election to apply § 1281 to
    /// all short-term obligations held — triggers § 1282(b)
    /// exception.
    ElectionToApplyToAllStOs,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section1282Input {
    pub holder_status: HolderStatus,
    /// Interest expense on indebtedness incurred or continued to
    /// purchase or carry the short-term obligation (cents).
    pub interest_expense_on_indebtedness_cents: i64,
    /// Interest income (including OID) includible in gross income
    /// for the taxable year with respect to the short-term
    /// obligation (cents). Subtracted from interest expense to
    /// compute NDIE.
    pub interest_income_includible_cents: i64,
    /// Daily portion of acquisition discount under § 1283(b)(1)
    /// (cents per day).
    pub daily_portion_acquisition_discount_cents: i64,
    /// Days the holder held the short-term obligation during the
    /// taxable year.
    pub days_held_in_year: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section1282Result {
    /// Net direct interest expense (cents) = max(0, interest paid −
    /// interest income).
    pub net_direct_interest_expense_cents: i64,
    /// Accumulated daily portions of acquisition discount allocable
    /// to days held in the taxable year (cents).
    pub accumulated_discount_for_year_cents: i64,
    /// Interest deferred under § 1282(a) (cents) = min(NDIE,
    /// accumulated discount). Zero if § 1282(b) exception applies.
    pub deferred_interest_cents: i64,
    /// Interest currently deductible after § 1282(a) deferral
    /// (cents) = NDIE − deferred. Equals full NDIE if § 1282(b)
    /// exception applies.
    pub currently_deductible_interest_cents: i64,
    /// True if § 1282(b) exception applies (§ 1281 holder OR
    /// § 1282(b)(2) election).
    pub section_1282b_exception_applies: bool,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn compute(input: &Section1282Input) -> Section1282Result {
    let mut notes: Vec<String> = Vec::new();

    // § 1282(b) exception: § 1281 holder OR § 1282(b)(2) election.
    let exception_applies = matches!(
        input.holder_status,
        HolderStatus::Section1281Holder | HolderStatus::ElectionToApplyToAllStOs
    );

    // Net direct interest expense = max(0, interest paid - interest
    // income).
    let ndie = input
        .interest_expense_on_indebtedness_cents
        .saturating_sub(input.interest_income_includible_cents)
        .max(0);

    // § 1282(a) deferral computation.
    let accumulated_discount = input
        .daily_portion_acquisition_discount_cents
        .max(0)
        .saturating_mul(input.days_held_in_year as i64);

    let (deferred, currently_deductible) = if exception_applies {
        // § 1282(b) — no deferral; full NDIE deductible.
        (0, ndie)
    } else {
        // § 1282(a) — defer to extent of accumulated discount.
        let defer = ndie.min(accumulated_discount);
        (defer, ndie.saturating_sub(defer))
    };

    // Notes.
    if exception_applies {
        notes.push(format!(
            "§ 1282(b) exception applies — {} is exempt from § 1282 deferral because § 1281 \
             current inclusion of acquisition discount is in effect.",
            match input.holder_status {
                HolderStatus::Section1281Holder => "holder subject to § 1281 current inclusion",
                HolderStatus::ElectionToApplyToAllStOs => {
                    "holder has made § 1282(b)(2) election to apply § 1281 to all short-term obligations"
                }
                _ => unreachable!(),
            },
        ));
    } else {
        notes.push(format!(
            "§ 1282(a) — net direct interest expense of {} cents deferred to extent of {} \
             cents accumulated daily portions of acquisition discount; deferred portion of {} \
             cents carries forward until obligation is disposed of (parallels § 1277).",
            ndie, accumulated_discount, deferred,
        ));
    }

    notes.push(
        "Companion to section_1281 (current inclusion of acquisition discount for accrual + \
         dealer + bank + RIC + hedging + stripper + pass-thru holders); section_1283 \
         (definitions); section_1277 (long-term market-discount interest deferral parallel)."
            .to_string(),
    );

    Section1282Result {
        net_direct_interest_expense_cents: ndie,
        accumulated_discount_for_year_cents: accumulated_discount,
        deferred_interest_cents: deferred,
        currently_deductible_interest_cents: currently_deductible,
        section_1282b_exception_applies: exception_applies,
        citation: "26 U.S.C. § 1282(a) (general deferral rule — NDIE deductible only to extent \
                   exceeding daily portions of acquisition discount); § 1282(b)(1) (exception \
                   for § 1281 holders); § 1282(b)(2) (election to apply § 1281 to all \
                   short-term obligations); § 1282(c) (§ 1277 cross-reference); § 1282(d) \
                   (§ 1283(c) nongovernmental OID); § 1281 (current inclusion mandate); \
                   § 1283 (definitions); § 1277 (long-term parallel)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        holder: HolderStatus,
        interest_paid: i64,
        interest_income: i64,
        daily_portion: i64,
        days_held: u32,
    ) -> Section1282Input {
        Section1282Input {
            holder_status: holder,
            interest_expense_on_indebtedness_cents: interest_paid,
            interest_income_includible_cents: interest_income,
            daily_portion_acquisition_discount_cents: daily_portion,
            days_held_in_year: days_held,
        }
    }

    // ── § 1282(a) general deferral ─────────────────────────────

    #[test]
    fn other_holder_ndie_above_accrual_deferred_to_accrual_cap() {
        // NDIE = 1000 - 200 = 800. Accrual = 5 × 100 = 500.
        // Deferred = min(800, 500) = 500. Currently deductible = 300.
        let r = compute(&input(
            HolderStatus::OtherHolder,
            1_000,
            200,
            5,
            100,
        ));
        assert_eq!(r.net_direct_interest_expense_cents, 800);
        assert_eq!(r.accumulated_discount_for_year_cents, 500);
        assert_eq!(r.deferred_interest_cents, 500);
        assert_eq!(r.currently_deductible_interest_cents, 300);
        assert!(!r.section_1282b_exception_applies);
        assert!(r.citation.contains("§ 1282(a)"));
    }

    #[test]
    fn other_holder_ndie_below_accrual_full_deferral() {
        // NDIE = 300. Accrual = 500. Deferred = min(300, 500) = 300.
        // Currently deductible = 0.
        let r = compute(&input(
            HolderStatus::OtherHolder,
            500,
            200,
            5,
            100,
        ));
        assert_eq!(r.net_direct_interest_expense_cents, 300);
        assert_eq!(r.deferred_interest_cents, 300);
        assert_eq!(r.currently_deductible_interest_cents, 0);
    }

    #[test]
    fn other_holder_zero_ndie_no_deferral() {
        // Interest income exceeds interest paid → NDIE = 0.
        let r = compute(&input(
            HolderStatus::OtherHolder,
            500,
            700,
            5,
            100,
        ));
        assert_eq!(r.net_direct_interest_expense_cents, 0);
        assert_eq!(r.deferred_interest_cents, 0);
        assert_eq!(r.currently_deductible_interest_cents, 0);
    }

    #[test]
    fn other_holder_zero_accrual_no_deferral() {
        // No daily portion → no deferral; full NDIE deductible.
        let r = compute(&input(
            HolderStatus::OtherHolder,
            1_000,
            200,
            0,
            100,
        ));
        assert_eq!(r.deferred_interest_cents, 0);
        assert_eq!(r.currently_deductible_interest_cents, 800);
    }

    // ── § 1282(b)(1) exception for § 1281 holders ──────────────

    #[test]
    fn section_1281_holder_exception_no_deferral() {
        let r = compute(&input(
            HolderStatus::Section1281Holder,
            1_000,
            200,
            5,
            100,
        ));
        assert!(r.section_1282b_exception_applies);
        assert_eq!(r.deferred_interest_cents, 0);
        assert_eq!(r.currently_deductible_interest_cents, 800);
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("§ 1282(b)") && n.contains("§ 1281 current inclusion"))
        );
    }

    // ── § 1282(b)(2) election to apply § 1281 to all STOs ──────

    #[test]
    fn section_1282b2_election_triggers_exception() {
        let r = compute(&input(
            HolderStatus::ElectionToApplyToAllStOs,
            1_000,
            200,
            5,
            100,
        ));
        assert!(r.section_1282b_exception_applies);
        assert_eq!(r.deferred_interest_cents, 0);
        assert_eq!(r.currently_deductible_interest_cents, 800);
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("§ 1282(b)(2) election")
                    && n.contains("apply § 1281"))
        );
    }

    // ── Conservation invariants ────────────────────────────────

    #[test]
    fn deferred_plus_currently_deductible_equals_ndie_invariant() {
        for daily_portion in [0_i64, 1, 5, 10, 50] {
            for days_held in [0_u32, 50, 100, 200] {
                let r = compute(&input(
                    HolderStatus::OtherHolder,
                    1_000,
                    200,
                    daily_portion,
                    days_held,
                ));
                assert_eq!(
                    r.deferred_interest_cents + r.currently_deductible_interest_cents,
                    r.net_direct_interest_expense_cents,
                    "daily={} days={}",
                    daily_portion,
                    days_held,
                );
            }
        }
    }

    #[test]
    fn deferred_never_exceeds_accrual_invariant() {
        for interest_paid in [100_i64, 500, 1_000, 5_000] {
            let r = compute(&input(
                HolderStatus::OtherHolder,
                interest_paid,
                0,
                10,
                100,
            ));
            assert!(
                r.deferred_interest_cents <= r.accumulated_discount_for_year_cents,
                "interest_paid={} deferred={} accrual={}",
                interest_paid,
                r.deferred_interest_cents,
                r.accumulated_discount_for_year_cents,
            );
        }
    }

    #[test]
    fn deferred_never_exceeds_ndie_invariant() {
        for daily_portion in [1_i64, 10, 100, 1_000] {
            let r = compute(&input(
                HolderStatus::OtherHolder,
                500,
                100,
                daily_portion,
                100,
            ));
            assert!(
                r.deferred_interest_cents <= r.net_direct_interest_expense_cents,
                "daily={} deferred={} ndie={}",
                daily_portion,
                r.deferred_interest_cents,
                r.net_direct_interest_expense_cents,
            );
        }
    }

    // ── Regression-critical invariants ──────────────────────────

    #[test]
    fn exception_applies_for_1281_holder_and_election_invariant() {
        assert!(
            check_exception(HolderStatus::Section1281Holder),
        );
        assert!(
            check_exception(HolderStatus::ElectionToApplyToAllStOs),
        );
        assert!(
            !check_exception(HolderStatus::OtherHolder),
        );
    }

    fn check_exception(holder: HolderStatus) -> bool {
        compute(&input(holder, 1_000, 200, 5, 100)).section_1282b_exception_applies
    }

    #[test]
    fn exception_holders_always_deduct_full_ndie_invariant() {
        for holder in [
            HolderStatus::Section1281Holder,
            HolderStatus::ElectionToApplyToAllStOs,
        ] {
            let r = compute(&input(holder, 1_000, 200, 5, 100));
            assert_eq!(r.currently_deductible_interest_cents, 800);
            assert_eq!(r.deferred_interest_cents, 0);
        }
    }

    #[test]
    fn other_holder_deferral_proportional_to_days_held_invariant() {
        // Same daily portion, increasing days held → increasing
        // deferral (up to NDIE cap).
        for days_held in [50_u32, 100, 200, 500] {
            let r = compute(&input(
                HolderStatus::OtherHolder,
                10_000,
                0,
                5,
                days_held,
            ));
            let expected_deferred = std::cmp::min(10_000_i64, 5_i64 * days_held as i64);
            assert_eq!(r.deferred_interest_cents, expected_deferred);
        }
    }

    #[test]
    fn citation_pins_all_subsections() {
        let r = compute(&input(
            HolderStatus::OtherHolder,
            1_000,
            200,
            5,
            100,
        ));
        assert!(r.citation.contains("§ 1282(a)"));
        assert!(r.citation.contains("§ 1282(b)(1)"));
        assert!(r.citation.contains("§ 1282(b)(2)"));
        assert!(r.citation.contains("§ 1282(c)"));
        assert!(r.citation.contains("§ 1282(d)"));
        assert!(r.citation.contains("§ 1281"));
        assert!(r.citation.contains("§ 1283"));
        assert!(r.citation.contains("§ 1277"));
    }

    #[test]
    fn sibling_module_note_present() {
        let r = compute(&input(
            HolderStatus::OtherHolder,
            1_000,
            200,
            5,
            100,
        ));
        assert!(
            r.notes.iter().any(|n| n.contains("section_1281")
                && n.contains("section_1283")
                && n.contains("section_1277")),
            "sibling-module note must be present"
        );
    }

    #[test]
    fn defensive_negative_inputs_clamp_at_zero() {
        // Negative daily portion shouldn't produce negative
        // accrual.
        let r = compute(&input(
            HolderStatus::OtherHolder,
            1_000,
            200,
            -50,
            100,
        ));
        assert_eq!(r.accumulated_discount_for_year_cents, 0);
        assert_eq!(r.deferred_interest_cents, 0);
        assert_eq!(r.currently_deductible_interest_cents, 800);
    }

    #[test]
    fn ndie_clamps_at_zero_when_interest_income_exceeds_paid() {
        let r = compute(&input(
            HolderStatus::OtherHolder,
            100,
            500,
            10,
            100,
        ));
        assert_eq!(r.net_direct_interest_expense_cents, 0);
    }

    #[test]
    fn full_year_holding_accumulates_365_day_portions() {
        let r = compute(&input(
            HolderStatus::OtherHolder,
            100_000,
            0,
            10,
            365,
        ));
        assert_eq!(r.accumulated_discount_for_year_cents, 3_650);
    }

    #[test]
    fn zero_days_held_zero_accrual_zero_deferral() {
        let r = compute(&input(
            HolderStatus::OtherHolder,
            1_000,
            200,
            5,
            0,
        ));
        assert_eq!(r.accumulated_discount_for_year_cents, 0);
        assert_eq!(r.deferred_interest_cents, 0);
        assert_eq!(r.currently_deductible_interest_cents, 800);
    }
}
