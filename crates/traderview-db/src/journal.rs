use chrono::{DateTime, NaiveDate, Utc};
use sqlx::PgPool;
use traderview_core::JournalEntry;
use uuid::Uuid;

pub async fn list_for_day(
    pool: &PgPool,
    user_id: Uuid,
    day: NaiveDate,
) -> anyhow::Result<Vec<JournalEntry>> {
    let rows: Vec<Row> = sqlx::query_as(
        "SELECT id, user_id, trade_id, day, body_md, mood, created_at, updated_at
           FROM journal_entries
          WHERE user_id = $1 AND day = $2 ORDER BY created_at ASC",
    )
    .bind(user_id)
    .bind(day)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

pub async fn list_general(pool: &PgPool, user_id: Uuid) -> anyhow::Result<Vec<JournalEntry>> {
    let rows: Vec<Row> = sqlx::query_as(
        "SELECT id, user_id, trade_id, day, body_md, mood, created_at, updated_at
           FROM journal_entries
          WHERE user_id = $1 AND trade_id IS NULL AND day IS NULL
          ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

pub async fn list_for_trade(
    pool: &PgPool,
    user_id: Uuid,
    trade_id: Uuid,
) -> anyhow::Result<Vec<JournalEntry>> {
    let rows: Vec<Row> = sqlx::query_as(
        "SELECT id, user_id, trade_id, day, body_md, mood, created_at, updated_at
           FROM journal_entries
          WHERE user_id = $1 AND trade_id = $2 ORDER BY created_at ASC",
    )
    .bind(user_id)
    .bind(trade_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

pub async fn create(
    pool: &PgPool,
    user_id: Uuid,
    trade_id: Option<Uuid>,
    day: Option<NaiveDate>,
    body_md: &str,
    mood: Option<i16>,
) -> anyhow::Result<JournalEntry> {
    let row: Row = sqlx::query_as(
        "INSERT INTO journal_entries (user_id, trade_id, day, body_md, mood)
              VALUES ($1, $2, $3, $4, $5)
         RETURNING id, user_id, trade_id, day, body_md, mood, created_at, updated_at",
    )
    .bind(user_id)
    .bind(trade_id)
    .bind(day)
    .bind(body_md)
    .bind(mood)
    .fetch_one(pool)
    .await?;
    Ok(row.into())
}

pub async fn update(
    pool: &PgPool,
    user_id: Uuid,
    id: Uuid,
    body_md: &str,
    mood: Option<i16>,
) -> anyhow::Result<bool> {
    let r = sqlx::query(
        "UPDATE journal_entries SET body_md = $3, mood = $4, updated_at = now()
          WHERE id = $1 AND user_id = $2",
    )
    .bind(id)
    .bind(user_id)
    .bind(body_md)
    .bind(mood)
    .execute(pool)
    .await?;
    Ok(r.rows_affected() > 0)
}

pub async fn delete(pool: &PgPool, user_id: Uuid, id: Uuid) -> anyhow::Result<bool> {
    let r = sqlx::query("DELETE FROM journal_entries WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(r.rows_affected() > 0)
}

#[derive(sqlx::FromRow)]
pub struct Row {
    pub id: Uuid,
    pub user_id: Uuid,
    pub trade_id: Option<Uuid>,
    pub day: Option<NaiveDate>,
    pub body_md: String,
    pub mood: Option<i16>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Row> for JournalEntry {
    fn from(r: Row) -> Self {
        JournalEntry {
            id: r.id,
            user_id: r.user_id,
            trade_id: r.trade_id,
            day: r.day,
            body_md: r.body_md,
            mood: r.mood,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}
