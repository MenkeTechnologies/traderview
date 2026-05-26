//! `traderview-db` — Postgres pool factory, embedded-PG lifecycle, and the
//! repository layer (hand-written sqlx queries) used by `traderview-web`.

pub mod accounts;
pub mod alerts;
pub mod comments;
pub mod crypto;
pub mod disclosures;
pub mod earnings_iv;
pub mod economy;
pub mod embedded;
pub mod executions;
pub mod forum;
pub mod heatmap;
pub mod hotkeys;
pub mod imports;
pub mod journal;
pub mod market_data;
pub mod markets;
pub mod mentorships;
pub mod note_templates;
pub mod options;
pub mod paper;
pub mod plans;
pub mod prices;
pub mod scans;
pub mod screenshots;
pub mod search;
pub mod sectors;
pub mod sentiment;
pub mod settings;
pub mod short_interest;
pub mod shares;
pub mod tags;
pub mod trades;
pub mod users;
pub mod watchlists;

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::time::Duration;

pub async fn connect_external(database_url: &str) -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(16)
        .acquire_timeout(Duration::from_secs(10))
        .connect(database_url)
        .await?;
    Ok(pool)
}

/// Run all bundled migrations against an already-open pool.
/// Migrator is embedded at compile time from `../../migrations`.
pub async fn migrate(pool: &PgPool) -> anyhow::Result<()> {
    sqlx::migrate!("../../migrations").run(pool).await?;
    Ok(())
}

pub async fn ensure_local_user(pool: &PgPool) -> anyhow::Result<uuid::Uuid> {
    users::ensure_local(pool).await
}
