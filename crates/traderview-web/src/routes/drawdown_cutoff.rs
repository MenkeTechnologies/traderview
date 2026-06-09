//! Drawdown auto-cutoff routes.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use traderview_db::drawdown_cutoff as dc;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/drawdown-cutoff/config", get(get_config).put(put_config))
        .route("/drawdown-cutoff/log", get(log))
        .route("/drawdown-cutoff/evaluate", post(evaluate))
        .route("/drawdown-cutoff/reset", post(reset))
}

async fn get_config(
    State(s): State<AppState>,
    user: AuthUser,
) -> Result<Json<dc::DrawdownConfig>, ApiError> {
    Ok(Json(dc::get_config(&s.pool, user.id).await?))
}

#[derive(Deserialize)]
struct ConfigPatch {
    enabled: bool,
    max_drawdown_pct: f64,
}

async fn put_config(
    State(s): State<AppState>,
    user: AuthUser,
    Json(p): Json<ConfigPatch>,
) -> Result<Json<dc::DrawdownConfig>, ApiError> {
    if !(p.max_drawdown_pct > 0.0 && p.max_drawdown_pct <= 100.0) {
        return Err(ApiError::BadRequest(
            "max_drawdown_pct must be in (0, 100]".into(),
        ));
    }
    let prior = dc::get_config(&s.pool, user.id).await?;
    let cfg = dc::DrawdownConfig {
        user_id: user.id,
        enabled: p.enabled,
        max_drawdown_pct: p.max_drawdown_pct,
        ..prior
    };
    Ok(Json(dc::upsert_config(&s.pool, &cfg).await?))
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
) -> Result<Json<Vec<dc::DrawdownLogRow>>, ApiError> {
    Ok(Json(dc::recent_log(&s.pool, user.id, q.limit).await?))
}

async fn evaluate(
    State(s): State<AppState>,
    user: AuthUser,
) -> Result<Json<dc::EvaluationResult>, ApiError> {
    Ok(Json(dc::evaluate(&s.pool, user.id).await?))
}

async fn reset(
    State(s): State<AppState>,
    user: AuthUser,
) -> Result<Json<dc::DrawdownConfig>, ApiError> {
    Ok(Json(dc::reset(&s.pool, user.id).await?))
}
