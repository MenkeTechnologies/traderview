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
        .route("/calc/section-274",           post(section_274_route))
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
        .route("/calc/section-1366",          post(section_1366_route))
        .route("/calc/section-1377",          post(section_1377_route))
        .route("/calc/section-1367",          post(section_1367_route))
        .route("/calc/section-1368",          post(section_1368_route))
        .route("/calc/section-1374",          post(section_1374_route))
        .route("/calc/section-1375",          post(section_1375_route))
        .route("/calc/section-1411",          post(section_1411_route))
        .route("/calc/section-475c2",         post(section_475c2_route))
        .route("/calc/section-213",           post(section_213_route))
        .route("/calc/section-170",           post(section_170_route))
        .route("/calc/section-219",           post(section_219_route))
        .route("/calc/section-221",           post(section_221_route))
        .route("/calc/section-223",           post(section_223_route))
        .route("/calc/section-243",           post(section_243_route))
        .route("/calc/section-250",           post(section_250_route))
        .route("/calc/section-56a",           post(section_56a_route))
        .route("/calc/section-59a",           post(section_59a_route))
        .route("/calc/section-67g",           post(section_67g_route))
        .route("/calc/section-6042",          post(section_6042_route))
        .route("/calc/section-6045",          post(section_6045_route))
        .route("/calc/section-6049",          post(section_6049_route))
        .route("/calc/section-6050i",         post(section_6050i_route))
        .route("/calc/section-6050w",         post(section_6050w_route))
        .route("/calc/section-6212",          post(section_6212_route))
        .route("/calc/section-6213",          post(section_6213_route))
        .route("/calc/section-6201",          post(section_6201_route))
        .route("/calc/section-6203",          post(section_6203_route))
        .route("/calc/section-6303",          post(section_6303_route))
        .route("/calc/section-6304",          post(section_6304_route))
        .route("/calc/section-6306",          post(section_6306_route))
        .route("/calc/section-6320",          post(section_6320_route))
        .route("/calc/section-6321",          post(section_6321_route))
        .route("/calc/section-6323",          post(section_6323_route))
        .route("/calc/section-6325",          post(section_6325_route))
        .route("/calc/section-6330",          post(section_6330_route))
        .route("/calc/section-6331",          post(section_6331_route))
        .route("/calc/section-6332",          post(section_6332_route))
        .route("/calc/section-6334",          post(section_6334_route))
        .route("/calc/section-6402",          post(section_6402_route))
        .route("/calc/section-6404",          post(section_6404_route))
        .route("/calc/section-7201",          post(section_7201_route))
        .route("/calc/section-7202",          post(section_7202_route))
        .route("/calc/section-7203",          post(section_7203_route))
        .route("/calc/section-7212",          post(section_7212_route))
        .route("/calc/section-7216",          post(section_7216_route))
        .route("/calc/section-7206",          post(section_7206_route))
        .route("/calc/section-7207",          post(section_7207_route))
        .route("/calc/section-7421",          post(section_7421_route))
        .route("/calc/section-7422",          post(section_7422_route))
        .route("/calc/section-7426",          post(section_7426_route))
        .route("/calc/section-7429",          post(section_7429_route))
        .route("/calc/section-7430",          post(section_7430_route))
        .route("/calc/section-7433",          post(section_7433_route))
        .route("/calc/section-7434",          post(section_7434_route))
        .route("/calc/section-7463",          post(section_7463_route))
        .route("/calc/section-7491",          post(section_7491_route))
        .route("/calc/section-162a",          post(section_162a_route))
        .route("/calc/section-162f",          post(section_162f_route))
        .route("/calc/section-162m",          post(section_162m_route))
        .route("/calc/section-7502",          post(section_7502_route))
        .route("/calc/section-7503",          post(section_7503_route))
        .route("/calc/section-7508",          post(section_7508_route))
        .route("/calc/section-7508a",         post(section_7508a_route))
        .route("/calc/section-7521",          post(section_7521_route))
        .route("/calc/section-7522",          post(section_7522_route))
        .route("/calc/section-7525",          post(section_7525_route))
        .route("/calc/section-7811",          post(section_7811_route))
        .route("/calc/section-6501",          post(section_6501_route))
        .route("/calc/section-6502",          post(section_6502_route))
        .route("/calc/section-6531",          post(section_6531_route))
        .route("/calc/section-6532",          post(section_6532_route))
        .route("/calc/section-6511",          post(section_6511_route))
        .route("/calc/section-6601",          post(section_6601_route))
        .route("/calc/section-6611",          post(section_6611_route))
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
        .route("/calc/section-6020",          post(section_6020_route))
        .route("/calc/section-6038a",         post(section_6038a_route))
        .route("/calc/section-6038b",         post(section_6038b_route))
        .route("/calc/section-6038c",         post(section_6038c_route))
        .route("/calc/section-6038d",         post(section_6038d_route))
        .route("/calc/section-6011",          post(section_6011_route))
        .route("/calc/section-6111",          post(section_6111_route))
        .route("/calc/section-6112",          post(section_6112_route))
        .route("/calc/section-6662a",         post(section_6662a_route))
        .route("/calc/section-6663",          post(section_6663_route))
        .route("/calc/section-6664",          post(section_6664_route))
        .route("/calc/section-6672",          post(section_6672_route))
        .route("/calc/section-6694",          post(section_6694_route))
        .route("/calc/section-6695",          post(section_6695_route))
        .route("/calc/section-6695a",         post(section_6695a_route))
        .route("/calc/section-6700",          post(section_6700_route))
        .route("/calc/section-6701",          post(section_6701_route))
        .route("/calc/section-6707",          post(section_6707_route))
        .route("/calc/section-6707a",         post(section_6707a_route))
        .route("/calc/section-6708",          post(section_6708_route))
        .route("/calc/section-6713",          post(section_6713_route))
        .route("/calc/section-6851",          post(section_6851_route))
        .route("/calc/section-6861",          post(section_6861_route))
        .route("/calc/section-6862",          post(section_6862_route))
        .route("/calc/section-6863",          post(section_6863_route))
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
        .route("/calc/section-280g",          post(section_280g_route))
        .route("/calc/section-163d",          post(section_163d_route))
        .route("/calc/section-163h",          post(section_163h_route))
        .route("/calc/section-864b2",         post(section_864b2_route))
        .route("/calc/section-72t",           post(section_72t_route))
        .route("/calc/section-7345",          post(section_7345_route))
        .route("/calc/section-7623",          post(section_7623_route))
        .route("/calc/section-7405",          post(section_7405_route))
        .route("/calc/section-7408",          post(section_7408_route))
        .route("/calc/section-7701",          post(section_7701_route))
        .route("/calc/section-7872",          post(section_7872_route))
        .route("/calc/section-1291",          post(section_1291_route))
        .route("/calc/section-1293",          post(section_1293_route))
        .route("/calc/section-1294",          post(section_1294_route))
        .route("/calc/section-1295",          post(section_1295_route))
        .route("/calc/section-1058",          post(section_1058_route))
        .route("/calc/section-1092",          post(section_1092_route))
        .route("/calc/section-408",           post(section_408_route))
        .route("/calc/section-401k",          post(section_401k_route))
        .route("/calc/section-415",           post(section_415_route))
        .route("/calc/section-408a",          post(section_408a_route))
        .route("/calc/section-421",           post(section_421_route))
        .route("/calc/section-422",           post(section_422_route))
        .route("/calc/section-423",           post(section_423_route))
        .route("/calc/section-4501",          post(section_4501_route))
        .route("/calc/section-4940",          post(section_4940_route))
        .route("/calc/section-4941",          post(section_4941_route))
        .route("/calc/section-4942",          post(section_4942_route))
        .route("/calc/section-4943",          post(section_4943_route))
        .route("/calc/section-4944",          post(section_4944_route))
        .route("/calc/section-4945",          post(section_4945_route))
        .route("/calc/section-4958",          post(section_4958_route))
        .route("/calc/section-4960",          post(section_4960_route))
        .route("/calc/section-4972",          post(section_4972_route))
        .route("/calc/section-4973",          post(section_4973_route))
        .route("/calc/section-4974",          post(section_4974_route))
        .route("/calc/section-4975",          post(section_4975_route))
        .route("/calc/section-4978",          post(section_4978_route))
        .route("/calc/section-6166",          post(section_6166_route))
        .route("/calc/section-4980",          post(section_4980_route))
        .route("/calc/section-4980h",         post(section_4980h_route))
        .route("/calc/section-453",           post(section_453_route))
        .route("/calc/section-453a",          post(section_453a_route))
        .route("/calc/section-457a",          post(section_457a_route))
        .route("/calc/section-457b",          post(section_457b_route))
        .route("/calc/section-461g",          post(section_461g_route))
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
        .route("/calc/section-1042",          post(section_1042_route))
        .route("/calc/section-170e",          post(section_170e_route))
        .route("/calc/section-172",           post(section_172_route))
        .route("/calc/section-195",           post(section_195_route))
        .route("/calc/section-248",           post(section_248_route))
        .route("/calc/section-709",           post(section_709_route))
        .route("/calc/section-197",           post(section_197_route))
        .route("/calc/section-199a",          post(section_199a_route))
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

// ── §408 traditional IRA + SEP + SIMPLE + collectibles + QCD ─────────
// Mounted at /api/calc/section-408. § 408(a) IRA defined as trust for
// exclusive benefit; six requirements (within § 219 limits + qualified
// trustee + no life insurance + nonforfeitable + not commingled +
// § 401(a)(9) RMD). § 408(b) individual retirement annuity. 2026
// contribution limits aggregate with § 408A Roth: $7,500 base + $1,100
// catch-up (50+) = $8,600. § 219(g) deduction phase-out for ACTIVE
// PARTICIPANT in employer-sponsored plan (Single/HOH $81K-$91K; MFJ
// covered $129K-$149K; MFJ spouse-covered $242K-$252K; MFS $0-$10K).
// § 408(d)(1) distributions ordinary income; § 408(d)(2) PRO-RATA RULE
// aggregate across ALL IRAs (CRITICAL for backdoor Roth); § 408(d)(3)
// 60-day rollover + ONE-ROLLOVER-PER-YEAR per Bobrow v. Commissioner
// T.C. Memo 2014-21 + IRS Announcement 2014-15; § 408(d)(6) RMD per
// SECURE Act 2.0 (age 73 / 75 born 1960+); § 408(d)(8) QCD age 70½+
// $111,000 (2026) + § 408(d)(8)(F) $50K split-interest entity.
// § 408(k) SEP IRA 25%/$70K. § 408(m) COLLECTIBLES PROHIBITION
// (artwork, antique, gem, stamp/coin, alcohol) + § 408(m)(3) EXCEPTION
// for gold/silver coins/bullion. § 408(p) SIMPLE IRA (≤ 100 employees;
// 2026 $17K + $4K catch-up). § 408(q) deemed IRA. Companion to § 408A
// + § 72(t) + § 67(g) + § 1411 + § 475 + § 4975 + § 4973. Created by
// ERISA 1974 (Pub. L. 93-406, September 2, 1974); modified by SECURE
// Act 2019 + SECURE Act 2.0 of 2022.

async fn section_408_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_408::Section408Input>,
) -> Result<Json<traderview_expense::section_408::Section408Result>, ApiError> {
    Ok(Json(traderview_expense::section_408::check(&b)))
}

// ── §408A Roth IRA contribution + phase-out + qualified distribution ─
// Mounted at /api/calc/section-408a. § 408A(c)(1) 2026 contribution
// limit aggregate with § 408 traditional IRA — $7,500 base + $1,100
// catch-up (age 50+); § 408A(c)(3) income-based phase-out (Single/HOH
// $153K-$168K; MFJ $242K-$252K; MFS $0-$10K NOT cost-of-living
// adjusted); § 408A(c)(3)(B) modified AGI DISREGARDS Roth conversion
// income; § 408A(d)(2) qualified distribution two-prong test (5-year
// holding + age 59½ OR disability OR death OR first-time home up to
// $10K lifetime); § 408A(d)(3) ordering rules (contributions then
// conversions then earnings); § 408A(d)(3)(A) separate 5-year per-
// conversion holding period; § 408A(e) backdoor Roth (non-deductible
// traditional + conversion; pro-rata rule under § 408(d)(2));
// § 408A(c)(5) NO RMD during owner's lifetime; Roth distributions
// EXEMPT from § 1411 NIIT 3.8% surtax. Created by Taxpayer Relief Act
// of 1997 § 302 (Pub. L. 105-34, August 5, 1997); modified by SECURE
// Act of 2019 + SECURE Act 2.0 of 2022. Trader-critical fact patterns:
// high-income trader backdoor Roth; active trader self-directed Roth
// IRA escaping § 1411 NIIT; mega backdoor Roth (after-tax 401(k));
// § 72(t) substantially-equal-periodic-payments. § 4973 6% excise tax
// on excess contributions.

async fn section_408a_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_408a::Section408aInput>,
) -> Result<Json<traderview_expense::section_408a::Section408aResult>, ApiError> {
    Ok(Json(traderview_expense::section_408a::check(&b)))
}

// ── § 401(k) Cash or Deferred Arrangements ──────────────────────────
// Mounted at /api/calc/section-401k. Pure compute; 2026 limits per
// IRS Notice 2025-67: § 402(g)(1) elective deferral $24,500;
// § 414(v)(1) catch-up age 50+ $8,000; § 414(v)(2)(E) SECURE 2.0
// enhanced catch-up ages 60-63 $11,250; § 415(c)(1)(A) annual
// addition $72,000; § 401(a)(17) compensation limit $360,000;
// HCE threshold $160,000; § 414(v)(7) mandatory Roth catch-up
// threshold $150,000 (SECURE 2.0 § 603 effective 2026); § 401(k)(3)
// ADP test (HCE ADP ≤ greater of non-HCE × 1.25 OR non-HCE + 2%);
// § 401(k)(12) safe harbor (3% non-elective OR basic match);
// § 402A designated Roth § 401(k); SECURE 2.0 § 325 (no lifetime
// RMD on Roth § 401(k)); SECURE 2.0 § 604 (Roth employer match);
// mega backdoor Roth via § 408A(d)(3) in-plan rollover.

async fn section_401k_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_401k::Section401kInput>,
) -> Result<Json<traderview_expense::section_401k::Section401kResult>, ApiError> {
    Ok(Json(traderview_expense::section_401k::check(&b)))
}

// ── § 415 Limits on Benefits and Contributions (umbrella statute) ────
// Mounted at /api/calc/section-415. Pure compute; 2026 limits per
// IRS Notice 2025-67: § 415(b)(1)(A) DB annual benefit $290,000;
// § 415(c)(1)(A) DC annual addition $72,000; § 401(a)(17)
// compensation limit $360,000. § 415(a) disqualification cascade
// (denial of § 401(a) qualified status for entire plan if any
// participant exceeds). § 415(b) DB limit = lesser of dollar limit
// or 100% of average high-3-year compensation (NOT subject to
// § 401(a)(17)). § 415(c) DC annual addition = employer + employee
// pretax + Roth + forfeitures (EXCLUDES § 414(v) catch-up). § 415(d)
// CPI-U annual COLA adjustment. § 415(f) aggregation: all DC plans
// of single employer aggregated; all DB plans of single employer
// aggregated; DC and DB limits applied SEPARATELY (§ 415(f)(2));
// § 414(b)/(c)/(m)/(o) controlled group + affiliated service group
// treat related employers as single employer. § 415(g) anti-cutback;
// § 415(k) grandfathered old-limit benefits; § 415(n) USERRA make-up
// rights.

async fn section_415_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_415::Section415Input>,
) -> Result<Json<traderview_expense::section_415::Section415Result>, ApiError> {
    Ok(Json(traderview_expense::section_415::check(&b)))
}

// ── § 422 Incentive Stock Options (ISOs) ─────────────────────────────
// Mounted at /api/calc/section-422. Pure compute; § 422(b) 6-element
// statutory test (shareholder-approved plan + 10-year window + price
// ≥ FMV + 3-month employment trail + transferability + 10-year
// exercise period); § 422(d) $100K annual limit (excess auto NQSO);
// § 422(a) 2-year-from-grant + 1-year-from-exercise qualified-
// disposition holding periods; § 421(b) disqualifying-disposition
// ordinary-income lesser-of rule (FMV-exercise - strike OR sale -
// strike); § 56(b)(3) AMT preference on exercise spread; § 422(c)(2)
// same-year disqualifying-disposition AMT reversal; § 53 AMT credit
// recovery; § 1411 NIIT 3.8% on qualified-disposition LTCG.

async fn section_422_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_422::Section422Input>,
) -> Result<Json<traderview_expense::section_422::Section422Result>, ApiError> {
    Ok(Json(traderview_expense::section_422::check(&b)))
}

// ── § 421 General rules for statutory stock options ──────────────────
// Mounted at /api/calc/section-421. Pure compute; § 421(a) general
// rule — no income at grant or exercise of statutory option (§ 422
// ISO or § 423 ESPP) for regular income tax purposes + no § 162
// deduction to employer + only option price considered as received
// by corporation. § 421(b) disqualifying disposition — increase in
// FMV over option price at time of exercise treated as compensation
// (ordinary income) in year of disposition; § 162 deduction allowed
// to employer; additional gain treated as capital gain per § 1222
// holding period from exercise date. ISO qualifying-disposition
// holding requirements per § 422(a)(1): no disposition within 2
// years from grant date AND no disposition within 1 year from
// transfer (exercise) date; ESPP similar requirements per § 423(a)(1).
// Employment requirement per § 422(a)(2): individual must be employee
// of granting corporation (or related corp under § 424(e)/(f)) at all
// times from grant date through 3 months before exercise (death or
// disability extends). AMT preference per § 56(b)(3) on exercise
// spread creates 'phantom income' for trader-employees. Information
// reporting per § 6039 + Form 3921 (ISO) + Form 3922 (ESPP).
// Coordination with § 1042 (iter 480): ISO-exercised shares NOT
// 'qualified securities' for ESOP rollover. Coordination with § 83:
// § 421 OVERRIDES § 83 for qualifying ISO/ESPP exercises. Original
// framework Tax Reform Act of 1976 + Economic Recovery Tax Act of
// 1981 + American Jobs Creation Act of 2004.

async fn section_421_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_421::Section421Input>,
) -> Result<Json<traderview_expense::section_421::Section421Result>, ApiError> {
    Ok(Json(traderview_expense::section_421::check(&b)))
}

// ── § 423 Employee Stock Purchase Plans (ESPPs) ──────────────────────
// Mounted at /api/calc/section-423. Pure compute; § 423(b) 9-element
// statutory test (employees only + shareholder-approved + no 5%+
// owner + all employees eligible + same rights/privileges + price
// ≥ 85% of lower of offering/purchase FMV + 27-month/5-year outer
// limit + $25K annual accrual cap + non-transferable); § 423(b)(6)
// look-back provision (85% of LOWER of offering or purchase FMV);
// § 421(a) 2-year-from-offering + 1-year-from-purchase qualifying-
// disposition holding periods; § 423(c) qualifying-disposition
// ordinary-income lesser-of rule (discount-at-offering OR actual
// gain); § 421(b) disqualifying-disposition full-spread-at-purchase
// rule; § 162 employer deduction only on disqualifying; Notice
// 2002-47 + Rev. Rul. 71-52 FICA exemption on qualifying ordinary
// income; § 1411 NIIT 3.8% on qualifying LTCG; § 424(d)
// constructive-ownership rules; Form 3922 ESPP Transfer
// Information Statement.

async fn section_423_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_423::Section423Input>,
) -> Result<Json<traderview_expense::section_423::Section423Result>, ApiError> {
    Ok(Json(traderview_expense::section_423::check(&b)))
}

// ── § 4973 excise tax on excess IRA / Roth / HSA / Coverdell contrib ─
// Mounted at /api/calc/section-4973. Pure compute; 6% annual non-
// deductible excise tax on excess contributions to § 408(a)
// traditional IRA, § 408A Roth IRA, § 408(b) IRA annuity, § 408(p)
// SIMPLE IRA, § 530 Coverdell ESA, § 220 Archer MSA, § 223 HSA;
// § 4973(c) correction window (return due date plus extensions =
// October 15) with NIA computed under Treas. Reg. § 1.408-11(b);
// SECURE Act 2.0 § 333 eliminates additional § 72(t) 10% early-
// withdrawal penalty on NIA when corrective distribution made
// timely; SECURE Act 2.0 § 313 establishes 6-year statute of
// limitations (previously no SoL); § 4973(g) uncorrected excess
// carryover-absorbed into subsequent year limit; Form 5329 Parts
// III-VII reporting.

async fn section_4973_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_4973::Section4973Input>,
) -> Result<Json<traderview_expense::section_4973::Section4973Result>, ApiError> {
    Ok(Json(traderview_expense::section_4973::check(&b)))
}

// ── § 4972 tax on nondeductible contributions to qualified plans ─────
// Mounted at /api/calc/section-4972. Pure compute; § 4972(a) imposes
// 10% annual excise tax on nondeductible contributions to qualified
// employer plans paid by EMPLOYER (not individual — § 4973 covers
// individual side); § 4972(c)(1)(A) nondeductible = current-year
// employer contributions plus unused prior-year carryforwards less
// § 404 deduction limit; § 4972(c)(2) ordering rule applies deduction
// first to carryforwards then to current contributions; § 4972(d)
// qualified employer plan = § 401(a) qualified plan + § 403(a)
// qualified annuity + § 408(k) SEP-IRA + § 408(p) SIMPLE IRA;
// § 4972(c)(6) exceptions: (A) SEP excess allocable to participant
// under § 415, (B) PSP deductibility from post-plan-year-end
// compensation increase. Carryforward compounds annually until
// consumed by future § 404 deduction headroom or returned under
// § 401(a)(2) reversion (§ 4980 iter 460 reversion excise also
// applies). Form 5330 filing deadline last day of 7th month after
// employer tax-year-end. Distinction from § 4973 (iter 442 individual
// IRA 6% excise) + § 4974 (iter 436 RMD 25% post-SECURE 2.0) +
// § 4975 (iter 434 prohibited transactions 15%/100%). Original
// enactment Deficit Reduction Act of 1984 Pub. L. 98-369 with current
// 10% rate from Omnibus Budget Reconciliation Act of 1989 Pub. L.
// 101-239.

async fn section_4972_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_4972::Section4972Input>,
) -> Result<Json<traderview_expense::section_4972::Section4972Result>, ApiError> {
    Ok(Json(traderview_expense::section_4972::check(&b)))
}

// ── § 4974 excise tax on RMD failures (post-SECURE 2.0) ──────────────
// Mounted at /api/calc/section-4974. Pure compute; 25% standard +
// 10% reduced (within § 4974(e) 2-year correction window) excise
// tax on shortfall between RMD required and amount distributed;
// SECURE Act 2.0 § 302 reduced rate from 50% to 25%; SECURE Act
// 2.0 § 107 raised RMD age from 72 to 73 (born 1951-1959) and to
// 75 (born 1960+) effective January 1, 2033; § 408(d)(8) QCD up
// to $108K satisfies RMD without inclusion in gross income;
// § 408A(c)(5) exempts Roth IRA from lifetime RMD; § 401(a)(9)(B)
// post-death 5-category beneficiary stretch / 10-year rule;
// § 4974(d) reasonable-error waiver via Form 5329.

