//! Confluence autotrade pipeline routes.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use traderview_db::confluence_autotrade as ca;

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/confluence/autotrade/config",
            get(get_config).put(put_config),
        )
        .route("/confluence/autotrade/log", get(log))
        .route("/confluence/autotrade/run-once", post(run_once))
}

async fn get_config(
    State(s): State<AppState>,
    user: AuthUser,
) -> Result<Json<ca::AutotradeConfig>, ApiError> {
    Ok(Json(ca::get_config(&s.pool, user.id).await?))
}

#[derive(Deserialize)]
struct ConfigPatch {
    enabled: bool,
    min_score: f64,
    min_distinct_sources: i32,
    notional_usd: f64,
    cooldown_minutes: i32,
    max_open_positions: i32,
}

async fn put_config(
    State(s): State<AppState>,
    user: AuthUser,
    Json(p): Json<ConfigPatch>,
) -> Result<Json<ca::AutotradeConfig>, ApiError> {
    let cfg = ca::AutotradeConfig {
        user_id: user.id,
        enabled: p.enabled,
        min_score: p.min_score,
        min_distinct_sources: p.min_distinct_sources,
        notional_usd: p.notional_usd,
        cooldown_minutes: p.cooldown_minutes,
        max_open_positions: p.max_open_positions,
        updated_at: chrono::Utc::now(),
    };
    Ok(Json(ca::upsert_config(&s.pool, &cfg).await?))
}

#[derive(Deserialize)]
struct LimitQ {
    #[serde(default = "default_limit")]
    limit: i64,
}
fn default_limit() -> i64 {
    100
}

async fn log(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<LimitQ>,
) -> Result<Json<Vec<ca::AutotradeLogRow>>, ApiError> {
    Ok(Json(ca::recent_log(&s.pool, user.id, q.limit).await?))
}

async fn run_once(
    State(s): State<AppState>,
    user: AuthUser,
) -> Result<Json<ca::RunOnceResult>, ApiError> {
    Ok(Json(ca::run_once(&s.pool, user.id).await?))
}
