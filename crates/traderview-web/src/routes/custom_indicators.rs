use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{Duration, Utc};
use serde::Deserialize;
use traderview_core::BarInterval;
use traderview_db::custom_indicators::{CustomIndicator, EvalResult, IndicatorInput};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/custom-indicators", get(list).post(create))
        .route("/custom-indicators/:id", axum::routing::delete(delete))
        .route("/custom-indicators/eval/:symbol", post(eval))
}

async fn list(
    State(s): State<AppState>,
    u: AuthUser,
) -> Result<Json<Vec<CustomIndicator>>, ApiError> {
    Ok(Json(
        traderview_db::custom_indicators::list(&s.pool, u.id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

async fn create(
    State(s): State<AppState>,
    u: AuthUser,
    Json(body): Json<IndicatorInput>,
) -> Result<Json<CustomIndicator>, ApiError> {
    if body.name.trim().is_empty() {
        return Err(ApiError::BadRequest("name required".into()));
    }
    traderview_db::custom_indicators::validate(&body.definition)
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;
    Ok(Json(
        traderview_db::custom_indicators::create(&s.pool, u.id, &body)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

async fn delete(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let ok = traderview_db::custom_indicators::delete(&s.pool, u.id, id)
        .await
        .map_err(ApiError::Internal)?;
    if !ok {
        return Err(ApiError::NotFound);
    }
    Ok(Json(serde_json::json!({ "deleted": true })))
}

#[derive(Debug, Deserialize)]
struct EvalQ {
    interval: Option<String>,
    days: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct EvalBody {
    indicator_ids: Vec<Uuid>,
}

async fn eval(
    State(s): State<AppState>,
    u: AuthUser,
    Path(symbol): Path<String>,
    Query(q): Query<EvalQ>,
    Json(b): Json<EvalBody>,
) -> Result<Json<EvalResult>, ApiError> {
    let interval = match q.interval.as_deref().unwrap_or("1d") {
        "1m" => BarInterval::M1,
        "5m" => BarInterval::M5,
        "15m" => BarInterval::M15,
        "1h" => BarInterval::H1,
        "1d" => BarInterval::D1,
        "1w" => BarInterval::W1,
        other => return Err(ApiError::BadRequest(format!("unknown interval {other}"))),
    };
    let to = Utc::now();
    let from = to - Duration::days(q.days.unwrap_or(180).clamp(30, 1825));
    let sym = symbol.to_uppercase();
    Ok(Json(
        traderview_db::custom_indicators::evaluate(
            &s.pool,
            u.id,
            &sym,
            interval,
            from,
            to,
            &b.indicator_ids,
        )
        .await
        .map_err(ApiError::Internal)?,
    ))
}
