use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use traderview_db::journal_ai::{CachedAnalysis, LlmConfigDto};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/journal-ai/:trade_id/analyze", post(analyze))
        .route("/journal-ai/:trade_id/cached", get(cached))
        .route("/journal-ai/settings", get(get_settings).post(set_settings))
}

async fn get_settings(
    State(s): State<AppState>,
    u: AuthUser,
) -> Result<Json<LlmConfigDto>, ApiError> {
    Ok(Json(
        traderview_db::journal_ai::get_llm_settings(&s.pool, u.id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

async fn set_settings(
    State(s): State<AppState>,
    u: AuthUser,
    Json(body): Json<LlmConfigDto>,
) -> Result<Json<serde_json::Value>, ApiError> {
    traderview_db::journal_ai::set_llm_settings(&s.pool, u.id, &body)
        .await
        .map_err(ApiError::Internal)?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

async fn analyze(
    State(s): State<AppState>,
    u: AuthUser,
    Path(trade_id): Path<Uuid>,
) -> Result<Json<CachedAnalysis>, ApiError> {
    Ok(Json(
        traderview_db::journal_ai::analyze(&s.pool, u.id, trade_id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

async fn cached(
    State(s): State<AppState>,
    u: AuthUser,
    Path(trade_id): Path<Uuid>,
) -> Result<Json<Option<CachedAnalysis>>, ApiError> {
    let (_, hash) = traderview_db::journal_ai::build_context(&s.pool, u.id, trade_id)
        .await
        .map_err(ApiError::Internal)?;
    Ok(Json(
        traderview_db::journal_ai::get_cached(&s.pool, u.id, trade_id, &hash)
            .await
            .map_err(ApiError::Internal)?,
    ))
}