async fn section_4974_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_4974::Section4974Input>,
) -> Result<Json<traderview_expense::section_4974::Section4974Result>, ApiError> {
    Ok(Json(traderview_expense::section_4974::check(&b)))
}

// ── § 4501 Repurchase of Corporate Stock Excise Tax ──────────────────
// Mounted at /api/calc/section-4501 (iter 496). Pure compute. IRA 2022
// Pub. L. 117-169 § 10201 1% excise tax on stock buybacks by covered
// corporations effective for repurchases after December 31, 2022. § 4501(b)
// covered-corporation definition: domestic corporation traded on
// established securities market per § 7704(b)(1) — NYSE, NASDAQ, national
// exchanges. SPAC sponsor / shareholder redemptions explicitly subject
// per Final Regs TD 10002 (July 3, 2024); IRS rejected SPAC carve-out.
// § 4501(c)(3) netting rule: FMV of repurchases reduced by FMV of statutory
// § 4501(e) excepted repurchases plus FMV of stock issuances (including
// compensatory RSU vest, ISO/NSO exercise, ESPP, equity grants) per
// Treas. Reg. § 1.4501-2(c). § 4501(e) six exceptions: § 368
// reorganization, ESOP/retirement-plan contribution, $1M de minimis,
// dealer ordinary course, RIC/REIT, § 301 dividend treatment. § 4501(d)
// specified-affiliate extension to foreign-parent anti-inversion. Excise
// tax NOT deductible per § 275(a)(6) — permanent book-tax difference.
// Form 7208 attached to Form 720 quarterly. Coordination with § 280G
// (golden parachute), § 421 (statutory stock option issuance offset),
// § 56A (corporate AMT 15% — separate IRA 2022 provision), § 4960
// (ATEO executive comp 21%), § 1042 (ESOP rollover for retirement-plan
// exception).

async fn section_4501_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_4501::Section4501Input>,
) -> Result<Json<traderview_expense::section_4501::Section4501Result>, ApiError> {
    Ok(Json(traderview_expense::section_4501::check(&b)))
}

// ── § 4940 annual excise tax on private foundation NII ───────────────
// Mounted at /api/calc/section-4940. Pure compute; § 4940(a) imposes
// annual excise tax on net investment income of every domestic tax-
// exempt private foundation (except § 4940(d) exempt operating
// foundations). Post-Dec-20-2019 regime: single flat rate of 1.39%
// per Further Consolidated Appropriations Act, 2020, Pub. L. 116-94
// (signed December 20, 2019) which amended § 4940(a) and REPEALED
// former § 4940(e). Pre-Dec-20-2019 regime: 2% standard with 1%
// reduced under former § 4940(e) for foundations meeting
// distribution-requirement tests (qualifying distributions ≥ average
// 5-year payout + 1% of NII). Net investment income per § 4940(c):
// gross investment income (§ 4940(c)(1) — interest + dividends +
// rents + securities-loan payments + royalties) plus net capital
// gain from sale of investment property (§ 4940(c)(4)(A)) minus
// allowable deductions (§ 4940(c)(2) — ordinary and necessary
// expenses paid or incurred for production/collection of gross
// investment income). Exempt operating foundations per § 4940(d)
// four-part test: (A) publicly supported for at least 10 prior
// taxable years; (B) governing body broadly representative with not
// more than 25% disqualified persons under § 4946; (C) operating-
// foundation status under § 4942(j)(3); (D) no officer who is
// disqualified individual appointed by disqualified persons.
// Foreign private foundations under § 4948 separate regime at 4% on
// US-source gross investment income. § 501(c)(3) public charities NOT
// subject to § 4940. Quarterly estimated tax payments under § 6655.
// Original enactment Tax Reform Act of 1969, Pub. L. 91-172.

async fn section_4940_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_4940::Section4940Input>,
) -> Result<Json<traderview_expense::section_4940::Section4940Result>, ApiError> {
    Ok(Json(traderview_expense::section_4940::check(&b)))
}

// ── § 4941 taxes on self-dealing (private foundation regime) ─────────
// Mounted at /api/calc/section-4941. Pure compute; four-tier excise
// tax structure on private-foundation self-dealing: § 4941(a)(1)
// Tier-1 disqualified person 10% of amount involved per year of
// taxable period; § 4941(a)(2) Tier-1 foundation manager 5% (knowing
// willful participant) capped at $20,000 per act per § 4941(c)(2);
// § 4941(b)(1) Tier-2 DP 200% (uncorrected within taxable period);
// § 4941(b)(2) Tier-2 manager 50% (refusing correction) capped at
// $20,000. Six self-dealing categories per § 4941(d)(1): (A) sale/
// exchange/lease; (B) lending of money or extension of credit; (C)
// furnishing goods/services/facilities; (D) compensation/expense
// reimbursement; (E) transfer/use of income or assets; (F) agreement
// to pay government official (§ 4946(c)). Four statutory exceptions
// per § 4941(d)(2): § 4941(d)(2)(B) interest-free loan from DP to PF
// for charitable purpose; § 4941(d)(2)(C) DP furnishes goods to PF
// without charge; § 4941(d)(2)(D) PF furnishes to DP on no more
// favorable basis than to general public; § 4941(d)(2)(E) reasonable
// compensation for personal services necessary to exempt purpose
// (NOT permitted to government official). Disqualified person per
// § 4946: substantial contributors (§ 507(d)(2) > $5K AND > 2%) +
// foundation managers (§ 4946(b)) + 20%-owners of contributor
// entities + family per § 4946(d) + 35%-controlled entities + other
// related PFs + government officials (§ 4946(c)). Amount involved
// per § 4941(e)(1): greater of money + FMV given vs received; loans
// per § 4941(e)(2). Taxable period per § 4941(e)(3) begins on
// transaction date, ends on earliest of statutory notice / tax
// assessment / correction. Correction per § 4941(e)(4): undo +
// place PF in position no worse than under highest fiduciary
// standards. Distinct from § 4958 (iter 466) intermediate sanctions:
// § 4941 applies to PRIVATE FOUNDATIONS only (excluded from § 4958
// per § 4958(e)), uses lower 10% Tier-1 rate but per-se rule (no
// excess-benefit comparison required); original enactment Tax
// Reform Act of 1969, Pub. L. 91-172.

async fn section_4941_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_4941::Section4941Input>,
) -> Result<Json<traderview_expense::section_4941::Section4941Result>, ApiError> {
    Ok(Json(traderview_expense::section_4941::check(&b)))
}

// ── § 4942 taxes on PF failure to distribute income ──────────────────
// Mounted at /api/calc/section-4942. Pure compute; § 4942(a) 30%
// Tier-1 excise tax on undistributed income for each year/partial
// year deficiency remains uncorrected; § 4942(b) additional 100%
// Tier-2 if PF fails to make up deficient distribution within 90
// days of IRS notice. Distributable amount per § 4942(d) = minimum
// investment return (5% of non-charitable-use FMV under § 4942(e),
// reduced by acquisition indebtedness) reduced by § 4940 excise tax
// and UBI tax; must be paid as qualifying distributions by end of
// immediately following taxable year. Qualifying distributions per
// § 4942(g)(1)(A) = amount paid for § 170(c)(2)(B) religious/
// charitable/scientific/literary/educational/public purposes
// including reasonable and necessary administrative expenses, or
// § 4942(g)(1)(B) amount paid to acquire asset used directly in
// exempt purpose. § 4942(g)(2) set-asides for specific project
// payable within 60 months if suitability or cash distribution test
// satisfied. § 4942(h) treatment first out of prior-year UI then
// current year unless § 4942(h)(2) corpus election. § 4942(i) excess
// distributions carry forward FIVE years. Exceptions: § 4942(a)(2)(A)
// operating foundations § 4942(j)(3); § 4942(a)(2)(B) conduit
// foundations § 170(b)(1)(F)(ii); § 4942(j)(5) grandfathered PFs
// pre-May-27-1969. Distinct from § 4940 (iter 470) ANNUAL NII tax
// and § 4941 (iter 468) per-act self-dealing punitive — § 4942 is
// ANNUAL MINIMUM-DISTRIBUTION REQUIREMENT backed by 30% + 100%
// penalty. Original enactment Tax Reform Act of 1969 Pub. L. 91-172.

async fn section_4942_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_4942::Section4942Input>,
) -> Result<Json<traderview_expense::section_4942::Section4942Result>, ApiError> {
    Ok(Json(traderview_expense::section_4942::check(&b)))
}

// ── § 4943 taxes on PF excess business holdings ──────────────────────
// Mounted at /api/calc/section-4943. Pure compute; § 4943(a)(1) 10%
// Tier-1 excise tax on value of excess business holdings of private
// foundation as of date of greatest excess during taxable year;
// § 4943(b) 200% Tier-2 if not corrected within taxable period.
// Combined holding limits under § 4943(c)(2): § 4943(c)(2)(A) default
// 20% combined PF + DP voting stock of corporation (or equivalent
// profits interest in partnership/joint venture/unincorporated
// enterprise); § 4943(c)(2)(B) raised to 35% if PF establishes
// effective control of business is in non-DPs; § 4943(c)(2)(C) 2%
// de minimis — PF alone may hold up to 2% regardless of DPs.
// § 4943(c)(3)(B) non-voting stock — PF may hold ALL non-voting
// stock if combined DP voting holdings under applicable limit.
// Business enterprise per § 4943(d)(3) EXCLUDES: (A) functionally-
// related business substantially related to PF exempt purpose; (B)
// 95% passive income test trade or business with ≥ 95% gross income
// from interest + dividends + rents + royalties + capital gains; and
// § 4944(c) program-related investments. § 4943(c)(6) FIVE-YEAR
// disposition period for holdings acquired by gift/bequest/devise;
// § 4943(c)(7) IRS may grant additional 5-year (10-year total)
// extension for complex/unusual estates. § 4943(g) FAMILY BUSINESS
// EXCEPTION (added by Tax Cuts and Jobs Act 2017, Pub. L. 115-97,
// Dec 22 2017) permits 100% PF ownership of philanthropic business
// holding if ALL THREE: PF owns ALL voting stock at all times; PF
// received voting stock OTHER THAN by purchase; all net operating
// income distributed annually + no DP serves as director/officer/
// employee. Original enactment Tax Reform Act of 1969 Pub. L. 91-172.

async fn section_4943_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_4943::Section4943Input>,
) -> Result<Json<traderview_expense::section_4943::Section4943Result>, ApiError> {
    Ok(Json(traderview_expense::section_4943::check(&b)))
}

// ── § 4944 taxes on PF jeopardizing investments ──────────────────────
// Mounted at /api/calc/section-4944. Pure compute; § 4944(a)(1) 10%
// Tier-1 PF excise on amount of jeopardizing investment for each year
// or partial year in taxable period; § 4944(a)(2) 10% Tier-1 manager
// excise (knowing willful without reasonable cause) capped at $10,000
// per investment per § 4944(d)(2); § 4944(b)(1) 25% Tier-2 PF excise
// if not removed from jeopardy within taxable period; § 4944(b)(2) 5%
// Tier-2 manager excise (refuses correction) capped at $20,000.
// Jeopardizing investment standard = ordinary business care and
// prudence at TIME OF INVESTMENT (not hindsight) providing for long-
// term and short-term financial needs of PF to carry out exempt
// purposes; modern portfolio theory recognized. Categories typically
// scrutinized per 26 C.F.R. § 53.4944-1(a)(2): trading on margin +
// short sales + options/derivatives + futures/commodity + warrants +
// working interests oil/gas + land contracts + speculative private
// placements (NOT per se jeopardizing — facts and circumstances).
// § 4944(c) PROGRAM-RELATED INVESTMENT (PRI) EXCEPTION: NOT jeopardizing
// if ALL THREE — (1) primary purpose accomplishes § 170(c)(2)(B)
// charitable; (2) no significant income or appreciation purpose; (3)
// no political/lobbying purpose under § 4945(d)(1) or § 4945(d)(2).
// Distinction from § 4943 (iter 474): § 4943 limits concentration in
// single business enterprise; § 4944 evaluates prudence across
// portfolio. Original enactment Tax Reform Act of 1969 Pub. L. 91-172.

async fn section_4944_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_4944::Section4944Input>,
) -> Result<Json<traderview_expense::section_4944::Section4944Result>, ApiError> {
    Ok(Json(traderview_expense::section_4944::check(&b)))
}

// ── § 4945 taxes on PF taxable expenditures ──────────────────────────
// Mounted at /api/calc/section-4945. Pure compute; § 4945(a)(1) 20%
// Tier-1 PF excise on amount of taxable expenditure; § 4945(a)(2) 5%
// Tier-1 manager excise (knowingly agreed) capped at $10,000 per
// expenditure per § 4945(c)(2); § 4945(b)(1) 100% Tier-2 PF excise if
// not corrected within taxable period; § 4945(b)(2) 50% Tier-2 manager
// excise (refused correction) capped at $20,000. Five categories of
// taxable expenditures per § 4945(d): (1) § 4945(d)(1) influencing
// legislation/lobbying with § 4945(e) exceptions (nonpartisan analysis
// + technical advice + self-defense + employee communications); (2)
// § 4945(d)(2) influencing elections / voter registration with
// § 4945(f) five-condition safe harbor (501(c)(3)/(509(a)(1)-(3)) +
// substantial-all income + 85%+ non-DP support + 5+ state nonpartisan
// + non-earmarked); (3) § 4945(d)(3) grants to individuals without
// § 4945(g) advance IRS approval; (4) § 4945(d)(4) grants to
// organizations not § 509(a)(1)/(2)/(3) or § 4942(j)(3) operating
// foundation without § 4945(h) expenditure responsibility four-prong
// (pre-grant inquiry + written grant agreement + grantee reports +
// IRS Form 990-PF reports); (5) § 4945(d)(5) non-charitable
// expenditures outside § 170(c)(2)(B) charitable purposes.
// Distinction from § 4944 (iter 476): § 4944 evaluates prudence of
// INVESTMENTS (asset side); § 4945 evaluates propriety of
// EXPENDITURES (program side). Distinction from § 4941 (iter 468):
// § 4941 punishes self-dealing TRANSACTIONS between PF and DP; § 4945
// punishes expenditures outside permitted charitable purposes
// regardless of recipient relationship. Original enactment Tax Reform
// Act of 1969 Pub. L. 91-172.

async fn section_4945_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_4945::Section4945Input>,
) -> Result<Json<traderview_expense::section_4945::Section4945Result>, ApiError> {
    Ok(Json(traderview_expense::section_4945::check(&b)))
}

// ── § 4958 intermediate sanctions on excess benefit transactions ─────
// Mounted at /api/calc/section-4958. Pure compute; § 4958(a)(1)
// 25% excise tax on disqualified person who receives an excess
// benefit from a transaction with an applicable tax-exempt
// organization (ATEO); § 4958(b) additional 200% excise tax if
// not corrected within taxable period; § 4958(a)(2) 10% excise
// tax on knowing willful organization manager capped at $20K per
// transaction under § 4958(d)(2); § 4958(e) ATEO = § 501(c)(3)
// public charity (NOT private foundation — those use § 4941) +
// § 501(c)(4) social welfare + § 501(c)(29) qualified nonprofit
// health insurance issuer (added by ACA 2010) + 5-year look-back;
// § 4958(f)(1) disqualified person = substantial influence at
// any time during 5-year period including § 4958(f)(4) family +
// § 4958(f)(3) 35%-controlled entity; § 4958(f)(5) taxable
// period; § 4958(f)(6) correction via cash plus AFR interest;
// Treas. Reg. § 53.4958-6 rebuttable presumption of
// reasonableness three-prong safe harbor (advance approval by
// independent body + comparability data + contemporaneous
// documentation) shifts burden of proof to IRS; § 4961(b) 90-day
// post-assessment abatement for tier-2 200% tax; § 4958(c)(1)(A)
// excess benefit transaction = economic benefit exceeding
// consideration received including excessive compensation +
// bargain sale + above-market purchase + below-market loan +
// below-market rental + personal expense + automatic excess
// benefit per Treas. Reg. § 53.4958-4(c). Coordinate with § 4960
// (iter 464) ATEO 21% remuneration tax, not duplicative; original
// enactment Taxpayer Bill of Rights 2, Pub. L. 104-168 (July 30,
// 1996); PPA 2006 extended to donor-advised funds + supporting
// orgs.

async fn section_4958_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_4958::Section4958Input>,
) -> Result<Json<traderview_expense::section_4958::Section4958Result>, ApiError> {
    Ok(Json(traderview_expense::section_4958::check(&b)))
}

// ── § 4960 excise tax on excess tax-exempt org executive comp ────────
// Mounted at /api/calc/section-4960. Pure compute; § 4960(a) imposes
// 21% excise tax on applicable tax-exempt organization (ATEO) for sum
// of (i) remuneration paid by ATEO to covered employee exceeding
// $1,000,000 in the taxable year, plus (ii) any excess parachute
// payment paid by ATEO to covered employee; § 4960(c)(1) ATEO
// definition (§ 501(a) exempt org + § 521(b)(1) farmers coop +
// § 115(1) state/political subdivision instrumentality + § 527(e)(1)
// political org); § 4960(c)(2) covered employee — PRE-OBBBA regime
// (2018-2025): five highest-compensated employees for taxable year
// OR any preceding year beginning after 12/31/2016 forever-covered
// rule; POST-OBBBA regime (after 12/31/2025 per One Big Beautiful
// Bill Act, Pub. L. 119-21, July 4, 2025): five-employee cap REMOVED,
// any current or former employee over $1M triggers tax; § 4960(c)(3)
// remuneration = § 3401(a) wages excluding designated Roth plus
// § 457(f) vested deferred comp, EXCLUDES medical services by
// licensed medical professional (doctor, nurse, veterinarian) per
// § 4960(c)(3)(B); § 4960(c)(5) excess parachute payment modeled on
// § 280G — aggregate payments contingent on SEPARATION FROM
// EMPLOYMENT with present value ≥ 3× base amount triggers tax on
// amount EXCEEDING 1× base amount; § 4960(c)(7) coordination with
// § 162(m) $1M deduction cap (publicly held corporations); Final
// Treasury Regulations 26 C.F.R. § 53.4960-0 through § 53.4960-6
// effective January 15, 2021; original enactment Tax Cuts and Jobs
// Act, Pub. L. 115-97 (Dec. 22, 2017).

async fn section_4960_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_4960::Section4960Input>,
) -> Result<Json<traderview_expense::section_4960::Section4960Result>, ApiError> {
    Ok(Json(traderview_expense::section_4960::check(&b)))
}

// ── § 4975 prohibited transactions in IRA / qualified plans ──────────
// Mounted at /api/calc/section-4975. Pure compute; 15% standard +
// 100% non-correction excise tax on prohibited transactions between
// plan and disqualified person under 6 § 4975(c)(1) categories;
// § 4975(e)(2) 9-category disqualified person definition (including
// family § 4975(e)(6) = spouse + ancestor + lineal descendant +
// spouse of lineal descendant); § 408(e)(2) IRA disqualification
// triggers deemed distribution at FMV + § 72(t) 10% penalty if under
// 59½; § 4975(h) 90-day correction window for 100% tax abatement;
// DOL PTE 80-26 + 75-1 + 84-24 statutory exemptions.

async fn section_4975_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_4975::Section4975Input>,
) -> Result<Json<traderview_expense::section_4975::Section4975Result>, ApiError> {
    Ok(Json(traderview_expense::section_4975::check(&b)))
}

// ── § 4978 tax on certain dispositions by ESOPs ──────────────────────
// Mounted at /api/calc/section-4978. Pure compute; § 4978(a) 10%
// excise tax on amount realized on disposition of qualified securities
// by ESOP or eligible worker-owned cooperative within 3 years after
// acquisition under § 1042 sale or § 664(g) qualified gratuitous
// transfer. Tax paid by EMPLOYER that maintains the plan. § 4978(b)
// triggering conditions (either): (1) § 4978(b)(1) share count test —
// total employer securities held by plan after disposition less than
// total held immediately after § 1042 sale; (2) § 4978(b)(2) 30%-value
// test — value of qualified securities held after disposition less
// than 30% of total employer securities value at disposition (60% for
// § 664(g) qualified gratuitous transfer). § 4978(c) exceptions:
// § 4978(c)(1) distribution on separation from service / death /
// retirement / disability / divorce; § 4978(c)(2) employee stock
// purchase; § 4978(c)(3) merger or reorganization under § 354 +
// § 355 + § 356 + § 368 with ESOP retaining successor securities;
// § 4978(c)(4) diversification rights under § 401(a)(28)(B).
// Companion to § 1042 (iter 480): § 1042 provides seller capital
// gain deferral; § 4978 is employer recapture if ESOP fails 3-year
// hold. § 1042(b)(3) written consent to § 4978 recapture is
// prerequisite to seller § 1042 election. Form 5330 filing deadline
// last day of 7th month after employer tax-year-end. Original
// enactment Deficit Reduction Act of 1984 Pub. L. 98-369.

async fn section_4978_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_4978::Section4978Input>,
) -> Result<Json<traderview_expense::section_4978::Section4978Result>, ApiError> {
    Ok(Json(traderview_expense::section_4978::check(&b)))
}

// ── § 6166 estate tax installment for closely held business ──────────
// Mounted at /api/calc/section-6166. Pure compute; § 6166(a)(1)
// general rule allowing executor of estate where interest in closely
// held business exceeds 35% of adjusted gross estate to elect 14-year
// deferral (5 years interest-only + 10 years principal+interest);
// § 6166(a)(2) cross-references § 6166(b)(1) qualifying interests
// (sole proprietorship + partnership with 20%+ capital OR ≤45 partners
// + corporation with 20%+ voting stock OR ≤45 shareholders);
// § 6166(a)(3) election filing on timely Form 706 or amended return
// within 6 months of non-extended due date; § 6166(b)(6) adjusted
// gross estate = gross estate less § 2053 (debts/expenses/mortgages)
// and § 2054 (casualty losses) deductions; § 6166(f) 14-year deferral
// period; § 6601(j) subsidized 2% interest rate on first 2-percent
// portion ($1,830,000 multiplied by value of $1M ÷ applicable
// exclusion, 2024 indexed); § 6621 underpayment rate × 45% on excess;
// § 6166(g) acceleration events: § 6166(g)(1)(A) disposition of 50%+
// of decedent's interest + § 6166(g)(1)(B) withdrawal exceeding 50% +
// § 6166(g)(3) missed installment payment beyond 6-month grace +
// § 6166(g)(4) undistributed income must be applied to installments;
// § 303 stock redemption does NOT trigger acceleration. PV savings on
// $10M+ estate tax routinely exceed $2M-$3M. Distinct from § 6161
// general 12-month extension. Original enactment Tax Reform Act of
// 1976; amended Economic Growth and Tax Relief Reconciliation Act of
// 2001 Pub. L. 107-16.

async fn section_6166_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6166::Section6166Input>,
) -> Result<Json<traderview_expense::section_6166::Section6166Result>, ApiError> {
    Ok(Json(traderview_expense::section_6166::check(&b)))
}

