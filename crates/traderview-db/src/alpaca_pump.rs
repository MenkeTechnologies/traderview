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
use crate::alpaca_trading::{AlpacaError, AlpacaTrading, BrokerMode, TradeUpdateEvent};
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
pub async fn run_pump(pool: PgPool, client: AlpacaTrading, event_sink: Option<EventSink>) {
    let mut backoff = Duration::from_secs(1);
    const MAX_BACKOFF: Duration = Duration::from_secs(60);
    loop {
        // Repair fills the WS missed before (re)connecting: app closed
        // while orders filled, reconnect gaps, or rows lost to the old
        // skip-partials bug. Idempotent — a clean ledger sweeps to zero.
        match reconcile_recent_fills(&pool, &client, event_sink.as_ref()).await {
            Ok(0) => {}
            Ok(n) => tracing::info!(repaired = n, "alpaca fill reconcile inserted missing fills"),
            Err(e) => tracing::warn!(error = %e, "alpaca fill reconcile failed"),
        }
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
    // Consume BOTH `partial_fill` and the terminal `fill`. Each event's
    // `qty` is that slice only — the terminal `fill` does NOT carry the
    // cumulative aggregate (verified against the activities ledger: a
    // 3002-unit AVAX/USD buy emitted 14 partial_fill slices and a final
    // `fill` of 130.07; the old skip-partials logic ingested just the
    // 130.07 and the trades rollup closed a position that was 95% still
    // open at the broker). Replay-idempotency lives in
    // algo_fills.broker_fill_id keyed on the per-slice execution_id.
    if event.event != "fill" && event.event != "partial_fill" {
        return Ok(());
    }
    let coid_str = event.order.client_order_id.clone();
    let client_order_id = Uuid::from_str(&coid_str)
        .map_err(|e| anyhow::anyhow!("alpaca client_order_id {coid_str} is not a UUID: {e}"))?;

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

    // Hard-reject events missing price or qty. The prior behaviour was
    // to default both to zero — a malformed `fill` payload then flowed
    // through `record_fill` as a $0 trade, corrupting executions/trades
    // and breaking every downstream P&L number for that strategy. The
    // reconnect path will re-fetch state from /v2/orders on the next
    // session if a fill is genuinely lost.
    let Some(fill_price) = event.price else {
        tracing::error!(client_order_id = %coid_str, "alpaca fill missing price; dropping event");
        return Ok(());
    };
    let Some(fill_qty) = event.qty else {
        tracing::error!(client_order_id = %coid_str, "alpaca fill missing qty; dropping event");
        return Ok(());
    };
    if !fill_price.is_sign_positive() || fill_price.is_zero() {
        tracing::error!(
            client_order_id = %coid_str, %fill_price,
            "alpaca fill price non-positive; dropping event"
        );
        return Ok(());
    }
    if fill_qty.is_zero() {
        tracing::error!(
            client_order_id = %coid_str, %fill_qty,
            "alpaca fill qty zero; dropping event"
        );
        return Ok(());
    }
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
    // Slice-unique id for algo_fills idempotency; composite fallback if
    // Alpaca ever omits execution_id. The executions row keeps the order
    // id so per-order sums (reconcile) see every slice.
    let slice_id = event.execution_id.clone().unwrap_or_else(|| {
        format!("{}:{}:{}", event.order.id, event.event, fill_qty)
    });
    let immediate = ImmediateFill {
        price: fill_price,
        qty: fill_qty,
        fee: Decimal::zero(),
        executed_at,
        broker_fill_id: Some(slice_id),
        broker_order_id: Some(event.order.id),
    };
    crate::algo_engine::record_fill(pool, &strategy, &intent, order_id, &immediate, event_sink)
        .await
        .map_err(|e| anyhow::anyhow!("record_fill: {e}"))?;
    Ok(())
}

/// Given a broker order's cumulative fill state and what the executions
/// ledger already ingested for it, return the missing (qty, price) to
/// insert — priced so the ledger's total cost matches the broker's
/// `filled_avg_price × filled_qty` exactly. None when nothing is missing.
fn remainder_fill(
    filled_qty: Decimal,
    filled_avg_price: Decimal,
    ingested_qty: Decimal,
    ingested_notional: Decimal,
) -> Option<(Decimal, Decimal)> {
    let delta = filled_qty - ingested_qty;
    if delta <= Decimal::zero() {
        return None;
    }
    let mut price = (filled_avg_price * filled_qty - ingested_notional) / delta;
    // A negative/zero remainder price means the ingested rows already
    // over-account the order's notional (bad legacy data) — fall back to
    // the broker's average rather than writing a nonsense execution.
    if price <= Decimal::zero() {
        price = filled_avg_price;
    }
    Some((delta, price))
}

/// One REST sweep over recent Alpaca orders (last 500 closed + all open)
/// bound to algo_orders rows: compare the broker's cumulative
/// `filled_qty` against the executions already ingested for that broker
/// order id and insert the remainder via the standard record_fill
/// pipeline (executions → rollup → tag → realized P&L). Returns how many
/// repair fills were inserted.
pub async fn reconcile_recent_fills(
    pool: &PgPool,
    client: &AlpacaTrading,
    event_sink: Option<&EventSink>,
) -> anyhow::Result<usize> {
    let mut orders = client.list_orders("closed", 500).await?;
    orders.extend(client.list_open_orders().await?);
    let mut repaired = 0usize;
    for o in orders {
        let Some(filled_qty) = o.filled_qty else { continue };
        let Some(filled_avg) = o.filled_avg_price else { continue };
        if filled_qty <= Decimal::zero() || filled_avg <= Decimal::zero() {
            continue;
        }
        // Orders not stamped with our UUID client_order_id were placed
        // outside the app — not ours to ledger.
        let Ok(coid) = Uuid::from_str(&o.client_order_id) else { continue };
        let row: Option<(Uuid, Uuid, Uuid, String, String, Decimal)> = sqlx::query_as(
            "SELECT id, run_id, strategy_id, symbol, side, qty
               FROM algo_orders WHERE client_order_id = $1",
        )
        .bind(coid)
        .fetch_optional(pool)
        .await?;
        let Some((order_id, run_id, strategy_id, symbol, side, qty)) = row else { continue };
        let Some(strategy) = crate::algo::get_strategy_by_id(pool, strategy_id).await? else {
            continue;
        };
        let (ingested_qty, ingested_notional): (Decimal, Decimal) = sqlx::query_as(
            "SELECT COALESCE(SUM(qty), 0), COALESCE(SUM(qty * price), 0)
               FROM executions
              WHERE account_id = $1 AND broker_order_id = $2 AND symbol = $3",
        )
        .bind(strategy.account_id)
        .bind(&o.id)
        .bind(&symbol)
        .fetch_one(pool)
        .await?;
        let Some((miss_qty, miss_price)) =
            remainder_fill(filled_qty, filled_avg, ingested_qty, ingested_notional)
        else {
            continue;
        };
        tracing::info!(
            symbol, broker_order_id = %o.id, %miss_qty, %miss_price,
            "reconcile: inserting fill the WS pump missed"
        );
        let intent = crate::algo_engine::OrderIntent {
            strategy_id,
            run_id,
            client_order_id: coid,
            symbol,
            side: match side.as_str() {
                "sell" => traderview_core::algo_strategies::Side::Sell,
                _ => traderview_core::algo_strategies::Side::Buy,
            },
            qty,
            entry_price: miss_price,
            stop_price: Decimal::zero(),
            take_profit_price: Decimal::zero(),
        };
        let fill = ImmediateFill {
            price: miss_price,
            qty: miss_qty,
            fee: Decimal::zero(),
            executed_at: o.updated_at.unwrap_or_else(Utc::now),
            // Ingested-qty suffix keeps re-sweeps idempotent: the same
            // gap maps to the same id (algo_fills UNIQUE rejects the
            // replay); once repaired, delta is zero and no id is minted.
            broker_fill_id: Some(format!("{}:reconcile:{}", o.id, ingested_qty)),
            broker_order_id: Some(o.id.clone()),
        };
        crate::algo_engine::record_fill(pool, &strategy, &intent, order_id, &fill, event_sink)
            .await
            .map_err(|e| anyhow::anyhow!("reconcile record_fill: {e}"))?;
        repaired += 1;
    }
    Ok(repaired)
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
    let mode = if paper {
        BrokerMode::Paper
    } else {
        BrokerMode::Live
    };
    let client = AlpacaTrading::new(mode, key_id, secret);
    let registry_for_drop = registry.clone();
    let key = (user_id, paper);
    // Supervisor wrapper: when run_pump exits (fatal auth, or panics
    // from a decode bug / sqlx pool exhaustion / etc.), clear the
    // registry entry so a later `ensure_pump_for` can respawn. Without
    // this the registry stays "alive" forever — fills accumulate at the
    // broker with no consumer, and `has_pending_order` blocks all
    // future entries on that strategy.
    tokio::spawn(async move {
        use futures_util::FutureExt;
        let result = std::panic::AssertUnwindSafe(run_pump(pool, client, event_sink))
            .catch_unwind()
            .await;
        match result {
            Ok(()) => tracing::warn!(?key, "alpaca pump exited; clearing registry"),
            Err(_) => tracing::error!(?key, "alpaca pump panicked; clearing registry"),
        }
        registry_for_drop.lock().await.remove(&key);
    });
    true
}

#[cfg(test)]
mod tests {
    use super::remainder_fill;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    fn d(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    #[test]
    fn remainder_fill_full_miss_inserts_whole_order_at_avg() {
        // Nothing ingested — repair the entire order at the broker avg.
        let (qty, price) = remainder_fill(d("55"), d("1671.02"), d("0"), d("0")).unwrap();
        assert_eq!(qty, d("55"));
        assert_eq!(price, d("1671.02"));
    }

    #[test]
    fn remainder_fill_partial_miss_preserves_total_cost() {
        // The real AVAX case: 3002 filled at avg 6.701838978, but only the
        // final 130.07112291-unit slice (at 6.742404) was ingested. The
        // repair row must make ledger cost == broker cost exactly.
        let filled = d("3002");
        let avg = d("6.701838978");
        let ingested_qty = d("130.07112291");
        let ingested_notional = ingested_qty * d("6.742404");
        let (qty, price) = remainder_fill(filled, avg, ingested_qty, ingested_notional).unwrap();
        assert_eq!(qty, filled - ingested_qty);
        let ledger_cost = ingested_notional + qty * price;
        let broker_cost = filled * avg;
        assert!(
            (ledger_cost - broker_cost).abs() < d("0.0001"),
            "ledger {ledger_cost} != broker {broker_cost}"
        );
    }

    #[test]
    fn remainder_fill_fully_ingested_is_none() {
        let n = d("130.07112291") * d("6.58");
        assert!(remainder_fill(d("130.07112291"), d("6.58"), d("130.07112291"), n).is_none());
    }

    #[test]
    fn remainder_fill_over_ingested_is_none() {
        // Ledger somehow has MORE than the broker reports — never insert
        // a negative repair.
        assert!(remainder_fill(d("100"), d("5"), d("120"), d("600")).is_none());
    }

    #[test]
    fn remainder_fill_degenerate_price_falls_back_to_avg() {
        // Ingested notional already exceeds the broker's total cost; the
        // solved remainder price would be negative. Fall back to avg.
        let (_qty, price) = remainder_fill(d("100"), d("5"), d("50"), d("9999")).unwrap();
        assert_eq!(price, d("5"));
    }
}
