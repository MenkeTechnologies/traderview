//! Market-microstructure + regime + execution-quality endpoints.
//!
//! Feature provenance:
//!   - **Bookmap / Sierra Chart** — order-flow imbalance, liquidity over time,
//!     market impact (footprint / volume-profile family).
//!   - **DAS Trader Pro / Lightspeed** — order staleness, time-in-force
//!     enforcement, open-type classification (institutional intraday models).
//!   - **TrendSpider** — pattern + heatmap automation (intraday-heatmap,
//!     dow_hour heatmap surfaced here as bucket reports the frontend can paint).
//!   - **IBKR TWS** — risk regime detection, news-event handling, pre-trade
//!     checklists (institutional-grade risk slicing).
//!
//! All POST. Compute-only, no DB-scoped state.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use uuid::Uuid;
use axum::routing::post;
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use traderview_core::{
    cluster_analysis, correlation_clusters, dow_hour_heatmap, equity_regime, footprint,
    implementation_shortfall, intraday_heatmap, iv_backtest, iv_rank, liquidity, market_impact,
    market_profile, models::Trade, news_event_handler, oi_change, open_type, order_book_imbalance,
    order_flow, order_staleness, per_symbol_slippage, pyramid, setup_catalog, spread_tracker,
    stop_loss_backtest, stress_test, tilt_indicator, time_in_force, trade_plan_checklist,
    vwap_slippage,
};

pub fn router() -> Router<AppState> {
    Router::new()
        // ── Market microstructure ──────────────────────────────────────
        .route("/microstructure/order-book-imbalance", post(order_book_imbalance_route))
        .route("/microstructure/order-flow-classify",  post(order_flow_classify_route))
        .route("/microstructure/order-flow-aggregate", post(order_flow_aggregate_route))
        .route("/microstructure/liquidity",            post(liquidity_route))
        .route("/microstructure/market-impact",        post(market_impact_route))
        .route("/microstructure/per-symbol-slippage",  post(per_symbol_slippage_route))
        .route("/microstructure/vwap-slippage",        post(vwap_slippage_route))
        .route("/microstructure/order-staleness",      post(order_staleness_route))
        // ── Heatmaps ───────────────────────────────────────────────────
        .route("/heatmaps/intraday",                   post(intraday_heatmap_route))
        .route("/heatmaps/dow-hour",                   post(dow_hour_heatmap_route))
        // ── Regime + discipline ────────────────────────────────────────
        .route("/regime/equity",                       post(equity_regime_route))
        .route("/regime/news-event",                   post(news_event_route))
        .route("/discipline/time-in-force",            post(time_in_force_route))
        .route("/discipline/open-type",                post(open_type_route))
        .route("/discipline/trade-plan-checklist",     post(trade_plan_checklist_route))
        .route("/discipline/stop-loss-backtest",       post(stop_loss_backtest_route))
        .route("/discipline/stop-loss-best-of",        post(stop_loss_best_of_route))
        .route("/discipline/pyramid-plan",             post(pyramid_plan_route))
        // ── Options IV/OI history ──────────────────────────────────────
        .route("/options/calc/iv-rank",                post(iv_rank_route))
        .route("/options/calc/iv-backtest",            post(iv_backtest_route))
        .route("/options/calc/oi-change",              post(oi_change_route))
        // ── Clustering ─────────────────────────────────────────────────
        .route("/clusters/trade-features",             post(cluster_analysis_route))
        .route("/clusters/correlation",                post(correlation_clusters_route))
        // ── Setup tracking ─────────────────────────────────────────────
        .route("/setups/by-setup",                     post(setup_stats_route))
        // ── New: Footprint / TPO / Stress test ─────────────────────────
        .route("/microstructure/footprint",            post(footprint_route))
        .route("/microstructure/market-profile",       post(market_profile_route))
        .route("/microstructure/stress-test",          post(stress_test_route))
        // ── Cohort tilt indicator (TopstepX) ───────────────────────────
        .route("/sentiment/cohort-tilt",               post(cohort_tilt_route))
        // ── New: spread tracker + implementation shortfall ─────────────
        .route("/microstructure/spread-tracker",       post(spread_tracker_route))
        .route("/microstructure/implementation-shortfall", post(implementation_shortfall_route))
}

// ──────────────────────────────────────────────────────────────────────
// Market microstructure
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct ObiBody {
    bid_sizes: Vec<f64>,
    ask_sizes: Vec<f64>,
    levels: usize,
}

async fn order_book_imbalance_route(
    _u: AuthUser, Json(b): Json<ObiBody>,
) -> Json<order_book_imbalance::ObiReport> {
    Json(order_book_imbalance::compute(&b.bid_sizes, &b.ask_sizes, b.levels))
}