// ── § 4980 Tax on Reversion of Qualified Plan Assets to Employer ─────
// Mounted at /api/calc/section-4980. Pure compute; § 4980(a) 20%
// base excise tax on amount of employer reversion from qualified
// retirement plan (defined benefit pension); § 4980(d)(1)
// increases rate to 50% unless employer satisfies § 4980(d)(2)
// qualified replacement plan (QRP) or § 4980(d)(3) pro rata
// benefit increase requirement; § 4980(d)(2) QRP three
// requirements (95% active participants + 25% direct transfer +
// 7-year ratable allocation); § 4980(d)(3) pro rata benefit
// increase 20%+ of maximum reversion with immediate effect on
// plan termination; § 4980(c) employer reversion = cash or FMV
// received as result of plan termination (excludes non-§ 404
// deductible contributions); § 4980(d)(4) qualified participant
// definition; stacks with corporate income tax for 70-75%
// effective combined rate at 50% rate or 45-50% at 20% rate; Rev.
// Rul. 2003-85 + PLR 9701036 confirm DB-to-DC transfer
// preferential treatment.

async fn section_4980_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_4980::Section4980Input>,
) -> Result<Json<traderview_expense::section_4980::Section4980Result>, ApiError> {
    Ok(Json(traderview_expense::section_4980::check(&b)))
}

// ── § 4980H Employer Shared Responsibility Payment (ESRP / ACA) ──────
// Mounted at /api/calc/section-4980h. Pure compute; 2026 amounts:
// § 4980H(a) failure-to-offer-MEC penalty $3,340 per FT minus 30
// (monthly $278.33); § 4980H(b) unaffordable or non-MV penalty
// $5,010 per FT receiving PTC (monthly $417.50); § 4980H(c)(2)
// ALE = 50+ FT employees (30+ hours/week) including § 4980H(c)(2)
// (E) FTE-equivalents; § 4980H(c)(2)(D) seasonal worker 120-day-
// or-fewer exception; § 4980H(c)(4) affordability 9.96% of
// household income (2026, up from 9.02% for 2025); § 36B(c)(2)(C)
// (ii) minimum value 60% of expected healthcare costs; § 4980H(d)
// MEC definition under § 5000A(f); § 6056 Form 1094-C transmittal
// + Form 1095-C employee statement by January 31; § 6721/§ 6722
// civil penalties up to $310 per return; § 414(b)/(c)/(m)/(o)
// controlled group aggregation for ALE threshold determination
// + each entity separately assessed ESRP; Pub. L. 111-148 PPACA
// + Pub. L. 111-152 HCERA enacting authority.

async fn section_4980h_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_4980h::Section4980hInput>,
) -> Result<Json<traderview_expense::section_4980h::Section4980hResult>, ApiError> {
    Ok(Json(traderview_expense::section_4980h::check(&b)))
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

// ── § 461(g) Prepaid Interest Deduction Timing ───────────────────────
// Mounted at /api/calc/section-461g. Pure compute; § 461(g)(1)
// cash-basis taxpayer must treat prepaid interest like accrual-
// basis (interest allocable to period after close of taxable year
// CHARGED TO CAPITAL ACCOUNT and deducted in period properly
// allocable); § 461(g)(2) EXCEPTION for points on principal
// residence purchase or improvement (5 conditions: principal
// residence purchase/improvement + secured by residence +
// established practice in area + not excessive + percentage of
// principal); Rev. Rul. 87-22 refinancing exclusion (points
// amortized over loan life); Rev. Rul. 70-540 rental property
// straight-line amortization; Rev. Proc. 94-27 seller-paid
// points treated as buyer-paid; interaction with § 163(d)
// investment interest (margin loan) + § 163(j) business
// interest + § 163(h) qualified residence interest + § 475(f)
// trader mark-to-market reclassification + § 263A UNICAP
// capitalization for construction-period interest.

async fn section_461g_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_461g::Section461gInput>,
) -> Result<Json<traderview_expense::section_461g::Section461gResult>, ApiError> {
    Ok(Json(traderview_expense::section_461g::check(&b)))
}

// ── § 457(b) Governmental and Tax-Exempt Deferred Compensation ───────
// Mounted at /api/calc/section-457b. Pure compute; 2026 limits:
// elective deferral § 457(b)(2) $24,500; age-50 catch-up § 414(v)
// $8,000 (GOVERNMENTAL ONLY); ages-60-63 SECURE 2.0 § 109 enhanced
// catch-up $11,250 (GOVERNMENTAL ONLY); § 457(b)(3) special 3-year
// pre-retirement catch-up = lesser of 2× annual limit ($49,000) or
// underutilized prior-year limitation (BOTH governmental + tax-
// exempt). Two plan types: GOVERNMENTAL (§ 457(g) trust, no § 72(t)
// 10% penalty, rollovers permitted) vs TAX-EXEMPT (unfunded top-
// hat, substantial credit risk in employer bankruptcy, § 72(t)
// applies, rollovers not permitted). § 402(g)(1) NON-AGGREGATION
// rule allows DOUBLE DEFERRAL with § 401(k)/§ 403(b) ($49,000 in
// 2026). § 457(b)(3) ANTI-STACKING with § 414(v) catch-up
// (participant must choose ONE catch-up mechanism per year).

async fn section_457b_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_457b::Section457bInput>,
) -> Result<Json<traderview_expense::section_457b::Section457bResult>, ApiError> {
    Ok(Json(traderview_expense::section_457b::check(&b)))
}

// ── § 457A Nonqualified Deferred Compensation From Tax Indifferent Parties ──
// Mounted at /api/calc/section-457a (iter 494). Pure compute. EESA 2008
// anti-deferral provision targeting US hedge-fund / PE managers operating
// through Cayman / BVI master-feeder structures. Computes (1) nonqualified-
// entity classification per § 457A(b) (foreign corp w/o comprehensive
// foreign tax or ECI; partnership w/ substantially-all tax-indifferent
// allocations); (2) substantial-risk-of-forfeiture test under § 457A(d)(2)
// (future-performance-of-substantial-services standard, stricter than §
// 409A); (3) immediate-inclusion when SROF absent OR amount-not-determinable
// 20% additional tax + AFR + 1% interest under § 457A(c)(1)(B)(i)–(ii);
// (4) pre-2009 transition rule per § 457A(e) (must include by last tax year
// before 2017 per Notice 2009-8); (5) § 409A-exempt stock-right safe harbor
// per Notice 2009-8 Q&A 2 + Rev. Rul. 2014-18. Coordinates with § 409A
// (applies IN ADDITION per § 457A(d)(4)), § 457(b) (governmental NQDC), §
// 280G (golden parachutes on change in control).

async fn section_457a_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_457a::Section457aInput>,
) -> Result<Json<traderview_expense::section_457a::Section457aResult>, ApiError> {
    Ok(Json(traderview_expense::section_457a::check(&b)))
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

// ── §248 corporate organizational expenditures ──────────────────────
// Mounted at /api/calc/section-248. Parallel to § 195 startup
// expenditures with corporation-specific terminology. § 248(a)
// election yields lesser of $5,000 first-year deduction OR ($5,000 -
// max(0, org_costs - $50,000)) phase-out + 180-month amortization of
// remainder beginning month corporation begins business. § 248(b)
// organizational expenditure defined; Treas. Reg. § 1.248-1(b)
// excludes expenses for issuing/selling shares + § 351 transfer
// expenses + § 368 reorganization expenses. § 248(c) automatic
// election deemed per T.D. 9542 (Sept. 8, 2011). AJCA 2004 § 902
// harmonized § 248 with § 195 / § 709 (cross-reference modules).
// Trader-relevant when forming a C-corporation for trading
// operations.

async fn section_248_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_248::Section248Input>,
) -> Result<Json<traderview_expense::section_248::Section248Result>, ApiError> {
    if b.total_organizational_expenditures_cents < -10_000_000_000
        || b.total_organizational_expenditures_cents > 10_000_000_000_000
    {
        return Err(ApiError::BadRequest(
            "total_organizational_expenditures_cents out of plausible range".into(),
        ));
    }
    if b.months_active_in_first_year > 12 {
        return Err(ApiError::BadRequest(
            "months_active_in_first_year must be 0..=12".into(),
        ));
    }
    Ok(Json(traderview_expense::section_248::compute(&b)))
}

// ── §709 partnership organizational expenditures + syndication ──────
// Mounted at /api/calc/section-709. Parallel to § 195 + § 248 with
// partnership-specific terminology. § 709(b)(1) $5K first-year
// deduction + $50K phase-out floor + $55K ceiling + 180-month
// amortization. § 709(b)(2) organizational expense defined. § 709(b)
// (3) SYNDICATION EXPENSES (brokerage + registration + legal/
// accounting fees for prospectus + printing) PERMANENTLY CAPITALIZED
// to partner basis with NO amortization — DISTINCT from § 248 which
// only excludes share-issuance expenses. Treas. Reg. § 1.709-2(a)
// organizational definition; § 1.709-2(b) syndication definition.
// T.D. 9542 (Sept. 8, 2011) automatic election. AJCA 2004 § 902
// harmonization with § 195 / § 248.

async fn section_709_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_709::Section709Input>,
) -> Result<Json<traderview_expense::section_709::Section709Result>, ApiError> {
    if b.total_organizational_expenses_cents < -10_000_000_000
        || b.total_organizational_expenses_cents > 10_000_000_000_000
    {
        return Err(ApiError::BadRequest(
            "total_organizational_expenses_cents out of plausible range".into(),
        ));
    }
    if b.total_syndication_expenses_cents < -10_000_000_000
        || b.total_syndication_expenses_cents > 10_000_000_000_000
    {
        return Err(ApiError::BadRequest(
            "total_syndication_expenses_cents out of plausible range".into(),
        ));
    }
    if b.months_active_in_first_year > 12 {
        return Err(ApiError::BadRequest(
            "months_active_in_first_year must be 0..=12".into(),
        ));
    }
    Ok(Json(traderview_expense::section_709::compute(&b)))
}

// ── §197 amortization of goodwill and certain other intangibles ─────
// Mounted at /api/calc/section-197. § 197(a) 15-year (180-month)
// straight-line amortization beginning month acquired for any
// "amortizable section 197 intangible" — § 197(d) nine categories
// (goodwill, going concern value, workforce in place, books and
// records, patent/copyright/process, customer or supplier intangibles,
// government license, covenant not to compete, franchise/trademark/
// trade name). § 197(e) three exceptions covered (land, financial
// interest, lease of tangible property). § 197(c) requires post-
// August-10-1993 acquisition + trade-or-business use. § 197(f)(9)
// anti-churning bars amortization when intangible held during
// 7/25/1991-8/10/1993 transition by taxpayer or related (>20%) party,
// OR acquired from related party with continued use. § 197(b) bars
// any § 167 depreciation deduction. Trader-relevant when acquiring a
// trading business (customer list, workforce, goodwill, non-compete
// with seller).

async fn section_197_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_197::Section197Input>,
) -> Result<Json<traderview_expense::section_197::Section197Result>, ApiError> {
    if b.adjusted_basis_cents < -10_000_000_000
        || b.adjusted_basis_cents > 10_000_000_000_000
    {
        return Err(ApiError::BadRequest(
            "adjusted_basis_cents out of plausible range".into(),
        ));
    }
    if b.months_held_since_acquisition > 100_000 {
        return Err(ApiError::BadRequest(
            "months_held_since_acquisition looks invalid (>100000)".into(),
        ));
    }
    Ok(Json(traderview_expense::section_197::compute(&b)))
}

// ── §199A Qualified Business Income (QBI) deduction ─────────────────
// Mounted at /api/calc/section-199a. § 199A(a) basic 20% deduction =
// LESSER of (1) 20% × QBI (combined QBI amount) or (2) 20% × (Taxable
// Income − Net Capital Gain). § 199A(b)(2) W-2 wage / UBIA phase-in
// limitation applies when TI exceeds threshold: limits 20% × QBI to
// GREATER of (a) 50% × W-2 wages or (b) 25% × W-2 wages + 2.5% ×
// UBIA. § 199A(e)(2) 2026 thresholds: Single / HoH $201,750 phase-in
// begin / $276,750 phase-out complete; MFJ / QSS $403,500 / $553,500.
// § 199A(d)(2) SSTB phases out completely above upper threshold.
// OBBBA 2025 (Pub. L. 119-21) made § 199A PERMANENT, expanded phase-
// in window from $50K → $75K single / $100K → $150K joint, and added
// $400 minimum deduction when QBI ≥ $1,000 + material participation.
// Rev. Proc. 2019-38 — rental real estate safe harbor (250+ hours/yr
// + separate books + contemporaneous records) treats rental as trade
// or business for § 199A. Trader-critical for traders with § 475(f)
// MTM election + trader-landlords with rental real estate qualifying
// as trade or business. IRS Form 8995 / 8995-A.

async fn section_199a_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_199a::Section199AInput>,
) -> Result<Json<traderview_expense::section_199a::Section199AResult>, ApiError> {
    Ok(Json(traderview_expense::section_199a::check(&b)))
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

// ── §7623 IRS whistleblower awards (Tax Relief 2006 / BBA 2018 / TFA 2019) ─
// Mounted at /api/calc/section-7623. § 7623 framework spans 1867
// discretionary regime (§ 7623(a)) + 2006 Tax Relief and Health Care
// Act § 406 mandatory 15-30% regime (§ 7623(b)) + 2018 Bipartisan
// Budget Act § 41108 broadened "collected proceeds" definition
// (§ 7623(c)) + 2019 Taxpayer First Act § 1405 anti-retaliation
// protections (§ 7623(d)). Mandatory thresholds (§ 7623(b)(5)):
// amount in dispute > $2,000,000 AND if individual, gross income
// > $200,000. Public-information-based awards capped at 10%
// (§ 7623(b)(2)(A)); planned/initiated noncompliance reduces award
// (§ 7623(b)(2)(B)); criminal conviction arising from role denies
// award entirely (§ 7623(b)(3)). § 7623(b)(4) Tax Court appeal
// within 30 days. § 7623(c) "collected proceeds" includes criminal
// fines, civil forfeitures, and FBAR penalties under 31 USC § 5321.
// § 7623(d) remedies: reinstatement, DOUBLE back pay with interest,
// special damages, attorney fees. Trader-relevant: wealthy/
// sophisticated traders are precisely the IRS Whistleblower Office
// target taxpayer class — high gross income + complex tax positions
// makes them disproportionately exposed to whistleblower tips from
// disgruntled fund employees, ex-spouses, business partners, or
// accountants. Sibling cluster: § 6663 + § 7201 + § 7202 + § 7206 +
// § 6701 + § 7430 + § 6038D + § 6111 + § 6112.

async fn section_7623_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7623::Section7623Input>,
) -> Result<Json<traderview_expense::section_7623::Section7623Result>, ApiError> {
    if b.award_percentage_bps > 10_000 {
        return Err(ApiError::BadRequest(
            "award_percentage_bps must be ≤ 10000 (100%)".into(),
        ));
    }
    if b.days_to_tax_court_appeal > 36_500 {
        return Err(ApiError::BadRequest(
            "days_to_tax_court_appeal out of range".into(),
        ));
    }
    Ok(Json(traderview_expense::section_7623::check(&b)))
}

// ── §7405 IRS action for recovery of erroneous refunds ─────────────
// Mounted at /api/calc/section-7405. § 7405 is the IRS-side reverse
// mechanism to § 7422 (taxpayer-initiated refund suit). § 7405(a)
// recovers refunds erroneous within meaning of § 6514; § 7405(b)
// reaches refunds outside § 6514 scope. § 7405(d) statute of
// limitations — 2 years (730 days) from making of refund standard;
// 5 years (1825 days) if refund induced by fraud or misrepresentation
// of material fact. IRS burden of proof per IRM 5.17.4 + case law:
// (1) refund was erroneous; (2) amount of refund; (3) taxpayer
// received or benefited. Jurisdiction: district court (concurrent
// with Court of Federal Claims under 28 USC § 1346(a)(1)). Trader-
// relevant when IRS issues refund (e.g., NOL carryback via § 475(f)
// MTM election) and later determines computation was erroneous.

async fn section_7405_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7405::Section7405Input>,
) -> Result<Json<traderview_expense::section_7405::Section7405Result>, ApiError> {
    Ok(Json(traderview_expense::section_7405::check(&b)))
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

// ── § 1042 sales of stock to ESOPs or certain cooperatives ───────────
// Mounted at /api/calc/section-1042. Pure compute; § 1042(a) long-term
// capital gain on sale of qualified securities of domestic C
// corporation to ESOP is RECOGNIZED ONLY TO THE EXTENT amount realized
// exceeds cost of qualified replacement property (QRP) purchased
// during 15-month replacement period (3 months before sale + 12 months
// after, per § 1042(c)(6)). Five eligibility requirements per
// § 1042(b): (1) § 1042(b)(1) 3-year seller holding period; (2)
// § 1042(b)(2) ESOP must own 30%+ of each class of outstanding stock
// immediately after sale; (3) § 1042(b)(3) written consent to § 4978
// recapture (10% excise on employer if ESOP disposes within 3 years);
// (4) § 1042(b)(4) corporation must be DOMESTIC C CORP (S corps NOT
// eligible); (5) § 1042(c)(1)(B) qualified securities — not received
// via § 83 compensation / § 422 ISO / § 423 ESPP exercise, not
// readily tradable on established securities market. § 1042(c)(3)
// QRP categories: common stock + preferred + bonds + convertible
// floating-rate notes of domestic operating corporations; EXCLUDED
// are US government securities, non-US securities, domestic
// subsidiaries of non-US parents, FDIC CDs, mutual funds + money-
// market funds, and securities of the ESOP corporation. § 1042(d)
// basis = QRP cost reduced by non-recognized gain. § 1042(e)
// disposition recapture. § 1014 basis step-up at death permanently
// eliminates deferred gain — making § 1042 + estate planning among
// the most powerful trader-founder wealth-transfer strategies.
// Distinction from § 1031 like-kind exchange (real property only) and
// § 1045 QSBS rollover (60-day window). Original enactment Tax Reform
// Act of 1984 Pub. L. 98-369.

async fn section_1042_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1042::Section1042Input>,
) -> Result<Json<traderview_expense::section_1042::Section1042Result>, ApiError> {
    Ok(Json(traderview_expense::section_1042::check(&b)))
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

// ── §1058 securities loan non-recognition ─────────────────────────────
// Mounted at /api/calc/section-1058. Pure compute; § 1058(a) provides
// non-recognition treatment for securities loans satisfying four-prong
// § 1058(b) qualification test: (1) return identical securities; (2)
// dividend-equivalent payments to transferor during loan period; (3)
// risk of loss / opportunity for gain preserved; (4) terminable on
// demand per Treas. Reg. § 1.1058-1 + Rev. Proc. 2008-63 (5 business
// days). § 1058(a)(2) holding period tacking — loan period adds to
// transferor's holding period in returned securities. § 1058(c)
// "securities" definition = § 1236(c). Anshutz v. Commissioner, 135
// T.C. No. 5 (2010) + Calloway v. Commissioner, 135 T.C. No. 3 (2010)
// — variable prepaid forward contract bundled with stock loan FAILS
// § 1058(b)(3). Failure consequence: TAXABLE SALE at FMV + basis reset
// + holding period restart + potential § 1259 constructive sale.
// Trader-critical for Interactive Brokers SYEP + Robinhood Securities
// Lending + Schwab SLFPS + TD Ameritrade Securities Lending + hedge
// fund prime brokerage stock-loan + short seller's borrow (lender
// side). Companion: § 1259 (constructive sales), § 1092 (straddles),
// § 1236(c), § 475.

async fn section_1058_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1058::Section1058Input>,
) -> Result<Json<traderview_expense::section_1058::Section1058Result>, ApiError> {
    Ok(Json(traderview_expense::section_1058::check(&b)))
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

// ── §1291 PFIC default excess distribution + interest charge ─────────
// Mounted at /api/calc/section-1291. § 1291(a)(1)(A) excess
// distribution allocated RATABLY to each day in shareholder's
// holding period; § 1291(a)(1)(B) current-year + pre-PFIC-period
// portion taxed as ordinary; § 1291(a)(1)(C) intermediate-PFIC-year
// portion creates deferred tax at HIGHEST MARGINAL RATE for that
// year + § 6621 interest charge compounded DAILY. § 1291(a)(2)
// disposition gain converted to ordinary + interest charge.
// § 1291(b)(2)(A) excess = > 125% of 3-year average distributions;
// § 1291(b)(3)(B) FIRST YEAR all distributions excess.
// § 1291(d)(1) disabled by QEF election; § 1291(d)(2) purging
// election available; § 1291(f) disabled by mark-to-market.
// § 1291(g) § 988 currency translation. Trader-critical for
// foreign mutual funds + ETFs + hedge fund LP interests +
// offshore insurance products. Companion: § 1297 (PFIC definition)
// + § 1298 (special rules) + § 1295 (QEF election) + § 1296
// (mark-to-market) + § 6621 (underpayment interest rate) + Form
// 8621. Enacted by Tax Reform Act of 1986 § 1235 (Pub. L. 99-514,
// October 22, 1986); HIRE Act of 2010 § 521 added § 1298(f) annual
// reporting requirement.

async fn section_1291_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1291::Section1291Input>,
) -> Result<Json<traderview_expense::section_1291::Section1291Result>, ApiError> {
    if b.current_year_marginal_rate_bps > 10_000
        || b.prior_year_highest_marginal_rate_bps > 10_000
    {
        return Err(ApiError::BadRequest(
            "marginal rate bps must be ≤ 10000 (100%)".into(),
        ));
    }
    Ok(Json(traderview_expense::section_1291::check(&b)))
}

// ── §1293 PFIC QEF current-taxation mechanic ─────────────────────────
// Mounted at /api/calc/section-1293. § 1293(a)(1)(A) pro-rata
// inclusion of ORDINARY EARNINGS as ordinary income; § 1293(a)(1)(B)
// pro-rata inclusion of NET CAPITAL GAIN as LONG-TERM capital gain
// (CHARACTER PRESERVED regardless of shareholder holding period).
// § 1293(b)(1) ordinary earnings = E&P minus net capital gain;
// § 1293(b)(2) net capital gain per § 1222(11) (LT gain - ST loss).
// § 1293(c) pro rata share = daily-ratable distribution. § 1293(d)(1)
// basis INCREASED by inclusion (prevents double tax); § 1293(d)(2)
// basis DECREASED by PTI distribution. § 1293(e) coordinates with
// § 951 subpart F via § 1297(d) PFIC-CFC overlap rule. § 1293(f) +
// § 1294 deferral election (rarely used due to interest charge).
// Treas. Reg. § 1.1295-1(g) PFIC Annual Information Statement
// required for QEF election validity + ordinary/LTCG split.
// § 1(h)(11) qualified dividend treatment may apply for qualified
// foreign corporation status. Form 8621 + § 1298(f) annual
// reporting. Trader-critical for foreign-fund holders who elected
// QEF to escape § 1291 punitive regime. Completes PFIC framework
// cluster: § 1291 + § 1293 + § 1295 + § 1296 + § 1297 + § 1298.
// Sibling cluster: § 1222(11) + § 1294 + § 1297(d) + § 951 +
// § 1(h)(11) + Form 8621.

async fn section_1293_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1293::Section1293Input>,
) -> Result<Json<traderview_expense::section_1293::Section1293Result>, ApiError> {
    if b.pro_rata_share_bps > 10_000 {
        return Err(ApiError::BadRequest(
            "pro_rata_share_bps must be ≤ 10000 (100%)".into(),
        ));
    }
    Ok(Json(traderview_expense::section_1293::check(&b)))
}

