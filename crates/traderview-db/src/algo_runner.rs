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
use crate::algo_engine::{self, BrokerSink, EngineEvent, EventSink};
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
        let at_boundary = is_boundary(now, interval);
        // Per-tick heartbeat — fires every 10s, even on M1 strategies
        // that only formally evaluate once a minute. Lets the user see
        // the engine is alive and what the runner knows right now
        // without waiting 60s between status updates.
        if let Some(sink) = event_sink {
            let live_subs = crate::live_ticks::global().subs_len().await;
            let next = next_boundary(now, interval);
            let secs = (next - now).num_seconds().max(0);
            let open_run = algo::get_open_run(pool, s.id).await.ok().flatten();
            let (bars_processed, signals_emitted) = open_run
                .as_ref()
                .map(|r| (r.bars_processed, r.signals_emitted))
                .unwrap_or((0, 0));
            sink(EngineEvent::Heartbeat {
                strategy_id: s.id,
                universe_size: 0, // populated below after universe is resolved on boundary ticks
                subscribed_live: live_subs,
                bars_processed,
                signals_emitted,
                seconds_to_next_eval: secs,
            });
        }
        // Only drive the strategy on a bar boundary FOR ITS OWN TIMEFRAME —
        // a 1m strategy ignores the 10s ticks the loop also fires on.
        if !at_boundary {
            continue;
        }
        let symbols = match symbol_universe(pool, s).await {
            Ok(syms) => syms,
            Err(e) => {
                tracing::warn!(strategy = %s.id, error = %e, "symbol_universe failed");
                emit_skip(event_sink, s.id, format!("universe_error: {e}"));
                continue;
            }
        };
        if symbols.is_empty() {
            let reason = match s.universe_mode.as_str() {
                "watchlist" if s.watchlist_id.is_none() => {
                    "no_watchlist_id — pick a watchlist on the strategy".to_string()
                }
                "watchlist" => format!("watchlist {} has no symbols", s.watchlist_id.unwrap()),
                "autoscan" => {
                    // Empty here means no symbols have bars at the
                    // strategy's interval — almost always the live-
                    // tick worker isn't writing this resolution.
                    let tf = parse_timeframe(&s.timeframe);
                    format!(
                        "autoscan empty: no symbols with recent {} bars (live tick worker not writing this resolution — check Settings → Data sources for a configured Finnhub/Alpaca/Polygon key, and confirm watchlist symbols are valid for this market)",
                        tf.label()
                    )
                }
                other => format!("unknown universe_mode={other}"),
            };
            tracing::info!(strategy = %s.id, name = %s.name, reason = %reason, "algo_runner skip");
            emit_skip(event_sink, s.id, reason);
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
            tracing::info!(processed, "algo_runner tick");
        }
    }
}

fn emit_skip(event_sink: Option<&EventSink>, strategy_id: uuid::Uuid, reason: String) {
    if let Some(sink) = event_sink {
        sink(EngineEvent::TickSkipped {
            strategy_id,
            reason,
        });
    }
}

