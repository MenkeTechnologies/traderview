//! IRC §1259 — Constructive sale of appreciated financial positions.
//!
//! Before 1997, traders could lock in unrealized gains tax-free by
//! "shorting against the box" — taking a short position in stock they
//! already held long, eliminating market risk while deferring the
//! recognition of capital gain indefinitely. §1259 ended that: an
//! **appreciated financial position** is treated as **constructively
//! sold** when the taxpayer enters a hedge that substantially
//! eliminates risk of loss and opportunity for gain. The deemed sale
//! triggers gain recognition at fair market value as of the trigger
//! date.
//!
//! §1259(c) covered transactions:
//!
//!   * **Short sale of substantially identical property** —
//!     §1259(c)(1)(A). The classic short-against-the-box.
//!   * **Offsetting notional principal contract** — §1259(c)(1)(B).
//!     E.g. swap of total return on the same stock.
//!   * **Futures or forward contract** to deliver substantially
//!     identical property — §1259(c)(1)(C).
//!   * **Two or more transactions** that, when combined, have
//!     substantially the same effect as the above — §1259(c)(1)(D).
//!
//! §1259(c)(3)(A) **safe harbor** — the constructive sale is NOT
//! triggered if all THREE conditions are met:
//!
//!   1. The hedge transaction is closed before the **30th day after**
//!      the close of the taxable year (typically January 30).
//!   2. The taxpayer holds the appreciated long position throughout
//!      the **60-day period** beginning on the date the hedge was
//!      closed.
//!   3. The taxpayer's risk of loss on the long position is **not**
//!      reduced during that 60-day period (no replacement hedge,
//!      no offsetting position, no protective put bought at strike).
//!
//! Failing ANY of those three conditions causes the original hedge
//! to trigger the constructive sale retroactively, with gain
//! recognized as of the **hedge entry date** (not the close date).
//!
//! §1259(d) special rules:
//!
//!   * **§1256 contract exception** — §1259(c)(3)(C). Hedging
//!     transactions in §1256 mark-to-market contracts don't
//!     constructively sell the underlying long.
//!   * **Basis increased** by gain recognized — §1259(b)(2).
//!   * **Holding period restarts** on the date of the constructive
//!     sale for the position deemed sold.
//!
//! Pure compute. Caller asserts the facts; we evaluate the trigger,
//! safe harbor, and post-trigger basis + holding period.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HedgeType {
    /// §1259(c)(1)(A) — short sale of substantially identical property.
    ShortSaleSubstantiallyIdentical,
    /// §1259(c)(1)(B) — offsetting notional principal contract.
    OffsettingNotionalPrincipalContract,
    /// §1259(c)(1)(C) — futures contract for delivery of substantially
    /// identical property.
    FuturesContractSubstantiallyIdentical,
    /// §1259(c)(1)(C) — forward contract.
    ForwardContractSubstantiallyIdentical,
    /// §1259(c)(1)(D) — combined positions (collar, etc.).
    CombinedPositionsSameEconomicEffect,
    /// §1256 contract (regulated futures, broad-based index options,
    /// foreign currency contracts). Exempt under §1259(c)(3)(C).
    Section1256Contract,
    /// No hedge / standalone protective put at OTM strike / etc. Does
    /// not trigger constructive sale.
    NoCoveredTransaction,
}

