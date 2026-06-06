//! Trade-quality, psychology, performance, event, and portfolio analytics.
//!
//! Every endpoint is a POST that takes a JSON body matching the underlying
//! module's input shape and returns its report. No DB access — these are
//! pure compute over user-supplied inputs (typically the user's own trade
//! history or chain data).
//!
//! Provenance for which features to surface here: Edgewonk (Tiltmeter,
//! discipline efficiency, emotion logging, session report cards),
//! TraderVue (R-multiple, MFE/MAE-driven exit efficiency, streaks),
//! ThinkorSwim (probability of touch, portfolio Greeks, payoff diagrams),
//! TradingView (gap classification, calendar bias).

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::routing::post;
use axum::{Json, Router};
use chrono::{Duration, NaiveDate};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use traderview_core::{
    beta, beta_hedge, bracket_order, cagr, calendar_bias, concentration, discipline_score,
    drawdown_duration, earnings_move, emotion_tags, exit_timing, gap_analysis, halt_risk,
    hedge_ratio, high_water_mark, mae_stop_tuning, mean_reversion, models::TradeSide, overtrading,
    pead, portfolio_greeks, probability_of_touch, profit_factor, pyramid_rules, sector_exposure,
    sharpe_by_window, sortino, spread_payoff, strategy_decay, streaks, tilt_detector,
    trade_quality, treynor, volatility_regime, winloss_asymmetry,
};

pub fn router() -> Router<AppState> {
    Router::new()
        // ── Psychology / discipline ────────────────────────────────────
        .route("/analytics/tilt-detector", post(tilt_detector_route))
        .route("/analytics/discipline-score", post(discipline_score_route))
        .route("/analytics/emotion-tags", post(emotion_tags_route))
        .route("/analytics/overtrading", post(overtrading_route))
        .route("/analytics/streaks", post(streaks_route))
        .route(
            "/analytics/losing-streak-probability",
            post(losing_streak_prob_route),
        )
        .route(
            "/analytics/winloss-asymmetry",
            post(winloss_asymmetry_route),
        )
        .route("/analytics/pyramid-rules", post(pyramid_rules_route))
        // ── Performance metrics ────────────────────────────────────────
        .route("/analytics/cagr-simple", post(cagr_simple_route))
        .route("/analytics/cagr-rolling", post(cagr_rolling_route))
        .route("/analytics/profit-factor", post(profit_factor_route))
        .route("/analytics/sortino", post(sortino_route))
        .route("/analytics/treynor", post(treynor_route))
        .route("/analytics/sharpe-by-window", post(sharpe_by_window_route))
        .route("/analytics/high-water-mark", post(high_water_mark_route))
        .route(
            "/analytics/drawdown-duration",
            post(drawdown_duration_route),
        )
        // ── Event analytics ────────────────────────────────────────────
        .route(
            "/analytics/earnings-move-straddle",
            post(earnings_move_straddle_route),
        )
        .route("/analytics/earnings-move-iv", post(earnings_move_iv_route))
        .route("/analytics/pead", post(pead_route))
        .route("/analytics/gap-analysis", post(gap_analysis_route))
        .route("/analytics/calendar-bias", post(calendar_bias_route))
        .route("/analytics/halt-risk", post(halt_risk_route))
        // ── Trade-quality / execution ──────────────────────────────────
        .route("/analytics/trade-quality", post(trade_quality_route))
        .route("/analytics/exit-timing", post(exit_timing_route))
        .route("/analytics/mae-stop-tuning", post(mae_stop_tuning_route))
        .route("/analytics/bracket-order", post(bracket_order_route))
        .route(
            "/analytics/probability-of-touch",
            post(probability_of_touch_route),
        )
        // ── Portfolio + options ────────────────────────────────────────
        .route("/analytics/portfolio-greeks", post(portfolio_greeks_route))
        .route("/analytics/concentration", post(concentration_route))
        .route("/analytics/sector-exposure", post(sector_exposure_route))
        .route("/analytics/beta", post(beta_route))
        .route("/analytics/beta-hedge", post(beta_hedge_route))
        .route("/analytics/hedge-ratio", post(hedge_ratio_route))
        .route("/analytics/spread-payoff", post(spread_payoff_route))
        // ── New: Strategy decay + vol regime ────────────────────────────
        .route("/analytics/strategy-decay", post(strategy_decay_route))
        .route(
            "/analytics/volatility-regime",
            post(volatility_regime_route),
        )
        .route("/analytics/mean-reversion", post(mean_reversion_route))
}