#[derive(Deserialize)]
struct OrderFlowBody { ticks: Vec<order_flow::Tick> }

async fn order_flow_classify_route(
    _u: AuthUser, Json(b): Json<OrderFlowBody>,
) -> Json<Vec<order_flow::ClassifiedTick>> {
    Json(order_flow::classify(&b.ticks))
}

async fn order_flow_aggregate_route(
    _u: AuthUser, Json(b): Json<OrderFlowBody>,
) -> Json<order_flow::ImbalanceReport> {
    let classified = order_flow::classify(&b.ticks);
    Json(order_flow::aggregate(&classified))
}

#[derive(Deserialize)]
struct LiquidityBody {
    trades: Vec<Trade>,
    /// Average daily volume per symbol — keys must be uppercase symbol IDs.
    adv: HashMap<String, Decimal>,
}

async fn liquidity_route(
    _u: AuthUser, Json(b): Json<LiquidityBody>,
) -> Json<liquidity::LiquidityReport> {
    Json(liquidity::liquidity(&b.trades, &b.adv))
}

#[derive(Deserialize)]
struct MarketImpactBody {
    trades: Vec<market_impact::TradeImpact>,
    /// Bps above which slippage is considered to be "spiking" — defines
    /// the participation-rate threshold the analyzer flags.
    spike_bps: f64,
}

async fn market_impact_route(
    _u: AuthUser, Json(b): Json<MarketImpactBody>,
) -> Json<market_impact::MarketImpactReport> {
    Json(market_impact::analyze(&b.trades, b.spike_bps))
}

#[derive(Deserialize)]
struct SlippageRecord { symbol: String, slippage_bps: f64 }

#[derive(Deserialize)]
struct PerSymbolSlippageBody { records: Vec<SlippageRecord> }

async fn per_symbol_slippage_route(
    _u: AuthUser, Json(b): Json<PerSymbolSlippageBody>,
) -> Json<Vec<per_symbol_slippage::SymbolSlippage>> {
    let tuples: Vec<(String, f64)> = b.records.into_iter()
        .map(|r| (r.symbol, r.slippage_bps))
        .collect();
    Json(per_symbol_slippage::aggregate(&tuples))
}

