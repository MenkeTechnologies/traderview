use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use traderview_db::premarket::PremarketSnapshot;

pub fn router() -> Router<AppState> {
    Router::new().route("/premarket/snapshot", get(snapshot))
}

async fn snapshot(
    State(s): State<AppState>,
    _u: AuthUser,
) -> Result<Json<PremarketSnapshot>, ApiError> {
    Ok(Json(
        traderview_db::premarket::snapshot(&s.pool)
            .await
            .map_err(ApiError::Internal)?,
    ))
}
