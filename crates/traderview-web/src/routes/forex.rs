//! Forex (FX) endpoints — the trading-desk surface for the FX asset
//! class, the same way `crypto.rs` is for crypto.
//!
//! Live quotes ride the shared Yahoo seam (`market_data::quote` appends
//! the `=X` suffix for FX); the calculators are pure compute from
//! `traderview_core::forex_calc` (pip value / sizing / sessions). Carry
//! and CIP forwards live at `/calc/fx-carry` (the shared `fx_carry`
//! compute) and are not duplicated here.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{Datelike, Timelike, Utc};
use serde::Deserialize;
use traderview_core::forex_calc;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/forex/pairs", get(pairs))
        .route("/forex/sessions", get(sessions))
        .route("/forex/pip-value", post(pip_value))
        .route("/forex/position-size", post(position_size))
}

/// Live quotes for the major USD pairs, through the same cache equities
/// use. Empty entries (transient fetch failures) are simply absent.
async fn pairs(
    State(s): State<AppState>,
    _u: AuthUser,
) -> Json<Vec<traderview_db::market_data::QuoteSnapshot>> {
    let syms: Vec<String> = traderview_db::forex::MAJORS.iter().map(|m| m.to_string()).collect();
    Json(traderview_db::market_data::quotes(&s.pool, &syms).await)
}

/// Which FX centers are open right now, and whether the cash market is
/// open at all (it closes over the weekend).
async fn sessions(_u: AuthUser) -> Json<forex_calc::SessionStatus> {
    let now = Utc::now();
    Json(forex_calc::session_status(now.weekday().num_days_from_monday(), now.hour()))
}

#[derive(Deserialize)]
struct PipValueBody {
    pair: String,
    units: f64,
}

/// Quote-currency value of one pip on `units` of the pair.
async fn pip_value(
    _u: AuthUser,
    Json(b): Json<PipValueBody>,
) -> Result<Json<f64>, ApiError> {
    let pair = traderview_db::forex::normalize(&b.pair)
        .ok_or_else(|| ApiError::BadRequest(format!("{} is not a forex pair", b.pair)))?;
    Ok(Json(forex_calc::pip_value(&pair, b.units)))
}

#[derive(Deserialize)]
struct SizeBody {
    equity: f64,
    risk_pct: f64,
    stop_pips: f64,
    pair: String,
}

/// Risk-based position size: the lot whose loss at the stop equals
/// `equity × risk_pct/100`.
async fn position_size(
    _u: AuthUser,
    Json(b): Json<SizeBody>,
) -> Result<Json<forex_calc::PositionSize>, ApiError> {
    let pair = traderview_db::forex::normalize(&b.pair)
        .ok_or_else(|| ApiError::BadRequest(format!("{} is not a forex pair", b.pair)))?;
    forex_calc::position_size(b.equity, b.risk_pct, b.stop_pips, &pair)
        .map(Json)
        .ok_or_else(|| {
            ApiError::BadRequest("equity, risk_pct, and stop_pips must be positive".into())
        })
}
