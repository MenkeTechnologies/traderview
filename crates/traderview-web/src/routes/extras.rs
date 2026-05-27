//! Final batch: discipline circuit-breakers, margin calculators, calendar
//! helpers, sentiment indicators, and execution-TCA endpoints.
//!
//! Provenance from this round's research:
//!   - **tastytrade** — vertical-spread margin (Reg-T short-option model).
//!   - **MT5** — strategy correlation matrix (Expert Advisor portfolio fit).
//!   - **eToro / Robinhood** — recurring-investment scaffolding via the
//!     position_irr (XIRR) calculator for SIPs and DRIP analysis.
//!   - **TradeStation** — TWAP execution-quality (institutional TCA).

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::routing::post;
use axum::{Json, Router};
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use traderview_core::{
    alligator, atr_cone, breakout_detector, crossover, daily_loss_limit, drawdown_throttle,
    earnings_calendar, fair_value_gap, futures_roll, goal_tracker, holiday_calendar,
    models::{Trade, TradeSide}, mtm_reconciliation, options_margin, order_block, pair_trade,
    portfolio_heat, position_aging, position_irr, put_call_ratio, range_contraction,
    reconcile_1099b, risk_reward, rolling_zscore, round_levels, sip_simulator, spread_attribution,
    stop_hunt, strategy_correlation, symbol_filter, tax_lot_optimizer, timeframe_confluence,
    triple_screen, twap, volatility_stop, volume_burst,
};

pub fn router() -> Router<AppState> {
    Router::new()
        // ── Discipline circuit breakers ────────────────────────────────
        .route("/discipline/daily-loss-limit",    post(daily_loss_limit_route))
        .route("/discipline/drawdown-throttle",   post(drawdown_throttle_route))
        .route("/discipline/goal-tracker",        post(goal_tracker_route))
        // ── Options margin (Reg-T) ─────────────────────────────────────
        .route("/options/calc/margin-naked-short", post(margin_naked_short_route))
        .route("/options/calc/margin-vertical",    post(margin_vertical_route))
        // ── Portfolio reporting ────────────────────────────────────────
        .route("/portfolio/position-aging",       post(position_aging_route))
        .route("/portfolio/position-irr",         post(position_irr_route))
        // ── Sentiment indicator ────────────────────────────────────────
        .route("/sentiment/calc/put-call-ratio",  post(put_call_ratio_route))
        // ── Tax reconciliation ─────────────────────────────────────────
        .route("/tax/reconcile-1099b",            post(reconcile_1099b_route))
        // ── Risk:reward planning ───────────────────────────────────────
        .route("/calc/risk-reward",               post(risk_reward_route))
        // ── Rolling-window analytics ───────────────────────────────────
        .route("/analytics/rolling-zscore",       post(rolling_zscore_route))
        // ── Strategy + spread analytics ────────────────────────────────
        .route("/analytics/strategy-correlation", post(strategy_correlation_route))
        .route("/analytics/spread-attribution",   post(spread_attribution_route))
        .route("/analytics/pair-trade-signal",    post(pair_trade_signal_route))
        // ── Decision systems ───────────────────────────────────────────
        .route("/discipline/triple-screen",       post(triple_screen_route))
        // ── Execution-quality TCA ──────────────────────────────────────
        .route("/microstructure/twap",            post(twap_route))
        // ── Volatility-based stops ─────────────────────────────────────
        .route("/discipline/chandelier-stop",     post(chandelier_stop_route))
        .route("/discipline/vol-stop-close",      post(vol_stop_close_route))
        // ── Broker reconciliation ──────────────────────────────────────
        .route("/portfolio/mtm-reconciliation",   post(mtm_reconciliation_route))
        // ── Forecasting cones ──────────────────────────────────────────
        .route("/charts/atr-cone",                post(atr_cone_route))
        // ── Alligator indicator ────────────────────────────────────────
        .route("/bars/alligator",                 post(alligator_route))
        // ── Calendar helpers ───────────────────────────────────────────
        .route("/calendar/is-trading-day",        post(is_trading_day_route))
        .route("/calendar/next-trading-day",      post(next_trading_day_route))
        .route("/calendar/prior-trading-day",     post(prior_trading_day_route))
        .route("/calendar/add-trading-days",      post(add_trading_days_route))
        .route("/calendar/trading-days-between",  post(trading_days_between_route))
        .route("/calendar/earnings-window",       post(earnings_window_route))
        .route("/calendar/earnings-analysis",     post(earnings_analysis_route))
        // ── Symbol filter ──────────────────────────────────────────────
        .route("/filter/symbols",                 post(symbol_filter_route))
        // ── Futures roll schedule ──────────────────────────────────────
        .route("/futures/roll-schedule",          post(futures_roll_route))
        // ── New: SIP/DRIP + portfolio heat + HIFO lot optimizer ────────
        .route("/portfolio/sip-simulator",        post(sip_simulator_route))
        .route("/portfolio/heat",                 post(portfolio_heat_route))
        .route("/tax/lot-optimizer",              post(tax_lot_optimizer_route))
        // ── New: Volume burst + round levels + timeframe confluence ────
        .route("/analytics/volume-burst",         post(volume_burst_route))
        .route("/charts/round-levels",            post(round_levels_route))
        .route("/analytics/timeframe-confluence", post(timeframe_confluence_route))
        // ── Pattern primitives: crossover + breakout + range-contraction
        .route("/analytics/crossover",            post(crossover_route))
        .route("/analytics/breakout",             post(breakout_route))
        .route("/analytics/range-contraction",    post(range_contraction_route))
        // ── SMC primitives: stop hunt + FVG + order block ──────────────
        .route("/analytics/stop-hunt",            post(stop_hunt_route))
        .route("/analytics/fair-value-gap",       post(fair_value_gap_route))
        .route("/analytics/order-block",          post(order_block_route))
}

