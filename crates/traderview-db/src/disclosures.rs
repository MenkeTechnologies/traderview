//! Insider Form 4 + Senate + House STOCK Act disclosure pipeline.
//!
//! Pollers:
//!   * EDGAR Form 4 atom: https://www.sec.gov/cgi-bin/browse-edgar?action=getcurrent&type=4&output=atom
//!   * Senate STOCK Act:  https://efdsearch.senate.gov/search/  (HTML scrape via results table)
//!   * House STOCK Act:   https://disclosures-clerk.house.gov/PublicDisclosure/FinancialDisclosure
//!
//! Each poller upserts into `disclosures` (UNIQUE on kind + external_id makes
//! re-polling a no-op). On insert, fired-watcher rows are emitted so the push
//! delivery loop can pick them up.

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

const UA: &str =
    "traderview/0.1 (github.com/MenkeTechnologies/traderview; ops@menketechnologies.com)";

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent(UA)
        .timeout(std::time::Duration::from_secs(12))
        .build()
        .unwrap()
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Disclosure {
    pub id: Uuid,
    pub kind: String,
    pub external_id: String,
    pub symbol: Option<String>,
    pub filer_name: String,
    pub filer_role: Option<String>,
    pub txn_type: Option<String>,
    pub shares: Option<Decimal>,
    pub price: Option<Decimal>,
    pub amount_usd: Option<Decimal>,
    pub amount_range: Option<String>,
    pub txn_date: Option<NaiveDate>,
    pub filed_at: DateTime<Utc>,
    pub detected_at: DateTime<Utc>,
    pub source_url: Option<String>,
    pub raw: serde_json::Value,
}

// ===========================================================================
// Inserts + queries
// ===========================================================================

pub async fn upsert(pool: &PgPool, d: &Disclosure) -> anyhow::Result<bool> {
    let res = sqlx::query(
        "INSERT INTO disclosures
            (kind, external_id, symbol, filer_name, filer_role, txn_type,
             shares, price, amount_usd, amount_range, txn_date, filed_at, source_url, raw)
         VALUES ($1::disclosure_kind_t, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
         ON CONFLICT (kind, external_id) DO NOTHING",
    )
    .bind(&d.kind).bind(&d.external_id).bind(&d.symbol)
    .bind(&d.filer_name).bind(&d.filer_role).bind(&d.txn_type)
    .bind(d.shares).bind(d.price).bind(d.amount_usd)
    .bind(&d.amount_range).bind(d.txn_date).bind(d.filed_at).bind(&d.source_url).bind(&d.raw)
    .execute(pool).await?;
    Ok(res.rows_affected() > 0)
}

pub async fn list(
    pool: &PgPool,
    kind: Option<&str>,
    symbol: Option<&str>,
    limit: i64,
) -> anyhow::Result<Vec<Disclosure>> {
    let mut q = sqlx::QueryBuilder::new(
        "SELECT id, kind::text, external_id, symbol, filer_name, filer_role, txn_type,
                shares, price, amount_usd, amount_range, txn_date, filed_at, detected_at,
                source_url, raw FROM disclosures WHERE 1=1",
    );
    if let Some(k) = kind {
        q.push(" AND kind = ").push_bind(k).push("::disclosure_kind_t");
    }
    if let Some(s) = symbol {
        q.push(" AND symbol = ").push_bind(s);
    }
    q.push(" ORDER BY filed_at DESC LIMIT ").push_bind(limit);
    Ok(q.build_query_as().fetch_all(pool).await?)
}

// ===========================================================================
// EDGAR Form 4 atom feed poller
// ===========================================================================

pub async fn poll_edgar_form4(pool: &PgPool) -> anyhow::Result<usize> {
    let url = "https://www.sec.gov/cgi-bin/browse-edgar?action=getcurrent&type=4&output=atom";
    let body = client().get(url).send().await?.text().await?;
    let entries = parse_atom_entries(&body);
    let mut inserted = 0;
    for e in entries {
        // Atom <id> looks like
        //   urn:tag:sec.gov,2008:accession-number=0001209191-26-012345
        let accession = e.id.split("accession-number=").nth(1)
            .unwrap_or(&e.id).to_string();
        // Filer name lives in <title>: "4 - <FILER NAME> (CIK)"
        let title = e.title.clone();
        let filer = title.splitn(3, " - ").nth(1)
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| title.clone());
        let disclosure = Disclosure {
            id: Uuid::nil(),
            kind: "insider_form4".into(),
            external_id: accession,
            symbol: None,            // EDGAR Form 4 doesn't include ticker in the atom feed
            filer_name: filer,
            filer_role: None,
            txn_type: None,
            shares: None,
            price: None,
            amount_usd: None,
            amount_range: None,
            txn_date: None,
            filed_at: e.updated,
            detected_at: Utc::now(),
            source_url: Some(e.link),
            raw: serde_json::json!({"title": title}),
        };
        if upsert(pool, &disclosure).await.unwrap_or(false) {
            inserted += 1;
        }
    }
    Ok(inserted)
}

