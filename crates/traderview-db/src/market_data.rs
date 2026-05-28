//! Per-symbol market data — quote snapshot, news, earnings, dividends,
//! analyst recommendations, insider transactions, financials.
//!
//! Everything here hits Yahoo Finance's public endpoints (no auth):
//!   * chart      v8 — current price + change
//!   * quoteSummary v10 — earnings, dividends, recommendationTrend, etc.
//!   * search     v1 — news / ticker lookup
//!
//! Results are returned raw-ish so the frontend can render whatever it needs.
//! Failures degrade gracefully — each module returns `Option`.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::str::FromStr;

const UA: &str =
    "Mozilla/5.0 (compatible; traderview/0.1; +https://github.com/MenkeTechnologies/traderview)";

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent(UA)
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap()
}

// ===========================================================================
// Quote snapshot
// ===========================================================================

#[derive(Debug, Clone, Serialize)]
pub struct QuoteSnapshot {
    pub symbol: String,
    pub price: f64,
    pub prev_close: Option<f64>,
    pub change_pct: Option<f64>,
    pub day_high: Option<f64>,
    pub day_low: Option<f64>,
    pub volume: Option<i64>,
    pub market_state: Option<String>,
    pub fetched_at: DateTime<Utc>,
}

pub async fn quote(pool: &PgPool, symbol: &str) -> anyhow::Result<QuoteSnapshot> {
    // 60-second DB cache.
    if let Some(q) = read_quote_cache(pool, symbol).await? {
        if (Utc::now() - q.fetched_at).num_seconds() < 60 {
            return Ok(q);
        }
    }
    let fresh = fetch_quote_yahoo(symbol).await?;
    write_quote_cache(pool, &fresh).await?;
    Ok(fresh)
}

type QuoteCacheRow = (
    String,
    Decimal,
    Option<Decimal>,
    Option<Decimal>,
    Option<Decimal>,
    Option<Decimal>,
    Option<i64>,
    Option<String>,
    DateTime<Utc>,
);

async fn read_quote_cache(pool: &PgPool, symbol: &str) -> anyhow::Result<Option<QuoteSnapshot>> {
    let row: Option<QuoteCacheRow>
        = sqlx::query_as(
            "SELECT symbol, price, prev_close, change_pct, day_high, day_low, volume, market_state, fetched_at
               FROM quote_snapshots WHERE symbol = $1",
        )
        .bind(symbol)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(
        |(
            symbol,
            price,
            prev_close,
            change_pct,
            day_high,
            day_low,
            volume,
            market_state,
            fetched_at,
        )| QuoteSnapshot {
            symbol,
            price: dec_f64(price),
            prev_close: prev_close.map(dec_f64),
            change_pct: change_pct.map(dec_f64),
            day_high: day_high.map(dec_f64),
            day_low: day_low.map(dec_f64),
            volume,
            market_state,
            fetched_at,
        },
    ))
}

async fn write_quote_cache(pool: &PgPool, q: &QuoteSnapshot) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO quote_snapshots
            (symbol, price, prev_close, change_pct, day_high, day_low, volume, market_state, fetched_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, now())
         ON CONFLICT (symbol) DO UPDATE SET
            price = EXCLUDED.price, prev_close = EXCLUDED.prev_close,
            change_pct = EXCLUDED.change_pct,
            day_high = EXCLUDED.day_high, day_low = EXCLUDED.day_low,
            volume = EXCLUDED.volume, market_state = EXCLUDED.market_state,
            fetched_at = now()",
    )
    .bind(&q.symbol)
    .bind(Decimal::from_str(&q.price.to_string()).unwrap_or(Decimal::ZERO))
    .bind(q.prev_close.map(|x| Decimal::from_str(&x.to_string()).unwrap_or(Decimal::ZERO)))
    .bind(q.change_pct.map(|x| Decimal::from_str(&x.to_string()).unwrap_or(Decimal::ZERO)))
    .bind(q.day_high.map(|x| Decimal::from_str(&x.to_string()).unwrap_or(Decimal::ZERO)))
    .bind(q.day_low.map(|x| Decimal::from_str(&x.to_string()).unwrap_or(Decimal::ZERO)))
    .bind(q.volume)
    .bind(&q.market_state)
    .execute(pool)
    .await?;
    Ok(())
}

