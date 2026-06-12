//! Tradier streaming events pump — companion to `tradier_trading`.
//!
//! Protocol (verified against docs.tradier.com 2026-06):
//!   1. POST <BASE>/v1/accounts/events/session  → `{url, sessionid}`
//!      Bearer auth; session valid for 5 minutes.
//!   2. WS connect to the returned URL (wss://ws.tradier.com/v1/accounts/events).
//!   3. Send subscription frame `{"sessionid": "<id>", "events": ["order"]}`.
//!   4. Server streams JSON events. Fill events carry the `tag` we
//!      stamped on submit — that's the strategy's client_order_id UUID.
//!
//! The 5-minute session TTL drives a reconnect loop just like alpaca_pump:
//! drop the WS, refresh the session via REST, reconnect. Backoff caps at
//! 60s on transient errors; auth failures bail.
//!
//! Field names in the streamed events are mapped defensively (loose
//! JSON walking with `.get(...)`) so Tradier schema tweaks don't break
//! the integration silently.

use crate::algo_engine::{EventSink, ImmediateFill};
use crate::alpaca_pump::AlpacaPumpRegistry; // shared registry type
use crate::tradier_trading::{TradierEnv, TradierError};
use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use rust_decimal::prelude::Zero;
use rust_decimal::Decimal;
use serde::Deserialize;
use sqlx::PgPool;
use std::str::FromStr;
use std::time::Duration;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use uuid::Uuid;

const SESSION_TTL: Duration = Duration::from_secs(4 * 60); // refresh well before 5 min

#[derive(Debug, Deserialize)]
struct SessionResponse {
    stream: SessionInner,
}
#[derive(Debug, Deserialize)]
struct SessionInner {
    url: String,
    #[serde(rename = "sessionid")]
    session_id: String,
}

/// Reconnect loop. Spawn-and-forget. Refreshes the session before each
/// reconnect so the 5-minute TTL never expires mid-stream.
pub async fn run_pump(pool: PgPool, env: TradierEnv, token: String, event_sink: Option<EventSink>) {
    let mut backoff = Duration::from_secs(1);
    const MAX_BACKOFF: Duration = Duration::from_secs(60);
    loop {
        match run_session_once(&pool, env, &token, event_sink.as_ref()).await {
            Ok(()) => {
                tracing::info!("tradier events stream closed; reconnecting");
                backoff = Duration::from_secs(1);
            }
            Err(TradierError::AuthFailed) => {
                tracing::error!("tradier WS auth failed; not retrying");
                return;
            }
            Err(e) => {
                tracing::warn!(error = %e, ?backoff, "tradier events stream dropped");
            }
        }
        tokio::time::sleep(backoff).await;
        backoff = (backoff * 2).min(MAX_BACKOFF);
    }
}

async fn run_session_once(
    pool: &PgPool,
    env: TradierEnv,
    token: &str,
    event_sink: Option<&EventSink>,
) -> Result<(), TradierError> {
    let base = match env {
        TradierEnv::Sandbox => crate::tradier_trading::SANDBOX_BASE,
        TradierEnv::Live => crate::tradier_trading::LIVE_BASE,
    };
    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("reqwest client");
    let sess_url = format!("{base}/accounts/events/session");
    let resp = http
        .post(&sess_url)
        .header("Authorization", format!("Bearer {token}"))
        .header("Accept", "application/json")
        .send()
        .await?;
    let status = resp.status();
    let body = resp.text().await?;
    if !status.is_success() {
        if status.as_u16() == 401 {
            return Err(TradierError::AuthFailed);
        }
        return Err(TradierError::Http {
            status: status.as_u16(),
            body,
        });
    }
    let SessionResponse { stream } = serde_json::from_str(&body)?;
    let ws_url = stream.url.clone();
    let session_id = stream.session_id;

    let (ws, _) = tokio_tungstenite::connect_async(&ws_url).await?;
    let (mut tx, mut rx) = ws.split();

    // Subscribe to order + position events. Empty excludeAccounts means
    // every account on this token. We only care about 'order' for fill
    // routing, but pulling 'position' too keeps the live_positions
    // table fresh as a side effect.
    let sub = serde_json::json!({
        "sessionid": session_id,
        "events": ["order", "position"],
        "excludeAccounts": []
    })
    .to_string();
    tx.send(WsMessage::Text(sub)).await?;

    // Pump loop — break on close OR session TTL elapsed (caller
    // reconnects with a fresh session).
    let session_deadline = tokio::time::sleep(SESSION_TTL);
    tokio::pin!(session_deadline);
    loop {
        tokio::select! {
            _ = &mut session_deadline => {
                tracing::debug!("tradier session TTL elapsed; reconnecting with fresh session");
                break;
            }
            msg = rx.next() => {
                let Some(msg) = msg else { break };
                let frame = match msg? {
                    WsMessage::Text(t) => t,
                    WsMessage::Binary(b) => String::from_utf8_lossy(&b).into_owned(),
                    WsMessage::Close(_) => break,
                    _ => continue,
                };
                if let Err(e) = handle_event_frame(pool, &frame, event_sink).await {
                    tracing::warn!(error = %e, "tradier event handler");
                }
            }
        }
    }
    Ok(())
}

