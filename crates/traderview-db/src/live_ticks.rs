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

/// Exponential backoff with jitter for the live-tick WS workers. Each
/// successive failure waits min(BASE * 2^n, CAP) + rand(0..BASE)
/// seconds; a successful run resets the counter. Prevents the tight
/// 3-second reconnect storm the old fixed sleep produced when an
/// upstream was rate-limiting (Alpaca 406 connection-limit exceeded
/// would hammer the provider indefinitely).
struct Backoff {
    n: u32,
}

impl Backoff {
    const BASE_SECS: u64 = 2;
    const CAP_SECS: u64 = 60;

    fn new() -> Self {
        Self { n: 0 }
    }
    fn reset(&mut self) {
        self.n = 0;
    }
    fn next(&mut self) -> Duration {
        // 2 * 2^n clamped to CAP, plus 0..BASE seconds of jitter so
        // many simultaneous reconnects don't dogpile the upstream.
        let exp = (Self::BASE_SECS).saturating_mul(1u64 << self.n.min(6));
        let base = exp.min(Self::CAP_SECS);
        self.n = self.n.saturating_add(1);
        // Cheap jitter without dragging in `rand` here — XOR a few
        // bits of `Instant::now()` against a small mask. Good enough
        // for a few-second spread; not for cryptographic randomness.
        let now_nanos = std::time::Instant::now()
            .elapsed()
            .as_nanos() as u64;
        let jitter = (now_nanos ^ (self.n as u64)) % (Self::BASE_SECS + 1);
        Duration::from_secs(base.saturating_add(jitter))
    }
}
use tokio::sync::broadcast;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use traderview_core::{BarInterval, PriceBar};

pub const FINNHUB_WS: &str = "wss://ws.finnhub.io";
/// Polygon real-time SIP-tape WebSocket. Requires a Stocks Starter+ key
/// to authenticate; Advanced tier unlocks the full consolidated CTA/UTP
/// trade feed. Use `wss://delayed.polygon.io/stocks` if the user opts
/// for the free 15-min-delayed tier.
pub const POLYGON_WS: &str = "wss://socket.polygon.io/stocks";
/// Alpaca market-data WS — choose SIP vs IEX-only per user preference.
/// SIP needs Algo Trader+; IEX-only works on the free Algo Trader Free
/// tier and is the default fallback when `alpaca_use_sip_feed` is off.
pub const ALPACA_WS_SIP: &str = "wss://stream.data.alpaca.markets/v2/sip";
pub const ALPACA_WS_IEX: &str = "wss://stream.data.alpaca.markets/v2/iex";
/// Alpaca crypto WS — same auth/subscribe protocol as the equities feed
/// but a separate endpoint. Trades 24/7 so it's the canonical
/// weekend-debug stream when US equities are closed.
pub const ALPACA_WS_CRYPTO: &str = "wss://stream.data.alpaca.markets/v1beta3/crypto/us";
/// Crypto symbol whitelist — anything ending in `USD` whose prefix is
/// in this set is routed to the crypto WS instead of the equities WS.
/// Liberal; false-positives are harmless (Alpaca returns "symbol not
/// found" for unknown crypto codes and the WS keeps running).
const CRYPTO_PREFIXES: &[&str] = &[
    "BTC", "ETH", "LTC", "BCH", "DOGE", "SOL", "AVAX", "MATIC", "ADA", "DOT", "XRP", "LINK", "UNI",
    "SHIB", "AAVE", "ALGO", "ATOM", "BAT", "COMP", "CRV", "GRT", "MKR", "PAXG", "SUSHI", "TRX",
    "XLM", "XTZ", "YFI", "ZRX",
];

fn is_crypto_symbol(sym: &str) -> bool {
    let upper = sym.to_uppercase();
    // Canonical Alpaca form "BASE/USD" — the autoscan crypto universe
    // ships symbols already in this shape. Strip the slash before
    // checking the base against CRYPTO_PREFIXES, otherwise the prefix
    // slice picks up the slash ("BTC/USD" → prefix "BTC/" → no match)
    // and the symbol gets mis-routed to the equity IEX WS, which
    // silently never delivers trades.
    if let Some((base, quote)) = upper.split_once('/') {
        return quote == "USD" && CRYPTO_PREFIXES.contains(&base);
    }
    // BARE ticker (no quote currency) like "BTC" / "ETH" / "SOL" —
    // user-entered shorthand. Route to crypto WS as <ticker>/USD.
    if !upper.ends_with("USD") && CRYPTO_PREFIXES.contains(&upper.as_str()) {
        return true;
    }
    if !upper.ends_with("USD") {
        return false;
    }
    let prefix = &upper[..upper.len() - 3];
    CRYPTO_PREFIXES.contains(&prefix)
}

/// Convert a user-entered crypto symbol to the Alpaca v1beta3 crypto
/// WebSocket's required `BASE/QUOTE` format. Subscribing with the
/// wrong format triggers a server-side rejection ("invalid syntax",
/// code 400) and zero trades flow.
///
///   "BTC"     → "BTC/USD"
///   "BTCUSD"  → "BTC/USD"
///   "ETH/USD" → "ETH/USD"   (already normalised)
fn to_alpaca_crypto_symbol(sym: &str) -> String {
    let upper = sym.to_uppercase();
    if upper.contains('/') {
        return upper;
    }
    if upper.ends_with("USD") && upper.len() > 3 {
        let base = &upper[..upper.len() - 3];
        return format!("{base}/USD");
    }
    format!("{upper}/USD")
}

