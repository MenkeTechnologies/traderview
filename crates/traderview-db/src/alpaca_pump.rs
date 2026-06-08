//! Long-running consumer for Alpaca's trade_updates WebSocket. One
//! connection per (key_id, secret, paper/live) tuple; reconnect on
//! drop with exponential backoff. Each `fill` / `partial_fill` event
//! is mapped to a `record_fill` call so the executions + trades
//! pipeline picks it up the same way the InMemorySink path does.
//!
//! Spawned from `bin/server.rs` once per unique (creds, mode)
//! observed across the currently-active algo strategies. This module
//! is the WS-side companion to the REST `alpaca_trading` client.

use crate::algo_engine::{EventSink, ImmediateFill};
use crate::alpaca_trading::{
    AlpacaError, AlpacaTrading, BrokerMode, TradeUpdateEvent,
};
use chrono::Utc;
use rust_decimal::prelude::Zero;
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::collections::HashSet;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use uuid::Uuid;

/// Shared registry of running pumps keyed by (user_id, paper-bool).
/// Server startup populates from list_active_strategies; the
/// create/update strategy route calls `ensure_pump_for` to hot-spawn
/// for any newly-introduced (user, mode) tuple.
pub type AlpacaPumpRegistry = Arc<Mutex<HashSet<(Uuid, bool)>>>;

/// Reconnect loop. Caller `tokio::spawn`s and forgets. Exits only
/// on a fatal auth error (bad creds — no point retrying). Transient
/// network/WS errors trigger a backoff loop up to MAX_BACKOFF.
pub async fn run_pump(
    pool: PgPool,
    client: AlpacaTrading,
    event_sink: Option<EventSink>,
) {
    let mut backoff = Duration::from_secs(1);
    const MAX_BACKOFF: Duration = Duration::from_secs(60);
    loop {
        let pool_clone = pool.clone();
        let sink_clone = event_sink.clone();
        let res = client
            .trade_updates_stream(|event| {
                let p = pool_clone.clone();
                let s = sink_clone.clone();
                async move {
                    if let Err(e) = handle_trade_update(&p, event, s.as_ref()).await {
                        tracing::warn!(error = %e, "alpaca trade_updates handler");
                    }
                    Ok::<(), AlpacaError>(())
                }
            })
            .await;
        match res {
            Ok(()) => {
                tracing::info!("alpaca trade_updates stream closed; reconnecting");
            }
            Err(AlpacaError::AuthFailed) => {
                tracing::error!("alpaca WS auth failed; not retrying");
                return;
            }
            Err(e) => {
                tracing::warn!(error = %e, ?backoff, "alpaca trade_updates dropped; backing off");
            }
        }
        tokio::time::sleep(backoff).await;
        backoff = (backoff * 2).min(MAX_BACKOFF);
    }
}

/// Decode one trade_updates event and write it through the standard
/// engine fill path. `client_order_id` is the UUID the engine stamped
/// on the order at submit time; we use it to look up the
/// `algo_orders` row + the bound strategy, then call `record_fill`.
pub async fn handle_trade_update(
    pool: &PgPool,
    event: TradeUpdateEvent,
    event_sink: Option<&EventSink>,
) -> anyhow::Result<()> {
    if event.event != "fill" && event.event != "partial_fill" {
        // new / canceled / replaced / done_for_day etc. — interesting
        // for UI state but not for the executions pipeline.
        return Ok(());
    }
    let coid_str = event.order.client_order_id.clone();
    let client_order_id = Uuid::from_str(&coid_str).map_err(|e| {
        anyhow::anyhow!("alpaca client_order_id {coid_str} is not a UUID: {e}")
    })?;

    let row: Option<(Uuid, Uuid, Uuid, String, String, Decimal)> = sqlx::query_as(
        "SELECT id, run_id, strategy_id, symbol, side, qty
           FROM algo_orders WHERE client_order_id = $1",
    )
    .bind(client_order_id)
    .fetch_optional(pool)
    .await?;
    let Some((order_id, run_id, strategy_id, symbol, side, qty)) = row else {
        // Alpaca emitted a fill for a client_order_id we don't know —
        // either an order placed outside this app, or a fill that
        // arrived after the row was deleted. Log + drop.
        tracing::debug!(client_order_id = %coid_str, "trade_update for unknown order");
        return Ok(());
    };

    let strategy = crate::algo::get_strategy_by_id(pool, strategy_id).await?;
    let Some(strategy) = strategy else {
        tracing::warn!(strategy_id = %strategy_id, "fill for missing strategy");
        return Ok(());
    };

    let fill_price = event.price.unwrap_or_else(Decimal::zero);
    let fill_qty = event.qty.unwrap_or_else(Decimal::zero);
    let executed_at = event.timestamp.unwrap_or_else(Utc::now);

    let intent = crate::algo_engine::OrderIntent {
        strategy_id,
        run_id,
        client_order_id,
        symbol,
        side: match side.as_str() {
            "sell" => traderview_core::algo_strategies::Side::Sell,
            _ => traderview_core::algo_strategies::Side::Buy,
        },
        qty,
        entry_price: fill_price,
        stop_price: Decimal::zero(),
        take_profit_price: Decimal::zero(),
    };
    let immediate = ImmediateFill {
        price: fill_price,
        qty: fill_qty,
        fee: Decimal::zero(),
        executed_at,
        broker_fill_id: Some(event.order.id),
    };
    crate::algo_engine::record_fill(pool, &strategy, &intent, order_id, &immediate, event_sink)
        .await
        .map_err(|e| anyhow::anyhow!("record_fill: {e}"))?;
    Ok(())
}

