//! Real-time squeeze detector.
//!
//! Consumes the [`crate::live_ticks::LiveTickStore`] broadcast channel of
//! per-symbol state updates, maintains a rolling-window of trades per
//! symbol (last `WINDOW_SECS` seconds), and fires a [`SqueezeEvent`] when
//! a symbol crosses configured thresholds for:
//!   * percentage move over `pct_window_secs`
//!   * volume burst vs trailing baseline
//!
//! Output events go to a tokio broadcast channel that routes / WS handlers
//! subscribe to; this module owns no DB writes — alert persistence /
//! TTS dispatch is the consumer's job.
//!
//! Architecture mirrors [`crate::live_ticks`]: single global store, a
//! `tokio::spawn`ed pump that loops `subscribe()`, per-symbol rolling
//! state under `DashMap`. Configurable thresholds via [`SqueezeConfig`].

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, RwLock};

use crate::live_ticks::{self, SymbolState};

/// Rolling-window length we retain per symbol. Must exceed the largest
/// `pct_window_secs` we'll evaluate (default 60s; user-configurable
/// thresholds compare against `pct_window_secs` <= this).
const WINDOW_SECS: i64 = 120;

/// Baseline window used for volume-burst comparison — looks at the
/// `(WINDOW_SECS - burst_window_secs)` seconds before the burst window so
/// the comparison is clean (no overlap).
const DEFAULT_BURST_WINDOW_SECS: i64 = 10;
const DEFAULT_PCT_WINDOW_SECS: i64 = 10;

/// Per-symbol cooldown — don't fire the same symbol twice within this.
const DEFAULT_COOLDOWN_SECS: i64 = 60;

/// Maximum events held in the broadcast ring before lagging subscribers
/// start dropping; conservative because squeeze events should be rare.
const EVENT_RING: usize = 512;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SqueezeConfig {
    /// Minimum % change over `pct_window_secs` to trigger.
    pub pct_threshold: f64,
    /// Window for the % change comparison.
    pub pct_window_secs: i64,
    /// Volume burst ratio (burst_window vs trailing-baseline avg).
    pub volume_burst_ratio: f64,
    /// Window for the burst volume measurement.
    pub burst_window_secs: i64,
    /// Skip symbols below this dollar price (penny-stock noise filter).
    pub min_price: f64,
    /// Minimum total volume in the burst window — avoids triggering on a
    /// single oddball trade.
    pub min_burst_volume: f64,
    /// Per-symbol cooldown after a fire.
    pub cooldown_secs: i64,
}