/// Inverse of `to_alpaca_crypto_symbol`. Currently unused on the
/// receive path — we key state + 10s buckets + price_bars by the
/// canonical `BTC/USD` form so the algo runner's autoscan picks and
/// `fetch_recent_bars` queries round-trip cleanly. Kept here for the
/// reverse-mapping use case (settings UI showing a "user typed
/// BTCUSD → we subscribe BTC/USD" hint) + the existing tests pin
/// its behaviour.
#[allow(dead_code)]
fn from_alpaca_crypto_symbol(sym: &str) -> String {
    sym.replace('/', "")
}
const MAX_SYMS_PER_CONN: usize = 25;
/// Polygon's WS can fan a single connection across thousands of
/// symbols; cap at 500 per worker for memory + reconnect granularity.
const POLYGON_MAX_SYMS_PER_CONN: usize = 500;
/// Alpaca per-connection cap. Same memory + recovery rationale as
/// Polygon — 500 keeps reconnect storms bounded.
const ALPACA_MAX_SYMS_PER_CONN: usize = 500;

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
    /// Raw trade broadcast — every parsed Trade is sent through this
    /// channel so consumers (algo view's tape pane, debug tooling)
    /// can see the unaggregated tick stream as it arrives, not the
    /// per-state snapshot the main `tx` carries.
    tape_tx: broadcast::Sender<Trade>,
    /// Finnhub WS key — the legacy default provider.
    api_key: Arc<tokio::sync::RwLock<Option<String>>>,
    /// Polygon.io WS key. When present, the store prefers Polygon over
    /// Finnhub for live trades (SIP tape vs Finnhub aggregate). Falls
    /// through to Finnhub when absent or when Polygon WS fails to
    /// authenticate.
    polygon_key: Arc<tokio::sync::RwLock<Option<String>>>,
    /// Alpaca market-data WS credentials. Used as the middle provider
    /// in the priority chain (Polygon → Alpaca → Finnhub). When
    /// `alpaca_use_sip_feed` is true the worker connects to the SIP
    /// endpoint (Algo Trader+ required); otherwise the IEX-only feed
    /// (works on the free tier).
    alpaca_creds: Arc<tokio::sync::RwLock<Option<(String, String)>>>,
    alpaca_use_sip: Arc<AtomicBool>,
    /// Last observed Alpaca-WS auth result, populated by the worker
    /// when it sees the first auth-response frame. The test endpoint
    /// reads from this so it doesn't have to open a SECOND WS (Alpaca
    /// enforces 1 connection per account+feed, so a second attempt
    /// always fails with "connection limit exceeded" while the live
    /// worker holds the slot).
    /// `(success: bool, feed: "iex"|"sip"|"crypto", millis since UNIX
    /// epoch when observed)`.
    alpaca_last_auth: Arc<tokio::sync::RwLock<Option<(bool, String, i64)>>>,
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
    // Handles of every currently-running WS worker so a fresh
    // spawn_workers call (e.g. after the algo runner calls
    // ensure_subscribed) can ABORT the old workers before starting
    // new ones. Without this, every subscription change duplicated
    // the connection — both old + new fought for the single Alpaca
    // account slot, ALL of them failed with `connection limit
    // exceeded` (code 406), and ZERO trades flowed.
    workers: Arc<tokio::sync::Mutex<Vec<tokio::task::JoinHandle<()>>>>,
    /// Per-symbol trade counter for the rolling rate log. Reset every
    /// 60s when the rate is dumped to the desktop log. Lets the user
    /// confirm the ingest rate seen on the tape pane matches what
    /// the worker actually parsed off the WS.
    tape_counters: Arc<DashMap<String, u64>>,
    tape_window_start_ms: Arc<std::sync::atomic::AtomicI64>,
}