struct AtomEntry { id: String, title: String, link: String, updated: DateTime<Utc> }

fn parse_atom_entries(body: &str) -> Vec<AtomEntry> {
    // Tiny hand-rolled parser — atom is mostly structured enough that we can
    // pull <entry>…</entry> blocks and slice element text out by tag.
    let mut out = Vec::new();
    let mut rest = body;
    while let Some(start) = rest.find("<entry>") {
        rest = &rest[start..];
        let Some(end) = rest.find("</entry>") else { break };
        let block = &rest[..end];
        rest = &rest[end + "</entry>".len()..];

        let id    = inner_text(block, "id").unwrap_or_default();
        let title = inner_text(block, "title").unwrap_or_default();
        let updated = inner_text(block, "updated")
            .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
            .map(|d| d.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);
        let link = link_href(block).unwrap_or_default();
        out.push(AtomEntry { id, title, link, updated });
    }
    out
}

fn inner_text(block: &str, tag: &str) -> Option<String> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    let s = block.find(&open)? + open.len();
    let e = block[s..].find(&close)?;
    Some(block[s..s+e].trim().to_string())
}

fn link_href(block: &str) -> Option<String> {
    let i = block.find("<link")?;
    let href_i = block[i..].find("href=\"")? + 6;
    let abs_start = i + href_i;
    let end = block[abs_start..].find('"')?;
    Some(block[abs_start..abs_start+end].to_string())
}

// ===========================================================================
// Senate STOCK Act poller (HTML scrape)
//
// The Senate eFD search results page renders rows like:
//   <tr><td>last, first</td><td>filing type</td><td>filing date</td>
//       <td><a href="…/view/ptr/...">…</a></td></tr>
// We poll the "recent filings" listing and ingest accession URLs we haven't
// seen yet. Detailed per-filing parsing is left to a follow-up — we still
// capture the filer + filed_at + source URL up front so push fires
// immediately, then a worker can backfill the txn detail later.
// ===========================================================================

pub async fn poll_senate(pool: &PgPool) -> anyhow::Result<usize> {
    let url = "https://efdsearch.senate.gov/search/home/";
    let body = client().get(url).send().await?.text().await?;
    // Strip out anchor hrefs that look like a transaction filing link.
    let mut inserted = 0;
    for cap in extract_senate_rows(&body) {
        let filed_at = cap.filed.and_then(|d| d.and_hms_opt(0, 0, 0))
            .map(|n| n.and_utc()).unwrap_or_else(Utc::now);
        let d = Disclosure {
            id: Uuid::nil(),
            kind: "senate_stock".into(),
            external_id: cap.href.clone(),
            symbol: None,
            filer_name: cap.filer,
            filer_role: Some("U.S. Senator".into()),
            txn_type: None,
            shares: None,
            price: None,
            amount_usd: None,
            amount_range: None,
            txn_date: None,
            filed_at,
            detected_at: Utc::now(),
            source_url: Some(format!("https://efdsearch.senate.gov{}", cap.href)),
            raw: serde_json::json!({"filing_type": cap.filing_type}),
        };
        if upsert(pool, &d).await.unwrap_or(false) { inserted += 1; }
    }
    Ok(inserted)
}

struct SenateRow { filer: String, filing_type: String, filed: Option<NaiveDate>, href: String }

fn extract_senate_rows(body: &str) -> Vec<SenateRow> {
    // Match every <tr>…</tr>, then look for an anchor href ending in /view/.
    let mut out = Vec::new();
    let mut rest = body;
    while let Some(start) = rest.find("<tr") {
        rest = &rest[start..];
        let Some(end) = rest.find("</tr>") else { break };
        let row = &rest[..end];
        rest = &rest[end + 5..];
        let Some(href_i) = row.find("href=\"/search/view/") else { continue };
        let href_start = href_i + 6;
        let href_end = row[href_start..].find('"').unwrap_or(0);
        let href = row[href_start..href_start + href_end].to_string();
        // Pull plain-text td contents.
        let tds: Vec<String> = row.split("<td")
            .skip(1)
            .map(|s| s.split('>').skip(1).collect::<Vec<_>>().join(">"))
            .map(|s| s.split("</td>").next().unwrap_or("").to_string())
            .map(|s| strip_html(&s).trim().to_string())
            .collect();
        if tds.len() < 4 { continue; }
        let filer = tds.get(0).cloned().unwrap_or_default();
        let filing_type = tds.get(2).cloned().unwrap_or_default();
        let filed = tds.get(3).and_then(|s|
            NaiveDate::parse_from_str(s, "%m/%d/%Y").ok()
                .or_else(|| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()));
        out.push(SenateRow { filer, filing_type, filed, href });
    }
    out
}

