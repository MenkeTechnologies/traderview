use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use traderview_db::crypto::{CoinRow, Global, OnChainBtc};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/crypto/markets", get(markets))
        .route("/crypto/global", get(global_stats))
        .route("/crypto/btc/chain", get(btc_chain))
        .route("/crypto/calc/funding-arb", post(funding_arb))
}

/// Perp funding-rate arbitrage ledger — pure compute over the caller's
/// live prices and venue rates.
async fn funding_arb(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<traderview_core::funding_rate_arb::Input>,
) -> Result<Json<traderview_core::funding_rate_arb::Report>, ApiError> {
    traderview_core::funding_rate_arb::compute(&b)
        .map(Json)
        .ok_or_else(|| ApiError::BadRequest("invalid inputs: prices/notional must be positive, fee/days non-negative".into()))
}

#[derive(Deserialize)]
struct TopQ {
    #[serde(default = "default_n")]
    n: u32,
}
fn default_n() -> u32 {
    100
}

// 60-second caches (CoinGecko rate-limits hard).
type TimedCache<T> = Lazy<Arc<Mutex<Option<(Instant, T)>>>>;
static MK: TimedCache<Vec<CoinRow>> = Lazy::new(|| Arc::new(Mutex::new(None)));
static GB: TimedCache<Global> = Lazy::new(|| Arc::new(Mutex::new(None)));
static BC: TimedCache<OnChainBtc> = Lazy::new(|| Arc::new(Mutex::new(None)));

async fn markets(
    _s: State<AppState>,
    _u: AuthUser,
    Query(q): Query<TopQ>,
) -> Result<Json<Vec<CoinRow>>, ApiError> {
    {
        let g = MK.lock().await;
        if let Some((t, v)) = g.as_ref() {
            if t.elapsed() < Duration::from_secs(60) && v.len() >= q.n as usize {
                return Ok(Json(v.iter().take(q.n as usize).cloned().collect()));
            }
        }
    }
    let v = traderview_db::crypto::top(q.n)
        .await
        .map_err(ApiError::Internal)?;
    *MK.lock().await = Some((Instant::now(), v.clone()));
    Ok(Json(v))
}

async fn global_stats(_s: State<AppState>, _u: AuthUser) -> Result<Json<Global>, ApiError> {
    {
        let g = GB.lock().await;
        if let Some((t, v)) = g.as_ref() {
            if t.elapsed() < Duration::from_secs(60) {
                return Ok(Json(v.clone()));
            }
        }
    }
    let v = traderview_db::crypto::global()
        .await
        .map_err(ApiError::Internal)?;
    *GB.lock().await = Some((Instant::now(), v.clone()));
    Ok(Json(v))
}

async fn btc_chain(_s: State<AppState>, _u: AuthUser) -> Result<Json<OnChainBtc>, ApiError> {
    {
        let g = BC.lock().await;
        if let Some((t, v)) = g.as_ref() {
            if t.elapsed() < Duration::from_secs(60) {
                return Ok(Json(v.clone()));
            }
        }
    }
    let v = traderview_db::crypto::btc_onchain()
        .await
        .map_err(ApiError::Internal)?;
    *BC.lock().await = Some((Instant::now(), v.clone()));
    Ok(Json(v))
}
