//! Per-symbol research — quote, signals, news, earnings, dividends,
//! recommendations, insiders, fundamentals. All backed by Yahoo Finance.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use chrono::{Duration, Utc};
use serde::Deserialize;
use serde_json::Value;
use traderview_core::signals::{analyze, SignalReport};
use traderview_core::BarInterval;
use traderview_db::market_data::{NewsItem, QuoteSnapshot};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/symbols/:symbol/quote", get(quote))
        .route("/symbols/:symbol/signals", get(signals))
        .route("/symbols/:symbol/news", get(news))
        .route("/symbols/:symbol/earnings", get(earnings))
        .route("/symbols/:symbol/dividends", get(dividends))
        .route("/symbols/:symbol/recommendations", get(recommendations))
        .route("/symbols/:symbol/insiders", get(insiders))
        .route("/symbols/:symbol/fundamentals", get(fundamentals))
        .route("/symbols/:symbol/holders", get(holders))
        .route("/dividends/calendar", get(dividends_calendar))
}

async fn quote(
    State(s): State<AppState>,
    _user: AuthUser,
    Path(symbol): Path<String>,
) -> Result<Json<QuoteSnapshot>, ApiError> {
    Ok(Json(
        traderview_db::market_data::quote(&s.pool, &symbol)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

#[derive(Deserialize)]
struct SigQ {
    #[serde(default = "default_days")]
    days: i64,
}
fn default_days() -> i64 {
    365
}

async fn signals(
    State(s): State<AppState>,
    _user: AuthUser,
    Path(symbol): Path<String>,
    Query(q): Query<SigQ>,
) -> Result<Json<SignalReport>, ApiError> {
    let to = Utc::now();
    let from = to - Duration::days(q.days);
    let bars = traderview_db::prices::get_bars(&s.pool, &symbol, BarInterval::D1, from, to)
        .await
        .map_err(ApiError::Internal)?;
    if bars.is_empty() {
        return Err(ApiError::BadRequest(format!("no price bars for {symbol}")));
    }
    Ok(Json(analyze(&symbol, &bars)))
}

#[derive(Deserialize)]
struct NewsQ {
    #[serde(default = "default_news")]
    count: usize,
}
fn default_news() -> usize {
    20
}

async fn news(
    State(_s): State<AppState>,
    _user: AuthUser,
    Path(symbol): Path<String>,
    Query(q): Query<NewsQ>,
) -> Result<Json<Vec<NewsItem>>, ApiError> {
    Ok(Json(
        traderview_db::market_data::news(&symbol, q.count)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

async fn earnings(
    _s: State<AppState>,
    _u: AuthUser,
    Path(symbol): Path<String>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(
        traderview_db::market_data::earnings(&symbol)
            .await
            .map_err(ApiError::Internal)?,
    ))
}
async fn dividends(
    _s: State<AppState>,
    _u: AuthUser,
    Path(symbol): Path<String>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(
        traderview_db::market_data::dividends(&symbol)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

#[derive(Deserialize)]
struct DivCalQ {
    #[serde(default = "default_cal_days")]
    days: i64,
}
fn default_cal_days() -> i64 {
    14
}
async fn dividends_calendar(
    _s: State<AppState>,
    _u: AuthUser,
    Query(q): Query<DivCalQ>,
) -> Result<Json<Value>, ApiError> {
    // Clamp the horizon so we never fan out an unbounded number of per-date
    // Nasdaq calls. 1..=90 days.
    let days = q.days.clamp(1, 90);
    let from = Utc::now().date_naive();
    let to = from + Duration::days(days);
    Ok(Json(
        traderview_db::market_data::dividends_calendar(from, to)
            .await
            .map_err(ApiError::Internal)?,
    ))
}
async fn recommendations(
    _s: State<AppState>,
    _u: AuthUser,
    Path(symbol): Path<String>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(
        traderview_db::market_data::recommendations(&symbol)
            .await
            .map_err(ApiError::Internal)?,
    ))
}
async fn insiders(
    _s: State<AppState>,
    _u: AuthUser,
    Path(symbol): Path<String>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(
        traderview_db::market_data::insiders(&symbol)
            .await
            .map_err(ApiError::Internal)?,
    ))
}
async fn fundamentals(
    _s: State<AppState>,
    _u: AuthUser,
    Path(symbol): Path<String>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(
        traderview_db::market_data::fundamentals(&symbol)
            .await
            .map_err(ApiError::Internal)?,
    ))
}
async fn holders(
    _s: State<AppState>,
    _u: AuthUser,
    Path(symbol): Path<String>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(
        traderview_db::market_data::holders(&symbol)
            .await
            .map_err(ApiError::Internal)?,
    ))
}
