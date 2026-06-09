//! Sentiment-velocity detector.
//!
//! The existing `sentiment` module polls Reddit WSB + Stocktwits and
//! stores per-post rows in `mentions`. Its `ranked()` query already
//! computes hour-over-hour count deltas. What's missing is the
//! tradeable signal: **acceleration**. A symbol going from 50 mentions/hr
//! to 200/hr in two consecutive hours is the start of a real social
//! frenzy; one-off spikes are usually noise.
//!
//! This module wraps `ranked(hours=1)` with:
//!
//!   1. A per-symbol counter that increments whenever the symbol's
//!      `mention_count / max(prev_count, 1) >= VELOCITY_RATIO` AND
//!      `mention_count >= MIN_MENTIONS` (so quiet symbols don't qualify).
//!   2. Counter resets to 0 the moment the symbol fails the velocity
//!      condition on a check.
//!   3. When the counter reaches `CONSECUTIVE_THRESHOLD` (default 2),
//!      a `VelocityEvent` is broadcast.
//!   4. A `COOLDOWN_HOURS` window suppresses re-fires for the same
//!      symbol so a sustained two-day frenzy fires once, not 96 times.
//!
//! Cadence: 15-minute background refresh — Reddit + Stocktwits pollers
//! run every 60s, so 15 min is plenty of resolution and avoids
//! hammering the DB.

use chrono::{DateTime, Duration, Utc};
use dashmap::DashMap;
use rust_decimal::prelude::ToPrimitive;
use serde::Serialize;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::broadcast;

use crate::sentiment;

const REFRESH_SECS: u64 = 15 * 60;
/// Minimum cur/prev ratio to count as accelerating. 3× hour-over-hour
/// is the typical "definitely something" floor.
const VELOCITY_RATIO: f64 = 3.0;
/// Minimum mention count in the current hour. Avoids tiny tickers
/// jumping from 2 mentions to 8 and triggering.
const MIN_MENTIONS: i64 = 20;
/// Consecutive checks above threshold required before firing.
const CONSECUTIVE_THRESHOLD: u32 = 2;
/// After firing, suppress re-fires of the same symbol for this long.
const COOLDOWN_HOURS: i64 = 12;
/// Max ranked rows we pull from `sentiment::ranked` per round.
const RANK_LIMIT: i64 = 50;
const EMITTED_CAP: usize = 1_000;

#[derive(Debug, Clone, Serialize)]
pub struct VelocityEvent {
    pub symbol: String,
    pub mention_count: i64,
    pub prev_count: i64,
    pub ratio: f64,
    pub avg_sentiment: f64,
    pub sentiment_delta: f64,
    pub consecutive_hours: u32,
    pub fired_at: DateTime<Utc>,
}

/// One symbol's running acceleration state.
#[derive(Debug, Clone, Copy, Default)]
struct SymbolState {
    consecutive: u32,
    last_fire_at: Option<DateTime<Utc>>,
}

#[derive(Clone)]
pub struct VelocityStore {
    states: Arc<DashMap<String, SymbolState>>,
    emitted: Arc<DashMap<String, VelocityEvent>>,
    tx: broadcast::Sender<VelocityEvent>,
}

