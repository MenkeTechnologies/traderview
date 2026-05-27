//! FIFO trade roll-up.
//!
//! Per `(account_id, symbol, asset_class, option_leg)`:
//!  1. Sort executions by `executed_at` ASC.
//!  2. Maintain an open-lot queue per direction (long or short).
//!  3. Opens (Buy → long, Short → short) push to the queue.
//!  4. Closes (Sell against long, Cover against short) pop FIFO and record
//!     `(open_exec, close_exec, qty_consumed)` legs.
//!  5. A trade starts when a direction transitions 0 → nonzero and ends when
//!     it returns to 0. Crossing through 0 (e.g. selling more than open)
//!     reverses direction and starts a new trade.

use crate::models::{
    AssetClass, Execution, OptionType, Side, Trade, TradeExecutionLink, TradeSide, TradeStatus,
};
use crate::pnl::{gross_pnl, PricePoint};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::collections::VecDeque;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LotMethod {
    Fifo,
    Lifo,
}

#[derive(Debug, thiserror::Error)]
pub enum RollupError {
    #[error("execution {0} has zero or negative qty")]
    BadQty(Uuid),
}

#[derive(Debug, Clone)]
pub struct RolledTrade {
    pub trade: Trade,
    pub legs: Vec<TradeExecutionLink>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)] // exec_id/fee_per_unit/executed_at preserved for future leg attribution
struct OpenLot {
    exec_id: Uuid,
    qty_open: Decimal,
    price: Decimal,
    fee_per_unit: Decimal,
    executed_at: chrono::DateTime<chrono::Utc>,
}

/// Group key — same (account, symbol, asset_class, option-leg) collapses to
/// one roll-up stream. Each option leg is its own stream.
type GroupKey = (
    Uuid,
    String,
    AssetClass,
    Option<OptionType>,
    Option<NaiveDate>,
    Option<Decimal>,
);

fn key(e: &Execution) -> GroupKey {
    (
        e.account_id,
        e.symbol.clone(),
        e.asset_class,
        e.option_type,
        e.expiration,
        e.strike,
    )
}

/// Run a FIFO roll-up over the provided executions and return the resulting
/// trades + leg links. Input may be unsorted; the function sorts internally.
pub fn rollup(executions: &[Execution], method: LotMethod) -> Result<Vec<RolledTrade>, RollupError> {
    // Validate qty (the DB CHECK already enforces this, but defensive).
    for e in executions {
        if e.qty <= Decimal::ZERO {
            return Err(RollupError::BadQty(e.id));
        }
    }

    // Sort by executed_at; tiebreak by id for determinism.
    let mut sorted: Vec<&Execution> = executions.iter().collect();
    sorted.sort_by(|a, b| {
        a.executed_at
            .cmp(&b.executed_at)
            .then_with(|| a.id.cmp(&b.id))
    });

    // Bucket per group.
    let mut groups: std::collections::BTreeMap<GroupKey, Vec<&Execution>> =
        std::collections::BTreeMap::new();
    for e in sorted {
        groups.entry(key(e)).or_default().push(e);
    }

    let mut out = Vec::new();
    for ((_acct, _sym, _ac, _ot, _exp, _strk), execs) in groups {
        rollup_one_group(&execs, method, &mut out);
    }
    Ok(out)
}

