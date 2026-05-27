//! §475(f) Mark-to-Market trader election helper.
//!
//! A trader who elects MTM treats all open positions at year-end as if
//! sold at FMV (deemed sale), recognizing gain/loss as ordinary income
//! on Form 4797. Three load-bearing consequences:
//!   1. Losses are NOT subject to the $3k capital-loss cap.
//!   2. Wash-sale rules don't apply.
//!   3. Open-position gain/loss recognized annually — basis is reset.
//!
//! Election deadline: by April 15 for the CURRENT tax year (i.e.
//! retroactive). Once elected, it's permanent unless IRS approves
//! revocation. This helper computes the deemed-sale total for a list
//! of open positions at year-end FMV, plus the gross election summary.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenPositionAtYearEnd {
    pub symbol: String,
    pub qty: Decimal,
    pub avg_cost: Decimal,
    /// Fair market value as of Dec 31. Caller pulls from market_data.
    pub year_end_price: Decimal,
}

impl OpenPositionAtYearEnd {
    pub fn deemed_gain(&self) -> Decimal {
        (self.year_end_price - self.avg_cost) * self.qty
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Mtm475fReport {
    /// Sum of (year_end_price - avg_cost) × qty across every open
    /// position. Positive = additional ordinary gain to recognize,
    /// negative = additional ordinary loss.
    pub total_deemed_gain: Decimal,
    pub winners: Vec<DeemedSale>,
    pub losers: Vec<DeemedSale>,
    /// Sum of just the loss side. Under MTM these losses are ORDINARY —
    /// no $3k cap — making them more valuable than capital losses.
    pub total_deemed_loss: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeemedSale {
    pub symbol: String,
    pub qty: Decimal,
    pub avg_cost: Decimal,
    pub year_end_price: Decimal,
    pub deemed_gain: Decimal,
}

pub fn report(positions: &[OpenPositionAtYearEnd]) -> Mtm475fReport {
    let mut winners = Vec::new();
    let mut losers = Vec::new();
    let mut total = Decimal::ZERO;
    let mut total_loss = Decimal::ZERO;
    for p in positions {
        let gain = p.deemed_gain();
        let sale = DeemedSale {
            symbol: p.symbol.clone(),
            qty: p.qty,
            avg_cost: p.avg_cost,
            year_end_price: p.year_end_price,
            deemed_gain: gain,
        };
        if gain >= Decimal::ZERO { winners.push(sale); }
        else { total_loss += -gain; losers.push(sale); }
        total += gain;
    }
    winners.sort_by(|a, b| b.deemed_gain.cmp(&a.deemed_gain));
    losers.sort_by(|a, b| a.deemed_gain.cmp(&b.deemed_gain));
    Mtm475fReport {
        total_deemed_gain: total,
        winners,
        losers,
        total_deemed_loss: total_loss,
    }
}

/// Election-deadline check. Returns "valid" iff `today` is on or before
/// April 15 of `tax_year + 1` (the original return due date) AND no
/// extension has been filed yet. Caller passes both pieces of info.
pub fn election_deadline_status(
    today: chrono::NaiveDate,
    tax_year: i32,
    extension_filed: bool,
) -> ElectionStatus {
    let april_15 = chrono::NaiveDate::from_ymd_opt(tax_year + 1, 4, 15).unwrap();
    if today > april_15 {
        // Past Apr 15. If an extension was filed BEFORE Apr 15, can
        // attach a §475(f) election to the extended return.
        return if extension_filed {
            ElectionStatus::AllowedViaExtension
        } else {
            ElectionStatus::Missed
        };
    }
    ElectionStatus::OnTime
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ElectionStatus {
    /// On or before Apr 15 of the year following the tax year.
    OnTime,
    /// Past Apr 15 but an extension is on file — can still elect.
    AllowedViaExtension,
    /// Past the deadline with no extension — election must wait to
    /// next tax year.
    Missed,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn d(s: &str) -> Decimal { Decimal::from_str(s).unwrap() }

    fn pos(symbol: &str, qty: &str, cost: &str, yep: &str) -> OpenPositionAtYearEnd {
        OpenPositionAtYearEnd {
            symbol: symbol.into(),
            qty: d(qty),
            avg_cost: d(cost),
            year_end_price: d(yep),
        }
    }

    #[test]
    fn deemed_gain_positive_when_year_end_above_cost() {
        let p = pos("AAPL", "100", "150", "160");
        assert_eq!(p.deemed_gain(), d("1000"));
    }

    #[test]
    fn deemed_gain_negative_when_year_end_below_cost() {
        let p = pos("AAPL", "100", "150", "140");
        assert_eq!(p.deemed_gain(), d("-1000"));
    }

    #[test]
    fn report_aggregates_winners_and_losers() {
        let positions = vec![
            pos("AAPL", "100", "150", "160"),   // +1000
            pos("TSLA", "50",  "300", "280"),   // -1000
            pos("GME",  "200", "20",  "30"),    // +2000
        ];
        let r = report(&positions);
        assert_eq!(r.total_deemed_gain, d("2000"));
        assert_eq!(r.total_deemed_loss, d("1000"));
        assert_eq!(r.winners.len(), 2);
        assert_eq!(r.losers.len(), 1);
        // Winners sorted desc — GME ($2000) before AAPL ($1000).
        assert_eq!(r.winners[0].symbol, "GME");
    }

    #[test]
    fn empty_positions_yields_zero_report() {
        let r = report(&[]);
        assert_eq!(r.total_deemed_gain, Decimal::ZERO);
        assert!(r.winners.is_empty());
        assert!(r.losers.is_empty());
    }

    #[test]
    fn election_on_time_before_april_15() {
        let status = election_deadline_status(
            chrono::NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
            2025, false,
        );
        assert_eq!(status, ElectionStatus::OnTime);
    }

    #[test]
    fn election_on_april_15_inclusive() {
        let status = election_deadline_status(
            chrono::NaiveDate::from_ymd_opt(2026, 4, 15).unwrap(),
            2025, false,
        );
        assert_eq!(status, ElectionStatus::OnTime, "Apr 15 must be inclusive");
    }

    #[test]
    fn election_missed_after_april_15_no_extension() {
        let status = election_deadline_status(
            chrono::NaiveDate::from_ymd_opt(2026, 4, 16).unwrap(),
            2025, false,
        );
        assert_eq!(status, ElectionStatus::Missed);
    }

    #[test]
    fn election_allowed_via_extension() {
        let status = election_deadline_status(
            chrono::NaiveDate::from_ymd_opt(2026, 9, 1).unwrap(),
            2025, true,
        );
        assert_eq!(status, ElectionStatus::AllowedViaExtension);
    }

    #[test]
    fn break_even_position_counts_as_winner() {
        // Edge case: exactly $0 deemed gain → goes in winners (>= 0).
        let r = report(&[pos("X", "100", "10", "10")]);
        assert_eq!(r.winners.len(), 1);
        assert_eq!(r.losers.len(), 0);
        assert_eq!(r.total_deemed_gain, Decimal::ZERO);
    }
}
