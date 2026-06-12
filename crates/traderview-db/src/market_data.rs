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
    // Cookie+crumb auth — anonymous v10 calls 401 "Invalid Crumb".
    // One retry after invalidating a server-side-expired crumb.
    let url = format!(
        "https://query1.finance.yahoo.com/v10/finance/quoteSummary/{sym}",
        sym = enc(symbol),
    );
    let mut resp = None;
    for attempt in 0..2 {
        let auth = crate::yahoo_auth::get().await?;
        let r = auth
            .client
            .get(&url)
            .query(&[("modules", modules.join(",").as_str()), ("crumb", auth.crumb.as_str())])
            .send()
            .await?;
        if r.status() == reqwest::StatusCode::UNAUTHORIZED && attempt == 0 {
            crate::yahoo_auth::invalidate(&auth.crumb).await;
            continue;
        }
        resp = Some(r);
        break;
    }
    let resp = resp.expect("loop always sets resp on its final iteration");
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
/// unchanged. Falls back to Yahoo `quoteSummary` (cookie+crumb via
/// `yahoo_auth`) if no Finnhub key is configured.
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
/// `research.js::renderEarnings` works unchanged. Falls back to Yahoo
/// `quoteSummary` (cookie+crumb via `yahoo_auth`) when Finnhub fails.
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
/// `{recommendationTrend.trend[]}` envelope. Falls back to Yahoo
/// `quoteSummary` (cookie+crumb via `yahoo_auth`) when Finnhub fails.
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
/// **Backend:** Yahoo `v8/finance/chart` with `events=div,split` — the
/// chart endpoint needs no crumb, so this stays on the anonymous client.
/// (`v10/finance/quoteSummary`, the previous backend, requires the
/// cookie+crumb handshake — handled by `yahoo_auth` where still used.)
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
    // '/' is symbology translation, not escaping: brokers write slash pairs
    // (crypto ETH/USD, class shares BRK/B) where Yahoo uses a dash
    // (ETH-USD, BRK-B). Percent-encoding the slash instead made Yahoo 404,
    // and live_positions dropped every open crypto position as quoteless.
    s.replace('^', "%5E").replace('=', "%3D").replace('/', "-")
}

// ===========================================================================
// Market-wide dividend calendar (Nasdaq)
// ===========================================================================
//
// Yahoo / Finnhub only expose dividends per-symbol, so a market-wide calendar
// would require fanning out across every ticker. Nasdaq publishes a public
// per-date dividend calendar that returns *every* company going ex on a given
// day in a single call, which is what investing.com-style calendars use. We
// loop the requested date range (concurrently) and aggregate.
//
// Nasdaq blocks non-browser User-Agents, so this needs its own client with a
// browser UA + Accept headers (the shared `client()` UA gets dropped).

const NASDAQ_UA: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) \
     AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36";

fn nasdaq_client() -> reqwest::Client {
    use reqwest::header;
    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::ACCEPT,
        header::HeaderValue::from_static("application/json, text/plain, */*"),
    );
    headers.insert(
        header::ACCEPT_LANGUAGE,
        header::HeaderValue::from_static("en-US,en;q=0.9"),
    );
    reqwest::Client::builder()
        .user_agent(NASDAQ_UA)
        .default_headers(headers)
        .timeout(std::time::Duration::from_secs(12))
        .build()
        .unwrap()
}

/// Parse Nasdaq's `M/D/YYYY` date format into ISO `YYYY-MM-DD`. Returns
/// `None` for blanks / `N/A` / unparseable input.
fn nasdaq_date_iso(s: &str) -> Option<String> {
    let s = s.trim();
    if s.is_empty() || s.eq_ignore_ascii_case("N/A") {
        return None;
    }
    let parts: Vec<&str> = s.split('/').collect();
    if parts.len() != 3 {
        return None;
    }
    let m: u32 = parts[0].trim().parse().ok()?;
    let d: u32 = parts[1].trim().parse().ok()?;
    let y: i32 = parts[2].trim().parse().ok()?;
    chrono::NaiveDate::from_ymd_opt(y, m, d).map(|nd| nd.format("%Y-%m-%d").to_string())
}

/// Coerce a Nasdaq numeric field that may arrive as a JSON number, a
/// `$`-prefixed string, or `N/A` into an `f64`.
fn nasdaq_num(v: &serde_json::Value) -> Option<f64> {
    if let Some(n) = v.as_f64() {
        return Some(n);
    }
    let s = v.as_str()?.trim().trim_start_matches('$').replace(',', "");
    if s.is_empty() || s.eq_ignore_ascii_case("N/A") {
        return None;
    }
    s.parse().ok()
}

