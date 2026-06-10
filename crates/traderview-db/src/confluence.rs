//! Multi-signal confluence dashboard.
//!
//! Independent scanners are noise; **intersections** are signal. This
//! module subscribes to every scanner's broadcast channel, normalises
//! each event into a `(symbol, source, observed_at)` tuple, and
//! computes a per-symbol confluence score:
//!
//! ```text
//! score = Σ_{events in 24h} source_weight · recency_weight
//!       + diversity_bonus(distinct sources hit)
//! ```
//!
//! where:
//!   * `source_weight` is fixed per `Source` variant — insider buys
//!     score highest (academic edge), insider sells score *negative*
//!     (bearish), market-wide regimes score lower per symbol.
//!   * `recency_weight = 0.5^(age_hours / HALF_LIFE_HOURS)` (4h
//!     half-life) so a 2h-old hit weighs ~0.71 and a 24h-old hit ~0.02.
//!   * `diversity_bonus` adds +2.0 per distinct source beyond the
//!     second one — three independent scanners agreeing matters more
//!     than three hits from the same scanner.
//!
//! The output is a single ranked table — the symbols where the most
//! independent edges agree, weighted by recency. Top of the list is
//! the highest-conviction long candidate at this moment.
//!
//! Subscribers are spawned once per `Source` channel in
//! `spawn_consumers`. Each consumer reconnects on broadcast `Lagged`
//! and exits cleanly on `Closed` (caller drops the store on shutdown).
//!
//! Event log is bounded per symbol (`MAX_EVENTS_PER_SYMBOL`) with
//! oldest-first eviction; symbols with no events in the trailing
//! 24h are pruned on every `ranked()` call.

use chrono::{DateTime, Duration, Utc};
use dashmap::DashMap;
use serde::Serialize;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration as StdDuration;
use tokio::sync::broadcast;

const HALF_LIFE_HOURS: f64 = 4.0;
const WINDOW_HOURS: i64 = 24;
const DIVERSITY_BONUS_PER_SOURCE: f64 = 2.0;
const MAX_EVENTS_PER_SYMBOL: usize = 200;
const STORE_CAP_SYMBOLS: usize = 4_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    AfterHours,
    Catalyst,
    CatalystCorrelation,
    GammaSqueeze,
    Halt,
    InsiderBuy,
    InsiderSell,
    RvolAccel,
    SentimentVelocity,
    SqueezeDetector,
    Uoa,
}