async fn fetch_quote_yahoo(symbol: &str) -> anyhow::Result<QuoteSnapshot> {
    let url = format!(
        "https://query1.finance.yahoo.com/v8/finance/chart/{sym}?interval=1d&range=5d",
        sym = enc(symbol),
    );
    let resp = client().get(&url).send().await?;
    if !resp.status().is_success() {
        anyhow::bail!("chart HTTP {}", resp.status());
    }
    let raw: ChartResp = resp.json().await?;
    let r = raw
        .chart
        .result
        .and_then(|mut v| v.pop())
        .ok_or_else(|| anyhow::anyhow!("empty result"))?;
    let m = r.meta;
    let price = m.regular_market_price.unwrap_or(0.0);
    let prev = m.chart_previous_close.or(m.previous_close);
    let change_pct = prev.and_then(|p| {
        if p > 0.0 {
            Some((price - p) / p * 100.0)
        } else {
            None
        }
    });
    Ok(QuoteSnapshot {
        symbol: symbol.into(),
        price,
        prev_close: prev,
        change_pct,
        day_high: m.regular_market_day_high,
        day_low: m.regular_market_day_low,
        volume: m.regular_market_volume,
        market_state: m.market_state,
        fetched_at: Utc::now(),
    })
}

/// Bulk quote — concurrent fan-out. Yahoo handles ~16 parallel requests fine
/// for the chart endpoint (markets snapshot does the same). The previous
/// serial loop is what made `/api/premarket/snapshot` block for 150s.
pub async fn quotes(pool: &PgPool, symbols: &[String]) -> Vec<QuoteSnapshot> {
    let futs = symbols.iter().map(|s| {
        let pool = pool.clone();
        let sym = s.clone();
        async move { quote(&pool, &sym).await.ok() }
    });
    futures_util::future::join_all(futs)
        .await
        .into_iter()
        .flatten()
        .collect()
}

#[derive(serde::Deserialize)]
struct ChartResp {
    chart: ChartInner,
}
#[derive(serde::Deserialize)]
struct ChartInner {
    result: Option<Vec<ChartResult>>,
}
#[derive(serde::Deserialize)]
struct ChartResult {
    meta: ChartMeta,
}
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChartMeta {
    regular_market_price: Option<f64>,
    previous_close: Option<f64>,
    chart_previous_close: Option<f64>,
    regular_market_day_high: Option<f64>,
    regular_market_day_low: Option<f64>,
    regular_market_volume: Option<i64>,
    market_state: Option<String>,
}

// ===========================================================================
// quoteSummary v10 — earnings, dividends, recommendation, insiders, financials
// ===========================================================================

/// Fetch one or more quoteSummary modules. Returns raw JSON so the frontend
/// can render whatever shape Yahoo currently exposes.
pub async fn quote_summary(symbol: &str, modules: &[&str]) -> anyhow::Result<serde_json::Value> {
    let url = format!(
        "https://query1.finance.yahoo.com/v10/finance/quoteSummary/{sym}?modules={mods}",
        sym = enc(symbol),
        mods = modules.join(","),
    );
    let resp = client().get(&url).send().await?;
    let status = resp.status();
    if !status.is_success() {
        anyhow::bail!("quoteSummary HTTP {}", status);
    }
    let v: serde_json::Value = resp.json().await?;
    Ok(v["quoteSummary"]["result"]
        .get(0)
        .cloned()
        .unwrap_or(serde_json::Value::Null))
}

pub async fn fundamentals(symbol: &str) -> anyhow::Result<serde_json::Value> {
    quote_summary(
        symbol,
        &[
            "summaryDetail",
            "financialData",
            "defaultKeyStatistics",
            "incomeStatementHistory",
            "balanceSheetHistory",
            "cashflowStatementHistory",
            "assetProfile",
        ],
    )
    .await
}

pub async fn earnings(symbol: &str) -> anyhow::Result<serde_json::Value> {
    quote_summary(
        symbol,
        &[
            "earnings",
            "earningsHistory",
            "earningsTrend",
            "calendarEvents",
        ],
    )
    .await
}

pub async fn recommendations(symbol: &str) -> anyhow::Result<serde_json::Value> {
    quote_summary(symbol, &["recommendationTrend", "upgradeDowngradeHistory"]).await
}

pub async fn insiders(symbol: &str) -> anyhow::Result<serde_json::Value> {
    quote_summary(
        symbol,
        &[
            "insiderTransactions",
            "insiderHolders",
            "netSharePurchaseActivity",
        ],
    )
    .await
}

pub async fn dividends(symbol: &str) -> anyhow::Result<serde_json::Value> {
    quote_summary(symbol, &["summaryDetail", "calendarEvents", "fundProfile"]).await
}

pub async fn holders(symbol: &str) -> anyhow::Result<serde_json::Value> {
    quote_summary(
        symbol,
        &[
            "majorHoldersBreakdown",
            "institutionOwnership",
            "fundOwnership",
        ],
    )
    .await
}

// ===========================================================================
// News (Yahoo search endpoint)
// ===========================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsItem {
    pub uuid: Option<String>,
    pub title: Option<String>,
    pub publisher: Option<String>,
    pub link: Option<String>,
    pub provider_publish_time: Option<i64>,
    pub thumbnail: Option<String>,
}

