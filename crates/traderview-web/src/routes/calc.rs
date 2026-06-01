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
        .route("/calc/section-1244",          post(section_1244_route))
        .route("/calc/section-1245-1250",     post(section_1245_1250_route))
        .route("/calc/section-1202",          post(section_1202_route))
        .route("/calc/section-1045",          post(section_1045_route))
        .route("/calc/section-121",           post(section_121_route))
        .route("/calc/section-121d",          post(section_121d_route))
        .route("/calc/section-132",           post(section_132_route))
        .route("/calc/reps-qualification",    post(reps_qualification_route))
        .route("/calc/section-163j",          post(section_163j_route))
        .route("/calc/section-165d",          post(section_165d_route))
        .route("/calc/section-165g",          post(section_165g_route))
        .route("/calc/section-267",           post(section_267_route))
        .route("/calc/section-988",           post(section_988_route))
        .route("/calc/section-1296",          post(section_1296_route))
        .route("/calc/section-1341",          post(section_1341_route))
        .route("/calc/section-168g",          post(section_168g_route))
        .route("/calc/section-168k",          post(section_168k_route))
        .route("/calc/section-163j-tradeoff", post(section_163j_tradeoff_route))
        .route("/calc/section-164",           post(section_164_route))
        .route("/calc/section-165h",          post(section_165h_route))
        .route("/calc/section-25c",           post(section_25c_route))
        .route("/calc/section-25d",           post(section_25d_route))
        .route("/calc/section-30d",           post(section_30d_route))
        .route("/calc/mlp-ubti",              post(mlp_ubti_route))
        .route("/calc/section-1259",          post(section_1259_route))
        .route("/calc/section-1361",          post(section_1361_route))
        .route("/calc/section-1367",          post(section_1367_route))
        .route("/calc/section-1368",          post(section_1368_route))
        .route("/calc/section-1374",          post(section_1374_route))
        .route("/calc/section-1375",          post(section_1375_route))
        .route("/calc/section-475c2",         post(section_475c2_route))
        .route("/calc/section-213",           post(section_213_route))
        .route("/calc/section-170",           post(section_170_route))
        .route("/calc/section-219",           post(section_219_route))
        .route("/calc/section-221",           post(section_221_route))
        .route("/calc/section-223",           post(section_223_route))
        .route("/calc/section-243",           post(section_243_route))
        .route("/calc/section-250",           post(section_250_route))
        .route("/calc/section-59a",           post(section_59a_route))
        .route("/calc/section-6045",          post(section_6045_route))
        .route("/calc/section-6050i",         post(section_6050i_route))
        .route("/calc/section-6050w",         post(section_6050w_route))
        .route("/calc/section-6651",          post(section_6651_route))
        .route("/calc/section-6654",          post(section_6654_route))
        .route("/calc/section-6662",          post(section_6662_route))
        .route("/calc/section-448",           post(section_448_route))
        .route("/calc/section-444",           post(section_444_route))
        .route("/calc/section-3406",          post(section_3406_route))
        .route("/calc/section-305",           post(section_305_route))
        .route("/calc/section-331",           post(section_331_route))
        .route("/calc/section-332",           post(section_332_route))
        .route("/calc/section-1234a",         post(section_1234a_route))
        .route("/calc/section-1234b",         post(section_1234b_route))
        .route("/calc/section-263g",          post(section_263g_route))
        .route("/calc/section-1276",          post(section_1276_route))
        .route("/calc/section-1277",          post(section_1277_route))
        .route("/calc/section-1278",          post(section_1278_route))
        .route("/calc/section-1271",          post(section_1271_route))
        .route("/calc/section-1272",          post(section_1272_route))
        .route("/calc/section-1273",          post(section_1273_route))
        .route("/calc/section-1281",          post(section_1281_route))
        .route("/calc/section-1283",          post(section_1283_route))
        .route("/calc/section-1282",          post(section_1282_route))
        .route("/calc/section-7704",          post(section_7704_route))
        .route("/calc/section-6045b",         post(section_6045b_route))
        .route("/calc/section-6045a",         post(section_6045a_route))
        .route("/calc/section-1297",          post(section_1297_route))
        .route("/calc/section-1298",          post(section_1298_route))
        .route("/calc/section-6038d",         post(section_6038d_route))
        .route("/calc/section-6011",          post(section_6011_route))
        .route("/calc/section-6111",          post(section_6111_route))
        .route("/calc/section-6112",          post(section_6112_route))
        .route("/calc/section-6662a",         post(section_6662a_route))
        .route("/calc/section-6694",          post(section_6694_route))
        .route("/calc/section-6695",          post(section_6695_route))
        .route("/calc/section-6700",          post(section_6700_route))
        .route("/calc/section-6701",          post(section_6701_route))
        .route("/calc/section-336",           post(section_336_route))
        .route("/calc/section-351",           post(section_351_route))
        .route("/calc/section-451b",          post(section_451b_route))
        .route("/calc/section-1031-f",        post(section_1031_f_route))
        .route("/calc/section-1033",          post(section_1033_route))
        .route("/calc/section-481",           post(section_481_route))
        .route("/calc/section-530",           post(section_530_route))
        .route("/calc/section-280f",          post(section_280f_route))
        .route("/calc/section-280b",          post(section_280b_route))
        .route("/calc/section-280e",          post(section_280e_route))
        .route("/calc/section-163d",          post(section_163d_route))
        .route("/calc/section-163h",          post(section_163h_route))
        .route("/calc/section-864b2",         post(section_864b2_route))
        .route("/calc/section-72t",           post(section_72t_route))
        .route("/calc/section-7345",          post(section_7345_route))
        .route("/calc/section-7408",          post(section_7408_route))
        .route("/calc/section-7701",          post(section_7701_route))
        .route("/calc/section-7872",          post(section_7872_route))
        .route("/calc/section-1295",          post(section_1295_route))
        .route("/calc/section-1092",          post(section_1092_route))
        .route("/calc/section-453",           post(section_453_route))
        .route("/calc/section-453a",          post(section_453a_route))
        .route("/calc/section-461l",          post(section_461l_route))
        .route("/calc/section-465",           post(section_465_route))
        .route("/calc/section-691",           post(section_691_route))
        .route("/calc/section-704d",          post(section_704d_route))
        .route("/calc/section-704c",          post(section_704c_route))
        .route("/calc/section-721",           post(section_721_route))
        .route("/calc/section-731",           post(section_731_route))
        .route("/calc/section-752",           post(section_752_route))
        .route("/calc/section-1235",          post(section_1235_route))
        .route("/calc/section-754",           post(section_754_route))
        .route("/calc/section-871m",          post(section_871m_route))
        .route("/calc/section-911",           post(section_911_route))
        .route("/calc/section-401a9",         post(section_401a9_route))
        .route("/calc/section-409a",          post(section_409a_route))
        .route("/calc/section-382",           post(section_382_route))
        .route("/calc/section-83i",           post(section_83i_route))
        .route("/calc/section-408-d3",        post(section_408_d3_route))
        .route("/calc/section-408m",          post(section_408m_route))
        .route("/calc/section-41",            post(section_41_route))
        .route("/calc/section-408a-d3",       post(section_408A_d3_route))
        .route("/calc/section-174",           post(section_174_route))
        .route("/calc/section-179",           post(section_179_route))
        .route("/calc/section-183",           post(section_183_route))
        .route("/calc/section-263a",          post(section_263a_route))
        .route("/calc/section-168-e6",        post(section_168_e6_route))
        .route("/calc/section-108",           post(section_108_route))
        .route("/calc/section-104",           post(section_104_route))
        .route("/calc/section-1014",          post(section_1014_route))
        .route("/calc/section-1014e",         post(section_1014e_route))
        .route("/calc/section-1015",          post(section_1015_route))
        .route("/calc/section-1041",          post(section_1041_route))
        .route("/calc/section-170e",          post(section_170e_route))
        .route("/calc/section-172",           post(section_172_route))
        .route("/calc/section-195",           post(section_195_route))
        .route("/calc/section-83b",           post(section_83b_route))
        .route("/calc/section-83c",           post(section_83c_route))
        .route("/calc/section-1091",          post(section_1091_route))
        .route("/calc/section-1231",          post(section_1231_route))
        .route("/calc/section-1233",          post(section_1233_route))
        .route("/calc/section-1234",          post(section_1234_route))
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

// ── §1244 small business stock loss ────────────────────────────────────
// Mounted at /api/calc/section-1244. Pure compute; takes the full
// Section1244Input struct (loss + filing status + prior-claimed + the
// 5-test qualification checklist) and returns the ordinary/capital split.

async fn section_1244_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1244::Section1244Input>,
) -> Result<Json<traderview_expense::section_1244::Section1244Result>, ApiError> {
    if b.realized_loss < Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "realized_loss must be >= 0 (pass loss as positive number)".into(),
        ));
    }
    if b.ordinary_loss_claimed_this_year_so_far < Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "ordinary_loss_claimed_this_year_so_far must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_1244::compute(&b)))
}

// ── §1245 / §1250 depreciation recapture ───────────────────────────
// Mounted at /api/calc/section-1245-1250. §1245(a)(1) personal-
// property recapture = min(gain, accumulated depreciation) ordinary;
// §1250 real-property: post-1986 MACRS straight-line → zero ordinary
// + §1(h)(7) unrecaptured §1250 gain taxed at 25% maximum rate for
// individuals (vs. 20% LTCG); pre-1986 / accelerated → recapture of
// additional depreciation as ordinary; residual gain flows to §1231.

async fn section_1245_1250_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1245_1250::Section1245_1250Input>,
) -> Result<Json<traderview_expense::section_1245_1250::Section1245_1250Result>, ApiError> {
    if b.accumulated_depreciation_dollars < 0
        || b.additional_depreciation_dollars < 0
    {
        return Err(ApiError::BadRequest(
            "non-negative depreciation inputs required".into(),
        ));
    }
    Ok(Json(traderview_expense::section_1245_1250::compute(&b)))
}

// ── §1202 QSBS exclusion ──────────────────────────────────────────────
// Mounted at /api/calc/section-1202. Pure compute; up to $10M / 10× basis
// of gain on qualified small-business stock excluded at 50/75/100% depending
// on acquisition date.

async fn section_1202_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1202::Section1202Input>,
) -> Result<Json<traderview_expense::section_1202::Section1202Result>, ApiError> {
    if b.taxpayer_basis < Decimal::ZERO {
        return Err(ApiError::BadRequest("taxpayer_basis must be >= 0".into()));
    }
    if b.prior_exclusion_used_this_issuer < Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "prior_exclusion_used_this_issuer must be >= 0".into(),
        ));
    }
    if b.disposition_date < b.acquisition_date {
        return Err(ApiError::BadRequest(
            "disposition_date must be >= acquisition_date".into(),
        ));
    }
    Ok(Json(traderview_expense::section_1202::compute(&b)))
}

// ── §1045 QSBS rollover ───────────────────────────────────────────────
// Mounted at /api/calc/section-1045. Pure compute; rolls QSBS gain
// into replacement QSBS within 60 days, deferring up to the full
// gain. Holding period tacks for the §1202 5-year clock.

async fn section_1045_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1045::Section1045Input>,
) -> Result<Json<traderview_expense::section_1045::Section1045Result>, ApiError> {
    if b.sale_proceeds_net < Decimal::ZERO || b.replacement_cost < Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "sale_proceeds_net and replacement_cost must be >= 0".into(),
        ));
    }
    if b.original_sale_date < b.original_acquisition_date {
        return Err(ApiError::BadRequest(
            "original_sale_date must be >= original_acquisition_date".into(),
        ));
    }
    Ok(Json(traderview_expense::section_1045::compute(&b)))
}

// ── §163(j) business interest limitation ─────────────────────────────
// Mounted at /api/calc/section-163j. Pure compute; caps margin interest
// deduction at 30% × ATI + business interest income + floor plan
// financing for §475(f) traders. Indefinite carryforward; small-business
// exception ($30M gross receipts for 2024) bypasses the cap entirely.

async fn section_163j_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_163j::Section163jInput>,
) -> Result<Json<traderview_expense::section_163j::Section163jResult>, ApiError> {
    if b.business_interest_expense < Decimal::ZERO
        || b.business_interest_income < Decimal::ZERO
        || b.floor_plan_financing_interest < Decimal::ZERO
        || b.prior_year_carryforward < Decimal::ZERO
        || b.avg_3yr_gross_receipts < Decimal::ZERO
    {
        return Err(ApiError::BadRequest(
            "all dollar inputs must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_163j::compute(&b)))
}

// ── §165(d) wagering loss deduction ─────────────────────────────────
// Mounted at /api/calc/section-165d. Pre-OBBBA: 100% of losses up
// to winnings; post-OBBBA (P.L. 119-21 signed 2025-07-04 eff. 2026):
// 90% of losses + still capped at winnings; phantom-income emerges
// when 90% × losses < winnings ≤ losses; §162 trade-or-business
// expense carve-out preserved for professional gamblers; itemized
// Schedule A (Schedule C for professional).

async fn section_165d_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_165d::Section165dInput>,
) -> Result<Json<traderview_expense::section_165d::Section165dResult>, ApiError> {
    Ok(Json(traderview_expense::section_165d::compute(&b)))
}

// ── §165(g) worthless securities deduction ──────────────────────────
// Mounted at /api/calc/section-165g. Wholly worthless capital-asset
// security deemed sold last day of taxable year (§165(g)(1));
// §165(g)(2) security definition (stock + bond + debenture + registered
// indebtedness); §165(g)(3) affiliated-domestic-corporation ordinary
// loss exception (§1504(a)(2) 80%/80% + > 90% non-passive gross
// receipts); §1244 small business stock ordinary loss priority.

async fn section_165g_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_165g::Section165gInput>,
) -> Result<Json<traderview_expense::section_165g::Section165gResult>, ApiError> {
    if b.non_passive_gross_receipts_pct_bp > 10_000 {
        return Err(ApiError::BadRequest(
            "non_passive_gross_receipts_pct_bp must be ≤ 100% (10,000bp)".into(),
        ));
    }
    Ok(Json(traderview_expense::section_165g::compute(&b)))
}

// ── §453 installment sale gain deferral ──────────────────────────────
// Mounted at /api/calc/section-453. Pure compute; gross profit ratio
// applied to each year's principal payment with §453(k) marketable
// securities exclusion + §453(g) related-party 2-year resale anti-
// abuse + §453(d) opt-out election.

async fn section_453_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_453::Section453Input>,
) -> Result<Json<traderview_expense::section_453::Section453Result>, ApiError> {
    if b.sale_price < Decimal::ZERO
        || b.selling_costs < Decimal::ZERO
        || b.adjusted_basis < Decimal::ZERO
        || b.principal_received_this_year < Decimal::ZERO
        || b.interest_received_this_year < Decimal::ZERO
        || b.unrecognized_gain_remaining < Decimal::ZERO
    {
        return Err(ApiError::BadRequest(
            "all dollar inputs must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_453::compute(&b)))
}

// ── §453A nondealer installment interest charge ─────────────────────
// Mounted at /api/calc/section-453a. Pairs with §453: imposes a
// non-deductible interest charge on the deferred tax liability of
// large installment obligations exceeding $5M aggregate face at
// year-end. Per-sale floor $150k + non-dealer + non-personal-use +
// non-residential-lots/timeshares. Interest = applicable % ×
// deferred tax × §6621 underpayment rate (short-term AFR + 3 pp).

async fn section_453a_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_453a::Section453aInput>,
) -> Result<Json<traderview_expense::section_453a::Section453aResult>, ApiError> {
    if b.sales_price_dollars < 0
        || b.aggregate_year_end_face_obligations_dollars < 0
        || b.maximum_applicable_tax_rate_bp > 10_000
        || b.underpayment_rate_bp > 10_000
    {
        return Err(ApiError::BadRequest(
            "non-negative dollar inputs and rates ≤ 100% (10,000bp) required".into(),
        ));
    }
    Ok(Json(traderview_expense::section_453a::compute(&b)))
}

// ── §168(e)(6) Qualified Improvement Property ────────────────────────
// Mounted at /api/calc/section-168-e6. Pure compute; interior
// improvements to nonresidential buildings qualify as 15-year QIP +
// §168(k) bonus eligible. Excluded types (enlargement, elevator,
// internal structural framework) fall to 39-year nonresidential.

async fn section_168_e6_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_168_e6::Section168E6Input>,
) -> Result<Json<traderview_expense::section_168_e6::Section168E6Result>, ApiError> {
    if b.improvement_cost < Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "improvement_cost must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_168_e6::compute(&b)))
}

// ── §263A UNICAP (trader vs dealer classifier) ───────────────────────
// Mounted at /api/calc/section-263a. Pure compute; dealers must
// capitalize direct + indirect inventory costs; traders + investors
// are exempt. §263A(b)(2)(B) small business exception per §448(c)
// threshold.

async fn section_263a_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_263a::Section263AInput>,
) -> Result<Json<traderview_expense::section_263a::Section263AResult>, ApiError> {
    if b.direct_costs < Decimal::ZERO
        || b.indirect_costs_allocable_to_inventory < Decimal::ZERO
        || b.avg_3yr_gross_receipts < Decimal::ZERO
    {
        return Err(ApiError::BadRequest(
            "all dollar inputs must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_263a::compute(&b)))
}

// ── §174 R&D capitalization (post-TCJA) ──────────────────────────────
// Mounted at /api/calc/section-174. Pure compute; capitalizes R&D
// expenditures and amortizes over 5y domestic / 15y foreign with
// half-year convention. Hit algorithmic traders writing internal
// trading software starting in 2022.

async fn section_174_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_174::Section174Input>,
) -> Result<Json<traderview_expense::section_174::Section174Result>, ApiError> {
    if b.r_and_d_amount < Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "r_and_d_amount must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_174::compute(&b)))
}

// ── §179 election to expense certain depreciable business assets ─────
// Mounted at /api/calc/section-179. §179(b)(1) dollar cap (2026 =
// $2,560,000); §179(b)(2) phaseout dollar-for-dollar above threshold
// (2026 = $4,090,000); §179(b)(3)(A) taxable-income limitation with
// §179(b)(3)(B) indefinite carryforward; §179(b)(5) heavy-SUV sublimit
// (GVWR 6,001-14,000 lb. — 2026 = $32,000) with excess flowing to §168(k)
// 100% bonus depreciation made permanent by OBBBA §70302 (eff. 2025-01-01).
// Out of scope: §179(d)(3) related-party purchase restriction; §179(d)(10)
// recapture on business-use percentage drop below 50%; §179(f) qualified
// real-property carve-in (roofs, HVAC, fire alarm, security systems).

async fn section_179_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_179::Section179Input>,
) -> Result<Json<traderview_expense::section_179::Section179Result>, ApiError> {
    if b.qualifying_property_cents < 0
        || b.suv_property_cents < 0
        || b.dollar_cap_cents < 0
        || b.phaseout_threshold_cents < 0
        || b.suv_sublimit_cents < 0
    {
        return Err(ApiError::BadRequest(
            "non-negative cents values required".into(),
        ));
    }
    Ok(Json(traderview_expense::section_179::compute(&b)))
}

// ── §183 hobby loss rules ────────────────────────────────────────────
// Mounted at /api/calc/section-183. §183(a) general rule; §183(b)(1)
// always-allowed deductions (taxes, interest); §183(b)(2) capped at
// gross income − (b)(1) — effectively ZERO post-TCJA via §67(g)
// misc-itemized-deduction suspension made permanent by OBBBA 2025;
// §183(d) profit-motive presumption (3-of-5 standard / 2-of-7 horse);
// §183(e) deferral election; Reg. § 1.183-2(b) 9-factor backup test.

async fn section_183_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_183::Section183Input>,
) -> Result<Json<traderview_expense::section_183::Section183Result>, ApiError> {
    if b.nine_factors_favoring_profit > 9
        || b.gross_income_from_activity_dollars < 0
        || b.section_183b1_deductions_dollars < 0
        || b.other_activity_deductions_dollars < 0
    {
        return Err(ApiError::BadRequest(
            "nine_factors_favoring_profit must be ≤ 9 and dollar inputs must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_183::compute(&b)))
}

// ── §1234 options character + holding-period rules ──────────────────
// Mounted at /api/calc/section-1234. Pure compute; §1234(a) holder
// character mirrors underlying with option holding period driving
// ST/LT; §1234(b) writer is fixed short-term capital regardless of
// holding period; §1234(c) §1256 contract override; exercise +
// assignment are basis-adjustment events with no separate option
// gain/loss.

async fn section_1234_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1234::Section1234Input>,
) -> Result<Json<traderview_expense::section_1234::Section1234Result>, ApiError> {
    if b.option_close_date < b.option_open_date {
        return Err(ApiError::BadRequest(
            "option_close_date must be on or after option_open_date".into(),
        ));
    }
    if b.premium < Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "premium must be >= 0 (sign is implicit from role: writer = received, holder = paid)"
                .into(),
        ));
    }
    if b.close_proceeds_or_cost < Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "close_proceeds_or_cost must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_1234::compute(&b)))
}

