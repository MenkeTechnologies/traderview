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
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use traderview_db::data_source_keys::{self, DataSourceKeysDto};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/data-sources", get(get_keys).post(set_keys))
        .route("/data-sources/reveal", get(reveal_keys))
        .route("/data-sources/test-alpaca", post(test_alpaca))
        .route("/data-sources/test-finnhub", post(test_finnhub))
        .route("/data-sources/test-tradier", post(test_tradier))
        .route("/data-sources/test-tastytrade", post(test_tastytrade))
        .route("/data-sources/test-ibkr", post(test_ibkr))
        .route("/data-sources/test-schwab", post(test_schwab))
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

/// Sibling of `GET /data-sources` that returns the unmasked values so
/// the Settings UI can show the user their own keys when they click
/// the reveal toggle next to a password field. Standard auth — only
/// the row's owner can read their own secrets.
async fn reveal_keys(
    State(s): State<AppState>,
    u: AuthUser,
) -> Result<Json<DataSourceKeysDto>, ApiError> {
    Ok(Json(
        data_source_keys::get_unmasked(&s.pool, u.id)
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

/// Verify Alpaca creds. Prefers the running worker's last observed
/// auth status when fresh (so we don't fight Alpaca's 1-connection-
/// per-account limit and get back `connection limit exceeded`). Only
/// falls through to opening a fresh test connection when no worker
/// has recently auth'd against the requested feed.
async fn test_alpaca(
    State(s): State<AppState>,
    u: AuthUser,
    Json(body): Json<TestAlpacaBody>,
) -> Result<Json<TestAlpacaResp>, ApiError> {
    let want_feed: &'static str = if body.use_sip { "sip" } else { "iex" };
    // Fast path — running worker holds the connection slot, so a fresh
    // test would get "connection limit exceeded". Use the worker's last
    // observed auth result instead, as long as it's recent (< 10
    // minutes — covers Alpaca's reconnect cadence with margin).
    let store = traderview_db::live_ticks::global();
    if let Some((ok, feed, age_ms)) = store.alpaca_last_auth().await {
        if age_ms < 600_000 && (feed == want_feed || feed == "crypto") {
            let msg = if ok {
                "live worker holds the connection — credentials already validated"
            } else {
                "live worker observed auth failure recently — credentials likely invalid"
            };
            return Ok(Json(TestAlpacaResp {
                ok,
                feed: want_feed,
                detail: Some(serde_json::json!({ "msg": msg, "age_ms": age_ms, "observed_feed": feed })),
            }));
        }
    }
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
                    feed: want_feed,
                    detail: Some(serde_json::json!({
                        "msg": "no Alpaca credentials configured — paste key + secret first or save them"
                    })),
                }));
            };
            (id, sec)
        }
    };
    // `want_feed` from above is what we return — drop this duplicate.
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
    let (mut ok, mut detail) = match tokio::time::timeout(Duration::from_secs(5), test).await {
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
    // Implicit-success: Alpaca rejects the second concurrent connection
    // with "connection limit exceeded" — that's only possible if the
    // FIRST connection (held by something else with the same creds)
    // succeeded. Treat as ok so the button doesn't lie when the user
    // tests right after the live worker already auth'd.
    if !ok {
        if let Some(d) = &detail {
            let msg = d.get("msg").and_then(|v| v.as_str()).unwrap_or("");
            if msg.to_lowercase().contains("connection limit") {
                ok = true;
                detail = Some(serde_json::json!({
                    "msg": "another connection holds the slot — credentials work (live worker is already authenticated)"
                }));
            }
        }
    }
    Ok(Json(TestAlpacaResp { ok, feed: want_feed, detail }))
}

#[derive(Deserialize, Default)]
struct TestFinnhubBody {
    /// Optional override — `"***"` or empty means "use stored".
    #[serde(default)]
    api_key: Option<String>,
}

#[derive(serde::Serialize)]
struct TestFinnhubResp {
    ok: bool,
    /// HTTP status from the probe call (200 / 401 / 403 / etc.). 0 on
    /// network failure or timeout.
    http_status: u16,
    detail: Option<serde_json::Value>,
}

