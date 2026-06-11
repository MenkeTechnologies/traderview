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
//!   POST /calc/win-rate-confidence       — Wilson interval vs breakeven
//!   POST /calc/equity-curve-filter       — trade-above-own-MA backtest
//!   POST /sim/dual-momentum              — Antonacci GEM backtest
//!   GET  /symbols/:sym/turn-of-month     — TOM seasonality stats
//!   GET  /symbols/:sym/vol-cone          — realized-vol percentile cone
//!   GET  /symbols/:sym/day-of-week       — weekday return seasonality
//!   GET  /symbols/:sym/santa-rally       — Hirsch 7-session window stats
//!   GET  /symbols/:sym/overnight-split   — overnight vs intraday legs
//!   GET  /symbols/:sym/best-days         — miss-the-best-days study
//!   GET  /symbols/:sym/drawdown-episodes — top-N peak/trough/recovery
//!   GET  /symbols/:sym/opex-week         — third-Friday expiration window
//!   GET  /symbols/:sym/pre-holiday       — pre-holiday drift (2024+ cal)
//!   GET  /symbols/:sym/ex-div-study      — ex-date recovery behavior
//!   GET  /symbols/:sym/split-study       — split-date behavior
//!   POST /symbols/:sym/vol-rich-cheap    — IV term vs realized cone
//!   GET  /symbols/:sym/character-sheet   — all bar studies, one fetch
//!   POST /screeners/seasonality          — calendar edges across a list
//!   POST /screeners/risk                 — vol ranks + drawdown state
//!   POST /screeners/momentum             — 12-1, 52w-high, RS vs bench
//!   POST /screeners/mean-reversion       — RSI(2), 20d z, MA distance
//!   GET  /futures/:root/curve            — term structure + roll yield
//!   GET  /futures/carry-screen           — all curves ranked by roll
//!   GET  /screeners/snapshots/:name      — stored run + shape flips
//!   POST /symbols/:sym/event-study       — caller-dated FOMC/CPI study
//!   POST /calc/double-barrier            — target-vs-stop hit-first odds
//!   POST /calc/futures-sizing            — tick math + margin-capped size
//!   POST /calc/impermanent-loss          — AMM IL + fee-APR breakeven
//!   POST /calc/average-down              — position blend + bounce math
//!   POST /calc/leveraged-etf-decay       — k× daily-reset vol drag
//!   POST /calc/short-carry               — borrow/rebate/dividend carry
//!   POST /calc/asset-location            — taxable-account tax drag rank
//!   POST /calc/alpha-horizon             — signal-vs-cost breakeven days
//!   POST /calc/options-quick-math        — rule-of-16 / 0.8σ√T vs exact
//!   POST /calc/lynch-fair-value          — dividend-adjusted PEG

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
        .route("/calc/win-rate-confidence", post(post_win_rate_confidence))
        .route("/calc/equity-curve-filter", post(post_equity_curve_filter))
        .route("/sim/dual-momentum", post(post_dual_momentum))
        .route("/symbols/:symbol/turn-of-month", get(get_turn_of_month))
        .route("/symbols/:symbol/vol-cone", get(get_vol_cone))
        .route("/symbols/:symbol/day-of-week", get(get_day_of_week))
        .route("/symbols/:symbol/santa-rally", get(get_santa_rally))
        .route("/symbols/:symbol/overnight-split", get(get_overnight_split))
        .route("/symbols/:symbol/best-days", get(get_best_days))
        .route(
            "/symbols/:symbol/drawdown-episodes",
            get(get_drawdown_episodes),
        )
        .route("/symbols/:symbol/opex-week", get(get_opex_week))
        .route("/symbols/:symbol/pre-holiday", get(get_pre_holiday))
        .route("/symbols/:symbol/ex-div-study", get(get_ex_div_study))
        .route("/symbols/:symbol/split-study", get(get_split_study))
        .route("/symbols/:symbol/vol-rich-cheap", post(post_vol_rich_cheap))
        .route(
            "/symbols/:symbol/character-sheet",
            get(get_character_sheet),
        )
        .route("/screeners/seasonality", post(post_seasonality_screen))
        .route("/screeners/risk", post(post_risk_screen))
        .route("/screeners/momentum", post(post_momentum_screen))
        .route("/screeners/mean-reversion", post(post_mean_reversion_screen))
        .route("/futures/:root/curve", get(get_futures_curve))
        .route("/futures/carry-screen", get(get_carry_screen))
        .route("/screeners/snapshots/:name", get(get_screener_snapshot))
        .route("/symbols/:symbol/event-study", post(post_event_study))
        .route("/calc/double-barrier", post(post_double_barrier))
        .route("/calc/futures-sizing", post(post_futures_sizing))
        .route("/calc/impermanent-loss", post(post_impermanent_loss))
        .route("/calc/average-down", post(post_average_down))
        .route("/calc/leveraged-etf-decay", post(post_letf_decay))
        .route("/calc/short-carry", post(post_short_carry))
        .route("/calc/asset-location", post(post_asset_location))
        .route("/calc/alpha-horizon", post(post_alpha_horizon))
        .route("/calc/options-quick-math", post(post_options_quick_math))
        .route("/calc/lynch-fair-value", post(post_lynch_fair_value))
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