// ── §1231 quasi-capital gain/loss with §1231(c) recapture ──────────
// Mounted at /api/calc/section-1231. §1231(a)(1) net gain → LTCG;
// §1231(a)(2) net loss → ordinary; §1231(b) property = real /
// depreciable held > 1 year used in trade/business; §1231(c) 5-year
// lookback recaptures current net gain as ordinary up to prior-5-yr
// nonrecaptured net §1231 losses.

async fn section_1231_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1231::Section1231Input>,
) -> Result<Json<traderview_expense::section_1231::Section1231Result>, ApiError> {
    if b.current_year_gains_dollars < 0
        || b.current_year_losses_dollars < 0
    {
        return Err(ApiError::BadRequest(
            "current_year_gains_dollars and current_year_losses_dollars must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_1231::compute(&b)))
}

// ── §1233 short-sale character + holding-period rules ───────────────
// Mounted at /api/calc/section-1233. Pure compute; §1233(b) gain
// short-term + holding-period reset for short-held or during-short
// substantially identical; §1233(d) loss long-term for long-held
// substantially identical at short open.

async fn section_1233_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1233::Section1233Input>,
) -> Result<Json<traderview_expense::section_1233::Section1233Result>, ApiError> {
    if b.short_shares < 0 {
        return Err(ApiError::BadRequest("short_shares must be >= 0".into()));
    }
    if b.short_close_date < b.short_sale_date {
        return Err(ApiError::BadRequest(
            "short_close_date must be on or after short_sale_date".into(),
        ));
    }
    for p in b
        .substantially_identical_held_at_open
        .iter()
        .chain(b.substantially_identical_acquired_during_short.iter())
    {
        if p.shares < 0 {
            return Err(ApiError::BadRequest(
                "long position shares must be >= 0".into(),
            ));
        }
    }
    Ok(Json(traderview_expense::section_1233::compute(&b)))
}

// ── §83(b) restricted-stock election ─────────────────────────────────
// Mounted at /api/calc/section-83b. Validates 30-day deadline,
// computes ordinary income with vs without election, LTCG holding-
// period start (grant vs vesting), capital gain at sale,
// §83(b)(2) forfeiture trap with no refund.

async fn section_83b_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_83b::Section83bInput>,
) -> Result<Json<traderview_expense::section_83b::Section83bResult>, ApiError> {
    if b.vesting_date < b.grant_date {
        return Err(ApiError::BadRequest(
            "vesting_date must be on or after grant_date".into(),
        ));
    }
    if b.fmv_at_grant < Decimal::ZERO
        || b.amount_paid_at_grant < Decimal::ZERO
        || b.fmv_at_vesting < Decimal::ZERO
    {
        return Err(ApiError::BadRequest(
            "fmv_at_grant, amount_paid_at_grant, and fmv_at_vesting must be >= 0".into(),
        ));
    }
    if let Some(sp) = b.sale_price_per_share {
        if sp < Decimal::ZERO {
            return Err(ApiError::BadRequest(
                "sale_price_per_share must be >= 0".into(),
            ));
        }
    }
    Ok(Json(traderview_expense::section_83b::compute(&b)))
}

// ── §83(c) substantial risk of forfeiture timing rules ───────────
// Mounted at /api/calc/section-83c. §83(a) recognize on EARLIER of
// transferable or no-SRF; §83(c)(1) SRF requires future-performance
// of substantial services OR transfer-purpose condition + substantial
// possibility of forfeiture + likelihood of enforcement; §83(c)(2)
// transferability = transferee not subject to SRF; §83(c)(3) § 16(b)
// 6-month short-swing-profit restriction (treats property as SRF AND
// non-transferable until 6-month expiry or no-§16(b)-suit-on-profit);
// Treas. Reg. § 1.83-3(c) elaboration.

async fn section_83c_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_83c::Section83cInput>,
) -> Result<Json<traderview_expense::section_83c::Section83cResult>, ApiError> {
    if b.days_remaining_in_section_16b_period > 366 {
        return Err(ApiError::BadRequest(
            "days_remaining_in_section_16b_period must be <= 366".into(),
        ));
    }
    Ok(Json(traderview_expense::section_83c::compute(&b)))
}

// ── §172 Net Operating Loss deduction ────────────────────────────────
// Mounted at /api/calc/section-172. Three regimes by NOL year:
// pre-2018 legacy (2yr carryback / 20yr carryforward / no 80% limit),
// CARES Act 2018-2020 (5yr carryback / 100% offset), permanent TCJA
// post-2020 (no carryback / indefinite carryforward / 80% limit).
// §172(b)(1)(B) farming + insurance 2-year carryback exception.

async fn section_172_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_172::Section172Input>,
) -> Result<Json<traderview_expense::section_172::Section172Result>, ApiError> {
    if b.current_year_nol < Decimal::ZERO
        || b.current_year_taxable_income_before_nol < Decimal::ZERO
        || b.prior_year_nol_carryforward < Decimal::ZERO
    {
        return Err(ApiError::BadRequest(
            "all dollar inputs must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_172::compute(&b)))
}

// ── §195 startup expenditures — election to deduct $5k first-year ───
// (phased out dollar-for-dollar above $50k startup costs, fully phased
// at $55k) plus 180-month amortization of the remainder beginning with
// the month active trade or business begins. § 195(c)(1) excludes
// amounts deductible under §§ 163(a), 164, 174. § 195(d) automatic
// election per T.D. 9542 (Sept. 8, 2011) — caller passes
// `affirmative_capitalization_election = true` to opt OUT of the
// default deduction treatment. Trader-relevant for new TTS-elected
// LLCs and prop-trading entities organizing pre-launch operations.
async fn section_195_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_195::Section195Input>,
) -> Result<Json<traderview_expense::section_195::Section195Result>, ApiError> {
    if b.total_startup_expenditures_cents < -10_000_000_000
        || b.total_startup_expenditures_cents > 10_000_000_000_000
    {
        return Err(ApiError::BadRequest(
            "total_startup_expenditures_cents out of plausible range".into(),
        ));
    }
    if b.months_active_in_first_year > 12 {
        return Err(ApiError::BadRequest(
            "months_active_in_first_year must be 0..=12".into(),
        ));
    }
    Ok(Json(traderview_expense::section_195::compute(&b)))
}

// ── §170(e) charitable contribution of appreciated property ─────────
// Mounted at /api/calc/section-170e. Six rule paths cover LTCG-public
// FMV (30% AGI), basis election (50% AGI), QAS to private foundation
// (FMV, 20% AGI), private-foundation reduction (basis, 20% AGI),
// STCG/ordinary reduction (basis, 50% public / 30% private), and
// tangible unrelated use (basis). §170(d) 5-year carryforward.

async fn section_170e_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_170e::Section170eInput>,
) -> Result<Json<traderview_expense::section_170e::Section170eResult>, ApiError> {
    if b.fmv < Decimal::ZERO || b.basis < Decimal::ZERO || b.agi < Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "fmv, basis, and agi must be >= 0".into(),
        ));
    }
    if b.prior_year_carryover < Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "prior_year_carryover must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_170e::compute(&b)))
}

// ── §72(t) 10% additional tax on early retirement distributions ─────
// Mounted at /api/calc/section-72t. §72(t)(1) 10% additional tax on
// includible portion of pre-age-59½ distributions; §72(t)(2) ~14
// exceptions including age 59½, death, disability, SEPP, separation
// after 55, medical > 7.5% AGI, QDRO, higher education, first-time
// homebuyer $10k IRA-only, unemployed health, §72(t)(11) federally
// declared disaster $22k, birth/adoption $5k, SECURE 2.0 §326
// terminal illness, §115 emergency personal expense $1k (plan-
// optional), §314 domestic abuse $10k (plan-optional), §334 long-
// term care $2.5k eff. 2026 (plan-optional).

async fn section_72t_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_72t::Section72tInput>,
) -> Result<Json<traderview_expense::section_72t::Section72tResult>, ApiError> {
    if b.distribution_amount_dollars < 0 || b.includible_in_gross_income_dollars < 0 {
        return Err(ApiError::BadRequest(
            "non-negative dollar inputs required".into(),
        ));
    }
    Ok(Json(traderview_expense::section_72t::compute(&b)))
}

// ── § 7345 passport revocation for seriously delinquent tax debt ────
// Mounted at /api/calc/section-7345. FAST Act § 32101 (Pub. L.
// 114-94, December 4, 2015) authorizes IRS to certify "seriously
// delinquent tax debt" to State Department, which then denies,
// revokes, or limits passports. § 7345(b)(1) threshold: debt
// exceeding inflation-adjusted amount ($66,000 for 2025; $50K
// originally in 2015) including penalties + interest, with EITHER
// (A) lien filed + § 6320 administrative remedies exhausted OR
// (B) § 6331 levy issued. § 7345(b)(2) exclusions: installment
// agreement (§ 6159), offer in compromise (§ 7122), innocent
// spouse claim (§ 6015), CDP hearing pending (§ 6320/§ 6330),
// bankruptcy, identity theft, disaster area, currently-not-
// collectible status. § 7345(c) 30-day reversal notification.
// § 7345(e) judicial review in Tax Court OR District Court.
// Sibling cluster: § 6011 + § 6651 + § 6654 + § 6662 + § 6707A.

async fn section_7345_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7345::Section7345Input>,
) -> Result<Json<traderview_expense::section_7345::Section7345Result>, ApiError> {
    if b.assessed_tax_debt_cents < 0 || b.assessed_tax_debt_cents > 100_000_000_000 {
        return Err(ApiError::BadRequest(
            "assessed_tax_debt_cents out of range".into(),
        ));
    }
    if b.annual_threshold_cents < 0 || b.annual_threshold_cents > 100_000_000_000 {
        return Err(ApiError::BadRequest(
            "annual_threshold_cents out of range".into(),
        ));
    }
    Ok(Json(traderview_expense::section_7345::compute(&b)))
}

// ── § 7408 injunction remedy for preparer/promoter conduct ──────────
// Mounted at /api/calc/section-7408. Completes the preparer +
// promoter enforcement cluster: § 6694 + § 6695 + § 6700 + § 6701
// + § 7408. § 7408 is the EQUITABLE INJUNCTION remedy IRS uses
// to STOP ongoing promoter/aider conduct (not just penalize past
// conduct). Two-prong test under § 7408(b): (1) person engaged
// in specified conduct (§ 6700/§ 6701/§ 6707/§ 6708/Circular 230)
// AND (2) injunction appropriate to prevent recurrence. Action
// commenced at request of Secretary. § 7408(d) venue: district
// court for person's residence, principal place of business, OR
// district where conduct occurred. § 7408(e) treats non-resident
// U.S. citizens/residents as residing in D.C. § 7402(a)
// jurisdiction independent of any other government action.

async fn section_7408_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7408::Section7408Input>,
) -> Result<Json<traderview_expense::section_7408::Section7408Result>, ApiError> {
    Ok(Json(traderview_expense::section_7408::compute(&b)))
}

// ── §7701 entity classification check-the-box ───────────────────────
// Mounted at /api/calc/section-7701. Treas. Reg. § 301.7701-2 default
// classifications (single-member → disregarded entity; multi-member →
// partnership; per-se corporation via federal/state statute or
// § 301.7701-2(b)(8) foreign list); § 301.7701-3 Form 8832 election;
// § 301.7701-3(c)(1)(iv) 60-month lockout after change (waived by
// > 50% ownership change). CTB regs effective 1997-01-01.

async fn section_7701_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7701::Section7701Input>,
) -> Result<Json<traderview_expense::section_7701::Section7701Result>, ApiError> {
    Ok(Json(traderview_expense::section_7701::compute(&b)))
}

// ── §7872 below-market loans ─────────────────────────────────────────
// Mounted at /api/calc/section-7872. Pure compute; AFR imputation
// for below-market loans; §7872(c)(2)(A) $10k de minimis (gift loans
// only, no income-producing assets); §7872(d)(1) $100k NII cap with
// $1k floor; full AFR imputation otherwise.