/// Verify Finnhub creds via a REST probe — no WS, no 1-connection
/// limit dance. Hits `/api/v1/quote?symbol=AAPL&token=<key>` with a 5s
/// timeout and treats a 200 response with finite quote fields as
/// success. 401/403 with `{"error":"Invalid API key"}` is the
/// canonical bad-key shape.
async fn test_finnhub(
    State(s): State<AppState>,
    u: AuthUser,
    Json(body): Json<TestFinnhubBody>,
) -> Result<Json<TestFinnhubResp>, ApiError> {
    let mask = "***";
    let supplied = body
        .api_key
        .as_deref()
        .filter(|k| !k.is_empty() && *k != mask);
    let key = match supplied {
        Some(k) => k.to_string(),
        None => {
            let Some(k) = data_source_keys::finnhub_key_plain(&s.pool, u.id)
                .await
                .map_err(ApiError::Internal)?
            else {
                return Ok(Json(TestFinnhubResp {
                    ok: false,
                    http_status: 0,
                    detail: Some(serde_json::json!({
                        "msg": "no Finnhub key configured — paste a key first or save one"
                    })),
                }));
            };
            k
        }
    };
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return Ok(Json(TestFinnhubResp {
                ok: false,
                http_status: 0,
                detail: Some(serde_json::json!({ "msg": format!("client build: {e}") })),
            }));
        }
    };
    // reqwest's `.query` URL-encodes the params for us, so a key with
    // exotic characters (rare for Finnhub but cheap insurance) round-
    // trips correctly.
    let resp = match client
        .get("https://finnhub.io/api/v1/quote")
        .query(&[("symbol", "AAPL"), ("token", &key)])
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            return Ok(Json(TestFinnhubResp {
                ok: false,
                http_status: 0,
                detail: Some(serde_json::json!({ "msg": format!("network: {e}") })),
            }));
        }
    };
    let http_status = resp.status().as_u16();
    let body = resp.text().await.unwrap_or_default();
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap_or(serde_json::Value::Null);
    // Success criteria: HTTP 200 AND the response has a finite `c`
    // (current price) field — Finnhub returns 200 with `c: 0` and
    // empty arrays on some edge cases (delisted symbol, etc.), but
    // AAPL is rock-solid so any non-zero `c` confirms the key works.
    let ok = http_status == 200
        && parsed
            .get("c")
            .and_then(|v| v.as_f64())
            .map(|c| c > 0.0)
            .unwrap_or(false);
    let detail = if ok {
        Some(serde_json::json!({
            "msg": "Finnhub key valid — quote endpoint returned AAPL price",
            "aapl_last": parsed.get("c"),
        }))
    } else {
        // Surface Finnhub's own error string if present.
        let err_msg = parsed
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or(if http_status == 200 {
                "200 OK but no quote returned — key may have hit rate limit or have no entitlement"
            } else {
                "Finnhub rejected the request"
            });
        Some(serde_json::json!({ "msg": err_msg, "raw": parsed }))
    };
    Ok(Json(TestFinnhubResp {
        ok,
        http_status,
        detail,
    }))
}

#[derive(Deserialize)]
struct TestTradierBody {
    /// Pasted-but-not-yet-saved token; UI sends what's in the input
    /// field. Empty / mask → fall back to the stored value.
    #[serde(default)]
    access_token: Option<String>,
    #[serde(default)]
    account_id: Option<String>,
    #[serde(default = "default_true_inline")]
    sandbox: bool,
}

fn default_true_inline() -> bool { true }

#[derive(Serialize)]
struct TestTradierResp {
    ok: bool,
    http_status: u16,
    detail: Option<serde_json::Value>,
}

/// Verify Tradier creds via a balances probe. Sandbox responds
/// identically to production for this endpoint; the only difference is
/// the base URL.
async fn test_tradier(
    State(s): State<AppState>,
    u: AuthUser,
    Json(body): Json<TestTradierBody>,
) -> Result<Json<TestTradierResp>, ApiError> {
    let mask = "***";
    let supplied_token = body
        .access_token
        .as_deref()
        .filter(|k| !k.is_empty() && *k != mask);
    let supplied_acct = body
        .account_id
        .as_deref()
        .filter(|k| !k.is_empty() && *k != mask);
    let (token, account_id, sandbox) = match (supplied_token, supplied_acct) {
        (Some(t), Some(a)) => (t.to_string(), a.to_string(), body.sandbox),
        _ => {
            let Some((t, a, sb)) = data_source_keys::tradier_creds(&s.pool, u.id)
                .await
                .map_err(ApiError::Internal)?
            else {
                return Ok(Json(TestTradierResp {
                    ok: false,
                    http_status: 0,
                    detail: Some(serde_json::json!({
                        "msg": "no Tradier credentials configured — paste token + account_id first or save them"
                    })),
                }));
            };
            (t, a, sb)
        }
    };

    let env = if sandbox {
        traderview_db::tradier_trading::TradierEnv::Sandbox
    } else {
        traderview_db::tradier_trading::TradierEnv::Live
    };
    let client = traderview_db::tradier_trading::TradierTrading::new(env, &token, &account_id);
    match client.get_balances().await {
        Ok(b) => Ok(Json(TestTradierResp {
            ok: true,
            http_status: 200,
            detail: Some(serde_json::json!({
                "msg": "Tradier credentials valid — balances endpoint returned",
                "account_number": b.balances.account_number,
                "total_equity": b.balances.total_equity,
                "total_cash": b.balances.total_cash,
                "sandbox": sandbox,
            })),
        })),
        Err(e) => {
            // Map our typed error back to an HTTP-ish status the UI can act on.
            let (http_status, msg) = match &e {
                traderview_db::tradier_trading::TradierError::AuthFailed => (401, "invalid access_token".to_string()),
                traderview_db::tradier_trading::TradierError::InvalidRequest(b) => (400, format!("invalid request: {b}")),
                traderview_db::tradier_trading::TradierError::Http { status, body } => (*status, format!("http {status}: {body}")),
                other => (0, other.to_string()),
            };
            Ok(Json(TestTradierResp {
                ok: false,
                http_status,
                detail: Some(serde_json::json!({ "msg": msg, "sandbox": sandbox })),
            }))
        }
    }
}

