//! Schwab streaming events pump — companion to `schwab_trading`.
//!
//! Protocol (verified against developer.schwab.com 2026-06):
//!   1. GET /trader/v1/userPreference  →  carries `streamerInfo[0]`:
//!      { streamerSocketUrl, schwabClientCorrelId, schwabClientChannel,
//!        schwabClientFunctionId, schwabClientCustomerId }
//!   2. WS connect to `streamerSocketUrl`.
//!   3. Send LOGIN command:
//!      {"requests":[{"service":"ADMIN","command":"LOGIN","SchwabClientCustomerId":"…",
//!        "SchwabClientCorrelId":"…","parameters":{
//!         "Authorization":"<access_token>","SchwabClientChannel":"…",
//!         "SchwabClientFunctionId":"…" }}]}
//!   4. Subscribe to ACCT_ACTIVITY for fill events.
//!   5. Server streams `{"data":[{"service":"ACCT_ACTIVITY","content":[{
//!         "seq":…,"key":<accountKey>,
//!         "1":<subscriptionKey>,"2":<MESSAGE_TYPE>,"3":<MESSAGE_DATA xml>
//!      }]}]}`. MESSAGE_TYPE `OrderFill` / `ExecutionRouted` etc. carry
//!      an XML MESSAGE_DATA payload — we extract OrderID + price + qty
//!      via a lightweight scan (no XML dep).
//!
//! On 401 from the userPreference fetch we attempt one OAuth refresh via
//! the SchwabTrading client (which already owns the rotation lock) and
//! retry — same shape as the REST client's auto-refresh path.

use crate::algo_engine::{EventSink, ImmediateFill};
use crate::alpaca_pump::AlpacaPumpRegistry;
use crate::schwab_trading::{SchwabError, SchwabTrading, Tokens};
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

#[derive(Debug, Deserialize)]
struct UserPrefResponse {
    #[serde(rename = "streamerInfo")]
    streamer_info: Vec<StreamerInfo>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StreamerInfo {
    streamer_socket_url: String,
    schwab_client_customer_id: String,
    schwab_client_correl_id: String,
    schwab_client_channel: String,
    schwab_client_function_id: String,
}

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
}

impl From<SchwabError> for PumpError {
    fn from(e: SchwabError) -> Self {
        match e {
            SchwabError::AuthFailed => Self::AuthFailed,
            SchwabError::Transport(t) => Self::Transport(t),
            SchwabError::Http { status, body } => Self::Http { status, body },
            SchwabError::Decode(d) => Self::Decode(d),
            other => Self::Http { status: 0, body: other.to_string() },
        }
    }
}

/// Reconnect loop. Spawn-and-forget.
pub async fn run_pump(
    pool: PgPool,
    schwab: SchwabTrading,
    event_sink: Option<EventSink>,
) {
    let mut backoff = Duration::from_secs(1);
    const MAX_BACKOFF: Duration = Duration::from_secs(60);
    loop {
        match run_session_once(&pool, &schwab, event_sink.as_ref()).await {
            Ok(()) => {
                tracing::info!("schwab events stream closed; reconnecting");
                backoff = Duration::from_secs(1);
            }
            Err(PumpError::AuthFailed) => {
                // Try a refresh once; if it fails, bail (user needs UI).
                match schwab.refresh_access_token().await {
                    Ok(_) => {
                        tracing::info!("schwab access token refreshed; reconnecting");
                        backoff = Duration::from_secs(1);
                    }
                    Err(_) => {
                        tracing::error!("schwab refresh failed; pump exiting");
                        return;
                    }
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, ?backoff, "schwab events stream dropped");
            }
        }
        tokio::time::sleep(backoff).await;
        backoff = (backoff * 2).min(MAX_BACKOFF);
    }
}

