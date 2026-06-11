//! Classic-valuation API surface:
//!   GET  /symbols/:sym/fundamental-health  — Piotroski + Altman + Graham
//!   POST /calc/dcf                         — two-stage DCF intrinsic value
//!   GET  /symbols/:sym/gap-stats           — gap fill-rate statistics
//!   GET  /symbols/:sym/seasonality         — monthly-returns seasonality

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use traderview_db::dcf_valuation::{self, DcfInput, DcfReport};
use traderview_db::fundamental_health::{self, FundamentalHealth};
use traderview_db::gap_stats::{self, GapReport};
use traderview_db::rrg;
use traderview_db::valuation_models::{
    self, DdmInput, DdmReport, EpvInput, EpvReport, ReverseDcfInput, ReverseDcfReport,
    RuleOf40Report, WheelInput, WheelReport,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/symbols/:symbol/fundamental-health",
            get(get_fundamental_health),
        )
        .route("/calc/dcf", post(post_dcf))
        .route("/calc/reverse-dcf", post(post_reverse_dcf))
        .route("/calc/ddm", post(post_ddm))
        .route("/calc/epv", post(post_epv))
        .route("/calc/wheel", post(post_wheel))
        .route("/symbols/:symbol/rule-of-40", get(get_rule_of_40))
        .route("/symbols/:symbol/gap-stats", get(get_gap_stats))
        .route("/symbols/:symbol/seasonality", get(get_seasonality))
        .route("/rrg", get(get_rrg))
        .route("/symbols/:symbol/beneish", get(get_beneish))
        .route("/symbols/:symbol/chowder", get(get_chowder))
        .route("/market/fed-model", get(get_fed_model))
        .route("/market/nh-nl", get(get_nh_nl))
        .route("/sim/value-averaging", post(post_value_averaging))
        .route("/sim/cppi", post(post_cppi))
}

async fn get_beneish(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(symbol): Path<String>,
) -> Result<Json<traderview_db::beneish::BeneishReport>, ApiError> {
    let sym = validate_symbol(&symbol)?;
    traderview_db::beneish::compute(&sym)
        .await
        .map(Json)
        .map_err(|e| match e {
            traderview_db::beneish::BeneishError::NotEnoughReports { .. } => {
                ApiError::BadRequest(e.to_string())
            }
            traderview_db::beneish::BeneishError::Fetch(inner) => ApiError::Internal(inner),
        })
}

async fn get_chowder(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(symbol): Path<String>,
) -> Result<Json<traderview_db::market_valuation::ChowderReport>, ApiError> {
    let sym = validate_symbol(&symbol)?;
    traderview_db::market_valuation::chowder(&sym)
        .await
        .map(Json)
        .map_err(|e| match e {
            traderview_db::market_valuation::ChowderError::Fetch(inner) => {
                ApiError::Internal(inner)
            }
            other => ApiError::BadRequest(other.to_string()),
        })
}

async fn get_fed_model(
    State(s): State<AppState>,
    _u: AuthUser,
) -> Result<Json<traderview_db::market_valuation::FedModelReport>, ApiError> {
    traderview_db::market_valuation::fed_model(&s.pool)
        .await
        .map(Json)
        .map_err(ApiError::Internal)
}

async fn get_nh_nl(
    State(s): State<AppState>,
    _u: AuthUser,
) -> Result<Json<traderview_db::market_valuation::NhNlReport>, ApiError> {
    Ok(Json(traderview_db::market_valuation::nh_nl(&s.pool).await))
}

async fn post_value_averaging(
    State(s): State<AppState>,
    _u: AuthUser,
    Json(input): Json<traderview_db::strategy_simulators::VaInput>,
) -> Result<Json<traderview_db::strategy_simulators::VaReport>, ApiError> {
    traderview_db::strategy_simulators::value_averaging(&s.pool, &input)
        .await
        .map(Json)
        .map_err(|e| match e {
            traderview_db::strategy_simulators::SimError::PriceFetch(inner) => {
                ApiError::Internal(inner)
            }
            other => ApiError::BadRequest(other.to_string()),
        })
}

async fn post_cppi(
    State(s): State<AppState>,
    _u: AuthUser,
    Json(input): Json<traderview_db::strategy_simulators::CppiInput>,
) -> Result<Json<traderview_db::strategy_simulators::CppiReport>, ApiError> {
    traderview_db::strategy_simulators::cppi(&s.pool, &input)
        .await
        .map(Json)
        .map_err(|e| match e {
            traderview_db::strategy_simulators::SimError::PriceFetch(inner) => {
                ApiError::Internal(inner)
            }
            other => ApiError::BadRequest(other.to_string()),
        })
}

fn validate_symbol(s: &str) -> Result<String, ApiError> {
    let sym = s.trim().to_uppercase();
    if sym.is_empty() || sym.len() > 20 || sym.contains('/') || sym.contains('\\') {
        return Err(ApiError::BadRequest("invalid symbol".into()));
    }
    Ok(sym)
}

async fn get_fundamental_health(
    State(s): State<AppState>,
    _u: AuthUser,
    Path(symbol): Path<String>,
) -> Result<Json<FundamentalHealth>, ApiError> {
    let sym = validate_symbol(&symbol)?;
    let health = fundamental_health::compute(&sym)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!(e)))?;
    // Attach the live price for the Graham-upside calc. Quote failures
    // degrade gracefully — the health card renders without upside.
    let health = match traderview_db::market_data::quote(&s.pool, &sym).await {
        Ok(q) => fundamental_health::with_price(health, q.price),
        Err(_) => health,
    };
    Ok(Json(health))
}

