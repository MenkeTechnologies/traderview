use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use traderview_db::heatmap::HeatmapResponse;

pub fn router() -> Router<AppState> {
    Router::new().route("/heatmap", get(heatmap))
}

async fn heatmap(State(s): State<AppState>, user: AuthUser) -> Result<Json<HeatmapResponse>, ApiError> {
    Ok(Json(traderview_db::heatmap::build(&s.pool, user.id)
        .await.map_err(ApiError::Internal)?))
}
