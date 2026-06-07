use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Watchlist {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub position: i32,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
}

pub async fn list(pool: &PgPool, user_id: Uuid) -> anyhow::Result<Vec<Watchlist>> {
    Ok(sqlx::query_as::<_, Watchlist>(
        "SELECT id, user_id, name, position, is_default, created_at
           FROM watchlists WHERE user_id = $1 ORDER BY position, created_at",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?)
}

pub async fn create(pool: &PgPool, user_id: Uuid, name: &str) -> anyhow::Result<Watchlist> {
    Ok(sqlx::query_as::<_, Watchlist>(
        "INSERT INTO watchlists (user_id, name, position)
              VALUES ($1, $2, COALESCE((SELECT MAX(position) + 1 FROM watchlists WHERE user_id = $1), 0))
         RETURNING id, user_id, name, position, is_default, created_at",
    )
    .bind(user_id)
    .bind(name)
    .fetch_one(pool)
    .await?)
}

pub async fn rename(pool: &PgPool, user_id: Uuid, id: Uuid, name: &str) -> anyhow::Result<bool> {
    let r = sqlx::query("UPDATE watchlists SET name = $3 WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .bind(name)
        .execute(pool)
        .await?;
    Ok(r.rows_affected() > 0)
}

pub async fn delete(pool: &PgPool, user_id: Uuid, id: Uuid) -> anyhow::Result<bool> {
    let r = sqlx::query("DELETE FROM watchlists WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(r.rows_affected() > 0)
}

pub async fn ensure_default(pool: &PgPool, user_id: Uuid) -> anyhow::Result<Watchlist> {
    if let Some(w) = list(pool, user_id).await?.into_iter().next() {
        return Ok(w);
    }
    let w = create(pool, user_id, "Main").await?;
    sqlx::query("UPDATE watchlists SET is_default = TRUE WHERE id = $1")
        .bind(w.id)
        .execute(pool)
        .await?;
    Ok(w)
}

// ---- symbols -----------------------------------------------------------

pub async fn symbols(pool: &PgPool, watchlist_id: Uuid) -> anyhow::Result<Vec<String>> {
    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT symbol FROM watchlist_symbols WHERE watchlist_id = $1 ORDER BY position, added_at",
    )
    .bind(watchlist_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|(s,)| s).collect())
}

/// Union of every distinct symbol across every user's watchlists.
/// Used by the live-tick bridge so subscribed symbols flow into the
/// WS aggregator without going through the candidates/scanner path.
pub async fn all_distinct_symbols(pool: &PgPool) -> anyhow::Result<Vec<String>> {
    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT DISTINCT symbol FROM watchlist_symbols ORDER BY symbol",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|(s,)| s).collect())
}

pub async fn add_symbol(pool: &PgPool, watchlist_id: Uuid, symbol: &str) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO watchlist_symbols (watchlist_id, symbol, position)
              VALUES ($1, $2,
                COALESCE((SELECT MAX(position) + 1 FROM watchlist_symbols WHERE watchlist_id = $1), 0))
         ON CONFLICT DO NOTHING",
    )
    .bind(watchlist_id).bind(symbol.to_uppercase())
    .execute(pool).await?;
    Ok(())
}

pub async fn remove_symbol(
    pool: &PgPool,
    watchlist_id: Uuid,
    symbol: &str,
) -> anyhow::Result<bool> {
    let r = sqlx::query("DELETE FROM watchlist_symbols WHERE watchlist_id = $1 AND symbol = $2")
        .bind(watchlist_id)
        .bind(symbol.to_uppercase())
        .execute(pool)
        .await?;
    Ok(r.rows_affected() > 0)
}

pub async fn ensure_owner(
    pool: &PgPool,
    user_id: Uuid,
    watchlist_id: Uuid,
) -> anyhow::Result<bool> {
    let row: Option<(Uuid,)> = sqlx::query_as("SELECT user_id FROM watchlists WHERE id = $1")
        .bind(watchlist_id)
        .fetch_optional(pool)
        .await?;
    Ok(matches!(row, Some((u,)) if u == user_id))
}