async fn post_dcf(
    State(_s): State<AppState>,
    _u: AuthUser,
    Json(input): Json<DcfInput>,
) -> Result<Json<DcfReport>, ApiError> {
    dcf_valuation::compute(&input)
        .map(Json)
        .map_err(|e| ApiError::BadRequest(e.to_string()))
}

async fn post_reverse_dcf(
    State(_s): State<AppState>,
    _u: AuthUser,
    Json(input): Json<ReverseDcfInput>,
) -> Result<Json<ReverseDcfReport>, ApiError> {
    valuation_models::reverse_dcf(&input)
        .map(Json)
        .ok_or_else(|| {
            ApiError::BadRequest(
                "price not reachable within growth bracket [-50%, +100%] — check inputs".into(),
            )
        })
}

async fn post_ddm(
    State(_s): State<AppState>,
    _u: AuthUser,
    Json(input): Json<DdmInput>,
) -> Result<Json<DdmReport>, ApiError> {
    valuation_models::ddm(&input).map(Json).ok_or_else(|| {
        ApiError::BadRequest(
            "invalid DDM inputs — required return must exceed terminal growth, dividend > 0"
                .into(),
        )
    })
}

async fn post_epv(
    State(_s): State<AppState>,
    _u: AuthUser,
    Json(input): Json<EpvInput>,
) -> Result<Json<EpvReport>, ApiError> {
    valuation_models::epv(&input).map(Json).ok_or_else(|| {
        ApiError::BadRequest("invalid EPV inputs — WACC and shares must be positive".into())
    })
}

async fn post_wheel(
    State(_s): State<AppState>,
    _u: AuthUser,
    Json(input): Json<WheelInput>,
) -> Result<Json<WheelReport>, ApiError> {
    valuation_models::wheel(&input).map(Json).ok_or_else(|| {
        ApiError::BadRequest("invalid wheel inputs — strikes, price, DTEs must be positive".into())
    })
}

/// Rule of 40 from Finnhub metric=all: revenue growth (TTM YoY) +
/// FCF margin (FCF/share ÷ revenue/share). Both best-effort.
async fn get_rule_of_40(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(symbol): Path<String>,
) -> Result<Json<RuleOf40Report>, ApiError> {
    let sym = validate_symbol(&symbol)?;
    let m = traderview_db::finnhub_rest::metric_all(&sym)
        .await
        .map_err(ApiError::Internal)?;
    let g = |k: &str| m.get("metric").and_then(|x| x.get(k)).and_then(|v| v.as_f64());
    let rev_growth = g("revenueGrowthTTMYoy")
        .ok_or_else(|| ApiError::BadRequest(format!("no revenue growth data for {sym}")))?;
    let fcf_ps = g("cashFlowPerShareTTM").or_else(|| g("cashFlowPerShareAnnual"));
    let rev_ps = g("revenuePerShareTTM").or_else(|| g("revenuePerShareAnnual"));
    let fcf_margin = match (fcf_ps, rev_ps) {
        (Some(f), Some(r)) if r.abs() > 1e-9 => f / r * 100.0,
        _ => return Err(ApiError::BadRequest(format!("no FCF margin data for {sym}"))),
    };
    Ok(Json(valuation_models::rule_of_40(rev_growth, fcf_margin)))
}

async fn get_rrg(
    State(s): State<AppState>,
    _u: AuthUser,
) -> Result<Json<rrg::RrgReport>, ApiError> {
    Ok(Json(rrg::compute(&s.pool).await))
}

#[derive(Debug, Deserialize)]
struct GapQ {
    /// Minimum gap size in percent to count (default 0.5).
    threshold: Option<f64>,
}

async fn get_gap_stats(
    State(s): State<AppState>,
    _u: AuthUser,
    Path(symbol): Path<String>,
    Query(q): Query<GapQ>,
) -> Result<Json<GapReport>, ApiError> {
    let sym = validate_symbol(&symbol)?;
    match gap_stats::compute(&s.pool, &sym, q.threshold).await {
        Ok(r) => Ok(Json(r)),
        Err(gap_stats::GapError::Insufficient { symbol, got, need }) => Err(
            ApiError::BadRequest(format!(
                "not enough daily bars for {symbol}: have {got}, need {need}"
            )),
        ),
        Err(gap_stats::GapError::PriceFetch(e)) => Err(ApiError::Internal(e)),
    }
}

async fn get_seasonality(
    State(s): State<AppState>,
    _u: AuthUser,
    Path(symbol): Path<String>,
) -> Result<Json<traderview_core::monthly_seasonality::MonthlySeasonalityReport>, ApiError> {
    let sym = validate_symbol(&symbol)?;
    // 10 years of daily closes gives ≥9 observations per calendar month.
    let to = chrono::Utc::now();
    let from = to - chrono::Duration::days(365 * 10);
    let bars = traderview_db::prices::get_bars(
        &s.pool,
        &sym,
        traderview_core::BarInterval::D1,
        from,
        to,
    )
    .await
    .map_err(ApiError::Internal)?;
    use chrono::Datelike;
    let closes: Vec<traderview_core::monthly_seasonality::DailyClose> = bars
        .iter()
        .map(|b| traderview_core::monthly_seasonality::DailyClose {
            year: b.bar_time.year() as u16,
            month: b.bar_time.month() as u8,
            close: b.close.to_string().parse().unwrap_or(0.0),
        })
        .collect();
    traderview_core::monthly_seasonality::compute(&closes)
        .map(Json)
        .ok_or_else(|| {
            ApiError::BadRequest(format!(
                "not enough history for {sym} seasonality (need ≥2 full years)"
            ))
        })
}
