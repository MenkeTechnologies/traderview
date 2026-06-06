//! IRC § 1234A — Gain or loss from certain terminations.
//!
//! § 1234A treats gain or loss attributable to the cancellation,
//! lapse, expiration, or other termination of:
//!   (1) a right or obligation (other than a securities futures
//!       contract or option to acquire or sell a securities futures
//!       contract) with respect to property which is (or on
//!       acquisition would be) a CAPITAL ASSET in the hands of the
//!       taxpayer; OR
//!   (2) a § 1256 contract (as defined in § 1256) not described in
//!       § 1234A(1),
//! as GAIN OR LOSS FROM THE SALE OF A CAPITAL ASSET.
//!
//! Trader-critical because it governs the character of:
//!   - Cash-settled options that expire worthless.
//!   - Cancelled forward contracts.
//!   - Cash-settled swaps and notional principal contracts.
//!   - Abandoned options to acquire real or personal capital
//!     property (the "real-property-option-trap" — abandonment
//!     produces CAPITAL loss, NOT ordinary § 165 loss).
//!   - Terminated § 1256 contracts (§ 1234A(2) routes character to
//!     § 1256's 60/40 split — overriding the right's holding period).
//!
//! Two scope exclusions:
//!   - § 1234A applies ONLY where the underlying property is (or
//!     would on acquisition be) a CAPITAL ASSET. Rights or
//!     obligations on ordinary property (inventory under § 1221(a)(1),
//!     § 1231 trade-or-business depreciable property, dealer
//!     property) are OUTSIDE § 1234A.
//!   - Securities futures contracts (and options on them) are
//!     excluded from § 1234A(1) and routed to § 1234B instead.
//!
//! Holding-period rule (where § 1234A(1) governs): apply § 1222 to
//! the holding period of the RIGHT or OBLIGATION (NOT the underlying
//! property). Long-term per § 1222(3) requires holding period of
//! MORE THAN one year (> 365 days); else short-term per § 1222(1).
//!
//! § 1256 redirect (where § 1234A(2) governs): § 1256(a)(3) imposes
//! 60% long-term / 40% short-term split regardless of holding
//! period. The right's holding period is IGNORED.
//!
//! Citations: 26 U.S.C. § 1234A(1) (capital-asset-right termination);
//! § 1234A(2) (§ 1256-contract termination); § 1234B (securities
//! futures contracts — out of § 1234A scope); § 1256(a)(3) (60/40
//! split); § 1221 (capital asset definition); § 1222(1) (short-term);
//! § 1222(3) (long-term > 1 year).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UnderlyingCharacter {
    /// Property is (or would be on acquisition) a § 1221 capital
    /// asset in the taxpayer's hands.
    Capital,
    /// Property is ordinary — § 1221(a)(1) inventory, § 1221(a)(2)
    /// trade-or-business depreciable property under § 1231, dealer
    /// property, etc. § 1234A does NOT apply.
    Ordinary,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ContractType {
    /// Right or obligation (option, forward, swap, notional principal
    /// contract, etc.) that is NOT a § 1256 contract and NOT a
    /// securities futures contract. § 1234A(1) governs.
    NonSection1256NonSecuritiesFutures,
    /// § 1256 contract (regulated futures, foreign currency contract,
    /// non-equity option, dealer equity option, dealer securities
    /// futures contract). § 1234A(2) routes to § 1256 60/40 split.
    Section1256,
    /// Securities futures contract (single-stock or narrow-based
    /// security index future). § 1234A excludes; § 1234B governs.
    SecuritiesFuturesContract,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GainCharacter {
    /// § 1234A(1) capital-asset right held > 365 days at termination.
    LongTermCapital,
    /// § 1234A(1) capital-asset right held ≤ 365 days at termination.
    ShortTermCapital,
    /// § 1234A(2) — § 1256 contract terminated. § 1256(a)(3) 60/40
    /// split applies; holding period is irrelevant.
    Section1256_60_40,
    /// § 1234A does not reach this termination because the underlying
    /// property is ordinary. Character determined under other Code
    /// provisions (e.g., § 165 ordinary loss).
    OrdinaryNotInScope,
    /// Securities futures contract — § 1234A excludes; § 1234B
    /// governs the character.
    SecuritiesFuturesContract,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section1234AInput {
    /// Gain (positive) or loss (negative) recognized on the
    /// termination, in cents.
    pub gain_or_loss_cents: i64,
    /// Holding period of the RIGHT or OBLIGATION (not the underlying
    /// property), in days. Used only when § 1234A(1) governs.
    pub holding_period_days: u32,
    /// Character of the property the right or obligation relates to.
    pub underlying: UnderlyingCharacter,
    /// Classification of the contract terminated.
    pub contract_type: ContractType,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section1234AResult {
    /// True iff § 1234A reaches this termination (capital-asset
    /// underlying AND not a § 1234B securities futures contract).
    pub section_1234a_applies: bool,
    pub character: GainCharacter,
    /// Long-term portion of gain/loss, in cents. Equal to full
    /// gain/loss for long-term capital; zero for short-term capital;
    /// 60% (rounded toward zero) for § 1256 60/40; zero otherwise.
    pub long_term_cents: i64,
    /// Short-term portion of gain/loss, in cents. Equal to full
    /// gain/loss for short-term capital; zero for long-term capital;
    /// remainder (gain_or_loss − long_term) for § 1256 60/40; zero
    /// otherwise.
    pub short_term_cents: i64,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn compute(input: &Section1234AInput) -> Section1234AResult {
    let g = input.gain_or_loss_cents;
    let mut notes: Vec<String> = Vec::new();

    // Securities futures contract — § 1234A(1) explicit exclusion;
    // § 1234B governs character. § 1234A does NOT apply.
    if matches!(input.contract_type, ContractType::SecuritiesFuturesContract) {
        notes.push(
            "Securities futures contract — § 1234A(1) excludes; § 1234B governs character of \
             gain or loss on sale, exchange, or termination."
                .to_string(),
        );
        return Section1234AResult {
            section_1234a_applies: false,
            character: GainCharacter::SecuritiesFuturesContract,
            long_term_cents: 0,
            short_term_cents: 0,
            citation: "26 U.S.C. § 1234A(1) (excludes securities futures contracts); § 1234B \
                       (governs sale/exchange/termination of securities futures contracts)",
            notes,
        };
    }

    // Ordinary underlying — § 1234A reaches only capital-asset rights.
    if matches!(input.underlying, UnderlyingCharacter::Ordinary) {
        notes.push(
            "Ordinary underlying property — § 1234A does not apply; character of gain or loss \
             determined under other Code provisions (e.g., § 165 ordinary loss, § 1231 \
             quasi-capital treatment, § 1221(a)(1) inventory sale)."
                .to_string(),
        );
        return Section1234AResult {
            section_1234a_applies: false,
            character: GainCharacter::OrdinaryNotInScope,
            long_term_cents: 0,
            short_term_cents: 0,
            citation: "26 U.S.C. § 1234A(1) (capital-asset-right requirement); § 1221 (capital \
                       asset definition)",
            notes,
        };
    }

    // § 1256 contract terminated — § 1234A(2) routes to § 1256
    // 60/40 split regardless of holding period.
    if matches!(input.contract_type, ContractType::Section1256) {
        // 60% long-term, 40% short-term. Round long_term toward zero
        // by integer division; short_term is the remainder so the
        // two parts always sum exactly to g.
        let long_term = g * 60 / 100;
        let short_term = g - long_term;
        notes.push(
            "§ 1234A(2) routes character to § 1256 contract — § 1256(a)(3) 60/40 split: 60% \
             long-term + 40% short-term regardless of holding period."
                .to_string(),
        );
        notes.push(
            "Holding period of the right is IGNORED for § 1256 contracts — split is automatic."
                .to_string(),
        );
        return Section1234AResult {
            section_1234a_applies: true,
            character: GainCharacter::Section1256_60_40,
            long_term_cents: long_term,
            short_term_cents: short_term,
            citation: "26 U.S.C. § 1234A(2) (termination of § 1256 contract); § 1256(a)(3) \
                       (60/40 split regardless of holding period)",
            notes,
        };
    }

    // § 1234A(1) capital-asset right termination — apply § 1222
    // holding-period rule to the RIGHT (not underlying). Long-term
    // per § 1222(3) requires > 365 days.
    let long_term_hold = input.holding_period_days > 365;
    notes.push(
        "§ 1234A(1) applies — termination of capital-asset right treated as sale of capital \
         asset; holding period of the right (not underlying property) governs § 1222 character."
            .to_string(),
    );
    notes.push(
        "Real-property-option-trap: abandonment of an option to acquire capital property \
         (residence, vacation home, raw land) yields a CAPITAL loss under § 1234A, not an \
         ordinary § 165 loss."
            .to_string(),
    );
    if long_term_hold {
        Section1234AResult {
            section_1234a_applies: true,
            character: GainCharacter::LongTermCapital,
            long_term_cents: g,
            short_term_cents: 0,
            citation: "26 U.S.C. § 1234A(1) (capital-asset-right termination treated as sale of \
                       capital asset); § 1222(3) (long-term requires > 1 year)",
            notes,
        }
    } else {
        Section1234AResult {
            section_1234a_applies: true,
            character: GainCharacter::ShortTermCapital,
            long_term_cents: 0,
            short_term_cents: g,
            citation: "26 U.S.C. § 1234A(1) (capital-asset-right termination treated as sale of \
                       capital asset); § 1222(1) (short-term ≤ 1 year)",
            notes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(g: i64, days: u32, u: UnderlyingCharacter, c: ContractType) -> Section1234AInput {
        Section1234AInput {
            gain_or_loss_cents: g,
            holding_period_days: days,
            underlying: u,
            contract_type: c,
        }
    }

    // ── § 1234A(1) capital-asset right — long-term path ─────────

    #[test]
    fn capital_asset_right_held_400_days_long_term_full_gain() {
        let r = compute(&input(
            500_000,
            400,
            UnderlyingCharacter::Capital,
            ContractType::NonSection1256NonSecuritiesFutures,
        ));
        assert!(r.section_1234a_applies);
        assert_eq!(r.character, GainCharacter::LongTermCapital);
        assert_eq!(r.long_term_cents, 500_000);
        assert_eq!(r.short_term_cents, 0);
        assert!(r.citation.contains("§ 1234A(1)"));
        assert!(r.citation.contains("§ 1222(3)"));
    }

    #[test]
    fn capital_asset_right_boundary_366_days_long_term() {
        let r = compute(&input(
            1_000,
            366,
            UnderlyingCharacter::Capital,
            ContractType::NonSection1256NonSecuritiesFutures,
        ));
        assert_eq!(r.character, GainCharacter::LongTermCapital);
        assert_eq!(r.long_term_cents, 1_000);
    }

    // ── § 1234A(1) capital-asset right — short-term path ────────

    #[test]
    fn capital_asset_right_held_100_days_short_term_full_gain() {
        let r = compute(&input(
            500_000,
            100,
            UnderlyingCharacter::Capital,
            ContractType::NonSection1256NonSecuritiesFutures,
        ));
        assert!(r.section_1234a_applies);
        assert_eq!(r.character, GainCharacter::ShortTermCapital);
        assert_eq!(r.long_term_cents, 0);
        assert_eq!(r.short_term_cents, 500_000);
        assert!(r.citation.contains("§ 1234A(1)"));
        assert!(r.citation.contains("§ 1222(1)"));
    }

    #[test]
    fn capital_asset_right_boundary_365_days_short_term() {
        // § 1222(3) requires MORE THAN 1 year. Exactly 365 = short.
        let r = compute(&input(
            1_000,
            365,
            UnderlyingCharacter::Capital,
            ContractType::NonSection1256NonSecuritiesFutures,
        ));
        assert_eq!(r.character, GainCharacter::ShortTermCapital);
        assert_eq!(r.short_term_cents, 1_000);
    }

    #[test]
    fn capital_asset_right_loss_short_term() {
        // Negative gain (loss) — capital LOSS, same character path.
        let r = compute(&input(
            -250_000,
            30,
            UnderlyingCharacter::Capital,
            ContractType::NonSection1256NonSecuritiesFutures,
        ));
        assert!(r.section_1234a_applies);
        assert_eq!(r.character, GainCharacter::ShortTermCapital);
        assert_eq!(r.short_term_cents, -250_000);
        assert_eq!(r.long_term_cents, 0);
    }

    #[test]
    fn real_property_option_abandoned_capital_loss_not_ordinary() {
        // The real-property-option-trap. Taxpayer abandons option to
        // buy raw land (capital asset). § 1234A converts what would
        // otherwise be a § 165 abandonment loss into a CAPITAL loss.
        let r = compute(&input(
            -50_000_00,
            120,
            UnderlyingCharacter::Capital,
            ContractType::NonSection1256NonSecuritiesFutures,
        ));
        assert!(r.section_1234a_applies);
        assert_eq!(r.character, GainCharacter::ShortTermCapital);
        assert!(
            r.notes
                .iter()
                .any(|n| n.to_lowercase().contains("real-property-option-trap")),
            "must flag the § 1234A real-property-option-trap"
        );
    }

    // ── § 1234A(2) § 1256 contract — 60/40 split ────────────────

    #[test]
    fn section_1256_contract_terminated_60_40_split_basic() {
        let r = compute(&input(
            1_000,
            200,
            UnderlyingCharacter::Capital,
            ContractType::Section1256,
        ));
        assert!(r.section_1234a_applies);
        assert_eq!(r.character, GainCharacter::Section1256_60_40);
        assert_eq!(r.long_term_cents, 600);
        assert_eq!(r.short_term_cents, 400);
        assert!(r.citation.contains("§ 1234A(2)"));
        assert!(r.citation.contains("§ 1256(a)(3)"));
    }

    #[test]
    fn section_1256_contract_short_hold_5_days_still_60_40() {
        // § 1256(a)(3) overrides holding period — 5 days same split
        // as 5 years.
        let r = compute(&input(
            1_000,
            5,
            UnderlyingCharacter::Capital,
            ContractType::Section1256,
        ));
        assert_eq!(r.character, GainCharacter::Section1256_60_40);
        assert_eq!(r.long_term_cents, 600);
        assert_eq!(r.short_term_cents, 400);
    }

    #[test]
    fn section_1256_contract_loss_60_40_split() {
        // Loss splits same way. -1000 → -600 LT / -400 ST.
        let r = compute(&input(
            -1_000,
            50,
            UnderlyingCharacter::Capital,
            ContractType::Section1256,
        ));
        assert_eq!(r.character, GainCharacter::Section1256_60_40);
        assert_eq!(r.long_term_cents, -600);
        assert_eq!(r.short_term_cents, -400);
        // Parts must always sum exactly to gain.
        assert_eq!(r.long_term_cents + r.short_term_cents, -1_000);
    }

    #[test]
    fn section_1256_contract_zero_gain_zero_parts() {
        let r = compute(&input(
            0,
            100,
            UnderlyingCharacter::Capital,
            ContractType::Section1256,
        ));
        assert_eq!(r.long_term_cents, 0);
        assert_eq!(r.short_term_cents, 0);
    }

    #[test]
    fn section_1256_split_rounding_preserves_total() {
        // Odd amount where 60% is not integer: 1001 cents.
        // 1001 * 60 / 100 = 60060 / 100 = 600 LT (truncated).
        // ST = 1001 - 600 = 401.
        let r = compute(&input(
            1_001,
            10,
            UnderlyingCharacter::Capital,
            ContractType::Section1256,
        ));
        assert_eq!(r.long_term_cents, 600);
        assert_eq!(r.short_term_cents, 401);
        assert_eq!(r.long_term_cents + r.short_term_cents, 1_001);
    }

    #[test]
    fn section_1256_holding_period_ignored_invariant() {
        // Holding period must NOT change the result for § 1256.
        let day_5 = compute(&input(
            10_000,
            5,
            UnderlyingCharacter::Capital,
            ContractType::Section1256,
        ));
        let day_2000 = compute(&input(
            10_000,
            2000,
            UnderlyingCharacter::Capital,
            ContractType::Section1256,
        ));
        assert_eq!(day_5.long_term_cents, day_2000.long_term_cents);
        assert_eq!(day_5.short_term_cents, day_2000.short_term_cents);
        assert_eq!(day_5.character, day_2000.character);
    }

    #[test]
    fn section_1256_note_explains_60_40_and_holding_period_ignored() {
        let r = compute(&input(
            100,
            50,
            UnderlyingCharacter::Capital,
            ContractType::Section1256,
        ));
        assert!(r.notes.iter().any(|n| n.contains("60/40")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.to_lowercase().contains("holding period")));
    }

    // ── Securities futures contract — § 1234B governs ──────────

    #[test]
    fn securities_futures_contract_excluded_from_section_1234a() {
        let r = compute(&input(
            500_000,
            200,
            UnderlyingCharacter::Capital,
            ContractType::SecuritiesFuturesContract,
        ));
        assert!(!r.section_1234a_applies);
        assert_eq!(r.character, GainCharacter::SecuritiesFuturesContract);
        assert_eq!(r.long_term_cents, 0);
        assert_eq!(r.short_term_cents, 0);
        assert!(r.citation.contains("§ 1234B"));
    }

    #[test]
    fn securities_futures_with_long_hold_still_routes_to_1234b() {
        // Hold > 365 days does NOT pull SFC back into § 1234A.
        let r = compute(&input(
            100_000,
            500,
            UnderlyingCharacter::Capital,
            ContractType::SecuritiesFuturesContract,
        ));
        assert!(!r.section_1234a_applies);
        assert_eq!(r.character, GainCharacter::SecuritiesFuturesContract);
    }

    // ── Ordinary underlying — § 1234A does NOT apply ───────────

    #[test]
    fn ordinary_underlying_section_1234a_does_not_apply() {
        // Dealer holding inventory option — underlying is § 1221(a)(1)
        // inventory, NOT a capital asset. § 1234A does not reach.
        let r = compute(&input(
            100_000,
            400,
            UnderlyingCharacter::Ordinary,
            ContractType::NonSection1256NonSecuritiesFutures,
        ));
        assert!(!r.section_1234a_applies);
        assert_eq!(r.character, GainCharacter::OrdinaryNotInScope);
        assert_eq!(r.long_term_cents, 0);
        assert_eq!(r.short_term_cents, 0);
        assert!(r.citation.contains("§ 1221"));
    }

    #[test]
    fn ordinary_underlying_regardless_of_holding_period() {
        // Same gain, two holding periods, ordinary underlying → both
        // produce OrdinaryNotInScope. Holding period is irrelevant.
        let short = compute(&input(
            100_000,
            10,
            UnderlyingCharacter::Ordinary,
            ContractType::NonSection1256NonSecuritiesFutures,
        ));
        let long = compute(&input(
            100_000,
            2000,
            UnderlyingCharacter::Ordinary,
            ContractType::NonSection1256NonSecuritiesFutures,
        ));
        assert_eq!(short.character, long.character);
        assert!(!short.section_1234a_applies);
        assert!(!long.section_1234a_applies);
    }

    // ── Regression-critical invariants ──────────────────────────

    #[test]
    fn only_section_1256_routes_to_60_40_invariant() {
        // Across the three contract types, ONLY § 1256 yields the
        // 60/40 character. Hold = 100 days for all; capital underlying.
        let non_1256 = compute(&input(
            10_000,
            100,
            UnderlyingCharacter::Capital,
            ContractType::NonSection1256NonSecuritiesFutures,
        ));
        let s1256 = compute(&input(
            10_000,
            100,
            UnderlyingCharacter::Capital,
            ContractType::Section1256,
        ));
        let sfc = compute(&input(
            10_000,
            100,
            UnderlyingCharacter::Capital,
            ContractType::SecuritiesFuturesContract,
        ));
        assert_eq!(s1256.character, GainCharacter::Section1256_60_40);
        assert_ne!(non_1256.character, GainCharacter::Section1256_60_40);
        assert_ne!(sfc.character, GainCharacter::Section1256_60_40);
    }

    #[test]
    fn only_securities_futures_excluded_by_1234b_invariant() {
        // ONLY SFC produces SecuritiesFuturesContract character.
        let non_1256 = compute(&input(
            10_000,
            100,
            UnderlyingCharacter::Capital,
            ContractType::NonSection1256NonSecuritiesFutures,
        ));
        let s1256 = compute(&input(
            10_000,
            100,
            UnderlyingCharacter::Capital,
            ContractType::Section1256,
        ));
        let sfc = compute(&input(
            10_000,
            100,
            UnderlyingCharacter::Capital,
            ContractType::SecuritiesFuturesContract,
        ));
        assert_eq!(sfc.character, GainCharacter::SecuritiesFuturesContract);
        assert_ne!(non_1256.character, GainCharacter::SecuritiesFuturesContract);
        assert_ne!(s1256.character, GainCharacter::SecuritiesFuturesContract);
    }

    #[test]
    fn only_ordinary_underlying_skips_section_1234a_invariant() {
        // ONLY ordinary underlying produces applies=false (apart from
        // the SFC § 1234B exclusion, which has its own character).
        let capital = compute(&input(
            10_000,
            100,
            UnderlyingCharacter::Capital,
            ContractType::NonSection1256NonSecuritiesFutures,
        ));
        let ordinary = compute(&input(
            10_000,
            100,
            UnderlyingCharacter::Ordinary,
            ContractType::NonSection1256NonSecuritiesFutures,
        ));
        assert!(capital.section_1234a_applies);
        assert!(!ordinary.section_1234a_applies);
        assert_eq!(ordinary.character, GainCharacter::OrdinaryNotInScope);
    }

    #[test]
    fn only_non_1256_right_uses_holding_period_invariant() {
        // For the ONLY contract path that honors holding period
        // (§ 1234A(1) capital-asset right), 100d → short, 400d → long.
        let short = compute(&input(
            10_000,
            100,
            UnderlyingCharacter::Capital,
            ContractType::NonSection1256NonSecuritiesFutures,
        ));
        let long = compute(&input(
            10_000,
            400,
            UnderlyingCharacter::Capital,
            ContractType::NonSection1256NonSecuritiesFutures,
        ));
        assert_eq!(short.character, GainCharacter::ShortTermCapital);
        assert_eq!(long.character, GainCharacter::LongTermCapital);
        // Same holding-period swap on § 1256 does NOT change character.
        let short_1256 = compute(&input(
            10_000,
            100,
            UnderlyingCharacter::Capital,
            ContractType::Section1256,
        ));
        let long_1256 = compute(&input(
            10_000,
            400,
            UnderlyingCharacter::Capital,
            ContractType::Section1256,
        ));
        assert_eq!(short_1256.character, long_1256.character);
    }

    #[test]
    fn citation_pins_subsection_per_path() {
        // Each path must pin its own statutory authority.
        let lt = compute(&input(
            1,
            400,
            UnderlyingCharacter::Capital,
            ContractType::NonSection1256NonSecuritiesFutures,
        ));
        let st = compute(&input(
            1,
            100,
            UnderlyingCharacter::Capital,
            ContractType::NonSection1256NonSecuritiesFutures,
        ));
        let s1256 = compute(&input(
            1,
            100,
            UnderlyingCharacter::Capital,
            ContractType::Section1256,
        ));
        let sfc = compute(&input(
            1,
            100,
            UnderlyingCharacter::Capital,
            ContractType::SecuritiesFuturesContract,
        ));
        let ord = compute(&input(
            1,
            100,
            UnderlyingCharacter::Ordinary,
            ContractType::NonSection1256NonSecuritiesFutures,
        ));
        assert!(lt.citation.contains("§ 1234A(1)"));
        assert!(lt.citation.contains("§ 1222(3)"));
        assert!(st.citation.contains("§ 1234A(1)"));
        assert!(st.citation.contains("§ 1222(1)"));
        assert!(s1256.citation.contains("§ 1234A(2)"));
        assert!(s1256.citation.contains("§ 1256(a)(3)"));
        assert!(sfc.citation.contains("§ 1234B"));
        assert!(ord.citation.contains("§ 1221"));
    }

    #[test]
    fn section_1256_parts_always_sum_to_gain_property() {
        for g in [-1_000_000, -1, 0, 1, 7, 100, 999, 1_000_000] {
            let r = compute(&input(
                g,
                100,
                UnderlyingCharacter::Capital,
                ContractType::Section1256,
            ));
            assert_eq!(
                r.long_term_cents + r.short_term_cents,
                g,
                "60/40 parts must sum exactly to gain for g = {g}",
            );
        }
    }

    #[test]
    fn capital_asset_right_zero_days_short_term() {
        // Zero-day hold (same-day cancellation) — short-term.
        let r = compute(&input(
            5_000,
            0,
            UnderlyingCharacter::Capital,
            ContractType::NonSection1256NonSecuritiesFutures,
        ));
        assert_eq!(r.character, GainCharacter::ShortTermCapital);
        assert_eq!(r.short_term_cents, 5_000);
    }

    #[test]
    fn capital_asset_right_long_term_loss_full_amount_long() {
        // Loss on long-held right — full amount in long_term_cents.
        let r = compute(&input(
            -8_000,
            500,
            UnderlyingCharacter::Capital,
            ContractType::NonSection1256NonSecuritiesFutures,
        ));
        assert_eq!(r.character, GainCharacter::LongTermCapital);
        assert_eq!(r.long_term_cents, -8_000);
        assert_eq!(r.short_term_cents, 0);
    }
}