// ──────────────────────────────────────────────────────────────────────
// Psychology / discipline
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct TiltBody {
    events: Vec<tilt_detector::TradeEvent>,
    config: tilt_detector::TiltConfig,
}

async fn tilt_detector_route(
    _u: AuthUser,
    Json(b): Json<TiltBody>,
) -> Json<Vec<tilt_detector::TiltIncident>> {
    Json(tilt_detector::scan(&b.events, &b.config))
}

async fn discipline_score_route(
    _u: AuthUser,
    Json(inputs): Json<discipline_score::ScoreInputs>,
) -> Json<discipline_score::DisciplineScore> {
    Json(discipline_score::score(&inputs))
}

#[derive(Deserialize)]
struct EmotionTagsBody {
    trades: Vec<emotion_tags::TaggedTrade>,
}

async fn emotion_tags_route(
    _u: AuthUser,
    Json(b): Json<EmotionTagsBody>,
) -> Json<emotion_tags::EmotionReport> {
    Json(emotion_tags::analyze(&b.trades))
}

#[derive(Deserialize)]
struct OvertradingBody {
    days: Vec<overtrading::DayStats>,
}

async fn overtrading_route(
    _u: AuthUser,
    Json(b): Json<OvertradingBody>,
) -> Json<overtrading::OvertradingReport> {
    Json(overtrading::analyze(&b.days))
}

#[derive(Deserialize)]
struct StreaksBody {
    pnls: Vec<f64>,
}

async fn streaks_route(_u: AuthUser, Json(b): Json<StreaksBody>) -> Json<streaks::StreaksReport> {
    Json(streaks::analyze(&b.pnls))
}

#[derive(Deserialize)]
struct LosingStreakBody {
    loss_probability: f64,
    streak_length: usize,
    sample_size: usize,
}

#[derive(Serialize)]
struct LosingStreakResp {
    probability: f64,
}

async fn losing_streak_prob_route(
    _u: AuthUser,
    Json(b): Json<LosingStreakBody>,
) -> Json<LosingStreakResp> {
    Json(LosingStreakResp {
        probability: streaks::probability_of_losing_streak(
            b.loss_probability,
            b.streak_length,
            b.sample_size,
        ),
    })
}

#[derive(Deserialize)]
struct WinLossBody {
    pnls: Vec<f64>,
}

async fn winloss_asymmetry_route(
    _u: AuthUser,
    Json(b): Json<WinLossBody>,
) -> Json<winloss_asymmetry::AsymmetryReport> {
    Json(winloss_asymmetry::analyze(&b.pnls))
}

#[derive(Deserialize)]
struct PyramidRulesBody {
    entry_price: f64,
    adds: Vec<pyramid_rules::PyramidAdd>,
    /// Average True Range at the entry — used as the spacing reference for
    /// the `min_favorable_atrs_between_adds` rule.
    atr: f64,
    is_long: bool,
    rules: pyramid_rules::PyramidRules,
}

async fn pyramid_rules_route(
    _u: AuthUser,
    Json(b): Json<PyramidRulesBody>,
) -> Json<pyramid_rules::ValidationReport> {
    Json(pyramid_rules::validate(
        b.entry_price,
        &b.adds,
        b.atr,
        b.is_long,
        &b.rules,
    ))
}

// ──────────────────────────────────────────────────────────────────────
// Performance metrics
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct CagrSimpleBody {
    beginning: f64,
    ending: f64,
    years: f64,
}

#[derive(Serialize)]
struct CagrSimpleResp {
    cagr: Option<f64>,
}

async fn cagr_simple_route(_u: AuthUser, Json(b): Json<CagrSimpleBody>) -> Json<CagrSimpleResp> {
    Json(CagrSimpleResp {
        cagr: cagr::simple(b.beginning, b.ending, b.years),
    })
}

#[derive(Deserialize)]
struct CagrRollingBody {
    equity: Vec<cagr::EquityPoint>,
    period_years: f64,
}

