//! Cost-basis lot accounting + tax-optimal lot selection.
//!
//! When a position has been built in multiple lots at different prices,
//! the seller chooses which lots to close. The selection method
//! determines realized cap gain/loss:
//!
//!   - **FIFO** (default IRS): close oldest lots first.
//!   - **LIFO**: close newest lots first.
//!   - **HIFO**: close highest-cost lots first (minimize gain / max loss).
//!   - **LOFO**: close lowest-cost lots first (maximize gain — for
//!     tax-loss carryforward situations).
//!   - **Specific Lot ID**: caller passes the exact lots to close.
//!
//! Pure compute.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostLot {
    pub lot_id: String,
    pub acquired: NaiveDate,
    pub qty: Decimal,
    pub cost_per_share: Decimal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LotMethod {
    Fifo,
    Lifo,
    Hifo,
    Lofo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClosingEntry {
    pub lot_id: String,
    pub qty_closed: Decimal,
    pub cost_per_share: Decimal,
    pub realized_per_share: Decimal,
    pub realized_total: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CloseReport {
    pub closes: Vec<ClosingEntry>,
    pub total_realized: Decimal,
    pub qty_remaining_to_close: Decimal,
}

pub fn close(
    lots: &[CostLot],
    qty_to_close: Decimal,
    price_per_share: Decimal,
    method: LotMethod,
) -> CloseReport {
    let mut report = CloseReport {
        qty_remaining_to_close: qty_to_close,
        ..Default::default()
    };
    if lots.is_empty() || qty_to_close <= Decimal::ZERO {
        return report;
    }
    let mut sorted: Vec<&CostLot> = lots.iter().collect();
    sorted.sort_by(|a, b| match method {
        LotMethod::Fifo => a.acquired.cmp(&b.acquired),
        LotMethod::Lifo => b.acquired.cmp(&a.acquired),
        LotMethod::Hifo => b.cost_per_share.cmp(&a.cost_per_share),
        LotMethod::Lofo => a.cost_per_share.cmp(&b.cost_per_share),
    });
    let mut remaining = qty_to_close;
    for lot in sorted {
        if remaining <= Decimal::ZERO {
            break;
        }
        let take = remaining.min(lot.qty);
        let realized_per_share = price_per_share - lot.cost_per_share;
        let realized_total = realized_per_share * take;
        report.closes.push(ClosingEntry {
            lot_id: lot.lot_id.clone(),
            qty_closed: take,
            cost_per_share: lot.cost_per_share,
            realized_per_share,
            realized_total,
        });
        report.total_realized += realized_total;
        remaining -= take;
    }
    report.qty_remaining_to_close = remaining;
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn d(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }
    fn day(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    fn lots() -> Vec<CostLot> {
        vec![
            CostLot {
                lot_id: "A".into(),
                acquired: day(2024, 1, 15),
                qty: d("100"),
                cost_per_share: d("100"),
            },
            CostLot {
                lot_id: "B".into(),
                acquired: day(2024, 6, 10),
                qty: d("100"),
                cost_per_share: d("150"),
            },
            CostLot {
                lot_id: "C".into(),
                acquired: day(2025, 3, 5),
                qty: d("100"),
                cost_per_share: d("125"),
            },
        ]
    }

    #[test]
    fn empty_lots_returns_default() {
        let r = close(&[], d("100"), d("200"), LotMethod::Fifo);
        assert!(r.closes.is_empty());
    }

    #[test]
    fn fifo_takes_oldest_first() {
        // Sell 100 shares at $200. FIFO → lot A first (cost $100).
        // Realized per share = $100. Total = $10,000.
        let r = close(&lots(), d("100"), d("200"), LotMethod::Fifo);
        assert_eq!(r.closes.len(), 1);
        assert_eq!(r.closes[0].lot_id, "A");
        assert_eq!(r.total_realized, d("10000"));
    }

    #[test]
    fn lifo_takes_newest_first() {
        // LIFO → lot C first (cost $125). Realized = ($200 - $125) × 100 = $7500.
        let r = close(&lots(), d("100"), d("200"), LotMethod::Lifo);
        assert_eq!(r.closes[0].lot_id, "C");
        assert_eq!(r.total_realized, d("7500"));
    }

    #[test]
    fn hifo_takes_highest_cost_first() {
        // HIFO → lot B (cost $150) first. Realized = ($200 - $150) × 100 = $5000.
        let r = close(&lots(), d("100"), d("200"), LotMethod::Hifo);
        assert_eq!(r.closes[0].lot_id, "B");
        assert_eq!(r.total_realized, d("5000"));
    }

    #[test]
    fn lofo_takes_lowest_cost_first_maximizing_gain() {
        // LOFO → lot A (cost $100) first. Realized = $10,000.
        let r = close(&lots(), d("100"), d("200"), LotMethod::Lofo);
        assert_eq!(r.closes[0].lot_id, "A");
        assert_eq!(r.total_realized, d("10000"));
    }

    #[test]
    fn close_more_than_one_lot_spans_multiple_lots() {
        // Sell 250 shares. FIFO order: A (100), B (100), C (50).
        let r = close(&lots(), d("250"), d("200"), LotMethod::Fifo);
        assert_eq!(r.closes.len(), 3);
        assert_eq!(r.closes[0].qty_closed, d("100"));
        assert_eq!(r.closes[1].qty_closed, d("100"));
        assert_eq!(r.closes[2].qty_closed, d("50"));
        // Total realized:
        //   A: ($200-$100)×100 = 10,000.
        //   B: ($200-$150)×100 = 5,000.
        //   C: ($200-$125)×50 = 3,750.
        //   Total = 18,750.
        assert_eq!(r.total_realized, d("18750"));
    }

    #[test]
    fn over_close_leaves_remaining_quantity() {
        // Total lots = 300; try to close 500.
        let r = close(&lots(), d("500"), d("200"), LotMethod::Fifo);
        assert_eq!(r.qty_remaining_to_close, d("200"));
    }

    #[test]
    fn hifo_minimizes_gain_when_selling_winners() {
        // Compare HIFO vs LOFO on a gain — HIFO yields LESS gain.
        let hifo = close(&lots(), d("100"), d("200"), LotMethod::Hifo);
        let lofo = close(&lots(), d("100"), d("200"), LotMethod::Lofo);
        assert!(hifo.total_realized < lofo.total_realized);
    }

    #[test]
    fn partial_lot_close_keeps_remaining_in_lot() {
        // Sell 50 shares. FIFO takes 50 from lot A.
        let r = close(&lots(), d("50"), d("200"), LotMethod::Fifo);
        assert_eq!(r.closes.len(), 1);
        assert_eq!(r.closes[0].qty_closed, d("50"));
        // Lot A still has 50 unclosed shares — caller manages remaining state.
    }
}