impl VelocityStore {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(128);
        Self {
            states: Arc::new(DashMap::new()),
            emitted: Arc::new(DashMap::new()),
            tx,
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<VelocityEvent> {
        self.tx.subscribe()
    }

    pub fn latest(&self, limit: usize) -> Vec<VelocityEvent> {
        let mut all: Vec<VelocityEvent> = self.emitted.iter().map(|e| e.value().clone()).collect();
        all.sort_by_key(|e| std::cmp::Reverse(e.fired_at));
        all.truncate(limit);
        all
    }

    fn evict_if_full(&self) {
        if self.emitted.len() <= EMITTED_CAP {
            return;
        }
        let drop_n = self.emitted.len() / 4;
        let mut by_age: Vec<(String, DateTime<Utc>)> = self
            .emitted
            .iter()
            .map(|e| (e.key().clone(), e.value().fired_at))
            .collect();
        by_age.sort_by_key(|(_, t)| *t);
        for (key, _) in by_age.into_iter().take(drop_n) {
            self.emitted.remove(&key);
        }
    }
}

impl Default for VelocityStore {
    fn default() -> Self {
        Self::new()
    }
}

/// One row of input to the pure evaluator — a thin projection of
/// `sentiment::RankedSymbol` (we don't want a test dependency on sqlx
/// types).
#[derive(Debug, Clone)]
pub struct RankedInput {
    pub symbol: String,
    pub mention_count: i64,
    pub prev_count: i64,
    pub avg_sentiment: f64,
    pub sentiment_delta: f64,
}

/// Pure: given a batch of ranked rows + the prior states + a `now`
/// timestamp, return the freshly fired events and the updated state
/// map. Symbols that fail the velocity condition this round have their
/// counters reset (so a single quiet hour breaks the run).
pub fn evaluate(
    rows: &[RankedInput],
    states: &mut std::collections::HashMap<String, SymbolState>,
    now: DateTime<Utc>,
) -> Vec<VelocityEvent> {
    let mut out = Vec::new();
    // Track which symbols we touched this round so we can reset any
    // previously-tracked ones that fell out of the ranked window
    // (i.e. their volume dropped so far they didn't make the top-N).
    let mut touched: std::collections::HashSet<String> = std::collections::HashSet::new();
    for r in rows {
        touched.insert(r.symbol.clone());
        let ratio = if r.prev_count > 0 {
            r.mention_count as f64 / r.prev_count as f64
        } else if r.mention_count >= MIN_MENTIONS {
            // Going from 0 → meaningful is infinite acceleration.
            f64::INFINITY
        } else {
            0.0
        };
        let qualifies = ratio >= VELOCITY_RATIO && r.mention_count >= MIN_MENTIONS;
        let entry = states.entry(r.symbol.clone()).or_default();
        if qualifies {
            entry.consecutive += 1;
            let in_cooldown = entry
                .last_fire_at
                .map(|t| now - t < Duration::hours(COOLDOWN_HOURS))
                .unwrap_or(false);
            if entry.consecutive >= CONSECUTIVE_THRESHOLD && !in_cooldown {
                entry.last_fire_at = Some(now);
                out.push(VelocityEvent {
                    symbol: r.symbol.clone(),
                    mention_count: r.mention_count,
                    prev_count: r.prev_count,
                    ratio,
                    avg_sentiment: r.avg_sentiment,
                    sentiment_delta: r.sentiment_delta,
                    consecutive_hours: entry.consecutive,
                    fired_at: now,
                });
            }
        } else {
            entry.consecutive = 0;
        }
    }
    // Any symbol previously tracked but not in this round's ranked
    // rows has fallen out of the firehose entirely — reset its
    // consecutive counter so it can't "save up" inertia across quiet
    // windows.
    for (k, v) in states.iter_mut() {
        if !touched.contains(k) {
            v.consecutive = 0;
        }
    }
    out
}

/// Repository function: one tick of the refresher. Pulls ranked
/// rows from the live DB, evaluates, persists state + emits events.
pub async fn refresh_once(pool: &PgPool, store: &VelocityStore) -> anyhow::Result<usize> {
    let rows = sentiment::ranked(pool, 1, RANK_LIMIT).await?;
    let inputs: Vec<RankedInput> = rows
        .into_iter()
        .map(|r| RankedInput {
            symbol: r.symbol,
            mention_count: r.mention_count,
            prev_count: r.prev_count,
            avg_sentiment: r.avg_sentiment.to_f64().unwrap_or(0.0),
            sentiment_delta: r.sentiment_delta.to_f64().unwrap_or(0.0),
        })
        .collect();
    // Lift our DashMap state into a HashMap for the pure evaluator,
    // then write changes back. Acceptable for the small N (top-50).
    let mut snapshot: std::collections::HashMap<String, SymbolState> = store
        .states
        .iter()
        .map(|e| (e.key().clone(), *e.value()))
        .collect();
    let now = Utc::now();
    let events = evaluate(&inputs, &mut snapshot, now);
    for (k, v) in snapshot {
        store.states.insert(k, v);
    }
    let n = events.len();
    for ev in events {
        let key = format!("{}|{}", ev.symbol, ev.fired_at.timestamp());
        store.emitted.insert(key, ev.clone());
        let _ = store.tx.send(ev);
    }
    store.evict_if_full();
    Ok(n)
}

pub fn spawn_refresher(store: VelocityStore, pool: PgPool) {
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(45)).await;
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(REFRESH_SECS));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            interval.tick().await;
            if let Err(e) = refresh_once(&pool, &store).await {
                tracing::warn!(?e, "sentiment_velocity refresh failed");
            }
        }
    });
}

