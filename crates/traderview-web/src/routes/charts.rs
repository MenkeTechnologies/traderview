use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use traderview_core::{BarInterval, PriceBar};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/bars/:symbol", get(bars))
}

#[derive(Deserialize)]
struct BarsQ {
    interval: String,
    /// Unix seconds.
    from: i64,
    /// Unix seconds.
    to: i64,
}

#[derive(Serialize)]
struct BarsResponse {
    symbol: String,
    interval: String,
    bars: Vec<PriceBar>,
}

async fn bars(
    State(s): State<AppState>,
    Path(symbol): Path<String>,
    Query(q): Query<BarsQ>,
) -> Result<Json<BarsResponse>, ApiError> {
    let iv = parse_interval(&q.interval)
        .ok_or_else(|| ApiError::BadRequest(format!("unknown interval: {}", q.interval)))?;
    let from: DateTime<Utc> = Utc.timestamp_opt(q.from, 0).single().ok_or_else(|| ApiError::BadRequest("bad from".into()))?;
    let to: DateTime<Utc> = Utc.timestamp_opt(q.to, 0).single().ok_or_else(|| ApiError::BadRequest("bad to".into()))?;
    let bars = traderview_db::prices::get_bars(&s.pool, &symbol, iv, from, to)
        .await
        .map_err(ApiError::Internal)?;
    Ok(Json(BarsResponse {
        symbol,
        interval: iv.label().into(),
        bars,
    }))
}

fn parse_interval(s: &str) -> Option<BarInterval> {
    Some(match s {
        "1m" => BarInterval::M1,
        "5m" => BarInterval::M5,
        "15m" => BarInterval::M15,
        "1h" => BarInterval::H1,
        "1d" => BarInterval::D1,
        "1w" => BarInterval::W1,
        _ => return None,
    })
}
