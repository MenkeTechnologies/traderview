//! Background refreshers for precomputed views.
//!
//! Every dashboard tile that aggregates a multi-symbol universe
//! (sector heatmap, breadth, fear/greed, sector rotation, RRG) is
//! computed here on a fixed interval and parked in an in-memory cache;
//! the GET routes serve the cached JSON so opening a view never
//! triggers a multi-symbol compute. The Golden Stars universe refresh
//! (which persists to Postgres instead of the in-memory cache) lives
//! here too. Both binaries — the standalone server and the Tauri
//! desktop — call `spawn_refreshers` once at boot.

use dashmap::DashMap;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;

pub type TileCache = Arc<DashMap<&'static str, serde_json::Value>>;

// Tile keys.
pub const SECTORS: &str = "sectors";
pub const BREADTH: &str = "breadth";
pub const FEAR_GREED: &str = "fear_greed";
pub const SECTOR_ROTATION: &str = "sector_rotation";
pub const RRG: &str = "rrg";

// Refresh cadence per tile. Quote-driven gauges refresh every minute
// or five; daily-bar reports every few hours.
pub const BREADTH_REFRESH: Duration = Duration::from_secs(60);
pub const SECTORS_REFRESH: Duration = Duration::from_secs(5 * 60);
pub const FEAR_GREED_REFRESH: Duration = Duration::from_secs(5 * 60);
pub const SECTOR_ROTATION_REFRESH: Duration = Duration::from_secs(4 * 60 * 60);
pub const RRG_REFRESH: Duration = Duration::from_secs(4 * 60 * 60);
/// Golden Stars (stock recommendations) universe recompute cadence.
pub const GOLDEN_STARS_REFRESH: Duration = Duration::from_secs(4 * 60 * 60);

async fn compute_tile(pool: &PgPool, key: &'static str) -> anyhow::Result<serde_json::Value> {
    Ok(match key {
        SECTORS => serde_json::to_value(traderview_db::sectors::ranked(pool).await?)?,
        BREADTH => serde_json::to_value(traderview_db::breadth::snapshot(pool).await?)?,
        FEAR_GREED => serde_json::to_value(traderview_db::fear_greed::snapshot(pool).await?)?,
        SECTOR_ROTATION => {
            serde_json::to_value(traderview_db::sector_rotation::report(pool).await?)?
        }
        RRG => serde_json::to_value(traderview_db::rrg::compute(pool).await)?,
        other => anyhow::bail!("unknown tile key: {other}"),
    })
}

/// Serve a tile from the cache. Computes inline only on a cold cache —
/// the boot race before the first background refresh lands.
pub async fn tile(
    pool: &PgPool,
    cache: &TileCache,
    key: &'static str,
) -> anyhow::Result<serde_json::Value> {
    if let Some(v) = cache.get(key) {
        return Ok(v.clone());
    }
    let v = compute_tile(pool, key).await?;
    cache.insert(key, v.clone());
    Ok(v)
}

/// Spawn every background refresher. Call once at boot.
pub fn spawn_refreshers(pool: PgPool, cache: TileCache) {
    for (key, every) in [
        (SECTORS, SECTORS_REFRESH),
        (BREADTH, BREADTH_REFRESH),
        (FEAR_GREED, FEAR_GREED_REFRESH),
        (SECTOR_ROTATION, SECTOR_ROTATION_REFRESH),
        (RRG, RRG_REFRESH),
    ] {
        let pool = pool.clone();
        let cache = cache.clone();
        tokio::spawn(async move {
            loop {
                match compute_tile(&pool, key).await {
                    Ok(v) => {
                        cache.insert(key, v);
                    }
                    Err(e) => tracing::warn!(tile = key, error = %e, "tile refresh failed"),
                }
                tokio::time::sleep(every).await;
            }
        });
    }
    spawn_golden_stars(pool);
}

/// Golden Stars universe refresh. Recomputes the whole universe
/// (leaderboard + sector ETFs) every GOLDEN_STARS_REFRESH so no view
/// ever has to trigger a compute on demand. On boot, computes
/// immediately when the table is empty or the newest row is already
/// older than one interval (covers downtime); otherwise waits out the
/// remainder of the current interval. Verdict-change watchers fire
/// right after each compute.
fn spawn_golden_stars(pool: PgPool) {
    tokio::spawn(async move {
        let newest: Option<(Option<chrono::DateTime<chrono::Utc>>,)> =
            sqlx::query_as("SELECT MAX(computed_at) FROM stock_recommendations")
                .fetch_optional(&pool)
                .await
                .ok()
                .flatten();
        let age = newest
            .and_then(|(ts,)| ts)
            .and_then(|ts| (chrono::Utc::now() - ts).to_std().ok());
        if let Some(age) = age.filter(|a| *a < GOLDEN_STARS_REFRESH) {
            let wait = GOLDEN_STARS_REFRESH - age;
            tracing::info!(?wait, "golden-stars: data fresh, sleeping until next refresh");
            tokio::time::sleep(wait).await;
        } else {
            tracing::info!("golden-stars: no data or stale, refreshing now");
        }
        loop {
            let res = traderview_db::stock_recommendation::cron_compute_universe(
                &pool,
                traderview_db::stock_recommendation::DEFAULT_UNIVERSE,
            )
            .await;
            tracing::info!(
                succeeded = res.succeeded, failed = res.failed,
                "golden-stars background refresh done"
            );
            let fired = traderview_db::stock_recommendation_watchers::check_and_fire(&pool)
                .await
                .unwrap_or(0);
            if fired > 0 {
                tracing::info!(fired, "golden-stars watchers fired");
            }
            tokio::time::sleep(GOLDEN_STARS_REFRESH).await;
        }
    });
}
