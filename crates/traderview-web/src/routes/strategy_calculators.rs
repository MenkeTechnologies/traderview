//! Strategy-calculator API surface:
//!   POST /calc/grid-trading              — grid ladder + per-grid profit
//!   POST /calc/fixed-ratio               — Ryan Jones contract thresholds
//!   POST /calc/anti-martingale           — press-winners streak sizing
//!   POST /calc/risk-of-ruin              — analytic gambler's-ruin RoR
//!   POST /calc/taylor-rule               — prescribed policy rate + stance
//!   POST /calc/sahm-rule                 — unemployment recession trigger
//!   POST /calc/misery-index              — inflation + unemployment
//!   POST /calc/valuation-gauges          — Buffett / Tobin Q / ERP / ECY
//!   POST /calc/variance-risk-premium     — IV² − RV² short-vol edge
//!   POST /calc/scale-out                 — partial-exit ladder scenarios
//!   POST /calc/merger-arb                — deal spread + implied probability
//!   POST /calc/buyback-accretion         — EPS accretion + breakeven P/E
//!   POST /calc/cef-discount              — CEF discount + z-score screen
//!   POST /calc/adr-premium               — ADR vs ordinary parity arb
//!   POST /calc/sbc-dilution              — buyback yield net of SBC
//!   POST /calc/sum-of-parts              — SOTP NAV vs market cap
//!   POST /calc/odd-lot-tender            — odd-lot priority tender arb
//!   POST /calc/crack-spread              — refiner 3-2-1 margin
//!   POST /calc/crush-spread              — soybean board crush
//!   POST /calc/spark-spread              — generation margin (spark/dark)
//!   POST /calc/curve-trade               — DV01-neutral spread/butterfly
//!   POST /calc/cheapest-to-deliver       — basket basis + implied repo
//!   POST /calc/rebalance-bands           — Swedroe 5/25 drift screen
//!   POST /calc/iv-cone                   — IV-term expected-move bands
//!   POST /calc/fund-fees                 — 2-and-20 waterfall + drag
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
        .route("/calc/variance-risk-premium", post(post_vrp))
        .route("/calc/scale-out", post(post_scale_out))
        .route("/calc/merger-arb", post(post_merger_arb))
        .route("/calc/buyback-accretion", post(post_buyback_accretion))
        .route("/calc/cef-discount", post(post_cef_discount))
        .route("/calc/adr-premium", post(post_adr_premium))
        .route("/calc/sbc-dilution", post(post_sbc_dilution))
        .route("/calc/sum-of-parts", post(post_sum_of_parts))
        .route("/calc/odd-lot-tender", post(post_odd_lot_tender))
        .route("/calc/crack-spread", post(post_crack_spread))
        .route("/calc/crush-spread", post(post_crush_spread))
        .route("/calc/spark-spread", post(post_spark_spread))
        .route("/calc/curve-trade", post(post_curve_trade))
        .route("/calc/cheapest-to-deliver", post(post_ctd))
        .route("/calc/rebalance-bands", post(post_rebalance_bands))
        .route("/calc/iv-cone", post(post_iv_cone))
        .route("/calc/fund-fees", post(post_fund_fees))
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

#[derive(Debug, Deserialize)]
struct VrpBody {
    implied_vol_pct: f64,
    realized_vol_pct: f64,
}

async fn post_vrp(
    State(_s): State<AppState>,
    _u: AuthUser,
    Json(b): Json<VrpBody>,
) -> Result<Json<traderview_core::variance_risk_premium::VrpReport>, ApiError> {
    traderview_core::variance_risk_premium::compute(b.implied_vol_pct, b.realized_vol_pct)
        .map(Json)
        .ok_or_else(|| ApiError::BadRequest("vols must be positive".into()))
}

async fn post_scale_out(
    State(_s): State<AppState>,
    _u: AuthUser,
    Json(b): Json<traderview_core::scale_out_planner::ScaleOutInput>,
) -> Result<Json<traderview_core::scale_out_planner::ScaleOutReport>, ApiError> {
    traderview_core::scale_out_planner::compute(&b)
        .map(Json)
        .ok_or_else(|| {
            ApiError::BadRequest(
                "invalid ladder — targets must step away from entry, shares within position"
                    .into(),
            )
        })
}

async fn post_merger_arb(
    State(_s): State<AppState>,
    _u: AuthUser,
    Json(b): Json<traderview_core::merger_arb::MergerArbInput>,
) -> Result<Json<traderview_core::merger_arb::MergerArbReport>, ApiError> {
    traderview_core::merger_arb::compute(&b).map(Json).ok_or_else(|| {
        ApiError::BadRequest("invalid deal inputs — need deal > break, days > 0".into())
    })
}

async fn post_buyback_accretion(
    State(_s): State<AppState>,
    _u: AuthUser,
    Json(b): Json<traderview_core::buyback_accretion::BuybackInput>,
) -> Result<Json<traderview_core::buyback_accretion::BuybackReport>, ApiError> {
    traderview_core::buyback_accretion::compute(&b)
        .map(Json)
        .ok_or_else(|| ApiError::BadRequest("invalid buyback inputs".into()))
}

async fn post_cef_discount(
    State(_s): State<AppState>,
    _u: AuthUser,
    Json(b): Json<traderview_core::cef_discount::CefInput>,
) -> Result<Json<traderview_core::cef_discount::CefReport>, ApiError> {
    traderview_core::cef_discount::compute(&b)
        .map(Json)
        .ok_or_else(|| ApiError::BadRequest("invalid CEF inputs".into()))
}

