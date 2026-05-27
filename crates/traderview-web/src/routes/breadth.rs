use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use traderview_db::breadth::BreadthSnapshot;

pub fn router() -> Router<AppState> {
    Router::new().route("/breadth/snapshot", get(snapshot))
}

async fn snapshot(State(s): State<AppState>, _u: AuthUser) -> Result<Json<BreadthSnapshot>, ApiError> {
    Ok(Json(traderview_db::breadth::snapshot(&s.pool).await.map_err(ApiError::Internal)?))
}
