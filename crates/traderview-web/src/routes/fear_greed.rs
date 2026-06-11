use crate::auth::AuthUser;
use crate::background;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};

pub fn router() -> Router<AppState> {
    Router::new().route("/fear-greed", get(snapshot))
}

/// Served from the precomputed tile cache — the background refresher
/// recomputes the 7-component index on interval (see background.rs).
async fn snapshot(
    State(s): State<AppState>,
    _u: AuthUser,
) -> Result<Json<serde_json::Value>, ApiError> {
    Ok(Json(
        background::tile(&s.pool, &s.tiles, background::FEAR_GREED)
            .await
            .map_err(ApiError::Internal)?,
    ))
}
