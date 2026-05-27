use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use traderview_core::scan::Preset;
use traderview_db::scans::ScanRun;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new().route("/scans/run", get(run))
}

#[derive(Deserialize)]
struct RunQ {
    preset: Preset,
    watchlist_id: Option<Uuid>,
    #[serde(default = "default_limit")]
    limit: usize,
}
fn default_limit() -> usize { 50 }

async fn run(State(s): State<AppState>, user: AuthUser, Query(q): Query<RunQ>)
    -> Result<Json<ScanRun>, ApiError>
{
    Ok(Json(traderview_db::scans::run_preset(&s.pool, user.id, q.preset, q.watchlist_id, q.limit)
        .await.map_err(ApiError::Internal)?))
}