#[derive(Deserialize)]
struct TestTastytradeBody {
    #[serde(default)]
    login: Option<String>,
    #[serde(default)]
    password: Option<String>,
    #[serde(default)]
    session_token: Option<String>,
    #[serde(default)]
    account_number: Option<String>,
    #[serde(default = "default_true_inline")]
    sandbox: bool,
}

#[derive(Serialize)]
struct TestTastytradeResp {
    ok: bool,
    http_status: u16,
    detail: Option<serde_json::Value>,
}

/// Verify Tastytrade creds via the balances endpoint. If the UI
/// supplied a session_token, use it as-is; otherwise fall back to
/// (login, password) → mints a fresh token via POST /sessions.
async fn test_tastytrade(
    State(s): State<AppState>,
    u: AuthUser,
    Json(body): Json<TestTastytradeBody>,
) -> Result<Json<TestTastytradeResp>, ApiError> {
    const MASK: &str = "***";
    fn live(v: Option<&String>) -> Option<String> {
        v.map(|s| s.as_str())
            .filter(|k| !k.is_empty() && *k != MASK)
            .map(String::from)
    }
    let token = live(body.session_token.as_ref());
    let login = live(body.login.as_ref());
    let password = live(body.password.as_ref());
    let account = live(body.account_number.as_ref());

    // Resolve creds — prefer the just-pasted values; fall back to stored.
    let (account_number, sandbox, auth) = match account {
        Some(acc) => {
            let auth = match (token.clone(), login.clone(), password.clone()) {
                (Some(t), _, _) => traderview_db::tastytrade_trading::Auth::SessionToken(t),
                (None, Some(l), Some(p)) => traderview_db::tastytrade_trading::Auth::UserPass {
                    login: l, password: p, remember_me: true,
                },
                _ => {
                    let Some((stored_acc, stored_sb, stored_auth)) =
                        traderview_db::data_source_keys::tastytrade_creds(&s.pool, u.id)
                            .await
                            .map_err(ApiError::Internal)?
                    else {
                        return Ok(Json(TestTastytradeResp {
                            ok: false,
                            http_status: 0,
                            detail: Some(serde_json::json!({
                                "msg": "no Tastytrade credentials supplied or saved"
                            })),
                        }));
                    };
                    return run_tastytrade_probe(stored_acc, stored_sb, stored_auth).await;
                }
            };
            (acc, body.sandbox, auth)
        }
        None => {
            let Some((stored_acc, stored_sb, stored_auth)) =
                traderview_db::data_source_keys::tastytrade_creds(&s.pool, u.id)
                    .await
                    .map_err(ApiError::Internal)?
            else {
                return Ok(Json(TestTastytradeResp {
                    ok: false,
                    http_status: 0,
                    detail: Some(serde_json::json!({
                        "msg": "no Tastytrade credentials supplied or saved"
                    })),
                }));
            };
            (stored_acc, stored_sb, stored_auth)
        }
    };
    run_tastytrade_probe(account_number, sandbox, auth).await
}

#[derive(Debug, serde::Deserialize)]
#[serde(default)]
struct TestIbkrBody {
    account_id: Option<String>,
    base_url: Option<String>,
    bearer_token: Option<String>,
}

impl Default for TestIbkrBody {
    fn default() -> Self {
        Self { account_id: None, base_url: None, bearer_token: None }
    }
}