async fn post_win_rate_confidence(
    State(_s): State<AppState>,
    _u: AuthUser,
    Json(b): Json<traderview_core::win_rate_confidence::WinRateInput>,
) -> Result<Json<traderview_core::win_rate_confidence::WinRateReport>, ApiError> {
    traderview_core::win_rate_confidence::compute(&b)
        .map(Json)
        .ok_or_else(|| ApiError::BadRequest("invalid win-rate inputs".into()))
}

async fn post_equity_curve_filter(
    State(_s): State<AppState>,
    _u: AuthUser,
    Json(b): Json<traderview_core::equity_curve_filter::EcfInput>,
) -> Result<Json<traderview_core::equity_curve_filter::EcfReport>, ApiError> {
    traderview_core::equity_curve_filter::compute(&b)
        .map(Json)
        .ok_or_else(|| {
            ApiError::BadRequest("need more trades than the MA length, ma_length >= 2".into())
        })
}

async fn post_futures_sizing(
    State(_s): State<AppState>,
    _u: AuthUser,
    Json(b): Json<traderview_core::futures_sizing::FuturesSizingInput>,
) -> Result<Json<traderview_core::futures_sizing::FuturesSizingReport>, ApiError> {
    traderview_core::futures_sizing::compute(&b)
        .map(Json)
        .ok_or_else(|| ApiError::BadRequest("invalid sizing inputs".into()))
}

async fn post_impermanent_loss(
    State(_s): State<AppState>,
    _u: AuthUser,
    Json(b): Json<traderview_core::impermanent_loss::IlInput>,
) -> Result<Json<traderview_core::impermanent_loss::IlReport>, ApiError> {
    traderview_core::impermanent_loss::compute(&b)
        .map(Json)
        .ok_or_else(|| ApiError::BadRequest("price ratio and days must be positive".into()))
}

async fn post_average_down(
    State(_s): State<AppState>,
    _u: AuthUser,
    Json(b): Json<traderview_core::average_down::AverageDownInput>,
) -> Result<Json<traderview_core::average_down::AverageDownReport>, ApiError> {
    traderview_core::average_down::compute(&b)
        .map(Json)
        .ok_or_else(|| ApiError::BadRequest("all inputs must be positive".into()))
}

async fn post_letf_decay(
    State(_s): State<AppState>,
    _u: AuthUser,
    Json(b): Json<traderview_core::leveraged_etf_decay::LetfInput>,
) -> Result<Json<traderview_core::leveraged_etf_decay::LetfReport>, ApiError> {
    traderview_core::leveraged_etf_decay::compute(&b)
        .map(Json)
        .ok_or_else(|| ApiError::BadRequest("invalid leverage/vol inputs".into()))
}

