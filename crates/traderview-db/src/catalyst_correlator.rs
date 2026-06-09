//! Catalyst → price-action correlator.
//!
//! Subscribes to two firehoses simultaneously:
//!
//!   1. `catalysts::CatalystStore::subscribe()` — every fresh SEC filing
//!      or PR-wire headline as extracted by the catalyst aggregator,
//!      with `tickers: Vec<String>` already parsed off the title.
//!   2. `live_ticks::LiveTickStore::tape_subscribe()` — every parsed
//!      trade off the equities WebSocket feeds.
//!
//! The correlator implements a simple but high-leverage rule:
//!
//!   * For each catalyst, snapshot the prevailing price for every
//!     extracted ticker (from `LiveTickStore.get(symbol).last`). That
//!     becomes the **baseline** for a 60-second watch window.
//!   * Each incoming trade in that window updates `max_move_pct` for
//!     the corresponding pending row.
//!   * The first trade whose `|move_pct| ≥ THRESHOLD_PCT` (default 2%)
//!     locks in the **threshold-crossed** state, captures `latency_ms`
//!     (catalyst→cross), records peak / trough across the window, and
//!     emits a `Correlation` event on the broadcast channel.
//!   * The catalyst headline is run through a small sentiment lexicon
//!     and tagged Bullish / Bearish / Neutral so the UI can colour-code
//!     the row without an LLM.
//!
//! A correlation never replaces a stronger signal — once a row is
//! locked in, the watch window stays open until expiry so the peak /
//! trough fields keep updating to the largest observed move.
//!
//! Why this matters: a news headline by itself is noise. A headline
//! followed by a 3% spike inside 30 seconds is signal. The correlator
//! separates the two automatically.

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::Serialize;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;

use crate::catalysts::{Catalyst, CatalystKind, CatalystStore};
use crate::live_ticks::{LiveTickStore, Trade};

/// Move size (percent of baseline) at which a pending row becomes a
/// confirmed correlation. Tuned for small/mid-caps and earnings prints
/// where 2% in <60s reliably separates real reactions from noise.
const THRESHOLD_PCT: f64 = 2.0;
/// How long after a catalyst we'll keep watching the tape. Most real
/// reactions land in the first 30s; 60s buys headroom for slower-priced
/// names without flooding the pending set.
const WATCH_SECS: i64 = 60;
/// Max emitted correlations kept in memory. Same cap-and-evict-oldest-N
/// pattern as halts.
const EMITTED_CAP: usize = 4_000;

/// Coarse polarity of a catalyst headline as scored by [`score_sentiment`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Sentiment {
    Bullish,
    Bearish,
    Neutral,
}

impl Sentiment {
    pub fn as_str(self) -> &'static str {
        match self {
            Sentiment::Bullish => "bullish",
            Sentiment::Bearish => "bearish",
            Sentiment::Neutral => "neutral",
        }
    }
}

/// One pending row — one (catalyst, symbol) pair being watched.
#[derive(Debug, Clone)]
struct Pending {
    catalyst_id: String,
    symbol: String,
    title: String,
    source: String,
    kind: CatalystKind,
    sentiment: Sentiment,
    catalyst_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    baseline: f64,
    peak: f64,
    trough: f64,
    max_abs_move_pct: f64,
    threshold_crossed: bool,
    cross_latency_ms: Option<i64>,
}

/// Output row — a confirmed catalyst→price-action correlation.
#[derive(Debug, Clone, Serialize)]
pub struct Correlation {
    pub catalyst_id: String,
    pub symbol: String,
    pub title: String,
    pub source: String,
    pub kind: CatalystKind,
    pub sentiment: Sentiment,
    pub catalyst_at: DateTime<Utc>,
    pub observed_at: DateTime<Utc>,
    pub baseline: f64,
    pub peak: f64,
    pub trough: f64,
    pub max_abs_move_pct: f64,
    /// Signed move (peak vs baseline if positive, trough vs baseline if
    /// negative) — the canonical "how did this catalyst move the stock"
    /// number for ranking / sorting.
    pub signed_move_pct: f64,
    /// Milliseconds from catalyst publish to first threshold-crossing
    /// trade. None if the threshold wasn't crossed (those rows don't
    /// fire `Correlation` — they live as `Pending` until expiry).
    pub cross_latency_ms: Option<i64>,
}

