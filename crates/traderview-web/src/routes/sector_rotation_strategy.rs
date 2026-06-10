//! Faber-style sector momentum rotation strategy route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use traderview_db::sector_rotation_strategy;

pub fn router() -> Router<AppState> {
    Router::new().route("/sector-rotation-strategy/run", get(run))
}

#[derive(Deserialize)]
struct RunQ {
    #[serde(default = "default_days")]
    days_back: i64,
    #[serde(default = "default_lookback")]
    lookback_months: u32,
    #[serde(default = "default_top_k")]
    top_k: u32,
}
fn default_days() -> i64 {
    730
}
fn default_lookback() -> u32 {
    6
}
fn default_top_k() -> u32 {
    3
}

async fn run(
    State(s): State<AppState>,
    _user: AuthUser,
    Query(q): Query<RunQ>,
) -> Result<Json<sector_rotation_strategy::StrategyReport>, ApiError> {
    if !(q.lookback_months >= 1 && q.lookback_months <= 24) {
        return Err(ApiError::BadRequest(
            "lookback_months must be in [1, 24]".into(),
        ));
    }
    if !(q.top_k >= 1 && q.top_k <= 11) {
        return Err(ApiError::BadRequest("top_k must be in [1, 11]".into()));
    }
    Ok(Json(
        sector_rotation_strategy::run_strategy(&s.pool, q.days_back, q.lookback_months, q.top_k)
            .await?,
    ))
}
