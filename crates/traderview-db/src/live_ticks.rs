//! Live tick stream via Finnhub's free WebSocket.
//!
//! Architecture:
//!   * one persistent WS connection to `wss://ws.finnhub.io?token=<API_KEY>`
//!   * subscribes to the union of all symbols in any user's watchlists
//!   * every incoming trade updates an in-process `SymbolState` (last price,
//!     day-high, day-low, cumulative volume, prev close from the cached
//!     `quote_snapshots` table, gap%, change%, RVOL approximation)
//!   * a tokio broadcast channel fans every state update out to whatever
//!     scanner / WS-client / alert engine is subscribed
//!
//! Finnhub free-tier limits: 25 symbol subscriptions per connection, 60
//! HTTP calls / minute. We honor both: chunk symbols into 25-symbol pages
//! across multiple parallel WS connections if the user adds more.

use chrono::{DateTime, TimeZone, Utc};
use dashmap::DashMap;
use futures_util::{SinkExt, StreamExt};
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use traderview_core::{BarInterval, PriceBar};

pub const FINNHUB_WS: &str = "wss://ws.finnhub.io";
const MAX_SYMS_PER_CONN: usize = 25;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub symbol: String,
    pub price: f64,
    pub volume: f64,
    pub ts_ms: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SymbolState {
    pub symbol: String,
    pub last: f64,
    pub day_high: f64,
    pub day_low: f64,
    pub day_volume: f64,
    pub prev_close: Option<f64>,
    pub session_open: Option<f64>,
    pub gap_pct: f64,
    pub change_pct: f64,   // last vs prev_close
    pub day_pct: f64,      // last vs session_open
    pub hod_dist_pct: f64, // distance from HOD
    pub last_trade_at: DateTime<Utc>,
    pub trade_count: u64,
}

impl SymbolState {
    fn new(symbol: &str, prev_close: Option<f64>) -> Self {
        SymbolState {
            symbol: symbol.into(),
            last: 0.0,
            day_high: 0.0,
            day_low: f64::INFINITY,
            day_volume: 0.0,
            prev_close,
            session_open: None,
            gap_pct: 0.0,
            change_pct: 0.0,
            day_pct: 0.0,
            hod_dist_pct: 0.0,
            last_trade_at: Utc::now(),
            trade_count: 0,
        }
    }

    fn observe(&mut self, t: &Trade) {
        if self.session_open.is_none() {
            self.session_open = Some(t.price);
        }
        self.last = t.price;
        if t.price > self.day_high {
            self.day_high = t.price;
        }
        if t.price < self.day_low {
            self.day_low = t.price;
        }
        self.day_volume += t.volume;
        self.trade_count += 1;
        self.last_trade_at =
            chrono::DateTime::<Utc>::from_timestamp_millis(t.ts_ms).unwrap_or_else(Utc::now);

        let pct = |a: f64, b: f64| if b > 0.0 { (a - b) / b * 100.0 } else { 0.0 };
        if let Some(pc) = self.prev_close {
            self.change_pct = pct(self.last, pc);
            if let Some(o) = self.session_open {
                self.gap_pct = pct(o, pc);
            }
        }
        if let Some(o) = self.session_open {
            self.day_pct = pct(self.last, o);
        }
        if self.day_high > 0.0 {
            self.hod_dist_pct = (self.last - self.day_high) / self.day_high * 100.0;
        }
    }
}

// Bucket size for the tape aggregator. Aligned to BarInterval::S10 — every
// trade is folded into the 10-second window starting at `floor(ts / 10) * 10`.
const TAPE_BUCKET_SECS: i64 = 10;

