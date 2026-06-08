//! REST surface for momentum algo trading.
//!
//! CRUD for strategies + lifecycle endpoints (start/stop run, engage /
//! release kill switch). The actual engine tick pump (subscribe to live
//! tick bus, evaluate strategy, dispatch to broker) is NOT mounted here
//! — it's an in-process tokio task wired separately in `bin/server.rs`
//! when broker_mode != internal_sim.
//!
//! All write endpoints require the standard `AuthUser` extractor.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use serde::Deserialize;
use traderview_db::algo::{
    AlgoFill, AlgoOrder, AlgoRun, AlgoStrategy, AlgoStrategyInput, KillSwitchAudit,
};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/algo/strategies",
            get(list_strategies).post(create_strategy),
        )
        .route(
            "/algo/strategies/:id",
            put(update_strategy).delete(delete_strategy),
        )
        .route("/algo/strategies/:id/kill-switch", post(post_kill_switch))
        .route("/algo/strategies/:id/kill-history", get(get_kill_history))
        .route("/algo/strategies/:id/runs", get(list_runs).post(start_run))
        .route("/algo/strategies/:id/stop", post(stop_run))
        .route("/algo/runs/:id/orders", get(list_orders))
        .route("/algo/orders/:id/fills", get(list_fills))
        .route("/algo/strategies/:id/backtest", post(post_backtest))
        .route("/algo/strategies/:id/metrics", get(get_metrics))
        .route("/algo/strategies/:id/optimize", post(post_optimize))
        .route("/algo/strategies/:id/backtests", get(list_backtest_history))
        .route(
            "/algo/backtests/:id",
            axum::routing::delete(delete_backtest_row),
        )
}

// ─── strategy CRUD ──────────────────────────────────────────────────────────

