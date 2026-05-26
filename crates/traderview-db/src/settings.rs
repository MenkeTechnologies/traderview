use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use traderview_core::{FilterSet, UserSettings};
use uuid::Uuid;

pub async fn get(pool: &PgPool, user_id: Uuid) -> anyhow::Result<UserSettings> {
    let row: Option<Row> = sqlx::query_as(
        "SELECT user_id, default_account_id, base_currency, timezone, theme,
                starting_cash, dashboard_layout, updated_at
           FROM user_settings WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;
    if let Some(r) = row {
        Ok(r.into())
    } else {
        let r: Row = sqlx::query_as(
            "INSERT INTO user_settings (user_id) VALUES ($1)
             RETURNING user_id, default_account_id, base_currency, timezone, theme,
                       starting_cash, dashboard_layout, updated_at",
        )
        .bind(user_id)
        .fetch_one(pool)
        .await?;
        Ok(r.into())
    }
}

pub async fn upsert(pool: &PgPool, s: &UserSettings) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO user_settings
            (user_id, default_account_id, base_currency, timezone, theme,
             starting_cash, dashboard_layout, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, now())
         ON CONFLICT (user_id) DO UPDATE SET
            default_account_id = EXCLUDED.default_account_id,
            base_currency      = EXCLUDED.base_currency,
            timezone           = EXCLUDED.timezone,
            theme              = EXCLUDED.theme,
            starting_cash      = EXCLUDED.starting_cash,
            dashboard_layout   = EXCLUDED.dashboard_layout,
            updated_at         = now()",
    )
    .bind(s.user_id)
    .bind(s.default_account_id)
    .bind(&s.base_currency)
    .bind(&s.timezone)
    .bind(&s.theme)
    .bind(s.starting_cash)
    .bind(&s.dashboard_layout)
    .execute(pool)
    .await?;
    Ok(())
}

// ---------- filter sets -----------

pub async fn list_filters(pool: &PgPool, user_id: Uuid) -> anyhow::Result<Vec<FilterSet>> {
    let rows: Vec<FsRow> = sqlx::query_as(
        "SELECT id, user_id, name, payload, is_default, created_at
           FROM filter_sets WHERE user_id = $1 ORDER BY created_at",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

pub async fn save_filter(
    pool: &PgPool,
    user_id: Uuid,
    name: &str,
    payload: &serde_json::Value,
    is_default: bool,
) -> anyhow::Result<FilterSet> {
    let row: FsRow = sqlx::query_as(
        "INSERT INTO filter_sets (user_id, name, payload, is_default)
              VALUES ($1, $2, $3, $4)
         ON CONFLICT (user_id, name) DO UPDATE SET payload = $3, is_default = $4
         RETURNING id, user_id, name, payload, is_default, created_at",
    )
    .bind(user_id)
    .bind(name)
    .bind(payload)
    .bind(is_default)
    .fetch_one(pool)
    .await?;
    Ok(row.into())
}

pub async fn delete_filter(pool: &PgPool, user_id: Uuid, id: Uuid) -> anyhow::Result<bool> {
    let r = sqlx::query("DELETE FROM filter_sets WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(r.rows_affected() > 0)
}

#[derive(sqlx::FromRow)]
struct Row {
    user_id: Uuid,
    default_account_id: Option<Uuid>,
    base_currency: String,
    timezone: String,
    theme: String,
    starting_cash: Decimal,
    dashboard_layout: serde_json::Value,
    updated_at: DateTime<Utc>,
}

impl From<Row> for UserSettings {
    fn from(r: Row) -> Self {
        UserSettings {
            user_id: r.user_id,
            default_account_id: r.default_account_id,
            base_currency: r.base_currency,
            timezone: r.timezone,
            theme: r.theme,
            starting_cash: r.starting_cash,
            dashboard_layout: r.dashboard_layout,
            updated_at: r.updated_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct FsRow {
    id: Uuid,
    user_id: Uuid,
    name: String,
    payload: serde_json::Value,
    is_default: bool,
    created_at: DateTime<Utc>,
}

impl From<FsRow> for FilterSet {
    fn from(r: FsRow) -> Self {
        FilterSet {
            id: r.id,
            user_id: r.user_id,
            name: r.name,
            payload: r.payload,
            is_default: r.is_default,
            created_at: r.created_at,
        }
    }
}
