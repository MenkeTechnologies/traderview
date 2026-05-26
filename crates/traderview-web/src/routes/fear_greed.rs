use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use traderview_db::fear_greed::FearGreed;

pub fn router() -> Router<AppState> {
    Router::new().route("/fear-greed", get(snapshot))
}

async fn snapshot(State(s): State<AppState>, _u: AuthUser) -> Result<Json<FearGreed>, ApiError> {
    Ok(Json(traderview_db::fear_greed::snapshot(&s.pool).await.map_err(ApiError::Internal)?))
}
