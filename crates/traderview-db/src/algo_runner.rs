//! Background task that drives every active algo strategy on bar close.
//!
//! Reuses the live_ticks worker's price_bars output — the existing 10-second
//! bucket aggregator writes there in real time, and 1-minute bars are
//! rolled up from those. We just read the latest N bars per symbol +
//! interval and call `process_bar_window`.
//!
//! Clock alignment: the loop sleeps until `next_boundary(now, interval)`
//! so every 10s/60s tick happens at the same moment for every strategy
//! on that timeframe. That matches the bar boundary the live_ticks
//! worker flushes at, so a strategy never reads a half-closed bar.
//!
//! Wired into `bin/server.rs::main` via a `tokio::spawn(algo_runner::run_loop(pool))`.

use crate::algo::{self, AlgoStrategy};
use crate::algo_engine::{self, BrokerSink, InMemorySink};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::time::Duration;
use traderview_core::models::{BarInterval, PriceBar};
use uuid::Uuid;

/// How many bars to feed the strategy on each tick. Has to be ≥ each
/// strategy's `min_bars()`; 250 covers every shipping strategy with
/// margin for the longer-lookback ones (squeeze_lookback = 100).
const BAR_WINDOW_SIZE: i64 = 300;

/// One tick: load active strategies, pull bars, fire `process_bar_window`
/// for each (strategy, symbol). Returns the count of strategies actually
/// processed (skipped due to insufficient bars or empty universe → don't
/// count). Errors from individual strategies are logged + swallowed so
/// one broken config doesn't take the whole loop down.
pub async fn tick(pool: &PgPool, now: DateTime<Utc>) -> usize {
    let strategies = match algo::list_active_strategies(pool).await {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!(error = %e, "algo_runner: list_active_strategies failed");
            return 0;
        }
    };
    if strategies.is_empty() {
        return 0;
    }
    let sink = InMemorySink::default();
    let mut processed = 0usize;
    for s in &strategies {
        let interval = parse_timeframe(&s.timeframe);
        // Only drive the strategy on a bar boundary FOR ITS OWN TIMEFRAME —
        // a 1m strategy ignores the 10s ticks the loop also fires on.
        if !is_boundary(now, interval) {
            continue;
        }
        let symbols = symbol_universe(s);
        if symbols.is_empty() {
            tracing::debug!(strategy = %s.id, "no symbols in universe");
            continue;
        }
        match drive_strategy(pool, &sink, s, interval, &symbols).await {
            Ok(n) => processed += n,
            Err(e) => tracing::warn!(strategy = %s.id, error = %e, "drive_strategy failed"),
        }
    }
    processed
}

/// Run the tick loop forever. Sleeps until the next 10s clock boundary,
/// then calls `tick`. The 10s cadence covers BOTH sec10 strategies
/// (every tick) and min1 strategies (every 6th tick, on the 60s
/// boundary). Caller is expected to `tokio::spawn` this and forget;
/// no clean shutdown signal here yet.
pub async fn run_loop(pool: PgPool) -> ! {
    loop {
        let now = Utc::now();
        let next = next_boundary(now, BarInterval::S10);
        let wait = (next - now).num_milliseconds().max(0) as u64;
        tokio::time::sleep(Duration::from_millis(wait)).await;
        let processed = tick(&pool, Utc::now()).await;
        if processed > 0 {
            tracing::debug!(processed, "algo_runner tick");
        }
    }
}

// ─── helpers ────────────────────────────────────────────────────────────────

fn parse_timeframe(tf: &str) -> BarInterval {
    match tf {
        "sec10" => BarInterval::S10,
        _ => BarInterval::M1,
    }
}

/// `now` is on a bar boundary for `interval` when `now.timestamp() %
/// interval.seconds() == 0` to the nearest 500ms. The loop sleeps to the
/// boundary so this is true at tick time; the check is a guard against
/// scheduling jitter waking us a few hundred ms late.
fn is_boundary(now: DateTime<Utc>, interval: BarInterval) -> bool {
    let secs = interval.seconds();
    (now.timestamp() % secs).abs() <= 1
}

fn next_boundary(now: DateTime<Utc>, interval: BarInterval) -> DateTime<Utc> {
    let secs = interval.seconds();
    let now_secs = now.timestamp();
    let aligned = ((now_secs / secs) + 1) * secs;
    DateTime::<Utc>::from_timestamp(aligned, 0).unwrap_or(now)
}

/// For commit 18 the universe resolution is intentionally minimal:
/// strategies with `universe_mode='autoscan'` get an empty list (the
/// scanner integration lands in commit 19), and watchlist mode pulls
/// nothing yet either — the runner skips strategies with empty universes.
/// This keeps the loop alive without firing trades against the wrong
/// symbols while the wiring matures.
fn symbol_universe(_s: &AlgoStrategy) -> Vec<String> {
    Vec::new()
}