async fn section_7872_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7872::Section7872Input>,
) -> Result<Json<traderview_expense::section_7872::Section7872Result>, ApiError> {
    if b.loan_principal < Decimal::ZERO
        || b.loan_term_years < Decimal::ZERO
        || b.actual_interest_rate < Decimal::ZERO
        || b.applicable_federal_rate < Decimal::ZERO
        || b.aggregate_outstanding_between_parties < Decimal::ZERO
        || b.borrower_net_investment_income < Decimal::ZERO
    {
        return Err(ApiError::BadRequest(
            "all dollar/rate/term inputs must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_7872::compute(&b)))
}

// ── §1041 transfers between spouses ──────────────────────────────────
// Mounted at /api/calc/section-1041. Pure compute; §1041(a) no
// gain/loss; §1041(b) carryover basis (no dual basis); §1041(c)
// timing rules (1-year automatic / 1-6 year with instrument / 6+
// year with instrument); §1041(d) NR alien disqualification;
// §1223(2) holding period tacking.

async fn section_1041_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1041::Section1041Input>,
) -> Result<Json<traderview_expense::section_1041::Section1041Result>, ApiError> {
    if b.transferor_adjusted_basis < Decimal::ZERO
        || b.fmv_at_transfer < Decimal::ZERO
        || b.sale_price < Decimal::ZERO
    {
        return Err(ApiError::BadRequest(
            "transferor_adjusted_basis, fmv_at_transfer, and sale_price must be >= 0".into(),
        ));
    }
    if b.sale_date < b.transfer_date {
        return Err(ApiError::BadRequest(
            "sale_date must be on or after transfer_date".into(),
        ));
    }
    Ok(Json(traderview_expense::section_1041::compute(&b)))
}

// ── §1015 carryover basis on gifts ───────────────────────────────────
// Mounted at /api/calc/section-1015. Pure compute; §1015(a) general
// carryover; §1015(a) dual-basis rule for depreciated property with
// phantom zone; §1015(d) gift-tax basis increase with two ceilings
// (cap at net appreciation, cap at FMV); §1223(2) holding-period
// tacking on gain path; gift-date start on dual-basis loss path.

async fn section_1015_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1015::Section1015Input>,
) -> Result<Json<traderview_expense::section_1015::Section1015Result>, ApiError> {
    if b.donor_adjusted_basis < Decimal::ZERO
        || b.fmv_at_gift_date < Decimal::ZERO
        || b.gift_tax_paid < Decimal::ZERO
        || b.gift_amount_for_tax_purposes < Decimal::ZERO
        || b.sale_price < Decimal::ZERO
    {
        return Err(ApiError::BadRequest(
            "all dollar inputs must be >= 0".into(),
        ));
    }
    if b.sale_date < b.gift_date {
        return Err(ApiError::BadRequest(
            "sale_date must be on or after gift_date".into(),
        ));
    }
    Ok(Json(traderview_expense::section_1015::compute(&b)))
}

// ── §108 cancellation of debt income ─────────────────────────────────
// Mounted at /api/calc/section-108. §61(a)(12) gross-income default;
// §108(a)(1)(A) bankruptcy full exclusion (priority 1); §108(a)(1)(E)
// QPRI for pre-2026 arrangements (priority 2 over insolvency unless
// elected otherwise); §108(a)(1)(B) insolvency under §108(d)(3) test;
// §108(a)(1)(C) qualified farm; §108(a)(1)(D) QRPBI for non-C-corp;
// §108(b) attribute reduction = excluded amount.

async fn section_108_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_108::Section108Input>,
) -> Result<Json<traderview_expense::section_108::Section108Result>, ApiError> {
    if b.canceled_debt_amount < Decimal::ZERO
        || b.debtor_assets_fmv < Decimal::ZERO
        || b.debtor_liabilities < Decimal::ZERO
    {
        return Err(ApiError::BadRequest(
            "canceled_debt_amount, debtor_assets_fmv, and debtor_liabilities must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_108::compute(&b)))
}

// ── §104 damages for personal injury / sickness ─────────────────────
// Mounted at /api/calc/section-104. §104(a)(2) exclusion for damages
// on account of personal physical injury / sickness (compensatory +
// pain & suffering + lost wages + physical-origin emotional distress
// all excluded); non-physical emotional distress included except
// medical care amount; punitive damages included except § 104(c)
// wrongful-death only-punitives state carveout; interest always
// included; § 104(a) flush prior-§213 tax benefit recapture.

async fn section_104_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_104::Section104Input>,
) -> Result<Json<traderview_expense::section_104::Section104Result>, ApiError> {
    if b.physical_injury_compensatory_dollars < 0
        || b.pain_suffering_physical_origin_dollars < 0
        || b.lost_wages_physical_origin_dollars < 0
        || b.emotional_distress_physical_origin_dollars < 0
        || b.emotional_distress_non_physical_dollars < 0
        || b.medical_care_for_emotional_distress_dollars < 0
        || b.punitive_damages_dollars < 0
        || b.interest_on_award_dollars < 0
        || b.previously_deducted_medical_with_tax_benefit_dollars < 0
    {
        return Err(ApiError::BadRequest(
            "all dollar inputs must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_104::compute(&b)))
}

// ── §1014 stepped-up basis at death ──────────────────────────────────
// Mounted at /api/calc/section-1014. Pure compute; §1014(a)(1) DOD
// step-up; §1014(a)(2) §2032 alternate-valuation-date election;
// §1014(c) IRD denies step-up; §1014(e) 1-year clawback for deathbed
// gifts returning to donor; §1014(f) Form 706 consistent-basis cap.

async fn section_1014_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1014::Section1014Input>,
) -> Result<Json<traderview_expense::section_1014::Section1014Result>, ApiError> {
    if b.decedents_adjusted_basis < Decimal::ZERO || b.fmv_at_dod < Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "decedents_adjusted_basis and fmv_at_dod must be >= 0".into(),
        ));
    }
    if let Some(av) = b.fmv_at_alternate_valuation_date {
        if av < Decimal::ZERO {
            return Err(ApiError::BadRequest(
                "fmv_at_alternate_valuation_date must be >= 0".into(),
            ));
        }
    }
    if let Some(f706) = b.fmv_reported_on_form_706 {
        if f706 < Decimal::ZERO {
            return Err(ApiError::BadRequest(
                "fmv_reported_on_form_706 must be >= 0".into(),
            ));
        }
    }
    Ok(Json(traderview_expense::section_1014::compute(&b)))
}

// ── §1014(e) appreciated-property-by-gift-within-1-year-of-death ────
// Mounted at /api/calc/section-1014e. Anti-abuse companion to § 1014
// general step-up. Triggers when (1) decedent acquired property by
// gift, (2) within 1 year of death, (3) property passes back to
// donor or donor's spouse. Result: basis = decedent's adjusted
// basis immediately before death (no FMV step-up). Credit-shelter-
// trust workaround per NAEPC Journal analysis.

async fn section_1014e_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1014e::Section1014eInput>,
) -> Result<Json<traderview_expense::section_1014e::Section1014eResult>, ApiError> {
    if b.donor_adjusted_basis_at_gift_dollars < 0
        || b.fmv_at_gift_dollars < 0
        || b.decedent_adjusted_basis_immediately_before_death_dollars < 0
        || b.fmv_at_death_dollars < 0
    {
        return Err(ApiError::BadRequest(
            "all dollar inputs must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_1014e::compute(&b)))
}

// ── §1091 wash sale loss disallowance ────────────────────────────────
// Mounted at /api/calc/section-1091. Pure compute; 61-day window
// (sale_date ±30 days inclusive), FIFO basis allocation to replacement
// lots under §1091(d), Rev. Rul. 2008-5 IRA permanent-loss carve-out,
// and §475(f)(1)(C) MTM elector exemption.

async fn section_1091_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1091::Section1091Input>,
) -> Result<Json<traderview_expense::section_1091::Section1091Result>, ApiError> {
    if b.sale_shares < 0
        || b.sale_price_per_share < Decimal::ZERO
        || b.basis_per_share < Decimal::ZERO
    {
        return Err(ApiError::BadRequest(
            "sale_shares, sale_price_per_share, basis_per_share must be >= 0".into(),
        ));
    }
    for p in &b.replacement_purchases {
        if p.shares < 0 || p.price_per_share < Decimal::ZERO {
            return Err(ApiError::BadRequest(
                "replacement shares and price must be >= 0".into(),
            ));
        }
    }
    Ok(Json(traderview_expense::section_1091::compute(&b)))
}

// ── §408A(d)(3)(F) Roth conversion 5-year rule ───────────────────────
// Mounted at /api/calc/section-408a-d3. Pure compute; §408A(d)(4)
// ordering rules (contributions FIFO first, conversions FIFO with
// separate 5-year clocks, earnings last), §408A(d)(3)(F) 5-year
// aging per conversion, age 59½ bypasses 5-year for §72(t).

#[allow(non_snake_case)]
async fn section_408A_d3_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_408A_d3::Section408AD3Input>,
) -> Result<Json<traderview_expense::section_408A_d3::Section408AD3Result>, ApiError> {
    if b.withdrawal_amount < Decimal::ZERO
        || b.total_contributions_basis < Decimal::ZERO
        || b.earnings_balance < Decimal::ZERO
    {
        return Err(ApiError::BadRequest(
            "all dollar inputs must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_408A_d3::compute(&b)))
}

// ── §461(l) excess business loss limitation ─────────────────────────
// Mounted at /api/calc/section-461l. Completes loss-limit cascade
// after §704(d) → §465 → §469. Noncorporate taxpayers only; 2021+
// effective (CARES suspended 2018-2020). 2026 thresholds re-indexed
// by OBBBA: $256k single / $512k MFJ. Excess becomes §172 NOL
// carryforward.

async fn section_461l_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_461l::Section461lInput>,
) -> Result<Json<traderview_expense::section_461l::Section461lResult>, ApiError> {
    if b.aggregate_business_deductions_after_prior_limits < Decimal::ZERO
        || b.aggregate_business_income < Decimal::ZERO
    {
        return Err(ApiError::BadRequest(
            "aggregate dollar inputs must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_461l::compute(&b)))
}

// ── §691 income in respect of decedent (IRD) ─────────────────────────
// Mounted at /api/calc/section-691. §691(a) IRD includible in heir's
// gross income (character preserved); §691(c) deduction = heir's
// pro-rata share of federal estate tax attributable to total IRD per
// Treas. Reg. § 1.691(c)-1(a)(2) two-step. Pairs with §1014(c) IRD
// exception (no step-up on IRD assets).

async fn section_691_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_691::Section691Input>,
) -> Result<Json<traderview_expense::section_691::Section691Result>, ApiError> {
    if b.ird_received_by_heir < Decimal::ZERO
        || b.total_ird_in_estate < Decimal::ZERO
        || b.federal_estate_tax_attributable_to_total_ird < Decimal::ZERO
    {
        return Err(ApiError::BadRequest(
            "all dollar inputs must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_691::compute(&b)))
}

// ── §704(d) partner basis limitation ─────────────────────────────────
// Mounted at /api/calc/section-704d. Outside basis = beginning + cap
// contributions + share of income + §752 liability increases -
// §752 liability decreases - distributions. Loss allowed ≤ basis;
// excess carries forward indefinitely. Sequential pre-§465/§469/
// §461(l) limitation.

async fn section_704d_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_704d::Section704dInput>,
) -> Result<Json<traderview_expense::section_704d::Section704dResult>, ApiError> {
    if b.capital_contributions_this_year < Decimal::ZERO
        || b.share_of_partnership_income < Decimal::ZERO
        || b.share_of_recourse_liabilities_increase < Decimal::ZERO
        || b.share_of_nonrecourse_liabilities_increase < Decimal::ZERO
        || b.share_of_recourse_liabilities_decrease < Decimal::ZERO
        || b.share_of_nonrecourse_liabilities_decrease < Decimal::ZERO
        || b.distributions_received < Decimal::ZERO
        || b.allocated_partnership_loss < Decimal::ZERO
        || b.prior_year_suspended_loss < Decimal::ZERO
    {
        return Err(ApiError::BadRequest(
            "all dollar inputs other than beginning basis must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_704d::compute(&b)))
}

// ── § 721 partnership contribution non-recognition ─────────────────
// Mounted at /api/calc/section-721. Partnership-side counterpart
// to § 351. § 721(a) general bilateral non-recognition rule.
// § 721(b) investment company exception (> 80% readily marketable
// stocks/securities triggers gain recognition; prevents tax-free
// diversification). § 721(c) related foreign partner gain
// recognition (effective Jan 18, 2017) with Gain Deferral Method
// safe harbor under § 1.721(c)-3 allowing remedial-income
// allocation over recovery period. § 721(d) recapture rules tie
// in with § 736(a) retiring partner distributions. Sibling
// modules: § 351 (corporate-side counterpart), § 704(c) (built-
// in gain allocation), § 752 (partnership liabilities), § 754
// (basis adjustment election). Trader-relevant for hedge funds,
// real estate JVs, fund-of-fund structures.

async fn section_721_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_721::Section721Input>,
) -> Result<Json<traderview_expense::section_721::Section721Result>, ApiError> {
    if b.fmv_contributed_cents > 1_000_000_000_000
        || b.basis_contributed_cents > 1_000_000_000_000
    {
        return Err(ApiError::BadRequest(
            "fmv_contributed_cents or basis_contributed_cents out of range".into(),
        ));
    }
    Ok(Json(traderview_expense::section_721::compute(&b)))
}

// ── § 731 partnership distribution gain/loss recognition ────────────
// Mounted at /api/calc/section-731. Direct sibling to § 721
// (contribution non-recognition, iter 264) — completes the
// partnership contribution/distribution cycle. § 731(a)(1) gain
// recognition only to extent MONEY distributed exceeds partner's
// outside basis (applies to current AND liquidating). § 731(a)(2)
// LOSS recognition only on LIQUIDATING distribution when partner
// receives only money + § 751 hot assets + inventory. § 731(b)
// partnership-level non-recognition. § 731(c) marketable
// securities treated as money for gain calculation; § 731(c)(3)
// exceptions for investment partnerships + contribution rollover
// + reduction-of-net-gain rule. Sibling cluster: § 721 + § 732 +
// § 733 + § 736 + § 751 + § 754 + § 707(c).

async fn section_731_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_731::Section731Input>,
) -> Result<Json<traderview_expense::section_731::Section731Result>, ApiError> {
    if b.money_distributed_cents > 1_000_000_000_000
        || b.marketable_securities_fmv_distributed_cents > 1_000_000_000_000
        || b.partner_outside_basis_cents > 1_000_000_000_000
    {
        return Err(ApiError::BadRequest(
            "money_distributed_cents, marketable_securities_fmv_distributed_cents, or partner_outside_basis_cents out of range".into(),
        ));
    }
    Ok(Json(traderview_expense::section_731::compute(&b)))
}

// ── § 752 partnership liabilities — outside basis allocation ────────
// Mounted at /api/calc/section-752. Completes partnership cluster
// (§ 721 + § 731 + § 752). § 752(a) liability share increase
// treated as money contribution (basis +); § 752(b) decrease
// treated as money distribution (basis -, potential § 731(a)(1)
// gain). § 752(c) property-subject-to-liability rule.
// Treas. Reg. § 1.752-1 netting rule for single-transaction
// gross changes. Treas. Reg. § 1.752-2 recourse allocation
// (economic risk of loss). Treas. Reg. § 1.752-3 nonrecourse
// THREE-TIER allocation: tier 1 § 704(b) minimum gain; tier 2
// § 704(c) hypothetical-disposition gain; tier 3 excess-
// nonrecourse profit share. TD 10014 (December 2, 2024) final
// recourse regulations. Sibling cluster: § 721 + § 731 + § 704(b)
// + § 704(c) + § 704(d) + § 705.

async fn section_752_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_752::Section752Input>,
) -> Result<Json<traderview_expense::section_752::Section752Result>, ApiError> {
    if b.partner_share_liabilities_before_cents > 1_000_000_000_000
        || b.partner_share_liabilities_after_cents > 1_000_000_000_000
        || b.partner_outside_basis_before_cents > 1_000_000_000_000
    {
        return Err(ApiError::BadRequest(
            "partner_share_liabilities or partner_outside_basis_before_cents out of range".into(),
        ));
    }
    Ok(Json(traderview_expense::section_752::compute(&b)))
}

// ── §704(c) pre-contribution built-in gain/loss allocation ──────────
// Mounted at /api/calc/section-704c. §704(c)(1)(A) gain allocation on
// disposition; §704(c)(1)(B) 7-year anti-mixing-bowl (distribution to
// other partner); §737 reverse (contributor receives other property);
// §704(c)(1)(C) built-in loss restriction (AJCA 2004 §833(a)); three
// allocation methods (traditional / curative / remedial).

async fn section_704c_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_704c::Section704cInput>,
) -> Result<Json<traderview_expense::section_704c::Section704cResult>, ApiError> {
    if b.pre_contribution_built_in_gain < Decimal::ZERO
        || b.pre_contribution_built_in_loss < Decimal::ZERO
        || b.disposition_gain_realized < Decimal::ZERO
        || b.other_property_received_fmv < Decimal::ZERO
        || b.contributor_outside_basis < Decimal::ZERO
    {
        return Err(ApiError::BadRequest(
            "all dollar inputs must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_704c::compute(&b)))
}

// ── §1235 sale or exchange of patents ───────────────────────────────
// Mounted at /api/calc/section-1235. Automatic LTCG on transfer of
// all substantial rights in a patent by qualifying "holder"
// regardless of holding period. §1235(b) holder = inventor OR
// pre-reduction-to-practice financial backer who paid consideration
// and is not employer or related party. §1235(d) related-party
// disqualification (§267(b) modified, 25% threshold, siblings
// excluded). Treas. Reg. §1.1235-2(b) all-substantial-rights test
// (no geographic / duration / field-of-use limitations).
// Post-TCJA: §1221(a)(3) now excludes inventor's patent from
// capital-asset treatment, so §1235 is the only LTCG path.

async fn section_1235_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1235::Section1235Input>,
) -> Result<Json<traderview_expense::section_1235::Section1235Result>, ApiError> {
    if b.gain_amount_dollars < 0 {
        return Err(ApiError::BadRequest(
            "gain_amount_dollars must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_1235::compute(&b)))
}

// ── §754 election + §743(b) inside basis adjustment ─────────────────
// Mounted at /api/calc/section-754. §743(b) inside basis adjustment
// for transferee partner = outside basis − share of inside basis;
// applies when §754 election in effect OR §743(d)(1)(A) partnership
// BIL > $250k OR §743(d)(1)(B) (TCJA addition) transferee
// hypothetical loss > $250k. Sale/exchange + death-of-partner
// transfer types covered (death takes §1014 FMV outside basis).

async fn section_754_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_754::Section754Input>,
) -> Result<Json<traderview_expense::section_754::Section754Result>, ApiError> {
    if b.transferee_outside_basis < Decimal::ZERO
        || b.transferee_share_of_inside_basis < Decimal::ZERO
        || b.partnership_total_inside_basis < Decimal::ZERO
        || b.partnership_total_fmv < Decimal::ZERO
        || b.transferee_hypothetical_loss_on_immediate_sale < Decimal::ZERO
    {
        return Err(ApiError::BadRequest(
            "all dollar inputs must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_754::compute(&b)))
}

// ── §465 at-risk rules ───────────────────────────────────────────────
// Mounted at /api/calc/section-465. §465(a) loss limited to amount
// at risk; §465(b)(1) cash + basis + recourse; §465(b)(2) external
// pledged property; §465(b)(3) related-party reduces; §465(b)(4)
// general nonrecourse excluded; §465(b)(6) qualified nonrecourse for
// real property included; §465(d) suspended loss carryover; §465(e)
// negative-at-risk recapture.

async fn section_465_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_465::Section465Input>,
) -> Result<Json<traderview_expense::section_465::Section465Result>, ApiError> {
    if b.activity_loss_this_year < Decimal::ZERO
        || b.cash_and_basis_contributed < Decimal::ZERO
        || b.recourse_debt < Decimal::ZERO
        || b.external_pledged_property_fmv < Decimal::ZERO
        || b.qualified_nonrecourse_financing < Decimal::ZERO
        || b.other_nonrecourse_debt < Decimal::ZERO
        || b.related_party_borrowing < Decimal::ZERO
        || b.prior_year_suspended_loss < Decimal::ZERO
    {
        return Err(ApiError::BadRequest(
            "all dollar inputs must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_465::compute(&b)))
}

// ── §401(a)(9) Required Minimum Distributions (RMDs) ─────────────────
// Mounted at /api/calc/section-401a9. SECURE 2.0 age cohorts
// (1949-/1950/1951-1959/1960+), Roth IRA + Roth 401(k) post-2024
// exemptions, Uniform Lifetime Table factors (ages 72-100), §4974
// 25% / 10% correction-window penalty.

async fn section_401a9_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_401a9::Section401a9Input>,
) -> Result<Json<traderview_expense::section_401a9::Section401a9Result>, ApiError> {
    if b.prior_year_end_balance < Decimal::ZERO
        || b.actual_distribution_taken < Decimal::ZERO
    {
        return Err(ApiError::BadRequest(
            "prior_year_end_balance and actual_distribution_taken must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_401a9::compute(&b)))
}

// ── §409A nonqualified deferred compensation ─────────────────────────
// Mounted at /api/calc/section-409a. §409A(a)(1) three-tier penalty
// (immediate income inclusion + 20% additional tax + premium interest
// IRS rate + 1%); §409A(a)(2)(A) permitted distribution events check;
// §409A(a)(2)(B)(i) specified-employee 6-month delay for public
// companies; §409A(a)(3) anti-acceleration.

async fn section_409a_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_409a::Section409aInput>,
) -> Result<Json<traderview_expense::section_409a::Section409aResult>, ApiError> {
    if b.deferred_amount_vested < Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "deferred_amount_vested must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_409a::compute(&b)))
}

// ── §382 NOL limitation following ownership change ──────────────────
// Mounted at /api/calc/section-382. §382(b)(1) annual limitation =
// corp FMV × applicable LT tax-exempt rate; §382(g) ownership change
// (> 50% shift among 5%+ shareholders / 3-year testing period);
// §382(l)(5) bankruptcy exception waives the annual limit at the cost
// of a mandatory interest haircut; §382(h) NUBIG recognition can
// increase the limit during the 5-year recognition period. Pairs with
// /api/calc/section-172 for the underlying NOL deduction.

async fn section_382_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_382::Section382Input>,
) -> Result<Json<traderview_expense::section_382::Section382Result>, ApiError> {
    if b.corporation_fmv_at_change < Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "corporation_fmv_at_change must be >= 0".into(),
        ));
    }
    if b.pre_change_nol_carryover < Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "pre_change_nol_carryover must be >= 0".into(),
        ));
    }
    if b.mandatory_interest_haircut_l5 < Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "mandatory_interest_haircut_l5 must be >= 0".into(),
        ));
    }
    if b.recognized_built_in_gain_this_year < Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "recognized_built_in_gain_this_year must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_382::compute(&b)))
}

// ── §83(i) qualified equity grant 5-year income-tax deferral ────────
// Mounted at /api/calc/section-83i. TCJA addition; defers federal
// income tax (NOT FICA) up to 5 years on NQSO exercise / RSU vesting
// for eligible employees of eligible private corporations. §83(i)(2)(C)
// eligible-corp test (no tradable stock + 80% broad-based written
// plan); §83(i)(3)(B) excluded-employee exclusions (1% owner, CEO/CFO,
// top-4 paid in current or any 10 prior years); §83(i)(1)(B) deferral
// end triggers (5y max, IPO/tradable, buyback, revocation, becoming
// excluded — earliest wins); §83(i)(4)(A) 30-day election window.

async fn section_83i_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_83i::Section83iInput>,
) -> Result<Json<traderview_expense::section_83i::Section83iResult>, ApiError> {
    if b.deferred_income_amount < Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "deferred_income_amount must be >= 0".into(),
        ));
    }
    if b.fmv_at_vesting_for_fica < Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "fmv_at_vesting_for_fica must be >= 0".into(),
        ));
    }
    if b.as_of_date < b.vesting_or_exercise_date {
        return Err(ApiError::BadRequest(
            "as_of_date must be on or after vesting_or_exercise_date".into(),
        ));
    }
    Ok(Json(traderview_expense::section_83i::compute(&b)))
}

// ── §408(m) collectibles in IRA ──────────────────────────────────────
// Mounted at /api/calc/section-408m. Pure compute; §408(m)(1)
// prohibited collectible = deemed distribution; §408(m)(3)(A)
// Eagle / state-issued coin exception; §408(m)(3)(B) bullion
// exception with purity threshold (.995 gold / .999 silver / .9995
// platinum / .9995 palladium) AND trustee custody.

async fn section_408m_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_408m::Section408mInput>,
) -> Result<Json<traderview_expense::section_408m::Section408mResult>, ApiError> {
    if b.purchase_price < Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "purchase_price must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_408m::compute(&b)))
}

// ── §41 R&D credit (Regular + Alternative Simplified + §280C(c)) ─────
// Mounted at /api/calc/section-41. Practical for algorithmic traders
// building custom trading systems + data pipelines + ML models that
// qualify as research under §41(d). Two computation methods:
// §41(a)(1) Regular Credit = 20% × (QRE − base amount) where base =
// max(fixed-base-% × 4-year avg gross receipts, 50% × current QRE);
// fixed-base-% capped at 16% under §41(c)(3); startup uses 3%.
// §41(c)(4) Alternative Simplified Credit (ASC) = 14% × (QRE − 50% ×
// prior 3-year avg QRE); §41(c)(4)(B) startup path 6% × current QRE
// when no QRE in any of 3 prior years. §280C(c)(2) reduced-credit
// election reduces credit by 21% in exchange for keeping full §174
// deduction (§280C(c)(3) election must be on original return).

async fn section_41_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_41::Section41Input>,
) -> Result<Json<traderview_expense::section_41::Section41Result>, ApiError> {
    if b.current_year_qre_cents < 0
        || b.prior_3_year_avg_qre_cents < 0
        || b.prior_4_year_avg_gross_receipts_cents < 0
    {
        return Err(ApiError::BadRequest(
            "non-negative cents inputs required".into(),
        ));
    }
    Ok(Json(traderview_expense::section_41::compute(&b)))
}

// ── §408(d)(3) IRA 60-day rollover rules ─────────────────────────────
// Mounted at /api/calc/section-408-d3. Pure compute; validates that
// an indirect IRA rollover satisfies (a) 60-day deposit window,
// (b) Bobrow once-per-12-months aggregated across all IRAs, with
// §408(d)(3)(I) hardship-waiver path and §72(t) 10% early withdrawal
// penalty calculation on failed rollovers.

async fn section_408_d3_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_408_d3::Section408D3Input>,
) -> Result<Json<traderview_expense::section_408_d3::Section408D3Result>, ApiError> {
    if b.distribution_amount < Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "distribution_amount must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_408_d3::compute(&b)))
}

// ── §871(m) dividend-equivalent withholding for non-US persons ───────
// Mounted at /api/calc/section-871m. Pure compute; classifies a US-
// equity-linked derivative as a Specified Equity-Linked Instrument
// (SELI) based on delta + original term, applies statutory 30% or
// treaty-reduced rate.

async fn section_871m_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_871m::Section871MInput>,
) -> Result<Json<traderview_expense::section_871m::Section871MResult>, ApiError> {
    if b.dividend_equivalent_amount < Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "dividend_equivalent_amount must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_871m::compute(&b)))
}