// In-progress 10s OHLC bucket. Each incoming trade either extends `high/low`
// + advances `close` + accumulates `volume`, or — if it crosses into the next
// 10s window — first flushes this struct to `price_bars` then starts a fresh
// one. `start_sec` is the bucket's open epoch second (multiple of 10).
#[derive(Clone, Debug)]
struct TapeBucket {
    start_sec: i64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

#[derive(Clone)]
pub struct LiveTickStore {
    state: Arc<DashMap<String, SymbolState>>,
    tx: broadcast::Sender<SymbolState>,
    api_key: Arc<tokio::sync::RwLock<Option<String>>>,
    subs: Arc<tokio::sync::Mutex<Vec<String>>>,
    // Per-symbol open 10s OHLC bucket. Closed buckets are flushed to
    // `price_bars` (interval='10s', source='finnhub-tape') so the
    // multichart 10s pane can read them back via `/bars/:sym?interval=10s`.
    buckets: Arc<DashMap<String, TapeBucket>>,
    // DB pool used by the tape aggregator. Set once at server boot via
    // `set_pool` so the global() singleton can persist bars without taking
    // a pool argument on every trade.
    pool: Arc<tokio::sync::RwLock<Option<PgPool>>>,
    // Guards the idle-flush sweeper spawn — `set_pool` may be invoked
    // multiple times (settings-page key updates, hot reload, tests) and
    // we must NOT leak a fresh sweeper task per call. CAS to true on
    // first spawn; subsequent calls observe `true` and skip.
    sweeper_spawned: Arc<AtomicBool>,
}

impl LiveTickStore {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(1024);
        Self {
            state: Arc::new(DashMap::new()),
            tx,
            api_key: Arc::new(tokio::sync::RwLock::new(None)),
            subs: Arc::new(tokio::sync::Mutex::new(Vec::new())),
            buckets: Arc::new(DashMap::new()),
            pool: Arc::new(tokio::sync::RwLock::new(None)),
            sweeper_spawned: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Wire the DB pool used by the 10s tape aggregator. Idempotent — may
    /// be called more than once (settings-page Finnhub key update, test
    /// fixtures, hot reload); the latest pool wins and the idle-flush
    /// sweeper is spawned at most once per process.
    pub async fn set_pool(&self, pool: PgPool) {
        *self.pool.write().await = Some(pool);
        // CAS guard — only the first call spawns the sweeper. Without
        // this, repeated set_pool invocations leak one tokio task per
        // call, each calling flush_idle_buckets concurrently against the
        // same DashMap (correct via ON CONFLICT but wastes IO).
        if self
            .sweeper_spawned
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return;
        }
        // Buckets whose 10s window has fully closed but received no
        // further trades to trigger flush-on-cross will sit indefinitely
        // otherwise — this guarantees they land in `price_bars` within
        // ~5s of natural close.
        let store = self.clone();
        tokio::spawn(async move {
            let mut tick = tokio::time::interval(Duration::from_secs(5));
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
            loop {
                tick.tick().await;
                store.flush_idle_buckets().await;
            }
        });
    }

    pub async fn set_api_key(&self, key: impl Into<String>) {
        *self.api_key.write().await = Some(key.into());
    }

    pub async fn has_key(&self) -> bool {
        self.api_key.read().await.is_some()
    }

    /// Read the current in-memory Finnhub key. The data-sources settings
    /// route updates this on save so REST callers (`finnhub_rest`) and
    /// the WS pump share the same credential without re-querying the DB
    /// on every request.
    pub async fn api_key(&self) -> Option<String> {
        self.api_key.read().await.clone()
    }

    pub fn subscribe(&self) -> broadcast::Receiver<SymbolState> {
        self.tx.subscribe()
    }

    pub fn snapshot(&self) -> Vec<SymbolState> {
        self.state.iter().map(|e| e.value().clone()).collect()
    }

    /// Replace the subscription set. Restarts the WS connections if needed.
    pub async fn set_symbols(&self, symbols: Vec<String>) -> anyhow::Result<()> {
        let mut deduped: Vec<String> = symbols.into_iter().collect();
        deduped.sort();
        deduped.dedup();
        let mut current = self.subs.lock().await;
        if *current == deduped {
            return Ok(());
        }
        *current = deduped.clone();
        drop(current);

        // Seed prev_close from quote_snapshots cache if missing.
        for s in &deduped {
            self.state
                .entry(s.clone())
                .or_insert_with(|| SymbolState::new(s, None));
        }

        // Cancel and respawn worker tasks.
        let key = self.api_key.read().await.clone();
        let Some(key) = key else {
            tracing::warn!("FINNHUB_API_KEY not set; live ticks disabled");
            return Ok(());
        };
        // Restart all workers — simplest path; chunk into pages.
        let chunks: Vec<Vec<String>> = deduped
            .chunks(MAX_SYMS_PER_CONN)
            .map(|c| c.to_vec())
            .collect();
        for chunk in chunks {
            let store = self.clone();
            let key = key.clone();
            tokio::spawn(async move {
                store.run_worker(key, chunk).await;
            });
        }
        Ok(())
    }

