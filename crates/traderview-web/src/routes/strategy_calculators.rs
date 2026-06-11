//! Strategy-calculator API surface:
//!   POST /calc/grid-trading              — grid ladder + per-grid profit
//!   POST /calc/fixed-ratio               — Ryan Jones contract thresholds
//!   POST /calc/anti-martingale           — press-winners streak sizing
//!   POST /calc/risk-of-ruin              — analytic gambler's-ruin RoR
//!   POST /calc/taylor-rule               — prescribed policy rate + stance
//!   POST /calc/sahm-rule                 — unemployment recession trigger
//!   POST /calc/misery-index              — inflation + unemployment
//!   POST /calc/valuation-gauges          — Buffett / Tobin Q / ERP / ECY
//!   POST /sim/dual-momentum              — Antonacci GEM backtest
//!   GET  /symbols/:sym/turn-of-month     — TOM seasonality stats
//!   GET  /symbols/:sym/vol-cone          — realized-vol percentile cone
//!   GET  /symbols/:sym/day-of-week       — weekday return seasonality
//!   GET  /symbols/:sym/santa-rally       — Hirsch 7-session window stats

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use traderview_db::strategy_calculators::{
    self, AntiMartingaleInput, AntiMartingaleReport, DowReport, FixedRatioInput,
    FixedRatioReport, GridInput, GridReport, SantaReport, TomError, TomReport, VolConeReport,
};
use traderview_db::strategy_simulators::{self, GemInput, GemReport, SimError};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/calc/grid-trading", post(post_grid_trading))
        .route("/calc/fixed-ratio", post(post_fixed_ratio))
        .route("/calc/anti-martingale", post(post_anti_martingale))
        .route("/calc/risk-of-ruin", post(post_risk_of_ruin))
        .route("/calc/taylor-rule", post(post_taylor_rule))
        .route("/calc/sahm-rule", post(post_sahm_rule))
        .route("/calc/misery-index", post(post_misery_index))
        .route("/calc/valuation-gauges", post(post_valuation_gauges))
        .route("/sim/dual-momentum", post(post_dual_momentum))
        .route("/symbols/:symbol/turn-of-month", get(get_turn_of_month))
        .route("/symbols/:symbol/vol-cone", get(get_vol_cone))
        .route("/symbols/:symbol/day-of-week", get(get_day_of_week))
        .route("/symbols/:symbol/santa-rally", get(get_santa_rally))
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

async fn post_risk_of_ruin(
    State(_s): State<AppState>,
    _u: AuthUser,
    Json(input): Json<traderview_core::risk_of_ruin::RuinInput>,
) -> Result<Json<traderview_core::risk_of_ruin::RuinReport>, ApiError> {
    traderview_core::risk_of_ruin::compute(&input)
        .map(Json)
        .ok_or_else(|| ApiError::BadRequest("invalid risk-of-ruin inputs".into()))
}

async fn post_taylor_rule(
    State(_s): State<AppState>,
    _u: AuthUser,
    Json(input): Json<traderview_core::macro_calculators::TaylorInput>,
) -> Result<Json<traderview_core::macro_calculators::TaylorReport>, ApiError> {
    traderview_core::macro_calculators::taylor_rule(&input)
        .map(Json)
        .ok_or_else(|| ApiError::BadRequest("invalid Taylor-rule inputs".into()))
}

#[derive(Debug, Deserialize)]
struct SahmBody {
    /// Monthly unemployment rates oldest-first, % (≥ 15 months).
    monthly_unemployment: Vec<f64>,
}

async fn post_sahm_rule(
    State(_s): State<AppState>,
    _u: AuthUser,
    Json(input): Json<SahmBody>,
) -> Result<Json<traderview_core::macro_calculators::SahmReport>, ApiError> {
    traderview_core::macro_calculators::sahm_rule(&input.monthly_unemployment)
        .map(Json)
        .ok_or_else(|| {
            ApiError::BadRequest("need >= 15 valid monthly unemployment readings".into())
        })
}

#[derive(Debug, Deserialize)]
struct MiseryBody {
    inflation: f64,
    unemployment: f64,
}

async fn post_misery_index(
    State(_s): State<AppState>,
    _u: AuthUser,
    Json(input): Json<MiseryBody>,
) -> Result<Json<traderview_core::macro_calculators::MiseryReport>, ApiError> {
    traderview_core::macro_calculators::misery_index(input.inflation, input.unemployment)
        .map(Json)
        .ok_or_else(|| ApiError::BadRequest("invalid misery-index inputs".into()))
}

