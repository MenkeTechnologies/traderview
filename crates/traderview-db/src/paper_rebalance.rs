//! Paper-account rebalancer.
//!
//! Per-user named target weight sets (e.g. "60-40", "Boglehead 3-fund",
//! "magic-formula top 20"). For each target set:
//!
//!   1. Loads current paper positions + cash.
//!   2. Calls `traderview_core::rebalance::compute` with the saved
//!      target weights → produces a Plan with per-symbol drift +
//!      suggested trade qty/value.
//!   3. Reports `max_drift_pct` so the UI can highlight "you're 8% over
//!      target on tech — rebalance recommended."
//!
//! When |max drift| ≤ `drift_threshold_pct`, the plan is reported but
//! flagged as "within tolerance — no action needed." Above threshold,
//! flagged as "drift exceeds tolerance — consider rebalancing."

use chrono::{DateTime, Utc};
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use traderview_core::rebalance::{compute, HoldingInput, Plan, TargetInput};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct PaperRebalanceTarget {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub targets: Value,
    pub cash_target_pct: f64,
    pub drift_threshold_pct: f64,
    pub max_trades: i32,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PaperRebalanceTargetInput {
    pub name: String,
    pub targets: Value,
    #[serde(default)]
    pub cash_target_pct: f64,
    #[serde(default = "default_threshold")]
    pub drift_threshold_pct: f64,
    #[serde(default = "default_max_trades")]
    pub max_trades: i32,
    #[serde(default)]
    pub notes: Option<String>,
}

fn default_threshold() -> f64 {
    5.0
}
fn default_max_trades() -> i32 {
    20
}

#[derive(Debug, Clone, Serialize)]
pub struct PaperRebalancePlan {
    pub target: PaperRebalanceTarget,
    pub plan: Plan,
    pub max_drift_pct: f64,
    pub above_threshold: bool,
}

// ─── Pure compute ──────────────────────────────────────────────────────────

/// Largest absolute drift across all rows in the plan. Used by the
/// route layer to decide whether the rebalance is "above tolerance."
pub fn max_drift_pct(plan: &Plan) -> f64 {
    plan.rows
        .iter()
        .map(|r| r.drift_pct.abs())
        .fold(0.0_f64, f64::max)
}

/// Parse the targets JSON into the `TargetInput` shape that
/// `compute()` expects. Returns empty vec on malformed input.
pub fn parse_targets(json: &Value) -> Vec<TargetInput> {
    let Some(map) = json.as_object() else {
        return Vec::new();
    };
    map.iter()
        .filter_map(|(symbol, v)| {
            let weight = v.as_f64()?;
            Some(TargetInput {
                symbol: symbol.clone(),
                weight,
                price: None,
            })
        })
        .collect()
}

// ─── Repository ────────────────────────────────────────────────────────────

/// (id, user_id, name) of every target — the drift-watch sweep list.
pub async fn all_target_ids(pool: &PgPool) -> anyhow::Result<Vec<(Uuid, Uuid, String)>> {
    Ok(sqlx::query_as(
        "SELECT id, user_id, name FROM paper_rebalance_targets",
    )
    .fetch_all(pool)
    .await?)
}

pub async fn list(pool: &PgPool, user_id: Uuid) -> anyhow::Result<Vec<PaperRebalanceTarget>> {
    Ok(sqlx::query_as::<_, PaperRebalanceTarget>(
        "SELECT id, user_id, name, targets, cash_target_pct, drift_threshold_pct,
                max_trades, notes, created_at, updated_at
           FROM paper_rebalance_targets
          WHERE user_id = $1
          ORDER BY name",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?)
}

pub async fn upsert(
    pool: &PgPool,
    user_id: Uuid,
    dto: &PaperRebalanceTargetInput,
) -> anyhow::Result<PaperRebalanceTarget> {
    Ok(sqlx::query_as::<_, PaperRebalanceTarget>(
        "INSERT INTO paper_rebalance_targets
            (user_id, name, targets, cash_target_pct, drift_threshold_pct, max_trades, notes)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         ON CONFLICT (user_id, name) DO UPDATE SET
            targets             = EXCLUDED.targets,
            cash_target_pct     = EXCLUDED.cash_target_pct,
            drift_threshold_pct = EXCLUDED.drift_threshold_pct,
            max_trades          = EXCLUDED.max_trades,
            notes               = EXCLUDED.notes,
            updated_at          = now()
         RETURNING id, user_id, name, targets, cash_target_pct, drift_threshold_pct,
                   max_trades, notes, created_at, updated_at",
    )
    .bind(user_id)
    .bind(&dto.name)
    .bind(&dto.targets)
    .bind(dto.cash_target_pct)
    .bind(dto.drift_threshold_pct)
    .bind(dto.max_trades)
    .bind(&dto.notes)
    .fetch_one(pool)
    .await?)
}

pub async fn delete(pool: &PgPool, user_id: Uuid, id: Uuid) -> anyhow::Result<bool> {
    let r = sqlx::query("DELETE FROM paper_rebalance_targets WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(r.rows_affected() > 0)
}

#[derive(Debug, Clone, Serialize)]
pub struct ExecutedTrade {
    pub symbol: String,
    pub side: String,
    pub qty: i64,
    pub status: String,
    pub fill_price: Option<rust_decimal::Decimal>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExecutionResult {
    pub executed: Vec<ExecutedTrade>,
    pub skipped: Vec<String>,
    /// The plan that was executed — re-planned fresh at execute time,
    /// never a stale snapshot from the preview the user looked at.
    pub plan_max_drift_pct: f64,
}

/// Execute the rebalance: re-plan FRESH (prices move between preview
/// and click), then submit the plan's trades as paper market orders —
/// SELLS FIRST so the freed cash funds the buys. Each order goes
/// through paper::submit (friction, fills, journaling); a failed leg
/// is recorded and execution continues — a rebalance is N independent
/// orders, not an atomic spread.
pub async fn execute(
    pool: &PgPool,
    user_id: Uuid,
    target_id: Uuid,
) -> anyhow::Result<Option<ExecutionResult>> {
    let Some(p) = plan(pool, user_id, target_id).await? else {
        return Ok(None);
    };
    let account = crate::paper::ensure_default(pool, user_id).await?;
    // Sells first: ordering is the difference between "rebalanced" and
    // "insufficient cash" on accounts that are near fully invested.
    let mut trades: Vec<_> = p.plan.trades.clone();
    trades.sort_by_key(|t| if t.trade_qty < 0 { 0 } else { 1 });
    let mut executed = Vec::new();
    let mut skipped = Vec::new();
    for row in trades {
        if row.trade_qty == 0 {
            continue;
        }
        let side = if row.trade_qty > 0 {
            traderview_core::Side::Buy
        } else {
            traderview_core::Side::Sell
        };
        let qty = rust_decimal::Decimal::from(row.trade_qty.abs());
        let req = crate::paper::OrderRequest {
            symbol: row.symbol.clone(),
            side,
            qty,
            order_type: "market".into(),
            limit_price: None,
            stop_price: None,
            trail_value: None,
            trail_is_pct: None,
            time_in_force: None,
            expire_at: None,
            plan_note: None,
        };
        match crate::paper::submit(pool, user_id, account.id, req).await {
            Ok(o) => executed.push(ExecutedTrade {
                symbol: row.symbol.clone(),
                side: o.side.clone(),
                qty: row.trade_qty.abs(),
                status: o.status.clone(),
                fill_price: o.filled_price,
            }),
            Err(e) => skipped.push(format!("{}: {e}", row.symbol)),
        }
    }
    Ok(Some(ExecutionResult {
        executed,
        skipped,
        plan_max_drift_pct: p.max_drift_pct,
    }))
}

/// Build the rebalance plan for one named target set against the user's
/// default paper account. Snapshots positions + cash at call time.
pub async fn plan(
    pool: &PgPool,
    user_id: Uuid,
    target_id: Uuid,
) -> anyhow::Result<Option<PaperRebalancePlan>> {
    let target: Option<PaperRebalanceTarget> = sqlx::query_as(
        "SELECT id, user_id, name, targets, cash_target_pct, drift_threshold_pct,
                max_trades, notes, created_at, updated_at
           FROM paper_rebalance_targets WHERE id = $1 AND user_id = $2",
    )
    .bind(target_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;
    let Some(target) = target else {
        return Ok(None);
    };
    let account = crate::paper::ensure_default(pool, user_id).await?;
    let positions = crate::paper::positions(pool, account.id).await?;
    let mut holdings: Vec<HoldingInput> = Vec::new();
    for p in &positions {
        let qty = p.qty.to_f64().unwrap_or(0.0);
        if qty == 0.0 {
            continue;
        }
        let price = match crate::market_data::quote(pool, &p.symbol).await {
            Ok(q) => q.price,
            Err(_) => p.avg_price.to_f64().unwrap_or(0.0),
        };
        holdings.push(HoldingInput {
            symbol: p.symbol.clone(),
            qty,
            price,
        });
    }
    let cash = account.cash.to_f64().unwrap_or(0.0);
    let targets = parse_targets(&target.targets);
    let plan = compute(&holdings, &targets, cash, target.max_trades as usize);
    let max_drift = max_drift_pct(&plan);
    let above = max_drift > target.drift_threshold_pct;
    Ok(Some(PaperRebalancePlan {
        target,
        plan,
        max_drift_pct: max_drift,
        above_threshold: above,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_targets_round_trips() {
        let json = json!({"AAPL": 0.05, "MSFT": 0.05, "GOOG": 0.10});
        let parsed = parse_targets(&json);
        assert_eq!(parsed.len(), 3);
        let aapl = parsed.iter().find(|t| t.symbol == "AAPL").unwrap();
        assert!((aapl.weight - 0.05).abs() < 1e-9);
    }

    #[test]
    fn parse_targets_skips_non_numeric_values() {
        let json = json!({"AAPL": 0.05, "MSFT": "five percent", "GOOG": null});
        let parsed = parse_targets(&json);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].symbol, "AAPL");
    }

    #[test]
    fn parse_targets_empty_for_non_object() {
        assert!(parse_targets(&json!([])).is_empty());
        assert!(parse_targets(&json!("string")).is_empty());
        assert!(parse_targets(&json!(null)).is_empty());
    }

    fn make_plan(drifts: &[f64]) -> Plan {
        Plan {
            total_value: 100_000.0,
            cash_current: 0.0,
            cash_target: 0.0,
            rows: drifts
                .iter()
                .enumerate()
                .map(|(i, &d)| traderview_core::rebalance::PlanRow {
                    symbol: format!("S{i}"),
                    current_qty: 0.0,
                    current_value: 0.0,
                    current_pct: 0.0,
                    target_pct: 0.0,
                    drift_pct: d,
                    price: 100.0,
                    target_value: 0.0,
                    target_qty: 0,
                    trade_qty: 0,
                    trade_value: 0.0,
                    side: "hold",
                })
                .collect(),
            trades: Vec::new(),
            trade_count: 0,
            total_trade_value: 0.0,
            warnings: Vec::new(),
        }
    }

    #[test]
    fn max_drift_handles_positive_and_negative() {
        let plan = make_plan(&[3.0, -7.5, 1.0, 5.0]);
        assert_eq!(max_drift_pct(&plan), 7.5);
    }

    #[test]
    fn max_drift_zero_on_empty_plan() {
        let plan = make_plan(&[]);
        assert_eq!(max_drift_pct(&plan), 0.0);
    }
}
