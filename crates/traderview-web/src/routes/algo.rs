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
        .route("/algo/tournament", post(post_tournament))
        .route("/algo/portfolio", post(post_portfolio))
        .route("/algo/tournament-matrix", post(post_tournament_matrix))
        .route("/algo/strategies/:id/walk-forward", post(post_walk_forward))
        .route("/algo/strategies/:id/backtest-mc", post(post_backtest_mc))
        .route("/algo/strategies/:id/backtest-regimes", post(post_backtest_regimes))
        .route("/algo/strategies/:id/live-vs-backtest", get(get_live_vs_backtest))
        .route("/algo/strategies/:id/gate-fires", get(get_gate_fires))
        .route("/algo/strategies/:id/revisions", get(get_revisions))
        .route(
            "/algo/strategies/:id/revisions/:rev_id/restore",
            post(post_restore_revision),
        )
        .route("/algo/strategies/:id/fork", post(post_fork_strategy))
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

/// Valid strategy types are DERIVED from the registry — a hardcoded
/// list here silently rejected every strategy added after it was
/// typed (8 kinds were unsaveable while the factory, backtester, and
/// UI dropdown all supported them).
fn valid_strategy_types() -> Vec<&'static str> {
    traderview_core::algo_strategies::StrategyKind::all()
        .iter()
        .map(|k| k.as_str())
        .collect()
}

fn validate_strategy_type(strategy_type: &str) -> Result<(), ApiError> {
    let valid = valid_strategy_types();
    if !valid.contains(&strategy_type) {
        return Err(ApiError::BadRequest(format!(
            "unknown strategy_type {strategy_type}; expected one of: {}",
            valid.join(", "),
        )));
    }
    Ok(())
}

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
    validate_strategy_type(&body.strategy_type)?;
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
    validate_universe(&body).await?;
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

