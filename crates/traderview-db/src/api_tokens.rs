//! Personal Access Tokens — DB layer.
//!
//! Argon2 hashing lives in `traderview-web` (so this crate doesn't need
//! argon2 as a dep). This module is pure storage: insert, list, find-by-
//! prefix, revoke, bump-usage.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct ApiToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub prefix: String,
    pub hash: String,
    pub scopes: Vec<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub use_count: i64,
    pub rate_limit_per_min: i32,
    pub created_at: DateTime<Utc>,
}

/// Public-safe view (no hash, no secret).
#[derive(Debug, Clone, Serialize)]
pub struct ApiTokenSummary {
    pub id: Uuid,
    pub name: String,
    pub prefix: String,
    pub scopes: Vec<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub use_count: i64,
    pub rate_limit_per_min: i32,
    pub created_at: DateTime<Utc>,
}

impl From<ApiToken> for ApiTokenSummary {
    fn from(t: ApiToken) -> Self {
        Self {
            id: t.id,
            name: t.name,
            prefix: t.prefix,
            scopes: t.scopes,
            expires_at: t.expires_at,
            revoked_at: t.revoked_at,
            last_used_at: t.last_used_at,
            use_count: t.use_count,
            rate_limit_per_min: t.rate_limit_per_min,
            created_at: t.created_at,
        }
    }
}

pub struct NewToken<'a> {
    pub user_id: Uuid,
    pub name: &'a str,
    pub prefix: &'a str,
    pub hash: &'a str,
    pub scopes: &'a [String],
    pub expires_at: Option<DateTime<Utc>>,
    pub rate_limit_per_min: Option<i32>,
}

pub async fn insert(pool: &PgPool, t: NewToken<'_>) -> anyhow::Result<ApiToken> {
    let row: ApiToken = sqlx::query_as(
        "INSERT INTO api_tokens (user_id, name, prefix, hash, scopes, expires_at, rate_limit_per_min)
              VALUES ($1, $2, $3, $4, $5, $6, COALESCE($7, 60))
          RETURNING id, user_id, name, prefix, hash, scopes, expires_at, revoked_at,
                    last_used_at, use_count, rate_limit_per_min, created_at",
    )
    .bind(t.user_id).bind(t.name).bind(t.prefix).bind(t.hash)
    .bind(t.scopes).bind(t.expires_at).bind(t.rate_limit_per_min)
    .fetch_one(pool).await?;
    Ok(row)
}

pub async fn set_rate_limit(
    pool: &PgPool,
    user_id: Uuid,
    id: Uuid,
    new_limit: i32,
) -> anyhow::Result<bool> {
    if !(1..=10_000).contains(&new_limit) {
        anyhow::bail!("rate_limit_per_min must be in 1..=10000");
    }
    let r = sqlx::query(
        "UPDATE api_tokens SET rate_limit_per_min = $3
          WHERE id = $1 AND user_id = $2",
    )
    .bind(id)
    .bind(user_id)
    .bind(new_limit)
    .execute(pool)
    .await?;
    Ok(r.rows_affected() > 0)
}

pub async fn list_for_user(pool: &PgPool, user_id: Uuid) -> anyhow::Result<Vec<ApiTokenSummary>> {
    let rows: Vec<ApiToken> = sqlx::query_as(
        "SELECT id, user_id, name, prefix, hash, scopes, expires_at, revoked_at,
                last_used_at, use_count, rate_limit_per_min, created_at
           FROM api_tokens
          WHERE user_id = $1
          ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

/// Look up a token by its prefix. Returns the full row (including hash) so
/// the caller can verify the secret with their preferred hasher. Filters out
/// revoked / expired rows server-side.
pub async fn find_active_by_prefix(
    pool: &PgPool,
    prefix: &str,
) -> anyhow::Result<Option<ApiToken>> {
    let row: Option<ApiToken> = sqlx::query_as(
        "SELECT id, user_id, name, prefix, hash, scopes, expires_at, revoked_at,
                last_used_at, use_count, rate_limit_per_min, created_at
           FROM api_tokens
          WHERE prefix = $1
            AND revoked_at IS NULL
            AND (expires_at IS NULL OR expires_at > now())",
    )
    .bind(prefix)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn bump_usage(pool: &PgPool, id: Uuid) -> anyhow::Result<()> {
    sqlx::query(
        "UPDATE api_tokens SET last_used_at = now(), use_count = use_count + 1 WHERE id = $1",
    )
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn revoke(pool: &PgPool, user_id: Uuid, id: Uuid) -> anyhow::Result<bool> {
    let r = sqlx::query(
        "UPDATE api_tokens SET revoked_at = now()
          WHERE id = $1 AND user_id = $2 AND revoked_at IS NULL",
    )
    .bind(id)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(r.rows_affected() > 0)
}
