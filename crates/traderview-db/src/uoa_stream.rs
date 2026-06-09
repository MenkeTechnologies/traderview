//! Real-time Unusual Options Activity (UOA) stream.
//!
//! Today's UOA endpoint (`POST /analytics/unusual-options`) is on-demand:
//! the user pastes a chain blob and gets back hits. That's useful but
//! mistimes the workflow — by the time the user finds a hot ticker,
//! adds it, and pastes the chain, the unusual print is hours stale.
//!
//! This module flips that. A background poller rotates through the
//! top-N symbols by live trade activity (`LiveTickStore.snapshot()`
//! ranked by `trade_count`), fetches each one's options chain off
//! Yahoo, runs `unusual_options_activity::scan` with the default
//! thresholds, and broadcasts every newly-emitted hit on a tokio
//! channel. Hits are deduped by `(symbol, expiry, strike, option_type)`
//! so a long-lived print only fires once.
//!
//! Cadence: one full round of TOP_N symbols per ROUND_SECS. Inside a
//! round, requests are paced PACE_MS apart so we don't burst Yahoo
//! into a 429.
//!
//! Limitations:
//!   * Yahoo's chain only returns the *front-month* expiry by default.
//!     We pull the single returned strip per symbol — anything LEAPS
//!     or back-month requires a separate `?date=` call per expiry,
//!     which would 20× the budget for marginal benefit.
//!   * Yahoo can return stale volume + OI mid-session (the snapshots
//!     refresh every ~1-2 minutes). That's fine for a 60s scanner;
//!     it would not be fine for an HFT engine.
//!   * The `fill_side` classification is best-effort — Yahoo doesn't
//!     ship per-print trade tape, so we rely on `last_price` vs the
//!     latest `bid/ask`. AboveAsk/BelowBid is conservative.

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::Serialize;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;

use crate::live_ticks::LiveTickStore;
use crate::options;
use traderview_core::unusual_options_activity::{
    self as core_uoa, Config as UoaConfig, FillSide, OptionContract,
};

/// How many of the most-active symbols to scan per round.
const TOP_N: usize = 20;
/// Wall-clock seconds between full rounds. 60s keeps Yahoo's load
/// manageable (~1200 req/hr) while still surfacing fresh prints
/// inside the timescales day-traders react on.
const ROUND_SECS: u64 = 60;
/// Pacing between requests within a round so a single round doesn't
/// open 20 simultaneous TCP connections to Yahoo and trigger a 429.
const PACE_MS: u64 = 250;
/// Cap on emitted hits kept in memory.
const EMITTED_CAP: usize = 2_000;

