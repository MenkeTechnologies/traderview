//! After-hours mover scanner.
//!
//! Subscribes to `LiveTickStore::tape_subscribe()` and classifies every
//! incoming trade by US/Eastern wall-clock session:
//!
//!   PRE    04:00 – 09:30 ET   (Mon–Fri)
//!   RTH    09:30 – 16:00 ET   (Mon–Fri)
//!   POST   16:00 – 20:00 ET   (Mon–Fri)
//!   CLOSED everywhere else
//!
//! Per-symbol state tracks:
//!
//!   * `rth_close` — sticky last price seen during the most recent RTH
//!     window (carry-over across PRE / POST until the next RTH refresh).
//!   * `session` — the AH window currently being aggregated (Pre / Post).
//!   * `session_open` — first AH trade price; resets at each AH transition.
//!   * `ah_high` / `ah_low` / `ah_last` / `ah_volume` — aggregates for the
//!     current AH session.
//!   * `change_pct` — `(ah_last - rth_close) / rth_close * 100`.
//!   * `range_pct` — `(ah_high - ah_low) / rth_close * 100`.
//!   * `last_trade_at`, `trade_count`.
//!
//! The store exposes:
//!
//!   * [`AfterHoursStore::movers`] — top-N gainers/losers for a given session.
//!   * [`AfterHoursStore::subscribe`] — broadcast of every AH-state update.
//!
//! Crypto symbols trade 24/7, so they have no RTH boundary; they are
//! skipped at the classifier so the scanner never lights them up.
//!
//! The ET→UTC offset uses the same monthly approximation precedent as
//! `economy.rs` and `broker_dispatcher.rs` (Mar–Nov ≈ EDT UTC-4, else
//! EST UTC-5). Real DST flip-points are a few minutes off twice a year;
//! the after-hours mover scanner is wall-clock advisory, not a settlement
//! engine, so the approximation is fine.

use chrono::{DateTime, Datelike, FixedOffset, Timelike, Utc, Weekday};
use dashmap::DashMap;
use serde::Serialize;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;

use crate::live_ticks::{LiveTickStore, Trade};

/// Wall-clock trading session in US/Eastern, used only to classify
/// incoming ticks. The numeric ordering is meaningful: PRE < RTH < POST
/// so transition logic can use comparisons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Session {
    Closed,
    Pre,
    Rth,
    Post,
}

impl Session {
    pub fn as_str(self) -> &'static str {
        match self {
            Session::Closed => "closed",
            Session::Pre => "pre",
            Session::Rth => "rth",
            Session::Post => "post",
        }
    }
}

/// One row of the after-hours mover scanner.
#[derive(Debug, Clone, Serialize)]
pub struct AfterHoursState {
    pub symbol: String,
    /// Currently aggregated AH window: `Pre` or `Post`. After RTH starts
    /// the previous window's fields are preserved until the next AH event
    /// arrives — that lets a UI sort by "biggest pre-market move so far
    /// today" right up to and through the RTH open.
    pub session: Session,
    /// Last RTH close observed for this symbol. Required to compute
    /// change_pct in PRE/POST. None until at least one RTH trade has been
    /// seen.
    pub rth_close: Option<f64>,
    pub session_open: Option<f64>,
    pub ah_high: f64,
    pub ah_low: f64,
    pub ah_last: f64,
    pub ah_volume: f64,
    pub change_pct: f64,
    pub range_pct: f64,
    pub last_trade_at: DateTime<Utc>,
    pub trade_count: u64,
}

impl AfterHoursState {
    fn new(symbol: &str) -> Self {
        AfterHoursState {
            symbol: symbol.into(),
            session: Session::Closed,
            rth_close: None,
            session_open: None,
            ah_high: 0.0,
            ah_low: f64::INFINITY,
            ah_last: 0.0,
            ah_volume: 0.0,
            change_pct: 0.0,
            range_pct: 0.0,
            last_trade_at: Utc::now(),
            trade_count: 0,
        }
    }

