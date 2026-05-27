//! Short interest tracker.
//!
//! Two sources, both free:
//!   1. Yahoo `defaultKeyStatistics` quoteSummary module —
//!      sharesShort / shortRatio / shortPercentOfFloat / sharesShortPriorMonth
//!   2. FINRA Reg SHO daily short-volume TSV:
//!      https://cdn.finra.org/equity/regsho/daily/CNMSshvolYYYYMMDD.txt
//!      (one row per symbol per market center; we aggregate to per-day per-symbol)

use chrono::{Datelike, DateTime, Duration, NaiveDate, Utc};
use serde::Serialize;
use std::collections::HashMap;

const UA: &str = "traderview/0.1 (github.com/MenkeTechnologies/traderview)";

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent(UA)
        .timeout(std::time::Duration::from_secs(12))
        .build()
        .unwrap()
}

#[derive(Debug, Clone, Serialize)]
pub struct ShortStats {
    pub symbol: String,
    pub shares_short: Option<f64>,
    pub shares_short_prior: Option<f64>,
    pub short_ratio: Option<f64>,        // days to cover
    pub short_pct_float: Option<f64>,    // 0-1
    pub short_pct_outstanding: Option<f64>,
    pub float: Option<f64>,
    pub change_pct: Option<f64>,         // shares_short vs prior month
    pub fetched_at: DateTime<Utc>,
}

pub async fn yahoo_short_stats(symbol: &str) -> anyhow::Result<ShortStats> {
    let v = crate::market_data::quote_summary(symbol, &["defaultKeyStatistics"]).await?;
    let ks = &v["defaultKeyStatistics"];
    let raw = |path: &str| -> Option<f64> {
        ks[path]["raw"].as_f64().or_else(|| ks[path].as_f64())
    };
    let cur = raw("sharesShort");
    let prior = raw("sharesShortPriorMonth");
    let change = match (cur, prior) {
        (Some(c), Some(p)) if p > 0.0 => Some((c - p) / p * 100.0),
        _ => None,
    };
    Ok(ShortStats {
        symbol: symbol.to_uppercase(),
        shares_short: cur,
        shares_short_prior: prior,
        short_ratio: raw("shortRatio"),
        short_pct_float: raw("shortPercentOfFloat"),
        short_pct_outstanding: raw("sharesPercentSharesOut"),
        float: raw("floatShares"),
        change_pct: change,
        fetched_at: Utc::now(),
    })
}

#[derive(Debug, Clone, Serialize)]
pub struct FinraDay {
    pub date: NaiveDate,
    pub short_volume: u64,
    pub short_exempt_volume: u64,
    pub total_volume: u64,
    pub short_pct: f64,
}

/// Fetch & aggregate FINRA Reg SHO daily short volume for `symbol` over the
/// last `days` trading days. Returns a series sorted oldest→newest.
pub async fn finra_daily(symbol: &str, days: i64) -> anyhow::Result<Vec<FinraDay>> {
    let symbol = symbol.to_uppercase();
    let mut out: Vec<FinraDay> = Vec::new();
    // FINRA only publishes M-F. Walk back including weekends and skip 404s.
    let today = Utc::now().date_naive();
    let mut probed = 0;
    let mut day = today;
    while probed < days * 2 && out.len() < days as usize {
        probed += 1;
        day -= Duration::days(1);
        // Skip weekends.
        let wd = day.weekday();
        if matches!(wd, chrono::Weekday::Sat | chrono::Weekday::Sun) { continue; }
        if let Some(row) = fetch_finra_day(&symbol, day).await {
            out.push(row);
        }
    }
    out.sort_by_key(|r| r.date);
    Ok(out)
}

async fn fetch_finra_day(symbol: &str, day: NaiveDate) -> Option<FinraDay> {
    let url = format!(
        "https://cdn.finra.org/equity/regsho/daily/CNMSshvol{y}{m:02}{d:02}.txt",
        y = day.year(), m = day.month(), d = day.day(),
    );
    let resp = client().get(&url).send().await.ok()?;
    if !resp.status().is_success() { return None; }
    let body = resp.text().await.ok()?;
    let mut header_idx: HashMap<&str, usize> = HashMap::new();
    let mut total = FinraDay {
        date: day, short_volume: 0, short_exempt_volume: 0, total_volume: 0, short_pct: 0.0,
    };
    let mut found = false;
    for (i, line) in body.lines().enumerate() {
        if i == 0 {
            for (j, h) in line.split('|').enumerate() {
                header_idx.insert(Box::leak(h.trim().to_string().into_boxed_str()), j);
            }
            continue;
        }
        let parts: Vec<&str> = line.split('|').collect();
        let sym_i = match header_idx.get("Symbol") { Some(i) => *i, None => continue };
        if parts.get(sym_i).map(|s| s.trim()) != Some(symbol) { continue; }
        let pick = |k: &str| -> u64 {
            header_idx.get(k).and_then(|&i| parts.get(i))
                .and_then(|s| s.trim().parse::<u64>().ok()).unwrap_or(0)
        };
        total.short_volume        += pick("ShortVolume");
        total.short_exempt_volume += pick("ShortExemptVolume");
        total.total_volume        += pick("TotalVolume");
        found = true;
    }
    if !found { return None; }
    if total.total_volume > 0 {
        total.short_pct = total.short_volume as f64 / total.total_volume as f64 * 100.0;
    }
    Some(total)
}

/// Pull Yahoo short stats for a list of symbols and return them sorted by
/// `short_pct_float` desc. Used for the watchlist ranking view.
pub async fn ranked_universe(symbols: &[String]) -> Vec<ShortStats> {
    let mut out = Vec::new();
    for s in symbols {
        if let Ok(r) = yahoo_short_stats(s).await { out.push(r); }
    }
    out.sort_by(|a, b| b.short_pct_float.unwrap_or(0.0)
        .partial_cmp(&a.short_pct_float.unwrap_or(0.0))
        .unwrap_or(std::cmp::Ordering::Equal));
    out
}
