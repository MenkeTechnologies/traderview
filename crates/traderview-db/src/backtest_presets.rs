//! Backtest preset library + share-by-slug + fork.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use traderview_core::slug;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct BacktestPreset {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub preset: Value,
    pub is_public: bool,
    pub slug: String,
    pub origin_id: Option<Uuid>,
    pub fork_count: i32,
    pub run_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PresetInput {
    pub name: String,
    pub description: Option<String>,
    pub preset: Value,
    #[serde(default)] pub is_public: bool,
}

pub async fn list_mine(pool: &PgPool, user_id: Uuid) -> anyhow::Result<Vec<BacktestPreset>> {
    Ok(sqlx::query_as(
        "SELECT id, user_id, name, description, preset, is_public, slug, origin_id,
                fork_count, run_count, created_at, updated_at
           FROM backtest_presets WHERE user_id = $1 ORDER BY updated_at DESC",
    ).bind(user_id).fetch_all(pool).await?)
}

pub async fn list_public(pool: &PgPool, limit: i64) -> anyhow::Result<Vec<BacktestPreset>> {
    Ok(sqlx::query_as(
        "SELECT id, user_id, name, description, preset, is_public, slug, origin_id,
                fork_count, run_count, created_at, updated_at
           FROM backtest_presets
          WHERE is_public = TRUE
          ORDER BY fork_count DESC, updated_at DESC
          LIMIT $1",
    ).bind(limit).fetch_all(pool).await?)
}

pub async fn by_slug(pool: &PgPool, slug_str: &str) -> anyhow::Result<Option<BacktestPreset>> {
    Ok(sqlx::query_as(
        "SELECT id, user_id, name, description, preset, is_public, slug, origin_id,
                fork_count, run_count, created_at, updated_at
           FROM backtest_presets WHERE slug = $1",
    ).bind(slug_str).fetch_optional(pool).await?)
}

pub async fn create(pool: &PgPool, user_id: Uuid, dto: &PresetInput)
    -> anyhow::Result<BacktestPreset>
{
    let slug_str = generate_unique_slug(pool, &dto.name).await?;
    Ok(sqlx::query_as(
        "INSERT INTO backtest_presets
            (user_id, name, description, preset, is_public, slug)
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (user_id, name) DO UPDATE SET
            description = EXCLUDED.description,
            preset      = EXCLUDED.preset,
            is_public   = EXCLUDED.is_public,
            updated_at  = now()
         RETURNING id, user_id, name, description, preset, is_public, slug, origin_id,
                   fork_count, run_count, created_at, updated_at",
    )
    .bind(user_id).bind(&dto.name).bind(dto.description.as_deref())
    .bind(&dto.preset).bind(dto.is_public).bind(&slug_str)
    .fetch_one(pool).await?)
}

pub async fn update(pool: &PgPool, user_id: Uuid, id: Uuid, dto: &PresetInput)
    -> anyhow::Result<Option<BacktestPreset>>
{
    Ok(sqlx::query_as(
        "UPDATE backtest_presets SET
            name = $3, description = $4, preset = $5, is_public = $6,
            updated_at = now()
          WHERE id = $1 AND user_id = $2
          RETURNING id, user_id, name, description, preset, is_public, slug, origin_id,
                    fork_count, run_count, created_at, updated_at",
    )
    .bind(id).bind(user_id).bind(&dto.name).bind(dto.description.as_deref())
    .bind(&dto.preset).bind(dto.is_public)
    .fetch_optional(pool).await?)
}

pub async fn delete(pool: &PgPool, user_id: Uuid, id: Uuid) -> anyhow::Result<bool> {
    Ok(sqlx::query("DELETE FROM backtest_presets WHERE id = $1 AND user_id = $2")
        .bind(id).bind(user_id).execute(pool).await?.rows_affected() > 0)
}

pub async fn fork(pool: &PgPool, user_id: Uuid, source_slug: &str)
    -> anyhow::Result<BacktestPreset>
{
    let source = by_slug(pool, source_slug).await?
        .ok_or_else(|| anyhow::anyhow!("source preset not found"))?;
    if !source.is_public && source.user_id != user_id {
        anyhow::bail!("source preset is private");
    }
    // De-conflict name within forker's namespace.
    let mut name = format!("{} (fork)", source.name);
    let existing: Vec<(String,)> = sqlx::query_as(
        "SELECT name FROM backtest_presets WHERE user_id = $1 AND name LIKE $2",
    ).bind(user_id).bind(format!("{} (fork%", source.name)).fetch_all(pool).await.unwrap_or_default();
    if !existing.is_empty() {
        name = format!("{} (fork {})", source.name, existing.len() + 1);
    }
    let slug_str = generate_unique_slug(pool, &name).await?;
    let row: BacktestPreset = sqlx::query_as(
        "INSERT INTO backtest_presets
            (user_id, name, description, preset, is_public, slug, origin_id)
         VALUES ($1, $2, $3, $4, FALSE, $5, $6)
         RETURNING id, user_id, name, description, preset, is_public, slug, origin_id,
                   fork_count, run_count, created_at, updated_at",
    )
    .bind(user_id).bind(&name).bind(source.description.as_deref())
    .bind(&source.preset).bind(&slug_str).bind(source.id)
    .fetch_one(pool).await?;
    // Bump origin's fork_count (best-effort, no transaction — counter drift
    // is acceptable for a vanity stat).
    let _ = sqlx::query(
        "UPDATE backtest_presets SET fork_count = fork_count + 1 WHERE id = $1",
    ).bind(source.id).execute(pool).await;
    Ok(row)
}

pub async fn bump_run(pool: &PgPool, id: Uuid) -> anyhow::Result<()> {
    sqlx::query("UPDATE backtest_presets SET run_count = run_count + 1 WHERE id = $1")
        .bind(id).execute(pool).await?;
    Ok(())
}

async fn generate_unique_slug(pool: &PgPool, name: &str) -> anyhow::Result<String> {
    let base = slug::from_title(name);
    let base = if base.is_empty() { "preset".into() } else { base };
    // Try base first, then base-<random4> up to 5 times.
    for attempt in 0..6 {
        let candidate = if attempt == 0 { base.clone() }
                        else { format!("{}-{}", base, slug::random(4)) };
        let exists: Option<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM backtest_presets WHERE slug = $1",
        ).bind(&candidate).fetch_optional(pool).await?;
        if exists.is_none() { return Ok(candidate); }
    }
    Ok(format!("{}-{}", base, slug::random(8)))
}