impl HedgeType {
    pub fn is_covered_transaction(self) -> bool {
        matches!(
            self,
            HedgeType::ShortSaleSubstantiallyIdentical
                | HedgeType::OffsettingNotionalPrincipalContract
                | HedgeType::FuturesContractSubstantiallyIdentical
                | HedgeType::ForwardContractSubstantiallyIdentical
                | HedgeType::CombinedPositionsSameEconomicEffect
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Character {
    /// Long-term capital gain (position held > 1 year before deemed sale).
    LongTermCapitalGain,
    /// Short-term capital gain (held ≤ 1 year).
    ShortTermCapitalGain,
    /// No gain recognized (loss or zero appreciation).
    #[default]
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section1259Input {
    pub long_position_basis: Decimal,
    pub long_position_fmv_at_hedge_entry: Decimal,
    pub long_position_acquisition_date: NaiveDate,
    pub hedge_type: HedgeType,
    pub hedge_entry_date: NaiveDate,
    /// True if the §1259(c)(3)(A) safe harbor close happened: hedge
    /// closed on or before Jan 30 (= 30 days after Dec 31).
    pub hedge_closed_before_jan_30_next_year: bool,
    /// True if the taxpayer held the long position through the 60-day
    /// window after closing the hedge.
    pub long_held_60_days_after_hedge_close: bool,
    /// True if the taxpayer's risk on the long was NOT diminished
    /// during that 60-day window (no replacement hedge / no new
    /// protective put / no other §1259(c) transaction).
    pub no_risk_reduction_during_60_day_window: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Section1259Result {
    pub is_covered_hedge_transaction: bool,
    pub appreciation_at_hedge_entry: Decimal,
    pub safe_harbor_qualifies: bool,
    pub constructive_sale_triggered: bool,
    pub gain_recognized: Decimal,
    pub character: Character,
    /// New basis after constructive sale per §1259(b)(2): old basis +
    /// gain recognized. Equals FMV at the trigger date when the sale
    /// triggers.
    pub new_basis_after_constructive_sale: Decimal,
    /// New holding period start date per §1259(b)(2) — the deemed
    /// sale date.
    pub new_holding_period_start: Option<NaiveDate>,
    pub note: String,
}

fn long_term_held(acquisition: NaiveDate, sale: NaiveDate) -> bool {
    // §1222(3) requires holding "more than one year". Adding 12
    // calendar months handles leap-year edge correctly: a position
    // acquired Nov 1, 2023 and sold Nov 1, 2024 spans 366 days due to
    // the 2024 leap day, but is NOT LTCG (the IRS day-by-day rule
    // excludes the acquisition day from the count).
    match acquisition.checked_add_months(chrono::Months::new(12)) {
        Some(one_year_later) => sale > one_year_later,
        None => false,
    }
}

pub fn compute(input: &Section1259Input) -> Section1259Result {
    let mut r = Section1259Result {
        is_covered_hedge_transaction: input.hedge_type.is_covered_transaction(),
        appreciation_at_hedge_entry: input.long_position_fmv_at_hedge_entry
            - input.long_position_basis,
        new_basis_after_constructive_sale: input.long_position_basis,
        ..Section1259Result::default()
    };

    if !r.is_covered_hedge_transaction {
        r.note = match input.hedge_type {
            HedgeType::Section1256Contract => {
                "§1259(c)(3)(C) exception: §1256 contract — no constructive sale".into()
            }
            HedgeType::NoCoveredTransaction => "no covered §1259(c) hedge transaction".into(),
            _ => "hedge not within §1259(c) covered list".into(),
        };
        return r;
    }

    if r.appreciation_at_hedge_entry <= Decimal::ZERO {
        // §1259(b)(1) requires an "appreciated" financial position
        // (FMV > basis). A loss or break-even position triggers no
        // constructive sale.
        r.note = format!(
            "long position not appreciated (FMV ${} ≤ basis ${}) — §1259 does not apply",
            input.long_position_fmv_at_hedge_entry, input.long_position_basis
        );
        return r;
    }

    // Safe harbor evaluation per §1259(c)(3)(A): ALL three must hold.
    r.safe_harbor_qualifies = input.hedge_closed_before_jan_30_next_year
        && input.long_held_60_days_after_hedge_close
        && input.no_risk_reduction_during_60_day_window;

    if r.safe_harbor_qualifies {
        r.constructive_sale_triggered = false;
        r.note = "§1259(c)(3)(A) safe harbor: hedge closed timely AND 60-day un-hedged window honored — no constructive sale".into();
        return r;
    }

    // Triggered. Gain recognized at FMV - basis on the hedge entry date.
    r.constructive_sale_triggered = true;
    r.gain_recognized = r.appreciation_at_hedge_entry;
    r.character = if long_term_held(input.long_position_acquisition_date, input.hedge_entry_date) {
        Character::LongTermCapitalGain
    } else {
        Character::ShortTermCapitalGain
    };
    r.new_basis_after_constructive_sale =
        input.long_position_basis + r.gain_recognized;
    r.new_holding_period_start = Some(input.hedge_entry_date);

    r.note = format!(
        "§1259 constructive sale triggered on {} — ${} {:?} recognized; basis steps up to ${}; new holding period from {}",
        input.hedge_entry_date,
        r.gain_recognized,
        r.character,
        r.new_basis_after_constructive_sale,
        input.hedge_entry_date,
    );
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    fn base() -> Section1259Input {
        // Bought $50k stock 2 years ago, now worth $80k, shorting the box.
        Section1259Input {
            long_position_basis: dec!(50000),
            long_position_fmv_at_hedge_entry: dec!(80000),
            long_position_acquisition_date: date(2022, 6, 1),
            hedge_type: HedgeType::ShortSaleSubstantiallyIdentical,
            hedge_entry_date: date(2024, 11, 1),
            hedge_closed_before_jan_30_next_year: false,
            long_held_60_days_after_hedge_close: false,
            no_risk_reduction_during_60_day_window: false,
        }
    }

    #[test]
    fn classic_short_against_box_triggers_constructive_sale() {
        let r = compute(&base());
        assert!(r.is_covered_hedge_transaction);
        assert!(r.constructive_sale_triggered);
        assert_eq!(r.gain_recognized, dec!(30000));
        assert_eq!(r.character, Character::LongTermCapitalGain);
        assert_eq!(r.new_basis_after_constructive_sale, dec!(80000));
        assert_eq!(r.new_holding_period_start, Some(date(2024, 11, 1)));
    }

    #[test]
    fn short_term_holding_period_yields_short_term_gain() {
        // Acquired June 1 2024, hedged Nov 1 2024 — 5 months.
        let mut i = base();
        i.long_position_acquisition_date = date(2024, 6, 1);
        let r = compute(&i);
        assert_eq!(r.character, Character::ShortTermCapitalGain);
    }

    #[test]
    fn holding_exactly_one_year_is_short_term() {
        // §1222 LT requires > 1 year. Exactly 365 days isn't enough.
        let mut i = base();
        i.long_position_acquisition_date = date(2023, 11, 1);
        i.hedge_entry_date = date(2024, 11, 1);
        let r = compute(&i);
        assert_eq!(r.character, Character::ShortTermCapitalGain);
    }

    #[test]
    fn safe_harbor_all_three_conditions_pass_no_trigger() {
        let mut i = base();
        i.hedge_closed_before_jan_30_next_year = true;
        i.long_held_60_days_after_hedge_close = true;
        i.no_risk_reduction_during_60_day_window = true;
        let r = compute(&i);
        assert!(r.safe_harbor_qualifies);
        assert!(!r.constructive_sale_triggered);
        assert_eq!(r.gain_recognized, Decimal::ZERO);
        assert!(r.note.contains("safe harbor"));
    }

    #[test]
    fn safe_harbor_missing_60_day_window_triggers() {
        let mut i = base();
        i.hedge_closed_before_jan_30_next_year = true;
        i.long_held_60_days_after_hedge_close = false; // fails
        i.no_risk_reduction_during_60_day_window = true;
        let r = compute(&i);
        assert!(!r.safe_harbor_qualifies);
        assert!(r.constructive_sale_triggered);
    }

    #[test]
    fn safe_harbor_missing_no_risk_reduction_triggers() {
        let mut i = base();
        i.hedge_closed_before_jan_30_next_year = true;
        i.long_held_60_days_after_hedge_close = true;
        i.no_risk_reduction_during_60_day_window = false; // fails — replacement hedge
        let r = compute(&i);
        assert!(!r.safe_harbor_qualifies);
        assert!(r.constructive_sale_triggered);
    }

    #[test]
    fn safe_harbor_late_close_triggers() {
        let mut i = base();
        i.hedge_closed_before_jan_30_next_year = false; // closed in Feb
        i.long_held_60_days_after_hedge_close = true;
        i.no_risk_reduction_during_60_day_window = true;
        let r = compute(&i);
        assert!(!r.safe_harbor_qualifies);
        assert!(r.constructive_sale_triggered);
    }

    #[test]
    fn loss_position_no_constructive_sale() {
        // FMV < basis. §1259(b)(1) "appreciated" requirement fails.
        let mut i = base();
        i.long_position_fmv_at_hedge_entry = dec!(40000); // loss
        let r = compute(&i);
        assert!(!r.constructive_sale_triggered);
        assert!(r.note.contains("not appreciated"));
    }

    #[test]
    fn breakeven_position_no_constructive_sale() {
        let mut i = base();
        i.long_position_fmv_at_hedge_entry = i.long_position_basis;
        let r = compute(&i);
        assert!(!r.constructive_sale_triggered);
    }

    #[test]
    fn section_1256_contract_exempt_under_c_3_c() {
        // §1256 contracts are MTM — §1259(c)(3)(C) carves them out.
        let mut i = base();
        i.hedge_type = HedgeType::Section1256Contract;
        let r = compute(&i);
        assert!(!r.is_covered_hedge_transaction);
        assert!(!r.constructive_sale_triggered);
        assert!(r.note.contains("§1256"));
    }

    #[test]
    fn no_covered_transaction_no_trigger() {
        let mut i = base();
        i.hedge_type = HedgeType::NoCoveredTransaction;
        let r = compute(&i);
        assert!(!r.constructive_sale_triggered);
    }

    #[test]
    fn offsetting_notional_principal_contract_triggers() {
        let mut i = base();
        i.hedge_type = HedgeType::OffsettingNotionalPrincipalContract;
        let r = compute(&i);
        assert!(r.is_covered_hedge_transaction);
        assert!(r.constructive_sale_triggered);
    }

    #[test]
    fn forward_contract_triggers() {
        let mut i = base();
        i.hedge_type = HedgeType::ForwardContractSubstantiallyIdentical;
        let r = compute(&i);
        assert!(r.is_covered_hedge_transaction);
        assert!(r.constructive_sale_triggered);
    }

    #[test]
    fn futures_contract_triggers() {
        let mut i = base();
        i.hedge_type = HedgeType::FuturesContractSubstantiallyIdentical;
        let r = compute(&i);
        assert!(r.is_covered_hedge_transaction);
        assert!(r.constructive_sale_triggered);
    }

    #[test]
    fn combined_positions_trigger() {
        let mut i = base();
        i.hedge_type = HedgeType::CombinedPositionsSameEconomicEffect;
        let r = compute(&i);
        assert!(r.is_covered_hedge_transaction);
        assert!(r.constructive_sale_triggered);
    }

    #[test]
    fn basis_step_up_equals_gain_recognized() {
        // §1259(b)(2): new basis = old + gain. Verifies math.
        let r = compute(&base());
        assert_eq!(
            r.new_basis_after_constructive_sale,
            base().long_position_basis + r.gain_recognized
        );
    }

    #[test]
    fn new_basis_unchanged_when_safe_harbor_qualifies() {
        let mut i = base();
        i.hedge_closed_before_jan_30_next_year = true;
        i.long_held_60_days_after_hedge_close = true;
        i.no_risk_reduction_during_60_day_window = true;
        let r = compute(&i);
        // No trigger → basis stays at original cost.
        assert_eq!(r.new_basis_after_constructive_sale, dec!(50000));
        assert!(r.new_holding_period_start.is_none());
    }

    #[test]
    fn appreciation_calculation_matches_fmv_minus_basis() {
        let r = compute(&base());
        assert_eq!(r.appreciation_at_hedge_entry, dec!(30000));
    }

    #[test]
    fn loss_position_with_failed_safe_harbor_still_no_trigger() {
        // No appreciation → no trigger regardless of safe-harbor status.
        let mut i = base();
        i.long_position_fmv_at_hedge_entry = dec!(40000);
        i.hedge_closed_before_jan_30_next_year = false;
        let r = compute(&i);
        assert!(!r.constructive_sale_triggered);
    }
}