pub async fn news(symbol: &str, count: usize) -> anyhow::Result<Vec<NewsItem>> {
    let url = format!(
        "https://query1.finance.yahoo.com/v1/finance/search?q={q}&quotesCount=0&newsCount={n}",
        q = enc(symbol),
        n = count,
    );
    let resp = client().get(&url).send().await?;
    if !resp.status().is_success() {
        anyhow::bail!("news HTTP {}", resp.status());
    }
    let v: serde_json::Value = resp.json().await?;
    let news_arr = v["news"].as_array().cloned().unwrap_or_default();
    Ok(news_arr
        .into_iter()
        .map(|n| NewsItem {
            uuid: n["uuid"].as_str().map(|s| s.into()),
            title: n["title"].as_str().map(|s| s.into()),
            publisher: n["publisher"].as_str().map(|s| s.into()),
            link: n["link"].as_str().map(|s| s.into()),
            provider_publish_time: n["providerPublishTime"].as_i64(),
            thumbnail: n["thumbnail"]["resolutions"][0]["url"]
                .as_str()
                .map(|s| s.into()),
        })
        .collect())
}

// ===========================================================================
// Helpers
// ===========================================================================

fn dec_f64(d: Decimal) -> f64 {
    d.to_string().parse().unwrap_or(0.0)
}
fn enc(s: &str) -> String {
    s.replace('^', "%5E").replace('=', "%3D")
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── dec_f64: Decimal → f64 via string round-trip ──────────────────────

    #[test]
    fn dec_f64_round_trips_simple_decimal() {
        assert_eq!(dec_f64(Decimal::from(42)), 42.0);
        assert_eq!(dec_f64(Decimal::ZERO), 0.0);
        assert_eq!(dec_f64(Decimal::from(-7)), -7.0);
    }

    #[test]
    fn dec_f64_preserves_fractional_quote_prices() {
        // Quote snapshots store prices as Decimal in the DB; the f64 surface
        // is what hits the JSON response. Must not drift on representative
        // tick sizes.
        let d = Decimal::from_str("150.25").unwrap();
        assert_eq!(dec_f64(d), 150.25);
        let d = Decimal::from_str("0.01").unwrap();
        assert_eq!(dec_f64(d), 0.01);
    }

    #[test]
    fn dec_f64_handles_high_precision() {
        // rust_decimal supports up to 28 fractional digits — verify we don't
        // truncate aggressively on values that f64 can still represent.
        let d = Decimal::from_str("4.25").unwrap();
        assert!((dec_f64(d) - 4.25).abs() < 1e-12);
    }

    // ── enc: URL escape symbol-name special characters ────────────────────

    #[test]
    fn enc_escapes_caret_for_indices() {
        // Yahoo uses ^GSPC, ^VIX, ^DJI; the caret must be percent-encoded.
        assert_eq!(enc("^GSPC"), "%5EGSPC");
        assert_eq!(enc("^VIX"), "%5EVIX");
    }

    #[test]
    fn enc_escapes_equals_for_futures_and_fx() {
        // Yahoo futures (ES=F, CL=F) and FX (EURUSD=X) use '=' in the symbol.
        assert_eq!(enc("ES=F"), "ES%3DF");
        assert_eq!(enc("EURUSD=X"), "EURUSD%3DX");
    }

    #[test]
    fn enc_leaves_plain_tickers_untouched() {
        // The common case — equities should be a no-op through the encoder.
        assert_eq!(enc("AAPL"), "AAPL");
        assert_eq!(enc("MSFT"), "MSFT");
        assert_eq!(enc("BRK-B"), "BRK-B");
    }

    #[test]
    fn enc_handles_both_chars_in_one_symbol() {
        // Pathological but valid — caret AND equals in the same symbol must
        // both be escaped independently.
        assert_eq!(enc("^FOO=BAR"), "%5EFOO%3DBAR");
    }

    #[test]
    fn enc_returns_empty_for_empty_input() {
        // Defensive: empty symbol is a caller bug but must not panic.
        assert_eq!(enc(""), "");
    }

    // ── NewsItem serde round-trip pins public API shape ───────────────────

    #[test]
    fn news_item_serializes_with_snake_case_fields() {
        let item = NewsItem {
            uuid: Some("abc".into()),
            title: Some("Headline".into()),
            publisher: Some("Reuters".into()),
            link: Some("https://example.com".into()),
            provider_publish_time: Some(1700000000),
            thumbnail: None,
        };
        let v = serde_json::to_value(&item).unwrap();
        // The frontend keys these fields exactly — renaming would break it.
        assert_eq!(v["uuid"], "abc");
        assert_eq!(v["title"], "Headline");
        assert_eq!(v["provider_publish_time"], 1700000000);
        assert!(v["thumbnail"].is_null());
    }
}
