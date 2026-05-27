//! Live P/L tracker for OPEN positions.
//!
//! Walks `trades WHERE account_id=? AND status='open'`, fetches a fresh
//! `quote()` per distinct symbol (60s DB cache already in place), and
//! computes per-position + portfolio-level unrealized P/L. Day-delta uses
//! the quote's prev_close (regular-session prior close, Yahoo-provided).
//!
//! Multipliers are honored so options (× contract size) compute right.
//! Short positions invert the sign: unrealized = (entry - last) * qty * mult.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct LivePosition {
    pub trade_id: Uuid,
    pub symbol: String,
    pub side: String, // long | short
    pub asset_class: String,
    pub qty: f64,
    pub entry_avg: f64,
    pub multiplier: f64,
    pub last_price: f64,
    pub prev_close: Option<f64>,
    pub change_pct: Option<f64>,
    pub notional: f64,
    pub unrealized_pnl: f64,
    pub unrealized_pct: f64,
    pub day_pnl: Option<f64>, // (last - prev_close) * qty * mult * sideSign
    pub opened_at: DateTime<Utc>,
    pub market_state: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LiveSnapshot {
    pub account_id: Uuid,
    pub positions: Vec<LivePosition>,
    pub position_count: usize,
    pub total_notional: f64,
    pub total_unrealized_pnl: f64,
    pub total_day_pnl: f64,
    pub biggest_winner: Option<String>, // symbol
    pub biggest_loser: Option<String>,
    pub fetched_at: DateTime<Utc>,
}

type OpenPositionRow = (
    Uuid,
    String,
    String,
    String,
    Decimal,
    Decimal,
    Decimal,
    DateTime<Utc>,
);

pub async fn snapshot(pool: &PgPool, account_id: Uuid) -> anyhow::Result<LiveSnapshot> {
    let rows: Vec<OpenPositionRow> = sqlx::query_as(
        "SELECT id, symbol, side::text, asset_class::text, qty, entry_avg, multiplier, opened_at
               FROM trades
              WHERE account_id = $1 AND status = 'open'
              ORDER BY opened_at DESC",
    )
    .bind(account_id)
    .fetch_all(pool)
    .await?;

    // Dedupe symbol fetches across multiple open positions in the same name.
    let mut quote_cache: HashMap<String, crate::market_data::QuoteSnapshot> = HashMap::new();
    let mut positions = Vec::with_capacity(rows.len());

    for (id, symbol, side, asset_class, qty, entry, mult, opened_at) in rows {
        let q = match quote_cache.get(&symbol) {
            Some(q) => q.clone(),
            None => match crate::market_data::quote(pool, &symbol).await {
                Ok(q) => {
                    quote_cache.insert(symbol.clone(), q.clone());
                    q
                }
                Err(_) => continue,
            },
        };
        let qty_f = dec(qty);
        let entry_f = dec(entry);
        let mult_f = dec(mult).max(1.0);
        let side_sign = if side == "short" { -1.0 } else { 1.0 };
        let last = q.price;
        let notional = qty_f * last * mult_f;
        let unrealized = (last - entry_f) * qty_f * mult_f * side_sign;
        let unrealized_pct = if entry_f > 0.0 {
            (last - entry_f) / entry_f * 100.0 * side_sign
        } else {
            0.0
        };
        let day_pnl = q
            .prev_close
            .map(|pc| (last - pc) * qty_f * mult_f * side_sign);
        positions.push(LivePosition {
            trade_id: id,
            symbol: symbol.clone(),
            side,
            asset_class,
            qty: qty_f,
            entry_avg: entry_f,
            multiplier: mult_f,
            last_price: last,
            prev_close: q.prev_close,
            change_pct: q.change_pct,
            notional,
            unrealized_pnl: unrealized,
            unrealized_pct,
            day_pnl,
            opened_at,
            market_state: q.market_state,
        });
    }

    let position_count = positions.len();
    let total_notional: f64 = positions.iter().map(|p| p.notional.abs()).sum();
    let total_unrealized: f64 = positions.iter().map(|p| p.unrealized_pnl).sum();
    let total_day: f64 = positions.iter().filter_map(|p| p.day_pnl).sum();
    let biggest_winner = positions
        .iter()
        .filter(|p| p.unrealized_pnl > 0.0)
        .max_by(|a, b| a.unrealized_pnl.partial_cmp(&b.unrealized_pnl).unwrap())
        .map(|p| p.symbol.clone());
    let biggest_loser = positions
        .iter()
        .filter(|p| p.unrealized_pnl < 0.0)
        .min_by(|a, b| a.unrealized_pnl.partial_cmp(&b.unrealized_pnl).unwrap())
        .map(|p| p.symbol.clone());

    Ok(LiveSnapshot {
        account_id,
        positions,
        position_count,
        total_notional,
        total_unrealized_pnl: total_unrealized,
        total_day_pnl: total_day,
        biggest_winner,
        biggest_loser,
        fetched_at: Utc::now(),
    })
}

fn dec(d: Decimal) -> f64 {
    d.to_string().parse().unwrap_or(0.0)
}
