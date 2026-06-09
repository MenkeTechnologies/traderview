//! Sector RS rotation timing route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use traderview_db::{sector_rotation, sector_rotation_timing};

pub fn router() -> Router<AppState> {
    Router::new().route("/sector-timing/ranked", get(ranked))
}

async fn ranked(
    State(s): State<AppState>,
    _user: AuthUser,
) -> Result<Json<Vec<sector_rotation_timing::TimingMetrics>>, ApiError> {
    let report = sector_rotation::report(&s.pool).await?;
    Ok(Json(sector_rotation_timing::rank(&report)))
}