    /// Reset only the AH aggregates — `rth_close` carries over so we can
    /// still compute change_pct when the next AH session starts.
    fn reset_ah(&mut self) {
        self.session_open = None;
        self.ah_high = 0.0;
        self.ah_low = f64::INFINITY;
        self.ah_last = 0.0;
        self.ah_volume = 0.0;
        self.change_pct = 0.0;
        self.range_pct = 0.0;
    }

    fn observe(&mut self, t: &Trade, session: Session) {
        let now = chrono::DateTime::<Utc>::from_timestamp_millis(t.ts_ms).unwrap_or_else(Utc::now);
        self.last_trade_at = now;
        self.trade_count += 1;

        match session {
            Session::Rth => {
                // RTH trade — update the rolling close so we have a
                // baseline for the next PRE/POST window. Don't touch
                // ah_* aggregates: those describe the most recent AH
                // window and stay visible through RTH.
                self.rth_close = Some(t.price);
                self.session = Session::Rth;
            }
            Session::Pre | Session::Post => {
                // Window flipped (PRE→POST, POST→PRE, or first AH trade
                // after an RTH gap) → reset AH aggregates so the row
                // shows the active session's move, not stale state.
                if self.session != session {
                    self.reset_ah();
                    self.session = session;
                }
                if self.session_open.is_none() {
                    self.session_open = Some(t.price);
                }
                self.ah_last = t.price;
                if t.price > self.ah_high {
                    self.ah_high = t.price;
                }
                if t.price < self.ah_low {
                    self.ah_low = t.price;
                }
                self.ah_volume += t.volume;

                if let Some(close) = self.rth_close {
                    if close > 0.0 {
                        self.change_pct = (self.ah_last - close) / close * 100.0;
                        self.range_pct = (self.ah_high - self.ah_low) / close * 100.0;
                    }
                }
            }
            Session::Closed => {
                // Late-evening / overnight equity prints — keep state
                // visible but don't roll into a new session bucket.
            }
        }
    }
}

#[derive(Clone)]
pub struct AfterHoursStore {
    state: Arc<DashMap<String, AfterHoursState>>,
    tx: broadcast::Sender<AfterHoursState>,
}

