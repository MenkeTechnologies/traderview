//! Post-earnings drift route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use traderview_db::pead_tracker;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/pead/recent", get(recent))
        .route("/pead/top-drift", get(top_drift))
}

#[derive(Deserialize)]
struct LookQ {
    #[serde(default = "default_days")]
    days: i64,
    #[serde(default = "default_limit")]
    limit: usize,
}
fn default_days() -> i64 {
    90
}
fn default_limit() -> usize {
    100
}

async fn recent(
    State(s): State<AppState>,
    _user: AuthUser,
    Query(q): Query<LookQ>,
) -> Result<Json<Vec<pead_tracker::PeadRow>>, ApiError> {
    Ok(Json(pead_tracker::recent(&s.pool, q.days, q.limit).await?))
}

async fn top_drift(
    State(s): State<AppState>,
    _user: AuthUser,
    Query(q): Query<LookQ>,
) -> Result<Json<Vec<pead_tracker::PeadRow>>, ApiError> {
    Ok(Json(
        pead_tracker::top_drift(&s.pool, q.days, q.limit).await?,
    ))
}
