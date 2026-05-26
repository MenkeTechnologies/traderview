//! Per-user persisted chart drawings (trendlines / hlines / fibs / text).
//!
//! Storage is intentionally schema-light: `points` is a JSONB array shaped by
//! the frontend per `kind`. Server-side we only enforce ownership and the
//! `kind` enum.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct ChartDrawing {
    pub id: Uuid,
    pub user_id: Uuid,
    pub symbol: String,
    pub kind: String,
    pub points: serde_json::Value,
    pub label: Option<String>,
    pub color: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DrawingInput {
    pub kind: String,                    // trendline | hline | fib | text
    pub points: serde_json::Value,
    pub label: Option<String>,
    pub color: Option<String>,
}

pub async fn list_for_symbol(
    pool: &PgPool,
    user_id: Uuid,
    symbol: &str,
) -> anyhow::Result<Vec<ChartDrawing>> {
    let rows: Vec<ChartDrawing> = sqlx::query_as(
        "SELECT id, user_id, symbol, kind::text, points, label, color, created_at
           FROM chart_drawings
          WHERE user_id = $1 AND symbol = $2
          ORDER BY created_at ASC",
    )
    .bind(user_id)
    .bind(symbol)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn create(
    pool: &PgPool,
    user_id: Uuid,
    symbol: &str,
    d: &DrawingInput,
) -> anyhow::Result<ChartDrawing> {
    if !matches!(d.kind.as_str(), "trendline" | "hline" | "fib" | "text") {
        anyhow::bail!("unknown kind: {}", d.kind);
    }
    let row: ChartDrawing = sqlx::query_as(
        "INSERT INTO chart_drawings (user_id, symbol, kind, points, label, color)
              VALUES ($1, $2, $3::chart_drawing_kind_t, $4, $5, $6)
          RETURNING id, user_id, symbol, kind::text, points, label, color, created_at",
    )
    .bind(user_id)
    .bind(symbol)
    .bind(&d.kind)
    .bind(&d.points)
    .bind(d.label.as_deref())
    .bind(d.color.as_deref())
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn delete(pool: &PgPool, user_id: Uuid, id: Uuid) -> anyhow::Result<bool> {
    let r = sqlx::query("DELETE FROM chart_drawings WHERE id = $1 AND user_id = $2")
        .bind(id).bind(user_id)
        .execute(pool).await?;
    Ok(r.rows_affected() > 0)
}

pub async fn delete_all_for_symbol(
    pool: &PgPool,
    user_id: Uuid,
    symbol: &str,
) -> anyhow::Result<u64> {
    let r = sqlx::query("DELETE FROM chart_drawings WHERE user_id = $1 AND symbol = $2")
        .bind(user_id).bind(symbol)
        .execute(pool).await?;
    Ok(r.rows_affected())
}
