use crate::auth::AuthUser;
use crate::background;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};

pub fn router() -> Router<AppState> {
    Router::new().route("/sectors", get(list))
}

/// Served from the precomputed tile cache — the background refresher
/// recomputes the 11-ETF ranking on interval (see background.rs).
async fn list(
    State(s): State<AppState>,
    _user: AuthUser,
) -> Result<Json<serde_json::Value>, ApiError> {
    Ok(Json(
        background::tile(&s.pool, &s.tiles, background::SECTORS)
            .await
            .map_err(ApiError::Internal)?,
    ))
}
