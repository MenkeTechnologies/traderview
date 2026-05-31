//! IRC §1234 — Options to buy or sell.
//!
//! Character and holding-period rules for stock/securities options that
//! are NOT §1256 contracts (broad-based index options, futures options,
//! foreign currency options, etc., which mark to market 60/40 under §1256).
//!
//! Three subsections matter here:
//!
//! **§1234(a) — Holder's character mirrors the underlying.**  Gain or loss
//! on the sale, exchange, or lapse of an option to buy or sell property is
//! gain or loss of the SAME CHARACTER as the underlying property would have
//! had in the holder's hands. Plus the option's own holding period
//! determines short-vs-long-term (≤ 365 days = short-term per §1222
//! standard). If the option is exercised, there is no separate gain/loss
//! event under §1234(a)(4) — the premium adjusts basis of the acquired
//! (call) or disposed (put) underlying.
//!
//! **§1234(b) — Writer's gain/loss is ALWAYS short-term.**  Gain or loss
//! from a closing transaction on a written option, or gain on its lapse,
//! is treated as gain or loss from the sale of a capital asset HELD FOR
//! NOT MORE THAN 1 YEAR — regardless of how long the writer was on the
//! short side. This is the "premium is always short-term to the writer"
//! rule that defines covered-call and cash-secured-put taxation. Exception
//! under §1234(b)(2)(A): dealers in options get ordinary treatment.
//!
//! **§1234(c) — §1256 override.**  §1234 does not apply to §1256 contracts.
//! For those, fall through to `section_1256.rs` (already implemented).
//!
//! **§1234(a)(3) — ordinary underlying.**  If the underlying property
//! would have been ordinary (§1221 inventory etc.) in the holder's hands,
//! the holder's option result is ordinary character regardless of holding
//! period.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Whether the taxpayer is the holder (long the option) or the writer
/// (short the option / grantor).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OptionRole {
    Holder,
    Writer,
}

/// How the option was terminated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CloseType {
    /// Sold to a third party on the open market.
    Sold,
    /// Expired worthless / not exercised.
    Lapsed,
    /// Writer's offsetting purchase to cancel the obligation.
    BoughtBack,
    /// Holder exercised. NO realized event under §1234(a)(4); premium
    /// adjusts basis of the underlying.
    Exercised,
    /// Writer was assigned (counterparty exercised). NO realized event;
    /// premium adjusts proceeds on the writer's sale of underlying.
    Assigned,
}

