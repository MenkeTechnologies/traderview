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