async fn cagr_rolling_route(
    _u: AuthUser,
    Json(b): Json<CagrRollingBody>,
) -> Json<cagr::RollingReport> {
    Json(cagr::rolling(&b.equity, b.period_years))
}

#[derive(Deserialize)]
struct ProfitFactorBody {
    trade_pnls: Vec<f64>,
    monthly_pnls: Vec<f64>,
    equity_curve: Vec<f64>,
}

async fn profit_factor_route(
    _u: AuthUser,
    Json(b): Json<ProfitFactorBody>,
) -> Json<profit_factor::SystemQualityReport> {
    Json(profit_factor::analyze(
        &b.trade_pnls,
        &b.monthly_pnls,
        &b.equity_curve,
    ))
}

#[derive(Deserialize)]
struct SortinoBody {
    returns: Vec<f64>,
    minimum_acceptable_return: f64,
    annualization: f64,
}

async fn sortino_route(_u: AuthUser, Json(b): Json<SortinoBody>) -> Json<sortino::SortinoReport> {
    Json(sortino::compute(
        &b.returns,
        b.minimum_acceptable_return,
        b.annualization,
    ))
}

#[derive(Deserialize)]
struct TreynorBody {
    portfolio_returns: Vec<f64>,
    risk_free_per_period: f64,
    beta: f64,
}

async fn treynor_route(_u: AuthUser, Json(b): Json<TreynorBody>) -> Json<treynor::TreynorReport> {
    Json(treynor::treynor(
        &b.portfolio_returns,
        b.risk_free_per_period,
        b.beta,
    ))
}

#[derive(Deserialize)]
struct SharpeWindowBody {
    returns: Vec<sharpe_by_window::TradeReturn>,
    bucket: sharpe_by_window::Bucket,
    annualization: f64,
}

async fn sharpe_by_window_route(
    _u: AuthUser,
    Json(b): Json<SharpeWindowBody>,
) -> Json<Vec<sharpe_by_window::WindowStats>> {
    Json(sharpe_by_window::by(&b.returns, b.bucket, b.annualization))
}

#[derive(Deserialize)]
struct HwmBody {
    period: high_water_mark::PeriodInput,
    rates: high_water_mark::FeeRates,
}

async fn high_water_mark_route(
    _u: AuthUser,
    Json(b): Json<HwmBody>,
) -> Json<high_water_mark::PeriodFee> {
    Json(high_water_mark::compute(&b.period, &b.rates))
}

#[derive(Deserialize)]
struct DrawdownBody {
    equity: Vec<f64>,
}

async fn drawdown_duration_route(
    _u: AuthUser,
    Json(b): Json<DrawdownBody>,
) -> Json<drawdown_duration::DrawdownReport> {
    Json(drawdown_duration::analyze(&b.equity))
}

// ──────────────────────────────────────────────────────────────────────
// Event analytics
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct EarningsStraddleBody {
    underlying: f64,
    atm_call: f64,
    atm_put: f64,
}

async fn earnings_move_straddle_route(
    _u: AuthUser,
    Json(b): Json<EarningsStraddleBody>,
) -> Json<earnings_move::ExpectedMove> {
    Json(earnings_move::from_straddle(
        b.underlying,
        b.atm_call,
        b.atm_put,
    ))
}

#[derive(Deserialize)]
struct EarningsIvBody {
    underlying: f64,
    implied_vol: f64,
    days_to_expiry: f64,
}

async fn earnings_move_iv_route(
    _u: AuthUser,
    Json(b): Json<EarningsIvBody>,
) -> Json<earnings_move::ExpectedMove> {
    Json(earnings_move::from_iv(
        b.underlying,
        b.implied_vol,
        b.days_to_expiry,
    ))
}

async fn pead_route(_u: AuthUser, Json(input): Json<pead::PeadInput>) -> Json<pead::PeadReport> {
    Json(pead::analyze(&input))
}

#[derive(Deserialize)]
struct GapBody {
    prior_close: f64,
    today_open: f64,
}

async fn gap_analysis_route(_u: AuthUser, Json(b): Json<GapBody>) -> Json<gap_analysis::GapReport> {
    Json(gap_analysis::classify(b.prior_close, b.today_open))
}

