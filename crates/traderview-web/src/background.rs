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
/// Paper equity sampling — 15min resolves intraday swings without
/// flooding the table; unchanged readings are skipped anyway.
pub const PAPER_EQUITY_SNAPSHOT: Duration = Duration::from_secs(15 * 60);
/// Strategy drift sweep — live-vs-backtest divergence across every
/// active strategy. Twice a day: drift moves on the scale of sessions.
pub const STRATEGY_DRIFT_WATCH: Duration = Duration::from_secs(12 * 60 * 60);
/// Rebalance drift sweep — allocation drift also moves on the scale
/// of sessions.
pub const REBALANCE_DRIFT_WATCH: Duration = Duration::from_secs(12 * 60 * 60);
/// Auto-invest pass — cadences are daily+; an hourly pass keeps
/// catch-up snappy after the app was closed.
pub const PAPER_RECURRING_TICK: Duration = Duration::from_secs(60 * 60);
/// Morning digest hour (UTC). 12:00 UTC = pre-market US Eastern
/// year-round (07:00 EST / 08:00 EDT).
pub const DIGEST_HOUR_UTC: u32 = 12;
/// Paper dividend crediting — ex-dates are daily-bar data and the pass
/// is idempotent, so a few runs a day is plenty.
pub const PAPER_DIVIDEND_CREDIT: Duration = Duration::from_secs(6 * 60 * 60);
/// Paper split adjustment — splits are rare daily-bar events; a few
/// idempotent passes a day also keeps the trade-after-split skip
/// window small.
pub const PAPER_SPLIT_ADJUST: Duration = Duration::from_secs(6 * 60 * 60);
/// Paper option expiry settlement — expiry is a daily-date event and
/// the pass is idempotent; settles the day AFTER expiry.
pub const PAPER_OPTION_SETTLE: Duration = Duration::from_secs(6 * 60 * 60);

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
pub fn spawn_refreshers(pool: PgPool, cache: TileCache, hub: crate::realtime::Hub) {
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
    spawn_paper_twap_ticker(pool.clone(), hub.clone());
    spawn_strategy_drift_watch(pool.clone(), hub.clone());
    let hub2 = hub.clone();
    spawn_rebalance_drift_watch(pool.clone(), hub);
    spawn_paper_recurring(pool.clone());
    spawn_daily_digest(pool.clone(), hub2);
    spawn_paper_equity_snapshots(pool.clone());
    spawn_paper_dividend_credits(pool.clone());
    spawn_paper_split_adjustments(pool.clone());
    spawn_paper_option_settlement(pool.clone());
    spawn_golden_stars(pool);
}

/// Paper execution ticker — submits due child orders for working
/// parent orders AND fills resting limit/stop orders whose trigger the
/// current quote satisfies, all through the paper engine's fill model.
/// Background fills publish a PaperFill event so the user hears about
/// fills that happened while they weren't watching.
fn spawn_paper_twap_ticker(pool: PgPool, hub: crate::realtime::Hub) {
    tokio::spawn(async move {
        loop {
            match traderview_db::paper_parent_orders::tick(&pool).await {
                Ok(0) => {}
                Ok(n) => tracing::info!(children = n, "paper TWAP slices submitted"),
                Err(e) => tracing::warn!(error = %e, "paper TWAP tick failed"),
            }
            match traderview_db::paper::check_pending(&pool).await {
                Ok(fills) => {
                    if !fills.is_empty() {
                        tracing::info!(filled = fills.len(), "resting paper orders filled");
                    }
                    for f in fills {
                        hub.publish(crate::realtime::Event::PaperFill {
                            symbol: f.symbol,
                            side: f.side,
                            qty: f.qty.to_string().parse().unwrap_or(0.0),
                            price: f.price.to_string().parse().unwrap_or(0.0),
                            order_type: f.order_type,
                        });
                    }
                }
                Err(e) => tracing::warn!(error = %e, "paper pending check failed"),
            }
            tokio::time::sleep(PAPER_TWAP_TICK).await;
        }
    });
}

