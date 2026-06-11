//! Per-symbol Buy/Sell/Hold recommendation API. Backs:
//!
//!  - the featured panel on the research view (per-symbol verdict)
//!  - the Golden Stars leaderboard view (top scored symbols)
//!  - the sector-aggregation view (verdict per SPDR ETF)
//!  - the historical-accuracy backtest panel
//!  - the verdict-change webhook watcher CRUD
//!
//! Algorithm lives in `traderview_db::stock_recommendation`; this is a
//! thin HTTP shim that handles input validation + error mapping.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use serde::Deserialize;
use traderview_db::stock_recommendation::{
    backtest, compute, compute_with_weights, cron_compute_universe, latest_for_symbol,
    leaderboard, BacktestReport, CronResult, RecommendationError, StockRecommendation,
    StoredRecommendation, WeightOverrides, DEFAULT_UNIVERSE, SECTOR_ETFS,
};
use traderview_db::stock_recommendation_watchers::{
    self as watchers, Watcher, WatcherInput,
};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/symbols/:symbol/recommendation",
            get(get_recommendation),
        )
        .route(
            "/symbols/:symbol/recommendation/backtest",
            get(get_backtest),
        )
        .route("/recommendations/golden-stars", get(get_leaderboard))
        .route("/recommendations/sectors", get(get_sectors))
        .route(
            "/recommendations/cron/run",
            post(run_cron).get(run_cron_get),
        )
        .route(
            "/recommendations/watchers",
            get(list_watchers).post(upsert_watcher),
        )
        .route("/recommendations/watchers/:id", delete(delete_watcher))
}

fn validate_symbol(s: &str) -> Result<String, ApiError> {
    let sym = s.trim().to_uppercase();
    if sym.is_empty() || sym.len() > 20 || sym.contains('/') || sym.contains('\\') {
        return Err(ApiError::BadRequest("invalid symbol".into()));
    }
    Ok(sym)
}

#[derive(Debug, Deserialize, Default)]
struct WeightQ {
    trend_w: Option<f64>,
    momentum_w: Option<f64>,
    macd_w: Option<f64>,
    rsi_w: Option<f64>,
    adx_w: Option<f64>,
    volume_w: Option<f64>,
}

impl WeightQ {
    fn to_overrides(&self) -> WeightOverrides {
        WeightOverrides {
            trend: self.trend_w,
            momentum: self.momentum_w,
            macd: self.macd_w,
            rsi: self.rsi_w,
            adx: self.adx_w,
            volume: self.volume_w,
        }
    }
}

async fn get_recommendation(
    State(s): State<AppState>,
    _u: AuthUser,
    Path(symbol): Path<String>,
    Query(wq): Query<WeightQ>,
) -> Result<Json<StockRecommendation>, ApiError> {
    let sym = validate_symbol(&symbol)?;
    let overrides = wq.to_overrides();
    let result = if overrides.any() {
        compute_with_weights(&s.pool, &sym, Some(&overrides)).await
    } else {
        compute(&s.pool, &sym).await
    };
    match result {
        Ok(r) => Ok(Json(r)),
        Err(RecommendationError::Insufficient { symbol, got, need }) => Err(
            ApiError::BadRequest(format!(
                "not enough price history for {symbol}: have {got}, need {need}"
            )),
        ),
        Err(RecommendationError::InvalidPrice(p)) => Err(ApiError::BadRequest(format!(
            "latest close is non-positive: {p}"
        ))),
        Err(RecommendationError::PriceFetch(e)) => Err(ApiError::Internal(e)),
    }
}

#[derive(Debug, Deserialize)]
struct BacktestQ {
    horizon: Option<usize>,
}

async fn get_backtest(
    State(s): State<AppState>,
    _u: AuthUser,
    Path(symbol): Path<String>,
    Query(q): Query<BacktestQ>,
) -> Result<Json<BacktestReport>, ApiError> {
    let sym = validate_symbol(&symbol)?;
    // Cap horizon to avoid letting the user fetch a year of bars per
    // signal — keep it within a typical swing trader's window.
    let horizon = q.horizon.map(|h| h.clamp(1, 60));
    match backtest(&s.pool, &sym, horizon).await {
        Ok(r) => Ok(Json(r)),
        Err(RecommendationError::Insufficient { symbol, got, need }) => Err(
            ApiError::BadRequest(format!(
                "not enough bars to backtest {symbol}: have {got}, need {need}"
            )),
        ),
        Err(RecommendationError::InvalidPrice(p)) => Err(ApiError::BadRequest(format!(
            "non-positive price: {p}"
        ))),
        Err(RecommendationError::PriceFetch(e)) => Err(ApiError::Internal(e)),
    }
}

