//! Per-trade order-flow tape replay.
//!
//! Given a trade_id, assemble a unified timeline:
//!   * Bars covering [trade.opened_at - 1d, trade.closed_at + 1d] (or now)
//!   * Trade's executions as markers with side / qty / price / timestamp
//!   * Trade-level reference levels: entry_avg, stop_loss, initial_target
//!
//! Bar interval is chosen automatically by hold duration:
//!   < 4h          → 1m  bars
//!   < 5 days      → 5m  bars
//!   < 30 days     → 1h  bars
//!   ≥ 30 days     → 1d  bars
//!
//! This is the data layer; the frontend animates progressive bar reveal +
//! marker fade-in at a configurable speed.

use chrono::{DateTime, Duration, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::PgPool;
use traderview_core::BarInterval;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct ReplayBar {
    pub time: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReplayExec {
    pub id: Uuid,
    pub time: DateTime<Utc>,
    pub side: String, // buy / sell / short / cover
    pub qty: f64,
    pub price: f64,
    pub fee: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct TapeReplay {
    pub trade_id: Uuid,
    pub symbol: String,
    pub side: String,
    pub status: String,
    pub opened_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub entry_avg: f64,
    pub exit_avg: Option<f64>,
    pub qty: f64,
    pub stop_loss: Option<f64>,
    pub initial_target: Option<f64>,
    pub net_pnl: Option<f64>,
    pub interval: &'static str, // "1m" | "5m" | "1h" | "1d"
    pub bars: Vec<ReplayBar>,
    pub execs: Vec<ReplayExec>,
}

pub async fn build(pool: &PgPool, user_id: Uuid, trade_id: Uuid) -> anyhow::Result<TapeReplay> {
    let trade = crate::trades::get(pool, trade_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("trade not found"))?;
    // Ownership check via account.
    let owner: Option<(Uuid,)> = sqlx::query_as("SELECT user_id FROM accounts WHERE id = $1")
        .bind(trade.account_id)
        .fetch_optional(pool)
        .await?;
    match owner {
        Some((id,)) if id == user_id => {}
        Some(_) => anyhow::bail!("forbidden"),
        None => anyhow::bail!("account not found"),
    }

    let opened = trade.opened_at;
    let closed = trade.closed_at.unwrap_or_else(Utc::now);
    let hold = closed.signed_duration_since(opened);

    let (interval, interval_str): (BarInterval, &'static str) = if hold < Duration::hours(4) {
        (BarInterval::M1, "1m")
    } else if hold < Duration::days(5) {
        (BarInterval::M5, "5m")
    } else if hold < Duration::days(30) {
        (BarInterval::H1, "1h")
    } else {
        (BarInterval::D1, "1d")
    };
    let pad = match interval_str {
        "1m" => Duration::hours(1),
        "5m" => Duration::hours(6),
        "1h" => Duration::days(1),
        _ => Duration::days(3),
    };
    let from = opened - pad;
    let to = closed + pad;

    let raw_bars = crate::prices::get_bars(pool, &trade.symbol, interval, from, to)
        .await
        .unwrap_or_default();
    let bars: Vec<ReplayBar> = raw_bars
        .iter()
        .map(|b| ReplayBar {
            time: b.bar_time,
            open: dec(b.open),
            high: dec(b.high),
            low: dec(b.low),
            close: dec(b.close),
            volume: dec(b.volume),
        })
        .collect();

    let execs_raw = crate::executions::list_for_trade(pool, trade_id)
        .await
        .unwrap_or_default();
    let execs: Vec<ReplayExec> = execs_raw
        .iter()
        .map(|e| ReplayExec {
            id: e.id,
            time: e.executed_at,
            side: format!("{:?}", e.side).to_lowercase(),
            qty: dec(e.qty),
            price: dec(e.price),
            fee: dec(e.fee),
        })
        .collect();

    Ok(TapeReplay {
        trade_id,
        symbol: trade.symbol,
        side: format!("{:?}", trade.side).to_lowercase(),
        status: format!("{:?}", trade.status).to_lowercase(),
        opened_at: opened,
        closed_at: trade.closed_at,
        entry_avg: dec(trade.entry_avg),
        exit_avg: trade.exit_avg.map(dec),
        qty: dec(trade.qty),
        stop_loss: trade.stop_loss.map(dec),
        initial_target: trade.initial_target.map(dec),
        net_pnl: trade.net_pnl.map(dec),
        interval: interval_str,
        bars,
        execs,
    })
}

fn dec(d: Decimal) -> f64 {
    d.to_string().parse().unwrap_or(0.0)
}