fn emit_evaluated(
    event_sink: Option<&EventSink>,
    strategy_id: uuid::Uuid,
    symbol: String,
    bars: usize,
) {
    if let Some(sink) = event_sink {
        sink(EngineEvent::BarEvaluated {
            strategy_id,
            symbol,
            bars,
        });
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
            let syms = crate::watchlists::symbols(pool, watchlist_id).await?;
            // Same side-effect as autoscan — make sure the live-tick
            // worker is streaming this strategy's watchlist symbols.
            // The boot-time push covers the union of ALL user
            // watchlists, but if the user added symbols after boot the
            // worker wouldn't pick them up until the next set_symbols
            // call elsewhere. Explicit ensure here closes that gap.
            let store = crate::live_ticks::global();
            if store.has_any_provider().await {
                if let Err(e) = store.ensure_subscribed(syms.clone()).await {
                    tracing::warn!(error = %e, "ensure_subscribed for watchlist symbols");
                }
            }
            Ok(syms)
        }
        "autoscan" => {
            let n = s.autoscan_top_n.max(1) as i64;
            // entry_rules.asset_class = "crypto" → 24/7 universe
            // (overnight / weekend testing). Default: equity (RTH).
            let class = AssetClass::from_entry_rules(&s.entry_rules);
            let picks = autoscan_topn_class(pool, n, parse_timeframe(&s.timeframe), class).await?;
            // Side-effect: make sure the live-tick worker is streaming
            // every autoscan pick. Without this, autoscan picks from the
            // catalog but the WS only ever subscribed to the user's
            // watchlist union — so SPY/MSFT/etc. never get fresh 1m
            // bars, and the strategy sits in `no_bars` forever.
            let store = crate::live_ticks::global();
            if store.has_any_provider().await {
                if let Err(e) = store.ensure_subscribed(picks.clone()).await {
                    tracing::warn!(error = %e, "ensure_subscribed for autoscan picks");
                }
            }
            Ok(picks)
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
/// Seed list — guaranteed-tradeable mega-caps used when the symbols
/// catalog is empty (fresh install before the Finnhub seed runs) or
/// when no historical volume data exists yet (first-ever boot before
/// any user fetched price history). Real autoscan goes through the
/// `symbols` table — 24k+ US equities ranked by historical volume.
const AUTOSCAN_SEED: &[&str] = &[
    "AAPL", "MSFT", "NVDA", "GOOGL", "META", "AMZN", "TSLA", "AMD", "SPY", "QQQ",
];

/// Liquid Alpaca crypto pairs in the canonical `BASE/QUOTE` format
/// Alpaca's v1beta3 crypto WS expects. Used when a strategy opts in
/// to `entry_rules.asset_class = "crypto"` so the algo trader has a
/// universe that streams 24/7 — perfect for after-hours / weekend
/// development + paper testing when equities are silent.
const AUTOSCAN_CRYPTO_UNIVERSE: &[&str] = &[
    "BTC/USD",
    "ETH/USD",
    "SOL/USD",
    "AVAX/USD",
    "MATIC/USD",
    "DOGE/USD",
    "SHIB/USD",
    "LTC/USD",
    "BCH/USD",
    "LINK/USD",
    "UNI/USD",
    "AAVE/USD",
    "COMP/USD",
    "SUSHI/USD",
    "YFI/USD",
    "MKR/USD",
    "GRT/USD",
    "BAT/USD",
    "CRV/USD",
    "XRP/USD",
    "ADA/USD",
    "DOT/USD",
    "TRX/USD",
    "XLM/USD",
    "ALGO/USD",
];

/// Autoscan universe — pulls top-N most liquid US equities + ETFs from
/// the `symbols` catalog (24k+ rows seeded by Finnhub), ranked by
/// historical traded volume so the mega-caps surface first. The
/// caller then wires every pick to the live tick worker so they
/// start streaming and the strategy can evaluate live bars within
/// a minute.
///
/// Ranking:
///   1. LEFT JOIN `symbols` to `price_bars` on max-volume any interval.
///   2. ORDER BY volume DESC NULLS LAST, then `length(symbol)` ASC so
///      brand-new tickers without history fall to a stable order
///      (shorter symbols are usually older + more liquid; this is a
///      tiebreaker, not the primary sort).
///   3. Filter to Common Stock + ETP types so warrants / rights /
///      preference shares / GDRs don't pollute the candidate set.
///
/// Provider-side cap awareness:
///   * Alpaca IEX free tier: 30 symbols per connection (the live
///     tick worker chunks into 500-symbol workers, so multiple
///     connections work for SIP; with IEX you'll hit the cap fast).
///   * Polygon Stocks Starter+: thousands.
///   * Finnhub free: 25 symbols per connection, multiple connections.
///
/// The runner doesn't enforce the cap — pick whatever top_n the
/// strategy asks for and let the provider negotiate. Excess picks
/// may stay "no bars" until a slot frees up.
///
/// `interval` is accepted but currently unused — kept in the
/// signature so a future "pick only symbols already streaming the
/// strategy's resolution" optimisation can drop in without callers
/// changing.
/// Asset class the strategy's autoscan should pick from.
///   * Equity → query the `symbols` catalog ranked by historical volume.
///   * Crypto → curated 25-pair `BASE/QUOTE` list. Trades 24/7.
///
/// Selected via `entry_rules.asset_class` ("crypto" picks Crypto;
/// anything else falls through to Equity for backward compatibility).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetClass {
    Equity,
    Crypto,
}

impl AssetClass {
    pub fn from_entry_rules(v: &serde_json::Value) -> Self {
        match v.get("asset_class").and_then(|s| s.as_str()) {
            Some("crypto") => Self::Crypto,
            _ => Self::Equity,
        }
    }
}

pub async fn autoscan_topn(
    pool: &PgPool,
    top_n: i64,
    interval: BarInterval,
) -> Result<Vec<String>, anyhow::Error> {
    autoscan_topn_class(pool, top_n, interval, AssetClass::Equity).await
}

pub async fn autoscan_topn_class(
    pool: &PgPool,
    top_n: i64,
    _interval: BarInterval,
    class: AssetClass,
) -> Result<Vec<String>, anyhow::Error> {
    if class == AssetClass::Crypto {
        // Crypto universe is curated + small — take the first `top_n`
        // from the hardcoded list. Symbols are already in BASE/QUOTE
        // form so the Alpaca crypto WS subscribes directly.
        let cap = top_n.max(1) as usize;
        return Ok(AUTOSCAN_CRYPTO_UNIVERSE
            .iter()
            .take(cap)
            .map(|s| (*s).to_string())
            .collect());
    }
    let cap = top_n.max(1);
    // Query the catalog — symbols with the most historical volume
    // come first. The MAX() aggregate scoops the largest single bar
    // (a proxy for "this thing actually trades") rather than SUM()
    // which over-weights symbols with the most historical bars.
    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT s.symbol
           FROM symbols s
           LEFT JOIN (
             SELECT symbol, MAX(volume) AS max_vol
               FROM price_bars
              GROUP BY symbol
           ) pb ON pb.symbol = s.symbol
          WHERE s.exchange = 'US'
            AND s.asset_class = 'stock'
            AND (s.type IS NULL OR s.type IN ('Common Stock', 'ETP'))
          ORDER BY pb.max_vol DESC NULLS LAST, length(s.symbol) ASC, s.symbol ASC
          LIMIT $1",
    )
    .bind(cap)
    .fetch_all(pool)
    .await?;
    let mut picks: Vec<String> = rows.into_iter().map(|(s,)| s).collect();
    // Catalog empty (fresh install, Finnhub seed hasn't run) — fall
    // back to the hardcoded mega-cap seed so the user always gets
    // SOMETHING tradeable.
    if picks.is_empty() {
        picks = AUTOSCAN_SEED
            .iter()
            .take(cap as usize)
            .map(|s| (*s).to_string())
            .collect();
    }
    Ok(picks)
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
        // is the explicit gate. Emit a skip so the stdout pane shows
        // "test2 skipped: no_open_run — click Start to begin a run".
        emit_skip(event_sink, strategy.id, "no_open_run — click Start".into());
        return Ok(0);
    };
    let equity = account_equity(pool, strategy).await.unwrap_or(100_000.0);
    let mut open_positions: i64 = open_position_count(pool, strategy.account_id)
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

    // Track per-tick "no bars" misses so we can collapse the noise:
    // 25 symbols × per-symbol SKIP every minute = log spam. One summary
    // skip per tick listing the count + the first few symbols tells
    // the user what's happening without burying the EVAL heartbeats.
    let mut no_bars: Vec<String> = Vec::new();

    for symbol in symbols {
        let bars = fetch_recent_bars(pool, symbol, interval, BAR_WINDOW_SIZE).await?;
        if bars.is_empty() {
            no_bars.push(symbol.clone());
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
            Ok(Some(_)) => {
                // Optimistically count this entry against the cap for the
                // rest of the universe loop. Without this, a 25-symbol
                // universe with a 5-position cap and a strong-trend bar
                // would submit 25 entries — process_bar_window checked
                // the SAME stale `open_positions` snapshot 25 times.
                // process_bar_window returns Ok(Some) for both entries
                // AND exits; the exit case incorrectly increments the
                // cap counter here, but the cap is a CEILING not a floor
                // so over-counting is safe (it just makes subsequent
                // entries this tick skip — desirable for a thin window).
                open_positions += 1;
                driven += 1;
            }
            Ok(None) => {
                // No signal but the strategy WAS evaluated — emit a
                // heartbeat so the stdout pane proves the engine is
                // alive even on quiet bars.
                emit_evaluated(event_sink, strategy.id, symbol.clone(), bars.len());
                driven += 1;
            }
            Err(e) => {
                tracing::info!(
                    strategy = %strategy.id, symbol, error = %e,
                    "process_bar_window non-fatal"
                );
                // Risk-gate fires get an audit row so gate configs can
                // be tuned from data; infra errors don't.
                if let Some(gate) = e.gate_name() {
                    if let Err(rec) =
                        algo::record_gate_fire(pool, strategy.id, gate, &e.to_string()).await
                    {
                        tracing::warn!(strategy = %strategy.id, error = %rec, "gate-fire audit failed");
                    }
                }
                emit_skip(event_sink, strategy.id, format!("{symbol}: {e}"));
            }
        }
    }
    // One summary skip per tick — first 5 symbols + "and N more" when
    // there are more, so the user sees what's missing without 25 lines.
    if !no_bars.is_empty() {
        let head: Vec<_> = no_bars.iter().take(5).cloned().collect();
        // Escalation: ONLY show the "stuck" warning if no symbols
        // evaluated this tick AND no symbols evaluated in earlier
        // ticks (driven > 0 means some pair of the universe HAS
        // bars). Mixed universes are normal — thin / new pairs lag
        // until enough 10s buckets accumulate.
        let mins_since_start = (chrono::Utc::now() - run.started_at).num_minutes().max(0);
        let tail = if driven > 0 {
            format!(
                "other {} universe members already evaluating — these are still warming up (need ~{} bar window of 10s buckets)",
                interval.label(),
                interval.seconds() / 10,
            )
        } else if mins_since_start < 3 {
            format!(
                "first {} window after autoscan; symbols were just subscribed to live tick worker, bars will fill in over the next minute or two",
                interval.label(),
            )
        } else {
            format!(
                "still no bars after {} min — live tick worker is stuck (Alpaca/Polygon/Finnhub WS auth failure, connection-limit collision, or stale desktop binary still using old mmap after rebuild). Check ~/Library/Application Support/com.menketechnologies.traderview/traderview.log for `alpaca WS error` frames + restart the app",
                mins_since_start,
            )
        };
        let reason = if no_bars.len() > head.len() {
            format!(
                "no_bars ({}/{} of universe): {} … and {} more — {}",
                no_bars.len(),
                no_bars.len() + driven,
                head.join(", "),
                no_bars.len() - head.len(),
                tail,
            )
        } else {
            format!(
                "no_bars ({}/{} of universe): {} — {}",
                no_bars.len(),
                no_bars.len() + driven,
                head.join(", "),
                tail,
            )
        };
        emit_skip(event_sink, strategy.id, reason);
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
    // First try the requested interval directly — that's the fast path
    // when Yahoo / Polygon already cached bars for this resolution.
    let direct = read_bars_at(pool, symbol, interval, limit).await?;
    if !direct.is_empty() {
        return Ok(direct);
    }
    // Live-streamed symbols only have 10s buckets (live_ticks::feed_bucket
    // is the only writer outside the Yahoo / Polygon fetchers). When the
    // strategy asks for 1m / 5m / 15m / 1h and the cache is empty for
    // that resolution, aggregate from 10s on the fly so the strategy
    // can evaluate as soon as ~60 seconds of streaming has happened.
    // For 10s callers there's nothing to roll up — return the empty
    // result. For 1d/1w the gap is too large; same — return empty.
    let factor: usize = match interval {
        BarInterval::M1 => 6,
        BarInterval::M5 => 30,
        BarInterval::M15 => 90,
        BarInterval::H1 => 360,
        _ => return Ok(Vec::new()),
    };
    // Pull enough 10s rows to cover the requested 1m / etc. window.
    let s10_rows = read_bars_at(pool, symbol, BarInterval::S10, limit * factor as i64).await?;
    if s10_rows.is_empty() {
        return Ok(Vec::new());
    }
    Ok(rollup_s10(s10_rows, interval, factor))
}