// ── §1294 QEF election to extend time for payment of tax ─────────────
// Mounted at /api/calc/section-1294. § 1294(a)(1) U.S. shareholder of
// QEF MAY ELECT to extend time for payment of tax attributable to
// share of UNDISTRIBUTED EARNINGS. § 1294(b) undistributed earnings
// = § 1293(a) includible amount - distributions - disposed-stock
// portion. § 1294(c) § 6601 interest accrues at § 6621 quarterly
// underpayment rate, compounded DAILY per § 6622, must be paid on
// termination. § 1294(d)(1) election UNAVAILABLE if § 551 foreign
// personal holding company rules engaged; § 1294(d)(2) election
// UNAVAILABLE if § 951 subpart F CFC rules engaged (§ 1297(d)
// PFIC-CFC overlap rule resolves). § 1294(e) termination upon
// EARLIEST of (1) distribution reducing undistributed earnings;
// (2) QEF stock disposition; (3) affirmative termination; (4) death
// of individual shareholder; (5) QEF ceases to be QEF; (6)
// shareholder ceases to be U.S. person. Treas. Reg. § 1.1294-1T
// temporary regulations. Form 8621 annual election + reporting.
// RARELY USED in practice due to interest charge accrual + complex
// reporting. Completes PFIC framework cluster: § 1291 + § 1293 +
// § 1294 + § 1295 + § 1296 + § 1297 + § 1298. Sibling cluster:
// § 6601 + § 6621 + § 6622 + Form 8621 + § 551 + § 951 + § 1297(d).

async fn section_1294_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1294::Section1294Input>,
) -> Result<Json<traderview_expense::section_1294::Section1294Result>, ApiError> {
    if b.tax_rate_on_undistributed_bps > 10_000
        || b.section_6601_interest_rate_bps > 10_000
    {
        return Err(ApiError::BadRequest(
            "rate bps must be ≤ 10000 (100%)".into(),
        ));
    }
    Ok(Json(traderview_expense::section_1294::check(&b)))
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

// ── § 280G Golden Parachute Payments + § 4999 20% recipient excise ───
// Mounted at /api/calc/section-280g. Pure compute; § 280G(a) denies
// employer compensation deduction for "excess parachute payment" to
// "disqualified individual" "contingent on change in ownership or
// control"; § 280G(b)(1) parachute = aggregate present value ≥ 3×
// base amount triggers CLIFF on entire excess over 1× base (not
// just over 3× portion); § 280G(b)(3) base amount = 5-year
// annualized includible compensation; § 280G(c) disqualified
// individual = (1) officer (max 50 employees regardless of title);
// (2) 1%+ shareholder; (3) highly compensated (top 1% or top 250);
// § 280G(b)(2)(A) change in control = > 50% ownership / 35% voting
// in 12 months / majority board in 12 months / 40%+ asset
// acquisition; § 280G(b)(5) small business exception = private
// corp + (S election or > 75% shareholder vote with adequate
// disclosure cleansing vote); § 280G(b)(4) reasonable compensation
// for post-change services exception with clear and convincing
// evidence burden; § 4999 20% recipient excise tax on excess
// parachute payment; gross-up vs modified-cutback structures.

async fn section_280g_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_280g::Section280gInput>,
) -> Result<Json<traderview_expense::section_280g::Section280gResult>, ApiError> {
    Ok(Json(traderview_expense::section_280g::check(&b)))
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

// ── § 1366 S-corp pass-thru of items to shareholders ─────────────────
// Mounted at /api/calc/section-1366. Pure compute; cornerstone S-corp
// pass-through provision under § 1366(a)(1) — every shareholder reports
// pro rata share of (A) separately-stated items that could affect tax
// liability differently (capital gains/losses + § 1231 + charitable
// contributions + dividend income + tax-exempt interest + foreign tax
// credit + investment interest expense + § 179 expense + AMT
// preferences + § 199A QBI deduction + § 1411 NII items) and (B)
// non-separately-stated ordinary trade or business income/loss.
// § 1366(b) character flow-through: items treated by shareholder as
// if generated at shareholder level. § 1366(d)(1) three-tier loss
// limitation: § 1366(d)(1)(A) basis cap = adjusted basis in stock +
// adjusted basis in indebtedness; § 465 at-risk limitation; § 469
// passive activity loss limitation. § 1366(d)(2) suspended losses
// carry over indefinitely. § 1366(d)(3) post-termination transition
// period (1 year or 120 days after IRS notice). § 1366(e) family
// group reasonable compensation (IRS reallocation tool). § 1366(f)
// adjustment for § 1374 built-in gains tax and § 1375 passive
// investment income tax paid by S corp. Three-tier ordering per
// 26 C.F.R. § 1.1366-2: basis → at-risk → passive. Distinction from
// § 702 partnership pass-through (partners allow § 704(b) basis
// tracking and § 704 special allocations; S corp requires single-
// class-of-stock under § 1361(b)(1)(D)). Original framework Tax
// Reform Act of 1982 Subchapter S Revision Pub. L. 97-354.

async fn section_1366_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1366::Section1366Input>,
) -> Result<Json<traderview_expense::section_1366::Section1366Result>, ApiError> {
    Ok(Json(traderview_expense::section_1366::check(&b)))
}

// ── § 1377 S-corp pro rata + terminating election + PTTP ─────────────
// Mounted at /api/calc/section-1377. § 1377(a)(1) pro rata share
// general rule — DAILY ASSIGNMENT method: each shareholder's pro rata
// share determined by assigning equal portion of any S corp item to
// each day of taxable year, then dividing pro rata among shares
// outstanding on that day. Special-day rules per 26 C.F.R.
// § 1.1377-1(a)(2): disposing shareholder treated as shareholder for
// day of disposition; deceased shareholder for day of death.
// § 1377(a)(2) TERMINATING ELECTION ('closing of the books'): if
// shareholder's entire interest terminates AND all affected
// shareholders consent, corporation may elect to apply § 1377(a)(1)
// AS IF taxable year consisted of two separate years, first ending
// on termination date. Eligibility: § 1377(a)(2)(A) full disposition
// (sale + exchange + gift) OR § 1377(a)(2)(B) § 302 or § 303
// redemption + all-affected-shareholder consent + timely Form 1120-S
// attached statement. § 1377(b) POST-TERMINATION TRANSITION PERIOD
// (PTTP) definitions: § 1377(b)(1)(A) 1-year period after S corp
// ceases; § 1377(b)(1)(B) 120-day determination period; § 1377(b)(1)(C)
// 120-day E&P determination period. § 1377(b)(2) distribution during
// PTTP treated as reducing AAA under § 1368(c) first (tax-free), then
// E&P (dividend treatment). § 1377(b)(3) determination definitions
// (§ 1313(a)(1) IRS or court determination + Secretary determination
// + corporation-Secretary agreement). Distinction from § 706
// partnership: partnerships use varying interest rules under
// § 706(d) (interim closing or proration); S corps restricted to
// § 1377(a)(1) daily method unless § 1377(a)(2) election. Current
// framework Subchapter S Revision Act of 1982 Pub. L. 97-354.

async fn section_1377_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1377::Section1377Input>,
) -> Result<Json<traderview_expense::section_1377::Section1377Result>, ApiError> {
    Ok(Json(traderview_expense::section_1377::check(&b)))
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

// ── §1411 Net Investment Income Tax (NIIT) 3.8% surtax ──────────────
// Mounted at /api/calc/section-1411. § 1411(a)(1) tax = 3.8% × LESSER
// of net investment income (NII) or excess of MAGI over applicable
// threshold. § 1411(b) MAGI thresholds (NOT indexed; same since 2013
// ACA enactment): Single/HoH $200,000; MFJ/QSS $250,000; MFS $125,000.
// § 1411(c)(1) NII categories: interest + dividends + capital gains +
// passive rental income + royalties + non-qualified annuity income;
// § 1411(c)(1)(B) deductions for investment expenses + state tax;
// § 1411(c)(2) trade or business carve-outs for material participation;
// § 1411(c)(5) qualified retirement plan distributions EXCLUDED.
// § 469(c)(7) real estate professional carve-out — if taxpayer
// performs ≥ 750 hours per year in real property trades AND > 50% of
// personal services in real property, rental income may be treated as
// ACTIVE and excluded from NII. Pub. L. 119-21 OBBBA 2025 did NOT
// modify § 1411; 3.8% rate + thresholds + categories + retirement-
// plan exception remain identical to 2013 form. Trader-critical for
// any high-income trader. IRS Form 8960 (2025); IRS Topic 559.

async fn section_1411_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1411::Section1411Input>,
) -> Result<Json<traderview_expense::section_1411::Section1411Result>, ApiError> {
    Ok(Json(traderview_expense::section_1411::check(&b)))
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

// ── § 56A Corporate Alternative Minimum Tax (CAMT) ─────────────────
// Mounted at /api/calc/section-56a (iter 498). Pure compute. IRA 2022
// Pub. L. 117-169 § 10101; 15% minimum tax on adjusted financial statement
// income (AFSI) for applicable corporations effective for taxable years
// beginning after December 31, 2022. § 59(k)(1) applicable-corporation
// test: corporation (not S corp / RIC / REIT) with three-year-average AFSI
// exceeding $1B for years ending after December 31, 2021. § 59(k)(2)
// FPMG (Foreign-Parented Multinational Group) aggregation: US-resident
// member's AFSI test includes ALL FPMG members plus § 52 single-employer
// aggregation; US member applicable when FPMG aggregate > $1B AND US
// member's own three-year-average AFSI >= $100M safe-harbor floor.
// § 56A(c) sixteen AFSI adjustments to GAAP/IFRS book net income (federal
// tax back-out, defined benefit pension, qualified depreciation via § 168
// not book, cooperative dividends, CFC distributions, wholly-owned
// disregarded entity, consolidated tax group, etc.). § 56A(d) FSNOL
// limited to 80% of AFSI parallel to § 172 regular-tax NOL. § 38(c)(6)(E)
// general business credit usable against CAMT up to 75% of tentative
// minimum tax. § 53(c)-(d) CAMT credit carryforward indefinite against
// future regular tax. Form 4626 attached to Form 1120. Coordination with
// § 4501 (1% stock buyback excise — same IRA 2022 package), § 481
// (accounting method change AFSI restatement), § 55 (general AMT
// framework), § 53 (AMT credit), § 38 (general business credit), § 59A
// (BEAT — inbound FPMG members).

async fn section_56a_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_56a::Section56aInput>,
) -> Result<Json<traderview_expense::section_56a::Section56aResult>, ApiError> {
    Ok(Json(traderview_expense::section_56a::check(&b)))
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

// ── §67(g) TCJA misc itemized deduction suspension ────────────────────
// Mounted at /api/calc/section-67g. § 67(g) added by Tax Cuts and Jobs
// Act of 2017 § 11045 (Pub. L. 115-97, December 22, 2017); originally
// scheduled to sunset after December 31, 2025; One Big Beautiful Bill
// Act of 2025 (H.R. 1, signed July 4, 2025) made § 67(g) PERMANENT.
// Trader-critical because § 67(g) is the single most important reason
// traders ELECT trader status under § 475(f) — without trader status,
// investment expenses (advisory fees, custody fees, subscription fees,
// home office, trading platform fees, education, travel) become NON-
// DEDUCTIBLE under § 67(g) because they are § 212 expenses subject to
// the suspended 2%-of-AGI floor. With § 475(f) trader status, expenses
// qualify as § 162 trade-or-business deductions on Schedule C and
// ESCAPE § 67(g) suspension entirely. § 67(b) exempts 12 categories
// (§ 163 interest + § 164 taxes + § 165(a) casualty + § 170 charity +
// § 213 medical + § 691(c) IRD + § 215 alimony + § 217 moving (armed
// forces) + § 1341 claim of right + gambling losses + § 642(c)
// trust/estate charity + § 7702B(a)(2) qualified long-term care).
// § 67(e) preserves estate/trust administration expense deductibility
// per Treas. Reg. § 1.67-4 + IRS Notice 2018-61. Companion to § 475(f)
// (trader election) + § 162 (Schedule C) + § 212 (production-of-income)
// + § 1411 (NIIT 3.8% surtax — also exempts trader business income).

async fn section_67g_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_67g::Section67gInput>,
) -> Result<Json<traderview_expense::section_67g::Section67gResult>, ApiError> {
    Ok(Json(traderview_expense::section_67g::check(&b)))
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

// ── §6042 returns regarding payments of dividends (1099-DIV) ────────
// Mounted at /api/calc/section-6042. § 6042(a)(1) — every person who
// makes dividend payments aggregating $10 or more (or who receives
// as nominee) shall make a return. § 6042(b) — dividend defined per
// § 316 (corporate distributions out of E&P) + § 852 RIC + § 857
// REIT + stockbroker substitute payments; EXCLUDES exempt-interest
// dividends (§ 852(b)(5)) and § 3406 backup-withheld amounts.
// § 6042(c) — written statement to recipient by January 31. § 6042
// (d)(1) substitute dividend payments by broker on short sales
// reportable; § 6042(d)(2) uncertain payments rule — entire amount
// treated as dividend. Form 1099-DIV box breakdown: 1a ordinary +
// 1b qualified (§ 1(h)(11) 60-day holding period) + 2a-d capital
// gain (including § 1202 QSBS + § 1250 unrecaptured) + 3 return of
// capital + 5 § 199A REIT/PTP (20% deduction) + 8 foreign tax (§
// 901 FTC) + 12 exempt-interest + 13 specified private activity
// bond (AMT). Trader-critical: § 1411 NIIT 3.8% on dividends + §
// 988 foreign currency ADR conversions. Companion to § 6041 + §
// 6045 + § 6049 + § 6050W + § 3406 + § 1(h)(11) + § 199A + § 1411
// + § 1202 + § 988 + § 901.
async fn section_6042_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6042::Section6042Input>,
) -> Result<Json<traderview_expense::section_6042::Section6042Result>, ApiError> {
    Ok(Json(traderview_expense::section_6042::check(&b)))
}

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

// ── §6049 returns regarding payments of interest (1099-INT/OID) ─────
// Mounted at /api/calc/section-6049. § 6049(a) — every person who
// makes interest payments aggregating $10 or more (or who receives
// as nominee) shall make a return. § 6049(b) — interest defined:
// registered-form obligations + bank deposits + savings institution
// interest + insurance company-held + OID per § 1272 + broker-dealer
// custodial + Treasury obligations + municipal bond tax-exempt (§
// 103(a)). § 6049(c) — written statement to recipient by January 31.
// § 6049(d) — nominee/middleman pass-through; broker (§ 6045(c)) is
// middleman. § 6049(e) — backup withholding under § 3406 triggers
// reporting IRRESPECTIVE of $10 threshold. Form 1099-INT + Form 1099-
// OID. Trader-relevant sources: Treasury (T-bills + TIPS + Series I
// — federal tax/state-exempt); municipal bonds (federal tax-exempt
// § 103(a)); corporate bonds (taxable); zero-coupon (OID); money
// market funds; bank deposit; brokerage cash-balance; § 988 foreign
// currency. Companion to § 6041 + § 6042 + § 6045 + § 6045A + § 6045B
// + § 6050W + § 3406 backup withholding + § 103(a) tax-exempt muni +
// § 1272 OID + § 988 foreign currency.
async fn section_6049_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6049::Section6049Input>,
) -> Result<Json<traderview_expense::section_6049::Section6049Result>, ApiError> {
    Ok(Json(traderview_expense::section_6049::check(&b)))
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
// ── §6212 statutory notice of deficiency (SNOD / 90-day letter) ─────
// Mounted at /api/calc/section-6212. § 6212(a) Secretary authority to
// issue SNOD by CERTIFIED or REGISTERED mail when § 6211 deficiency
// exists; § 6212(b) load-bearing LAST KNOWN ADDRESS rule (Treas. Reg.
// § 301.6212-2) — mailing to wrong address renders SNOD INVALID and
// any subsequent assessment also invalid; § 6212(c) generally ONE
// SNOD per taxable year (exceptions: fraud, substantial omission,
// § 6861 jeopardy assessment, bankruptcy); § 6212(d) rescission with
// taxpayer's WRITTEN consent — SNOD treated as if never issued + §
// 6212(c) one-per-year limit does not bar subsequent re-issued SNOD;
// § 6213(a) petition deadline 90 days (or 150 days if taxpayer
// address outside US) + restraint on assessment during petition
// window and while petition pending; Hopkins v. Commissioner (T.C.
// 2024) — taxpayer may equitably rely on stated "last day to file"
// date even when incorrect. Natural sibling to section_6213 (Tax
// Court petition deadline + restrictions on assessment), section_6501
// (ASED), section_6502 (CSED).

async fn section_6212_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6212::Section6212Input>,
) -> Result<Json<traderview_expense::section_6212::Section6212Result>, ApiError> {
    Ok(Json(traderview_expense::section_6212::check(&b)))
}

// ── §6213 Tax Court petition 90-day rule ────────────────────────────
// Mounted at /api/calc/section-6213. § 6213(a) 90-day standard period
// (150 days if notice addressed to person outside US) for filing
// petition with Tax Court for redetermination of deficiency. Weekend/
// DC-holiday-at-last-day extension. § 6213(a) last sentence — petition
// timely if filed on or before Secretary-specified date in notice of
// deficiency (even if later than 90/150 days). § 6213(c) failure to
// file → assessment on notice and demand. Hallmark Research
// Collective (159 T.C. No. 6, 2022) holds deadline JURISDICTIONAL;
// Culp v. Commissioner (75 F.4th 196, 3d Cir. 2023) holds non-
// jurisdictional — circuit split. Trader-relevant when receiving IRS
// notice asserting § 475(f) MTM election was untimely or TTS criteria
// not satisfied — 90-day clock starts on notice mailing date.

async fn section_6213_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6213::Section6213Input>,
) -> Result<Json<traderview_expense::section_6213::Section6213Result>, ApiError> {
    if b.days_from_mailing_to_petition > 100_000 {
        return Err(ApiError::BadRequest(
            "days_from_mailing_to_petition looks invalid (>100000)".into(),
        ));
    }
    Ok(Json(traderview_expense::section_6213::compute(&b)))
}

// ── §6201 Assessment authority ──────────────────────────────────────
// Mounted at /api/calc/section-6201. Foundational grant of IRS power
// to determine and assess tax liability. § 6201(a)(1) taxes shown on
// return; § 6201(a)(2) stamp taxes; § 6201(a)(3) erroneous prepayment
// credits assessed as math/clerical error WITHOUT § 6213(b)(2)
// abatement availability; § 6201(b) deficiency restriction
// cross-references § 6213(a) (SNOD + 90-day Tax Court window
// prerequisite); § 6201(c) child compensation assessment;
// § 6201(d) (RRA 98 § 3201) burden-shifting rule — Secretary bears
// burden of producing reasonable and probative information beyond
// information return itself when taxpayer asserts reasonable dispute
// AND fully cooperates; trader-critical for 1099-B + 1099-K + K-1
// disputes. Procedural predicate for § 6203 (method of assessment)
// + § 6303 (notice and demand) + § 6321 (lien) + § 6331 (levy).
async fn section_6201_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6201::Section6201Input>,
) -> Result<Json<traderview_expense::section_6201::Section6201Result>, ApiError> {
    Ok(Json(traderview_expense::section_6201::check(&b)))
}

// ── §6203 Method of assessment ──────────────────────────────────────
// Mounted at /api/calc/section-6203. Mechanical procedure by which
// IRS assessment under § 6201 becomes effective. § 6203 — assessment
// made by recording liability of taxpayer in office of Secretary;
// upon request of taxpayer, Secretary shall furnish taxpayer copy
// of record of assessment. 26 CFR § 301.6203-1 — assessment officer
// signs summary record providing (1) identification of taxpayer,
// (2) character of liability, (3) taxable period if applicable, (4)
// amount of assessment. Form 23-C signed assessment certificate
// (internal IRS document, NOT released to taxpayers); Form 4340
// Certificate of Assessments is the document IRS provides per Rev.
// Rul. 2007-21 and is presumptive evidence per March v. IRS, 335
// F.3d 1186 (10th Cir. 2003). Trader-procedural-critical because
// no lawful § 6321 lien attachment OR § 6331 levy authority engages
// without valid § 6203 record of assessment.
async fn section_6203_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6203::Section6203Input>,
) -> Result<Json<traderview_expense::section_6203::Section6203Result>, ApiError> {
    Ok(Json(traderview_expense::section_6203::check(&b)))
}

// ── §6303 notice and demand for tax ─────────────────────────────────
// Mounted at /api/calc/section-6303. § 6303(a) — Secretary shall,
// within 60 days after assessment under § 6203, give notice to each
// person liable for unpaid tax stating amount and demanding payment.
// § 6303(a) manner of delivery — (1) left at dwelling; (2) left at
// usual place of business; or (3) sent by mail to last known address.
// Certified mail NOT required. § 6303(a) failure to give notice
// within 60 days does NOT invalidate notice. § 6303(b) — if tax
// assessed BEFORE last date prescribed for payment, demand shall not
// be made until AFTER such date (except jeopardy finding under
// § 6861/§ 6862 with § 7429 review). Foundational predicate for
// § 6321 lien attachment + § 6331 levy authority (10-day neglect
// rule begins after notice and demand). Trader-relevant because no
// lawful IRS lien, levy, or seizure may proceed without proper
// § 6303 notice and demand. 26 CFR § 301.6303-1.

async fn section_6303_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6303::Section6303Input>,
) -> Result<Json<traderview_expense::section_6303::Section6303Result>, ApiError> {
    Ok(Json(traderview_expense::section_6303::check(&b)))
}

// ── §6304 Fair Tax Collection Practices ─────────────────────────────
// Mounted at /api/calc/section-6304. RRA 98 § 3466-imported FDCPA
// (15 USC § 1692) protections: § 6304(a) communications (8 a.m.-9
// p.m. local time default convenient window; represented-taxpayer
// bypass prohibition under § 7521; workplace-contact restriction
// when employer prohibits); § 6304(b) harassment and abuse
// prohibitions (threats, obscene language, repeated phone ringing,
// anonymous calls without identity disclosure); § 6304(c) civil
// damages via § 7433 (capped $1M reckless/intentional, $100K
// negligence). Trader-relevant when revenue officer or § 6306
// private collection agency contractor uses abusive collection
// tactics against trader-taxpayer.
async fn section_6304_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6304::Section6304Input>,
) -> Result<Json<traderview_expense::section_6304::Section6304Result>, ApiError> {
    Ok(Json(traderview_expense::section_6304::check(&b)))
}

// ── §6306 Qualified Tax Collection Contracts (PCAs) ─────────────────
// Mounted at /api/calc/section-6306. American Jobs Creation Act of
// 2004 § 881-added private collection agency program; made
// MANDATORY by FAST Act of 2015 § 32102 for inactive tax
// receivables. § 6306(c) inactive defined as (1) removed from
// active inventory, (2) > 2 years post-assessment unassigned, or
// (3) > 365 days no contact on assigned receivable. § 6306(d) 8+
// exclusion categories (pending OIC/IA, innocent spouse, deceased,
// under 18, combat zone, identity theft, examination/litigation/
// criminal/levy, appeals, disability/SSI under § 223 or title XVI,
// AGI ≤ 200% federal poverty level per Taxpayer First Act of 2019
// § 1205). § 6306(b) 7-year installment agreement cap. § 6306(e)
// PCA restrictions (no enforcement, no § 7521(b)(2) audio
// recording). § 6306(f) § 6304 + § 7433 + FDCPA extend to PCA
// contractor. § 6306(j) 25%/25%/50% revenue split. Trader-relevant
// for old IRS receivables that become inactive and get assigned
// to one of four authorized PCAs (CBE Group, Coast Professional,
// ConServe, Pioneer Credit Recovery).
async fn section_6306_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6306::Section6306Input>,
) -> Result<Json<traderview_expense::section_6306::Section6306Result>, ApiError> {
    Ok(Json(traderview_expense::section_6306::check(&b)))
}