fn rollup_one_group(
    execs: &[&Execution],
    method: LotMethod,
    out: &mut Vec<RolledTrade>,
) {
    // Two queues — only one is non-empty at a time in a non-pathological feed,
    // but we tolerate both being populated (which would indicate ambiguous flips).
    let mut long_lots: VecDeque<OpenLot> = VecDeque::new();
    let mut short_lots: VecDeque<OpenLot> = VecDeque::new();

    // The trade currently being built (per direction).
    let mut current_long: Option<TradeBuilder> = None;
    let mut current_short: Option<TradeBuilder> = None;

    for e in execs {
        let fee_per_unit = if e.qty.is_zero() {
            Decimal::ZERO
        } else {
            e.fee / e.qty
        };

        match e.side {
            Side::Buy => {
                // Either opening a long, OR covering an existing short.
                let mut remaining = e.qty;
                if !short_lots.is_empty() {
                    remaining = close_against(
                        &mut short_lots,
                        TradeSide::Short,
                        e,
                        remaining,
                        fee_per_unit,
                        &mut current_short,
                        out,
                        method,
                    );
                }
                if remaining > Decimal::ZERO {
                    let lot = OpenLot {
                        exec_id: e.id,
                        qty_open: remaining,
                        price: e.price,
                        fee_per_unit,
                        executed_at: e.executed_at,
                    };
                    if current_long.is_none() {
                        current_long = Some(TradeBuilder::new(e, TradeSide::Long));
                    }
                    current_long
                        .as_mut()
                        .unwrap()
                        .observe_open(e, remaining, fee_per_unit);
                    long_lots.push_back(lot);
                }
            }
            Side::Short => {
                let mut remaining = e.qty;
                if !long_lots.is_empty() {
                    remaining = close_against(
                        &mut long_lots,
                        TradeSide::Long,
                        e,
                        remaining,
                        fee_per_unit,
                        &mut current_long,
                        out,
                        method,
                    );
                }
                if remaining > Decimal::ZERO {
                    let lot = OpenLot {
                        exec_id: e.id,
                        qty_open: remaining,
                        price: e.price,
                        fee_per_unit,
                        executed_at: e.executed_at,
                    };
                    if current_short.is_none() {
                        current_short = Some(TradeBuilder::new(e, TradeSide::Short));
                    }
                    current_short
                        .as_mut()
                        .unwrap()
                        .observe_open(e, remaining, fee_per_unit);
                    short_lots.push_back(lot);
                }
            }
            Side::Sell => {
                // Closes long; if it exceeds open longs, the excess starts a short.
                let mut remaining = e.qty;
                if !long_lots.is_empty() {
                    remaining = close_against(
                        &mut long_lots,
                        TradeSide::Long,
                        e,
                        remaining,
                        fee_per_unit,
                        &mut current_long,
                        out,
                        method,
                    );
                }
                if remaining > Decimal::ZERO {
                    // Flipped short.
                    let lot = OpenLot {
                        exec_id: e.id,
                        qty_open: remaining,
                        price: e.price,
                        fee_per_unit,
                        executed_at: e.executed_at,
                    };
                    if current_short.is_none() {
                        current_short = Some(TradeBuilder::new(e, TradeSide::Short));
                    }
                    current_short
                        .as_mut()
                        .unwrap()
                        .observe_open(e, remaining, fee_per_unit);
                    short_lots.push_back(lot);
                }
            }
            Side::Cover => {
                let mut remaining = e.qty;
                if !short_lots.is_empty() {
                    remaining = close_against(
                        &mut short_lots,
                        TradeSide::Short,
                        e,
                        remaining,
                        fee_per_unit,
                        &mut current_short,
                        out,
                        method,
                    );
                }
                if remaining > Decimal::ZERO {
                    let lot = OpenLot {
                        exec_id: e.id,
                        qty_open: remaining,
                        price: e.price,
                        fee_per_unit,
                        executed_at: e.executed_at,
                    };
                    if current_long.is_none() {
                        current_long = Some(TradeBuilder::new(e, TradeSide::Long));
                    }
                    current_long
                        .as_mut()
                        .unwrap()
                        .observe_open(e, remaining, fee_per_unit);
                    long_lots.push_back(lot);
                }
            }
        }
    }

    // Any remaining open lots → open trades.
    if let Some(b) = current_long {
        if !long_lots.is_empty() {
            out.push(b.finalize_open(&long_lots));
        }
    }
    if let Some(b) = current_short {
        if !short_lots.is_empty() {
            out.push(b.finalize_open(&short_lots));
        }
    }
}