/// One UOA hit augmented with a server-side timestamp so callers can
/// sort by recency without parsing a payload.
#[derive(Debug, Clone, Serialize)]
pub struct UoaEvent {
    pub symbol: String,
    pub expiry: String,
    pub strike: f64,
    pub option_type: String,
    pub volume: f64,
    pub open_interest: f64,
    pub vol_oi_ratio: f64,
    pub premium_paid: f64,
    pub fill_side: FillSide,
    pub observed_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct UoaStreamStore {
    /// Deduped emitted hits keyed by `(symbol|expiry|strike|type)`.
    emitted: Arc<DashMap<String, UoaEvent>>,
    tx: broadcast::Sender<UoaEvent>,
}

impl UoaStreamStore {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(256);
        Self {
            emitted: Arc::new(DashMap::new()),
            tx,
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<UoaEvent> {
        self.tx.subscribe()
    }

    /// Recent emitted hits newest-first.
    pub fn latest(&self, limit: usize) -> Vec<UoaEvent> {
        let mut all: Vec<UoaEvent> = self.emitted.iter().map(|e| e.value().clone()).collect();
        all.sort_by_key(|e| std::cmp::Reverse(e.observed_at));
        all.truncate(limit);
        all
    }

    /// Same as `latest` filtered to one underlier.
    pub fn latest_for(&self, symbol: &str, limit: usize) -> Vec<UoaEvent> {
        let sym_upper = symbol.to_ascii_uppercase();
        let mut hits: Vec<UoaEvent> = self
            .emitted
            .iter()
            .filter(|e| e.value().symbol == sym_upper)
            .map(|e| e.value().clone())
            .collect();
        hits.sort_by_key(|e| std::cmp::Reverse(e.observed_at));
        hits.truncate(limit);
        hits
    }

    /// Record a hit. Returns true if it's the first time we've seen
    /// this contract key — false on dedupe. Callers use the bool to
    /// decide whether to fire the broadcast (avoids re-spamming a
    /// long-lived print every round).
    fn observe(&self, ev: UoaEvent) -> bool {
        let key = format!(
            "{}|{}|{}|{}",
            ev.symbol, ev.expiry, ev.strike, ev.option_type
        );
        if self.emitted.contains_key(&key) {
            return false;
        }
        self.emitted.insert(key, ev.clone());
        let _ = self.tx.send(ev);
        self.evict_if_full();
        true
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
}

impl Default for UoaStreamStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Choose the most-active TOP_N symbols from `LiveTickStore` by raw
/// `trade_count`. Crypto symbols are filtered out — options chains are
/// US-equity only on Yahoo's free endpoint. Tied trade_count falls
/// back to alphabetical for determinism.
pub fn top_n_active(ticks: &LiveTickStore, n: usize) -> Vec<String> {
    let mut rows: Vec<(String, u64)> = ticks
        .snapshot()
        .into_iter()
        .filter(|s| !is_crypto_like(&s.symbol))
        .map(|s| (s.symbol, s.trade_count))
        .collect();
    rows.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    rows.into_iter().take(n).map(|(s, _)| s).collect()
}

fn is_crypto_like(sym: &str) -> bool {
    let s = sym.to_ascii_uppercase();
    if s.contains('/') {
        return true;
    }
    const BASES: &[&str] = &[
        "BTC", "ETH", "LTC", "BCH", "DOGE", "SOL", "AVAX", "MATIC", "ADA", "DOT", "XRP", "LINK",
        "UNI", "SHIB", "AAVE", "ALGO", "ATOM", "BAT", "COMP", "CRV", "GRT", "MKR", "PAXG", "SUSHI",
        "TRX", "XLM", "XTZ", "YFI", "ZRX",
    ];
    if s.ends_with("USD") && s.len() > 3 {
        let base = &s[..s.len() - 3];
        if BASES.contains(&base) {
            return true;
        }
    }
    false
}

/// Convert a Yahoo chain into the core UOA scanner's `OptionContract`
/// shape. Missing-field defaults are conservative — None volume → 0
/// (won't trip min_volume), None OI → 0 (won't divide), missing
/// last_price → 0 (won't pass min_premium_paid).
pub fn chain_to_contracts(chain: &options::Chain) -> Vec<OptionContract> {
    let mut out = Vec::with_capacity(chain.calls.len() + chain.puts.len());
    let push_side = |out: &mut Vec<OptionContract>, side: &[options::OptionContract], ty: &str| {
        for c in side {
            out.push(OptionContract {
                symbol: chain.symbol.clone(),
                strike: c.strike,
                expiry: chain.expiration.format("%Y-%m-%d").to_string(),
                option_type: ty.into(),
                volume: c.volume.unwrap_or(0) as f64,
                open_interest: c.open_interest.unwrap_or(0) as f64,
                last_price: c.last_price.unwrap_or(0.0),
                bid: c.bid.unwrap_or(0.0),
                ask: c.ask.unwrap_or(0.0),
            });
        }
    };
    push_side(&mut out, &chain.calls, "call");
    push_side(&mut out, &chain.puts, "put");
    out
}

/// Spawn the rotation poller. Idempotent: call once at boot.
pub fn spawn_poller(store: UoaStreamStore, ticks: LiveTickStore) {
    let cfg = UoaConfig::default();
    tokio::spawn(async move {
        // Stagger startup so we don't hammer Yahoo the instant the
        // process boots.
        tokio::time::sleep(Duration::from_secs(15)).await;
        let mut interval = tokio::time::interval(Duration::from_secs(ROUND_SECS));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            interval.tick().await;
            let symbols = top_n_active(&ticks, TOP_N);
            if symbols.is_empty() {
                continue;
            }
            for sym in symbols {
                match options::chain(&sym, None).await {
                    Ok(chain) => {
                        let contracts = chain_to_contracts(&chain);
                        let hits = core_uoa::scan(&contracts, &cfg);
                        let observed_at = Utc::now();
                        for h in hits {
                            store.observe(UoaEvent {
                                symbol: h.symbol,
                                expiry: h.expiry,
                                strike: h.strike,
                                option_type: h.option_type,
                                volume: h.volume,
                                open_interest: h.open_interest,
                                vol_oi_ratio: h.vol_oi_ratio,
                                premium_paid: h.premium_paid,
                                fill_side: h.fill_side,
                                observed_at,
                            });
                        }
                    }
                    Err(e) => {
                        tracing::debug!(?e, symbol = %sym, "uoa_stream chain fetch failed");
                    }
                }
                tokio::time::sleep(Duration::from_millis(PACE_MS)).await;
            }
        }
    });
}

pub fn global() -> UoaStreamStore {
    static STORE: once_cell::sync::OnceCell<UoaStreamStore> = once_cell::sync::OnceCell::new();
    STORE
        .get_or_init(|| {
            let s = UoaStreamStore::new();
            spawn_poller(s.clone(), crate::live_ticks::global());
            s
        })
        .clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ev(sym: &str, strike: f64, ty: &str, premium: f64, age_secs: i64) -> UoaEvent {
        UoaEvent {
            symbol: sym.into(),
            expiry: "2026-06-19".into(),
            strike,
            option_type: ty.into(),
            volume: 1000.0,
            open_interest: 500.0,
            vol_oi_ratio: 2.0,
            premium_paid: premium,
            fill_side: FillSide::Midpoint,
            observed_at: Utc::now() - chrono::Duration::seconds(age_secs),
        }
    }

    #[test]
    fn first_observe_emits_dedupe_blocks_second() {
        let store = UoaStreamStore::new();
        let e = ev("AAPL", 200.0, "call", 1_000_000.0, 0);
        assert!(store.observe(e.clone()), "first observe should emit");
        assert!(!store.observe(e), "duplicate must dedupe");
        assert_eq!(store.emitted.len(), 1);
    }

    #[test]
    fn different_contracts_are_separate_hits() {
        let store = UoaStreamStore::new();
        assert!(store.observe(ev("AAPL", 200.0, "call", 1.0, 0)));
        assert!(store.observe(ev("AAPL", 200.0, "put", 1.0, 0)));
        assert!(store.observe(ev("AAPL", 210.0, "call", 1.0, 0)));
        assert!(store.observe(ev("MSFT", 200.0, "call", 1.0, 0)));
        assert_eq!(store.emitted.len(), 4);
    }

    #[test]
    fn latest_sorts_newest_first() {
        let store = UoaStreamStore::new();
        // Insert oldest first; latest() must reverse to newest-first.
        store.observe(ev("AAA", 100.0, "call", 1.0, 30));
        store.observe(ev("BBB", 100.0, "call", 1.0, 10));
        store.observe(ev("CCC", 100.0, "call", 1.0, 60));
        let l = store.latest(10);
        assert_eq!(l[0].symbol, "BBB");
        assert_eq!(l[1].symbol, "AAA");
        assert_eq!(l[2].symbol, "CCC");
    }

    #[test]
    fn latest_for_filters_by_symbol() {
        let store = UoaStreamStore::new();
        store.observe(ev("AAPL", 100.0, "call", 1.0, 0));
        store.observe(ev("AAPL", 110.0, "call", 1.0, 0));
        store.observe(ev("MSFT", 100.0, "call", 1.0, 0));
        let hits = store.latest_for("AAPL", 10);
        assert_eq!(hits.len(), 2);
        assert!(hits.iter().all(|h| h.symbol == "AAPL"));
    }

    #[test]
    fn evict_caps_at_max_entries() {
        let store = UoaStreamStore::new();
        for i in 0..2_500 {
            store.observe(ev(&format!("S{i:05}"), i as f64, "call", 1.0, i as i64));
        }
        assert!(store.emitted.len() <= EMITTED_CAP);
    }

    #[test]
    fn chain_to_contracts_handles_none_fields() {
        let chain = options::Chain {
            symbol: "TEST".into(),
            spot: 100.0,
            expirations: vec![],
            expiration: chrono::NaiveDate::from_ymd_opt(2026, 6, 19).unwrap(),
            calls: vec![options::OptionContract {
                strike: 100.0,
                bid: None,
                ask: None,
                last_price: None,
                implied_vol: None,
                volume: None,
                open_interest: None,
                in_the_money: false,
            }],
            puts: vec![],
        };
        let cs = chain_to_contracts(&chain);
        assert_eq!(cs.len(), 1);
        let c = &cs[0];
        assert_eq!(c.symbol, "TEST");
        assert_eq!(c.volume, 0.0);
        assert_eq!(c.open_interest, 0.0);
        assert_eq!(c.last_price, 0.0);
        // Default scanner config requires volume ≥ 500, so a fully-empty
        // contract must not be flagged as UOA.
        let hits = core_uoa::scan(&cs, &UoaConfig::default());
        assert!(hits.is_empty());
    }

    #[test]
    fn is_crypto_like_skips_btc_eth() {
        assert!(is_crypto_like("BTC/USD"));
        assert!(is_crypto_like("ETHUSD"));
        assert!(!is_crypto_like("AAPL"));
        assert!(!is_crypto_like("SPY"));
    }
}