// ── §6320 Collection Due Process (CDP) for liens ────────────────────
// Mounted at /api/calc/section-6320. Parallel framework to § 6330
// (CDP for levies). § 6320(a)(2)(B) 5-business-day notice deadline
// (Letter 3172) after NFTL filing + § 6320(a)(3)(B) 30-day CDP
// request window starting day AFTER 5-business-day notice period +
// § 6320(b)(1) fair CDP hearing right + § 6320(c) issues considered
// (incorporates § 6330(c) collection alternatives + spousal defenses
// + underlying-liability gating + lien-specific § 6323(j) WITHDRAWAL,
// § 6325(d) SUBORDINATION, § 6325(b) DISCHARGE) + § 6320(d) Tax Court
// review (incorporates § 6330(d)(1) 30-day petition window). Key
// difference vs § 6330 — lien REMAINS in place during CDP review;
// no automatic withdrawal. Boechler v. Commissioner (596 U.S. 199,
// 2022) holding likely extends via § 6320(d) incorporation of
// § 6330(d)(1). Trader-relevant when receiving Letter 3172 after IRS
// files Notice of Federal Tax Lien.

async fn section_6320_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6320::Section6320Input>,
) -> Result<Json<traderview_expense::section_6320::Section6320Result>, ApiError> {
    if b.business_days_from_nftl_filing_to_notice > 10_000
        || b.days_from_notice_to_cdp_request > 100_000
        || b.days_from_determination_to_tax_court_petition > 100_000
    {
        return Err(ApiError::BadRequest(
            "day inputs out of plausible range".into(),
        ));
    }
    Ok(Json(traderview_expense::section_6320::compute(&b)))
}

// ── §6321 lien for taxes (foundational IRS general tax lien) ────────
// Mounted at /api/calc/section-6321. § 6321 three-element test for
// automatic federal tax lien arising by operation of law: (1)
// assessment by IRS under § 6201 + (2) notice and demand for payment
// under § 6303 + (3) taxpayer neglects or refuses to pay after demand.
// When all three present, lien arises AUTOMATICALLY upon ALL property
// and rights to property of taxpayer (real + personal + tangible +
// intangible), relating back to assessment date. NFTL filing under §
// 6323(f) is NOT required for lien to ATTACH (only for priority
// against third parties under § 6323). § 6322 lien continues until
// liability satisfied OR becomes unenforceable by lapse of time
// (paired with § 6502 10-year CSED). Drye v. United States, 528 U.S.
// 49 (1999) — lien attaches to whatever interest state law gives
// taxpayer; United States v. Craft, 535 U.S. 274 (2002) — tenancy by
// entirety property still subject to lien. Trader-relevant for trader-
// landlords facing automatic lien exposure on rental property
// holdings. Foundational lien-constellation companion to § 6322 +
// § 6323 + § 6325 + § 6334 (exempt property) + § 7426 (third-party
// wrongful levy) + § 7433 (unauthorized collection damages). IRM
// 5.17.2.

async fn section_6321_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6321::Section6321Input>,
) -> Result<Json<traderview_expense::section_6321::Section6321Result>, ApiError> {
    Ok(Json(traderview_expense::section_6321::check(&b)))
}

// ── §6323 federal tax lien validity / priority against third parties ─
// Mounted at /api/calc/section-6323. § 6323(a) four protected classes
// — lien NOT valid against (1) purchaser; (2) holder of security
// interest; (3) mechanic's lienor; (4) judgment lien creditor UNTIL
// NFTL filed under § 6323(f); first-in-time wins among NFTL filing
// and competing perfection. § 6323(b) ten superpriorities — priority
// OVER federal tax lien EVEN AFTER NFTL filed when interest arose
// without actual notice: (1) securities; (2) motor vehicles; (3)
// retail purchase; (4) casual sale; (5) possessory lien; (6) real
// property tax/special assessment; (7) residential mechanic's lien
// (repair/improvement); (8) attorney's lien; (9) insurance contracts;
// (10) passbook loans. § 6323(c)+(d) 45-day window for commercial
// transactions financing agreements + after-acquired personal
// property without actual notice. § 6323(g) NFTL refiling required
// every 10 years (paired with § 6502 CSED). Trader-relevant for
// trader-landlords whose rental property holdings interact with
// mortgages + judgment liens + mechanics' liens + secured creditors.
// Foundational lien-priority constellation companion to § 6321 (lien
// attachment) + § 6322 (period of lien) + § 6325 (release) + § 6334
// (exempt property) + § 7426 (third-party wrongful levy). Rev. Rul.
// 2003-108; IRM 5.12.1, 5.12.2, 5.12.7, 5.12.8.

async fn section_6323_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6323::Section6323Input>,
) -> Result<Json<traderview_expense::section_6323::Section6323Result>, ApiError> {
    Ok(Json(traderview_expense::section_6323::check(&b)))
}

// ── §6325 release of lien or discharge of property ──────────────────
// Mounted at /api/calc/section-6325. § 6325(a) RELEASE — Secretary
// SHALL issue certificate of release within 30 days upon (1) full
// satisfaction OR legally unenforceable OR (2) bond accepted;
// extinguishes lien entirely. § 6325(b) DISCHARGE of specific
// property: (b)(1) double-value rule (property ≥ 2× (lien + senior
// liens)); (b)(2)(A) partial payment for US net interest; (b)(2)(B)
// no-value determination; (b)(3) proceeds substituted; (b)(4)
// purchaser deposit. § 6325(d) SUBORDINATION: (d)(1) payment for
// subordinated amount OR (d)(2) ultimate collection facilitated
// (typical trader-landlord mortgage refinance to extract equity for
// IRS payment). § 6325(e) NON-ATTACHMENT certificate (confusion-of-
// names cases). § 6325(f) — certificates are CONCLUSIVE. Trader-
// relevant for trader-landlords seeking to (a) extinguish lien upon
// full payment, (b) discharge individual rental property for sale
// or refinance, or (c) subordinate IRS lien to allow junior
// financing. Completes lien constellation: § 6321 + § 6322 + § 6323
// + § 6325 + § 6334 + § 7426. 26 CFR § 301.6325-1; IRM 5.12.10; IRS
// Pub. 783 (Discharge); IRS Pub. 784 (Subordination).

async fn section_6325_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6325::Section6325Input>,
) -> Result<Json<traderview_expense::section_6325::Section6325Result>, ApiError> {
    Ok(Json(traderview_expense::section_6325::check(&b)))
}

// ── §6330 Collection Due Process (CDP) for levies ───────────────────
// Mounted at /api/calc/section-6330. § 6330(a) 30-day pre-levy notice
// + § 6330(b) right to fair CDP hearing before IRS Appeals + § 6330(c)
// matters at hearing (collection alternatives — § 6159 installment
// agreement / § 7122 offer in compromise / currently not collectible
// — + spousal defenses + underlying-liability challenge if no prior
// opportunity) + § 6330(d)(1) 30-day Tax Court petition + § 6330(e)
// collection suspension during pending review + § 6330(f) jeopardy /
// state refund / Federal contractor / disqualified employment tax
// levy exceptions. Boechler v. Commissioner (596 U.S. 199, 2022)
// UNANIMOUSLY held § 6330(d)(1) deadline is NON-jurisdictional and
// SUBJECT TO equitable tolling — sharp contrast to § 6213(a)
// deficiency petition deadline (Hallmark Research Collective).
// Trader-relevant when receiving IRS Final Notice of Intent to Levy
// (Letter 1058 / LT-11).

async fn section_6330_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6330::Section6330Input>,
) -> Result<Json<traderview_expense::section_6330::Section6330Result>, ApiError> {
    if b.days_from_final_notice_to_cdp_request > 100_000
        || b.days_from_determination_to_tax_court_petition > 100_000
    {
        return Err(ApiError::BadRequest(
            "day inputs out of plausible range (>100000)".into(),
        ));
    }
    Ok(Json(traderview_expense::section_6330::compute(&b)))
}

// ── §6331 levy and distraint authority ──────────────────────────────
// Mounted at /api/calc/section-6331. § 6331(a) — Secretary may levy
// upon property of taxpayer who has failed to pay tax within 10 days
// after § 6303 notice and demand. § 6331(d) — 30-day pre-levy notice
// required (in person, dwelling/place of business, or certified/
// registered mail to last known address). § 6331(e) continuous wage
// levy — attaches to (1) wages earned but not yet paid; (2) advances
// subsequent to levy; (3) wages becoming payable subsequent to levy;
// continues until released. § 6331(h) — continuous levy on up to 15%
// of specified federal payments (Social Security + federal employee
// retirement). § 6331(j) jeopardy levy exception — 30-day pre-levy
// notice DOES NOT apply if Secretary finds collection in jeopardy
// (paired with § 6861/§ 6862 jeopardy assessment + § 7429 judicial
// review). § 6331(k) — no levy while (1) innocent spouse relief
// request under § 6015 pending OR (2) CDP hearing under § 6330
// pending. Foundational levy statute. Trader-relevant for any
// taxpayer facing IRS levy threat. Pair with § 6321 (lien) + § 6323
// (priority) + § 6325 (release) + § 6334 (exempt property). 26 CFR
// § 301.6331-1; IRM 5.17.3.

async fn section_6331_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6331::Section6331Input>,
) -> Result<Json<traderview_expense::section_6331::Section6331Result>, ApiError> {
    Ok(Json(traderview_expense::section_6331::check(&b)))
}

// ── §6332 surrender of property subject to levy ─────────────────────
// Mounted at /api/calc/section-6332. § 6332(a) any person in
// possession of property subject to levy must surrender upon demand
// by Secretary. § 6332(c) 21-day bank hold: banks surrender deposits
// ONLY AFTER 21 days after service of levy (error-correction window).
// § 6332(b) wage/salary cross-references § 6331(e) continuous wage
// levy. § 6332(d)(1) personal liability — failure to surrender =
// liability equal to value of property NOT surrendered, capped at
// tax + costs + § 6621 underpayment interest. § 6332(d)(2) 50%
// additional penalty for failure WITHOUT REASONABLE CAUSE; NO credit
// against underlying tax. § 6332(e) discharge safe harbor —
// compliant surrender DISCHARGES third party from any obligation to
// delinquent taxpayer. Trader-relevant on both sides: trader-traders
// facing IRS levy on brokerage accounts (broker as third party);
// trader-landlords as third-party levy recipients (employers,
// vendors). Pair with § 6331 (levy authority) + § 6321 (lien) +
// § 6303 (notice and demand) + § 6334 (exempt property) + § 7426
// (third-party wrongful levy INVERSE pathway). 26 CFR § 301.6332-1;
// IRM 5.17.3.

async fn section_6332_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6332::Section6332Input>,
) -> Result<Json<traderview_expense::section_6332::Section6332Result>, ApiError> {
    Ok(Json(traderview_expense::section_6332::check(&b)))
}

// ── §6334 property exempt from levy ─────────────────────────────────
// Mounted at /api/calc/section-6334. § 6334(a) thirteen enumerated
// exemption categories: (1) wearing apparel + school books; (2) fuel
// + provisions + furniture + household ≤ $11,980 (2026 indexed); (3)
// books + tools of trade ≤ $5,990 (2026 indexed); (4) unemployment;
// (5) undelivered mail; (6) annuity/pension; (7) workmen's comp; (8)
// child support; (9) wage minimum exemption; (10) military disability;
// (11) public assistance; (12) Job Training Partnership Act; (13)
// residence in small-deficiency cases (unpaid tax ≤ $5,000). §
// 6334(d)(4)(B) — 2026 wage exemption parameter $5,300. § 6334(e)(1)
// — principal residence (§ 121) requires district court judge or
// magistrate WRITTEN approval before levy; district courts have
// EXCLUSIVE jurisdiction. § 6334(e)(2) — self-employed assets +
// non-rental residential real property require IRS area director
// approval. Trader-relevant for tools-of-trade exemption (trading
// rigs / monitors / books), wage exemption, principal-residence
// judicial-approval gate. Companion to § 7421 (Anti-Injunction Act +
// § 7426 wrongful-levy exception) + § 7433 (civil damages for
// unauthorized collection) + § 7430 (litigation costs) + § 7811
// (TAOs). Rev. Proc. 2025-32 + Pub. L. 119-21 (OBBBA).

async fn section_6334_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6334::Section6334Input>,
) -> Result<Json<traderview_expense::section_6334::Section6334Result>, ApiError> {
    Ok(Json(traderview_expense::section_6334::check(&b)))
}

// ── §6402 refund offsets / Treasury Offset Program ──────────────────
// Mounted at /api/calc/section-6402. § 6402 statutory hierarchy
// applies overpayments to debts in priority order: § 6402(a) IRS
// internal revenue tax (IRS handles directly), § 6402(c)(1) past-due
// child support ASSIGNED to a State, § 6402(d) federal agency non-tax
// debt (student loans etc.), § 6402(c)(2) child support NOT assigned
// to a State, § 6402(e) state income tax, § 6402(f) state unemployment
// compensation, § 6402(g) state TANF. § 6402(n) injured spouse rule
// (Form 8379) protects non-debtor spouse's share of joint refund.
// Centralized administration: Treasury Offset Program (TOP) under
// Bureau of the Fiscal Service since 1999.

async fn section_6402_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6402::Section6402Input>,
) -> Result<Json<traderview_expense::section_6402::Section6402Result>, ApiError> {
    if b.injured_spouse_share_bps > 100_000 {
        return Err(ApiError::BadRequest(
            "injured_spouse_share_bps out of plausible range".into(),
        ));
    }
    Ok(Json(traderview_expense::section_6402::compute(&b)))
}

// ── §7502 timely mailing treated as timely filing ───────────────────
// Mounted at /api/calc/section-7502. § 7502(a) US postmark = filing
// date when postmark within prescribed period + envelope properly
// addressed + sufficient postage; § 7502(c)(1) registered mail prima
// facie evidence; § 7502(c)(2) certified mail registration = postmark
// date; § 7502(f) designated PDS per Notice 2016-30 (FedEx First/
// Priority/Standard Overnight, 2 Day, International Priority/First/
// Economy; UPS Next Day Air variants, 2nd Day Air variants, Worldwide
// Express; DHL Express 9:00/10:30/12:00, Worldwide, Envelope, Import
// Express variants). Non-designated services (FedEx Ground, UPS
// Ground, FedEx Home Delivery) DO NOT qualify. Electronic filing
// governed by 26 CFR § 301.7502-1(d) e-file acknowledgment timestamp.
// Anderson v. United States (9th Cir. 1992) — § 7502 displaces common-
// law mailbox rule. Critical paired with § 6213(a) Hallmark jurisdic-
// tional deadlines and § 6330(d) Boechler equitable tolling.

async fn section_7502_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7502::Section7502Input>,
) -> Result<Json<traderview_expense::section_7502::Section7502Result>, ApiError> {
    Ok(Json(traderview_expense::section_7502::compute(&b)))
}

// ── §7503 weekend/holiday extension rule ────────────────────────────
// Mounted at /api/calc/section-7503. § 7503 — when last day for
// performing any act falls on Saturday, Sunday, or legal holiday,
// performance is timely if performed on next succeeding business
// day. Legal holiday defined: (1) legal holiday in District of
// Columbia (5 USC § 6103 — includes Juneteenth since 2021) AND (2)
// statewide legal holiday in State where office located outside DC
// but within internal revenue district. DC Emancipation Day (April
// 16, Rev. Rul. 2015-13) regularly extends federal tax filing
// deadline by 1 business day when April 15 falls on weekend. § 7503
// stacks with § 7502 timely-mailing rule. Applies to taxpayer acts
// (return filing + payment + § 6213 Tax Court petition + § 6511
// refund claim + elections) AND Commissioner acts (§ 6212 SNOD +
// § 6303 notice and demand + § 6851 termination notice).
async fn section_7503_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7503::Section7503Input>,
) -> Result<Json<traderview_expense::section_7503::Section7503Result>, ApiError> {
    Ok(Json(traderview_expense::section_7503::check(&b)))
}

// ── §7508A presidentially-declared disaster deadline postponement ───
// Mounted at /api/calc/section-7508a. § 7508A(a) Secretary's
// discretionary postponement (up to ONE YEAR / 365 days) for taxpayers
// affected by federally declared disaster (Stafford Act / 42 USC §
// 5121 et seq.) or significant fire. § 7508A(b) terroristic or
// military action postponement. § 7508A(c) special rules for pensions
// + retirement plan loan repayments. § 7508A(d) MANDATORY 60-day
// postponement period for federally declared disasters with specified
// incident date declared after December 20, 2019 (Taxpayer Certainty
// and Disaster Tax Relief Act of 2019, Pub. L. 116-94 Div. Q § 205);
// runs CONCURRENTLY with Secretary's discretionary postponement if
// Secretary period ≥ 60 days. Disaster area defined under § 1033(h)
// (3) = area eligible for federal assistance under Stafford Act.
// Postponed acts include filing returns + paying tax + filing amended
// returns + Tax Court petitions + § 6511 refund claims + § 6212 SNOD
// responses + § 6213 deficiency challenges. Trader-relevant for any
// trader in federally-declared disaster area (CA wildfires + FL
// hurricanes + TX flooding + tornado disasters) needing to extend
// filing/payment/refund-claim deadlines. Procedural-companion to §
// 7421 + § 7426 + § 7433 + § 7430 + § 6212 + § 6213 + § 6511. 26 CFR
// § 301.7508A-1.

async fn section_7508a_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7508a::Section7508AInput>,
) -> Result<Json<traderview_expense::section_7508a::Section7508AResult>, ApiError> {
    Ok(Json(traderview_expense::section_7508a::check(&b)))
}

// ── §7508 combat zone / contingency operation postponement ──────────
// Mounted at /api/calc/section-7508. Trader-relevant for active-duty
// military traders + military spouses serving in combat zones,
// contingency operations, or qualified hazardous duty areas. § 7508(a)
// IRS DISREGARDS time during (1) combat zone service (Executive Order
// designated), (2) Secretary of Defense designated contingency
// operation outside US, or (3) qualified hazardous duty area, PLUS
// 180 days after last day in such area / operation / qualified
// hospitalization. Hospitalization INSIDE the United States capped at
// 5 years (1825 days); hospitalization OUTSIDE not capped. § 7508(b)
// military spouse extension. § 7508(c) qualified hazardous duty area
// includes Sinai Peninsula. Postponed acts include filing returns +
// paying tax + § 6511 refund claims + § 6212 SNOD + § 6213
// deficiency. Distinct from § 7508A presidentially-declared disaster
// postponement (different qualifying event). 26 CFR § 301.7508-1; IRS
// Notice 2003-21; IRS Form 15109; IRS Pub. 3 Armed Forces' Tax Guide.

async fn section_7508_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7508::Section7508Input>,
) -> Result<Json<traderview_expense::section_7508::Section7508Result>, ApiError> {
    Ok(Json(traderview_expense::section_7508::check(&b)))
}

// ── §7811 Taxpayer Assistance Orders (TAOs) ─────────────────────────
// Mounted at /api/calc/section-7811. § 7811(a)(1) National Taxpayer
// Advocate may issue TAO on Form 911 application if taxpayer suffering
// or about to suffer significant hardship. § 7811(a)(2) four enumerated
// hardship categories: (A) immediate adverse action, (B) delay > 30
// days, (C) significant costs, (D) irreparable injury. § 7811(b) TAO
// may order IRS to release levied property OR cease/take/refrain from
// action. § 7811(c) modification or rescission limited to NTA /
// Commissioner / Deputy Commissioner. § 7811(d) statute of limitations
// suspended during application + decision period. § 7811(e) TAO
// INDEPENDENT of other remedies (CDP, Tax Court, refund litigation).
// Trader-relevant for IRS administrative actions causing hardship.

async fn section_7811_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7811::Section7811Input>,
) -> Result<Json<traderview_expense::section_7811::Section7811Result>, ApiError> {
    Ok(Json(traderview_expense::section_7811::compute(&b)))
}

// ── §7521 procedures involving taxpayer interviews ──────────────────
// Mounted at /api/calc/section-7521. § 7521(a)(1) taxpayer recording
// right (advance request + own equipment + own expense). § 7521(a)(2)
// IRS recording requires advance notice + reimbursable transcript on
// taxpayer request. § 7521(b)(1)(A) explanation of audit process for
// tax determination interviews; § 7521(b)(1)(B) explanation of
// collection process for collection interviews. § 7521(c) right to
// representation via attorney / CPA / enrolled agent / enrolled
// actuary / authorized rep with Form 2848 power of attorney + IRS
// MUST suspend interview when taxpayer requests representation
// consultation. § 7521(c) administrative-summons exception bars
// suspension right. § 7521(c) delay bypass with Immediate Supervisor
// consent. Trader-relevant for audit / collection / examination
// interviews — paired with § 7811 (TAOs) and § 6330/§ 6320 (CDP).

async fn section_7521_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7521::Section7521Input>,
) -> Result<Json<traderview_expense::section_7521::Section7521Result>, ApiError> {
    Ok(Json(traderview_expense::section_7521::compute(&b)))
}

// ── §7522 content of tax due, deficiency, and other notices ─────────
// Mounted at /api/calc/section-7522. Added by Taxpayer Bill of
// Rights of 1988 (TBOR 1, Pub. L. 100-647 § 6233). § 7522(a) any
// covered notice shall describe the basis for, and identify the
// amounts (if any) of, any tax due + interest + additional amounts
// + additions to tax + assessable penalties; SAFE HARBOR —
// inadequate description shall NOT INVALIDATE such notice.
// § 7522(b)(1) applies to § 6155 + § 6212 + § 6303 notices (CP14,
// Letter 1058, Letter 3171/5071C SNOD); § 7522(b)(2) applies to
// CP2000 Automated Underreporter notices generated from 1099-B +
// 1099-K + K-1 information return matching (30-day response / 60-
// day outside US); § 7522(b)(3) applies to Letter 525 first
// proposed-deficiency 30-day letter with IRS Independent Office of
// Appeals review opportunity (Taxpayer First Act of 2019 § 1001
// redesignation). Trader-procedural-critical content-disclosure
// layer over § 6201 + § 6203 + § 6212 + § 6303.
async fn section_7522_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7522::Section7522Input>,
) -> Result<Json<traderview_expense::section_7522::Section7522Result>, ApiError> {
    Ok(Json(traderview_expense::section_7522::check(&b)))
}