async fn run_session_once(
    pool: &PgPool,
    schwab: &SchwabTrading,
    event_sink: Option<&EventSink>,
) -> Result<(), PumpError> {
    let info = fetch_user_preference(schwab).await?;
    let ws_url = info.streamer_socket_url.clone();
    let (ws, _) = tokio_tungstenite::connect_async(&ws_url).await?;
    let (mut tx, mut rx) = ws.split();

    // LOGIN request_id="1"
    let access_token = schwab_access_token_clone(schwab).await;
    let login = serde_json::json!({
        "requests": [{
            "requestid": "1",
            "service": "ADMIN",
            "command": "LOGIN",
            "SchwabClientCustomerId": info.schwab_client_customer_id,
            "SchwabClientCorrelId": info.schwab_client_correl_id,
            "parameters": {
                "Authorization": access_token,
                "SchwabClientChannel": info.schwab_client_channel,
                "SchwabClientFunctionId": info.schwab_client_function_id,
            }
        }]
    }).to_string();
    tx.send(WsMessage::Text(login)).await?;

    // SUBS ACCT_ACTIVITY (request_id="2")
    let sub = serde_json::json!({
        "requests": [{
            "requestid": "2",
            "service": "ACCT_ACTIVITY",
            "command": "SUBS",
            "SchwabClientCustomerId": info.schwab_client_customer_id,
            "SchwabClientCorrelId": info.schwab_client_correl_id,
            "parameters": { "keys": "default", "fields": "0,1,2,3" }
        }]
    }).to_string();
    tx.send(WsMessage::Text(sub)).await?;

    loop {
        let Some(msg) = rx.next().await else { break };
        let frame = match msg? {
            WsMessage::Text(t) => t,
            WsMessage::Binary(b) => String::from_utf8_lossy(&b).into_owned(),
            WsMessage::Close(_) => break,
            _ => continue,
        };
        if let Err(e) = handle_event_frame(pool, &frame, event_sink).await {
            tracing::warn!(error = %e, "schwab event handler");
        }
    }
    Ok(())
}

async fn schwab_access_token_clone(s: &SchwabTrading) -> String {
    // SchwabTrading holds the lock; ask via get_account? no — we only
    // need the access_token. Expose via refresh path: a "current access
    // token" probe. Simplest correct route: a single get_account call
    // would also work but adds latency. We rely on the internal lock
    // through a debug-only side channel — `place_order`'s `send_post`
    // builds the bearer the same way. Here we just call `refresh` only
    // if the streamer rejects; otherwise we use the token from the most
    // recent persisted value via the dispatcher's snapshot. Since
    // SchwabTrading is the canonical owner, do a cheap account fetch
    // to confirm the token is live AND extract a header echo.
    //
    // For commit 43 we keep this simple: take whatever access_token the
    // last refresh stored. SchwabTrading exposes that via the
    // `current_access_token` helper (added below). If the streamer
    // rejects we bubble up AuthFailed and the reconnect loop refreshes.
    s.current_access_token_public().await
}

async fn fetch_user_preference(s: &SchwabTrading) -> Result<StreamerInfo, PumpError> {
    let body = s.get_user_preference().await?;
    let parsed: UserPrefResponse = serde_json::from_str(&body)?;
    parsed
        .streamer_info
        .into_iter()
        .next()
        .ok_or_else(|| PumpError::Http {
            status: 0,
            body: "userPreference returned no streamerInfo".into(),
        })
}

/// One server frame.
pub async fn handle_event_frame(
    pool: &PgPool,
    frame: &str,
    event_sink: Option<&EventSink>,
) -> anyhow::Result<()> {
    let v: serde_json::Value = match serde_json::from_str(frame) {
        Ok(v) => v,
        Err(_) => return Ok(()), // skip heartbeats / pings
    };
    // Real fill data lives in `data[].content[]` with service=ACCT_ACTIVITY.
    let Some(data) = v.get("data").and_then(|x| x.as_array()) else { return Ok(()); };
    for stream in data {
        let svc = stream.get("service").and_then(|x| x.as_str()).unwrap_or("");
        if svc != "ACCT_ACTIVITY" {
            continue;
        }
        let Some(items) = stream.get("content").and_then(|x| x.as_array()) else { continue };
        for ev in items {
            handle_one_acct_activity(pool, ev, event_sink).await.ok();
        }
    }
    Ok(())
}