impl LiveTickStore {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(1024);
        let (tape_tx, _) = broadcast::channel(2048);
        Self {
            state: Arc::new(DashMap::new()),
            tx,
            tape_tx,
            api_key: Arc::new(tokio::sync::RwLock::new(None)),
            polygon_key: Arc::new(tokio::sync::RwLock::new(None)),
            alpaca_creds: Arc::new(tokio::sync::RwLock::new(None)),
            alpaca_use_sip: Arc::new(AtomicBool::new(false)),
            alpaca_last_auth: Arc::new(tokio::sync::RwLock::new(None)),
            subs: Arc::new(tokio::sync::Mutex::new(Vec::new())),
            buckets: Arc::new(DashMap::new()),
            pool: Arc::new(tokio::sync::RwLock::new(None)),
            sweeper_spawned: Arc::new(AtomicBool::new(false)),
            workers: Arc::new(tokio::sync::Mutex::new(Vec::new())),
            tape_counters: Arc::new(DashMap::new()),
            tape_window_start_ms: Arc::new(std::sync::atomic::AtomicI64::new(0)),
        }
    }

    /// Per-symbol ingest counter — increments on every parsed Trade
    /// (called from `feed_bucket`). Every ~60 seconds we dump a
    /// per-symbol breakdown to the log so the user can compare the
    /// frontend tape pane's "N/sec" header against what the worker
    /// actually observed at the WS layer.
    fn observe_for_rate_log(&self, t: &Trade) {
        *self.tape_counters.entry(t.symbol.clone()).or_insert(0) += 1;
        let now_ms = chrono::Utc::now().timestamp_millis();
        let started = self
            .tape_window_start_ms
            .load(std::sync::atomic::Ordering::Acquire);
        if started == 0 {
            self.tape_window_start_ms
                .store(now_ms, std::sync::atomic::Ordering::Release);
            return;
        }
        if now_ms - started < 60_000 {
            return;
        }
        // Window elapsed — log + reset.
        if self
            .tape_window_start_ms
            .compare_exchange(
                started,
                now_ms,
                std::sync::atomic::Ordering::AcqRel,
                std::sync::atomic::Ordering::Acquire,
            )
            .is_err()
        {
            // Another thread is doing the dump — let it.
            return;
        }
        let mut total: u64 = 0;
        let mut per: Vec<(String, u64)> = Vec::new();
        for entry in self.tape_counters.iter() {
            per.push((entry.key().clone(), *entry.value()));
            total += *entry.value();
        }
        per.sort_by_key(|x| std::cmp::Reverse(x.1));
        let top = per
            .iter()
            .take(10)
            .map(|(s, n)| format!("{s}={n}"))
            .collect::<Vec<_>>()
            .join(" ");
        let secs = (now_ms - started) as f64 / 1000.0;
        tracing::info!(
            total_trades = total,
            rate = format!("{:.2}/sec", total as f64 / secs),
            window_secs = secs,
            top10 = %top,
            "live_ticks: 60s tape ingest summary"
        );
        self.tape_counters.clear();
    }

    /// Wire the Polygon WS key. Mirrors `set_api_key` so settings-page
    /// POSTs can update the provider without a restart. When this key
    /// is present, `restart_workers` switches the live tape to Polygon.
    pub async fn set_polygon_key(&self, key: impl Into<String>) {
        *self.polygon_key.write().await = Some(key.into());
    }

    pub async fn has_polygon_key(&self) -> bool {
        self.polygon_key.read().await.is_some()
    }

    /// Wire the Alpaca market-data credentials. The SIP-vs-IEX choice
    /// is governed by `set_alpaca_use_sip` — flipping that does NOT
    /// restart workers automatically, callers (settings POST handler)
    /// trigger `restart_workers()` themselves once both setters land.
    pub async fn set_alpaca_creds(&self, key_id: impl Into<String>, secret: impl Into<String>) {
        *self.alpaca_creds.write().await = Some((key_id.into(), secret.into()));
    }

    pub fn set_alpaca_use_sip(&self, on: bool) {
        self.alpaca_use_sip.store(on, Ordering::Release);
    }

    pub async fn has_alpaca_creds(&self) -> bool {
        self.alpaca_creds.read().await.is_some()
    }

    /// Snapshot of the most recent Alpaca WS auth result observed by
    /// any running worker. Used by the "Test Alpaca" Settings button
    /// so it can confirm creds without opening a second WS (which
    /// Alpaca rejects with "connection limit exceeded"). Returns
    /// `(success, feed, age_ms)` where `age_ms` is how long ago we
    /// last saw the auth response.
    pub async fn alpaca_last_auth(&self) -> Option<(bool, String, i64)> {
        let now = chrono::Utc::now().timestamp_millis();
        self.alpaca_last_auth
            .read()
            .await
            .clone()
            .map(|(ok, feed, ts)| (ok, feed, now - ts))
    }

    pub(crate) async fn record_alpaca_auth(&self, ok: bool, feed: &str) {
        *self.alpaca_last_auth.write().await =
            Some((ok, feed.to_string(), chrono::Utc::now().timestamp_millis()));
    }

    /// True when any provider key is configured. Callers that ungate
    /// `set_symbols` (the candidates / squeeze reconcile loops) should
    /// check this instead of `has_key()` — `has_key()` is Finnhub-only
    /// and gates the WS spawn behind the legacy provider only.
    pub async fn has_any_provider(&self) -> bool {
        self.has_key().await || self.has_polygon_key().await || self.has_alpaca_creds().await
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

    /// O(1) lookup of the current `SymbolState` for one ticker. Returns
    /// `None` if no trade has been observed for that symbol yet today.
    /// Used by callers that need a baseline price (catalyst correlator,
    /// alerting, position-size calculators) without iterating the full
    /// snapshot.
    pub fn get(&self, symbol: &str) -> Option<SymbolState> {
        self.state.get(symbol).map(|e| e.value().clone())
    }

    /// How many symbols are currently in the live subscription set.
    /// Used by the algo runner's heartbeat event so the user can see
    /// "live=27 symbols streaming" without having to query elsewhere.
    pub async fn subs_len(&self) -> usize {
        self.subs.lock().await.len()
    }

    /// Union extra symbols into the existing subscription set without
    /// dropping any current watchlist subscriptions. Workers restart
    /// only when the set actually changes. Used by the algo runner so
    /// autoscan picks start streaming alongside the user's watchlist
    /// — otherwise autoscan picks SPY/MSFT/etc. but only watchlist
    /// symbols are getting WS frames, and the strategy never sees
    /// real-time 1m bars for them.
    pub async fn ensure_subscribed(&self, symbols: Vec<String>) -> anyhow::Result<()> {
        if symbols.is_empty() {
            return Ok(());
        }
        let mut current = self.subs.lock().await;
        let mut union: Vec<String> = current.clone();
        union.extend(symbols);
        union.sort();
        union.dedup();
        if *current == union {
            return Ok(());
        }
        *current = union.clone();
        drop(current);
        for s in &union {
            self.state
                .entry(s.clone())
                .or_insert_with(|| SymbolState::new(s, None));
        }
        self.spawn_workers(union).await
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

        // Seed each symbol's SymbolState with its prev_close so the live
        // scanner's TOP GAPPERS / TOP GAINERS / TOP LOSERS panels render
        // immediately when the first WS tick arrives. Without this, gap_pct
        // and change_pct stay at 0 (panels stay empty) until a separate
        // quote-fetch path happens to warm the cache.
        //
        // Uses `market_data::quotes()` which is cache-first + Yahoo-backed
        // — works without Finnhub. Failure is non-fatal; symbols without a
        // resolved prev_close just won't show gap until one lands.
        let pool_opt = self.pool.read().await.clone();
        let prev_closes: std::collections::HashMap<String, f64> = if let Some(pool) = pool_opt {
            crate::market_data::quotes(&pool, &deduped)
                .await
                .into_iter()
                .filter_map(|q| q.prev_close.map(|pc| (q.symbol, pc)))
                .collect()
        } else {
            std::collections::HashMap::new()
        };
        for s in &deduped {
            let seeded = prev_closes.get(s).copied();
            self.state
                .entry(s.clone())
                .and_modify(|st| {
                    if st.prev_close.is_none() {
                        st.prev_close = seeded;
                    }
                })
                .or_insert_with(|| SymbolState::new(s, seeded));
        }

        self.spawn_workers(deduped).await
    }

    /// Force a re-spawn of the WS worker(s) without changing the
    /// subscription set. Called from the settings-page POST after a
    /// key change so the live tape switches providers (Finnhub →
    /// Polygon, or vice versa) without a process restart.
    pub async fn restart_workers(&self) -> anyhow::Result<()> {
        let symbols = self.subs.lock().await.clone();
        if symbols.is_empty() {
            return Ok(());
        }
        self.spawn_workers(symbols).await
    }

    /// Spawn the WS worker tasks for the given symbol list. Provider
    /// priority — cheapest-real-time-SIP first:
    ///   1. Alpaca (Algo Trader+ ~$99/mo with SIP toggle, else IEX-only
    ///      on the free tier) — the default real-time path.
    ///   2. Polygon (~$199/mo Advanced) — full CTA/UTP, switch up here
    ///      when Alpaca's symbol coverage / latency isn't enough.
    ///   3. Finnhub (aggregate, free 60 calls/min) — last-resort fallback.
    ///
    /// Skips entirely when no provider key is configured.
    async fn spawn_workers(&self, symbols: Vec<String>) -> anyhow::Result<()> {
        // BEFORE spawning anything new, abort every worker spawned by a
        // prior call. Alpaca enforces ONE connection per account+feed
        // (IEX, SIP, crypto) — if the boot-time crypto worker is still
        // alive when the algo runner's first ensure_subscribed fires a
        // second crypto worker, the new one gets "connection limit
        // exceeded" (code 406) and ZERO trades flow on either. Same
        // problem on Polygon and Finnhub at their own rate-limit
        // boundaries.
        {
            let mut handles = self.workers.lock().await;
            let n = handles.len();
            for h in handles.drain(..) {
                h.abort();
            }
            if n > 0 {
                tracing::info!(aborted = n, "live_ticks: stopped prior WS workers");
            }
        }

        // 1. Alpaca — cheapest real-time SIP at $99/mo (Algo Trader+).
        //    IEX-only fallback on the free Algo Trader tier when
        //    `alpaca_use_sip_feed` is off. Crypto-shaped symbols
        //    (BTCUSD, ETHUSD, ...) are routed to the crypto WS so
        //    weekend / after-hours testing works without waiting for
        //    US equity market open.
        if let Some((id, secret)) = self.alpaca_creds.read().await.clone() {
            let use_sip = self.alpaca_use_sip.load(Ordering::Acquire);
            let (crypto, stocks): (Vec<String>, Vec<String>) =
                symbols.into_iter().partition(|s| is_crypto_symbol(s));
            let mut new_handles = Vec::new();
            for chunk in stocks.chunks(ALPACA_MAX_SYMS_PER_CONN).map(|c| c.to_vec()) {
                let store = self.clone();
                let id = id.clone();
                let secret = secret.clone();
                let n = chunk.len();
                tracing::info!(
                    feed = if use_sip { "sip" } else { "iex" },
                    symbols = n,
                    "live_ticks: spawning alpaca equity worker",
                );
                let h = tokio::spawn(async move {
                    use futures_util::FutureExt;
                    if let Err(_) = std::panic::AssertUnwindSafe(
                        store.run_alpaca_worker(id, secret, use_sip, chunk),
                    )
                    .catch_unwind()
                    .await
                    {
                        tracing::error!("live_ticks: alpaca equity worker PANICKED");
                    }
                });
                new_handles.push(h);
            }
            for chunk in crypto.chunks(ALPACA_MAX_SYMS_PER_CONN).map(|c| c.to_vec()) {
                let store = self.clone();
                let id = id.clone();
                let secret = secret.clone();
                let n = chunk.len();
                tracing::info!(symbols = n, "live_ticks: spawning alpaca crypto worker");
                let h = tokio::spawn(async move {
                    use futures_util::FutureExt;
                    if let Err(_) = std::panic::AssertUnwindSafe(
                        store.run_alpaca_crypto_worker(id, secret, chunk),
                    )
                    .catch_unwind()
                    .await
                    {
                        tracing::error!("live_ticks: alpaca crypto worker PANICKED");
                    }
                });
                new_handles.push(h);
            }
            *self.workers.lock().await = new_handles;
            return Ok(());
        }
        // 2. Polygon — full CTA/UTP SIP tape, used when Alpaca isn't
        //    configured (or its coverage / latency isn't enough).
        if let Some(key) = self.polygon_key.read().await.clone() {
            let mut new_handles = Vec::new();
            for chunk in symbols
                .chunks(POLYGON_MAX_SYMS_PER_CONN)
                .map(|c| c.to_vec())
            {
                let store = self.clone();
                let key = key.clone();
                let n = chunk.len();
                tracing::info!(symbols = n, "live_ticks: spawning polygon worker");
                let h = tokio::spawn(async move {
                    use futures_util::FutureExt;
                    if let Err(_) =
                        std::panic::AssertUnwindSafe(store.run_polygon_worker(key, chunk))
                            .catch_unwind()
                            .await
                    {
                        tracing::error!("live_ticks: polygon worker PANICKED");
                    }
                });
                new_handles.push(h);
            }
            *self.workers.lock().await = new_handles;
            return Ok(());
        }
        // 3. Finnhub — aggregate fallback.
        let finnhub_key = self.api_key.read().await.clone();
        let Some(key) = finnhub_key else {
            tracing::warn!(
                "no live-tick provider key (polygon / alpaca / finnhub) set; live ticks disabled"
            );
            return Ok(());
        };
        let mut new_handles = Vec::new();
        for chunk in symbols.chunks(MAX_SYMS_PER_CONN).map(|c| c.to_vec()) {
            let store = self.clone();
            let key = key.clone();
            let n = chunk.len();
            tracing::info!(symbols = n, "live_ticks: spawning finnhub worker");
            let h = tokio::spawn(async move {
                use futures_util::FutureExt;
                if let Err(_) =
                    std::panic::AssertUnwindSafe(store.run_worker(key, chunk))
                        .catch_unwind()
                        .await
                {
                    tracing::error!("live_ticks: finnhub worker PANICKED");
                }
            });
            new_handles.push(h);
        }
        *self.workers.lock().await = new_handles;
        Ok(())
    }

    async fn run_worker(self, api_key: String, symbols: Vec<String>) {
        let mut backoff = Backoff::new();
        loop {
            match self.run_once(&api_key, &symbols).await {
                Ok(()) => {
                    tracing::info!("finnhub WS exited cleanly; reconnecting");
                    backoff.reset();
                }
                Err(e) => {
                    let wait = backoff.next();
                    tracing::warn!(?e, ?wait, "finnhub WS error; backing off");
                    tokio::time::sleep(wait).await;
                    continue;
                }
            }
            // Clean exits still need a short pause to avoid a tight
            // loop on a misbehaving upstream that immediately disconnects.
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
    /// Subscribe to the raw trade tape — every parsed Trade is sent
    /// here as the worker reads it off the WS. Late subscribers miss
    /// historical trades; the broadcast channel is bounded to 2048
    /// and oldest-evicts on lag.
    pub fn tape_subscribe(&self) -> broadcast::Receiver<Trade> {
        self.tape_tx.subscribe()
    }

    fn feed_bucket(&self, t: &Trade) {
        // Fire the raw trade onto the tape broadcast BEFORE bucket
        // bookkeeping so a consumer sees every tick even if a
        // downstream bucket close takes a moment to persist. No
        // backpressure — the send is non-blocking and lossy on lag.
        let _ = self.tape_tx.send(t.clone());
        // Per-symbol ingest counter, logged every 60s. The user sees
        // the same number on the frontend tape pane header — if they
        // diverge by more than the broadcast-lag drop count, we know
        // the renderer (not the ingest) is the bottleneck.
        self.observe_for_rate_log(t);
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
            if t.price > entry.high {
                entry.high = t.price;
            }
            if t.price < entry.low {
                entry.low = t.price;
            }
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
        let Some(pool) = pool else {
            // Loud once per process — the boot path forgot to call
            // set_pool, so every closed 10s bucket evaporates. The
            // sweeper hits this in a tight loop; CAS guard ensures
            // we only emit one error so it surfaces clearly without
            // flooding the log.
            static WARNED: std::sync::atomic::AtomicBool =
                std::sync::atomic::AtomicBool::new(false);
            if WARNED
                .compare_exchange(
                    false,
                    true,
                    std::sync::atomic::Ordering::AcqRel,
                    std::sync::atomic::Ordering::Acquire,
                )
                .is_ok()
            {
                tracing::error!(
                    symbol,
                    "live_ticks: persist_bucket called but pool=None — set_pool was never wired at boot; every 10s bar silently dropped. price_bars stays empty, algo strategies SKIP forever."
                );
            }
            return;
        };
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

    /// Alpaca market-data WS worker. Same Trade-observe + feed_bucket
    /// pipeline as the other providers; the only differences are URL
    /// (SIP vs IEX) and the auth + subscribe protocol shape.
    /// Protocol:
    ///   1. Connect to `/v2/sip` (Algo Trader+) or `/v2/iex` (free).
    ///   2. `{action:"auth", key:<id>, secret:<secret>}` — wait for
    ///      `[{T:"success", msg:"authenticated"}]`.
    ///   3. `{action:"subscribe", trades:["AAPL","MSFT",...]}`.
    ///   4. Receive `[{T:"t", S, p, s, t, ...}, ...]` arrays. `T:"t"`
    ///      = trade event; ignore `q` (quote) / `b` (bar) / `subscription`
    ///      acks.
    async fn run_alpaca_worker(
        self,
        key_id: String,
        secret: String,
        use_sip: bool,
        symbols: Vec<String>,
    ) {
        let mut backoff = Backoff::new();
        loop {
            match self
                .run_alpaca_once(&key_id, &secret, use_sip, &symbols)
                .await
            {
                Ok(()) => {
                    tracing::info!("alpaca WS exited cleanly; reconnecting");
                    backoff.reset();
                }
                Err(e) => {
                    let wait = backoff.next();
                    tracing::warn!(?e, ?wait, "alpaca WS error; backing off");
                    tokio::time::sleep(wait).await;
                    continue;
                }
            }
            tokio::time::sleep(Duration::from_secs(3)).await;
        }
    }

    async fn run_alpaca_once(
        &self,
        key_id: &str,
        secret: &str,
        use_sip: bool,
        symbols: &[String],
    ) -> anyhow::Result<()> {
        let url = if use_sip {
            ALPACA_WS_SIP
        } else {
            ALPACA_WS_IEX
        };
        let (ws, _) = tokio_tungstenite::connect_async(url).await?;
        let (mut tx, mut rx) = ws.split();
        // 1. Auth.
        let auth = serde_json::json!({
            "action": "auth",
            "key": key_id,
            "secret": secret,
        })
        .to_string();
        tx.send(WsMessage::Text(auth)).await?;
        // 2. Subscribe to trades. Alpaca takes a JSON array of symbols
        //    in one frame — pass them all at once.
        if !symbols.is_empty() {
            let sub = serde_json::json!({
                "action": "subscribe",
                "trades": symbols,
            })
            .to_string();
            tx.send(WsMessage::Text(sub)).await?;
        }
        // 3. Read loop. Frames are arrays of events.
        while let Some(msg) = rx.next().await {
            match msg? {
                WsMessage::Text(t) => {
                    let arr: serde_json::Value = match serde_json::from_str(&t) {
                        Ok(v) => v,
                        Err(_) => continue,
                    };
                    let Some(events) = arr.as_array() else {
                        continue;
                    };
                    let feed_name = if use_sip { "sip" } else { "iex" };
                    for ev in events {
                        let kind = ev.get("T").and_then(|v| v.as_str()).unwrap_or("");
                        if kind == "error" {
                            tracing::warn!(
                                msg = ?ev.get("msg"),
                                code = ?ev.get("code"),
                                "alpaca WS error frame"
                            );
                            // Only record auth failure for the codes
                            // Alpaca actually uses to signal bad creds /
                            // missing entitlement:
                            //   401 = not authenticated
                            //   402 = auth failed (wrong key/secret)
                            //   410 = forbidden (no entitlement for feed)
                            // Other 4xx codes (405 symbol limit, 406
                            // connection limit, 407 slow client, etc.)
                            // are operational issues, NOT auth failures —
                            // recording them as such made the Settings
                            // "Test Alpaca" button report false-negative
                            // when a stale prior process held the
                            // connection slot.
                            if let Some(code) = ev.get("code").and_then(|v| v.as_i64()) {
                                if matches!(code, 401 | 402 | 410) {
                                    self.record_alpaca_auth(false, feed_name).await;
                                } else if code == 406 {
                                    // Connection-limit-exceeded actually
                                    // proves the credentials work — Alpaca
                                    // would only reject this if a prior
                                    // session with the SAME creds had
                                    // already authenticated.
                                    self.record_alpaca_auth(true, feed_name).await;
                                }
                            }
                            continue;
                        }
                        if kind == "success"
                            && ev.get("msg").and_then(|v| v.as_str()) == Some("authenticated")
                        {
                            self.record_alpaca_auth(true, feed_name).await;
                            tracing::info!(feed = %feed_name, "alpaca WS authenticated");
                            continue;
                        }
                        if kind == "subscription" {
                            let trades_n = ev
                                .get("trades")
                                .and_then(|v| v.as_array())
                                .map(|a| a.len())
                                .unwrap_or(0);
                            tracing::info!(
                                feed = %feed_name,
                                trades = trades_n,
                                "alpaca WS subscribed",
                            );
                            continue;
                        }
                        if kind != "t" {
                            // quote / bar / other — ignore (we only consume
                            // trades for the tape).
                            continue;
                        }
                        let sym = ev.get("S").and_then(|v| v.as_str()).unwrap_or_default();
                        let price = ev.get("p").and_then(|v| v.as_f64()).unwrap_or(0.0);
                        let size = ev.get("s").and_then(|v| v.as_f64()).unwrap_or(0.0);
                        // Alpaca returns `t` as RFC3339 timestamp string.
                        let ts_ms = ev
                            .get("t")
                            .and_then(|v| v.as_str())
                            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                            .map(|dt| dt.timestamp_millis())
                            .unwrap_or(0);
                        if sym.is_empty() || price <= 0.0 || ts_ms <= 0 {
                            continue;
                        }
                        let trade = Trade {
                            symbol: sym.to_string(),
                            price,
                            volume: size,
                            ts_ms,
                        };
                        if let Some(mut state) = self.state.get_mut(&trade.symbol) {
                            state.observe(&trade);
                            let snap = state.clone();
                            drop(state);
                            let _ = self.tx.send(snap);
                        }
                        self.feed_bucket(&trade);
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

    /// Alpaca crypto WS worker. Trades 24/7 so this is the canonical
    /// weekend / after-hours debug path — subscribe to `BTCUSD` to
    /// confirm the wire-up while US equities are closed. Auth +
    /// subscribe frames mirror the equities worker; the only delta is
    /// the endpoint URL.
    async fn run_alpaca_crypto_worker(self, key_id: String, secret: String, symbols: Vec<String>) {
        let mut backoff = Backoff::new();
        loop {
            match self
                .run_alpaca_crypto_once(&key_id, &secret, &symbols)
                .await
            {
                Ok(()) => {
                    tracing::info!("alpaca crypto WS exited cleanly; reconnecting");
                    backoff.reset();
                }
                Err(e) => {
                    let wait = backoff.next();
                    tracing::warn!(?e, ?wait, "alpaca crypto WS error; backing off");
                    tokio::time::sleep(wait).await;
                    continue;
                }
            }
            tokio::time::sleep(Duration::from_secs(3)).await;
        }
    }

    async fn run_alpaca_crypto_once(
        &self,
        key_id: &str,
        secret: &str,
        symbols: &[String],
    ) -> anyhow::Result<()> {
        tracing::info!(
            symbols = symbols.len(),
            url = ALPACA_WS_CRYPTO,
            "alpaca crypto WS connecting"
        );
        let (ws, _) = tokio_tungstenite::connect_async(ALPACA_WS_CRYPTO).await?;
        tracing::info!("alpaca crypto WS connected; auth in flight");
        let (mut tx, mut rx) = ws.split();
        let auth = serde_json::json!({
            "action": "auth",
            "key": key_id,
            "secret": secret,
        })
        .to_string();
        tx.send(WsMessage::Text(auth)).await?;
        // Alpaca's v1beta3 crypto WS rejects non-`BASE/QUOTE` formats
        // ("invalid syntax", code 400). Normalize every symbol before
        // subscribing so user shorthand ("BTC", "BTCUSD") and already-
        // normalized values ("ETH/USD") all reach the wire as the
        // server-expected `BASE/QUOTE` form.
        let normalized: Vec<String> = symbols.iter().map(|s| to_alpaca_crypto_symbol(s)).collect();
        if !normalized.is_empty() {
            let sub = serde_json::json!({
                "action": "subscribe",
                "trades": normalized,
            })
            .to_string();
            tx.send(WsMessage::Text(sub)).await?;
        }
        while let Some(msg) = rx.next().await {
            match msg? {
                WsMessage::Text(t) => {
                    let arr: serde_json::Value = match serde_json::from_str(&t) {
                        Ok(v) => v,
                        Err(_) => continue,
                    };
                    let Some(events) = arr.as_array() else {
                        continue;
                    };
                    for ev in events {
                        let kind = ev.get("T").and_then(|v| v.as_str()).unwrap_or("");
                        if kind == "error" {
                            tracing::warn!(
                                msg = ?ev.get("msg"),
                                code = ?ev.get("code"),
                                "alpaca crypto WS error frame"
                            );
                            // Same code-class filter as the equities
                            // worker — only true auth failures (401 /
                            // 402 / 410) poison the cache. 406 implies
                            // creds work (some other connection holds
                            // the slot).
                            if let Some(code) = ev.get("code").and_then(|v| v.as_i64()) {
                                if matches!(code, 401 | 402 | 410) {
                                    self.record_alpaca_auth(false, "crypto").await;
                                } else if code == 406 {
                                    self.record_alpaca_auth(true, "crypto").await;
                                }
                            }
                            continue;
                        }
                        if kind == "success"
                            && ev.get("msg").and_then(|v| v.as_str()) == Some("authenticated")
                        {
                            self.record_alpaca_auth(true, "crypto").await;
                            tracing::info!("alpaca WS authenticated feed=crypto");
                            continue;
                        }
                        if kind == "subscription" {
                            let trades_n = ev
                                .get("trades")
                                .and_then(|v| v.as_array())
                                .map(|a| a.len())
                                .unwrap_or(0);
                            tracing::info!(trades = trades_n, "alpaca WS subscribed feed=crypto");
                            continue;
                        }
                        if kind != "t" {
                            continue;
                        }
                        let sym = ev.get("S").and_then(|v| v.as_str()).unwrap_or_default();
                        let price = ev.get("p").and_then(|v| v.as_f64()).unwrap_or(0.0);
                        let size = ev.get("s").and_then(|v| v.as_f64()).unwrap_or(0.0);
                        let ts_ms = ev
                            .get("t")
                            .and_then(|v| v.as_str())
                            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                            .map(|dt| dt.timestamp_millis())
                            .unwrap_or(0);
                        if sym.is_empty() || price <= 0.0 || ts_ms <= 0 {
                            continue;
                        }
                        // Use the Alpaca-canonical "BTC/USD" symbol AS
                        // the key — autoscan picks pass that form
                        // through ensure_subscribed → state map →
                        // price_bars, and the algo runner's
                        // fetch_recent_bars queries the same form.
                        // Denormalizing here (strip the slash) used to
                        // mis-key 10s buckets under "BTCUSD" while the
                        // runner queried "BTC/USD" → no bars found.
                        let key = sym.to_string();
                        let trade = Trade {
                            symbol: key.clone(),
                            price,
                            volume: size,
                            ts_ms,
                        };
                        if let Some(mut state) = self.state.get_mut(&key) {
                            state.observe(&trade);
                            let snap = state.clone();
                            drop(state);
                            let _ = self.tx.send(snap);
                        }
                        self.feed_bucket(&trade);
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

    /// Polygon WS worker. Reuses the same Trade-observe + feed_bucket
    /// pipeline as the Finnhub worker so every downstream consumer
    /// (live scanner, tape, 10s aggregator) keeps working untouched.
    /// Protocol:
    ///   1. Connect to `wss://socket.polygon.io/stocks`
    ///   2. `{action:"auth", params:"<api_key>"}` — wait for status=auth_success
    ///   3. `{action:"subscribe", params:"T.AAPL,T.MSFT,..."}` (T. = trades)
    ///   4. Receive `[{ev:"T", sym, p, s, t, ...}, ...]` arrays
    async fn run_polygon_worker(self, api_key: String, symbols: Vec<String>) {
        let mut backoff = Backoff::new();
        loop {
            match self.run_polygon_once(&api_key, &symbols).await {
                Ok(()) => {
                    tracing::info!("polygon WS exited cleanly; reconnecting");
                    backoff.reset();
                }
                Err(e) => {
                    let wait = backoff.next();
                    tracing::warn!(?e, ?wait, "polygon WS error; backing off");
                    tokio::time::sleep(wait).await;
                    continue;
                }
            }
            tokio::time::sleep(Duration::from_secs(3)).await;
        }
    }

    async fn run_polygon_once(&self, api_key: &str, symbols: &[String]) -> anyhow::Result<()> {
        let (ws, _) = tokio_tungstenite::connect_async(POLYGON_WS).await?;
        let (mut tx, mut rx) = ws.split();
        // 1. Authenticate.
        let auth = serde_json::json!({ "action": "auth", "params": api_key }).to_string();
        tx.send(WsMessage::Text(auth)).await?;
        // 2. Subscribe to trades for each symbol. Polygon supports
        //    `T.AAPL,T.MSFT,...` in one frame — batch for efficiency.
        let params = symbols
            .iter()
            .map(|s| format!("T.{s}"))
            .collect::<Vec<_>>()
            .join(",");
        if !params.is_empty() {
            let sub = serde_json::json!({ "action": "subscribe", "params": params }).to_string();
            tx.send(WsMessage::Text(sub)).await?;
        }
        // 3. Read loop. Polygon ships arrays of events:
        //    [{"ev":"status","status":"connected"}], [{"ev":"T",...}, ...]
        while let Some(msg) = rx.next().await {
            match msg? {
                WsMessage::Text(t) => {
                    let arr: serde_json::Value = match serde_json::from_str(&t) {
                        Ok(v) => v,
                        Err(_) => continue,
                    };
                    let Some(events) = arr.as_array() else {
                        continue;
                    };
                    for ev in events {
                        if ev.get("ev").and_then(|v| v.as_str()) != Some("T") {
                            // Could be a status, auth, or error frame;
                            // log auth failures so the user sees them.
                            if let Some(status) = ev.get("status").and_then(|v| v.as_str()) {
                                if status == "auth_failed" {
                                    tracing::warn!(
                                        msg = ?ev.get("message"),
                                        "polygon WS auth failed — falling back to finnhub on next restart"
                                    );
                                }
                            }
                            continue;
                        }
                        let sym = ev.get("sym").and_then(|v| v.as_str()).unwrap_or_default();
                        let price = ev.get("p").and_then(|v| v.as_f64()).unwrap_or(0.0);
                        let size = ev.get("s").and_then(|v| v.as_f64()).unwrap_or(0.0);
                        let ts = ev.get("t").and_then(|v| v.as_i64()).unwrap_or(0);
                        if sym.is_empty() || price <= 0.0 || ts <= 0 {
                            continue;
                        }
                        let trade = Trade {
                            symbol: sym.to_string(),
                            price,
                            volume: size,
                            ts_ms: ts,
                        };
                        if let Some(mut state) = self.state.get_mut(&trade.symbol) {
                            state.observe(&trade);
                            let snap = state.clone();
                            drop(state);
                            let _ = self.tx.send(snap);
                        }
                        self.feed_bucket(&trade);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crypto_classifier_recognises_bare_tickers() {
        assert!(is_crypto_symbol("BTC"), "bare BTC should route to crypto");
        assert!(is_crypto_symbol("ETH"), "bare ETH should route to crypto");
        assert!(is_crypto_symbol("BTCUSD"));
        assert!(is_crypto_symbol("ETHUSD"));
        assert!(!is_crypto_symbol("AAPL"));
        assert!(!is_crypto_symbol("SPY"));
        // FOOUSD: USD-ending but unknown prefix → equity (not crypto).
        assert!(!is_crypto_symbol("FOOUSD"));
    }

    #[test]
    fn crypto_classifier_recognises_slash_form() {
        // Canonical Alpaca form — the autoscan crypto universe ships
        // these directly. Regression: pre-fix, "BTC/USD" sliced to
        // prefix "BTC/" which wasn't in CRYPTO_PREFIXES, so the symbol
        // got mis-routed to the equity IEX WS where it silently never
        // received trades.
        assert!(is_crypto_symbol("BTC/USD"));
        assert!(is_crypto_symbol("ETH/USD"));
        assert!(is_crypto_symbol("SOL/USD"));
        assert!(is_crypto_symbol("AVAX/USD"));
        assert!(
            is_crypto_symbol("eth/usd"),
            "lowercase should still classify"
        );
        // Wrong quote currency stays equity-side (Alpaca crypto WS
        // only does USD pairs).
        assert!(!is_crypto_symbol("BTC/EUR"));
        // Slash form with unknown base → equity fallback.
        assert!(!is_crypto_symbol("FOO/USD"));
    }

    #[test]
    fn to_alpaca_crypto_symbol_normalises_all_input_shapes() {
        assert_eq!(to_alpaca_crypto_symbol("BTC"), "BTC/USD");
        assert_eq!(to_alpaca_crypto_symbol("btc"), "BTC/USD");
        assert_eq!(to_alpaca_crypto_symbol("BTCUSD"), "BTC/USD");
        assert_eq!(to_alpaca_crypto_symbol("ETHUSD"), "ETH/USD");
        // Already normalised — pass through (uppercased).
        assert_eq!(to_alpaca_crypto_symbol("ETH/USD"), "ETH/USD");
        assert_eq!(to_alpaca_crypto_symbol("eth/usd"), "ETH/USD");
        // 3-letter ticker that isn't USD — treat as bare crypto.
        assert_eq!(to_alpaca_crypto_symbol("SOL"), "SOL/USD");
    }

    #[test]
    fn from_alpaca_crypto_symbol_strips_slash() {
        assert_eq!(from_alpaca_crypto_symbol("BTC/USD"), "BTCUSD");
        assert_eq!(from_alpaca_crypto_symbol("ETH/USD"), "ETHUSD");
        // Already de-slashed — round-trips cleanly.
        assert_eq!(from_alpaca_crypto_symbol("BTCUSD"), "BTCUSD");
    }

    #[test]
    fn normalize_round_trip_is_idempotent() {
        // user input → Alpaca subscribe → server echoes → state key
        // must match what the user typed (or a stable canonical).
        for input in ["BTC", "BTCUSD", "ETH/USD", "eth", "DOGE"] {
            let alpaca = to_alpaca_crypto_symbol(input);
            assert!(
                alpaca.contains('/'),
                "Alpaca form must have slash: {alpaca}"
            );
            let back = from_alpaca_crypto_symbol(&alpaca);
            assert!(!back.contains('/'), "state key must have NO slash: {back}");
            // Round-trip preserves the BASE+USD body.
            let normalised_input = if input.contains('/') {
                input.to_uppercase().replace('/', "")
            } else if input.to_uppercase().ends_with("USD") {
                input.to_uppercase()
            } else {
                format!("{}USD", input.to_uppercase())
            };
            assert_eq!(back, normalised_input);
        }
    }
}
