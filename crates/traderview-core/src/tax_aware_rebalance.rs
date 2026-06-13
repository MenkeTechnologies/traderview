//! Tax-aware rebalance — trade toward target weights while minimizing
//! the tax the rebalance realizes.
//!
//! The plain rebalancer only cares about weights; selling to hit a target
//! can realize a large capital gain it never reports. This plans the same
//! drift correction but, on the SELL side, chooses tax lots by strategy
//! (HIFO to minimize gain, MaxLossHarvest to realize losses) via the
//! shared [`crate::tax_lot_optimizer`], reports the gain each sale
//! triggers, and leaves holdings whose drift is within a no-trade band
//! untouched — trading them would realize tax for a trivial correction.

use crate::tax_lot_optimizer::{close, ClosingEntry, CostLot, LotStrategy};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Holding {
    pub symbol: String,
    /// Current price per share.
    pub price: Decimal,
    /// Target portfolio weight as a fraction in [0, 1].
    pub target_weight: Decimal,
    /// Open tax lots for this holding.
    pub lots: Vec<CostLot>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Action {
    pub symbol: String,
    pub current_value: Decimal,
    pub target_value: Decimal,
    /// Signed traded value: positive = buy, negative = sell, zero = held
    /// (within band). For sells it is the value actually sold, which can
    /// be less than the drift when MaxLossHarvest runs out of loss lots.
    pub trade_value: Decimal,
    /// Capital gain the sale realizes (zero for buys and holds; negative
    /// when harvesting losses).
    pub realized_gain: Decimal,
    pub closed_lots: Vec<ClosingEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Plan {
    pub total_value: Decimal,
    pub actions: Vec<Action>,
    pub total_realized_gain: Decimal,
    /// `tax_rate` applied to the NET realized gain. A net loss yields
    /// zero tax — it's a harvested benefit, not a liability.
    pub estimated_tax: Decimal,
}

/// Plan a tax-aware rebalance. `band` is the no-trade threshold as a
/// fraction of total value: a holding within `band` of its target is
/// left alone. `strategy` picks the sell lots; `tax_rate` (fraction)
/// estimates the tax on the net realized gain.
pub fn plan(
    holdings: &[Holding],
    strategy: LotStrategy,
    tax_rate: Decimal,
    band: Decimal,
) -> Plan {
    let total_value: Decimal = holdings
        .iter()
        .map(|h| shares(h) * h.price)
        .sum();

    let mut actions = Vec::with_capacity(holdings.len());
    let mut total_realized_gain = Decimal::ZERO;

    for h in holdings {
        let current_value = shares(h) * h.price;
        let target_value = h.target_weight * total_value;
        let drift = target_value - current_value;

        let within_band =
            total_value > Decimal::ZERO && (drift.abs() / total_value) < band;

        let (trade_value, realized_gain, closed_lots) = if within_band || drift.is_zero() {
            (Decimal::ZERO, Decimal::ZERO, Vec::new())
        } else if drift > Decimal::ZERO {
            // Underweight → buy. No tax event.
            (drift, Decimal::ZERO, Vec::new())
        } else if h.price > Decimal::ZERO {
            // Overweight → sell `drift` worth, choosing lots by strategy.
            let qty_to_sell = (-drift) / h.price;
            let report = close(&h.lots, qty_to_sell, h.price, strategy);
            let sold_qty = qty_to_sell - report.qty_remaining_to_close;
            (-(sold_qty * h.price), report.total_realized_gain, report.entries)
        } else {
            (Decimal::ZERO, Decimal::ZERO, Vec::new())
        };

        total_realized_gain += realized_gain;
        actions.push(Action {
            symbol: h.symbol.clone(),
            current_value,
            target_value,
            trade_value,
            realized_gain,
            closed_lots,
        });
    }

    let estimated_tax = total_realized_gain.max(Decimal::ZERO) * tax_rate;
    Plan { total_value, actions, total_realized_gain, estimated_tax }
}

fn shares(h: &Holding) -> Decimal {
    h.lots.iter().map(|l| l.qty_open).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(v: i64) -> Decimal {
        Decimal::from(v)
    }
    fn dq(s: &str) -> Decimal {
        s.parse().unwrap()
    }
    fn lot(id: &str, qty: i64, cost: i64) -> CostLot {
        CostLot { lot_id: id.into(), qty_open: d(qty), cost_per_share: d(cost) }
    }

    // 100 sh AAPL @ $100 (cost $50) = $10k; 100 sh MSFT @ $100 (cost $90)
    // = $10k. Total $20k, currently 50/50.
    fn two_holdings(aapl_target: &str, msft_target: &str) -> Vec<Holding> {
        vec![
            Holding {
                symbol: "AAPL".into(),
                price: d(100),
                target_weight: dq(aapl_target),
                lots: vec![lot("a1", 100, 50)],
            },
            Holding {
                symbol: "MSFT".into(),
                price: d(100),
                target_weight: dq(msft_target),
                lots: vec![lot("m1", 100, 90)],
            },
        ]
    }

    #[test]
    fn already_balanced_no_trades() {
        let p = plan(&two_holdings("0.5", "0.5"), LotStrategy::Hifo, dq("0.2"), Decimal::ZERO);
        assert_eq!(p.total_value, d(20_000));
        for a in &p.actions {
            assert_eq!(a.trade_value, Decimal::ZERO);
        }
        assert_eq!(p.total_realized_gain, Decimal::ZERO);
        assert_eq!(p.estimated_tax, Decimal::ZERO);
    }

    #[test]
    fn shift_to_70_30_sells_overweight_and_realizes_gain() {
        // Target AAPL 30% ($6k) / MSFT 70% ($14k). AAPL is overweight by
        // $4k → sell 40 sh @ $100, cost $50 → gain $50×40 = $2,000.
        let p = plan(&two_holdings("0.3", "0.7"), LotStrategy::Hifo, dq("0.2"), Decimal::ZERO);
        let aapl = p.actions.iter().find(|a| a.symbol == "AAPL").unwrap();
        let msft = p.actions.iter().find(|a| a.symbol == "MSFT").unwrap();
        assert_eq!(aapl.trade_value, d(-4_000)); // sold $4k
        assert_eq!(aapl.realized_gain, d(2_000)); // (100-50)*40
        assert_eq!(msft.trade_value, d(4_000)); // bought $4k, no gain
        assert_eq!(msft.realized_gain, Decimal::ZERO);
        assert_eq!(p.total_realized_gain, d(2_000));
    }

    #[test]
    fn tax_rate_applies_to_net_gain() {
        let p = plan(&two_holdings("0.3", "0.7"), LotStrategy::Hifo, dq("0.25"), Decimal::ZERO);
        assert_eq!(p.estimated_tax, d(500)); // 2000 * 0.25
    }

    #[test]
    fn loss_harvest_realizes_loss_and_zero_tax() {
        // MSFT cost $90, price $100 → no loss there. Build a losing lot.
        let holdings = vec![
            Holding {
                symbol: "TSLA".into(),
                price: d(100),
                target_weight: dq("0.5"),
                lots: vec![lot("t1", 100, 130)], // $130 cost, now $100 → loss
            },
            Holding {
                symbol: "NVDA".into(),
                price: d(100),
                target_weight: dq("0.5"),
                lots: vec![lot("n1", 100, 40)],
            },
        ];
        // Total $20k, 50/50 already — force a sell by shifting to 30/70.
        let mut h = holdings;
        h[0].target_weight = dq("0.3");
        h[1].target_weight = dq("0.7");
        let p = plan(&h, LotStrategy::MaxLossHarvest, dq("0.25"), Decimal::ZERO);
        let tsla = p.actions.iter().find(|a| a.symbol == "TSLA").unwrap();
        // Sells 40 sh of the $130 lot → loss (100-130)*40 = -1200.
        assert_eq!(tsla.realized_gain, d(-1_200));
        assert!(p.total_realized_gain < Decimal::ZERO);
        assert_eq!(p.estimated_tax, Decimal::ZERO); // a net loss is not taxed
    }

    #[test]
    fn no_trade_band_skips_small_drift() {
        // 52/48 vs a 50/50 target is 2% drift; a 5% band leaves it alone.
        let p = plan(&two_holdings("0.52", "0.48"), LotStrategy::Hifo, dq("0.25"), dq("0.05"));
        for a in &p.actions {
            assert_eq!(a.trade_value, Decimal::ZERO, "{} should be within band", a.symbol);
        }
    }
}
