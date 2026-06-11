//! Classic-valuation API surface:
//!   GET  /symbols/:sym/fundamental-health  — Piotroski + Altman + Graham
//!   POST /calc/dcf                         — two-stage DCF intrinsic value
//!   GET  /symbols/:sym/gap-stats           — gap fill-rate statistics
//!   GET  /symbols/:sym/seasonality         — monthly-returns seasonality

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use traderview_db::dcf_valuation::{self, DcfInput, DcfReport};
use traderview_db::fundamental_health::{self, FundamentalHealth};
use traderview_db::gap_stats::{self, GapReport};

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/symbols/:symbol/fundamental-health",
            get(get_fundamental_health),
        )
        .route("/calc/dcf", post(post_dcf))
        .route("/symbols/:symbol/gap-stats", get(get_gap_stats))
        .route("/symbols/:symbol/seasonality", get(get_seasonality))
}

fn validate_symbol(s: &str) -> Result<String, ApiError> {
    let sym = s.trim().to_uppercase();
    if sym.is_empty() || sym.len() > 20 || sym.contains('/') || sym.contains('\\') {
        return Err(ApiError::BadRequest("invalid symbol".into()));
    }
    Ok(sym)
}

async fn get_fundamental_health(
    State(s): State<AppState>,
    _u: AuthUser,
    Path(symbol): Path<String>,
) -> Result<Json<FundamentalHealth>, ApiError> {
    let sym = validate_symbol(&symbol)?;
    let health = fundamental_health::compute(&sym)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!(e)))?;
    // Attach the live price for the Graham-upside calc. Quote failures
    // degrade gracefully — the health card renders without upside.
    let health = match traderview_db::market_data::quote(&s.pool, &sym).await {
        Ok(q) => fundamental_health::with_price(health, q.price),
        Err(_) => health,
    };
    Ok(Json(health))
}

async fn post_dcf(
    State(_s): State<AppState>,
    _u: AuthUser,
    Json(input): Json<DcfInput>,
) -> Result<Json<DcfReport>, ApiError> {
    dcf_valuation::compute(&input)
        .map(Json)
        .map_err(|e| ApiError::BadRequest(e.to_string()))
}

#[derive(Debug, Deserialize)]
struct GapQ {
    /// Minimum gap size in percent to count (default 0.5).
    threshold: Option<f64>,
}

async fn get_gap_stats(
    State(s): State<AppState>,
    _u: AuthUser,
    Path(symbol): Path<String>,
    Query(q): Query<GapQ>,
) -> Result<Json<GapReport>, ApiError> {
    let sym = validate_symbol(&symbol)?;
    match gap_stats::compute(&s.pool, &sym, q.threshold).await {
        Ok(r) => Ok(Json(r)),
        Err(gap_stats::GapError::Insufficient { symbol, got, need }) => Err(
            ApiError::BadRequest(format!(
                "not enough daily bars for {symbol}: have {got}, need {need}"
            )),
        ),
        Err(gap_stats::GapError::PriceFetch(e)) => Err(ApiError::Internal(e)),
    }
}

async fn get_seasonality(
    State(s): State<AppState>,
    _u: AuthUser,
    Path(symbol): Path<String>,
) -> Result<Json<traderview_core::monthly_seasonality::MonthlySeasonalityReport>, ApiError> {
    let sym = validate_symbol(&symbol)?;
    // 10 years of daily closes gives ≥9 observations per calendar month.
    let to = chrono::Utc::now();
    let from = to - chrono::Duration::days(365 * 10);
    let bars = traderview_db::prices::get_bars(
        &s.pool,
        &sym,
        traderview_core::BarInterval::D1,
        from,
        to,
    )
    .await
    .map_err(ApiError::Internal)?;
    use chrono::Datelike;
    let closes: Vec<traderview_core::monthly_seasonality::DailyClose> = bars
        .iter()
        .map(|b| traderview_core::monthly_seasonality::DailyClose {
            year: b.bar_time.year() as u16,
            month: b.bar_time.month() as u8,
            close: b.close.to_string().parse().unwrap_or(0.0),
        })
        .collect();
    traderview_core::monthly_seasonality::compute(&closes)
        .map(Json)
        .ok_or_else(|| {
            ApiError::BadRequest(format!(
                "not enough history for {sym} seasonality (need ≥2 full years)"
            ))
        })
}