// ── §7525 federally authorized tax practitioner privilege ───────────
// Mounted at /api/calc/section-7525. § 7525(a)(1) extends attorney-
// client common-law privilege to CPA / EA / attorney / enrolled actuary
// / enrolled retirement plan agent (FATP under 31 USC § 330 / Circular
// 230). § 7525(a)(3)(A) noncriminal-only — categorically EXCLUDED from
// criminal tax matters (grand jury, indictment, IRS-CI referral) and
// state / local tax matters. § 7525(b) written tax-shelter-promotion
// communications categorically excluded (§ 6662(d)(2)(C)(ii) shelter
// definition). United States v. Frederick, 182 F.3d 496 (7th Cir.
// 1999) — return-preparation work NOT covered. Trader-relevant for
// protecting CPA / EA communications on M2M (§ 475(f)), straddle
// (§ 1092), § 1256 60/40, § 1091 wash sale, qualified trader status,
// § 988 / § 1297 / § 6038D international advice. Paired with § 7521
// (interview procedure) and § 7811 (TAOs).

async fn section_7525_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7525::Section7525Input>,
) -> Result<Json<traderview_expense::section_7525::Section7525Result>, ApiError> {
    Ok(Json(traderview_expense::section_7525::check(&b)))
}

// ── §7201 attempt to evade or defeat tax (apex criminal felony) ─────
// Mounted at /api/calc/section-7201. Apex criminal tax statute —
// 5-year FELONY with $250K individual / $500K corporation fine
// (18 U.S.C. § 3571 Criminal Fines Improvement Act supersedes
// § 7201 original $100K cap). Four-element test (BEYOND REASONABLE
// DOUBT burden on government): (1) existence of tax deficiency
// (additional tax owed) + (2) WILLFULNESS voluntary intentional
// violation of known duty + (3) AFFIRMATIVE ACT of evasion (Spies
// doctrine — omissions alone insufficient + mere failure to file
// or pay does NOT satisfy) + (4) SUBSTANTIAL amount. Spies v.
// United States, 317 U.S. 492 (1943) enumerates 7 affirmative-act
// indicia: double set of books + false entries + false invoices +
// destruction of records + concealment of assets + covering up
// income sources + handling affairs to avoid usual records. Two
// forms (Sansone v. United States, 380 U.S. 343 (1965)): evasion
// of ASSESSMENT (false return, hidden income) vs evasion of
// PAYMENT (concealment after assessment, transfers to nominees).
// Cheek v. United States, 498 U.S. 192 (1991) good-faith
// misunderstanding (subjective belief test) defeats willfulness.
// § 6531 criminal SOL 6 years. Spies-Daly doctrine permits
// PARALLEL civil § 6663 75% prosecution + § 6501(c)(1)/(c)(2)
// UNLIMITED ASED + § 7491 burden shifts do NOT apply. Pairs with
// section_7206 (3-year felony / tax perjury) + section_6663 (civil
// fraud 75%). IRM 9.1.3.

async fn section_7201_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7201::Section7201Input>,
) -> Result<Json<traderview_expense::section_7201::Section7201Result>, ApiError> {
    Ok(Json(traderview_expense::section_7201::check(&b)))
}

// ── §7202 willful failure to collect or pay over tax (felony) ───────
// Mounted at /api/calc/section-7202. Criminal FELONY (5-year
// imprisonment cap) + $250K individual / $500K corporation fine
// (18 U.S.C. § 3571 supersedes § 7202's original $10K cap).
// Criminal counterpart to § 6672 Trust Fund Recovery Penalty (civil
// 100%). Same conduct triggers BOTH § 7202 felony and § 6672
// 100% civil penalty per Spies-Daly doctrine. Four-element test
// (BEYOND REASONABLE DOUBT): (1) duty to collect / account for /
// pay over tax + (2) WILLFUL failure (voluntary intentional
// violation of known legal duty + no evil/bad intent required) +
// (3) amount required to be withheld and paid + (4) defendant was
// a RESPONSIBLE PERSON (status, duty, AND authority to avoid
// default; same standard as § 6672). Trust fund taxes reached:
// § 3402 income withholding + § 3101 employee FICA (Social
// Security + Medicare) + § 3301 FUTA; NOT REACHED: § 3111 employer
// FICA match. Cheek v. United States, 498 U.S. 192 (1991)
// good-faith subjective belief defeats willfulness. § 6531
// criminal SOL 6 years. Parallel civil: § 6672 TFRP (100%) +
// § 6651(a)(2) failure-to-pay (0.5%/month) + 11 U.S.C. § 523(a)(7)
// NONDISCHARGEABLE in bankruptcy. § 7491 burden shifts do NOT
// apply to criminal cases. IRM 9.1.3 + IRM 8.25.1. Pairs with
// section_6672 (civil counterpart) + section_7201 (felony evasion)
// + section_7203 (misdemeanor failure to file) + section_7206
// (felony perjury). Critical trader-business operational risk for
// any entity with W-2 employees.

async fn section_7202_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7202::Section7202Input>,
) -> Result<Json<traderview_expense::section_7202::Section7202Result>, ApiError> {
    Ok(Json(traderview_expense::section_7202::check(&b)))
}

// ── §7203 willful failure to file / pay / supply info (misdemeanor) ─
// Mounted at /api/calc/section-7203. Criminal MISDEMEANOR (1-year
// imprisonment cap) + $100K individual / $200K corporation fine
// (18 U.S.C. § 3571 supersedes § 7203 original $25K/$100K caps).
// Distinct from § 7201 5-year felony (requires affirmative acts)
// and § 7206 3-year felony (perjury). § 7203 reaches MERE
// OMISSIONS (failure to file return + failure to pay tax +
// failure to supply information + failure to keep records);
// Spies v. United States, 317 U.S. 492 (1943) — willful omission
// COUPLED with affirmative acts elevates to § 7201. Three-element
// test (BEYOND REASONABLE DOUBT): (1) required by law to file /
// pay / supply / keep records + (2) failure at time required +
// (3) WILLFULNESS. Cheek v. United States, 498 U.S. 192 (1991) —
// genuine good-faith subjective belief negates willfulness EVEN
// IF OBJECTIVELY UNREASONABLE; NOT a defense for constitutional
// challenges or tax-protester arguments. § 6050I FELONY
// EXCEPTION (cash reporting >$10K) elevates to 5 YEARS
// imprisonment. § 6531 criminal SOL 6 years. § 6651(a)(1) civil
// failure-to-file penalty (5%/month up to 25%) + § 6651(a)(2)
// civil failure-to-pay penalty (0.5%/month) PARALLEL prosecution
// permitted per Spies-Daly. § 6501(c)(3) UNLIMITED ASED when no
// return filed. § 7491 burden shifts do NOT apply to criminal
// cases. IRM 9.1.3. Completes criminal tax statute trio with
// section_7201 (felony 5-year apex) and section_7206 (3-year
// felony / perjury).

async fn section_7203_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7203::Section7203Input>,
) -> Result<Json<traderview_expense::section_7203::Section7203Result>, ApiError> {
    Ok(Json(traderview_expense::section_7203::check(&b)))
}

// ── §7212 attempts to interfere with administration (felony) ────────
// Mounted at /api/calc/section-7212. Criminal FELONY (3-year cap)
// + $250K individual / $500K corporation fine (18 U.S.C. § 3571
// supersedes original $5K cap). Two clauses: officer-specific
// (corruptly OR by force/threats endeavors to intimidate or
// impede IRS officer/employee in official capacity) + omnibus
// (any other way corruptly OR by force/threats obstructs or
// impedes due administration of Title 26). § 7212(a) threats-only
// downgrade: when offense committed ONLY by threats of force
// (no actual force + no corrupt act), 1-year misdemeanor +
// $3K fine. Marinello v. United States, 138 S. Ct. 1101 (2018)
// — omnibus clause requires NEXUS to known pending OR reasonably
// foreseeable proceeding (routine non-compliance with tax code
// requirements absent nexus does NOT constitute § 7212 violation).
// 'Corrupt' = act performed with INTENTION TO SECURE UNLAWFUL
// BENEFIT. § 6531 general 3-year criminal SOL applies (not 6).
// Spies-Daly parallel civil § 6663 fraud + § 6672 TFRP + § 6501
// (c)(1) UNLIMITED ASED for fraud. IRM 9.1.3.

async fn section_7212_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7212::Section7212Input>,
) -> Result<Json<traderview_expense::section_7212::Section7212Result>, ApiError> {
    Ok(Json(traderview_expense::section_7212::check(&b)))
}

// ── §7216 disclosure or use by preparers (criminal misdemeanor) ─────
// Mounted at /api/calc/section-7216. Pairs with § 6713 civil
// penalty ($250/disclosure + $10K annual cap). Criminal misdemeanor
// 1-year + $100K individual / $200K corporation fine (18 U.S.C. §
// 3571 supersedes original $1K cap) for preparer who knowingly or
// recklessly discloses or uses tax return info for purpose other
// than preparing return. § 7216(b) exceptions: taxpayer consent +
// non-consent permissible disclosures under 26 CFR § 301.7216-2.
// Consent must comply with 26 CFR § 301.7216-3 + Rev. Proc.
// 2013-14 (written + signed before disclosure + specific recipient
// + specific purpose + duration + prominent and separate).
// Identity-theft enhancement up to $100,000 separate from § 3571.
// § 6531 general 3-year criminal SOL. § 6713(a) no-fault civil
// penalty does not require knowing or reckless conduct. Pairs with
// section_6531 + section_6713 (civil counterpart). IRM 25.5.1 +
// IRM 4.10 preparer penalty procedural manuals.

async fn section_7216_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7216::Section7216Input>,
) -> Result<Json<traderview_expense::section_7216::Section7216Result>, ApiError> {
    Ok(Json(traderview_expense::section_7216::check(&b)))
}

// ── §7206 fraud and false statements (criminal felony) ──────────────
// Mounted at /api/calc/section-7206. Five enumerated criminal tax
// offenses: § 7206(1) tax perjury (workhorse statute) — willfully
// makes and subscribes return / statement / document containing
// declaration under penalty of perjury knowing it false as to
// material matter; § 7206(2) aiding or assisting preparation of
// false document — reaches return preparers, advisors, third
// parties even when taxpayer-signer innocent; § 7206(3) fraudulent
// bonds + permits + entries; § 7206(4) removal or concealment of
// taxed goods with intent to defraud; § 7206(5) compromises and
// closing agreement fraud under § 7121 / § 7122. Penalties: up to
// 3 YEARS imprisonment + fine $250K individual / $500K corporation
// (18 U.S.C. § 3571 Criminal Fines Improvement Act supersedes §
// 7206's original $100K cap) + costs of prosecution. § 7206(1)
// five-element test: (1) made and subscribed + (2) false as to
// material matter + (3) declaration under penalty of perjury + (4)
// did not believe true + (5) willful with specific intent.
// Cheek v. United States, 498 U.S. 192 (1991) good-faith
// misunderstanding defeats willfulness (subjective belief test).
// § 6531 criminal SOL: 6 years for § 7206(1)/(2)/(3)/(4); 3 years
// for § 7206(5). Spies-Daly doctrine permits parallel civil §
// 6663 75% fraud penalty + § 6501(c)(1) UNLIMITED ASED + § 6501
// (c)(2) UNLIMITED ASED for willful evasion. § 7491 burden shifts
// do NOT apply — government bears BEYOND REASONABLE DOUBT burden.
// IRM 9.1.3 Criminal Statutory Provisions and Common Law.

async fn section_7206_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7206::Section7206Input>,
) -> Result<Json<traderview_expense::section_7206::Section7206Result>, ApiError> {
    Ok(Json(traderview_expense::section_7206::check(&b)))
}

// ── §7207 fraudulent returns/statements/other documents (misdemeanor)
// Mounted at /api/calc/section-7207. Pairs with § 7206 (felony perjury
// alternative when document signed under penalties of perjury) and §
// 7434 (civil damages for fraudulent information return). Criminal
// MISDEMEANOR 1-year cap + $100K individual / $200K corporation fine
// (18 U.S.C. § 3571 supersedes § 7207's original $10K / $50K caps).
// Three-element test BEYOND REASONABLE DOUBT: (1) delivery or
// disclosure to IRS officer or employee of list/return/account/
// statement/other document + (2) document false or fraudulent as to
// material matter + (3) willfully or with knowledge of falsity.
// Broader scope than § 7206 (covers documents NOT signed under
// penalties of perjury) but lower penalty. Cheek v. United States,
// 498 U.S. 192 (1991) good-faith subjective belief defeats willfulness.
// § 6531(2) 6-year SOL (enumerated 6-year offense). Typical fact
// patterns: fabricated receipts during audit + altered K-1 + fraudulent
// supporting documents + false Form 433 collection information
// statement. Spies-Daly parallel civil § 7434 + § 6663 + § 6501(c)(1)
// UNLIMITED ASED. IRM 9.1.3.

async fn section_7207_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7207::Section7207Input>,
) -> Result<Json<traderview_expense::section_7207::Section7207Result>, ApiError> {
    Ok(Json(traderview_expense::section_7207::check(&b)))
}

// ── §7434 civil damages for fraudulent information return ──────────
// Mounted at /api/calc/section-7434. Trader-relevant CIVIL remedy
// when third party (employer / broker / payor) willfully files
// fraudulent W-2 / 1099 / other information return against
// taxpayer. § 7434(a) cause of action; § 7434(b) damages — greater
// of $5,000 OR actual damages + court costs + court's-discretion
// attorney fees; § 7434(d) statute of limitations — later of 6
// years from filing OR 1 year from reasonable discovery; § 7434(e)
// plaintiff must provide complaint copy to Secretary (IRS notice).
// Derolf v. Risinger Bros. misclassification carveout — most
// courts hold misclassification (1099 instead of W-2) without
// amount misstatement does NOT support § 7434 claim; plaintiff
// must allege FRAUDULENT AMOUNT MISSTATEMENT. Trader-relevant
// scenarios: broker files incorrect 1099-B inflating proceeds;
// employer files false W-2; payor files fake 1099-NEC; retaliatory
// false W-2 / 1099 from former employer. Civil judgment provides
// collateral-estoppel leverage in Tax Court / refund litigation
// arising from fraudulent 1099 / W-2 deficiency notice. Distinct
// from criminal statutes (§§ 7201 / 7202 / 7203 / 7206), civil
// fraud (§ 6663), and TFRP (§ 6672).

async fn section_7434_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7434::Section7434Input>,
) -> Result<Json<traderview_expense::section_7434::Section7434Result>, ApiError> {
    Ok(Json(traderview_expense::section_7434::check(&b)))
}

// ── §7463 disputes involving $50,000 or less (Tax Court small case) ─
// Mounted at /api/calc/section-7463. § 7463(a) — Tax Court small
// case procedure for petitions where amount in dispute does not
// exceed $50,000 per taxable year (income), per estate (estate
// tax), per calendar year (gift tax), or per period/event (excise
// tax); proceedings at option of taxpayer concurred by Tax Court
// BEFORE the hearing. § 7463(b) — decision NOT REVIEWED IN ANY
// OTHER COURT and NOT TREATED AS PRECEDENT. § 7463(c) — taxpayer
// or Secretary may discontinue designation before final decision.
// § 7463(d) — proceedings under Tax Court Rules 170-175; as
// informally as possible; any evidence with probative value
// admissible. § 7463(f) — also available for § 6320 (CDP-lien),
// § 6330 (CDP-levy), § 6015 (innocent spouse), § 7436 (worker
// classification) under $50,000. Procedural tradeoff: faster +
// cheaper + informal + pro se friendly BUT no appeal + no
// precedential value. Trader-relevant for smaller audit
// deficiencies seeking faster + cheaper Tax Court resolution.
async fn section_7463_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7463::Section7463Input>,
) -> Result<Json<traderview_expense::section_7463::Section7463Result>, ApiError> {
    Ok(Json(traderview_expense::section_7463::check(&b)))
}

// ── §7491 burden of proof shifts to Secretary ───────────────────────
// Mounted at /api/calc/section-7491. § 7491(a)(1) general burden
// shift on factual issues under Subtitle A income tax / B estate-
// gift-GST when taxpayer introduces CREDIBLE EVIDENCE (court would
// find sufficient to base decision on); § 7491(a)(2) three
// threshold conditions (A) substantiation + (B) records maintained
// + (C) cooperation with reasonable IRS requests for witnesses /
// information / documents / meetings / interviews; § 7491(a)(2)(C)
// net worth limitation — corporations + partnerships + trusts with
// net worth EXCEEDING $7,000,000 EXCLUDED from (a)(1) shifting
// (individuals + estates unlimited); § 7491(b) statistical
// reconstruction burden — Secretary bears burden on any income
// item reconstructed by statistical methods from unrelated
// taxpayers (BLS surveys, market-segment analysis) for INDIVIDUAL
// Subtitle A; § 7491(c) penalty PRODUCTION burden (not persuasion)
// for any penalty or addition to tax including § 6651, § 6662, §
// 6663, § 6672. Enacted under IRS Restructuring and Reform Act of
// 1998 (Pub. L. No. 105-206). Cross-references § 7454(a) fraud +
// accumulated earnings burden + § 6664(c) reasonable cause
// defense. Highly relevant to trader-tax controversy on § 1256
// MTM, § 988 currency, § 1202 QSBS, § 475(f) trader-tax-status.

async fn section_7491_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7491::Section7491Input>,
) -> Result<Json<traderview_expense::section_7491::Section7491Result>, ApiError> {
    Ok(Json(traderview_expense::section_7491::check(&b)))
}

// ── §7430 awarding of costs and certain fees ────────────────────────
// Mounted at /api/calc/section-7430. § 7430(a) court may award
// reasonable administrative + litigation costs to prevailing party
// against the IRS. § 7430(b)(1) exhaustion of administrative remedies
// required. § 7430(c)(4)(A) prevailing party = substantially prevailed
// on amount or most significant issue. § 7430(c)(4)(B) IRS substantial
// justification defense defeats prevailing party status. § 7430(c)(4)
// (D) + 28 U.S.C. § 2412(d)(2)(B) net worth limits — individual ≤ $2M,
// business entity ≤ $7M, 500-employee ceiling. § 7430(c)(7) qualified
// offer rule — taxpayer treated as prevailing party if QO liability ≥
// judgment. § 7430(c)(1)(B)(iii) hourly cap — 2026: $260/hour per
// Rev. Proc. 2025-32. Trader-relevant when prevailing against IRS in
// Tax Court or refund litigation.

async fn section_7430_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7430::Section7430Input>,
) -> Result<Json<traderview_expense::section_7430::Section7430Result>, ApiError> {
    if b.employee_count_at_filing > 1_000_000_000 {
        return Err(ApiError::BadRequest(
            "employee_count_at_filing looks invalid".into(),
        ));
    }
    if b.attorney_hours_billed > 1_000_000 {
        return Err(ApiError::BadRequest(
            "attorney_hours_billed looks invalid (>1000000)".into(),
        ));
    }
    Ok(Json(traderview_expense::section_7430::compute(&b)))
}

// ── §7421 Anti-Injunction Act (AIA) ─────────────────────────────────
// Mounted at /api/calc/section-7421. § 7421(a) general bar — "no suit
// for the purpose of restraining the assessment or collection of any
// tax shall be maintained in any court by any person." Eleven
// statutory exceptions: §§ 6015(e) + 6212(a)+(c) + 6213(a) + 6232(c)
// + 6330(e)(1) + 6331(i) + 6672(c) + 6694(c) + 7426(a)+(b)(1) +
// 7429(b) + 7436. Enochs v. Williams Packing, 370 U.S. 1 (1962)
// judicial 2-prong exception: (1) government cannot ultimately
// prevail AND (2) equity jurisdiction exists; BOTH required
// conjunctively. CIC Services v. IRS, 593 U.S. 209 (2021) — pre-
// enforcement challenge to IRS reporting requirement / regulation is
// NOT a suit to restrain assessment or collection within § 7421(a).
// Trader-procedural-critical: default answer for TRO/preliminary
// injunction against IRS levy/lien/assessment = NEVER. Default
// pathway: pay tax + file refund claim under § 6402/§ 7422 + refund
// suit. Paired with § 7521 (interview procedure) + § 7525 (FATP
// privilege) + § 7811 (TAOs) + § 7430 (litigation costs).

async fn section_7421_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7421::Section7421Input>,
) -> Result<Json<traderview_expense::section_7421::Section7421Result>, ApiError> {
    Ok(Json(traderview_expense::section_7421::check(&b)))
}

// ── §7422 civil actions for refund ──────────────────────────────────
// Mounted at /api/calc/section-7422. Completes refund-procedure
// constellation. Four pre-suit requirements: (1) Flora full-payment
// rule (Flora v. United States, 362 U.S. 145 (1960) — taxpayer must
// FULLY PAY assessment before suing in district court / Court of
// Federal Claims); (2) administrative claim filed under § 6511
// (within later of 3 years after return filing or 2 years after
// payment); (3) § 6532(a) 6-month wait period (180 days from admin
// claim filing, unless IRS issues disallowance sooner); (4) §
// 6532(a) 2-year filing window after IRS mails notice of
// disallowance. § 7422(e) concurrent jurisdiction limitation: if
// Secretary mails notice of deficiency BEFORE hearing, proceedings
// stayed during Tax Court petition window + 60 days; if taxpayer
// files Tax Court petition, district court / Court of Federal
// Claims loses jurisdiction to extent acquired by Tax Court.
// Jurisdiction: district court (28 USC § 1346(a)(1)) concurrent
// with Court of Federal Claims (28 USC § 1491). Pair with § 7421
// AIA exception (refund-after-payment is AIA-exception pathway) +
// § 7508A disaster postponement of § 6511 deadlines + § 7430
// litigation costs.

async fn section_7422_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7422::Section7422Input>,
) -> Result<Json<traderview_expense::section_7422::Section7422Result>, ApiError> {
    Ok(Json(traderview_expense::section_7422::check(&b)))
}

// ── §7426 third-party wrongful levy + surplus + substituted proceeds ─
// Mounted at /api/calc/section-7426. § 7426(a)(1) wrongful levy — any
// person OTHER than the assessed taxpayer with interest or lien on
// property wrongfully levied; civil action in district court. §
// 7426(a)(2) surplus proceeds — claimant interest JUNIOR to United
// States, entitled to excess sale proceeds. § 7426(a)(3) substituted
// sales proceeds — fund substituted for property under agreement. §
// 7426(c) SOL — 2 years (730 days) for wrongful levy post-12/22/2017
// TCJA Pub. L. 115-97 § 11071; pre-TCJA 9 months (274 days). § 7426(h)
// civil damages for unauthorized collection: lesser of $1,000,000
// (reckless/intentional) / $100,000 (negligence) OR actual damages +
// costs; mirrors § 7433 framework. § 7421(a) Anti-Injunction Act
// exception — § 7426(a) + (b)(1) statutorily excepted. Trader-relevant
// when IRS levies on third-party property (joint accounts + nominee
// accounts + community-property third-party interests + trader's co-
// owner / lender / lien-holder rights in seized rental property).
// Procedural-companion to § 7421 + § 7433 + § 7430 + § 6334. Pair with
// IRS Pub. 4528 (Making an Administrative Wrongful Levy Claim) + IRM
// 34.5.3 (Suits Brought Against the United States).

async fn section_7426_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7426::Section7426Input>,
) -> Result<Json<traderview_expense::section_7426::Section7426Result>, ApiError> {
    Ok(Json(traderview_expense::section_7426::check(&b)))
}

