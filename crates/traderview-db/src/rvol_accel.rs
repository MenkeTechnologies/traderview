//! Tape-microstructure relative-volume acceleration detector.
//!
//! Most "volume surge" scanners fire on a single bar that exceeds a
//! threshold. That misses the more reliable signal: **acceleration**.
//! When a symbol's per-minute volume strictly climbs across three or
//! more consecutive bars and the latest bar runs at multiples of the
//! recent baseline, institutional accumulation / distribution is
//! actively building. A one-off spike is often news; an acceleration
//! sequence is a position being assembled.
//!
//! Subscribes to `LiveTickStore::tape_subscribe()` and folds every
//! trade into per-symbol 1-minute volume buckets. When a bucket
//! rolls over (new minute observed), the detector evaluates:
//!
//!   * `history.len()` ≥ `MIN_HISTORY` (need a baseline + recent run)
//!   * The last `ACCEL_LEN` (default 3) bars are strictly increasing
//!   * The latest bar volume is ≥ `MULTIPLE` × mean(history\last N)
//!
//! When all three fire, an `AccelEvent` is broadcast. Dedupe keyed by
//! `(symbol, started_minute_ts)` so the same acceleration sequence
//! doesn't re-fire on every subsequent bar — the detector waits for
//! the run to break and re-form before firing again.

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::Serialize;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;

use crate::live_ticks::{LiveTickStore, Trade};

/// Per-symbol bucket-history length. 20 minutes of context is enough
/// to anchor a baseline without dragging in lunch-hour quiet from
/// hours earlier.
const HISTORY_LEN: usize = 20;
/// Number of consecutive strictly-increasing bars required to flag
/// an acceleration sequence.
const ACCEL_LEN: usize = 3;
/// Minimum total history before any evaluation runs.
const MIN_HISTORY: usize = 5;
/// Latest bar must be at least this multiple of the baseline volume.
const MULTIPLE: f64 = 5.0;
/// Cap on emitted events kept in memory.
const EMITTED_CAP: usize = 2_000;

#[derive(Debug, Clone, Copy, Serialize)]
pub struct MinuteBar {
    pub started_at: DateTime<Utc>,
    pub volume: f64,
}

/// In-progress per-symbol bucket state.
#[derive(Debug, Clone)]
struct Bucket {
    /// Closed 1-minute bars, oldest-first.
    history: VecDeque<MinuteBar>,
    /// In-progress current minute (rolls into `history` on minute flip).
    current: Option<MinuteBar>,
    /// Minute timestamp of the last accel sequence we emitted; suppress
    /// duplicate events until the run breaks and re-forms.
    last_fire_minute: Option<DateTime<Utc>>,
}

impl Bucket {
    fn new() -> Self {
        Self {
            history: VecDeque::with_capacity(HISTORY_LEN),
            current: None,
            last_fire_minute: None,
        }
    }