#[derive(Deserialize)]
struct CalendarBiasBody {
    trades: Vec<calendar_bias::CalendarTaggedTrade>,
}

async fn calendar_bias_route(
    _u: AuthUser,
    Json(b): Json<CalendarBiasBody>,
) -> Json<calendar_bias::CalendarBiasReport> {
    Json(calendar_bias::analyze(&b.trades))
}

#[derive(Deserialize)]
struct HaltRiskBody {
    events: Vec<halt_risk::HaltEvent>,
    /// Reference date for the report.
    now: NaiveDate,
    /// Lookback window in days. `chrono::Duration` isn't directly JSON-
    /// deserializable so we accept day-count here.
    lookback_days: i64,
}

async fn halt_risk_route(
    _u: AuthUser,
    Json(b): Json<HaltRiskBody>,
) -> Result<Json<halt_risk::HaltRiskReport>, ApiError> {
    if b.lookback_days <= 0 {
        return Err(ApiError::BadRequest("lookback_days must be > 0".into()));
    }
    Ok(Json(halt_risk::analyze(
        &b.events,
        b.now,
        Duration::days(b.lookback_days),
    )))
}

// ──────────────────────────────────────────────────────────────────────
// Trade quality / execution
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct TradeQualityBody {
    side: TradeSide,
    entry_price: Decimal,
    entry_bar: trade_quality::BarRange,
    exit_price: Decimal,
    exit_bar: trade_quality::BarRange,
}

async fn trade_quality_route(
    _u: AuthUser,
    Json(b): Json<TradeQualityBody>,
) -> Json<trade_quality::QualityScore> {
    Json(trade_quality::score(
        b.side,
        b.entry_price,
        b.entry_bar,
        b.exit_price,
        b.exit_bar,
    ))
}

#[derive(Deserialize)]
struct ExitTimingBody {
    trades: Vec<exit_timing::TradeExit>,
}

async fn exit_timing_route(
    _u: AuthUser,
    Json(b): Json<ExitTimingBody>,
) -> Json<exit_timing::ExitTimingReport> {
    Json(exit_timing::evaluate(&b.trades))
}

#[derive(Deserialize)]
struct MaeStopBody {
    trades: Vec<mae_stop_tuning::TradeMae>,
}

async fn mae_stop_tuning_route(
    _u: AuthUser,
    Json(b): Json<MaeStopBody>,
) -> Json<mae_stop_tuning::StopTuningReport> {
    Json(mae_stop_tuning::analyze(&b.trades))
}

#[derive(Deserialize)]
struct BracketOrderBody {
    order: bracket_order::BracketOrder,
    bars: Vec<bracket_order::PriceBar>,
}

async fn bracket_order_route(
    _u: AuthUser,
    Json(b): Json<BracketOrderBody>,
) -> Json<bracket_order::ResolvedBracket> {
    Json(bracket_order::resolve(&b.order, &b.bars))
}

#[derive(Deserialize)]
struct PotBody {
    spot: f64,
    strike: f64,
    sigma: f64,
    days_to_expiry: f64,
    #[serde(default = "default_pot_r")]
    r: f64,
    #[serde(default)]
    q: f64,
}
fn default_pot_r() -> f64 {
    0.045
}

async fn probability_of_touch_route(
    _u: AuthUser,
    Json(b): Json<PotBody>,
) -> Json<probability_of_touch::PotReport> {
    Json(probability_of_touch::compute(
        b.spot,
        b.strike,
        b.sigma,
        b.days_to_expiry,
        b.r,
        b.q,
    ))
}

// ──────────────────────────────────────────────────────────────────────
// Portfolio + options
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct PortfolioGreeksBody {
    positions: Vec<portfolio_greeks::OptionPosition>,
}

async fn portfolio_greeks_route(
    _u: AuthUser,
    Json(b): Json<PortfolioGreeksBody>,
) -> Json<portfolio_greeks::PortfolioGreeks> {
    Json(portfolio_greeks::aggregate(&b.positions))
}

#[derive(Deserialize)]
struct ConcentrationBody {
    holdings: Vec<concentration::Holding>,
}

