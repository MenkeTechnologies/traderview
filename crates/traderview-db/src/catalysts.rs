//! Catalyst firehose — SEC EDGAR latest-filings Atom feed + major PR wires
//! (Business Wire, PR Newswire, GlobeNewswire, AccessWire), with ticker NER.
//!
//! Both pollers push into a unified `CatalystStore` (DashMap dedupe +
//! broadcast channel) so the frontend can subscribe to one WebSocket and
//! receive every fresh filing or press release within seconds.
//!
//! `extract_tickers` runs against the headline + summary and returns any
//! `$TICKER` cashtags plus 1-5-letter all-caps words that look like tickers
//! (filtered through a stoplist of common English words).

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::Serialize;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;

pub const EDGAR_LATEST: &str =
    "https://www.sec.gov/cgi-bin/browse-edgar?action=getcurrent&type=&company=&dateb=&owner=include&count=40&output=atom";

pub const PR_WIRES: &[(&str, &str)] = &[
    ("Business Wire",   "https://feed.businesswire.com/rss/home/?rss=G1QFDERJXkJeGVtRVA=="),
    ("PR Newswire",     "https://www.prnewswire.com/rss/news-releases-list.rss"),
    ("GlobeNewswire",   "https://www.globenewswire.com/RssFeed/orgclass/1/feedTitle/GlobeNewswire%20-%20News%20about%20Public%20Companies"),
    ("AccessWire",      "https://www.accesswire.com/api/rss.ashx"),
];

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CatalystKind {
    SecFiling,
    PressRelease,
}

#[derive(Debug, Clone, Serialize)]
pub struct Catalyst {
    pub kind: CatalystKind,
    pub source: String,           // "EDGAR" | wire name
    pub form_type: Option<String>, // "8-K", "13D", "S-1", etc.
    pub title: String,
    pub summary: String,
    pub link: Option<String>,
    pub published_at: DateTime<Utc>,
    pub fetched_at: DateTime<Utc>,
    pub tickers: Vec<String>,
}

#[derive(Clone)]
pub struct CatalystStore {
    seen: Arc<DashMap<String, Catalyst>>, // dedupe key → catalyst
    tx: broadcast::Sender<Catalyst>,
}

impl CatalystStore {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(2048);
        Self {
            seen: Arc::new(DashMap::new()),
            tx,
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Catalyst> { self.tx.subscribe() }

    /// Newest first.
    pub fn latest(&self, limit: usize) -> Vec<Catalyst> {
        let mut all: Vec<Catalyst> = self.seen.iter().map(|e| e.value().clone()).collect();
        all.sort_by(|a, b| b.fetched_at.cmp(&a.fetched_at));
        all.truncate(limit);
        all
    }

    /// Latest catalysts filtered to a single symbol.
    pub fn latest_for(&self, symbol: &str, limit: usize) -> Vec<Catalyst> {
        let sym_upper = symbol.to_ascii_uppercase();
        let mut hits: Vec<Catalyst> = self
            .seen
            .iter()
            .filter(|e| e.value().tickers.iter().any(|t| t == &sym_upper))
            .map(|e| e.value().clone())
            .collect();
        hits.sort_by(|a, b| b.fetched_at.cmp(&a.fetched_at));
        hits.truncate(limit);
        hits
    }

    fn observe(&self, key: String, c: Catalyst) {
        if !self.seen.contains_key(&key) {
            self.seen.insert(key, c.clone());
            let _ = self.tx.send(c);
        }
    }
}

impl Default for CatalystStore { fn default() -> Self { Self::new() } }

pub fn global() -> CatalystStore {
    static STORE: once_cell::sync::OnceCell<CatalystStore> = once_cell::sync::OnceCell::new();
    STORE
        .get_or_init(|| {
            let s = CatalystStore::new();
            spawn_pollers(s.clone());
            s
        })
        .clone()
}

// ===========================================================================
// Pollers
// ===========================================================================

fn spawn_pollers(store: CatalystStore) {
    // EDGAR every 6s — they request you not hit it faster.
    let s = store.clone();
    tokio::spawn(async move { run_poller(s, 6, fetch_edgar).await; });
    // PR wires every 30s — they're rate-limited too.
    for (name, url) in PR_WIRES {
        let s = store.clone();
        let name = (*name).to_string();
        let url = (*url).to_string();
        tokio::spawn(async move {
            run_poller(s, 30, move |store| {
                let name = name.clone();
                let url  = url.clone();
                async move { fetch_rss(&name, &url, &store).await }
            }).await;
        });
    }
}

async fn run_poller<F, Fut>(store: CatalystStore, interval_secs: u64, mut f: F)
where
    F: FnMut(CatalystStore) -> Fut + Send + 'static,
    Fut: std::future::Future<Output = ()> + Send,
{
    let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        interval.tick().await;
        f(store.clone()).await;
    }
}

fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent("traderview/0.1 (catalyst-radar; ops@menketechnologies.com)")
        .timeout(Duration::from_secs(10))
        .build()
        .expect("reqwest client")
}

async fn fetch_edgar(store: CatalystStore) {
    let client = http_client();
    let resp = match client.get(EDGAR_LATEST).send().await {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!(?e, "EDGAR fetch failed");
            return;
        }
    };
    if !resp.status().is_success() {
        tracing::warn!(status = ?resp.status(), "EDGAR HTTP error");
        return;
    }
    let body = match resp.text().await { Ok(b) => b, Err(_) => return };
    for c in parse_atom(&body, "EDGAR", CatalystKind::SecFiling) {
        let key = format!("edgar|{}", c.link.clone().unwrap_or_else(|| c.title.clone()));
        store.observe(key, c);
    }
}

async fn fetch_rss(name: &str, url: &str, store: &CatalystStore) {
    let client = http_client();
    let resp = match client.get(url).send().await {
        Ok(r) => r,
        Err(e) => { tracing::warn!(?e, source=name, "PR fetch failed"); return; }
    };
    if !resp.status().is_success() {
        tracing::warn!(status=?resp.status(), source=name, "PR HTTP error");
        return;
    }
    let body = match resp.text().await { Ok(b) => b, Err(_) => return };
    for c in parse_rss_or_atom(&body, name, CatalystKind::PressRelease) {
        let key = format!("{name}|{}", c.link.clone().unwrap_or_else(|| c.title.clone()));
        store.observe(key, c);
    }
}

// ===========================================================================
// Atom / RSS parsing (lenient — works for both)
// ===========================================================================

fn parse_atom(body: &str, source: &str, kind: CatalystKind) -> Vec<Catalyst> {
    parse_feed(body, source, kind, /*entry=*/"entry", /*summary=*/"summary")
}

fn parse_rss_or_atom(body: &str, source: &str, kind: CatalystKind) -> Vec<Catalyst> {
    // Quick heuristic: <entry> => Atom; <item> => RSS.
    if body.contains("<entry") {
        parse_atom(body, source, kind)
    } else {
        parse_feed(body, source, kind, "item", "description")
    }
}