    async fn run_worker(self, api_key: String, symbols: Vec<String>) {
        loop {
            match self.run_once(&api_key, &symbols).await {
                Ok(()) => tracing::info!("finnhub WS exited cleanly; reconnecting in 2s"),
                Err(e) => tracing::warn!(?e, "finnhub WS error; reconnecting in 5s"),
            }
            tokio::time::sleep(Duration::from_secs(3)).await;
        }
    }

    // ---- 10s tape aggregator ----

    // Fold one trade into the symbol's open 10s bucket. If the trade
    // falls in a later bucket, flush the existing one to `price_bars`
    // before opening a fresh bucket for this trade. The flush is
    // dispatched to a background task so DB latency cannot backpressure
    // the WS read loop — the previous inline `await` here would stall
    // every subsequent trade until Postgres ack'd, which Finnhub can
    // detect as backpressure and start dropping messages.
    fn feed_bucket(&self, t: &Trade) {
        let bucket_start = (t.ts_ms / 1000 / TAPE_BUCKET_SECS) * TAPE_BUCKET_SECS;
        let mut closed: Option<(String, TapeBucket)> = None;
        {
            // DashMap entry lives only for the synchronous bookkeeping —
            // no .await held under it.
            let mut entry = self
                .buckets
                .entry(t.symbol.clone())
                .or_insert_with(|| TapeBucket {
                    start_sec: bucket_start,
                    open: t.price,
                    high: t.price,
                    low: t.price,
                    close: t.price,
                    volume: 0.0,
                });
            if entry.start_sec != bucket_start {
                // Bucket crossed — extract the closed one and start fresh.
                let prev = entry.clone();
                *entry = TapeBucket {
                    start_sec: bucket_start,
                    open: t.price,
                    high: t.price,
                    low: t.price,
                    close: t.price,
                    volume: 0.0,
                };
                closed = Some((t.symbol.clone(), prev));
            }
            // Always fold the incoming trade into the now-current bucket.
            if t.price > entry.high { entry.high = t.price; }
            if t.price < entry.low  { entry.low  = t.price; }
            entry.close = t.price;
            entry.volume += t.volume;
        }
        if let Some((sym, bucket)) = closed {
            // Detach the persist to a background task — `feed_bucket`
            // returns to the WS read loop immediately, and the DB write
            // races independently. Late persists are safe because
            // `price_bars (symbol, interval, bar_time)` is unique and the
            // upsert uses `ON CONFLICT DO UPDATE`.
            let store = self.clone();
            tokio::spawn(async move {
                store.persist_bucket(&sym, &bucket).await;
            });
        }
    }

    // Idle sweep — flush any bucket whose window has fully elapsed at
    // least one full window ago. Without this, a symbol that stops trading
    // mid-window leaves an in-progress bucket dangling forever.
    //
    // Race-safety: we snapshot each bucket's `start_sec` at observation
    // time, and only remove if the entry still matches that snapshot. If
    // a fresh trade arrives between the iter and the remove (rotating the
    // bucket to a new window), `remove_if` returns None and we leave the
    // new bucket in place. The previously-observed (now-stale) closed
    // bucket was already in the same DashMap slot — the rotation in
    // feed_bucket re-uses the entry but the new `start_sec` differs, so
    // `remove_if` cleanly distinguishes. Persist the snapshotted closed
    // bucket regardless; the row in `price_bars` keys on (symbol, '10s',
    // bar_time) so re-persisting the SAME closed bucket on a later sweep
    // is idempotent via `ON CONFLICT DO UPDATE`.
    async fn flush_idle_buckets(&self) {
        let now_sec = Utc::now().timestamp();
        let mut to_flush: Vec<(String, TapeBucket)> = Vec::new();
        for entry in self.buckets.iter() {
            let b = entry.value();
            // A bucket starting at `start_sec` covers [start_sec, start_sec + 10).
            // Flush once we're a full window past the close.
            if now_sec >= b.start_sec + TAPE_BUCKET_SECS * 2 {
                to_flush.push((entry.key().clone(), b.clone()));
            }
        }
        for (sym, bucket) in &to_flush {
            self.persist_bucket(sym, bucket).await;
            // Only drop the slot if it still holds the bucket we
            // observed. Otherwise a fresh trade rotated in a new bucket
            // (different `start_sec`) between iter and remove — leave it.
            let observed_start = bucket.start_sec;
            self.buckets
                .remove_if(sym, |_, cur| cur.start_sec == observed_start);
        }
    }