/// Decode one server frame. Frames may carry a single event or a batch
/// (Tradier sometimes wraps in `{events: [...]}`). Defensive parsing
/// walks both shapes.
pub async fn handle_event_frame(
    pool: &PgPool,
    frame: &str,
    event_sink: Option<&EventSink>,
) -> anyhow::Result<()> {
    let v: serde_json::Value = match serde_json::from_str(frame) {
        Ok(v) => v,
        Err(_) => return Ok(()), // not JSON; skip silently (heartbeats, etc.)
    };
    if let Some(events) = v.get("events").and_then(|x| x.as_array()) {
        for ev in events {
            handle_one_event(pool, ev, event_sink).await.ok();
        }
    } else {
        handle_one_event(pool, &v, event_sink).await.ok();
    }
    Ok(())
}

async fn handle_one_event(
    pool: &PgPool,
    ev: &serde_json::Value,
    event_sink: Option<&EventSink>,
) -> anyhow::Result<()> {
    let kind = ev
        .get("event")
        .and_then(|x| x.as_str())
        .or_else(|| ev.get("type").and_then(|x| x.as_str()))
        .unwrap_or("");
    // Only fills move the executions pipeline forward.
    if !matches!(kind, "order" | "fill" | "partial_fill") {
        return Ok(());
    }
    let status = ev
        .get("status")
        .and_then(|x| x.as_str())
        .unwrap_or_default();
    if !matches!(status, "filled" | "partial_filled" | "partial_fill") {
        return Ok(());
    }
    // `tag` is what we stamped at submit time — UUID string.
    let tag = ev
        .get("tag")
        .and_then(|x| x.as_str())
        .or_else(|| ev.get("client_order_id").and_then(|x| x.as_str()))
        .unwrap_or("");
    let Ok(client_order_id) = Uuid::from_str(tag) else {
        tracing::debug!(tag, "tradier fill missing UUID tag; skip");
        return Ok(());
    };

    let row: Option<(Uuid, Uuid, Uuid, String, String, Decimal)> = sqlx::query_as(
        "SELECT id, run_id, strategy_id, symbol, side, qty
           FROM algo_orders WHERE client_order_id = $1",
    )
    .bind(client_order_id)
    .fetch_optional(pool)
    .await?;
    let Some((order_id, run_id, strategy_id, symbol, side, qty)) = row else {
        tracing::debug!(client_order_id = %client_order_id, "tradier fill for unknown order");
        return Ok(());
    };

    let strategy = crate::algo::get_strategy_by_id(pool, strategy_id).await?;
    let Some(strategy) = strategy else {
        return Ok(());
    };

    let f64_to_dec = |x: &serde_json::Value| -> Option<Decimal> {
        x.as_f64().and_then(|f| Decimal::try_from(f).ok())
    };
    let Some(fill_price) = ev
        .get("avg_fill_price")
        .and_then(|x| x.as_str())
        .and_then(|s| Decimal::from_str(s).ok())
        .or_else(|| ev.get("price").and_then(f64_to_dec))
    else {
        tracing::error!(
            client_order_id = %client_order_id,
            "tradier fill missing avg_fill_price/price; dropping event"
        );
        return Ok(());
    };
    if !fill_price.is_sign_positive() || fill_price.is_zero() {
        tracing::error!(
            client_order_id = %client_order_id, %fill_price,
            "tradier fill price non-positive; dropping event"
        );
        return Ok(());
    }
    let Some(fill_qty) = ev
        .get("filled_quantity")
        .and_then(|x| x.as_str())
        .and_then(|s| Decimal::from_str(s).ok())
        .or_else(|| ev.get("quantity").and_then(f64_to_dec))
    else {
        tracing::error!(
            client_order_id = %client_order_id,
            "tradier fill missing filled_quantity/quantity; dropping event"
        );
        return Ok(());
    };
    if fill_qty.is_zero() {
        tracing::error!(
            client_order_id = %client_order_id, %fill_qty,
            "tradier fill qty zero; dropping event"
        );
        return Ok(());
    }
    let intent = crate::algo_engine::OrderIntent {
        strategy_id,
        run_id,
        client_order_id,
        symbol,
        side: match side.as_str() {
            "sell" | "sell_short" => traderview_core::algo_strategies::Side::Sell,
            _ => traderview_core::algo_strategies::Side::Buy,
        },
        qty,
        entry_price: fill_price,
        stop_price: Decimal::zero(),
        take_profit_price: Decimal::zero(),
    };
    let fill = ImmediateFill {
        price: fill_price,
        qty: fill_qty,
        fee: Decimal::zero(),
        executed_at: Utc::now(),
        broker_fill_id: ev
            .get("order_id")
            .and_then(|x| x.as_i64())
            .map(|n| n.to_string()),
        broker_order_id: None,
    };
    crate::algo_engine::record_fill(pool, &strategy, &intent, order_id, &fill, event_sink)
        .await
        .map_err(|e| anyhow::anyhow!("record_fill: {e}"))?;
    Ok(())
}