async fn post_short_carry(
    State(_s): State<AppState>,
    _u: AuthUser,
    Json(b): Json<traderview_core::short_economics::ShortCarryInput>,
) -> Result<Json<traderview_core::short_economics::ShortCarryReport>, ApiError> {
    traderview_core::short_economics::compute(&b)
        .map(Json)
        .ok_or_else(|| ApiError::BadRequest("invalid short-carry inputs".into()))
}

async fn post_asset_location(
    State(_s): State<AppState>,
    _u: AuthUser,
    Json(b): Json<traderview_core::asset_location::AssetLocationInput>,
) -> Result<Json<traderview_core::asset_location::AssetLocationReport>, ApiError> {
    traderview_core::asset_location::compute(&b)
        .map(Json)
        .ok_or_else(|| ApiError::BadRequest("invalid asset-location inputs".into()))
}

async fn post_alpha_horizon(
    State(_s): State<AppState>,
    _u: AuthUser,
    Json(b): Json<traderview_core::alpha_horizon::AlphaHorizonInput>,
) -> Result<Json<traderview_core::alpha_horizon::AlphaHorizonReport>, ApiError> {
    traderview_core::alpha_horizon::compute(&b)
        .map(Json)
        .ok_or_else(|| ApiError::BadRequest("invalid alpha-horizon inputs".into()))
}

async fn post_options_quick_math(
    State(_s): State<AppState>,
    _u: AuthUser,
    Json(b): Json<traderview_core::options_quick_math::QuickMathInput>,
) -> Result<Json<traderview_core::options_quick_math::QuickMathReport>, ApiError> {
    traderview_core::options_quick_math::compute(&b)
        .map(Json)
        .ok_or_else(|| ApiError::BadRequest("spot, IV, and days must be positive".into()))
}