// ── §7429 review of jeopardy levy or assessment procedures ──────────
// Mounted at /api/calc/section-7429. Trader-relevant when IRS believes
// collection is in jeopardy (taxpayer planning to flee, conceal
// assets, dispose of assets to evade collection) and invokes jeopardy
// assessment under § 6861 (income/estate/gift tax) + § 6862 (other
// taxes) + immediate collection. § 7429(a) administrative review
// framework: (1) IRS provides written statement within 5 days of
// jeopardy assessment/levy; (2) taxpayer requests administrative
// review within 30 days; (3) IRS responds within 15 calendar days.
// § 7429(b) judicial review: filed within 90 days from earlier of (a)
// district director's notice of determination or (b) 16th day after
// administrative review request; DISTRICT COURT has EXCLUSIVE
// jurisdiction (no Tax Court alternative); court determines within
// 20 calendar days whether (1) assessment is REASONABLE and (2)
// amount is APPROPRIATE; extension up to 40 additional calendar
// days available for reasonable grounds (combined 60-day maximum).
// § 7421(a)(11) Anti-Injunction Act exception — § 7429(b) judicial
// review is one of 11 statutory exceptions to AIA bar. Procedural-
// companion to § 7421 + § 7426 + § 7433 + § 7430 + § 6321 + § 6323 +
// § 6325 + § 6334. 26 CFR § 301.7429-3; IRM 5.1.4; IRM 5.17.15; IRM
// 8.24.2.

async fn section_7429_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7429::Section7429Input>,
) -> Result<Json<traderview_expense::section_7429::Section7429Result>, ApiError> {
    Ok(Json(traderview_expense::section_7429::check(&b)))
}

// ── §7433 civil damages for unauthorized collection actions ─────────
// Mounted at /api/calc/section-7433. § 7433(a) cause of action — IRS
// officer or employee recklessly OR intentionally OR by reason of
// negligence disregards any IRC provision or regulation in connection
// with collection of federal tax. § 7433(b)(1) damages cap: lesser of
// $1,000,000 (reckless or intentional) / $100,000 (negligence) OR sum
// of actual direct economic damages + costs of action. § 7433(d)(1)
// exhaustion of administrative remedies required. § 7433(d)(2)
// mitigation reduction. § 7433(d)(3) 2-year SOL from accrual.
// § 7433A parallel regime for qualified tax collection contractors.
// Trader-relevant for wrongful levy beyond statutory limits + lien
// without notice + collection during § 6330 CDP appeal + § 6331/
// § 6332/§ 6334 violations. Companion to § 7421 (Anti-Injunction
// Act + § 7426 wrongful levy exception) + § 7430 (litigation costs)
// + § 7521 (interview procedure) + § 7525 (FATP privilege) + § 7811
// (TAOs).

async fn section_7433_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7433::Section7433Input>,
) -> Result<Json<traderview_expense::section_7433::Section7433Result>, ApiError> {
    Ok(Json(traderview_expense::section_7433::check(&b)))
}

// ── §162(f) fines and penalties nondeductibility ────────────────────
// Mounted at /api/calc/section-162f. § 162(f)(1) general rule (post-
// TCJA Dec 22, 2017) — no deduction for amounts paid to government in
// relation to violation or investigation. § 162(f)(2) restitution /
// remediation / compliance exception requires BOTH § 162(f)(2)(B)
// identification (court order or settlement agreement explicitly
// identifies amount + purpose) AND § 162(f)(2)(A) establishment
// (taxpayer establishes payment was for identified purpose). § 162(f)
// (3) routine investigation / court costs unaffected. § 162(f)(5) qui
// tam payments to relators outside § 162(f)(1) prohibition. § 6050X
// Form 1098-F reporting at $50K threshold. § 162(q) separate sexual-
// harassment-NDA restriction. TCJA § 13306 grandfathers pre-December
// 22, 2017 binding orders. Trader-relevant for FINRA / SEC / CFTC /
// exchange disciplinary fines.

// ── § 162(a) Trade or Business Expenses (FOUNDATIONAL) ───────────────
// Mounted at /api/calc/section-162a. Pure compute; FOUNDATIONAL
// deduction provision; Welch v. Helvering, 290 U.S. 111 (1933)
// four-element test (ordinary + necessary + carrying on trade or
// business + not capital expenditure); § 162(a)(1) reasonable
// compensation (subject to § 162(m) iter 446 $1M cap + § 280G
// iter 444 golden parachute); § 162(a)(2) traveling expenses
// (subject to § 274 iter 454 specific limits — meals 50% + entertainment
// disallowed + $25 gift cap + foreign convention reasonableness +
// luxury water travel cap); § 162(a)(3) rentals/other payments;
// § 162(c) illegal payment exceptions; § 162(e) lobbying; § 162(f)
// fines and penalties; INDOPCO 503 U.S. 79 (1992) § 263 long-term-
// benefit capitalization; Higgins 312 U.S. 212 (1941) investor
// vs trade or business; § 475(f) iter 458 trader mark-to-market
// election converts to trade or business; § 280E cannabis
// trafficking complete disallowance; § 183 hobby loss profit
// motive (3-of-5 year presumption + 9-factor Treas. Reg.
// § 1.183-2(b) test).

async fn section_162a_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_162a::Section162aInput>,
) -> Result<Json<traderview_expense::section_162a::Section162aResult>, ApiError> {
    Ok(Json(traderview_expense::section_162a::check(&b)))
}

async fn section_162f_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_162f::Section162fInput>,
) -> Result<Json<traderview_expense::section_162f::Section162fResult>, ApiError> {
    if b.payment_amount_cents < 0 {
        return Err(ApiError::BadRequest(
            "payment_amount_cents must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_162f::compute(&b)))
}

// ── § 162(m) $1M public-company executive comp deduction limit ───────
// Mounted at /api/calc/section-162m. Pure compute; § 162(m)(1)
// $1,000,000 annual cap on covered-employee remuneration deductible
// to publicly held corporation; § 162(m)(2) PUBLICLY HELD = SEC § 12
// registration OR § 15(d) reporting (TCJA-expanded to include
// foreign private issuers); § 162(m)(3) COVERED EMPLOYEE = CEO +
// CFO + top 3 most highly compensated officers + ONCE-COVERED-
// ALWAYS-COVERED (post-2016 status survives departure + death);
// post-2026 ARPA FIVE (Pub. L. 117-2 § 9708) adds next 5 most
// highly compensated (NOT necessarily officers; retested
// annually); TCJA 2017 § 13601 eliminated performance-based and
// commission exceptions; pre-TCJA written binding contract on
// November 2, 2017 transition rule preserves former § 162(m)(4)(C)
// performance-based exception; § 162(m) and § 280G can apply
// simultaneously on same compensation.

async fn section_162m_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_162m::Section162mInput>,
) -> Result<Json<traderview_expense::section_162m::Section162mResult>, ApiError> {
    Ok(Json(traderview_expense::section_162m::check(&b)))
}

// ── §6404 abatement of interest + tax + penalties ───────────────────
// Mounted at /api/calc/section-6404. § 6404(a) general abatement
// authority for excessive / post-SOL / erroneously assessed; §
// 6404(b) no statutory taxpayer RIGHT for interest / additions to
// tax / additional amounts / assessable penalties (relies on
// discretionary IRS authority OR § 6404(e)/(f)/(g)); § 6404(c)
// small tax balance ($5 or less); § 6404(e)(1) UNREASONABLE ERROR
// OR DELAY by IRS employee — error/delay after written IRS contact
// + taxpayer did not contribute + act was MINISTERIAL (procedural/
// mechanical, no judgment) OR MANAGERIAL (administrative, loss of
// records or personnel discretion); legal-judgment delays NOT
// abatable; § 6404(e)(2) erroneous refund check $50K cap; § 6404(f)
// erroneous written advice three-element test (written request +
// accurate facts + reasonable reliance); § 6404(g) 36-month
// interest suspension for individuals when IRS fails to notify
// within 1,095 days of timely return filing (21-day grace); §
// 6404(h) Tax Court review of § 6404(e) refusals for abuse of
// discretion within 180 days; Treas. Reg. § 301.6404-2; IRM 20.2.7
// Abatement and Suspension of Underpayment Interest; Form 843 +
// § 6511 lookback (3 years from return OR 2 years from payment).

async fn section_6404_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6404::Section6404Input>,
) -> Result<Json<traderview_expense::section_6404::Section6404Result>, ApiError> {
    Ok(Json(traderview_expense::section_6404::check(&b)))
}

// ── §6531 periods of limitation on criminal prosecutions ────────────
// Mounted at /api/calc/section-6531. Cross-cutting reference statute
// that determines criminal SOL for ALL Title 26 criminal tax
// prosecutions. General rule: 3 YEARS from commission of offense.
// 6-YEAR exception for enumerated offenses: § 7201 evasion + § 7202
// trust fund failure + § 7203 failure to FILE/PAY (NOT failure to
// keep records or supply info) + § 7206(1) filing false return +
// § 7206(2) aiding false return + § 7207 fraudulent returns/
// statements + § 7212(b) rescue of seized property + § 7214 unlawful
// acts of revenue officers + 18 U.S.C. § 371 Klein conspiracy.
// 3-year SOL: § 7203 records/info + § 7205 false withholding
// exemption + § 7206(3)/(4)/(5) + § 7212(a) general obstruction +
// all other Title 26 offenses. § 6531(4) carveout: 6-year for
// failure to file does NOT apply to partnership Form 1065 + exempt
// org Form 990 + S-corp Form 1120-S returns under Part III
// Subchapter A Chapter 61 (3-year SOL applies). Final-paragraph
// tolling: defendant outside US or fugitive tolls SOL until 6
// months after return/surrender. Toussie v. United States, 397
// U.S. 112 (1970) continuing-offense doctrine narrowed but
// affirmative-act-doctrine cases survive — SOL runs from LAST
// affirmative act for § 7201. § 6531 SOL is JURISDICTIONAL. DOJ
// Criminal Tax Manual § 7.00 + IRM 25.6.2.1. Pairs with section_
// 7201 + section_7202 + section_7203 + section_7206 + section_7212.

async fn section_6531_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6531::Section6531Input>,
) -> Result<Json<traderview_expense::section_6531::Section6531Result>, ApiError> {
    Ok(Json(traderview_expense::section_6531::check(&b)))
}

// ── §6532 periods of limitation on refund + wrongful-levy suits ─────
// Mounted at /api/calc/section-6532. § 6532(a) taxpayer refund suit
// under § 7422 — 6-month floor + 2-year ceiling from notice of
// disallowance mailed certified/registered; § 6532(a)(2) written
// extension; § 6532(a)(3) reconsideration does NOT extend;
// § 6532(a)(4) waiver of certified-mail requirement runs from waiver
// filing. § 6532(b) US erroneous refund suit under § 7405 — 2 years
// standard; 5 years if refund induced by FRAUD OR MISREPRESENTATION
// OF A MATERIAL FACT. § 6532(c)(1) third-party wrongful levy suit
// under § 7426 — 2 YEARS from date of levy (TCJA 2017 § 11071
// EXTENDED prior 9-month period to 2 years, effective for levies
// made after December 22, 2017); § 6532(c)(2) § 6343(b)
// administrative-claim extension to SOONER of (A) 12 months from
// claim filing OR (B) 6 months from IRS disallowance. Trader-
// critical for every refund-suit scenario (NOL § 172/§ 475(f)
// carryback, § 1256 60/40 mark-to-market amended return, § 1091
// wash-sale recomputation, § 988 currency loss restatement) and
// every third-party broker-account wrongful-levy scenario. Sibling
// cluster: § 7422 + § 7426 + § 7405 + § 6511 + § 6343(b).

async fn section_6532_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6532::Section6532Input>,
) -> Result<Json<traderview_expense::section_6532::Section6532Result>, ApiError> {
    if b.written_extension_days_added > 36_500 {
        return Err(ApiError::BadRequest(
            "written_extension_days_added out of range".into(),
        ));
    }
    Ok(Json(traderview_expense::section_6532::check(&b)))
}

// ── §6501 limitations on assessment + collection (ASED) ─────────────
// Mounted at /api/calc/section-6501. § 6501(a) 3-year default ASED
// from filing date; § 6501(b)(1) early-filed return deemed filed on
// statutory due date; § 6501(c)(1) UNLIMITED for false/fraudulent
// return with intent to evade tax (clear-and-convincing burden);
// § 6501(c)(2) UNLIMITED for willful attempt to evade; § 6501(c)(3)
// UNLIMITED for no return filed (3-year clock starts only upon
// filing); § 6501(c)(4) Form 872 consent extension + IRM 25.6.22
// three-rights disclosure requirement; § 6501(e)(1)(A)(i) 6-year for
// >25% gross-income omission; § 6501(e)(1)(B) 6-year for basis
// overstatement (post-2015 Surface Transportation Act amendment
// overruling Home Concrete & Supply v. United States, 132 S. Ct.
// 1836 (2012)). Trader-critical defensive shield against IRS audit
// reach-back on wash-sale disallowances, § 1256 mark-to-market,
// § 988 currency, § 1202 QSBS holding-period determinations.

async fn section_6501_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6501::Section6501Input>,
) -> Result<Json<traderview_expense::section_6501::Section6501Result>, ApiError> {
    Ok(Json(traderview_expense::section_6501::check(&b)))
}

// ── §6502 collection after assessment (CSED) ────────────────────────
// Mounted at /api/calc/section-6502. § 6502(a)(1) 10-year base CSED
// from date of assessment — after CSED, IRS BARRED from collecting
// via levy (§ 6331), lien (§ 6321), or court proceeding. Six
// independent suspension triggers each extend CSED: § 6502(a)(2)
// installment agreement + 90 days post-expiration; § 6331(k)(1) OIC
// suspended from submission through accept/reject/withdraw/return
// + ADDITIONAL 30 days if rejected; § 6330(e)(1) CDP hearing
// request suspends through conclusion + 90-day floor if < 90 days
// remain on CSED; § 6503(h) bankruptcy automatic stay + 6 months
// after stay terminates; § 7508(a) military combat zone + 180 days;
// § 6503(c) taxpayer continuously absent from US 6+ months +
// absence period + return + 6 months. Overlapping suspensions run
// CONCURRENTLY not cumulatively per IRM 5.1.19.3.4. Natural sibling
// to section_6501 (ASED — 3/6/unlimited assessment statute) and
// section_7811 (TAOs).

async fn section_6502_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6502::Section6502Input>,
) -> Result<Json<traderview_expense::section_6502::Section6502Result>, ApiError> {
    Ok(Json(traderview_expense::section_6502::check(&b)))
}

// ── §6511 limitations on credit or refund ───────────────────────────
// Mounted at /api/calc/section-6511. §6511(a) general 3-year-from-
// filing or 2-year-from-payment whichever later; §6511(b)(2) 3-year
// or 2-year lookback rule on refund amount; §6511(d)(1) 7-year bad
// debt / worthless security (§§ 166, 832(c), 165(g)); §6511(d)(2)(A)
// NOL/capital loss carryback period ends 3 years after due date of
// LOSS-year return; §6511(d)(3)(A) 10-year foreign tax credit;
// §6511(h) financial-disability suspension flagged for caller. Rev.
// Rul. 2020-8 (suspending Rev. Rul. 71-533) flagged for FTC carryback
// from NOL carryback open question. Trader-critical for Form 1040-X
// amended returns claiming missed § 475(f) MTM elections, missed
// § 901 FTCs, worthless-security losses, or NOL carrybacks.

async fn section_6511_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6511::Section6511Input>,
) -> Result<Json<traderview_expense::section_6511::Section6511Result>, ApiError> {
    if b.return_tax_year < 1900 || b.return_tax_year > 2200 {
        return Err(ApiError::BadRequest("return_tax_year out of range".into()));
    }
    if let Some(y) = b.carryback_loss_year {
        if !(1900..=2200).contains(&y) {
            return Err(ApiError::BadRequest(
                "carryback_loss_year out of range".into(),
            ));
        }
    }
    Ok(Json(traderview_expense::section_6511::compute(&b)))
}

// ── §6601 interest on underpayment + §6621 rate + §6622 compounding ─
// Mounted at /api/calc/section-6601. § 6601(a) interest from last
// date prescribed for payment under § 6601(b)(1) (extension to file
// does not extend time to pay) until paid. § 6622(a) daily
// compounding. § 6621(a)(2) underpayment rate = federal short-term
// rate + 3%; § 6621(c) large corporate underpayment rate = federal
// short-term rate + 5% (after applicable date — generally 30 days
// after IRS notice). Quarterly rates published via Revenue Ruling.
// 2026 Q1 (Rev. Rul. 2025-22): 7% underpayment / 9% large corporate.
// 2026 Q2 (Rev. Rul. 2026-5): 6% underpayment / 8% large corporate.
// Trader-relevant when amended return / audit produces additional
// tax — interest runs from ORIGINAL April 15 due date regardless of
// extension to file. § 6601 interest is non-deductible personal
// interest under § 163(h) for individuals but deductible business
// interest under § 163(a) for sole-proprietor traders.

async fn section_6601_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6601::Section6601Input>,
) -> Result<Json<traderview_expense::section_6601::Section6601Result>, ApiError> {
    if b.rate_quarter == 0 || b.rate_quarter > 4 {
        return Err(ApiError::BadRequest(
            "rate_quarter must be 1, 2, 3, or 4".into(),
        ));
    }
    if b.days_outstanding > 1_000_000 {
        return Err(ApiError::BadRequest(
            "days_outstanding looks invalid (>1000000)".into(),
        ));
    }
    Ok(Json(traderview_expense::section_6601::compute(&b)))
}

// ── §6611 interest on overpayments (companion to §6601) ─────────────
// Mounted at /api/calc/section-6611. § 6611(a) general overpayment
// interest at § 6621 rate. § 6611(b)(2) refund interest from
// overpayment date to 30 days before refund check (§ 6611(b)(1) credit
// path). § 6611(e)(1) 45-day SAFE HARBOR — refund within 45 days of
// return-due date triggers ZERO interest. § 6611(e)(2) parallel 45-day
// safe harbor for refund claims (Form 1040-X). § 6611(e)(3) IRS-
// initiated adjustment SUBTRACTS 45 days from interest period. § 6621(a)
// (1) overpayment rates: individual FST + 3%, corporate FST + 2%,
// corporate > $10K GATT rate FST + 0.5% (Pub. L. 103-465 § 713). § 6622
// (a) daily compounding. 2026 Q1 (Rev. Rul. 2025-22): 7% individual /
// 6% corporate / 4.5% GATT. 2026 Q2 (Rev. Rul. 2026-5): 6% / 5% / 3.5%.
// Interest received treated as gross income under § 61(a)(4).

async fn section_6611_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6611::Section6611Input>,
) -> Result<Json<traderview_expense::section_6611::Section6611Result>, ApiError> {
    if b.rate_quarter == 0 || b.rate_quarter > 4 {
        return Err(ApiError::BadRequest(
            "rate_quarter must be 1, 2, 3, or 4".into(),
        ));
    }
    if b.days_from_overpayment_to_refund > 1_000_000 {
        return Err(ApiError::BadRequest(
            "days_from_overpayment_to_refund looks invalid (>1000000)".into(),
        ));
    }
    Ok(Json(traderview_expense::section_6611::compute(&b)))
}

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

// ── §6020 returns prepared for or executed by Secretary ─────────────
// Mounted at /api/calc/section-6020. § 6020(a) voluntary preparation
// (taxpayer consents + discloses + signs); SFR signed by taxpayer
// counts as filed return and starts § 6501 ASED. § 6020(b)(1)
// involuntary preparation when taxpayer fails to make return or
// makes false/fraudulent return; Secretary makes return from own
// knowledge and testimony. § 6020(b)(2) — Secretary-prepared return
// is PRIMA FACIE GOOD AND SUFFICIENT for all legal purposes. § 6020
// (b) SFR does NOT satisfy Beard test (Beard v. Commissioner, 82
// T.C. 766 (1984), aff'd 793 F.2d 139 (6th Cir. 1986)) prong 4
// (executed under penalties of perjury BY TAXPAYER); § 6501 ASED
// NEVER STARTS on § 6020(b) SFR — IRS may assess at any time
// forever. Late-filed valid return AFTER SFR starts § 6501 ASED
// clock. 26 CFR § 301.6020-1 + Form 13496 — § 6020(b) return must
// identify taxpayer + contain sufficient info + purport to be a
// return. Trader-relevant because non-filing trader receives § 6020
// (b) SFR with worst-case computations (no Schedule C deductions +
// no § 475(f) M2M + no § 1091 wash sale + no cost basis on 1099-B).
async fn section_6020_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6020::Section6020Input>,
) -> Result<Json<traderview_expense::section_6020::Section6020Result>, ApiError> {
    Ok(Json(traderview_expense::section_6020::check(&b)))
}

// ── §6038A Form 5472 25%-foreign-owned domestic corp + DRE ─────────
// Mounted at /api/calc/section-6038a. § 6038A(a) requires every
// 25%-foreign-owned domestic corp + foreign corp engaged in US
// trade/business to file Form 5472 reporting related-party
// transactions. § 6038A(c)(1) 25% threshold = direct or indirect
// foreign ownership of voting power OR total value at ANY TIME
// during taxable year. Treas. Reg. § 1.6038A-1(c) per T.D. 9796
// (December 13, 2016) effective tax years beginning 2017-01-01 —
// foreign-owned US single-member LLC disregarded entities treated as
// DOMESTIC CORPORATIONS for limited § 6038A purposes (Form 5472
// filed as attachment to pro-forma Form 1120). § 6038A(d)(1) BASE
// PENALTY $25,000 per taxable year per reporting corporation;
// § 6038A(d)(2) CONTINUATION PENALTY $25,000 per 30-day period (or
// fraction) after 90-day IRS notification — UNCAPPED; § 6038A(d)(3)
// reasonable cause defense under Treas. Reg. § 1.6038A-4(b).
// § 6501(c)(8) — § 6501 assessment SOL does NOT start running until
// required § 6038A return is filed, keeping ASED OPEN INDEFINITELY.
// Trader-critical for foreign-owned DE/WY/NV trading LLCs, foreign
// hedge fund US-LLC conduits, jointly-owned US LLCs with foreign
// family/business partners, and § 475(f) MTM-elected entities with
// intra-family transfers. Sibling cluster: § 6038D + § 6038 +
// § 6038B + § 6038C + § 6501(c)(8).

async fn section_6038a_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6038a::Section6038aInput>,
) -> Result<Json<traderview_expense::section_6038a::Section6038aResult>, ApiError> {
    if b.max_foreign_ownership_bps > 10_000 {
        return Err(ApiError::BadRequest(
            "max_foreign_ownership_bps must be ≤ 10000 (100%)".into(),
        ));
    }
    if b.days_since_irs_notification > 100_000 {
        return Err(ApiError::BadRequest(
            "days_since_irs_notification out of range".into(),
        ));
    }
    Ok(Json(traderview_expense::section_6038a::check(&b)))
}