#[derive(Debug, serde::Serialize)]
struct TestIbkrResp {
    ok: bool,
    http_status: u16,
    detail: Option<serde_json::Value>,
}

/// Probes /portfolio/{id}/summary against the configured (or supplied)
/// IBKR Client Portal Web API. A 200 + parseable summary = creds work;
/// 401 = gateway not authenticated; transport error = gateway not
/// running on the configured base URL.
async fn test_ibkr(
    State(s): State<AppState>,
    u: AuthUser,
    Json(body): Json<TestIbkrBody>,
) -> Result<Json<TestIbkrResp>, ApiError> {
    const MASK: &str = "***";
    fn live(v: Option<&String>) -> Option<String> {
        v.map(|s| s.as_str())
            .filter(|k| !k.is_empty() && *k != MASK)
            .map(String::from)
    }
    let supplied_account = live(body.account_id.as_ref());
    let supplied_base = live(body.base_url.as_ref());
    let supplied_token = live(body.bearer_token.as_ref());

    let (account_id, base_url, token) = match supplied_account {
        Some(a) => {
            let base = supplied_base.unwrap_or_else(|| {
                traderview_db::ibkr_trading::DEFAULT_LOCAL_BASE.to_string()
            });
            (a, base, supplied_token)
        }
        None => {
            let Some((a, b, t)) =
                traderview_db::data_source_keys::ibkr_creds(&s.pool, u.id)
                    .await
                    .map_err(ApiError::Internal)?
            else {
                return Ok(Json(TestIbkrResp {
                    ok: false,
                    http_status: 0,
                    detail: Some(serde_json::json!({
                        "msg": "no IBKR credentials supplied or saved"
                    })),
                }));
            };
            (a, b, t)
        }
    };

    let client = traderview_db::ibkr_trading::IbkrTrading::new(
        base_url.clone(), token.clone(), account_id.clone(),
    );
    match client.get_summary().await {
        Ok(sum) => Ok(Json(TestIbkrResp {
            ok: true,
            http_status: 200,
            detail: Some(serde_json::json!({
                "msg": "IBKR Client Portal credentials valid — summary endpoint returned",
                "net_liquidation": sum.net_liquidation.as_ref().and_then(|v| v.amount),
                "buying_power": sum.buying_power.as_ref().and_then(|v| v.amount),
                "base_url": base_url,
                "account_id": account_id,
                "auth_mode": if token.is_some() { "bearer" } else { "cookie_jar" },
            })),
        })),
        Err(e) => {
            let (http_status, msg) = match &e {
                traderview_db::ibkr_trading::IbkrError::AuthFailed => (
                    401,
                    "auth failed — gateway not logged in or bearer token expired".into(),
                ),
                traderview_db::ibkr_trading::IbkrError::InvalidRequest(b) => (
                    400, format!("invalid request: {b}"),
                ),
                traderview_db::ibkr_trading::IbkrError::Http { status, body } => (
                    *status, format!("http {status}: {body}"),
                ),
                traderview_db::ibkr_trading::IbkrError::Transport(t) => (
                    0,
                    format!("transport error — is the gateway running at {base_url}? ({t})"),
                ),
                other => (0, other.to_string()),
            };
            Ok(Json(TestIbkrResp {
                ok: false,
                http_status,
                detail: Some(serde_json::json!({
                    "msg": msg, "base_url": base_url, "account_id": account_id,
                })),
            }))
        }
    }
}

#[derive(Debug, serde::Deserialize, Default)]
#[serde(default)]
struct TestSchwabBody {
    client_id: Option<String>,
    client_secret: Option<String>,
    access_token: Option<String>,
    refresh_token: Option<String>,
    account_hash: Option<String>,
}

#[derive(Debug, serde::Serialize)]
struct TestSchwabResp {
    ok: bool,
    http_status: u16,
    detail: Option<serde_json::Value>,
}

