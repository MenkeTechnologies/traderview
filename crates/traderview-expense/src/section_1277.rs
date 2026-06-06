//! IRC § 1277 — Deferral of interest deduction allocable to accrued
//! market discount.
//!
//! Direct companion to `section_1276` (which converts disposition
//! gain to ordinary income up to accrued market discount). § 1277
//! closes the loophole that would otherwise let a bond trader
//! deduct margin-loan interest CURRENTLY while DEFERRING the
//! offsetting market-discount income recognition to disposition.
//!
//! Three operative subsections:
//!
//!   § 1277(a) — GENERAL RULE: Net direct interest expense (NDIE)
//!     with respect to any market discount bond is deductible in the
//!     current taxable year only to the extent that NDIE exceeds
//!     the portion of market discount allocable to the days during
//!     the taxable year on which the taxpayer held the bond.
//!     Translation: deductible = max(0, NDIE − current-year accrued
//!     discount); deferred = min(NDIE, current-year accrued
//!     discount).
//!
//!   § 1277(b)(1) — NET-INTEREST-INCOME CARRYOVER (early
//!     recovery): Any disallowed interest expense carried over from
//!     prior years may be treated as paid or accrued in a later
//!     taxable year to the extent of NET INTEREST INCOME on the
//!     bond for that later year. Net interest income = interest
//!     income on the bond minus interest expense on the carrying
//!     debt for that year.
//!
//!   § 1277(b)(2) — DISPOSITION CARRYOVER (terminal recovery): Any
//!     amount remaining as disallowed deferred interest after
//!     § 1277(b)(1) is treated as paid or accrued in the taxable
//!     year in which the bond is disposed of (sale, exchange,
//!     retirement, gift, etc.). All previously deferred amounts
//!     are recovered at disposition.
//!
//!   § 1277(c) — DEFINITION OF NET DIRECT INTEREST EXPENSE: the
//!     EXCESS (if any) of (1) interest paid or accrued during the
//!     taxable year on indebtedness incurred or continued to
//!     purchase or carry the bond, OVER (2) the aggregate amount of
//!     interest (including OID under § 1272(a)) includible in
//!     gross income for the taxable year with respect to the bond.
//!
//! § 1277(d) was struck out entirely by Pub. L. 103-66 (1993). No
//! current provision exists at that subsection.
//!
//! § 1278(b) — CURRENT-INCLUSION ELECTION coordination: A taxpayer
//! who elects current inclusion of accrued market discount under
//! § 1278(b) is NOT subject to § 1277 deferral. The trade-off:
//! current ordinary-income recognition of accrued discount each
//! year vs. current deduction of carrying interest each year. Both
//! halves of the income/expense match.
//!
//! Citations: 26 U.S.C. § 1277(a) (general deferral rule);
//! § 1277(b)(1) (net-interest-income carryover); § 1277(b)(2)
//! (disposition carryover); § 1277(c)(1) (interest on indebtedness);
//! § 1277(c)(2) (offset for interest includible in gross income);
//! § 1278(b) (current-inclusion election exempts from § 1277);
//! § 1276 (companion market-discount ordinary-income conversion).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Section1277Input {
    /// § 1277(c)(1) — interest paid or accrued during the taxable
    /// year on indebtedness incurred or continued to purchase or
    /// carry the market discount bond (cents).
    pub interest_on_indebtedness_cents: i64,
    /// § 1277(c)(2) — aggregate interest (including OID under
    /// § 1272(a)) includible in gross income for the taxable year
    /// with respect to the bond (cents).
    pub interest_income_on_bond_cents: i64,
    /// § 1277(a) — portion of market discount allocable to the
    /// days during the taxable year on which the taxpayer held the
    /// bond (cents). Caller computes under § 1276(b)(1) ratable
    /// accrual or § 1276(b)(2) constant-yield election.
    pub accrued_market_discount_for_year_cents: i64,
    /// § 1278(b) current-inclusion election. If true, § 1277
    /// deferral does NOT apply — entire NDIE deductible currently
    /// (because the matching market-discount income is also
    /// recognized currently).
    pub current_inclusion_election: bool,
    /// True if the bond is disposed of in this taxable year.
    /// Triggers § 1277(b)(2) terminal recovery of all remaining
    /// deferred interest.
    pub disposition_year: bool,
    /// § 1277(b)(1) — net interest income for the year on this bond
    /// (cents). Caller computes; typically equals
    /// `interest_income_on_bond − interest_on_indebtedness` clamped
    /// at zero, but the statute treats it as a separate test.
    pub net_interest_income_for_year_cents: i64,
    /// Carryover of disallowed interest from all prior years
    /// (cents). Added to the current-year deferral pool before
    /// applying § 1277(b)(1) and § 1277(b)(2) recoveries.
    pub prior_year_disallowed_carryover_cents: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section1277Result {
    /// § 1277(c) net direct interest expense (cents). Clamped at
    /// zero where interest income exceeds interest paid.
    pub net_direct_interest_expense_cents: i64,
    /// Amount of current-year NDIE deferred under § 1277(a) (cents).
    /// Equals min(NDIE, accrued market discount for year). Zero
    /// where § 1278(b) current-inclusion election is in effect.
    pub current_year_deferred_cents: i64,
    /// Amount of current-year NDIE deductible after § 1277(a)
    /// deferral. Plus any § 1277(b)(1) or § 1277(b)(2) carryover
    /// recovery.
    pub current_year_deductible_cents: i64,
    /// Carryover recovered under § 1277(b)(1) net-interest-income
    /// test (cents).
    pub carryover_recovered_under_1277b1_cents: i64,
    /// Carryover recovered under § 1277(b)(2) disposition terminal
    /// rule (cents). Non-zero only in disposition years.
    pub carryover_recovered_under_1277b2_cents: i64,
    /// Net carryover surviving to future years (cents). Zero in
    /// disposition years (terminal recovery clears the pool) or
    /// when § 1278(b) election makes deferral inapplicable.
    pub remaining_carryover_to_future_years_cents: i64,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn compute(input: &Section1277Input) -> Section1277Result {
    let mut notes: Vec<String> = Vec::new();

    // § 1277(c) — net direct interest expense.
    let ndie = input
        .interest_on_indebtedness_cents
        .saturating_sub(input.interest_income_on_bond_cents)
        .max(0);

    // § 1278(b) current-inclusion election — deferral does not
    // apply. Entire NDIE deductible currently; any prior carryover
    // also recovered (since the taxpayer is on a current-recognition
    // basis for the entire bond stream).
    if input.current_inclusion_election {
        let recovered = input.prior_year_disallowed_carryover_cents.max(0);
        notes.push(
            "§ 1278(b) current-inclusion election applies — § 1277 deferral does NOT engage; \
             NDIE deductible currently, and any prior-year disallowed carryover is fully \
             recovered (because the matching market-discount income is recognized currently)."
                .to_string(),
        );
        return Section1277Result {
            net_direct_interest_expense_cents: ndie,
            current_year_deferred_cents: 0,
            current_year_deductible_cents: ndie.saturating_add(recovered),
            carryover_recovered_under_1277b1_cents: 0,
            carryover_recovered_under_1277b2_cents: recovered,
            remaining_carryover_to_future_years_cents: 0,
            citation: "26 U.S.C. § 1278(b) (current-inclusion election exempts taxpayer from \
                       § 1277 deferral); § 1277(c) (net direct interest expense definition)",
            notes,
        };
    }

    // § 1277(a) — current-year NDIE deferred up to accrued market
    // discount allocable to the year. Remainder is deductible
    // currently.
    let accrued = input.accrued_market_discount_for_year_cents.max(0);
    let current_year_deferred = ndie.min(accrued);
    let current_year_post_1277a_deductible = ndie.saturating_sub(current_year_deferred);

    let prior_carryover = input.prior_year_disallowed_carryover_cents.max(0);

    // § 1277(b)(1) — early recovery of prior carryover to extent
    // of net interest income on the bond for the year.
    let net_interest_income = input.net_interest_income_for_year_cents.max(0);
    let recovered_1277b1 = prior_carryover.min(net_interest_income);

    // § 1277(b)(2) — terminal recovery at disposition.
    let after_1277b1_pool = prior_carryover
        .saturating_sub(recovered_1277b1)
        .saturating_add(current_year_deferred);
    let recovered_1277b2 = if input.disposition_year {
        after_1277b1_pool
    } else {
        0
    };

    let remaining_carryover = if input.disposition_year {
        0
    } else {
        after_1277b1_pool
    };

    let current_year_deductible = current_year_post_1277a_deductible
        .saturating_add(recovered_1277b1)
        .saturating_add(recovered_1277b2);

    if input.disposition_year && (recovered_1277b1 + recovered_1277b2) > 0 {
        notes.push(
            "§ 1277(b)(2) — disposition year: all previously deferred / carried interest \
             expense is recovered (treated as paid or accrued in the disposition year)."
                .to_string(),
        );
    } else if recovered_1277b1 > 0 {
        notes.push(
            "§ 1277(b)(1) — net-interest-income carryover recovery: prior-year disallowed \
             interest is treated as paid or accrued this year to the extent of net interest \
             income on the bond."
                .to_string(),
        );
    }

    if current_year_deferred > 0 && !input.disposition_year {
        notes.push(format!(
            "§ 1277(a) — current-year NDIE of {} cents deferred to extent of {} cents accrued \
             market discount allocable to the days held; remainder of {} cents deductible \
             currently. Deferred amount carries forward to future years.",
            ndie, current_year_deferred, current_year_post_1277a_deductible,
        ));
    }

    Section1277Result {
        net_direct_interest_expense_cents: ndie,
        current_year_deferred_cents: current_year_deferred,
        current_year_deductible_cents: current_year_deductible,
        carryover_recovered_under_1277b1_cents: recovered_1277b1,
        carryover_recovered_under_1277b2_cents: recovered_1277b2,
        remaining_carryover_to_future_years_cents: remaining_carryover,
        citation: "26 U.S.C. § 1277(a) (general deferral rule — NDIE deductible only to extent \
                   exceeding accrued market discount); § 1277(b)(1) (net-interest-income \
                   carryover recovery); § 1277(b)(2) (terminal disposition recovery); \
                   § 1277(c) (net direct interest expense definition); § 1278(b) \
                   (current-inclusion election coordination); § 1276 (companion market-\
                   discount ordinary-income conversion)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        interest_debt: i64,
        interest_income: i64,
        accrued: i64,
        current_incl: bool,
        disposition: bool,
        net_interest: i64,
        prior_carryover: i64,
    ) -> Section1277Input {
        Section1277Input {
            interest_on_indebtedness_cents: interest_debt,
            interest_income_on_bond_cents: interest_income,
            accrued_market_discount_for_year_cents: accrued,
            current_inclusion_election: current_incl,
            disposition_year: disposition,
            net_interest_income_for_year_cents: net_interest,
            prior_year_disallowed_carryover_cents: prior_carryover,
        }
    }

    // ── § 1277(c) NDIE computation ──────────────────────────────

    #[test]
    fn ndie_basic_excess_of_interest_over_bond_income() {
        // $10,000 margin interest; $3,000 bond interest. NDIE = $7,000.
        let r = compute(&input(10_000, 3_000, 5_000, false, false, 0, 0));
        assert_eq!(r.net_direct_interest_expense_cents, 7_000);
    }

    #[test]
    fn ndie_clamps_at_zero_when_interest_income_exceeds_interest_paid() {
        // Bond income > margin interest. NDIE = 0.
        let r = compute(&input(2_000, 5_000, 1_000, false, false, 0, 0));
        assert_eq!(r.net_direct_interest_expense_cents, 0);
        assert_eq!(r.current_year_deferred_cents, 0);
        assert_eq!(r.current_year_deductible_cents, 0);
    }

    // ── § 1277(a) general deferral rule ─────────────────────────

    #[test]
    fn deferred_equals_min_of_ndie_and_accrued_discount() {
        // NDIE $7,000; accrued $5,000. Deferred = $5,000; deductible
        // currently = $2,000.
        let r = compute(&input(10_000, 3_000, 5_000, false, false, 0, 0));
        assert_eq!(r.current_year_deferred_cents, 5_000);
        assert_eq!(r.current_year_deductible_cents, 2_000);
    }

    #[test]
    fn ndie_below_accrued_full_ndie_deferred() {
        // NDIE $3,000; accrued $5,000. Deferred = $3,000; deductible
        // = 0.
        let r = compute(&input(6_000, 3_000, 5_000, false, false, 0, 0));
        assert_eq!(r.net_direct_interest_expense_cents, 3_000);
        assert_eq!(r.current_year_deferred_cents, 3_000);
        assert_eq!(r.current_year_deductible_cents, 0);
    }

    #[test]
    fn ndie_above_accrued_excess_deductible_currently() {
        // NDIE $15,000; accrued $5,000. Deferred = $5,000; current
        // deductible = $10,000.
        let r = compute(&input(15_000, 0, 5_000, false, false, 0, 0));
        assert_eq!(r.current_year_deferred_cents, 5_000);
        assert_eq!(r.current_year_deductible_cents, 10_000);
    }

    #[test]
    fn zero_accrued_discount_full_ndie_deductible_currently() {
        let r = compute(&input(10_000, 0, 0, false, false, 0, 0));
        assert_eq!(r.current_year_deferred_cents, 0);
        assert_eq!(r.current_year_deductible_cents, 10_000);
    }

    #[test]
    fn negative_accrued_clamps_at_zero() {
        // Defensive — negative input should not produce a deferral.
        let r = compute(&input(10_000, 0, -5_000, false, false, 0, 0));
        assert_eq!(r.current_year_deferred_cents, 0);
        assert_eq!(r.current_year_deductible_cents, 10_000);
    }

    // ── § 1277(b)(1) net-interest-income recovery ───────────────

    #[test]
    fn prior_carryover_recovered_to_net_interest_income() {
        // Prior carryover $4,000; net interest income $2,500.
        // Recovered = $2,500. Remaining = $1,500.
        let r = compute(&input(0, 0, 0, false, false, 2_500, 4_000));
        assert_eq!(r.carryover_recovered_under_1277b1_cents, 2_500);
        assert_eq!(r.current_year_deductible_cents, 2_500);
        assert_eq!(r.remaining_carryover_to_future_years_cents, 1_500);
    }

    #[test]
    fn net_interest_income_exceeds_prior_recovers_full_carryover() {
        // Prior $4,000; net interest income $10,000. Recovered =
        // $4,000 (capped at carryover).
        let r = compute(&input(0, 10_000, 0, false, false, 10_000, 4_000));
        assert_eq!(r.carryover_recovered_under_1277b1_cents, 4_000);
        assert_eq!(r.remaining_carryover_to_future_years_cents, 0);
    }

    #[test]
    fn zero_net_interest_income_no_recovery() {
        let r = compute(&input(0, 0, 0, false, false, 0, 4_000));
        assert_eq!(r.carryover_recovered_under_1277b1_cents, 0);
        assert_eq!(r.remaining_carryover_to_future_years_cents, 4_000);
    }

    // ── § 1277(b)(2) disposition-year terminal recovery ─────────

    #[test]
    fn disposition_year_recovers_all_carryover() {
        // Disposition year. Prior $4,000 + new defer $3,000 = pool
        // $7,000. b1 recovery = 0 (no net interest income). b2
        // recovery = $7,000.
        let r = compute(&input(5_000, 0, 3_000, false, true, 0, 4_000));
        assert_eq!(r.current_year_deferred_cents, 3_000);
        assert_eq!(r.carryover_recovered_under_1277b2_cents, 7_000);
        assert_eq!(r.remaining_carryover_to_future_years_cents, 0);
    }

    #[test]
    fn disposition_year_combines_b1_and_b2_recovery() {
        // Prior $4,000; net interest income $1,000 (b1 recovers
        // $1,000); new defer $2,000. b2 pool = (4000-1000) + 2000
        // = $5,000.
        let r = compute(&input(3_000, 0, 2_000, false, true, 1_000, 4_000));
        assert_eq!(r.carryover_recovered_under_1277b1_cents, 1_000);
        assert_eq!(r.carryover_recovered_under_1277b2_cents, 5_000);
        assert_eq!(r.remaining_carryover_to_future_years_cents, 0);
    }

    #[test]
    fn disposition_year_no_carryover_no_recovery() {
        // No prior carryover; non-disposition NDIE deferred would
        // be 0 with no accrual; nothing to recover.
        let r = compute(&input(5_000, 0, 0, false, true, 0, 0));
        assert_eq!(r.carryover_recovered_under_1277b2_cents, 0);
        assert_eq!(r.remaining_carryover_to_future_years_cents, 0);
    }

    // ── § 1278(b) current-inclusion election ────────────────────

    #[test]
    fn current_inclusion_election_no_deferral_full_deduction() {
        // NDIE $7,000; election → fully deductible.
        let r = compute(&input(10_000, 3_000, 5_000, true, false, 0, 0));
        assert_eq!(r.current_year_deferred_cents, 0);
        assert_eq!(r.current_year_deductible_cents, 7_000);
        assert!(r.citation.contains("§ 1278(b)"));
    }

    #[test]
    fn current_inclusion_recovers_prior_carryover() {
        // Election fully recovers prior-year disallowed carryover.
        let r = compute(&input(10_000, 3_000, 5_000, true, false, 0, 2_500));
        assert_eq!(r.current_year_deductible_cents, 9_500);
        assert_eq!(r.carryover_recovered_under_1277b2_cents, 2_500);
        assert_eq!(r.remaining_carryover_to_future_years_cents, 0);
    }

    #[test]
    fn current_inclusion_zero_ndie_zero_deduction_but_recovers_carryover() {
        let r = compute(&input(2_000, 5_000, 5_000, true, false, 0, 1_000));
        assert_eq!(r.net_direct_interest_expense_cents, 0);
        assert_eq!(r.current_year_deductible_cents, 1_000);
        assert_eq!(r.remaining_carryover_to_future_years_cents, 0);
    }

    // ── Multi-year flow simulation ──────────────────────────────

    #[test]
    fn multi_year_carryover_then_disposition_terminal_recovery() {
        // Year 1: NDIE $5,000; accrued $5,000 → defer $5,000; current
        // deductible 0; remaining carryover $5,000.
        let y1 = compute(&input(7_000, 2_000, 5_000, false, false, 0, 0));
        assert_eq!(y1.current_year_deferred_cents, 5_000);
        assert_eq!(y1.current_year_deductible_cents, 0);
        assert_eq!(y1.remaining_carryover_to_future_years_cents, 5_000);

        // Year 2 (same numbers; non-disposition):
        // NDIE $5,000; accrued $5,000 → defer $5,000; b1 recovery $0;
        // current deductible 0; remaining $5,000 + $5,000 = $10,000.
        let y2 = compute(&input(
            7_000,
            2_000,
            5_000,
            false,
            false,
            0,
            y1.remaining_carryover_to_future_years_cents,
        ));
        assert_eq!(y2.remaining_carryover_to_future_years_cents, 10_000);

        // Year 3 (disposition): No new accrual, no new NDIE; recovers
        // all $10,000.
        let y3 = compute(&input(
            0,
            0,
            0,
            false,
            true,
            0,
            y2.remaining_carryover_to_future_years_cents,
        ));
        assert_eq!(y3.carryover_recovered_under_1277b2_cents, 10_000);
        assert_eq!(y3.current_year_deductible_cents, 10_000);
        assert_eq!(y3.remaining_carryover_to_future_years_cents, 0);
    }

    // ── Regression-critical invariants ──────────────────────────

    #[test]
    fn deferred_never_exceeds_ndie_invariant() {
        for accrued in [0_i64, 1_000, 5_000, 10_000, 50_000] {
            let r = compute(&input(10_000, 0, accrued, false, false, 0, 0));
            assert!(
                r.current_year_deferred_cents <= r.net_direct_interest_expense_cents,
                "deferred {} > NDIE {} at accrued {}",
                r.current_year_deferred_cents,
                r.net_direct_interest_expense_cents,
                accrued,
            );
        }
    }

    #[test]
    fn deferred_never_exceeds_accrued_invariant() {
        for ndie_paid in [0_i64, 1_000, 5_000, 10_000, 50_000] {
            let r = compute(&input(ndie_paid, 0, 5_000, false, false, 0, 0));
            assert!(
                r.current_year_deferred_cents <= 5_000,
                "deferred {} > accrued 5000 at NDIE {}",
                r.current_year_deferred_cents,
                ndie_paid,
            );
        }
    }

    #[test]
    fn current_year_deferred_plus_post_1277a_deductible_equals_ndie_invariant() {
        // The § 1277(a) split: post-1277a deductible + deferred =
        // NDIE. (Carryover recoveries are added on top.)
        for accrued in [0_i64, 1_000, 5_000, 10_000, 50_000] {
            let r = compute(&input(10_000, 0, accrued, false, false, 0, 0));
            let post_1277a = r
                .current_year_deductible_cents
                .saturating_sub(r.carryover_recovered_under_1277b1_cents)
                .saturating_sub(r.carryover_recovered_under_1277b2_cents);
            assert_eq!(
                post_1277a + r.current_year_deferred_cents,
                r.net_direct_interest_expense_cents,
                "split conservation must hold at accrued {}",
                accrued,
            );
        }
    }

    #[test]
    fn disposition_year_zeros_remaining_carryover_invariant() {
        for prior in [0_i64, 1_000, 5_000, 10_000] {
            for ndie_paid in [0_i64, 5_000, 15_000] {
                for accrued in [0_i64, 5_000] {
                    let r = compute(&input(ndie_paid, 0, accrued, false, true, 0, prior));
                    assert_eq!(
                        r.remaining_carryover_to_future_years_cents, 0,
                        "disposition year must zero carryover (prior={prior} NDIE={ndie_paid} \
                         accrued={accrued})",
                    );
                }
            }
        }
    }

    #[test]
    fn citation_pins_all_subsections_in_default_path() {
        let r = compute(&input(10_000, 0, 5_000, false, false, 0, 0));
        assert!(r.citation.contains("§ 1277(a)"));
        assert!(r.citation.contains("§ 1277(b)(1)"));
        assert!(r.citation.contains("§ 1277(b)(2)"));
        assert!(r.citation.contains("§ 1277(c)"));
        assert!(r.citation.contains("§ 1278(b)"));
        assert!(r.citation.contains("§ 1276"));
    }

    #[test]
    fn citation_pins_section_1278b_path_when_election_in_effect() {
        let r = compute(&input(10_000, 0, 5_000, true, false, 0, 0));
        assert!(r.citation.contains("§ 1278(b)"));
        assert!(r.citation.contains("§ 1277(c)"));
    }

    #[test]
    fn note_documents_section_1277a_deferral_when_present() {
        let r = compute(&input(10_000, 0, 5_000, false, false, 0, 0));
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("§ 1277(a)") && n.contains("deferred")),
            "§ 1277(a) deferral note must be present"
        );
    }

    #[test]
    fn note_documents_disposition_terminal_recovery_when_present() {
        let r = compute(&input(0, 0, 0, false, true, 0, 5_000));
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("§ 1277(b)(2)") && n.contains("disposition")),
            "§ 1277(b)(2) disposition recovery note must be present"
        );
    }

    #[test]
    fn current_inclusion_note_present_when_election_in_effect() {
        let r = compute(&input(10_000, 0, 5_000, true, false, 0, 1_000));
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("§ 1278(b)") && n.contains("current-inclusion")),
            "§ 1278(b) current-inclusion note must be present"
        );
    }
}