fn parse_feed(
    body: &str,
    source: &str,
    kind: CatalystKind,
    entry_tag: &str,
    summary_tag: &str,
) -> Vec<Catalyst> {
    use quick_xml::events::Event;
    use quick_xml::Reader;
    let mut reader = Reader::from_str(body);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();

    let mut out = Vec::new();
    let mut in_entry = false;
    let mut current_tag: Option<String> = None;
    let mut current_attr_href: Option<String> = None;
    let mut title = String::new();
    let mut summary = String::new();
    let mut link = String::new();
    let mut published = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == entry_tag {
                    in_entry = true;
                    title.clear(); summary.clear(); link.clear(); published.clear();
                }
                current_tag = Some(name.clone());
                if in_entry && name == "link" {
                    // Atom uses <link href="..."/>; capture the attribute.
                    for a in e.attributes().with_checks(false).flatten() {
                        if a.key.as_ref() == b"href" {
                            current_attr_href = Some(String::from_utf8_lossy(&a.value).to_string());
                        }
                    }
                }
            }
            Ok(Event::Empty(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if in_entry && name == "link" {
                    for a in e.attributes().with_checks(false).flatten() {
                        if a.key.as_ref() == b"href" {
                            link = String::from_utf8_lossy(&a.value).to_string();
                        }
                    }
                }
            }
            Ok(Event::End(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == entry_tag && in_entry {
                    if !title.is_empty() {
                        let mut form_type: Option<String> = None;
                        if source == "EDGAR" {
                            // EDGAR title looks like: "8-K - ACME CORP (0000012345) (Filer)"
                            if let Some(dash) = title.find('-') {
                                let t = title[..dash].trim().to_string();
                                if !t.is_empty() { form_type = Some(t); }
                            }
                        }
                        let body_text = format!("{title} {summary}");
                        let tickers = extract_tickers(&body_text);
                        let final_link = if !link.is_empty() {
                            Some(link.clone())
                        } else if let Some(h) = current_attr_href.take() {
                            Some(h)
                        } else { None };
                        let pub_at = parse_pub_date(&published).unwrap_or_else(Utc::now);
                        out.push(Catalyst {
                            kind,
                            source: source.into(),
                            form_type,
                            title: title.clone(),
                            summary: summary.clone(),
                            link: final_link,
                            published_at: pub_at,
                            fetched_at: Utc::now(),
                            tickers,
                        });
                    }
                    in_entry = false;
                }
                current_tag = None;
            }
            Ok(Event::Text(e)) => {
                if in_entry {
                    let txt = e.unescape().unwrap_or_default().to_string();
                    match current_tag.as_deref() {
                        Some("title") => title.push_str(&txt),
                        s if s == Some(summary_tag) => summary.push_str(&txt),
                        Some("link") => if link.is_empty() { link.push_str(&txt); },
                        Some("updated") | Some("published") | Some("pubDate") => {
                            published.push_str(&txt);
                        }
                        _ => {}
                    }
                }
            }
            Ok(Event::CData(e)) => {
                if in_entry {
                    let txt = String::from_utf8_lossy(&e).to_string();
                    match current_tag.as_deref() {
                        Some("title") => title.push_str(&txt),
                        s if s == Some(summary_tag) => summary.push_str(&txt),
                        _ => {}
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => { tracing::warn!(?e, "feed parse error"); break; }
            _ => {}
        }
        buf.clear();
    }
    out
}

fn parse_pub_date(s: &str) -> Option<DateTime<Utc>> {
    if s.is_empty() { return None; }
    if let Ok(d) = chrono::DateTime::parse_from_rfc2822(s) {
        return Some(d.with_timezone(&Utc));
    }
    if let Ok(d) = chrono::DateTime::parse_from_rfc3339(s) {
        return Some(d.with_timezone(&Utc));
    }
    None
}

// ===========================================================================
// Ticker NER — naive but effective for retail squeeze plays
// ===========================================================================

pub fn extract_tickers(text: &str) -> Vec<String> {
    let mut out = std::collections::BTreeSet::<String>::new();
    // Pass 1: $TICKER cashtags — unambiguous.
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '$' {
            let mut s = String::new();
            while let Some(&p) = chars.peek() {
                if p.is_ascii_uppercase() || p.is_ascii_digit() || p == '.' || p == '-' {
                    s.push(p);
                    chars.next();
                } else { break; }
            }
            if (1..=8).contains(&s.len()) {
                out.insert(s);
            }
        }
    }
    // Pass 2: parenthetical (TICKER) — common in PR headlines.
    let mut in_paren = false;
    let mut buf = String::new();
    for c in text.chars() {
        match c {
            '(' => { in_paren = true; buf.clear(); }
            ')' => {
                if in_paren && (1..=8).contains(&buf.len())
                    && buf.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '.' || c == '-')
                    && !STOP_WORDS.contains(&buf.as_str())
                {
                    out.insert(std::mem::take(&mut buf));
                }
                in_paren = false; buf.clear();
            }
            _ if in_paren => buf.push(c),
            _ => {}
        }
    }
    // Pass 3: NYSE:XXX / NASDAQ:XXX format.
    for prefix in ["NYSE:", "NASDAQ:", "AMEX:", "OTC:"] {
        let mut s = text;
        while let Some(i) = s.find(prefix) {
            let rest = &s[i + prefix.len()..];
            let tk: String = rest
                .chars()
                .take_while(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || *c == '.')
                .collect();
            if (1..=8).contains(&tk.len()) {
                out.insert(tk);
            }
            s = &rest[..];
        }
    }
    out.into_iter().collect()
}

const STOP_WORDS: &[&str] = &[
    "I", "A", "AT", "BE", "DO", "GO", "IF", "IN", "IS", "IT", "MY", "NO", "OF", "ON", "OR",
    "SO", "TO", "UP", "US", "WE", "AM", "AN", "AS", "BY", "HE", "HI",
    "AND", "ARE", "BUT", "FOR", "GET", "HAS", "HER", "HIM", "HIS", "HOW", "NOT", "NOW",
    "OUR", "OUT", "PUT", "SHE", "THE", "TOP", "TWO", "WAS", "WHO", "WHY", "YOU",
    "ABOUT", "AFTER", "AGAIN", "ALSO", "BEFORE", "BEEN", "BEING", "BELOW", "BOTH",
    "CEO", "CFO", "COO", "CIO", "CTO", "CMO", "CRO",
    "USA", "UK", "EU", "EUR", "GBP", "JPY", "CAD", "USD", "AUD", "CHF",
    "AI", "API", "ETF", "IPO", "LLC", "INC", "LTD", "COM", "CORP", "PLC", "AG", "SA",
    "Q1", "Q2", "Q3", "Q4", "YOY", "QOQ", "FY", "TBD", "PR", "FYI", "ESG",
    "NEWS", "BREAKING",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cashtag_extraction() {
        let t = "$AAPL and $TSLA spike on news; also $SPCE";
        let mut v = extract_tickers(t);
        v.sort();
        assert_eq!(v, vec!["AAPL", "SPCE", "TSLA"]);
    }

    #[test]
    fn parenthetical_extraction() {
        let t = "Acme Corp (ACME) reported strong Q4 earnings (CEO statement)";
        let v = extract_tickers(t);
        assert!(v.contains(&"ACME".to_string()));
        assert!(!v.contains(&"CEO".to_string())); // stop-listed
    }

    #[test]
    fn exchange_prefix_extraction() {
        let t = "NASDAQ:NVDA jumped after the announcement";
        let v = extract_tickers(t);
        assert!(v.contains(&"NVDA".to_string()));
    }
}
