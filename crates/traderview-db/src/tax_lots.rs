//! Tax-lot accounting for the `executions` table.
//!
//! Methods: FIFO (default), LIFO. Specific-Identification requires UI lot-
//! mapping persistence and is not in this first cut.
//!
//! Handles long positions only (buy → open lot, sell → close lot). Short/cover
//! events are passed through and reported but not matched, since shorts are
//! always short-term per IRC §1233 and the engine'd need separate inventory.
//!
//! Wash-sale detection: per IRC §1091, a realized loss is disallowed if
//! substantially identical securities are purchased within 30 days before or
//! after the loss sale. We flag and quantify the disallowed amount on the
//! realized event; we do NOT shift the disallowed loss into the basis of
//! the replacement lot (that's a second-order adjustment best handled by
//! tax software).
//!
//! Holding-period: long-term = held ≥ 365 days (IRS uses > 1 year; we use
//! `>= 365` as the integer-day approximation).

use chrono::{DateTime, Datelike, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

const WASH_SALE_WINDOW_DAYS: i64 = 30;
const LONG_TERM_DAYS: i64 = 365;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LotMethod { Fifo, Lifo }

#[derive(Debug, Clone, sqlx::FromRow)]
struct ExecutionRow {
    id: Uuid,
    symbol: String,
    side: String,
    qty: rust_decimal::Decimal,
    price: rust_decimal::Decimal,
    fee: rust_decimal::Decimal,
    executed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OpenLot {
    pub exec_id: Uuid,
    pub symbol: String,
    pub qty_remaining: f64,
    pub cost_per_share: f64,    // fee-loaded
    pub cost_basis: f64,        // qty_remaining * cost_per_share
    pub acquired_at: DateTime<Utc>,
    pub holding_days: i64,
    pub long_term: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct RealizedEvent {
    pub symbol: String,
    pub buy_exec_id: Uuid,
    pub sell_exec_id: Uuid,
    pub acquired_at: DateTime<Utc>,
    pub disposed_at: DateTime<Utc>,
    pub qty: f64,
    pub cost_basis: f64,
    pub proceeds: f64,
    pub gain_loss: f64,
    pub holding_days: i64,
    pub long_term: bool,
    pub wash_sale_disallowed: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct TaxReport {
    pub account_id: Uuid,
    pub year: i32,
    pub method: LotMethod,
    pub realized: Vec<RealizedEvent>,
    pub open_lots: Vec<OpenLot>,
    pub short_term_gain: f64,
    pub short_term_loss: f64,
    pub long_term_gain: f64,
    pub long_term_loss: f64,
    pub net_short_term: f64,
    pub net_long_term: f64,
    pub net_total: f64,
    pub total_proceeds: f64,
    pub total_basis: f64,
    pub wash_sale_total: f64,
    pub realized_count: usize,
    pub open_lot_count: usize,
    pub open_basis: f64,
    pub skipped_short_events: usize,
    pub fetched_at: DateTime<Utc>,
}

pub async fn compute(
    pool: &PgPool,
    account_id: Uuid,
    year: i32,
    method: LotMethod,
) -> anyhow::Result<TaxReport> {
    let execs: Vec<ExecutionRow> = sqlx::query_as(
        "SELECT id, symbol, side::text AS side, qty, price, fee, executed_at
           FROM executions
          WHERE account_id = $1
          ORDER BY executed_at, id",
    )
    .bind(account_id)
    .fetch_all(pool)
    .await?;

    // Group by symbol so the lot queue is per-symbol.
    let mut by_symbol: HashMap<String, Vec<ExecutionRow>> = HashMap::new();
    for e in execs {
        by_symbol.entry(e.symbol.clone()).or_default().push(e);
    }

    let mut all_realized: Vec<RealizedEvent> = Vec::new();
    let mut all_open: Vec<OpenLot> = Vec::new();
    let mut skipped = 0usize;

    for (symbol, events) in by_symbol {
        // Per-symbol lot queue. Each entry: (exec_id, qty_remaining, cost_per_share, acquired_at)
        let mut queue: Vec<(Uuid, f64, f64, DateTime<Utc>)> = Vec::new();
        // Track all buys of this symbol for wash-sale lookup.
        let mut buy_dates: Vec<DateTime<Utc>> = Vec::new();

        for e in &events {
            let qty   = dec(e.qty);
            let price = dec(e.price);
            let fee   = dec(e.fee);
            match e.side.as_str() {
                "buy" => {
                    let cps = price + (fee / qty.max(1e-9));
                    queue.push((e.id, qty, cps, e.executed_at));
                    buy_dates.push(e.executed_at);
                }
                "sell" => {
                    let proceeds_per_share = price - (fee / qty.max(1e-9));
                    let mut remaining = qty;
                    while remaining > 1e-9 && !queue.is_empty() {
                        let pick_idx = match method {
                            LotMethod::Fifo => 0,
                            LotMethod::Lifo => queue.len() - 1,
                        };
                        let (buy_id, lot_qty, cps, acquired_at) = queue[pick_idx];
                        let consumed = remaining.min(lot_qty);
                        let cost_basis = consumed * cps;
                        let proceeds   = consumed * proceeds_per_share;
                        let holding_days = (e.executed_at - acquired_at).num_days();
                        all_realized.push(RealizedEvent {
                            symbol: symbol.clone(),
                            buy_exec_id: buy_id,
                            sell_exec_id: e.id,
                            acquired_at,
                            disposed_at: e.executed_at,
                            qty: consumed,
                            cost_basis,
                            proceeds,
                            gain_loss: proceeds - cost_basis,
                            holding_days,
                            long_term: holding_days >= LONG_TERM_DAYS,
                            wash_sale_disallowed: 0.0,
                        });
                        let new_lot_qty = lot_qty - consumed;
                        if new_lot_qty <= 1e-9 {
                            queue.remove(pick_idx);
                        } else {
                            queue[pick_idx].1 = new_lot_qty;
                        }
                        remaining -= consumed;
                    }
                    // If `remaining > 0` here, the user sold short without an open
                    // long inventory. We don't model short lots in v1 — skip cleanly.
                    if remaining > 1e-9 {
                        skipped += 1;
                    }
                }
                "short" | "cover" => {
                    skipped += 1;
                }
                _ => {}
            }
        }

        // Whatever's left in the queue is open.
        for (id, qty_rem, cps, acquired_at) in queue {
            let hd = (Utc::now() - acquired_at).num_days();
            all_open.push(OpenLot {
                exec_id: id,
                symbol: symbol.clone(),
                qty_remaining: qty_rem,
                cost_per_share: cps,
                cost_basis: qty_rem * cps,
                acquired_at,
                holding_days: hd,
                long_term: hd >= LONG_TERM_DAYS,
            });
        }

        // Wash-sale pass: for each loss event, search same-symbol buy_dates
        // within ±30 days of the sale.
        for ev in all_realized.iter_mut().filter(|r| r.symbol == symbol && r.gain_loss < 0.0) {
            let sale = ev.disposed_at;
            let lo = sale - Duration::days(WASH_SALE_WINDOW_DAYS);
            let hi = sale + Duration::days(WASH_SALE_WINDOW_DAYS);
            let any_buy_in_window = buy_dates.iter().any(|b| *b >= lo && *b <= hi && *b != ev.acquired_at);
            if any_buy_in_window {
                ev.wash_sale_disallowed = (-ev.gain_loss).max(0.0);
            }
        }
    }

    // Sort realized chronologically.
    all_realized.sort_by_key(|r| r.disposed_at);
    all_open.sort_by(|a, b| a.symbol.cmp(&b.symbol).then(a.acquired_at.cmp(&b.acquired_at)));

    // Filter to the requested tax year. Realized events use disposed_at.year;
    // open lots are reported as of now regardless.
    let year_realized: Vec<RealizedEvent> = all_realized
        .into_iter()
        .filter(|r| r.disposed_at.year() == year)
        .collect();

    let mut st_g = 0.0; let mut st_l = 0.0;
    let mut lt_g = 0.0; let mut lt_l = 0.0;
    let mut proceeds = 0.0; let mut basis = 0.0;
    let mut wash_total = 0.0;
    for r in &year_realized {
        proceeds += r.proceeds;
        basis    += r.cost_basis;
        wash_total += r.wash_sale_disallowed;
        // The disallowed portion of the loss is removed from the net,
        // i.e., treat the loss as `gain_loss + wash_sale_disallowed`.
        let recognized = r.gain_loss + r.wash_sale_disallowed;
        if r.long_term {
            if recognized >= 0.0 { lt_g += recognized; } else { lt_l += -recognized; }
        } else if recognized >= 0.0 { st_g += recognized; } else { st_l += -recognized; }
    }
    let open_basis: f64 = all_open.iter().map(|l| l.cost_basis).sum();
    let net_st = st_g - st_l;
    let net_lt = lt_g - lt_l;

    Ok(TaxReport {
        account_id,
        year,
        method,
        realized_count: year_realized.len(),
        open_lot_count: all_open.len(),
        open_basis,
        realized: year_realized,
        open_lots: all_open,
        short_term_gain: st_g,
        short_term_loss: st_l,
        long_term_gain: lt_g,
        long_term_loss: lt_l,
        net_short_term: net_st,
        net_long_term: net_lt,
        net_total: net_st + net_lt,
        total_proceeds: proceeds,
        total_basis: basis,
        wash_sale_total: wash_total,
        skipped_short_events: skipped,
        fetched_at: Utc::now(),
    })
}

fn dec(d: rust_decimal::Decimal) -> f64 {
    d.to_string().parse().unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_exec(side: &str, qty: f64, price: f64, fee: f64, ts: i64) -> ExecutionRow {
        ExecutionRow {
            id: Uuid::new_v4(),
            symbol: "AAPL".into(),
            side: side.into(),
            qty: rust_decimal::Decimal::try_from(qty).unwrap(),
            price: rust_decimal::Decimal::try_from(price).unwrap(),
            fee: rust_decimal::Decimal::try_from(fee).unwrap(),
            executed_at: DateTime::from_timestamp(ts, 0).unwrap(),
        }
    }

    fn run_engine(execs: Vec<ExecutionRow>, method: LotMethod) -> (Vec<RealizedEvent>, Vec<OpenLot>) {
        let mut queue: Vec<(Uuid, f64, f64, DateTime<Utc>)> = Vec::new();
        let mut realized: Vec<RealizedEvent> = Vec::new();
        let mut buy_dates: Vec<DateTime<Utc>> = Vec::new();
        for e in &execs {
            let qty = dec(e.qty);
            let price = dec(e.price);
            let fee = dec(e.fee);
            match e.side.as_str() {
                "buy" => {
                    let cps = price + (fee / qty.max(1e-9));
                    queue.push((e.id, qty, cps, e.executed_at));
                    buy_dates.push(e.executed_at);
                }
                "sell" => {
                    let pps = price - (fee / qty.max(1e-9));
                    let mut remaining = qty;
                    while remaining > 1e-9 && !queue.is_empty() {
                        let idx = match method { LotMethod::Fifo => 0, LotMethod::Lifo => queue.len() - 1 };
                        let (bid, lq, cps, at) = queue[idx];
                        let consumed = remaining.min(lq);
                        let cb = consumed * cps;
                        let pr = consumed * pps;
                        let hd = (e.executed_at - at).num_days();
                        realized.push(RealizedEvent {
                            symbol: e.symbol.clone(),
                            buy_exec_id: bid, sell_exec_id: e.id,
                            acquired_at: at, disposed_at: e.executed_at,
                            qty: consumed, cost_basis: cb, proceeds: pr,
                            gain_loss: pr - cb, holding_days: hd,
                            long_term: hd >= LONG_TERM_DAYS,
                            wash_sale_disallowed: 0.0,
                        });
                        let nq = lq - consumed;
                        if nq <= 1e-9 { queue.remove(idx); } else { queue[idx].1 = nq; }
                        remaining -= consumed;
                    }
                }
                _ => {}
            }
        }
        let opens: Vec<OpenLot> = queue.into_iter().map(|(id, q, cps, at)| {
            let hd = (Utc::now() - at).num_days();
            OpenLot {
                exec_id: id, symbol: "AAPL".into(),
                qty_remaining: q, cost_per_share: cps,
                cost_basis: q * cps, acquired_at: at,
                holding_days: hd, long_term: hd >= LONG_TERM_DAYS,
            }
        }).collect();
        (realized, opens)
    }

    #[test]
    fn fifo_vs_lifo_differs_on_realized_basis() {
        // Buy 10 @ 100, then 10 @ 200. Sell 10 @ 300.
        // FIFO consumes the @100 lot → basis 1000, gain 2000.
        // LIFO consumes the @200 lot → basis 2000, gain 1000.
        let day = 86_400;
        let execs = vec![
            make_exec("buy",  10.0, 100.0, 0.0, 0),
            make_exec("buy",  10.0, 200.0, 0.0, 10 * day),
            make_exec("sell", 10.0, 300.0, 0.0, 20 * day),
        ];
        let (fifo, _) = run_engine(execs.clone(), LotMethod::Fifo);
        let (lifo, _) = run_engine(execs, LotMethod::Lifo);
        assert_eq!(fifo.len(), 1);
        assert_eq!(lifo.len(), 1);
        assert!((fifo[0].cost_basis - 1000.0).abs() < 1e-6, "FIFO basis = {}", fifo[0].cost_basis);
        assert!((lifo[0].cost_basis - 2000.0).abs() < 1e-6, "LIFO basis = {}", lifo[0].cost_basis);
        assert!((fifo[0].gain_loss - 2000.0).abs() < 1e-6);
        assert!((lifo[0].gain_loss - 1000.0).abs() < 1e-6);
    }

    #[test]
    fn long_term_classification_at_365_days() {
        let day = 86_400;
        let execs = vec![
            make_exec("buy",  10.0, 100.0, 0.0, 0),
            make_exec("sell", 10.0, 110.0, 0.0, 365 * day),
        ];
        let (r, _) = run_engine(execs, LotMethod::Fifo);
        assert!(r[0].long_term, "365 days should be long-term");
        assert_eq!(r[0].holding_days, 365);
    }

    #[test]
    fn partial_sell_leaves_open_lot() {
        let day = 86_400;
        let execs = vec![
            make_exec("buy",  10.0, 100.0, 0.0, 0),
            make_exec("sell", 6.0,  120.0, 0.0, 30 * day),
        ];
        let (r, opens) = run_engine(execs, LotMethod::Fifo);
        assert_eq!(r.len(), 1);
        assert!((r[0].qty - 6.0).abs() < 1e-9);
        assert_eq!(opens.len(), 1);
        assert!((opens[0].qty_remaining - 4.0).abs() < 1e-9);
    }
}
