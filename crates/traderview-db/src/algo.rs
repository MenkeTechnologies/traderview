//! Algorithmic momentum trading — strategy / run / order / fill repository.
//!
//! Schema lives in `migrations/0052_algo_trading.sql`. Engine code that
//! consumes these helpers lives in `traderview-core::algo_engine`; this
//! module is pure persistence.
//!
//! Lifecycle invariants enforced here (not just at the SQL level):
//! - At most one open run per strategy (`algo_runs_one_open_per_strategy`).
//! - `kill_switch = TRUE` is recorded in `algo_kill_switch_audit` and
//!   sets `last_kill_at`; releasing it does the same with `'released'`.
//! - `paper_locked_until` is enforced at the engine boundary, not in SQL,
//!   so the DTO returns it raw and the engine refuses to send to live
//!   while `now() <= paper_locked_until`.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;
use sqlx::PgPool;
use uuid::Uuid;

// ─── DTOs ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct AlgoStrategy {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub enabled: bool,
    pub timeframe: String, // 'sec10' | 'min1'
    pub universe_mode: String, // 'watchlist' | 'autoscan'
    pub watchlist_id: Option<Uuid>,
    pub autoscan_top_n: i32,
    pub side_mode: String, // 'long' | 'short' | 'both'
    /// Discriminator picked up by `traderview-core::algo_strategies::from_kind`.
    /// One of: momentum | mean_reversion | orb | donchian_trend | bb_squeeze.
    pub strategy_type: String,
    /// Broker account the strategy produces executions against. Enforced
    /// NOT NULL since migration 0056 — every algo strategy MUST be
    /// bound to a real `accounts` row at the schema level. Route layer
    /// rejects unbound input with a friendly error before INSERT.
    pub account_id: Uuid,
    pub entry_rules: Json,
    pub exit_rules: Json,
    pub sizing: Json,
    pub risk_gates: Json,
    pub broker_mode: String, // 'internal_sim' | 'alpaca_paper' | 'alpaca_live'
    pub paper_locked_until: DateTime<Utc>,
    pub kill_switch: bool,
    pub kill_reason: Option<String>,
    pub last_kill_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AlgoStrategyInput {
    pub name: String,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_timeframe")]
    pub timeframe: String,
    #[serde(default = "default_universe_mode")]
    pub universe_mode: String,
    #[serde(default)]
    pub watchlist_id: Option<Uuid>,
    #[serde(default = "default_autoscan_top_n")]
    pub autoscan_top_n: i32,
    #[serde(default = "default_side_mode")]
    pub side_mode: String,
    #[serde(default = "default_strategy_type")]
    pub strategy_type: String,
    /// Required at the route layer with a friendly error message. Kept
    /// Option here ONLY so the JSON deserializer doesn't 400 on a
    /// missing field — the route handler maps None → BadRequest with
    /// a usable message before the DB INSERT (which would otherwise
    /// 23502 on NOT NULL).
    #[serde(default)]
    pub account_id: Option<Uuid>,
    #[serde(default)]
    pub entry_rules: Json,
    #[serde(default)]
    pub exit_rules: Json,
    #[serde(default)]
    pub sizing: Json,
    #[serde(default)]
    pub risk_gates: Json,
    #[serde(default = "default_broker_mode")]
    pub broker_mode: String,
}