// ──────────────────────────────────────────────────────────────────────
// Discipline
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct LossLimitBody {
    today_pnl: Decimal,
    config: daily_loss_limit::LossLimitConfig,
}

async fn daily_loss_limit_route(
    _u: AuthUser, Json(b): Json<LossLimitBody>,
) -> Json<daily_loss_limit::LossLimitReport> {
    Json(daily_loss_limit::evaluate(b.today_pnl, &b.config))
}

#[derive(Deserialize)]
struct DdThrottleBody {
    equity_history: Vec<f64>,
    config: drawdown_throttle::ThrottleConfig,
}

async fn drawdown_throttle_route(
    _u: AuthUser, Json(b): Json<DdThrottleBody>,
) -> Json<drawdown_throttle::ThrottleReport> {
    Json(drawdown_throttle::evaluate(&b.equity_history, &b.config))
}

#[derive(Deserialize)]
struct GoalTrackerBody {
    goals: goal_tracker::Goals,
    equity_history: Vec<f64>,
    today: NaiveDate,
}

async fn goal_tracker_route(
    _u: AuthUser, Json(b): Json<GoalTrackerBody>,
) -> Json<goal_tracker::ProgressReport> {
    Json(goal_tracker::evaluate(&b.goals, &b.equity_history, b.today))
}

// ──────────────────────────────────────────────────────────────────────
// Options margin
// ──────────────────────────────────────────────────────────────────────

async fn margin_naked_short_route(
    _u: AuthUser, Json(opt): Json<options_margin::NakedShortOption>,
) -> Json<options_margin::MarginReport> {
    Json(options_margin::naked_short(&opt))
}

async fn margin_vertical_route(
    _u: AuthUser, Json(spread): Json<options_margin::VerticalSpread>,
) -> Json<options_margin::MarginReport> {
    Json(options_margin::vertical(&spread))
}

// ──────────────────────────────────────────────────────────────────────
// Portfolio
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct PositionAgingBody {
    positions: Vec<position_aging::OpenPosition>,
    now: DateTime<Utc>,
    /// Position is flagged as stale after holding this many days.
    stale_threshold_days: i64,
}

