use chrono::{DateTime, Utc};
use sqlx::PgPool;
use traderview_core::NoteTemplate;
use uuid::Uuid;

pub async fn list(pool: &PgPool, user_id: Uuid, scope: Option<&str>) -> anyhow::Result<Vec<NoteTemplate>> {
    let rows: Vec<Row> = if let Some(sc) = scope {
        sqlx::query_as(
            "SELECT id, user_id, name, scope, body_md, is_default, created_at, updated_at
               FROM note_templates WHERE user_id = $1 AND scope = $2
              ORDER BY is_default DESC, name",
        )
        .bind(user_id)
        .bind(sc)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as(
            "SELECT id, user_id, name, scope, body_md, is_default, created_at, updated_at
               FROM note_templates WHERE user_id = $1
              ORDER BY scope, is_default DESC, name",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?
    };
    Ok(rows.into_iter().map(Into::into).collect())
}

pub async fn upsert(
    pool: &PgPool,
    user_id: Uuid,
    name: &str,
    scope: &str,
    body_md: &str,
    is_default: bool,
) -> anyhow::Result<NoteTemplate> {
    // If this template is being marked default, clear other defaults in same scope first.
    if is_default {
        sqlx::query(
            "UPDATE note_templates SET is_default = FALSE
              WHERE user_id = $1 AND scope = $2 AND name <> $3",
        )
        .bind(user_id)
        .bind(scope)
        .bind(name)
        .execute(pool)
        .await?;
    }
    let row: Row = sqlx::query_as(
        "INSERT INTO note_templates (user_id, name, scope, body_md, is_default)
              VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (user_id, name) DO UPDATE
             SET scope = EXCLUDED.scope,
                 body_md = EXCLUDED.body_md,
                 is_default = EXCLUDED.is_default,
                 updated_at = now()
         RETURNING id, user_id, name, scope, body_md, is_default, created_at, updated_at",
    )
    .bind(user_id)
    .bind(name)
    .bind(scope)
    .bind(body_md)
    .bind(is_default)
    .fetch_one(pool)
    .await?;
    Ok(row.into())
}

pub async fn default_for(
    pool: &PgPool,
    user_id: Uuid,
    scope: &str,
) -> anyhow::Result<Option<NoteTemplate>> {
    let row: Option<Row> = sqlx::query_as(
        "SELECT id, user_id, name, scope, body_md, is_default, created_at, updated_at
           FROM note_templates
          WHERE user_id = $1 AND scope = $2 AND is_default = TRUE
          LIMIT 1",
    )
    .bind(user_id)
    .bind(scope)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(Into::into))
}

pub async fn delete(pool: &PgPool, user_id: Uuid, id: Uuid) -> anyhow::Result<bool> {
    let r = sqlx::query("DELETE FROM note_templates WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(r.rows_affected() > 0)
}

#[derive(sqlx::FromRow)]
struct Row {
    id: Uuid,
    user_id: Uuid,
    name: String,
    scope: String,
    body_md: String,
    is_default: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<Row> for NoteTemplate {
    fn from(r: Row) -> Self {
        NoteTemplate {
            id: r.id,
            user_id: r.user_id,
            name: r.name,
            scope: r.scope,
            body_md: r.body_md,
            is_default: r.is_default,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}
