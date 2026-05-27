//! Per-user custom dashboards.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Dashboard {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub layout: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DashboardInput {
    pub name: String,
    pub layout: Option<Value>,
}

pub async fn list_for_user(pool: &PgPool, user_id: Uuid) -> anyhow::Result<Vec<Dashboard>> {
    let rows: Vec<Dashboard> = sqlx::query_as(
        "SELECT id, user_id, name, layout, created_at, updated_at
           FROM dashboards WHERE user_id = $1 ORDER BY updated_at DESC",
    ).bind(user_id).fetch_all(pool).await?;
    Ok(rows)
}

pub async fn get(pool: &PgPool, user_id: Uuid, id: Uuid) -> anyhow::Result<Option<Dashboard>> {
    let row: Option<Dashboard> = sqlx::query_as(
        "SELECT id, user_id, name, layout, created_at, updated_at
           FROM dashboards WHERE user_id = $1 AND id = $2",
    ).bind(user_id).bind(id).fetch_optional(pool).await?;
    Ok(row)
}

pub async fn create(pool: &PgPool, user_id: Uuid, dto: &DashboardInput) -> anyhow::Result<Dashboard> {
    let layout = dto.layout.clone().unwrap_or_else(|| Value::Array(vec![]));
    let row: Dashboard = sqlx::query_as(
        "INSERT INTO dashboards (user_id, name, layout)
              VALUES ($1, $2, $3)
          RETURNING id, user_id, name, layout, created_at, updated_at",
    ).bind(user_id).bind(&dto.name).bind(&layout).fetch_one(pool).await?;
    Ok(row)
}

pub async fn update(pool: &PgPool, user_id: Uuid, id: Uuid, dto: &DashboardInput) -> anyhow::Result<Option<Dashboard>> {
    let layout = dto.layout.clone().unwrap_or_else(|| Value::Array(vec![]));
    let row: Option<Dashboard> = sqlx::query_as(
        "UPDATE dashboards SET name = $3, layout = $4, updated_at = now()
          WHERE user_id = $1 AND id = $2
          RETURNING id, user_id, name, layout, created_at, updated_at",
    ).bind(user_id).bind(id).bind(&dto.name).bind(&layout).fetch_optional(pool).await?;
    Ok(row)
}

pub async fn delete(pool: &PgPool, user_id: Uuid, id: Uuid) -> anyhow::Result<bool> {
    let r = sqlx::query("DELETE FROM dashboards WHERE user_id = $1 AND id = $2")
        .bind(user_id).bind(id).execute(pool).await?;
    Ok(r.rows_affected() > 0)
}
