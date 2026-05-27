use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use chrono::{Duration, Utc};
use serde::Deserialize;
use traderview_core::backtest::{run, walk_forward, BtResult, OptMetric, Preset, PresetKind, WfResult};
use traderview_core::BarInterval;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/backtest/run", post(run_handler))
        .route("/backtest/walk-forward", post(walk_forward_handler))
}

#[derive(Deserialize)]
struct Body {
    symbol: String,
    preset: Preset,
    #[serde(default = "default_days")]
    days: i64,
    #[serde(default = "default_capital")]
    initial_capital: f64,
    #[serde(default)]
    fee_per_trade: f64,
}
fn default_days() -> i64 { 730 }
fn default_capital() -> f64 { 10_000.0 }

async fn run_handler(State(s): State<AppState>, _u: AuthUser, Json(b): Json<Body>)
    -> Result<Json<BtResult>, ApiError>
{
    let to = Utc::now();
    let from = to - Duration::days(b.days);
    let bars = traderview_db::prices::get_bars(&s.pool, &b.symbol.to_uppercase(),
        BarInterval::D1, from, to).await.map_err(ApiError::Internal)?;
    if bars.is_empty() {
        return Err(ApiError::BadRequest(format!("no bars for {}", b.symbol)));
    }
    Ok(Json(run(&bars, b.preset, b.initial_capital, b.fee_per_trade)))
}

#[derive(Deserialize)]
struct WfBody {
    symbol: String,
    kind: PresetKind,
    #[serde(default = "default_wf_days")]
    days: i64,
    #[serde(default = "default_is_bars")]
    is_bars: usize,
    #[serde(default = "default_oos_bars")]
    oos_bars: usize,
    #[serde(default)]
    step_bars: Option<usize>,
    #[serde(default = "default_capital")]
    initial_capital: f64,
    #[serde(default)]
    fee_per_trade: f64,
    #[serde(default = "default_metric")]
    metric: OptMetric,
}
fn default_wf_days() -> i64 { 1825 }   // 5y
fn default_is_bars() -> usize { 252 }   // ~1y
fn default_oos_bars() -> usize { 63 }   // ~1q
fn default_metric() -> OptMetric { OptMetric::Return }

async fn walk_forward_handler(
    State(s): State<AppState>,
    _u: AuthUser,
    Json(b): Json<WfBody>,
) -> Result<Json<WfResult>, ApiError> {
    let to = Utc::now();
    let from = to - Duration::days(b.days);
    let bars = traderview_db::prices::get_bars(&s.pool, &b.symbol.to_uppercase(),
        BarInterval::D1, from, to).await.map_err(ApiError::Internal)?;
    if bars.len() < b.is_bars + b.oos_bars {
        return Err(ApiError::BadRequest(format!(
            "need at least {} bars for {} (got {}); pass a larger 'days' or shrink the windows",
            b.is_bars + b.oos_bars, b.symbol, bars.len()
        )));
    }
    let step = b.step_bars.unwrap_or(b.oos_bars);
    let r = walk_forward(&bars, traderview_core::backtest::WfConfig {
        kind: b.kind,
        is_bars: b.is_bars, oos_bars: b.oos_bars, step,
        initial_capital: b.initial_capital, fee_per_trade: b.fee_per_trade,
        metric: b.metric,
    });
    Ok(Json(r))
}
