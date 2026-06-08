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
use crate::algo_engine::{self, BrokerSink, EventSink};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::time::Duration;
use traderview_core::models::{BarInterval, PriceBar};

/// How many bars to feed the strategy on each tick. Has to be ≥ each
/// strategy's `min_bars()`; 250 covers every shipping strategy with
/// margin for the longer-lookback ones (squeeze_lookback = 100).
const BAR_WINDOW_SIZE: i64 = 300;

/// One tick: load active strategies, pull bars, fire `process_bar_window`
/// for each (strategy, symbol). Returns the count of strategies actually
/// processed (skipped due to insufficient bars or empty universe → don't
/// count). Errors from individual strategies are logged + swallowed so
/// one broken config doesn't take the whole loop down.
pub async fn tick(pool: &PgPool, now: DateTime<Utc>, event_sink: Option<&EventSink>) -> usize {
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
    let mut processed = 0usize;
    for s in &strategies {
        let interval = parse_timeframe(&s.timeframe);
        // Only drive the strategy on a bar boundary FOR ITS OWN TIMEFRAME —
        // a 1m strategy ignores the 10s ticks the loop also fires on.
        if !is_boundary(now, interval) {
            continue;
        }
        let symbols = match symbol_universe(pool, s).await {
            Ok(syms) => syms,
            Err(e) => {
                tracing::warn!(strategy = %s.id, error = %e, "symbol_universe failed");
                continue;
            }
        };
        if symbols.is_empty() {
            tracing::debug!(strategy = %s.id, "no symbols in universe");
            continue;
        }
        // PEAD layer: narrow the universe to symbols with a recent
        // positive earnings surprise. Without this filter PEAD would
        // fire on any uptrending stock (the technical confirm in the
        // strategy module isn't enough on its own).
        let symbols = if s.strategy_type == "pead" {
            let min_surprise = s
                .entry_rules
                .get("min_surprise_pct")
                .and_then(|v| v.as_f64())
                .unwrap_or(5.0) as f32;
            let max_days = s
                .entry_rules
                .get("max_days_since_earnings")
                .and_then(|v| v.as_i64())
                .unwrap_or(5) as i32;
            match pead_eligible_symbols(pool, &symbols, min_surprise, max_days).await {
                Ok(filtered) if !filtered.is_empty() => filtered,
                Ok(_) => {
                    tracing::debug!(strategy = %s.id, "no symbols with recent positive surprise");
                    continue;
                }
                Err(e) => {
                    tracing::warn!(strategy = %s.id, error = %e, "pead_eligible_symbols failed");
                    continue;
                }
            }
        } else {
            symbols
        };
        // Build the right sink per-strategy via the dispatcher:
        // alpaca → AlpacaSink (real REST); tradier → TradierSink (real REST);
        // ibkr/td/tastytrade → IntegrationPendingSink (rejects).
        let sink_box = match crate::broker_dispatcher::sink_for_strategy(pool, s).await {
            Ok(b) => b,
            Err(e) => {
                tracing::warn!(strategy = %s.id, error = %e, "broker_dispatcher failed");
                continue;
            }
        };
        match drive_strategy(pool, sink_box.as_ref(), s, interval, &symbols, event_sink).await {
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
pub async fn run_loop(pool: PgPool, event_sink: Option<EventSink>) -> ! {
    loop {
        let now = Utc::now();
        let next = next_boundary(now, BarInterval::S10);
        let wait = (next - now).num_milliseconds().max(0) as u64;
        tokio::time::sleep(Duration::from_millis(wait)).await;
        let processed = tick(&pool, Utc::now(), event_sink.as_ref()).await;
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

async fn symbol_universe(pool: &PgPool, s: &AlgoStrategy) -> Result<Vec<String>, anyhow::Error> {
    // Pairs / stat-arb: universe is hard-wired by the strategy's own
    // entry_rules (the legs of the spread). Ignore watchlist / autoscan.
    if s.strategy_type == "pairs" {
        let a = s
            .entry_rules
            .get("symbol_a")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let b = s
            .entry_rules
            .get("symbol_b")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if a.is_empty() || b.is_empty() {
            return Ok(Vec::new());
        }
        return Ok(vec![a, b]);
    }
    match s.universe_mode.as_str() {
        "watchlist" => {
            let Some(watchlist_id) = s.watchlist_id else {
                return Ok(Vec::new());
            };
            crate::watchlists::symbols(pool, watchlist_id).await
        }
        "autoscan" => {
            let n = s.autoscan_top_n.max(1) as i64;
            autoscan_topn(pool, n).await
        }
        _ => Ok(Vec::new()),
    }
}

/// For strategy_type='pead' the universe is further filtered to
/// symbols whose most recent `earnings_cal` event posted a positive
/// surprise above `min_surprise_pct` within the last
/// `max_days_since_earnings` days. The published PEAD anomaly only
/// holds in this narrow post-announcement window; without the gate
/// the strategy would fire on every "stock making new highs above
/// SMA" which is just a momentum re-implementation.
pub async fn pead_eligible_symbols(
    pool: &PgPool,
    symbols: &[String],
    min_surprise_pct: f32,
    max_days_since: i32,
) -> Result<Vec<String>, anyhow::Error> {
    if symbols.is_empty() {
        return Ok(Vec::new());
    }
    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT DISTINCT symbol FROM earnings_events
          WHERE symbol = ANY($1)
            AND surprise_pct IS NOT NULL
            AND surprise_pct >= $2
            AND earnings_date >= current_date - $3::int
          ORDER BY symbol",
    )
    .bind(symbols)
    .bind(min_surprise_pct)
    .bind(max_days_since)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|(s,)| s).collect())
}

/// Top-N symbols by traded volume in the last 30 minutes of 10s bars
/// (current activity proxy — high turnover ≈ high RVOL right now).
/// Cheaper than a true rolling-RVOL z-score and good enough for
/// strategies that want "whatever's hot today". The intraday tick
/// worker keeps `price_bars` fresh, so this query reflects truly
/// real-time activity rather than yesterday's close.
pub async fn autoscan_topn(pool: &PgPool, top_n: i64) -> Result<Vec<String>, anyhow::Error> {
    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT symbol
           FROM price_bars
          WHERE interval = '10s'::bar_interval_t
            AND bar_time >= now() - INTERVAL '30 minutes'
          GROUP BY symbol
          ORDER BY SUM(volume) DESC
          LIMIT $1",
    )
    .bind(top_n)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|(s,)| s).collect())
}

