use chrono::{DateTime, Utc};
use sqlx::PgPool;
use traderview_core::Import;
use uuid::Uuid;

pub async fn create(
    pool: &PgPool,
    account_id: Uuid,
    source: &str,
    filename: &str,
    sha256: &str,
    row_count: i32,
) -> anyhow::Result<Import> {
    let (id, imported_at): (Uuid, DateTime<Utc>) = sqlx::query_as(
        "INSERT INTO imports (account_id, source, filename, sha256, row_count)
              VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (account_id, sha256) DO UPDATE SET filename = EXCLUDED.filename
         RETURNING id, imported_at",
    )
    .bind(account_id)
    .bind(source)
    .bind(filename)
    .bind(sha256)
    .bind(row_count)
    .fetch_one(pool)
    .await?;
    Ok(Import {
        id,
        account_id,
        source: source.into(),
        filename: filename.into(),
        sha256: sha256.into(),
        row_count,
        imported_at,
    })
}

pub async fn list(pool: &PgPool, account_id: Uuid) -> anyhow::Result<Vec<Import>> {
    type ImportRow = (Uuid, Uuid, String, String, String, i32, DateTime<Utc>);
    let rows: Vec<ImportRow> = sqlx::query_as(
        "SELECT id, account_id, source, filename, sha256, row_count, imported_at
           FROM imports WHERE account_id = $1 ORDER BY imported_at DESC",
    )
    .bind(account_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(
            |(id, account_id, source, filename, sha256, row_count, imported_at)| Import {
                id,
                account_id,
                source,
                filename,
                sha256,
                row_count,
                imported_at,
            },
        )
        .collect())
}
