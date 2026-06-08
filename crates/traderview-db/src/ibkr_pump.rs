//! IBKR Client Portal Web API streaming events pump — companion to
//! `ibkr_trading`.
//!
//! Protocol (verified against interactivebrokers.github.io/cpwebapi
//! /websockets, 2026-06):
//!   1. Tickle /tickle to refresh session before connect.
//!   2. WS connect to wss://<gateway-host>/v1/api/ws  (the gateway's
//!      WS endpoint mirrors the REST base — the dispatcher hands us
//!      the configured base URL and we swap https→wss).
//!   3. Send `s<topic>` text frames to subscribe. We send:
//!        - `sor+{}`  → Live orders feed.
//!        - `str+{}`  → Trades feed (fills).
//!        - `tic`     → Keepalive tickle (every 60s).
//!   4. Server emits JSON frames:
//!        - `topic:"sor", args:[{orderId, status, ...}]`
//!        - `topic:"str", args:[{execId, orderId, symbol, side, price, size, ...}]`
//!   5. On disconnect (5xx, idle timeout, gateway re-auth) the
//!      reconnect loop tickles + re-subscribes.
//!
//! IBKR fills land on the `str` (trades) topic. We match each fill's
//! `orderId` to our `algo_orders.broker_order_id` lookup — since the
//! place_order response carried that id back to the engine on submit,
//! the row exists. Failing that, we fall back to a UUID scan of any
//! `cOID` carried on the order envelope (IBKR echoes the client-side
//! tag we set on submit).