async fn position_aging_route(
    _u: AuthUser, Json(b): Json<PositionAgingBody>,
) -> Json<position_aging::AgingReport> {
    Json(position_aging::evaluate(&b.positions, b.now, b.stale_threshold_days))
}

#[derive(Deserialize)]
struct PositionIrrBody { flows: Vec<position_irr::CashFlow> }

#[derive(Serialize)]
struct PositionIrrResp { irr: Option<f64> }

async fn position_irr_route(
    _u: AuthUser, Json(b): Json<PositionIrrBody>,
) -> Json<PositionIrrResp> {
    Json(PositionIrrResp { irr: position_irr::annualized_irr(&b.flows) })
}

// ──────────────────────────────────────────────────────────────────────
// Sentiment
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct PutCallRatioBody {
    input: put_call_ratio::PutCallInput,
    thresholds: put_call_ratio::Thresholds,
}

async fn put_call_ratio_route(
    _u: AuthUser, Json(b): Json<PutCallRatioBody>,
) -> Json<put_call_ratio::PutCallReport> {
    Json(put_call_ratio::compute(&b.input, &b.thresholds))
}

// ──────────────────────────────────────────────────────────────────────
// Tax reconciliation
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct Reconcile1099bBody {
    year: i32,
    trades: Vec<Trade>,
    rows: Vec<reconcile_1099b::B1099Row>,
}

async fn reconcile_1099b_route(
    _u: AuthUser, Json(b): Json<Reconcile1099bBody>,
) -> Json<reconcile_1099b::ReconReport> {
    Json(reconcile_1099b::reconcile(b.year, &b.trades, &b.rows))
}

// ──────────────────────────────────────────────────────────────────────
// Risk:reward
// ──────────────────────────────────────────────────────────────────────

async fn risk_reward_route(
    _u: AuthUser, Json(input): Json<risk_reward::RrInput>,
) -> Result<Json<risk_reward::RrReport>, ApiError> {
    risk_reward::compute(&input)
        .map(Json)
        .map_err(|e| ApiError::BadRequest(e.into()))
}

// ──────────────────────────────────────────────────────────────────────
// Rolling window analytics
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct RollingZBody { series: Vec<f64>, window: usize }

async fn rolling_zscore_route(
    _u: AuthUser, Json(b): Json<RollingZBody>,
) -> Json<Vec<rolling_zscore::ZPoint>> {
    Json(rolling_zscore::compute(&b.series, b.window))
}

// ──────────────────────────────────────────────────────────────────────
// Strategy + spread analytics
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct StrategyCorrelationBody {
    strategies: Vec<strategy_correlation::StrategyReturns>,
    high_threshold: f64,
}

async fn strategy_correlation_route(
    _u: AuthUser, Json(b): Json<StrategyCorrelationBody>,
) -> Json<strategy_correlation::CorrReport> {
    Json(strategy_correlation::analyze(&b.strategies, b.high_threshold))
}

async fn spread_attribution_route(
    _u: AuthUser, Json(t): Json<spread_attribution::PairTrade>,
) -> Json<spread_attribution::AttributionReport> {
    Json(spread_attribution::attribute(&t))
}

#[derive(Deserialize)]
struct PairTradeBody {
    /// y-leg price series (regression dependent).
    y: Vec<f64>,
    /// x-leg price series (regression independent).
    x: Vec<f64>,
    config: pair_trade::PairConfig,
}

async fn pair_trade_signal_route(
    _u: AuthUser, Json(b): Json<PairTradeBody>,
) -> Result<Json<pair_trade::PairReport>, ApiError> {
    pair_trade::analyze(&b.y, &b.x, &b.config)
        .ok_or_else(|| ApiError::BadRequest(
            "y and x must be the same length, at least 3 long, with non-zero x-variance".into()
        ))
        .map(Json)
}

// ──────────────────────────────────────────────────────────────────────
// Decision systems
// ──────────────────────────────────────────────────────────────────────

async fn triple_screen_route(
    _u: AuthUser, Json(input): Json<triple_screen::TripleScreenInput>,
) -> Json<TripleScreenResp> {
    Json(TripleScreenResp { verdict: triple_screen::evaluate(&input) })
}

