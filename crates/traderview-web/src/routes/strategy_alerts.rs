use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use traderview_db::strategy_alerts::{
    EvalStats, StrategyAlert, StrategyAlertFire, StrategyAlertInput,
};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/strategy-alerts",            get(list).post(create))
        .route("/strategy-alerts/:id",        axum::routing::delete(delete).put(update))
        .route("/strategy-alerts/fires",      get(recent_fires))
        .route("/strategy-alerts/evaluate-now", post(evaluate_now))
}

async fn list(
    State(s): State<AppState>,
    u: AuthUser,
) -> Result<Json<Vec<StrategyAlert>>, ApiError> {
    Ok(Json(traderview_db::strategy_alerts::list(&s.pool, u.id)
        .await.map_err(ApiError::Internal)?))
}

async fn create(
    State(s): State<AppState>,
    u: AuthUser,
    Json(body): Json<StrategyAlertInput>,
) -> Result<Json<StrategyAlert>, ApiError> {
    if body.name.trim().is_empty() {
        return Err(ApiError::BadRequest("name required".into()));
    }
    Ok(Json(traderview_db::strategy_alerts::create(&s.pool, u.id, &body)
        .await.map_err(ApiError::Internal)?))
}

async fn update(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<StrategyAlertInput>,
) -> Result<Json<StrategyAlert>, ApiError> {
    traderview_db::strategy_alerts::update(&s.pool, u.id, id, &body)
        .await.map_err(ApiError::Internal)?
        .map(Json).ok_or(ApiError::NotFound)
}

async fn delete(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let ok = traderview_db::strategy_alerts::delete(&s.pool, u.id, id)
        .await.map_err(ApiError::Internal)?;
    if !ok { return Err(ApiError::NotFound); }
    Ok(Json(serde_json::json!({ "deleted": true })))
}

async fn recent_fires(
    State(s): State<AppState>,
    u: AuthUser,
) -> Result<Json<Vec<StrategyAlertFire>>, ApiError> {
    Ok(Json(traderview_db::strategy_alerts::recent_fires(&s.pool, u.id, 100)
        .await.map_err(ApiError::Internal)?))
}

async fn evaluate_now(
    State(s): State<AppState>,
    _u: AuthUser,
) -> Result<Json<EvalStats>, ApiError> {
    Ok(Json(traderview_db::strategy_alerts::evaluate_all(&s.pool)
        .await.map_err(ApiError::Internal)?))
}