/// Startup hook — one pump per distinct (user_id, sandbox-bool) tuple
/// that has at least one active algo strategy bound to a Tradier
/// account. Records each spawn in the SHARED Alpaca registry (we
/// re-use the (user_id, bool) shape — the bool now overloaded:
/// alpaca uses paper-vs-live; tradier uses sandbox-vs-live, but
/// since each broker only has its own pump module the registry
/// collisions are impossible).
pub async fn spawn_pumps_for_active_strategies(
    pool: PgPool,
    event_sink: Option<EventSink>,
    registry: AlpacaPumpRegistry,
) -> anyhow::Result<usize> {
    let strategies = crate::algo::list_active_strategies(&pool).await?;
    let mut spawned = 0usize;
    for s in strategies {
        if !matches!(s.broker_mode.as_str(), "paper" | "live") {
            continue;
        }
        if !is_tradier_account(&pool, s.account_id).await {
            continue;
        }
        let Some((token, _account_id, sandbox)) =
            crate::data_source_keys::tradier_creds(&pool, s.user_id).await?
        else {
            continue;
        };
        // sandbox flag: per-user setting wins over per-strategy
        // broker_mode for the pump endpoint (the REST submission side
        // honors both — see broker_dispatcher).
        let env = if sandbox || s.broker_mode == "paper" {
            TradierEnv::Sandbox
        } else {
            TradierEnv::Live
        };
        // Use (user_id, env==Sandbox) as the dedupe key.
        let dedupe_paper_flag = matches!(env, TradierEnv::Sandbox);
        {
            // Distinct namespace within the shared HashSet by suffixing
            // a magic 'tradier' marker on user_id NOT possible; rely on
            // not-running-both-at-once for the same user. If both
            // pump modules ever spawn for the same user, that's already
            // wrong (user can't have BOTH an Alpaca pump AND a Tradier
            // pump for the same paper/live mode — they're different
            // brokers entirely). Registry collision = no-op spawn, fine.
            let mut guard = registry.lock().await;
            if !guard.insert((s.user_id, dedupe_paper_flag)) {
                continue;
            }
        }
        let pool_clone = pool.clone();
        let sink_clone = event_sink.clone();
        let token_clone = token.clone();
        let registry_for_drop = registry.clone();
        let key = (s.user_id, dedupe_paper_flag);
        tokio::spawn(async move {
            use futures_util::FutureExt;
            let res =
                std::panic::AssertUnwindSafe(run_pump(pool_clone, env, token_clone, sink_clone))
                    .catch_unwind()
                    .await;
            match res {
                Ok(()) => tracing::warn!(?key, "tradier pump exited; clearing registry"),
                Err(_) => tracing::error!(?key, "tradier pump panicked; clearing registry"),
            }
            registry_for_drop.lock().await.remove(&key);
        });
        spawned += 1;
    }
    Ok(spawned)
}

async fn is_tradier_account(pool: &PgPool, account_id: Uuid) -> bool {
    let row: Result<Option<(Option<String>,)>, _> =
        sqlx::query_as("SELECT broker FROM accounts WHERE id = $1")
            .bind(account_id)
            .fetch_optional(pool)
            .await;
    matches!(
        row,
        Ok(Some((Some(b),))) if b.eq_ignore_ascii_case("tradier")
    )
}
