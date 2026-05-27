//! `traderview-db` — Postgres pool factory, embedded-PG lifecycle, and the
//! repository layer (hand-written sqlx queries) used by `traderview-web`.

pub mod accounts;
pub mod accounts_overview;
pub mod alerts;
pub mod api_tokens;
pub mod backtest_presets;
pub mod breadth;
pub mod catalysts;
pub mod chart_drawings;
pub mod compare;
pub mod comments;
pub mod corr_matrix;
pub mod crypto;
pub mod csv_wizard;
pub mod custom_indicators;
pub mod dashboards;
pub mod darkpool;
pub mod discipline;
pub mod disclosures;
pub mod earnings_cal;
pub mod earnings_iv;
pub mod economy;
pub mod embedded;
pub mod fear_greed;
pub mod executions;
pub mod fill_quality;
pub mod goals;
pub mod halts;
pub mod forum;
pub mod heatmap;
pub mod hotkeys;
pub mod imports;
pub mod journal;
pub mod journal_ai;
pub mod live_positions;
pub mod live_ticks;
pub mod market_data;
pub mod markets;
pub mod mentorships;
pub mod mood_analytics;
pub mod news;
pub mod note_templates;
pub mod options;
pub mod paper;
pub mod plans;
pub mod premarket;
pub mod prices;
pub mod r_distribution;
pub mod rebalance;
pub mod scans;
pub mod screenshots;
pub mod search;
pub mod sector_rotation;
pub mod sectors;
pub mod sentiment;
pub mod settings;
pub mod short_interest;
pub mod strategy_alerts;
pub mod vol;
pub mod vol_surface;
pub mod webhooks;
pub mod shares;
pub mod tags;
pub mod tape_replay;
pub mod tax_lots;
pub mod trade_compare;
pub mod trade_reviews;
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