// ── §911 foreign earned income exclusion ─────────────────────────────
// Mounted at /api/calc/section-911. §911(a)(1) FEIE inflation-indexed
// (2025 $130k / 2026 $132,900 caller-supplied year-agnostic) + §911(a)(2)
// housing exclusion + §911(b)(1) foreign earned income definition (no
// US-gov / passive / pension) + §911(c)(2) housing cap 30% × FEIE +
// §911(d)(1)(A) bona fide residence test + §911(d)(1)(B) physical
// presence test ≥ 330 full days + §911(d)(7) base housing 16% × FEIE.

async fn section_911_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_911::Section911Input>,
) -> Result<Json<traderview_expense::section_911::Section911Result>, ApiError> {
    if b.feie_inflation_adjusted_cap_dollars < 0
        || b.foreign_earned_income_dollars < 0
        || b.housing_expenses_dollars < 0
        || b.physical_presence_days_in_12_month_period > 366
    {
        return Err(ApiError::BadRequest(
            "non-negative dollar inputs and days in 12-month period ≤ 366 required".into(),
        ));
    }
    Ok(Json(traderview_expense::section_911::compute(&b)))
}

// ── §1092 straddle loss deferral ──────────────────────────────────────
// Mounted at /api/calc/section-1092. Pure compute; defers loss on a
// closed straddle leg up to unrecognized gain on remaining legs;
// §1092(c)(4)(B) qualified covered call carve-out exempts qualifying
// covered call positions from straddle treatment.

async fn section_1092_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1092::Section1092Input>,
) -> Result<Json<traderview_expense::section_1092::Section1092Result>, ApiError> {
    if b.realized_loss_on_disposed_leg < Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "realized_loss_on_disposed_leg must be >= 0 (pass loss as positive)".into(),
        ));
    }
    Ok(Json(traderview_expense::section_1092::compute(&b)))
}

// ── §1295 PFIC Qualified Electing Fund election ──────────────────────
// Mounted at /api/calc/section-1295. Pure compute; pro-rata
// inclusion of PFIC ordinary earnings + net capital gain per §1293.
// Character preserved (LTCG stays LTCG). Basis + PTI account
// evolution; previously-taxed-income distribution excluded from gross.

async fn section_1295_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1295::Section1295Input>,
) -> Result<Json<traderview_expense::section_1295::Section1295Result>, ApiError> {
    if b.adjusted_basis_year_start < Decimal::ZERO
        || b.distributions_received < Decimal::ZERO
        || b.pti_account_year_start < Decimal::ZERO
    {
        return Err(ApiError::BadRequest(
            "adjusted_basis_year_start, distributions_received, pti_account_year_start must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_1295::compute(&b)))
}

// ── §864(b)(2) non-US trader/investor safe harbor ────────────────────
// Mounted at /api/calc/section-864b2. Pure compute; classifies a
// non-US person's US securities/commodities trading as effectively
// connected or not, based on the four-factor test (non-US person /
// own-account / not a dealer / no US office).

async fn section_864b2_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_864b2::Section864B2Input>,
) -> Result<Json<traderview_expense::section_864b2::Section864B2Result>, ApiError> {
    Ok(Json(traderview_expense::section_864b2::compute(&b)))
}

// ── §163(d) investment interest expense limitation ───────────────────
// Mounted at /api/calc/section-163d. Pure compute; investment
// interest deductible only up to net investment income, indefinite
// carryforward. Models the §1(h)(11)(D)(i) QD election and
// §163(d)(4)(B)(iii) LTCG election that boost the limit but forfeit
// preferential rates.

async fn section_163d_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_163d::Section163dInput>,
) -> Result<Json<traderview_expense::section_163d::Section163dResult>, ApiError> {
    if b.investment_interest_expense < Decimal::ZERO
        || b.interest_income < Decimal::ZERO
        || b.ordinary_dividends < Decimal::ZERO
        || b.qualified_dividends < Decimal::ZERO
        || b.other_investment_expenses < Decimal::ZERO
        || b.prior_year_carryforward < Decimal::ZERO
    {
        return Err(ApiError::BadRequest(
            "all dollar inputs must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_163d::compute(&b)))
}

// ── §163(h) home mortgage interest deduction ────────────────────────
// Mounted at /api/calc/section-163h. Universal qualified residence
// interest computation: $750k acquisition indebtedness cap (TCJA,
// made permanent by OBBBA 2025 § 70108); $1M grandfathered cap for
// pre-2017-12-16 mortgages; MFS half-caps ($375k / $500k); home
// equity interest permanently disallowed unless acquisition use;
// PMI premiums reinstated as deductible 2026+ per OBBBA; refinance
// blended-cap calculation under § 163(h)(3)(F)(iii).

async fn section_163h_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_163h::Section163hInput>,
) -> Result<Json<traderview_expense::section_163h::Section163hResult>, ApiError> {
    if b.acquisition_indebtedness_balance < Decimal::ZERO
        || b.non_acquisition_home_equity_balance < Decimal::ZERO
        || b.interest_paid_acquisition < Decimal::ZERO
        || b.interest_paid_non_acquisition_home_equity < Decimal::ZERO
        || b.mortgage_insurance_premiums_paid < Decimal::ZERO
    {
        return Err(ApiError::BadRequest(
            "all dollar inputs must be >= 0".into(),
        ));
    }
    if let Some(grand) = b.grandfathered_refinance_portion {
        if grand < Decimal::ZERO {
            return Err(ApiError::BadRequest(
                "grandfathered_refinance_portion must be >= 0 when set".into(),
            ));
        }
    }
    Ok(Json(traderview_expense::section_163h::compute(&b)))
}

// ── §280F luxury auto depreciation cap ───────────────────────────────
// Mounted at /api/calc/section-280f. Pure compute; caps annual
// depreciation on passenger autos under §280F(a)(1). Year-by-year
// caps from Rev. Proc. tables 2020-2024 (caller_override for 2025+).
// §280F(d)(5) heavy-vehicle carve-out for > 6,000 lb GVWR exempts.

async fn section_280f_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_280f::Section280FInput>,
) -> Result<Json<traderview_expense::section_280f::Section280FResult>, ApiError> {
    if b.cost_basis < Decimal::ZERO {
        return Err(ApiError::BadRequest("cost_basis must be >= 0".into()));
    }
    Ok(Json(traderview_expense::section_280f::compute(&b)))
}

// ── §280B demolition of structures ──────────────────────────────────
// Mounted at /api/calc/section-280b. §280B(1) NO deduction for
// demolition costs or loss sustained; §280B(2) capitalized to land
// basis. IRS Notice 90-21 casualty exception allows separate § 165
// casualty loss when structure was casualty-damaged before
// demolition; § 168(i)(4) GAA election can permit abandonment loss
// via GAA termination under Treas. Reg. § 1.168(i)-1(e).

async fn section_280b_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_280b::Section280BInput>,
) -> Result<Json<traderview_expense::section_280b::Section280BResult>, ApiError> {
    if b.demolition_costs_paid_dollars < 0
        || b.structure_remaining_adjusted_basis_dollars < 0
        || b.structure_salvage_value_dollars < 0
        || b.land_pre_demolition_basis_dollars < 0
    {
        return Err(ApiError::BadRequest(
            "all dollar inputs must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_280b::compute(&b)))
}

// ── §280E controlled-substance trafficking deduction disallowance ────
// Mounted at /api/calc/section-280e. Disallows §162 deductions for
// trafficking in Schedule I/II controlled substances regardless of
// state legalization. COGS always allowed (Champ T.C. 2007); non-
// trafficking bifurcated activity expenses always allowed.
// EO 14370 (2025-12-18) directs DEA Schedule I → III rescheduling
// for marijuana; DOJ Final Order partially reschedules FDA-approved
// and state-licensed medical marijuana but leaves bulk / unlicensed
// crops in Schedule I.

async fn section_280e_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_280e::Section280eInput>,
) -> Result<Json<traderview_expense::section_280e::Section280eResult>, ApiError> {
    if b.gross_revenue_dollars < 0
        || b.cogs_dollars < 0
        || b.trafficking_business_expenses_dollars < 0
        || b.non_trafficking_bifurcated_expenses_dollars < 0
    {
        return Err(ApiError::BadRequest(
            "all dollar inputs must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_280e::compute(&b)))
}

// ── §481(a) accounting method change adjustment ──────────────────────
// Mounted at /api/calc/section-481. Pure compute; cumulative MTM
// adjustment for §475(f) trader-status election, 4-year ratable
// spread on positive (gain) per Rev. Proc. 2015-13, immediate
// recognition on negative (loss).

async fn section_481_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_481::Section481Input>,
) -> Result<Json<traderview_expense::section_481::Section481Result>, ApiError> {
    Ok(Json(traderview_expense::section_481::compute(&b)))
}

// ── §530 Coverdell Education Savings Accounts (ESA) ──────────────────
// Mounted at /api/calc/section-530. §530(b)(1)(A)(ii) $2,000 statutory
// annual contribution limit per beneficiary (unchanged since 2002; does
// NOT inflation-adjust); §530(b)(1)(A)(i) beneficiary must be under
// age 18 for contributions; §530(c) MAGI phaseout 95K-110K single +
// 190K-220K MFJ; §530(d)(7) special-needs beneficiary exception waives
// both age limits; §530(d)(8) age-30 distribution requirement (waived
// for special needs); §4973 6% excise tax on excess imposed on the
// BENEFICIARY (not contributor) annually. Sibling to section_223 (HSA)
// and section_219 (IRA) tax-favored savings vehicles.

async fn section_530_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_530::Section530Input>,
) -> Result<Json<traderview_expense::section_530::Section530Result>, ApiError> {
    if b.contributor_modified_agi_cents < 0
        || b.aggregate_contributions_for_beneficiary_cents < 0
    {
        return Err(ApiError::BadRequest(
            "non-negative cents inputs required".into(),
        ));
    }
    if !(1990..=2100).contains(&b.year) {
        return Err(ApiError::BadRequest(
            "year must be in [1990, 2100]".into(),
        ));
    }
    Ok(Json(traderview_expense::section_530::compute(&b)))
}

// ── §1031(f) related-party 2-year clawback ───────────────────────────
// Mounted at /api/calc/section-1031-f. Pure compute; evaluates whether
// a subsequent disposition of property received in a related-party
// §1031 exchange triggers retroactive gain recognition.

async fn section_1031_f_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1031_f::Section1031FInput>,
) -> Result<Json<traderview_expense::section_1031_f::Section1031FResult>, ApiError> {
    if b.deferred_gain_at_exchange < Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "deferred_gain_at_exchange must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_1031_f::compute(&b)))
}

// ── §1033 involuntary conversion gain-deferral ───────────────────────
// Mounted at /api/calc/section-1033. §1033(a)(2)(A) gain recognized to
// extent amount realized exceeds replacement cost (capped at realized
// gain); §1033(b)(2) basis in replacement = replacement cost − deferred
// gain; replacement windows: 2-year general (§1033(a)(2)(B)(i)) / 3-year
// condemnation real-property-trade-or-investment (§1033(g)(4)) / 4-year
// federally-declared-disaster principal residence (§1033(h)(1)(B)) / 5-year
// qualifying-disaster property (§1033(h)(2)(A)); similar-or-related-in-
// service-or-use test under Treas. Reg. § 1.1033(a)-2 (functional-use for
// owner-users, end-use for lessors of investment real estate); §1033(a)(2)
// election required for proceeds-into-property path (mandatory §1033(a)(1)
// when proceeds converted directly into property).

async fn section_1033_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1033::Section1033Input>,
) -> Result<Json<traderview_expense::section_1033::Section1033Result>, ApiError> {
    if b.amount_realized_cents < 0
        || b.replacement_cost_cents < 0
    {
        return Err(ApiError::BadRequest(
            "amount_realized_cents and replacement_cost_cents must be non-negative".into(),
        ));
    }
    Ok(Json(traderview_expense::section_1033::compute(&b)))
}

// ── §1259 constructive sale of appreciated financial position ────────
// Mounted at /api/calc/section-1259. Pure compute; evaluates whether
// a hedge transaction triggers constructive sale of an appreciated
// long position, including the §1259(c)(3)(A) safe harbor.

async fn section_1259_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1259::Section1259Input>,
) -> Result<Json<traderview_expense::section_1259::Section1259Result>, ApiError> {
    Ok(Json(traderview_expense::section_1259::compute(&b)))
}

// ── §1361 S-corp eligibility 6-prong test ──────────────────────────
// Mounted at /api/calc/section-1361. §1361(b)(1) eligibility prongs:
// (A) domestic corporation + (B) not ineligible corp under
// §1361(b)(2) (financial institutions reserve method / insurance
// Subchapter L / FSC / DISC) + (C) ≤ 100 shareholders after
// §1361(c)(1) family attribution + (D) shareholders limited to
// individuals / qualifying estates / qualifying trusts (no
// partnerships / no non-S corps) + (E) no nonresident alien
// shareholders + (F) only one class of stock (voting-rights
// differences ARE permitted; economic differences NOT).

async fn section_1361_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1361::Section1361Input>,
) -> Result<Json<traderview_expense::section_1361::Section1361Result>, ApiError> {
    Ok(Json(traderview_expense::section_1361::compute(&b)))
}

// ── § 1367 S-corp shareholder stock basis adjustments ───────────────
// Mounted at /api/calc/section-1367. Core math for trader S-corp
// entity selection. § 1367(a)(1) increases (separately stated +
// nonseparately computed income + depletion excess); § 1367(a)(2)
// decreases (distributions + losses + nondeductibles + depletion);
// Treas. Reg. § 1.1367-1(f) standard ordering — increases →
// distributions → NONDEDUCTIBLES (lost if excess) → LOSSES
// (suspended if excess under § 1366(d)(2)); Treas. Reg.
// § 1.1367-1(g) election — losses-before-nondeductibles, with
// nondeductibles SUSPENDED instead of lost. § 1368(b)(2) excess
// distribution treated as capital gain. Sibling S-corp cluster:
// § 1361 (definition + eligibility) + § 1366 (pass-through) +
// § 1368 (distribution mechanics) + § 1374 (built-in gains tax).
// Form 7203 is the IRS basis-tracking schedule.

async fn section_1367_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1367::Section1367Input>,
) -> Result<Json<traderview_expense::section_1367::Section1367Result>, ApiError> {
    Ok(Json(traderview_expense::section_1367::compute(&b)))
}

// ── § 1368 S-corp distributions ─────────────────────────────────────
// Mounted at /api/calc/section-1368. Direct sibling to § 1367
// (basis adjustments). § 1368(b) governs S-corps without
// accumulated E&P: tax-free basis reduction then capital gain.
// § 1368(c) four-step ordering for S-corps with E&P: AAA tax-free
// → E&P dividend → remaining-basis tax-free → capital gain.
// § 1368(e)(1)(C) net-negative-adjustment rule excludes net
// negative from AAA-available calculation for distribution
// purposes. § 1368(e)(3) election with unanimous shareholder
// consent reverses (c)(1) and (c)(2) — distribute E&P first as
// dividends (used to purge E&P for § 1375 PII tax avoidance).
// Form 1120-S Schedule M-2 tracks AAA + E&P year over year.
// Sibling cluster: § 1361 + § 1366 + § 1367 + § 1374 + § 1375.

async fn section_1368_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1368::Section1368Input>,
) -> Result<Json<traderview_expense::section_1368::Section1368Result>, ApiError> {
    Ok(Json(traderview_expense::section_1368::compute(&b)))
}

// ── § 1375 S-corp passive investment income tax ─────────────────────
// Mounted at /api/calc/section-1375. Completes S-corp cluster
// (§ 1361 + § 1366 + § 1367 + § 1368 + § 1374 + § 1375). Tax
// engages when S-corp has accumulated E&P AND passive investment
// income exceeds 25% of gross receipts. § 1375(b)(1)(B) ENPI
// formula: NPI × (PII - 25% × GR) / PII. § 1375(b)(1)(A) caps
// ENPI at taxable income. Tax at highest § 11(b) corporate rate
// (21% post-TCJA). Companion: § 1362(d)(3) — three consecutive
// years of E&P + >25% PII terminates S election. § 1362(g) —
// 5-year re-election waiting period after termination.

async fn section_1375_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1375::Section1375Input>,
) -> Result<Json<traderview_expense::section_1375::Section1375Result>, ApiError> {
    if b.corporate_tax_rate_bps < 0 || b.corporate_tax_rate_bps > 10_000 {
        return Err(ApiError::BadRequest(
            "corporate_tax_rate_bps out of range".into(),
        ));
    }
    Ok(Json(traderview_expense::section_1375::compute(&b)))
}

// ── §1374 S-corp built-in gains (BIG) tax ───────────────────────────
// Mounted at /api/calc/section-1374. Models the 5-year §1374(d)(7)
// recognition period (PATH Act 2015), the §1374(d)(2) lesser-of-three
// NRBIG computation (recognized BIG vs taxable-income limit vs NUBIG
// ceiling), §1374(b)(2) C-corp NOL deduction, §1374(b)(3) credit
// offset, and §1374(d)(2)(B) NRBIG carryforward when TI limit binds.
// 21% rate under §11(b) post-TCJA but rate is parameterized.

