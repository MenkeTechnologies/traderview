use chrono::{DateTime, Utc};
use sqlx::PgPool;
use traderview_core::Comment;
use uuid::Uuid;

pub async fn create(
    pool: &PgPool,
    share_id: Uuid,
    author_id: Uuid,
    parent_id: Option<Uuid>,
    body_md: &str,
) -> anyhow::Result<Comment> {
    let row: Row = sqlx::query_as(
        "INSERT INTO comments (share_id, author_id, parent_id, body_md)
              VALUES ($1, $2, $3, $4)
         RETURNING id, share_id, author_id, parent_id, body_md, created_at, updated_at",
    )
    .bind(share_id)
    .bind(author_id)
    .bind(parent_id)
    .bind(body_md)
    .fetch_one(pool)
    .await?;
    Ok(row.into())
}

pub async fn list_for_share(pool: &PgPool, share_id: Uuid) -> anyhow::Result<Vec<Comment>> {
    let rows: Vec<Row> = sqlx::query_as(
        "SELECT id, share_id, author_id, parent_id, body_md, created_at, updated_at
           FROM comments WHERE share_id = $1 ORDER BY created_at ASC",
    )
    .bind(share_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

pub async fn delete(pool: &PgPool, author_id: Uuid, id: Uuid) -> anyhow::Result<bool> {
    let r = sqlx::query("DELETE FROM comments WHERE id = $1 AND author_id = $2")
        .bind(id)
        .bind(author_id)
        .execute(pool)
        .await?;
    Ok(r.rows_affected() > 0)
}

#[derive(sqlx::FromRow)]
struct Row {
    id: Uuid,
    share_id: Uuid,
    author_id: Uuid,
    parent_id: Option<Uuid>,
    body_md: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<Row> for Comment {
    fn from(r: Row) -> Self {
        Comment {
            id: r.id,
            share_id: r.share_id,
            author_id: r.author_id,
            parent_id: r.parent_id,
            body_md: r.body_md,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}