async fn handle_one_acct_activity(
    pool: &PgPool,
    ev: &serde_json::Value,
    event_sink: Option<&EventSink>,
) -> anyhow::Result<()> {
    // Field codes per Schwab streamer ACCT_ACTIVITY schema:
    //   "1" = subscription key
    //   "2" = MESSAGE_TYPE (OrderFill | ExecutionRouted | OrderEntryRequest…)
    //   "3" = MESSAGE_DATA (XML payload)
    let msg_type = ev.get("2").and_then(|x| x.as_str()).unwrap_or("");
    if !matches!(msg_type, "OrderFill" | "OrderActivity" | "ExecutionRouted") {
        return Ok(());
    }
    let xml = ev.get("3").and_then(|x| x.as_str()).unwrap_or("");
    if xml.is_empty() {
        return Ok(());
    }

    // Lightweight extraction — Schwab XML carries the placement's
    // `OrderID` (internal Schwab id) AND the `ClientOrderId` we stamped
    // on submit (as `<EnteredBy>` or `<OrderInstructions>` text in the
    // OrderActivity envelope). The tag we set is the strategy's
    // client_order_id UUID. We pull it via a regex-free scan that looks
    // for a UUID v4 substring — the lookup against algo_orders confirms
    // it's ours.
    let client_order_id = scan_for_uuid(xml);
    let Some(client_order_id) = client_order_id else {
        tracing::debug!("schwab fill XML missing UUID tag; skip");
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
        tracing::debug!(client_order_id = %client_order_id, "schwab fill for unknown order");
        return Ok(());
    };

    let strategy = crate::algo::get_strategy_by_id(pool, strategy_id).await?;
    let Some(strategy) = strategy else { return Ok(()); };

    let fill_price = scan_decimal_after(xml, "<ExecutionPrice>")
        .or_else(|| scan_decimal_after(xml, "<Price>"))
        .or_else(|| scan_decimal_after(xml, "<AveragePrice>"))
        .unwrap_or_else(Decimal::zero);
    let fill_qty = scan_decimal_after(xml, "<ExecutionQuantity>")
        .or_else(|| scan_decimal_after(xml, "<FilledQuantity>"))
        .or_else(|| scan_decimal_after(xml, "<Quantity>"))
        .unwrap_or(qty);
    let broker_fill_id = scan_text_after(xml, "<OrderKey>")
        .or_else(|| scan_text_after(xml, "<OrderID>"));

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

// ─── lightweight XML scanners ───────────────────────────────────────────────
// Pulling in `quick-xml` for one MESSAGE_DATA parse would be overkill;
// these helpers handle the canonical Schwab order-activity XML shape.

fn scan_text_after(xml: &str, open_tag: &str) -> Option<String> {
    let i = xml.find(open_tag)?;
    let start = i + open_tag.len();
    let rest = &xml[start..];
    let end = rest.find("</")?;
    let value = rest[..end].trim();
    if value.is_empty() { None } else { Some(value.to_string()) }
}

fn scan_decimal_after(xml: &str, open_tag: &str) -> Option<Decimal> {
    let raw = scan_text_after(xml, open_tag)?;
    Decimal::from_str(&raw).ok()
}

fn scan_for_uuid(xml: &str) -> Option<Uuid> {
    // UUIDv4 string scan — 36-char window with the expected hyphen
    // positions. Avoids a regex dep.
    let bytes = xml.as_bytes();
    if bytes.len() < 36 { return None; }
    'outer: for start in 0..=bytes.len() - 36 {
        let chunk = &xml[start..start + 36];
        if chunk.as_bytes()[8] != b'-'
            || chunk.as_bytes()[13] != b'-'
            || chunk.as_bytes()[18] != b'-'
            || chunk.as_bytes()[23] != b'-'
        {
            continue;
        }
        for (i, c) in chunk.bytes().enumerate() {
            let is_hyphen = i == 8 || i == 13 || i == 18 || i == 23;
            if is_hyphen { continue; }
            if !c.is_ascii_hexdigit() { continue 'outer; }
        }
        if let Ok(u) = Uuid::from_str(chunk) { return Some(u); }
    }
    None
}

/// Startup hook — one pump per distinct user_id with at least one
/// active Schwab-backed strategy. Reuses the AlpacaPumpRegistry shape
/// to dedupe; (user_id, false) marks the live pump slot for that user
/// (Schwab has no sandbox option — it's prod-only for retail clients).
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
        if !is_schwab_account(&pool, s.account_id).await {
            continue;
        }
        let Some((client_id, client_secret, tokens, account_hash)) =
            crate::data_source_keys::schwab_creds(&pool, s.user_id).await?
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
        let user_id = s.user_id;
        let persist_pool = pool.clone();
        let persist: crate::schwab_trading::TokenCallback =
            std::sync::Arc::new(move |new_tokens: Tokens| {
                let pool = persist_pool.clone();
                tokio::spawn(async move {
                    let _ = crate::data_source_keys::save_schwab_tokens(&pool, user_id, &new_tokens).await;
                });
            });
        let client =
            SchwabTrading::new(client_id, client_secret, tokens, account_hash)
                .on_token_refresh(persist);
        tokio::spawn(run_pump(pool_clone, client, sink_clone));
        spawned += 1;
    }
    Ok(spawned)
}

