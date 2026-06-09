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
        .route("/confluence/autotrade/sweep-exits", post(sweep_exits))
        .route("/confluence/autotrade/tags", get(list_tags))
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
    sizing_mode: String,
    kelly_horizon_days: i32,
    kelly_max_fraction: f64,
    correlation_gate_enabled: bool,
    max_pairwise_correlation: f64,
    correlation_window_days: i32,
    max_holding_days: i32,
    degradation_threshold_checks: i32,
    stop_loss_pct: f64,
    take_profit_pct: f64,
    trailing_stop_enabled: bool,
    trailing_stop_pct: f64,
}

async fn put_config(
    State(s): State<AppState>,
    user: AuthUser,
    Json(p): Json<ConfigPatch>,
) -> Result<Json<ca::AutotradeConfig>, ApiError> {
    if traderview_db::position_sizer::SizingMode::parse(&p.sizing_mode).is_none() {
        return Err(ApiError::BadRequest(
            "sizing_mode must be fixed_notional | half_kelly | quarter_kelly".into(),
        ));
    }
    if !(p.kelly_max_fraction > 0.0 && p.kelly_max_fraction <= 1.0) {
        return Err(ApiError::BadRequest(
            "kelly_max_fraction must be in (0, 1]".into(),
        ));
    }
    if !(p.max_pairwise_correlation > 0.0 && p.max_pairwise_correlation <= 1.0) {
        return Err(ApiError::BadRequest(
            "max_pairwise_correlation must be in (0, 1]".into(),
        ));
    }
    if !(p.correlation_window_days >= 10 && p.correlation_window_days <= 252) {
        return Err(ApiError::BadRequest(
            "correlation_window_days must be in [10, 252]".into(),
        ));
    }
    if !(p.max_holding_days >= 0 && p.max_holding_days <= 365) {
        return Err(ApiError::BadRequest(
            "max_holding_days must be in [0, 365]".into(),
        ));
    }
    if !(p.degradation_threshold_checks >= 1 && p.degradation_threshold_checks <= 20) {
        return Err(ApiError::BadRequest(
            "degradation_threshold_checks must be in [1, 20]".into(),
        ));
    }
    if !(p.stop_loss_pct >= 0.0 && p.stop_loss_pct <= 50.0) {
        return Err(ApiError::BadRequest(
            "stop_loss_pct must be in [0, 50]".into(),
        ));
    }
    if !(p.take_profit_pct >= 0.0 && p.take_profit_pct <= 200.0) {
        return Err(ApiError::BadRequest(
            "take_profit_pct must be in [0, 200]".into(),
        ));
    }
    if !(p.trailing_stop_pct >= 0.0 && p.trailing_stop_pct <= 50.0) {
        return Err(ApiError::BadRequest(
            "trailing_stop_pct must be in [0, 50]".into(),
        ));
    }
    let cfg = ca::AutotradeConfig {
        user_id: user.id,
        enabled: p.enabled,
        min_score: p.min_score,
        min_distinct_sources: p.min_distinct_sources,
        notional_usd: p.notional_usd,
        cooldown_minutes: p.cooldown_minutes,
        max_open_positions: p.max_open_positions,
        sizing_mode: p.sizing_mode,
        kelly_horizon_days: p.kelly_horizon_days,
        kelly_max_fraction: p.kelly_max_fraction,
        correlation_gate_enabled: p.correlation_gate_enabled,
        max_pairwise_correlation: p.max_pairwise_correlation,
        correlation_window_days: p.correlation_window_days,
        max_holding_days: p.max_holding_days,
        degradation_threshold_checks: p.degradation_threshold_checks,
        stop_loss_pct: p.stop_loss_pct,
        take_profit_pct: p.take_profit_pct,
        trailing_stop_enabled: p.trailing_stop_enabled,
        trailing_stop_pct: p.trailing_stop_pct,
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

async fn sweep_exits(
    State(s): State<AppState>,
    user: AuthUser,
) -> Result<Json<traderview_db::autotrade_exits::SweepResult>, ApiError> {
    Ok(Json(
        traderview_db::autotrade_exits::sweep_exits(&s.pool, user.id).await?,
    ))
}

async fn list_tags(
    State(s): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<traderview_db::autotrade_exits::PositionTag>>, ApiError> {
    let account = traderview_db::paper::ensure_default(&s.pool, user.id).await?;
    Ok(Json(
        traderview_db::autotrade_exits::list_tags(&s.pool, account.id).await?,
    ))
}
