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
        .route("/calc/section-1202",          post(section_1202_route))
        .route("/calc/section-1045",          post(section_1045_route))
        .route("/calc/section-121",           post(section_121_route))
        .route("/calc/reps-qualification",    post(reps_qualification_route))
        .route("/calc/section-163j",          post(section_163j_route))
        .route("/calc/section-267",           post(section_267_route))
        .route("/calc/section-988",           post(section_988_route))
        .route("/calc/section-1296",          post(section_1296_route))
        .route("/calc/section-168g",          post(section_168g_route))
        .route("/calc/section-163j-tradeoff", post(section_163j_tradeoff_route))
        .route("/calc/mlp-ubti",              post(mlp_ubti_route))
        .route("/calc/section-1259",          post(section_1259_route))
        .route("/calc/section-1031-f",        post(section_1031_f_route))
        .route("/calc/section-481",           post(section_481_route))
        .route("/calc/section-280f",          post(section_280f_route))
        .route("/calc/section-163d",          post(section_163d_route))
        .route("/calc/section-864b2",         post(section_864b2_route))
        .route("/calc/section-7872",          post(section_7872_route))
        .route("/calc/section-1295",          post(section_1295_route))
        .route("/calc/section-1092",          post(section_1092_route))
        .route("/calc/section-453",           post(section_453_route))
        .route("/calc/section-465",           post(section_465_route))
        .route("/calc/section-871m",          post(section_871m_route))
        .route("/calc/section-401a9",         post(section_401a9_route))
        .route("/calc/section-408-d3",        post(section_408_d3_route))
        .route("/calc/section-408m",          post(section_408m_route))
        .route("/calc/section-408a-d3",       post(section_408A_d3_route))
        .route("/calc/section-174",           post(section_174_route))
        .route("/calc/section-263a",          post(section_263a_route))
        .route("/calc/section-168-e6",        post(section_168_e6_route))
        .route("/calc/section-108",           post(section_108_route))
        .route("/calc/section-1014",          post(section_1014_route))
        .route("/calc/section-1015",          post(section_1015_route))
        .route("/calc/section-1041",          post(section_1041_route))
        .route("/calc/section-170e",          post(section_170e_route))
        .route("/calc/section-83b",           post(section_83b_route))
        .route("/calc/section-1091",          post(section_1091_route))
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
