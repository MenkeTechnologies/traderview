//! Catalyst-driven candidate aggregator for the live-ticks scanner.
//!
//! Subscribes to the existing catalyst + halt firehoses, scores each
//! observed symbol by recency + source priority, and periodically
//! reconciles the LiveTickStore subscription set so the streaming layer
//! always covers the top-N catalyst candidates.
//!
//! Scoring decay: a fresh catalyst scores 100; the score halves every
//! `HALF_LIFE_SECS` (default 30 min). After ~6 half-lives the candidate
//! drops below the cutoff and is replaced by something fresher.
//!
//! Output: drives [`crate::live_ticks::LiveTickStore::set_symbols`] with
//! the current top-N every `RECONCILE_SECS`. Top-N is configurable per
//! the upstream streaming cap:
//!   * Finnhub free: 25
//!   * Finnhub paid: 50
//!   * Webull MQTT: ~500 (Phase 2)
//!   * Polygon Developer: unbounded — bypass the candidate filter

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, RwLock};

use crate::catalysts::{self, Catalyst, CatalystKind};
use crate::halts::{self, Halt};
use crate::live_ticks;

const HALF_LIFE_SECS: f64 = 1_800.0; // 30 min
const RECONCILE_SECS: u64 = 30;
const DEFAULT_TOP_N: usize = 25; // Finnhub free tier ceiling.
const MIN_KEEP_SCORE: f64 = 1.0; // candidates below this get evicted.

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    EdgarFiling,
    PressRelease,
    Halt,
    HaltResume,
}

impl Source {
    /// Base score before recency decay. Higher = stronger trigger.
    fn base_score(self) -> f64 {
        match self {
            Source::EdgarFiling => 70.0,  // 8-K, S-1, etc. — concrete catalyst
            Source::PressRelease => 60.0, // BW / PRN / GNW — sometimes fluff
            Source::Halt => 100.0,        // halt while live trading = strongest
            Source::HaltResume => 90.0,   // resume = imminent move
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Candidate {
    pub symbol: String,
    pub score: f64,
    pub last_source: Source,
    pub last_seen: DateTime<Utc>,
    pub last_title: Option<String>,
    pub hit_count: u32,
}

#[derive(Clone)]
pub struct CandidateStore {
    scored: Arc<DashMap<String, Candidate>>,
    top_n: Arc<RwLock<usize>>,
    tx: broadcast::Sender<Vec<Candidate>>,
}

impl CandidateStore {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(64);
        Self {
            scored: Arc::new(DashMap::new()),
            top_n: Arc::new(RwLock::new(DEFAULT_TOP_N)),
            tx,
        }
    }

    pub async fn set_top_n(&self, n: usize) {
        *self.top_n.write().await = n.max(1);
    }

    pub async fn get_top_n(&self) -> usize {
        *self.top_n.read().await
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Vec<Candidate>> {
        self.tx.subscribe()
    }

    /// Score-ranked snapshot, decayed to `now`.
    pub fn snapshot(&self) -> Vec<Candidate> {
        let now = Utc::now();
        let mut rows: Vec<Candidate> = self
            .scored
            .iter()
            .map(|e| {
                let mut c = e.value().clone();
                c.score = decay(c.score, c.last_seen, now);
                c
            })
            .filter(|c| c.score >= MIN_KEEP_SCORE)
            .collect();
        rows.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        rows
    }

    fn observe(&self, symbol: &str, source: Source, title: Option<String>) {
        let sym = symbol.to_ascii_uppercase();
        let now = Utc::now();
        let base = source.base_score();
        self.scored
            .entry(sym.clone())
            .and_modify(|c| {
                // Decay the existing score to "now" before adding the fresh
                // base — avoids re-charging old candidates from a stale score.
                let decayed = decay(c.score, c.last_seen, now);
                // Additive accumulation but capped so a single noisy symbol
                // can't dominate the ranking forever.
                c.score = (decayed + base).min(500.0);
                c.last_source = source;
                c.last_seen = now;
                c.last_title = title.clone();
                c.hit_count += 1;
            })
            .or_insert(Candidate {
                symbol: sym,
                score: base,
                last_source: source,
                last_seen: now,
                last_title: title,
                hit_count: 1,
            });
    }

    /// Drop entries whose score has decayed below the keep threshold so
    /// the DashMap doesn't grow unbounded over a long session.
    fn evict_stale(&self) {
        let now = Utc::now();
        let stale: Vec<String> = self
            .scored
            .iter()
            .filter_map(|e| {
                let c = e.value();
                let s = decay(c.score, c.last_seen, now);
                if s < MIN_KEEP_SCORE {
                    Some(c.symbol.clone())
                } else {
                    None
                }
            })
            .collect();
        for s in stale {
            self.scored.remove(&s);
        }
    }
}

impl Default for CandidateStore {
    fn default() -> Self {
        Self::new()
    }
}

fn decay(score: f64, observed_at: DateTime<Utc>, now: DateTime<Utc>) -> f64 {
    let age_secs = (now - observed_at).num_seconds().max(0) as f64;
    score * (0.5_f64).powf(age_secs / HALF_LIFE_SECS)
}

/// Process-wide singleton.
pub fn global() -> CandidateStore {
    static STORE: once_cell::sync::OnceCell<CandidateStore> = once_cell::sync::OnceCell::new();
    STORE.get_or_init(CandidateStore::new).clone()
}

/// Spawn the aggregator. Single-call at app boot.
pub fn spawn_aggregator(store: CandidateStore) {
    let cat_store = catalysts::global();
    let halt_store = halts::global();

    // Catalyst pump.
    {
        let store = store.clone();
        let mut rx = cat_store.subscribe();
        tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(c) => ingest_catalyst(&store, c),
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        });
    }

    // Halt pump.
    {
        let store = store.clone();
        let mut rx = halt_store.subscribe();
        tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(h) => ingest_halt(&store, h),
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        });
    }

    // Reconcile loop — push top-N to LiveTickStore + broadcast snapshot.
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(Duration::from_secs(RECONCILE_SECS));
        loop {
            tick.tick().await;
            store.evict_stale();
            let snap = store.snapshot();
            let n = store.get_top_n().await;
            let symbols: Vec<String> =
                snap.iter().take(n).map(|c| c.symbol.clone()).collect();
            if !symbols.is_empty() {
                let live = live_ticks::global();
                if live.has_key().await {
                    if let Err(e) = live.set_symbols(symbols.clone()).await {
                        tracing::warn!(error = %e, "live_ticks set_symbols failed");
                    } else {
                        tracing::info!(
                            n = symbols.len(),
                            "candidate reconcile pushed to live_ticks"
                        );
                    }
                }
            }
            let _ = store.tx.send(snap);
        }
    });
}

