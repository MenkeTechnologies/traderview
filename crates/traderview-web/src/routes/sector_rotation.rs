use crate::auth::AuthUser;
use crate::background;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};

pub fn router() -> Router<AppState> {
    Router::new().route("/sector-rotation", get(report))
}

/// Served from the precomputed tile cache — the background refresher
/// recomputes the 12-symbol rotation report on interval (see
/// background.rs).
async fn report(
    State(s): State<AppState>,
    _u: AuthUser,
) -> Result<Json<serde_json::Value>, ApiError> {
    Ok(Json(
        background::tile(&s.pool, &s.tiles, background::SECTOR_ROTATION)
            .await
            .map_err(ApiError::Internal)?,
    ))
}