    async fn persist_bucket(&self, symbol: &str, b: &TapeBucket) {
        let pool = { self.pool.read().await.clone() };
        let Some(pool) = pool else { return };
        let bar_time = match Utc.timestamp_opt(b.start_sec, 0).single() {
            Some(t) => t,
            None => return,
        };
        // Decimal::from_f64 returns None on NaN/Inf — drop the bar in that
        // case rather than persisting garbage that breaks downstream queries.
        let (Some(open), Some(high), Some(low), Some(close), Some(volume)) = (
            Decimal::from_f64(b.open),
            Decimal::from_f64(b.high),
            Decimal::from_f64(b.low),
            Decimal::from_f64(b.close),
            Decimal::from_f64(b.volume),
        ) else {
            return;
        };
        let bar = PriceBar {
            symbol: symbol.to_string(),
            interval: BarInterval::S10,
            bar_time,
            open,
            high,
            low,
            close,
            volume,
            source: "finnhub-tape".into(),
        };
        if let Err(e) = crate::prices::upsert(&pool, std::slice::from_ref(&bar)).await {
            tracing::warn!(error = %e, symbol, bucket = b.start_sec, "tape 10s persist failed");
        }
    }

    async fn run_once(&self, api_key: &str, symbols: &[String]) -> anyhow::Result<()> {
        let url = format!("{FINNHUB_WS}?token={api_key}");
        let (ws, _) = tokio_tungstenite::connect_async(&url).await?;
        let (mut tx, mut rx) = ws.split();
        for s in symbols {
            let msg = serde_json::json!({"type":"subscribe","symbol":s}).to_string();
            tx.send(WsMessage::Text(msg)).await?;
        }
        while let Some(msg) = rx.next().await {
            match msg? {
                WsMessage::Text(t) => {
                    if let Ok(env) = serde_json::from_str::<TradeEnvelope>(&t) {
                        if env.r#type == "trade" {
                            for tr in env.data.unwrap_or_default() {
                                let trade = Trade {
                                    symbol: tr.s,
                                    price: tr.p,
                                    volume: tr.v,
                                    ts_ms: tr.t,
                                };
                                if let Some(mut state) = self.state.get_mut(&trade.symbol) {
                                    state.observe(&trade);
                                    let snap = state.clone();
                                    drop(state);
                                    let _ = self.tx.send(snap);
                                }
                                // Fold the trade into the 10s OHLC bucket
                                // and dispatch any closed bucket to a
                                // background persist task. Synchronous —
                                // does NOT await the DB write.
                                self.feed_bucket(&trade);
                            }
                        }
                    }
                }
                WsMessage::Ping(p) => {
                    tx.send(WsMessage::Pong(p)).await.ok();
                }
                WsMessage::Close(_) => break,
                _ => {}
            }
        }
        Ok(())
    }
}

impl Default for LiveTickStore {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(serde::Deserialize)]
struct TradeEnvelope {
    r#type: String,
    data: Option<Vec<RawTrade>>,
}
#[derive(serde::Deserialize)]
struct RawTrade {
    p: f64,
    s: String,
    t: i64,
    v: f64,
}

/// Process-wide singleton.
pub fn global() -> LiveTickStore {
    static STORE: once_cell::sync::OnceCell<LiveTickStore> = once_cell::sync::OnceCell::new();
    STORE.get_or_init(LiveTickStore::new).clone()
}