async fn post_adr_premium(
    State(_s): State<AppState>,
    _u: AuthUser,
    Json(b): Json<traderview_core::adr_premium::AdrInput>,
) -> Result<Json<traderview_core::adr_premium::AdrReport>, ApiError> {
    traderview_core::adr_premium::compute(&b)
        .map(Json)
        .ok_or_else(|| ApiError::BadRequest("invalid ADR inputs".into()))
}

async fn post_sbc_dilution(
    State(_s): State<AppState>,
    _u: AuthUser,
    Json(b): Json<traderview_core::sbc_dilution::SbcInput>,
) -> Result<Json<traderview_core::sbc_dilution::SbcReport>, ApiError> {
    traderview_core::sbc_dilution::compute(&b)
        .map(Json)
        .ok_or_else(|| ApiError::BadRequest("invalid SBC inputs".into()))
}

async fn post_sum_of_parts(
    State(_s): State<AppState>,
    _u: AuthUser,
    Json(b): Json<traderview_core::sum_of_parts::SotpInput>,
) -> Result<Json<traderview_core::sum_of_parts::SotpReport>, ApiError> {
    traderview_core::sum_of_parts::compute(&b)
        .map(Json)
        .ok_or_else(|| ApiError::BadRequest("invalid SOTP inputs".into()))
}

async fn post_odd_lot_tender(
    State(_s): State<AppState>,
    _u: AuthUser,
    Json(b): Json<traderview_core::odd_lot_tender::OddLotInput>,
) -> Result<Json<traderview_core::odd_lot_tender::OddLotReport>, ApiError> {
    traderview_core::odd_lot_tender::compute(&b)
        .map(Json)
        .ok_or_else(|| ApiError::BadRequest("invalid tender inputs".into()))
}

#[derive(Debug, Deserialize)]
struct CrackBody {
    crude: f64,
    gasoline: f64,
    distillate: f64,
}

async fn post_crack_spread(
    State(_s): State<AppState>,
    _u: AuthUser,
    Json(b): Json<CrackBody>,
) -> Result<Json<traderview_core::processing_spreads::CrackReport>, ApiError> {
    traderview_core::processing_spreads::crack_321(b.crude, b.gasoline, b.distillate)
        .map(Json)
        .ok_or_else(|| ApiError::BadRequest("prices must be positive".into()))
}

#[derive(Debug, Deserialize)]
struct CrushBody {
    beans: f64,
    meal: f64,
    oil: f64,
}

async fn post_crush_spread(
    State(_s): State<AppState>,
    _u: AuthUser,
    Json(b): Json<CrushBody>,
) -> Result<Json<traderview_core::processing_spreads::CrushReport>, ApiError> {
    traderview_core::processing_spreads::soybean_crush(b.beans, b.meal, b.oil)
        .map(Json)
        .ok_or_else(|| ApiError::BadRequest("prices must be positive".into()))
}

#[derive(Debug, Deserialize)]
struct SparkBody {
    power: f64,
    fuel: f64,
    heat_rate: f64,
}

async fn post_spark_spread(
    State(_s): State<AppState>,
    _u: AuthUser,
    Json(b): Json<SparkBody>,
) -> Result<Json<traderview_core::processing_spreads::SparkReport>, ApiError> {
    traderview_core::processing_spreads::spark_spread(b.power, b.fuel, b.heat_rate)
        .map(Json)
        .ok_or_else(|| ApiError::BadRequest("inputs must be positive".into()))
}

async fn post_curve_trade(
    State(_s): State<AppState>,
    _u: AuthUser,
    Json(b): Json<traderview_core::curve_trade::CurveTradeInput>,
) -> Result<Json<traderview_core::curve_trade::CurveTradeReport>, ApiError> {
    traderview_core::curve_trade::compute(&b)
        .map(Json)
        .ok_or_else(|| ApiError::BadRequest("need 2 or 3 valid legs".into()))
}

async fn post_ctd(
    State(_s): State<AppState>,
    _u: AuthUser,
    Json(b): Json<traderview_core::cheapest_to_deliver::CtdInput>,
) -> Result<Json<traderview_core::cheapest_to_deliver::CtdReport>, ApiError> {
    traderview_core::cheapest_to_deliver::compute(&b)
        .map(Json)
        .ok_or_else(|| ApiError::BadRequest("invalid deliverable basket".into()))
}

async fn post_rebalance_bands(
    State(_s): State<AppState>,
    _u: AuthUser,
    Json(b): Json<traderview_core::rebalance_bands::BandsInput>,
) -> Result<Json<traderview_core::rebalance_bands::BandsReport>, ApiError> {
    traderview_core::rebalance_bands::compute(&b)
        .map(Json)
        .ok_or_else(|| ApiError::BadRequest("invalid band inputs".into()))
}

#[derive(Debug, Deserialize)]
struct IvConeBody {
    spot: f64,
    term: Vec<traderview_core::iv_cone::TermPoint>,
}

async fn post_iv_cone(
    State(_s): State<AppState>,
    _u: AuthUser,
    Json(b): Json<IvConeBody>,
) -> Result<Json<Vec<traderview_core::iv_cone::ConeRow>>, ApiError> {
    traderview_core::iv_cone::compute(b.spot, &b.term)
        .map(Json)
        .ok_or_else(|| ApiError::BadRequest("invalid term structure".into()))
}

async fn post_fund_fees(
    State(_s): State<AppState>,
    _u: AuthUser,
    Json(b): Json<traderview_core::fund_fees::FundFeesInput>,
) -> Result<Json<traderview_core::fund_fees::FundFeesReport>, ApiError> {
    traderview_core::fund_fees::compute(&b)
        .map(Json)
        .ok_or_else(|| ApiError::BadRequest("invalid fee inputs".into()))
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
