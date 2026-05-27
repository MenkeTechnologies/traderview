use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct AlertRule {
    pub id: Uuid,
    pub user_id: Uuid,
    pub symbol: String,
    pub trigger: String,
    pub threshold: Option<Decimal>,
    pub sound: String,
    pub voice_text: Option<String>,
    pub enabled: bool,
    pub triggered_at: Option<DateTime<Utc>>,
    pub trigger_count: i32,
    pub created_at: DateTime<Utc>,
}

pub async fn list(pool: &PgPool, user_id: Uuid) -> anyhow::Result<Vec<AlertRule>> {
    Ok(sqlx::query_as::<_, AlertRule>(
        "SELECT id, user_id, symbol, trigger::text, threshold, sound, voice_text,
                enabled, triggered_at, trigger_count, created_at
           FROM alert_rules WHERE user_id = $1 ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?)
}

pub async fn create(
    pool: &PgPool,
    user_id: Uuid,
    symbol: &str,
    trigger: &str,
    threshold: Option<Decimal>,
    sound: &str,
    voice_text: Option<&str>,
) -> anyhow::Result<AlertRule> {
    Ok(sqlx::query_as::<_, AlertRule>(
        "INSERT INTO alert_rules (user_id, symbol, trigger, threshold, sound, voice_text)
              VALUES ($1, $2, $3::alert_trigger_t, $4, $5, $6)
         RETURNING id, user_id, symbol, trigger::text, threshold, sound, voice_text,
                   enabled, triggered_at, trigger_count, created_at",
    )
    .bind(user_id)
    .bind(symbol.to_uppercase())
    .bind(trigger)
    .bind(threshold)
    .bind(sound)
    .bind(voice_text)
    .fetch_one(pool)
    .await?)
}

pub async fn delete(pool: &PgPool, user_id: Uuid, id: Uuid) -> anyhow::Result<bool> {
    let r = sqlx::query("DELETE FROM alert_rules WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(r.rows_affected() > 0)
}

pub async fn toggle(pool: &PgPool, user_id: Uuid, id: Uuid, enabled: bool) -> anyhow::Result<bool> {
    let r = sqlx::query("UPDATE alert_rules SET enabled = $3 WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .bind(enabled)
        .execute(pool)
        .await?;
    Ok(r.rows_affected() > 0)
}

pub async fn mark_fired(pool: &PgPool, id: Uuid) -> anyhow::Result<()> {
    sqlx::query(
        "UPDATE alert_rules SET triggered_at = now(), trigger_count = trigger_count + 1
          WHERE id = $1",
    )
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}
