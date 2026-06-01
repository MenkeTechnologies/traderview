//! IRC § 263(g) — Capitalization of certain interest and carrying
//! charges in the case of straddles.
//!
//! Trader-critical companion to `section_1092` (straddle definition)
//! and `section_1256` (hedging-transaction definition). § 263(g)
//! prevents a trader holding offsetting derivative positions from
//! deducting margin-loan interest and other carrying charges in the
//! current year; the disallowed amount is CAPITALIZED into the basis
//! of the straddle property instead.
//!
//! Four operative subparagraphs (added by §§ 501 and 502 of the
//! Economic Recovery Tax Act of 1981, Pub. L. 97-34, 95 Stat. 172):
//!
//!   § 263(g)(1) — GENERAL RULE: "No deduction shall be allowed for
//!     interest and carrying charges properly allocable to personal
//!     property which is part of a straddle." Disallowed amount is
//!     chargeable to the capital account with respect to the personal
//!     property to which it relates.
//!
//!   § 263(g)(2) — INTEREST AND CARRYING CHARGES DEFINED as the
//!     EXCESS of:
//!       (A) the sum of:
//!           (i) interest on indebtedness incurred or continued to
//!               purchase or carry the personal property, AND
//!           (ii) all other amounts (including charges to insure,
//!                store, or transport the personal property) paid
//!                or incurred to carry the personal property,
//!     OVER:
//!       (B) the sum of:
//!           (i) the amount of interest (including OID) includible
//!               in gross income for the taxable year with respect
//!               to such property,
//!           (ii) any amount treated as ordinary income from such
//!                property,
//!           (iii) dividends (net of the dividends-received
//!                 deduction under § 243), AND
//!           (iv) any payment with respect to a security loan with
//!                respect to such property includible in gross
//!                income for the taxable year.
//!
//!   § 263(g)(3) — HEDGING EXCEPTION: "This subsection shall not
//!     apply in the case of any hedging transaction (as defined in
//!     section 1256(e))." § 1256(e) requires the transaction to be
//!     a bona fide hedge of ordinary property, ordinary obligations,
//!     or borrowings (identified as such before the close of the day
//!     it was entered into).
//!
//!   § 263(g)(4) — COORDINATION RULES with § 263(h) short-sale
//!     coordination and §§ 1277 / 1282 market-discount / OID rules.
//!
//! Operative consequence — disallowance is DEFERRED, not permanent.
//! Capitalizing the carrying charges into basis means a higher
//! basis on eventual disposition → smaller gain (or larger loss) at
//! that point. The income tax effect is timing-only, not absolute.
//!
//! Citations: 26 U.S.C. § 263(g)(1) (general rule disallowing
//! deduction); § 263(g)(2)(A) (interest + carrying costs definition);
//! § 263(g)(2)(B) (offset for includible income); § 263(g)(3)
//! (hedging-transaction exception); § 263(g)(4) (coordination);
//! § 1092(c) (straddle definition); § 1256(e) (hedging-transaction
//! definition); ERTA 1981 §§ 501, 502.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Section263GInput {
    /// True if the position is part of a § 1092(c) straddle. § 263(g)
    /// applies only to straddle positions.
    pub is_part_of_straddle: bool,
    /// True if the transaction qualifies as a § 1256(e) hedging
    /// transaction. § 263(g)(3) exempts hedging transactions even
    /// where the position would otherwise be a straddle.
    pub is_hedging_transaction: bool,
    /// § 263(g)(2)(A)(i) — interest on indebtedness incurred or
    /// continued to purchase or carry the straddle property (cents).
    pub interest_on_indebtedness_cents: i64,
    /// § 263(g)(2)(A)(ii) — other carrying costs: storage,
    /// insurance, transport (cents).
    pub carrying_costs_cents: i64,
    /// § 263(g)(2)(B)(i) — interest (including OID) includible in
    /// gross income with respect to the property (cents).
    pub interest_received_cents: i64,
    /// § 263(g)(2)(B)(ii) — amount treated as ordinary income from
    /// the property (cents).
    pub ordinary_income_from_property_cents: i64,
    /// § 263(g)(2)(B)(iii) — gross dividends received (cents).
    pub dividends_received_cents: i64,
    /// § 263(g)(2)(B)(iii) — dividends-received deduction under § 243
    /// (cents). Subtracted from dividends_received to compute net
    /// dividend offset.
    pub dividend_received_deduction_cents: i64,
    /// § 263(g)(2)(B)(iv) — payments received with respect to a
    /// security loan that are includible in gross income (cents).
    pub security_loan_fees_received_cents: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section263GResult {
    /// True iff § 263(g) reaches this position (straddle AND not a
    /// hedging transaction).
    pub section_263g_applies: bool,
    /// § 263(g)(2)(A) total — interest plus carrying costs (cents).
    pub gross_interest_and_carrying_charges_cents: i64,
    /// § 263(g)(2)(B) total — sum of offset income items (cents).
    pub offset_income_cents: i64,
    /// Excess of (A) over (B) — the amount subject to capitalization
    /// when § 263(g) applies (cents). Zero where (B) ≥ (A) (no
    /// negative excess).
    pub net_interest_and_carrying_charges_cents: i64,
    /// Amount of the current-year deduction disallowed (cents).
    /// Equal to net excess when § 263(g) applies; zero otherwise.
    pub disallowed_deduction_cents: i64,
    /// Amount chargeable to the capital account (basis of the
    /// straddle property) under § 263(g)(1) (cents). Equal to
    /// disallowed_deduction_cents — disallowance and capitalization
    /// are the SAME dollars by statute.
    pub capitalized_to_basis_cents: i64,
    /// Current-year deductible portion (cents). When § 263(g) does
    /// not apply, this equals gross_interest_and_carrying_charges.
    /// When § 263(g) applies, this equals (gross − disallowed) which
    /// for the formula above equals the offset_income amount, but
    /// only up to the gross amount (no negative deduction).
    pub current_deduction_allowed_cents: i64,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn compute(input: &Section263GInput) -> Section263GResult {
    let mut notes: Vec<String> = Vec::new();

    let gross = input
        .interest_on_indebtedness_cents
        .saturating_add(input.carrying_costs_cents);

    // Net dividend offset = dividends − dividends-received deduction.
    // The DRD cannot exceed the dividend amount itself (saturate at 0).
    let net_dividend_offset = input
        .dividends_received_cents
        .saturating_sub(input.dividend_received_deduction_cents)
        .max(0);

    let offset = input
        .interest_received_cents
        .saturating_add(input.ordinary_income_from_property_cents)
        .saturating_add(net_dividend_offset)
        .saturating_add(input.security_loan_fees_received_cents);

    // § 263(g)(2) net = max(0, gross − offset).
    let net_excess = gross.saturating_sub(offset).max(0);

    let section_263g_applies = input.is_part_of_straddle && !input.is_hedging_transaction;

    // § 263(g)(3) hedging exception takes priority over the general
    // rule. Note the priority chain explicitly.
    if input.is_hedging_transaction {
        notes.push(
            "§ 263(g)(3) — hedging transaction (§ 1256(e)) — § 263(g) does NOT apply; \
             interest and carrying charges are currently deductible subject to other Code \
             limits (§ 163, § 162, etc.)."
                .to_string(),
        );
    } else if !input.is_part_of_straddle {
        notes.push(
            "§ 263(g) — position is not part of a § 1092(c) straddle; § 263(g) does not \
             reach this transaction. Interest + carrying charges currently deductible subject \
             to other Code limits."
                .to_string(),
        );
    } else {
        notes.push(
            "§ 263(g)(1) — position is part of a § 1092(c) straddle and is not a § 1256(e) \
             hedging transaction. Disallowance applies."
                .to_string(),
        );
        notes.push(
            "§ 263(g) disallowance is timing-only, not permanent — the disallowed amount is \
             chargeable to the capital account (basis of the straddle property) under \
             § 263(g)(1), reducing eventual gain or increasing eventual loss on disposition."
                .to_string(),
        );
    }

    let (disallowed, capitalized, current_deductible, citation) = if section_263g_applies {
        // The "deductible portion" is what § 263(g) does NOT disallow.
        // Since the disallowed amount is the EXCESS of A over B, the
        // deductible portion is whatever portion of A is matched
        // dollar-for-dollar by B (i.e., min(A, B)).
        let deductible = gross.saturating_sub(net_excess);
        (
            net_excess,
            net_excess,
            deductible,
            "26 U.S.C. § 263(g)(1) (general rule disallowing deduction for interest and \
             carrying charges on straddle property; disallowed amount capitalized to basis); \
             § 263(g)(2)(A)–(B) (excess-of-gross-over-includible-income definition); \
             § 1092(c) (straddle definition)",
        )
    } else if input.is_hedging_transaction {
        (
            0,
            0,
            gross,
            "26 U.S.C. § 263(g)(3) (hedging-transaction exception); § 1256(e) (definition of \
             hedging transaction)",
        )
    } else {
        (
            0,
            0,
            gross,
            "26 U.S.C. § 263(g) (inapplicable — not a § 1092(c) straddle); § 1092(c) (straddle \
             definition)",
        )
    };

    Section263GResult {
        section_263g_applies,
        gross_interest_and_carrying_charges_cents: gross,
        offset_income_cents: offset,
        net_interest_and_carrying_charges_cents: net_excess,
        disallowed_deduction_cents: disallowed,
        capitalized_to_basis_cents: capitalized,
        current_deduction_allowed_cents: current_deductible,
        citation,
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        straddle: bool,
        hedging: bool,
        interest_debt: i64,
        carrying: i64,
        interest_rcv: i64,
        ord_inc: i64,
        div_rcv: i64,
        drd: i64,
        sec_loan_fees: i64,
    ) -> Section263GInput {
        Section263GInput {
            is_part_of_straddle: straddle,
            is_hedging_transaction: hedging,
            interest_on_indebtedness_cents: interest_debt,
            carrying_costs_cents: carrying,
            interest_received_cents: interest_rcv,
            ordinary_income_from_property_cents: ord_inc,
            dividends_received_cents: div_rcv,
            dividend_received_deduction_cents: drd,
            security_loan_fees_received_cents: sec_loan_fees,
        }
    }

    // ── § 263(g)(1) general rule — straddle + non-hedging ───────

    #[test]
    fn straddle_non_hedging_disallows_excess_above_offset() {
        // Gross = 10,000 interest. Offset = 3,000. Excess = 7,000.
        let r = compute(&input(true, false, 10_000, 0, 3_000, 0, 0, 0, 0));
        assert!(r.section_263g_applies);
        assert_eq!(r.gross_interest_and_carrying_charges_cents, 10_000);
        assert_eq!(r.offset_income_cents, 3_000);
        assert_eq!(r.net_interest_and_carrying_charges_cents, 7_000);
        assert_eq!(r.disallowed_deduction_cents, 7_000);
        assert_eq!(r.capitalized_to_basis_cents, 7_000);
        assert_eq!(r.current_deduction_allowed_cents, 3_000);
        assert!(r.citation.contains("§ 263(g)(1)"));
        assert!(r.citation.contains("§ 1092(c)"));
    }

    #[test]
    fn straddle_offset_exceeds_gross_zero_disallowance() {
        // Gross = 5,000. Offset = 8,000. Excess = 0 (no negative).
        let r = compute(&input(true, false, 5_000, 0, 8_000, 0, 0, 0, 0));
        assert!(r.section_263g_applies);
        assert_eq!(r.net_interest_and_carrying_charges_cents, 0);
        assert_eq!(r.disallowed_deduction_cents, 0);
        assert_eq!(r.capitalized_to_basis_cents, 0);
        assert_eq!(r.current_deduction_allowed_cents, 5_000);
    }

    #[test]
    fn straddle_zero_gross_zero_disallowance() {
        let r = compute(&input(true, false, 0, 0, 0, 0, 0, 0, 0));
        assert!(r.section_263g_applies);
        assert_eq!(r.disallowed_deduction_cents, 0);
        assert_eq!(r.current_deduction_allowed_cents, 0);
    }

    #[test]
    fn straddle_interest_plus_carrying_costs_both_in_gross() {
        // Interest 6,000 + carrying 4,000 = 10,000 gross.
        let r = compute(&input(true, false, 6_000, 4_000, 0, 0, 0, 0, 0));
        assert_eq!(r.gross_interest_and_carrying_charges_cents, 10_000);
        assert_eq!(r.disallowed_deduction_cents, 10_000);
    }

    // ── § 263(g)(2)(B) offset components ────────────────────────

    #[test]
    fn offset_interest_income_includible() {
        let r = compute(&input(true, false, 10_000, 0, 4_000, 0, 0, 0, 0));
        assert_eq!(r.offset_income_cents, 4_000);
        assert_eq!(r.disallowed_deduction_cents, 6_000);
    }

    #[test]
    fn offset_ordinary_income_from_property() {
        let r = compute(&input(true, false, 10_000, 0, 0, 2_500, 0, 0, 0));
        assert_eq!(r.offset_income_cents, 2_500);
        assert_eq!(r.disallowed_deduction_cents, 7_500);
    }

    #[test]
    fn offset_dividend_net_of_drd() {
        // Dividend $1,000; DRD $500. Net offset = $500.
        let r = compute(&input(true, false, 10_000, 0, 0, 0, 1_000, 500, 0));
        assert_eq!(r.offset_income_cents, 500);
        assert_eq!(r.disallowed_deduction_cents, 9_500);
    }

    #[test]
    fn drd_exceeding_dividend_does_not_create_negative_offset() {
        // DRD $2,000 on a $1,000 dividend should saturate at 0,
        // not produce −$1,000 offset.
        let r = compute(&input(true, false, 10_000, 0, 0, 0, 1_000, 2_000, 0));
        assert_eq!(r.offset_income_cents, 0);
        assert_eq!(r.disallowed_deduction_cents, 10_000);
    }

    #[test]
    fn offset_security_loan_fees_received() {
        let r = compute(&input(true, false, 10_000, 0, 0, 0, 0, 0, 750));
        assert_eq!(r.offset_income_cents, 750);
        assert_eq!(r.disallowed_deduction_cents, 9_250);
    }

    #[test]
    fn offset_all_four_components_sum() {
        // Interest 1,000 + ord 500 + (div 600 − DRD 200) + sec loan 300
        // = 1,000 + 500 + 400 + 300 = 2,200.
        let r = compute(&input(true, false, 10_000, 0, 1_000, 500, 600, 200, 300));
        assert_eq!(r.offset_income_cents, 2_200);
        assert_eq!(r.disallowed_deduction_cents, 7_800);
    }

    // ── § 263(g)(3) hedging-transaction exception ───────────────

    #[test]
    fn hedging_transaction_section_263g_does_not_apply() {
        // § 263(g)(3): § 1256(e) hedging transaction exempts even
        // when the position is in form a straddle.
        let r = compute(&input(true, true, 10_000, 0, 0, 0, 0, 0, 0));
        assert!(!r.section_263g_applies);
        assert_eq!(r.disallowed_deduction_cents, 0);
        assert_eq!(r.capitalized_to_basis_cents, 0);
        assert_eq!(r.current_deduction_allowed_cents, 10_000);
        assert!(r.citation.contains("§ 263(g)(3)"));
        assert!(r.citation.contains("§ 1256(e)"));
    }

    #[test]
    fn hedging_exception_priority_over_straddle_status() {
        // Even with straddle = true, hedging = true wins.
        let with_hedge = compute(&input(true, true, 10_000, 0, 1_000, 0, 0, 0, 0));
        let without_hedge = compute(&input(true, false, 10_000, 0, 1_000, 0, 0, 0, 0));
        assert!(!with_hedge.section_263g_applies);
        assert!(without_hedge.section_263g_applies);
        assert_eq!(with_hedge.disallowed_deduction_cents, 0);
        assert_eq!(without_hedge.disallowed_deduction_cents, 9_000);
    }

    // ── Non-straddle path ──────────────────────────────────────

    #[test]
    fn non_straddle_section_263g_does_not_apply() {
        let r = compute(&input(false, false, 10_000, 0, 0, 0, 0, 0, 0));
        assert!(!r.section_263g_applies);
        assert_eq!(r.disallowed_deduction_cents, 0);
        assert_eq!(r.current_deduction_allowed_cents, 10_000);
        assert!(r.citation.contains("inapplicable"));
    }

    #[test]
    fn non_straddle_holds_full_gross_deductible() {
        let r = compute(&input(false, false, 10_000, 5_000, 0, 0, 0, 0, 0));
        assert_eq!(r.gross_interest_and_carrying_charges_cents, 15_000);
        assert_eq!(r.current_deduction_allowed_cents, 15_000);
    }

    // ── Disallowance == Capitalization invariant ────────────────

    #[test]
    fn disallowed_amount_equals_capitalized_amount_invariant() {
        // § 263(g)(1) requires disallowed amount to be chargeable to
        // the capital account — they are the SAME dollars.
        for (straddle, hedging) in [(true, false), (true, true), (false, false)] {
            let r = compute(&input(straddle, hedging, 8_000, 0, 1_500, 0, 0, 0, 0));
            assert_eq!(
                r.disallowed_deduction_cents,
                r.capitalized_to_basis_cents,
                "straddle={straddle} hedging={hedging}: disallowed and capitalized must match",
            );
        }
    }

    #[test]
    fn deductible_plus_disallowed_equals_gross_invariant() {
        // Conservation: nothing disappears. Deductible + disallowed
        // must equal gross interest + carrying charges across all
        // paths.
        for (straddle, hedging) in [(true, false), (true, true), (false, false)] {
            let r = compute(&input(straddle, hedging, 12_000, 3_000, 4_000, 0, 0, 0, 0));
            assert_eq!(
                r.current_deduction_allowed_cents + r.disallowed_deduction_cents,
                r.gross_interest_and_carrying_charges_cents,
                "straddle={straddle} hedging={hedging}: conservation must hold",
            );
        }
    }

    // ── Multi-path regression invariants ───────────────────────

    #[test]
    fn only_straddle_and_non_hedging_triggers_section_263g_invariant() {
        // Across the 4 combos of (straddle, hedging), only
        // straddle=true + hedging=false triggers § 263(g).
        let s_h = compute(&input(true, true, 10_000, 0, 0, 0, 0, 0, 0));
        let s_nh = compute(&input(true, false, 10_000, 0, 0, 0, 0, 0, 0));
        let ns_h = compute(&input(false, true, 10_000, 0, 0, 0, 0, 0, 0));
        let ns_nh = compute(&input(false, false, 10_000, 0, 0, 0, 0, 0, 0));

        assert!(!s_h.section_263g_applies);
        assert!(s_nh.section_263g_applies);
        assert!(!ns_h.section_263g_applies);
        assert!(!ns_nh.section_263g_applies);

        // Only s_nh has nonzero disallowance.
        assert!(s_nh.disallowed_deduction_cents > 0);
        assert_eq!(s_h.disallowed_deduction_cents, 0);
        assert_eq!(ns_h.disallowed_deduction_cents, 0);
        assert_eq!(ns_nh.disallowed_deduction_cents, 0);
    }

    #[test]
    fn citation_pins_subsection_per_path() {
        let s_nh = compute(&input(true, false, 1, 0, 0, 0, 0, 0, 0));
        let s_h = compute(&input(true, true, 1, 0, 0, 0, 0, 0, 0));
        let ns = compute(&input(false, false, 1, 0, 0, 0, 0, 0, 0));

        assert!(s_nh.citation.contains("§ 263(g)(1)"));
        assert!(s_nh.citation.contains("§ 263(g)(2)"));
        assert!(s_nh.citation.contains("§ 1092(c)"));
        assert!(s_h.citation.contains("§ 263(g)(3)"));
        assert!(s_h.citation.contains("§ 1256(e)"));
        assert!(ns.citation.contains("inapplicable"));
        assert!(ns.citation.contains("§ 1092(c)"));
    }

    #[test]
    fn timing_only_note_present_when_section_263g_applies() {
        let r = compute(&input(true, false, 10_000, 0, 0, 0, 0, 0, 0));
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("timing-only") && n.contains("basis")),
            "must explain that disallowance is timing-only via basis adjustment"
        );
    }

    #[test]
    fn hedging_note_cites_section_1256e_definition() {
        let r = compute(&input(true, true, 10_000, 0, 0, 0, 0, 0, 0));
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("§ 263(g)(3)") && n.contains("§ 1256(e)")),
            "hedging-transaction note must cite § 263(g)(3) and § 1256(e)"
        );
    }

    #[test]
    fn non_straddle_note_explains_inapplicability() {
        let r = compute(&input(false, false, 10_000, 0, 0, 0, 0, 0, 0));
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("not part of a § 1092(c) straddle")),
            "non-straddle note must reference § 1092(c)"
        );
    }

    // ── Real-world fact patterns ───────────────────────────────

    #[test]
    fn protective_put_on_long_stock_straddle_with_margin_interest() {
        // Trader holds 100 sh AAPL long on margin and buys a
        // protective put at strike below market. § 1092(c) straddle.
        // Margin interest $5,000 + storage $0 = gross $5,000.
        // Dividends received $200, no DRD. Offset $200.
        // Disallowed = $4,800.
        let r = compute(&input(true, false, 5_000, 0, 0, 0, 200, 0, 0));
        assert!(r.section_263g_applies);
        assert_eq!(r.disallowed_deduction_cents, 4_800);
        assert_eq!(r.capitalized_to_basis_cents, 4_800);
        assert_eq!(r.current_deduction_allowed_cents, 200);
    }

    #[test]
    fn iron_condor_no_underlying_stock_carrying_charges_only() {
        // Iron condor = straddle (offsetting positions). No
        // long-stock margin interest, but commissions / per-contract
        // fees can be carrying costs. $300 carrying, $0 offset.
        let r = compute(&input(true, false, 0, 300, 0, 0, 0, 0, 0));
        assert!(r.section_263g_applies);
        assert_eq!(r.disallowed_deduction_cents, 300);
        assert_eq!(r.current_deduction_allowed_cents, 0);
    }

    #[test]
    fn inventory_hedge_uses_hedging_exception() {
        // § 1256(e) hedging — manufacturer uses commodity futures
        // to hedge inventory exposure. Even though paired derivative
        // positions are "straddle"-shaped, § 263(g)(3) exempts.
        let r = compute(&input(true, true, 50_000, 0, 0, 0, 0, 0, 0));
        assert!(!r.section_263g_applies);
        assert_eq!(r.disallowed_deduction_cents, 0);
        assert_eq!(r.current_deduction_allowed_cents, 50_000);
    }

    #[test]
    fn long_only_position_no_straddle_full_deduction() {
        // Buy-and-hold AAPL on margin with no offsetting put — NOT
        // a straddle. § 263(g) doesn't apply; full margin-interest
        // deduction (subject to § 163(d) investment-interest limit
        // separately).
        let r = compute(&input(false, false, 10_000, 0, 1_000, 0, 0, 0, 0));
        assert!(!r.section_263g_applies);
        assert_eq!(r.disallowed_deduction_cents, 0);
        assert_eq!(r.current_deduction_allowed_cents, 10_000);
    }
}
