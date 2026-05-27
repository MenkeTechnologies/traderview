//! Rebalance-target persistence + live snapshot.
//!
//! `snapshot_account` reads open positions for an account, fetches a quote
//! per distinct symbol (60s cache), and assembles the `HoldingInput` list
//! that `traderview_core::rebalance::compute` expects.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use std::collections::HashMap;
use traderview_core::rebalance::{compute, HoldingInput, Plan, TargetInput};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct RebalanceTarget {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub account_id: Option<Uuid>,
    pub targets: Value,
    pub max_trades: i32,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RebalanceTargetInput {
    pub name: String,
    pub account_id: Option<Uuid>,
    pub targets: Value,
    #[serde(default = "default_max")] pub max_trades: i32,
    pub notes: Option<String>,
}
fn default_max() -> i32 { 20 }

#[derive(Debug, Clone, Deserialize)]
pub struct RunBody {
    pub account_id: Uuid,
    pub targets: Vec<TargetInput>,
    #[serde(default)] pub cash: f64,
    #[serde(default = "default_max_usize")] pub max_trades: usize,
}
fn default_max_usize() -> usize { 20 }

#[derive(Debug, Clone, Serialize)]
pub struct PlanResponse {
    pub plan: Plan,
    pub snapshot: Vec<HoldingInput>,
    pub computed_at: DateTime<Utc>,
}

pub async fn list(pool: &PgPool, user_id: Uuid) -> anyhow::Result<Vec<RebalanceTarget>> {
    Ok(sqlx::query_as(
        "SELECT id, user_id, name, account_id, targets, max_trades, notes,
                created_at, updated_at
           FROM rebalance_targets WHERE user_id = $1 ORDER BY updated_at DESC",
    ).bind(user_id).fetch_all(pool).await?)
}

pub async fn create(pool: &PgPool, user_id: Uuid, dto: &RebalanceTargetInput)
    -> anyhow::Result<RebalanceTarget>
{
    let _v: Vec<TargetInput> = serde_json::from_value(dto.targets.clone())?;
    Ok(sqlx::query_as(
        "INSERT INTO rebalance_targets (user_id, name, account_id, targets, max_trades, notes)
              VALUES ($1, $2, $3, $4, $5, $6)
          ON CONFLICT (user_id, name) DO UPDATE SET
              account_id = EXCLUDED.account_id,
              targets    = EXCLUDED.targets,
              max_trades = EXCLUDED.max_trades,
              notes      = EXCLUDED.notes,
              updated_at = now()
          RETURNING id, user_id, name, account_id, targets, max_trades, notes,
                    created_at, updated_at",
    )
    .bind(user_id).bind(&dto.name).bind(dto.account_id)
    .bind(&dto.targets).bind(dto.max_trades).bind(dto.notes.as_deref())
    .fetch_one(pool).await?)
}

pub async fn delete(pool: &PgPool, user_id: Uuid, id: Uuid) -> anyhow::Result<bool> {
    Ok(sqlx::query("DELETE FROM rebalance_targets WHERE id = $1 AND user_id = $2")
        .bind(id).bind(user_id).execute(pool).await?.rows_affected() > 0)
}

/// Pull every open trade for an account, dedupe per-symbol qty, fetch
/// quotes, build a `HoldingInput` list.
pub async fn snapshot_account(pool: &PgPool, account_id: Uuid)
    -> anyhow::Result<Vec<HoldingInput>>
{
    let rows: Vec<(String, Decimal, Decimal, String)> = sqlx::query_as(
        "SELECT symbol, qty, multiplier, side::text FROM trades
          WHERE account_id = $1 AND status = 'open'",
    ).bind(account_id).fetch_all(pool).await?;

    // Sum signed qty per symbol (short = negative); use multiplier so options
    // count their full notional, not just contract count.
    let mut by_sym: HashMap<String, f64> = HashMap::new();
    for (symbol, qty, mult, side) in rows {
        let q = dec(qty) * dec(mult).max(1.0);
        let signed = if side == "short" { -q } else { q };
        *by_sym.entry(symbol).or_insert(0.0) += signed;
    }

    let mut out = Vec::with_capacity(by_sym.len());
    for (sym, qty) in by_sym {
        let price = match crate::market_data::quote(pool, &sym).await {
            Ok(q) => q.price,
            Err(_) => continue,
        };
        out.push(HoldingInput { symbol: sym, qty, price });
    }
    Ok(out)
}

pub async fn run(pool: &PgPool, body: &RunBody) -> anyhow::Result<PlanResponse> {
    let snap = snapshot_account(pool, body.account_id).await?;
    let plan = compute(&snap, &body.targets, body.cash, body.max_trades);
    Ok(PlanResponse {
        plan, snapshot: snap, computed_at: Utc::now(),
    })
}

fn dec(d: Decimal) -> f64 { d.to_string().parse().unwrap_or(0.0) }