#[derive(Debug, Deserialize)]
struct ValuationGaugesBody {
    // Buffett indicator (both or neither).
    total_market_cap: Option<f64>,
    gdp: Option<f64>,
    // Tobin's Q.
    market_value: Option<f64>,
    replacement_cost: Option<f64>,
    // ERP.
    pe_ratio: Option<f64>,
    treasury_yield_pct: Option<f64>,
    // Excess CAPE yield.
    cape: Option<f64>,
    real_yield_pct: Option<f64>,
}

#[derive(Debug, serde::Serialize)]
struct ValuationGaugesReport {
    buffett: Option<traderview_core::valuation_gauges::BuffettReport>,
    tobin: Option<traderview_core::valuation_gauges::TobinReport>,
    erp: Option<traderview_core::valuation_gauges::ErpReport>,
    ecy: Option<traderview_core::valuation_gauges::EcyReport>,
}

/// Composite endpoint — each gauge computes when its inputs are
/// supplied; at least one must be.
async fn post_valuation_gauges(
    State(_s): State<AppState>,
    _u: AuthUser,
    Json(b): Json<ValuationGaugesBody>,
) -> Result<Json<ValuationGaugesReport>, ApiError> {
    use traderview_core::valuation_gauges as vg;
    let report = ValuationGaugesReport {
        buffett: match (b.total_market_cap, b.gdp) {
            (Some(m), Some(g)) => vg::buffett_indicator(m, g),
            _ => None,
        },
        tobin: match (b.market_value, b.replacement_cost) {
            (Some(m), Some(r)) => vg::tobins_q(m, r),
            _ => None,
        },
        erp: match (b.pe_ratio, b.treasury_yield_pct) {
            (Some(p), Some(t)) => vg::equity_risk_premium(p, t),
            _ => None,
        },
        ecy: match (b.cape, b.real_yield_pct) {
            (Some(c), Some(r)) => vg::excess_cape_yield(c, r),
            _ => None,
        },
    };
    if report.buffett.is_none()
        && report.tobin.is_none()
        && report.erp.is_none()
        && report.ecy.is_none()
    {
        return Err(ApiError::BadRequest(
            "supply inputs for at least one gauge".into(),
        ));
    }
    Ok(Json(report))
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

fn validate_symbol(s: &str) -> Result<String, ApiError> {
    let sym = s.trim().to_uppercase();
    if sym.is_empty() || sym.len() > 20 || sym.contains('/') || sym.contains('\\') {
        return Err(ApiError::BadRequest("invalid symbol".into()));
    }
    Ok(sym)
}

fn map_tom_err(e: TomError) -> ApiError {
    match e {
        TomError::PriceFetch(inner) => ApiError::Internal(inner),
        other => ApiError::BadRequest(other.to_string()),
    }
}

async fn get_turn_of_month(
    State(s): State<AppState>,
    _u: AuthUser,
    Path(symbol): Path<String>,
    Query(q): Query<TomQ>,
) -> Result<Json<TomReport>, ApiError> {
    let sym = validate_symbol(&symbol)?;
    strategy_calculators::turn_of_month(&s.pool, &sym, q.years.unwrap_or(10))
        .await
        .map(Json)
        .map_err(map_tom_err)
}

async fn get_vol_cone(
    State(s): State<AppState>,
    _u: AuthUser,
    Path(symbol): Path<String>,
    Query(q): Query<TomQ>,
) -> Result<Json<VolConeReport>, ApiError> {
    let sym = validate_symbol(&symbol)?;
    strategy_calculators::vol_cone(&s.pool, &sym, q.years.unwrap_or(5))
        .await
        .map(Json)
        .map_err(map_tom_err)
}

async fn get_day_of_week(
    State(s): State<AppState>,
    _u: AuthUser,
    Path(symbol): Path<String>,
    Query(q): Query<TomQ>,
) -> Result<Json<DowReport>, ApiError> {
    let sym = validate_symbol(&symbol)?;
    strategy_calculators::day_of_week(&s.pool, &sym, q.years.unwrap_or(10))
        .await
        .map(Json)
        .map_err(map_tom_err)
}

async fn get_santa_rally(
    State(s): State<AppState>,
    _u: AuthUser,
    Path(symbol): Path<String>,
    Query(q): Query<TomQ>,
) -> Result<Json<SantaReport>, ApiError> {
    let sym = validate_symbol(&symbol)?;
    strategy_calculators::santa_rally(&s.pool, &sym, q.years.unwrap_or(15))
        .await
        .map(Json)
        .map_err(map_tom_err)
}