pub fn global(pool: PgPool) -> VelocityStore {
    static STORE: once_cell::sync::OnceCell<VelocityStore> = once_cell::sync::OnceCell::new();
    STORE
        .get_or_init(|| {
            let s = VelocityStore::new();
            spawn_refresher(s.clone(), pool);
            s
        })
        .clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn row(sym: &str, cur: i64, prev: i64) -> RankedInput {
        RankedInput {
            symbol: sym.into(),
            mention_count: cur,
            prev_count: prev,
            avg_sentiment: 0.0,
            sentiment_delta: 0.0,
        }
    }

    #[test]
    fn no_fire_on_first_qualifying_round() {
        let mut states: HashMap<String, SymbolState> = HashMap::new();
        // 30 cur / 5 prev = 6× ratio, well above 3× threshold; cur >= 20.
        let rows = vec![row("AAA", 30, 5)];
        let evs = evaluate(&rows, &mut states, Utc::now());
        assert!(evs.is_empty(), "first qualifying round must arm, not fire");
        assert_eq!(states.get("AAA").unwrap().consecutive, 1);
    }

    #[test]
    fn fires_on_two_consecutive_qualifying_rounds() {
        let mut states: HashMap<String, SymbolState> = HashMap::new();
        let rows = vec![row("AAA", 30, 5)];
        let _ = evaluate(&rows, &mut states, Utc::now());
        let rows2 = vec![row("AAA", 50, 10)]; // 5×, still qualifies
        let evs = evaluate(&rows2, &mut states, Utc::now());
        assert_eq!(evs.len(), 1);
        assert_eq!(evs[0].symbol, "AAA");
        assert_eq!(evs[0].consecutive_hours, 2);
        assert!(evs[0].ratio >= VELOCITY_RATIO);
    }

    #[test]
    fn quiet_hour_resets_consecutive_counter() {
        let mut states: HashMap<String, SymbolState> = HashMap::new();
        let _ = evaluate(&[row("AAA", 30, 5)], &mut states, Utc::now());
        // Next round AAA doesn't qualify (ratio 1.0) — counter resets.
        let _ = evaluate(&[row("AAA", 30, 30)], &mut states, Utc::now());
        assert_eq!(states.get("AAA").unwrap().consecutive, 0);
        // Two more "qualifies" rounds needed to fire.
        let evs = evaluate(&[row("AAA", 100, 20)], &mut states, Utc::now());
        assert!(evs.is_empty(), "counter must rebuild from 0");
    }

    #[test]
    fn falling_out_of_ranked_window_resets_counter() {
        let mut states: HashMap<String, SymbolState> = HashMap::new();
        // Build AAA up to 1 consecutive.
        let _ = evaluate(&[row("AAA", 30, 5)], &mut states, Utc::now());
        assert_eq!(states.get("AAA").unwrap().consecutive, 1);
        // Next round AAA is absent (didn't make top-N). Counter must reset.
        let _ = evaluate(&[row("BBB", 50, 10)], &mut states, Utc::now());
        assert_eq!(states.get("AAA").unwrap().consecutive, 0);
    }

    #[test]
    fn cooldown_blocks_re_fire_within_window() {
        let mut states: HashMap<String, SymbolState> = HashMap::new();
        let t0 = Utc::now();
        let _ = evaluate(&[row("AAA", 30, 5)], &mut states, t0);
        let evs = evaluate(&[row("AAA", 60, 10)], &mut states, t0);
        assert_eq!(evs.len(), 1);
        // Immediately try to re-fire — within cooldown.
        let evs2 = evaluate(&[row("AAA", 200, 30)], &mut states, t0 + Duration::hours(1));
        assert!(evs2.is_empty(), "cooldown must suppress in-window re-fire");
    }

    #[test]
    fn fire_resumes_after_cooldown_when_acceleration_persists() {
        let mut states: HashMap<String, SymbolState> = HashMap::new();
        let t0 = Utc::now();
        let _ = evaluate(&[row("AAA", 30, 5)], &mut states, t0);
        let _ = evaluate(&[row("AAA", 60, 10)], &mut states, t0); // fires
                                                                  // Step well past cooldown with sustained acceleration. The next
                                                                  // qualifying call should fire again.
        let evs = evaluate(
            &[row("AAA", 90, 15)],
            &mut states,
            t0 + Duration::hours(COOLDOWN_HOURS + 1),
        );
        assert_eq!(evs.len(), 1, "post-cooldown sustained accel re-fires");
    }

    #[test]
    fn small_volume_does_not_trigger() {
        let mut states: HashMap<String, SymbolState> = HashMap::new();
        // Ratio 4×, but only 8 mentions — below MIN_MENTIONS.
        let _ = evaluate(&[row("TINY", 8, 2)], &mut states, Utc::now());
        let _ = evaluate(&[row("TINY", 8, 2)], &mut states, Utc::now());
        assert_eq!(states.get("TINY").map(|s| s.consecutive).unwrap_or(0), 0);
    }

    #[test]
    fn zero_prev_treats_as_infinite_when_cur_meaningful() {
        let mut states: HashMap<String, SymbolState> = HashMap::new();
        let _ = evaluate(&[row("AAA", 50, 0)], &mut states, Utc::now());
        let evs = evaluate(&[row("AAA", 60, 0)], &mut states, Utc::now());
        assert_eq!(evs.len(), 1, "0→meaningful counts as acceleration");
    }
}