async fn section_1374_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1374::Section1374Input>,
) -> Result<Json<traderview_expense::section_1374::Section1374Result>, ApiError> {
    if b.nubig_at_conversion < Decimal::ZERO
        || b.recognized_big_this_year < Decimal::ZERO
        || b.recognized_bil_this_year < Decimal::ZERO
        || b.cumulative_prior_nrbig < Decimal::ZERO
        || b.c_corp_nol_carryforward < Decimal::ZERO
        || b.c_corp_credit_offset < Decimal::ZERO
        || b.nrbig_carryforward_from_prior_year < Decimal::ZERO
    {
        return Err(ApiError::BadRequest(
            "all dollar inputs must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_1374::compute(&b)))
}

// ── §475(c)(2) dealer-in-securities classification ──────────────────
// Mounted at /api/calc/section-475c2. Returns one of Dealer /
// TraderWithMtmElection / TraderWithoutMtmElection / Investor based
// on the §475(c)(2) two-prong dealer test (customer + inventory
// prongs), Treas. Reg. §1.475(c)-1 negligible-sales exception, IRS
// Topic 429 trader case-law criteria (short-term profit motive,
// substantial activity, continuous & regular), and §475(f) election
// status. Drives downstream wash-sale, $3k capital loss cap, and
// ordinary-vs-capital character treatment across the system.

async fn section_475c2_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_475c2::Section475c2Input>,
) -> Result<Json<traderview_expense::section_475c2::Section475c2Result>, ApiError> {
    Ok(Json(traderview_expense::section_475c2::compute(&b)))
}

// ── §213 medical expense deduction ──────────────────────────────────
// Mounted at /api/calc/section-213. §213(a) 7.5% AGI floor (CAA 2020
// § 103 made permanent); §213(d) qualified medical care; §213(d)(10)
// age-tiered LTC premium caps from IRS Rev. Proc. 2024-40 (2025) and
// Rev. Proc. 2025-32 (2026); HSA/FSA/HRA reimbursement
// double-deduction prevention. Requires Schedule A itemization.

async fn section_213_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_213::Section213Input>,
) -> Result<Json<traderview_expense::section_213::Section213Result>, ApiError> {
    if b.adjusted_gross_income < Decimal::ZERO
        || b.qualified_medical_expenses_other_than_ltc_premiums < Decimal::ZERO
        || b.ltc_premiums_paid < Decimal::ZERO
        || b.hsa_fsa_hra_reimbursements < Decimal::ZERO
    {
        return Err(ApiError::BadRequest(
            "all dollar inputs must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_213::compute(&b)))
}

// ── §170 charitable contribution deduction (post-OBBBA 2026 changes) ──
// Mounted at /api/calc/section-170. §170(a) general deduction;
// §170(b)(1) per-category AGI ceilings (60% cash to public charity made
// permanent by OBBBA + 50% non-cash to public + 30% capital-gain
// property or cash to 30%-limit orgs + 20% capital-gain property to
// private foundations); §170(b)(1)(I) OBBBA §70425 0.5% AGI FLOOR for
// itemizers eff. tax years after 2025-12-31 (amounts below floor carry
// forward 5 years); §170(p) OBBBA non-itemizer above-the-line deduction
// $1,000 single / $2,000 MFJ for cash to public charity only eff. 2026;
// §170(d)(1) 5-year carryforward for ceiling-blocked + floor-blocked.
// Sibling to section_170e (built-in-gain ordinary-income reduction).

async fn section_170_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_170::Section170Input>,
) -> Result<Json<traderview_expense::section_170::Section170Result>, ApiError> {
    if b.agi_cents < 0
        || b.cash_to_public_charity_cents < 0
        || b.capital_gain_property_to_public_charity_cents < 0
        || b.cash_to_private_foundation_cents < 0
    {
        return Err(ApiError::BadRequest(
            "non-negative cents inputs required".into(),
        ));
    }
    if !(1990..=2100).contains(&b.year) {
        return Err(ApiError::BadRequest(
            "year must be in [1990, 2100]".into(),
        ));
    }
    Ok(Json(traderview_expense::section_170::compute(&b)))
}

// ── §219 IRA contribution deduction + Roth phaseout ──────────────────
// Mounted at /api/calc/section-219. §219(a) above-the-line Traditional
// IRA deduction; §219(b)(5)(A) 2026 $7,500 base contribution limit (was
// $7,000 for 2024/2025); §219(b)(5)(B) age-50+ catch-up — SECURE 2.0
// indexed starting 2024: 2026 = $1,100 (was statutory $1,000 pre-SECURE
// -2.0 still applies for 2024). §219(g) Traditional deduction phaseout
// when taxpayer OR spouse covered by workplace retirement plan: 2026
// Single $81K-$91K + MFJ taxpayer-covered $129K-$149K + MFJ spouse-only
// covered $242K-$252K (§219(g)(7) widened range) + MFS $0-$10K.
// §408A(c)(3) Roth contribution phaseout: 2026 Single $153K-$168K + MFJ
// $242K-$252K + MFS $0-$10K. §4973 6% excise on excess. Earned income
// caps contribution under §219(b)(1).

async fn section_219_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_219::Section219Input>,
) -> Result<Json<traderview_expense::section_219::Section219Result>, ApiError> {
    if b.contributions_cents < 0 {
        return Err(ApiError::BadRequest(
            "contributions_cents must be non-negative".into(),
        ));
    }
    if !(1990..=2100).contains(&b.year) {
        return Err(ApiError::BadRequest(
            "year must be in [1990, 2100]".into(),
        ));
    }
    Ok(Json(traderview_expense::section_219::compute(&b)))
}

// ── §221 student loan interest deduction (above-the-line) ────────────
// Mounted at /api/calc/section-221. §221(a) above-the-line deduction
// up to $2,500 for interest paid on qualified education loans;
// §221(b)(1) STATUTORY $2,500 cap does NOT inflation-adjust;
// §221(b)(2) MAGI phaseout — 2026 single/HoH $85K-$100K + MFJ
// $175K-$205K + 2025 single $80K-$95K + MFJ $165K-$195K; §221(e)(2)
// EXCLUDES Married Filing Separately filers entirely. Above-the-line
// = available even without itemizing.

async fn section_221_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_221::Section221Input>,
) -> Result<Json<traderview_expense::section_221::Section221Result>, ApiError> {
    if b.interest_paid_cents < 0 || b.modified_agi_cents < 0 {
        return Err(ApiError::BadRequest(
            "non-negative cents inputs required".into(),
        ));
    }
    if !(1990..=2100).contains(&b.year) {
        return Err(ApiError::BadRequest(
            "year must be in [1990, 2100]".into(),
        ));
    }
    Ok(Json(traderview_expense::section_221::compute(&b)))
}

// ── §223 Health Savings Accounts (HSAs) — triple-tax-advantaged ──────
// Mounted at /api/calc/section-223. §223(a) above-the-line deduction;
// §223(b)(2) contribution limits (2026 self-only $4,400 + family
// $8,750; 2025 $4,300 + $8,550); §223(b)(3) age-55+ catch-up $1,000
// STATUTORY not inflation-adjusted; §223(c)(2) HDHP definition (2026
// self-only min deductible $1,700 max OOP $8,500; family $3,400 / $17K;
// 2025 self-only $1,650 / $8,300; family $3,300 / $16,600). §4973 6%
// excise tax on excess contributions modeled.

async fn section_223_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_223::Section223Input>,
) -> Result<Json<traderview_expense::section_223::Section223Result>, ApiError> {
    if b.contributions_cents < 0
        || b.hdhp_deductible_cents < 0
        || b.hdhp_out_of_pocket_max_cents < 0
    {
        return Err(ApiError::BadRequest(
            "non-negative cents inputs required".into(),
        ));
    }
    if !(1990..=2100).contains(&b.year) {
        return Err(ApiError::BadRequest(
            "year must be in [1990, 2100]".into(),
        ));
    }
    Ok(Json(traderview_expense::section_223::compute(&b)))
}

// ── §243 / §246 Dividends Received Deduction (DRD) ──────────────────
// Mounted at /api/calc/section-243. C-corp DRD with §243(a)(1) 50%
// baseline tier (<20% owned), §243(c) 65% (20-79%), §243(b) 100%
// (≥80% qualifying group); §246(c) holding-period (>45 days in
// 91-day window for common / short-preferred, >90 days in 181-day
// window for long-preferred — failure = full disallowance); §246A
// debt-financed portfolio stock reduction (DRD% × (100% −
// indebtedness%), not applicable to 80%+ tier).

async fn section_243_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_243::Section243Input>,
) -> Result<Json<traderview_expense::section_243::Section243Result>, ApiError> {
    if b.dividend_received_dollars < 0 {
        return Err(ApiError::BadRequest(
            "dividend_received_dollars must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_243::compute(&b)))
}

// ── §250 GILTI/FDII (NCTI/FDDEI post-OBBBA 2025) deduction ──────────
// Mounted at /api/calc/section-250. TCJA §14202 50% GILTI / 37.5%
// FDII deductions; OBBBA 2025 amendments effective tax years after
// 2025-12-31 rename to NCTI/FDDEI, reduce deductions to 40%/33.34%,
// eliminate DTIR/NDTIR (QBAI 10% return), raise FTC from 80% to 90%.

async fn section_250_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_250::Section250Input>,
) -> Result<Json<traderview_expense::section_250::Section250Result>, ApiError> {
    if b.gilti_ncti_income_dollars < 0
        || b.fdii_fddei_income_dollars < 0
        || b.qbai_dollars < 0
        || b.deemed_paid_foreign_taxes_dollars < 0
    {
        return Err(ApiError::BadRequest(
            "all dollar inputs must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_250::compute(&b)))
}

// ── §59A BEAT (Base Erosion and Anti-Abuse Tax) ─────────────────────
// Mounted at /api/calc/section-59a. TCJA §14401 BEAT for large
// multinationals: $500M 3-yr avg gross receipts gate, 3% base
// erosion percentage gate (2% banks/dealers), rate 5%→10%→10.5%
// post-OBBBA (was scheduled 12.5% under TCJA, repealed by OBBBA);
// banks/dealers +1% surcharge throughout; S corps/REITs/RICs
// categorically excluded under §59A(e)(2).

async fn section_59a_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_59a::Section59aInput>,
) -> Result<Json<traderview_expense::section_59a::Section59aResult>, ApiError> {
    if b.gross_receipts_year_minus_1_dollars < 0
        || b.gross_receipts_year_minus_2_dollars < 0
        || b.gross_receipts_year_minus_3_dollars < 0
        || b.base_erosion_payments_dollars < 0
        || b.total_deductions_dollars < 0
        || b.nol_deduction_dollars < 0
        || b.regular_tax_liability_dollars < 0
    {
        return Err(ApiError::BadRequest(
            "all dollar inputs except taxable_income must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_59a::compute(&b)))
}

// ── §6045 broker information reporting (Form 1099-B / 1099-DA) ───────
// Mounted at /api/calc/section-6045. §6045(a) requires brokers (anyone
// in ordinary-course-of-business standing ready to effect sales for
// others) to file Form 1099-B (securities + barter) or new Form 1099-DA
// (digital assets, eff. 2025-01-01). §6045(g) bifurcates into COVERED
// (broker required to report adjusted basis) vs NON-COVERED (gross
// proceeds only). Acquisition cutoffs per Treas. Reg. § 1.6045-1(a)(15):
// stock 2011-01-01 + mutual fund/DRIP 2012-01-01 + less-complex debt
// 2014-01-01 + more-complex debt 2016-01-01 + digital asset 2026-01-01
// (NEW under IIJA § 80603 amending § 6045; requires continuous broker-
// account holding). NO DE MINIMIS — even one cent triggers reporting.

async fn section_6045_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6045::Section6045Input>,
) -> Result<Json<traderview_expense::section_6045::Section6045Result>, ApiError> {
    if b.proceeds_cents < 0 {
        return Err(ApiError::BadRequest(
            "proceeds_cents must be non-negative".into(),
        ));
    }
    if !(1990..=2100).contains(&b.acquisition_year)
        || !(1..=12).contains(&b.acquisition_month)
        || !(1..=31).contains(&b.acquisition_day)
        || !(1990..=2100).contains(&b.transaction_year)
    {
        return Err(ApiError::BadRequest(
            "acquisition + transaction dates must be valid".into(),
        ));
    }
    Ok(Json(traderview_expense::section_6045::compute(&b)))
}

// ── §6050I cash transaction reporting (Form 8300) ────────────────────
// Mounted at /api/calc/section-6050i. §6050I(a) requires any person
// engaged in a trade or business who receives more than $10,000 in
// cash in one transaction (or two or more related transactions within
// 24 hours) to report to IRS AND FinCEN within 15 days on Form 8300.
// §6050I(d) cash definition includes currency + cashier's checks +
// money orders + bank drafts WITH FACE AMOUNT ≤ $10,000 (personal
// checks and wire transfers are NOT cash). IIJA §80603 added digital
// assets effective 2024-01-01 BUT IRS Announcement 2024-04
// SUSPENDED implementation — digital assets currently EXCLUDED from
// § 6050I cash pending IRS regulations. §6721 intentional-disregard
// penalty = greater of $250K or aggregate; §7203 willful-failure
// criminal exposure.

async fn section_6050i_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6050i::Section6050IInput>,
) -> Result<Json<traderview_expense::section_6050i::Section6050IResult>, ApiError> {
    if b.single_instrument_face_amount_cents < 0
        || b.aggregate_related_24_hour_amount_cents < 0
    {
        return Err(ApiError::BadRequest(
            "non-negative cents inputs required".into(),
        ));
    }
    Ok(Json(traderview_expense::section_6050i::compute(&b)))
}

// ── §6050W payment-settlement-entity 1099-K reporting threshold ──────
// Mounted at /api/calc/section-6050w. Two PSE categories with different
// thresholds: §6050W(d)(1) merchant acquiring entity (Stripe, Square,
// traditional card processors) — NO de minimis; every dollar reportable.
// §6050W(d)(3) Third-Party Settlement Organization (PayPal, Venmo, Cash
// App, Zelle, eBay, Etsy, StubHub, Airbnb) — bouncing-ball threshold.
// OBBBA §70432 (eff. 2025-01-01) RETROACTIVELY restored the original
// $20,000 AND 200 transactions strict-greater-than threshold for 2025+,
// superseding the ARPA $600 nominal and IRS Notice 2024-85 transitional
// $5K/$2,500. Historical years 2022 (ARPA $600), 2023 (delayed to $20K/200
// per Notice 2023-74), 2024 (transitional $5K per Notice 2024-85) pinned
// for pre-2025 accuracy.

async fn section_6050w_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6050w::Section6050WInput>,
) -> Result<Json<traderview_expense::section_6050w::Section6050WResult>, ApiError> {
    if b.gross_amount_cents < 0 {
        return Err(ApiError::BadRequest(
            "gross_amount_cents must be non-negative".into(),
        ));
    }
    if !(1990..=2100).contains(&b.year) {
        return Err(ApiError::BadRequest(
            "year must be in [1990, 2100]".into(),
        ));
    }
    Ok(Json(traderview_expense::section_6050w::compute(&b)))
}

// ── §6651 failure-to-file / failure-to-pay penalty ──────────────────
// Mounted at /api/calc/section-6651. §6651(a)(1) FTF 5%/month / 25%
// max; §6651(a)(2) FTP 0.5%/month / 25% max; §6651(c)(1) FTF reduced
// by FTP for overlap months (net 4.5%/month FTF + 0.5%/month FTP =
// 5%/month combined); §6651(f) fraud 15%/month / 75% max; §6651(g)
// minimum-penalty floor for returns > 60 days late (lesser of
// inflation-adjusted amount Rev. Proc. 2025 = $510 or 100% tax);
// §6651(h) installment-rate 0.25%/month when timely-filed-with-
// extension + §6159 agreement; reasonable-cause defense (NOT for fraud).

async fn section_6651_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6651::Section6651Input>,
) -> Result<Json<traderview_expense::section_6651::Section6651Result>, ApiError> {
    if b.tax_required_dollars < 0
        || b.minimum_penalty_inflation_adjusted_dollars < 0
    {
        return Err(ApiError::BadRequest(
            "non-negative dollar inputs required".into(),
        ));
    }
    Ok(Json(traderview_expense::section_6651::compute(&b)))
}

// ── §6654 individual estimated-tax underpayment penalty ─────────────
// Mounted at /api/calc/section-6654. §6654(d)(1)(B)(i) 90%-current-year
// safe harbor; §6654(d)(1)(B)(ii) 100%-prior-year safe harbor;
// §6654(d)(1)(C) 110% high-AGI uplift when prior-year AGI > $150,000
// ($75,000 MFS); §6654(e)(1) $1,000 de minimis exception; required
// installment = (lesser of the two safe-harbor amounts) ÷ 4; underpayment
// per quarter accrues at the § 6621(a)(2) federal-short-term-rate + 3
// percentage points (2026 Q1 = 7%, Q2 = 6%). Out of scope: §6654(d)(2)
// annualized-income exception, §6654(i) farmer/fisherman two-thirds
// rule, §6654(e)(3) retired/disabled waiver.

async fn section_6654_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6654::Section6654Input>,
) -> Result<Json<traderview_expense::section_6654::Section6654Result>, ApiError> {
    for p in &b.quarterly_payments_cents {
        if *p < 0 {
            return Err(ApiError::BadRequest(
                "quarterly_payments_cents must be non-negative".into(),
            ));
        }
    }
    Ok(Json(traderview_expense::section_6654::compute(&b)))
}

// ── §6662 accuracy-related penalty ──────────────────────────────────
// Mounted at /api/calc/section-6662. §6662(a) 20% baseline on
// portion of underpayment attributable to misconduct; §6662(h) 40%
// for gross valuation misstatement (claimed ≥ 200% correct);
// §6662(b) 8 categories (negligence, substantial understatement,
// valuation misstatement, etc.); §6662(d) substantial-understatement
// threshold (greater of 10% of correct tax or $5k individual /
// $10k corporate, capped at $10M); §6664(c) reasonable-cause-and-
// good-faith defense (UNAVAILABLE for §6662(b)(6) economic substance
// + §6662(b)(7) undisclosed foreign asset); no stacking.

async fn section_6662_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6662::Section6662Input>,
) -> Result<Json<traderview_expense::section_6662::Section6662Result>, ApiError> {
    if b.underpayment_dollars < 0
        || b.correct_tax_required_dollars < 0
        || b.claimed_value_dollars < 0
        || b.correct_value_for_valuation_dollars < 0
    {
        return Err(ApiError::BadRequest(
            "non-negative dollar inputs required".into(),
        ));
    }
    Ok(Json(traderview_expense::section_6662::compute(&b)))
}

// ── §448 small business gross receipts test + cascade exemptions ────
// Mounted at /api/calc/section-448. §448(a) mandatory accrual for
// C-corps / partnerships with C-corp partner; §448(b)(3) small
// business exception when §448(c) 3-year average gross receipts ≤
// inflation-indexed threshold ($25M TCJA base; $30M 2024 / $31M 2025
// / $32M 2026); §448(a)(3) tax shelter disqualification; §448(c)(2)
// §52(a)/(b) aggregation. Cascade exemptions: §263A UNICAP, §471
// inventory, §163(j) business interest, §460 long-term contracts.

async fn section_448_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_448::Section448Input>,
) -> Result<Json<traderview_expense::section_448::Section448Result>, ApiError> {
    if b.gross_receipts_year_minus_1_dollars < 0
        || b.gross_receipts_year_minus_2_dollars < 0
        || b.gross_receipts_year_minus_3_dollars < 0
        || b.aggregated_gross_receipts_year_minus_1_dollars < 0
        || b.aggregated_gross_receipts_year_minus_2_dollars < 0
        || b.aggregated_gross_receipts_year_minus_3_dollars < 0
    {
        return Err(ApiError::BadRequest(
            "all gross receipts inputs must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_448::compute(&b)))
}

// ── §444 fiscal year election ───────────────────────────────────────
// Mounted at /api/calc/section-444. §444(a) election availability for
// partnerships, S-corps, and PSCs; §444(b)(2) 3-month deferral cap
// (only Sept 30 / Oct 31 / Nov 30 fiscal year ends qualify when
// required year is calendar); §7519 required payment for partnerships
// and S-corps (Form 8752, due May 15); §280H deduction limitations
// for PSCs.

async fn section_444_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_444::Section444Input>,
) -> Result<Json<traderview_expense::section_444::Section444Result>, ApiError> {
    if b.net_income_for_election_year_dollars < 0 {
        return Err(ApiError::BadRequest(
            "net_income_for_election_year_dollars must be >= 0".into(),
        ));
    }
    if b.required_tax_year_end_month == 0 || b.required_tax_year_end_month > 12 {
        return Err(ApiError::BadRequest(
            "required_tax_year_end_month must be 1..=12".into(),
        ));
    }
    if b.proposed_fiscal_year_end_month == 0 || b.proposed_fiscal_year_end_month > 12 {
        return Err(ApiError::BadRequest(
            "proposed_fiscal_year_end_month must be 1..=12".into(),
        ));
    }
    Ok(Json(traderview_expense::section_444::compute(&b)))
}

// ── §3406 backup withholding ────────────────────────────────────────
// Mounted at /api/calc/section-3406. §3406(a)(1)(A) TIN-not-furnished
// trigger; §3406(a)(1)(B) IRS-notified-incorrect-TIN trigger (BWH-B
// program, CP 2100 / CP 2100A); §3406(a)(1)(C) notified-payee
// underreporting trigger (BWH-C, interest/dividend only);
// §3406(a)(1)(D) payee-certification-failure trigger; §3406(b)(1)(A)
// 24% rate (4th lowest §1(c) rate, post-TCJA).

async fn section_3406_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_3406::Section3406Input>,
) -> Result<Json<traderview_expense::section_3406::Section3406Result>, ApiError> {
    if b.payment_amount_dollars < 0 {
        return Err(ApiError::BadRequest(
            "payment_amount_dollars must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_3406::compute(&b)))
}

// ── §305 stock dividend distribution classification ─────────────────
// Mounted at /api/calc/section-305. §305(a) general rule excludes
// stock-on-stock distributions from gross income; §305(b) 5
// taxable exceptions (in lieu of money / disproportionate / common-
// and-preferred / on preferred stock / convertible preferred w/o
// safe harbor); §305(c) deemed distributions from capital-structure
// events; §307(a) basis allocation between old and new shares when
// §305(a) applies; §301 distribution treatment when taxable
// (dividend up to E&P + basis recovery + capital gain).

async fn section_305_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_305::Section305Input>,
) -> Result<Json<traderview_expense::section_305::Section305Result>, ApiError> {
    Ok(Json(traderview_expense::section_305::compute(&b)))
}

// ── §331 shareholder gain/loss in corporate complete liquidation ─
// Mounted at /api/calc/section-331. §331(a) treats liquidating
// distribution as in full payment for stock (§1001 exchange);
// §331(b) §301 dividend rules inapplicable; capital character when
// stock is capital asset (§1221); §332 corporate-parent 80%/80%
// (§1504(a)(2)) non-recognition exception + §334(b) carryover
// basis; §334(a) shareholder basis in non-cash property = FMV;
// partial liquidations fall to §302 redemption analysis.

async fn section_331_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_331::Section331Input>,
) -> Result<Json<traderview_expense::section_331::Section331Result>, ApiError> {
    if b.adjusted_basis_in_stock_dollars < 0
        || b.cash_received_dollars < 0
        || b.fmv_non_cash_property_received_dollars < 0
    {
        return Err(ApiError::BadRequest(
            "non-negative dollar inputs required".into(),
        ));
    }
    Ok(Json(traderview_expense::section_331::compute(&b)))
}

// ── §332 complete liquidations of subsidiaries ──────────────────────
// Mounted at /api/calc/section-332. §332(a) parent corporation no
// gain/loss recognition on receipt of property in complete liquidation
// of subsidiary IF 4-prong test satisfied: (1) §332(b)(2) 80% voting
// power AND (2) 80% value (§1504(a)(2) test) AND (3) continuous 80%
// ownership maintained from plan-adoption date through final
// distribution AND (4) complete liquidation (all property distributed,
// all stock cancelled). §337(a) parallel subsidiary non-recognition.
// §334(b)(1) parent takes carryover basis (NOT FMV). Failing any
// prong falls to §331/§336 FMV recognition.

async fn section_332_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_332::Section332Input>,
) -> Result<Json<traderview_expense::section_332::Section332Result>, ApiError> {
    if b.fmv_of_property_distributed_cents < 0
        || b.subsidiary_adjusted_basis_cents < 0
        || b.parent_basis_in_subsidiary_stock_cents < 0
    {
        return Err(ApiError::BadRequest(
            "non-negative cents inputs required".into(),
        ));
    }
    if b.voting_power_owned_bp > 10_000 || b.value_owned_bp > 10_000 {
        return Err(ApiError::BadRequest(
            "voting_power_owned_bp and value_owned_bp must be ≤ 10000 (100%)".into(),
        ));
    }
    Ok(Json(traderview_expense::section_332::compute(&b)))
}

// ── §1234A character of gain/loss on right termination ─────────────
// Mounted at /api/calc/section-1234a. §1234A(1) treats gain/loss on
// cancellation, lapse, expiration, or other termination of a right or
// obligation with respect to property that is (or would be on
// acquisition) a capital asset as gain/loss from the sale of a
// capital asset; holding period of the RIGHT governs §1222 character.
// §1234A(2) routes character of terminated §1256 contracts to the
// §1256(a)(3) 60/40 split, ignoring holding period. §1234A excludes
// securities futures contracts — §1234B governs those. Ordinary
// underlying property is OUTSIDE §1234A scope (§ 165 / § 1231 govern).

async fn section_1234a_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1234a::Section1234AInput>,
) -> Result<Json<traderview_expense::section_1234a::Section1234AResult>, ApiError> {
    Ok(Json(traderview_expense::section_1234a::compute(&b)))
}