/// Strategy drift watch — sweeps every active strategy through the
/// shared live-vs-backtest comparison and publishes a StrategyDrift
/// event for degraded/watch verdicts. Healthy and insufficient-sample
/// strategies stay silent: the feed is for divergence, not status.
fn spawn_strategy_drift_watch(pool: PgPool, hub: crate::realtime::Hub) {
    tokio::spawn(async move {
        // Webhook dedup: notify on ENTERING a bad verdict, not on every
        // 12h sweep while it persists (the live feed still gets every
        // event; phones don't). In-memory — a process restart
        // re-notifies once, which is acceptable at this cadence.
        let mut last_verdict: std::collections::HashMap<uuid::Uuid, &'static str> =
            Default::default();
        loop {
            match traderview_db::algo::all_active_strategy_ids(&pool).await {
                Ok(rows) => {
                    for (id, user_id, name) in rows {
                        match traderview_db::algo::live_divergence(&pool, user_id, id).await {
                            Ok(Some((report, _, _)))
                                if matches!(report.verdict, "degraded" | "watch") =>
                            {
                                tracing::warn!(
                                    strategy = %name,
                                    verdict = report.verdict,
                                    z = ?report.win_rate_z,
                                    "strategy drift detected"
                                );
                                hub.publish(crate::realtime::Event::StrategyDrift {
                                    strategy_id: id.to_string(),
                                    name: name.clone(),
                                    verdict: report.verdict,
                                    win_rate_z: report.win_rate_z,
                                    live_trades: report.live_trades,
                                });
                                // Drift is rare and actionable — worth
                                // the user's webhooks (Slack/Discord/
                                // ntfy), unlike chatty gate fires.
                                let is_new =
                                    last_verdict.insert(id, report.verdict) != Some(report.verdict);
                                if !is_new {
                                    continue;
                                }
                                let payload = traderview_db::webhooks::AlertPayload {
                                    title: format!("Strategy drift: {name}"),
                                    message: format!(
                                        "{} — win-rate z {} over {} live trades; live record diverging from backtest",
                                        report.verdict,
                                        report
                                            .win_rate_z
                                            .map(|z| format!("{z:.2}"))
                                            .unwrap_or_else(|| "n/a".into()),
                                        report.live_trades
                                    ),
                                    symbol: None,
                                    kind: "strategy_drift".into(),
                                    url: None,
                                    fired_at: chrono::Utc::now(),
                                };
                                traderview_db::webhooks::fan_out_all(&pool, user_id, &payload).await;
                            }
                            Ok(_) => {
                                last_verdict.remove(&id);
                            }
                            Err(e) => {
                                tracing::warn!(strategy = %name, error = %e, "drift check failed")
                            }
                        }
                    }
                }
                Err(e) => tracing::warn!(error = %e, "drift sweep failed"),
            }
            tokio::time::sleep(STRATEGY_DRIFT_WATCH).await;
        }
    });
}

/// Rebalance drift watch — plans every paper target portfolio and
/// publishes RebalanceDrift when max drift crosses the target's OWN
/// drift_threshold_pct. Within-tolerance portfolios stay silent.
fn spawn_rebalance_drift_watch(pool: PgPool, hub: crate::realtime::Hub) {
    tokio::spawn(async move {
        // Same transition dedup as the strategy watch.
        let mut drifted: std::collections::HashSet<uuid::Uuid> = Default::default();
        loop {
            match traderview_db::paper_rebalance::all_target_ids(&pool).await {
                Ok(rows) => {
                    for (id, user_id, name) in rows {
                        match traderview_db::paper_rebalance::plan(&pool, user_id, id).await {
                            Ok(Some(p)) if p.above_threshold => {
                                tracing::warn!(
                                    target = %name,
                                    drift = p.max_drift_pct,
                                    threshold = p.target.drift_threshold_pct,
                                    "rebalance drift above tolerance"
                                );
                                hub.publish(crate::realtime::Event::RebalanceDrift {
                                    target_id: id.to_string(),
                                    name: name.clone(),
                                    max_drift_pct: p.max_drift_pct,
                                    threshold_pct: p.target.drift_threshold_pct,
                                });
                                let is_new = drifted.insert(id);
                                if !is_new {
                                    continue;
                                }
                                let payload = traderview_db::webhooks::AlertPayload {
                                    title: format!("Rebalance needed: {name}"),
                                    message: format!(
                                        "max drift {:.1}% exceeds the {:.1}% tolerance",
                                        p.max_drift_pct, p.target.drift_threshold_pct
                                    ),
                                    symbol: None,
                                    kind: "rebalance_drift".into(),
                                    url: None,
                                    fired_at: chrono::Utc::now(),
                                };
                                traderview_db::webhooks::fan_out_all(&pool, user_id, &payload).await;
                            }
                            Ok(_) => {
                                drifted.remove(&id);
                            }
                            Err(e) => tracing::debug!(
                                target = %name, error = %e,
                                "rebalance drift check failed (transient quotes likely)"
                            ),
                        }
                    }
                }
                Err(e) => tracing::warn!(error = %e, "rebalance drift sweep failed"),
            }
            tokio::time::sleep(REBALANCE_DRIFT_WATCH).await;
        }
    });
}

/// Auto-invest — submits due recurring buys through the normal paper
/// fill path; the module advances schedules from the SCHEDULED time.
fn spawn_paper_recurring(pool: PgPool) {
    tokio::spawn(async move {
        loop {
            match traderview_db::paper_recurring::tick(&pool).await {
                Ok(0) => {}
                Ok(n) => tracing::info!(submitted = n, "auto-invest orders submitted"),
                Err(e) => tracing::warn!(error = %e, "auto-invest tick failed"),
            }
            tokio::time::sleep(PAPER_RECURRING_TICK).await;
        }
    });
}

