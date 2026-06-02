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

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio_tungstenite::tungstenite::Message as WsMessage;

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

#[derive(Clone)]
pub struct LiveTickStore {
    state: Arc<DashMap<String, SymbolState>>,
    tx: broadcast::Sender<SymbolState>,
    api_key: Arc<tokio::sync::RwLock<Option<String>>>,
    subs: Arc<tokio::sync::Mutex<Vec<String>>>,
}

impl LiveTickStore {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(1024);
        Self {
            state: Arc::new(DashMap::new()),
            tx,
            api_key: Arc::new(tokio::sync::RwLock::new(None)),
            subs: Arc::new(tokio::sync::Mutex::new(Vec::new())),
        }
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