#[derive(Serialize)]
struct TripleScreenResp { verdict: triple_screen::Verdict }

// ──────────────────────────────────────────────────────────────────────
// TWAP TCA
// ──────────────────────────────────────────────────────────────────────

async fn twap_route(
    _u: AuthUser, Json(input): Json<twap::TwapInput>,
) -> Json<TwapResp> {
    Json(TwapResp { result: twap::compute(&input) })
}

#[derive(Serialize)]
struct TwapResp { result: Option<twap::TwapResult> }

// ──────────────────────────────────────────────────────────────────────
// Volatility-based stops
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct ChandelierBody {
    bars: Vec<volatility_stop::Bar>,
    atr: Vec<f64>,
    side: TradeSide,
    config: volatility_stop::StopConfig,
}

async fn chandelier_stop_route(
    _u: AuthUser, Json(b): Json<ChandelierBody>,
) -> Json<Vec<volatility_stop::StopPoint>> {
    Json(volatility_stop::chandelier(&b.bars, &b.atr, b.side, &b.config))
}

async fn vol_stop_close_route(
    _u: AuthUser, Json(b): Json<ChandelierBody>,
) -> Json<Vec<volatility_stop::StopPoint>> {
    Json(volatility_stop::vol_stop_close(&b.bars, &b.atr, b.side, &b.config))
}

// ──────────────────────────────────────────────────────────────────────
// Broker reconciliation
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct MtmReconciliationBody {
    broker: Vec<mtm_reconciliation::BrokerPosition>,
    internal: Vec<mtm_reconciliation::InternalPosition>,
    threshold_dollars: Decimal,
}

async fn mtm_reconciliation_route(
    _u: AuthUser, Json(b): Json<MtmReconciliationBody>,
) -> Json<mtm_reconciliation::ReconciliationReport> {
    Json(mtm_reconciliation::reconcile(&b.broker, &b.internal, b.threshold_dollars))
}

// ──────────────────────────────────────────────────────────────────────
// ATR cone projection
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct AtrConeBody { entry: f64, daily_atr: f64, horizon_days: usize }

async fn atr_cone_route(
    _u: AuthUser, Json(b): Json<AtrConeBody>,
) -> Json<Vec<atr_cone::ConePoint>> {
    Json(atr_cone::project(b.entry, b.daily_atr, b.horizon_days))
}

// ──────────────────────────────────────────────────────────────────────
// Alligator
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct AlligatorBody { bars: Vec<alligator::Bar> }

async fn alligator_route(
    _u: AuthUser, Json(b): Json<AlligatorBody>,
) -> Json<Vec<alligator::AlligatorPoint>> {
    Json(alligator::compute(&b.bars))
}

// ──────────────────────────────────────────────────────────────────────
// Calendar helpers
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct DateBody { date: NaiveDate }

#[derive(Serialize)]
struct BoolResp { value: bool }

#[derive(Serialize)]
struct DateResp { date: NaiveDate }

#[derive(Serialize)]
struct CountResp { count: i32 }

async fn is_trading_day_route(_u: AuthUser, Json(b): Json<DateBody>) -> Json<BoolResp> {
    Json(BoolResp { value: holiday_calendar::is_trading_day(b.date) })
}

async fn next_trading_day_route(_u: AuthUser, Json(b): Json<DateBody>) -> Json<DateResp> {
    Json(DateResp { date: holiday_calendar::next_trading_day(b.date) })
}

async fn prior_trading_day_route(_u: AuthUser, Json(b): Json<DateBody>) -> Json<DateResp> {
    Json(DateResp { date: holiday_calendar::prior_trading_day(b.date) })
}

#[derive(Deserialize)]
struct AddTradingDaysBody { date: NaiveDate, days: i32 }

async fn add_trading_days_route(
    _u: AuthUser, Json(b): Json<AddTradingDaysBody>,
) -> Json<DateResp> {
    Json(DateResp { date: holiday_calendar::add_trading_days(b.date, b.days) })
}

