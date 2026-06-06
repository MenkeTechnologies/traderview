use chrono::{DateTime, Utc};
use sqlx::PgPool;
use traderview_core::Screenshot;
use uuid::Uuid;

pub struct NewScreenshot<'a> {
    pub user_id: Uuid,
    pub trade_id: Option<Uuid>,
    pub journal_id: Option<Uuid>,
    pub filename: &'a str,
    pub mime_type: &'a str,
    pub bytes: &'a [u8],
    pub caption: &'a str,
}

pub async fn create(pool: &PgPool, ss: NewScreenshot<'_>) -> anyhow::Result<Screenshot> {
    let size = ss.bytes.len() as i64;
    let (id, position, created_at): (Uuid, i32, DateTime<Utc>) = sqlx::query_as(
        "INSERT INTO screenshots
            (user_id, trade_id, journal_id, filename, mime_type, size_bytes, bytes, caption, position)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8,
                 COALESCE(
                    (SELECT MAX(position) + 1 FROM screenshots
                       WHERE (trade_id = $2 AND $2 IS NOT NULL)
                          OR (journal_id = $3 AND $3 IS NOT NULL)), 0))
         RETURNING id, position, created_at",
    )
    .bind(ss.user_id)
    .bind(ss.trade_id)
    .bind(ss.journal_id)
    .bind(ss.filename)
    .bind(ss.mime_type)
    .bind(size)
    .bind(ss.bytes)
    .bind(ss.caption)
    .fetch_one(pool)
    .await?;
    Ok(Screenshot {
        id,
        user_id: ss.user_id,
        trade_id: ss.trade_id,
        journal_id: ss.journal_id,
        filename: ss.filename.into(),
        mime_type: ss.mime_type.into(),
        size_bytes: size,
        caption: ss.caption.into(),
        position,
        created_at,
    })
}

pub async fn list_for_trade(pool: &PgPool, trade_id: Uuid) -> anyhow::Result<Vec<Screenshot>> {
    let rows: Vec<Row> = sqlx::query_as(
        "SELECT id, user_id, trade_id, journal_id, filename, mime_type, size_bytes, caption, position, created_at
           FROM screenshots WHERE trade_id = $1 ORDER BY position, created_at",
    )
    .bind(trade_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

pub async fn get_bytes(
    pool: &PgPool,
    user_id: Uuid,
    id: Uuid,
) -> anyhow::Result<Option<(String, Vec<u8>)>> {
    let row: Option<(String, Vec<u8>)> =
        sqlx::query_as("SELECT mime_type, bytes FROM screenshots WHERE id = $1 AND user_id = $2")
            .bind(id)
            .bind(user_id)
            .fetch_optional(pool)
            .await?;
    Ok(row)
}

pub async fn delete(pool: &PgPool, user_id: Uuid, id: Uuid) -> anyhow::Result<bool> {
    let r = sqlx::query("DELETE FROM screenshots WHERE id = $1 AND user_id = $2")
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
    pub journal_id: Option<Uuid>,
    pub filename: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub caption: String,
    pub position: i32,
    pub created_at: DateTime<Utc>,
}

impl From<Row> for Screenshot {
    fn from(r: Row) -> Self {
        Screenshot {
            id: r.id,
            user_id: r.user_id,
            trade_id: r.trade_id,
            journal_id: r.journal_id,
            filename: r.filename,
            mime_type: r.mime_type,
            size_bytes: r.size_bytes,
            caption: r.caption,
            position: r.position,
            created_at: r.created_at,
        }
    }
}