async fn drive_strategy(
    pool: &PgPool,
    sink: &dyn BrokerSink,
    strategy: &AlgoStrategy,
    interval: BarInterval,
    symbols: &[String],
) -> Result<usize, anyhow::Error> {
    let Some(run) = algo::get_open_run(pool, strategy.id).await? else {
        // No open run → strategy is enabled but not actively running.
        // The runner does not auto-start runs in commit 18; the UI's
        // "start" button is the explicit gate.
        return Ok(0);
    };
    let equity = 100_000.0; // placeholder; commit 19 plumbs real account equity
    let open_positions: i64 = 0; // placeholder; commit 19 reads live_positions
    let mut driven = 0usize;
    for symbol in symbols {
        let bars = fetch_recent_bars(pool, symbol, interval, BAR_WINDOW_SIZE).await?;
        if bars.is_empty() {
            continue;
        }
        match algo_engine::process_bar_window(
            pool, sink, strategy, run.id, &bars, equity, open_positions,
        )
        .await
        {
            Ok(_) => driven += 1,
            Err(e) => {
                tracing::debug!(
                    strategy = %strategy.id, symbol, error = %e,
                    "process_bar_window non-fatal"
                );
            }
        }
    }
    let _ = algo::increment_run_counter(
        pool,
        run.id,
        algo::RunCounter::BarsProcessed,
        driven as i64,
    )
    .await;
    Ok(driven)
}

async fn fetch_recent_bars(
    pool: &PgPool,
    symbol: &str,
    interval: BarInterval,
    limit: i64,
) -> Result<Vec<PriceBar>, anyhow::Error> {
    type Row = (
        String,
        String,
        DateTime<Utc>,
        Decimal,
        Decimal,
        Decimal,
        Decimal,
        Decimal,
        String,
    );
    // Pull the most recent `limit` bars, then return them in chronological
    // order so the strategy sees them oldest-to-newest like every other caller.
    let mut rows: Vec<Row> = sqlx::query_as(
        "SELECT symbol, interval::text, bar_time, open, high, low, close, volume, source
           FROM price_bars
          WHERE symbol = $1 AND interval = $2::bar_interval_t
          ORDER BY bar_time DESC
          LIMIT $3",
    )
    .bind(symbol)
    .bind(interval.label())
    .bind(limit)
    .fetch_all(pool)
    .await?;
    rows.reverse();
    Ok(rows
        .into_iter()
        .map(|(symbol, iv, bar_time, open, high, low, close, volume, source)| PriceBar {
            symbol,
            interval: parse_interval(&iv),
            bar_time,
            open,
            high,
            low,
            close,
            volume,
            source,
        })
        .collect())
}

fn parse_interval(s: &str) -> BarInterval {
    match s {
        "10s" => BarInterval::S10,
        "1m" => BarInterval::M1,
        "5m" => BarInterval::M5,
        "15m" => BarInterval::M15,
        "1h" => BarInterval::H1,
        "1d" => BarInterval::D1,
        "1w" => BarInterval::W1,
        _ => BarInterval::M1,
    }
}

// Silence unused-import warning until commit 19 adds the symbol resolver
// that constructs Uuid values from watchlist_id.
const _: fn() = || {
    let _ = std::mem::size_of::<Uuid>();
};

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    // 1_700_000_040 is a clean M1 + S10 epoch boundary (1700000040 / 60 = 28333334).
    // 1_700_000_010 is a clean S10 boundary but NOT M1.
    const ON_S10_OFF_M1: i64 = 1_700_000_010;
    const ON_S10_ON_M1: i64 = 1_700_000_040;

    #[test]
    fn next_boundary_aligns_to_interval() {
        // 5s after a 10s boundary → next is exactly +5s.
        let now = Utc.timestamp_opt(ON_S10_OFF_M1 - 5, 0).unwrap();
        let nb = next_boundary(now, BarInterval::S10);
        assert_eq!(nb.timestamp(), ON_S10_OFF_M1);

        // Already on a boundary — function advances one interval forward,
        // not 'returns same'. That's the contract: caller's sleep arrives
        // at the NEXT tick, never re-fires the current one.
        let on = Utc.timestamp_opt(ON_S10_ON_M1, 0).unwrap();
        let nb_60 = next_boundary(on, BarInterval::M1);
        assert_eq!(nb_60.timestamp(), ON_S10_ON_M1 + 60);
    }

    #[test]
    fn is_boundary_tolerates_jitter() {
        let exact = Utc.timestamp_opt(ON_S10_OFF_M1, 0).unwrap();
        assert!(is_boundary(exact, BarInterval::S10));
        let off = Utc.timestamp_opt(ON_S10_OFF_M1 + 3, 0).unwrap();
        assert!(!is_boundary(off, BarInterval::S10));
        // M1 boundary check — only true on minute boundaries.
        let min = Utc.timestamp_opt(ON_S10_ON_M1, 0).unwrap();
        assert!(is_boundary(min, BarInterval::M1));
        let almost = Utc.timestamp_opt(ON_S10_ON_M1 - 1, 0).unwrap();
        assert!(!is_boundary(almost, BarInterval::M1));
    }

    #[test]
    fn parse_timeframe_falls_back_to_min1() {
        assert!(matches!(parse_timeframe("sec10"), BarInterval::S10));
        assert!(matches!(parse_timeframe("min1"), BarInterval::M1));
        assert!(matches!(parse_timeframe("garbage"), BarInterval::M1));
    }
}
