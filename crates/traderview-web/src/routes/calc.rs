//! Stateless risk / sizing / tax / fixed-income calculators.
//!
//! Every endpoint here is pure compute: it takes a JSON body, runs a single
//! function from `traderview-core`, and returns the result. No database
//! access, no auth-scoped data. Useful as building blocks for both the UI
//! sidebars and third-party integrations.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::routing::post;
use axum::{Json, Router};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use traderview_core::{
    bond_duration, buying_power, carry_score, commission_optimizer, cost_basis, currency_exposure,
    dynamic_kelly, kelly, margin_call, margin_runway, monte_carlo, optimal_f, risk_on_off,
    risk_parity, tax_loss_harvest, var_estimator, vix_term_structure, wash_sale, yield_curve,
};

pub fn router() -> Router<AppState> {
    Router::new()
        // ── Position sizing ───────────────────────────────────────────
        .route("/calc/kelly",                 post(kelly_route))
        .route("/calc/dynamic-kelly",         post(dynamic_kelly_route))
        .route("/calc/optimal-f",             post(optimal_f_route))
        // ── Risk / VaR ────────────────────────────────────────────────
        .route("/calc/var-historical",        post(var_historical_route))
        .route("/calc/var-gaussian",          post(var_gaussian_route))
        .route("/calc/monte-carlo",           post(monte_carlo_route))
        .route("/calc/risk-parity",           post(risk_parity_route))
        .route("/calc/risk-on-off",           post(risk_on_off_route))
        // ── Margin / buying power ─────────────────────────────────────
        .route("/calc/margin-call",           post(margin_call_route))
        .route("/calc/margin-runway",         post(margin_runway_route))
        .route("/calc/buying-power",          post(buying_power_route))
        // ── Tax / fees ────────────────────────────────────────────────
        .route("/calc/tax-loss-harvest",      post(tax_loss_harvest_route))
        .route("/calc/wash-sale",             post(wash_sale_route))
        .route("/calc/cost-basis",            post(cost_basis_route))
        .route("/calc/commission-optimizer",  post(commission_optimizer_route))
        // ── Fixed income / FX ─────────────────────────────────────────
        .route("/calc/yield-curve",           post(yield_curve_route))
        .route("/calc/bond-duration",         post(bond_duration_route))
        .route("/calc/carry-score",           post(carry_score_route))
        .route("/calc/currency-exposure",     post(currency_exposure_route))
        .route("/calc/vix-term-structure",    post(vix_term_structure_route))
}

// ─── Position sizing ────────────────────────────────────────────────────

async fn kelly_route(
    _u: AuthUser, Json(input): Json<kelly::KellyInput>,
) -> Json<kelly::KellyOutput> {
    Json(kelly::compute(&input))
}

#[derive(Deserialize)]
struct DynamicKellyBody {
    trade_pnls: Vec<f64>,
    window: usize,
}

async fn dynamic_kelly_route(
    _u: AuthUser, Json(b): Json<DynamicKellyBody>,
) -> Result<Json<Vec<dynamic_kelly::DynamicKellyPoint>>, ApiError> {
    if b.window == 0 {
        return Err(ApiError::BadRequest("window must be > 0".into()));
    }
    Ok(Json(dynamic_kelly::compute(&b.trade_pnls, b.window)))
}

#[derive(Deserialize)]
struct OptimalFBody {
    returns: Vec<f64>,
}

async fn optimal_f_route(
    _u: AuthUser, Json(b): Json<OptimalFBody>,
) -> Json<optimal_f::OptimalFReport> {
    Json(optimal_f::compute(&b.returns))
}

// ─── VaR ────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct VarBody {
    daily_returns: Vec<f64>,
    position_value: f64,
    /// Confidence as a fraction in (0, 1). e.g. 0.95 for 95% VaR.
    confidence: f64,
}

async fn var_historical_route(
    _u: AuthUser, Json(b): Json<VarBody>,
) -> Result<Json<var_estimator::VarReport>, ApiError> {
    validate_confidence(b.confidence)?;
    Ok(Json(var_estimator::historical(&b.daily_returns, b.position_value, b.confidence)))
}

async fn var_gaussian_route(
    _u: AuthUser, Json(b): Json<VarBody>,
) -> Result<Json<var_estimator::VarReport>, ApiError> {
    validate_confidence(b.confidence)?;
    Ok(Json(var_estimator::parametric_gaussian(&b.daily_returns, b.position_value, b.confidence)))
}

fn validate_confidence(c: f64) -> Result<(), ApiError> {
    if !(c > 0.0 && c < 1.0) {
        return Err(ApiError::BadRequest("confidence must be in (0, 1) exclusive".into()));
    }
    Ok(())
}

// ─── Monte Carlo ────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct MonteCarloBody {
    historical_r: Vec<f64>,
    config: monte_carlo::McConfig,
}

async fn monte_carlo_route(
    _u: AuthUser, Json(b): Json<MonteCarloBody>,
) -> Result<Json<monte_carlo::McReport>, ApiError> {
    monte_carlo::simulate(&b.historical_r, &b.config)
        .ok_or_else(|| ApiError::BadRequest(
            "monte carlo input invalid — historical_r non-empty, n_curves > 0, trades_per_curve > 0".into()
        ))
        .map(Json)
}

// ─── Risk parity / on-off ───────────────────────────────────────────────

#[derive(Deserialize)]
struct RiskParityBody {
    assets: Vec<risk_parity::AssetVol>,
}

async fn risk_parity_route(
    _u: AuthUser, Json(b): Json<RiskParityBody>,
) -> Json<risk_parity::RiskParityReport> {
    Json(risk_parity::allocate(&b.assets))
}

