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

/// Fundamentals lookup.
///
/// **Backend:** Finnhub `/stock/profile2` + `/stock/metric?metric=all`
/// (free tier). The previous Yahoo `quoteSummary[summaryDetail,
/// defaultKeyStatistics, financialData, assetProfile, ...]` backend
/// returned 401 "Invalid Crumb" since late 2023.
///
/// Adapter `finnhub_rest::fundamentals_yahoo_shape` composes a JSON
/// envelope that matches Yahoo's nested `{raw, fmt}` shape so the
/// existing frontend extractor in `research.js::renderFund` works
/// unchanged. Falls back to Yahoo `quoteSummary` if no Finnhub key is
/// configured (will still 401 for now — set the key in Settings).
pub async fn fundamentals(symbol: &str) -> anyhow::Result<serde_json::Value> {
    match crate::finnhub_rest::fundamentals_yahoo_shape(symbol).await {
        Ok(v) => return Ok(v),
        Err(e) => {
            tracing::warn!(
                symbol,
                error = %e,
                "finnhub fundamentals failed; falling back to Yahoo quoteSummary"
            );
        }
    }
    fundamentals_yahoo_legacy(symbol).await
}

async fn fundamentals_yahoo_legacy(symbol: &str) -> anyhow::Result<serde_json::Value> {
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

/// Earnings lookup.
///
/// **Backend:** Finnhub `/stock/eps-surprise` (history) + `/calendar/earnings`
/// (next-quarter date + estimates). Adapter
/// `finnhub_rest::earnings_yahoo_shape` returns Yahoo's
/// `{earningsHistory.history[], calendarEvents.earnings.{earningsDate,
/// earningsAverage, revenueAverage}}` shape so the existing extractor in
/// `research.js::renderEarnings` works unchanged. Falls back to broken
/// Yahoo `quoteSummary` only when no Finnhub key is configured.
pub async fn earnings(symbol: &str) -> anyhow::Result<serde_json::Value> {
    match crate::finnhub_rest::earnings_yahoo_shape(symbol).await {
        Ok(v) => return Ok(v),
        Err(e) => {
            tracing::warn!(
                symbol,
                error = %e,
                "finnhub earnings failed; falling back to Yahoo quoteSummary"
            );
        }
    }
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

/// Analyst recommendations.
///
/// **Backend:** Finnhub `/stock/recommendation` (free tier). Adapter
/// `finnhub_rest::recommendations_yahoo_shape` returns Yahoo's
/// `{recommendationTrend.trend[]}` envelope. Falls back to broken Yahoo
/// `quoteSummary` only when no Finnhub key is configured.
pub async fn recommendations(symbol: &str) -> anyhow::Result<serde_json::Value> {
    match crate::finnhub_rest::recommendations_yahoo_shape(symbol).await {
        Ok(v) => return Ok(v),
        Err(e) => {
            tracing::warn!(
                symbol,
                error = %e,
                "finnhub recommendations failed; falling back to Yahoo quoteSummary"
            );
        }
    }
    quote_summary(symbol, &["recommendationTrend", "upgradeDowngradeHistory"]).await
}

/// Insider transactions.
///
/// **Backend:** Finnhub `/stock/insider-transactions` (FREE tier, last
/// ~3 months of Form 4 filings). Adapter `finnhub_rest::insiders_yahoo_shape`
/// wraps the response in Yahoo's `{insiderTransactions: {transactions[]}}`
/// envelope so `research.js::renderInsiders` keeps working unchanged.
/// Falls back to Yahoo `quoteSummary[insiderTransactions,…]` when no
/// Finnhub key is configured — note the Yahoo path returns 401 since
/// late 2023 ("Invalid Crumb") so without Finnhub the Insider Activity
/// panel will show "no data".
pub async fn insiders(symbol: &str) -> anyhow::Result<serde_json::Value> {
    match crate::finnhub_rest::insiders_yahoo_shape(symbol).await {
        Ok(v) => return Ok(v),
        Err(e) => {
            tracing::warn!(
                symbol,
                error = %e,
                "finnhub insiders failed; falling back to Yahoo quoteSummary"
            );
        }
    }
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

/// Dividend lookup.
///
/// **Backend:** Yahoo `v8/finance/chart` with `events=div,split`. This is
/// the only Yahoo endpoint that still works without the crumb+cookie
/// dance — `v10/finance/quoteSummary` (the previous backend) has required
/// a crumb since late 2023 and returns 401 / 429 otherwise.
///
/// **Output shape:** synthesized to look like the old quoteSummary payload
/// (`{summaryDetail: {...}, calendarEvents: {...}}`) so the existing
/// `frontend/js/_dividend_calendar_inputs.js::extractDividend` keeps
/// working without any frontend change.
///
/// **Approximations:**
/// - `dividendRate` = sum of the trailing four dividend amounts (≈ annual)
/// - `dividendYield` = `dividendRate / regularMarketPrice` (decimal, e.g. `0.025`)
/// - `exDividendDate` = last known ex-date; the prior backend already only
///   surfaced "last known" since `calendarEvents` was unreliable for
///   forward dates. Frontend filters past dates out, so the upcoming-only
///   filter still works once we cross over a quarterly boundary.
/// - `dividendDate` (pay date) = not available from v8 chart → omitted.
/// - `payoutRatio` = not available from v8 chart → null.
///
/// Non-payers return an empty `summaryDetail` so `extractDividend()`
/// returns `null` (same as before for ETFs / non-paying stocks).
pub async fn dividends(symbol: &str) -> anyhow::Result<serde_json::Value> {
    let url = format!(
        "https://query1.finance.yahoo.com/v8/finance/chart/{sym}\
         ?interval=1d&range=2y&events=div,split",
        sym = enc(symbol),
    );
    let resp = client().get(&url).send().await?;
    if !resp.status().is_success() {
        anyhow::bail!("dividends chart HTTP {}", resp.status());
    }
    let v: serde_json::Value = resp.json().await?;
    let result = &v["chart"]["result"][0];
    let last_price = result["meta"]["regularMarketPrice"].as_f64();

    // events.dividends is an object keyed by unix-ts strings; values are
    // `{amount: f64, date: i64}`. Sort by date so we can take "trailing 4".
    let mut events: Vec<(i64, f64)> = result["events"]["dividends"]
        .as_object()
        .map(|m| {
            m.values()
                .filter_map(|ev| {
                    let date = ev["date"].as_i64()?;
                    let amount = ev["amount"].as_f64()?;
                    Some((date, amount))
                })
                .collect()
        })
        .unwrap_or_default();
    events.sort_by_key(|(d, _)| *d);

    // Non-payer / ETF: emit an empty envelope so extractDividend returns null.
    if events.is_empty() {
        return Ok(serde_json::json!({
            "summaryDetail": {},
            "calendarEvents": {},
        }));
    }

    let last = events.last().copied().unwrap();
    let trailing_4: f64 = events.iter().rev().take(4).map(|(_, a)| *a).sum();
    let dividend_yield = match (trailing_4, last_price) {
        (rate, Some(px)) if px > 0.0 => Some(rate / px),
        _ => None,
    };

    // Mimic Yahoo's `{raw: <num>}` envelopes; the frontend's `rawNum()`
    // helper reads `.raw` first.
    fn raw_f64(x: f64) -> serde_json::Value {
        serde_json::json!({ "raw": x })
    }
    fn raw_i64(x: i64) -> serde_json::Value {
        serde_json::json!({ "raw": x })
    }

    let mut summary = serde_json::Map::new();
    summary.insert("dividendRate".into(), raw_f64(trailing_4));
    if let Some(y) = dividend_yield {
        summary.insert("dividendYield".into(), raw_f64(y));
    }
    summary.insert("lastDividendValue".into(), raw_f64(last.1));
    summary.insert("lastDividendDate".into(), raw_i64(last.0));
    summary.insert("exDividendDate".into(), raw_i64(last.0));

    let mut calendar = serde_json::Map::new();
    calendar.insert("exDividendDate".into(), raw_i64(last.0));

    Ok(serde_json::json!({
        "summaryDetail": serde_json::Value::Object(summary),
        "calendarEvents": serde_json::Value::Object(calendar),
    }))
}

/// Major-holders / institutional-ownership lookup.
///
/// **Status:** no free data source. Yahoo `quoteSummary[majorHoldersBreakdown,
/// institutionOwnership, fundOwnership]` returns 401 ("Invalid Crumb") since
/// late 2023; Finnhub `/stock/ownership` + `/stock/fund-ownership` are
/// **premium-tier only** (free tier returns 403). EDGAR Form 13F is a
/// possible free path but the parser isn't built yet.
///
/// When Yahoo fails, return an empty-but-shaped envelope so
/// `renderHolders` falls through to the empty-cards layout instead of
/// "no data". Frontend can decide how to surface the "needs premium data
/// source" message — keeping the contract here as `Result<Value>` so a
/// future EDGAR-based backend can drop in without changing the route.
pub async fn holders(symbol: &str) -> anyhow::Result<serde_json::Value> {
    match quote_summary(
        symbol,
        &[
            "majorHoldersBreakdown",
            "institutionOwnership",
            "fundOwnership",
        ],
    )
    .await
    {
        Ok(v) => Ok(v),
        Err(e) => {
            tracing::warn!(
                symbol,
                error = %e,
                "holders: Yahoo quoteSummary blocked and no free Finnhub equivalent; returning empty envelope"
            );
            Ok(serde_json::json!({
                "majorHoldersBreakdown": {},
                "institutionOwnership":  { "ownershipList": [] },
                "fundOwnership":         { "ownershipList": [] },
                "_source_note": "holders data requires a premium tier (Finnhub paid, IEX, Polygon) or an EDGAR 13F parser — not available on free providers since Yahoo locked quoteSummary behind crumb auth in late 2023"
            }))
        }
    }
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