/// Morning digest — sleeps to the next DIGEST_HOUR_UTC, assembles one
/// summary per user (paper equity day-change, drifting strategies,
/// gate fires, earnings in held names, rebalance drift), publishes to
/// the live feed and the user's webhooks. Empty digests stay silent —
/// a digest of empty sections trains the user to stop reading.
fn spawn_daily_digest(pool: PgPool, hub: crate::realtime::Hub) {
    tokio::spawn(async move {
        loop {
            let next = traderview_db::digest::next_digest_time(chrono::Utc::now(), DIGEST_HOUR_UTC);
            let wait = (next - chrono::Utc::now()).num_seconds().max(1) as u64;
            tokio::time::sleep(Duration::from_secs(wait)).await;
            match traderview_db::digest::audience(&pool).await {
                Ok(users) => {
                    for user_id in users {
                        match traderview_db::digest::for_user(&pool, user_id).await {
                            Ok(d) if !d.is_empty() => {
                                let summary = traderview_db::digest::format_digest(&d);
                                tracing::info!(user = %user_id, %summary, "daily digest");
                                hub.publish(crate::realtime::Event::DailyDigest {
                                    summary: summary.clone(),
                                });
                                let payload = traderview_db::webhooks::AlertPayload {
                                    title: "Morning digest".into(),
                                    message: summary,
                                    symbol: None,
                                    kind: "daily_digest".into(),
                                    url: None,
                                    fired_at: chrono::Utc::now(),
                                };
                                traderview_db::webhooks::fan_out_all(&pool, user_id, &payload)
                                    .await;
                            }
                            Ok(_) => {}
                            Err(e) => {
                                tracing::warn!(user = %user_id, error = %e, "digest failed")
                            }
                        }
                    }
                }
                Err(e) => tracing::warn!(error = %e, "digest audience query failed"),
            }
        }
    });
}

/// Paper equity sampler — one snapshot per account per interval,
/// skipped when equity is unchanged or a position can't be marked.
fn spawn_paper_equity_snapshots(pool: PgPool) {
    tokio::spawn(async move {
        loop {
            match traderview_db::paper_equity::snapshot_all(&pool).await {
                Ok(0) => {}
                Ok(n) => tracing::info!(snapshots = n, "paper equity sampled"),
                Err(e) => tracing::warn!(error = %e, "paper equity sampling failed"),
            }
            tokio::time::sleep(PAPER_EQUITY_SNAPSHOT).await;
        }
    });
}

/// Paper option expiry settlement — ITM cash-settles at intrinsic vs
/// the underlying spot, OTM expires worthless; without this, expired
/// positions zombie (the chain stops quoting and the equity sampler
/// marks the whole account unmarkable forever).
fn spawn_paper_option_settlement(pool: PgPool) {
    tokio::spawn(async move {
        loop {
            match traderview_db::paper::settle_expired_options(&pool).await {
                Ok(0) => {}
                Ok(n) => tracing::info!(settled = n, "expired paper options settled"),
                Err(e) => tracing::warn!(error = %e, "option settlement failed"),
            }
            tokio::time::sleep(PAPER_OPTION_SETTLE).await;
        }
    });
}

/// Paper dividend crediting — reconstructs each account's share count
/// going into recent ex-dates from the fill ledger and posts the cash
/// (longs credited, shorts debited). Idempotent via the unique
/// account × symbol × ex-date constraint.
fn spawn_paper_dividend_credits(pool: PgPool) {
    tokio::spawn(async move {
        loop {
            match traderview_db::paper_dividends::credit_all(&pool).await {
                Ok(0) => {}
                Ok(n) => tracing::info!(credits = n, "paper dividends credited"),
                Err(e) => tracing::warn!(error = %e, "paper dividend crediting failed"),
            }
            tokio::time::sleep(PAPER_DIVIDEND_CREDIT).await;
        }
    });
}

/// Paper split adjustment — rewrites positions held through a stock
/// split (qty × ratio, avg ÷ ratio, value-preserving) so the equity
/// curve doesn't record a fake 75% drawdown on a 4:1. Idempotent via
/// the unique account × symbol × split-date constraint.
fn spawn_paper_split_adjustments(pool: PgPool) {
    tokio::spawn(async move {
        loop {
            match traderview_db::paper_splits::adjust_all(&pool).await {
                Ok(0) => {}
                Ok(n) => tracing::info!(adjustments = n, "paper splits applied"),
                Err(e) => tracing::warn!(error = %e, "paper split adjustment failed"),
            }
            tokio::time::sleep(PAPER_SPLIT_ADJUST).await;
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
