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
              autoscan_top_n, side_mode, strategy_type, entry_rules, exit_rules,
              sizing, risk_gates, broker_mode)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
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
                strategy_type = $10, entry_rules = $11, exit_rules = $12,
                sizing = $13, risk_gates = $14, broker_mode = $15,
                updated_at = now()
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
    .bind(reason)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
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