// ── §1234B character of gain/loss on securities futures contracts ─
// Mounted at /api/calc/section-1234b. §1234B(a) character mirrors
// underlying property (capital underlying → capital character;
// ordinary underlying → ordinary character). §1234B(b) — gain/loss
// on sale/exchange/termination of a securities futures contract TO
// SELL property is treated as SHORT-TERM CAPITAL regardless of
// holding period (parallels § 1233 short-sale rule). §1256(b)(1)(E)
// override — DEALER securities futures contracts are § 1256 contracts
// and get the § 1256(a)(3) 60/40 split, BEFORE § 1234B engages.
// §1234B(c) defines SFC via Securities Exchange Act § 3(a)(55)(A).
// §1234B(d) — SFC is not a commodity futures contract.

async fn section_1234b_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1234b::Section1234BInput>,
) -> Result<Json<traderview_expense::section_1234b::Section1234BResult>, ApiError> {
    Ok(Json(traderview_expense::section_1234b::compute(&b)))
}

// ── §263(g) capitalization of interest + carrying charges on straddles ─
// Mounted at /api/calc/section-263g. §263(g)(1) general rule disallows
// current deduction for interest + carrying charges allocable to
// personal property that is part of a §1092(c) straddle; disallowed
// amount is chargeable to the capital account (basis) of the straddle
// property — timing-only, not permanent. §263(g)(2) defines interest
// and carrying charges as the EXCESS of (A) interest-on-indebtedness +
// other carrying costs (storage / insurance / transport) OVER (B)
// interest received + ordinary income from property + dividends net of
// §243 DRD + security loan fee payments includible in gross income.
// §263(g)(3) exempts §1256(e) hedging transactions (bona fide hedge of
// inventory / ordinary obligations / borrowings; identified before
// close of day entered into). §263(g)(4) provides coordination rules
// with §263(h) short-sale + §1277/§1282 market-discount/OID rules.

async fn section_263g_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_263g::Section263GInput>,
) -> Result<Json<traderview_expense::section_263g::Section263GResult>, ApiError> {
    if b.interest_on_indebtedness_cents < 0
        || b.carrying_costs_cents < 0
        || b.interest_received_cents < 0
        || b.ordinary_income_from_property_cents < 0
        || b.dividends_received_cents < 0
        || b.dividend_received_deduction_cents < 0
        || b.security_loan_fees_received_cents < 0
    {
        return Err(ApiError::BadRequest(
            "non-negative cents inputs required".into(),
        ));
    }
    Ok(Json(traderview_expense::section_263g::compute(&b)))
}

// ── §1276 market-discount-bond ordinary-income recharacterization ─
// Mounted at /api/calc/section-1276. §1276(a)(1) general rule: gain
// on disposition of any market discount bond is ordinary income up
// to accrued market discount. §1276(a)(2): non-sale dispositions
// (gift, distribution) treated as realizing FMV. §1276(a)(3): partial
// principal payment ordinary up to accrued. §1276(a)(4): amount
// treated as INTEREST for purposes of the Code (with carve-outs for
// §§ 103, 871(a), 881, 1441, 1442, 6049). §1276(b)(1) ratable accrual
// default = market_discount × (days_held / total_days). §1276(b)(2)
// constant-yield election uses §1272(a) OID formula (caller supplies
// computed accrual). §1278(a)(2)(A) defines market discount = stated
// redemption − basis (clamped at zero). §1278(b) current-inclusion
// election lets taxpayer recognize annually; prior-year accrual
// subtracts from §1276 disposition cap to avoid double inclusion.

async fn section_1276_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1276::Section1276Input>,
) -> Result<Json<traderview_expense::section_1276::Section1276Result>, ApiError> {
    if b.purchase_price_cents < 0
        || b.stated_redemption_at_maturity_cents < 0
        || b.realized_amount_cents < 0
        || b.constant_yield_accrual_cents < 0
        || b.prior_years_accrual_already_taxed_cents < 0
    {
        return Err(ApiError::BadRequest(
            "non-negative cents inputs required".into(),
        ));
    }
    Ok(Json(traderview_expense::section_1276::compute(&b)))
}

// ── §1277 deferral of interest deduction on market-discount bonds ─
// Mounted at /api/calc/section-1277. Direct companion to §1276.
// §1277(a) general rule: net direct interest expense (NDIE) on
// indebtedness to purchase/carry a market discount bond is deductible
// in the current year ONLY to the extent it exceeds the portion of
// market discount allocable to the days during the taxable year on
// which the taxpayer held the bond. §1277(b)(1) net-interest-income
// carryover recovery: disallowed amount recovered in later year up
// to net interest income on that bond. §1277(b)(2) disposition
// terminal recovery: all remaining deferred amount recovered in the
// disposition year. §1277(c) NDIE definition = excess of interest
// paid/accrued on indebtedness OVER interest (incl. OID under
// §1272(a)) includible in gross income on the bond. §1278(b)
// current-inclusion election exempts taxpayer from §1277 deferral
// because matching market-discount income is recognized currently.
// §1277(d) was struck out entirely by Pub. L. 103-66 (1993).

async fn section_1277_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1277::Section1277Input>,
) -> Result<Json<traderview_expense::section_1277::Section1277Result>, ApiError> {
    if b.interest_on_indebtedness_cents < 0
        || b.interest_income_on_bond_cents < 0
        || b.accrued_market_discount_for_year_cents < 0
        || b.net_interest_income_for_year_cents < 0
        || b.prior_year_disallowed_carryover_cents < 0
    {
        return Err(ApiError::BadRequest(
            "non-negative cents inputs required".into(),
        ));
    }
    Ok(Json(traderview_expense::section_1277::compute(&b)))
}

// ── §1278 market-discount-bond definitions + § 1278(b) election ───
// Mounted at /api/calc/section-1278. The definitional + election
// module that both §1276 (ordinary-income recharacterization) and
// §1277 (interest-deduction deferral) cross-reference. §1278(a)(1)
// market discount bond definition with carve-outs for U.S. savings
// bonds, short-term obligations (≤ 1 year to maturity), and §453B
// installment obligations. §1278(a)(2)(A) market discount = stated
// redemption price at maturity − basis at acquisition. §1278(a)(2)(B)
// OID bonds use REVISED ISSUE PRICE (acquisition-date OID-adjusted
// basis) in lieu of stated redemption price. §1278(a)(2)(C) DE
// MINIMIS rule — raw discount STRICTLY LESS THAN ¼ of 1% of stated
// redemption × complete years to maturity is treated as ZERO.
// §1278(b)(1) current-inclusion election — switches off §1276
// disposition recharacterization AND §1277 interest deferral.
// §1278(b)(2) election scope to all market discount bonds acquired
// during or after year of election. §1278(b)(3) election IRREVOCABLE
// absent Secretary's consent.

async fn section_1278_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1278::Section1278Input>,
) -> Result<Json<traderview_expense::section_1278::Section1278Result>, ApiError> {
    if b.stated_redemption_price_cents < 0
        || b.revised_issue_price_cents < 0
        || b.purchase_price_cents < 0
    {
        return Err(ApiError::BadRequest(
            "non-negative cents inputs required".into(),
        ));
    }
    Ok(Json(traderview_expense::section_1278::compute(&b)))
}

// ── §1271 retirement of debt instruments treated as sale/exchange ─
// Mounted at /api/calc/section-1271. §1271(a)(1) general rule
// amounts received on retirement of any debt instrument considered
// amounts received in exchange therefor — default capital character.
// §1271(a)(2) intent-to-call OID instruments — gain up to OID
// (reduced by §1271(c) prior-year inclusions) recharacterized as
// ordinary income; carve-outs for tax-exempt obligations and
// premium-buyers. §1271(a)(3) short-term government obligations
// ≤1 year to maturity — gain up to ratable share of acquisition
// discount recharacterized as ordinary. §1271(a)(4) short-term
// nongovernment obligations — gain up to ratable share of OID
// recharacterized as ordinary. §1271(b) natural-person issuer
// exception — § 1271 does not apply to obligations issued by
// natural persons before June 9, 1997. §1271(c) no double
// inclusion — § 1271, § 1272, and § 1286 do not require inclusion
// of amounts previously includible in gross income.

async fn section_1271_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1271::Section1271Input>,
) -> Result<Json<traderview_expense::section_1271::Section1271Result>, ApiError> {
    if b.purchase_price_cents < 0
        || b.redemption_amount_cents < 0
        || b.original_issue_discount_cents < 0
        || b.oid_previously_included_cents < 0
        || b.acquisition_discount_cents < 0
        || b.ratable_short_term_accrual_cents < 0
    {
        return Err(ApiError::BadRequest(
            "non-negative cents inputs required".into(),
        ));
    }
    Ok(Json(traderview_expense::section_1271::compute(&b)))
}

// ── §1272 current inclusion of original issue discount (OID) ──────
// Mounted at /api/calc/section-1272. §1272(a)(1) general rule —
// holder must include sum of daily portions of OID in gross income
// each year regardless of cash received (phantom income). § 1272(a)(2)
// carve-outs: (A) tax-exempt obligations; (B) U.S. savings bonds;
// (C) short-term obligations ≤ 1 year (§ 1281 + § 1283 govern);
// (D) natural-person small loans ≤ $10,000 not for tax avoidance.
// § 1272(a)(3) daily-portion ratable allocation by days held.
// § 1272(a)(6) prepayable mortgage-backed / REMIC special PV
// methodology. § 1272(a)(7) acquisition-premium reduction —
// secondary-market basis above adjusted issue price reduces daily-
// portion by fraction (basis − AIP) / (stated redemption − AIP).
// Companion to § 1271 (retirement; § 1271(c) no double inclusion)
// and § 1273 (OID definition).

async fn section_1272_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1272::Section1272Input>,
) -> Result<Json<traderview_expense::section_1272::Section1272Result>, ApiError> {
    if b.adjusted_issue_price_start_of_year_cents < 0
        || b.adjusted_issue_price_end_of_year_cents < 0
        || b.acquisition_premium_cents < 0
        || b.stated_redemption_minus_aip_at_acquisition_cents < 0
    {
        return Err(ApiError::BadRequest(
            "non-negative cents inputs required".into(),
        ));
    }
    Ok(Json(traderview_expense::section_1272::compute(&b)))
}

// ── §1273 OID definition + issue price determination ─────────────
// Mounted at /api/calc/section-1273. Definitional anchor for the
// OID cluster. § 1273(a)(1) OID = excess of stated redemption price
// at maturity over issue price. § 1273(a)(2) SRPM = amount fixed by
// last modification of purchase agreement. § 1273(a)(3) DE MINIMIS
// — raw OID strictly less than ¼ of 1% × SRPM × complete years to
// maturity treated as ZERO (same factor as § 1278(a)(2)(C) market
// discount). § 1273(b)(1) publicly offered cash issue = initial
// offering price to public. § 1273(b)(2) non-public cash = price
// paid by first buyer. § 1273(b)(3) traded debt (issued for property
// where debt OR property is publicly traded) = FMV of debt
// instrument. § 1273(b)(4) residual case = SRPM minus OID (caller-
// supplied OID typically from § 1274 AFR imputation). § 1273(b)(5)
// "property" includes services and right to use property.

async fn section_1273_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1273::Section1273Input>,
) -> Result<Json<traderview_expense::section_1273::Section1273Result>, ApiError> {
    if b.stated_redemption_price_at_maturity_cents < 0
        || b.initial_public_offering_price_cents < 0
        || b.first_buyer_price_cents < 0
        || b.fmv_of_debt_instrument_cents < 0
        || b.residual_oid_amount_cents < 0
    {
        return Err(ApiError::BadRequest(
            "non-negative cents inputs required".into(),
        ));
    }
    Ok(Json(traderview_expense::section_1273::compute(&b)))
}

// ── §1281 current inclusion of acquisition discount on short-term ─
// Mounted at /api/calc/section-1281. Bookend to OID cluster: § 1272
// governs long-term OID; § 1281 governs short-term obligations
// (≤ 1 year). § 1272(a)(2)(C) and § 1271(a)(3)/(a)(4) cross-reference
// § 1281. Critical distinction: § 1281 applies ONLY to specific
// holder categories — § 1281(b)(1)(A) accrual-method taxpayers +
// (B) dealers + (C) banks (§ 581) + (D) RICs + common trust funds +
// (E) § 1256(e)(2) hedging-transaction-identified + (F) stripped-
// bond strippers + § 1281(b)(2) pass-thru entities. Cash-method
// individual investors are OUTSIDE § 1281 scope and defer to
// § 1271(a)(3)/(a)(4) ratable accrual at disposition. § 1281(c)
// cross-references § 1283(c) for nongovernmental obligation OID-
// only limitation. § 1283(a)(1) short-term obligation definition
// (≤ 1 year to maturity); § 1283(a)(2) acquisition discount = SRPM
// minus basis at acquisition.

async fn section_1281_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1281::Section1281Input>,
) -> Result<Json<traderview_expense::section_1281::Section1281Result>, ApiError> {
    Ok(Json(traderview_expense::section_1281::compute(&b)))
}

// ── §1283 short-term obligation + acquisition discount definitions ─
// Mounted at /api/calc/section-1283. Definitional anchor for short-
// term obligation cluster. § 1281 (current inclusion) and § 1282
// (interest deduction deferral) both cross-reference § 1283 for
// underlying terms. § 1283(a)(1) defines short-term obligation as
// any bond/debenture/note/certificate with fixed maturity ≤ 1 year
// from date of issue (with tax-exempt carve-out). § 1283(a)(2)
// acquisition discount = SRPM minus basis. § 1283(b)(1) daily-
// portion ratable accrual = total discount divided by days from
// acquisition to maturity inclusive. § 1283(b)(2) constant-yield
// election parallels § 1272(a)(3) OID rules. § 1283(c) nongovern-
// mental obligations substitute OID for acquisition discount.
// § 1283(d) basis increased by § 1281 prior-year inclusion.

async fn section_1283_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1283::Section1283Input>,
) -> Result<Json<traderview_expense::section_1283::Section1283Result>, ApiError> {
    if b.stated_redemption_price_at_maturity_cents < 0
        || b.basis_at_acquisition_cents < 0
        || b.oid_amount_for_nongovernmental_cents < 0
    {
        return Err(ApiError::BadRequest(
            "non-negative cents inputs required".into(),
        ));
    }
    Ok(Json(traderview_expense::section_1283::compute(&b)))
}

// ── §1282 short-term obligation interest-deduction deferral ──────
// Mounted at /api/calc/section-1282. Direct short-term-obligation
// companion to section_1277 (long-term market-discount interest
// deferral parallel). § 1282(a) general rule defers net direct
// interest expense (NDIE) on indebtedness incurred to purchase or
// carry short-term obligation to extent of daily portions of
// acquisition discount allocable to days held in year. § 1282(b)(1)
// exception for § 1281 holders (already including discount
// currently — accrual + dealer + bank + RIC + hedging + stripper +
// pass-thru). § 1282(b)(2) election to apply § 1281 to all
// short-term obligations triggers § 1282(b) exception. § 1282(c)
// cross-reference to § 1277 long-term rules. § 1282(d) § 1283(c)
// nongovernmental OID substitution. Companion to section_1281
// (current inclusion mandate) + section_1283 (definitions).

async fn section_1282_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1282::Section1282Input>,
) -> Result<Json<traderview_expense::section_1282::Section1282Result>, ApiError> {
    if b.interest_expense_on_indebtedness_cents < 0
        || b.interest_income_includible_cents < 0
    {
        return Err(ApiError::BadRequest(
            "non-negative cents inputs required".into(),
        ));
    }
    Ok(Json(traderview_expense::section_1282::compute(&b)))
}

// ── §7704 publicly traded partnership corporate treatment ────────
// Mounted at /api/calc/section-7704. Trader-critical for any
// investor holding master limited partnerships (MLPs) or PTPs.
// §7704(a) general rule treats PTP as CORPORATION losing pass-
// through status — unless §7704(c) exception applies. §7704(b)
// PTP definition has two prongs: (1) interests traded on
// established securities market OR (2) readily tradable on
// secondary market. §7704(c)(1) requires continuous compliance
// with 90% test every taxable year beginning after 1987-12-31.
// §7704(c)(2) 90% qualifying-income test. §7704(d)(1) seven
// qualifying-income categories: (A) interest + (B) dividends +
// (C) real property rents + (D) gain from real property + (E)
// mineral/natural-resource income + (F) qualifying capital asset
// gain + (G) commodities income. §7704(e) inadvertent-termination
// relief requires all three prongs: (i) inadvertent failure +
// (ii) corrective steps within reasonable time + (iii) agreement
// to required adjustments.

async fn section_7704_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7704::Section7704Input>,
) -> Result<Json<traderview_expense::section_7704::Section7704Result>, ApiError> {
    if b.gross_income_total_cents < 0 || b.qualifying_income_cents < 0 {
        return Err(ApiError::BadRequest(
            "non-negative cents inputs required".into(),
        ));
    }
    Ok(Json(traderview_expense::section_7704::compute(&b)))
}