/// Close `closing_qty` units against `lots` (which represent open positions
/// in `open_side`). Returns any leftover qty that overflowed (i.e. closed more
/// than was open — caller treats as a reversal).
#[allow(clippy::too_many_arguments)]
fn close_against(
    lots: &mut VecDeque<OpenLot>,
    open_side: TradeSide,
    closing_exec: &Execution,
    mut closing_qty: Decimal,
    closing_fee_per_unit: Decimal,
    current_holder: &mut Option<TradeBuilder>,
    out: &mut Vec<RolledTrade>,
    method: LotMethod,
) -> Decimal {
    while closing_qty > Decimal::ZERO {
        let lot = match method {
            LotMethod::Fifo => lots.front_mut(),
            LotMethod::Lifo => lots.back_mut(),
        };
        let Some(lot) = lot else { break };

        let take = if lot.qty_open <= closing_qty {
            lot.qty_open
        } else {
            closing_qty
        };

        // Record leg on the active builder.
        {
            let b = current_holder
                .as_mut()
                .expect("non-empty lot queue implies an active builder");
            b.observe_close_leg(lot, closing_exec, take, closing_fee_per_unit, open_side);
        }

        lot.qty_open -= take;
        closing_qty -= take;
        if lot.qty_open.is_zero() {
            match method {
                LotMethod::Fifo => {
                    lots.pop_front();
                }
                LotMethod::Lifo => {
                    lots.pop_back();
                }
            }
        }

        if lots.is_empty() {
            // Trade is fully closed — take ownership and finalize.
            let finalized = current_holder
                .take()
                .expect("we asserted Some above")
                .finalize_closed();
            out.push(finalized);
            break;
        }
    }
    closing_qty
}

#[derive(Debug, Clone)]
struct TradeBuilder {
    id: Uuid,
    account_id: Uuid,
    symbol: String,
    side: TradeSide,
    asset_class: AssetClass,
    option_type: Option<OptionType>,
    strike: Option<Decimal>,
    expiration: Option<NaiveDate>,
    multiplier: Decimal,
    tick_size: Option<Decimal>,
    tick_value: Option<Decimal>,
    base_ccy: Option<String>,
    quote_ccy: Option<String>,
    pip_size: Option<Decimal>,
    opened_at: chrono::DateTime<chrono::Utc>,
    last_close_at: Option<chrono::DateTime<chrono::Utc>>,
    qty_total: Decimal, // sum of opens
    notional_in: Decimal,
    notional_out: Decimal,
    qty_closed: Decimal,
    gross_pnl: Decimal,
    fees: Decimal,
    legs: Vec<TradeExecutionLink>,
}

impl TradeBuilder {
    fn new(first_open: &Execution, side: TradeSide) -> Self {
        TradeBuilder {
            id: Uuid::new_v4(),
            account_id: first_open.account_id,
            symbol: first_open.symbol.clone(),
            side,
            asset_class: first_open.asset_class,
            option_type: first_open.option_type,
            strike: first_open.strike,
            expiration: first_open.expiration,
            multiplier: first_open.multiplier,
            tick_size: first_open.tick_size,
            tick_value: first_open.tick_value,
            base_ccy: first_open.base_ccy.clone(),
            quote_ccy: first_open.quote_ccy.clone(),
            pip_size: first_open.pip_size,
            opened_at: first_open.executed_at,
            last_close_at: None,
            qty_total: Decimal::ZERO,
            notional_in: Decimal::ZERO,
            notional_out: Decimal::ZERO,
            qty_closed: Decimal::ZERO,
            gross_pnl: Decimal::ZERO,
            fees: Decimal::ZERO,
            legs: Vec::new(),
        }
    }

    fn observe_open(&mut self, e: &Execution, qty: Decimal, fee_per_unit: Decimal) {
        self.qty_total += qty;
        self.notional_in += e.price * qty;
        self.fees += fee_per_unit * qty;
        self.legs.push(TradeExecutionLink {
            trade_id: self.id,
            execution_id: e.id,
            qty_used: qty,
        });
    }