/// Character of the underlying property as it would be in the holder's
/// hands. Drives §1234(a) for the holder; §1234(b) is fixed-character
/// regardless of underlying.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnderlyingCharacter {
    /// §1221 capital asset (most stock/ETF options).
    Capital,
    /// §1221(a)(1) inventory or §1221(a)(2) trade or business property
    /// — option result is ordinary character per §1234(a)(3).
    Ordinary,
    /// §1256 contract — §1234 doesn't apply; route to `section_1256.rs`.
    Section1256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section1234Input {
    pub role: OptionRole,
    pub close_type: CloseType,
    pub option_open_date: NaiveDate,
    pub option_close_date: NaiveDate,
    /// For holder: premium paid (positive cost basis).
    /// For writer: premium received (positive proceeds).
    pub premium: Decimal,
    /// For holder Sold: sale proceeds. For writer BoughtBack: buyback cost.
    /// For Lapsed: ignored (worthless, treated as 0).
    /// For Exercised / Assigned: ignored (no realized event).
    pub close_proceeds_or_cost: Decimal,
    pub underlying_character: UnderlyingCharacter,
    /// §1234(b)(2)(A) carve-out: dealers in options get ordinary
    /// treatment, not the fixed short-term capital character.
    pub is_dealer_in_options: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaxCharacter {
    ShortTermCapital,
    LongTermCapital,
    Ordinary,
    Section1256, // 60/40 — caller routes to section_1256.rs
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Section1234Rule {
    /// §1234(a) — holder, capital underlying, character driven by option
    /// holding period.
    HolderCapital,
    /// §1234(a)(3) — holder, ordinary underlying, ordinary character.
    HolderOrdinary,
    /// §1234(a)(4) — holder exercise. No realized event; basis adjustment
    /// only.
    HolderExercise,
    /// §1234(b)(1) — writer, capital short-term regardless of holding
    /// period.
    WriterShortTerm,
    /// §1234(b)(2)(A) — writer who is a dealer in options. Ordinary
    /// character.
    WriterDealerOrdinary,
    /// §1234(b) doesn't reach the writer's assignment either — the
    /// premium becomes a price adjustment on the writer's sale of
    /// underlying. No realized event on the option itself.
    WriterAssignment,
    /// §1256 contract — §1234 does not apply.
    Section1256Override,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section1234Result {
    pub character: TaxCharacter,
    pub rule: Section1234Rule,
    pub gain_loss: Decimal,
    /// True when the close type is Exercised or Assigned — premium folds
    /// into basis/proceeds of the underlying and there is no separately
    /// reportable gain/loss on the option itself.
    pub is_basis_adjustment_event: bool,
    pub note: String,
}

/// Holding-period boundary used by §1234(a) — matches §1222 (≤ 365 days
/// is short-term; > 365 days is long-term).
const ONE_YEAR_DAYS: i64 = 365;

pub fn compute(input: &Section1234Input) -> Section1234Result {
    // §1234(c) — §1256 contracts bypass §1234 entirely.
    if matches!(input.underlying_character, UnderlyingCharacter::Section1256) {
        return Section1234Result {
            character: TaxCharacter::Section1256,
            rule: Section1234Rule::Section1256Override,
            gain_loss: Decimal::ZERO,
            is_basis_adjustment_event: false,
            note: "§1256 contract — §1234 does not apply; route to section_1256 60/40 MTM rules"
                .into(),
        };
    }

    // Basis-adjustment events: option doesn't produce a separate gain/loss.
    match (input.role, input.close_type) {
        (OptionRole::Holder, CloseType::Exercised) => {
            return Section1234Result {
                character: TaxCharacter::ShortTermCapital, // placeholder, see flag
                rule: Section1234Rule::HolderExercise,
                gain_loss: Decimal::ZERO,
                is_basis_adjustment_event: true,
                note: "§1234(a)(4) — holder exercised; premium adjusts basis of underlying, no separate option event"
                    .into(),
            };
        }
        (OptionRole::Writer, CloseType::Assigned) => {
            return Section1234Result {
                character: TaxCharacter::ShortTermCapital, // placeholder
                rule: Section1234Rule::WriterAssignment,
                gain_loss: Decimal::ZERO,
                is_basis_adjustment_event: true,
                note: "writer assigned; premium adjusts proceeds on sale of underlying, no separate option event"
                    .into(),
            };
        }
        _ => {}
    }

    // Compute the gain/loss. For holder: proceeds - premium. For writer:
    // premium - buyback (or premium - 0 on lapse).
    let gain_loss = match input.role {
        OptionRole::Holder => match input.close_type {
            CloseType::Sold => input.close_proceeds_or_cost - input.premium,
            CloseType::Lapsed => -input.premium, // total loss of premium
            _ => Decimal::ZERO,
        },
        OptionRole::Writer => match input.close_type {
            CloseType::Sold => input.close_proceeds_or_cost - input.premium,
            CloseType::Lapsed => input.premium, // full premium retained as gain
            CloseType::BoughtBack => input.premium - input.close_proceeds_or_cost,
            _ => Decimal::ZERO,
        },
    };

    // Writer path — §1234(b).
    if matches!(input.role, OptionRole::Writer) {
        if input.is_dealer_in_options {
            return Section1234Result {
                character: TaxCharacter::Ordinary,
                rule: Section1234Rule::WriterDealerOrdinary,
                gain_loss,
                is_basis_adjustment_event: false,
                note: format!(
                    "§1234(b)(2)(A) — writer is a dealer in options; gain/loss of ${} is ordinary",
                    gain_loss.round_dp(2)
                ),
            };
        }
        return Section1234Result {
            character: TaxCharacter::ShortTermCapital,
            rule: Section1234Rule::WriterShortTerm,
            gain_loss,
            is_basis_adjustment_event: false,
            note: format!(
                "§1234(b)(1) — writer's gain/loss of ${} is short-term capital regardless of option holding period",
                gain_loss.round_dp(2)
            ),
        };
    }

    // Holder path — §1234(a). Character mirrors the underlying.
    if matches!(input.underlying_character, UnderlyingCharacter::Ordinary) {
        return Section1234Result {
            character: TaxCharacter::Ordinary,
            rule: Section1234Rule::HolderOrdinary,
            gain_loss,
            is_basis_adjustment_event: false,
            note: format!(
                "§1234(a)(3) — holder's underlying is ordinary character; gain/loss of ${} is ordinary regardless of option holding period",
                gain_loss.round_dp(2)
            ),
        };
    }

    let days_held = (input.option_close_date - input.option_open_date).num_days();
    let character = if days_held > ONE_YEAR_DAYS {
        TaxCharacter::LongTermCapital
    } else {
        TaxCharacter::ShortTermCapital
    };
    Section1234Result {
        character,
        rule: Section1234Rule::HolderCapital,
        gain_loss,
        is_basis_adjustment_event: false,
        note: format!(
            "§1234(a) — holder option held {}d; gain/loss of ${} is {} capital",
            days_held,
            gain_loss.round_dp(2),
            if days_held > ONE_YEAR_DAYS {
                "long-term"
            } else {
                "short-term"
            }
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    fn writer_input(close: CloseType, premium: Decimal, close_amt: Decimal) -> Section1234Input {
        Section1234Input {
            role: OptionRole::Writer,
            close_type: close,
            option_open_date: d(2026, 1, 1),
            option_close_date: d(2026, 6, 1),
            premium,
            close_proceeds_or_cost: close_amt,
            underlying_character: UnderlyingCharacter::Capital,
            is_dealer_in_options: false,
        }
    }

    fn holder_input(close: CloseType, premium: Decimal, close_amt: Decimal) -> Section1234Input {
        Section1234Input {
            role: OptionRole::Holder,
            close_type: close,
            option_open_date: d(2026, 1, 1),
            option_close_date: d(2026, 6, 1),
            premium,
            close_proceeds_or_cost: close_amt,
            underlying_character: UnderlyingCharacter::Capital,
            is_dealer_in_options: false,
        }
    }

    #[test]
    fn writer_lapsed_call_short_term_gain_equals_premium() {
        // Sold-to-open a call for $500 premium, expires worthless.
        // §1234(b)(1) — premium is short-term capital gain.
        let r = compute(&writer_input(CloseType::Lapsed, dec!(500), Decimal::ZERO));
        assert_eq!(r.character, TaxCharacter::ShortTermCapital);
        assert_eq!(r.rule, Section1234Rule::WriterShortTerm);
        assert_eq!(r.gain_loss, dec!(500));
        assert!(!r.is_basis_adjustment_event);
    }

    #[test]
    fn writer_bought_back_below_premium_short_term_gain() {
        // Wrote put for $800, bought back for $300 → $500 ST gain.
        let r = compute(&writer_input(CloseType::BoughtBack, dec!(800), dec!(300)));
        assert_eq!(r.character, TaxCharacter::ShortTermCapital);
        assert_eq!(r.gain_loss, dec!(500));
    }

    #[test]
    fn writer_bought_back_above_premium_short_term_loss() {
        // Wrote put for $300, position moved against → bought back for $700.
        // Loss of $400. Still ST per §1234(b)(1) (writer is ALWAYS ST).
        let r = compute(&writer_input(CloseType::BoughtBack, dec!(300), dec!(700)));
        assert_eq!(r.character, TaxCharacter::ShortTermCapital);
        assert_eq!(r.gain_loss, dec!(-400));
    }

    #[test]
    fn writer_held_over_one_year_still_short_term() {
        // §1234(b) is fixed-character. Even if the writer holds the
        // short side for 18 months, the close is ST. Pinned because
        // §1234(b)(1)'s "regardless of holding period" language is the
        // entire reason this rule exists — it's the bright-line override
        // against §1234(a)'s mirror-character behavior.
        let mut i = writer_input(CloseType::Lapsed, dec!(500), Decimal::ZERO);
        i.option_open_date = d(2024, 1, 1);
        i.option_close_date = d(2026, 1, 15); // 745 days
        let r = compute(&i);
        assert_eq!(r.character, TaxCharacter::ShortTermCapital);
        assert_eq!(r.rule, Section1234Rule::WriterShortTerm);
    }

    #[test]
    fn writer_assigned_is_basis_adjustment_event_no_gain_loss() {
        // Counterparty exercised → the premium folds into the writer's
        // proceeds on the underlying sale. There is no separately
        // reportable option event. is_basis_adjustment_event = true.
        let r = compute(&writer_input(CloseType::Assigned, dec!(500), Decimal::ZERO));
        assert!(r.is_basis_adjustment_event);
        assert_eq!(r.gain_loss, Decimal::ZERO);
        assert_eq!(r.rule, Section1234Rule::WriterAssignment);
    }

    #[test]
    fn writer_dealer_in_options_gets_ordinary_character() {
        // §1234(b)(2)(A) carve-out. Dealer's premium gain is ordinary,
        // not ST capital. Most retail traders are not dealers, but the
        // rule matters for futures-options market makers etc.
        let mut i = writer_input(CloseType::Lapsed, dec!(500), Decimal::ZERO);
        i.is_dealer_in_options = true;
        let r = compute(&i);
        assert_eq!(r.character, TaxCharacter::Ordinary);
        assert_eq!(r.rule, Section1234Rule::WriterDealerOrdinary);
    }

    #[test]
    fn holder_sold_short_term_capital_gain() {
        // Bought call for $200, sold for $350 after 6 months. Holding
        // period ≤ 365 days → ST capital gain of $150.
        let r = compute(&holder_input(CloseType::Sold, dec!(200), dec!(350)));
        assert_eq!(r.character, TaxCharacter::ShortTermCapital);
        assert_eq!(r.rule, Section1234Rule::HolderCapital);
        assert_eq!(r.gain_loss, dec!(150));
    }

    #[test]
    fn holder_sold_long_term_capital_gain() {
        // LEAP held > 1 year, sold at gain. §1234(a) → LT capital.
        let mut i = holder_input(CloseType::Sold, dec!(1000), dec!(1500));
        i.option_open_date = d(2024, 1, 1);
        i.option_close_date = d(2026, 1, 5); // 735 days
        let r = compute(&i);
        assert_eq!(r.character, TaxCharacter::LongTermCapital);
        assert_eq!(r.gain_loss, dec!(500));
    }

    #[test]
    fn holder_lapsed_capital_loss_of_premium() {
        // Holder bought call for $500, expired worthless. Loss = -$500.
        // Held 5 months → ST capital loss.
        let r = compute(&holder_input(CloseType::Lapsed, dec!(500), Decimal::ZERO));
        assert_eq!(r.gain_loss, dec!(-500));
        assert_eq!(r.character, TaxCharacter::ShortTermCapital);
    }

    #[test]
    fn holder_long_term_lapsed_capital_loss() {
        // LEAP held > 1 year, expired worthless → LT capital loss of
        // premium. This is the long-term-loss-on-options trap that hits
        // patient holders who let LEAPS expire vs sell-to-close.
        let mut i = holder_input(CloseType::Lapsed, dec!(1500), Decimal::ZERO);
        i.option_open_date = d(2024, 1, 1);
        i.option_close_date = d(2026, 1, 15); // 745 days
        let r = compute(&i);
        assert_eq!(r.character, TaxCharacter::LongTermCapital);
        assert_eq!(r.gain_loss, dec!(-1500));
    }

    #[test]
    fn holder_exercised_no_realized_event() {
        // §1234(a)(4) — premium folds into basis of acquired underlying.
        // The option itself does not produce a separately reportable event.
        let r = compute(&holder_input(CloseType::Exercised, dec!(500), Decimal::ZERO));
        assert!(r.is_basis_adjustment_event);
        assert_eq!(r.gain_loss, Decimal::ZERO);
        assert_eq!(r.rule, Section1234Rule::HolderExercise);
    }

    #[test]
    fn one_year_boundary_holder_365_days_short_term() {
        // Option held exactly 365 days → ≤ 1 year → ST.
        let mut i = holder_input(CloseType::Sold, dec!(100), dec!(200));
        i.option_open_date = d(2025, 6, 1);
        i.option_close_date = d(2026, 6, 1);
        let days = (i.option_close_date - i.option_open_date).num_days();
        assert_eq!(days, 365);
        let r = compute(&i);
        assert_eq!(r.character, TaxCharacter::ShortTermCapital);
    }

    #[test]
    fn one_year_boundary_holder_366_days_long_term() {
        // Option held 366 days → > 1 year → LT.
        let mut i = holder_input(CloseType::Sold, dec!(100), dec!(200));
        i.option_open_date = d(2025, 5, 31);
        i.option_close_date = d(2026, 6, 1);
        let days = (i.option_close_date - i.option_open_date).num_days();
        assert_eq!(days, 366);
        let r = compute(&i);
        assert_eq!(r.character, TaxCharacter::LongTermCapital);
    }

    #[test]
    fn holder_ordinary_underlying_is_ordinary_regardless_of_holding() {
        // §1234(a)(3) — if the underlying would be ordinary in the
        // holder's hands (e.g., dealer's inventory), the option result
        // is ordinary regardless of how long the option was held.
        let mut i = holder_input(CloseType::Sold, dec!(100), dec!(200));
        i.option_open_date = d(2024, 1, 1);
        i.option_close_date = d(2026, 6, 1); // > 1 year
        i.underlying_character = UnderlyingCharacter::Ordinary;
        let r = compute(&i);
        assert_eq!(r.character, TaxCharacter::Ordinary);
        assert_eq!(r.rule, Section1234Rule::HolderOrdinary);
    }

    #[test]
    fn section_1256_underlying_bypasses_1234() {
        // §1234(c) — §1256 contracts (broad-based index options, futures
        // options, etc.) are not governed by §1234. The compute returns
        // the Section1256 marker character so the caller routes to
        // section_1256.rs for 60/40 MTM handling.
        let mut i = writer_input(CloseType::Lapsed, dec!(500), Decimal::ZERO);
        i.underlying_character = UnderlyingCharacter::Section1256;
        let r = compute(&i);
        assert_eq!(r.character, TaxCharacter::Section1256);
        assert_eq!(r.rule, Section1234Rule::Section1256Override);
        assert!(r.note.contains("§1256"));
    }

    #[test]
    fn writer_sold_option_to_another_writer_still_short_term() {
        // §1234(b) reaches "closing transactions" which includes selling
        // the obligation on. Even though "sold" sounds like a holder
        // event, when the role is Writer, §1234(b) still governs.
        let r = compute(&writer_input(CloseType::Sold, dec!(500), dec!(700)));
        assert_eq!(r.character, TaxCharacter::ShortTermCapital);
        assert_eq!(r.gain_loss, dec!(200));
    }

    #[test]
    fn zero_premium_writer_lapsed_zero_gain() {
        let r = compute(&writer_input(CloseType::Lapsed, Decimal::ZERO, Decimal::ZERO));
        assert_eq!(r.gain_loss, Decimal::ZERO);
        assert_eq!(r.character, TaxCharacter::ShortTermCapital);
    }

    #[test]
    fn section_1256_override_takes_priority_over_dealer_flag() {
        // If the option is a §1256 contract, the §1256 override fires
        // BEFORE the dealer-in-options check. Dealers in §1256 contracts
        // route through §1256, not §1234(b)(2)(A) ordinary. Catches a
        // future incorrect branch ordering.
        let mut i = writer_input(CloseType::Lapsed, dec!(500), Decimal::ZERO);
        i.underlying_character = UnderlyingCharacter::Section1256;
        i.is_dealer_in_options = true;
        let r = compute(&i);
        assert_eq!(r.character, TaxCharacter::Section1256);
        assert_eq!(r.rule, Section1234Rule::Section1256Override);
    }

    #[test]
    fn section_1256_override_takes_priority_over_holder_exercise() {
        // Similar: §1256 override fires before basis-adjustment events.
        let mut i = holder_input(CloseType::Exercised, dec!(500), Decimal::ZERO);
        i.underlying_character = UnderlyingCharacter::Section1256;
        let r = compute(&i);
        assert_eq!(r.rule, Section1234Rule::Section1256Override);
        assert!(!r.is_basis_adjustment_event);
    }

    #[test]
    fn note_describes_holding_period_days_for_holder_capital() {
        // The note text includes the actual holding-period day count so
        // a downstream UI can show the user how close they were to the
        // ST/LT boundary.
        let r = compute(&holder_input(CloseType::Sold, dec!(100), dec!(200)));
        assert!(r.note.contains("§1234(a)"));
        assert!(r.note.contains("short-term"));
        assert!(r.note.contains("151d")); // Jan 1 → Jun 1 = 151 days
    }

    #[test]
    fn writer_note_explicitly_states_regardless_of_holding_period() {
        // Reinforces the §1234(b)(1) bright-line rule in the human-
        // readable note. Removing this phrase would still pass the
        // character assertion but lose the user-facing explanation.
        let r = compute(&writer_input(CloseType::Lapsed, dec!(500), Decimal::ZERO));
        assert!(r.note.contains("§1234(b)(1)"));
        assert!(r.note.contains("regardless of option holding period"));
    }
}
