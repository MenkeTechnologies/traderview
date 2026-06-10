//! Strategy-alert persistence + background evaluation.
//!
//! `evaluate_all()` runs every enabled rule, resolving metric data from cached
//! price_bars + live quotes, and fires on the false→true edge (transition).
//! Fan-out reuses the existing webhooks fan-out helper.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use std::collections::HashMap;
use traderview_core::strategy_alert::{
    collect_symbols, evaluate, LeafEval, Metric, MetricInput, Node,
};
use traderview_core::BarInterval;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct StrategyAlert {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub enabled: bool,
    pub ast: Value,
    pub webhook_ids: Vec<Uuid>,
    pub last_truth: Option<bool>,
    pub last_evaluated_at: Option<DateTime<Utc>>,
    pub last_fired_at: Option<DateTime<Utc>>,
    pub fire_count: i32,
    pub last_eval_error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StrategyAlertInput {
    pub name: String,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    pub ast: Value,
    #[serde(default)]
    pub webhook_ids: Vec<Uuid>,
}
fn default_enabled() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct StrategyAlertFire {
    pub id: Uuid,
    pub alert_id: Uuid,
    pub fired_at: DateTime<Utc>,
    pub snapshot: Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct EvalStats {
    pub evaluated: usize,
    pub fired: usize,
    pub errors: usize,
}

pub async fn list(pool: &PgPool, user_id: Uuid) -> anyhow::Result<Vec<StrategyAlert>> {
    Ok(sqlx::query_as(
        "SELECT id, user_id, name, enabled, ast, webhook_ids, last_truth,
                last_evaluated_at, last_fired_at, fire_count, last_eval_error,
                created_at, updated_at
           FROM strategy_alerts WHERE user_id = $1 ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?)
}

pub async fn create(
    pool: &PgPool,
    user_id: Uuid,
    dto: &StrategyAlertInput,
) -> anyhow::Result<StrategyAlert> {
    // Verify the ast parses as a Node so users get an immediate error rather
    // than a silent runtime failure later.
    let _node: Node = serde_json::from_value(dto.ast.clone())?;
    Ok(sqlx::query_as(
        "INSERT INTO strategy_alerts (user_id, name, enabled, ast, webhook_ids)
              VALUES ($1, $2, $3, $4, $5)
          RETURNING id, user_id, name, enabled, ast, webhook_ids, last_truth,
                    last_evaluated_at, last_fired_at, fire_count, last_eval_error,
                    created_at, updated_at",
    )
    .bind(user_id)
    .bind(&dto.name)
    .bind(dto.enabled)
    .bind(&dto.ast)
    .bind(&dto.webhook_ids)
    .fetch_one(pool)
    .await?)
}

pub async fn update(
    pool: &PgPool,
    user_id: Uuid,
    id: Uuid,
    dto: &StrategyAlertInput,
) -> anyhow::Result<Option<StrategyAlert>> {
    let _node: Node = serde_json::from_value(dto.ast.clone())?;
    Ok(sqlx::query_as(
        "UPDATE strategy_alerts SET
            name = $3, enabled = $4, ast = $5, webhook_ids = $6,
            updated_at = now()
          WHERE id = $1 AND user_id = $2
          RETURNING id, user_id, name, enabled, ast, webhook_ids, last_truth,
                    last_evaluated_at, last_fired_at, fire_count, last_eval_error,
                    created_at, updated_at",
    )
    .bind(id)
    .bind(user_id)
    .bind(&dto.name)
    .bind(dto.enabled)
    .bind(&dto.ast)
    .bind(&dto.webhook_ids)
    .fetch_optional(pool)
    .await?)
}

pub async fn delete(pool: &PgPool, user_id: Uuid, id: Uuid) -> anyhow::Result<bool> {
    Ok(
        sqlx::query("DELETE FROM strategy_alerts WHERE id = $1 AND user_id = $2")
            .bind(id)
            .bind(user_id)
            .execute(pool)
            .await?
            .rows_affected()
            > 0,
    )
}

pub async fn recent_fires(
    pool: &PgPool,
    user_id: Uuid,
    limit: i64,
) -> anyhow::Result<Vec<StrategyAlertFire>> {
    Ok(sqlx::query_as(
        "SELECT f.id, f.alert_id, f.fired_at, f.snapshot
           FROM strategy_alert_fires f
           JOIN strategy_alerts a ON a.id = f.alert_id
          WHERE a.user_id = $1
          ORDER BY f.fired_at DESC
          LIMIT $2",
    )
    .bind(user_id)
    .bind(limit)
    .fetch_all(pool)
    .await?)
}

pub async fn evaluate_all(pool: &PgPool) -> anyhow::Result<EvalStats> {
    let rows: Vec<StrategyAlert> = sqlx::query_as(
        "SELECT id, user_id, name, enabled, ast, webhook_ids, last_truth,
                last_evaluated_at, last_fired_at, fire_count, last_eval_error,
                created_at, updated_at
           FROM strategy_alerts WHERE enabled = TRUE",
    )
    .fetch_all(pool)
    .await?;
    evaluate_rows(pool, rows).await
}

/// Evaluate only the alerts owned by `user_id`. Used by the route
/// layer to gate POST /strategy-alerts/evaluate-now — without this,
/// any authed user could trigger evaluation of every other user's
/// alerts (which fire webhooks).
pub async fn evaluate_for_user(pool: &PgPool, user_id: Uuid) -> anyhow::Result<EvalStats> {
    let rows: Vec<StrategyAlert> = sqlx::query_as(
        "SELECT id, user_id, name, enabled, ast, webhook_ids, last_truth,
                last_evaluated_at, last_fired_at, fire_count, last_eval_error,
                created_at, updated_at
           FROM strategy_alerts WHERE enabled = TRUE AND user_id = $1",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    evaluate_rows(pool, rows).await
}

async fn evaluate_rows(pool: &PgPool, rows: Vec<StrategyAlert>) -> anyhow::Result<EvalStats> {

    let mut evaluated = 0;
    let mut fired = 0;
    let mut errors = 0;
    for r in rows {
        match evaluate_one(pool, &r).await {
            Ok(did_fire) => {
                evaluated += 1;
                if did_fire {
                    fired += 1;
                }
            }
            Err(e) => {
                errors += 1;
                let _ = sqlx::query(
                    "UPDATE strategy_alerts SET last_eval_error = $2, last_evaluated_at = now()
                      WHERE id = $1",
                )
                .bind(r.id)
                .bind(e.to_string())
                .execute(pool)
                .await;
            }
        }
    }
    Ok(EvalStats {
        evaluated,
        fired,
        errors,
    })
}

async fn evaluate_one(pool: &PgPool, row: &StrategyAlert) -> anyhow::Result<bool> {
    let node: Node = serde_json::from_value(row.ast.clone())?;
    // Pre-fetch every (symbol, max-history) needed by this rule.
    let mut wants: HashMap<String, u32> = HashMap::new();
    collect_symbols(&node, &mut wants);
    let mut metric_cache: HashMap<String, MetricInput> = HashMap::new();
    let now = Utc::now();
    for (sym, history) in &wants {
        let mut input = MetricInput {
            latest_price: None,
            closes: vec![],
        };
        // Live price for any leaf that uses Price/Quote.
        if let Ok(q) = crate::market_data::quote(pool, sym).await {
            input.latest_price = Some(q.price);
        }
        if *history > 0 {
            let from = now - Duration::days((*history as i64) * 2 + 30); // padding for non-trading days
            if let Ok(bars) = crate::prices::get_bars(pool, sym, BarInterval::D1, from, now).await {
                input.closes = bars.iter().map(|b| dec(b.close)).collect();
            }
        }
        metric_cache.insert(sym.clone(), input);
    }

    let mut trace: Vec<LeafEval> = Vec::new();
    let truth = evaluate(
        &node,
        &mut |sym: &str, _m: &Metric| {
            metric_cache.get(sym).cloned().unwrap_or(MetricInput {
                latest_price: None,
                closes: vec![],
            })
        },
        &mut trace,
    );

    let did_fire = matches!(row.last_truth, Some(false) | None) && truth;

    if did_fire {
        let snapshot = serde_json::json!({
            "leaves": trace,
            "ast": row.ast.clone(),
            "evaluated_at": now,
        });
        sqlx::query(
            "INSERT INTO strategy_alert_fires (alert_id, snapshot)
                  VALUES ($1, $2)",
        )
        .bind(row.id)
        .bind(&snapshot)
        .execute(pool)
        .await?;
        sqlx::query(
            "UPDATE strategy_alerts SET
                last_truth = TRUE, last_fired_at = now(), fire_count = fire_count + 1,
                last_evaluated_at = now(), last_eval_error = NULL
              WHERE id = $1",
        )
        .bind(row.id)
        .execute(pool)
        .await?;
        // Webhook fan-out (best-effort).
        if !row.webhook_ids.is_empty() {
            let payload = crate::webhooks::AlertPayload {
                kind: "strategy_alert".into(),
                title: row.name.clone(),
                message: format!("strategy '{}' fired", row.name),
                symbol: None,
                url: None,
                fired_at: now,
            };
            crate::webhooks::fan_out(pool, row.user_id, &row.webhook_ids, &payload).await;
        }
    } else {
        sqlx::query(
            "UPDATE strategy_alerts SET
                last_truth = $2, last_evaluated_at = now(), last_eval_error = NULL
              WHERE id = $1",
        )
        .bind(row.id)
        .bind(truth)
        .execute(pool)
        .await?;
    }
    Ok(did_fire)
}

fn dec(d: rust_decimal::Decimal) -> f64 {
    d.to_string().parse().unwrap_or(0.0)
}