    fn observe_close_leg(
        &mut self,
        open_lot: &OpenLot,
        close_exec: &Execution,
        qty: Decimal,
        close_fee_per_unit: Decimal,
        open_side: TradeSide,
    ) {
        let pnl = gross_pnl(
            self.asset_class,
            open_side,
            qty,
            PricePoint {
                entry: open_lot.price,
                exit: close_exec.price,
                multiplier: self.multiplier,
                tick_size: self.tick_size,
                tick_value: self.tick_value,
            },
        );
        self.gross_pnl += pnl;
        self.notional_out += close_exec.price * qty;
        self.qty_closed += qty;
        // Open-side fees were already accrued in `observe_open` when the lot was
        // created; only add the close-side fee here.
        self.fees += close_fee_per_unit * qty;
        self.last_close_at = Some(close_exec.executed_at);
        self.legs.push(TradeExecutionLink {
            trade_id: self.id,
            execution_id: close_exec.id,
            qty_used: qty,
        });
    }

    fn finalize_closed(self) -> RolledTrade {
        let qty = self.qty_closed;
        let entry_avg = if qty.is_zero() {
            Decimal::ZERO
        } else {
            self.notional_in / qty
        };
        let exit_avg = if qty.is_zero() {
            None
        } else {
            Some(self.notional_out / qty)
        };
        let net = self.gross_pnl - self.fees;
        let trade = Trade {
            id: self.id,
            account_id: self.account_id,
            symbol: self.symbol,
            side: self.side,
            status: TradeStatus::Closed,
            opened_at: self.opened_at,
            closed_at: self.last_close_at,
            qty,
            entry_avg,
            exit_avg,
            gross_pnl: Some(self.gross_pnl),
            fees: self.fees,
            net_pnl: Some(net),
            asset_class: self.asset_class,
            option_type: self.option_type,
            strike: self.strike,
            expiration: self.expiration,
            multiplier: self.multiplier,
            tick_size: self.tick_size,
            tick_value: self.tick_value,
            base_ccy: self.base_ccy,
            quote_ccy: self.quote_ccy,
            pip_size: self.pip_size,
            stop_loss: None,
            risk_amount: None,
            initial_target: None,
            mfe: None,
            mae: None,
            best_exit_pnl: None,
            exit_efficiency: None,
        };
        RolledTrade {
            trade,
            legs: self.legs,
        }
    }