async fn post_lynch_fair_value(
    State(_s): State<AppState>,
    _u: AuthUser,
    Json(b): Json<traderview_core::lynch_fair_value::LynchInput>,
) -> Result<Json<traderview_core::lynch_fair_value::LynchReport>, ApiError> {
    traderview_core::lynch_fair_value::compute(&b)
        .map(Json)
        .ok_or_else(|| ApiError::BadRequest("invalid Lynch inputs".into()))
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

async fn get_overnight_split(
    State(s): State<AppState>,
    _u: AuthUser,
    Path(symbol): Path<String>,
    Query(q): Query<TomQ>,
) -> Result<Json<strategy_calculators::OvernightSplitReport>, ApiError> {
    let sym = validate_symbol(&symbol)?;
    strategy_calculators::overnight_split(&s.pool, &sym, q.years.unwrap_or(10))
        .await
        .map(Json)
        .map_err(map_tom_err)
}

#[derive(Debug, Deserialize)]
struct NQ {
    years: Option<u32>,
    n: Option<usize>,
}

async fn get_best_days(
    State(s): State<AppState>,
    _u: AuthUser,
    Path(symbol): Path<String>,
    Query(q): Query<NQ>,
) -> Result<Json<strategy_calculators::ConcentrationSymbolReport>, ApiError> {
    let sym = validate_symbol(&symbol)?;
    strategy_calculators::best_worst_days(&s.pool, &sym, q.years.unwrap_or(10), q.n.unwrap_or(10))
        .await
        .map(Json)
        .map_err(map_tom_err)
}

async fn get_drawdown_episodes(
    State(s): State<AppState>,
    _u: AuthUser,
    Path(symbol): Path<String>,
    Query(q): Query<NQ>,
) -> Result<Json<strategy_calculators::EpisodesSymbolReport>, ApiError> {
    let sym = validate_symbol(&symbol)?;
    strategy_calculators::drawdown_episodes(&s.pool, &sym, q.years.unwrap_or(10), q.n.unwrap_or(5))
        .await
        .map(Json)
        .map_err(map_tom_err)
}

async fn get_pre_holiday(
    State(s): State<AppState>,
    _u: AuthUser,
    Path(symbol): Path<String>,
    Query(q): Query<TomQ>,
) -> Result<Json<strategy_calculators::EventStudyReport>, ApiError> {
    let sym = validate_symbol(&symbol)?;
    strategy_calculators::pre_holiday(&s.pool, &sym, q.years.unwrap_or(5))
        .await
        .map(Json)
        .map_err(map_tom_err)
}

#[derive(Debug, Deserialize)]
struct OpexQ {
    years: Option<u32>,
    quarterly: Option<bool>,
}

async fn get_opex_week(
    State(s): State<AppState>,
    _u: AuthUser,
    Path(symbol): Path<String>,
    Query(q): Query<OpexQ>,
) -> Result<Json<strategy_calculators::EventStudyReport>, ApiError> {
    let sym = validate_symbol(&symbol)?;
    strategy_calculators::opex_week(
        &s.pool,
        &sym,
        q.years.unwrap_or(10),
        q.quarterly.unwrap_or(false),
    )
    .await
    .map(Json)
    .map_err(map_tom_err)
}

async fn get_split_study(
    State(s): State<AppState>,
    _u: AuthUser,
    Path(symbol): Path<String>,
    Query(q): Query<TomQ>,
) -> Result<Json<strategy_calculators::SplitStudyReport>, ApiError> {
    let sym = validate_symbol(&symbol)?;
    strategy_calculators::split_study(&s.pool, &sym, q.years.unwrap_or(15))
        .await
        .map(Json)
        .map_err(map_tom_err)
}

async fn get_ex_div_study(
    State(s): State<AppState>,
    _u: AuthUser,
    Path(symbol): Path<String>,
    Query(q): Query<TomQ>,
) -> Result<Json<strategy_calculators::EventStudyReport>, ApiError> {
    let sym = validate_symbol(&symbol)?;
    strategy_calculators::ex_div_study(&s.pool, &sym, q.years.unwrap_or(10))
        .await
        .map(Json)
        .map_err(map_tom_err)
}

#[derive(Debug, Deserialize)]
struct EventStudyBody {
    /// Event dates as YYYY-MM-DD strings.
    dates: Vec<String>,
    years: Option<u32>,
    window_before: Option<u32>,
    window_after: Option<u32>,
}

async fn post_event_study(
    State(s): State<AppState>,
    _u: AuthUser,
    Path(symbol): Path<String>,
    Json(b): Json<EventStudyBody>,
) -> Result<Json<strategy_calculators::EventStudyReport>, ApiError> {
    let sym = validate_symbol(&symbol)?;
    if b.dates.is_empty() || b.dates.len() > 500 {
        return Err(ApiError::BadRequest("supply 1..=500 event dates".into()));
    }
    let dates: Vec<chrono::NaiveDate> = b
        .dates
        .iter()
        .map(|s| s.trim().parse())
        .collect::<Result<_, _>>()
        .map_err(|_| ApiError::BadRequest("dates must be YYYY-MM-DD".into()))?;
    strategy_calculators::event_study(
        &s.pool,
        &sym,
        b.years.unwrap_or(10),
        &dates,
        b.window_before.unwrap_or(3).min(20),
        b.window_after.unwrap_or(3).min(20),
    )
    .await
    .map(Json)
    .map_err(map_tom_err)
}

async fn get_character_sheet(
    State(s): State<AppState>,
    _u: AuthUser,
    Path(symbol): Path<String>,
    Query(q): Query<TomQ>,
) -> Result<Json<strategy_calculators::CharacterSheet>, ApiError> {
    let sym = validate_symbol(&symbol)?;
    strategy_calculators::character_sheet(&s.pool, &sym, q.years.unwrap_or(10))
        .await
        .map(Json)
        .map_err(map_tom_err)
}

#[derive(Debug, Deserialize)]
struct SeasonalityScreenBody {
    symbols: Vec<String>,
    years: Option<u32>,
}

fn clean_symbol_list(raw: &[String]) -> Result<Vec<String>, ApiError> {
    let symbols: Vec<String> = raw
        .iter()
        .map(|x| x.trim().to_uppercase())
        .filter(|x| !x.is_empty() && x.len() <= 20)
        .take(30)
        .collect();
    if symbols.is_empty() {
        return Err(ApiError::BadRequest("supply 1..=30 symbols".into()));
    }
    Ok(symbols)
}

async fn post_seasonality_screen(
    State(s): State<AppState>,
    _u: AuthUser,
    Json(b): Json<SeasonalityScreenBody>,
) -> Result<Json<strategy_calculators::SeasonalityScreen>, ApiError> {
    let symbols = clean_symbol_list(&b.symbols)?;
    Ok(Json(
        strategy_calculators::seasonality_screen(&s.pool, &symbols, b.years.unwrap_or(10)).await,
    ))
}

async fn post_risk_screen(
    State(s): State<AppState>,
    _u: AuthUser,
    Json(b): Json<SeasonalityScreenBody>,
) -> Result<Json<strategy_calculators::RiskScreen>, ApiError> {
    let symbols = clean_symbol_list(&b.symbols)?;
    Ok(Json(
        strategy_calculators::risk_screen(&s.pool, &symbols, b.years.unwrap_or(5)).await,
    ))
}

async fn post_mean_reversion_screen(
    State(s): State<AppState>,
    _u: AuthUser,
    Json(b): Json<SeasonalityScreenBody>,
) -> Result<Json<strategy_calculators::MeanReversionScreen>, ApiError> {
    let symbols = clean_symbol_list(&b.symbols)?;
    Ok(Json(
        strategy_calculators::mean_reversion_screen(&s.pool, &symbols, b.years.unwrap_or(2))
            .await,
    ))
}

#[derive(Debug, Deserialize)]
struct FuturesCurveQ {
    /// Yahoo exchange suffix: NYM, CMX, CME, CBT (all live-verified).
    exchange: Option<String>,
    months: Option<usize>,
}

async fn get_futures_curve(
    State(s): State<AppState>,
    _u: AuthUser,
    Path(root): Path<String>,
    Query(q): Query<FuturesCurveQ>,
) -> Result<Json<strategy_calculators::FuturesCurveReport>, ApiError> {
    let root = root.trim().to_uppercase();
    if root.is_empty() || root.len() > 4 || !root.bytes().all(|b| b.is_ascii_alphanumeric()) {
        return Err(ApiError::BadRequest("invalid futures root".into()));
    }
    let exchange = q
        .exchange
        .as_deref()
        .unwrap_or("NYM")
        .trim()
        .to_uppercase();
    if !["NYM", "CMX", "CME", "CBT"].contains(&exchange.as_str()) {
        return Err(ApiError::BadRequest(
            "exchange must be one of NYM, CMX, CME, CBT".into(),
        ));
    }
    strategy_calculators::futures_curve(&s.pool, &root, &exchange, q.months.unwrap_or(8))
        .await
        .map(Json)
        .map_err(map_tom_err)
}

#[derive(Debug, Deserialize)]
struct CarryScreenQ {
    months: Option<usize>,
}

async fn get_carry_screen(
    State(s): State<AppState>,
    _u: AuthUser,
    Query(q): Query<CarryScreenQ>,
) -> Result<Json<strategy_calculators::CarryScreen>, ApiError> {
    Ok(Json(
        strategy_calculators::carry_screen(&s.pool, q.months.unwrap_or(6)).await,
    ))
}

#[derive(Debug, serde::Serialize)]
struct SnapshotResponse {
    screener: String,
    created_at: chrono::DateTime<chrono::Utc>,
    payload: serde_json::Value,
    /// Categorical flips vs the prior snapshot (carry shape,
    /// underwater state) — empty when no prior run exists.
    changes: Vec<traderview_db::screener_snapshots::ShapeChange>,
}

async fn get_screener_snapshot(
    State(s): State<AppState>,
    _u: AuthUser,
    Path(name): Path<String>,
) -> Result<Json<SnapshotResponse>, ApiError> {
    use traderview_db::screener_snapshots as snaps;
    let name = name.trim().to_lowercase();
    if !snaps::SCREENERS.contains(&name.as_str()) {
        return Err(ApiError::BadRequest(format!(
            "unknown screener — one of {:?}",
            snaps::SCREENERS
        )));
    }
    let mut rows = snaps::latest_two(&s.pool, &name)
        .await
        .map_err(ApiError::Internal)?;
    if rows.is_empty() {
        return Err(ApiError::BadRequest(
            "no snapshot yet — the background refresher runs twice a day".into(),
        ));
    }
    let latest = rows.remove(0);
    let changes = rows
        .first()
        .map(|prior| snaps::detect_changes(&prior.payload, &latest.payload))
        .unwrap_or_default();
    Ok(Json(SnapshotResponse {
        screener: name,
        created_at: latest.created_at,
        payload: latest.payload,
        changes,
    }))
}

#[derive(Debug, Deserialize)]
struct MomentumScreenBody {
    symbols: Vec<String>,
    benchmark: Option<String>,
    years: Option<u32>,
}

async fn post_momentum_screen(
    State(s): State<AppState>,
    _u: AuthUser,
    Json(b): Json<MomentumScreenBody>,
) -> Result<Json<strategy_calculators::MomentumScreen>, ApiError> {
    let symbols = clean_symbol_list(&b.symbols)?;
    let bench = b
        .benchmark
        .as_deref()
        .unwrap_or("SPY")
        .trim()
        .to_uppercase();
    if bench.is_empty() || bench.len() > 20 {
        return Err(ApiError::BadRequest("invalid benchmark".into()));
    }
    strategy_calculators::momentum_screen(&s.pool, &symbols, &bench, b.years.unwrap_or(3))
        .await
        .map(Json)
        .map_err(map_tom_err)
}

#[derive(Debug, Deserialize)]
struct VolRichCheapBody {
    /// (trading-day horizon, IV %) pairs from the live chain.
    term: Vec<(usize, f64)>,
    years: Option<u32>,
}

async fn post_vol_rich_cheap(
    State(s): State<AppState>,
    _u: AuthUser,
    Path(symbol): Path<String>,
    Json(b): Json<VolRichCheapBody>,
) -> Result<Json<strategy_calculators::VolRichCheapReport>, ApiError> {
    let sym = validate_symbol(&symbol)?;
    if b.term.is_empty() || b.term.len() > 50 {
        return Err(ApiError::BadRequest("supply 1..=50 term points".into()));
    }
    strategy_calculators::vol_rich_cheap(&s.pool, &sym, b.years.unwrap_or(5), &b.term)
        .await
        .map(Json)
        .map_err(map_tom_err)
}

#[derive(Debug, Deserialize)]
struct DoubleBarrierBody {
    spot: f64,
    lower: f64,
    upper: f64,
    #[serde(default)]
    drift: f64,
    vol: f64,
}

async fn post_double_barrier(
    State(_s): State<AppState>,
    _u: AuthUser,
    Json(b): Json<DoubleBarrierBody>,
) -> Result<Json<traderview_core::double_barrier::DoubleBarrierReport>, ApiError> {
    traderview_core::double_barrier::compute(b.spot, b.lower, b.upper, b.drift, b.vol)
        .map(Json)
        .ok_or_else(|| ApiError::BadRequest("need lower < spot < upper, vol > 0".into()))
}