#[derive(Clone)]
pub struct CorrelationStore {
    /// Per-symbol queue of in-flight pending rows. Vec because a single
    /// hot ticker can have multiple overlapping catalysts in flight.
    pending: Arc<DashMap<String, Vec<Pending>>>,
    /// Locked-in correlations keyed by `(catalyst_id, symbol)`.
    emitted: Arc<DashMap<String, Correlation>>,
    tx: broadcast::Sender<Correlation>,
}

impl CorrelationStore {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(512);
        Self {
            pending: Arc::new(DashMap::new()),
            emitted: Arc::new(DashMap::new()),
            tx,
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Correlation> {
        self.tx.subscribe()
    }

    /// Snapshot of locked-in correlations, newest-first.
    pub fn latest(&self, limit: usize) -> Vec<Correlation> {
        let mut all: Vec<Correlation> = self.emitted.iter().map(|e| e.value().clone()).collect();
        all.sort_by_key(|c| std::cmp::Reverse(c.observed_at));
        all.truncate(limit);
        all
    }

    /// Same as `latest` but filtered to one symbol.
    pub fn latest_for(&self, symbol: &str, limit: usize) -> Vec<Correlation> {
        let sym_upper = symbol.to_ascii_uppercase();
        let mut hits: Vec<Correlation> = self
            .emitted
            .iter()
            .filter(|e| e.value().symbol == sym_upper)
            .map(|e| e.value().clone())
            .collect();
        hits.sort_by_key(|c| std::cmp::Reverse(c.observed_at));
        hits.truncate(limit);
        hits
    }

    /// Process one catalyst — open a pending row for every extracted
    /// ticker that we already have a live price on. Tickers without a
    /// baseline (no trade yet today) are dropped silently; the next
    /// catalyst for the same ticker will pick them up.
    fn observe_catalyst(&self, cat: &Catalyst, baseline_fn: &dyn Fn(&str) -> Option<f64>) {
        let sentiment = score_sentiment(&cat.title, &cat.summary);
        let expires_at = cat.published_at + chrono::Duration::seconds(WATCH_SECS);
        let catalyst_id = catalyst_id(cat);
        for ticker in &cat.tickers {
            let sym_upper = ticker.to_ascii_uppercase();
            let baseline = match baseline_fn(&sym_upper) {
                Some(b) if b > 0.0 => b,
                _ => continue,
            };
            let row = Pending {
                catalyst_id: catalyst_id.clone(),
                symbol: sym_upper.clone(),
                title: cat.title.clone(),
                source: cat.source.clone(),
                kind: cat.kind,
                sentiment,
                catalyst_at: cat.published_at,
                expires_at,
                baseline,
                peak: baseline,
                trough: baseline,
                max_abs_move_pct: 0.0,
                threshold_crossed: false,
                cross_latency_ms: None,
            };
            self.pending.entry(sym_upper).or_default().push(row);
        }
    }

    /// Process one trade — fold into every pending row for that symbol
    /// whose watch window hasn't expired. Returns the freshly emitted
    /// correlations so callers can fan them out.
    fn observe_trade(&self, trade: &Trade) -> Vec<Correlation> {
        let sym_upper = trade.symbol.to_ascii_uppercase();
        let trade_ts = DateTime::<Utc>::from_timestamp_millis(trade.ts_ms).unwrap_or_else(Utc::now);
        let mut emitted = Vec::new();
        if let Some(mut rows) = self.pending.get_mut(&sym_upper) {
            rows.retain_mut(|p| {
                if trade_ts > p.expires_at {
                    return false;
                }
                if trade.price > p.peak {
                    p.peak = trade.price;
                }
                if trade.price < p.trough {
                    p.trough = trade.price;
                }
                let move_pct = (trade.price - p.baseline) / p.baseline * 100.0;
                if move_pct.abs() > p.max_abs_move_pct {
                    p.max_abs_move_pct = move_pct.abs();
                }
                if !p.threshold_crossed && move_pct.abs() >= THRESHOLD_PCT {
                    p.threshold_crossed = true;
                    p.cross_latency_ms = Some((trade_ts - p.catalyst_at).num_milliseconds().max(0));
                    let signed = signed_move(p.baseline, p.peak, p.trough);
                    let corr = Correlation {
                        catalyst_id: p.catalyst_id.clone(),
                        symbol: p.symbol.clone(),
                        title: p.title.clone(),
                        source: p.source.clone(),
                        kind: p.kind,
                        sentiment: p.sentiment,
                        catalyst_at: p.catalyst_at,
                        observed_at: trade_ts,
                        baseline: p.baseline,
                        peak: p.peak,
                        trough: p.trough,
                        max_abs_move_pct: p.max_abs_move_pct,
                        signed_move_pct: signed,
                        cross_latency_ms: p.cross_latency_ms,
                    };
                    emitted.push(corr);
                }
                true
            });
            if rows.is_empty() {
                drop(rows);
                self.pending.remove(&sym_upper);
            }
        }
        for c in &emitted {
            let key = format!("{}|{}", c.catalyst_id, c.symbol);
            self.emitted.insert(key, c.clone());
            let _ = self.tx.send(c.clone());
        }
        if !emitted.is_empty() {
            self.evict_if_full();
        }
        emitted
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

    /// Sweep pending rows whose window has expired. Called periodically
    /// from the spawn task — needed because a symbol that gets a
    /// catalyst but then never trades again would leave the pending row
    /// dangling forever.
    fn expire_stale(&self, now: DateTime<Utc>) {
        let mut empty_keys: Vec<String> = Vec::new();
        for mut entry in self.pending.iter_mut() {
            entry.value_mut().retain(|p| p.expires_at >= now);
            if entry.value().is_empty() {
                empty_keys.push(entry.key().clone());
            }
        }
        for k in empty_keys {
            self.pending.remove(&k);
        }
    }
}

impl Default for CorrelationStore {
    fn default() -> Self {
        Self::new()
    }
}

fn signed_move(baseline: f64, peak: f64, trough: f64) -> f64 {
    let up = (peak - baseline) / baseline * 100.0;
    let down = (trough - baseline) / baseline * 100.0;
    if up.abs() >= down.abs() {
        up
    } else {
        down
    }
}

/// Stable ID for a catalyst. The catalyst struct doesn't carry one, so
/// build it from the fields that combine to be unique across the dedupe
/// set: (kind, source, title, published_at).
fn catalyst_id(c: &Catalyst) -> String {
    format!(
        "{:?}|{}|{}|{}",
        c.kind,
        c.source,
        c.title,
        c.published_at.timestamp_millis()
    )
}

// ─── Sentiment lexicon ──────────────────────────────────────────────────────

const BULLISH: &[&str] = &[
    "beat",
    "beats",
    "exceeds",
    "exceeded",
    "raises",
    "raised",
    "guidance higher",
    "approval",
    "approved",
    "upgrade",
    "upgraded",
    "buyback",
    "acquires",
    "acquisition",
    "merger",
    "partnership",
    "agreement signed",
    "wins",
    "awarded",
    "patent granted",
    "fda approval",
    "fda accepts",
    "phase 3 success",
    "exceeded estimates",
    "record revenue",
    "record quarter",
    "expansion",
    "secures funding",
    "contract awarded",
    "positive results",
    "breakthrough",
    "launches",
    "rollout",
];

const BEARISH: &[&str] = &[
    "miss",
    "misses",
    "missed",
    "lowered",
    "lowers",
    "downgrade",
    "downgraded",
    "guidance lower",
    "guides lower",
    "investigation",
    "subpoena",
    "lawsuit",
    "delisting",
    "going concern",
    "restatement",
    "ceo resigns",
    "ceo departure",
    "layoffs",
    "bankruptcy",
    "default",
    "delay",
    "delays",
    "delayed",
    "halt",
    "recall",
    "fraud",
    "sec complaint",
    "phase 3 failure",
    "phase 3 miss",
    "warning letter",
    "non-compliance",
    "data breach",
    "cyberattack",
    "guidance withdrawn",
    "going to chapter 11",
];

/// Cheap lexicon-based scoring over `title + summary`. Returns
/// `Sentiment::Bullish` / `Sentiment::Bearish` if the bullish or
/// bearish hit count strictly dominates, else `Neutral`. The lexicon
/// is conservative on purpose — false-positive bullish/bearish tags
/// are worse than neutral non-tags in a trading workflow.
///
/// Matching uses word boundaries — without them, "beat" + "beats"
/// both fire on "beats estimates", double-counting one sentiment and
/// breaking the tied-counts → Neutral invariant.
pub fn score_sentiment(title: &str, summary: &str) -> Sentiment {
    let blob = format!("{} {}", title, summary).to_ascii_lowercase();
    let bull = BULLISH.iter().filter(|w| word_match(&blob, w)).count();
    let bear = BEARISH.iter().filter(|w| word_match(&blob, w)).count();
    if bull > bear {
        Sentiment::Bullish
    } else if bear > bull {
        Sentiment::Bearish
    } else {
        Sentiment::Neutral
    }
}

/// True when `needle` appears in `haystack` bracketed by non-alphabetic
/// chars (or the string ends). Handles multi-word lexicon entries
/// ("guidance higher", "fda approval") because the boundary check
/// applies only to the outer edges of the match.
fn word_match(haystack: &str, needle: &str) -> bool {
    let h = haystack.as_bytes();
    let n = needle.as_bytes();
    if n.is_empty() || h.len() < n.len() {
        return false;
    }
    let is_alpha = |b: u8| b.is_ascii_alphabetic();
    for start in 0..=(h.len() - n.len()) {
        if &h[start..start + n.len()] != n {
            continue;
        }
        let before_ok = start == 0 || !is_alpha(h[start - 1]);
        let after = start + n.len();
        let after_ok = after == h.len() || !is_alpha(h[after]);
        if before_ok && after_ok {
            return true;
        }
    }
    false
}

// ─── Wiring ────────────────────────────────────────────────────────────────

/// Spawn the two consumer tasks (catalyst + trade) and a periodic
/// sweeper for stale pending rows. Survives broadcast Lagged by
/// re-subscribing.
pub fn spawn_correlator(store: CorrelationStore, catalysts: CatalystStore, ticks: LiveTickStore) {
    // Catalyst consumer.
    {
        let store = store.clone();
        let ticks_for_baseline = ticks.clone();
        tokio::spawn(async move {
            loop {
                let mut rx = catalysts.subscribe();
                loop {
                    match rx.recv().await {
                        Ok(cat) => {
                            let baseline = |sym: &str| -> Option<f64> {
                                ticks_for_baseline
                                    .get(sym)
                                    .map(|s| s.last)
                                    .filter(|p| *p > 0.0)
                            };
                            store.observe_catalyst(&cat, &baseline);
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                            tracing::warn!(skipped, "catalyst_correlator lagged catalyst stream");
                            continue;
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                    }
                }
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        });
    }
    // Trade consumer.
    {
        let store = store.clone();
        tokio::spawn(async move {
            loop {
                let mut rx = ticks.tape_subscribe();
                loop {
                    match rx.recv().await {
                        Ok(t) => {
                            store.observe_trade(&t);
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                            tracing::warn!(skipped, "catalyst_correlator lagged tape");
                            continue;
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                    }
                }
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        });
    }
    // Periodic sweeper.
    {
        let store = store.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(10));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                interval.tick().await;
                store.expire_stale(Utc::now());
            }
        });
    }
}

pub fn global() -> CorrelationStore {
    static STORE: once_cell::sync::OnceCell<CorrelationStore> = once_cell::sync::OnceCell::new();
    STORE
        .get_or_init(|| {
            let s = CorrelationStore::new();
            spawn_correlator(
                s.clone(),
                crate::catalysts::global(),
                crate::live_ticks::global(),
            );
            s
        })
        .clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cat(title: &str, ticker: &str, ts: DateTime<Utc>) -> Catalyst {
        Catalyst {
            kind: CatalystKind::PressRelease,
            source: "TEST".into(),
            form_type: None,
            title: title.into(),
            summary: String::new(),
            link: None,
            published_at: ts,
            fetched_at: ts,
            tickers: vec![ticker.into()],
        }
    }

    fn trade(sym: &str, price: f64, ts_ms: i64) -> Trade {
        Trade {
            symbol: sym.into(),
            price,
            volume: 100.0,
            ts_ms,
        }
    }

    #[test]
    fn sentiment_bullish_on_beats_and_raises() {
        let s = score_sentiment("Acme beats Q3 estimates, raises FY guidance", "");
        assert_eq!(s, Sentiment::Bullish);
    }

    #[test]
    fn sentiment_bearish_on_lawsuit_and_downgrade() {
        let s = score_sentiment("FDA recall and lawsuit filed against Acme", "");
        assert_eq!(s, Sentiment::Bearish);
    }

    #[test]
    fn sentiment_neutral_when_no_lexicon_hits() {
        let s = score_sentiment("Acme to present at investor conference", "");
        assert_eq!(s, Sentiment::Neutral);
    }

    #[test]
    fn sentiment_neutral_when_both_sides_hit() {
        // "beats" + "investigation" both fire; counts are tied → Neutral.
        let s = score_sentiment("Acme beats estimates amid SEC investigation", "");
        assert_eq!(s, Sentiment::Neutral);
    }

    #[test]
    fn catalyst_without_baseline_creates_no_pending() {
        let store = CorrelationStore::new();
        let now = Utc::now();
        store.observe_catalyst(&cat("Acme up", "ACME", now), &|_| None);
        assert!(store.pending.is_empty());
    }

    #[test]
    fn trade_below_threshold_does_not_emit() {
        let store = CorrelationStore::new();
        let now = Utc::now();
        store.observe_catalyst(&cat("Acme beats", "ACME", now), &|_| Some(100.0));
        // 1% move → below 2% threshold.
        let emitted = store.observe_trade(&trade("ACME", 101.0, now.timestamp_millis() + 500));
        assert!(emitted.is_empty());
        // Pending row still alive with max_abs_move_pct = 1.0.
        let rows = store.pending.get("ACME").unwrap();
        assert_eq!(rows.len(), 1);
        assert!((rows[0].max_abs_move_pct - 1.0).abs() < 1e-9);
    }

    #[test]
    fn threshold_crossing_emits_correlation_once() {
        let store = CorrelationStore::new();
        // Use a timestamp aligned to an exact millisecond boundary so
        // (trade_ts - cat_ts).num_milliseconds() doesn't lose a count
        // to sub-ms truncation in from_timestamp_millis.
        let cat_ts = DateTime::<Utc>::from_timestamp_millis(1_700_000_000_000).unwrap();
        store.observe_catalyst(&cat("Acme FDA approval", "ACME", cat_ts), &|_| Some(100.0));
        let crossed = store.observe_trade(&trade("ACME", 103.0, cat_ts.timestamp_millis() + 1_500));
        assert_eq!(crossed.len(), 1);
        let c = &crossed[0];
        assert_eq!(c.symbol, "ACME");
        assert_eq!(c.sentiment, Sentiment::Bullish);
        assert!((c.signed_move_pct - 3.0).abs() < 1e-9);
        assert_eq!(c.cross_latency_ms.unwrap(), 1_500);
        // A second trade in the same window must NOT re-emit.
        let again = store.observe_trade(&trade("ACME", 104.0, cat_ts.timestamp_millis() + 2_500));
        assert!(again.is_empty());
        // ... but it must update peak / max_move.
        let snap = store.latest(10);
        // Note: emitted snapshot is frozen at cross-time. Verify
        // pending row reflects the new peak instead.
        let pend = store.pending.get("ACME").unwrap();
        assert!((pend[0].peak - 104.0).abs() < 1e-9);
        assert_eq!(snap.len(), 1);
    }

    #[test]
    fn trade_after_expiry_drops_pending() {
        let store = CorrelationStore::new();
        let cat_ts = Utc::now();
        store.observe_catalyst(&cat("Acme up", "ACME", cat_ts), &|_| Some(100.0));
        // Trade 65s later — past the 60s window.
        let emitted =
            store.observe_trade(&trade("ACME", 110.0, cat_ts.timestamp_millis() + 65_000));
        assert!(emitted.is_empty());
        assert!(store.pending.get("ACME").is_none());
    }

    #[test]
    fn expire_stale_clears_dead_rows() {
        let store = CorrelationStore::new();
        let cat_ts = Utc::now();
        store.observe_catalyst(&cat("Acme up", "ACME", cat_ts), &|_| Some(100.0));
        assert!(store.pending.contains_key("ACME"));
        let future = cat_ts + chrono::Duration::seconds(WATCH_SECS + 5);
        store.expire_stale(future);
        assert!(!store.pending.contains_key("ACME"));
    }

    #[test]
    fn signed_move_picks_largest_absolute_swing() {
        // Peak +5%, trough -1% → signed = +5.
        assert!((signed_move(100.0, 105.0, 99.0) - 5.0).abs() < 1e-9);
        // Peak +1%, trough -4% → signed = -4.
        assert!((signed_move(100.0, 101.0, 96.0) - -4.0).abs() < 1e-9);
    }

    #[test]
    fn evict_caps_emitted_set() {
        let store = CorrelationStore::new();
        for i in 0..5_000 {
            let c = Correlation {
                catalyst_id: format!("id-{i}"),
                symbol: format!("S{i:05}"),
                title: "x".into(),
                source: "TEST".into(),
                kind: CatalystKind::PressRelease,
                sentiment: Sentiment::Neutral,
                catalyst_at: Utc::now(),
                observed_at: Utc::now() + chrono::Duration::seconds(i),
                baseline: 100.0,
                peak: 100.0,
                trough: 100.0,
                max_abs_move_pct: 0.0,
                signed_move_pct: 0.0,
                cross_latency_ms: None,
            };
            store
                .emitted
                .insert(format!("{}|{}", c.catalyst_id, c.symbol), c);
        }
        store.evict_if_full();
        assert!(store.emitted.len() <= EMITTED_CAP);
    }
}