/// Convenience for the server startup: spawn pumps for every distinct
/// `(user_id, paper-or-live)` tuple referenced by an active algo
/// strategy. Records each spawn in `registry` so the route layer can
/// later avoid double-spawning via `ensure_pump_for`. Returns count.
pub async fn spawn_pumps_for_active_strategies(
    pool: PgPool,
    event_sink: Option<EventSink>,
    registry: AlpacaPumpRegistry,
) -> anyhow::Result<usize> {
    let strategies = crate::algo::list_active_strategies(&pool).await?;
    let mut spawned = 0usize;
    for s in strategies {
        let paper = matches!(s.broker_mode.as_str(), "paper");
        let live = matches!(s.broker_mode.as_str(), "live");
        if !paper && !live {
            continue;
        }
        // Only spawn Alpaca pumps for strategies whose bound account is
        // on Alpaca. Other broker accounts get their own pump module
        // (Tradier / IBKR / etc. in follow-up commits).
        if !is_alpaca_account(&pool, s.account_id).await {
            continue;
        }
        if ensure_pump_for(
            registry.clone(),
            pool.clone(),
            s.user_id,
            paper,
            event_sink.clone(),
        )
        .await
        {
            spawned += 1;
        }
    }
    Ok(spawned)
}

/// True when the account exists and its broker is 'alpaca'. Used to
/// keep Alpaca-specific pump spawn from misfiring on Tradier/IBKR/etc.
/// strategies. Errors / missing accounts return false (defensive).
async fn is_alpaca_account(pool: &PgPool, account_id: Uuid) -> bool {
    let row: Result<Option<(Option<String>,)>, _> =
        sqlx::query_as("SELECT broker FROM accounts WHERE id = $1")
            .bind(account_id)
            .fetch_optional(pool)
            .await;
    matches!(
        row,
        Ok(Some((Some(b),))) if b.eq_ignore_ascii_case("alpaca")
    )
}

/// Idempotent spawn — returns true when a new pump was actually
/// started, false when one already existed for this (user, mode) or
/// the user has no Alpaca credentials yet. Called from the routes
/// after a successful create_strategy / update_strategy whose
/// broker_mode lands in {paper, live} and the account.broker is 'alpaca'.
///
/// Mutex is held only across the insert; the long-running pump task
/// is spawned AFTER the lock is dropped so the registry stays
/// responsive even while pumps reconnect.
pub async fn ensure_pump_for(
    registry: AlpacaPumpRegistry,
    pool: PgPool,
    user_id: Uuid,
    paper: bool,
    event_sink: Option<EventSink>,
) -> bool {
    {
        let mut guard = registry.lock().await;
        if !guard.insert((user_id, paper)) {
            return false;
        }
    }
    let creds = match crate::data_source_keys::alpaca_creds_plain(&pool, user_id).await {
        Ok(Some(c)) => c,
        Ok(None) => {
            tracing::warn!(user_id = %user_id, "alpaca strategy enabled but no creds");
            // Roll back the registry entry so a later cred save can retry.
            registry.lock().await.remove(&(user_id, paper));
            return false;
        }
        Err(e) => {
            tracing::warn!(user_id = %user_id, error = %e, "alpaca_creds_plain failed");
            registry.lock().await.remove(&(user_id, paper));
            return false;
        }
    };
    let (key_id, secret, _) = creds;
    let mode = if paper { BrokerMode::Paper } else { BrokerMode::Live };
    let client = AlpacaTrading::new(mode, key_id, secret);
    tokio::spawn(run_pump(pool, client, event_sink));
    true
}
