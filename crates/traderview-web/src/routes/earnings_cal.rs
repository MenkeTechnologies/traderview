use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use traderview_db::earnings_cal::{EarningsEvent, PollStats};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/earnings/calendar",  get(calendar))
        .route("/earnings/surprises", get(surprises))
        .route("/earnings/poll-now",  post(poll_now))
        .route("/earnings/symbol/:symbol/refresh", post(refresh_symbol))
}

#[derive(Debug, Deserialize)]
struct DaysQ { days: Option<i64> }

async fn calendar(
    State(s): State<AppState>,
    _u: AuthUser,
    Query(p): Query<DaysQ>,
) -> Result<Json<Vec<EarningsEvent>>, ApiError> {
    let days = p.days.unwrap_or(7).clamp(1, 90);
    Ok(Json(traderview_db::earnings_cal::calendar_upcoming(&s.pool, days)
        .await.map_err(ApiError::Internal)?))
}

async fn surprises(
    State(s): State<AppState>,
    _u: AuthUser,
    Query(p): Query<DaysQ>,
) -> Result<Json<Vec<EarningsEvent>>, ApiError> {
    let days = p.days.unwrap_or(30).clamp(1, 365);
    Ok(Json(traderview_db::earnings_cal::surprises_recent(&s.pool, days)
        .await.map_err(ApiError::Internal)?))
}

async fn poll_now(
    State(s): State<AppState>,
    _u: AuthUser,
) -> Result<Json<PollStats>, ApiError> {
    Ok(Json(traderview_db::earnings_cal::poll_watchlists(&s.pool)
        .await.map_err(ApiError::Internal)?))
}

async fn refresh_symbol(
    State(s): State<AppState>,
    _u: AuthUser,
    Path(symbol): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let sym = symbol.to_uppercase();
    let (events, reactions) = traderview_db::earnings_cal::poll_symbol(&s.pool, &sym)
        .await.map_err(ApiError::Internal)?;
    Ok(Json(serde_json::json!({
        "symbol": sym, "events_upserted": events, "reactions_computed": reactions
    })))
}
