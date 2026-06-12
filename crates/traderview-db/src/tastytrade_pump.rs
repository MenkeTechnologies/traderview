//! Tastytrade account streamer — fills route through `record_fill`.
//!
//! Protocol (per docs.tastytrade + tastytrade-api-js SDK 2026-06):
//!   Production URL: wss://streamer.tastyworks.com
//!   Sandbox URL:    wss://streamer.cert.tastyworks.com
//!   Connect frame:  `{"action": "connect", "value": "<auth-token>",
//!                     "auth-token": "<auth-token>"}`
//!   Subscribe:      `{"action": "account-subscribe",
//!                     "value": ["<acct1>", "<acct2>"],
//!                     "auth-token": "<auth-token>"}`
//!   Events stream as `{type: "Order"|"AccountBalance"|..., data: {...}}`.
//!   Fill events carry an `order` payload with id, status, legs,
//!   executions[]: {fill-price, quantity, fill-type}.
//!
//! Defensive parsing: tastytrade's account streamer schema evolves
//! frequently and the public docs are sparse. We walk events loosely
//! with serde_json::Value + .get() so server-side field renames don't
//! silently break fill recording.

use crate::algo_engine::{EventSink, ImmediateFill};
use crate::alpaca_pump::AlpacaPumpRegistry;
use crate::tastytrade_trading::{Auth, TastytradeEnv, TastytradeTrading};
use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use rust_decimal::prelude::Zero;
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::str::FromStr;
use std::time::Duration;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use uuid::Uuid;

const STREAMER_LIVE: &str = "wss://streamer.tastyworks.com";
const STREAMER_SANDBOX: &str = "wss://streamer.cert.tastyworks.com";