async fn concentration_route(
    _u: AuthUser,
    Json(b): Json<ConcentrationBody>,
) -> Json<concentration::ConcentrationReport> {
    Json(concentration::evaluate(&b.holdings))
}

#[derive(Deserialize)]
struct SectorExposureBody {
    positions: Vec<sector_exposure::PositionWithSector>,
}

async fn sector_exposure_route(
    _u: AuthUser,
    Json(b): Json<SectorExposureBody>,
) -> Json<sector_exposure::SectorReport> {
    Json(sector_exposure::analyze(&b.positions))
}

#[derive(Deserialize)]
struct BetaBody {
    asset: Vec<f64>,
    benchmark: Vec<f64>,
}

async fn beta_route(
    _u: AuthUser,
    Json(b): Json<BetaBody>,
) -> Result<Json<beta::BetaReport>, ApiError> {
    beta::estimate(&b.asset, &b.benchmark)
        .ok_or_else(|| {
            ApiError::BadRequest(
                "asset + benchmark must be the same length and at least 2 long".into(),
            )
        })
        .map(Json)
}

#[derive(Deserialize)]
struct BetaHedgeBody {
    position_notional: f64,
    beta: f64,
    benchmark_price: f64,
    /// Fraction of position to hedge (1.0 = full neutralization, 0.5 = half).
    partial_pct: f64,
}

async fn beta_hedge_route(
    _u: AuthUser,
    Json(b): Json<BetaHedgeBody>,
) -> Json<beta_hedge::HedgeReport> {
    Json(beta_hedge::compute(
        b.position_notional,
        b.beta,
        b.benchmark_price,
        b.partial_pct,
    ))
}

#[derive(Deserialize)]
struct HedgeRatioBody {
    positions: Vec<hedge_ratio::Position>,
    beta_by_symbol: HashMap<String, Decimal>,
    spy_price: Option<Decimal>,
}

async fn hedge_ratio_route(
    _u: AuthUser,
    Json(b): Json<HedgeRatioBody>,
) -> Json<hedge_ratio::HedgeReport> {
    Json(hedge_ratio::compute(
        &b.positions,
        &b.beta_by_symbol,
        b.spy_price,
    ))
}

#[derive(Deserialize)]
struct SpreadPayoffBody {
    legs: Vec<spread_payoff::Leg>,
    price_low: f64,
    price_high: f64,
    steps: usize,
    multiplier: f64,
}

async fn spread_payoff_route(
    _u: AuthUser,
    Json(b): Json<SpreadPayoffBody>,
) -> Result<Json<spread_payoff::PayoffReport>, ApiError> {
    if b.steps == 0 || b.price_high <= b.price_low {
        return Err(ApiError::BadRequest(
            "steps > 0 and price_high > price_low required".into(),
        ));
    }
    Ok(Json(spread_payoff::payoff(
        &b.legs,
        b.price_low,
        b.price_high,
        b.steps,
        b.multiplier,
    )))
}

// ──────────────────────────────────────────────────────────────────────
// Strategy decay detector + volatility regime classifier.
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct StrategyDecayBody {
    rolling_sharpe: Vec<f64>,
    config: strategy_decay::DecayConfig,
}

async fn strategy_decay_route(
    _u: AuthUser,
    Json(b): Json<StrategyDecayBody>,
) -> Json<strategy_decay::DecayReport> {
    Json(strategy_decay::analyze(&b.rolling_sharpe, &b.config))
}

#[derive(Deserialize)]
struct VolatilityRegimeBody {
    current_vol: f64,
    history: Vec<f64>,
}

async fn volatility_regime_route(
    _u: AuthUser,
    Json(b): Json<VolatilityRegimeBody>,
) -> Json<volatility_regime::VolRegimeReport> {
    Json(volatility_regime::classify(b.current_vol, &b.history))
}

#[derive(Deserialize)]
struct MeanReversionBody {
    closes: Vec<f64>,
    config: mean_reversion::MeanRevConfig,
}

async fn mean_reversion_route(
    _u: AuthUser,
    Json(b): Json<MeanReversionBody>,
) -> Json<mean_reversion::MeanRevReport> {
    Json(mean_reversion::analyze(&b.closes, &b.config))
}
