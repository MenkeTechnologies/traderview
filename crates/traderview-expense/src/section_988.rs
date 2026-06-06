//! IRC §988 — Treatment of certain nonfunctional currency transactions.
//!
//! Default rule: gain or loss on a "§988 transaction" is **ordinary**
//! income or loss. This catches every forex trader who imagined their
//! P&L was capital-gain-eligible, and bites every active trader who
//! holds foreign-currency-denominated debt or accrues income in a
//! non-functional currency.
//!
//! §988 transactions per §988(c)(1)(B):
//!
//!   1. Acquiring/disposing of nonfunctional-currency debt instruments.
//!   2. Accruing nonfunctional-currency-denominated expense/income.
//!   3. Forward contracts, futures contracts, or options denominated
//!      in nonfunctional currency (unless §1256(g) covers them).
//!   4. Disposition of nonfunctional currency itself.
//!
//! Three carve-outs / interactions modeled here:
//!
//!   * **§988(c)(1)(D) personal-use exclusion** — gain (NOT loss) on
//!     a personal currency transaction is excluded if total gain
//!     ≤ $200 per transaction. Loss is still nondeductible (personal
//!     losses don't deduct anyway per §165(c)). Travelers buying
//!     euros for vacation routinely qualify.
//!
//!   * **§988(a)(1)(B) capital election** — a taxpayer may elect to
//!     treat gain or loss on a forward/futures/option contract that
//!     is a capital asset and isn't part of a straddle as CAPITAL.
//!     Election made by clear identification on the books before the
//!     close of trading on the day of the transaction (Reg.
//!     §1.988-3(b)). Caller asserts whether the election was
//!     properly made and timely identified.
//!
//!   * **§1256(g) interaction** — regulated futures contracts that are
//!     "foreign currency contracts" within §1256(g)(2) default to
//!     §1256 60/40 treatment, NOT §988. §988(c)(1)(D)(i) lets the
//!     trader elect §988 ordinary character instead (the "kick-out"
//!     election). Caller flags whether the §1256(g) characterization
//!     applies and whether the §988 kick-out election was made.
//!
//! Pure compute. Caller asserts facts; we determine final character.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Classes of §988 transactions per §988(c)(1)(B).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionKind {
    /// Forex spot — direct disposition of nonfunctional currency.
    ForexSpot,
    /// Forex forward contract.
    ForwardContract,
    /// Forex futures contract NOT covered by §1256(g). Use
    /// `ForexFuturesSection1256g` for regulated currency futures.
    NonRegulatedFuturesContract,
    /// Regulated futures contract covered by §1256(g) — defaults to
    /// §1256 60/40 unless the trader elects §988.
    ForexFuturesSection1256g,
    /// Forex option contract.
    OptionContract,
    /// Disposition or accrual of nonfunctional-currency-denominated
    /// debt instrument.
    FxDenominatedDebt,
    /// Accrued nonfunctional-currency expense or income.
    AccruedFxItem,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Character {
    #[default]
    Ordinary,
    Capital,
    /// §1256(g) 60% LTCG / 40% STCG.
    Section1256Sixty40,
    /// Gain excluded entirely under §988(c)(1)(D) personal use.
    ExcludedPersonalUse,
    /// Loss disallowed entirely as a personal loss.
    DisallowedPersonalLoss,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section988Input {
    pub gain_or_loss: Decimal,
    pub transaction_kind: TransactionKind,
    /// True if this is a personal-use transaction (vacation forex,
    /// non-business hedges). The §988(c)(1)(D) $200 exclusion only
    /// applies to gains here.
    pub personal_use: bool,
    /// True if the taxpayer made a timely §988(a)(1)(B) capital
    /// election (identification on books before close of trading on
    /// trade date per Reg. §1.988-3(b)).
    pub section_988a1b_capital_election: bool,
    /// True if the taxpayer made the §988(c)(1)(D)(i) kick-out
    /// election to opt §1256(g) regulated currency futures BACK
    /// into ordinary §988 treatment.
    pub section_988_kickout_election: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Section988Result {
    pub character: Character,
    pub taxable_amount: Decimal,
    pub note: String,
}

fn personal_use_threshold() -> Decimal {
    Decimal::from_str("200").unwrap()
}

pub fn compute(input: &Section988Input) -> Section988Result {
    let mut r = Section988Result {
        taxable_amount: input.gain_or_loss,
        ..Section988Result::default()
    };

    // §988(c)(1)(D) personal-use exclusion: gain only, ≤ $200 per
    // transaction. Loss is a nondeductible personal loss.
    if input.personal_use {
        if input.gain_or_loss > Decimal::ZERO {
            if input.gain_or_loss <= personal_use_threshold() {
                r.character = Character::ExcludedPersonalUse;
                r.taxable_amount = Decimal::ZERO;
                r.note = format!(
                    "§988(c)(1)(D) personal-use exclusion: ${} ≤ $200, gain not recognized",
                    input.gain_or_loss
                );
                return r;
            } else {
                // Over $200 personal gain — still ordinary, no
                // exclusion. (Threshold doesn't graduate.)
                r.character = Character::Ordinary;
                r.note = format!(
                    "personal forex gain ${} > $200 threshold, full amount ordinary",
                    input.gain_or_loss
                );
                return r;
            }
        }
        if input.gain_or_loss < Decimal::ZERO {
            r.character = Character::DisallowedPersonalLoss;
            r.taxable_amount = Decimal::ZERO;
            r.note =
                "personal forex loss disallowed under §165(c); no §988 ordinary loss treatment"
                    .into();
            return r;
        }
        // gain == 0
        r.note = "no gain or loss".into();
        return r;
    }

    // Trade-or-business / investment transactions.
    match input.transaction_kind {
        TransactionKind::ForexFuturesSection1256g => {
            // §1256(g) regulated currency futures: 60/40 unless the
            // §988(c)(1)(D)(i) kick-out election routes them back to
            // §988 ordinary.
            if input.section_988_kickout_election {
                r.character = Character::Ordinary;
                r.note =
                    "§988(c)(1)(D)(i) kick-out election: §1256(g) futures treated as §988 ordinary"
                        .into();
            } else {
                r.character = Character::Section1256Sixty40;
                r.note = "§1256(g) foreign currency contract: 60% LTCG / 40% STCG".into();
            }
        }
        TransactionKind::ForwardContract
        | TransactionKind::NonRegulatedFuturesContract
        | TransactionKind::OptionContract => {
            // §988(a)(1)(B) capital election available for these.
            if input.section_988a1b_capital_election {
                r.character = Character::Capital;
                r.note = "§988(a)(1)(B) capital election: forward/futures/option treated as capital gain/loss".into();
            } else {
                r.character = Character::Ordinary;
                r.note = "§988 default ordinary character for forex derivative".into();
            }
        }
        TransactionKind::ForexSpot
        | TransactionKind::FxDenominatedDebt
        | TransactionKind::AccruedFxItem => {
            // Spot + debt + accruals: always ordinary; no §988(a)(1)(B)
            // election available for these classes.
            r.character = Character::Ordinary;
            r.note = format!(
                "§988 ordinary character — {:?} not eligible for §988(a)(1)(B) capital election",
                input.transaction_kind
            );
        }
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn base() -> Section988Input {
        Section988Input {
            gain_or_loss: dec!(5000),
            transaction_kind: TransactionKind::ForexSpot,
            personal_use: false,
            section_988a1b_capital_election: false,
            section_988_kickout_election: false,
        }
    }

    #[test]
    fn spot_default_is_ordinary() {
        let r = compute(&base());
        assert_eq!(r.character, Character::Ordinary);
        assert_eq!(r.taxable_amount, dec!(5000));
    }

    #[test]
    fn loss_on_spot_also_ordinary_no_3k_cap() {
        // §988 ordinary loss is fully deductible — bypasses the
        // §1212(b) $3k capital loss cap.
        let mut i = base();
        i.gain_or_loss = dec!(-10000);
        let r = compute(&i);
        assert_eq!(r.character, Character::Ordinary);
        assert_eq!(r.taxable_amount, dec!(-10000));
    }

    #[test]
    fn forward_with_capital_election_is_capital() {
        let mut i = base();
        i.transaction_kind = TransactionKind::ForwardContract;
        i.section_988a1b_capital_election = true;
        let r = compute(&i);
        assert_eq!(r.character, Character::Capital);
        assert!(r.note.contains("§988(a)(1)(B)"));
    }

    #[test]
    fn forward_without_election_is_ordinary() {
        let mut i = base();
        i.transaction_kind = TransactionKind::ForwardContract;
        let r = compute(&i);
        assert_eq!(r.character, Character::Ordinary);
    }

    #[test]
    fn regulated_currency_futures_default_to_1256_60_40() {
        let mut i = base();
        i.transaction_kind = TransactionKind::ForexFuturesSection1256g;
        let r = compute(&i);
        assert_eq!(r.character, Character::Section1256Sixty40);
        assert!(r.note.contains("§1256(g)"));
    }

    #[test]
    fn regulated_currency_futures_with_kickout_election_is_ordinary() {
        let mut i = base();
        i.transaction_kind = TransactionKind::ForexFuturesSection1256g;
        i.section_988_kickout_election = true;
        let r = compute(&i);
        assert_eq!(r.character, Character::Ordinary);
        assert!(r.note.contains("kick-out"));
    }

    #[test]
    fn personal_use_gain_under_200_excluded() {
        let mut i = base();
        i.gain_or_loss = dec!(150);
        i.personal_use = true;
        let r = compute(&i);
        assert_eq!(r.character, Character::ExcludedPersonalUse);
        assert_eq!(r.taxable_amount, Decimal::ZERO);
        assert!(r.note.contains("personal-use"));
    }

    #[test]
    fn personal_use_gain_exactly_200_excluded() {
        // Threshold is "≤ $200", so exactly $200 should be excluded.
        let mut i = base();
        i.gain_or_loss = dec!(200);
        i.personal_use = true;
        let r = compute(&i);
        assert_eq!(r.character, Character::ExcludedPersonalUse);
    }

    #[test]
    fn personal_use_gain_over_200_fully_ordinary_no_partial_exclusion() {
        // §988(c)(1)(D) doesn't graduate — over $200, the WHOLE gain
        // is ordinary income.
        let mut i = base();
        i.gain_or_loss = dec!(201);
        i.personal_use = true;
        let r = compute(&i);
        assert_eq!(r.character, Character::Ordinary);
        assert_eq!(r.taxable_amount, dec!(201));
    }

    #[test]
    fn personal_use_loss_disallowed_not_ordinary() {
        // Personal forex loss = nondeductible §165(c) personal loss.
        // NOT ordinary §988 loss.
        let mut i = base();
        i.gain_or_loss = dec!(-300);
        i.personal_use = true;
        let r = compute(&i);
        assert_eq!(r.character, Character::DisallowedPersonalLoss);
        assert_eq!(r.taxable_amount, Decimal::ZERO);
    }

    #[test]
    fn fx_denominated_debt_always_ordinary_no_election() {
        // §988(a)(1)(B) election doesn't apply to debt instruments.
        let mut i = base();
        i.transaction_kind = TransactionKind::FxDenominatedDebt;
        i.section_988a1b_capital_election = true; // ignored
        let r = compute(&i);
        assert_eq!(r.character, Character::Ordinary);
        assert!(r.note.contains("not eligible"));
    }

    #[test]
    fn forex_spot_always_ordinary_election_does_not_apply() {
        let mut i = base();
        i.section_988a1b_capital_election = true; // ignored
        let r = compute(&i);
        assert_eq!(r.character, Character::Ordinary);
    }

    #[test]
    fn option_contract_capital_election_applies() {
        let mut i = base();
        i.transaction_kind = TransactionKind::OptionContract;
        i.section_988a1b_capital_election = true;
        let r = compute(&i);
        assert_eq!(r.character, Character::Capital);
    }

    #[test]
    fn personal_use_zero_gain_returns_no_op() {
        let mut i = base();
        i.gain_or_loss = Decimal::ZERO;
        i.personal_use = true;
        let r = compute(&i);
        assert!(r.note.contains("no gain"));
    }

    #[test]
    fn personal_use_kickout_election_ignored_on_personal_route() {
        // Personal-use route runs first; election flags don't matter.
        let mut i = base();
        i.gain_or_loss = dec!(150);
        i.personal_use = true;
        i.section_988_kickout_election = true;
        let r = compute(&i);
        assert_eq!(r.character, Character::ExcludedPersonalUse);
    }

    #[test]
    fn accrued_fx_item_is_ordinary_no_election() {
        let mut i = base();
        i.transaction_kind = TransactionKind::AccruedFxItem;
        i.section_988a1b_capital_election = true;
        let r = compute(&i);
        assert_eq!(r.character, Character::Ordinary);
    }

    #[test]
    fn non_regulated_futures_with_election_is_capital() {
        let mut i = base();
        i.transaction_kind = TransactionKind::NonRegulatedFuturesContract;
        i.section_988a1b_capital_election = true;
        let r = compute(&i);
        assert_eq!(r.character, Character::Capital);
    }
}