fn ingest_catalyst(store: &CandidateStore, c: Catalyst) {
    let source = match c.kind {
        CatalystKind::SecFiling => Source::EdgarFiling,
        CatalystKind::PressRelease => Source::PressRelease,
    };
    for sym in &c.tickers {
        store.observe(sym, source, Some(c.title.clone()));
    }
}

fn ingest_halt(store: &CandidateStore, h: Halt) {
    // Treat resume as a separate, hotter signal than the halt itself —
    // resume is when the move actually happens.
    let resumed = !h.reason_code.is_empty()
        && (h.reason_label.eq_ignore_ascii_case("Quotation Resumption")
            || h.reason_code == "C9"
            || h.reason_code == "R4");
    let source = if resumed { Source::HaltResume } else { Source::Halt };
    let title = Some(format!("HALT {} ({})", h.symbol, h.reason_label));
    store.observe(&h.symbol, source, title);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decay_halves_every_half_life() {
        let now = Utc::now();
        let then = now - chrono::Duration::seconds(HALF_LIFE_SECS as i64);
        let s = decay(100.0, then, now);
        assert!((s - 50.0).abs() < 0.01, "expected ~50, got {s}");
    }

    #[test]
    fn observe_dedupes_and_accumulates() {
        let store = CandidateStore::new();
        store.observe("AAPL", Source::PressRelease, Some("first".into()));
        store.observe("AAPL", Source::PressRelease, Some("second".into()));
        let snap = store.snapshot();
        assert_eq!(snap.len(), 1);
        assert_eq!(snap[0].hit_count, 2);
        assert!(snap[0].score > Source::PressRelease.base_score());
    }

    #[test]
    fn halt_outranks_pr_at_equal_age() {
        let store = CandidateStore::new();
        store.observe("XXX", Source::PressRelease, None);
        store.observe("YYY", Source::Halt, None);
        let snap = store.snapshot();
        assert_eq!(snap[0].symbol, "YYY");
    }
}