#[derive(Debug, Deserialize)]
struct LeaderboardQ {
    #[serde(default = "default_leaderboard_limit")]
    limit: i64,
    /// Filter: only return verdicts with score ≥ min_score. Default 0
    /// shows everything; a UI tab for "Buy candidates only" passes 60.
    min_score: Option<f64>,
}

fn default_leaderboard_limit() -> i64 {
    50
}

async fn get_leaderboard(
    State(s): State<AppState>,
    _u: AuthUser,
    Query(q): Query<LeaderboardQ>,
) -> Result<Json<Vec<StoredRecommendation>>, ApiError> {
    let limit = q.limit.clamp(1, 500);
    Ok(Json(
        leaderboard(&s.pool, limit, q.min_score)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

/// Sector heatmap: serve the latest stored recommendation for each of
/// the 11 SPDR ETFs. All of them are in DEFAULT_UNIVERSE, so the
/// background refresh keeps them fresh — this route never computes.
async fn get_sectors(
    State(s): State<AppState>,
    _u: AuthUser,
) -> Result<Json<Vec<SectorEntry>>, ApiError> {
    let mut out = Vec::with_capacity(SECTOR_ETFS.len());
    for (ticker, name) in SECTOR_ETFS {
        match latest_for_symbol(&s.pool, ticker).await {
            Ok(Some(r)) => out.push(SectorEntry {
                ticker: ticker.to_string(),
                name: name.to_string(),
                recommendation: Some(r),
                error: None,
            }),
            Ok(None) => out.push(SectorEntry {
                ticker: ticker.to_string(),
                name: name.to_string(),
                recommendation: None,
                error: Some("pending background compute".to_string()),
            }),
            Err(e) => out.push(SectorEntry {
                ticker: ticker.to_string(),
                name: name.to_string(),
                recommendation: None,
                error: Some(format!("{e}")),
            }),
        }
    }
    Ok(Json(out))
}

#[derive(Debug, serde::Serialize)]
struct SectorEntry {
    ticker: String,
    name: String,
    recommendation: Option<StoredRecommendation>,
    error: Option<String>,
}

/// Manual trigger of the nightly compute. POST = run; GET = identical
/// (admins call this from a browser tab). The auth check is identical
/// to the other gated routes — adding admin-only gating is a follow-up.
async fn run_cron(
    State(s): State<AppState>,
    _u: AuthUser,
) -> Result<Json<CronRunResult>, ApiError> {
    run_cron_inner(s).await
}

async fn run_cron_get(
    State(s): State<AppState>,
    _u: AuthUser,
) -> Result<Json<CronRunResult>, ApiError> {
    run_cron_inner(s).await
}

#[derive(Debug, serde::Serialize)]
struct CronRunResult {
    compute: CronResult,
    fired_alerts: usize,
}

async fn run_cron_inner(s: AppState) -> Result<Json<CronRunResult>, ApiError> {
    let compute_res = cron_compute_universe(&s.pool, DEFAULT_UNIVERSE).await;
    let fired_alerts = watchers::check_and_fire(&s.pool)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "watcher check_and_fire failed");
            0
        });
    Ok(Json(CronRunResult {
        compute: compute_res,
        fired_alerts,
    }))
}

// ─── watcher CRUD ────────────────────────────────────────────────────

async fn list_watchers(
    State(s): State<AppState>,
    u: AuthUser,
) -> Result<Json<Vec<Watcher>>, ApiError> {
    Ok(Json(
        watchers::list(&s.pool, u.id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

async fn upsert_watcher(
    State(s): State<AppState>,
    u: AuthUser,
    Json(input): Json<WatcherInput>,
) -> Result<Json<Watcher>, ApiError> {
    if input.symbol.trim().is_empty() {
        return Err(ApiError::BadRequest("symbol required".into()));
    }
    Ok(Json(
        watchers::upsert(&s.pool, u.id, input)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

async fn delete_watcher(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<bool>, ApiError> {
    Ok(Json(
        watchers::delete(&s.pool, u.id, id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}
