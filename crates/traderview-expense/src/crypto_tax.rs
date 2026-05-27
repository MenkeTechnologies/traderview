//! Crypto / digital-asset tax classifier.
//!
//! Per IRS Notice 2014-21 + Rev. Rul. 2019-24, crypto is treated as
//! property for federal tax purposes:
//!   - Sales / exchanges: capital gain/loss (ST/LT by holding period).
//!   - Mining / staking rewards / airdrops: ordinary income at FMV
//!     at the time received (cost basis = that FMV).
//!   - Hard-fork airdrops to existing holders: ordinary income (Rev.
//!     Rul. 2019-24) at FMV when wallet shows "dominion and control".
//!
//! This module categorizes a list of crypto events and emits the
//! totals for the income line (Schedule 1) and the capital lots
//! (Schedule D / Form 8949).
//!
//! Pure compute.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
#[cfg(test)]
use std::str::FromStr;

use crate::schedule_d::is_long_term;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CryptoEventKind {
    /// Token swap or sale → capital gain/loss.
    Sale,
    /// PoW mining reward → ordinary income at receipt.
    MiningReward,
    /// PoS staking reward → ordinary income at receipt.
    StakingReward,
    /// Airdrop (free token distribution) → ordinary income at FMV.
    Airdrop,
    /// Hard fork producing new coin to existing holder → ordinary income.
    HardForkAirdrop,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoEvent {
    pub kind: CryptoEventKind,
    pub symbol: String,
    pub when: NaiveDate,
    /// For Sale: realized P&L (price - basis × qty).
    /// For income events: USD FMV at receipt.
    pub amount_usd: Decimal,
    /// For Sale only — when the sold lot was originally acquired.
    /// Determines ST vs LT bucket.
    pub acquired: Option<NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CryptoTaxReport {
    /// Ordinary income (Schedule 1 line 8z "other income").
    /// Sum of mining + staking + airdrop + hardfork FMVs.
    pub ordinary_income: Decimal,
    pub mining_income: Decimal,
    pub staking_income: Decimal,
    pub airdrop_income: Decimal,
    pub hardfork_income: Decimal,
    /// Capital gain/loss (Schedule D).
    pub short_term_capital: Decimal,
    pub long_term_capital: Decimal,
    pub net_capital: Decimal,
}

pub fn classify(events: &[CryptoEvent]) -> CryptoTaxReport {
    let mut r = CryptoTaxReport::default();
    for e in events {
        match e.kind {
            CryptoEventKind::MiningReward => {
                r.mining_income += e.amount_usd;
                r.ordinary_income += e.amount_usd;
            }
            CryptoEventKind::StakingReward => {
                r.staking_income += e.amount_usd;
                r.ordinary_income += e.amount_usd;
            }
            CryptoEventKind::Airdrop => {
                r.airdrop_income += e.amount_usd;
                r.ordinary_income += e.amount_usd;
            }
            CryptoEventKind::HardForkAirdrop => {
                r.hardfork_income += e.amount_usd;
                r.ordinary_income += e.amount_usd;
            }
            CryptoEventKind::Sale => {
                // IRS rule: long-term requires holding MORE THAN ONE CALENDAR
                // YEAR (calendar-date, not 365 days — leap years matter).
                // Missing acquired date defaults to ST (conservative).
                let lt = match e.acquired {
                    Some(acq) => is_long_term(acq, e.when),
                    None => false,
                };
                if lt {
                    r.long_term_capital += e.amount_usd;
                } else {
                    r.short_term_capital += e.amount_usd;
                }
            }
        }
    }
    r.net_capital = r.short_term_capital + r.long_term_capital;
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> Decimal { Decimal::from_str(s).unwrap() }
    fn day(y: i32, m: u32, d: u32) -> NaiveDate { NaiveDate::from_ymd_opt(y, m, d).unwrap() }

    fn ev(kind: CryptoEventKind, sym: &str, when: NaiveDate, amount: &str,
          acquired: Option<NaiveDate>) -> CryptoEvent
    {
        CryptoEvent {
            kind,
            symbol: sym.into(),
            when,
            amount_usd: d(amount),
            acquired,
        }
    }

    #[test]
    fn empty_returns_default() {
        let r = classify(&[]);
        assert_eq!(r.ordinary_income, Decimal::ZERO);
        assert_eq!(r.net_capital, Decimal::ZERO);
    }

    #[test]
    fn mining_reward_classified_as_ordinary_income() {
        let events = vec![ev(CryptoEventKind::MiningReward, "BTC", day(2026, 3, 1), "5000", None)];
        let r = classify(&events);
        assert_eq!(r.mining_income, d("5000"));
        assert_eq!(r.ordinary_income, d("5000"));
        assert_eq!(r.net_capital, Decimal::ZERO);
    }

    #[test]
    fn staking_reward_separate_bucket_but_same_ordinary_total() {
        let events = vec![
            ev(CryptoEventKind::StakingReward, "ETH", day(2026, 3, 1), "2000", None),
            ev(CryptoEventKind::MiningReward,  "BTC", day(2026, 3, 1), "5000", None),
        ];
        let r = classify(&events);
        assert_eq!(r.staking_income, d("2000"));
        assert_eq!(r.mining_income, d("5000"));
        assert_eq!(r.ordinary_income, d("7000"));
    }

    #[test]
    fn airdrop_and_hardfork_separate_but_both_ordinary() {
        let events = vec![
            ev(CryptoEventKind::Airdrop,         "UNI", day(2026, 3, 1), "100", None),
            ev(CryptoEventKind::HardForkAirdrop, "BCH", day(2026, 3, 1), "500", None),
        ];
        let r = classify(&events);
        assert_eq!(r.airdrop_income, d("100"));
        assert_eq!(r.hardfork_income, d("500"));
        assert_eq!(r.ordinary_income, d("600"));
    }

    #[test]
    fn sale_held_under_one_year_short_term() {
        let events = vec![ev(
            CryptoEventKind::Sale, "BTC",
            day(2026, 6, 1), "1000",
            Some(day(2026, 1, 1)),
        )];
        let r = classify(&events);
        assert_eq!(r.short_term_capital, d("1000"));
        assert_eq!(r.long_term_capital, Decimal::ZERO);
    }

    #[test]
    fn sale_held_over_one_year_long_term() {
        let events = vec![ev(
            CryptoEventKind::Sale, "BTC",
            day(2026, 6, 1), "1000",
            Some(day(2024, 1, 1)),
        )];
        let r = classify(&events);
        assert_eq!(r.long_term_capital, d("1000"));
        assert_eq!(r.short_term_capital, Decimal::ZERO);
    }

    #[test]
    fn leap_year_held_exactly_one_year_is_short_term() {
        // 2024 = leap. Jan 15 2024 → Jan 15 2025 spans 366 calendar days
        // but is still EXACTLY one year. A naive 365-day rule would
        // misclassify as long-term. Calendar-date rule says short-term.
        let events = vec![ev(
            CryptoEventKind::Sale, "BTC",
            day(2025, 1, 15), "1000",
            Some(day(2024, 1, 15)),
        )];
        let r = classify(&events);
        assert_eq!(r.short_term_capital, d("1000"),
            "exactly-1-year hold across leap year must be short-term");
    }

    #[test]
    fn sale_missing_acquired_date_defaults_to_short_term() {
        // Conservative: no holding-period proof → ST treatment (higher tax).
        let events = vec![ev(
            CryptoEventKind::Sale, "BTC",
            day(2026, 6, 1), "1000",
            None,
        )];
        let r = classify(&events);
        assert_eq!(r.short_term_capital, d("1000"));
    }

    #[test]
    fn negative_amount_means_loss() {
        let events = vec![ev(
            CryptoEventKind::Sale, "ETH",
            day(2026, 6, 1), "-500",
            Some(day(2026, 1, 1)),
        )];
        let r = classify(&events);
        assert_eq!(r.short_term_capital, d("-500"));
    }

    #[test]
    fn net_capital_combines_st_plus_lt() {
        let events = vec![
            ev(CryptoEventKind::Sale, "A", day(2026, 6, 1), "1000",  Some(day(2026, 1, 1))),
            ev(CryptoEventKind::Sale, "B", day(2026, 6, 1), "-500",  Some(day(2024, 1, 1))),
        ];
        let r = classify(&events);
        assert_eq!(r.short_term_capital, d("1000"));
        assert_eq!(r.long_term_capital, d("-500"));
        assert_eq!(r.net_capital, d("500"));
    }

    #[test]
    fn mixed_events_independently_tracked() {
        let events = vec![
            ev(CryptoEventKind::MiningReward, "BTC", day(2026, 3, 1), "5000", None),
            ev(CryptoEventKind::Sale,         "BTC", day(2026, 6, 1), "3000", Some(day(2024, 1, 1))),
            ev(CryptoEventKind::Airdrop,      "UNI", day(2026, 9, 1), "100",  None),
        ];
        let r = classify(&events);
        assert_eq!(r.ordinary_income, d("5100"));    // 5000 + 100
        assert_eq!(r.long_term_capital, d("3000"));
        assert_eq!(r.short_term_capital, Decimal::ZERO);
    }
}
