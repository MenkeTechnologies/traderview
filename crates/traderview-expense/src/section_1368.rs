//! IRC § 1368 — S corporation distributions.
//!
//! Direct sibling to § 1367 (basis adjustments, iter 246), § 1361
//! (definition + eligibility), § 1366 (pass-through items), and
//! § 1374 (built-in gains tax). § 1368 governs the SHAREHOLDER-
//! LEVEL tax treatment of cash and property distributions from
//! an S corporation. The result varies sharply depending on
//! whether the S-corp has accumulated earnings and profits (E&P)
//! from a prior C-corp period.
//!
//! § 1368(b) — NO ACCUMULATED E&P (S-corp never was C-corp, or
//! E&P fully distributed). Simple two-step ordering:
//!   (1) Tax-free reduction of stock basis (down to zero);
//!   (2) Excess treated as gain from sale of stock under
//!       § 1368(b)(2) — typically long-term capital gain.
//!
//! § 1368(c) — WITH ACCUMULATED E&P (S-corp converted from C-corp;
//! default ordering rule). Four-step ordering:
//!   § 1368(c)(1) — TAX-FREE up to AAA. Treated as if § 1368(b)
//!     applies; reduces stock basis under § 1367(a)(2)(A).
//!   § 1368(c)(2) — DIVIDEND up to accumulated E&P. Treated as
//!     ordinary dividend income under § 301 to shareholder; does
//!     NOT reduce stock basis.
//!   § 1368(c)(3) — TAX-FREE up to remaining stock basis (after
//!     step (c)(1) reduction). Reduces basis under § 1367(a)(2)(A).
//!   § 1368(b)(2) — Excess treated as capital gain on stock sale.
//!
//! § 1368(e)(1)(A) — ACCUMULATED ADJUSTMENTS ACCOUNT (AAA): An
//! account adjusted similar to § 1367 adjustments to stock basis
//! EXCEPT no adjustment for tax-exempt income and no adjustment
//! for federal taxes attributable to any taxable year in which
//! the corporation was a C corporation. AAA may go negative for
//! losses; distributions cannot reduce AAA below zero under
//! default ordering.
//!
//! § 1368(e)(1)(C) — NET NEGATIVE ADJUSTMENT RULE: If for any
//! taxable year the net negative adjustment exceeds the net
//! positive adjustment, the AAA for purposes of § 1368(c)(1)
//! shall be determined WITHOUT REGARD to such net negative
//! adjustment. Net negative is added back when determining AAA
//! available for distribution (current-year-cushion for
//! losses).
//!
//! § 1368(e)(3) — ELECTION to TREAT DISTRIBUTIONS AS FROM E&P
//! FIRST. With unanimous consent of all shareholders to whom a
//! distribution is made during the tax year, the S-corp may
//! elect to reverse steps (c)(1) and (c)(2) — distribute E&P
//! first as dividends, then AAA. Useful for purging E&P to
//! avoid the § 1375 passive investment income tax or future
//! § 1374 built-in gains tax exposure.
//!
//! Citations: 26 U.S.C. § 1368 (general distribution rules);
//! 26 U.S.C. § 1368(a) (treatment); 26 U.S.C. § 1368(b) (no-E&P
//! two-step); 26 U.S.C. § 1368(b)(1) (tax-free basis reduction);
//! 26 U.S.C. § 1368(b)(2) (excess capital gain); 26 U.S.C.
//! § 1368(c) (with-E&P four-step); 26 U.S.C. § 1368(c)(1) (AAA);
//! 26 U.S.C. § 1368(c)(2) (E&P dividend); 26 U.S.C. § 1368(c)(3)
//! (remaining basis); 26 U.S.C. § 1368(e)(1)(A) (AAA definition);
//! 26 U.S.C. § 1368(e)(1)(C) (net negative adjustment rule);
//! 26 U.S.C. § 1368(e)(3) (E&P-first election); Treas. Reg.
//! § 1.1368-1 (distribution treatment); Treas. Reg.
//! § 1.1368-2 (AAA mechanics). Sibling modules: § 1361
//! (definition); § 1366 (pass-through); § 1367 (basis
//! adjustments); § 1374 (built-in gains tax); § 1375 (passive
//! investment income tax). Form 1120-S Schedule M-2 tracks AAA.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Section1368Input {
    pub distribution_cents: i64,
    pub beginning_aaa_cents: i64,
    /// Accumulated earnings and profits from prior C-corp period.
    /// Always non-negative under E&P accounting rules.
    pub accumulated_e_and_p_cents: i64,
    pub beginning_stock_basis_cents: i64,
    /// Net positive adjustments (income items) for the year —
    /// § 1367(a)(1) items applicable to AAA.
    pub net_positive_adjustments_for_year_cents: i64,
    /// Net negative adjustments (losses + nondeductibles) for the
    /// year — § 1367(a)(2) items applicable to AAA (excluding
    /// distributions themselves).
    pub net_negative_adjustments_for_year_cents: i64,
    /// § 1368(e)(3) — taxpayer elects to treat distributions as
    /// from E&P first (requires unanimous shareholder consent).
    pub elects_e_and_p_first_under_1368e3: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section1368Result {
    /// § 1368(c)(1) tax-free portion from AAA — reduces stock
    /// basis under § 1367(a)(2)(A).
    pub tax_free_from_aaa_cents: i64,
    /// § 1368(c)(2) dividend portion from accumulated E&P —
    /// ordinary dividend income; does NOT reduce basis.
    pub dividend_from_e_and_p_cents: i64,
    /// § 1368(c)(3) tax-free portion from remaining stock basis
    /// — reduces basis.
    pub tax_free_from_basis_cents: i64,
    /// § 1368(b)(2) excess treated as capital gain on sale.
    pub capital_gain_cents: i64,
    pub ending_aaa_cents: i64,
    pub ending_e_and_p_cents: i64,
    pub ending_stock_basis_cents: i64,
    /// AAA available for § 1368(c)(1) distribution (after
    /// § 1368(e)(1)(C) net-negative-adjustment rule and $0 floor).
    pub aaa_available_for_distribution_cents: i64,
    /// True if § 1368(e)(3) election engaged.
    pub e_and_p_first_election_engaged: bool,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn compute(input: &Section1368Input) -> Section1368Result {
    let mut notes: Vec<String> = Vec::new();

    let distribution = input.distribution_cents.max(0);
    let beginning_aaa = input.beginning_aaa_cents;
    let accumulated_e_and_p = input.accumulated_e_and_p_cents.max(0);
    let beginning_basis = input.beginning_stock_basis_cents.max(0);
    let net_positive = input.net_positive_adjustments_for_year_cents.max(0);
    let net_negative = input.net_negative_adjustments_for_year_cents.max(0);

    // § 1368(e)(1)(C) — net negative adjustment rule: when net
    // negative exceeds net positive, ignore net negative for
    // distribution purposes.
    let aaa_distribution_base = if net_negative > net_positive {
        beginning_aaa.saturating_add(net_positive)
    } else {
        beginning_aaa
            .saturating_add(net_positive)
            .saturating_sub(net_negative)
    };
    let aaa_available_for_distribution = aaa_distribution_base.max(0);

    let has_e_and_p = accumulated_e_and_p > 0;

    let mut running_basis = beginning_basis;
    let mut remaining = distribution;
    let tax_free_from_aaa;
    let dividend;
    let tax_free_from_basis;

    let e_and_p_first_election_engaged = input.elects_e_and_p_first_under_1368e3 && has_e_and_p;

    if !has_e_and_p {
        // § 1368(b) — no E&P; basis reduction → capital gain.
        tax_free_from_aaa = 0;
        dividend = 0;
        tax_free_from_basis = remaining.min(running_basis);
        running_basis = running_basis.saturating_sub(tax_free_from_basis);
        remaining = remaining.saturating_sub(tax_free_from_basis);
    } else if e_and_p_first_election_engaged {
        // § 1368(e)(3) election — E&P first.
        dividend = remaining.min(accumulated_e_and_p);
        remaining = remaining.saturating_sub(dividend);

        tax_free_from_aaa = remaining.min(aaa_available_for_distribution);
        running_basis = running_basis.saturating_sub(tax_free_from_aaa);
        remaining = remaining.saturating_sub(tax_free_from_aaa);

        tax_free_from_basis = remaining.min(running_basis);
        running_basis = running_basis.saturating_sub(tax_free_from_basis);
        remaining = remaining.saturating_sub(tax_free_from_basis);
    } else {
        // § 1368(c) default — AAA → dividend → basis → capital gain.
        tax_free_from_aaa = remaining.min(aaa_available_for_distribution);
        running_basis = running_basis.saturating_sub(tax_free_from_aaa);
        remaining = remaining.saturating_sub(tax_free_from_aaa);

        dividend = remaining.min(accumulated_e_and_p);
        remaining = remaining.saturating_sub(dividend);

        tax_free_from_basis = remaining.min(running_basis);
        running_basis = running_basis.saturating_sub(tax_free_from_basis);
        remaining = remaining.saturating_sub(tax_free_from_basis);
    }

    let capital_gain = remaining;

    // End-of-year AAA reflects all adjustments + distribution from AAA.
    let ending_aaa = beginning_aaa
        .saturating_add(net_positive)
        .saturating_sub(net_negative)
        .saturating_sub(tax_free_from_aaa);

    let ending_e_and_p = accumulated_e_and_p.saturating_sub(dividend);
    let ending_stock_basis = running_basis;

    if !has_e_and_p {
        notes.push(format!(
            "§ 1368(b) (no E&P) — two-step: (1) tax-free reduction of basis ({} cents); \
             (2) excess capital gain ({} cents under § 1368(b)(2)). AAA tracking is \
             irrelevant when no accumulated E&P.",
            tax_free_from_basis, capital_gain,
        ));
    } else if e_and_p_first_election_engaged {
        notes.push(format!(
            "§ 1368(e)(3) ELECTION engaged — distributions treated as from E&P first. \
             Requires unanimous shareholder consent. Reorders to: E&P dividend ({} cents) \
             → AAA tax-free ({} cents) → basis tax-free ({} cents) → capital gain \
             ({} cents). Useful for purging E&P to avoid § 1375 passive-investment-income \
             tax or future § 1374 BIG tax exposure.",
            dividend, tax_free_from_aaa, tax_free_from_basis, capital_gain,
        ));
    } else {
        notes.push(format!(
            "§ 1368(c) default ordering — AAA tax-free ({} cents) → E&P dividend ({} \
             cents) → basis tax-free ({} cents) → capital gain ({} cents). AAA \
             available for distribution: {} cents (§ 1368(e)(1)(C) net-negative-\
             adjustment rule applied: {}).",
            tax_free_from_aaa,
            dividend,
            tax_free_from_basis,
            capital_gain,
            aaa_available_for_distribution,
            if net_negative > net_positive {
                "net negative ignored — added back"
            } else {
                "no net-negative adjustment to ignore"
            },
        ));
    }

    if capital_gain > 0 {
        notes.push(format!(
            "§ 1368(b)(2) — {} cents treated as gain from sale of stock (typically \
             long-term capital gain). Report on Schedule D + Form 8949.",
            capital_gain,
        ));
    }

    if dividend > 0 {
        notes.push(format!(
            "§ 1368(c)(2) — {} cents reportable as ORDINARY DIVIDEND income to \
             shareholder under § 301. No basis reduction; no AAA reduction (E&P only).",
            dividend,
        ));
    }

    notes.push(
        "Sibling S-corp cluster: § 1361 (definition + eligibility); § 1366 (pass-through \
         items + § 1366(d) basis-limited loss); § 1367 (basis adjustments — distributions \
         reduce basis under § 1367(a)(2)(A)); § 1374 (built-in gains tax on C-to-S \
         conversions); § 1375 (passive investment income tax — engages when S-corp has \
         E&P AND > 25% passive income). § 1368 governs SHAREHOLDER-level tax treatment; \
         § 1367 governs basis tracking. Form 1120-S Schedule M-2 tracks AAA + E&P year \
         over year."
            .to_string(),
    );

    Section1368Result {
        tax_free_from_aaa_cents: tax_free_from_aaa,
        dividend_from_e_and_p_cents: dividend,
        tax_free_from_basis_cents: tax_free_from_basis,
        capital_gain_cents: capital_gain,
        ending_aaa_cents: ending_aaa,
        ending_e_and_p_cents: ending_e_and_p,
        ending_stock_basis_cents: ending_stock_basis,
        aaa_available_for_distribution_cents: aaa_available_for_distribution,
        e_and_p_first_election_engaged,
        citation: "26 U.S.C. § 1368 (general distribution rules); 26 U.S.C. § 1368(a) \
                   (treatment); 26 U.S.C. § 1368(b) (no-E&P two-step); 26 U.S.C. \
                   § 1368(b)(1) (tax-free basis reduction); 26 U.S.C. § 1368(b)(2) \
                   (excess capital gain); 26 U.S.C. § 1368(c) (with-E&P four-step); \
                   26 U.S.C. § 1368(c)(1) (AAA); 26 U.S.C. § 1368(c)(2) (E&P dividend); \
                   26 U.S.C. § 1368(c)(3) (remaining basis); 26 U.S.C. § 1368(e)(1)(A) \
                   (AAA definition); 26 U.S.C. § 1368(e)(1)(C) (net negative adjustment \
                   rule); 26 U.S.C. § 1368(e)(3) (E&P-first election); Treas. Reg. \
                   § 1.1368-1 (distribution treatment); Treas. Reg. § 1.1368-2 (AAA \
                   mechanics); Form 1120-S Schedule M-2 (AAA + E&P tracking)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> Section1368Input {
        Section1368Input {
            distribution_cents: 5_000_000,        // $50K
            beginning_aaa_cents: 10_000_000,      // $100K
            accumulated_e_and_p_cents: 0,
            beginning_stock_basis_cents: 20_000_000, // $200K
            net_positive_adjustments_for_year_cents: 0,
            net_negative_adjustments_for_year_cents: 0,
            elects_e_and_p_first_under_1368e3: false,
        }
    }

    // ── § 1368(b) — no E&P paths ───────────────────────────────

    #[test]
    fn no_e_and_p_distribution_within_basis_all_tax_free() {
        let r = compute(&input());
        assert_eq!(r.tax_free_from_basis_cents, 5_000_000);
        assert_eq!(r.tax_free_from_aaa_cents, 0);
        assert_eq!(r.dividend_from_e_and_p_cents, 0);
        assert_eq!(r.capital_gain_cents, 0);
        assert_eq!(r.ending_stock_basis_cents, 15_000_000);
    }

    #[test]
    fn no_e_and_p_distribution_exceeds_basis_capital_gain() {
        let mut b = input();
        b.distribution_cents = 25_000_000; // $250K against $200K basis
        let r = compute(&b);
        assert_eq!(r.tax_free_from_basis_cents, 20_000_000);
        assert_eq!(r.capital_gain_cents, 5_000_000);
        assert_eq!(r.ending_stock_basis_cents, 0);
    }

    #[test]
    fn no_e_and_p_zero_basis_full_capital_gain() {
        let mut b = input();
        b.beginning_stock_basis_cents = 0;
        let r = compute(&b);
        assert_eq!(r.tax_free_from_basis_cents, 0);
        assert_eq!(r.capital_gain_cents, 5_000_000);
    }

    // ── § 1368(c) — with E&P default ordering ──────────────────

    #[test]
    fn with_e_and_p_distribution_within_aaa_all_tax_free() {
        let mut b = input();
        b.accumulated_e_and_p_cents = 5_000_000;
        // Distribution $50K vs AAA $100K → all from AAA.
        let r = compute(&b);
        assert_eq!(r.tax_free_from_aaa_cents, 5_000_000);
        assert_eq!(r.dividend_from_e_and_p_cents, 0);
        assert_eq!(r.tax_free_from_basis_cents, 0);
        assert_eq!(r.capital_gain_cents, 0);
        assert_eq!(r.ending_aaa_cents, 5_000_000);
        assert_eq!(r.ending_e_and_p_cents, 5_000_000);
        assert_eq!(r.ending_stock_basis_cents, 15_000_000);
    }

    #[test]
    fn with_e_and_p_aaa_exhausted_then_dividend() {
        let mut b = input();
        b.distribution_cents = 15_000_000;       // $150K
        b.beginning_aaa_cents = 5_000_000;        // $50K AAA
        b.accumulated_e_and_p_cents = 8_000_000;  // $80K E&P
        // Step 1: $50K from AAA. Remaining: $100K.
        // Step 2: $80K dividend. Remaining: $20K.
        // Step 3: $20K tax-free from basis (basis was $200K - $50K = $150K).
        // Step 4: $0 capital gain.
        let r = compute(&b);
        assert_eq!(r.tax_free_from_aaa_cents, 5_000_000);
        assert_eq!(r.dividend_from_e_and_p_cents, 8_000_000);
        assert_eq!(r.tax_free_from_basis_cents, 2_000_000);
        assert_eq!(r.capital_gain_cents, 0);
        assert_eq!(r.ending_aaa_cents, 0);
        assert_eq!(r.ending_e_and_p_cents, 0);
        assert_eq!(r.ending_stock_basis_cents, 13_000_000);
    }

    #[test]
    fn with_e_and_p_exhausts_all_four_paths_capital_gain() {
        let mut b = input();
        b.distribution_cents = 100_000_000;       // $1M huge
        b.beginning_aaa_cents = 5_000_000;        // $50K
        b.accumulated_e_and_p_cents = 8_000_000;  // $80K
        b.beginning_stock_basis_cents = 10_000_000; // $100K
        // AAA: $50K, basis decreases to $50K. Remaining $950K.
        // Dividend: $80K. Remaining $870K.
        // Basis: $50K applied. Remaining $820K.
        // Capital gain: $820K.
        let r = compute(&b);
        assert_eq!(r.tax_free_from_aaa_cents, 5_000_000);
        assert_eq!(r.dividend_from_e_and_p_cents, 8_000_000);
        assert_eq!(r.tax_free_from_basis_cents, 5_000_000);
        assert_eq!(r.capital_gain_cents, 82_000_000);
        assert_eq!(r.ending_stock_basis_cents, 0);
    }

    // ── § 1368(e)(3) election — E&P first ───────────────────────

    #[test]
    fn election_e_and_p_first_reorders() {
        let mut b = input();
        b.distribution_cents = 10_000_000;        // $100K
        b.beginning_aaa_cents = 3_000_000;        // $30K
        b.accumulated_e_and_p_cents = 4_000_000;  // $40K
        b.elects_e_and_p_first_under_1368e3 = true;
        // Election order: $40K dividend → $30K AAA → $30K basis → $0 cap gain.
        let r = compute(&b);
        assert_eq!(r.dividend_from_e_and_p_cents, 4_000_000);
        assert_eq!(r.tax_free_from_aaa_cents, 3_000_000);
        assert_eq!(r.tax_free_from_basis_cents, 3_000_000);
        assert_eq!(r.capital_gain_cents, 0);
        assert!(r.e_and_p_first_election_engaged);
    }

    #[test]
    fn election_ignored_when_no_e_and_p() {
        let mut b = input();
        b.elects_e_and_p_first_under_1368e3 = true;
        b.accumulated_e_and_p_cents = 0;
        let r = compute(&b);
        assert!(!r.e_and_p_first_election_engaged);
        // Falls through to § 1368(b) — basis reduction.
        assert_eq!(r.tax_free_from_basis_cents, 5_000_000);
    }

    // ── § 1368(e)(1)(C) net-negative adjustment rule ───────────

    #[test]
    fn net_negative_adjustment_ignored_for_distribution() {
        let mut b = input();
        b.distribution_cents = 8_000_000;            // $80K
        b.beginning_aaa_cents = 10_000_000;          // $100K
        b.accumulated_e_and_p_cents = 5_000_000;
        b.net_positive_adjustments_for_year_cents = 3_000_000;
        b.net_negative_adjustments_for_year_cents = 8_000_000;
        // net negative ($80K) > net positive ($30K) → ignore net negative for distribution.
        // AAA available = $100K beginning + $30K positive = $130K.
        let r = compute(&b);
        assert_eq!(r.aaa_available_for_distribution_cents, 13_000_000);
        // All $80K distribution comes from AAA.
        assert_eq!(r.tax_free_from_aaa_cents, 8_000_000);
        // End-of-year AAA reflects everything: $100K + $30K - $80K - $80K dist = -$30K.
        assert_eq!(r.ending_aaa_cents, -3_000_000);
    }

    #[test]
    fn net_positive_exceeds_net_negative_both_apply() {
        let mut b = input();
        b.distribution_cents = 5_000_000;
        b.beginning_aaa_cents = 10_000_000;
        b.accumulated_e_and_p_cents = 5_000_000;
        b.net_positive_adjustments_for_year_cents = 8_000_000;
        b.net_negative_adjustments_for_year_cents = 3_000_000;
        // Both apply: AAA available = $100K + $80K - $30K = $150K.
        let r = compute(&b);
        assert_eq!(r.aaa_available_for_distribution_cents, 15_000_000);
    }

    // ── AAA bound at 0 for distribution purposes ──────────────

    #[test]
    fn negative_aaa_floored_at_zero_for_distribution() {
        let mut b = input();
        b.distribution_cents = 5_000_000;
        b.beginning_aaa_cents = -10_000_000; // negative AAA
        b.accumulated_e_and_p_cents = 8_000_000;
        // AAA available = max(0, -$100K) = 0. All distribution → E&P first.
        let r = compute(&b);
        assert_eq!(r.aaa_available_for_distribution_cents, 0);
        assert_eq!(r.tax_free_from_aaa_cents, 0);
        assert_eq!(r.dividend_from_e_and_p_cents, 5_000_000);
    }

    // ── Zero distribution ─────────────────────────────────────

    #[test]
    fn zero_distribution_no_tax_effects() {
        let mut b = input();
        b.distribution_cents = 0;
        let r = compute(&b);
        assert_eq!(r.tax_free_from_aaa_cents, 0);
        assert_eq!(r.dividend_from_e_and_p_cents, 0);
        assert_eq!(r.tax_free_from_basis_cents, 0);
        assert_eq!(r.capital_gain_cents, 0);
        assert_eq!(r.ending_stock_basis_cents, b.beginning_stock_basis_cents);
    }

    // ── Multi-regime invariants ───────────────────────────────

    #[test]
    fn dividend_only_when_e_and_p_present_invariant() {
        // No E&P: dividend always 0 regardless of other inputs.
        for distribution in [0, 5_000_000, 100_000_000] {
            for basis in [0, 10_000_000, 50_000_000] {
                let mut b = input();
                b.distribution_cents = distribution;
                b.beginning_stock_basis_cents = basis;
                b.accumulated_e_and_p_cents = 0;
                let r = compute(&b);
                assert_eq!(r.dividend_from_e_and_p_cents, 0);
            }
        }
    }

    #[test]
    fn dividend_never_reduces_basis_invariant() {
        let mut b = input();
        b.distribution_cents = 5_000_000;
        b.beginning_aaa_cents = 0;
        b.accumulated_e_and_p_cents = 10_000_000;
        b.beginning_stock_basis_cents = 20_000_000;
        // Default order: AAA ($0), then dividend ($5M), then basis (none needed).
        let r = compute(&b);
        assert_eq!(r.dividend_from_e_and_p_cents, 5_000_000);
        // Basis unchanged because no tax-free portion reduced it.
        assert_eq!(r.ending_stock_basis_cents, 20_000_000);
    }

    #[test]
    fn distribution_sum_equals_input_invariant() {
        // Total of all 4 categories should equal distribution.
        for distribution in [5_000_000, 50_000_000, 100_000_000] {
            let mut b = input();
            b.distribution_cents = distribution;
            b.beginning_aaa_cents = 10_000_000;
            b.accumulated_e_and_p_cents = 20_000_000;
            b.beginning_stock_basis_cents = 15_000_000;
            let r = compute(&b);
            let total = r.tax_free_from_aaa_cents
                + r.dividend_from_e_and_p_cents
                + r.tax_free_from_basis_cents
                + r.capital_gain_cents;
            assert_eq!(total, distribution, "distribution={}", distribution);
        }
    }

    #[test]
    fn election_vs_default_when_e_and_p_present_different_ordering() {
        let mut b = input();
        b.distribution_cents = 10_000_000;
        b.beginning_aaa_cents = 3_000_000;
        b.accumulated_e_and_p_cents = 4_000_000;
        b.beginning_stock_basis_cents = 5_000_000;

        let r_default = compute(&b);
        b.elects_e_and_p_first_under_1368e3 = true;
        let r_election = compute(&b);

        // Default: AAA $3M → divid $4M → basis $3M → cap gain $0. Total $10M ✓
        // Election: divid $4M → AAA $3M → basis $3M → cap gain $0. Total $10M ✓
        // Same dollar totals but ordering visible in flags.
        assert!(!r_default.e_and_p_first_election_engaged);
        assert!(r_election.e_and_p_first_election_engaged);
        // Both produce same dividend amount in this specific case.
        assert_eq!(r_default.dividend_from_e_and_p_cents, r_election.dividend_from_e_and_p_cents);
    }

    // ── Citation + sibling cluster ────────────────────────────

    #[test]
    fn citation_pins_all_subsections() {
        let r = compute(&input());
        assert!(r.citation.contains("§ 1368"));
        assert!(r.citation.contains("§ 1368(a)"));
        assert!(r.citation.contains("§ 1368(b)"));
        assert!(r.citation.contains("§ 1368(b)(1)"));
        assert!(r.citation.contains("§ 1368(b)(2)"));
        assert!(r.citation.contains("§ 1368(c)"));
        assert!(r.citation.contains("§ 1368(c)(1)"));
        assert!(r.citation.contains("§ 1368(c)(2)"));
        assert!(r.citation.contains("§ 1368(c)(3)"));
        assert!(r.citation.contains("§ 1368(e)(1)(A)"));
        assert!(r.citation.contains("§ 1368(e)(1)(C)"));
        assert!(r.citation.contains("§ 1368(e)(3)"));
        assert!(r.citation.contains("§ 1.1368-1"));
        assert!(r.citation.contains("§ 1.1368-2"));
        assert!(r.citation.contains("Form 1120-S"));
    }

    #[test]
    fn sibling_modules_note_present() {
        let r = compute(&input());
        assert!(
            r.notes.iter().any(|n| n.contains("§ 1361")
                && n.contains("§ 1366")
                && n.contains("§ 1367")
                && n.contains("§ 1374")
                && n.contains("§ 1375")
                && n.contains("Form 1120-S Schedule M-2")),
            "S-corp cluster note must reference § 1361 + § 1366 + § 1367 + § 1374 + § 1375 + Form 1120-S Schedule M-2"
        );
    }

    // ── Defensive input clamping ───────────────────────────────

    #[test]
    fn defensive_negative_distribution_clamped() {
        let mut b = input();
        b.distribution_cents = -1_000_000;
        let r = compute(&b);
        assert_eq!(r.tax_free_from_basis_cents, 0);
        assert_eq!(r.capital_gain_cents, 0);
    }

    #[test]
    fn defensive_negative_basis_clamped() {
        let mut b = input();
        b.beginning_stock_basis_cents = -10_000_000;
        let r = compute(&b);
        assert_eq!(r.tax_free_from_basis_cents, 0);
        assert_eq!(r.capital_gain_cents, 5_000_000);
    }

    #[test]
    fn defensive_negative_e_and_p_clamped() {
        let mut b = input();
        b.accumulated_e_and_p_cents = -5_000_000;
        let r = compute(&b);
        // Treated as 0 E&P → § 1368(b) path.
        assert_eq!(r.dividend_from_e_and_p_cents, 0);
        assert_eq!(r.tax_free_from_basis_cents, 5_000_000);
    }

    #[test]
    fn boundary_distribution_exactly_aaa_no_dividend() {
        let mut b = input();
        b.distribution_cents = 10_000_000;
        b.beginning_aaa_cents = 10_000_000;
        b.accumulated_e_and_p_cents = 5_000_000;
        // Distribution exactly = AAA available → no dividend, no basis tax-free.
        let r = compute(&b);
        assert_eq!(r.tax_free_from_aaa_cents, 10_000_000);
        assert_eq!(r.dividend_from_e_and_p_cents, 0);
        assert_eq!(r.tax_free_from_basis_cents, 0);
    }

    #[test]
    fn boundary_distribution_one_cent_above_aaa() {
        let mut b = input();
        b.distribution_cents = 10_000_001;
        b.beginning_aaa_cents = 10_000_000;
        b.accumulated_e_and_p_cents = 5_000_000;
        let r = compute(&b);
        assert_eq!(r.tax_free_from_aaa_cents, 10_000_000);
        assert_eq!(r.dividend_from_e_and_p_cents, 1);
    }
}
