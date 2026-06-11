//! Verdict-change watcher CRUD + alert delivery.
//!
//! Users register a (symbol, optional verdict filter, webhook ids).
//! After every nightly compute, `check_and_fire` walks each watcher,
//! compares the new verdict against `last_verdict`, and dispatches a
//! webhook payload through the existing `webhooks::fire` plumbing on
//! a flip. `last_verdict` then bumps so the next run only fires on
//! subsequent flips, not on every same-verdict night.

use crate::webhooks::{fan_out, AlertPayload};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Watcher {
    pub id: Uuid,
    pub user_id: Uuid,
    pub symbol: String,
    pub fire_on: Option<Vec<String>>,
    pub webhook_ids: Vec<Uuid>,
    pub last_verdict: Option<String>,
    pub last_fired_at: Option<DateTime<Utc>>,
    pub fire_count: i32,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct WatcherInput {
    pub symbol: String,
    pub fire_on: Option<Vec<String>>,
    pub webhook_ids: Vec<Uuid>,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

pub async fn list(pool: &PgPool, user_id: Uuid) -> anyhow::Result<Vec<Watcher>> {
    let rows = sqlx::query_as::<_, Watcher>(
        "SELECT id, user_id, symbol, fire_on, webhook_ids, last_verdict,
                last_fired_at, fire_count, enabled, created_at, updated_at
           FROM stock_recommendation_watchers
          WHERE user_id = $1
          ORDER BY symbol ASC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Upsert one watcher per (user, symbol) — UNIQUE in the schema. Same
/// shape as the CRUD upsert pattern used by strategy_alerts so the UI
/// can call this with both "create" and "update" intent without
/// branching.
pub async fn upsert(
    pool: &PgPool,
    user_id: Uuid,
    input: WatcherInput,
) -> anyhow::Result<Watcher> {
    let symbol = input.symbol.trim().to_uppercase();
    let row = sqlx::query_as::<_, Watcher>(
        "INSERT INTO stock_recommendation_watchers
            (user_id, symbol, fire_on, webhook_ids, enabled)
         VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (user_id, symbol) DO UPDATE
            SET fire_on = EXCLUDED.fire_on,
                webhook_ids = EXCLUDED.webhook_ids,
                enabled = EXCLUDED.enabled,
                updated_at = now()
         RETURNING id, user_id, symbol, fire_on, webhook_ids, last_verdict,
                   last_fired_at, fire_count, enabled, created_at, updated_at",
    )
    .bind(user_id)
    .bind(symbol)
    .bind(input.fire_on)
    .bind(&input.webhook_ids)
    .bind(input.enabled)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn delete(pool: &PgPool, user_id: Uuid, id: Uuid) -> anyhow::Result<bool> {
    let r = sqlx::query(
        "DELETE FROM stock_recommendation_watchers WHERE id = $1 AND user_id = $2",
    )
    .bind(id)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(r.rows_affected() > 0)
}

/// After the nightly compute writes new rows to `stock_recommendations`,
/// walk every enabled watcher and fire webhooks for flips. Returns the
/// number of webhooks fired. Errors per-watcher are logged + skipped —
/// the cron must not abort because one watcher's webhook 500s.
pub async fn check_and_fire(pool: &PgPool) -> anyhow::Result<usize> {
    let watchers: Vec<Watcher> = sqlx::query_as(
        "SELECT id, user_id, symbol, fire_on, webhook_ids, last_verdict,
                last_fired_at, fire_count, enabled, created_at, updated_at
           FROM stock_recommendation_watchers
          WHERE enabled = TRUE",
    )
    .fetch_all(pool)
    .await?;
    let mut fired = 0usize;
    for w in watchers {
        let Some(current) = crate::stock_recommendation::latest_for_symbol(pool, &w.symbol)
            .await
            .unwrap_or(None)
        else {
            // No recommendation yet for this symbol — skip silently.
            continue;
        };
        let new_v = current.verdict.clone();
        let prev = w.last_verdict.as_deref();
        let flipped = match prev {
            None => true,                          // first-ever observation
            Some(p) => p != new_v.as_str(),        // verdict transition
        };
        if !flipped {
            continue;
        }
        // Apply the fire_on filter: only alert when the new verdict is
        // in the user's allowed set. Empty / None = fire on any change.
        if let Some(fire_on) = &w.fire_on {
            if !fire_on.is_empty() && !fire_on.iter().any(|v| v == &new_v) {
                // Still bump last_verdict so the next change against
                // this NEW baseline fires.
                let _ = bump_last_verdict(pool, w.id, &new_v).await;
                continue;
            }
        }
        // Dispatch via every bound webhook. fan_out filters by user_id
        // so a stale webhook id from another account silently skips.
        let payload = AlertPayload {
            title: format!("{} → {}", w.symbol, new_v.to_uppercase()),
            message: format!(
                "{} verdict flipped: {} → {} (score {:.0}, ★{}, target ${} / {:+.1}%)",
                w.symbol,
                prev.unwrap_or("(new)"),
                new_v,
                current.score,
                current.stars,
                current.target_price,
                current.upside_pct,
            ),
            symbol: Some(w.symbol.clone()),
            kind: "stock_rec_verdict_change".into(),
            url: None,
            fired_at: Utc::now(),
        };
        fan_out(pool, w.user_id, &w.webhook_ids, &payload).await;
        fired += w.webhook_ids.len();
        let _ = bump_last_verdict(pool, w.id, &new_v).await;
    }
    Ok(fired)
}

async fn bump_last_verdict(pool: &PgPool, id: Uuid, verdict: &str) -> anyhow::Result<()> {
    sqlx::query(
        "UPDATE stock_recommendation_watchers
            SET last_verdict = $2, last_fired_at = now(),
                fire_count = fire_count + 1, updated_at = now()
          WHERE id = $1",
    )
    .bind(id)
    .bind(verdict)
    .execute(pool)
    .await?;
    Ok(())
}