#[derive(Deserialize)]
struct TradingDaysBetweenBody { start: NaiveDate, end: NaiveDate }

async fn trading_days_between_route(
    _u: AuthUser, Json(b): Json<TradingDaysBetweenBody>,
) -> Json<CountResp> {
    Json(CountResp { count: holiday_calendar::trading_days_between(b.start, b.end) })
}

#[derive(Deserialize)]
struct EarningsWindowBody {
    events: Vec<earnings_calendar::EarningsEvent>,
    today: NaiveDate,
    /// Look-ahead window in days when checking whether earnings fall inside
    /// the trade-holding period.
    hold_days: i64,
}

async fn earnings_window_route(
    _u: AuthUser, Json(b): Json<EarningsWindowBody>,
) -> Json<Vec<String>> {
    Json(earnings_calendar::earnings_within_window(&b.events, b.today, b.hold_days))
}

#[derive(Deserialize)]
struct EarningsAnalysisBody {
    events: Vec<earnings_calendar::EarningsEvent>,
    today: NaiveDate,
}

async fn earnings_analysis_route(
    _u: AuthUser, Json(b): Json<EarningsAnalysisBody>,
) -> Json<earnings_calendar::EarningsReport> {
    Json(earnings_calendar::analyze(&b.events, b.today))
}

// ──────────────────────────────────────────────────────────────────────
// Symbol filter
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct SymbolFilterBody {
    filter: symbol_filter::SymbolFilter,
    symbol: String,
}

#[derive(Serialize)]
struct SymbolFilterResp { decision: symbol_filter::FilterDecision }

async fn symbol_filter_route(
    _u: AuthUser, Json(b): Json<SymbolFilterBody>,
) -> Json<SymbolFilterResp> {
    Json(SymbolFilterResp { decision: b.filter.check(&b.symbol) })
}

// ──────────────────────────────────────────────────────────────────────
// Futures roll schedule — surfaces contracts approaching expiration so
// the trader can roll forward before liquidity dries up. tastytrade /
// IBKR / NinjaTrader-class feature.
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct FuturesRollBody {
    positions: Vec<futures_roll::FuturesPosition>,
    today: NaiveDate,
    /// Days-out window to surface upcoming rolls.
    roll_window_days: i64,
}

async fn futures_roll_route(
    _u: AuthUser, Json(b): Json<FuturesRollBody>,
) -> Json<futures_roll::RollReport> {
    Json(futures_roll::schedule(&b.positions, b.today, b.roll_window_days))
}

// ──────────────────────────────────────────────────────────────────────
// SIP/DRIP simulator — eToro/Robinhood/Coinbase recurring-deposit math.
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct SipBody {
    bars: Vec<sip_simulator::PriceBar>,
    spec: sip_simulator::ScheduleSpec,
}

async fn sip_simulator_route(
    _u: AuthUser, Json(b): Json<SipBody>,
) -> Json<sip_simulator::SipReport> {
    Json(sip_simulator::simulate(&b.bars, &b.spec))
}

// ──────────────────────────────────────────────────────────────────────
// Portfolio heat — correlated-position budget enforcement.
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct PortfolioHeatBody {
    open_positions: Vec<portfolio_heat::OpenRiskPosition>,
    correlations: Vec<portfolio_heat::CorrEdge>,
    candidate: portfolio_heat::CandidateTrade,
    config: portfolio_heat::HeatConfig,
}

async fn portfolio_heat_route(
    _u: AuthUser, Json(b): Json<PortfolioHeatBody>,
) -> Json<portfolio_heat::HeatReport> {
    Json(portfolio_heat::evaluate(&b.open_positions, &b.correlations, &b.candidate, &b.config))
}

// ──────────────────────────────────────────────────────────────────────
// Tax-lot optimizer — HIFO / Lifoust / MaxLossHarvest selection.
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct TaxLotOptimizerBody {
    lots: Vec<tax_lot_optimizer::CostLot>,
    qty_to_close: Decimal,
    sell_price: Decimal,
    strategy: tax_lot_optimizer::LotStrategy,
}