/// Fetch + normalize a single day's dividend calendar from Nasdaq.
async fn nasdaq_dividend_day(
    client: &reqwest::Client,
    date: chrono::NaiveDate,
) -> anyhow::Result<Vec<serde_json::Value>> {
    let url = format!(
        "https://api.nasdaq.com/api/calendar/dividends?date={}",
        date.format("%Y-%m-%d")
    );
    let resp = client.get(&url).send().await?;
    if !resp.status().is_success() {
        anyhow::bail!("nasdaq dividends HTTP {}", resp.status());
    }
    let v: serde_json::Value = resp.json().await?;
    let rows = match v["data"]["calendar"]["rows"].as_array() {
        Some(r) => r,
        None => return Ok(Vec::new()), // weekend / holiday / no data
    };
    let ex_iso = date.format("%Y-%m-%d").to_string();
    let mut out = Vec::with_capacity(rows.len());
    for r in rows {
        let symbol = match r["symbol"].as_str() {
            Some(s) if !s.trim().is_empty() => s.trim().to_string(),
            _ => continue,
        };
        let amount = nasdaq_num(&r["dividend_Rate"]);
        let annual = nasdaq_num(&r["indicated_Annual_Dividend"]);
        let payments_per_year = match (amount, annual) {
            (Some(a), Some(ann)) if a > 0.0 && ann > 0.0 => {
                let ppy = (ann / a).round();
                if ppy >= 1.0 && ppy <= 52.0 {
                    Some(ppy as i64)
                } else {
                    None
                }
            }
            _ => None,
        };
        out.push(serde_json::json!({
            "symbol": symbol,
            "company": r["companyName"].as_str().unwrap_or("").trim(),
            "ex_date": ex_iso,
            "pay_date": r["payment_Date"].as_str().and_then(nasdaq_date_iso),
            "record_date": r["record_Date"].as_str().and_then(nasdaq_date_iso),
            "announcement_date": r["announcement_Date"].as_str().and_then(nasdaq_date_iso),
            "amount": amount,
            "annual_dividend": annual,
            "payments_per_year": payments_per_year,
        }));
    }
    Ok(out)
}

/// Market-wide upcoming dividend calendar for the inclusive date range
/// `[from, to]`. Aggregates Nasdaq's per-date feed (fetched concurrently)
/// and returns rows sorted by ex-date then symbol.
pub async fn dividends_calendar(
    from: chrono::NaiveDate,
    to: chrono::NaiveDate,
) -> anyhow::Result<serde_json::Value> {
    use futures_util::{stream, StreamExt};

    let span = (to - from).num_days().max(0);
    let dates: Vec<chrono::NaiveDate> = (0..=span)
        .filter_map(|i| from.checked_add_signed(chrono::Duration::days(i)))
        .collect();

    let client = nasdaq_client();
    let per_day: Vec<Vec<serde_json::Value>> = stream::iter(dates.into_iter().map(|date| {
        let client = client.clone();
        async move {
            nasdaq_dividend_day(&client, date)
                .await
                .unwrap_or_else(|e| {
                    tracing::warn!(%date, error = %e, "nasdaq dividend day failed");
                    Vec::new()
                })
        }
    }))
    .buffer_unordered(6)
    .collect()
    .await;

    let mut rows: Vec<serde_json::Value> = per_day.into_iter().flatten().collect();
    rows.sort_by(|a, b| {
        let ax = a["ex_date"].as_str().unwrap_or("");
        let bx = b["ex_date"].as_str().unwrap_or("");
        ax.cmp(bx).then_with(|| {
            a["symbol"]
                .as_str()
                .unwrap_or("")
                .cmp(b["symbol"].as_str().unwrap_or(""))
        })
    });

    Ok(serde_json::json!({
        "from": from.format("%Y-%m-%d").to_string(),
        "to": to.format("%Y-%m-%d").to_string(),
        "count": rows.len(),
        "rows": rows,
    }))
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
    fn enc_maps_slash_pairs_to_yahoo_dash() {
        // Broker slash symbology (crypto pairs, class shares) maps to
        // Yahoo's dash form — translation, not percent-encoding. ETH%2FUSD
        // 404s on Yahoo and Live P/L dropped the position entirely.
        assert_eq!(enc("ETH/USD"), "ETH-USD");
        assert_eq!(enc("ADA/USD"), "ADA-USD");
        assert_eq!(enc("BRK/B"), "BRK-B");
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

    // ── Nasdaq dividend-calendar helpers ──────────────────────────────────

    #[test]
    fn nasdaq_date_iso_converts_mdy_to_iso() {
        assert_eq!(nasdaq_date_iso("6/9/2026").as_deref(), Some("2026-06-09"));
        assert_eq!(nasdaq_date_iso("12/25/2026").as_deref(), Some("2026-12-25"));
    }

    #[test]
    fn nasdaq_date_iso_rejects_blank_na_and_garbage() {
        assert_eq!(nasdaq_date_iso(""), None);
        assert_eq!(nasdaq_date_iso("N/A"), None);
        assert_eq!(nasdaq_date_iso("not-a-date"), None);
        assert_eq!(nasdaq_date_iso("13/40/2026"), None);
    }

    #[test]
    fn nasdaq_num_handles_number_string_and_na() {
        use serde_json::json;
        assert_eq!(nasdaq_num(&json!(0.27)), Some(0.27));
        assert_eq!(nasdaq_num(&json!("$1.08")), Some(1.08));
        assert_eq!(nasdaq_num(&json!("1,234.5")), Some(1234.5));
        assert_eq!(nasdaq_num(&json!("N/A")), None);
        assert_eq!(nasdaq_num(&json!("")), None);
        assert_eq!(nasdaq_num(&serde_json::Value::Null), None);
    }
}
