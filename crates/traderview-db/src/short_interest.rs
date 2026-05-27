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
    parse_finra_body(&body, symbol, day)
}

/// Pipe-delimited Reg-SHO daily-file parser.
///
/// Header: `Date|Symbol|ShortVolume|ShortExemptVolume|TotalVolume|Market`
/// A single symbol may appear on multiple lines (one per market: NYSE, NSDQ,
/// ARCA, etc.) — we sum across markets. Computes `short_pct = short_volume
/// / total_volume * 100` when total_volume is non-zero.
///
/// Returns `None` when the symbol has no rows in the file — signals
/// "FINRA had no data for this date" so the caller can skip.
pub(crate) fn parse_finra_body(body: &str, symbol: &str, day: NaiveDate) -> Option<FinraDay> {
    let mut header_idx: HashMap<String, usize> = HashMap::new();
    let mut total = FinraDay {
        date: day, short_volume: 0, short_exempt_volume: 0, total_volume: 0, short_pct: 0.0,
    };
    let mut found = false;
    for (i, line) in body.lines().enumerate() {
        if i == 0 {
            for (j, h) in line.split('|').enumerate() {
                header_idx.insert(h.trim().to_string(), j);
            }
            continue;
        }
        let parts: Vec<&str> = line.split('|').collect();
        let sym_i = match header_idx.get("Symbol") { Some(i) => *i, None => continue };
        if parts.get(sym_i).map(|s| s.trim()) != Some(symbol) { continue; }
        let pick = |k: &str| -> u64 {
            header_idx.get(k).and_then(|i| parts.get(*i))
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Canonical FINRA Reg-SHO daily-file body, header + 3 sample rows for
    /// AAPL across markets + a non-AAPL row that must be ignored.
    const SAMPLE: &str = "Date|Symbol|ShortVolume|ShortExemptVolume|TotalVolume|Market
20260527|AAPL|100000|500|400000|N
20260527|AAPL|50000|0|200000|Q
20260527|AAPL|25000|0|100000|P
20260527|TSLA|999999|999|9999999|N
";

    fn day() -> NaiveDate { NaiveDate::from_ymd_opt(2026, 5, 27).unwrap() }

    #[test]
    fn parses_and_sums_across_markets() {
        // AAPL across 3 markets: 100000 + 50000 + 25000 = 175000 short.
        // Total: 400000 + 200000 + 100000 = 700000.
        // Short pct: 175000 / 700000 = 25.0%.
        let r = parse_finra_body(SAMPLE, "AAPL", day()).expect("AAPL rows present");
        assert_eq!(r.short_volume, 175_000);
        assert_eq!(r.short_exempt_volume, 500);
        assert_eq!(r.total_volume, 700_000);
        assert!((r.short_pct - 25.0).abs() < 1e-9);
    }

    #[test]
    fn ignores_non_matching_symbols() {
        let r = parse_finra_body(SAMPLE, "TSLA", day()).expect("TSLA row present");
        assert_eq!(r.short_volume, 999_999);
        assert_eq!(r.total_volume, 9_999_999);
        // TSLA must not pick up AAPL's data.
        assert_ne!(r.short_volume, 175_000);
    }

    #[test]
    fn returns_none_when_symbol_absent() {
        let r = parse_finra_body(SAMPLE, "NVDA", day());
        assert!(r.is_none(), "no NVDA rows → must return None, not zeroed FinraDay");
    }

    #[test]
    fn handles_zero_total_volume_without_divide_by_zero() {
        let body = "Date|Symbol|ShortVolume|ShortExemptVolume|TotalVolume|Market
20260527|FOO|0|0|0|N
";
        let r = parse_finra_body(body, "FOO", day()).expect("FOO row present");
        assert_eq!(r.total_volume, 0);
        assert_eq!(r.short_pct, 0.0, "zero total must yield 0%, not NaN/Inf");
    }

    #[test]
    fn skips_unparseable_numeric_cells() {
        // Cell with letters → should silently become 0, not panic.
        let body = "Date|Symbol|ShortVolume|ShortExemptVolume|TotalVolume|Market
20260527|FOO|abc|0|1000|N
";
        let r = parse_finra_body(body, "FOO", day()).expect("FOO row");
        assert_eq!(r.short_volume, 0);   // unparseable → 0
        assert_eq!(r.total_volume, 1_000);
    }

    #[test]
    fn empty_body_returns_none() {
        let r = parse_finra_body("", "AAPL", day());
        assert!(r.is_none());
    }

    #[test]
    fn header_only_returns_none() {
        let r = parse_finra_body(
            "Date|Symbol|ShortVolume|ShortExemptVolume|TotalVolume|Market",
            "AAPL", day(),
        );
        assert!(r.is_none(), "header without data rows = no result");
    }

    #[test]
    fn preserves_input_date() {
        let target = NaiveDate::from_ymd_opt(2020, 1, 15).unwrap();
        let r = parse_finra_body(SAMPLE, "AAPL", target).unwrap();
        assert_eq!(r.date, target, "parser uses caller-supplied date, not header");
    }
}