impl Source {
    pub fn weight(self) -> f64 {
        match self {
            // Insider buys: best free-data edge per Lakonishok & Lee 2001.
            Source::InsiderBuy => 3.0,
            // News + price reaction confirmed within 60s — the catalyst
            // already proved itself as actionable in this stock.
            Source::CatalystCorrelation => 2.5,
            // Negative dealer gamma + spot near pin — mechanical edge.
            Source::GammaSqueeze => 2.0,
            // Sustained social acceleration across two consecutive checks.
            Source::SentimentVelocity => 2.0,
            // Smart-money options positioning.
            Source::Uoa => 2.0,
            // Institutional volume accumulation pattern.
            Source::RvolAccel => 1.5,
            // 24/7 squeeze detector (catalyst + price + volume).
            Source::SqueezeDetector => 1.5,
            // Wall-clock-session move; informative but less rare.
            Source::AfterHours => 1.5,
            // Trading halt — informative but binary, doesn't pick direction.
            Source::Halt => 1.5,
            // Raw catalyst headline without confirmed price reaction.
            Source::Catalyst => 1.0,
            // Insider sells: bearish, but with much lower predictive power
            // than buys (insiders sell for diversification / tax reasons).
            // Net-negative contribution dampens scores on names with heavy
            // insider distribution.
            Source::InsiderSell => -1.5,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Source::AfterHours => "after_hours",
            Source::Catalyst => "catalyst",
            Source::CatalystCorrelation => "catalyst_correlation",
            Source::GammaSqueeze => "gamma_squeeze",
            Source::Halt => "halt",
            Source::InsiderBuy => "insider_buy",
            Source::InsiderSell => "insider_sell",
            Source::RvolAccel => "rvol_accel",
            Source::SentimentVelocity => "sentiment_velocity",
            Source::SqueezeDetector => "squeeze_detector",
            Source::Uoa => "uoa",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ConfluenceEvent {
    pub symbol: String,
    pub source: Source,
    pub observed_at: DateTime<Utc>,
    /// One-line human-readable summary for the UI's tooltip / drill-down.
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConfluenceRow {
    pub symbol: String,
    pub score: f64,
    pub event_count: usize,
    pub distinct_sources: usize,
    pub events: Vec<ConfluenceEvent>,
    /// Sources currently contributing to the score (deduped). Order is
    /// the order they're first observed in the trailing window.
    pub sources_hit: Vec<Source>,
}

#[derive(Clone)]
pub struct ConfluenceStore {
    by_symbol: Arc<DashMap<String, Vec<ConfluenceEvent>>>,
    tx: broadcast::Sender<ConfluenceEvent>,
}

impl ConfluenceStore {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(512);
        Self {
            by_symbol: Arc::new(DashMap::new()),
            tx,
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<ConfluenceEvent> {
        self.tx.subscribe()
    }

    /// Record a single event, prune trailing-window-stale entries for
    /// this symbol, and broadcast to subscribers.
    pub fn observe(&self, ev: ConfluenceEvent) {
        let now = ev.observed_at;
        let cutoff = now - Duration::hours(WINDOW_HOURS);
        let sym = ev.symbol.clone();
        {
            let mut entry = self.by_symbol.entry(sym.clone()).or_default();
            entry.push(ev.clone());
            // Drop anything past the trailing window.
            entry.retain(|e| e.observed_at >= cutoff);
            // Cap per-symbol log so a runaway scanner can't OOM us.
            let len = entry.len();
            if len > MAX_EVENTS_PER_SYMBOL {
                let drop = len - MAX_EVENTS_PER_SYMBOL;
                entry.drain(..drop);
            }
        }
        let _ = self.tx.send(ev);
        self.evict_if_full();
    }

    fn evict_if_full(&self) {
        if self.by_symbol.len() <= STORE_CAP_SYMBOLS {
            return;
        }
        // Drop the symbols whose most-recent event is oldest.
        let drop_n = self.by_symbol.len() / 4;
        let mut by_age: Vec<(String, DateTime<Utc>)> = self
            .by_symbol
            .iter()
            .map(|e| {
                let newest = e
                    .value()
                    .iter()
                    .map(|ev| ev.observed_at)
                    .max()
                    .unwrap_or_else(Utc::now);
                (e.key().clone(), newest)
            })
            .collect();
        by_age.sort_by_key(|(_, t)| *t);
        for (key, _) in by_age.into_iter().take(drop_n) {
            self.by_symbol.remove(&key);
        }
    }

    /// Compute the top-N confluence rows. `min_distinct_sources` filters
    /// out symbols that only hit a single scanner — the whole point of
    /// confluence is multi-source agreement, so the default of 2 is the
    /// useful floor.
    pub fn ranked(
        &self,
        now: DateTime<Utc>,
        limit: usize,
        min_distinct_sources: usize,
    ) -> Vec<ConfluenceRow> {
        let cutoff = now - Duration::hours(WINDOW_HOURS);
        let mut rows: Vec<ConfluenceRow> = self
            .by_symbol
            .iter()
            .filter_map(|entry| {
                let events: Vec<ConfluenceEvent> = entry
                    .value()
                    .iter()
                    .filter(|e| e.observed_at >= cutoff)
                    .cloned()
                    .collect();
                if events.is_empty() {
                    return None;
                }
                let row = compute_row(entry.key(), &events, now);
                if row.distinct_sources < min_distinct_sources {
                    return None;
                }
                Some(row)
            })
            .collect();
        rows.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        rows.truncate(limit);
        rows
    }

    /// Latest events for one symbol (debug / drill-down).
    pub fn events_for(&self, symbol: &str, limit: usize) -> Vec<ConfluenceEvent> {
        let sym_upper = symbol.to_ascii_uppercase();
        match self.by_symbol.get(&sym_upper) {
            Some(entry) => {
                let mut out: Vec<ConfluenceEvent> = entry.value().clone();
                out.sort_by_key(|e| std::cmp::Reverse(e.observed_at));
                out.truncate(limit);
                out
            }
            None => Vec::new(),
        }
    }
}

impl Default for ConfluenceStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Pure scoring: given a symbol's event list and the current wall-clock
/// time, compute the confluence row.
pub fn compute_row(symbol: &str, events: &[ConfluenceEvent], now: DateTime<Utc>) -> ConfluenceRow {
    let mut score = 0.0_f64;
    let mut sources_hit: Vec<Source> = Vec::new();
    let mut seen: HashSet<Source> = HashSet::new();
    let mut sorted = events.to_vec();
    sorted.sort_by_key(|e| std::cmp::Reverse(e.observed_at));
    for e in &sorted {
        score += e.source.weight() * recency_weight(now, e.observed_at);
        if seen.insert(e.source) {
            sources_hit.push(e.source);
        }
    }
    // Diversity bonus: starts contributing from the 3rd distinct source on.
    let bonus_sources = sources_hit.len().saturating_sub(2);
    score += bonus_sources as f64 * DIVERSITY_BONUS_PER_SOURCE;
    ConfluenceRow {
        symbol: symbol.to_ascii_uppercase(),
        score,
        event_count: events.len(),
        distinct_sources: sources_hit.len(),
        events: sorted,
        sources_hit,
    }
}

/// Exponential recency: half-life HALF_LIFE_HOURS hours, floored at 0
/// when the event is past `now`.
pub fn recency_weight(now: DateTime<Utc>, observed: DateTime<Utc>) -> f64 {
    let age_hours = (now - observed).num_seconds() as f64 / 3600.0;
    if age_hours <= 0.0 {
        1.0
    } else {
        0.5_f64.powf(age_hours / HALF_LIFE_HOURS)
    }
}

// ─── Background consumer ───────────────────────────────────────────────────

/// Spawn one tokio task per upstream broadcast channel. Each task
/// translates upstream events into `ConfluenceEvent` and folds them
/// into the store. Reconnects on `Lagged`, exits on `Closed`.
pub fn spawn_consumers(store: ConfluenceStore) {
    spawn_after_hours(store.clone());
    spawn_catalysts(store.clone());
    spawn_catalyst_correlations(store.clone());
    spawn_gamma_squeeze(store.clone());
    spawn_halts(store.clone());
    spawn_insider_stream(store.clone());
    spawn_rvol_accel(store.clone());
    spawn_sentiment_velocity_polling(store.clone());
    spawn_squeeze_detector(store.clone());
    spawn_uoa(store);
}

fn spawn_after_hours(store: ConfluenceStore) {
    tokio::spawn(async move {
        let src = crate::after_hours::global();
        loop {
            let mut rx = src.subscribe();
            loop {
                match rx.recv().await {
                    Ok(s) => {
                        if matches!(
                            s.session,
                            crate::after_hours::Session::Pre | crate::after_hours::Session::Post
                        ) {
                            store.observe(ConfluenceEvent {
                                symbol: s.symbol.clone(),
                                source: Source::AfterHours,
                                observed_at: s.last_trade_at,
                                detail: format!(
                                    "{} {:+.2}% vs RTH close ({} vol)",
                                    s.session.as_str(),
                                    s.change_pct,
                                    fmt_n(s.ah_volume)
                                ),
                            });
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            tokio::time::sleep(StdDuration::from_secs(2)).await;
        }
    });
}

fn spawn_catalysts(store: ConfluenceStore) {
    tokio::spawn(async move {
        let src = crate::catalysts::global();
        loop {
            let mut rx = src.subscribe();
            loop {
                match rx.recv().await {
                    Ok(c) => {
                        for ticker in &c.tickers {
                            store.observe(ConfluenceEvent {
                                symbol: ticker.to_ascii_uppercase(),
                                source: Source::Catalyst,
                                observed_at: c.published_at,
                                detail: format!(
                                    "{} ({}): {}",
                                    c.source,
                                    c.form_type.as_deref().unwrap_or("-"),
                                    truncate(&c.title, 80)
                                ),
                            });
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            tokio::time::sleep(StdDuration::from_secs(2)).await;
        }
    });
}

fn spawn_catalyst_correlations(store: ConfluenceStore) {
    tokio::spawn(async move {
        let src = crate::catalyst_correlator::global();
        loop {
            let mut rx = src.subscribe();
            loop {
                match rx.recv().await {
                    Ok(c) => {
                        store.observe(ConfluenceEvent {
                            symbol: c.symbol.clone(),
                            source: Source::CatalystCorrelation,
                            observed_at: c.observed_at,
                            detail: format!(
                                "{:?} catalyst → {:+.2}% in {}ms — {}",
                                c.sentiment,
                                c.signed_move_pct,
                                c.cross_latency_ms.unwrap_or(0),
                                truncate(&c.title, 60)
                            ),
                        });
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            tokio::time::sleep(StdDuration::from_secs(2)).await;
        }
    });
}

fn spawn_gamma_squeeze(store: ConfluenceStore) {
    tokio::spawn(async move {
        let src = crate::gamma_squeeze::global();
        loop {
            let mut rx = src.subscribe();
            loop {
                match rx.recv().await {
                    Ok(c) => {
                        store.observe(ConfluenceEvent {
                            symbol: c.symbol.clone(),
                            source: Source::GammaSqueeze,
                            observed_at: c.observed_at,
                            detail: format!(
                                "spot {:.2} pin {:.2} ({:+.2}%) GEX ${:.1}M",
                                c.spot,
                                c.largest_negative_strike.unwrap_or(0.0),
                                c.pin_distance_pct.unwrap_or(0.0),
                                c.total_gex / 1_000_000.0
                            ),
                        });
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            tokio::time::sleep(StdDuration::from_secs(2)).await;
        }
    });
}

fn spawn_halts(store: ConfluenceStore) {
    tokio::spawn(async move {
        let src = crate::halts::global();
        loop {
            let mut rx = src.subscribe();
            loop {
                match rx.recv().await {
                    Ok(h) => {
                        store.observe(ConfluenceEvent {
                            symbol: h.symbol.clone(),
                            source: Source::Halt,
                            observed_at: h.fetched_at,
                            detail: format!("{} {}", h.reason_code, h.reason_label),
                        });
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            tokio::time::sleep(StdDuration::from_secs(2)).await;
        }
    });
}

fn spawn_insider_stream(store: ConfluenceStore) {
    tokio::spawn(async move {
        let src = crate::insider_stream::global();
        loop {
            let mut rx = src.subscribe();
            loop {
                match rx.recv().await {
                    Ok(ev) => {
                        let source = match ev.kind {
                            crate::insider_stream::TxKind::Buy => Some(Source::InsiderBuy),
                            crate::insider_stream::TxKind::Sell => Some(Source::InsiderSell),
                            _ => None,
                        };
                        if let Some(source) = source {
                            store.observe(ConfluenceEvent {
                                symbol: ev.symbol.clone(),
                                source,
                                observed_at: ev.observed_at,
                                detail: format!(
                                    "{} {} ({}) ${}",
                                    ev.kind.as_str(),
                                    ev.insider_name,
                                    ev.officer_title.as_deref().unwrap_or("-"),
                                    fmt_dollars(ev.dollar_value)
                                ),
                            });
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            tokio::time::sleep(StdDuration::from_secs(2)).await;
        }
    });
}

fn spawn_rvol_accel(store: ConfluenceStore) {
    tokio::spawn(async move {
        let src = crate::rvol_accel::global();
        loop {
            let mut rx = src.subscribe();
            loop {
                match rx.recv().await {
                    Ok(ev) => {
                        store.observe(ConfluenceEvent {
                            symbol: ev.symbol.clone(),
                            source: Source::RvolAccel,
                            observed_at: ev.observed_at,
                            detail: format!(
                                "{:.2}× baseline · latest {} vs base {}",
                                ev.multiple,
                                fmt_n(ev.latest_volume),
                                fmt_n(ev.baseline_volume)
                            ),
                        });
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            tokio::time::sleep(StdDuration::from_secs(2)).await;
        }
    });
}

/// Sentiment velocity is per-symbol but its `global()` constructor
/// needs a PgPool — we don't have one here. Instead poll the store's
/// `latest()` snapshot every 60s for fresh events. The first time we
/// see an event we record it; subsequent polls dedup on `fired_at`.
fn spawn_sentiment_velocity_polling(store: ConfluenceStore) {
    use std::collections::HashSet;
    tokio::spawn(async move {
        let mut seen: HashSet<String> = HashSet::new();
        let mut interval = tokio::time::interval(StdDuration::from_secs(60));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            interval.tick().await;
            // `try_init` returns None if no caller has constructed it
            // yet (route hasn't been hit + boot hook hasn't fired with
            // a pool). Skip silently in that case.
            let Some(svc_store) = crate::sentiment_velocity::try_global() else {
                continue;
            };
            for ev in svc_store.latest(50) {
                let key = format!("{}|{}", ev.symbol, ev.fired_at.timestamp());
                if !seen.insert(key) {
                    continue;
                }
                store.observe(ConfluenceEvent {
                    symbol: ev.symbol.clone(),
                    source: Source::SentimentVelocity,
                    observed_at: ev.fired_at,
                    detail: format!(
                        "{:.1}× mentions · {} cur / {} prior",
                        ev.ratio, ev.mention_count, ev.prev_count
                    ),
                });
            }
        }
    });
}

fn spawn_squeeze_detector(store: ConfluenceStore) {
    tokio::spawn(async move {
        let src = crate::squeeze_detector::global();
        loop {
            let mut rx = src.subscribe();
            loop {
                match rx.recv().await {
                    Ok(ev) => {
                        store.observe(ConfluenceEvent {
                            symbol: ev.symbol.clone(),
                            source: Source::SqueezeDetector,
                            observed_at: ev.fired_at,
                            detail: format!(
                                "{:+.2}% · burst {:.1}× baseline",
                                ev.pct_change, ev.burst_ratio
                            ),
                        });
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            tokio::time::sleep(StdDuration::from_secs(2)).await;
        }
    });
}

fn spawn_uoa(store: ConfluenceStore) {
    tokio::spawn(async move {
        let src = crate::uoa_stream::global();
        loop {
            let mut rx = src.subscribe();
            loop {
                match rx.recv().await {
                    Ok(ev) => {
                        store.observe(ConfluenceEvent {
                            symbol: ev.symbol.clone(),
                            source: Source::Uoa,
                            observed_at: ev.observed_at,
                            detail: format!(
                                "{} {:.0} @ {}exp ${} prem · vol/oi {:.1}",
                                ev.option_type,
                                ev.strike,
                                ev.expiry,
                                fmt_dollars(ev.premium_paid),
                                ev.vol_oi_ratio
                            ),
                        });
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            tokio::time::sleep(StdDuration::from_secs(2)).await;
        }
    });
}

pub fn global() -> ConfluenceStore {
    static STORE: once_cell::sync::OnceCell<ConfluenceStore> = once_cell::sync::OnceCell::new();
    STORE
        .get_or_init(|| {
            let s = ConfluenceStore::new();
            spawn_consumers(s.clone());
            s
        })
        .clone()
}

// ─── Formatting helpers (private — only used inside `detail` strings) ──────

fn fmt_n(n: f64) -> String {
    if n >= 1_000_000.0 {
        format!("{:.2}M", n / 1_000_000.0)
    } else if n >= 1_000.0 {
        format!("{:.1}K", n / 1_000.0)
    } else {
        format!("{n:.0}")
    }
}

fn fmt_dollars(n: f64) -> String {
    let abs = n.abs();
    if abs >= 1_000_000.0 {
        format!("{:.2}M", abs / 1_000_000.0)
    } else if abs >= 1_000.0 {
        format!("{:.0}K", abs / 1_000.0)
    } else {
        format!("{abs:.0}")
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let mut out: String = s.chars().take(max - 1).collect();
    out.push('…');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ev(sym: &str, source: Source, age_hours: f64, now: DateTime<Utc>) -> ConfluenceEvent {
        ConfluenceEvent {
            symbol: sym.into(),
            source,
            observed_at: now - chrono::Duration::seconds((age_hours * 3600.0) as i64),
            detail: "test".into(),
        }
    }

    #[test]
    fn source_weights_match_documented_ordering() {
        // InsiderBuy > CatalystCorrelation > GammaSqueeze
        assert!(Source::InsiderBuy.weight() > Source::CatalystCorrelation.weight());
        assert!(Source::CatalystCorrelation.weight() > Source::GammaSqueeze.weight());
        // InsiderSell is the only negative source.
        assert!(Source::InsiderSell.weight() < 0.0);
        for s in [
            Source::AfterHours,
            Source::Catalyst,
            Source::CatalystCorrelation,
            Source::GammaSqueeze,
            Source::Halt,
            Source::InsiderBuy,
            Source::RvolAccel,
            Source::SentimentVelocity,
            Source::SqueezeDetector,
            Source::Uoa,
        ] {
            assert!(s.weight() > 0.0, "{:?} should be non-negative", s);
        }
    }

    #[test]
    fn recency_weight_half_lives() {
        let now = Utc::now();
        // 0 hours ago → weight = 1.0
        let w0 = recency_weight(now, now);
        assert!((w0 - 1.0).abs() < 1e-9);
        // HALF_LIFE_HOURS ago → 0.5
        let half = now - chrono::Duration::seconds((HALF_LIFE_HOURS * 3600.0) as i64);
        let wh = recency_weight(now, half);
        assert!((wh - 0.5).abs() < 1e-9);
        // Two half-lives ago → 0.25
        let two = now - chrono::Duration::seconds((2.0 * HALF_LIFE_HOURS * 3600.0) as i64);
        assert!((recency_weight(now, two) - 0.25).abs() < 1e-9);
        // Future timestamps → 1.0 (clamped, no negative-age weirdness)
        let future = now + chrono::Duration::hours(1);
        assert_eq!(recency_weight(now, future), 1.0);
    }

    #[test]
    fn compute_row_aggregates_distinct_sources() {
        let now = Utc::now();
        let events = vec![
            ev("AAA", Source::InsiderBuy, 0.0, now),
            ev("AAA", Source::Uoa, 1.0, now),
            ev("AAA", Source::Uoa, 2.0, now), // duplicate source — not extra distinct
        ];
        let row = compute_row("AAA", &events, now);
        assert_eq!(row.symbol, "AAA");
        assert_eq!(row.event_count, 3);
        assert_eq!(row.distinct_sources, 2);
        assert_eq!(row.sources_hit.len(), 2);
        // No diversity bonus yet (need ≥3 distinct sources).
        // Score: 3.0·1.0 + 2.0·recency(1h) + 2.0·recency(2h)
        let expected = 3.0
            + 2.0 * recency_weight(now, now - chrono::Duration::hours(1))
            + 2.0 * recency_weight(now, now - chrono::Duration::hours(2));
        assert!((row.score - expected).abs() < 1e-6);
    }

    #[test]
    fn diversity_bonus_only_kicks_in_from_third_source() {
        let now = Utc::now();
        // Two distinct sources → no bonus.
        let two = vec![
            ev("BBB", Source::InsiderBuy, 0.0, now),
            ev("BBB", Source::Uoa, 0.0, now),
        ];
        let row2 = compute_row("BBB", &two, now);
        let expected2 = Source::InsiderBuy.weight() + Source::Uoa.weight();
        assert!((row2.score - expected2).abs() < 1e-6);

        // Three distinct sources → +2.0 bonus.
        let three = vec![
            ev("CCC", Source::InsiderBuy, 0.0, now),
            ev("CCC", Source::Uoa, 0.0, now),
            ev("CCC", Source::GammaSqueeze, 0.0, now),
        ];
        let row3 = compute_row("CCC", &three, now);
        let expected3 = Source::InsiderBuy.weight()
            + Source::Uoa.weight()
            + Source::GammaSqueeze.weight()
            + DIVERSITY_BONUS_PER_SOURCE;
        assert!((row3.score - expected3).abs() < 1e-6);

        // Four distinct sources → +4.0 bonus.
        let four = vec![
            ev("DDD", Source::InsiderBuy, 0.0, now),
            ev("DDD", Source::Uoa, 0.0, now),
            ev("DDD", Source::GammaSqueeze, 0.0, now),
            ev("DDD", Source::RvolAccel, 0.0, now),
        ];
        let row4 = compute_row("DDD", &four, now);
        let expected4 = Source::InsiderBuy.weight()
            + Source::Uoa.weight()
            + Source::GammaSqueeze.weight()
            + Source::RvolAccel.weight()
            + 2.0 * DIVERSITY_BONUS_PER_SOURCE;
        assert!((row4.score - expected4).abs() < 1e-6);
    }

    #[test]
    fn insider_sell_subtracts_from_score() {
        let now = Utc::now();
        let with_sell = vec![
            ev("EEE", Source::InsiderBuy, 0.0, now),  // +3.0
            ev("EEE", Source::InsiderSell, 0.0, now), // -1.5
        ];
        let row = compute_row("EEE", &with_sell, now);
        let expected = Source::InsiderBuy.weight() + Source::InsiderSell.weight();
        assert!((row.score - expected).abs() < 1e-6);
        // Two distinct sources (Buy AND Sell on the same name) — that's
        // information about insider activity volume.
        assert_eq!(row.distinct_sources, 2);
    }

    #[test]
    fn observe_prunes_events_outside_window() {
        let store = ConfluenceStore::new();
        let now = Utc::now();
        // Old event from 30h ago — should be pruned on the next insert.
        store.observe(ev("AAA", Source::Catalyst, 30.0, now));
        store.observe(ev("AAA", Source::InsiderBuy, 1.0, now));
        let entries = store.events_for("AAA", 10);
        assert_eq!(entries.len(), 1, "30h-old event must be pruned");
        assert!(matches!(entries[0].source, Source::InsiderBuy));
    }

    #[test]
    fn ranked_filters_by_min_distinct_sources() {
        let store = ConfluenceStore::new();
        let now = Utc::now();
        // AAA: only one source.
        store.observe(ev("AAA", Source::Catalyst, 0.5, now));
        // BBB: three distinct sources.
        store.observe(ev("BBB", Source::InsiderBuy, 0.5, now));
        store.observe(ev("BBB", Source::Uoa, 0.5, now));
        store.observe(ev("BBB", Source::GammaSqueeze, 0.5, now));
        // CCC: two distinct sources.
        store.observe(ev("CCC", Source::Halt, 0.5, now));
        store.observe(ev("CCC", Source::RvolAccel, 0.5, now));
        let r2 = store.ranked(now, 10, 2);
        let syms: Vec<&str> = r2.iter().map(|r| r.symbol.as_str()).collect();
        assert!(syms.contains(&"BBB"));
        assert!(syms.contains(&"CCC"));
        assert!(!syms.contains(&"AAA"));
        let r3 = store.ranked(now, 10, 3);
        let syms3: Vec<&str> = r3.iter().map(|r| r.symbol.as_str()).collect();
        assert_eq!(syms3, vec!["BBB"], "min=3 keeps only the 3-source name");
    }

    #[test]
    fn ranked_orders_highest_score_first() {
        let store = ConfluenceStore::new();
        let now = Utc::now();
        // Hi: insider buy (3.0) + uoa (2.0) + gamma_squeeze (2.0) + diversity 2.0 = 9.0
        store.observe(ev("HI", Source::InsiderBuy, 0.0, now));
        store.observe(ev("HI", Source::Uoa, 0.0, now));
        store.observe(ev("HI", Source::GammaSqueeze, 0.0, now));
        // Lo: catalyst (1.0) + halt (1.5) = 2.5
        store.observe(ev("LO", Source::Catalyst, 0.0, now));
        store.observe(ev("LO", Source::Halt, 0.0, now));
        let r = store.ranked(now, 10, 2);
        assert_eq!(r[0].symbol, "HI");
        assert_eq!(r[1].symbol, "LO");
        assert!(r[0].score > r[1].score);
    }

    #[test]
    fn old_events_decay_below_fresh_single_hit() {
        let store = ConfluenceStore::new();
        let now = Utc::now();
        // Two distinct sources but both 12h old — recency_weight = 0.5^3 = 0.125.
        // Score = (3.0 + 2.0) · 0.125 = 0.625
        store.observe(ev("OLD", Source::InsiderBuy, 12.0, now));
        store.observe(ev("OLD", Source::Uoa, 12.0, now));
        // One fresh hit (0h) + one stale (12h) on FRESH — still beats OLD because
        // the fresh one preserves full weight.
        store.observe(ev("FRESH", Source::Catalyst, 0.0, now));
        store.observe(ev("FRESH", Source::Halt, 12.0, now));
        let rows = store.ranked(now, 10, 2);
        let by_sym: std::collections::HashMap<_, _> =
            rows.iter().map(|r| (r.symbol.as_str(), r.score)).collect();
        let old = by_sym.get("OLD").copied().unwrap_or(0.0);
        let fresh = by_sym.get("FRESH").copied().unwrap_or(0.0);
        assert!(fresh > old, "fresh 1.0 + decayed 1.5 > decayed (3.0+2.0)");
    }

    #[test]
    fn observe_caps_per_symbol_event_log() {
        let store = ConfluenceStore::new();
        let now = Utc::now();
        // Push way more than MAX_EVENTS_PER_SYMBOL.
        for i in 0..(MAX_EVENTS_PER_SYMBOL + 50) {
            store.observe(ConfluenceEvent {
                symbol: "MANY".into(),
                source: Source::Catalyst,
                observed_at: now - chrono::Duration::seconds(i as i64),
                detail: format!("e{i}"),
            });
        }
        let entry = store.by_symbol.get("MANY").unwrap();
        assert!(entry.len() <= MAX_EVENTS_PER_SYMBOL);
    }
}