/// Reject configurations where `universe_mode='watchlist'` is set but
/// no `watchlist_id` is picked — the silent failure mode where the
/// strategy sits enabled, runs every tick against an empty universe,
/// and produces zero observable behaviour.
async fn validate_universe(body: &traderview_db::algo::AlgoStrategyInput) -> Result<(), ApiError> {
    if body.universe_mode == "watchlist" && body.watchlist_id.is_none() {
        return Err(ApiError::BadRequest(
            "universe_mode='watchlist' requires a watchlist_id — \
             pick a watchlist or switch to autoscan"
                .into(),
        ));
    }
    Ok(())
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
    validate_strategy_type(&body.strategy_type)?;
    let account_id = body
        .account_id
        .ok_or_else(|| ApiError::BadRequest("account_id required".into()))?;
    if !user_owns_account(&s.pool, u.id, account_id).await? {
        return Err(ApiError::BadRequest(
            "account_id does not belong to you".into(),
        ));
    }
    validate_account_for_algo(&s.pool, account_id, &body.broker_mode).await?;
    validate_universe(&body).await?;
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
    u: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<KillSwitchAudit>>, ApiError> {
    // Ownership check — without this any authed user can read another
    // user's kill-switch audit trail (reasons, timestamps), which is
    // forensic data we treat as private.
    traderview_db::algo::get_strategy(&s.pool, u.id, id)
        .await
        .map_err(ApiError::Internal)?
        .ok_or(ApiError::NotFound)?;
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
    u: AuthUser,
    Path(id): Path<Uuid>,
    Query(q): Query<ListRunsQuery>,
) -> Result<Json<Vec<AlgoOrder>>, ApiError> {
    // Ownership check on the run — without this any authed user can
    // enumerate run UUIDs and read another trader's complete order tape.
    let owner = traderview_db::algo::run_owner(&s.pool, id)
        .await
        .map_err(ApiError::Internal)?
        .ok_or(ApiError::NotFound)?;
    if owner != u.id {
        return Err(ApiError::NotFound);
    }
    Ok(Json(
        traderview_db::algo::list_orders(&s.pool, id, q.limit.clamp(1, 500))
            .await
            .map_err(ApiError::Internal)?,
    ))
}

async fn list_fills(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<AlgoFill>>, ApiError> {
    // Ownership check on the order — same bypass as list_orders.
    let owner = traderview_db::algo::order_owner(&s.pool, id)
        .await
        .map_err(ApiError::Internal)?
        .ok_or(ApiError::NotFound)?;
    if owner != u.id {
        return Err(ApiError::NotFound);
    }
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
    /// Replay the strategy's time-replayable risk gates (entry window,
    /// daily entry cap, loss cooldown) inside the backtest, so the
    /// simulated system is the one that will actually run live.
    #[serde(default)]
    apply_gates: bool,
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
    let gates = if body.apply_gates {
        traderview_core::algo_backtest::BtGates {
            entry_window: strategy
                .risk_gates
                .get("entry_window")
                .and_then(|v| v.as_str())
                .and_then(traderview_db::algo_engine::parse_entry_window),
            max_entries_per_day: strategy
                .risk_gates
                .get("max_entries_per_day")
                .and_then(|v| v.as_u64())
                .filter(|n| *n > 0)
                .map(|n| n as usize),
            loss_cooldown_minutes: strategy
                .risk_gates
                .get("loss_cooldown_minutes")
                .and_then(|v| v.as_i64())
                .filter(|n| *n > 0),
        }
    } else {
        traderview_core::algo_backtest::BtGates::default()
    };
    let result =
        traderview_core::algo_backtest::run_with_gates(&bars, strat.as_ref(), &sizing, cfg, gates);

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

#[derive(serde::Deserialize)]
struct TournamentBody {
    #[serde(default = "default_bt_symbol")]
    symbol: String,
    #[serde(default = "default_bt_interval")]
    interval: String,
    #[serde(default = "default_bt_days_back")]
    days_back: i64,
    #[serde(default)]
    initial_equity: Option<f64>,
    #[serde(default)]
    fee_per_trade: Option<f64>,
    #[serde(default)]
    slippage_bps: Option<f64>,
    /// "long" | "short" | "both" (default both — fair to every family).
    #[serde(default)]
    side_mode: Option<String>,
    #[serde(default)]
    rank_by: traderview_core::algo_tournament::RankMetric,
}

/// POST /algo/tournament — run EVERY single-symbol registry strategy
/// with default rules over the same bars and rank them. Multi-symbol
/// strategies are reported as skipped, never silently dropped.
async fn post_tournament(
    State(s): State<AppState>,
    _u: AuthUser,
    Json(body): Json<TournamentBody>,
) -> Result<Json<traderview_core::algo_tournament::TournamentResult>, ApiError> {
    let interval: traderview_core::BarInterval = serde_json::from_value(serde_json::Value::String(
        normalize_interval(&body.interval),
    ))
    .map_err(|e| ApiError::BadRequest(format!("interval: {e}")))?;
    let to = chrono::Utc::now();
    let from = to - chrono::Duration::days(body.days_back);
    let symbol = body.symbol.trim().to_uppercase();
    let bars = traderview_db::prices::get_bars(&s.pool, &symbol, interval, from, to)
        .await
        .map_err(ApiError::Internal)?;
    if bars.len() < 30 {
        return Err(ApiError::BadRequest(format!(
            "only {} bars available for {symbol} — need >=30",
            bars.len()
        )));
    }
    let side_mode = match body.side_mode.as_deref() {
        Some("long") => traderview_core::algo_strategies::SideMode::Long,
        Some("short") => traderview_core::algo_strategies::SideMode::Short,
        _ => traderview_core::algo_strategies::SideMode::Both,
    };
    let cfg = traderview_core::algo_backtest::BacktestConfig {
        initial_equity: body.initial_equity.unwrap_or(100_000.0),
        fee_per_trade: body.fee_per_trade.unwrap_or(1.0),
        slippage_bps: body.slippage_bps.unwrap_or(5.0),
        side_mode,
    };
    Ok(Json(traderview_core::algo_tournament::tournament(
        &bars,
        &traderview_core::algo_strategies::Sizing::default(),
        &cfg,
        body.rank_by,
    )))
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

#[derive(serde::Deserialize)]
struct TournamentMatrixBody {
    /// 2..=20 symbols — the column axis.
    symbols: Vec<String>,
    #[serde(default = "default_bt_interval")]
    interval: String,
    #[serde(default = "default_bt_days_back")]
    days_back: i64,
    #[serde(default)]
    initial_equity: Option<f64>,
    #[serde(default)]
    fee_per_trade: Option<f64>,
    #[serde(default)]
    slippage_bps: Option<f64>,
    #[serde(default)]
    rank_by: traderview_core::algo_tournament::RankMetric,
}

#[derive(serde::Serialize)]
struct TournamentMatrixResult {
    matrix: traderview_core::algo_tournament::MatrixResult,
    /// Symbols with too few bars — reported, never silently dropped.
    skipped_symbols: Vec<String>,
}

/// POST /algo/tournament-matrix — strategies × symbols: every
/// single-symbol registry kind backtested on every listed symbol,
/// with the per-column best and a buy-and-hold baseline per symbol.
async fn post_tournament_matrix(
    State(s): State<AppState>,
    _u: AuthUser,
    Json(body): Json<TournamentMatrixBody>,
) -> Result<Json<TournamentMatrixResult>, ApiError> {
    if body.symbols.len() < 2 || body.symbols.len() > 20 {
        return Err(ApiError::BadRequest("symbols must list 2..=20 entries".into()));
    }
    let interval: traderview_core::BarInterval = serde_json::from_value(serde_json::Value::String(
        normalize_interval(&body.interval),
    ))
    .map_err(|e| ApiError::BadRequest(format!("interval: {e}")))?;
    let to = chrono::Utc::now();
    let from = to - chrono::Duration::days(body.days_back);
    let mut per_symbol = Vec::new();
    let mut skipped_symbols = Vec::new();
    for raw in &body.symbols {
        let symbol = raw.trim().to_uppercase();
        if symbol.is_empty() {
            continue;
        }
        let bars = traderview_db::prices::get_bars(&s.pool, &symbol, interval, from, to)
            .await
            .map_err(ApiError::Internal)?;
        if bars.len() < 30 {
            skipped_symbols.push(symbol);
        } else {
            per_symbol.push((symbol, bars));
        }
    }
    if per_symbol.is_empty() {
        return Err(ApiError::BadRequest(
            "no symbol had enough bars (need >=30 each)".into(),
        ));
    }
    let cfg = traderview_core::algo_backtest::BacktestConfig {
        initial_equity: body.initial_equity.unwrap_or(100_000.0),
        fee_per_trade: body.fee_per_trade.unwrap_or(1.0),
        slippage_bps: body.slippage_bps.unwrap_or(5.0),
        side_mode: traderview_core::algo_strategies::SideMode::Both,
    };
    let matrix = traderview_core::algo_tournament::matrix(
        &per_symbol,
        &traderview_core::algo_strategies::Sizing::default(),
        &cfg,
        body.rank_by,
    );
    Ok(Json(TournamentMatrixResult {
        matrix,
        skipped_symbols,
    }))
}

#[derive(serde::Deserialize)]
struct PortfolioBody {
    #[serde(default = "default_bt_symbol")]
    symbol: String,
    #[serde(default = "default_bt_interval")]
    interval: String,
    #[serde(default = "default_bt_days_back")]
    days_back: i64,
    #[serde(default)]
    initial_equity: Option<f64>,
    #[serde(default)]
    fee_per_trade: Option<f64>,
    #[serde(default)]
    slippage_bps: Option<f64>,
    /// Strategy kinds to combine (2..=21, default rules each).
    kinds: Vec<String>,
}

/// POST /algo/portfolio — run the chosen strategy kinds over the same
/// bars, return the pairwise return-correlation matrix, per-leg stats,
/// the equal-weight combined curve, and the diversification benefit
/// (combined Sharpe minus average individual Sharpe).
async fn post_portfolio(
    State(s): State<AppState>,
    _u: AuthUser,
    Json(body): Json<PortfolioBody>,
) -> Result<Json<traderview_core::algo_strategy_portfolio::PortfolioResult>, ApiError> {
    if body.kinds.len() < 2 || body.kinds.len() > 21 {
        return Err(ApiError::BadRequest("kinds must list 2..=21 strategies".into()));
    }
    let interval: traderview_core::BarInterval = serde_json::from_value(serde_json::Value::String(
        normalize_interval(&body.interval),
    ))
    .map_err(|e| ApiError::BadRequest(format!("interval: {e}")))?;
    let to = chrono::Utc::now();
    let from = to - chrono::Duration::days(body.days_back);
    let symbol = body.symbol.trim().to_uppercase();
    let bars = traderview_db::prices::get_bars(&s.pool, &symbol, interval, from, to)
        .await
        .map_err(ApiError::Internal)?;
    if bars.len() < 30 {
        return Err(ApiError::BadRequest(format!(
            "only {} bars available for {symbol} — need >=30",
            bars.len()
        )));
    }
    let cfg = traderview_core::algo_backtest::BacktestConfig {
        initial_equity: body.initial_equity.unwrap_or(100_000.0),
        fee_per_trade: body.fee_per_trade.unwrap_or(1.0),
        slippage_bps: body.slippage_bps.unwrap_or(5.0),
        side_mode: traderview_core::algo_strategies::SideMode::Both,
    };
    traderview_core::algo_strategy_portfolio::analyze(
        &bars,
        &body.kinds,
        &traderview_core::algo_strategies::Sizing::default(),
        &cfg,
    )
    .map(Json)
    .map_err(ApiError::BadRequest)
}

#[derive(serde::Deserialize)]
struct WalkForwardBody {
    #[serde(default = "default_bt_symbol")]
    symbol: String,
    #[serde(default = "default_bt_interval")]
    interval: String,
    #[serde(default = "default_bt_days_back")]
    days_back: i64,
    #[serde(default)]
    initial_equity: Option<f64>,
    #[serde(default)]
    fee_per_trade: Option<f64>,
    #[serde(default)]
    slippage_bps: Option<f64>,
    grid: serde_json::Map<String, serde_json::Value>,
    #[serde(default = "default_metric")]
    metric: traderview_core::algo_optimize::OptimizeMetric,
    #[serde(default = "default_wf_is_bars")]
    is_bars: usize,
    #[serde(default = "default_wf_oos_bars")]
    oos_bars: usize,
    #[serde(default)]
    step_bars: Option<usize>,
}

fn default_wf_is_bars() -> usize {
    120
}
fn default_wf_oos_bars() -> usize {
    60
}

/// POST /algo/strategies/:id/walk-forward — rolling optimize-then-
/// validate against the strategy's saved kind/rules/sizing/side_mode.
/// The wf_efficiency in the result is the overfit detector the plain
/// optimizer can't provide.
async fn post_walk_forward(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<WalkForwardBody>,
) -> Result<Json<traderview_core::algo_walk_forward::AlgoWfResult>, ApiError> {
    let strategy = traderview_db::algo::get_strategy(&s.pool, u.id, id)
        .await
        .map_err(ApiError::Internal)?
        .ok_or_else(|| ApiError::BadRequest("strategy not found".into()))?;
    let interval: traderview_core::BarInterval = serde_json::from_value(serde_json::Value::String(
        normalize_interval(&body.interval),
    ))
    .map_err(|e| ApiError::BadRequest(format!("interval: {e}")))?;
    let to = chrono::Utc::now();
    let from = to - chrono::Duration::days(body.days_back);
    let bars = traderview_db::prices::get_bars(&s.pool, &body.symbol, interval, from, to)
        .await
        .map_err(ApiError::Internal)?;
    if bars.len() < body.is_bars + body.oos_bars {
        return Err(ApiError::BadRequest(format!(
            "only {} bars for {} — need at least is_bars + oos_bars = {}",
            bars.len(),
            body.symbol,
            body.is_bars + body.oos_bars
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
    traderview_core::algo_walk_forward::run(
        &bars,
        &strategy.strategy_type,
        &strategy.entry_rules,
        &body.grid,
        &sizing,
        cfg,
        body.metric,
        body.is_bars,
        body.oos_bars,
        body.step_bars,
    )
    .map(Json)
    .map_err(|e| ApiError::BadRequest(format!("walk-forward: {e}")))
}

#[derive(serde::Deserialize)]
struct BacktestMcBody {
    #[serde(default = "default_bt_symbol")]
    symbol: String,
    #[serde(default = "default_bt_interval")]
    interval: String,
    #[serde(default = "default_bt_days_back")]
    days_back: i64,
    #[serde(default)]
    initial_equity: Option<f64>,
    #[serde(default)]
    fee_per_trade: Option<f64>,
    #[serde(default)]
    slippage_bps: Option<f64>,
    #[serde(default = "default_mc_curves")]
    n_curves: usize,
    #[serde(default)]
    trades_per_curve: Option<usize>,
    #[serde(default = "default_ruin_fraction")]
    ruin_fraction: f64,
    /// Fixed seed reproduces the distribution exactly; omitted = time-based.
    #[serde(default)]
    seed: Option<u64>,
}

fn default_mc_curves() -> usize {
    1000
}
fn default_ruin_fraction() -> f64 {
    0.5
}

#[derive(serde::Serialize)]
struct BacktestMcResult {
    backtest_summary: traderview_core::algo_backtest::AlgoBtSummary,
    trades_used: usize,
    mc: traderview_core::monte_carlo::McReport,
}

/// POST /algo/strategies/:id/backtest-mc — run the backtest, then
/// resample its ACTUAL trade PnLs into thousands of orderings. The
/// single backtest's max drawdown is one draw from a distribution;
/// this returns the distribution.
async fn post_backtest_mc(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<BacktestMcBody>,
) -> Result<Json<BacktestMcResult>, ApiError> {
    let strategy = traderview_db::algo::get_strategy(&s.pool, u.id, id)
        .await
        .map_err(ApiError::Internal)?
        .ok_or_else(|| ApiError::BadRequest("strategy not found".into()))?;
    let interval: traderview_core::BarInterval = serde_json::from_value(serde_json::Value::String(
        normalize_interval(&body.interval),
    ))
    .map_err(|e| ApiError::BadRequest(format!("interval: {e}")))?;
    let to = chrono::Utc::now();
    let from = to - chrono::Duration::days(body.days_back);
    let bars = traderview_db::prices::get_bars(&s.pool, &body.symbol, interval, from, to)
        .await
        .map_err(ApiError::Internal)?;
    if bars.len() < 30 {
        return Err(ApiError::BadRequest(format!(
            "only {} bars available for {} — need >=30",
            bars.len(),
            body.symbol
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
    let initial_equity = body.initial_equity.unwrap_or(100_000.0);
    let cfg = traderview_core::algo_backtest::BacktestConfig {
        initial_equity,
        fee_per_trade: body.fee_per_trade.unwrap_or(1.0),
        slippage_bps: body.slippage_bps.unwrap_or(5.0),
        side_mode,
    };
    let bt = traderview_core::algo_backtest::run(&bars, strat.as_ref(), &sizing, cfg);
    let pnls: Vec<f64> = bt.trades.iter().map(|t| t.pnl).collect();
    if pnls.len() < 10 {
        return Err(ApiError::BadRequest(format!(
            "{} trades is too few for a meaningful resequencing distribution — need >=10",
            pnls.len()
        )));
    }
    let params = traderview_core::algo_backtest_mc::McBridgeParams {
        n_curves: body.n_curves.min(20_000),
        trades_per_curve: body.trades_per_curve,
        ruin_fraction: body.ruin_fraction,
        seed: body.seed.unwrap_or_else(|| chrono::Utc::now().timestamp() as u64),
    };
    let mc = traderview_core::algo_backtest_mc::from_pnls(&pnls, initial_equity, &params)
        .ok_or_else(|| ApiError::BadRequest("degenerate MC parameters".into()))?;
    Ok(Json(BacktestMcResult {
        backtest_summary: bt.summary,
        trades_used: pnls.len(),
        mc,
    }))
}

#[derive(serde::Deserialize)]
struct BacktestRegimesBody {
    #[serde(default = "default_bt_symbol")]
    symbol: String,
    #[serde(default = "default_bt_interval")]
    interval: String,
    #[serde(default = "default_bt_days_back")]
    days_back: i64,
    #[serde(default)]
    initial_equity: Option<f64>,
    #[serde(default)]
    fee_per_trade: Option<f64>,
    #[serde(default)]
    slippage_bps: Option<f64>,
    /// Regime-classifier efficiency window.
    #[serde(default = "default_regime_period")]
    regime_period: usize,
}

fn default_regime_period() -> usize {
    20
}

#[derive(serde::Serialize)]
struct BacktestRegimesResult {
    backtest_summary: traderview_core::algo_backtest::AlgoBtSummary,
    attribution: traderview_core::algo_regime_attribution::RegimeAttribution,
}

/// POST /algo/strategies/:id/backtest-regimes — backtest, then bucket
/// every trade by the market regime at its ENTRY bar. Answers "WHEN
/// does this strategy earn" so a regime gate is added deliberately.
async fn post_backtest_regimes(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<BacktestRegimesBody>,
) -> Result<Json<BacktestRegimesResult>, ApiError> {
    let strategy = traderview_db::algo::get_strategy(&s.pool, u.id, id)
        .await
        .map_err(ApiError::Internal)?
        .ok_or_else(|| ApiError::BadRequest("strategy not found".into()))?;
    let interval: traderview_core::BarInterval = serde_json::from_value(serde_json::Value::String(
        normalize_interval(&body.interval),
    ))
    .map_err(|e| ApiError::BadRequest(format!("interval: {e}")))?;
    let to = chrono::Utc::now();
    let from = to - chrono::Duration::days(body.days_back);
    let bars = traderview_db::prices::get_bars(&s.pool, &body.symbol, interval, from, to)
        .await
        .map_err(ApiError::Internal)?;
    if bars.len() < 30 {
        return Err(ApiError::BadRequest(format!(
            "only {} bars available for {} — need >=30",
            bars.len(),
            body.symbol
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
    let bt = traderview_core::algo_backtest::run(&bars, strat.as_ref(), &sizing, cfg);
    let attribution =
        traderview_core::algo_regime_attribution::attribute(&bt.trades, &bars, body.regime_period);
    Ok(Json(BacktestRegimesResult {
        backtest_summary: bt.summary,
        attribution,
    }))
}

#[derive(serde::Serialize)]
struct LiveVsBacktestResult {
    report: traderview_core::live_vs_backtest::DivergenceReport,
    /// Which persisted backtest supplied the expectation.
    expectation_backtest_at: chrono::DateTime<chrono::Utc>,
    expectation_symbol: String,
}

/// GET /algo/strategies/:id/live-vs-backtest — reconstruct the live
/// fill record into FIFO round trips and test the live win rate
/// against the LATEST persisted backtest's expectation. The answer to
/// "is the live strategy still the one I backtested".
async fn get_live_vs_backtest(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<LiveVsBacktestResult>, ApiError> {
    let (report, expectation_backtest_at, expectation_symbol) =
        traderview_db::algo::live_divergence(&s.pool, u.id, id)
            .await
            .map_err(ApiError::Internal)?
            .ok_or_else(|| {
                ApiError::BadRequest("no persisted backtest — run a backtest first so there's an expectation to test against".into())
            })?;
    Ok(Json(LiveVsBacktestResult {
        report,
        expectation_backtest_at,
        expectation_symbol,
    }))
}

#[derive(serde::Deserialize)]
struct GateFiresQ {
    #[serde(default = "default_gate_window")]
    window_days: i64,
}
fn default_gate_window() -> i64 {
    7
}

/// GET /algo/strategies/:id/gate-fires — per-gate fire counts over the
/// trailing window + the latest audit rows. The data that turns gate
/// tuning from vibes into engineering.
async fn get_gate_fires(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
    Query(q): Query<GateFiresQ>,
) -> Result<Json<traderview_db::algo::GateFireSummary>, ApiError> {
    traderview_db::algo::gate_fire_summary(&s.pool, u.id, id, q.window_days.clamp(1, 365))
        .await
        .map_err(ApiError::Internal)?
        .map(Json)
        .ok_or_else(|| ApiError::BadRequest("strategy not found".into()))
}

/// GET /algo/strategies/:id/revisions — prior configs newest-first.
/// Pairs with drift detection: when performance falls off, this is
/// where "what changed" gets answered.
async fn get_revisions(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<traderview_db::algo::StrategyRevision>>, ApiError> {
    traderview_db::algo::list_revisions(&s.pool, u.id, id, 50)
        .await
        .map_err(ApiError::Internal)?
        .map(Json)
        .ok_or_else(|| ApiError::BadRequest("strategy not found".into()))
}

/// POST /algo/strategies/:id/revisions/:rev_id/restore — write the
/// revision's config back through update_strategy, which snapshots the
/// CURRENT config first: a restore is itself reversible. Fields the
/// revision doesn't capture (enabled, universe, watchlist, account)
/// keep their current values. Restoring to live broker_mode is
/// downgraded to paper — the paper-lock rationale applies to restores
/// exactly as it does to fresh saves.
async fn post_restore_revision(
    State(s): State<AppState>,
    u: AuthUser,
    Path((id, rev_id)): Path<(Uuid, i64)>,
) -> Result<Json<AlgoStrategy>, ApiError> {
    let current = traderview_db::algo::get_strategy(&s.pool, u.id, id)
        .await
        .map_err(ApiError::Internal)?
        .ok_or_else(|| ApiError::BadRequest("strategy not found".into()))?;
    let rev = traderview_db::algo::get_revision(&s.pool, u.id, id, rev_id)
        .await
        .map_err(ApiError::Internal)?
        .ok_or_else(|| ApiError::BadRequest("revision not found".into()))?;
    // The registry could have dropped a kind since the revision was
    // taken — the same validation fresh saves get.
    validate_strategy_type(&rev.strategy_type)?;
    let broker_mode = if rev.broker_mode == "live" {
        "paper".to_string()
    } else {
        rev.broker_mode.clone()
    };
    let input = AlgoStrategyInput {
        name: rev.name.clone(),
        enabled: current.enabled,
        timeframe: rev.timeframe.clone(),
        universe_mode: current.universe_mode.clone(),
        watchlist_id: current.watchlist_id,
        autoscan_top_n: current.autoscan_top_n,
        side_mode: rev.side_mode.clone(),
        strategy_type: rev.strategy_type.clone(),
        account_id: Some(current.account_id),
        entry_rules: rev.entry_rules.clone(),
        exit_rules: rev.exit_rules.clone(),
        sizing: rev.sizing.clone(),
        risk_gates: rev.risk_gates.clone(),
        broker_mode,
    };
    traderview_db::algo::update_strategy(&s.pool, u.id, id, input)
        .await
        .map_err(ApiError::Internal)?
        .map(Json)
        .ok_or_else(|| ApiError::BadRequest("strategy not found".into()))
}

#[derive(serde::Deserialize)]
struct ForkBody {
    /// Challenger name; default "<champion> (fork)".
    #[serde(default)]
    name: Option<String>,
}

/// POST /algo/strategies/:id/fork — champion/challenger: duplicate the
/// strategy so a tweak can run side by side with the original instead
/// of replacing it. The fork starts DISABLED in internal_sim
/// regardless of the champion's mode (an experiment never inherits
/// live execution), and the auto-tagging from fills (algo:<name>)
/// keeps the two attributable separately in the journal for free.
async fn post_fork_strategy(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<ForkBody>,
) -> Result<Json<AlgoStrategy>, ApiError> {
    let src = traderview_db::algo::get_strategy(&s.pool, u.id, id)
        .await
        .map_err(ApiError::Internal)?
        .ok_or_else(|| ApiError::BadRequest("strategy not found".into()))?;
    let name = body
        .name
        .map(|n| n.trim().to_string())
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| format!("{} (fork)", src.name));
    let input = AlgoStrategyInput {
        name,
        enabled: false,
        timeframe: src.timeframe.clone(),
        universe_mode: src.universe_mode.clone(),
        watchlist_id: src.watchlist_id,
        autoscan_top_n: src.autoscan_top_n,
        side_mode: src.side_mode.clone(),
        strategy_type: src.strategy_type.clone(),
        account_id: Some(src.account_id),
        entry_rules: src.entry_rules.clone(),
        exit_rules: src.exit_rules.clone(),
        sizing: src.sizing.clone(),
        risk_gates: src.risk_gates.clone(),
        broker_mode: "internal_sim".into(),
    };
    traderview_db::algo::create_strategy(&s.pool, u.id, input)
        .await
        .map(Json)
        .map_err(ApiError::Internal)
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The immune-system test for the bug this replaces: a hardcoded
    /// type list silently rejected 8 registry strategies. Validation
    /// must accept EVERY kind the registry exposes, forever.
    #[test]
    fn every_registry_kind_is_saveable() {
        for kind in traderview_core::algo_strategies::StrategyKind::all() {
            validate_strategy_type(kind.as_str())
                .unwrap_or_else(|_| panic!("{} rejected by route validation", kind.as_str()));
        }
    }

    #[test]
    fn unknown_kind_is_rejected_with_the_full_list() {
        let err = validate_strategy_type("not_a_strategy").unwrap_err();
        let msg = format!("{err:?}");
        assert!(msg.contains("not_a_strategy"));
        // The error enumerates the registry, derived not typed.
        assert!(msg.contains("macd_cross") && msg.contains("pairs"));
    }
}
