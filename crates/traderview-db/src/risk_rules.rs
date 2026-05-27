//! Persistence for pre-trade risk-gate rules.
//!
//! The engine itself (`traderview_core::risk_gate`) is pure compute.
//! This module is the thin DB layer that loads / stores rule rows and
//! assembles the runtime context (account equity, today's P&L, etc).

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use traderview_core::risk_gate::{GateContext, RecentTrade, RiskRule};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredRule {
    pub id: Uuid,
    pub user_id: Uuid,
    pub account_id: Option<Uuid>,
    pub rule: RiskRule,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

/// List all rules belonging to a user, optionally filtered by account.
pub async fn list(pool: &PgPool, user_id: Uuid, account_id: Option<Uuid>)
    -> anyhow::Result<Vec<StoredRule>>
{
    let rows: Vec<(Uuid, Uuid, Option<Uuid>, serde_json::Value, bool, DateTime<Utc>)> = sqlx::query_as(
        "SELECT id, user_id, account_id, rule, enabled, created_at
           FROM risk_rules
          WHERE user_id = $1
            AND ($2::uuid IS NULL OR account_id IS NULL OR account_id = $2)
          ORDER BY created_at ASC",
    )
    .bind(user_id)
    .bind(account_id)
    .fetch_all(pool)
    .await?;
    let mut out = Vec::with_capacity(rows.len());
    for (id, user_id, account_id, rule_json, enabled, created_at) in rows {
        let rule: RiskRule = serde_json::from_value(rule_json)?;
        out.push(StoredRule { id, user_id, account_id, rule, enabled, created_at });
    }
    Ok(out)
}

pub async fn create(pool: &PgPool, user_id: Uuid, account_id: Option<Uuid>, rule: &RiskRule)
    -> anyhow::Result<Uuid>
{
    let id: (Uuid,) = sqlx::query_as(
        "INSERT INTO risk_rules (user_id, account_id, rule) VALUES ($1, $2, $3)
         RETURNING id",
    )
    .bind(user_id)
    .bind(account_id)
    .bind(serde_json::to_value(rule)?)
    .fetch_one(pool)
    .await?;
    Ok(id.0)
}

pub async fn delete(pool: &PgPool, user_id: Uuid, id: Uuid) -> anyhow::Result<u64> {
    let r = sqlx::query("DELETE FROM risk_rules WHERE id = $1 AND user_id = $2")
        .bind(id).bind(user_id)
        .execute(pool).await?;
    Ok(r.rows_affected())
}

pub async fn set_enabled(pool: &PgPool, user_id: Uuid, id: Uuid, enabled: bool)
    -> anyhow::Result<u64>
{
    let r = sqlx::query(
        "UPDATE risk_rules SET enabled = $3 WHERE id = $1 AND user_id = $2",
    )
    .bind(id).bind(user_id).bind(enabled)
    .execute(pool).await?;
    Ok(r.rows_affected())
}

/// Build the runtime `GateContext` for `user_id` + `account_id`.
///
/// Loads:
///   - account equity (starting cash + cumulative net P&L)
///   - today's realized P&L
///   - count of currently-open positions on this account
///   - all closed trades from today (for streak + cool-down rules)
pub async fn build_context(pool: &PgPool, account_id: Uuid)
    -> anyhow::Result<GateContext>
{
    use chrono::Utc;
    let today_start = Utc::now().date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();

    let equity: Option<(Decimal,)> = sqlx::query_as(
        "SELECT COALESCE(starting_cash, 0)::numeric
              + COALESCE((SELECT SUM(net_pnl)
                            FROM trades
                           WHERE account_id = $1 AND net_pnl IS NOT NULL), 0)::numeric
           FROM accounts WHERE id = $1",
    )
    .bind(account_id)
    .fetch_optional(pool)
    .await?;
    let account_equity = equity.map(|x| x.0).unwrap_or(Decimal::ZERO);

    let today_pnl: Option<(Option<Decimal>,)> = sqlx::query_as(
        "SELECT SUM(net_pnl) FROM trades
          WHERE account_id = $1 AND closed_at >= $2 AND net_pnl IS NOT NULL",
    )
    .bind(account_id).bind(today_start)
    .fetch_optional(pool).await?;
    let today_realized_pnl = today_pnl.and_then(|x| x.0).unwrap_or(Decimal::ZERO);

    let open: Option<(i64,)> = sqlx::query_as(
        "SELECT COUNT(*) FROM trades WHERE account_id = $1 AND status = 'open'",
    )
    .bind(account_id).fetch_optional(pool).await?;
    let open_position_count = open.map(|x| x.0 as usize).unwrap_or(0);

    let today_trades: Vec<(DateTime<Utc>, Decimal)> = sqlx::query_as(
        "SELECT closed_at, net_pnl FROM trades
          WHERE account_id = $1 AND closed_at >= $2 AND net_pnl IS NOT NULL
          ORDER BY closed_at ASC",
    )
    .bind(account_id).bind(today_start)
    .fetch_all(pool).await?;
    let today_closed_trades = today_trades.into_iter()
        .map(|(closed_at, net_pnl)| RecentTrade { closed_at, net_pnl })
        .collect();

    Ok(GateContext {
        account_equity,
        today_realized_pnl,
        open_position_count,
        today_closed_trades,
    })
}