fn strip_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out
}

// ===========================================================================
// House STOCK Act poller (HTML scrape — disclosures-clerk.house.gov)
// ===========================================================================

pub async fn poll_house(pool: &PgPool) -> anyhow::Result<usize> {
    let url = "https://disclosures-clerk.house.gov/PublicDisclosure/FinancialDisclosure";
    let body = client().get(url).send().await?.text().await?;
    let mut inserted = 0;
    // The page links each filing under '/ViewMember?member=…' and '/View?…'.
    for href in extract_house_links(&body) {
        let d = Disclosure {
            id: Uuid::nil(),
            kind: "house_stock".into(),
            external_id: href.clone(),
            symbol: None,
            filer_name: "(House filing)".into(),
            filer_role: Some("U.S. Representative".into()),
            txn_type: None,
            shares: None, price: None, amount_usd: None, amount_range: None,
            txn_date: None,
            filed_at: Utc::now(),
            detected_at: Utc::now(),
            source_url: Some(format!("https://disclosures-clerk.house.gov{}", href)),
            raw: serde_json::json!({}),
        };
        if upsert(pool, &d).await.unwrap_or(false) { inserted += 1; }
    }
    Ok(inserted)
}

fn extract_house_links(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = body;
    let needle = "href=\"/PublicDisclosure/ViewDocument";
    while let Some(i) = rest.find(needle) {
        let start = i + 6;
        let end = rest[start..].find('"').unwrap_or(0);
        out.push(rest[start..start+end].to_string());
        rest = &rest[start + end..];
    }
    out
}

// ===========================================================================
// Run all pollers once.
// ===========================================================================

#[derive(Debug, Clone, Serialize)]
pub struct PollResult {
    pub edgar_inserted: usize,
    pub senate_inserted: usize,
    pub house_inserted: usize,
}

pub async fn poll_all(pool: &PgPool) -> PollResult {
    let edgar = poll_edgar_form4(pool).await.unwrap_or_else(|e| {
        tracing::warn!(error = ?e, "edgar poll failed"); 0
    });
    let senate = poll_senate(pool).await.unwrap_or_else(|e| {
        tracing::warn!(error = ?e, "senate poll failed"); 0
    });
    let house = poll_house(pool).await.unwrap_or_else(|e| {
        tracing::warn!(error = ?e, "house poll failed"); 0
    });
    PollResult { edgar_inserted: edgar, senate_inserted: senate, house_inserted: house }
}

// ===========================================================================
// Disclosure watchers
// ===========================================================================

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Watcher {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub kinds: Vec<String>,
    pub symbols: Option<Vec<String>>,
    pub filers: Option<Vec<String>>,
    pub min_amount_usd: Option<Decimal>,
    pub enabled: bool,
    pub sound: String,
    pub created_at: DateTime<Utc>,
}

pub async fn list_watchers(pool: &PgPool, user_id: Uuid) -> anyhow::Result<Vec<Watcher>> {
    Ok(sqlx::query_as::<_, Watcher>(
        "SELECT id, user_id, name, kinds, symbols, filers, min_amount_usd, enabled, sound, created_at
           FROM disclosure_watchers WHERE user_id = $1 ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool).await?)
}

pub async fn create_watcher(
    pool: &PgPool,
    user_id: Uuid,
    name: &str,
    kinds: &[String],
    symbols: Option<&[String]>,
    filers: Option<&[String]>,
    min_amount_usd: Option<Decimal>,
    sound: &str,
) -> anyhow::Result<Watcher> {
    Ok(sqlx::query_as::<_, Watcher>(
        "INSERT INTO disclosure_watchers (user_id, name, kinds, symbols, filers, min_amount_usd, sound)
              VALUES ($1, $2, $3, $4, $5, $6, $7)
         RETURNING id, user_id, name, kinds, symbols, filers, min_amount_usd, enabled, sound, created_at",
    )
    .bind(user_id).bind(name).bind(kinds).bind(symbols).bind(filers).bind(min_amount_usd).bind(sound)
    .fetch_one(pool).await?)
}

pub async fn delete_watcher(pool: &PgPool, user_id: Uuid, id: Uuid) -> anyhow::Result<bool> {
    let r = sqlx::query("DELETE FROM disclosure_watchers WHERE id = $1 AND user_id = $2")
        .bind(id).bind(user_id).execute(pool).await?;
    Ok(r.rows_affected() > 0)
}