async fn risk_on_off_route(
    _u: AuthUser, Json(snap): Json<risk_on_off::CrossAssetSnapshot>,
) -> Json<risk_on_off::RiskReport> {
    Json(risk_on_off::evaluate(&snap))
}

// ─── Margin / buying power ──────────────────────────────────────────────

async fn margin_call_route(
    _u: AuthUser, Json(snap): Json<margin_call::AccountSnapshot>,
) -> Json<margin_call::MarginCallReport> {
    Json(margin_call::evaluate(&snap))
}

#[derive(Deserialize)]
struct MarginRunwayBody {
    account_equity: f64,
    position_value: f64,
    maintenance_req_pct: f64,
}

async fn margin_runway_route(
    _u: AuthUser, Json(b): Json<MarginRunwayBody>,
) -> Json<margin_runway::MarginRunwayReport> {
    Json(margin_runway::compute(b.account_equity, b.position_value, b.maintenance_req_pct))
}

async fn buying_power_route(
    _u: AuthUser, Json(input): Json<buying_power::BpInput>,
) -> Json<buying_power::BpReport> {
    Json(buying_power::compute(&input))
}

// ─── Tax / fees ─────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct TaxLossHarvestBody {
    losers: Vec<tax_loss_harvest::OpenLoser>,
    recent_buys: Vec<tax_loss_harvest::RecentBuy>,
    today: NaiveDate,
    /// YTD realized loss in dollars (positive when net-losing) — used to
    /// flag harvests that push past the $3k ordinary-income offset cap.
    realized_loss_ytd: Decimal,
    /// Trader-status mark-to-market (§475(f)) election — when true, the
    /// $3k cap doesn't apply.
    #[serde(default)]
    mtm_elected: bool,
}

async fn tax_loss_harvest_route(
    _u: AuthUser, Json(b): Json<TaxLossHarvestBody>,
) -> Json<tax_loss_harvest::HarvestReport> {
    Json(tax_loss_harvest::suggest(
        &b.losers, &b.recent_buys, b.today, b.realized_loss_ytd, b.mtm_elected,
    ))
}

#[derive(Deserialize)]
struct WashSaleBody {
    closings: Vec<wash_sale::ClosingTrade>,
    openings: Vec<wash_sale::OpeningExecution>,
}

#[derive(Serialize)]
struct WashSaleResp {
    hits: Vec<wash_sale::WashHit>,
    total_disallowed: Decimal,
}

async fn wash_sale_route(
    _u: AuthUser, Json(b): Json<WashSaleBody>,
) -> Json<WashSaleResp> {
    let hits = wash_sale::detect_hits(&b.closings, &b.openings);
    let total_disallowed = wash_sale::total_disallowed(&hits);
    Json(WashSaleResp { hits, total_disallowed })
}

#[derive(Deserialize)]
struct CostBasisBody {
    lots: Vec<cost_basis::CostLot>,
    qty_to_close: Decimal,
    price_per_share: Decimal,
    method: cost_basis::LotMethod,
}

async fn cost_basis_route(
    _u: AuthUser, Json(b): Json<CostBasisBody>,
) -> Json<cost_basis::CloseReport> {
    Json(cost_basis::close(&b.lots, b.qty_to_close, b.price_per_share, b.method))
}

#[derive(Deserialize)]
struct CommissionOptimizerBody {
    executions: Vec<commission_optimizer::Execution>,
    tiers: Vec<commission_optimizer::Tier>,
}

async fn commission_optimizer_route(
    _u: AuthUser, Json(b): Json<CommissionOptimizerBody>,
) -> Json<commission_optimizer::OptimizerReport> {
    Json(commission_optimizer::evaluate(&b.executions, &b.tiers))
}

// ─── Fixed income / FX ──────────────────────────────────────────────────

async fn yield_curve_route(
    _u: AuthUser, Json(c): Json<yield_curve::YieldCurve>,
) -> Json<yield_curve::CurveReport> {
    Json(yield_curve::classify(&c))
}

#[derive(Deserialize)]
struct BondDurationBody {
    cash_flows: Vec<bond_duration::CashFlow>,
    ytm: f64,
    compounding_per_year: usize,
}

async fn bond_duration_route(
    _u: AuthUser, Json(b): Json<BondDurationBody>,
) -> Json<bond_duration::DurationReport> {
    Json(bond_duration::compute(&b.cash_flows, b.ytm, b.compounding_per_year))
}

#[derive(Deserialize)]
struct CarryScoreBody {
    long_rate: f64,
    funding_rate: f64,
    annualized_vol: f64,
}

async fn carry_score_route(
    _u: AuthUser, Json(b): Json<CarryScoreBody>,
) -> Json<carry_score::CarryReport> {
    Json(carry_score::score(b.long_rate, b.funding_rate, b.annualized_vol))
}

#[derive(Deserialize)]
struct CurrencyExposureBody {
    positions: Vec<currency_exposure::ForeignPosition>,
    home_currency: String,
    /// Map of currency-code → spot rate to convert TO home (e.g. for a USD
    /// home, EUR → 1.08 means 1 EUR = 1.08 USD).
    fx_to_home: BTreeMap<String, f64>,
}

async fn currency_exposure_route(
    _u: AuthUser, Json(b): Json<CurrencyExposureBody>,
) -> Json<currency_exposure::CurrencyReport> {
    Json(currency_exposure::analyze(&b.positions, &b.home_currency, &b.fx_to_home))
}

async fn vix_term_structure_route(
    _u: AuthUser, Json(ts): Json<vix_term_structure::VixTermStructure>,
) -> Json<vix_term_structure::TermStructureReport> {
    Json(vix_term_structure::analyze(&ts))
}