// ── §6045B issuer Form 8937 organizational action basis reporting ─
// Mounted at /api/calc/section-6045b. § 6045B requires issuers of
// specified securities to report organizational actions affecting
// basis to the IRS via Form 8937 within fixed deadline. § 6045B(a)
// return must describe the action AND include quantitative effect
// on basis. § 6045B(b) deadline = earlier of (1) 45 days after
// action OR (2) January 15 of year following calendar year of
// action. § 6045B(c) issuer must furnish written statement to
// nominees and holders by January 15 of following year. § 6045B(d)
// specified security defined by § 6045(g)(3). § 6045B(e) PUBLIC
// WEBSITE WAIVER via Treas. Reg. § 1.6045B-1(a)(3) — issuer is
// deemed to satisfy IRS filing duty by posting completed signed
// Form 8937 on public website for at least 10 YEARS. Companion to
// section_6045 (broker Form 1099-B downstream reporting).

async fn section_6045b_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6045b::Section6045BInput>,
) -> Result<Json<traderview_expense::section_6045b::Section6045BResult>, ApiError> {
    if b.days_since_action > 100_000 || b.website_posting_duration_years > 100 {
        return Err(ApiError::BadRequest(
            "counters look invalid (>threshold)".into(),
        ));
    }
    Ok(Json(traderview_expense::section_6045b::check(&b)))
}

// ── §6045A broker-to-broker custody transfer statement ────────────
// Mounted at /api/calc/section-6045a. § 6045A requires the
// transferring broker (or other applicable person) to furnish a
// written information statement to the receiving broker within 15
// days of the transfer. Receiving broker uses statement to populate
// Form 1099-B basis reporting under § 6045 on eventual sale.
// § 6045A(a) general rule + § 6045A(b)(1) broker definition via
// § 6045(c)(1) + § 6045A(b)(2) other person per Secretary + § 6045A(c)
// 15-day deadline + § 6045A(d) digital-asset transfer return regime
// added by Infrastructure Investment and Jobs Act of 2021 Pub. L.
// 117-58 § 80603 effective post-2025-12-31 — broker transferring
// digital asset to non-broker account must make return showing
// transfer info. Treas. Reg. § 1.6045A-1 statement content: basis,
// acquisition date, wash-sale flag per § 1091. Companion to
// section_6045 (downstream Form 1099-B) and section_6045b (upstream
// issuer Form 8937).

async fn section_6045a_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6045a::Section6045AInput>,
) -> Result<Json<traderview_expense::section_6045a::Section6045AResult>, ApiError> {
    if b.days_since_transfer > 100_000 {
        return Err(ApiError::BadRequest(
            "days_since_transfer looks invalid (>100000)".into(),
        ));
    }
    Ok(Json(traderview_expense::section_6045a::check(&b)))
}

// ── §1297 PFIC classification income + asset tests ───────────────
// Mounted at /api/calc/section-1297. Trader-critical for any
// investor holding foreign mutual funds + foreign ETFs + foreign
// stock. § 1297(a)(1) 75% income test — 75% or more of gross income
// is passive income. § 1297(a)(2) 50% asset test — 50% or more of
// average assets produce passive income. EITHER test triggers PFIC
// status which subjects shareholder to § 1291 punitive excess-
// distribution + interest-charge regime unless QEF (§ 1295) or
// mark-to-market (§ 1296) election made. § 1297(b)(1) passive
// income = § 954(c) foreign personal holding company income.
// § 1297(b)(2) exceptions — (A) active banking + (B) active
// insurance + (C) related-party allocable income. § 1297(c) 25%
// look-through rule — foreign corp owning 25%+ of subsidiary by
// value is treated as holding proportionate share of subsidiary's
// assets and income. § 1297(d) once-a-PFIC qualified portion
// exception with § 1298(b)(1) purging election.

async fn section_1297_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1297::Section1297Input>,
) -> Result<Json<traderview_expense::section_1297::Section1297Result>, ApiError> {
    if b.gross_income_total_cents < 0
        || b.passive_income_cents < 0
        || b.avg_total_assets_cents < 0
    {
        return Err(ApiError::BadRequest(
            "non-negative cents inputs required".into(),
        ));
    }
    if b.avg_passive_assets_bp > 10_000 {
        return Err(ApiError::BadRequest(
            "avg_passive_assets_bp must be ≤ 10000 (100%)".into(),
        ));
    }
    Ok(Json(traderview_expense::section_1297::compute(&b)))
}

// ── §1298 PFIC attribution + special rules + annual reporting ────
// Mounted at /api/calc/section-1298. Direct companion to section_1297
// (which cross-references § 1298(b)(1) purging election in § 1297(d)).
// § 1298(a)(2) 50% value corporation attribution; § 1298(a)(3)
// partnership/estate/trust proportionate attribution; § 1298(a)(4)
// options attribution per regulations; § 1298(b)(1) purging election
// under § 1291(d)(2) — pay current tax on accumulated PFIC gain to
// shed PFIC taint going forward; § 1298(b)(6) PLEDGE-AS-SECURITY
// DEEMED DISPOSITION — using PFIC stock as security for loan
// triggers deemed sale under § 1291; § 1298(f) annual Form 8621
// reporting required for every U.S. PFIC shareholder.

async fn section_1298_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1298::Section1298Input>,
) -> Result<Json<traderview_expense::section_1298::Section1298Result>, ApiError> {
    if b.pfic_stock_value_cents < 0 {
        return Err(ApiError::BadRequest(
            "non-negative cents inputs required".into(),
        ));
    }
    if b.upstream_value_ownership_bp > 10_000 {
        return Err(ApiError::BadRequest(
            "upstream_value_ownership_bp must be ≤ 10000 (100%)".into(),
        ));
    }
    Ok(Json(traderview_expense::section_1298::compute(&b)))
}

// ── §6038D Form 8938 foreign financial asset reporting ───────────
// Mounted at /api/calc/section-6038d. Trader-critical for anyone
// with offshore brokerage accounts + foreign mutual fund holdings
// (PFICs covered by § 1297/1298) + foreign retirement accounts +
// foreign-issued bonds + interests in foreign entities. § 6038D(a)
// requires individuals to attach Form 8938 if aggregate value of
// specified foreign financial assets exceeds threshold under
// § 6038D(b). Treas. Reg. § 1.6038D-2 tiers thresholds by filing
// status + residency. § 6038D(c) required information includes
// institution name + address + account number + issuer info +
// maximum value of asset during taxable year. § 6038D(d) $10,000
// initial penalty per failure to disclose. § 6038D(e) continuing
// $10,000 per 30-day period after 90-day IRS notice grace, capped
// at $50,000. § 6038D(g) reasonable-cause-AND-not-willful-neglect
// exception. Distinct from FinCEN Form 114 (FBAR) Bank Secrecy Act
// filing under 31 U.S.C. § 5314 with separate threshold and
// penalty regime.

async fn section_6038d_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6038d::Section6038DInput>,
) -> Result<Json<traderview_expense::section_6038d::Section6038DResult>, ApiError> {
    if b.aggregate_value_year_end_cents < 0
        || b.aggregate_value_any_time_during_year_cents < 0
    {
        return Err(ApiError::BadRequest(
            "non-negative cents inputs required".into(),
        ));
    }
    if b.days_since_irs_notice > 100_000 {
        return Err(ApiError::BadRequest(
            "days_since_irs_notice looks invalid (>100000)".into(),
        ));
    }
    Ok(Json(traderview_expense::section_6038d::compute(&b)))
}

// ── § 6011 reportable transaction disclosure (Form 8886) ──────────
// Mounted at /api/calc/section-6011. Treas. Reg. § 1.6011-4(b)
// designates five reportable-transaction categories: listed
// transactions (b)(2); confidential transactions (b)(3) with
// $250K corporate / $50K noncorporate fee thresholds; transactions
// with contractual protection (b)(4); loss transactions (b)(5)
// with $2M individual / $10M entity single-year thresholds;
// transactions of interest (b)(6). Failure to disclose triggers
// § 6707A penalty — 75% of tax reduction, floored at $5K
// individual / $10K entity, capped at $100K individual / $200K
// entity for listed transactions. Companion to § 6111 material
// advisor disclosure (Form 8918), § 6112 advisor list maintenance,
// and § 6662A 20%/30% reportable-transaction-understatement penalty.

async fn section_6011_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6011::Section6011Input>,
) -> Result<Json<traderview_expense::section_6011::Section6011Result>, ApiError> {
    if b.fee_paid_to_advisor_cents < 0
        || b.single_year_loss_claimed_cents < 0
        || b.multi_year_loss_total_cents < 0
    {
        return Err(ApiError::BadRequest(
            "non-negative cents inputs required".into(),
        ));
    }
    Ok(Json(traderview_expense::section_6011::compute(&b)))
}

// ── § 6111 material advisor disclosure (Form 8918) ──────────────────
// Mounted at /api/calc/section-6111. Direct sibling to § 6011
// (taxpayer-side Form 8886). § 6111(b)(1) two-prong test: (A)
// provided material aid/assistance/advice for reportable
// transaction AND (B) gross income exceeds threshold ($50K
// natural-person / $250K other under Treas. Reg.
// § 301.6111-3(b)(3)). Filing deadline: last day of month
// following calendar-quarter-end (§ 301.6111-3(e)). § 6707
// penalties: $50K non-listed; greater of $200K or 50% of gross
// income for listed transactions, reduced to $50K for
// unintentional failures per § 6707(b)(1) flush. Statute of
// limitations: 3 years from Form 8918 filing; unlimited if no
// return filed. Companion to § 6112 (advisor list maintenance),
// § 6707A (taxpayer penalty), § 6662A (reportable-transaction-
// understatement accuracy penalty on underlying tax).

async fn section_6111_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6111::Section6111Input>,
) -> Result<Json<traderview_expense::section_6111::Section6111Result>, ApiError> {
    if b.gross_income_from_transaction_cents < 0 {
        return Err(ApiError::BadRequest(
            "gross_income_from_transaction_cents must be non-negative".into(),
        ));
    }
    if b.days_late_after_quarter_end < 0 || b.days_late_after_quarter_end > 100_000 {
        return Err(ApiError::BadRequest(
            "days_late_after_quarter_end out of range".into(),
        ));
    }
    Ok(Json(traderview_expense::section_6111::compute(&b)))
}

// ── § 6112 material advisor list maintenance ────────────────────────
// Mounted at /api/calc/section-6112. Sixth member of the
// disclosure-regime cluster (§ 6011 + § 6111 + § 6707 + § 6707A
// + § 6662A + § 6112). § 6112(a) requires material advisors to
// maintain a list of all persons advised on a reportable
// transaction; § 6112(b)(1)(A) requires production within 20
// BUSINESS DAYS of IRS written request. Treas. Reg.
// § 301.6112-1(b)(2) defines three required list components:
// itemized statement + detailed transaction description + copies
// of documents. § 6708(a) imposes $10,000-per-day penalty for
// each day after the 20-business-day deadline; reasonable cause
// excused on day-by-day basis per § 301.6708-1(c). The only
// per-day-accruing penalty in the disclosure-regime cluster.

async fn section_6112_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6112::Section6112Input>,
) -> Result<Json<traderview_expense::section_6112::Section6112Result>, ApiError> {
    if b.business_days_since_request < 0
        || b.business_days_since_request > 100_000
    {
        return Err(ApiError::BadRequest(
            "business_days_since_request out of range".into(),
        ));
    }
    if b.days_with_reasonable_cause < 0
        || b.days_with_reasonable_cause > 100_000
    {
        return Err(ApiError::BadRequest(
            "days_with_reasonable_cause out of range".into(),
        ));
    }
    Ok(Json(traderview_expense::section_6112::compute(&b)))
}

// ── § 6662A reportable-transaction-understatement penalty ──────────
// Mounted at /api/calc/section-6662a. Direct sibling to § 6011
// (taxpayer Form 8886), § 6111 (advisor Form 8918), § 6707
// (advisor penalty), § 6707A (taxpayer disclosure penalty).
// § 6662A taxes the SUBSTANTIVE tax position taken on the return
// when a reportable transaction is involved — not just the
// disclosure failure standalone. § 6662A(a) 20% baseline rate;
// § 6662A(c) 30% enhanced rate when transaction was not
// adequately disclosed under § 6011 regulations. § 6662A(b)(1)
// understatement = (income increase × highest tax rate) +
// credit decrease. § 6664(d) reasonable-cause exception requires
// ALL THREE prongs: (A) adequate disclosure; (B) substantial
// authority; (C) more-likely-than-not belief. § 6662A(e)(2)(A)
// coordination prevents stacking with § 6662 on the same
// understatement.

async fn section_6662a_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6662a::Section6662AInput>,
) -> Result<Json<traderview_expense::section_6662a::Section6662AResult>, ApiError> {
    if b.highest_tax_rate_bps > 10_000 || b.highest_tax_rate_bps < -10_000 {
        return Err(ApiError::BadRequest(
            "highest_tax_rate_bps out of range".into(),
        ));
    }
    Ok(Json(traderview_expense::section_6662a::compute(&b)))
}

// ── § 6694 tax return preparer penalties ────────────────────────────
// Mounted at /api/calc/section-6694. § 6694(a) unreasonable-
// position penalty: greater of $1,000 OR 50% of preparer fee.
// Three trigger paths under § 6694(a)(2): (A) undisclosed +
// no substantial authority; (B) disclosed but no reasonable
// basis; (C) tax shelter / § 6662A reportable transaction
// without more-likely-than-not standard. § 6694(a)(3) reasonable-
// cause + good-faith exception. § 6694(b) willful or reckless
// conduct: greater of $5,000 OR 75% of fee; no reasonable-cause
// exception. § 6694(b)(3) no-stacking — (b) replaces (a) when
// both trigger. Sibling preparer + promoter penalty cluster:
// § 6695 + § 6700 + § 6701. Taxpayer-side companions: § 6662
// + § 6662A + § 6707A.

async fn section_6694_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6694::Section6694Input>,
) -> Result<Json<traderview_expense::section_6694::Section6694Result>, ApiError> {
    if b.preparer_fee_cents < 0 || b.preparer_fee_cents > 1_000_000_000_000 {
        return Err(ApiError::BadRequest(
            "preparer_fee_cents out of range".into(),
        ));
    }
    Ok(Json(traderview_expense::section_6694::compute(&b)))
}

// ── § 6695 preparer information return penalties ───────────────────
// Mounted at /api/calc/section-6695. Direct sibling to § 6694
// (substantive position penalty) — covers PROCEDURAL failures
// by the preparer. Five per-failure subsections (a)-(e) at $60
// each (2025; max $31,500/year per subsection): copy to taxpayer,
// signature, PTIN, retention, info return. Higher-tier
// per-failure $635: § 6695(f) refund check negotiation; § 6695(g)
// due diligence on credits (EITC, CTC/ACTC/ODC, AOTC, HOH) —
// max combined per return = $2,540 ($635 × 4). Treas. Reg.
// § 1.6695-2 requires Form 8867 + worksheet + knowledge
// requirement + 3-year retention. 2025 amounts per Rev. Proc.
// 2024-40 inflation adjustments.

async fn section_6695_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6695::Section6695Input>,
) -> Result<Json<traderview_expense::section_6695::Section6695Result>, ApiError> {
    if b.per_failure_penalty_cents < 0 || b.per_failure_penalty_cents > 1_000_000 {
        return Err(ApiError::BadRequest(
            "per_failure_penalty_cents out of range".into(),
        ));
    }
    if b.annual_max_cap_cents < 0 || b.annual_max_cap_cents > 100_000_000_000 {
        return Err(ApiError::BadRequest(
            "annual_max_cap_cents out of range".into(),
        ));
    }
    if b.higher_tier_penalty_cents < 0 || b.higher_tier_penalty_cents > 10_000_000 {
        return Err(ApiError::BadRequest(
            "higher_tier_penalty_cents out of range".into(),
        ));
    }
    Ok(Json(traderview_expense::section_6695::compute(&b)))
}

// ── § 6700 promoter penalties for abusive tax shelter promotion ────
// Mounted at /api/calc/section-6700. Third member of the preparer
// + promoter penalty cluster (after § 6694 + § 6695). Two-prong
// structure: § 6700(a)(1) promoter status (organizes/sells plan
// or arrangement) + § 6700(a)(2)(A) false/fraudulent statement
// with scienter (50% gross income penalty per AJCA 2004), or
// § 6700(a)(2)(B) + § 6700(b)(1) gross valuation overstatement
// exceeding 200% threshold with direct relationship to
// deduction/credit ($1,000 floor or lesser-of-gross-income).
// Penalty applies REGARDLESS of participant reliance or actual
// underreporting. Sibling cluster: § 6694 + § 6695 + § 6701
// (aiding/abetting) + § 7408 (injunction remedy). Effective
// since January 1, 1990; substantially amended by AJCA 2004
// Pub. L. 108-357 § 818.

async fn section_6700_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6700::Section6700Input>,
) -> Result<Json<traderview_expense::section_6700::Section6700Result>, ApiError> {
    if b.gross_income_from_activity_cents < 0
        || b.gross_income_from_activity_cents > 1_000_000_000_000
    {
        return Err(ApiError::BadRequest(
            "gross_income_from_activity_cents out of range".into(),
        ));
    }
    if b.stated_value_cents > 1_000_000_000_000 || b.correct_value_cents > 1_000_000_000_000 {
        return Err(ApiError::BadRequest(
            "stated_value_cents or correct_value_cents out of range".into(),
        ));
    }
    Ok(Json(traderview_expense::section_6700::compute(&b)))
}

// ── § 6701 aiding and abetting understatement of tax liability ─────
// Mounted at /api/calc/section-6701. Fourth and final member of
// the preparer + promoter penalty cluster (§ 6694 + § 6695 +
// § 6700 + § 6701). § 6701 captures the broadest range of
// conduct — any person who aids, assists, procures, or advises
// preparation of a document they KNOW would result in
// understatement of another's tax. Three-element test under
// § 6701(a): (1) aid/assist/procure/advise; (2) material-matter
// knowledge; (3) understatement-knowledge. § 6701(b)(1)
// penalties: $1,000 non-corporate / $10,000 corporate per
// document. § 6701(b)(2) one-per-taxpayer-per-period limit.
// § 6701(f) coordination — § 6701 supersedes § 6694(a)/(b) on
// same document. § 7408 injunction remedy available.

async fn section_6701_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6701::Section6701Input>,
) -> Result<Json<traderview_expense::section_6701::Section6701Result>, ApiError> {
    if b.number_of_documents < 0 || b.number_of_documents > 1_000_000 {
        return Err(ApiError::BadRequest(
            "number_of_documents out of range".into(),
        ));
    }
    if b.number_of_distinct_taxpayers < 0
        || b.number_of_distinct_taxpayers > 1_000_000
    {
        return Err(ApiError::BadRequest(
            "number_of_distinct_taxpayers out of range".into(),
        ));
    }
    Ok(Json(traderview_expense::section_6701::compute(&b)))
}

// ── §336 gain/loss on property distributed in complete liquidation ─
// Mounted at /api/calc/section-336. §336(a) FMV sale treatment of
// distributed property; §336(b) liability ≥ FMV adjustment;
// §336(d)(1) related-party loss disallowance (>50% ownership);
// §336(d)(2) 5-year anti-tax-avoidance built-in loss disallowance;
// §336(d)(3) §332 subsidiary 80%+ parent full nonrecognition.

