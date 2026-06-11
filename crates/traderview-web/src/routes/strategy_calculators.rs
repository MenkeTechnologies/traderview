//! Strategy-calculator API surface:
//!   POST /calc/grid-trading              — grid ladder + per-grid profit
//!   POST /calc/fixed-ratio               — Ryan Jones contract thresholds
//!   POST /calc/anti-martingale           — press-winners streak sizing
//!   POST /sim/dual-momentum              — Antonacci GEM backtest
//!   GET  /symbols/:sym/turn-of-month     — TOM seasonality stats

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use traderview_db::strategy_calculators::{
    self, AntiMartingaleInput, AntiMartingaleReport, FixedRatioInput, FixedRatioReport,
    GridInput, GridReport, TomError, TomReport,
};
use traderview_db::strategy_simulators::{self, GemInput, GemReport, SimError};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/calc/grid-trading", post(post_grid_trading))
        .route("/calc/fixed-ratio", post(post_fixed_ratio))
        .route("/calc/anti-martingale", post(post_anti_martingale))
        .route("/sim/dual-momentum", post(post_dual_momentum))
        .route("/symbols/:symbol/turn-of-month", get(get_turn_of_month))
}

async fn post_grid_trading(
    State(_s): State<AppState>,
    _u: AuthUser,
    Json(input): Json<GridInput>,
) -> Result<Json<GridReport>, ApiError> {
    strategy_calculators::grid_trading(&input)
        .map(Json)
        .map_err(|e| ApiError::BadRequest(e.into()))
}

async fn post_fixed_ratio(
    State(_s): State<AppState>,
    _u: AuthUser,
    Json(input): Json<FixedRatioInput>,
) -> Result<Json<FixedRatioReport>, ApiError> {
    strategy_calculators::fixed_ratio(&input)
        .map(Json)
        .map_err(|e| ApiError::BadRequest(e.into()))
}

async fn post_anti_martingale(
    State(_s): State<AppState>,
    _u: AuthUser,
    Json(input): Json<AntiMartingaleInput>,
) -> Result<Json<AntiMartingaleReport>, ApiError> {
    strategy_calculators::anti_martingale(&input)
        .map(Json)
        .map_err(|e| ApiError::BadRequest(e.into()))
}

async fn post_dual_momentum(
    State(s): State<AppState>,
    _u: AuthUser,
    Json(input): Json<GemInput>,
) -> Result<Json<GemReport>, ApiError> {
    strategy_simulators::dual_momentum(&s.pool, &input)
        .await
        .map(Json)
        .map_err(|e| match e {
            SimError::PriceFetch(inner) => ApiError::Internal(inner),
            other => ApiError::BadRequest(other.to_string()),
        })
}

#[derive(Debug, Deserialize)]
struct TomQ {
    years: Option<u32>,
}

async fn get_turn_of_month(
    State(s): State<AppState>,
    _u: AuthUser,
    Path(symbol): Path<String>,
    Query(q): Query<TomQ>,
) -> Result<Json<TomReport>, ApiError> {
    let sym = symbol.trim().to_uppercase();
    if sym.is_empty() || sym.len() > 20 || sym.contains('/') || sym.contains('\\') {
        return Err(ApiError::BadRequest("invalid symbol".into()));
    }
    strategy_calculators::turn_of_month(&s.pool, &sym, q.years.unwrap_or(10))
        .await
        .map(Json)
        .map_err(|e| match e {
            TomError::PriceFetch(inner) => ApiError::Internal(inner),
            other => ApiError::BadRequest(other.to_string()),
        })
}
