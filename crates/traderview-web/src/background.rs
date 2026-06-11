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
/// Market-heatmap universe (~210 quotes) recompute cadence — matches
/// the 60s on-disk quote cache so tiles stay one refresh fresh.
pub const HEATMAP_REFRESH: Duration = Duration::from_secs(60);
/// World-markets snapshot (16 Yahoo pins) recompute cadence.
pub const MARKETS_REFRESH: Duration = Duration::from_secs(60);
/// Screener snapshot cadence — the screens run on daily bars, so
/// twice a day keeps them fresh without burning quota.
pub const SCREENER_REFRESH: Duration = Duration::from_secs(12 * 60 * 60);
/// Paper TWAP ticker — child-order submission resolution. 5s gives
/// per-slice timing error well under any allowed interval.
pub const PAPER_TWAP_TICK: Duration = Duration::from_secs(5);

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
    spawn_heatmap_universe(pool.clone());
    spawn_markets_snapshot();
    spawn_screener_snapshots(pool.clone());
    spawn_paper_twap_ticker(pool.clone());
    spawn_golden_stars(pool);
}

/// Paper execution ticker — submits due child orders for working
/// parent orders AND fills resting limit/stop orders whose trigger the
/// current quote satisfies, all through the paper engine's fill model.
fn spawn_paper_twap_ticker(pool: PgPool) {
    tokio::spawn(async move {
        loop {
            match traderview_db::paper_parent_orders::tick(&pool).await {
                Ok(0) => {}
                Ok(n) => tracing::info!(children = n, "paper TWAP slices submitted"),
                Err(e) => tracing::warn!(error = %e, "paper TWAP tick failed"),
            }
            match traderview_db::paper::check_pending(&pool).await {
                Ok(0) => {}
                Ok(n) => tracing::info!(filled = n, "resting paper orders filled"),
                Err(e) => tracing::warn!(error = %e, "paper pending check failed"),
            }
            tokio::time::sleep(PAPER_TWAP_TICK).await;
        }
    });
}

/// Screener snapshot refresh — persists each run of the four bar
/// screeners (default ETF universe) plus the carry screen, so the
/// snapshot routes serve history + shape flips without recomputing.
/// Skips the run when a snapshot newer than the cadence exists
/// (restart-safe, like golden stars).
fn spawn_screener_snapshots(pool: PgPool) {
    use traderview_db::screener_snapshots::{self as snaps, SNAPSHOT_UNIVERSE};
    use traderview_db::strategy_calculators as calc;
    tokio::spawn(async move {
        loop {
            let fresh = snaps::latest_two(&pool, "carry")
                .await
                .ok()
                .and_then(|v| v.first().map(|s| s.created_at))
                .map(|ts| {
                    (chrono::Utc::now() - ts).to_std().unwrap_or_default() < SCREENER_REFRESH
                })
                .unwrap_or(false);
            if !fresh {
                let symbols: Vec<String> =
                    SNAPSHOT_UNIVERSE.iter().map(|s| s.to_string()).collect();
                let runs: Vec<(&str, serde_json::Value)> = vec![
                    (
                        "seasonality",
                        serde_json::to_value(calc::seasonality_screen(&pool, &symbols, 10).await)
                            .unwrap_or_default(),
                    ),
                    (
                        "risk",
                        serde_json::to_value(calc::risk_screen(&pool, &symbols, 5).await)
                            .unwrap_or_default(),
                    ),
                    (
                        "momentum",
                        calc::momentum_screen(&pool, &symbols, "SPY", 3)
                            .await
                            .ok()
                            .and_then(|r| serde_json::to_value(r).ok())
                            .unwrap_or_default(),
                    ),
                    (
                        "mean-reversion",
                        serde_json::to_value(
                            calc::mean_reversion_screen(&pool, &symbols, 2).await,
                        )
                        .unwrap_or_default(),
                    ),
                    (
                        "carry",
                        serde_json::to_value(calc::carry_screen(&pool, 6).await)
                            .unwrap_or_default(),
                    ),
                ];
                for (name, payload) in runs {
                    if payload.is_null() {
                        tracing::warn!(screener = name, "screener snapshot produced no payload");
                        continue;
                    }
                    if let Err(e) = snaps::save(&pool, name, &payload).await {
                        tracing::warn!(screener = name, error = %e, "screener snapshot save failed");
                    }
                }
                tracing::info!("screener snapshots refreshed");
            }
            tokio::time::sleep(SCREENER_REFRESH).await;
        }
    });
}

/// World-markets snapshot refresh — 16 Yahoo chart pins for the
/// dashboard world map, kept warm so no request pays the ~1.2s fetch.
fn spawn_markets_snapshot() {
    tokio::spawn(async move {
        loop {
            if let Err(e) = traderview_db::markets::refresh().await {
                tracing::warn!(error = %e, "markets snapshot refresh failed");
            }
            tokio::time::sleep(MARKETS_REFRESH).await;
        }
    });
}

/// Market-heatmap universe refresh. The grid's ~210 quote fan-out runs
/// here on interval; the route only merges the per-user watchlist
/// overlay on top of the precomputed tiles.
fn spawn_heatmap_universe(pool: PgPool) {
    tokio::spawn(async move {
        loop {
            match traderview_db::heatmap::refresh_universe(&pool).await {
                Ok(n) => tracing::debug!(tiles = n, "heatmap universe refreshed"),
                Err(e) => tracing::warn!(error = %e, "heatmap universe refresh failed"),
            }
            tokio::time::sleep(HEATMAP_REFRESH).await;
        }
    });
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