async fn list_strategies(
    State(s): State<AppState>,
    u: AuthUser,
) -> Result<Json<Vec<AlgoStrategy>>, ApiError> {
    Ok(Json(
        traderview_db::algo::list_strategies(&s.pool, u.id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

const VALID_STRATEGY_TYPES: &[&str] = &[
    "momentum",
    "mean_reversion",
    "orb",
    "donchian_trend",
    "bb_squeeze",
    "ttm_squeeze",
    "vwap_scalp",
    "supertrend",
    "heikin_ashi_trend",
    "connors_rsi2",
    "order_block_sweep",
    "pead",
    "pairs",
];

async fn user_owns_account(
    pool: &sqlx::PgPool,
    user_id: Uuid,
    account_id: Uuid,
) -> Result<bool, ApiError> {
    let row: Option<(Uuid,)> =
        sqlx::query_as("SELECT id FROM accounts WHERE id = $1 AND user_id = $2")
            .bind(account_id)
            .bind(user_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| ApiError::Internal(e.into()))?;
    Ok(row.is_some())
}

/// Brokers with at least a scaffolded algo-trading adapter in this
/// codebase. Real WS+REST integration status (commit 33):
///   alpaca     — full (REST place_order/cancel + WS trade_updates pump)
///   tradier    — scaffolded; real client lands in commit 34
///   ibkr       — scaffolded; needs local TWS/Gateway, real client deferred
///   td         — scaffolded; Schwab API migration pending real client
///   tastytrade — scaffolded; real client deferred
///
/// The broker_dispatcher in `traderview_db::broker_dispatcher` returns
/// EngineError::Broker("integration_pending: <broker>") at submit time
/// for the scaffolded ones. UI + validation layers don't gate on impl
/// status — picking one of them creates a working strategy row that
/// just can't fire orders until the adapter is real.
const ALGO_SUPPORTED_BROKERS: &[&str] = &[
    "alpaca",
    "tradier",
    "ibkr",
    "td",
    "tdameritrade",
    "schwab",
    "tastytrade",
];

/// `internal_sim` is broker-agnostic; `paper` and `live` route via the
/// broker named in `account.broker`. Each broker resolves its own
/// paper-vs-live endpoint internally (e.g. paper-api.alpaca.markets vs
/// api.alpaca.markets; sandbox.tradier.com vs api.tradier.com).
fn broker_account_compatible(account_broker: &str, _broker_mode: &str) -> bool {
    let broker = account_broker.to_ascii_lowercase();
    ALGO_SUPPORTED_BROKERS.contains(&broker.as_str())
}

/// Verify the account's broker is in the algo-supported set AND
/// matches the requested broker_mode. Returns a friendly error
/// message; route layer wraps into a 400 BadRequest.
async fn validate_account_for_algo(
    pool: &sqlx::PgPool,
    account_id: Uuid,
    broker_mode: &str,
) -> Result<(), ApiError> {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT COALESCE(NULLIF(broker, ''), 'unknown') FROM accounts WHERE id = $1",
    )
    .bind(account_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| ApiError::Internal(e.into()))?;
    let Some((broker,)) = row else {
        return Err(ApiError::BadRequest("account not found".into()));
    };
    if !broker_account_compatible(&broker, broker_mode) {
        return Err(ApiError::BadRequest(format!(
            "account is on broker '{broker}', which is not in the algo-supported set. \
             Supported brokers: {}.",
            ALGO_SUPPORTED_BROKERS.join(", "),
        )));
    }
    Ok(())
}

async fn create_strategy(
    State(s): State<AppState>,
    u: AuthUser,
    Json(body): Json<AlgoStrategyInput>,
) -> Result<Json<AlgoStrategy>, ApiError> {
    if body.name.trim().is_empty() {
        return Err(ApiError::BadRequest("name required".into()));
    }
    if !VALID_STRATEGY_TYPES.contains(&body.strategy_type.as_str()) {
        return Err(ApiError::BadRequest(format!(
            "unknown strategy_type {}; expected one of: {}",
            body.strategy_type,
            VALID_STRATEGY_TYPES.join(", "),
        )));
    }
    // Every strategy must be bound to a real broker account so fills
    // flow into the standard executions → trades pipeline. Refuse the
    // save outright instead of letting the row sit inert and confuse
    // the user about why nothing happens.
    let account_id = body
        .account_id
        .ok_or_else(|| ApiError::BadRequest(
            "account_id required — pick a broker account before saving (Settings → Accounts to add one)".into(),
        ))?;
    if !user_owns_account(&s.pool, u.id, account_id).await? {
        return Err(ApiError::BadRequest(
            "account_id does not belong to you".into(),
        ));
    }
    validate_account_for_algo(&s.pool, account_id, &body.broker_mode).await?;
    // Engine code enforces paper_locked_until at submit-time, but the
    // route also rejects naked broker_mode='live' at create-time so the
    // UI doesn't show a misleading "saved" toast for a config the
    // engine will refuse to run.
    if body.broker_mode == "live" {
        return Err(ApiError::BadRequest(
            "new strategies must start in internal_sim or paper; switch to live after the 30-day paper-lock expires".into(),
        ));
    }
    let broker_mode = body.broker_mode.clone();
    let created = traderview_db::algo::create_strategy(&s.pool, u.id, body)
        .await
        .map_err(ApiError::Internal)?;
    maybe_hot_spawn_pump(&s, u.id, &broker_mode, account_id).await;
    Ok(Json(created))
}

/// If the strategy's broker_mode is paper/live AND its account is on
/// Alpaca, ensure a trade_updates pump exists for (user, paper-or-live).
/// Idempotent — no-op if one is already running. Errors don't fail the
/// route; the pump just won't push WS events until restart. Per-broker
/// pump spawn (Tradier / IBKR / etc.) lands as each adapter ships.
async fn maybe_hot_spawn_pump(
    state: &AppState,
    user_id: Uuid,
    broker_mode: &str,
    account_id: Uuid,
) {
    let paper = match broker_mode {
        "paper" => true,
        "live" => false,
        _ => return,
    };
    // Only Alpaca accounts get the Alpaca pump.
    let broker: Option<(Option<String>,)> =
        sqlx::query_as("SELECT broker FROM accounts WHERE id = $1")
            .bind(account_id)
            .fetch_optional(&state.pool)
            .await
            .ok()
            .flatten();
    let is_alpaca = matches!(
        broker,
        Some((Some(b),)) if b.eq_ignore_ascii_case("alpaca")
    );
    if !is_alpaca {
        return;
    }
    let sink = state.build_engine_event_sink();
    let spawned = traderview_db::alpaca_pump::ensure_pump_for(
        state.alpaca_pumps.clone(),
        state.pool.clone(),
        user_id,
        paper,
        Some(sink),
    )
    .await;
    if spawned {
        tracing::info!(user_id = %user_id, paper, "alpaca pump hot-spawned");
    }
}

async fn update_strategy(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<AlgoStrategyInput>,
) -> Result<Json<AlgoStrategy>, ApiError> {
    if !VALID_STRATEGY_TYPES.contains(&body.strategy_type.as_str()) {
        return Err(ApiError::BadRequest(format!(
            "unknown strategy_type {}; expected one of: {}",
            body.strategy_type,
            VALID_STRATEGY_TYPES.join(", "),
        )));
    }
    let account_id = body
        .account_id
        .ok_or_else(|| ApiError::BadRequest("account_id required".into()))?;
    if !user_owns_account(&s.pool, u.id, account_id).await? {
        return Err(ApiError::BadRequest(
            "account_id does not belong to you".into(),
        ));
    }
    validate_account_for_algo(&s.pool, account_id, &body.broker_mode).await?;
    // Allow broker_mode='live' on update only if existing paper_locked_until
    // has expired. Engine still re-checks at submit.
    if body.broker_mode == "live" {
        let existing = traderview_db::algo::get_strategy(&s.pool, u.id, id)
            .await
            .map_err(ApiError::Internal)?
            .ok_or(ApiError::NotFound)?;
        if chrono::Utc::now() <= existing.paper_locked_until {
            return Err(ApiError::BadRequest(format!(
                "strategy is paper-locked until {}; cannot promote to live yet",
                existing.paper_locked_until
            )));
        }
    }
    let broker_mode = body.broker_mode.clone();
    let updated = traderview_db::algo::update_strategy(&s.pool, u.id, id, body)
        .await
        .map_err(ApiError::Internal)?
        .ok_or(ApiError::NotFound)?;
    maybe_hot_spawn_pump(&s, u.id, &broker_mode, updated.account_id).await;
    Ok(Json(updated))
}

async fn delete_strategy(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let ok = traderview_db::algo::delete_strategy(&s.pool, u.id, id)
        .await
        .map_err(ApiError::Internal)?;
    if !ok {
        return Err(ApiError::NotFound);
    }
    Ok(Json(serde_json::json!({ "deleted": true })))
}

// ─── kill switch ────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct KillSwitchBody {
    engaged: bool,
    #[serde(default)]
    reason: Option<String>,
}

async fn post_kill_switch(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<KillSwitchBody>,
) -> Result<Json<AlgoStrategy>, ApiError> {
    traderview_db::algo::set_kill_switch(&s.pool, u.id, id, body.engaged, body.reason)
        .await
        .map_err(ApiError::Internal)?
        .map(Json)
        .ok_or(ApiError::NotFound)
}

async fn get_kill_history(
    State(s): State<AppState>,
    _u: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<KillSwitchAudit>>, ApiError> {
    Ok(Json(
        traderview_db::algo::kill_switch_history(&s.pool, id, 100)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

// ─── run lifecycle ──────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct StopRunBody {
    #[serde(default = "default_stop_reason")]
    reason: String,
}

fn default_stop_reason() -> String {
    "user".into()
}

async fn start_run(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<AlgoRun>, ApiError> {
    let strategy = traderview_db::algo::get_strategy(&s.pool, u.id, id)
        .await
        .map_err(ApiError::Internal)?
        .ok_or(ApiError::NotFound)?;
    if strategy.kill_switch {
        return Err(ApiError::BadRequest(
            "kill switch engaged — release it before starting a run".into(),
        ));
    }
    if !strategy.enabled {
        return Err(ApiError::BadRequest("strategy is disabled".into()));
    }
    // account_id is NOT NULL since migration 0056 — strategy must be
    // bound. No defensive check needed; the row could not have been
    // saved otherwise.
    if let Some(open) = traderview_db::algo::get_open_run(&s.pool, id)
        .await
        .map_err(ApiError::Internal)?
    {
        // Idempotent: re-starting a strategy with an open run returns
        // that run instead of erroring — UI re-mounting shouldn't break.
        return Ok(Json(open));
    }
    Ok(Json(
        traderview_db::algo::start_run(&s.pool, id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

async fn stop_run(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<StopRunBody>,
) -> Result<Json<AlgoRun>, ApiError> {
    // Ownership check — get_strategy returns None for cross-user access.
    traderview_db::algo::get_strategy(&s.pool, u.id, id)
        .await
        .map_err(ApiError::Internal)?
        .ok_or(ApiError::NotFound)?;
    let open = traderview_db::algo::get_open_run(&s.pool, id)
        .await
        .map_err(ApiError::Internal)?
        .ok_or_else(|| ApiError::BadRequest("no open run for this strategy".into()))?;
    traderview_db::algo::stop_run(&s.pool, open.id, &body.reason, None, None)
        .await
        .map_err(ApiError::Internal)?
        .map(Json)
        .ok_or(ApiError::NotFound)
}

#[derive(Debug, Deserialize)]
struct ListRunsQuery {
    #[serde(default = "default_limit")]
    limit: i64,
}

fn default_limit() -> i64 {
    25
}

async fn list_runs(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
    Query(q): Query<ListRunsQuery>,
) -> Result<Json<Vec<AlgoRun>>, ApiError> {
    // Ownership check.
    traderview_db::algo::get_strategy(&s.pool, u.id, id)
        .await
        .map_err(ApiError::Internal)?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(
        traderview_db::algo::list_runs(&s.pool, id, q.limit.clamp(1, 200))
            .await
            .map_err(ApiError::Internal)?,
    ))
}

async fn list_orders(
    State(s): State<AppState>,
    _u: AuthUser,
    Path(id): Path<Uuid>,
    Query(q): Query<ListRunsQuery>,
) -> Result<Json<Vec<AlgoOrder>>, ApiError> {
    Ok(Json(
        traderview_db::algo::list_orders(&s.pool, id, q.limit.clamp(1, 500))
            .await
            .map_err(ApiError::Internal)?,
    ))
}

async fn list_fills(
    State(s): State<AppState>,
    _u: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<AlgoFill>>, ApiError> {
    Ok(Json(
        traderview_db::algo::list_fills(&s.pool, id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

#[derive(Debug, Deserialize)]
struct BacktestBody {
    /// Symbol to replay. Defaults to "SPY" so a quick `Backtest` button
    /// without input still produces useful output for a watchlist-based
    /// strategy whose entry_rules don't pin a symbol.
    #[serde(default = "default_bt_symbol")]
    symbol: String,
    /// Bar interval — same enum used by the prices repo.
    /// Accepted: "min1" | "min5" | "min15" | "min30" | "hour1" | "day1".
    #[serde(default = "default_bt_interval")]
    interval: String,
    /// Days back from `now()` to backtest. Wins over (from, to) when both set.
    #[serde(default = "default_bt_days_back")]
    days_back: i64,
    #[serde(default)]
    from: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default)]
    to: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default)]
    initial_equity: Option<f64>,
    #[serde(default)]
    fee_per_trade: Option<f64>,
    #[serde(default)]
    slippage_bps: Option<f64>,
}

fn default_bt_symbol() -> String {
    "SPY".into()
}
fn default_bt_interval() -> String {
    "5m".into()
}
fn default_bt_days_back() -> i64 {
    60
}

/// Accept the algo-strategy timeframe labels ("min1" / "min5") AND the
/// BarInterval serde renames ("1m" / "5m") so either the strategy's own
/// timeframe field OR a user-typed interval works.
fn normalize_interval(s: &str) -> String {
    match s {
        "min1" => "1m".into(),
        "min5" => "5m".into(),
        "min15" => "15m".into(),
        "min30" => "15m".into(), // closest cached interval
        "hour1" => "1h".into(),
        "day1" => "1d".into(),
        other => other.to_string(),
    }
}

/// POST /algo/strategies/:id/backtest — pulls historical bars from the
/// prices cache (Yahoo fetch on miss) and replays the strategy through
/// `traderview_core::algo_backtest::run`. Returns trades, equity curve,
/// and summary stats as JSON.
async fn post_backtest(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<BacktestBody>,
) -> Result<Json<traderview_core::algo_backtest::AlgoBtResult>, ApiError> {
    let strategy = traderview_db::algo::get_strategy(&s.pool, u.id, id)
        .await
        .map_err(ApiError::Internal)?
        .ok_or_else(|| ApiError::BadRequest("strategy not found".into()))?;
    // BarInterval has serde rename labels (10s, 1m, 5m, 15m, 1h, 1d, 1w)
    // but no FromStr; deserialize through a JSON value so callers can use
    // either the rename label or the variant name ("M5", "min5", etc.).
    let interval: traderview_core::BarInterval = serde_json::from_value(serde_json::Value::String(
        normalize_interval(&body.interval),
    ))
    .map_err(|e| ApiError::BadRequest(format!("interval: {e}")))?;
    let to = body.to.unwrap_or_else(chrono::Utc::now);
    let from = body
        .from
        .unwrap_or_else(|| to - chrono::Duration::days(body.days_back));
    let bars = traderview_db::prices::get_bars(&s.pool, &body.symbol, interval, from, to)
        .await
        .map_err(ApiError::Internal)?;
    if bars.len() < 30 {
        return Err(ApiError::BadRequest(format!(
            "only {} bars available for {} {:?} between {from} and {to} — need >=30",
            bars.len(),
            body.symbol,
            body.interval
        )));
    }
    let strat =
        traderview_core::algo_strategies::from_kind(&strategy.strategy_type, &strategy.entry_rules)
            .map_err(|e| ApiError::BadRequest(format!("strategy_type: {e}")))?;
    let sizing: traderview_core::algo_strategies::Sizing =
        serde_json::from_value(strategy.sizing.clone()).unwrap_or_default();
    let side_mode = match strategy.side_mode.as_str() {
        "short" => traderview_core::algo_strategies::SideMode::Short,
        "both" => traderview_core::algo_strategies::SideMode::Both,
        _ => traderview_core::algo_strategies::SideMode::Long,
    };
    let cfg = traderview_core::algo_backtest::BacktestConfig {
        initial_equity: body.initial_equity.unwrap_or(100_000.0),
        fee_per_trade: body.fee_per_trade.unwrap_or(1.0),
        slippage_bps: body.slippage_bps.unwrap_or(5.0),
        side_mode,
    };
    let result = traderview_core::algo_backtest::run(&bars, strat.as_ref(), &sizing, cfg);

    // Persist the summary so the history modal can show drift across
    // re-runs without re-fetching bars. Failure to persist isn't fatal
    // — the run still returns to the caller; we just log.
    use rust_decimal::Decimal;
    use std::str::FromStr;
    let dec = |v: f64| Decimal::from_str(&format!("{:.8}", v)).unwrap_or_default();
    let pf = if result.summary.profit_factor.is_finite() {
        result.summary.profit_factor
    } else {
        0.0
    };
    let insert = traderview_db::algo::AlgoBacktestInsert {
        strategy_id: strategy.id,
        user_id: u.id,
        symbol: body.symbol.clone(),
        interval: body.interval.clone(),
        range_from: from,
        range_to: to,
        initial_equity: dec(cfg.initial_equity),
        fee_per_trade: dec(cfg.fee_per_trade),
        slippage_bps: dec(cfg.slippage_bps),
        entry_rules: strategy.entry_rules.clone(),
        trades: result.summary.trades as i64,
        wins: result.summary.wins as i64,
        losses: result.summary.losses as i64,
        win_rate: dec(result.summary.win_rate),
        avg_r: dec(result.summary.avg_r),
        profit_factor: dec(pf),
        total_return_pct: dec(result.summary.total_return_pct),
        max_drawdown_pct: dec(result.summary.max_drawdown_pct),
        final_equity: dec(result.summary.final_equity),
        sharpe: dec(result.summary.sharpe),
    };
    if let Err(e) = traderview_db::algo::save_backtest(&s.pool, insert).await {
        tracing::warn!(error = %e, "backtest history persist failed");
    }
    Ok(Json(result))
}

async fn list_backtest_history(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
    Query(q): Query<HistoryQuery>,
) -> Result<Json<Vec<traderview_db::algo::AlgoBacktestRow>>, ApiError> {
    Ok(Json(
        traderview_db::algo::list_backtests(&s.pool, u.id, id, q.limit.unwrap_or(50))
            .await
            .map_err(ApiError::Internal)?,
    ))
}

async fn delete_backtest_row(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let n = traderview_db::algo::delete_backtest(&s.pool, u.id, id)
        .await
        .map_err(ApiError::Internal)?;
    Ok(Json(serde_json::json!({ "deleted": n })))
}

#[derive(Debug, Deserialize, Default)]
struct HistoryQuery {
    #[serde(default)]
    limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct OptimizeBody {
    /// Backtest bar source — same shape as BacktestBody.
    #[serde(default = "default_bt_symbol")]
    symbol: String,
    #[serde(default = "default_bt_interval")]
    interval: String,
    #[serde(default = "default_bt_days_back")]
    days_back: i64,
    #[serde(default)]
    from: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default)]
    to: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default)]
    initial_equity: Option<f64>,
    #[serde(default)]
    fee_per_trade: Option<f64>,
    #[serde(default)]
    slippage_bps: Option<f64>,
    /// Parameter grid — { key: [v1, v2, ...] }. Each key whose value
    /// is an array becomes a sweep dimension; scalar values pass
    /// through as a single candidate.
    grid: serde_json::Map<String, serde_json::Value>,
    #[serde(default = "default_metric")]
    metric: traderview_core::algo_optimize::OptimizeMetric,
    #[serde(default = "default_top_n")]
    top_n: usize,
}

fn default_metric() -> traderview_core::algo_optimize::OptimizeMetric {
    traderview_core::algo_optimize::OptimizeMetric::Sharpe
}

fn default_top_n() -> usize {
    10
}

/// POST /algo/strategies/:id/optimize — runs the parameter optimizer
/// against the strategy's saved kind + sizing + side_mode. Returns
/// top-N entry_rules configs ranked by the chosen metric.
async fn post_optimize(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<OptimizeBody>,
) -> Result<Json<traderview_core::algo_optimize::OptimizeResult>, ApiError> {
    let strategy = traderview_db::algo::get_strategy(&s.pool, u.id, id)
        .await
        .map_err(ApiError::Internal)?
        .ok_or_else(|| ApiError::BadRequest("strategy not found".into()))?;
    let interval: traderview_core::BarInterval = serde_json::from_value(serde_json::Value::String(
        normalize_interval(&body.interval),
    ))
    .map_err(|e| ApiError::BadRequest(format!("interval: {e}")))?;
    let to = body.to.unwrap_or_else(chrono::Utc::now);
    let from = body
        .from
        .unwrap_or_else(|| to - chrono::Duration::days(body.days_back));
    let bars = traderview_db::prices::get_bars(&s.pool, &body.symbol, interval, from, to)
        .await
        .map_err(ApiError::Internal)?;
    if bars.len() < 30 {
        return Err(ApiError::BadRequest(format!(
            "only {} bars available for {} {:?} between {from} and {to} — need >=30",
            bars.len(),
            body.symbol,
            body.interval
        )));
    }
    let sizing: traderview_core::algo_strategies::Sizing =
        serde_json::from_value(strategy.sizing.clone()).unwrap_or_default();
    let side_mode = match strategy.side_mode.as_str() {
        "short" => traderview_core::algo_strategies::SideMode::Short,
        "both" => traderview_core::algo_strategies::SideMode::Both,
        _ => traderview_core::algo_strategies::SideMode::Long,
    };
    let cfg = traderview_core::algo_backtest::BacktestConfig {
        initial_equity: body.initial_equity.unwrap_or(100_000.0),
        fee_per_trade: body.fee_per_trade.unwrap_or(1.0),
        slippage_bps: body.slippage_bps.unwrap_or(5.0),
        side_mode,
    };
    let result = traderview_core::algo_optimize::run(
        &bars,
        &strategy.strategy_type,
        &strategy.entry_rules,
        &body.grid,
        &sizing,
        cfg,
        body.metric,
        body.top_n,
    )
    .map_err(|e| ApiError::BadRequest(format!("optimize: {e}")))?;
    Ok(Json(result))
}

/// GET /algo/strategies/:id/metrics — dashboard rollup.
async fn get_metrics(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<traderview_db::algo::StrategyMetrics>, ApiError> {
    traderview_db::algo::strategy_metrics(&s.pool, u.id, id)
        .await
        .map_err(ApiError::Internal)?
        .map(Json)
        .ok_or_else(|| ApiError::BadRequest("strategy not found".into()))
}