use crate::algo_engine::{EventSink, ImmediateFill};
use crate::alpaca_pump::AlpacaPumpRegistry;
use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use rust_decimal::prelude::Zero;
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::str::FromStr;
use std::time::Duration;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
enum PumpError {
    #[error("auth failed")]
    AuthFailed,
    #[error("transport: {0}")]
    Transport(#[from] reqwest::Error),
    #[error("ws: {0}")]
    Ws(#[from] tokio_tungstenite::tungstenite::Error),
    #[error("http {status}: {body}")]
    Http { status: u16, body: String },
    #[error("decode: {0}")]
    Decode(#[from] serde_json::Error),
    #[error("config: {0}")]
    Config(String),
}

const TICKLE_INTERVAL: Duration = Duration::from_secs(60);

pub async fn run_pump(
    pool: PgPool,
    rest_base: String,
    bearer: Option<String>,
    event_sink: Option<EventSink>,
) {
    let mut backoff = Duration::from_secs(1);
    const MAX_BACKOFF: Duration = Duration::from_secs(60);
    loop {
        match run_session_once(&pool, &rest_base, bearer.as_deref(), event_sink.as_ref()).await {
            Ok(()) => {
                tracing::info!("ibkr events stream closed; reconnecting");
                backoff = Duration::from_secs(1);
            }
            Err(PumpError::AuthFailed) => {
                tracing::error!("ibkr WS auth failed; gateway needs login");
                return;
            }
            Err(e) => {
                tracing::warn!(error = %e, ?backoff, "ibkr events stream dropped");
            }
        }
        tokio::time::sleep(backoff).await;
        backoff = (backoff * 2).min(MAX_BACKOFF);
    }
}

async fn run_session_once(
    pool: &PgPool,
    rest_base: &str,
    bearer: Option<&str>,
    event_sink: Option<&EventSink>,
) -> Result<(), PumpError> {
    // Step 1 — POST /tickle so the session is alive.
    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .cookie_store(true)
        .danger_accept_invalid_certs(true)
        .build()?;
    let tickle_url = format!("{rest_base}/tickle");
    let mut req = http.post(&tickle_url).header("Accept", "application/json");
    if let Some(t) = bearer {
        req = req.header("Authorization", format!("Bearer {t}"));
    }
    let resp = req.send().await?;
    let status = resp.status();
    let _ = resp.text().await?;
    if !status.is_success() {
        if status.as_u16() == 401 { return Err(PumpError::AuthFailed); }
        return Err(PumpError::Http { status: status.as_u16(), body: "tickle".into() });
    }

    // Step 2 — WS connect. Swap https→wss / http→ws and append /ws.
    let ws_url = derive_ws_url(rest_base)?;
    let (ws, _) = tokio_tungstenite::connect_async(&ws_url).await?;
    let (mut tx, mut rx) = ws.split();

    // Step 3 — subscribe to orders + trades.
    // IBKR uses plaintext "s<topic>+{json}" frames. Empty {} = default.
    tx.send(WsMessage::Text("sor+{}".into())).await?;
    tx.send(WsMessage::Text("str+{}".into())).await?;

    // Step 4 — tickle every TICKLE_INTERVAL inside the loop.
    let mut tickle = tokio::time::interval(TICKLE_INTERVAL);
    tickle.tick().await; // skip immediate fire
    loop {
        tokio::select! {
            _ = tickle.tick() => {
                // text "tic" keeps the WS alive even if the REST
                // /tickle path is unreachable for a moment.
                if tx.send(WsMessage::Text("tic".into())).await.is_err() {
                    break;
                }
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
                    tracing::warn!(error = %e, "ibkr event handler");
                }
            }
        }
    }
    Ok(())
}

fn derive_ws_url(rest_base: &str) -> Result<String, PumpError> {
    // rest_base looks like "https://localhost:5000/v1/api". Swap the
    // scheme and append "/ws" — IBKR's gateway exposes the WS on the
    // same prefix.
    let (scheme, rest) = if let Some(r) = rest_base.strip_prefix("https://") {
        ("wss", r)
    } else if let Some(r) = rest_base.strip_prefix("http://") {
        ("ws", r)
    } else {
        return Err(PumpError::Config(format!("unrecognised IBKR base URL: {rest_base}")));
    };
    Ok(format!("{scheme}://{rest}/ws"))
}

/// One server frame. IBKR can emit `topic:"sts"` (system status),
/// `topic:"sor"` (order lifecycle), `topic:"str"` (trades / fills),
/// and various `topic:"act"` flavors. We only act on `str`.
pub async fn handle_event_frame(
    pool: &PgPool,
    frame: &str,
    event_sink: Option<&EventSink>,
) -> anyhow::Result<()> {
    let v: serde_json::Value = match serde_json::from_str(frame) {
        Ok(v) => v,
        Err(_) => return Ok(()), // not JSON; skip (pings etc.)
    };
    let topic = v.get("topic").and_then(|x| x.as_str()).unwrap_or("");
    if topic != "str" {
        return Ok(());
    }
    let Some(items) = v.get("args").and_then(|x| x.as_array()) else { return Ok(()); };
    for ev in items {
        handle_one_trade(pool, ev, event_sink).await.ok();
    }
    Ok(())
}

async fn handle_one_trade(
    pool: &PgPool,
    ev: &serde_json::Value,
    event_sink: Option<&EventSink>,
) -> anyhow::Result<()> {
    // IBKR trade event shape (canonical fields):
    //   { execution_id, order_id (or orderId), symbol, side ("BOT"/"SLD"),
    //     price, size, account, conid, cOID (client order id echo) }
    let broker_order_id = ev
        .get("orderId")
        .and_then(|x| x.as_str())
        .or_else(|| ev.get("order_id").and_then(|x| x.as_str()))
        .map(String::from);

    let coid = ev
        .get("cOID")
        .and_then(|x| x.as_str())
        .and_then(|s| Uuid::from_str(s).ok());

    let row: Option<(Uuid, Uuid, Uuid, String, String, Decimal, Uuid)> =
        if let Some(coid) = coid {
            sqlx::query_as(
                "SELECT id, run_id, strategy_id, symbol, side, qty, client_order_id
                   FROM algo_orders WHERE client_order_id = $1",
            )
            .bind(coid)
            .fetch_optional(pool)
            .await?
        } else if let Some(boid) = broker_order_id.as_deref() {
            sqlx::query_as(
                "SELECT id, run_id, strategy_id, symbol, side, qty, client_order_id
                   FROM algo_orders WHERE broker_order_id = $1",
            )
            .bind(boid)
            .fetch_optional(pool)
            .await?
        } else {
            None
        };

    let Some((order_id, run_id, strategy_id, symbol, side, qty, client_order_id)) = row else {
        tracing::debug!(?broker_order_id, ?coid, "ibkr trade for unknown order");
        return Ok(());
    };

    let strategy = crate::algo::get_strategy_by_id(pool, strategy_id).await?;
    let Some(strategy) = strategy else { return Ok(()); };

    let f64_to_dec = |x: &serde_json::Value| -> Option<Decimal> {
        x.as_f64().and_then(|f| Decimal::try_from(f).ok())
    };
    let fill_price = ev
        .get("price")
        .and_then(f64_to_dec)
        .or_else(|| ev.get("price").and_then(|x| x.as_str()).and_then(|s| Decimal::from_str(s).ok()))
        .unwrap_or_else(Decimal::zero);
    let fill_qty = ev
        .get("size")
        .and_then(f64_to_dec)
        .or_else(|| ev.get("size").and_then(|x| x.as_str()).and_then(|s| Decimal::from_str(s).ok()))
        .unwrap_or(qty);
    let broker_fill_id = ev
        .get("execution_id")
        .and_then(|x| x.as_str())
        .map(String::from);

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
        broker_fill_id,
    };
    crate::algo_engine::record_fill(pool, &strategy, &intent, order_id, &fill, event_sink)
        .await
        .map_err(|e| anyhow::anyhow!("record_fill: {e}"))?;
    Ok(())
}

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
        if !is_ibkr_account(&pool, s.account_id).await {
            continue;
        }
        let Some((_account_id, base_url, bearer)) =
            crate::data_source_keys::ibkr_creds(&pool, s.user_id).await?
        else {
            continue;
        };
        {
            let mut guard = registry.lock().await;
            if !guard.insert((s.user_id, false)) {
                continue;
            }
        }
        let pool_clone = pool.clone();
        let sink_clone = event_sink.clone();
        tokio::spawn(run_pump(pool_clone, base_url, bearer, sink_clone));
        spawned += 1;
    }
    Ok(spawned)
}

async fn is_ibkr_account(pool: &PgPool, account_id: Uuid) -> bool {
    let row: Result<Option<(Option<String>,)>, _> =
        sqlx::query_as("SELECT broker FROM accounts WHERE id = $1")
            .bind(account_id)
            .fetch_optional(pool)
            .await;
    matches!(
        row,
        Ok(Some((Some(b),))) if b.eq_ignore_ascii_case("ibkr")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derive_ws_url_swaps_scheme_and_appends_ws() {
        assert_eq!(
            derive_ws_url("https://localhost:5000/v1/api").unwrap(),
            "wss://localhost:5000/v1/api/ws"
        );
        assert_eq!(
            derive_ws_url("http://gateway.internal:8080/v1/api").unwrap(),
            "ws://gateway.internal:8080/v1/api/ws"
        );
    }

    #[test]
    fn derive_ws_url_rejects_bad_scheme() {
        assert!(derive_ws_url("ftp://gateway").is_err());
        assert!(derive_ws_url("localhost").is_err());
    }
}
