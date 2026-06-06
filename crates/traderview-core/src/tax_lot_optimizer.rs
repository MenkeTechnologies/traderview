//! Tax-lot optimizer — pick which lots to sell to minimize realized gain.
//!
//! Existing `cost_basis.rs` handles FIFO and LIFO. This module adds:
//!
//!   - **HIFO** (Highest-In-First-Out): sell the highest-cost lots first to
//!     realize the smallest gain (or biggest loss). Default lot-selection
//!     strategy for tax-aware traders.
//!   - **LowestGain**: same idea but explicit — sort lots by realized-gain
//!     per share and consume cheapest-gain lots first.
//!   - **MaxLossHarvest**: sell only lots that would realize a LOSS, ranked
//!     by largest loss. Stops short when no losing lots remain. Used during
//!     tax-loss harvesting season.
//!
//! Returns the per-lot consumption plan + total realized gain/loss so the
//! caller can compare to FIFO/LIFO outcomes and pick.
//!
//! Pure compute. Distinct from cost_basis::close (which only routes through
//! FIFO/LIFO via the rollup::LotMethod enum).

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostLot {
    pub lot_id: String,
    pub qty_open: Decimal,
    pub cost_per_share: Decimal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LotStrategy {
    /// Highest cost-per-share consumed first (minimize gain).
    Hifo,
    /// Lowest cost-per-share first (maximize gain — useful if you're
    /// trying to recognize capital gain in a low-bracket year).
    Lifoust,
    /// Only consume lots that produce a LOSS (cost > sell_price), ranked
    /// largest loss first. Stops if no more losing lots.
    MaxLossHarvest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClosingEntry {
    pub lot_id: String,
    pub qty_consumed: Decimal,
    pub cost_per_share: Decimal,
    pub realized_gain: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CloseReport {
    pub entries: Vec<ClosingEntry>,
    pub total_realized_gain: Decimal,
    pub qty_remaining_to_close: Decimal,
    /// True when the requested close-qty couldn't be fully filled — either
    /// the lots ran out (Hifo / Lifoust) or no more losing lots existed
    /// (MaxLossHarvest).
    pub partial: bool,
}

pub fn close(
    lots: &[CostLot],
    qty_to_close: Decimal,
    sell_price: Decimal,
    strategy: LotStrategy,
) -> CloseReport {
    if qty_to_close <= Decimal::ZERO || lots.is_empty() {
        return CloseReport {
            qty_remaining_to_close: qty_to_close,
            partial: qty_to_close > Decimal::ZERO,
            ..Default::default()
        };
    }
    // Sort a working copy by the strategy's selection order.
    let mut working: Vec<CostLot> = lots.to_vec();
    match strategy {
        LotStrategy::Hifo => {
            working.sort_by_key(|a| std::cmp::Reverse(a.cost_per_share));
        }
        LotStrategy::Lifoust => {
            working.sort_by_key(|a| a.cost_per_share);
        }
        LotStrategy::MaxLossHarvest => {
            // Only keep lots that would realize a LOSS at sell_price.
            working.retain(|l| l.cost_per_share > sell_price);
            // Among losing lots, rank by largest loss per share.
            working.sort_by_key(|a| std::cmp::Reverse(a.cost_per_share));
        }
    }
    let mut remaining = qty_to_close;
    let mut entries = Vec::new();
    let mut total_gain = Decimal::ZERO;
    for lot in working {
        if remaining <= Decimal::ZERO {
            break;
        }
        let take = remaining.min(lot.qty_open);
        let gain_per_share = sell_price - lot.cost_per_share;
        let gain = take * gain_per_share;
        entries.push(ClosingEntry {
            lot_id: lot.lot_id,
            qty_consumed: take,
            cost_per_share: lot.cost_per_share,
            realized_gain: gain,
        });
        total_gain += gain;
        remaining -= take;
    }
    CloseReport {
        entries,
        total_realized_gain: total_gain,
        qty_remaining_to_close: remaining,
        partial: remaining > Decimal::ZERO,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn dec(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }
    fn lot(id: &str, qty: &str, cost: &str) -> CostLot {
        CostLot {
            lot_id: id.into(),
            qty_open: dec(qty),
            cost_per_share: dec(cost),
        }
    }

    #[test]
    fn empty_lots_returns_full_remaining() {
        let r = close(&[], dec("10"), dec("100"), LotStrategy::Hifo);
        assert!(r.entries.is_empty());
        assert_eq!(r.qty_remaining_to_close, dec("10"));
        assert!(r.partial);
    }

    #[test]
    fn zero_qty_to_close_is_a_noop() {
        let lots = vec![lot("a", "5", "50")];
        let r = close(&lots, dec("0"), dec("60"), LotStrategy::Hifo);
        assert!(r.entries.is_empty());
        assert_eq!(r.total_realized_gain, Decimal::ZERO);
        assert!(!r.partial);
    }

    #[test]
    fn hifo_consumes_highest_cost_first_to_minimize_gain() {
        // 3 lots @ $50, $80, $100. Sell 1 share at $90 → HIFO picks $100 lot
        // (loss of $10). FIFO would pick $50 (gain of $40). HIFO is the
        // tax-minimizing choice.
        let lots = vec![
            lot("low", "1", "50"),
            lot("mid", "1", "80"),
            lot("high", "1", "100"),
        ];
        let r = close(&lots, dec("1"), dec("90"), LotStrategy::Hifo);
        assert_eq!(r.entries.len(), 1);
        assert_eq!(r.entries[0].lot_id, "high");
        assert_eq!(
            r.total_realized_gain,
            dec("-10"),
            "HIFO at sell=90 gives -10 from $100 cost"
        );
    }

    #[test]
    fn lifoust_consumes_lowest_cost_first_to_maximize_gain() {
        let lots = vec![
            lot("low", "1", "50"),
            lot("mid", "1", "80"),
            lot("high", "1", "100"),
        ];
        let r = close(&lots, dec("1"), dec("90"), LotStrategy::Lifoust);
        assert_eq!(r.entries[0].lot_id, "low");
        assert_eq!(r.total_realized_gain, dec("40"));
    }

    #[test]
    fn max_loss_harvest_only_consumes_losing_lots() {
        // At sell=90: lot at $50 = +$40 gain (skip), $80 = +$10 gain (skip),
        // $100 = -$10 loss (eligible). Asked for 5 qty but only 1 losing
        // share exists — partial fill.
        let lots = vec![
            lot("low", "5", "50"),
            lot("mid", "5", "80"),
            lot("high", "1", "100"),
        ];
        let r = close(&lots, dec("5"), dec("90"), LotStrategy::MaxLossHarvest);
        assert_eq!(r.entries.len(), 1);
        assert_eq!(r.entries[0].lot_id, "high");
        assert_eq!(r.total_realized_gain, dec("-10"));
        assert_eq!(r.qty_remaining_to_close, dec("4"));
        assert!(r.partial);
    }

    #[test]
    fn max_loss_harvest_with_no_losing_lots_consumes_nothing() {
        let lots = vec![lot("low", "5", "50"), lot("mid", "5", "60")];
        let r = close(&lots, dec("3"), dec("100"), LotStrategy::MaxLossHarvest);
        assert!(r.entries.is_empty());
        assert_eq!(r.qty_remaining_to_close, dec("3"));
        assert!(r.partial);
    }

    #[test]
    fn hifo_spans_multiple_lots_when_needed() {
        // 2 lots × 5 shares each. Sell 7 shares → consumes the $100 lot
        // fully (5) + 2 from the $80 lot (the next-highest cost).
        let lots = vec![lot("a", "5", "80"), lot("b", "5", "100")];
        let r = close(&lots, dec("7"), dec("90"), LotStrategy::Hifo);
        assert_eq!(r.entries.len(), 2);
        assert_eq!(r.entries[0].lot_id, "b");
        assert_eq!(r.entries[0].qty_consumed, dec("5"));
        assert_eq!(r.entries[1].lot_id, "a");
        assert_eq!(r.entries[1].qty_consumed, dec("2"));
        // Gain: 5×(90-100) + 2×(90-80) = -50 + 20 = -30.
        assert_eq!(r.total_realized_gain, dec("-30"));
        assert!(!r.partial);
    }
}
