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
    pub timeframe: String,     // 'sec10' | 'min1'
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
    /// The thesis — why this strategy should work, written before it
    /// runs. Documentation, not config: deliberately NOT snapshotted
    /// into revisions (editing the thesis isn't a config change).
    pub notes: Option<String>,
    pub paper_locked_until: DateTime<Utc>,
    pub kill_switch: bool,
    pub kill_reason: Option<String>,
    pub last_kill_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// Tombstone — soft-delete marker. NULL = active. Set by
    /// `delete_strategy` so historical runs / orders / fills stay
    /// intact while the UI hides the row.
    #[serde(default)]
    pub deleted_at: Option<DateTime<Utc>>,
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
    #[serde(default)]
    pub notes: Option<String>,
}

fn default_timeframe() -> String {
    "min1".into()
}
fn default_universe_mode() -> String {
    "watchlist".into()
}
fn default_autoscan_top_n() -> i32 {
    25
}
fn default_side_mode() -> String {
    "long".into()
}
fn default_broker_mode() -> String {
    "internal_sim".into()
}
fn default_strategy_type() -> String {
    "momentum".into()
}

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
    /// 'entry' | 'exit' — set by the engine at submit time.
    #[serde(default = "default_order_kind")]
    pub kind: String,
}

fn default_order_kind() -> String {
    "entry".into()
}

fn default_order_class() -> String {
    "simple".into()
}

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
    // Soft-deleted strategies (deleted_at IS NOT NULL) drop out of
    // the UI list but stay in the DB so their run history survives.
    Ok(sqlx::query_as::<_, AlgoStrategy>(
        "SELECT * FROM algo_strategies
          WHERE user_id = $1 AND deleted_at IS NULL
          ORDER BY created_at DESC",
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
            AND deleted_at IS NULL
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
pub async fn get_strategy_by_id(pool: &PgPool, id: Uuid) -> anyhow::Result<Option<AlgoStrategy>> {
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
              entry_rules, exit_rules, sizing, risk_gates, broker_mode, notes)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
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
    .bind(input.notes.as_deref().map(str::trim).filter(|s| !s.is_empty()))
    .fetch_one(pool)
    .await?)
}

