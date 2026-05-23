//! traderview-db — Postgres pool factory + embedded-Postgres lifecycle.

pub mod embedded;
pub mod repo;

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::time::Duration;

/// Connect to an external Postgres (web deploy / docker-compose).
pub async fn connect_external(database_url: &str) -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(16)
        .acquire_timeout(Duration::from_secs(10))
        .connect(database_url)
        .await?;
    Ok(pool)
}

/// Run all bundled migrations against an already-open pool.
///
/// The migrator is embedded at compile time from the workspace `migrations/`
/// directory, so the binary is self-sufficient — no need to ship the .sql files.
pub async fn migrate(pool: &PgPool) -> anyhow::Result<()> {
    sqlx::migrate!("../../migrations").run(pool).await?;
    Ok(())
}
