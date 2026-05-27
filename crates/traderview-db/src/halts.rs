//! Nasdaq trading-halt scanner.
//!
//! Polls Nasdaq Trader's public halts RSS every 3 seconds, parses each halt
//! into a structured row with the official reason code, and exposes:
//!   * `latest()` — most recent N halts (live state)
//!   * `subscribe()` — broadcast channel of new halts
//!
//! Nasdaq RSS shape (verified): `<item><title>`, `<description>` containing
//! `Symbol`, `Issue Name`, `Halt Time`, `Halt Reason`, `Resumption Date`,
//! `Resumption Quote Time`, `Resumption Trade Time` as `<br/>`-separated lines.
//!
//! Reason codes (official Nasdaq Trader documentation):
//!   T1  News Pending           T2  News Released
//!   T5  Single Stock Trading Pause / Volatility (5%+/-)
//!   T6  Halt - Extraordinary Market Activity
//!   T8  Halt ETF                T12 Trading For Additional Information
//!   H4  Halt Non Compliance     H9  Halt Filings Non Current
//!   H10 Halt SEC Trading Suspension     H11 Halt Regulatory Concern
//!   O1  Operations Halt, Contact Market Operations
//!   IPO IPO Issue Not Yet Trading       M1  Corporate Action
//!   M2  Quotation Not Available  LUDP  Volatility Trading Pause (LULD)
//!   LUDS  Volatility Trading Pause - Straddle (LULD)
//!   MWC1 Market Wide Circuit Breaker Halt Level 1
//!   MWC2 Level 2  MWC3 Level 3  MWCO Carry-over from previous day
//!   T3   News and Resumption    R4   Issue Available for Quotation
//!   R9   Issue Available for Trading  C3   Issue Cleared to Trade
//!   D    Deletion (delisting)   IPOQ Quotation Open (IPO)

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::Serialize;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;

pub const HALT_FEED_URL: &str =
    "https://www.nasdaqtrader.com/rss.aspx?feed=tradehalts";