async fn is_schwab_account(pool: &PgPool, account_id: Uuid) -> bool {
    let row: Result<Option<(Option<String>,)>, _> =
        sqlx::query_as("SELECT broker FROM accounts WHERE id = $1")
            .bind(account_id)
            .fetch_optional(pool)
            .await;
    matches!(
        row,
        Ok(Some((Some(b),))) if {
            let b = b.to_ascii_lowercase();
            b == "schwab" || b == "td"
        }
    )
}

#[cfg(test)]
mod scanner_tests {
    use super::*;
    fn dec(s: &str) -> Decimal { Decimal::from_str(s).unwrap() }

    #[test]
    fn uuid_extracted_from_xml() {
        let xml = "<OrderActivity><ClientOrderId>abc</ClientOrderId>\
            <EnteredBy>123e4567-e89b-12d3-a456-426614174000</EnteredBy>\
            </OrderActivity>";
        assert_eq!(
            scan_for_uuid(xml).unwrap().to_string(),
            "123e4567-e89b-12d3-a456-426614174000"
        );
    }

    #[test]
    fn uuid_missing_returns_none() {
        assert!(scan_for_uuid("no uuid here just text").is_none());
    }

    #[test]
    fn execution_price_extracted() {
        let xml = "<OrderFillMessage><ExecutionPrice>187.55</ExecutionPrice>\
            <ExecutionQuantity>100</ExecutionQuantity></OrderFillMessage>";
        assert_eq!(scan_decimal_after(xml, "<ExecutionPrice>"), Some(dec("187.55")));
        assert_eq!(scan_decimal_after(xml, "<ExecutionQuantity>"), Some(dec("100")));
    }

    #[test]
    fn order_key_extracted() {
        let xml = "<OrderFillMessage><OrderKey>SCHWAB-9988776</OrderKey></OrderFillMessage>";
        assert_eq!(scan_text_after(xml, "<OrderKey>"), Some("SCHWAB-9988776".into()));
    }

    #[test]
    fn missing_tag_returns_none() {
        assert!(scan_decimal_after("<x>1</x>", "<y>").is_none());
        assert!(scan_text_after("<x>1</x>", "<y>").is_none());
    }
}
