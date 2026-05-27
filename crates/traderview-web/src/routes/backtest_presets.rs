use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use traderview_db::backtest_presets::{BacktestPreset, PresetInput};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/backtest-presets", get(list_mine).post(create))
        .route(
            "/backtest-presets/:id",
            axum::routing::delete(delete).put(update),
        )
        .route("/backtest-presets/public", get(list_public))
        .route("/backtest-presets/slug/:slug", get(by_slug))
        .route("/backtest-presets/slug/:slug/fork", post(fork))
}

async fn list_mine(
    State(s): State<AppState>,
    u: AuthUser,
) -> Result<Json<Vec<BacktestPreset>>, ApiError> {
    Ok(Json(
        traderview_db::backtest_presets::list_mine(&s.pool, u.id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

#[derive(Debug, Deserialize)]
struct Limit {
    limit: Option<i64>,
}

async fn list_public(
    State(s): State<AppState>,
    _u: AuthUser,
    Query(p): Query<Limit>,
) -> Result<Json<Vec<BacktestPreset>>, ApiError> {
    let limit = p.limit.unwrap_or(50).clamp(1, 200);
    Ok(Json(
        traderview_db::backtest_presets::list_public(&s.pool, limit)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

async fn create(
    State(s): State<AppState>,
    u: AuthUser,
    Json(body): Json<PresetInput>,
) -> Result<Json<BacktestPreset>, ApiError> {
    if body.name.trim().is_empty() {
        return Err(ApiError::BadRequest("name required".into()));
    }
    Ok(Json(
        traderview_db::backtest_presets::create(&s.pool, u.id, &body)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

async fn update(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<PresetInput>,
) -> Result<Json<BacktestPreset>, ApiError> {
    traderview_db::backtest_presets::update(&s.pool, u.id, id, &body)
        .await
        .map_err(ApiError::Internal)?
        .map(Json)
        .ok_or(ApiError::NotFound)
}

async fn delete(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let ok = traderview_db::backtest_presets::delete(&s.pool, u.id, id)
        .await
        .map_err(ApiError::Internal)?;
    if !ok {
        return Err(ApiError::NotFound);
    }
    Ok(Json(serde_json::json!({ "deleted": true })))
}

async fn by_slug(
    State(s): State<AppState>,
    _u: AuthUser,
    Path(slug): Path<String>,
) -> Result<Json<BacktestPreset>, ApiError> {
    traderview_db::backtest_presets::by_slug(&s.pool, &slug)
        .await
        .map_err(ApiError::Internal)?
        .map(Json)
        .ok_or(ApiError::NotFound)
}

async fn fork(
    State(s): State<AppState>,
    u: AuthUser,
    Path(slug): Path<String>,
) -> Result<Json<BacktestPreset>, ApiError> {
    Ok(Json(
        traderview_db::backtest_presets::fork(&s.pool, u.id, &slug)
            .await
            .map_err(ApiError::Internal)?,
    ))
}
