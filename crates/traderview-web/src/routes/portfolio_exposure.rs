//! Portfolio factor-exposure dashboard route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use traderview_db::portfolio_exposure;

pub fn router() -> Router<AppState> {
    Router::new().route("/portfolio-exposure", get(exposure))
}

async fn exposure(
    State(s): State<AppState>,
    user: AuthUser,
) -> Result<Json<portfolio_exposure::ExposureReport>, ApiError> {
    Ok(Json(
        portfolio_exposure::compute_exposure(&s.pool, user.id).await?,
    ))
}