/// Probes /trader/v1/accounts/{accountHash} against the configured (or
/// supplied) Schwab Trader API credentials. A 200 + parseable account
/// summary = creds work; 401 = refresh token expired (user needs to
/// re-run OAuth flow).
async fn test_schwab(
    State(s): State<AppState>,
    u: AuthUser,
    Json(body): Json<TestSchwabBody>,
) -> Result<Json<TestSchwabResp>, ApiError> {
    const MASK: &str = "***";
    fn live(v: Option<&String>) -> Option<String> {
        v.map(|s| s.as_str())
            .filter(|k| !k.is_empty() && *k != MASK)
            .map(String::from)
    }
    let supplied = (
        live(body.client_id.as_ref()),
        live(body.client_secret.as_ref()),
        live(body.access_token.as_ref()),
        live(body.refresh_token.as_ref()),
        live(body.account_hash.as_ref()),
    );

    let (client_id, client_secret, tokens, account_hash) = match supplied {
        (Some(cid), Some(csec), Some(at), Some(rt), Some(ah)) => (
            cid, csec,
            traderview_db::schwab_trading::Tokens { access_token: at, refresh_token: rt },
            ah,
        ),
        _ => {
            let Some(c) = traderview_db::data_source_keys::schwab_creds(&s.pool, u.id)
                .await
                .map_err(ApiError::Internal)?
            else {
                return Ok(Json(TestSchwabResp {
                    ok: false,
                    http_status: 0,
                    detail: Some(serde_json::json!({
                        "msg": "no Schwab credentials supplied or saved — run the OAuth flow first"
                    })),
                }));
            };
            c
        }
    };

    let pool = s.pool.clone();
    let user_id = u.id;
    let persist: traderview_db::schwab_trading::TokenCallback =
        std::sync::Arc::new(move |new_tokens| {
            let pool = pool.clone();
            tokio::spawn(async move {
                let _ = traderview_db::data_source_keys::save_schwab_tokens(
                    &pool, user_id, &new_tokens,
                ).await;
            });
        });
    let client = traderview_db::schwab_trading::SchwabTrading::new(
        client_id, client_secret, tokens, account_hash.clone(),
    ).on_token_refresh(persist);
    match client.get_account().await {
        Ok(acc) => Ok(Json(TestSchwabResp {
            ok: true,
            http_status: 200,
            detail: Some(serde_json::json!({
                "msg": "Schwab credentials valid — account summary returned",
                "account_hash": account_hash,
                "liquidation_value": acc.securities_account
                    .as_ref()
                    .and_then(|s| s.current_balances.as_ref())
                    .and_then(|b| b.liquidation_value),
                "buying_power": acc.securities_account
                    .as_ref()
                    .and_then(|s| s.current_balances.as_ref())
                    .and_then(|b| b.buying_power),
            })),
        })),
        Err(e) => {
            let (http_status, msg) = match &e {
                traderview_db::schwab_trading::SchwabError::AuthFailed => (
                    401,
                    "auth failed — refresh token expired, re-run the OAuth flow".into(),
                ),
                traderview_db::schwab_trading::SchwabError::InvalidRequest(b) => (
                    400, format!("invalid request: {b}"),
                ),
                traderview_db::schwab_trading::SchwabError::Http { status, body } => (
                    *status, format!("http {status}: {body}"),
                ),
                other => (0, other.to_string()),
            };
            Ok(Json(TestSchwabResp {
                ok: false,
                http_status,
                detail: Some(serde_json::json!({ "msg": msg, "account_hash": account_hash })),
            }))
        }
    }
}

async fn run_tastytrade_probe(
    account_number: String,
    sandbox: bool,
    auth: traderview_db::tastytrade_trading::Auth,
) -> Result<Json<TestTastytradeResp>, ApiError> {
    let env = if sandbox {
        traderview_db::tastytrade_trading::TastytradeEnv::Sandbox
    } else {
        traderview_db::tastytrade_trading::TastytradeEnv::Live
    };
    let client = traderview_db::tastytrade_trading::TastytradeTrading::new(env, auth, account_number);
    match client.get_balances().await {
        Ok(b) => Ok(Json(TestTastytradeResp {
            ok: true,
            http_status: 200,
            detail: Some(serde_json::json!({
                "msg": "Tastytrade credentials valid — balances endpoint returned",
                "account_number": b.data.account_number,
                "net_liquidating_value": b.data.net_liquidating_value,
                "equity_buying_power": b.data.equity_buying_power,
                "sandbox": sandbox,
            })),
        })),
        Err(e) => {
            let (http_status, msg) = match &e {
                traderview_db::tastytrade_trading::TastytradeError::AuthFailed => (401, "auth failed".to_string()),
                traderview_db::tastytrade_trading::TastytradeError::InvalidRequest(b) => (400, format!("invalid request: {b}")),
                traderview_db::tastytrade_trading::TastytradeError::Http { status, body } => (*status, format!("http {status}: {body}")),
                other => (0, other.to_string()),
            };
            Ok(Json(TestTastytradeResp {
                ok: false,
                http_status,
                detail: Some(serde_json::json!({ "msg": msg, "sandbox": sandbox })),
            }))
        }
    }
}