async fn tax_lot_optimizer_route(
    _u: AuthUser, Json(b): Json<TaxLotOptimizerBody>,
) -> Json<tax_lot_optimizer::CloseReport> {
    Json(tax_lot_optimizer::close(&b.lots, b.qty_to_close, b.sell_price, b.strategy))
}

// ──────────────────────────────────────────────────────────────────────
// Volume burst + round levels + timeframe confluence.
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct VolumeBurstBody {
    bars: Vec<volume_burst::VolumeBar>,
    config: volume_burst::BurstConfig,
}

async fn volume_burst_route(
    _u: AuthUser, Json(b): Json<VolumeBurstBody>,
) -> Json<volume_burst::BurstReport> {
    Json(volume_burst::detect(&b.bars, &b.config))
}

#[derive(Deserialize)]
struct RoundLevelsBody {
    current_price: f64,
    /// Optional ATR for distance-in-ATRs annotations.
    atr: Option<f64>,
    config: round_levels::LevelsConfig,
}

async fn round_levels_route(
    _u: AuthUser, Json(b): Json<RoundLevelsBody>,
) -> Json<round_levels::LevelsReport> {
    Json(round_levels::detect(b.current_price, b.atr, &b.config))
}

#[derive(Deserialize)]
struct ConfluenceBody { verdicts: Vec<timeframe_confluence::TimeframeVerdict> }

async fn timeframe_confluence_route(
    _u: AuthUser, Json(b): Json<ConfluenceBody>,
) -> Json<timeframe_confluence::ConfluenceReport> {
    Json(timeframe_confluence::analyze(&b.verdicts))
}

// ──────────────────────────────────────────────────────────────────────
// Crossover + breakout + range-contraction primitives.
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct CrossoverBody { a: Vec<Option<f64>>, b: Vec<Option<f64>> }

async fn crossover_route(
    _u: AuthUser, Json(body): Json<CrossoverBody>,
) -> Json<crossover::CrossReport> {
    Json(crossover::detect(&body.a, &body.b))
}

#[derive(Deserialize)]
struct BreakoutBody {
    bars: Vec<breakout_detector::OhlcBar>,
    config: breakout_detector::BreakoutConfig,
}

async fn breakout_route(
    _u: AuthUser, Json(b): Json<BreakoutBody>,
) -> Json<breakout_detector::BreakoutReport> {
    Json(breakout_detector::detect(&b.bars, &b.config))
}

#[derive(Deserialize)]
struct RangeContractionBody { bars: Vec<range_contraction::OhlcBar> }

async fn range_contraction_route(
    _u: AuthUser, Json(b): Json<RangeContractionBody>,
) -> Json<range_contraction::PatternReport> {
    Json(range_contraction::detect(&b.bars))
}

// ──────────────────────────────────────────────────────────────────────
// Smart-money concepts: stop hunt + fair value gap + order block.
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct StopHuntBody {
    bars: Vec<stop_hunt::OhlcBar>,
    config: stop_hunt::StopHuntConfig,
}

async fn stop_hunt_route(
    _u: AuthUser, Json(b): Json<StopHuntBody>,
) -> Json<stop_hunt::SweepReport> {
    Json(stop_hunt::detect(&b.bars, &b.config))
}

#[derive(Deserialize)]
struct FairValueGapBody { bars: Vec<fair_value_gap::OhlcBar> }

async fn fair_value_gap_route(
    _u: AuthUser, Json(b): Json<FairValueGapBody>,
) -> Json<fair_value_gap::FvgReport> {
    Json(fair_value_gap::detect(&b.bars))
}

#[derive(Deserialize)]
struct OrderBlockBody {
    bars: Vec<order_block::OhlcBar>,
    config: order_block::OrderBlockConfig,
}

async fn order_block_route(
    _u: AuthUser, Json(b): Json<OrderBlockBody>,
) -> Json<order_block::OrderBlockReport> {
    Json(order_block::detect(&b.bars, &b.config))
}
