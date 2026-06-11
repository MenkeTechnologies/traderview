//! Real-time insider Form 4 stream.
//!
//! Edge case in the academic literature that consistently shows up:
//! cluster insider buys (multiple insiders, often officers + directors,
//! buying open-market within the same window) beat the market by
//! ~5-10% per year over the next 6-12 months (Lakonishok & Lee 2001,
//! Cohen Malloy Pomorski 2012). Most retail traders don't see these
//! because the data is fragmented across SEC Form 4 filings.
//!
//! Pipeline:
//!
//!   1. Subscribe to `catalysts::CatalystStore::subscribe()` and
//!      filter to `form_type == "4"`. The catalyst aggregator
//!      already pulls EDGAR Atom every 6s.
//!   2. For each Form 4 catalyst, fetch the filing's index page,
//!      scrape the primary `.xml` filename, then fetch + parse the
//!      ownership XML. Pacing: 1 fetch/sec to stay well under SEC's
//!      published 10 req/sec ceiling.
//!   3. Extract: issuer ticker, insider name, officer-or-director
//!      flag, officer title, transaction code (P/S/A/G/...), shares,
//!      price-per-share, computed dollar value.
//!   4. Broadcast `InsiderEvent` rows; cap emitted set at 4000 with
//!      oldest-first eviction.
//!
//! Transaction code reference (SEC Form 4 codings):
//!   P  Open-market purchase            S  Open-market sale
//!   A  Grant / award                   M  Option exercise
//!   F  Tax withholding                 G  Bona-fide gift
//!   J  Other                           D  Disposition
//!
//! Only `P` (buys) and `S` (sales) are reliably tradeable signals —
//! grants/option-exercises/tax-withholds get a separate `kind` tag so
//! the UI can filter them out without losing them entirely.

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::Serialize;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;

use crate::catalysts::{Catalyst, CatalystKind, CatalystStore};

const PACE_MS: u64 = 1_000;
const EMITTED_CAP: usize = 4_000;
const UA: &str = "traderview/0.1 contact: github.com/MenkeTechnologies/traderview";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TxKind {
    Buy,
    Sell,
    Grant,
    OptionExercise,
    TaxWithhold,
    Gift,
    Other,
}