pub async fn update_strategy(
    pool: &PgPool,
    user_id: Uuid,
    id: Uuid,
    input: AlgoStrategyInput,
) -> anyhow::Result<Option<AlgoStrategy>> {
    // Snapshot the PRIOR config before overwriting — the revision
    // history that lets drift detection ask "what changed". Inside
    // one INSERT…SELECT so the snapshot can't race the update.
    sqlx::query(
        "INSERT INTO algo_strategy_revisions
            (strategy_id, name, timeframe, side_mode, strategy_type,
             entry_rules, exit_rules, sizing, risk_gates, broker_mode)
         SELECT id, name, timeframe, side_mode, strategy_type,
                entry_rules, exit_rules, sizing, risk_gates, broker_mode
           FROM algo_strategies
          WHERE id = $1 AND user_id = $2",
    )
    .bind(id)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(sqlx::query_as::<_, AlgoStrategy>(
        "UPDATE algo_strategies
            SET name = $3, enabled = $4, timeframe = $5, universe_mode = $6,
                watchlist_id = $7, autoscan_top_n = $8, side_mode = $9,
                strategy_type = $10, account_id = $11, entry_rules = $12,
                exit_rules = $13, sizing = $14, risk_gates = $15,
                broker_mode = $16, notes = $17, updated_at = now()
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
    .bind(input.notes.as_deref().map(str::trim).filter(|s| !s.is_empty()))
    .fetch_optional(pool)
    .await?)
}

/// Soft delete — tombstone the strategy so its runs / orders / fills
/// stay intact (ON DELETE CASCADE on those FKs used to nuke them).
/// The runner skips strategies with `deleted_at IS NOT NULL`, the
/// UI list filters them out, but the runs panel can still resolve
/// the historical strategy name.
///
/// Re-creates the same logical effect of a delete from the UI's
/// perspective while preserving audit + analytics data. Restore is
/// via `UPDATE algo_strategies SET deleted_at = NULL` (manual SQL —
/// no dedicated route).
pub async fn delete_strategy(pool: &PgPool, user_id: Uuid, id: Uuid) -> anyhow::Result<bool> {
    // Wrap in a transaction so the soft-delete + open-run stop happen
    // atomically. Without the stop step, the open run sat in the DB
    // forever — the runner skipped it (enabled=false filter) but
    // `stopped_at IS NULL` made every "runs" lookup show a phantom
    // in-flight run for a strategy the user just deleted.
    let mut tx = pool.begin().await?;
    let r = sqlx::query(
        "UPDATE algo_strategies
            SET deleted_at = now(),
                enabled = FALSE,
                updated_at = now()
          WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL",
    )
    .bind(id)
    .bind(user_id)
    .execute(&mut *tx)
    .await?;
    if r.rows_affected() == 0 {
        tx.rollback().await.ok();
        return Ok(false);
    }
    // Stop any open runs so the runs panel doesn't display a
    // permanently-in-flight orphan. Idempotent — fires zero updates
    // when the strategy has no open run.
    sqlx::query(
        "UPDATE algo_runs
            SET stopped_at = now(),
                stopped_reason = 'user'
          WHERE strategy_id = $1 AND stopped_at IS NULL",
    )
    .bind(id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(true)
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

    let current: Option<AlgoStrategy> =
        sqlx::query_as("SELECT * FROM algo_strategies WHERE id = $1 AND user_id = $2 FOR UPDATE")
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

/// Return the user_id that owns the strategy bound to `run_id`, or
/// `None` if no such run exists. Used by the web layer to gate
/// /algo/runs/:id/orders + cross-tenant reads.
pub async fn run_owner(pool: &PgPool, run_id: Uuid) -> anyhow::Result<Option<Uuid>> {
    let row: Option<(Uuid,)> = sqlx::query_as(
        "SELECT s.user_id
           FROM algo_runs r
           JOIN algo_strategies s ON s.id = r.strategy_id
          WHERE r.id = $1",
    )
    .bind(run_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|(uid,)| uid))
}

/// Return the user_id that owns the strategy bound to `order_id`, or
/// `None` if no such order exists. Used by the web layer to gate
/// /algo/orders/:id/fills.
pub async fn order_owner(pool: &PgPool, order_id: Uuid) -> anyhow::Result<Option<Uuid>> {
    let row: Option<(Uuid,)> = sqlx::query_as(
        "SELECT s.user_id
           FROM algo_orders o
           JOIN algo_strategies s ON s.id = o.strategy_id
          WHERE o.id = $1",
    )
    .bind(order_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|(uid,)| uid))
}

// ─── runs ───────────────────────────────────────────────────────────────────

pub async fn start_run(pool: &PgPool, strategy_id: Uuid) -> anyhow::Result<AlgoRun> {
    Ok(
        sqlx::query_as::<_, AlgoRun>("INSERT INTO algo_runs (strategy_id) VALUES ($1) RETURNING *")
            .bind(strategy_id)
            .fetch_one(pool)
            .await?,
    )
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

pub async fn get_open_run(pool: &PgPool, strategy_id: Uuid) -> anyhow::Result<Option<AlgoRun>> {
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

pub async fn add_realized_pnl(pool: &PgPool, run_id: Uuid, delta: Decimal) -> anyhow::Result<()> {
    sqlx::query("UPDATE algo_runs SET pnl_realized = pnl_realized + $2 WHERE id = $1")
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
    // Idempotent on `client_order_id` so a retry after a network
    // timeout (where the caller can't tell if the first INSERT
    // committed) returns the existing row instead of raising 23505.
    let coid = insert.client_order_id;
    let row: Option<AlgoOrder> = sqlx::query_as::<_, AlgoOrder>(
        "INSERT INTO algo_orders
             (run_id, strategy_id, client_order_id, symbol, side,
              order_type, order_class, qty, limit_price, stop_price, raw_request, kind)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
         ON CONFLICT (client_order_id) DO NOTHING
         RETURNING *",
    )
    .bind(run_id)
    .bind(strategy_id)
    .bind(coid)
    .bind(insert.symbol)
    .bind(insert.side)
    .bind(insert.order_type)
    .bind(insert.order_class)
    .bind(insert.qty)
    .bind(insert.limit_price)
    .bind(insert.stop_price)
    .bind(insert.raw_request)
    .bind(insert.kind)
    .fetch_optional(pool)
    .await?;
    if let Some(r) = row {
        return Ok(r);
    }
    // Conflict path: the row already exists from a prior submit. Look
    // it up so the caller can continue with the existing row's id.
    let existing: AlgoOrder = sqlx::query_as::<_, AlgoOrder>(
        "SELECT * FROM algo_orders WHERE client_order_id = $1",
    )
    .bind(coid)
    .fetch_one(pool)
    .await?;
    Ok(existing)
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

/// True when an outstanding (not yet filled / rejected / canceled /
/// expired) algo_orders row exists for (strategy_id, symbol, side).
/// The exit pass calls this BEFORE submitting a close — without it,
/// real-Alpaca flows (where the entry fill lands later via the
/// trade_updates WebSocket, not synchronously) re-submit the same
/// close every tick until the WS finally closes the trade row, which
/// can over-sell several times before settling.
pub async fn has_pending_order(
    pool: &PgPool,
    strategy_id: Uuid,
    symbol: &str,
    side: &str,
) -> anyhow::Result<bool> {
    let (n,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM algo_orders
          WHERE strategy_id = $1
            AND symbol = $2
            AND side = $3
            AND status NOT IN ('filled', 'rejected', 'canceled', 'expired')",
    )
    .bind(strategy_id)
    .bind(symbol)
    .bind(side)
    .fetch_one(pool)
    .await?;
    Ok(n > 0)
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

pub async fn insert_fill(pool: &PgPool, insert: AlgoFillInsert) -> anyhow::Result<Option<AlgoFill>> {
    // Idempotent on broker_fill_id (UNIQUE in the schema). Alpaca's
    // trade_updates WS replays prior fills on every reconnect — without
    // ON CONFLICT, the second insert raises 23505 and bubbles up
    // through record_fill, aborting the entire flow including the
    // executions + trades::rollup_account writes that already
    // happened. Returns None on conflict so the caller can detect
    // "this fill was already processed; skip the rest of record_fill".
    let fill_value = insert.fill_qty * insert.fill_price;
    let row = sqlx::query_as::<_, AlgoFill>(
        "INSERT INTO algo_fills
             (order_id, broker_fill_id, fill_qty, fill_price, fill_value, commission, raw)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         ON CONFLICT (broker_fill_id) DO NOTHING
         RETURNING *",
    )
    .bind(insert.order_id)
    .bind(insert.broker_fill_id)
    .bind(insert.fill_qty)
    .bind(insert.fill_price)
    .bind(fill_value)
    .bind(insert.commission)
    .bind(insert.raw)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn list_fills(pool: &PgPool, order_id: Uuid) -> anyhow::Result<Vec<AlgoFill>> {
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

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct StrategyFill {
    pub symbol: String,
    pub side: String,
    pub fill_qty: Decimal,
    pub fill_price: Decimal,
    pub commission: Decimal,
    pub filled_at: chrono::DateTime<chrono::Utc>,
}

/// Every fill the strategy has received, chronological — the raw
/// material for live round-trip reconstruction.
/// The strategy's closed round trips, chronological by close time —
/// THE shared reconstruction (fills grouped per symbol, FIFO trips,
/// then re-sorted). Consumers: live_divergence, the max-drawdown gate.
pub async fn strategy_trips(
    pool: &PgPool,
    user_id: Uuid,
    strategy_id: Uuid,
) -> anyhow::Result<Vec<traderview_core::live_vs_backtest::Trip>> {
    use rust_decimal::prelude::ToPrimitive;
    let fills = fills_for_strategy(pool, user_id, strategy_id).await?;
    let mut by_symbol: std::collections::BTreeMap<String, Vec<traderview_core::live_vs_backtest::Fill>> =
        std::collections::BTreeMap::new();
    for f in fills {
        by_symbol.entry(f.symbol.clone()).or_default().push(
            traderview_core::live_vs_backtest::Fill {
                buy: f.side == "buy",
                qty: f.fill_qty.to_f64().unwrap_or(0.0),
                price: f.fill_price.to_f64().unwrap_or(0.0),
                commission: f.commission.to_f64().unwrap_or(0.0),
                ts: f.filled_at.timestamp(),
                flag: false,
            },
        );
    }
    let mut trips = Vec::new();
    for fills in by_symbol.values() {
        trips.extend(traderview_core::live_vs_backtest::round_trips(fills));
    }
    trips.sort_by_key(|t| t.closed_ts);
    Ok(trips)
}

#[derive(Debug, serde::Serialize)]
pub struct PnlPoint {
    /// Epoch seconds of the trip's CLOSE — realized PnL lands when
    /// the trip closes, not when it opens.
    pub ts: i64,
    pub cum_pnl: f64,
}

#[derive(Debug, serde::Serialize)]
pub struct PnlCurve {
    pub points: Vec<PnlPoint>,
    pub total_pnl: f64,
    pub peak_pnl: f64,
    /// Same realized_drawdown the circuit breaker evaluates.
    pub current_drawdown: f64,
    pub trips: usize,
}

/// Cumulative realized PnL by trip close time — the strategy's own
/// equity-curve analogue (realized only; open positions don't move
/// this line until they close).
pub async fn pnl_curve(
    pool: &PgPool,
    user_id: Uuid,
    strategy_id: Uuid,
) -> anyhow::Result<PnlCurve> {
    let trips = strategy_trips(pool, user_id, strategy_id).await?;
    let pnls: Vec<f64> = trips.iter().map(|t| t.pnl).collect();
    let mut cum = 0.0;
    let mut peak = 0.0_f64;
    let points = trips
        .iter()
        .map(|t| {
            cum += t.pnl;
            peak = peak.max(cum);
            PnlPoint { ts: t.closed_ts, cum_pnl: cum }
        })
        .collect();
    Ok(PnlCurve {
        points,
        total_pnl: cum,
        peak_pnl: peak,
        current_drawdown: traderview_core::live_vs_backtest::realized_drawdown(&pnls),
        trips: trips.len(),
    })
}

pub async fn fills_for_strategy(
    pool: &PgPool,
    user_id: Uuid,
    strategy_id: Uuid,
) -> anyhow::Result<Vec<StrategyFill>> {
    Ok(sqlx::query_as::<_, StrategyFill>(
        "SELECT o.symbol, o.side, f.fill_qty, f.fill_price, f.commission, f.filled_at
           FROM algo_fills f
           JOIN algo_orders o ON o.id = f.order_id
           JOIN algo_strategies s ON s.id = o.strategy_id
          WHERE o.strategy_id = $1 AND s.user_id = $2
          ORDER BY f.filled_at",
    )
    .bind(strategy_id)
    .bind(user_id)
    .fetch_all(pool)
    .await?)
}

/// Live-vs-backtest divergence for one strategy: live fills FIFO into
/// closed round trips, tested against the LATEST persisted backtest.
/// None when no backtest exists (nothing to test against). Shared by
/// the on-demand route and the background drift watch — one code path.
#[derive(Debug, Clone, Serialize)]
pub struct LiveDivergence {
    pub report: traderview_core::live_vs_backtest::DivergenceReport,
    /// Trade-quality stats over the live trips (chronological).
    pub stats: Option<traderview_core::live_vs_backtest::TripStats>,
    /// Hold-duration discipline check over the live trips.
    pub hold: Option<traderview_core::live_vs_backtest::HoldStats>,
    pub backtest_at: DateTime<Utc>,
    pub backtest_symbol: String,
}

pub async fn live_divergence(
    pool: &PgPool,
    user_id: Uuid,
    strategy_id: Uuid,
) -> anyhow::Result<Option<LiveDivergence>> {
    use rust_decimal::prelude::ToPrimitive;
    let backtests = list_backtests(pool, user_id, strategy_id, 1).await?;
    let Some(bt) = backtests.first() else {
        return Ok(None);
    };
    let trips = strategy_trips(pool, user_id, strategy_id).await?;
    let pnls: Vec<f64> = trips.iter().map(|t| t.pnl).collect();
    let holds: Vec<(f64, i64)> = trips
        .iter()
        .map(|t| (t.pnl, t.closed_ts - t.opened_ts))
        .collect();
    let report = traderview_core::live_vs_backtest::compare(
        &pnls,
        traderview_core::live_vs_backtest::Expectation {
            win_rate: bt.win_rate.to_f64().unwrap_or(0.0),
            profit_factor: bt.profit_factor.to_f64().unwrap_or(0.0),
        },
    );
    Ok(Some(LiveDivergence {
        report,
        stats: traderview_core::live_vs_backtest::trip_stats(&pnls),
        hold: traderview_core::live_vs_backtest::hold_stats(&holds),
        backtest_at: bt.created_at,
        backtest_symbol: bt.symbol.clone(),
    }))
}

/// (id, user_id, name) of every live strategy — the drift-watch sweep
/// list. Soft-deleted strategies excluded.
pub async fn all_active_strategy_ids(
    pool: &PgPool,
) -> anyhow::Result<Vec<(Uuid, Uuid, String, Option<String>)>> {
    Ok(sqlx::query_as(
        "SELECT id, user_id, name, notes FROM algo_strategies WHERE deleted_at IS NULL",
    )
    .fetch_all(pool)
    .await?)
}

/// Epoch seconds of the most recent LOSING closed round trip, from
/// the strategy's fill record — the loss-cooldown gate's clock.
pub async fn last_losing_trip_ts(
    pool: &PgPool,
    user_id: Uuid,
    strategy_id: Uuid,
) -> anyhow::Result<Option<i64>> {
    use rust_decimal::prelude::ToPrimitive;
    let fills = fills_for_strategy(pool, user_id, strategy_id).await?;
    let mut by_symbol: std::collections::BTreeMap<String, Vec<traderview_core::live_vs_backtest::Fill>> =
        std::collections::BTreeMap::new();
    for f in fills {
        by_symbol.entry(f.symbol.clone()).or_default().push(
            traderview_core::live_vs_backtest::Fill {
                buy: f.side == "buy",
                qty: f.fill_qty.to_f64().unwrap_or(0.0),
                price: f.fill_price.to_f64().unwrap_or(0.0),
                commission: f.commission.to_f64().unwrap_or(0.0),
                ts: f.filled_at.timestamp(),
                flag: false,
            },
        );
    }
    let mut last: Option<i64> = None;
    for fills in by_symbol.values() {
        for trip in traderview_core::live_vs_backtest::round_trips(fills) {
            if trip.pnl < 0.0 && last.map_or(true, |l| trip.closed_ts > l) {
                last = Some(trip.closed_ts);
            }
        }
    }
    Ok(last)
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct StrategyRevision {
    pub id: i64,
    pub name: String,
    pub timeframe: String,
    pub side_mode: String,
    pub strategy_type: String,
    pub entry_rules: Json,
    pub exit_rules: Json,
    pub sizing: Json,
    pub risk_gates: Json,
    pub broker_mode: String,
    pub replaced_at: DateTime<Utc>,
}

/// Prior configs newest-first, ownership-checked. None = not yours.
pub async fn list_revisions(
    pool: &PgPool,
    user_id: Uuid,
    strategy_id: Uuid,
    limit: i64,
) -> anyhow::Result<Option<Vec<StrategyRevision>>> {
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
    Ok(Some(
        sqlx::query_as::<_, StrategyRevision>(
            "SELECT id, name, timeframe, side_mode, strategy_type,
                    entry_rules, exit_rules, sizing, risk_gates, broker_mode, replaced_at
               FROM algo_strategy_revisions
              WHERE strategy_id = $1
              ORDER BY replaced_at DESC
              LIMIT $2",
        )
        .bind(strategy_id)
        .bind(limit.clamp(1, 200))
        .fetch_all(pool)
        .await?,
    ))
}

/// Single revision row, ownership-checked through the strategy join.
pub async fn get_revision(
    pool: &PgPool,
    user_id: Uuid,
    strategy_id: Uuid,
    revision_id: i64,
) -> anyhow::Result<Option<StrategyRevision>> {
    Ok(sqlx::query_as::<_, StrategyRevision>(
        "SELECT r.id, r.name, r.timeframe, r.side_mode, r.strategy_type,
                r.entry_rules, r.exit_rules, r.sizing, r.risk_gates, r.broker_mode, r.replaced_at
           FROM algo_strategy_revisions r
           JOIN algo_strategies s ON s.id = r.strategy_id
          WHERE r.id = $1 AND r.strategy_id = $2 AND s.user_id = $3",
    )
    .bind(revision_id)
    .bind(strategy_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?)
}

/// One audit row per risk-gate fire. Non-fatal at call sites: the
/// audit must never break the tick loop.
pub async fn record_gate_fire(
    pool: &PgPool,
    strategy_id: Uuid,
    gate: &str,
    detail: &str,
) -> anyhow::Result<()> {
    sqlx::query("INSERT INTO algo_gate_fires (strategy_id, gate, detail) VALUES ($1, $2, $3)")
        .bind(strategy_id)
        .bind(gate)
        .bind(detail)
        .execute(pool)
        .await?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct GateFireCount {
    pub gate: String,
    pub fires: i64,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct GateFireRow {
    pub gate: String,
    pub detail: String,
    pub fired_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GateFireSummary {
    pub window_days: i64,
    pub counts: Vec<GateFireCount>,
    pub recent: Vec<GateFireRow>,
}

/// Per-gate fire counts over the trailing window + the latest rows.
pub async fn gate_fire_summary(
    pool: &PgPool,
    user_id: Uuid,
    strategy_id: Uuid,
    window_days: i64,
) -> anyhow::Result<Option<GateFireSummary>> {
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
    let counts: Vec<GateFireCount> = sqlx::query_as(
        "SELECT gate, count(*) AS fires FROM algo_gate_fires
          WHERE strategy_id = $1 AND fired_at >= now() - make_interval(days => $2::int)
          GROUP BY gate ORDER BY fires DESC",
    )
    .bind(strategy_id)
    .bind(window_days)
    .fetch_all(pool)
    .await?;
    let recent: Vec<GateFireRow> = sqlx::query_as(
        "SELECT gate, detail, fired_at FROM algo_gate_fires
          WHERE strategy_id = $1 ORDER BY fired_at DESC LIMIT 50",
    )
    .bind(strategy_id)
    .fetch_all(pool)
    .await?;
    Ok(Some(GateFireSummary {
        window_days,
        counts,
        recent,
    }))
}

/// Entry orders submitted today (UTC) — the overtrading-gate counter.
pub async fn entries_today(pool: &PgPool, strategy_id: Uuid) -> anyhow::Result<i64> {
    let (n,): (i64,) = sqlx::query_as(
        "SELECT count(*) FROM algo_orders
          WHERE strategy_id = $1 AND kind = 'entry'
            AND submitted_at >= date_trunc('day', now() AT TIME ZONE 'UTC')",
    )
    .bind(strategy_id)
    .fetch_one(pool)
    .await?;
    Ok(n)
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
    Ok(
        sqlx::query("DELETE FROM algo_backtests WHERE id = $1 AND user_id = $2")
            .bind(backtest_id)
            .bind(user_id)
            .execute(pool)
            .await?
            .rows_affected(),
    )
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
    let owned: Option<(Uuid,)> =
        sqlx::query_as("SELECT id FROM algo_strategies WHERE id = $1 AND user_id = $2")
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