#[derive(Debug, thiserror::Error)]
enum PumpError {
    #[error("ws: {0}")]
    Ws(#[from] tokio_tungstenite::tungstenite::Error),
    /// Reserved for the future session-token expiry path. The match
    /// arm in `run_pump` already handles it; nothing constructs it
    /// today because `resolve_token` swallows errors into a unit type,
    /// but keeping the variant ensures the reconnect loop knows what
    /// to do once that wiring is plumbed.
    #[allow(dead_code)]
    #[error("auth failed")]
    AuthFailed,
}

/// Reconnect loop. Same shape as the Tradier and Alpaca pumps:
/// auth-fail bails; other errors back off up to 60s.
pub async fn run_pump(
    pool: PgPool,
    env: TastytradeEnv,
    auth: Auth,
    account_numbers: Vec<String>,
    event_sink: Option<EventSink>,
) {
    let mut backoff = Duration::from_secs(1);
    const MAX_BACKOFF: Duration = Duration::from_secs(60);
    loop {
        // Tastytrade auth: if we have a long-lived session token, use it
        // directly. Otherwise mint one via the REST client first.
        let token = match resolve_token(env, &auth).await {
            Ok(t) => t,
            Err(_) => {
                tracing::error!("tastytrade streamer auth failed; not retrying");
                return;
            }
        };
        match run_session_once(&pool, env, &token, &account_numbers, event_sink.as_ref()).await {
            Ok(()) => {
                tracing::info!("tastytrade streamer closed; reconnecting");
                backoff = Duration::from_secs(1);
            }
            Err(PumpError::AuthFailed) => {
                tracing::error!("tastytrade streamer auth failed mid-stream; not retrying");
                return;
            }
            Err(e) => {
                tracing::warn!(error = %e, ?backoff, "tastytrade streamer dropped");
            }
        }
        tokio::time::sleep(backoff).await;
        backoff = (backoff * 2).min(MAX_BACKOFF);
    }
}

/// Resolve a session token from the Auth enum. If the user only saved
/// (login, password), we hit POST /sessions via the REST client first
/// (the client caches the token internally, so this is cheap).
async fn resolve_token(env: TastytradeEnv, auth: &Auth) -> Result<String, ()> {
    match auth {
        Auth::SessionToken(t) => Ok(t.clone()),
        Auth::UserPass { .. } => {
            // We need a dummy account_number here just to construct
            // the client; the /sessions call doesn't use it.
            let client = TastytradeTrading::new(env, auth.clone(), "PLACEHOLDER".to_string());
            // Force a token by making any cheap authenticated call —
            // get_balances will fail on the placeholder account but
            // still trigger the POST /sessions exchange. We then read
            // the cached token directly via a re-issued ensure-call.
            // Simpler: just call get_balances on the placeholder and
            // ignore the eventual 404; the token will have been set.
            let _ = client.get_balances().await;
            // Now hit a dummy ensure: but TastytradeTrading doesn't
            // expose the cached token. For first-pass implementation
            // we'll re-do the login here directly to avoid leaking the
            // cache through the API.
            // Hack: call get_balances first; the request path inside
            // ensure_token does the login. We don't have a way to
            // extract the cached token without exposing it. So we'll
            // re-implement the login inline:
            // Outer match already gated to `Auth::UserPass`, but keep the
            // unreachable-replacement defensive — a new Auth variant must
            // not panic the long-lived pump task.
            let (login, password, remember_me) = match auth {
                Auth::UserPass {
                    login,
                    password,
                    remember_me,
                } => (login.clone(), password.clone(), *remember_me),
                _ => {
                    tracing::error!(
                        "tastytrade pump: unexpected Auth variant in UserPass branch; aborting login"
                    );
                    return Err(());
                }
            };
            let base = match env {
                TastytradeEnv::Sandbox => crate::tastytrade_trading::SANDBOX_BASE,
                TastytradeEnv::Live => crate::tastytrade_trading::LIVE_BASE,
            };
            let http = reqwest::Client::new();
            let resp = http
                .post(format!("{base}/sessions"))
                .header("Accept", "application/json")
                .json(&serde_json::json!({
                    "login": login,
                    "password": password,
                    "remember-me": remember_me,
                }))
                .send()
                .await
                .map_err(|_| ())?;
            if !resp.status().is_success() {
                return Err(());
            }
            let body = resp.text().await.map_err(|_| ())?;
            let v: serde_json::Value = serde_json::from_str(&body).map_err(|_| ())?;
            v.get("data")
                .and_then(|d| d.get("session-token"))
                .and_then(|t| t.as_str())
                .map(String::from)
                .ok_or(())
        }
    }
}

async fn run_session_once(
    pool: &PgPool,
    env: TastytradeEnv,
    token: &str,
    account_numbers: &[String],
    event_sink: Option<&EventSink>,
) -> Result<(), PumpError> {
    let url = match env {
        TastytradeEnv::Sandbox => STREAMER_SANDBOX,
        TastytradeEnv::Live => STREAMER_LIVE,
    };
    let (ws, _) = tokio_tungstenite::connect_async(url).await?;
    let (mut tx, mut rx) = ws.split();

    // 1. Connect: auth-token frame.
    let connect = serde_json::json!({
        "action": "connect",
        "value": account_numbers,
        "auth-token": token,
    })
    .to_string();
    tx.send(WsMessage::Text(connect)).await?;

    // 2. Subscribe to account events.
    let subscribe = serde_json::json!({
        "action": "account-subscribe",
        "value": account_numbers,
        "auth-token": token,
    })
    .to_string();
    tx.send(WsMessage::Text(subscribe)).await?;

    // Heartbeat — ping every 30s to keep the connection alive.
    let mut hb = tokio::time::interval(Duration::from_secs(30));

    loop {
        tokio::select! {
            _ = hb.tick() => {
                let h = serde_json::json!({"action":"heartbeat","auth-token": token}).to_string();
                if tx.send(WsMessage::Text(h)).await.is_err() {
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
                    tracing::warn!(error = %e, "tastytrade event handler");
                }
            }
        }
    }
    Ok(())
}

/// Decode one server frame. Tastytrade ships singletons + 'data' /
/// 'message' envelopes; we walk both.
pub async fn handle_event_frame(
    pool: &PgPool,
    frame: &str,
    event_sink: Option<&EventSink>,
) -> anyhow::Result<()> {
    let v: serde_json::Value = match serde_json::from_str(frame) {
        Ok(v) => v,
        Err(_) => return Ok(()),
    };
    // Common envelopes: top-level event OR {data: {...}} OR {messages: [...]}.
    if let Some(data) = v.get("data") {
        handle_one_event(pool, data, event_sink).await.ok();
    } else if let Some(msgs) = v.get("messages").and_then(|m| m.as_array()) {
        for m in msgs {
            handle_one_event(pool, m, event_sink).await.ok();
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
    // Tastytrade order events carry `type: "Order"` and either a status
    // field on the order body OR an executions[] array carrying the fills.
    let typ = ev.get("type").and_then(|x| x.as_str()).unwrap_or_default();
    if !matches!(typ, "Order" | "OrderFill" | "Fill") {
        return Ok(());
    }

    // Locate the order shape — may be at ev.order or at ev directly.
    let order_obj = ev.get("order").filter(|v| v.is_object()).unwrap_or(ev);

    // We tag every submission via the strategy's client_order_id UUID.
    // Tastytrade stores arbitrary metadata under an opaque key; for now
    // we rely on the broker_order_id roundtrip via algo_orders.broker_order_id.
    let broker_order_id = order_obj
        .get("id")
        .and_then(|x| x.as_i64())
        .map(|n| n.to_string())
        .or_else(|| {
            order_obj
                .get("id")
                .and_then(|x| x.as_str())
                .map(String::from)
        });
    let Some(boid) = broker_order_id else {
        return Ok(());
    };

    // Resolve to our algo_orders row by broker_order_id.
    let row: Option<(Uuid, Uuid, Uuid, Uuid, String, String, Decimal)> = sqlx::query_as(
        "SELECT id, client_order_id, run_id, strategy_id, symbol, side, qty
           FROM algo_orders WHERE broker_order_id = $1",
    )
    .bind(&boid)
    .fetch_optional(pool)
    .await?;
    let Some((order_id, client_order_id, run_id, strategy_id, symbol, side, qty)) = row else {
        tracing::debug!(broker_order_id = %boid, "tastytrade event for unknown order");
        return Ok(());
    };

    // Only act on fill states.
    let status = order_obj
        .get("status")
        .and_then(|x| x.as_str())
        .unwrap_or_default();
    if !matches!(
        status,
        "Filled" | "Routed" | "Partially Filled" | "filled" | "partial_fill"
    ) {
        return Ok(());
    }

    let f64_to_dec = |x: &serde_json::Value| -> Option<Decimal> {
        x.as_f64().and_then(|f| Decimal::try_from(f).ok())
    };
    // Try executions[].fill-price first; fall back to order-level price.
    let fill_pair: Option<(Decimal, Decimal)> = order_obj
        .get("executions")
        .and_then(|x| x.as_array())
        .and_then(|arr| arr.last())
        .and_then(|exec| {
            let p = exec
                .get("fill-price")
                .and_then(|x| x.as_str())
                .and_then(|s| Decimal::from_str(s).ok())
                .or_else(|| exec.get("fill-price").and_then(f64_to_dec))?;
            let q = exec
                .get("quantity")
                .and_then(|x| x.as_str())
                .and_then(|s| Decimal::from_str(s).ok())
                .or_else(|| exec.get("quantity").and_then(f64_to_dec))?;
            Some((p, q))
        })
        .or_else(|| {
            // Order-level price + the row's bound qty (caller pulled it).
            let p = order_obj
                .get("price")
                .and_then(|x| x.as_str())
                .and_then(|s| Decimal::from_str(s).ok())
                .or_else(|| order_obj.get("price").and_then(f64_to_dec))?;
            Some((p, qty))
        });
    let Some((fill_price, fill_qty)) = fill_pair else {
        tracing::error!(
            client_order_id = %client_order_id,
            "tastytrade fill missing price/qty in executions+order; dropping event"
        );
        return Ok(());
    };
    if !fill_price.is_sign_positive() || fill_price.is_zero() {
        tracing::error!(
            client_order_id = %client_order_id, %fill_price,
            "tastytrade fill price non-positive; dropping event"
        );
        return Ok(());
    }
    if fill_qty.is_zero() {
        tracing::error!(
            client_order_id = %client_order_id, %fill_qty,
            "tastytrade fill qty zero; dropping event"
        );
        return Ok(());
    }

    let strategy = crate::algo::get_strategy_by_id(pool, strategy_id).await?;
    let Some(strategy) = strategy else {
        return Ok(());
    };

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
    let fill = ImmediateFill {
        price: fill_price,
        qty: fill_qty,
        fee: Decimal::zero(),
        executed_at: Utc::now(),
        broker_fill_id: Some(boid),
        broker_order_id: None,
    };
    crate::algo_engine::record_fill(pool, &strategy, &intent, order_id, &fill, event_sink)
        .await
        .map_err(|e| anyhow::anyhow!("record_fill: {e}"))?;
    Ok(())
}

/// Startup hook — one pump per distinct (user_id, sandbox-bool) tuple
/// with at least one active Tastytrade-bound strategy.
pub async fn spawn_pumps_for_active_strategies(
    pool: PgPool,
    event_sink: Option<EventSink>,
    registry: AlpacaPumpRegistry,
) -> anyhow::Result<usize> {
    let strategies = crate::algo::list_active_strategies(&pool).await?;
    let mut spawned = 0usize;
    // Group strategies by user_id; collect account numbers per user.
    use std::collections::HashMap;
    let mut by_user: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
    for s in strategies {
        if !matches!(s.broker_mode.as_str(), "paper" | "live") {
            continue;
        }
        if !is_tastytrade_account(&pool, s.account_id).await {
            continue;
        }
        by_user.entry(s.user_id).or_default().push(s.account_id);
    }
    for (user_id, _) in by_user.into_iter() {
        let Some((account_number, sandbox, auth)) =
            crate::data_source_keys::tastytrade_creds(&pool, user_id).await?
        else {
            continue;
        };
        let env = if sandbox {
            TastytradeEnv::Sandbox
        } else {
            TastytradeEnv::Live
        };
        // Dedupe in the registry by (user_id, sandbox-bool).
        {
            let mut guard = registry.lock().await;
            if !guard.insert((user_id, sandbox)) {
                continue;
            }
        }
        let pool_clone = pool.clone();
        let sink_clone = event_sink.clone();
        let auth_clone = auth.clone();
        let registry_for_drop = registry.clone();
        let key = (user_id, sandbox);
        tokio::spawn(async move {
            use futures_util::FutureExt;
            let res = std::panic::AssertUnwindSafe(run_pump(
                pool_clone,
                env,
                auth_clone,
                vec![account_number],
                sink_clone,
            ))
            .catch_unwind()
            .await;
            match res {
                Ok(()) => tracing::warn!(?key, "tastytrade pump exited; clearing registry"),
                Err(_) => tracing::error!(?key, "tastytrade pump panicked; clearing registry"),
            }
            registry_for_drop.lock().await.remove(&key);
        });
        spawned += 1;
    }
    Ok(spawned)
}

async fn is_tastytrade_account(pool: &PgPool, account_id: Uuid) -> bool {
    let row: Result<Option<(Option<String>,)>, _> =
        sqlx::query_as("SELECT broker FROM accounts WHERE id = $1")
            .bind(account_id)
            .fetch_optional(pool)
            .await;
    matches!(
        row,
        Ok(Some((Some(b),))) if b.eq_ignore_ascii_case("tastytrade")
    )
}