#[derive(Debug, Clone, Serialize)]
pub struct Halt {
    pub symbol: String,
    pub issue_name: String,
    pub halt_date: String,
    pub halt_time: String,
    pub reason_code: String,
    pub reason_label: String,
    pub resumption_date: Option<String>,
    pub resumption_quote_time: Option<String>,
    pub resumption_trade_time: Option<String>,
    pub fetched_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct HaltStore {
    /// Symbol → latest halt for that symbol today.
    latest: Arc<DashMap<String, Halt>>,
    /// Broadcast of every newly-observed halt.
    tx: broadcast::Sender<Halt>,
}

impl HaltStore {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(256);
        Self {
            latest: Arc::new(DashMap::new()),
            tx,
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Halt> {
        self.tx.subscribe()
    }

    /// Snapshot the current halts, newest first.
    pub fn latest(&self, limit: usize) -> Vec<Halt> {
        let mut all: Vec<Halt> = self.latest.iter().map(|e| e.value().clone()).collect();
        all.sort_by_key(|a| std::cmp::Reverse(a.fetched_at));
        all.truncate(limit);
        all
    }

    fn observe(&self, h: Halt) {
        // Use the (symbol, halt_time, reason_code) tuple as dedupe key.
        let key = format!("{}|{}|{}", h.symbol, h.halt_time, h.reason_code);
        if !self.latest.contains_key(&key) {
            self.latest.insert(key, h.clone());
            // Best-effort broadcast; ignore lagging receivers.
            let _ = self.tx.send(h);
            self.evict_if_full();
        }
    }

    /// Bound the in-memory map so it doesn't grow forever across multi-day
    /// sessions. When over `MAX_ENTRIES`, evict the oldest 25% in a single
    /// O(N) pass — keeps the amortized insert cost flat.
    fn evict_if_full(&self) {
        const MAX_ENTRIES: usize = 2_000;
        const EVICT_FRACTION: usize = 4; // drop oldest 1/N
        if self.latest.len() <= MAX_ENTRIES { return; }
        let drop_n = self.latest.len() / EVICT_FRACTION;
        let mut by_age: Vec<(String, DateTime<Utc>)> = self.latest
            .iter()
            .map(|e| (e.key().clone(), e.value().fetched_at))
            .collect();
        by_age.sort_by_key(|(_, t)| *t);   // oldest first
        for (key, _) in by_age.into_iter().take(drop_n) {
            self.latest.remove(&key);
        }
    }
}

impl Default for HaltStore {
    fn default() -> Self { Self::new() }
}

/// Spawn the polling loop. Runs forever; cancel by dropping the store.
pub fn spawn_poller(store: HaltStore) {
    tokio::spawn(async move {
        let client = reqwest::Client::builder()
            .user_agent("traderview/0.1")
            .timeout(Duration::from_secs(10))
            .build()
            .expect("build reqwest client");
        let mut interval = tokio::time::interval(Duration::from_secs(3));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            interval.tick().await;
            match client.get(HALT_FEED_URL).send().await {
                Ok(resp) => match resp.text().await {
                    Ok(body) => {
                        let halts = parse_rss(&body);
                        for h in halts {
                            store.observe(h);
                        }
                    }
                    Err(e) => tracing::warn!(?e, "halt feed body read failed"),
                },
                Err(e) => tracing::warn!(?e, "halt feed fetch failed"),
            }
        }
    });
}

fn parse_rss(body: &str) -> Vec<Halt> {
    // Use quick-xml to walk items rather than regex. The RSS is small.
    use quick_xml::events::Event;
    use quick_xml::Reader;

    let mut reader = Reader::from_str(body);
    reader.config_mut().trim_text(true);

    let mut out = Vec::new();
    let mut buf = Vec::new();
    let mut in_item = false;
    let mut current_desc = String::new();
    let mut current_title = String::new();
    let mut current_tag: Option<String> = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "item" {
                    in_item = true;
                    current_desc.clear();
                    current_title.clear();
                }
                current_tag = Some(name);
            }
            Ok(Event::End(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "item" && in_item {
                    if let Some(h) = halt_from_description(&current_title, &current_desc) {
                        out.push(h);
                    }
                    in_item = false;
                }
                current_tag = None;
            }
            Ok(Event::Text(e)) if in_item => {
                let txt = e.unescape().unwrap_or_default().to_string();
                if let Some(tag) = &current_tag {
                    match tag.as_str() {
                        "title" => current_title.push_str(&txt),
                        "description" => current_desc.push_str(&txt),
                        _ => {}
                    }
                }
            }
            Ok(Event::CData(e)) if in_item => {
                let txt = String::from_utf8_lossy(&e).to_string();
                if let Some(tag) = &current_tag {
                    match tag.as_str() {
                        "title" => current_title.push_str(&txt),
                        "description" => current_desc.push_str(&txt),
                        _ => {}
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                tracing::warn!(?e, "halt RSS parse error");
                break;
            }
            _ => {}
        }
        buf.clear();
    }
    out
}

fn halt_from_description(_title: &str, desc: &str) -> Option<Halt> {
    // The description is a sequence of "Key: Value<br/>" pairs.
    let clean = desc.replace("<br/>", "\n").replace("<br />", "\n").replace("<BR>", "\n");
    let mut fields = std::collections::HashMap::<&str, String>::new();
    for line in clean.lines() {
        if let Some((k, v)) = line.split_once(':') {
            fields.insert(k.trim(), v.trim().to_string());
        }
    }
    let symbol = fields.get("Symbol").cloned()?;
    if symbol.is_empty() {
        return None;
    }
    let reason_code = fields.get("Reason Code").cloned().unwrap_or_default();
    Some(Halt {
        symbol,
        issue_name: fields.get("Issue Name").cloned().unwrap_or_default(),
        halt_date: fields.get("Halt Date").cloned().unwrap_or_default(),
        halt_time: fields.get("Halt Time").cloned().unwrap_or_default(),
        reason_label: reason_label(&reason_code).into(),
        reason_code,
        resumption_date: fields.get("Resumption Date").cloned().filter(|s| !s.is_empty()),
        resumption_quote_time: fields.get("Resumption Quote Time").cloned().filter(|s| !s.is_empty()),
        resumption_trade_time: fields.get("Resumption Trade Time").cloned().filter(|s| !s.is_empty()),
        fetched_at: Utc::now(),
    })
}

pub fn reason_label(code: &str) -> &'static str {
    match code {
        "T1"   => "News Pending",
        "T2"   => "News Released",
        "T3"   => "News and Resumption",
        "T5"   => "Single Stock Trading Pause (5%+/-)",
        "T6"   => "Extraordinary Market Activity",
        "T8"   => "Halt ETF",
        "T12"  => "Trading For Additional Information",
        "H4"   => "Halt Non Compliance",
        "H9"   => "Halt Filings Non Current",
        "H10"  => "Halt SEC Trading Suspension",
        "H11"  => "Halt Regulatory Concern",
        "O1"   => "Operations Halt",
        "IPO"  => "IPO Issue Not Yet Trading",
        "IPOQ" => "IPO Quotation Open",
        "M1"   => "Corporate Action",
        "M2"   => "Quotation Not Available",
        "LUDP" => "Volatility Pause (LULD)",
        "LUDS" => "Volatility Pause — Straddle (LULD)",
        "MWC1" => "Market Wide Circuit Breaker — Level 1",
        "MWC2" => "Market Wide Circuit Breaker — Level 2",
        "MWC3" => "Market Wide Circuit Breaker — Level 3",
        "MWCO" => "Market Wide Circuit Breaker — Carry-over",
        "R4"   => "Issue Available for Quotation",
        "R9"   => "Issue Available for Trading",
        "C3"   => "Issue Cleared to Trade",
        "D"    => "Deletion (Delisting)",
        _      => "Unknown Halt",
    }
}