impl AfterHoursStore {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(512);
        Self {
            state: Arc::new(DashMap::new()),
            tx,
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<AfterHoursState> {
        self.tx.subscribe()
    }

    /// Snapshot the top-N movers in the requested session, ranked by
    /// signed `change_pct`. `gainers=true` sorts descending; otherwise
    /// ascending (biggest losers first). Symbols whose change_pct is
    /// below `min_abs_pct` are filtered out so the table isn't full of
    /// 0.01% prints.
    pub fn movers(
        &self,
        session: Session,
        gainers: bool,
        limit: usize,
        min_abs_pct: f64,
    ) -> Vec<AfterHoursState> {
        let mut rows: Vec<AfterHoursState> = self
            .state
            .iter()
            .filter(|e| {
                let s = e.value();
                s.session == session && s.rth_close.is_some() && s.change_pct.abs() >= min_abs_pct
            })
            .map(|e| e.value().clone())
            .collect();
        if gainers {
            rows.sort_by(|a, b| {
                b.change_pct
                    .partial_cmp(&a.change_pct)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        } else {
            rows.sort_by(|a, b| {
                a.change_pct
                    .partial_cmp(&b.change_pct)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }
        rows.truncate(limit);
        rows
    }

    /// Full snapshot, newest-trade first. Used by the WS handler to
    /// hydrate clients on connect.
    pub fn snapshot(&self, limit: usize) -> Vec<AfterHoursState> {
        let mut rows: Vec<AfterHoursState> = self
            .state
            .iter()
            .filter(|e| e.value().session != Session::Closed)
            .map(|e| e.value().clone())
            .collect();
        rows.sort_by_key(|r| std::cmp::Reverse(r.last_trade_at));
        rows.truncate(limit);
        rows
    }

    fn observe(&self, trade: &Trade) {
        if is_crypto_like(&trade.symbol) {
            return;
        }
        let session = classify(now_et());
        // Don't record CLOSED-session prints — equity venues do report
        // late prints, but they aren't actionable for a mover scanner.
        if matches!(session, Session::Closed) {
            return;
        }
        let updated = {
            let mut entry = self
                .state
                .entry(trade.symbol.clone())
                .or_insert_with(|| AfterHoursState::new(&trade.symbol));
            entry.observe(trade, session);
            entry.clone()
        };
        // Only fan out PRE/POST updates — RTH trades just refresh the
        // rolling close in the background and shouldn't spam the WS.
        if matches!(updated.session, Session::Pre | Session::Post) {
            let _ = self.tx.send(updated);
        }
        self.evict_if_full();
    }

    fn evict_if_full(&self) {
        const MAX_ENTRIES: usize = 4_000;
        if self.state.len() <= MAX_ENTRIES {
            return;
        }
        // Drop the oldest 1/4. Same evict strategy as halts.
        let drop_n = self.state.len() / 4;
        let mut by_age: Vec<(String, DateTime<Utc>)> = self
            .state
            .iter()
            .map(|e| (e.key().clone(), e.value().last_trade_at))
            .collect();
        by_age.sort_by_key(|(_, t)| *t);
        for (key, _) in by_age.into_iter().take(drop_n) {
            self.state.remove(&key);
        }
    }
}

impl Default for AfterHoursStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Spawn the classifier task. Subscribes to the global LiveTickStore tape
/// and folds every trade into AfterHoursState. Reconnects the subscription
/// on broadcast Lagged (raw tape is 2048 deep; under heavy load a slow
/// subscriber can fall behind).
pub fn spawn_classifier(store: AfterHoursStore, ticks: LiveTickStore) {
    tokio::spawn(async move {
        loop {
            let mut rx = ticks.tape_subscribe();
            loop {
                match rx.recv().await {
                    Ok(t) => store.observe(&t),
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                        tracing::warn!(skipped, "after_hours classifier lagged tape");
                        continue;
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
            }
            // Channel closed (store dropped) — back off and retry.
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    });
}

/// Singleton store + auto-spawn the classifier on first access. Mirrors
/// the halts/live_ticks lazy-init pattern so any route can call
/// `after_hours::global()` without worrying about wiring order.
pub fn global() -> AfterHoursStore {
    static STORE: once_cell::sync::OnceCell<AfterHoursStore> = once_cell::sync::OnceCell::new();
    STORE
        .get_or_init(|| {
            let s = AfterHoursStore::new();
            spawn_classifier(s.clone(), crate::live_ticks::global());
            s
        })
        .clone()
}

// ─── Session classification ────────────────────────────────────────────────

/// Current wall-clock in US/Eastern using a monthly EDT/EST switch.
/// Same precedent as `economy.rs` + `broker_dispatcher.rs` — avoids
/// pulling in chrono-tz/tzdata for a single classifier.
fn now_et() -> DateTime<FixedOffset> {
    let month = Utc::now().month();
    let offset_h = if (3..=11).contains(&month) { 4 } else { 5 };
    Utc::now().with_timezone(&FixedOffset::west_opt(offset_h * 3600).expect("ET offset"))
}

fn classify(et: DateTime<FixedOffset>) -> Session {
    if matches!(et.weekday(), Weekday::Sat | Weekday::Sun) {
        return Session::Closed;
    }
    let mins = et.hour() * 60 + et.minute();
    if (240..570).contains(&mins) {
        Session::Pre
    } else if (570..960).contains(&mins) {
        Session::Rth
    } else if (960..1200).contains(&mins) {
        Session::Post
    } else {
        Session::Closed
    }
}

/// Skip 24/7 quote-currency symbols at the classifier — they trade
/// through every "session" so an AH scanner would always flag them.
/// Conservative match: anything containing "/USD" or ending in "USD"
/// with a known crypto-prefix base. False positives are harmless;
/// false negatives just mean a crypto print shows up in the table.
fn is_crypto_like(sym: &str) -> bool {
    let s = sym.to_ascii_uppercase();
    if s.contains('/') {
        return true;
    }
    const CRYPTO_BASES: &[&str] = &[
        "BTC", "ETH", "LTC", "BCH", "DOGE", "SOL", "AVAX", "MATIC", "ADA", "DOT", "XRP", "LINK",
        "UNI", "SHIB", "AAVE", "ALGO", "ATOM", "BAT", "COMP", "CRV", "GRT", "MKR", "PAXG", "SUSHI",
        "TRX", "XLM", "XTZ", "YFI", "ZRX",
    ];
    if s.ends_with("USD") && s.len() > 3 {
        let base = &s[..s.len() - 3];
        if CRYPTO_BASES.contains(&base) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn trade(sym: &str, price: f64, vol: f64, ts_ms: i64) -> Trade {
        Trade {
            symbol: sym.into(),
            price,
            volume: vol,
            ts_ms,
        }
    }

    #[test]
    fn classify_session_windows() {
        let mk = |h: u32, m: u32| {
            let d = chrono::NaiveDate::from_ymd_opt(2026, 6, 8).unwrap(); // Mon, EDT
            let t = chrono::NaiveTime::from_hms_opt(h, m, 0).unwrap();
            let ndt = chrono::NaiveDateTime::new(d, t);
            FixedOffset::west_opt(4 * 3600)
                .unwrap()
                .from_local_datetime(&ndt)
                .single()
                .unwrap()
        };
        use chrono::TimeZone;
        assert_eq!(classify(mk(3, 59)), Session::Closed); // before PRE
        assert_eq!(classify(mk(4, 0)), Session::Pre);
        assert_eq!(classify(mk(9, 29)), Session::Pre);
        assert_eq!(classify(mk(9, 30)), Session::Rth);
        assert_eq!(classify(mk(15, 59)), Session::Rth);
        assert_eq!(classify(mk(16, 0)), Session::Post);
        assert_eq!(classify(mk(19, 59)), Session::Post);
        assert_eq!(classify(mk(20, 0)), Session::Closed);
    }

    #[test]
    fn classify_weekend_is_closed() {
        use chrono::TimeZone;
        // Saturday 2026-06-06 at 10:00 ET — would be RTH on a weekday.
        let d = chrono::NaiveDate::from_ymd_opt(2026, 6, 6).unwrap();
        let t = chrono::NaiveTime::from_hms_opt(10, 0, 0).unwrap();
        let ndt = chrono::NaiveDateTime::new(d, t);
        let sat = FixedOffset::west_opt(4 * 3600)
            .unwrap()
            .from_local_datetime(&ndt)
            .single()
            .unwrap();
        assert_eq!(classify(sat), Session::Closed);
    }

    #[test]
    fn rth_trade_updates_close_without_publishing() {
        let mut st = AfterHoursState::new("AAPL");
        st.observe(&trade("AAPL", 200.0, 100.0, 0), Session::Rth);
        assert_eq!(st.rth_close, Some(200.0));
        assert_eq!(st.session, Session::Rth);
        // RTH ticks must NOT populate AH aggregates.
        assert_eq!(st.ah_last, 0.0);
        assert_eq!(st.ah_volume, 0.0);
        assert_eq!(st.session_open, None);
    }

    #[test]
    fn post_session_change_pct_against_rth_close() {
        let mut st = AfterHoursState::new("NVDA");
        st.observe(&trade("NVDA", 500.0, 1.0, 0), Session::Rth);
        st.observe(&trade("NVDA", 525.0, 1000.0, 1), Session::Post);
        assert_eq!(st.session, Session::Post);
        assert_eq!(st.session_open, Some(525.0));
        assert_eq!(st.ah_last, 525.0);
        assert_eq!(st.ah_high, 525.0);
        assert_eq!(st.ah_low, 525.0);
        assert!((st.change_pct - 5.0).abs() < 1e-9);
        assert_eq!(st.ah_volume, 1000.0);
    }

    #[test]
    fn session_flip_resets_ah_but_keeps_rth_close() {
        let mut st = AfterHoursState::new("TSLA");
        st.observe(&trade("TSLA", 100.0, 1.0, 0), Session::Rth);
        st.observe(&trade("TSLA", 110.0, 500.0, 1), Session::Post);
        // Next AM PRE session — AH aggregates must reset, rth_close
        // must persist so the new change_pct stays valid.
        st.observe(&trade("TSLA", 105.0, 200.0, 2), Session::Pre);
        assert_eq!(st.session, Session::Pre);
        assert_eq!(st.rth_close, Some(100.0));
        assert_eq!(st.session_open, Some(105.0));
        assert_eq!(st.ah_volume, 200.0); // not 700
        assert!((st.change_pct - 5.0).abs() < 1e-9);
    }

    #[test]
    fn high_low_range_tracks_intra_session_swing() {
        let mut st = AfterHoursState::new("AMC");
        st.observe(&trade("AMC", 10.0, 1.0, 0), Session::Rth);
        st.observe(&trade("AMC", 11.0, 10.0, 1), Session::Post);
        st.observe(&trade("AMC", 12.0, 10.0, 2), Session::Post);
        st.observe(&trade("AMC", 9.5, 10.0, 3), Session::Post);
        assert_eq!(st.ah_high, 12.0);
        assert_eq!(st.ah_low, 9.5);
        assert_eq!(st.ah_last, 9.5);
        // range_pct = (12 - 9.5) / 10.0 * 100 = 25.0
        assert!((st.range_pct - 25.0).abs() < 1e-9);
        // change_pct = (9.5 - 10) / 10 * 100 = -5.0
        assert!((st.change_pct - -5.0).abs() < 1e-9);
    }

    #[test]
    fn movers_sorts_and_filters() {
        let store = AfterHoursStore::new();
        let session = Session::Post;
        // Seed three symbols with different change_pct so we can verify
        // gainers/losers ordering and the min_abs_pct filter.
        for (sym, rth, ah) in [
            ("GAIN", 100.0, 110.0),  // +10%
            ("FLAT", 100.0, 100.05), // +0.05% — filtered out at 1%
            ("LOSE", 100.0, 92.0),   // -8%
        ] {
            let mut s = AfterHoursState::new(sym);
            s.observe(&trade(sym, rth, 1.0, 0), Session::Rth);
            s.observe(&trade(sym, ah, 100.0, 1), session);
            store.state.insert(sym.into(), s);
        }
        let gainers = store.movers(session, true, 10, 1.0);
        assert_eq!(gainers.len(), 2);
        assert_eq!(gainers[0].symbol, "GAIN");
        assert_eq!(gainers[1].symbol, "LOSE");
        let losers = store.movers(session, false, 10, 1.0);
        assert_eq!(losers[0].symbol, "LOSE");
        assert_eq!(losers[1].symbol, "GAIN");
    }

    #[test]
    fn crypto_symbols_are_skipped() {
        assert!(is_crypto_like("BTC/USD"));
        assert!(is_crypto_like("BTCUSD"));
        assert!(is_crypto_like("ETHUSD"));
        assert!(!is_crypto_like("AAPL"));
        assert!(!is_crypto_like("SPY"));
        assert!(!is_crypto_like("TSLA"));
    }

    #[test]
    fn evict_drops_oldest_when_over_cap() {
        let store = AfterHoursStore::new();
        // Insert 5000 distinct symbols with staggered last_trade_at so
        // eviction has an ordering signal.
        for i in 0..5_000 {
            let mut s = AfterHoursState::new(&format!("S{i:05}"));
            s.last_trade_at = Utc::now() + chrono::Duration::seconds(i);
            store.state.insert(format!("S{i:05}"), s);
        }
        store.evict_if_full();
        assert!(
            store.state.len() <= 4_000,
            "post-evict size still over cap: {}",
            store.state.len()
        );
        // Oldest entries (S00000..) should be the ones gone.
        assert!(!store.state.contains_key("S00000"));
        assert!(store.state.contains_key("S04999"));
    }
}