async fn section_336_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_336::Section336Input>,
) -> Result<Json<traderview_expense::section_336::Section336Result>, ApiError> {
    if b.distributed_property_fmv_dollars < 0
        || b.distributed_property_adjusted_basis_dollars < 0
        || b.built_in_loss_at_contribution_dollars < 0
        || b.liability_amount_on_property_dollars < 0
    {
        return Err(ApiError::BadRequest(
            "all dollar inputs must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_336::compute(&b)))
}

// ── §351 corporate formation non-recognition ────────────────────────
// Mounted at /api/calc/section-351. §351(a) non-recognition when
// transferors meet §368(c) 80% voting + 80% nonvoting control test
// immediately after exchange; §351(b)(1) boot gain recognition;
// §351(b)(2) loss never recognized; §351(d) services exclusion;
// §357(a) liabilities not boot; §357(b) tax-avoidance full-boot;
// §357(c) excess-liability-over-basis gain; §358(a) substituted
// stock basis; §362(a) corp carryover basis + gain.

async fn section_351_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_351::Section351Input>,
) -> Result<Json<traderview_expense::section_351::Section351Result>, ApiError> {
    if b.property_adjusted_basis_dollars < 0
        || b.property_fmv_dollars < 0
        || b.stock_fmv_received_dollars < 0
        || b.boot_received_dollars < 0
        || b.liabilities_assumed_by_corp_dollars < 0
        || b.control_group_voting_pct_bp > 10_000
        || b.control_group_nonvoting_pct_bp > 10_000
    {
        return Err(ApiError::BadRequest(
            "non-negative dollar inputs and control percentages ≤ 100% (10,000bp) required".into(),
        ));
    }
    Ok(Json(traderview_expense::section_351::compute(&b)))
}

// ── §451(b) AFS conformity / all-events test acceleration ───────────
// Mounted at /api/calc/section-451b. TCJA P.L. 115-97 §13221 added
// §451(b) requiring accrual taxpayers with AFS to recognize income
// no later than when recognized on AFS; §451(b)(3) AFS hierarchy
// (SEC-filed > audited > certified); §451(b) cost offset election
// (TD 9941 eff. 2020-12-21); §451(c) 1-year advance payment deferral.

async fn section_451b_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_451b::Section451bInput>,
) -> Result<Json<traderview_expense::section_451b::Section451bResult>, ApiError> {
    if b.afs_revenue_recognized_for_item_dollars < 0
        || b.classic_all_events_test_amount_dollars < 0
        || b.costs_incurred_to_date_for_cost_offset_dollars < 0
        || b.advance_payment_received_current_year_dollars < 0
        || b.afs_advance_payment_recognized_current_year_dollars < 0
    {
        return Err(ApiError::BadRequest(
            "all dollar inputs must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_451b::compute(&b)))
}

// ── MLP K-1 UBTI tracker for IRAs ─────────────────────────────────────
// Mounted at /api/calc/mlp-ubti. Aggregates K-1 line items into
// Unrelated Business Taxable Income, applies §512(b) exclusions for
// passive items, §514 debt-financed inclusion, §512(b)(12) $1k
// specific deduction, and trust-rate tax per §511(b)(2).

async fn mlp_ubti_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::mlp_ubti::MlpUbtiInput>,
) -> Result<Json<traderview_expense::mlp_ubti::MlpUbtiResult>, ApiError> {
    Ok(Json(traderview_expense::mlp_ubti::compute(&b)))
}

// ── §168(g) Alternative Depreciation System (ADS) ────────────────────
// Mounted at /api/calc/section-168g. Pure compute; computes the
// annual ADS deduction for a property at a given year, with a GDS
// comparison so callers can sum up multi-property differences and
// feed into the §163(j) tradeoff analyzer.

async fn section_168g_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_168g::Section168gInput>,
) -> Result<Json<traderview_expense::section_168g::Section168gResult>, ApiError> {
    if b.depreciable_basis < Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "depreciable_basis must be >= 0".into(),
        ));
    }
    if !(1..=12).contains(&b.placed_in_service_month) {
        return Err(ApiError::BadRequest(
            "placed_in_service_month must be 1..12".into(),
        ));
    }
    Ok(Json(traderview_expense::section_168g::compute(&b)))
}

// ── §168(k) bonus depreciation (post-OBBBA 100% permanent) ──────────
// Mounted at /api/calc/section-168k. § 168(k)(1) additional first-year
// depreciation deduction; § 168(k)(2) qualified property (MACRS ≤ 20
// years + no prior use); § 168(k)(6) pre-OBBBA TCJA phasedown rate
// schedule (100% 2018-2022, 80% 2023, 60% 2024, 40% 2025, 20% 2026,
// 0% 2027+); OBBBA § 70302 permanently restores 100% for property
// acquired AND placed in service after 2025-01-19 — eliminating the
// TCJA phasedown's 2026-2027 step-down years. Transition election
// permits 40% (60% long-production/aircraft) for FYE-after-2025-01-19
// year. Used property eligible if no prior use by taxpayer (TCJA
// 2017 expansion preserved by OBBBA). Distinct from § 179 expensing
// which has dollar caps; § 168(k) has no dollar cap, no income limit,
// no phaseout — works alongside § 179 (§ 179 first, then § 168(k) on
// remainder, then MACRS).

async fn section_168k_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_168k::Section168KInput>,
) -> Result<Json<traderview_expense::section_168k::Section168KResult>, ApiError> {
    if b.property_cost_cents < 0 {
        return Err(ApiError::BadRequest(
            "property_cost_cents must be non-negative".into(),
        ));
    }
    if !(1990..=2100).contains(&b.acquisition_year)
        || !(1..=12).contains(&b.acquisition_month)
        || !(1..=31).contains(&b.acquisition_day)
        || !(1990..=2100).contains(&b.placed_in_service_year)
        || !(1..=12).contains(&b.placed_in_service_month)
        || !(1..=31).contains(&b.placed_in_service_day)
    {
        return Err(ApiError::BadRequest(
            "acquisition + placed_in_service dates must be valid".into(),
        ));
    }
    Ok(Json(traderview_expense::section_168k::compute(&b)))
}

// ── §163(j)(7)(B) electing-RPTB tradeoff analyzer ────────────────────
// Mounted at /api/calc/section-163j-tradeoff. Pure compute; turns
// annual depreciation sacrificed + annual interest disallowed into
// after-tax net benefit using the taxpayer's marginal rate.

async fn section_163j_tradeoff_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_168g::Section163jTradeoffInput>,
) -> Result<Json<traderview_expense::section_168g::Section163jTradeoffResult>, ApiError> {
    if b.marginal_federal_rate < Decimal::ZERO
        || b.marginal_federal_rate > Decimal::ONE
    {
        return Err(ApiError::BadRequest(
            "marginal_federal_rate must be 0..1".into(),
        ));
    }
    if b.annual_depreciation_sacrificed < Decimal::ZERO
        || b.annual_interest_disallowed_under_163j < Decimal::ZERO
    {
        return Err(ApiError::BadRequest(
            "depreciation / interest amounts must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_168g::analyze_tradeoff(&b)))
}

// ── §164 SALT deduction cap (TCJA + OBBBA expansion) ─────────────────
// Mounted at /api/calc/section-164. TCJA §164(b)(6) capped SALT at
// $10K ($5K MFS) for 2018-2024. OBBBA §70413 (eff. 2025-01-01)
// temporarily expanded the cap to $40K ($20K MFS) for 2025 with annual
// 1% compounded growth through 2029; 30% high-income phaseout above
// $500K MAGI ($250K MFS) with threshold also growing 1%/yr; statutory
// $10K ($5K MFS) floor — phaseout never drives the cap below the floor.
// Automatic sunset to TCJA $10K cap in 2030. Out of scope: pass-through-
// entity (PTET) workaround state-level elections.

async fn section_164_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_164::Section164Input>,
) -> Result<Json<traderview_expense::section_164::Section164Result>, ApiError> {
    if b.salt_paid_cents < 0 {
        return Err(ApiError::BadRequest(
            "salt_paid_cents must be non-negative".into(),
        ));
    }
    if !(1900..=2100).contains(&b.year) {
        return Err(ApiError::BadRequest(
            "year must be in [1900, 2100]".into(),
        ));
    }
    Ok(Json(traderview_expense::section_164::compute(&b)))
}

// ── §165(h) personal casualty loss deduction ─────────────────────────
// Mounted at /api/calc/section-165h. Three time-windowed regimes:
// pre-TCJA (≤2017) any sudden-unexpected-identifiable event qualifies
// subject to $100 per-event + 10% AGI floors; TCJA window 2018-2025
// §165(h)(5) suspends personal casualty losses EXCEPT for federally
// declared disasters (FEMA); OBBBA §70423 (eff. tax years after
// 2025-12-31) makes TCJA suspension PERMANENT for non-disaster losses
// AND EXPANDS qualifying events to include state-declared disasters
// (natural catastrophes hurricane/tornado/storm/earthquake or any
// fire/flood/explosion the state deems severe). Per-event $500 floor
// + no 10% AGI floor for congressionally designated qualified-disaster
// losses. Loss = lesser of (basis, FMV decline) − insurance, capped at 0.

async fn section_165h_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_165h::Section165HInput>,
) -> Result<Json<traderview_expense::section_165h::Section165HResult>, ApiError> {
    if b.adjusted_basis_cents < 0 || b.decline_in_fmv_cents < 0
        || b.insurance_reimbursement_cents < 0 || b.agi_cents < 0
    {
        return Err(ApiError::BadRequest(
            "non-negative cents inputs required".into(),
        ));
    }
    if !(1900..=2100).contains(&b.year) {
        return Err(ApiError::BadRequest(
            "year must be in [1900, 2100]".into(),
        ));
    }
    Ok(Json(traderview_expense::section_165h::compute(&b)))
}

// ── §25C Energy Efficient Home Improvement Credit (OBBBA term 12/31/25)
// Mounted at /api/calc/section-25c. IRA 2022 30% credit for energy-
// efficiency improvements with layered cap structure totaling up to
// $3,200/year. General $1,200 envelope (§25C(b)(1)) with sub-caps:
// $600 windows+skylights (§25C(b)(2)(A)) + $250/door / $500 aggregate
// doors (§25C(b)(2)(B)) + $600/item energy property (§25C(b)(2)(C)) +
// $150 home energy audit (§25C(b)(2)(D)) + insulation no sub-cap.
// Heat pump SEPARATE $2,000 cap (§25C(b)(3)) above and beyond the
// general $1,200. NONREFUNDABLE no carryforward (distinct from §25D).
// OBBBA §70425 ACCELERATED termination to property PLACED IN SERVICE
// after 2025-12-31 — wiping out IRA's 2032 sunset.

async fn section_25c_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_25c::Section25CInput>,
) -> Result<Json<traderview_expense::section_25c::Section25CResult>, ApiError> {
    if b.windows_skylights_cost_cents < 0
        || b.doors_cost_cents < 0
        || b.insulation_cost_cents < 0
        || b.energy_property_cost_cents < 0
        || b.heat_pump_cost_cents < 0
        || b.home_energy_audit_cost_cents < 0
    {
        return Err(ApiError::BadRequest(
            "non-negative cents inputs required".into(),
        ));
    }
    if !(1990..=2100).contains(&b.placed_in_service_year)
        || !(1..=12).contains(&b.placed_in_service_month)
        || !(1..=31).contains(&b.placed_in_service_day)
    {
        return Err(ApiError::BadRequest(
            "placed_in_service date must be a valid year/month/day".into(),
        ));
    }
    Ok(Json(traderview_expense::section_25c::compute(&b)))
}

// ── §25D Residential Clean Energy Credit (OBBBA termination 12/31/25) ─
// Mounted at /api/calc/section-25d. IRA 2022 30% credit for qualifying
// clean energy property installed at taxpayer's residence (primary +
// secondary homes; NOT pure rentals). Qualifying property §25D(d):
// solar electric + solar water heater + fuel cell + small wind +
// geothermal heat pump + battery storage with capacity ≥ 3 kWh under
// §25D(d)(6) added 2023 (biomass terminated end of 2022 moved to §25C).
// Nonrefundable with §25D(c) INDEFINITE carryforward to succeeding
// years. OBBBA §70426 ACCELERATED termination to expenditures made
// after December 31, 2025 — wiping out 2026-2034 step-down years
// originally scheduled under IRA.

async fn section_25d_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_25d::Section25DInput>,
) -> Result<Json<traderview_expense::section_25d::Section25DResult>, ApiError> {
    if b.qualifying_property_cost_cents < 0 || b.current_year_tax_liability_cents < 0 {
        return Err(ApiError::BadRequest(
            "non-negative cents inputs required".into(),
        ));
    }
    if !(1990..=2100).contains(&b.expenditure_year)
        || !(1..=12).contains(&b.expenditure_month)
        || !(1..=31).contains(&b.expenditure_day)
    {
        return Err(ApiError::BadRequest(
            "expenditure date must be a valid year/month/day".into(),
        ));
    }
    Ok(Json(traderview_expense::section_25d::compute(&b)))
}

// ── §30D Clean Vehicle Credit (post-OBBBA termination 2025-09-30) ────
// Mounted at /api/calc/section-30d. IRA 2022 bifurcated $7,500 credit:
// $3,750 critical-minerals (§30D(e)(1)) + $3,750 battery-components
// (§30D(e)(2)). MSRP caps §30D(f)(11): $55K cars / $80K SUVs+trucks+vans.
// MAGI hard-cutoff §30D(f)(10): $150K Single/MFS + $225K HoH + $300K MFJ.
// OBBBA §70424 (eff. 2025-09-30) TERMINATED §30D for vehicles acquired
// after September 30, 2025 — accelerating the IRA's 2032 sunset by 7+
// years. IRS binding-contract carve-out: written binding contract +
// payment ≤ 2025-09-30 preserves credit even if vehicle placed in
// service later. Out of scope: §25E used clean vehicle credit (also
// terminated 2025-09-30); §30D(g) transfer-to-dealer election.

async fn section_30d_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_30d::Section30DInput>,
) -> Result<Json<traderview_expense::section_30d::Section30DResult>, ApiError> {
    if b.msrp_cents < 0 || b.modified_agi_cents < 0 {
        return Err(ApiError::BadRequest(
            "msrp_cents and modified_agi_cents must be non-negative".into(),
        ));
    }
    if !(1990..=2100).contains(&b.acquisition_year)
        || !(1..=12).contains(&b.acquisition_month)
        || !(1..=31).contains(&b.acquisition_day)
    {
        return Err(ApiError::BadRequest(
            "acquisition date must be a valid year/month/day".into(),
        ));
    }
    Ok(Json(traderview_expense::section_30d::compute(&b)))
}

// ── §1296 PFIC mark-to-market election ───────────────────────────────
// Mounted at /api/calc/section-1296. Pure compute; annual mark of
// marketable PFIC stock as ordinary income, with loss limited to
// prior unreversed inclusions.

async fn section_1296_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1296::Section1296Input>,
) -> Result<Json<traderview_expense::section_1296::Section1296Result>, ApiError> {
    if b.adjusted_basis_year_start < Decimal::ZERO
        || b.fair_market_value_year_end < Decimal::ZERO
        || b.prior_unreversed_inclusions < Decimal::ZERO
    {
        return Err(ApiError::BadRequest("all dollar inputs must be >= 0".into()));
    }
    Ok(Json(traderview_expense::section_1296::compute(&b)))
}

// ── §1341 claim-of-right doctrine — lesser-of deduction vs credit ────
// Mounted at /api/calc/section-1341. Codifies the claim-of-right
// doctrine: when income reported in a prior year is restored in a later
// year and exceeds the §1341(a)(3) $3,000 threshold, taxpayer chooses
// the LESSER of Method A (§1341(a)(4) deduction) and Method B (§1341(a)(5)
// refundable credit = current-year tax without relief minus prior-year
// tax decrease that would have resulted had the now-repaid income been
// excluded). §1341(b)(2) mandates the lesser; no election required.

async fn section_1341_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1341::Section1341Input>,
) -> Result<Json<traderview_expense::section_1341::Section1341Result>, ApiError> {
    if b.repayment_amount_cents < 0 {
        return Err(ApiError::BadRequest(
            "repayment_amount_cents must be non-negative".into(),
        ));
    }
    Ok(Json(traderview_expense::section_1341::compute(&b)))
}

// ── §988 forex transaction character ─────────────────────────────────
// Mounted at /api/calc/section-988. Pure compute; classifies forex /
// FX-denominated debt / forex derivatives into ordinary / capital /
// §1256 60/40 / personal-use-excluded character based on transaction
// kind + §988(a)(1)(B) capital election + §988(c)(1)(D)(i) kick-out
// election + personal-use flag.

async fn section_988_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_988::Section988Input>,
) -> Result<Json<traderview_expense::section_988::Section988Result>, ApiError> {
    Ok(Json(traderview_expense::section_988::compute(&b)))
}

// ── §267 related-party loss disallowance ─────────────────────────────
// Mounted at /api/calc/section-267. Pure compute; §267(a)(1) disallows
// the loss when seller and buyer are related per the §267(b) 10-category
// list. §267(d) preserves the disallowance for buyer's subsequent gain
// reduction (capped at buyer's actual gain).

async fn section_267_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_267::Section267Input>,
) -> Result<Json<traderview_expense::section_267::Section267Result>, ApiError> {
    if b.realized_loss < Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "realized_loss must be >= 0 (pass loss as positive number)".into(),
        ));
    }
    if b.buyer_purchase_price < Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "buyer_purchase_price must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_267::compute(&b)))
}

// ── §469(c)(7) Real Estate Professional Status qualification ─────────
// Mounted at /api/calc/reps-qualification. Pure compute; checks the
// 750-hour test, the >50%-of-personal-services test, and material
// participation. Returns whether REPS is met (flips rental losses from
// passive to non-passive in §469 PAL).

async fn reps_qualification_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::reps_qualification::RepsInput>,
) -> Result<Json<traderview_expense::reps_qualification::RepsResult>, ApiError> {
    if b.other_personal_services_hours < Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "other_personal_services_hours must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::reps_qualification::compute(&b)))
}

// ── §121 home sale exclusion ──────────────────────────────────────────
// Mounted at /api/calc/section-121. Pure compute; up to $250k single /
// $500k MFJ of gain on principal-residence sale excluded with the 2-of-5
// year ownership + use tests, §121(b)(4) hardship pro-rata, §121(b)(5)
// non-qualified-use proportional reduction, and §121(d)(6) post-1997
// depreciation recapture.

async fn section_121_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_121::Section121Input>,
) -> Result<Json<traderview_expense::section_121::Section121Result>, ApiError> {
    if b.sale_price < Decimal::ZERO
        || b.selling_costs < Decimal::ZERO
        || b.depreciation_post_1997 < Decimal::ZERO
    {
        return Err(ApiError::BadRequest(
            "sale_price / selling_costs / depreciation_post_1997 must be >= 0".into(),
        ));
    }
    if b.non_qualified_use_days_post_2008 > b.total_ownership_days_post_2008
        && b.total_ownership_days_post_2008 > 0
    {
        return Err(ApiError::BadRequest(
            "non_qualified_use_days_post_2008 must be <= total_ownership_days_post_2008".into(),
        ));
    }
    Ok(Json(traderview_expense::section_121::compute(&b)))
}

// ── §121(d) divorce special rules ───────────────────────────────────
// Mounted at /api/calc/section-121d. §121(d)(2) holding-period
// tacking from §1041(a) transferor spouse; §121(d)(3)(A) use
// attribution via former-spouse occupation under divorce or
// separation instrument. Lets divorced spouse meet 2-year ownership
// and 2-year use tests even after years of non-occupation.

async fn section_121d_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_121d::Section121dInput>,
) -> Result<Json<traderview_expense::section_121d::Section121dResult>, ApiError> {
    if b.gain_realized_on_sale_dollars < 0 {
        return Err(ApiError::BadRequest(
            "gain_realized_on_sale_dollars must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_121d::compute(&b)))
}

// ── §132 fringe benefits exclusion ───────────────────────────────────
// Mounted at /api/calc/section-132. §132(a) 8 fringe-benefit
// exclusion categories: (1) no-additional-cost + (2) qualified
// employee discount (services 20% / goods gross-profit %) + (3)
// working condition + (4) de minimis + (5) qualified transportation
// (§132(f) 2026 $340/mo each for parking and transit) + (6)
// qualified moving PERMANENTLY SUSPENDED by OBBBA 2025 P.L. 119-21
// except armed forces / intelligence community + (7) retirement
// planning + (8) military base realignment.

async fn section_132_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_132::Section132Input>,
) -> Result<Json<traderview_expense::section_132::Section132Result>, ApiError> {
    if b.fringe_value_dollars < 0
        || b.parking_monthly_cap_dollars < 0
        || b.transit_monthly_cap_dollars < 0
    {
        return Err(ApiError::BadRequest(
            "non-negative dollar inputs required".into(),
        ));
    }
    Ok(Json(traderview_expense::section_132::compute(&b)))
}
