use sqlx::PgPool;
use traderview_core::Tag;
use uuid::Uuid;

pub async fn list(pool: &PgPool, user_id: Uuid) -> anyhow::Result<Vec<Tag>> {
    let rows: Vec<(Uuid, Uuid, String, String)> = sqlx::query_as(
        "SELECT id, user_id, name, color FROM tags WHERE user_id = $1 ORDER BY name",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(id, user_id, name, color)| Tag {
            id,
            user_id,
            name,
            color,
        })
        .collect())
}

pub async fn create(pool: &PgPool, user_id: Uuid, name: &str, color: &str) -> anyhow::Result<Tag> {
    let (id,): (Uuid,) =
        sqlx::query_as("INSERT INTO tags (user_id, name, color) VALUES ($1, $2, $3) RETURNING id")
            .bind(user_id)
            .bind(name)
            .bind(color)
            .fetch_one(pool)
            .await?;
    Ok(Tag {
        id,
        user_id,
        name: name.into(),
        color: color.into(),
    })
}

/// Get-or-create a tag by name. The insert is guarded by WHERE NOT
/// EXISTS rather than a unique constraint (the tags table predates
/// this and may hold user-created duplicates); the residual two-
/// connection race worst-cases as a duplicate tag, not an error.
pub async fn ensure(pool: &PgPool, user_id: Uuid, name: &str, color: &str) -> anyhow::Result<Uuid> {
    sqlx::query(
        "INSERT INTO tags (user_id, name, color)
         SELECT $1, $2, $3
          WHERE NOT EXISTS (SELECT 1 FROM tags WHERE user_id = $1 AND name = $2)",
    )
    .bind(user_id)
    .bind(name)
    .bind(color)
    .execute(pool)
    .await?;
    let (id,): (Uuid,) =
        sqlx::query_as("SELECT id FROM tags WHERE user_id = $1 AND name = $2 LIMIT 1")
            .bind(user_id)
            .bind(name)
            .fetch_one(pool)
            .await?;
    Ok(id)
}

pub async fn delete(pool: &PgPool, user_id: Uuid, tag_id: Uuid) -> anyhow::Result<bool> {
    let r = sqlx::query("DELETE FROM tags WHERE id = $1 AND user_id = $2")
        .bind(tag_id)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(r.rows_affected() > 0)
}

pub async fn attach_to_trade(pool: &PgPool, trade_id: Uuid, tag_id: Uuid) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO trade_tags (trade_id, tag_id) VALUES ($1, $2)
         ON CONFLICT DO NOTHING",
    )
    .bind(trade_id)
    .bind(tag_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn detach_from_trade(pool: &PgPool, trade_id: Uuid, tag_id: Uuid) -> anyhow::Result<()> {
    sqlx::query("DELETE FROM trade_tags WHERE trade_id = $1 AND tag_id = $2")
        .bind(trade_id)
        .bind(tag_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn tags_for_trade(pool: &PgPool, trade_id: Uuid) -> anyhow::Result<Vec<Tag>> {
    let rows: Vec<(Uuid, Uuid, String, String)> = sqlx::query_as(
        "SELECT t.id, t.user_id, t.name, t.color
           FROM tags t JOIN trade_tags tt ON tt.tag_id = t.id
          WHERE tt.trade_id = $1 ORDER BY t.name",
    )
    .bind(trade_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(id, user_id, name, color)| Tag {
            id,
            user_id,
            name,
            color,
        })
        .collect())
}