/// Single-resolution read used by both the fast path and the rollup
/// fallback. No reverse / mapping logic duplicated.
async fn read_bars_at(
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

/// Aggregate N consecutive 10s bars into one bar at `target` interval.
/// Group by floor(bar_time_secs / target_secs) so the resulting bars
/// align to clean wall-clock boundaries (00:00, 00:01, …) the same
/// way Yahoo / Polygon would have written them. Open = first bar's
/// open, close = last bar's close, high/low = max/min across group,
/// volume = sum.
fn rollup_s10(s10_bars: Vec<PriceBar>, target: BarInterval, _factor: usize) -> Vec<PriceBar> {
    use std::collections::BTreeMap;
    let target_secs = target.seconds();
    let mut buckets: BTreeMap<i64, Vec<PriceBar>> = BTreeMap::new();
    for b in s10_bars {
        let key = (b.bar_time.timestamp() / target_secs) * target_secs;
        buckets.entry(key).or_default().push(b);
    }
    // Drop the last bucket only if its target window is STILL IN PROGRESS
    // (i.e. wall-clock time hasn't crossed `key + target_secs`). Earlier
    // logic dropped any bucket with fewer than `factor` constituents,
    // which was wrong for sparse markets: a thinly-traded symbol
    // legitimately produces 3 ten-second bars per minute (because the
    // other 3 windows had zero trades), and the resulting M1 bar's
    // OHLCV is still mathematically correct — we just have gaps that
    // had no volume. Dropping those mathematically-valid bars left
    // crypto strategies in `no_bars` forever during quiet sessions.
    let now_sec = chrono::Utc::now().timestamp();
    let mut out: Vec<PriceBar> = Vec::with_capacity(buckets.len());
    for (key, group) in buckets {
        // Window not yet closed → still accumulating, skip.
        if now_sec < key + target_secs {
            continue;
        }
        let first = group.first().expect("non-empty group");
        let last = group.last().expect("non-empty group");
        let high = group.iter().map(|b| b.high).max().unwrap_or(first.high);
        let low = group.iter().map(|b| b.low).min().unwrap_or(first.low);
        let volume = group.iter().map(|b| b.volume).sum();
        out.push(PriceBar {
            symbol: first.symbol.clone(),
            interval: target,
            bar_time: chrono::DateTime::<Utc>::from_timestamp(key, 0).unwrap_or(first.bar_time),
            open: first.open,
            high,
            low,
            close: last.close,
            volume,
            source: "rollup_s10".into(),
        });
    }
    out
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

    fn s10_bar(secs: i64, o: &str, h: &str, l: &str, c: &str, v: u64) -> PriceBar {
        use std::str::FromStr;
        PriceBar {
            symbol: "TEST".into(),
            interval: BarInterval::S10,
            bar_time: Utc.timestamp_opt(secs, 0).unwrap(),
            open: Decimal::from_str(o).unwrap(),
            high: Decimal::from_str(h).unwrap(),
            low: Decimal::from_str(l).unwrap(),
            close: Decimal::from_str(c).unwrap(),
            volume: Decimal::from(v),
            source: "test".into(),
        }
    }

    #[test]
    fn rollup_s10_to_m1_uses_first_open_last_close_max_high_min_low_sum_volume() {
        // 6 ten-second bars covering one full minute [60, 120).
        let bars = vec![
            s10_bar(60, "100", "101", "99", "100.5", 100),
            s10_bar(70, "100.5", "102", "100.2", "101.7", 200),
            s10_bar(80, "101.7", "103.5", "101", "103.0", 150),
            s10_bar(90, "103.0", "104", "102.0", "102.5", 300),
            s10_bar(100, "102.5", "102.8", "101.0", "101.2", 50),
            s10_bar(110, "101.2", "103.0", "100.8", "102.0", 250),
        ];
        let m1 = rollup_s10(bars, BarInterval::M1, 6);
        assert_eq!(m1.len(), 1, "complete bucket should produce one M1");
        let b = &m1[0];
        assert_eq!(b.bar_time.timestamp(), 60, "aligns to floor(t/60)*60");
        assert_eq!(b.interval, BarInterval::M1);
        assert_eq!(b.open.to_string(), "100");
        assert_eq!(b.close.to_string(), "102.0");
        assert_eq!(b.high.to_string(), "104");
        assert_eq!(b.low.to_string(), "99");
        assert_eq!(b.volume.to_string(), "1050");
        assert_eq!(b.source, "rollup_s10");
    }

    #[test]
    fn asset_class_picks_crypto_when_entry_rules_says_crypto() {
        let r = serde_json::json!({ "asset_class": "crypto" });
        assert_eq!(AssetClass::from_entry_rules(&r), AssetClass::Crypto);
    }

    #[test]
    fn asset_class_defaults_to_equity_when_absent_or_other() {
        assert_eq!(
            AssetClass::from_entry_rules(&serde_json::json!({})),
            AssetClass::Equity
        );
        assert_eq!(
            AssetClass::from_entry_rules(&serde_json::json!({ "asset_class": "fx" })),
            AssetClass::Equity,
            "unknown asset_class falls back to equity, not panic"
        );
        assert_eq!(
            AssetClass::from_entry_rules(&serde_json::json!({ "asset_class": "equity" })),
            AssetClass::Equity
        );
    }

    #[test]
    fn rollup_keeps_closed_buckets_even_when_sparse() {
        // Synthetic timestamps far in the past so both [60,120) and
        // [120,180) windows are LONG closed by wall-clock time. The
        // rollup must NOT drop sparse buckets — a thinly-traded
        // symbol with 3 ten-second bars per minute still produces a
        // mathematically-correct M1 OHLCV. Earlier behaviour required
        // 6 constituents per bucket and dropped every sparse window,
        // leaving crypto strategies in no_bars forever overnight.
        let bars = vec![
            s10_bar(60, "100", "100", "100", "100", 10),
            s10_bar(80, "100", "100", "100", "100", 10),
            s10_bar(100, "100", "100", "100", "100", 10),
            s10_bar(120, "101", "101", "101", "101", 10),
            s10_bar(140, "101", "101", "101", "101", 10),
        ];
        let m1 = rollup_s10(bars, BarInterval::M1, 6);
        assert_eq!(m1.len(), 2, "BOTH closed buckets survive");
        assert_eq!(m1[0].bar_time.timestamp(), 60);
        assert_eq!(m1[1].bar_time.timestamp(), 120);
    }
}