/// Singleton store shared across the axum app.
pub fn global() -> HaltStore {
    static STORE: once_cell::sync::OnceCell<HaltStore> = once_cell::sync::OnceCell::new();
    STORE
        .get_or_init(|| {
            let s = HaltStore::new();
            spawn_poller(s.clone());
            s
        })
        .clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a Halt with a deterministic dedupe key. We embed `i` into
    /// `halt_time` so observe()'s `(symbol, halt_time, reason_code)` key is
    /// unique per call — without that, the dedupe in observe() short-circuits
    /// and we never hit the eviction path.
    fn halt(i: usize) -> Halt {
        Halt {
            symbol: format!("S{i:05}"),
            issue_name: format!("Issue {i}"),
            halt_date: "2026-05-27".into(),
            halt_time: format!("09:{:02}:{:02}", (i / 60) % 60, i % 60),
            reason_code: "T1".into(),
            reason_label: "News pending".into(),
            resumption_date: None,
            resumption_quote_time: None,
            resumption_trade_time: None,
            // Stagger fetched_at by `i` seconds so eviction has an ordering
            // signal: oldest = smallest i.
            fetched_at: Utc::now() + chrono::Duration::seconds(i as i64),
        }
    }

    #[test]
    fn observe_caps_at_max_entries() {
        let store = HaltStore::new();
        // Insert 500 over the cap and verify we never sit above MAX_ENTRIES.
        for i in 0..2_500 {
            store.observe(halt(i));
            assert!(
                store.latest.len() <= 2_000,
                "store grew past cap: {} after {} inserts", store.latest.len(), i + 1
            );
        }
        // After eviction the store should be at or below cap and well above
        // the post-eviction floor (cap - cap/EVICT_FRACTION = 1500).
        assert!(store.latest.len() <= 2_000);
        assert!(store.latest.len() >= 1_500,
            "post-eviction floor breached: {}", store.latest.len());
    }

    #[test]
    fn eviction_drops_oldest_first() {
        let store = HaltStore::new();
        // Insert exactly cap+1 to trigger one eviction pass.
        for i in 0..2_001 {
            store.observe(halt(i));
        }
        // The oldest (i=0..500) should be gone; the newest (i=2000) must survive.
        let oldest_key = format!("{}|{}|{}", halt(0).symbol, halt(0).halt_time, "T1");
        let newest_key = format!("{}|{}|{}", halt(2000).symbol, halt(2000).halt_time, "T1");
        assert!(!store.latest.contains_key(&oldest_key),
            "oldest entry survived eviction");
        assert!(store.latest.contains_key(&newest_key),
            "newest entry was incorrectly evicted");
    }

    #[test]
    fn observe_dedupes_identical_halts() {
        let store = HaltStore::new();
        let h = halt(42);
        store.observe(h.clone());
        store.observe(h.clone());
        store.observe(h);
        assert_eq!(store.latest.len(), 1, "duplicate halts were not deduped");
    }

    #[test]
    fn latest_returns_newest_first() {
        let store = HaltStore::new();
        for i in 0..10 {
            store.observe(halt(i));
        }
        let v = store.latest(3);
        assert_eq!(v.len(), 3);
        // halt(i) has fetched_at = now + i seconds, so i=9 is newest.
        assert_eq!(v[0].symbol, "S00009");
        assert_eq!(v[1].symbol, "S00008");
        assert_eq!(v[2].symbol, "S00007");
    }
}