fn default_timeframe() -> String { "min1".into() }
fn default_universe_mode() -> String { "watchlist".into() }
fn default_autoscan_top_n() -> i32 { 25 }
fn default_side_mode() -> String { "long".into() }
fn default_broker_mode() -> String { "internal_sim".into() }
fn default_strategy_type() -> String { "momentum".into() }

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct AlgoRun {
    pub id: Uuid,
    pub strategy_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub stopped_at: Option<DateTime<Utc>>,
    pub stopped_reason: Option<String>,
    pub bars_processed: i64,
    pub signals_emitted: i64,
    pub orders_submitted: i64,
    pub fills_received: i64,
    pub pnl_realized: Decimal,
    pub pnl_unrealized_at_stop: Option<Decimal>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct AlgoOrder {
    pub id: Uuid,
    pub run_id: Uuid,
    pub strategy_id: Uuid,
    pub client_order_id: Uuid,
    pub broker_order_id: Option<String>,
    pub symbol: String,
    pub side: String,
    pub order_type: String,
    pub order_class: String,
    pub qty: Decimal,
    pub limit_price: Option<Decimal>,
    pub stop_price: Option<Decimal>,
    pub status: String,
    pub submitted_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub raw_request: Option<Json>,
    pub raw_response: Option<Json>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AlgoOrderInsert {
    pub client_order_id: Uuid,
    pub symbol: String,
    pub side: String,
    pub order_type: String,
    #[serde(default = "default_order_class")]
    pub order_class: String,
    pub qty: Decimal,
    pub limit_price: Option<Decimal>,
    pub stop_price: Option<Decimal>,
    pub raw_request: Option<Json>,
}

fn default_order_class() -> String { "simple".into() }

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct AlgoFill {
    pub id: Uuid,
    pub order_id: Uuid,
    pub broker_fill_id: Option<String>,
    pub fill_qty: Decimal,
    pub fill_price: Decimal,
    pub fill_value: Decimal,
    pub commission: Decimal,
    pub filled_at: DateTime<Utc>,
    pub raw: Option<Json>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AlgoFillInsert {
    pub order_id: Uuid,
    pub broker_fill_id: Option<String>,
    pub fill_qty: Decimal,
    pub fill_price: Decimal,
    pub commission: Decimal,
    pub raw: Option<Json>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct KillSwitchAudit {
    pub id: Uuid,
    pub strategy_id: Uuid,
    pub actor_user_id: Uuid,
    pub action: String, // 'engaged' | 'released'
    pub reason: Option<String>,
    pub at: DateTime<Utc>,
}

// ─── strategy CRUD ──────────────────────────────────────────────────────────

pub async fn list_strategies(pool: &PgPool, user_id: Uuid) -> anyhow::Result<Vec<AlgoStrategy>> {
    Ok(sqlx::query_as::<_, AlgoStrategy>(
        "SELECT * FROM algo_strategies WHERE user_id = $1 ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?)
}

/// Every enabled strategy in the system whose kill_switch is OFF.
/// Used by the algo runner background task to drive bar ticks. Sorts
/// by timeframe so 10s strategies fire first within a tick (they have
/// the tightest latency budget).
pub async fn list_active_strategies(pool: &PgPool) -> anyhow::Result<Vec<AlgoStrategy>> {
    Ok(sqlx::query_as::<_, AlgoStrategy>(
        "SELECT * FROM algo_strategies
          WHERE enabled = TRUE AND kill_switch = FALSE
          ORDER BY
            CASE timeframe WHEN 'sec10' THEN 0 ELSE 1 END,
            created_at",
    )
    .fetch_all(pool)
    .await?)
}

/// System-internal lookup with no per-user scope. Used by background
/// pumps (Alpaca trade_updates, scheduled runner) that already know
/// the strategy_id is valid — they got it from a FK-linked row. Do not
/// expose to HTTP routes; use `get_strategy(pool, user_id, id)` there.
pub async fn get_strategy_by_id(
    pool: &PgPool,
    id: Uuid,
) -> anyhow::Result<Option<AlgoStrategy>> {
    Ok(
        sqlx::query_as::<_, AlgoStrategy>("SELECT * FROM algo_strategies WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?,
    )
}

pub async fn get_strategy(
    pool: &PgPool,
    user_id: Uuid,
    id: Uuid,
) -> anyhow::Result<Option<AlgoStrategy>> {
    Ok(sqlx::query_as::<_, AlgoStrategy>(
        "SELECT * FROM algo_strategies WHERE id = $1 AND user_id = $2",
    )
    .bind(id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?)
}

pub async fn create_strategy(
    pool: &PgPool,
    user_id: Uuid,
    input: AlgoStrategyInput,
) -> anyhow::Result<AlgoStrategy> {
    Ok(sqlx::query_as::<_, AlgoStrategy>(
        "INSERT INTO algo_strategies
             (user_id, name, enabled, timeframe, universe_mode, watchlist_id,
              autoscan_top_n, side_mode, strategy_type, account_id,
              entry_rules, exit_rules, sizing, risk_gates, broker_mode)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
         RETURNING *",
    )
    .bind(user_id)
    .bind(input.name)
    .bind(input.enabled)
    .bind(input.timeframe)
    .bind(input.universe_mode)
    .bind(input.watchlist_id)
    .bind(input.autoscan_top_n)
    .bind(input.side_mode)
    .bind(input.strategy_type)
    .bind(input.account_id)
    .bind(input.entry_rules)
    .bind(input.exit_rules)
    .bind(input.sizing)
    .bind(input.risk_gates)
    .bind(input.broker_mode)
    .fetch_one(pool)
    .await?)
}

pub async fn update_strategy(
    pool: &PgPool,
    user_id: Uuid,
    id: Uuid,
    input: AlgoStrategyInput,
) -> anyhow::Result<Option<AlgoStrategy>> {
    Ok(sqlx::query_as::<_, AlgoStrategy>(
        "UPDATE algo_strategies
            SET name = $3, enabled = $4, timeframe = $5, universe_mode = $6,
                watchlist_id = $7, autoscan_top_n = $8, side_mode = $9,
                strategy_type = $10, account_id = $11, entry_rules = $12,
                exit_rules = $13, sizing = $14, risk_gates = $15,
                broker_mode = $16, updated_at = now()
          WHERE id = $1 AND user_id = $2
          RETURNING *",
    )
    .bind(id)
    .bind(user_id)
    .bind(input.name)
    .bind(input.enabled)
    .bind(input.timeframe)
    .bind(input.universe_mode)
    .bind(input.watchlist_id)
    .bind(input.autoscan_top_n)
    .bind(input.side_mode)
    .bind(input.strategy_type)
    .bind(input.account_id)
    .bind(input.entry_rules)
    .bind(input.exit_rules)
    .bind(input.sizing)
    .bind(input.risk_gates)
    .bind(input.broker_mode)
    .fetch_optional(pool)
    .await?)
}

pub async fn delete_strategy(pool: &PgPool, user_id: Uuid, id: Uuid) -> anyhow::Result<bool> {
    let r = sqlx::query("DELETE FROM algo_strategies WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(r.rows_affected() > 0)
}

// ─── kill switch ────────────────────────────────────────────────────────────

/// Flip the kill switch and append an audit row in one tx. Returns the
/// updated strategy. No-ops (and returns the existing row) if the strategy
/// is already in the requested state.
pub async fn set_kill_switch(
    pool: &PgPool,
    user_id: Uuid,
    strategy_id: Uuid,
    engaged: bool,
    reason: Option<String>,
) -> anyhow::Result<Option<AlgoStrategy>> {
    let mut tx = pool.begin().await?;

    let current: Option<AlgoStrategy> = sqlx::query_as(
        "SELECT * FROM algo_strategies WHERE id = $1 AND user_id = $2 FOR UPDATE",
    )
    .bind(strategy_id)
    .bind(user_id)
    .fetch_optional(&mut *tx)
    .await?;

    let Some(current) = current else {
        tx.rollback().await.ok();
        return Ok(None);
    };

    if current.kill_switch == engaged {
        tx.rollback().await.ok();
        return Ok(Some(current));
    }

    let updated: AlgoStrategy = sqlx::query_as(
        "UPDATE algo_strategies
            SET kill_switch = $3, kill_reason = $4,
                last_kill_at = now(), updated_at = now()
          WHERE id = $1 AND user_id = $2
          RETURNING *",
    )
    .bind(strategy_id)
    .bind(user_id)
    .bind(engaged)
    .bind(reason.clone())
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO algo_kill_switch_audit (strategy_id, actor_user_id, action, reason)
         VALUES ($1, $2, $3, $4)",
    )
    .bind(strategy_id)
    .bind(user_id)
    .bind(if engaged { "engaged" } else { "released" })
    .bind(reason.clone())
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    // Fan out to user webhooks AFTER the transaction commits so a flaky
    // sink can't roll back the audit row. Best-effort.
    let (title, kind) = if engaged {
        (
            format!("Kill switch engaged: {}", updated.name),
            "algo_kill_engaged",
        )
    } else {
        (
            format!("Kill switch released: {}", updated.name),
            "algo_kill_released",
        )
    };
    let payload = crate::webhooks::AlertPayload {
        title,
        message: reason.unwrap_or_else(|| "(no reason supplied)".into()),
        symbol: None,
        kind: kind.into(),
        url: None,
        fired_at: Utc::now(),
    };
    crate::webhooks::fan_out_all(pool, user_id, &payload).await;
    Ok(Some(updated))
}

pub async fn kill_switch_history(
    pool: &PgPool,
    strategy_id: Uuid,
    limit: i64,
) -> anyhow::Result<Vec<KillSwitchAudit>> {
    Ok(sqlx::query_as::<_, KillSwitchAudit>(
        "SELECT * FROM algo_kill_switch_audit
          WHERE strategy_id = $1 ORDER BY at DESC LIMIT $2",
    )
    .bind(strategy_id)
    .bind(limit)
    .fetch_all(pool)
    .await?)
}

// ─── runs ───────────────────────────────────────────────────────────────────

pub async fn start_run(pool: &PgPool, strategy_id: Uuid) -> anyhow::Result<AlgoRun> {
    Ok(sqlx::query_as::<_, AlgoRun>(
        "INSERT INTO algo_runs (strategy_id) VALUES ($1) RETURNING *",
    )
    .bind(strategy_id)
    .fetch_one(pool)
    .await?)
}

pub async fn stop_run(
    pool: &PgPool,
    run_id: Uuid,
    reason: &str,
    pnl_unrealized: Option<Decimal>,
    last_error: Option<String>,
) -> anyhow::Result<Option<AlgoRun>> {
    Ok(sqlx::query_as::<_, AlgoRun>(
        "UPDATE algo_runs
            SET stopped_at = now(), stopped_reason = $2,
                pnl_unrealized_at_stop = $3, last_error = $4
          WHERE id = $1 AND stopped_at IS NULL
          RETURNING *",
    )
    .bind(run_id)
    .bind(reason)
    .bind(pnl_unrealized)
    .bind(last_error)
    .fetch_optional(pool)
    .await?)
}

pub async fn get_open_run(
    pool: &PgPool,
    strategy_id: Uuid,
) -> anyhow::Result<Option<AlgoRun>> {
    Ok(sqlx::query_as::<_, AlgoRun>(
        "SELECT * FROM algo_runs
          WHERE strategy_id = $1 AND stopped_at IS NULL
          LIMIT 1",
    )
    .bind(strategy_id)
    .fetch_optional(pool)
    .await?)
}

pub async fn list_runs(
    pool: &PgPool,
    strategy_id: Uuid,
    limit: i64,
) -> anyhow::Result<Vec<AlgoRun>> {
    Ok(sqlx::query_as::<_, AlgoRun>(
        "SELECT * FROM algo_runs
          WHERE strategy_id = $1
          ORDER BY started_at DESC LIMIT $2",
    )
    .bind(strategy_id)
    .bind(limit)
    .fetch_all(pool)
    .await?)
}

pub async fn increment_run_counter(
    pool: &PgPool,
    run_id: Uuid,
    counter: RunCounter,
    delta: i64,
) -> anyhow::Result<()> {
    let col = match counter {
        RunCounter::BarsProcessed => "bars_processed",
        RunCounter::SignalsEmitted => "signals_emitted",
        RunCounter::OrdersSubmitted => "orders_submitted",
        RunCounter::FillsReceived => "fills_received",
    };
    // SAFE: `col` is matched from a closed enum above, never user input.
    let q = format!("UPDATE algo_runs SET {col} = {col} + $2 WHERE id = $1");
    sqlx::query(&q)
        .bind(run_id)
        .bind(delta)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn add_realized_pnl(
    pool: &PgPool,
    run_id: Uuid,
    delta: Decimal,
) -> anyhow::Result<()> {
    sqlx::query(
        "UPDATE algo_runs SET pnl_realized = pnl_realized + $2 WHERE id = $1",
    )
    .bind(run_id)
    .bind(delta)
    .execute(pool)
    .await?;
    Ok(())
}

pub enum RunCounter {
    BarsProcessed,
    SignalsEmitted,
    OrdersSubmitted,
    FillsReceived,
}

// ─── orders + fills ─────────────────────────────────────────────────────────

pub async fn insert_order(
    pool: &PgPool,
    run_id: Uuid,
    strategy_id: Uuid,
    insert: AlgoOrderInsert,
) -> anyhow::Result<AlgoOrder> {
    Ok(sqlx::query_as::<_, AlgoOrder>(
        "INSERT INTO algo_orders
             (run_id, strategy_id, client_order_id, symbol, side,
              order_type, order_class, qty, limit_price, stop_price, raw_request)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
         RETURNING *",
    )
    .bind(run_id)
    .bind(strategy_id)
    .bind(insert.client_order_id)
    .bind(insert.symbol)
    .bind(insert.side)
    .bind(insert.order_type)
    .bind(insert.order_class)
    .bind(insert.qty)
    .bind(insert.limit_price)
    .bind(insert.stop_price)
    .bind(insert.raw_request)
    .fetch_one(pool)
    .await?)
}

/// Called once we have the broker's response. Sets `broker_order_id`,
/// `status`, `raw_response`, and optionally `error`.
pub async fn mark_order_submitted(
    pool: &PgPool,
    id: Uuid,
    broker_order_id: Option<String>,
    status: &str,
    raw_response: Option<Json>,
    error: Option<String>,
) -> anyhow::Result<()> {
    sqlx::query(
        "UPDATE algo_orders
            SET broker_order_id = $2, status = $3, raw_response = $4,
                error = $5, updated_at = now()
          WHERE id = $1",
    )
    .bind(id)
    .bind(broker_order_id)
    .bind(status)
    .bind(raw_response)
    .bind(error)
    .execute(pool)
    .await?;
    Ok(())
}

/// Called from the trade_updates WS handler.
pub async fn update_order_status(
    pool: &PgPool,
    client_order_id: Uuid,
    status: &str,
) -> anyhow::Result<Option<AlgoOrder>> {
    Ok(sqlx::query_as::<_, AlgoOrder>(
        "UPDATE algo_orders SET status = $2, updated_at = now()
          WHERE client_order_id = $1 RETURNING *",
    )
    .bind(client_order_id)
    .bind(status)
    .fetch_optional(pool)
    .await?)
}

pub async fn list_orders(
    pool: &PgPool,
    run_id: Uuid,
    limit: i64,
) -> anyhow::Result<Vec<AlgoOrder>> {
    Ok(sqlx::query_as::<_, AlgoOrder>(
        "SELECT * FROM algo_orders WHERE run_id = $1
          ORDER BY submitted_at DESC LIMIT $2",
    )
    .bind(run_id)
    .bind(limit)
    .fetch_all(pool)
    .await?)
}

pub async fn insert_fill(pool: &PgPool, insert: AlgoFillInsert) -> anyhow::Result<AlgoFill> {
    let fill_value = insert.fill_qty * insert.fill_price;
    Ok(sqlx::query_as::<_, AlgoFill>(
        "INSERT INTO algo_fills
             (order_id, broker_fill_id, fill_qty, fill_price, fill_value, commission, raw)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         RETURNING *",
    )
    .bind(insert.order_id)
    .bind(insert.broker_fill_id)
    .bind(insert.fill_qty)
    .bind(insert.fill_price)
    .bind(fill_value)
    .bind(insert.commission)
    .bind(insert.raw)
    .fetch_one(pool)
    .await?)
}

pub async fn list_fills(
    pool: &PgPool,
    order_id: Uuid,
) -> anyhow::Result<Vec<AlgoFill>> {
    Ok(sqlx::query_as::<_, AlgoFill>(
        "SELECT * FROM algo_fills WHERE order_id = $1 ORDER BY filled_at DESC",
    )
    .bind(order_id)
    .fetch_all(pool)
    .await?)
}

// ─── persisted backtest history ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct AlgoBacktestRow {
    pub id: Uuid,
    pub strategy_id: Uuid,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub symbol: String,
    pub interval: String,
    pub range_from: DateTime<Utc>,
    pub range_to: DateTime<Utc>,
    pub initial_equity: Decimal,
    pub fee_per_trade: Decimal,
    pub slippage_bps: Decimal,
    pub entry_rules: Json,
    pub trades: i64,
    pub wins: i64,
    pub losses: i64,
    pub win_rate: Decimal,
    pub avg_r: Decimal,
    pub profit_factor: Decimal,
    pub total_return_pct: Decimal,
    pub max_drawdown_pct: Decimal,
    pub final_equity: Decimal,
    pub sharpe: Decimal,
}

#[derive(Debug, Clone)]
pub struct AlgoBacktestInsert {
    pub strategy_id: Uuid,
    pub user_id: Uuid,
    pub symbol: String,
    pub interval: String,
    pub range_from: DateTime<Utc>,
    pub range_to: DateTime<Utc>,
    pub initial_equity: Decimal,
    pub fee_per_trade: Decimal,
    pub slippage_bps: Decimal,
    pub entry_rules: Json,
    pub trades: i64,
    pub wins: i64,
    pub losses: i64,
    pub win_rate: Decimal,
    pub avg_r: Decimal,
    pub profit_factor: Decimal,
    pub total_return_pct: Decimal,
    pub max_drawdown_pct: Decimal,
    pub final_equity: Decimal,
    pub sharpe: Decimal,
}

pub async fn save_backtest(
    pool: &PgPool,
    insert: AlgoBacktestInsert,
) -> anyhow::Result<AlgoBacktestRow> {
    Ok(sqlx::query_as::<_, AlgoBacktestRow>(
        "INSERT INTO algo_backtests
            (strategy_id, user_id, symbol, interval, range_from, range_to,
             initial_equity, fee_per_trade, slippage_bps, entry_rules,
             trades, wins, losses, win_rate, avg_r, profit_factor,
             total_return_pct, max_drawdown_pct, final_equity, sharpe)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20)
         RETURNING *",
    )
    .bind(insert.strategy_id)
    .bind(insert.user_id)
    .bind(insert.symbol)
    .bind(insert.interval)
    .bind(insert.range_from)
    .bind(insert.range_to)
    .bind(insert.initial_equity)
    .bind(insert.fee_per_trade)
    .bind(insert.slippage_bps)
    .bind(insert.entry_rules)
    .bind(insert.trades)
    .bind(insert.wins)
    .bind(insert.losses)
    .bind(insert.win_rate)
    .bind(insert.avg_r)
    .bind(insert.profit_factor)
    .bind(insert.total_return_pct)
    .bind(insert.max_drawdown_pct)
    .bind(insert.final_equity)
    .bind(insert.sharpe)
    .fetch_one(pool)
    .await?)
}

pub async fn list_backtests(
    pool: &PgPool,
    user_id: Uuid,
    strategy_id: Uuid,
    limit: i64,
) -> anyhow::Result<Vec<AlgoBacktestRow>> {
    Ok(sqlx::query_as::<_, AlgoBacktestRow>(
        "SELECT * FROM algo_backtests
          WHERE strategy_id = $1 AND user_id = $2
          ORDER BY created_at DESC
          LIMIT $3",
    )
    .bind(strategy_id)
    .bind(user_id)
    .bind(limit.clamp(1, 200))
    .fetch_all(pool)
    .await?)
}

pub async fn delete_backtest(
    pool: &PgPool,
    user_id: Uuid,
    backtest_id: Uuid,
) -> anyhow::Result<u64> {
    Ok(sqlx::query(
        "DELETE FROM algo_backtests WHERE id = $1 AND user_id = $2",
    )
    .bind(backtest_id)
    .bind(user_id)
    .execute(pool)
    .await?
    .rows_affected())
}

// ─── live metrics: rollup for the dashboard tab ─────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct EquityPoint {
    pub at: DateTime<Utc>,
    /// Cumulative pnl_realized up to AND including this run (in dollars).
    pub cumulative_pnl: Decimal,
    /// pnl_realized of this run alone — the delta from the previous point.
    pub delta_pnl: Decimal,
    pub run_id: Uuid,
}

#[derive(Debug, Clone, Serialize)]
pub struct StrategyMetrics {
    pub strategy_id: Uuid,
    pub runs: i64,
    pub bars_processed: i64,
    pub signals_emitted: i64,
    pub orders_submitted: i64,
    pub fills_received: i64,
    pub orders_rejected: i64,
    pub total_realized_pnl: Decimal,
    pub equity_curve: Vec<EquityPoint>,
    pub recent_orders: Vec<AlgoOrder>,
}

/// Aggregates everything the dashboard needs in ONE round trip:
///   - Lifetime counters (runs, bars, signals, orders, fills).
///   - Equity curve: every stopped run's pnl_realized cumulatively.
///     In-flight (`stopped_at IS NULL`) runs are skipped — their P&L
///     isn't settled yet.
///   - Recent orders (last 50) for the trades-table strip.
///   - Orders-rejected count: algo_orders.status = 'rejected'.
pub async fn strategy_metrics(
    pool: &PgPool,
    user_id: Uuid,
    strategy_id: Uuid,
) -> anyhow::Result<Option<StrategyMetrics>> {
    // Authorization: confirm the strategy belongs to this user before
    // exposing aggregate P&L numbers.
    let owned: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM algo_strategies WHERE id = $1 AND user_id = $2",
    )
    .bind(strategy_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;
    if owned.is_none() {
        return Ok(None);
    }

    let totals: (i64, i64, i64, i64, i64, Decimal) = sqlx::query_as(
        "SELECT
             COUNT(*)::bigint                                 AS runs,
             COALESCE(SUM(bars_processed), 0)::bigint         AS bars,
             COALESCE(SUM(signals_emitted), 0)::bigint        AS signals,
             COALESCE(SUM(orders_submitted), 0)::bigint       AS orders,
             COALESCE(SUM(fills_received), 0)::bigint         AS fills,
             COALESCE(SUM(pnl_realized), 0)::numeric          AS pnl
           FROM algo_runs
          WHERE strategy_id = $1",
    )
    .bind(strategy_id)
    .fetch_one(pool)
    .await?;

    let rejected: (i64,) = sqlx::query_as(
        "SELECT COUNT(*)::bigint FROM algo_orders
          WHERE strategy_id = $1 AND status = 'rejected'",
    )
    .bind(strategy_id)
    .fetch_one(pool)
    .await?;

    // Equity curve: walk stopped runs in chronological order.
    let curve_rows: Vec<(Uuid, DateTime<Utc>, Decimal)> = sqlx::query_as(
        "SELECT id, stopped_at, pnl_realized FROM algo_runs
          WHERE strategy_id = $1 AND stopped_at IS NOT NULL
          ORDER BY stopped_at ASC",
    )
    .bind(strategy_id)
    .fetch_all(pool)
    .await?;
    let mut equity_curve = Vec::with_capacity(curve_rows.len());
    let mut cum = Decimal::ZERO;
    for (rid, at, delta) in curve_rows {
        cum += delta;
        equity_curve.push(EquityPoint {
            at,
            cumulative_pnl: cum,
            delta_pnl: delta,
            run_id: rid,
        });
    }

    let recent_orders: Vec<AlgoOrder> = sqlx::query_as::<_, AlgoOrder>(
        "SELECT * FROM algo_orders
          WHERE strategy_id = $1
          ORDER BY submitted_at DESC
          LIMIT 50",
    )
    .bind(strategy_id)
    .fetch_all(pool)
    .await?;

    Ok(Some(StrategyMetrics {
        strategy_id,
        runs: totals.0,
        bars_processed: totals.1,
        signals_emitted: totals.2,
        orders_submitted: totals.3,
        fills_received: totals.4,
        orders_rejected: rejected.0,
        total_realized_pnl: totals.5,
        equity_curve,
        recent_orders,
    }))
}
