//! Wash-sale detection (IRS §1091).
//!
//! A wash sale is a loss-realizing sell where the same (or substantially
//! identical) security was bought within ±30 days. The loss is disallowed
//! and added to the basis of the replacement lot.
//!
//! Pure compute. We don't model "substantially identical" (options /
//! ETFs / different share classes) — that's a tax-advisor question. We
//! flag every replacement BUY of the SAME ticker within ±30 days of a
//! losing sell. The user reviews and refines.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClosingTrade {
    pub trade_id: Uuid,
    pub symbol: String,
    pub closed_at: NaiveDate,
    pub net_pnl: Decimal,           // negative if loss
    pub qty: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpeningExecution {
    pub execution_id: Uuid,
    pub symbol: String,
    pub executed_at: NaiveDate,
    pub qty: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WashHit {
    pub losing_trade_id: Uuid,
    pub symbol: String,
    pub loss_amount: Decimal,
    /// Days between the losing close and the suspect replacement buy.
    /// Negative = bought BEFORE the loss; positive = bought AFTER.
    pub days_offset: i64,
    pub replacement_execution_id: Uuid,
    /// Caller may use this to suggest "disallow loss, add to lot
    /// basis" — we just flag.
    pub disallowed_loss_estimate: Decimal,
}

const WASH_WINDOW_DAYS: i64 = 30;

/// For each LOSING closing trade, find every replacement buy of the
/// same symbol within ±30 days. Return one WashHit per qualifying
/// (losing_trade, replacement_exec) pair.
pub fn detect_hits(
    closings: &[ClosingTrade],
    openings: &[OpeningExecution],
) -> Vec<WashHit> {
    let mut hits = Vec::new();
    for closing in closings {
        if closing.net_pnl >= Decimal::ZERO { continue; }
        let loss = -closing.net_pnl;
        for open in openings {
            if open.symbol != closing.symbol { continue; }
            let days = (open.executed_at - closing.closed_at).num_days();
            if days.abs() > WASH_WINDOW_DAYS { continue; }
            // Disallowed loss is min(loss, opening_qty × per-share loss).
            // Without per-share basis we approximate: cap at loss × (min
            // qty / closing qty) — straightforward proportion.
            let qty_ratio = if closing.qty.is_zero() {
                Decimal::ZERO
            } else {
                (open.qty.min(closing.qty) / closing.qty)
                    .max(Decimal::ZERO).min(Decimal::ONE)
            };
            let disallowed = loss * qty_ratio;
            hits.push(WashHit {
                losing_trade_id: closing.trade_id,
                symbol: closing.symbol.clone(),
                loss_amount: loss,
                days_offset: days,
                replacement_execution_id: open.execution_id,
                disallowed_loss_estimate: disallowed,
            });
        }
    }
    hits
}

/// Total disallowed-loss estimate across all hits. Use to surface "your
/// $5,000 of losses had $3,200 disallowed by wash-sale" in the report.
pub fn total_disallowed(hits: &[WashHit]) -> Decimal {
    hits.iter().map(|h| h.disallowed_loss_estimate).sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn d(s: &str) -> Decimal { Decimal::from_str(s).unwrap() }
    fn day(y: i32, m: u32, d_: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d_).unwrap()
    }

    fn losing_close(symbol: &str, day_: NaiveDate, pnl: &str, qty: &str) -> ClosingTrade {
        ClosingTrade {
            trade_id: Uuid::new_v4(),
            symbol: symbol.into(),
            closed_at: day_,
            net_pnl: d(pnl),
            qty: d(qty),
        }
    }

    fn open(symbol: &str, day_: NaiveDate, qty: &str) -> OpeningExecution {
        OpeningExecution {
            execution_id: Uuid::new_v4(),
            symbol: symbol.into(),
            executed_at: day_,
            qty: d(qty),
        }
    }

    #[test]
    fn winning_trade_never_flags() {
        let closings = vec![losing_close("AAPL", day(2026, 6, 1), "500", "100")];
        let openings = vec![open("AAPL", day(2026, 6, 15), "100")];
        let hits = detect_hits(&closings, &openings);
        assert!(hits.is_empty(), "wash-sale only applies to losses");
    }

    #[test]
    fn replacement_buy_inside_window_flags() {
        let closings = vec![losing_close("AAPL", day(2026, 6, 1), "-500", "100")];
        let openings = vec![open("AAPL", day(2026, 6, 15), "100")];   // +14 days
        let hits = detect_hits(&closings, &openings);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].days_offset, 14);
        assert_eq!(hits[0].loss_amount, d("500"));
        assert_eq!(hits[0].disallowed_loss_estimate, d("500"));
    }

    #[test]
    fn replacement_buy_outside_window_does_not_flag() {
        let closings = vec![losing_close("AAPL", day(2026, 6, 1), "-500", "100")];
        let openings = vec![open("AAPL", day(2026, 7, 5), "100")];    // +34 days
        let hits = detect_hits(&closings, &openings);
        assert!(hits.is_empty());
    }

    #[test]
    fn replacement_buy_before_loss_also_counts() {
        // The wash-sale window is BIDIRECTIONAL ±30 days — buying
        // 20 days before the loss-sell also disqualifies.
        let closings = vec![losing_close("AAPL", day(2026, 6, 30), "-500", "100")];
        let openings = vec![open("AAPL", day(2026, 6, 10), "100")];   // -20 days
        let hits = detect_hits(&closings, &openings);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].days_offset, -20);
    }

    #[test]
    fn different_symbol_does_not_flag() {
        let closings = vec![losing_close("AAPL", day(2026, 6, 1), "-500", "100")];
        let openings = vec![open("TSLA", day(2026, 6, 15), "100")];
        let hits = detect_hits(&closings, &openings);
        assert!(hits.is_empty());
    }

    #[test]
    fn partial_replacement_qty_estimates_proportional_disallow() {
        // Sold 100 sh at a loss, only bought back 30 → only 30% of the
        // loss is disallowed.
        let closings = vec![losing_close("AAPL", day(2026, 6, 1), "-500", "100")];
        let openings = vec![open("AAPL", day(2026, 6, 5), "30")];
        let hits = detect_hits(&closings, &openings);
        assert_eq!(hits[0].disallowed_loss_estimate, d("150.0"));
    }

    #[test]
    fn replacement_qty_exceeding_close_caps_at_full_loss() {
        // Bought 500 back vs only sold 100 — disallowed loss caps at
        // the entire $500 loss, not proportionally more.
        let closings = vec![losing_close("AAPL", day(2026, 6, 1), "-500", "100")];
        let openings = vec![open("AAPL", day(2026, 6, 5), "500")];
        let hits = detect_hits(&closings, &openings);
        assert_eq!(hits[0].disallowed_loss_estimate, d("500"));
    }

    #[test]
    fn boundary_exactly_at_30_days_flags() {
        let closings = vec![losing_close("AAPL", day(2026, 6, 1), "-500", "100")];
        let openings = vec![open("AAPL", day(2026, 7, 1), "100")];   // +30 days
        let hits = detect_hits(&closings, &openings);
        assert_eq!(hits.len(), 1, "30 days exactly is INSIDE the window");
    }

    #[test]
    fn total_disallowed_sums_correctly() {
        let losing = losing_close("AAPL", day(2026, 6, 1), "-500", "100");
        let openings = vec![
            open("AAPL", day(2026, 6, 5),  "100"),     // disallows 500
            open("AAPL", day(2026, 6, 20), "100"),     // disallows another 500
        ];
        let hits = detect_hits(&[losing], &openings);
        assert_eq!(total_disallowed(&hits), d("1000"));
    }

    #[test]
    fn empty_inputs_return_empty() {
        let hits = detect_hits(&[], &[]);
        assert!(hits.is_empty());
        assert_eq!(total_disallowed(&hits), Decimal::ZERO);
    }
}