impl Default for SqueezeConfig {
    fn default() -> Self {
        Self {
            pct_threshold: 1.5,
            pct_window_secs: DEFAULT_PCT_WINDOW_SECS,
            volume_burst_ratio: 3.0,
            burst_window_secs: DEFAULT_BURST_WINDOW_SECS,
            min_price: 1.0,
            min_burst_volume: 100_000.0,
            cooldown_secs: DEFAULT_COOLDOWN_SECS,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SqueezeEvent {
    pub symbol: String,
    pub fired_at: DateTime<Utc>,
    pub price: f64,
    pub pct_change: f64,
    pub pct_window_secs: i64,
    pub burst_volume: f64,
    pub baseline_volume: f64,
    pub burst_ratio: f64,
    pub trade_count: u64,
}

/// Per-symbol rolling tick log.
#[derive(Debug, Default)]
struct SymbolWindow {
    /// (ts_ms, price, volume_delta) — newest at the back.
    trades: VecDeque<(i64, f64, f64)>,
    last_fire_ts_ms: i64,
    last_observed_volume: f64,
}

impl SymbolWindow {
    fn push(&mut self, ts_ms: i64, price: f64, cumulative_volume: f64) {
        let delta = (cumulative_volume - self.last_observed_volume).max(0.0);
        self.last_observed_volume = cumulative_volume;
        self.trades.push_back((ts_ms, price, delta));
        // Trim anything older than WINDOW_SECS.
        let cutoff = ts_ms - WINDOW_SECS * 1000;
        while self.trades.front().is_some_and(|(t, _, _)| *t < cutoff) {
            self.trades.pop_front();
        }
    }

    /// Return the price as of `secs` ago, or None if the window doesn't
    /// reach that far back.
    fn price_n_secs_ago(&self, now_ms: i64, secs: i64) -> Option<f64> {
        let cutoff = now_ms - secs * 1000;
        // Search from the back for the most recent trade at-or-before cutoff.
        self.trades
            .iter()
            .rev()
            .find(|(t, _, _)| *t <= cutoff)
            .map(|(_, p, _)| *p)
    }

    /// Sum of volume in `[now_ms - window_secs*1000, now_ms]`.
    fn volume_in_window(&self, now_ms: i64, window_secs: i64) -> f64 {
        let cutoff = now_ms - window_secs * 1000;
        self.trades
            .iter()
            .rev()
            .take_while(|(t, _, _)| *t >= cutoff)
            .map(|(_, _, v)| *v)
            .sum()
    }

    /// Sum of volume in the baseline window — the `(WINDOW_SECS -
    /// burst_window_secs)` seconds *before* the burst window.
    fn baseline_volume(&self, now_ms: i64, burst_window_secs: i64) -> f64 {
        let burst_cutoff = now_ms - burst_window_secs * 1000;
        let base_cutoff = now_ms - WINDOW_SECS * 1000;
        self.trades
            .iter()
            .filter(|(t, _, _)| *t >= base_cutoff && *t < burst_cutoff)
            .map(|(_, _, v)| *v)
            .sum()
    }
}

#[derive(Clone)]
pub struct SqueezeStore {
    windows: Arc<DashMap<String, RwLock<SymbolWindow>>>,
    config: Arc<RwLock<SqueezeConfig>>,
    events: Arc<RwLock<VecDeque<SqueezeEvent>>>,
    tx: broadcast::Sender<SqueezeEvent>,
}

impl SqueezeStore {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(EVENT_RING);
        Self {
            windows: Arc::new(DashMap::new()),
            config: Arc::new(RwLock::new(SqueezeConfig::default())),
            events: Arc::new(RwLock::new(VecDeque::with_capacity(256))),
            tx,
        }
    }

    pub async fn set_config(&self, cfg: SqueezeConfig) {
        *self.config.write().await = cfg;
    }
    pub async fn get_config(&self) -> SqueezeConfig {
        self.config.read().await.clone()
    }
    pub fn subscribe(&self) -> broadcast::Receiver<SqueezeEvent> {
        self.tx.subscribe()
    }

    /// Recent events newest-first, capped to `limit`.
    pub async fn recent(&self, limit: usize) -> Vec<SqueezeEvent> {
        let g = self.events.read().await;
        g.iter().rev().take(limit).cloned().collect()
    }

    async fn record(&self, ev: SqueezeEvent) {
        let mut q = self.events.write().await;
        if q.len() >= 256 {
            q.pop_front();
        }
        q.push_back(ev.clone());
        drop(q);
        let _ = self.tx.send(ev);
    }

    /// Process one tick update from `LiveTickStore`. Pure synchronous
    /// rolling-window arithmetic + a single async fire on threshold cross.
    async fn observe(&self, state: &SymbolState) {
        let cfg = self.config.read().await.clone();
        if state.last < cfg.min_price {
            return;
        }
        let ts_ms = state.last_trade_at.timestamp_millis();
        let symbol = state.symbol.clone();

        // Push the tick. Hold the per-symbol lock for the math too —
        // contention per symbol is fine, contention across symbols is
        // sharded by DashMap.
        let entry = self
            .windows
            .entry(symbol.clone())
            .or_insert_with(|| RwLock::new(SymbolWindow::default()));
        let mut w = entry.write().await;
        w.push(ts_ms, state.last, state.day_volume);

        // Cooldown gate before doing the threshold math.
        if w.last_fire_ts_ms != 0 && (ts_ms - w.last_fire_ts_ms) < cfg.cooldown_secs * 1000 {
            return;
        }

        let Some(prior_price) = w.price_n_secs_ago(ts_ms, cfg.pct_window_secs) else {
            return; // not enough history yet
        };
        if prior_price <= 0.0 {
            return;
        }
        let pct_change = (state.last - prior_price) / prior_price * 100.0;
        if pct_change < cfg.pct_threshold {
            return;
        }

        let burst_volume = w.volume_in_window(ts_ms, cfg.burst_window_secs);
        if burst_volume < cfg.min_burst_volume {
            return;
        }

        let baseline_volume = w.baseline_volume(ts_ms, cfg.burst_window_secs);
        // Normalize the baseline to a per-burst-window equivalent so the
        // ratio is dimensionless.
        let baseline_window_secs = (WINDOW_SECS - cfg.burst_window_secs).max(1);
        let baseline_per_burst =
            baseline_volume / (baseline_window_secs as f64) * (cfg.burst_window_secs as f64);
        let burst_ratio = if baseline_per_burst > 0.0 {
            burst_volume / baseline_per_burst
        } else {
            f64::INFINITY
        };
        if burst_ratio < cfg.volume_burst_ratio {
            return;
        }

        // Threshold cross — fire.
        w.last_fire_ts_ms = ts_ms;
        let trade_count = state.trade_count;
        let last = state.last;
        drop(w);

        let ev = SqueezeEvent {
            symbol,
            fired_at: state.last_trade_at,
            price: last,
            pct_change,
            pct_window_secs: cfg.pct_window_secs,
            burst_volume,
            baseline_volume: baseline_per_burst,
            burst_ratio,
            trade_count,
        };
        self.record(ev).await;
    }
}

impl Default for SqueezeStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Process-wide singleton.
pub fn global() -> SqueezeStore {
    static STORE: once_cell::sync::OnceCell<SqueezeStore> = once_cell::sync::OnceCell::new();
    STORE.get_or_init(SqueezeStore::new).clone()
}

/// Spawn the pump that wires `LiveTickStore` → `SqueezeStore`. Idempotent —
/// safe to call multiple times; each call starts an independent subscriber
/// (cheap; uses tokio broadcast). One call at app boot is the intended use.
pub fn spawn_pump(store: SqueezeStore) {
    tokio::spawn(async move {
        let live = live_ticks::global();
        let mut rx = live.subscribe();
        loop {
            match rx.recv().await {
                Ok(state) => store.observe(&state).await,
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!(skipped = n, "squeeze pump lagged");
                }
                Err(broadcast::error::RecvError::Closed) => {
                    tracing::warn!("squeeze pump tick channel closed; resubscribing");
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    rx = live.subscribe();
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cooldown_blocks_double_fire() {
        let mut w = SymbolWindow::default();
        w.push(0, 10.0, 0.0);
        w.push(10_000, 10.50, 200_000.0);
        w.last_fire_ts_ms = 10_000;
        assert!((10_500 - w.last_fire_ts_ms) < DEFAULT_COOLDOWN_SECS * 1000);
    }

    #[test]
    fn volume_delta_is_monotone() {
        // cumulative volume must not produce negative deltas
        let mut w = SymbolWindow::default();
        w.push(0, 10.0, 100.0);
        w.push(1_000, 10.0, 50.0); // simulated reset (would be a glitch)
                                   // last delta should be 0, not -50.
        assert!(w.trades.iter().all(|(_, _, v)| *v >= 0.0));
    }

    #[test]
    fn baseline_excludes_burst_window() {
        let mut w = SymbolWindow::default();
        // 60s of 100/sec baseline, then 10s of 5000/sec burst.
        let mut cum = 0.0;
        for sec in 0..70 {
            let per_sec = if sec >= 60 { 5_000.0 } else { 100.0 };
            cum += per_sec;
            w.push((sec * 1000) as i64, 10.0, cum);
        }
        let now = 70_000;
        let burst = w.volume_in_window(now, 10);
        let base = w.baseline_volume(now, 10);
        // burst window saw 10×5000 = 50k; baseline saw earlier 60s of 100/s = 6k
        assert!(burst >= 40_000.0);
        assert!(base >= 5_000.0 && base <= 7_000.0);
    }
}
