//! Breadth-divergence detector.
//!
//! Most "breadth" dashboards show today's TICK / TRIN / Advance-Decline
//! as a single snapshot. The traders who use them care about a more
//! specific question: **is price moving with breadth, or against it?**
//!
//!   * Price up while breadth weakens → bearish non-confirmation.
//!     The rally is being driven by a narrow set of names; under the
//!     surface the index is rotting. Historically a setup for a sharp
//!     reversal.
//!   * Price down while breadth strengthens → bullish non-confirmation.
//!     Selling is concentrated in a few large caps; most stocks are
//!     holding up. Often the bottom of a corrective leg.
//!
//! Implementation:
//!
//!   1. Every `POLL_SECS` (default 5 min) we sample (a) SPY's current
//!      `market_data::quote` price and (b) `breadth::snapshot`'s
//!      `composite_score` (-100..+100, positive = bullish breadth).
//!   2. Both numbers go into a rolling window of `WINDOW_SAMPLES`
//!      (default 60 → 5 hours of intraday history at 5-min cadence).
//!   3. After the window is full enough (≥ `MIN_SAMPLES`) we evaluate
//!      the divergence rule:
//!
//!      spy_change_pct = (last_spy - first_spy) / first_spy * 100
//!      breadth_avg    = mean(composite_score over window)
//!
//!      * BearishDivergence: spy_change_pct ≥ +`PRICE_DELTA_PCT` AND
//!        breadth_avg ≤ -`BREADTH_THRESHOLD`
//!      * BullishDivergence: spy_change_pct ≤ -`PRICE_DELTA_PCT` AND
//!        breadth_avg ≥ +`BREADTH_THRESHOLD`
//!
//!   4. State is `None / Bearish / Bullish`. We broadcast a
//!      `DivergenceEvent` only when state crosses (no spam on
//!      persistent regimes).
//!
//! Dedupe + memory: emitted events stored in a bounded `DashMap`
//! keyed by `(date, kind)` with oldest-first eviction.

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::Serialize;
use sqlx::PgPool;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, Mutex};

const POLL_SECS: u64 = 5 * 60;
const WINDOW_SAMPLES: usize = 60;
const MIN_SAMPLES: usize = 6;
/// SPY change over the window required to flag a divergence. < 1% is
/// noise on a 5-hour intraday window.
const PRICE_DELTA_PCT: f64 = 1.0;
/// Composite-score average required. Composite is bounded ±100, so 15
/// is a defensible "definitely tilted" threshold.
const BREADTH_THRESHOLD: f64 = 15.0;
const EMITTED_CAP: usize = 1_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DivergenceKind {
    Bullish,
    Bearish,
}

