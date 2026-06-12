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
        .route("/crypto/calc/funding-arb-live", post(funding_arb_live))
        .route("/crypto/funding-scan", get(funding_scan))
        .route("/crypto/positioning", post(positioning))
}

#[derive(Deserialize)]
struct FundingArbLiveBody {
    base: String,
    notional_usd: f64,
    #[serde(default = "default_taker_fee")]
    taker_fee_pct: f64,
    #[serde(default = "default_days_held")]
    days_held: f64,
}
fn default_taker_fee() -> f64 {
    0.05
}
fn default_days_held() -> f64 {
    30.0
}

#[derive(serde::Serialize)]
struct FundingArbLive {
    snapshot: traderview_db::crypto::FundingSnapshot,
    report: traderview_core::funding_rate_arb::Report,
    /// Realized-funding regime over the last ~30 intervals — a rich
    /// rate that's a one-interval spike is a trap, not a trade.
    persistence: Option<traderview_db::crypto::FundingPersistence>,
}

/// Funding arb with LIVE OKX inputs: spot/perp/funding fetched fresh,
/// the venue's variable funding interval normalized to the 8h
/// convention, then the same pure ledger. Returns the snapshot used so
/// the numbers are auditable.
async fn funding_arb_live(
    _s: State<AppState>,
    _u: AuthUser,
    Json(b): Json<FundingArbLiveBody>,
) -> Result<Json<FundingArbLive>, ApiError> {
    let snapshot = traderview_db::crypto::funding_snapshot(&b.base)
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;
    // History failure degrades to None — the ledger still answers.
    let persistence = traderview_db::crypto::funding_history(&b.base, 30)
        .await
        .ok()
        .as_deref()
        .and_then(traderview_db::crypto::funding_persistence);
    let report = traderview_core::funding_rate_arb::compute(&traderview_core::funding_rate_arb::Input {
        spot_price: snapshot.spot,
        perp_price: snapshot.perp,
        funding_rate_8h: snapshot.funding_rate_8h,
        notional_usd: b.notional_usd,
        taker_fee_pct: b.taker_fee_pct,
        days_held: b.days_held,
    })
    .ok_or_else(|| ApiError::BadRequest("invalid inputs".into()))?;
    Ok(Json(FundingArbLive { snapshot, report, persistence }))
}

/// Funding sweep across the major-perp universe, 5-minute cached
/// (funding rates move per-interval, not per-second; the sweep is
/// ~36 venue calls).
async fn funding_scan(
    _s: State<AppState>,
    _u: AuthUser,
) -> Result<Json<traderview_db::crypto::FundingScan>, ApiError> {
    {
        let g = FS.lock().await;
        if let Some((t, v)) = g.as_ref() {
            if t.elapsed() < Duration::from_secs(5 * 60) {
                return Ok(Json(v.clone()));
            }
        }
    }
    let scan = traderview_db::crypto::funding_scan().await;
    *FS.lock().await = Some((Instant::now(), scan.clone()));
    Ok(Json(scan))
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
static FS: TimedCache<traderview_db::crypto::FundingScan> = Lazy::new(|| Arc::new(Mutex::new(None)));

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

#[derive(serde::Deserialize)]
struct PositioningBody {
    base: String,
}

/// Live OKX positioning: OI×price quadrant, long/short account ratio,
/// taker flow, funding — who is positioned where, in one read.
async fn positioning(
    Json(b): Json<PositioningBody>,
) -> Result<Json<traderview_db::crypto::Positioning>, ApiError> {
    traderview_db::crypto::positioning(&b.base)
        .await
        .map(Json)
        .map_err(|e| ApiError::BadRequest(e.to_string()))
}
