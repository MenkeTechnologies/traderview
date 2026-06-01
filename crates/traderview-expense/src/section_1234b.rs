//! IRC § 1234B — Gains or losses from securities futures contracts.
//!
//! The companion section that § 1234A explicitly redirects to.
//! Governs the character of gain or loss on the sale, exchange, or
//! termination of a securities futures contract (single-stock future
//! or narrow-based security index future per § 3(a)(55)(A) of the
//! Securities Exchange Act of 1934).
//!
//! Three statutory subsections do the work:
//!
//!   § 1234B(a) — Gain or loss attributable to the sale, exchange,
//!     or termination of a securities futures contract shall be
//!     considered gain or loss from the sale or exchange of property
//!     which has the SAME CHARACTER as the property to which the
//!     contract relates has (or would have) in the taxpayer's hands.
//!     So underlying capital → capital character; underlying ordinary
//!     → ordinary character.
//!
//!   § 1234B(b) — If the gain/loss on the sale, exchange, or
//!     termination of a securities futures contract TO SELL property
//!     is considered as gain/loss from a capital asset, it shall be
//!     treated as SHORT-TERM CAPITAL gain or loss. This applies
//!     REGARDLESS OF HOLDING PERIOD. Mirrors § 1233 short-sale rule.
//!
//!   § 1234B(d) — A securities futures contract shall NOT be treated
//!     as a commodity futures contract. (Prevents accidental fall-in
//!     to commodity-futures branches of § 1256.)
//!
//! § 1256(b)(1)(E) override — DEALER securities futures contracts
//! ARE § 1256 contracts and get the § 1256(a)(3) 60/40 split. This
//! override fires BEFORE § 1234B. So the routing order for a
//! securities futures contract terminated by a trader is:
//!   1. Dealer SFC? → § 1256 60/40.
//!   2. Else SFC to SELL with capital underlying? → § 1234B(b) short-term.
//!   3. Else SFC with capital underlying? → § 1234B(a) holding-period split.
//!   4. Else SFC with ordinary underlying? → § 1234B(a) ordinary character.
//!
//! The § 1234B(b) short-term-regardless rule is the load-bearing
//! trader trap: a single-stock-future SHORT position held for 10
//! years still produces short-term capital gain/loss on close,
//! defeating any long-term-rate planning. The corresponding LONG
//! position uses normal holding-period math.
//!
//! Citations: 26 U.S.C. § 1234B(a) (character mirrors underlying);
//! § 1234B(b) (short-term capital regardless of holding period for
//! SFC to sell); § 1234B(c) (definition — Securities Exchange Act
//! § 3(a)(55)(A)); § 1234B(d) (not a commodity futures contract);
//! § 1256(b)(1)(E) (dealer SFC routes to § 1256 60/40); § 1256(a)(3)
//! (60/40 split); § 1222(1) (short-term ≤ 1 year); § 1222(3)
//! (long-term > 1 year).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Position {
    /// Long SFC — contract to BUY the underlying. § 1234B(b) does
    /// NOT apply; normal § 1222 holding-period split governs.
    Long,
    /// Short SFC — contract to SELL the underlying. § 1234B(b)
    /// forces SHORT-TERM CAPITAL regardless of holding period.
    Short,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GainCharacter {
    /// Capital, long-term per § 1222(3) (> 1 year hold).
    LongTermCapital,
    /// Capital, short-term per § 1222(1) (≤ 1 year hold).
    ShortTermCapital,
    /// Ordinary character (mirrors underlying ordinary property via
    /// § 1234B(a)).
    Ordinary,
    /// Dealer SFC routed to § 1256(a)(3) 60/40 split.
    Section1256_60_40,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section1234BInput {
    /// Gain (positive) or loss (negative) on sale, exchange, or
    /// termination of the SFC, in cents.
    pub gain_or_loss_cents: i64,
    /// Holding period of the contract in days. Used for long-position
    /// capital-underlying path; ignored for short-position (§ 1234B(b))
    /// and dealer (§ 1256) paths.
    pub holding_period_days: u32,
    /// Long (contract to buy) or short (contract to sell).
    pub position: Position,
    /// Whether the underlying property is (or would be on acquisition)
    /// a § 1221 capital asset in the taxpayer's hands. Drives § 1234B(a)
    /// character mirroring.
    pub underlying_is_capital_asset: bool,
    /// Whether the contract is a DEALER securities futures contract
    /// under § 1256(g)(9) — § 1256(b)(1)(E) routes it to § 1256 60/40
    /// rather than § 1234B.
    pub is_dealer_securities_futures_contract: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section1234BResult {
    pub character: GainCharacter,
    /// Long-term portion of gain/loss, in cents.
    pub long_term_cents: i64,
    /// Short-term portion of gain/loss, in cents.
    pub short_term_cents: i64,
    /// Ordinary portion of gain/loss, in cents. Non-zero only when
    /// underlying is ordinary.
    pub ordinary_cents: i64,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn compute(input: &Section1234BInput) -> Section1234BResult {
    let g = input.gain_or_loss_cents;
    let mut notes: Vec<String> = Vec::new();

    // Dealer SFC override — § 1256(b)(1)(E) routes dealer securities
    // futures contracts to § 1256 60/40 split. This fires BEFORE
    // § 1234B character analysis.
    if input.is_dealer_securities_futures_contract {
        let long_term = g * 60 / 100;
        let short_term = g - long_term;
        notes.push(
            "§ 1256(b)(1)(E) — DEALER securities futures contract is a § 1256 contract; \
             § 1256(a)(3) 60/40 split applies (60% long-term + 40% short-term) regardless of \
             holding period."
                .to_string(),
        );
        return Section1234BResult {
            character: GainCharacter::Section1256_60_40,
            long_term_cents: long_term,
            short_term_cents: short_term,
            ordinary_cents: 0,
            citation: "26 U.S.C. § 1256(b)(1)(E) (dealer securities futures contract is a § 1256 \
                       contract); § 1256(a)(3) (60/40 split regardless of holding period)",
            notes,
        };
    }

    // § 1234B(a) character mirror — if underlying is ordinary, the
    // SFC gain/loss is ordinary regardless of position or holding
    // period.
    if !input.underlying_is_capital_asset {
        notes.push(
            "§ 1234B(a) — character mirrors underlying property; ordinary underlying produces \
             ordinary gain or loss regardless of position or holding period."
                .to_string(),
        );
        return Section1234BResult {
            character: GainCharacter::Ordinary,
            long_term_cents: 0,
            short_term_cents: 0,
            ordinary_cents: g,
            citation: "26 U.S.C. § 1234B(a) (character mirrors underlying property in \
                       taxpayer's hands)",
            notes,
        };
    }

    // § 1234B(b) — contract to SELL → ALWAYS short-term capital
    // regardless of holding period.
    if matches!(input.position, Position::Short) {
        notes.push(
            "§ 1234B(b) — securities futures contract TO SELL with capital underlying produces \
             SHORT-TERM CAPITAL gain or loss REGARDLESS of holding period; parallels the § 1233 \
             short-sale rule."
                .to_string(),
        );
        notes.push(
            "Holding period is IGNORED for short-position SFCs — a 10-year-held short SFC still \
             produces short-term capital gain/loss on close."
                .to_string(),
        );
        return Section1234BResult {
            character: GainCharacter::ShortTermCapital,
            long_term_cents: 0,
            short_term_cents: g,
            ordinary_cents: 0,
            citation: "26 U.S.C. § 1234B(a) (character mirrors capital underlying); § 1234B(b) \
                       (short-term capital for SFC to sell regardless of holding period)",
            notes,
        };
    }

    // § 1234B(a) — long SFC with capital underlying. Apply § 1222
    // holding-period rule: > 365 days = long-term, else short-term.
    let long_term_hold = input.holding_period_days > 365;
    notes.push(
        "§ 1234B(a) — long securities futures contract with capital underlying; § 1222 \
         holding-period split applies (> 1 year = long-term per § 1222(3); else short-term per \
         § 1222(1))."
            .to_string(),
    );
    if long_term_hold {
        Section1234BResult {
            character: GainCharacter::LongTermCapital,
            long_term_cents: g,
            short_term_cents: 0,
            ordinary_cents: 0,
            citation: "26 U.S.C. § 1234B(a) (character mirrors capital underlying); § 1222(3) \
                       (long-term capital > 1 year holding period)",
            notes,
        }
    } else {
        Section1234BResult {
            character: GainCharacter::ShortTermCapital,
            long_term_cents: 0,
            short_term_cents: g,
            ordinary_cents: 0,
            citation: "26 U.S.C. § 1234B(a) (character mirrors capital underlying); § 1222(1) \
                       (short-term capital ≤ 1 year holding period)",
            notes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        g: i64,
        days: u32,
        pos: Position,
        cap_underlying: bool,
        dealer: bool,
    ) -> Section1234BInput {
        Section1234BInput {
            gain_or_loss_cents: g,
            holding_period_days: days,
            position: pos,
            underlying_is_capital_asset: cap_underlying,
            is_dealer_securities_futures_contract: dealer,
        }
    }

    // ── § 1234B(a) long SFC + capital underlying ─────────────────

    #[test]
    fn long_sfc_capital_underlying_held_400d_long_term() {
        let r = compute(&input(1_000_000, 400, Position::Long, true, false));
        assert_eq!(r.character, GainCharacter::LongTermCapital);
        assert_eq!(r.long_term_cents, 1_000_000);
        assert_eq!(r.short_term_cents, 0);
        assert_eq!(r.ordinary_cents, 0);
        assert!(r.citation.contains("§ 1234B(a)"));
        assert!(r.citation.contains("§ 1222(3)"));
    }

    #[test]
    fn long_sfc_capital_underlying_held_100d_short_term() {
        let r = compute(&input(1_000_000, 100, Position::Long, true, false));
        assert_eq!(r.character, GainCharacter::ShortTermCapital);
        assert_eq!(r.short_term_cents, 1_000_000);
        assert!(r.citation.contains("§ 1234B(a)"));
        assert!(r.citation.contains("§ 1222(1)"));
    }

    #[test]
    fn long_sfc_boundary_366d_long_term() {
        let r = compute(&input(1_000, 366, Position::Long, true, false));
        assert_eq!(r.character, GainCharacter::LongTermCapital);
    }

    #[test]
    fn long_sfc_boundary_365d_short_term() {
        // § 1222(3) requires MORE THAN 1 year. Exactly 365 = ST.
        let r = compute(&input(1_000, 365, Position::Long, true, false));
        assert_eq!(r.character, GainCharacter::ShortTermCapital);
    }

    #[test]
    fn long_sfc_loss_long_term() {
        let r = compute(&input(-500_000, 500, Position::Long, true, false));
        assert_eq!(r.character, GainCharacter::LongTermCapital);
        assert_eq!(r.long_term_cents, -500_000);
        assert_eq!(r.short_term_cents, 0);
    }

    // ── § 1234B(b) short SFC — ALWAYS short-term ─────────────────

    #[test]
    fn short_sfc_capital_underlying_held_5d_short_term() {
        let r = compute(&input(500_000, 5, Position::Short, true, false));
        assert_eq!(r.character, GainCharacter::ShortTermCapital);
        assert_eq!(r.short_term_cents, 500_000);
        assert!(r.citation.contains("§ 1234B(b)"));
    }

    #[test]
    fn short_sfc_capital_underlying_held_400d_still_short_term() {
        // The load-bearing § 1234B(b) rule — long hold does NOT make
        // a short SFC long-term.
        let r = compute(&input(500_000, 400, Position::Short, true, false));
        assert_eq!(r.character, GainCharacter::ShortTermCapital);
        assert_eq!(r.short_term_cents, 500_000);
        assert_eq!(r.long_term_cents, 0);
        assert!(r.citation.contains("§ 1234B(b)"));
    }

    #[test]
    fn short_sfc_held_3650_days_still_short_term_regression_critical() {
        // 10-year hold. § 1234B(b) overrides everything.
        let r = compute(&input(1_000_000, 3650, Position::Short, true, false));
        assert_eq!(r.character, GainCharacter::ShortTermCapital);
        assert_eq!(r.short_term_cents, 1_000_000);
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("Holding period is IGNORED")),
            "must flag short-position holding-period override"
        );
    }

    #[test]
    fn short_sfc_loss_short_term() {
        // Loss treated same way — ST capital.
        let r = compute(&input(-250_000, 200, Position::Short, true, false));
        assert_eq!(r.character, GainCharacter::ShortTermCapital);
        assert_eq!(r.short_term_cents, -250_000);
    }

    #[test]
    fn short_sfc_holding_period_invariance() {
        // Same gain across short hold and long hold MUST produce
        // identical output (§ 1234B(b) ignores days).
        let short_hold = compute(&input(10_000, 5, Position::Short, true, false));
        let long_hold = compute(&input(10_000, 5000, Position::Short, true, false));
        assert_eq!(short_hold.character, long_hold.character);
        assert_eq!(short_hold.long_term_cents, long_hold.long_term_cents);
        assert_eq!(short_hold.short_term_cents, long_hold.short_term_cents);
    }

    // ── § 1234B(a) ordinary underlying ─────────────────────────

    #[test]
    fn sfc_ordinary_underlying_long_position_ordinary_character() {
        // Dealer in securities holding ordinary inventory underlying.
        let r = compute(&input(100_000, 400, Position::Long, false, false));
        assert_eq!(r.character, GainCharacter::Ordinary);
        assert_eq!(r.ordinary_cents, 100_000);
        assert_eq!(r.long_term_cents, 0);
        assert_eq!(r.short_term_cents, 0);
        assert!(r.citation.contains("§ 1234B(a)"));
    }

    #[test]
    fn sfc_ordinary_underlying_short_position_still_ordinary() {
        // § 1234B(b) ONLY applies if gain/loss is CAPITAL. Ordinary
        // underlying takes ordinary character via § 1234B(a) and the
        // § 1234B(b) short-term-regardless rule never engages.
        let r = compute(&input(100_000, 50, Position::Short, false, false));
        assert_eq!(r.character, GainCharacter::Ordinary);
        assert_eq!(r.ordinary_cents, 100_000);
    }

    #[test]
    fn sfc_ordinary_underlying_holding_period_irrelevant() {
        let short = compute(&input(50_000, 10, Position::Long, false, false));
        let long = compute(&input(50_000, 2000, Position::Long, false, false));
        assert_eq!(short.character, long.character);
        assert_eq!(short.character, GainCharacter::Ordinary);
    }

    // ── § 1256(b)(1)(E) dealer SFC override ────────────────────

    #[test]
    fn dealer_sfc_routes_to_section_1256_60_40_split() {
        let r = compute(&input(1_000, 100, Position::Long, true, true));
        assert_eq!(r.character, GainCharacter::Section1256_60_40);
        assert_eq!(r.long_term_cents, 600);
        assert_eq!(r.short_term_cents, 400);
        assert!(r.citation.contains("§ 1256(b)(1)(E)"));
        assert!(r.citation.contains("§ 1256(a)(3)"));
    }

    #[test]
    fn dealer_sfc_short_position_still_60_40_not_short_term() {
        // Dealer override fires BEFORE § 1234B(b). Short dealer SFC
        // gets 60/40, NOT pure short-term.
        let r = compute(&input(1_000, 100, Position::Short, true, true));
        assert_eq!(r.character, GainCharacter::Section1256_60_40);
        assert_eq!(r.long_term_cents, 600);
        assert_eq!(r.short_term_cents, 400);
    }

    #[test]
    fn dealer_sfc_loss_60_40_split() {
        let r = compute(&input(-1_000, 100, Position::Long, true, true));
        assert_eq!(r.character, GainCharacter::Section1256_60_40);
        assert_eq!(r.long_term_cents, -600);
        assert_eq!(r.short_term_cents, -400);
    }

    #[test]
    fn dealer_sfc_holding_period_ignored_invariant() {
        let day_5 = compute(&input(10_000, 5, Position::Long, true, true));
        let day_2000 = compute(&input(10_000, 2000, Position::Long, true, true));
        assert_eq!(day_5.long_term_cents, day_2000.long_term_cents);
        assert_eq!(day_5.short_term_cents, day_2000.short_term_cents);
    }

    #[test]
    fn dealer_sfc_60_40_rounding_preserves_total() {
        // 1001 cents × 60% = 600 LT (truncated); 1001 − 600 = 401 ST.
        let r = compute(&input(1_001, 50, Position::Long, true, true));
        assert_eq!(r.long_term_cents, 600);
        assert_eq!(r.short_term_cents, 401);
        assert_eq!(r.long_term_cents + r.short_term_cents, 1_001);
    }

    // ── Regression-critical multi-path invariants ──────────────

    #[test]
    fn dealer_override_fires_before_short_position_short_term_rule() {
        // Non-dealer short SFC + capital underlying → ST capital.
        // Dealer short SFC + capital underlying → 60/40 (§ 1256 wins).
        let non_dealer = compute(&input(10_000, 100, Position::Short, true, false));
        let dealer = compute(&input(10_000, 100, Position::Short, true, true));
        assert_eq!(non_dealer.character, GainCharacter::ShortTermCapital);
        assert_eq!(dealer.character, GainCharacter::Section1256_60_40);
    }

    #[test]
    fn dealer_override_fires_before_ordinary_underlying_rule() {
        // Non-dealer + ordinary underlying → Ordinary (§ 1234B(a)).
        // Dealer + ordinary underlying → 60/40 (§ 1256 wins).
        let non_dealer = compute(&input(10_000, 100, Position::Long, false, false));
        let dealer = compute(&input(10_000, 100, Position::Long, false, true));
        assert_eq!(non_dealer.character, GainCharacter::Ordinary);
        assert_eq!(dealer.character, GainCharacter::Section1256_60_40);
    }

    #[test]
    fn only_short_position_capital_underlying_triggers_1234b_b_invariant() {
        // Among (long-cap, short-cap, long-ord, short-ord) ONLY the
        // SHORT+CAPITAL combo invokes § 1234B(b)'s short-term-always
        // rule. The other three follow their own paths.
        let long_cap_400 = compute(&input(1_000, 400, Position::Long, true, false));
        let short_cap_400 = compute(&input(1_000, 400, Position::Short, true, false));
        let long_ord_400 = compute(&input(1_000, 400, Position::Long, false, false));
        let short_ord_400 = compute(&input(1_000, 400, Position::Short, false, false));

        assert_eq!(long_cap_400.character, GainCharacter::LongTermCapital);
        assert_eq!(short_cap_400.character, GainCharacter::ShortTermCapital);
        assert_eq!(long_ord_400.character, GainCharacter::Ordinary);
        assert_eq!(short_ord_400.character, GainCharacter::Ordinary);
    }

    #[test]
    fn citation_pins_subsection_per_path() {
        let long_cap_lt = compute(&input(1, 400, Position::Long, true, false));
        let long_cap_st = compute(&input(1, 100, Position::Long, true, false));
        let short_cap = compute(&input(1, 400, Position::Short, true, false));
        let ord = compute(&input(1, 100, Position::Long, false, false));
        let dealer = compute(&input(1, 100, Position::Long, true, true));

        assert!(long_cap_lt.citation.contains("§ 1234B(a)"));
        assert!(long_cap_lt.citation.contains("§ 1222(3)"));
        assert!(long_cap_st.citation.contains("§ 1234B(a)"));
        assert!(long_cap_st.citation.contains("§ 1222(1)"));
        assert!(short_cap.citation.contains("§ 1234B(b)"));
        assert!(short_cap.citation.contains("§ 1234B(a)"));
        assert!(ord.citation.contains("§ 1234B(a)"));
        assert!(dealer.citation.contains("§ 1256(b)(1)(E)"));
        assert!(dealer.citation.contains("§ 1256(a)(3)"));
    }

    #[test]
    fn dealer_60_40_parts_sum_to_gain_property() {
        for g in [-1_000_000, -1, 0, 1, 7, 100, 999, 1_000_000] {
            let r = compute(&input(g, 100, Position::Long, true, true));
            assert_eq!(
                r.long_term_cents + r.short_term_cents,
                g,
                "60/40 parts must sum exactly to gain for g = {g}",
            );
            assert_eq!(r.ordinary_cents, 0);
        }
    }

    #[test]
    fn exactly_one_of_long_short_ordinary_60_40_holds_per_result() {
        // Mutually exclusive bucket invariant — exactly one of
        // long_term_cents, short_term_cents, ordinary_cents holds
        // the gain (except dealer SFC which splits across LT + ST).
        let long_cap = compute(&input(100, 400, Position::Long, true, false));
        assert_eq!(long_cap.long_term_cents, 100);
        assert_eq!(long_cap.short_term_cents, 0);
        assert_eq!(long_cap.ordinary_cents, 0);

        let short_cap = compute(&input(100, 400, Position::Short, true, false));
        assert_eq!(short_cap.long_term_cents, 0);
        assert_eq!(short_cap.short_term_cents, 100);
        assert_eq!(short_cap.ordinary_cents, 0);

        let ord = compute(&input(100, 400, Position::Long, false, false));
        assert_eq!(ord.long_term_cents, 0);
        assert_eq!(ord.short_term_cents, 0);
        assert_eq!(ord.ordinary_cents, 100);

        // Dealer: split LT + ST, no ordinary.
        let dealer = compute(&input(100, 400, Position::Long, true, true));
        assert_eq!(dealer.long_term_cents, 60);
        assert_eq!(dealer.short_term_cents, 40);
        assert_eq!(dealer.ordinary_cents, 0);
    }

    #[test]
    fn note_for_short_position_documents_1233_parallel() {
        let r = compute(&input(1_000, 100, Position::Short, true, false));
        assert!(
            r.notes.iter().any(|n| n.contains("§ 1233")),
            "short-position note should reference parallel § 1233 short-sale rule"
        );
    }

    #[test]
    fn note_for_dealer_documents_section_1256_routing() {
        let r = compute(&input(1_000, 100, Position::Long, true, true));
        assert!(
            r.notes.iter().any(|n| n.contains("60/40")),
            "dealer SFC note should describe 60/40 split"
        );
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("§ 1256(b)(1)(E)") || n.contains("§ 1256(a)(3)")),
            "dealer SFC note should cite § 1256 routing"
        );
    }

    #[test]
    fn note_for_ordinary_underlying_mirrors_character() {
        let r = compute(&input(1_000, 100, Position::Long, false, false));
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("mirrors underlying") && n.contains("ordinary")),
            "ordinary-underlying note must describe character mirror"
        );
    }
}