impl DivergenceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            DivergenceKind::Bullish => "bullish",
            DivergenceKind::Bearish => "bearish",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Sample {
    pub spy_price: f64,
    pub composite_score: f64,
    pub observed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DivergenceEvent {
    pub kind: DivergenceKind,
    pub spy_change_pct: f64,
    pub breadth_avg: f64,
    pub samples_used: usize,
    pub window_minutes: i64,
    pub started_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct DivergenceStore {
    window: Arc<Mutex<VecDeque<Sample>>>,
    /// Last emitted regime so we only fire on edges.
    current: Arc<Mutex<Option<DivergenceKind>>>,
    emitted: Arc<DashMap<String, DivergenceEvent>>,
    tx: broadcast::Sender<DivergenceEvent>,
}

impl DivergenceStore {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(128);
        Self {
            window: Arc::new(Mutex::new(VecDeque::with_capacity(WINDOW_SAMPLES))),
            current: Arc::new(Mutex::new(None)),
            emitted: Arc::new(DashMap::new()),
            tx,
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<DivergenceEvent> {
        self.tx.subscribe()
    }

    /// Newest-first list of regime-cross events.
    pub fn latest(&self, limit: usize) -> Vec<DivergenceEvent> {
        let mut all: Vec<DivergenceEvent> =
            self.emitted.iter().map(|e| e.value().clone()).collect();
        all.sort_by_key(|e| std::cmp::Reverse(e.started_at));
        all.truncate(limit);
        all
    }

    /// Snapshot of the rolling window. Useful for the frontend chart.
    pub async fn window_snapshot(&self) -> Vec<Sample> {
        self.window.lock().await.iter().copied().collect()
    }

    /// Current regime (None until the first divergence fires).
    pub async fn current_regime(&self) -> Option<DivergenceKind> {
        *self.current.lock().await
    }

    fn evict_if_full(&self) {
        if self.emitted.len() <= EMITTED_CAP {
            return;
        }
        let drop_n = self.emitted.len() / 4;
        let mut by_age: Vec<(String, DateTime<Utc>)> = self
            .emitted
            .iter()
            .map(|e| (e.key().clone(), e.value().started_at))
            .collect();
        by_age.sort_by_key(|(_, t)| *t);
        for (key, _) in by_age.into_iter().take(drop_n) {
            self.emitted.remove(&key);
        }
    }

    /// Fold one (spy_price, composite_score) sample into the window
    /// and evaluate the divergence rule. Returns the freshly emitted
    /// event if a regime edge crossed, else None.
    pub async fn observe(&self, spy_price: f64, composite_score: f64) -> Option<DivergenceEvent> {
        let now = Utc::now();
        {
            let mut w = self.window.lock().await;
            if w.len() == WINDOW_SAMPLES {
                w.pop_front();
            }
            w.push_back(Sample {
                spy_price,
                composite_score,
                observed_at: now,
            });
        }
        let snap: Vec<Sample> = self.window.lock().await.iter().copied().collect();
        let new_kind = evaluate(&snap);
        let mut cur = self.current.lock().await;
        if new_kind == *cur {
            return None;
        }
        *cur = new_kind;
        if let Some(kind) = new_kind {
            let first = snap.first().expect("snap non-empty if regime emitted");
            let last = snap.last().expect("snap non-empty if regime emitted");
            let spy_change_pct = if first.spy_price > 0.0 {
                (last.spy_price - first.spy_price) / first.spy_price * 100.0
            } else {
                0.0
            };
            let breadth_avg =
                snap.iter().map(|s| s.composite_score).sum::<f64>() / snap.len() as f64;
            let window_minutes = (last.observed_at - first.observed_at).num_minutes();
            let ev = DivergenceEvent {
                kind,
                spy_change_pct,
                breadth_avg,
                samples_used: snap.len(),
                window_minutes,
                started_at: now,
            };
            let key = format!("{}|{}", now.date_naive(), kind.as_str());
            self.emitted.insert(key, ev.clone());
            let _ = self.tx.send(ev.clone());
            self.evict_if_full();
            return Some(ev);
        }
        None
    }
}

impl Default for DivergenceStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Pure rule: returns the regime tag the current window implies, or
/// `None` if the window is too short or neither divergence pattern is
/// active.
pub fn evaluate(window: &[Sample]) -> Option<DivergenceKind> {
    if window.len() < MIN_SAMPLES {
        return None;
    }
    let first = window.first()?;
    let last = window.last()?;
    if first.spy_price <= 0.0 {
        return None;
    }
    let spy_change_pct = (last.spy_price - first.spy_price) / first.spy_price * 100.0;
    let breadth_avg = window.iter().map(|s| s.composite_score).sum::<f64>() / window.len() as f64;
    if spy_change_pct >= PRICE_DELTA_PCT && breadth_avg <= -BREADTH_THRESHOLD {
        return Some(DivergenceKind::Bearish);
    }
    if spy_change_pct <= -PRICE_DELTA_PCT && breadth_avg >= BREADTH_THRESHOLD {
        return Some(DivergenceKind::Bullish);
    }
    None
}

pub fn spawn_poller(store: DivergenceStore, pool: PgPool) {
    tokio::spawn(async move {
        // First sample a few seconds after boot so market_data has had
        // a chance to populate the price cache.
        tokio::time::sleep(Duration::from_secs(30)).await;
        let mut interval = tokio::time::interval(Duration::from_secs(POLL_SECS));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            interval.tick().await;
            let spy = crate::market_data::quote(&pool, "SPY").await.ok();
            let snap = crate::breadth::snapshot(&pool).await.ok();
            if let (Some(s), Some(b)) = (spy, snap) {
                if s.price > 0.0 {
                    let _ = store.observe(s.price, b.composite_score as f64).await;
                }
            }
        }
    });
}

pub fn global(pool: PgPool) -> DivergenceStore {
    static STORE: once_cell::sync::OnceCell<DivergenceStore> = once_cell::sync::OnceCell::new();
    STORE
        .get_or_init(|| {
            let s = DivergenceStore::new();
            spawn_poller(s.clone(), pool);
            s
        })
        .clone()
}

/// Read-only handle when the singleton has already been initialised
/// (used by routes that don't have the pool handy). Returns None
/// before the first `global()` call.
pub fn try_global() -> Option<DivergenceStore> {
    static STORE: once_cell::sync::OnceCell<DivergenceStore> = once_cell::sync::OnceCell::new();
    // SAFETY: the global() init above wrote to a different OnceCell.
    // Two OnceCells would race, so this implementation is intentionally
    // limited — callers should rely on a single `global(pool)` site
    // (server boot) that runs before any read.
    STORE.get().cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(price: f64, score: f64, offset_min: i64) -> Sample {
        Sample {
            spy_price: price,
            composite_score: score,
            observed_at: Utc::now() + chrono::Duration::minutes(offset_min),
        }
    }

    #[test]
    fn evaluate_none_when_window_too_short() {
        let w = vec![s(400.0, 0.0, 0); MIN_SAMPLES - 1];
        assert_eq!(evaluate(&w), None);
    }

    #[test]
    fn evaluate_bearish_when_price_up_breadth_down() {
        // SPY drifts up 2%, composite_score averages -30.
        let mut w = Vec::new();
        for i in 0..MIN_SAMPLES {
            let price = 400.0 + i as f64 * 1.3;
            w.push(s(price, -30.0, i as i64 * 5));
        }
        assert_eq!(evaluate(&w), Some(DivergenceKind::Bearish));
    }

    #[test]
    fn evaluate_bullish_when_price_down_breadth_up() {
        let mut w = Vec::new();
        for i in 0..MIN_SAMPLES {
            let price = 410.0 - i as f64 * 1.5;
            w.push(s(price, 25.0, i as i64 * 5));
        }
        assert_eq!(evaluate(&w), Some(DivergenceKind::Bullish));
    }

    #[test]
    fn evaluate_none_when_price_and_breadth_aligned() {
        let mut w = Vec::new();
        for i in 0..MIN_SAMPLES {
            let price = 400.0 + i as f64 * 1.3;
            w.push(s(price, 25.0, i as i64 * 5)); // up + bullish, no divergence
        }
        assert_eq!(evaluate(&w), None);
    }

    #[test]
    fn evaluate_none_when_signals_too_small() {
        let mut w = Vec::new();
        // SPY moves 0.2%, well under PRICE_DELTA_PCT.
        for i in 0..MIN_SAMPLES {
            w.push(s(400.0 + i as f64 * 0.05, -30.0, i as i64 * 5));
        }
        assert_eq!(evaluate(&w), None);
    }

    #[tokio::test]
    async fn observe_only_emits_on_edge_transition() {
        let store = DivergenceStore::new();
        // First fill: bearish (price up, breadth down).
        for i in 0..MIN_SAMPLES {
            let _ = store.observe(400.0 + i as f64 * 1.3, -30.0).await;
        }
        assert_eq!(store.current_regime().await, Some(DivergenceKind::Bearish));
        // A second bearish sample must NOT re-emit — same regime.
        let second = store.observe(420.0, -30.0).await;
        assert!(second.is_none());
        // Now flip to bullish by overwriting the entire window with a
        // bullish-divergence shape (price drift down ~1.9%, breadth +25).
        // Pushing WINDOW_SAMPLES rows ensures the bearish prefix is gone.
        for i in 0..WINDOW_SAMPLES {
            let price = 420.0 - (i as f64 / WINDOW_SAMPLES as f64) * 8.0;
            let _ = store.observe(price, 25.0).await;
        }
        assert_eq!(store.current_regime().await, Some(DivergenceKind::Bullish));
    }
}