impl TxKind {
    fn from_code(c: &str) -> TxKind {
        match c.trim().to_ascii_uppercase().as_str() {
            "P" => TxKind::Buy,
            "S" | "D" => TxKind::Sell,
            "A" => TxKind::Grant,
            "M" => TxKind::OptionExercise,
            "F" => TxKind::TaxWithhold,
            "G" => TxKind::Gift,
            _ => TxKind::Other,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            TxKind::Buy => "buy",
            TxKind::Sell => "sell",
            TxKind::Grant => "grant",
            TxKind::OptionExercise => "option_exercise",
            TxKind::TaxWithhold => "tax_withhold",
            TxKind::Gift => "gift",
            TxKind::Other => "other",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct InsiderEvent {
    pub symbol: String,
    pub insider_name: String,
    pub is_officer: bool,
    pub is_director: bool,
    pub is_ten_percent_owner: bool,
    pub officer_title: Option<String>,
    pub transaction_code: String,
    pub kind: TxKind,
    pub shares: f64,
    pub price_per_share: f64,
    pub dollar_value: f64,
    pub transaction_date: Option<String>,
    pub filing_link: Option<String>,
    pub observed_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct InsiderStore {
    emitted: Arc<DashMap<String, InsiderEvent>>,
    tx: broadcast::Sender<InsiderEvent>,
}

impl InsiderStore {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(256);
        Self {
            emitted: Arc::new(DashMap::new()),
            tx,
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<InsiderEvent> {
        self.tx.subscribe()
    }

    pub fn latest(&self, limit: usize) -> Vec<InsiderEvent> {
        let mut all: Vec<InsiderEvent> = self.emitted.iter().map(|e| e.value().clone()).collect();
        all.sort_by_key(|e| std::cmp::Reverse(e.observed_at));
        all.truncate(limit);
        all
    }

    pub fn latest_for(&self, symbol: &str, limit: usize) -> Vec<InsiderEvent> {
        let sym_upper = symbol.to_ascii_uppercase();
        let mut hits: Vec<InsiderEvent> = self
            .emitted
            .iter()
            .filter(|e| e.value().symbol == sym_upper)
            .map(|e| e.value().clone())
            .collect();
        hits.sort_by_key(|e| std::cmp::Reverse(e.observed_at));
        hits.truncate(limit);
        hits
    }

    /// Top insider buys in the trailing `days` days, ranked by sum of
    /// dollar_value across all Buy transactions per symbol. Used by the
    /// UI's "cluster buys" panel.
    pub fn top_buys(&self, days: i64, limit: usize) -> Vec<(String, f64, usize)> {
        let cutoff = Utc::now() - chrono::Duration::days(days);
        let mut by_symbol: std::collections::HashMap<String, (f64, usize)> =
            std::collections::HashMap::new();
        for e in self.emitted.iter() {
            let v = e.value();
            if v.kind == TxKind::Buy && v.observed_at >= cutoff {
                let entry = by_symbol.entry(v.symbol.clone()).or_insert((0.0, 0));
                entry.0 += v.dollar_value;
                entry.1 += 1;
            }
        }
        let mut rows: Vec<(String, f64, usize)> =
            by_symbol.into_iter().map(|(s, (v, n))| (s, v, n)).collect();
        rows.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        rows.truncate(limit);
        rows
    }

    fn observe(&self, ev: InsiderEvent) -> bool {
        // Dedup key: (symbol, insider_name, transaction_date, txcode, shares).
        // Same insider filing the same transaction shouldn't fire twice
        // if the catalyst feed re-presents the same Form 4.
        let key = format!(
            "{}|{}|{}|{}|{}",
            ev.symbol,
            ev.insider_name,
            ev.transaction_date.as_deref().unwrap_or("-"),
            ev.transaction_code,
            ev.shares
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

impl Default for InsiderStore {
    fn default() -> Self {
        Self::new()
    }
}

// ─── EDGAR XML fetch + parse ───────────────────────────────────────────────

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent(UA)
        .timeout(Duration::from_secs(15))
        .build()
        .expect("reqwest client")
}

/// Given an EDGAR filing index page URL (`/Archives/edgar/data/<cik>/<accession>/-index.htm`),
/// fetch it and find the primary Form 4 XML file. Returns the full URL
/// to the XML, or None if not findable.
async fn find_form4_xml(index_url: &str) -> Option<String> {
    let body = client()
        .get(index_url)
        .send()
        .await
        .ok()?
        .text()
        .await
        .ok()?;
    // The index page lists every file in the filing. Form 4 primary
    // doc is conventionally a `.xml` whose href looks like
    // `/Archives/edgar/data/.../wf-form4_xxxxxx.xml` or similar.
    extract_form4_xml_href(&body, index_url)
}

/// Pure helper: scan a page body for the first `.xml` href that looks
/// like a Form 4 primary doc (filename contains "form4" or "F345").
/// Resolves relative paths against the index URL.
pub fn extract_form4_xml_href(body: &str, index_url: &str) -> Option<String> {
    // Cheap href scan — full HTML parsing would be overkill.
    let lower = body.to_ascii_lowercase();
    let mut search_from = 0usize;
    while let Some(hpos) = lower[search_from..].find("href=") {
        let start = search_from + hpos + 5;
        let bytes = body.as_bytes();
        if start >= bytes.len() {
            break;
        }
        let quote = bytes[start];
        if quote != b'"' && quote != b'\'' {
            search_from = start + 1;
            continue;
        }
        let end = body[start + 1..].find(quote as char)?;
        let href = &body[start + 1..start + 1 + end];
        let h_lc = href.to_ascii_lowercase();
        let looks_like_form4 = h_lc.ends_with(".xml")
            && (h_lc.contains("form4") || h_lc.contains("f345") || h_lc.contains("primary_doc"));
        if looks_like_form4 {
            return Some(absolutize_href(index_url, href));
        }
        search_from = start + 1 + end;
    }
    None
}

/// Shared with thirteen_f (13F infotable discovery uses the same
/// EDGAR index-page convention).
pub(crate) fn absolutize_href(base: &str, href: &str) -> String {
    if href.starts_with("http") {
        return href.to_string();
    }
    if href.starts_with('/') {
        // Resolve against the scheme + host of base.
        if let Some(scheme_end) = base.find("://") {
            if let Some(host_end) = base[scheme_end + 3..].find('/') {
                return format!("{}{}", &base[..scheme_end + 3 + host_end], href);
            }
        }
        return format!("https://www.sec.gov{href}");
    }
    // Relative path — chop the last segment off the base and append.
    let parent = match base.rfind('/') {
        Some(i) => &base[..=i],
        None => "",
    };
    format!("{parent}{href}")
}

/// Parsed Form 4 XML — just the fields we care about.
#[derive(Debug, Default)]
pub struct ParsedForm4 {
    pub issuer_symbol: Option<String>,
    pub insider_name: Option<String>,
    pub is_officer: bool,
    pub is_director: bool,
    pub is_ten_percent_owner: bool,
    pub officer_title: Option<String>,
    pub transactions: Vec<ParsedTransaction>,
}

#[derive(Debug, Clone)]
pub struct ParsedTransaction {
    pub transaction_date: Option<String>,
    pub transaction_code: String,
    pub shares: f64,
    pub price_per_share: f64,
}

/// Pure XML parser for a Form 4 ownershipDocument. Handles both
/// nonDerivativeTable + derivativeTable (we capture both — derivative
/// transactions are usually option exercises). Missing optional fields
/// default to None / 0.0 rather than failing.
pub fn parse_form4(xml: &str) -> ParsedForm4 {
    use quick_xml::events::Event;
    use quick_xml::Reader;
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut out = ParsedForm4::default();
    let mut path: Vec<String> = Vec::new();
    let mut current_value = String::new();
    let mut current_tx: Option<ParsedTransaction> = None;
    let mut in_value_node = false;
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                path.push(name.clone());
                if matches!(
                    name.as_str(),
                    "nonDerivativeTransaction" | "derivativeTransaction"
                ) {
                    current_tx = Some(ParsedTransaction {
                        transaction_date: None,
                        transaction_code: String::new(),
                        shares: 0.0,
                        price_per_share: 0.0,
                    });
                }
                if name == "value" {
                    in_value_node = true;
                    current_value.clear();
                }
            }
            Ok(Event::Text(e)) => {
                let txt = e.unescape().unwrap_or_default().to_string();
                if in_value_node {
                    current_value.push_str(&txt);
                } else if let Some(parent) = path.last() {
                    let trimmed = txt.trim();
                    if trimmed.is_empty() {
                        // skip
                    } else {
                        match parent.as_str() {
                            // Some Form 4s put values directly in
                            // the leaf node without a <value> child.
                            "officerTitle" => {
                                out.officer_title = Some(trimmed.to_string());
                            }
                            "transactionCode" => {
                                if let Some(tx) = &mut current_tx {
                                    tx.transaction_code = trimmed.to_string();
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            Ok(Event::End(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "value" && in_value_node {
                    in_value_node = false;
                    let v = current_value.trim().to_string();
                    // Decide where to file this value based on the
                    // 2nd-most-recent element in the path.
                    // path = [..., parent, "value"]
                    let parent = path.get(path.len().saturating_sub(2)).cloned();
                    if let Some(parent) = parent {
                        match parent.as_str() {
                            "issuerTradingSymbol" => {
                                if !v.is_empty() {
                                    out.issuer_symbol = Some(v.clone());
                                }
                            }
                            "rptOwnerName" => {
                                out.insider_name = Some(v.clone());
                            }
                            "officerTitle" => {
                                out.officer_title = Some(v.clone());
                            }
                            "isOfficer" => {
                                out.is_officer = v == "1" || v.eq_ignore_ascii_case("true");
                            }
                            "isDirector" => {
                                out.is_director = v == "1" || v.eq_ignore_ascii_case("true");
                            }
                            "isTenPercentOwner" => {
                                out.is_ten_percent_owner =
                                    v == "1" || v.eq_ignore_ascii_case("true");
                            }
                            "transactionDate" => {
                                if let Some(tx) = &mut current_tx {
                                    tx.transaction_date = Some(v.clone());
                                }
                            }
                            "transactionCode" => {
                                if let Some(tx) = &mut current_tx {
                                    tx.transaction_code = v.clone();
                                }
                            }
                            "transactionShares" => {
                                if let Some(tx) = &mut current_tx {
                                    tx.shares = v.parse().unwrap_or(0.0);
                                }
                            }
                            "transactionPricePerShare" => {
                                if let Some(tx) = &mut current_tx {
                                    tx.price_per_share = v.parse().unwrap_or(0.0);
                                }
                            }
                            _ => {}
                        }
                    }
                    current_value.clear();
                }
                if matches!(
                    name.as_str(),
                    "nonDerivativeTransaction" | "derivativeTransaction"
                ) {
                    if let Some(tx) = current_tx.take() {
                        if !tx.transaction_code.is_empty() {
                            out.transactions.push(tx);
                        }
                    }
                }
                path.pop();
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    out
}

/// Convert one parsed Form 4 into per-transaction InsiderEvent rows.
pub fn to_events(parsed: &ParsedForm4, filing_link: Option<String>) -> Vec<InsiderEvent> {
    let symbol = match parsed.issuer_symbol.as_deref() {
        Some(s) if !s.is_empty() => s.to_ascii_uppercase(),
        _ => return Vec::new(),
    };
    let insider_name = parsed
        .insider_name
        .clone()
        .unwrap_or_else(|| "(unknown)".into());
    let now = Utc::now();
    parsed
        .transactions
        .iter()
        .filter(|t| t.shares > 0.0)
        .map(|t| {
            let kind = TxKind::from_code(&t.transaction_code);
            InsiderEvent {
                symbol: symbol.clone(),
                insider_name: insider_name.clone(),
                is_officer: parsed.is_officer,
                is_director: parsed.is_director,
                is_ten_percent_owner: parsed.is_ten_percent_owner,
                officer_title: parsed.officer_title.clone(),
                transaction_code: t.transaction_code.clone(),
                kind,
                shares: t.shares,
                price_per_share: t.price_per_share,
                dollar_value: t.shares * t.price_per_share,
                transaction_date: t.transaction_date.clone(),
                filing_link: filing_link.clone(),
                observed_at: now,
            }
        })
        .collect()
}

// ─── Background consumer ───────────────────────────────────────────────────

pub fn spawn_consumer(store: InsiderStore, catalysts: CatalystStore) {
    tokio::spawn(async move {
        loop {
            let mut rx = catalysts.subscribe();
            loop {
                match rx.recv().await {
                    Ok(cat) => {
                        if !matches!(cat.kind, CatalystKind::SecFiling) {
                            continue;
                        }
                        if cat.form_type.as_deref() != Some("4") {
                            continue;
                        }
                        process_catalyst(&store, &cat).await;
                        // Pace EDGAR fetches.
                        tokio::time::sleep(Duration::from_millis(PACE_MS)).await;
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                        tracing::warn!(skipped, "insider_stream lagged catalyst stream");
                        continue;
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
            }
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    });
}

async fn process_catalyst(store: &InsiderStore, cat: &Catalyst) {
    let Some(index_url) = cat.link.as_ref() else {
        return;
    };
    let Some(xml_url) = find_form4_xml(index_url).await else {
        tracing::debug!(link = %index_url, "no Form 4 XML link found");
        return;
    };
    let Ok(resp) = client().get(&xml_url).send().await else {
        return;
    };
    if !resp.status().is_success() {
        return;
    }
    let Ok(body) = resp.text().await else {
        return;
    };
    let parsed = parse_form4(&body);
    let events = to_events(&parsed, Some(xml_url.clone()));
    for ev in events {
        store.observe(ev);
    }
}

pub fn global() -> InsiderStore {
    static STORE: once_cell::sync::OnceCell<InsiderStore> = once_cell::sync::OnceCell::new();
    STORE
        .get_or_init(|| {
            let s = InsiderStore::new();
            spawn_consumer(s.clone(), crate::catalysts::global());
            s
        })
        .clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tx_kind_classification() {
        assert_eq!(TxKind::from_code("P"), TxKind::Buy);
        assert_eq!(TxKind::from_code("p"), TxKind::Buy);
        assert_eq!(TxKind::from_code("S"), TxKind::Sell);
        assert_eq!(TxKind::from_code("A"), TxKind::Grant);
        assert_eq!(TxKind::from_code("M"), TxKind::OptionExercise);
        assert_eq!(TxKind::from_code("F"), TxKind::TaxWithhold);
        assert_eq!(TxKind::from_code("G"), TxKind::Gift);
        assert_eq!(TxKind::from_code("X"), TxKind::Other);
    }

    #[test]
    fn extract_form4_xml_href_finds_link_in_index_page() {
        let body = r#"
            <html><body>
            <a href="acme-2026-balance.htm">balance</a>
            <a href="wf-form4_1738000000.xml">primary doc</a>
            <a href="exhibit99.pdf">exhibit</a>
            </body></html>
        "#;
        let base = "https://www.sec.gov/Archives/edgar/data/12345/000123456726000123/-index.htm";
        let url = extract_form4_xml_href(body, base).expect("should find form4 xml");
        assert!(url.ends_with("/wf-form4_1738000000.xml"));
        assert!(url.starts_with("https://www.sec.gov/"));
    }

    #[test]
    fn extract_form4_xml_href_handles_absolute_href() {
        let body = r#"<a href="https://www.sec.gov/Archives/edgar/data/1/form4_x.xml">x</a>"#;
        let url = extract_form4_xml_href(body, "https://example.com/").expect("absolute");
        assert_eq!(url, "https://www.sec.gov/Archives/edgar/data/1/form4_x.xml");
    }

    #[test]
    fn extract_form4_xml_href_handles_root_relative() {
        let body = r#"<a href="/Archives/edgar/data/1/form4_y.xml">y</a>"#;
        let url = extract_form4_xml_href(body, "https://www.sec.gov/some/page.htm")
            .expect("root-relative");
        assert_eq!(url, "https://www.sec.gov/Archives/edgar/data/1/form4_y.xml");
    }

    #[test]
    fn extract_form4_xml_href_returns_none_when_no_xml() {
        let body = r#"<a href="acme.htm">no xml here</a>"#;
        assert!(extract_form4_xml_href(body, "https://www.sec.gov/x/y.htm").is_none());
    }

    #[test]
    fn parse_form4_extracts_issuer_insider_and_transactions() {
        // Minimal but realistic Form 4 XML.
        let xml = r#"<?xml version="1.0"?>
        <ownershipDocument>
            <issuer>
                <issuerTradingSymbol><value>ACME</value></issuerTradingSymbol>
            </issuer>
            <reportingOwner>
                <reportingOwnerId>
                    <rptOwnerName><value>DOE, JANE</value></rptOwnerName>
                </reportingOwnerId>
                <reportingOwnerRelationship>
                    <isOfficer><value>1</value></isOfficer>
                    <isDirector><value>0</value></isDirector>
                    <isTenPercentOwner><value>0</value></isTenPercentOwner>
                    <officerTitle><value>Chief Executive Officer</value></officerTitle>
                </reportingOwnerRelationship>
            </reportingOwner>
            <nonDerivativeTable>
                <nonDerivativeTransaction>
                    <transactionDate><value>2026-01-15</value></transactionDate>
                    <transactionCoding>
                        <transactionCode><value>P</value></transactionCode>
                    </transactionCoding>
                    <transactionAmounts>
                        <transactionShares><value>10000</value></transactionShares>
                        <transactionPricePerShare><value>50.50</value></transactionPricePerShare>
                    </transactionAmounts>
                </nonDerivativeTransaction>
            </nonDerivativeTable>
        </ownershipDocument>"#;
        let p = parse_form4(xml);
        assert_eq!(p.issuer_symbol.as_deref(), Some("ACME"));
        assert_eq!(p.insider_name.as_deref(), Some("DOE, JANE"));
        assert!(p.is_officer);
        assert!(!p.is_director);
        assert_eq!(p.officer_title.as_deref(), Some("Chief Executive Officer"));
        assert_eq!(p.transactions.len(), 1);
        let t = &p.transactions[0];
        assert_eq!(t.transaction_code, "P");
        assert_eq!(t.shares, 10000.0);
        assert_eq!(t.price_per_share, 50.50);
        assert_eq!(t.transaction_date.as_deref(), Some("2026-01-15"));
    }

    #[test]
    fn parse_form4_handles_multiple_transactions() {
        let xml = r#"<?xml version="1.0"?>
        <ownershipDocument>
            <issuer><issuerTradingSymbol><value>BBB</value></issuerTradingSymbol></issuer>
            <reportingOwner>
                <reportingOwnerId><rptOwnerName><value>SMITH JOHN</value></rptOwnerName></reportingOwnerId>
            </reportingOwner>
            <nonDerivativeTable>
                <nonDerivativeTransaction>
                    <transactionCoding><transactionCode><value>P</value></transactionCode></transactionCoding>
                    <transactionAmounts>
                        <transactionShares><value>5000</value></transactionShares>
                        <transactionPricePerShare><value>10</value></transactionPricePerShare>
                    </transactionAmounts>
                </nonDerivativeTransaction>
                <nonDerivativeTransaction>
                    <transactionCoding><transactionCode><value>S</value></transactionCode></transactionCoding>
                    <transactionAmounts>
                        <transactionShares><value>2000</value></transactionShares>
                        <transactionPricePerShare><value>12</value></transactionPricePerShare>
                    </transactionAmounts>
                </nonDerivativeTransaction>
            </nonDerivativeTable>
        </ownershipDocument>"#;
        let p = parse_form4(xml);
        assert_eq!(p.transactions.len(), 2);
        assert_eq!(p.transactions[0].transaction_code, "P");
        assert_eq!(p.transactions[1].transaction_code, "S");
    }

    #[test]
    fn to_events_filters_zero_share_transactions() {
        let parsed = ParsedForm4 {
            issuer_symbol: Some("CCC".into()),
            insider_name: Some("X".into()),
            is_officer: true,
            is_director: false,
            is_ten_percent_owner: false,
            officer_title: None,
            transactions: vec![
                ParsedTransaction {
                    transaction_date: None,
                    transaction_code: "P".into(),
                    shares: 0.0,
                    price_per_share: 10.0,
                },
                ParsedTransaction {
                    transaction_date: None,
                    transaction_code: "P".into(),
                    shares: 100.0,
                    price_per_share: 10.0,
                },
            ],
        };
        let events = to_events(&parsed, None);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].shares, 100.0);
        assert_eq!(events[0].dollar_value, 1000.0);
    }

    #[test]
    fn to_events_skips_when_issuer_missing() {
        let parsed = ParsedForm4 {
            issuer_symbol: None,
            ..Default::default()
        };
        assert!(to_events(&parsed, None).is_empty());
    }

    #[test]
    fn store_dedupes_same_filing_replay() {
        let store = InsiderStore::new();
        let ev = InsiderEvent {
            symbol: "ACME".into(),
            insider_name: "DOE, JANE".into(),
            is_officer: true,
            is_director: false,
            is_ten_percent_owner: false,
            officer_title: Some("CEO".into()),
            transaction_code: "P".into(),
            kind: TxKind::Buy,
            shares: 10000.0,
            price_per_share: 50.0,
            dollar_value: 500_000.0,
            transaction_date: Some("2026-01-15".into()),
            filing_link: None,
            observed_at: Utc::now(),
        };
        assert!(store.observe(ev.clone()));
        assert!(!store.observe(ev.clone()), "replay must dedupe");
        let mut ev2 = ev.clone();
        ev2.shares = 5000.0; // different shares → different key.
        assert!(store.observe(ev2));
        assert_eq!(store.emitted.len(), 2);
    }

    #[test]
    fn top_buys_ranks_by_dollar_value_and_filters_sells() {
        let store = InsiderStore::new();
        let buy = |sym: &str, dollars: f64| InsiderEvent {
            symbol: sym.into(),
            insider_name: "X".into(),
            is_officer: false,
            is_director: false,
            is_ten_percent_owner: false,
            officer_title: None,
            transaction_code: "P".into(),
            kind: TxKind::Buy,
            shares: 1.0,
            price_per_share: dollars,
            dollar_value: dollars,
            transaction_date: None,
            filing_link: None,
            observed_at: Utc::now(),
        };
        let sell = |sym: &str, dollars: f64| InsiderEvent {
            kind: TxKind::Sell,
            transaction_code: "S".into(),
            ..buy(sym, dollars)
        };
        // Dedup is on (symbol, insider, date, code, shares); shares differ
        // per call below by passing distinct dollars-as-shares.
        let mut bigbuy1 = buy("AAA", 1_000_000.0);
        bigbuy1.shares = 1.0;
        store.observe(bigbuy1);
        let mut bigbuy2 = buy("AAA", 500_000.0);
        bigbuy2.shares = 2.0;
        store.observe(bigbuy2);
        let mut bigbuy3 = buy("BBB", 2_000_000.0);
        bigbuy3.shares = 3.0;
        store.observe(bigbuy3);
        // Sell on AAA must NOT count toward top_buys.
        let mut s1 = sell("AAA", 10_000_000.0);
        s1.shares = 4.0;
        store.observe(s1);
        let top = store.top_buys(30, 5);
        assert_eq!(top.len(), 2);
        // BBB ($2M) > AAA ($1.5M total) → BBB first.
        assert_eq!(top[0].0, "BBB");
        assert!((top[0].1 - 2_000_000.0).abs() < 1e-6);
        assert_eq!(top[1].0, "AAA");
        assert!((top[1].1 - 1_500_000.0).abs() < 1e-6);
    }
}