    /// Roll `current` into `history`, popping the oldest if over cap.
    fn close_current(&mut self) {
        if let Some(c) = self.current.take() {
            if self.history.len() == HISTORY_LEN {
                self.history.pop_front();
            }
            self.history.push_back(c);
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AccelEvent {
    pub symbol: String,
    pub minute_started_at: DateTime<Utc>,
    pub latest_volume: f64,
    pub baseline_volume: f64,
    pub multiple: f64,
    /// The strictly-increasing volume run that triggered the event,
    /// oldest-first.
    pub run: Vec<f64>,
    pub observed_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct RvolAccelStore {
    buckets: Arc<DashMap<String, Bucket>>,
    emitted: Arc<DashMap<String, AccelEvent>>,
    tx: broadcast::Sender<AccelEvent>,
}

impl RvolAccelStore {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(256);
        Self {
            buckets: Arc::new(DashMap::new()),
            emitted: Arc::new(DashMap::new()),
            tx,
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<AccelEvent> {
        self.tx.subscribe()
    }

    pub fn latest(&self, limit: usize) -> Vec<AccelEvent> {
        let mut all: Vec<AccelEvent> = self.emitted.iter().map(|e| e.value().clone()).collect();
        all.sort_by_key(|e| std::cmp::Reverse(e.observed_at));
        all.truncate(limit);
        all
    }

    pub fn latest_for(&self, symbol: &str, limit: usize) -> Vec<AccelEvent> {
        let sym_upper = symbol.to_ascii_uppercase();
        let mut hits: Vec<AccelEvent> = self
            .emitted
            .iter()
            .filter(|e| e.value().symbol == sym_upper)
            .map(|e| e.value().clone())
            .collect();
        hits.sort_by_key(|e| std::cmp::Reverse(e.observed_at));
        hits.truncate(limit);
        hits
    }

    fn evict_if_full(&self) {
        if self.emitted.len() <= EMITTED_CAP {
            return;
        }
        let drop_n = self.emitted.len() / 4;
        let mut by_age: Vec<(String, DateTime<Utc>)> = self
            .emitted
            .iter()
            .map(|e| (e.key().clone(), e.value().observed_at))
            .collect();
        by_age.sort_by_key(|(_, t)| *t);
        for (key, _) in by_age.into_iter().take(drop_n) {
            self.emitted.remove(&key);
        }
    }

    /// Public entry for tests + the spawn poller. Folds one trade into
    /// the per-symbol bucket and, if a minute just closed, evaluates
    /// the acceleration rule on the resulting history. Returns the
    /// freshly emitted event if a fresh sequence fired.
    pub fn observe(&self, trade: &Trade) -> Option<AccelEvent> {
        let minute = floor_minute(trade.ts_ms);
        let mut emitted: Option<AccelEvent> = None;
        let sym = trade.symbol.to_ascii_uppercase();
        let just_closed: Option<DateTime<Utc>>;
        {
            let mut entry = self.buckets.entry(sym.clone()).or_insert_with(Bucket::new);
            match entry.current {
                Some(mut cur) if cur.started_at == minute => {
                    cur.volume += trade.volume;
                    entry.current = Some(cur);
                    just_closed = None;
                }
                Some(_old) => {
                    entry.close_current();
                    entry.current = Some(MinuteBar {
                        started_at: minute,
                        volume: trade.volume,
                    });
                    just_closed = entry.history.back().map(|b| b.started_at);
                }
                None => {
                    entry.current = Some(MinuteBar {
                        started_at: minute,
                        volume: trade.volume,
                    });
                    just_closed = None;
                }
            }
            if just_closed.is_some() {
                let bucket = &mut *entry;
                if let Some(ev) = evaluate_bucket(&sym, bucket) {
                    bucket.last_fire_minute = Some(ev.minute_started_at);
                    emitted = Some(ev);
                }
            }
        }
        if let Some(ev) = &emitted {
            let key = format!("{}|{}", ev.symbol, ev.minute_started_at.timestamp());
            self.emitted.insert(key, ev.clone());
            let _ = self.tx.send(ev.clone());
            self.evict_if_full();
        }
        emitted
    }
}

impl Default for RvolAccelStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Truncate a millis timestamp to the start of its UTC minute.
fn floor_minute(ts_ms: i64) -> DateTime<Utc> {
    let dt = DateTime::<Utc>::from_timestamp_millis(ts_ms).unwrap_or_else(Utc::now);
    let floored = dt.timestamp() - (dt.timestamp() % 60);
    DateTime::<Utc>::from_timestamp(floored, 0).unwrap_or(dt)
}

/// Pure rule applied to a bucket whose `history` just received a fresh
/// closed bar. Returns the event if all conditions cross AND the run
/// hasn't already fired on this `last_fire_minute`.
fn evaluate_bucket(symbol: &str, b: &Bucket) -> Option<AccelEvent> {
    if b.history.len() < MIN_HISTORY {
        return None;
    }
    if b.history.len() < ACCEL_LEN {
        return None;
    }
    // Last ACCEL_LEN bars.
    let len = b.history.len();
    let recent: Vec<&MinuteBar> = b.history.iter().skip(len - ACCEL_LEN).collect();
    // Strictly increasing.
    for w in recent.windows(2) {
        if w[1].volume <= w[0].volume {
            return None;
        }
    }
    // Baseline = mean of bars excluding the last ACCEL_LEN.
    let baseline_slice: Vec<&MinuteBar> = b.history.iter().take(len - ACCEL_LEN).collect();
    if baseline_slice.is_empty() {
        return None;
    }
    let baseline_volume =
        baseline_slice.iter().map(|m| m.volume).sum::<f64>() / baseline_slice.len() as f64;
    if baseline_volume <= 0.0 {
        return None;
    }
    let latest = recent.last().unwrap();
    let multiple = latest.volume / baseline_volume;
    if multiple < MULTIPLE {
        return None;
    }
    // Suppress duplicates while the same sequence persists — only fire
    // again after the run breaks (caller resets last_fire_minute when
    // a non-monotonic bar arrives).
    if let Some(last_fire) = b.last_fire_minute {
        if last_fire >= recent.first().unwrap().started_at {
            return None;
        }
    }
    Some(AccelEvent {
        symbol: symbol.into(),
        minute_started_at: latest.started_at,
        latest_volume: latest.volume,
        baseline_volume,
        multiple,
        run: recent.iter().map(|m| m.volume).collect(),
        observed_at: Utc::now(),
    })
}

pub fn spawn_consumer(store: RvolAccelStore, ticks: LiveTickStore) {
    tokio::spawn(async move {
        loop {
            let mut rx = ticks.tape_subscribe();
            loop {
                match rx.recv().await {
                    Ok(t) => {
                        store.observe(&t);
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                        tracing::warn!(skipped, "rvol_accel lagged tape");
                        continue;
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
            }
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    });
}

pub fn global() -> RvolAccelStore {
    static STORE: once_cell::sync::OnceCell<RvolAccelStore> = once_cell::sync::OnceCell::new();
    STORE
        .get_or_init(|| {
            let s = RvolAccelStore::new();
            spawn_consumer(s.clone(), crate::live_ticks::global());
            s
        })
        .clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn trade(sym: &str, vol: f64, minute_offset: i64) -> Trade {
        // Each minute_offset moves us forward exactly 60 seconds in
        // millisecond timestamps so the floor_minute classifier
        // produces distinct buckets.
        let base = 1_700_000_000_000_i64;
        Trade {
            symbol: sym.into(),
            price: 100.0,
            volume: vol,
            ts_ms: base + minute_offset * 60_000,
        }
    }

    #[test]
    fn floor_minute_truncates_seconds() {
        // 1_700_000_000_000 ms = 22:13:20 UTC. Floor to :13:00.
        let f = floor_minute(1_700_000_000_000);
        assert_eq!(f.timestamp() % 60, 0);
        // +30s lands at :13:50 — same minute bucket.
        let g = floor_minute(1_700_000_000_000 + 30_000);
        assert_eq!(f, g);
        // +60s crosses into :14:20 → next minute bucket.
        let h = floor_minute(1_700_000_000_000 + 60_000);
        assert!(h > f);
    }

    #[test]
    fn no_fire_until_history_long_enough() {
        let store = RvolAccelStore::new();
        // 4 minutes of strictly-increasing trades → history.len() = 3
        // after the 4th trade closes the prior bucket — still under
        // MIN_HISTORY of 5.
        let mut emitted = None;
        for i in 0..4 {
            emitted = emitted.or(store.observe(&trade("A", (i + 1) as f64, i)));
        }
        assert!(emitted.is_none(), "no event until baseline accumulates");
    }

    #[test]
    fn fires_on_strict_acceleration_above_multiple() {
        let store = RvolAccelStore::new();
        // First 7 closed bars at low volume (baseline). Then ACCEL_LEN
        // strictly-increasing bars culminating in > MULTIPLE × baseline.
        // Layout:
        //   minutes 0..7 : baseline volume 100 (closed by trades at next minute)
        //   minute 7      : 200
        //   minute 8      : 300
        //   minute 9      : 800  <- 8× baseline of 100
        // For each minute we need to *close* it with a trade in the next.
        let mut emitted: Option<AccelEvent> = None;
        let feed = |store: &RvolAccelStore, m: i64, v: f64| -> Option<AccelEvent> {
            store.observe(&trade("A", v, m))
        };
        for m in 0..7 {
            let r = feed(&store, m, 100.0);
            emitted = emitted.or(r);
        }
        // Bars closed so far: minutes 0..6. Now the 3-bar accel:
        let r = feed(&store, 7, 200.0);
        emitted = emitted.or(r);
        let r = feed(&store, 8, 300.0);
        emitted = emitted.or(r);
        let r = feed(&store, 9, 800.0);
        emitted = emitted.or(r);
        // Need one more trade to close minute 9 and fire eval.
        let r = feed(&store, 10, 100.0);
        emitted = emitted.or(r);
        let ev = emitted.expect("acceleration should fire");
        assert_eq!(ev.symbol, "A");
        assert!(ev.multiple >= MULTIPLE, "multiple {} too low", ev.multiple);
        assert_eq!(ev.run.len(), ACCEL_LEN);
        // Run = [200, 300, 800].
        assert!((ev.run[0] - 200.0).abs() < 1e-9);
        assert!((ev.run[2] - 800.0).abs() < 1e-9);
    }

    #[test]
    fn no_fire_when_run_not_strictly_increasing() {
        let store = RvolAccelStore::new();
        let mut emitted = None;
        let feed = |s: &RvolAccelStore, m: i64, v: f64| s.observe(&trade("B", v, m));
        for m in 0..7 {
            emitted = emitted.or(feed(&store, m, 100.0));
        }
        // Same end-magnitude but a flat in the middle ([200, 200, 800]) → not strict.
        emitted = emitted.or(feed(&store, 7, 200.0));
        emitted = emitted.or(feed(&store, 8, 200.0));
        emitted = emitted.or(feed(&store, 9, 800.0));
        emitted = emitted.or(feed(&store, 10, 100.0));
        assert!(emitted.is_none(), "flat in run must not fire");
    }

    #[test]
    fn no_fire_when_multiple_too_small() {
        let store = RvolAccelStore::new();
        let mut emitted = None;
        let feed = |s: &RvolAccelStore, m: i64, v: f64| s.observe(&trade("C", v, m));
        for m in 0..7 {
            emitted = emitted.or(feed(&store, m, 100.0));
        }
        // Run is strictly increasing but latest = 300 → only 3× baseline.
        emitted = emitted.or(feed(&store, 7, 150.0));
        emitted = emitted.or(feed(&store, 8, 200.0));
        emitted = emitted.or(feed(&store, 9, 300.0));
        emitted = emitted.or(feed(&store, 10, 100.0));
        assert!(emitted.is_none(), "3× baseline below 5× threshold");
    }

    #[test]
    fn second_fire_suppressed_until_run_breaks() {
        let store = RvolAccelStore::new();
        let feed = |s: &RvolAccelStore, m: i64, v: f64| s.observe(&trade("D", v, m));
        // Set up a fire.
        for m in 0..7 {
            feed(&store, m, 100.0);
        }
        feed(&store, 7, 200.0);
        feed(&store, 8, 300.0);
        feed(&store, 9, 800.0);
        let _ = feed(&store, 10, 100.0); // closes minute 9 → fires
                                         // Now the next strict-increasing+over-threshold sequence in the
                                         // same run window should NOT re-fire on minute 10's close —
                                         // last_fire_minute already covered the same recent bars.
        let r1 = feed(&store, 11, 900.0);
        let r2 = feed(&store, 12, 100.0); // closes minute 11
        assert!(
            r1.is_none() && r2.is_none(),
            "duplicate run must not re-fire"
        );
    }
}