    fn finalize_open(self, _remaining: &VecDeque<OpenLot>) -> RolledTrade {
        let qty = self.qty_total - self.qty_closed;
        let entry_avg = if self.qty_total.is_zero() {
            Decimal::ZERO
        } else {
            self.notional_in / self.qty_total
        };
        let trade = Trade {
            id: self.id,
            account_id: self.account_id,
            symbol: self.symbol,
            side: self.side,
            status: TradeStatus::Open,
            opened_at: self.opened_at,
            closed_at: None,
            qty,
            entry_avg,
            exit_avg: None,
            gross_pnl: None,
            fees: self.fees,
            net_pnl: None,
            asset_class: self.asset_class,
            option_type: self.option_type,
            strike: self.strike,
            expiration: self.expiration,
            multiplier: self.multiplier,
            tick_size: self.tick_size,
            tick_value: self.tick_value,
            base_ccy: self.base_ccy,
            quote_ccy: self.quote_ccy,
            pip_size: self.pip_size,
            stop_loss: None,
            risk_amount: None,
            initial_target: None,
            mfe: None,
            mae: None,
            best_exit_pnl: None,
            exit_efficiency: None,
        };
        RolledTrade {
            trade,
            legs: self.legs,
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    fn exec(
        symbol: &str,
        side: Side,
        qty: u64,
        price: u64,
        fee: u64,
        ts: i64,
    ) -> Execution {
        Execution {
            id: Uuid::new_v4(),
            account_id: Uuid::nil(),
            symbol: symbol.into(),
            side,
            qty: Decimal::from(qty),
            price: Decimal::from(price),
            fee: Decimal::from(fee),
            executed_at: Utc.timestamp_opt(ts, 0).unwrap(),
            broker_order_id: None,
            raw: serde_json::json!({}),
            import_id: None,
            asset_class: AssetClass::Stock,
            option_type: None,
            strike: None,
            expiration: None,
            multiplier: Decimal::ONE,
            tick_size: None,
            tick_value: None,
            base_ccy: None,
            quote_ccy: None,
            pip_size: None,
        }
    }

    #[test]
    fn single_round_trip_long() {
        let execs = vec![
            exec("AAPL", Side::Buy, 100, 150, 1, 1_000),
            exec("AAPL", Side::Sell, 100, 160, 1, 2_000),
        ];
        let trades = rollup(&execs, LotMethod::Fifo).unwrap();
        assert_eq!(trades.len(), 1);
        let t = &trades[0].trade;
        assert_eq!(t.side, TradeSide::Long);
        assert_eq!(t.status, TradeStatus::Closed);
        assert_eq!(t.qty, Decimal::from(100));
        assert_eq!(t.gross_pnl, Some(Decimal::from(1_000)));
        assert_eq!(t.net_pnl, Some(Decimal::from(998)));
    }

    #[test]
    fn partial_close_keeps_trade_open() {
        let execs = vec![
            exec("MSFT", Side::Buy, 100, 200, 0, 1_000),
            exec("MSFT", Side::Sell, 40, 210, 0, 2_000),
        ];
        let trades = rollup(&execs, LotMethod::Fifo).unwrap();
        assert_eq!(trades.len(), 1);
        let t = &trades[0].trade;
        assert_eq!(t.status, TradeStatus::Open);
        assert_eq!(t.qty, Decimal::from(60));
    }

    #[test]
    fn fifo_orders_two_buys_one_sell() {
        let execs = vec![
            exec("GOOG", Side::Buy, 50, 100, 0, 1_000),
            exec("GOOG", Side::Buy, 50, 120, 0, 2_000),
            exec("GOOG", Side::Sell, 100, 130, 0, 3_000),
        ];
        let trades = rollup(&execs, LotMethod::Fifo).unwrap();
        assert_eq!(trades.len(), 1);
        let t = &trades[0].trade;
        // Avg entry over 100 shares = (50*100 + 50*120)/100 = 110
        assert_eq!(t.entry_avg, Decimal::from(110));
        // P&L = (130 - 100) * 50 + (130 - 120) * 50 = 1500 + 500 = 2000
        assert_eq!(t.gross_pnl, Some(Decimal::from(2_000)));
    }

    #[test]
    fn short_round_trip() {
        let execs = vec![
            exec("TSLA", Side::Short, 10, 300, 0, 1_000),
            exec("TSLA", Side::Cover, 10, 280, 0, 2_000),
        ];
        let trades = rollup(&execs, LotMethod::Fifo).unwrap();
        assert_eq!(trades.len(), 1);
        let t = &trades[0].trade;
        assert_eq!(t.side, TradeSide::Short);
        assert_eq!(t.gross_pnl, Some(Decimal::from(200))); // (300-280)*10
    }

    #[test]
    fn sell_overshoot_flips_to_short() {
        let execs = vec![
            exec("NVDA", Side::Buy, 50, 100, 0, 1_000),
            exec("NVDA", Side::Sell, 100, 110, 0, 2_000), // closes long, opens 50 short
            exec("NVDA", Side::Cover, 50, 105, 0, 3_000),
        ];
        let trades = rollup(&execs, LotMethod::Fifo).unwrap();
        // Expect two trades: closed long + closed short
        assert_eq!(trades.len(), 2);
        assert_eq!(trades[0].trade.side, TradeSide::Long);
        assert_eq!(trades[0].trade.gross_pnl, Some(Decimal::from(500))); // 50*(110-100)
        assert_eq!(trades[1].trade.side, TradeSide::Short);
        assert_eq!(trades[1].trade.gross_pnl, Some(Decimal::from(250))); // 50*(110-105)
    }

    #[test]
    fn option_multiplier_applies() {
        let mut e1 = exec("SPY 250630C500", Side::Buy, 1, 5, 0, 1_000);
        let mut e2 = exec("SPY 250630C500", Side::Sell, 1, 7, 0, 2_000);
        e1.asset_class = AssetClass::Option;
        e2.asset_class = AssetClass::Option;
        e1.multiplier = Decimal::from(100);
        e2.multiplier = Decimal::from(100);
        e1.option_type = Some(OptionType::Call);
        e2.option_type = Some(OptionType::Call);
        let trades = rollup(&[e1, e2], LotMethod::Fifo).unwrap();
        assert_eq!(trades.len(), 1);
        // 1 contract * 100 multiplier * (7 - 5) = 200
        assert_eq!(trades[0].trade.gross_pnl, Some(Decimal::from(200)));
    }

    // ─── Edge cases (added after the 4-pass audit) ────────────────────────

    #[test]
    fn empty_input_returns_no_trades() {
        let trades = rollup(&[], LotMethod::Fifo).unwrap();
        assert!(trades.is_empty());
    }

    #[test]
    fn fees_are_summed_into_net_pnl() {
        // Gross = 1000, fees = 5 + 5 → net = 990.
        let execs = vec![
            exec("AAPL", Side::Buy,  100, 150, 5, 1_000),
            exec("AAPL", Side::Sell, 100, 160, 5, 2_000),
        ];
        let t = &rollup(&execs, LotMethod::Fifo).unwrap()[0].trade;
        assert_eq!(t.gross_pnl, Some(Decimal::from(1_000)));
        assert_eq!(t.net_pnl,   Some(Decimal::from(990)));
        assert_eq!(t.fees,      Decimal::from(10));
    }

    #[test]
    fn losing_long_records_negative_pnl() {
        // Bought at 150, sold at 140 → -1000 gross.
        let execs = vec![
            exec("AAPL", Side::Buy,  100, 150, 0, 1_000),
            exec("AAPL", Side::Sell, 100, 140, 0, 2_000),
        ];
        let t = &rollup(&execs, LotMethod::Fifo).unwrap()[0].trade;
        assert_eq!(t.gross_pnl, Some(Decimal::from(-1_000)));
        assert_eq!(t.side, TradeSide::Long);
    }

    #[test]
    fn losing_short_records_negative_pnl() {
        // Shorted at 300, covered at 320 → -200 gross.
        let execs = vec![
            exec("TSLA", Side::Short, 10, 300, 0, 1_000),
            exec("TSLA", Side::Cover, 10, 320, 0, 2_000),
        ];
        let t = &rollup(&execs, LotMethod::Fifo).unwrap()[0].trade;
        assert_eq!(t.gross_pnl, Some(Decimal::from(-200)));
        assert_eq!(t.side, TradeSide::Short);
    }

    #[test]
    fn multiple_round_trips_same_symbol_yield_separate_trades() {
        // Two independent round trips on AAPL — must produce two CLOSED trades,
        // not one merged one. Otherwise per-trade P&L reporting is broken.
        let execs = vec![
            exec("AAPL", Side::Buy,  100, 150, 0, 1_000),
            exec("AAPL", Side::Sell, 100, 155, 0, 2_000),  // close trade #1
            exec("AAPL", Side::Buy,  100, 160, 0, 3_000),  // open trade #2
            exec("AAPL", Side::Sell, 100, 170, 0, 4_000),  // close trade #2
        ];
        let trades = rollup(&execs, LotMethod::Fifo).unwrap();
        assert_eq!(trades.len(), 2);
        assert_eq!(trades[0].trade.gross_pnl, Some(Decimal::from(500)));
        assert_eq!(trades[1].trade.gross_pnl, Some(Decimal::from(1_000)));
    }

    #[test]
    fn buy_overshoot_after_short_flips_to_long() {
        // Symmetric counterpart of `sell_overshoot_flips_to_short`.
        let execs = vec![
            exec("META", Side::Short, 50, 400, 0, 1_000),
            exec("META", Side::Cover, 100, 380, 0, 2_000),  // closes 50 short, opens 50 long
            exec("META", Side::Sell,  50, 390, 0, 3_000),
        ];
        let trades = rollup(&execs, LotMethod::Fifo).unwrap();
        assert_eq!(trades.len(), 2);
        assert_eq!(trades[0].trade.side, TradeSide::Short);
        // Short closed: 50 * (400 - 380) = 1000
        assert_eq!(trades[0].trade.gross_pnl, Some(Decimal::from(1_000)));
        assert_eq!(trades[1].trade.side, TradeSide::Long);
        // Long: bought at 380 (the cover), sold at 390 → 50 * 10 = 500
        assert_eq!(trades[1].trade.gross_pnl, Some(Decimal::from(500)));
    }

    #[test]
    fn out_of_order_executions_get_sorted_chronologically() {
        // Importer may emit rows in arbitrary order. Rollup must sort by
        // executed_at internally before matching; otherwise FIFO is wrong.
        let execs = vec![
            exec("AAPL", Side::Sell, 100, 160, 0, 2_000),  // later
            exec("AAPL", Side::Buy,  100, 150, 0, 1_000),  // earlier
        ];
        let trades = rollup(&execs, LotMethod::Fifo).unwrap();
        assert_eq!(trades.len(), 1);
        // P&L must still be +1000 (buy first at 150, sell at 160).
        assert_eq!(trades[0].trade.gross_pnl, Some(Decimal::from(1_000)));
        assert_eq!(trades[0].trade.side, TradeSide::Long);
    }

    #[test]
    fn different_symbols_produce_independent_trades() {
        let execs = vec![
            exec("AAPL", Side::Buy,  100, 150, 0, 1_000),
            exec("TSLA", Side::Buy,   10, 300, 0, 1_500),
            exec("AAPL", Side::Sell, 100, 160, 0, 2_000),
            exec("TSLA", Side::Sell,  10, 290, 0, 2_500),
        ];
        let trades = rollup(&execs, LotMethod::Fifo).unwrap();
        assert_eq!(trades.len(), 2);
        // Each symbol's P&L computed independently — not commingled.
        let aapl = trades.iter().find(|t| t.trade.symbol == "AAPL").expect("AAPL");
        let tsla = trades.iter().find(|t| t.trade.symbol == "TSLA").expect("TSLA");
        assert_eq!(aapl.trade.gross_pnl, Some(Decimal::from(1_000)));
        assert_eq!(tsla.trade.gross_pnl, Some(Decimal::from(-100)));
    }

    #[test]
    fn fully_open_position_carries_no_pnl_yet() {
        // Pure buy with no offset — open trade, no P&L.
        let execs = vec![exec("AAPL", Side::Buy, 100, 150, 1, 1_000)];
        let t = &rollup(&execs, LotMethod::Fifo).unwrap()[0].trade;
        assert_eq!(t.status, TradeStatus::Open);
        assert!(t.gross_pnl.is_none() || t.gross_pnl == Some(Decimal::ZERO));
        assert_eq!(t.qty, Decimal::from(100));
    }

    #[test]
    fn opened_at_uses_first_open_leg_timestamp() {
        let execs = vec![
            exec("AAPL", Side::Buy,  50, 150, 0, 1_000),
            exec("AAPL", Side::Buy,  50, 152, 0, 1_500),
            exec("AAPL", Side::Sell, 100, 160, 0, 2_000),
        ];
        let t = &rollup(&execs, LotMethod::Fifo).unwrap()[0].trade;
        assert_eq!(t.opened_at.timestamp(), 1_000);
        assert_eq!(t.closed_at.expect("closed").timestamp(), 2_000);
    }
}