async fn drive_strategy(
    pool: &PgPool,
    sink: &dyn BrokerSink,
    strategy: &AlgoStrategy,
    interval: BarInterval,
    symbols: &[String],
    event_sink: Option<&EventSink>,
) -> Result<usize, anyhow::Error> {
    let Some(run) = algo::get_open_run(pool, strategy.id).await? else {
        // No open run → strategy is enabled but not actively running.
        // The runner does not auto-start runs; the UI's "start" button
        // is the explicit gate.
        return Ok(0);
    };
    let equity = account_equity(pool, strategy).await.unwrap_or(100_000.0);
    let open_positions: i64 = open_position_count(pool, strategy.account_id)
        .await
        .unwrap_or(0);
    let mut driven = 0usize;

    // Multi-symbol path (pairs / stat-arb): the symbols vec here is the
    // strategy's required_symbols, not the universe. Fetch bars for each
    // leg and call the multi evaluator once.
    if strategy.strategy_type == "pairs" {
        let mut bars_by_symbol = std::collections::HashMap::new();
        for symbol in symbols {
            let bars = fetch_recent_bars(pool, symbol, interval, BAR_WINDOW_SIZE).await?;
            if bars.is_empty() {
                continue;
            }
            bars_by_symbol.insert(symbol.clone(), bars);
        }
        if !bars_by_symbol.is_empty() {
            match algo_engine::process_bar_window_multi(
                pool,
                sink,
                strategy,
                run.id,
                &bars_by_symbol,
                equity,
                open_positions,
                event_sink,
            )
            .await
            {
                Ok(_) => driven = 1,
                Err(e) => tracing::debug!(
                    strategy = %strategy.id, error = %e,
                    "process_bar_window_multi non-fatal"
                ),
            }
        }
        let _ = algo::increment_run_counter(
            pool,
            run.id,
            algo::RunCounter::BarsProcessed,
            driven as i64,
        )
        .await;
        return Ok(driven);
    }

    for symbol in symbols {
        let bars = fetch_recent_bars(pool, symbol, interval, BAR_WINDOW_SIZE).await?;
        if bars.is_empty() {
            continue;
        }
        match algo_engine::process_bar_window(
            pool,
            sink,
            strategy,
            run.id,
            &bars,
            equity,
            open_positions,
            event_sink,
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
    let _ =
        algo::increment_run_counter(pool, run.id, algo::RunCounter::BarsProcessed, driven as i64)
            .await;
    Ok(driven)
}

/// Account equity = starting_equity from strategy.risk_gates JSON
/// (default 100_000) + sum of closed-trade net_pnl on the strategy's
/// bound account. Doesn't subtract mark-to-market on open positions
/// yet — that needs a quote lookup per held symbol; the first iter
/// approximates equity by realized P&L only.
pub async fn account_equity(pool: &PgPool, strategy: &AlgoStrategy) -> Result<f64, anyhow::Error> {
    let starting = strategy
        .risk_gates
        .get("starting_equity")
        .and_then(|v| v.as_f64())
        .unwrap_or(100_000.0);
    let (realized,): (Option<f64>,) = sqlx::query_as(
        "SELECT COALESCE(SUM(net_pnl), 0)::float8
           FROM trades
          WHERE account_id = $1 AND status = 'closed' AND net_pnl IS NOT NULL",
    )
    .bind(strategy.account_id)
    .fetch_one(pool)
    .await?;
    Ok(starting + realized.unwrap_or(0.0))
}

/// Count of currently-open trades against the account. Feeds the
/// max_concurrent_positions risk gate so a strategy that hit its cap
/// stops opening new ones until something closes.
pub async fn open_position_count(
    pool: &PgPool,
    account_id: uuid::Uuid,
) -> Result<i64, anyhow::Error> {
    let (n,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM trades WHERE account_id = $1 AND status = 'open'")
            .bind(account_id)
            .fetch_one(pool)
            .await?;
    Ok(n)
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
        .map(
            |(symbol, iv, bar_time, open, high, low, close, volume, source)| PriceBar {
                symbol,
                interval: parse_interval(&iv),
                bar_time,
                open,
                high,
                low,
                close,
                volume,
                source,
            },
        )
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
