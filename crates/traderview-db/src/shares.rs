use chrono::{DateTime, Utc};
use sqlx::PgPool;
use traderview_core::{slug, TradeShare};
use uuid::Uuid;

pub async fn create(
    pool: &PgPool,
    trade_id: Uuid,
    owner_id: Uuid,
    is_public: bool,
    show_notes: bool,
    show_screenshots: bool,
) -> anyhow::Result<TradeShare> {
    // Retry slug generation on the (rare) chance of collision.
    for _ in 0..6 {
        let s = slug::random(8);
        let res: Result<Row, sqlx::Error> = sqlx::query_as(
            "INSERT INTO trade_shares
                (trade_id, owner_id, slug, is_public, show_notes, show_screenshots)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, trade_id, owner_id, slug, is_public,
                       show_notes, show_screenshots, view_count, created_at, expires_at",
        )
        .bind(trade_id)
        .bind(owner_id)
        .bind(&s)
        .bind(is_public)
        .bind(show_notes)
        .bind(show_screenshots)
        .fetch_one(pool)
        .await;
        match res {
            Ok(r) => return Ok(r.into()),
            Err(sqlx::Error::Database(db)) if db.constraint() == Some("trade_shares_slug_key") => {
                continue
            }
            Err(e) => return Err(e.into()),
        }
    }
    anyhow::bail!("could not allocate slug after 6 attempts")
}

pub async fn by_slug(pool: &PgPool, slug: &str) -> anyhow::Result<Option<TradeShare>> {
    let row: Option<Row> = sqlx::query_as(
        "SELECT id, trade_id, owner_id, slug, is_public, show_notes, show_screenshots,
                view_count, created_at, expires_at
           FROM trade_shares WHERE slug = $1",
    )
    .bind(slug)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(Into::into))
}

pub async fn bump_view(pool: &PgPool, slug: &str) -> anyhow::Result<()> {
    sqlx::query("UPDATE trade_shares SET view_count = view_count + 1 WHERE slug = $1")
        .bind(slug)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn list_public(pool: &PgPool, limit: i64) -> anyhow::Result<Vec<TradeShare>> {
    let rows: Vec<Row> = sqlx::query_as(
        "SELECT id, trade_id, owner_id, slug, is_public, show_notes, show_screenshots,
                view_count, created_at, expires_at
           FROM trade_shares
          WHERE is_public = TRUE AND (expires_at IS NULL OR expires_at > now())
          ORDER BY created_at DESC LIMIT $1",
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

pub async fn list_for_owner(pool: &PgPool, owner_id: Uuid) -> anyhow::Result<Vec<TradeShare>> {
    let rows: Vec<Row> = sqlx::query_as(
        "SELECT id, trade_id, owner_id, slug, is_public, show_notes, show_screenshots,
                view_count, created_at, expires_at
           FROM trade_shares WHERE owner_id = $1 ORDER BY created_at DESC",
    )
    .bind(owner_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

pub async fn delete(pool: &PgPool, owner_id: Uuid, id: Uuid) -> anyhow::Result<bool> {
    let r = sqlx::query("DELETE FROM trade_shares WHERE id = $1 AND owner_id = $2")
        .bind(id)
        .bind(owner_id)
        .execute(pool)
        .await?;
    Ok(r.rows_affected() > 0)
}

#[derive(sqlx::FromRow)]
struct Row {
    id: Uuid,
    trade_id: Uuid,
    owner_id: Uuid,
    slug: String,
    is_public: bool,
    show_notes: bool,
    show_screenshots: bool,
    view_count: i64,
    created_at: DateTime<Utc>,
    expires_at: Option<DateTime<Utc>>,
}

impl From<Row> for TradeShare {
    fn from(r: Row) -> Self {
        TradeShare {
            id: r.id,
            trade_id: r.trade_id,
            owner_id: r.owner_id,
            slug: r.slug,
            is_public: r.is_public,
            show_notes: r.show_notes,
            show_screenshots: r.show_screenshots,
            view_count: r.view_count,
            created_at: r.created_at,
            expires_at: r.expires_at,
        }
    }
}