// vwap_slippage::VwapInput has `#[serde(skip)]` on its bars field, so we
// accept a parallel wrapper here and reconstruct it server-side.
#[derive(Deserialize)]
struct VwapSlippageBody {
    side: traderview_core::TradeSide,
    fill_price: Decimal,
    bars: Vec<vwap_slippage::BarOhlcv>,
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum VwapSlippageResp {
    Computed(vwap_slippage::VwapResult),
    Empty { reason: &'static str },
}

async fn vwap_slippage_route(
    _u: AuthUser, Json(b): Json<VwapSlippageBody>,
) -> Json<VwapSlippageResp> {
    let input = vwap_slippage::VwapInput { side: b.side, fill_price: b.fill_price, bars: b.bars };
    let resp = match vwap_slippage::compute(&input) {
        Some(r) => VwapSlippageResp::Computed(r),
        None    => VwapSlippageResp::Empty { reason: "no bars or zero total volume" },
    };
    Json(resp)
}

#[derive(Deserialize)]
struct StalenessBody {
    orders: Vec<order_staleness::RestingOrder>,
    now: DateTime<Utc>,
    thresholds: order_staleness::StaleThresholds,
}

async fn order_staleness_route(
    _u: AuthUser, Json(b): Json<StalenessBody>,
) -> Json<order_staleness::StaleReport> {
    Json(order_staleness::evaluate(&b.orders, b.now, &b.thresholds))
}

// ──────────────────────────────────────────────────────────────────────
// Heatmaps
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct IntradayHeatmapBody { trades: Vec<intraday_heatmap::IntradayTrade> }

async fn intraday_heatmap_route(
    _u: AuthUser, Json(b): Json<IntradayHeatmapBody>,
) -> Json<intraday_heatmap::IntradayHeatmapReport> {
    Json(intraday_heatmap::build(&b.trades))
}

#[derive(Deserialize)]
struct DowHourHeatmapBody { trades: Vec<Trade> }

async fn dow_hour_heatmap_route(
    _u: AuthUser, Json(b): Json<DowHourHeatmapBody>,
) -> Json<dow_hour_heatmap::DowHourHeatmap> {
    Json(dow_hour_heatmap::build(&b.trades))
}

// ──────────────────────────────────────────────────────────────────────
// Regime + discipline
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct EquityRegimeBody {
    equity: Vec<f64>,
    config: equity_regime::DetectorConfig,
}

async fn equity_regime_route(
    _u: AuthUser, Json(b): Json<EquityRegimeBody>,
) -> Json<equity_regime::RegimeReport> {
    Json(equity_regime::analyze(&b.equity, &b.config))
}

#[derive(Deserialize)]
struct NewsEventBody {
    positions: Vec<news_event_handler::OpenPosition>,
    events: Vec<news_event_handler::NewsEvent>,
}

async fn news_event_route(
    _u: AuthUser, Json(b): Json<NewsEventBody>,
) -> Json<news_event_handler::NewsActionReport> {
    Json(news_event_handler::evaluate(&b.positions, &b.events))
}

#[derive(Deserialize)]
struct TifBody {
    order: time_in_force::OrderState,
    now: DateTime<Utc>,
    /// Day-of-week session-open date used to compute session boundaries.
    session_open: chrono::NaiveDate,
}

async fn time_in_force_route(
    _u: AuthUser, Json(b): Json<TifBody>,
) -> Json<time_in_force::TifVerdict> {
    Json(time_in_force::evaluate(&b.order, b.now, b.session_open))
}

async fn open_type_route(
    _u: AuthUser, Json(input): Json<open_type::OpenInput>,
) -> Json<open_type::OpenTypeReport> {
    Json(open_type::classify(&input))
}

#[derive(Deserialize)]
struct ChecklistBody {
    plan: trade_plan_checklist::PlannedTrade,
    config: trade_plan_checklist::ChecklistConfig,
}

async fn trade_plan_checklist_route(
    _u: AuthUser, Json(b): Json<ChecklistBody>,
) -> Json<trade_plan_checklist::ChecklistReport> {
    Json(trade_plan_checklist::evaluate(&b.plan, &b.config))
}

#[derive(Deserialize)]
struct StopBacktestBody {
    trades: Vec<stop_loss_backtest::TradeOutcome>,
    params: stop_loss_backtest::StopParams,
    side_long: bool,
}

async fn stop_loss_backtest_route(
    _u: AuthUser, Json(b): Json<StopBacktestBody>,
) -> Json<stop_loss_backtest::MethodResult> {
    Json(stop_loss_backtest::simulate(&b.trades, &b.params, b.side_long))
}

#[derive(Deserialize)]
struct StopBestOfBody {
    trades: Vec<stop_loss_backtest::TradeOutcome>,
    candidates: Vec<stop_loss_backtest::StopParams>,
    side_long: bool,
}

async fn stop_loss_best_of_route(
    _u: AuthUser, Json(b): Json<StopBestOfBody>,
) -> Json<Vec<stop_loss_backtest::MethodResult>> {
    Json(stop_loss_backtest::best_of(&b.trades, &b.candidates, b.side_long))
}

async fn pyramid_plan_route(
    _u: AuthUser, Json(input): Json<pyramid::PlanInput>,
) -> Json<pyramid::PlanReport> {
    Json(pyramid::build(&input))
}

// ──────────────────────────────────────────────────────────────────────
// Options IV / OI history
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct IvRankBody { current_iv: f64, history: Vec<f64> }

async fn iv_rank_route(
    _u: AuthUser, Json(b): Json<IvRankBody>,
) -> Json<iv_rank::IvRankReport> {
    Json(iv_rank::compute(b.current_iv, &b.history))
}

#[derive(Deserialize)]
struct IvBacktestBody {
    implied_move_pct: f64,
    realized_pcts: Vec<f64>,
}

async fn iv_backtest_route(
    _u: AuthUser, Json(b): Json<IvBacktestBody>,
) -> Json<iv_backtest::StraddleBacktest> {
    Json(iv_backtest::backtest(b.implied_move_pct, &b.realized_pcts))
}

#[derive(Deserialize)]
struct OiChangeBody {
    snapshots: Vec<oi_change::StrikeOiSnapshot>,
    pct_threshold: f64,
    min_oi: u64,
}

async fn oi_change_route(
    _u: AuthUser, Json(b): Json<OiChangeBody>,
) -> Json<oi_change::OiAlertReport> {
    Json(oi_change::analyze(&b.snapshots, b.pct_threshold, b.min_oi))
}

// ──────────────────────────────────────────────────────────────────────
// Clustering
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct ClusterAnalysisBody {
    features: Vec<cluster_analysis::TradeFeature>,
    k: usize,
    max_iters: usize,
}

async fn cluster_analysis_route(
    _u: AuthUser, Json(b): Json<ClusterAnalysisBody>,
) -> Json<cluster_analysis::ClusterReport> {
    Json(cluster_analysis::analyze(&b.features, b.k, b.max_iters))
}

#[derive(Deserialize)]
struct CorrEdge { a: String, b: String, corr: f64 }

#[derive(Deserialize)]
struct CorrelationClustersBody {
    positions: Vec<correlation_clusters::Position>,
    /// Pairwise correlations. The {a, b, corr} shape is more JSON-friendly
    /// than a map keyed by (String, String). Order of a/b does not matter
    /// — the engine looks up both orderings.
    correlations: Vec<CorrEdge>,
    threshold: f64,
}

async fn correlation_clusters_route(
    _u: AuthUser, Json(b): Json<CorrelationClustersBody>,
) -> Json<Vec<correlation_clusters::Cluster>> {
    let mut corr: BTreeMap<(String, String), f64> = BTreeMap::new();
    for e in b.correlations {
        // Insert both directions so the engine finds the edge regardless of
        // which symbol it iterates first.
        corr.insert((e.a.clone(), e.b.clone()), e.corr);
        corr.insert((e.b, e.a), e.corr);
    }
    Json(correlation_clusters::cluster(&b.positions, &corr, b.threshold))
}

// ──────────────────────────────────────────────────────────────────────
// Setup tracking
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct SetupStatsBody {
    trades: Vec<Trade>,
    /// trade_id → setup name. Trades not in this map are treated as untagged
    /// and excluded from the per-setup stats.
    trade_setups: HashMap<Uuid, String>,
}

async fn setup_stats_route(
    _u: AuthUser, Json(b): Json<SetupStatsBody>,
) -> Json<Vec<setup_catalog::SetupStats>> {
    Json(setup_catalog::stats_by_setup(&b.trades, &b.trade_setups))
}

// ──────────────────────────────────────────────────────────────────────
// Footprint chart — per-bar bid/ask volume + delta. Sierra/Bookmap class.
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct FootprintBody {
    ticks: Vec<footprint::BarTick>,
    /// Price quantization step (e.g. 0.01 for stocks, 0.25 for ES).
    tick_size: f64,
}

async fn footprint_route(
    _u: AuthUser, Json(b): Json<FootprintBody>,
) -> Result<Json<footprint::FootprintReport>, ApiError> {
    if !(b.tick_size.is_finite() && b.tick_size > 0.0) {
        return Err(ApiError::BadRequest("tick_size must be a positive finite number".into()));
    }
    Ok(Json(footprint::build(&b.ticks, b.tick_size)))
}

// ──────────────────────────────────────────────────────────────────────
// Market profile / TPO — Sierra Chart class. Time spent at each level.
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct MarketProfileBody {
    brackets: Vec<market_profile::BracketRange>,
    tick_size: f64,
}

async fn market_profile_route(
    _u: AuthUser, Json(b): Json<MarketProfileBody>,
) -> Result<Json<market_profile::TpoReport>, ApiError> {
    if !(b.tick_size.is_finite() && b.tick_size > 0.0) {
        return Err(ApiError::BadRequest("tick_size must be a positive finite number".into()));
    }
    Ok(Json(market_profile::build(&b.brackets, b.tick_size)))
}

// ──────────────────────────────────────────────────────────────────────
// Portfolio stress test — price × IV × time grid for option books.
// tastytrade Risk Analysis class.
// ──────────────────────────────────────────────────────────────────────

async fn stress_test_route(
    _u: AuthUser, Json(input): Json<stress_test::StressInput>,
) -> Json<stress_test::StressReport> {
    Json(stress_test::analyze(&input))
}

// ──────────────────────────────────────────────────────────────────────
// Cohort tilt indicator — TopstepX class. Aggregate long/short bias
// across a group of traders' positions.
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct CohortTiltBody { positions: Vec<tilt_indicator::TraderPosition> }

async fn cohort_tilt_route(
    _u: AuthUser, Json(b): Json<CohortTiltBody>,
) -> Json<tilt_indicator::TiltReport> {
    Json(tilt_indicator::aggregate(&b.positions))
}

// ──────────────────────────────────────────────────────────────────────
// Bid/ask spread tracker + implementation shortfall (institutional TCA).
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct SpreadTrackerBody { samples: Vec<spread_tracker::QuoteSnapshot> }

async fn spread_tracker_route(
    _u: AuthUser, Json(b): Json<SpreadTrackerBody>,
) -> Json<spread_tracker::SpreadReport> {
    Json(spread_tracker::analyze(&b.samples))
}

async fn implementation_shortfall_route(
    _u: AuthUser, Json(input): Json<implementation_shortfall::ShortfallInput>,
) -> Json<implementation_shortfall::ShortfallReport> {
    Json(implementation_shortfall::analyze(&input))
}