// ── §6038B Form 926 / Form 8865 transfer to foreign corp + partnership ─
// Mounted at /api/calc/section-6038b. § 6038B(a)(1)(A) Form 926
// transfers to foreign corp (§ 332/§ 351/§ 354/§ 355/§ 356/§ 361
// exchanges); § 6038B(a)(1)(B) Form 8865 § 721 contribution to
// foreign partnership. § 6038B(b)(1) BASE PENALTY = 10% of FMV at
// time of transfer; § 6038B(b)(1)(A) CAPPED at $100,000;
// § 6038B(b)(1)(B) INTENTIONAL DISREGARD removes cap. § 6038B(b)(2)
// failure forces § 367 gain recognition AS IF property sold at FMV
// (in addition to monetary penalty). § 6038B(c) reasonable cause
// defense under Treas. Reg. § 1.6038B-1(f)(3) / § 1.6038B-2(j)(3).
// § 367(d) intangibles trigger DEEMED-SALE treatment requiring
// annual commensurate-with-income inclusion. § 721(c) gain-deferral
// method under Treas. Reg. § 1.721(c)-3 available for related-party
// foreign partnership transfers (multi-year reporting + remedial
// allocations). § 6501(c)(8) — § 6501 assessment SOL OPEN
// INDEFINITELY on non-filing. Trader-critical for cryptocurrency
// transfers to foreign exchanges/wallets (Notice 2014-21 property
// classification), intangible asset transfers (trading algorithms,
// proprietary models, IP), § 351 contributions to foreign-
// incorporated trading entities, § 721 contributions to foreign
// partnership trading vehicles, and master/feeder/parallel fund
// structures. Sibling cluster: § 6038A + § 6038D + § 367 + § 721 +
// § 721(c) + § 6501(c)(8).

async fn section_6038b_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6038b::Section6038bInput>,
) -> Result<Json<traderview_expense::section_6038b::Section6038bResult>, ApiError> {
    if b.ownership_pct_after_transfer_bps > 10_000 {
        return Err(ApiError::BadRequest(
            "ownership_pct_after_transfer_bps must be ≤ 10000 (100%)".into(),
        ));
    }
    Ok(Json(traderview_expense::section_6038b::check(&b)))
}

// ── §6038C foreign corp engaged in US trade or business — Form 5472 ─
// Mounted at /api/calc/section-6038c. § 6038C(a) — foreign corp
// engaged in US T/B at any time during taxable year SHALL furnish
// information described in § 6038A(b) (related party + reportable
// transactions) AND maintain records prescribed by regulations.
// § 6038C(b) — penalties of § 6038A apply (cross-reference):
// $25,000 base + $25,000/30-day continuation (UNCAPPED after 90-day
// notification) + reasonable cause defense. § 6038C(c) — LIMITED
// AGENT authorization rule: rules apply to any transaction with
// foreign-person related party UNLESS related party AGREES to
// authorize reporting corp as limited agent for § 7602 (examination)
// + § 7603 (service of summons) + § 7604 (enforcement of summons)
// purposes. § 6038C(d) — terms 'related party' + 'foreign person' +
// 'records' have same meaning as § 6038A(c) (cross-reference).
// § 864(b)(2) trading safe harbor — foreign person NOT a dealer who
// trades for own account through resident broker/agent does NOT
// have US T/B; if safe harbor qualifies, NO § 6038C exposure.
// § 882 — foreign corp engaged in US T/B taxed on ECI; § 6038C
// provides reporting backbone. § 6501(c)(8) — § 6501 ASED OPEN
// INDEFINITELY on non-filing. Anti-avoidance backstop closing
// foreign-corp reporting cluster with § 6038A + § 6038B. Trader-
// critical for foreign hedge fund LPs with US branch, foreign
// proprietary trading firms with US-based traders (potential loss
// of § 864(b)(2) safe harbor), foreign brokerages with US
// permanent establishment, foreign trader-managed family offices
// with US ECI. Statutory origin: Omnibus Budget Reconciliation Act
// of 1990 § 11315 (Pub. L. 101-508, November 5, 1990).

async fn section_6038c_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6038c::Section6038cInput>,
) -> Result<Json<traderview_expense::section_6038c::Section6038cResult>, ApiError> {
    if b.days_since_irs_notification > 100_000 {
        return Err(ApiError::BadRequest(
            "days_since_irs_notification out of range".into(),
        ));
    }
    Ok(Json(traderview_expense::section_6038c::check(&b)))
}

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

// ── § 6663 civil fraud penalty (75%) ────────────────────────────────
// Mounted at /api/calc/section-6663. § 6663(a) 75% penalty on portion
// of underpayment attributable to fraud; § 6663(b) burden-shift rule —
// once IRS proves any portion as fraud, ENTIRE underpayment treated as
// fraud unless taxpayer carves out by preponderance; § 6663(c) joint
// return innocent spouse exception — penalty does not apply to spouse
// whose conduct did not contribute (cross-reference § 6015); § 6662(b)(7)
// non-stacking with accuracy-related penalty (mutually exclusive on
// same dollar); § 7454(a) IRS bears CLEAR AND CONVINCING burden of
// proof (heightened standard greater than preponderance, less than
// beyond reasonable doubt); Spies v. United States, 317 U.S. 492 (1943)
// badges of fraud doctrine (9-badge enumeration); § 6501(c)(1) UNLIMITED
// ASED when fraud established; § 6651(f) parallel 75% failure-to-file
// penalty; 11 U.S.C. § 523(a)(1)(C) NONDISCHARGEABLE in personal
// bankruptcy; Spies-Daly doctrine permits parallel civil + criminal
// prosecution under § 7201 / § 7206; IRM 25.1.6 Civil Fraud procedural
// manual. § 6664(c)(1) reasonable-cause defense theoretically applies
// but rarely succeeds. Natural sibling to section_6664 + section_6501 +
// section_6502 + section_6212.

async fn section_6663_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6663::Section6663Input>,
) -> Result<Json<traderview_expense::section_6663::Section6663Result>, ApiError> {
    Ok(Json(traderview_expense::section_6663::check(&b)))
}

// ── § 6664 reasonable cause + good faith defense ────────────────────
// Mounted at /api/calc/section-6664. § 6664(c)(1) general rule —
// no penalty under § 6662 or § 6663 may be imposed for any portion
// of an underpayment where taxpayer shows reasonable cause AND
// good faith. § 6664(c)(2) economic-substance strict-liability
// bar — defense NOT available for transactions lacking economic
// substance under § 7701(o); § 6662(b)(6) + § 6662(i) impose 20%
// (40% non-disclosed) strict-liability penalty with no escape.
// § 6664(d) reportable-transaction heightened defense for § 6662A
// requires ALL THREE elements: (A) adequate disclosure per
// § 6664(d)(3)(A); (B) substantial authority per § 6664(d)(3)(B);
// (C) reasonable belief more-likely-than-not per § 6664(d)(3)(C).
// Treas. Reg. § 1.6664-4 implementing regulation — facts-and-
// circumstances analysis (education, sophistication, business
// experience, advisor reliance with complete + accurate facts).
// Treas. Reg. § 1.6662-3(c)(2) regulation-invalidity adequate-
// disclosure rule. Cross-cutting defense applies to section_6662
// (accuracy) + section_6662a (reportable) + section_6663 (civil
// fraud). Highly relevant for aggressive § 1256 MTM / § 988 / §
// 1202 / § 475(f) trader positions.

async fn section_6664_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6664::Section6664Input>,
) -> Result<Json<traderview_expense::section_6664::Section6664Result>, ApiError> {
    Ok(Json(traderview_expense::section_6664::check(&b)))
}

// ── § 6672 Trust Fund Recovery Penalty (TFRP) ───────────────────────
// Mounted at /api/calc/section-6672. 100% PERSONAL liability on
// responsible persons for unpaid trust fund portion of employment
// taxes (§ 3402 income tax withholding + § 3101 employee FICA; NOT
// § 3111 employer FICA + § 3301 FUTA). Two-prong test: (1)
// responsible person = significant (not exclusive) control over
// finances OR officer/director/designated status OR check-signing /
// payment authority; (2) willfulness = knew taxes due OR reckless
// disregard OR used available funds to pay other creditors (no
// evil intent required). § 6672(b)(1) IRS MUST send preliminary
// notice (Letter 1153 + Form 2751) at least 60 days before
// assessment. § 6672(d) joint and several liability + state-law
// contribution claim among co-responsible persons. 11 U.S.C. §
// 523(a)(7) NONDISCHARGEABLE in personal bankruptcy + § 507(a)(8)(C)
// priority claim. Critical trader-business operational risk for
// LLC / S-corp / C-corp with W-2 employees.

async fn section_6672_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6672::Section6672Input>,
) -> Result<Json<traderview_expense::section_6672::Section6672Result>, ApiError> {
    Ok(Json(traderview_expense::section_6672::check(&b)))
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

// ── §6695A appraiser penalty for substantial/gross valuation ──
// misstatements. Mounted at /api/calc/section-6695a. Added by
// Pension Protection Act of 2006 § 1219 to penalize appraisers
// whose appraisals support substantial or gross valuation
// misstatements. § 6695A(a) — penalty imposed when appraiser knew
// or reasonably should have known appraisal would be used on
// return AND claimed value results in § 6662(e) substantial (≥
// 150%) OR § 6662(g) estate/gift understatement (≤ 65%) OR §
// 6662(h) gross (≥ 200%) valuation misstatement. § 6695A(b) —
// penalty equals LESSER OF (1) greater of 10% of underpayment or
// $1,000 AND (2) 125% of gross income received from appraisal.
// § 6695A(c) good-faith exception — no penalty if appraiser
// establishes value was MORE LIKELY THAN NOT (51% confidence) the
// proper value. Effective dates: general rule after August 17,
// 2006; facade easement special rule after July 25, 2006. Trader-
// relevant for art donations + conservation easements (§ 170(h))
// + facade easements + § 1031 real estate + partnership interest
// valuations + syndicated conservation easement deductions
// (§ 6707A listed transaction crossover) + estate/gift tax
// valuations.
async fn section_6695a_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6695a::Section6695AInput>,
) -> Result<Json<traderview_expense::section_6695a::Section6695AResult>, ApiError> {
    Ok(Json(traderview_expense::section_6695a::check(&b)))
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

// ── §6707A reportable transaction penalty ───────────────────────────
// Mounted at /api/calc/section-6707a. § 6707A(b)(1) base = 75% of
// decrease in tax shown on return as result of transaction. §
// 6707A(b)(2) maximum: listed transaction = $200,000 entity /
// $100,000 natural person; other reportable = $50,000 entity /
// $10,000 natural person. § 6707A(b)(3) minimum: $10,000 entity /
// $5,000 natural person. § 6707A(c)(2) listed transaction =
// substantially similar to transaction specifically identified by
// Secretary as tax avoidance under § 6011. Trader-relevant for
// partnerships caught in syndicated conservation easements + micro-
// captive § 831(b) insurance + monetized installment sales +
// abusive § 6011 reportable transactions. Stacks on top of § 6662A
// accuracy-related penalty. CIC Services v. IRS, 593 U.S. 209 (2021)
// — pre-enforcement challenge to § 6707A reporting requirements NOT
// barred by § 7421(a) Anti-Injunction Act.

// ── §6707 material advisor failure to furnish reportable transaction info ─
// Mounted at /api/calc/section-6707. § 6707(a) — material advisor
// required to file Form 8918 under § 6111 with respect to ANY
// reportable transaction must do so on or before deadline OR file
// return with complete/accurate information; failure = penalty.
// § 6707(b)(1) OTHER REPORTABLE TRANSACTIONS: flat $50,000 base
// penalty. § 6707(b)(2) LISTED TRANSACTIONS: GREATER of $200,000 OR
// 50% of gross income from aid/assistance/advice; 50% rate
// SUBSTITUTED with 75% when failure or act is INTENTIONAL.
// § 6707(c)(1) Commissioner may rescind penalty for non-listed
// transactions if rescission promotes tax compliance and effective
// tax administration. § 6707(c)(2) LISTED TRANSACTIONS NOT
// ELIGIBLE FOR RESCISSION — strict liability. § 6707(c)(3) NO
// JUDICIAL REVIEW of denial of rescission. § 6664(d) reasonable
// cause defense AVAILABLE for non-listed but NOT for listed
// transactions. Enacted by American Jobs Creation Act of 2004 § 815
// (Pub. L. 108-357, October 22, 2004). Trader-critical for material
// advisors on basket option contracts (Notice 2015-73), conservation
// easement syndications (Notice 2017-10), micro-captive insurance
// (Notice 2016-66), § 643 distribution-tier-out trusts, STARS
// foreign-tax-credit shelters. Sibling cluster: § 6011 (Form 8886
// taxpayer disclosure) + § 6111 (Form 8918 material advisor) +
// § 6112 (advisor list maintenance) + § 6707A (taxpayer return
// disclosure penalty) + § 6662A (reportable-transaction
// understatement accuracy penalty) + IRM 20.1.13.

async fn section_6707_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6707::Section6707Input>,
) -> Result<Json<traderview_expense::section_6707::Section6707Result>, ApiError> {
    Ok(Json(traderview_expense::section_6707::check(&b)))
}

async fn section_6707a_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6707a::Section6707AInput>,
) -> Result<Json<traderview_expense::section_6707a::Section6707AResult>, ApiError> {
    Ok(Json(traderview_expense::section_6707a::check(&b)))
}

// ── §6713 civil penalty for disclosure or use of information by ─────
// return preparers. Mounted at /api/calc/section-6713. Civil
// companion to § 7216 criminal penalty. § 6713(a) — $250 per
// disclosure/use; $10,000 annual cap. § 6713(b) — § 7216(b)
// exceptions apply: court order/subpoena + preparer in firm +
// assisting firm (e-filing) + bookkeeping + quality/peer review +
// professional liability insurance + tax authority investigation +
// other federal law + taxpayer written consent. § 6713 strict
// liability (no knowing/reckless requirement) vs § 7216 criminal
// requires KNOWING OR RECKLESS conduct (misdemeanor + 1 year + $1,000
// fine + costs). Both penalties may apply concurrently to the same
// disclosure. Trader-relevant for preparer monetizing/sharing trader
// financial data (1099-Bs + § 475(f) M2M + cost basis + § 1091 wash
// sale + § 1256 60/40 + § 988 + § 6038D). Rev. Proc. 2013-14 +
// AICPA sample consent forms. Companion to § 7216 criminal +
// § 7525 FATP + § 6694 preparer substantive + § 6695 preparer
// procedural + Circular 230 § 10.50.
// ── §6708 material advisor failure to maintain list of advisees ─────
// Mounted at /api/calc/section-6708. § 6708(a)(1) — material advisor
// required to maintain § 6112 list FAILS to make list available
// upon WRITTEN IRS request within 20 BUSINESS DAYS = $10,000 PER DAY
// for each day after 20th day, NO STATUTORY MAXIMUM. § 6708(a)(2)
// reasonable cause exception (distinct from § 6664(d)). Treas. Reg.
// § 301.6708-1(c)(3)(ii) extension request requires reason + period
// required + good-faith-effort description. § 6112(b) cross-reference:
// list content (advisee identifiers + transaction ID + timing +
// amount + tax treatment); 30 CALENDAR DAYS preparation period; 7
// YEARS retention; separate list per transaction; one list for
// substantially similar transactions. Coordination: § 6707 penalizes
// failure to FILE Form 8918 disclosure; § 6708 penalizes failure to
// MAINTAIN AND PRODUCE the § 6112 list — TWO INDEPENDENT penalties
// on same material advisor for same transaction. Enacted by American
// Jobs Creation Act of 2004 § 815 (Pub. L. 108-357, October 22,
// 2004). Trader-critical for material advisors on basket option
// contracts (Notice 2015-73), conservation easement syndications
// (Notice 2017-10), micro-captive insurance (Notice 2016-66), § 643
// distribution-tier-out trusts, STARS foreign-tax-credit shelters.

async fn section_6708_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6708::Section6708Input>,
) -> Result<Json<traderview_expense::section_6708::Section6708Result>, ApiError> {
    if b.business_days_since_irs_request > 100_000 {
        return Err(ApiError::BadRequest(
            "business_days_since_irs_request out of range".into(),
        ));
    }
    Ok(Json(traderview_expense::section_6708::check(&b)))
}

async fn section_6713_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6713::Section6713Input>,
) -> Result<Json<traderview_expense::section_6713::Section6713Result>, ApiError> {
    Ok(Json(traderview_expense::section_6713::check(&b)))
}

// ── §6851 termination assessment of income tax ──────────────────────
// Mounted at /api/calc/section-6851. Emergency procedure by which IRS
// may TERMINATE a taxpayer's taxable year mid-year when Secretary
// finds taxpayer designing to depart from US, conceal property, or
// jeopardize collection. § 6851(a)(1) — three triggers: (A)
// departing/removing property; (B) concealing self/property; (C)
// other jeopardizing act including corporate liquidation. § 6851(a)
// (2) — tax computed as if terminated period were taxable year AND
// by placing entire tax base on annual basis (annualization). §
// 6851(b) — SNOD within 60 days after LATER of full-year return due
// date or taxpayer's filing date. § 6851(c) — amounts collected
// treated as collected on date of entire-year assessment. § 6851(d)
// cross-references § 7429 review + § 6863 stay + § 6213(a) Tax
// Court petition right. § 6851 vs § 6861 distinction: § 6851
// terminates CURRENT/preceding taxable year BEFORE return due date;
// § 6861 jeopardy-assesses EXISTING DEFICIENCY AFTER return filing.
// § 6851 + § 6863 interaction: 10-day payment requirement unless
// bond filed. Companion to § 6852 + § 6861 + § 6863 + § 7429
// jeopardy/termination constellation.
async fn section_6851_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6851::Section6851Input>,
) -> Result<Json<traderview_expense::section_6851::Section6851Result>, ApiError> {
    Ok(Json(traderview_expense::section_6851::check(&b)))
}

// ── §6861 jeopardy assessment of income, estate, gift, and certain ──
// excise taxes. Mounted at /api/calc/section-6861. § 6861(a) emergency
// authority — if Secretary believes assessment or collection of
// deficiency will be jeopardized by delay, Secretary shall
// immediately assess deficiency together with interest, additional
// amounts, and additions to tax; § 6861(b) 60-day SNOD mailing
// requirement when assessment precedes § 6212(a) SNOD; § 6861(f)
// IMMEDIATE § 6321 lien attachment + § 6331 levy authority (no
// 10-day neglect rule); § 6861(g) abatement if Tax Court determines
// deficiency less than jeopardy assessment. § 7429 review procedures
// — § 7429(a)(1)(A) Chief Counsel for IRS personal written approval
// required; § 7429(a)(1)(B) 5-day written statement requirement;
// § 7429(a)(2) 30-day administrative review window; § 7429(b)(1)
// 90-day judicial review in district court; § 7429(g) burden split
// — Secretary bears burden on reasonableness, taxpayer bears burden
// on amount appropriateness. Companion to § 6201 (assessment
// authority), § 6203 (method of assessment), § 6212 (SNOD), § 6303
// (notice and demand), § 6321 (lien), § 6331 (levy), § 6863 (stay
// of collection), § 7522 (content of notices).
async fn section_6861_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6861::Section6861Input>,
) -> Result<Json<traderview_expense::section_6861::Section6861Result>, ApiError> {
    Ok(Json(traderview_expense::section_6861::check(&b)))
}

// ── §6862 jeopardy assessment of taxes other than income, estate, ───
// gift, and certain excise taxes. Mounted at /api/calc/section-6862.
// § 6862(a) — if Secretary believes collection of any tax (OTHER than
// income tax + estate tax + gift tax + chapter 41/42/43/44 excise
// taxes) will be jeopardized by delay, Secretary shall immediately
// assess tax; whether or not due date for return and payment has
// expired; immediately due and payable. § 6862(b) — § 6331(a) levy
// without regard to 10-day notice requirement. In-scope taxes:
// employment (§ 3402 + § 3111 + § 3301 + § 3406) + excise non-
// chapter-41-44 (alcohol § 5001 + tobacco § 5701 + fuel § 4081 +
// manufacturer § 4221 + communications § 4251 + air transportation
// § 4261) + foreign withholding (§§ 1441-1446 + FATCA chapter 4).
// § 7429 procedural cluster — Chief Counsel personal approval + 5-
// day SPECIFIC FACTS AND REASONS statement (not mere conclusions) +
// 30-day administrative + 90-day judicial review + burden split. §
// 6862 + § 6863(b)(3)(A) sale prohibition SPECIFICALLY applies to
// § 6862(a) (not § 6861). Companion to § 6851 + § 6861 + § 6863 +
// § 7429 jeopardy/termination cluster.
async fn section_6862_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6862::Section6862Input>,
) -> Result<Json<traderview_expense::section_6862::Section6862Result>, ApiError> {
    Ok(Json(traderview_expense::section_6862::check(&b)))
}

// ── §6863 stay of collection of jeopardy assessments ────────────────
// Mounted at /api/calc/section-6863. Procedural pressure-relief valve
// when § 6861 (income/estate/gift jeopardy), § 6862 (other-tax
// jeopardy), § 6851 (income tax termination), or § 6852 (qualified-
// person termination) assessment has been imposed. § 6863(a) bond
// to stay collection — taxpayer may stay collection by filing bond
// in amount equal to amount of stay desired (capped at jeopardy
// amount + interest); § 6863(b)(1) bond filed before § 6213(a)
// petition triggers further condition requiring payment if petition
// not filed within 90-day window (150 days outside US); § 6863(b)(2)
// bond proportionately reduced upon final Tax Court decision if
// determining amount less than jeopardy assessment AND taxpayer
// requests; § 6863(b)(3)(A) § 6862(a) seized property may NOT be
// sold pending § 7429(b) district court judgment; § 6863(b)(3)(B)
// three sale exceptions (taxpayer consent + excessive conservation
// costs + perishable); § 6863(g) court abatement authority when
// assessment unreasonable OR amount inappropriate. Companion to
// § 6851 + § 6852 + § 6861 + § 6862 jeopardy/termination
// assessments; preserves § 6213(a) Tax Court petition right;
// subject to § 7429 review framework.
async fn section_6863_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6863::Section6863Input>,
) -> Result<Json<traderview_expense::section_6863::Section6863Result>, ApiError> {
    Ok(Json(traderview_expense::section_6863::check(&b)))
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

// ── § 274 Meals, Entertainment, Gift, Travel deduction limits ────────
// Mounted at /api/calc/section-274. Pure compute; § 274(a)
// ENTERTAINMENT fully disallowed post-TCJA 2017 § 13304 (Pub. L.
// 115-97); § 274(k) BUSINESS MEALS 50% subject to (1) not lavish/
// extravagant; (2) taxpayer or employee present; § 274(n) general
// 50% limit; § 274(o) PER SE entertainment facilities (country
// clubs, sporting events, golf, yachts, etc.) no deduction;
// § 274(b) GIFT limit $25 per recipient per year with $4
// promotional-item exception; § 274(d) substantiation (amount +
// time/place + business purpose + business relationship) with
// complete-denial on failure (Sanford v. Commissioner, 50 T.C.
// 823 (1968); COHAN RULE rejected); § 274(h) FOREIGN CONVENTION
// 4-part reasonableness test; § 274(m) LUXURY WATER TRAVEL 2×
// federal per diem daily cap (~$1,140/day 2026); temporary 100%
// restaurant meal exception 2021-2022 expired (Pub. L. 116-260
// § 210); OBBBA 2026 § 274(o) updates (Pub. L. 119-21 § 70202)
// modified employer-convenience meal deduction.

async fn section_274_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_274::Section274Input>,
) -> Result<Json<traderview_expense::section_274::Section274Result>, ApiError> {
    Ok(Json(traderview_expense::section_274::check(&b)))
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
