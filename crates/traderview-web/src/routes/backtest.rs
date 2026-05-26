use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use chrono::{Duration, Utc};
use serde::Deserialize;
use traderview_core::backtest::{run, BtResult, Preset};
use traderview_core::BarInterval;

pub fn router() -> Router<AppState> {
    Router::new().route("/backtest/run", post(run_handler))
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
