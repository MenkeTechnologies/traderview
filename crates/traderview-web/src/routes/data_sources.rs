//! Per-user market-data provider credentials — Finnhub, Alpaca.
//!
//! `GET /data-sources`  returns the current state with secrets masked as `"***"`.
//! `POST /data-sources` upserts. Secret fields containing `"***"` or empty
//! string are interpreted as "leave the column alone" so the UI can submit
//! the form without re-typing the key.
//!
//! When the Finnhub key changes, the live-ticks WebSocket loop's in-memory
//! key slot is updated too so the live scanner picks up the new credential
//! without a process restart.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use std::time::Duration;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use traderview_db::data_source_keys::{self, DataSourceKeysDto};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/data-sources", get(get_keys).post(set_keys))
        .route("/data-sources/test-alpaca", post(test_alpaca))
}

async fn get_keys(
    State(s): State<AppState>,
    u: AuthUser,
) -> Result<Json<DataSourceKeysDto>, ApiError> {
    Ok(Json(
        data_source_keys::get(&s.pool, u.id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

async fn set_keys(
    State(s): State<AppState>,
    u: AuthUser,
    Json(body): Json<DataSourceKeysDto>,
) -> Result<Json<serde_json::Value>, ApiError> {
    data_source_keys::set(&s.pool, u.id, &body)
        .await
        .map_err(ApiError::Internal)?;
    // Best-effort refresh of the in-memory live-ticks keys so the WS
    // loops pick up the new values without a process restart. Each
    // setter falls back to the DB+env resolver — if neither yields a
    // key, leave the slot untouched.
    let store = traderview_db::live_ticks::global();
    if let Ok(Some(k)) = data_source_keys::finnhub_key_plain(&s.pool, u.id).await {
        store.set_api_key(k).await;
    }
    if let Ok(Some(k)) = data_source_keys::polygon_key_plain(&s.pool, u.id).await {
        store.set_polygon_key(k).await;
    }
    if let Ok(Some((id, secret, _paper))) =
        data_source_keys::alpaca_creds_plain(&s.pool, u.id).await
    {
        store.set_alpaca_creds(id, secret).await;
    }
    store.set_alpaca_use_sip(body.alpaca_use_sip_feed);
    // Switching key sets means the WS workers' subscription state /
    // provider choice could change — trigger a re-subscribe so the
    // active feed lines up with the credentials we just stored.
    let _ = store.restart_workers().await;
    Ok(Json(serde_json::json!({ "ok": true })))
}

#[derive(Deserialize, Default)]
struct TestAlpacaBody {
    /// Optional override — when supplied, the test uses these creds
    /// without persisting them. Lets the user verify a freshly-typed
    /// key/secret before clicking Save. Empty / mask string → fall
    /// back to the stored creds.
    #[serde(default)]
    key_id: Option<String>,
    #[serde(default)]
    secret: Option<String>,
    /// Test the SIP endpoint (`/v2/sip`, Algo Trader+) vs IEX
    /// (`/v2/iex`, free tier). Independent of the persisted toggle.
    #[serde(default)]
    use_sip: bool,
}

#[derive(serde::Serialize)]
struct TestAlpacaResp {
    ok: bool,
    feed: &'static str,
    /// First WS frame we received — usually `[{T:"success",msg:"connected"}]`
    /// or `[{T:"error",code,msg}]`. Surfaced verbatim so the user can
    /// see Alpaca's own diagnostic.
    detail: Option<serde_json::Value>,
}

/// Verify Alpaca creds by opening a WS, sending the auth frame, and
/// waiting for the first response. Times out after 5s. Test-only —
/// disconnects immediately on success, never subscribes.
async fn test_alpaca(
    State(s): State<AppState>,
    u: AuthUser,
    Json(body): Json<TestAlpacaBody>,
) -> Result<Json<TestAlpacaResp>, ApiError> {
    // Resolve creds: supplied form values win over the stored row so
    // the user can test a brand-new key before saving.
    let mask = "***";
    let valid_str = |s: &str| !s.is_empty() && s != mask;
    let (key_id, secret) = match (
        body.key_id.as_deref().filter(|s| valid_str(s)),
        body.secret.as_deref().filter(|s| valid_str(s)),
    ) {
        (Some(k), Some(s)) => (k.to_string(), s.to_string()),
        _ => {
            // Fall back to stored / env.
            let Some((id, sec, _)) = data_source_keys::alpaca_creds_plain(&s.pool, u.id)
                .await
                .map_err(ApiError::Internal)?
            else {
                return Ok(Json(TestAlpacaResp {
                    ok: false,
                    feed: if body.use_sip { "sip" } else { "iex" },
                    detail: Some(serde_json::json!({
                        "msg": "no Alpaca credentials configured — paste key + secret first or save them"
                    })),
                }));
            };
            (id, sec)
        }
    };
    let feed: &'static str = if body.use_sip { "sip" } else { "iex" };
    let url = if body.use_sip {
        traderview_db::live_ticks::ALPACA_WS_SIP
    } else {
        traderview_db::live_ticks::ALPACA_WS_IEX
    };
    // Cap the whole test at 5s so a misconfigured firewall / DNS
    // failure doesn't hang the request.
    let test = async move {
        let (ws, _) = tokio_tungstenite::connect_async(url).await?;
        let (mut tx, mut rx) = ws.split();
        let auth = serde_json::json!({
            "action": "auth",
            "key": key_id,
            "secret": secret,
        })
        .to_string();
        tx.send(WsMessage::Text(auth)).await?;
        // Alpaca's first frame on connect is `[{T:"success",msg:"connected"}]`
        // (welcome), then the auth-response frame is the second. Keep
        // reading until we see either `T:"success" msg:"authenticated"`
        // or `T:"error"`.
        for _ in 0..6 {
            let Some(msg) = rx.next().await else { break };
            let WsMessage::Text(t) = msg? else { continue };
            let v: serde_json::Value = serde_json::from_str(&t)?;
            let Some(arr) = v.as_array() else { continue };
            for ev in arr {
                let kind = ev.get("T").and_then(|x| x.as_str()).unwrap_or("");
                let msg = ev.get("msg").and_then(|x| x.as_str()).unwrap_or("");
                if kind == "error" {
                    return Ok::<_, anyhow::Error>((false, Some(ev.clone())));
                }
                if kind == "success" && msg.eq_ignore_ascii_case("authenticated") {
                    return Ok::<_, anyhow::Error>((true, Some(ev.clone())));
                }
                // welcome / status frames — keep reading
            }
        }
        Ok((false, Some(serde_json::json!({"msg": "no auth response after 6 frames"}))))
    };
    let (ok, detail) = match tokio::time::timeout(Duration::from_secs(5), test).await {
        Ok(Ok((ok, d))) => (ok, d),
        Ok(Err(e)) => (
            false,
            Some(serde_json::json!({ "msg": format!("WS error: {e}") })),
        ),
        Err(_) => (
            false,
            Some(serde_json::json!({ "msg": "timeout after 5s" })),
        ),
    };
    Ok(Json(TestAlpacaResp { ok, feed, detail }))
}
