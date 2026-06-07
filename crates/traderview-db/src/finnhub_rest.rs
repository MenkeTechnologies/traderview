//! Finnhub REST client + Yahoo-shape adapters.
//!
//! Covers the free-tier endpoints that fix the broken Yahoo `quoteSummary`
//! calls:
//!   * `/quote`                    — last/change/day-high/low
//!   * `/stock/profile2`           — name/sector/industry/employees
//!   * `/stock/metric?metric=all`  — fundamentals (PE/PB/52w/etc.)
//!   * `/stock/recommendation`     — analyst trend
//!   * `/stock/eps-surprise`       — per-period EPS actual/estimate/surprise
//!   * `/calendar/earnings`        — calendar events (next earnings date)
//!   * `/calendar/ipo`             — IPO calendar
//!   * `/company-news`             — per-symbol news with `from`/`to`
//!   * `/news?category=general`    — broad market news
//!
//! Each "Yahoo-shape" adapter (`fundamentals_yahoo_shape`,
//! `earnings_yahoo_shape`, `recommendations_yahoo_shape`) returns JSON
//! that matches what the existing frontend extractors already parse
//! (Yahoo's nested `{raw, fmt}` envelopes), so widgets work unchanged.
//!
//! Key resolution order:
//!   1. Process-memory slot in `live_ticks::global()` (settings POST sets this)
//!   2. `FINNHUB_API_KEY` env var
//!   3. None → `not_configured` error
//!
//! Free-tier rate limit is 60 calls/min. This module makes no attempt to
//! throttle — backend callers should cache per-symbol on their own
//! cadence. The Finnhub server returns 429 if you bust the cap.

use serde::Deserialize;
use serde_json::{json, Value};

use crate::live_ticks;

const FINNHUB_BASE: &str = "https://finnhub.io/api/v1";

pub fn ua() -> &'static str {
    "Mozilla/5.0 (compatible; traderview/0.1; +https://github.com/MenkeTechnologies/traderview)"
}

/// Three-tier key resolution: in-memory → env → None.
pub async fn resolve_key() -> anyhow::Result<String> {
    if let Some(k) = live_ticks::global().api_key().await {
        if !k.is_empty() {
            return Ok(k);
        }
    }
    if let Ok(k) = std::env::var("FINNHUB_API_KEY") {
        if !k.is_empty() {
            return Ok(k);
        }
    }
    anyhow::bail!(
        "finnhub api key not configured; set FINNHUB_API_KEY env or save in Settings → Data Sources"
    );
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent(ua())
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .expect("reqwest client build")
}

async fn get_json(path: &str, query: &[(&str, &str)]) -> anyhow::Result<Value> {
    let key = resolve_key().await?;
    let url = format!("{FINNHUB_BASE}{path}");
    let resp = client()
        .get(&url)
        .query(query)
        .query(&[("token", &key)])
        .send()
        .await?;
    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        // Finnhub returns 403 Forbidden for endpoints not included in
        // the caller's plan. Mark these with a sentinel prefix so
        // route handlers can map to ApiError::Forbidden (HTTP 403)
        // instead of a generic 500. The frontend then renders a
        // "premium required" affordance rather than a server-error
        // toast.
        if status == reqwest::StatusCode::FORBIDDEN {
            anyhow::bail!(
                "FINNHUB_PREMIUM_REQUIRED: {path} — your Finnhub plan does not include this endpoint. Body: {}",
                body.chars().take(300).collect::<String>(),
            );
        }
        anyhow::bail!(
            "finnhub {path} HTTP {status}: {}",
            body.chars().take(300).collect::<String>()
        );
    }
    Ok(resp.json().await?)
}

/// Convert an `anyhow::Error` from a Finnhub call into the right HTTP
/// status. Caller's plan doesn't include the endpoint → 403; anything
/// else → 500 with the original cause attached. Pub so the
/// `finnhub_extras` route module can use it without duplicating the
/// sentinel-string check.
pub fn is_premium_required(err: &anyhow::Error) -> bool {
    err.to_string().starts_with("FINNHUB_PREMIUM_REQUIRED")
}

// ──────────────────────────────────────────────────────────────────────
// Raw endpoint wrappers (return Finnhub-shaped JSON verbatim).
// ──────────────────────────────────────────────────────────────────────

pub async fn quote(symbol: &str) -> anyhow::Result<Value> {
    get_json("/quote", &[("symbol", symbol)]).await
}
pub async fn profile2(symbol: &str) -> anyhow::Result<Value> {
    get_json("/stock/profile2", &[("symbol", symbol)]).await
}
pub async fn metric_all(symbol: &str) -> anyhow::Result<Value> {
    get_json("/stock/metric", &[("symbol", symbol), ("metric", "all")]).await
}
/// Time series of short-interest snapshots between two settlement
/// dates. Free-tier endpoint per Finnhub's docs / announcements.
/// Returns `{ symbol, data: [{ settlementDate, shortInterest }, ...] }`
/// — newest record first.
pub async fn stock_short_interest(
    symbol: &str,
    from: &str,
    to: &str,
) -> anyhow::Result<Value> {
    get_json(
        "/stock/short-interest",
        &[("symbol", symbol), ("from", from), ("to", to)],
    )
    .await
}
pub async fn recommendation(symbol: &str) -> anyhow::Result<Value> {
    get_json("/stock/recommendation", &[("symbol", symbol)]).await
}
pub async fn eps_surprise(symbol: &str) -> anyhow::Result<Value> {
    get_json("/stock/eps-surprise", &[("symbol", symbol)]).await
}
pub async fn earnings_calendar(
    from: &str,
    to: &str,
    symbol: Option<&str>,
) -> anyhow::Result<Value> {
    let mut q: Vec<(&str, &str)> = vec![("from", from), ("to", to)];
    if let Some(s) = symbol {
        q.push(("symbol", s));
    }
    get_json("/calendar/earnings", &q).await
}
pub async fn ipo_calendar(from: &str, to: &str) -> anyhow::Result<Value> {
    get_json("/calendar/ipo", &[("from", from), ("to", to)]).await
}
pub async fn company_news(symbol: &str, from: &str, to: &str) -> anyhow::Result<Value> {
    get_json(
        "/company-news",
        &[("symbol", symbol), ("from", from), ("to", to)],
    )
    .await
}
pub async fn general_news(category: &str) -> anyhow::Result<Value> {
    get_json("/news", &[("category", category)]).await
}
pub async fn financials_reported(symbol: &str) -> anyhow::Result<Value> {
    get_json("/stock/financials-reported", &[("symbol", symbol)]).await
}
pub async fn peers(symbol: &str) -> anyhow::Result<Value> {
    get_json("/stock/peers", &[("symbol", symbol)]).await
}
pub async fn upgrade_downgrade(symbol: &str) -> anyhow::Result<Value> {
    get_json("/stock/upgrade-downgrade", &[("symbol", symbol)]).await
}

// ──────────────────────────────────────────────────────────────────────
// Yahoo-shape adapters — return `{summaryDetail: …, defaultKeyStatistics: …,
// financialData: …, assetProfile: …}` so the existing frontend extractors
// in research.js work unchanged.
// ──────────────────────────────────────────────────────────────────────

/// Wrap a numeric value as Yahoo's `{raw, fmt}` envelope, or null when
/// the value is missing / NaN.
fn raw(v: Option<f64>) -> Value {
    match v {
        Some(n) if n.is_finite() => json!({ "raw": n }),
        _ => Value::Null,
    }
}
fn raw_int(v: Option<i64>) -> Value {
    match v {
        Some(n) => json!({ "raw": n }),
        None => Value::Null,
    }
}

#[derive(Deserialize)]
struct ProfileResp {
    name: Option<String>,
    ticker: Option<String>,
    exchange: Option<String>,
    country: Option<String>,
    currency: Option<String>,
    #[serde(rename = "finnhubIndustry")]
    industry: Option<String>,
    #[serde(rename = "shareOutstanding")]
    share_outstanding: Option<f64>,
    #[serde(rename = "marketCapitalization")]
    market_cap_m_usd: Option<f64>,
    ipo: Option<String>,
    weburl: Option<String>,
    phone: Option<String>,
    logo: Option<String>,
}

/// Build the Yahoo `quoteSummary`-shaped fundamentals envelope from
/// `/stock/profile2` + `/stock/metric?metric=all`. Frontend's research
/// view reads:
///   * `summaryDetail.{marketCap,trailingPE,forwardPE,dividendYield,
///                     fiftyTwoWeekHigh,fiftyTwoWeekLow,beta}`
///   * `defaultKeyStatistics.{pegRatio,priceToBook,trailingEps}`
///   * `financialData.{profitMargins,totalRevenue,debtToEquity,returnOnEquity}`
///   * `assetProfile.{sector,industry,fullTimeEmployees,longBusinessSummary}`
pub async fn fundamentals_yahoo_shape(symbol: &str) -> anyhow::Result<Value> {
    // Run in parallel — both are independent Finnhub calls.
    let (prof_v, met_v) = tokio::join!(profile2(symbol), metric_all(symbol));
    let prof_v = prof_v?;
    let met_v = met_v?;
    let prof: ProfileResp = serde_json::from_value(prof_v.clone()).unwrap_or(ProfileResp {
        name: None,
        ticker: None,
        exchange: None,
        country: None,
        currency: None,
        industry: None,
        share_outstanding: None,
        market_cap_m_usd: None,
        ipo: None,
        weburl: None,
        phone: None,
        logo: None,
    });
    let m = met_v.get("metric").cloned().unwrap_or(Value::Null);
    let mf = |k: &str| m.get(k).and_then(|x| x.as_f64());

    let summary_detail = json!({
        // Finnhub returns market cap in *millions* USD; Yahoo's marketCap
        // is the raw figure, so multiply by 1e6.
        "marketCap":          raw(prof.market_cap_m_usd.map(|m| m * 1e6)),
        "trailingPE":         raw(mf("peTTM").or_else(|| mf("peNormalizedAnnual"))),
        "forwardPE":          raw(mf("peInclExtraTTM")),
        "dividendYield":      raw(mf("currentDividendYieldTTM").map(|y| y / 100.0)),
        "dividendRate":       raw(mf("dividendPerShareAnnual")),
        "fiftyTwoWeekHigh":   raw(mf("52WeekHigh")),
        "fiftyTwoWeekLow":    raw(mf("52WeekLow")),
        "beta":               raw(mf("beta")),
        "payoutRatio":        raw(mf("payoutRatioTTM").map(|p| p / 100.0)),
    });
    let key_stats = json!({
        "pegRatio":     raw(mf("pegRatio").or_else(|| mf("pegAnnual"))),
        "priceToBook":  raw(mf("pbAnnual").or_else(|| mf("pbQuarterly"))),
        "trailingEps":  raw(mf("epsTTM").or_else(|| mf("epsBasicExclExtraItemsTTM"))),
    });
    let financial_data = json!({
        "profitMargins":   raw(mf("netProfitMarginTTM").map(|m| m / 100.0)),
        "totalRevenue":    raw(mf("revenueTTM")),
        "debtToEquity":    raw(mf("totalDebt/totalEquityAnnual").or_else(|| mf("totalDebt/totalEquityQuarterly"))),
        "returnOnEquity":  raw(mf("roeTTM").map(|m| m / 100.0)),
    });
    let asset_profile = json!({
        "sector":   prof.industry.clone(),
        "industry": prof.industry.clone(),
        "fullTimeEmployees": Value::Null,  // Finnhub free tier doesn't expose
        "longBusinessSummary": Value::Null, // ditto
        "country":  prof.country,
        "phone":    prof.phone,
        "website":  prof.weburl,
        "logo_url": prof.logo,
        "ipo":      prof.ipo,
        "shareOutstanding": raw(prof.share_outstanding),
        "name":     prof.name,
        "ticker":   prof.ticker,
        "exchange": prof.exchange,
        "currency": prof.currency,
    });
    Ok(json!({
        "summaryDetail":         summary_detail,
        "defaultKeyStatistics":  key_stats,
        "financialData":         financial_data,
        "assetProfile":          asset_profile,
    }))
}

#[derive(Deserialize)]
struct EpsItem {
    period: Option<String>,
    actual: Option<f64>,
    estimate: Option<f64>,
    surprise: Option<f64>,
    #[serde(rename = "surprisePercent")]
    surprise_percent: Option<f64>,
}

#[derive(Deserialize)]
struct EarningsCalRow {
    date: Option<String>,
    #[serde(rename = "epsEstimate")]
    eps_estimate: Option<f64>,
    #[serde(rename = "revenueEstimate")]
    revenue_estimate: Option<f64>,
}

/// Yahoo earnings envelope:
///   * `earningsHistory.history[]` ← from `/stock/eps-surprise`
///   * `calendarEvents.earnings.{earningsDate, earningsAverage, revenueAverage}`
///     ← from `/calendar/earnings?symbol=&from=today&to=+1y`
pub async fn earnings_yahoo_shape(symbol: &str) -> anyhow::Result<Value> {
    let today = chrono::Utc::now().date_naive();
    let next_year = today + chrono::Duration::days(365);
    let from_str = today.to_string();
    let to_str = next_year.to_string();
    let (eps_v, cal_v) = tokio::join!(
        eps_surprise(symbol),
        earnings_calendar(&from_str, &to_str, Some(symbol)),
    );
    // `eps_surprise` and `earnings_calendar` are independent slices: the
    // first feeds the historical EPS table, the second feeds the
    // "next earnings" cards. Don't let one failing endpoint blank the
    // entire Earnings panel — log + fall through with an empty Value so
    // whichever side succeeded still renders. The calendar is the more
    // useful slice (next-quarter ETA + estimates), and `eps_surprise`
    // tends to be the one that rate-limits on Finnhub's free tier.
    let eps_v = eps_v.unwrap_or_else(|e| {
        tracing::warn!(symbol, error = %e, "eps_surprise failed; rendering earnings without history");
        Value::Null
    });
    let cal_v = cal_v.unwrap_or_else(|e| {
        tracing::warn!(symbol, error = %e, "earnings_calendar failed; rendering earnings without next-quarter");
        Value::Null
    });

    let history: Vec<EpsItem> = serde_json::from_value(eps_v.clone()).unwrap_or_default();
    let yahoo_history: Vec<Value> = history
        .iter()
        .map(|h| {
            json!({
                "period":          h.period.clone().unwrap_or_default(),
                "epsEstimate":     raw(h.estimate),
                "epsActual":       raw(h.actual),
                "epsDifference":   raw(h.surprise),
                "surprisePercent": raw(h.surprise_percent),
            })
        })
        .collect();

    // Next earnings: take the first calendar row >= today.
    let cal_rows: Vec<EarningsCalRow> = cal_v
        .get("earningsCalendar")
        .cloned()
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();
    let next = cal_rows.into_iter().next();
    let (next_date_unix, eps_est, rev_est) = match next {
        Some(r) => {
            let dt = r
                .date
                .as_deref()
                .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
                .and_then(|d| d.and_hms_opt(0, 0, 0))
                .map(|ndt| ndt.and_utc().timestamp());
            (dt, r.eps_estimate, r.revenue_estimate)
        }
        None => (None, None, None),
    };

    Ok(json!({
        "earningsHistory": { "history": yahoo_history },
        "calendarEvents": {
            "earnings": {
                "earningsDate":    [ raw_int(next_date_unix) ],
                "earningsAverage": raw(eps_est),
                "revenueAverage":  raw(rev_est),
            }
        }
    }))
}

#[derive(Deserialize)]
struct RecRow {
    period: Option<String>,
    #[serde(rename = "strongBuy")]
    strong_buy: Option<i64>,
    buy: Option<i64>,
    hold: Option<i64>,
    sell: Option<i64>,
    #[serde(rename = "strongSell")]
    strong_sell: Option<i64>,
}

/// Wrap Finnhub's flat recommendation array as Yahoo's
/// `{recommendationTrend: {trend: [...]}}` shape so research.js renders it.
pub async fn recommendations_yahoo_shape(symbol: &str) -> anyhow::Result<Value> {
    let v = recommendation(symbol).await?;
    let rows: Vec<RecRow> = serde_json::from_value(v).unwrap_or_default();
    let trend: Vec<Value> = rows
        .into_iter()
        .map(|r| {
            json!({
                "period":     r.period.unwrap_or_default(),
                "strongBuy":  r.strong_buy,
                "buy":        r.buy,
                "hold":       r.hold,
                "sell":       r.sell,
                "strongSell": r.strong_sell,
            })
        })
        .collect();
    Ok(json!({ "recommendationTrend": { "trend": trend } }))
}

#[derive(Deserialize)]
struct InsiderTxRow {
    name: Option<String>,
    share: Option<i64>,
    change: Option<i64>,
    #[serde(rename = "filingDate")]
    filing_date: Option<String>,
    #[serde(rename = "transactionDate")]
    transaction_date: Option<String>,
    #[serde(rename = "transactionCode")]
    transaction_code: Option<String>,
    #[serde(rename = "transactionPrice")]
    transaction_price: Option<f64>,
}

/// Wrap Finnhub's `/stock/insider-transactions` (FREE tier) as Yahoo's
/// `{insiderTransactions: {transactions: [...]}}` shape so
/// `research.js::renderInsiders` works unchanged. Was previously routed
/// through Yahoo `quoteSummary[insiderTransactions]` which 401s with
/// "Invalid Crumb" since late 2023, so the Insider Activity panel
/// always rendered "no data".
///
/// Field mapping (Finnhub → Yahoo):
///   - `transactionDate`  → `startDate.{raw:unix, fmt:"YYYY-MM-DD"}`
///   - `name`             → `filerName`
///   - `filerRelation`    → empty string (Finnhub does not expose role)
///   - `transactionCode`  → `transactionText` (S=Sale, P=Purchase, …)
///   - `share` (abs)      → `shares.{raw, fmt}`
///   - share × price      → `value.{raw, fmt}`
pub async fn insiders_yahoo_shape(symbol: &str) -> anyhow::Result<Value> {
    // 90-day window matches Form 4 retention on Finnhub's free tier.
    let today = chrono::Utc::now().date_naive();
    let from = (today - chrono::Duration::days(90)).to_string();
    let to = today.to_string();
    let v = insider_transactions(symbol, &from, &to).await?;
    let rows: Vec<InsiderTxRow> = v
        .get("data")
        .cloned()
        .and_then(|d| serde_json::from_value(d).ok())
        .unwrap_or_default();
    let mut txs: Vec<Value> = Vec::with_capacity(rows.len());
    for r in rows {
        let shares_abs = r.share.unwrap_or(0).abs();
        let value = match (r.share, r.transaction_price) {
            (Some(s), Some(p)) if p.is_finite() => Some((s.abs() as f64) * p),
            _ => None,
        };
        // `change > 0` → acquisition; `change < 0` → disposition. Mirror
        // Yahoo's freeform transactionText so the column reads naturally.
        let tx_text = match (r.transaction_code.as_deref(), r.change) {
            (Some(code), _) => code.to_string(),
            (None, Some(c)) if c > 0 => "Buy".into(),
            (None, Some(c)) if c < 0 => "Sale".into(),
            _ => String::new(),
        };
        let date_unix = r
            .transaction_date
            .as_deref()
            .or(r.filing_date.as_deref())
            .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
            .and_then(|d| d.and_hms_opt(0, 0, 0))
            .map(|ndt| ndt.and_utc().timestamp());
        let date_fmt = r
            .transaction_date
            .clone()
            .or(r.filing_date.clone())
            .unwrap_or_default();
        txs.push(json!({
            "startDate":       { "raw": date_unix, "fmt": date_fmt },
            "filerName":       r.name.unwrap_or_default(),
            "filerRelation":   "",
            "transactionText": tx_text,
            "shares":          { "raw": shares_abs },
            "value":           raw(value),
        }));
    }
    Ok(json!({ "insiderTransactions": { "transactions": txs } }))
}

// ──────────────────────────────────────────────────────────────────────
// Comprehensive port of every documented Finnhub REST endpoint. Premium-
// only endpoints are included so the wiring is complete; calls return 403
// on a free-tier key, which the caller surfaces as a 500. Frontend views
// for premium-only data are intentionally omitted.
//
// Each wrapper is a thin `get_json` call — response shapes pass through
// to the caller verbatim. Yahoo-shape adapters above remain the only
// places where we synthesize an envelope to match the legacy extractor.
// ──────────────────────────────────────────────────────────────────────

// ───── Stock — per-symbol fundamentals & ownership ─────
pub async fn company_profile_legacy(symbol: &str) -> anyhow::Result<Value> {
    get_json("/stock/profile", &[("symbol", symbol)]).await
}
pub async fn company_executive(symbol: &str) -> anyhow::Result<Value> {
    get_json("/stock/executive", &[("symbol", symbol)]).await
}
pub async fn stock_dividends(symbol: &str, from: &str, to: &str) -> anyhow::Result<Value> {
    get_json(
        "/stock/dividend",
        &[("symbol", symbol), ("from", from), ("to", to)],
    )
    .await
}
pub async fn stock_basic_dividends(symbol: &str) -> anyhow::Result<Value> {
    get_json("/stock/dividend2", &[("symbol", symbol)]).await
}
pub async fn stock_splits(symbol: &str, from: &str, to: &str) -> anyhow::Result<Value> {
    get_json(
        "/stock/split",
        &[("symbol", symbol), ("from", from), ("to", to)],
    )
    .await
}
pub async fn price_target(symbol: &str) -> anyhow::Result<Value> {
    get_json("/stock/price-target", &[("symbol", symbol)]).await
}
pub async fn option_chain(symbol: &str) -> anyhow::Result<Value> {
    get_json("/stock/option-chain", &[("symbol", symbol)]).await
}
pub async fn financials(symbol: &str, statement: &str, freq: &str) -> anyhow::Result<Value> {
    get_json(
        "/stock/financials",
        &[("symbol", symbol), ("statement", statement), ("freq", freq)],
    )
    .await
}
pub async fn fund_ownership(symbol: &str, limit: i64) -> anyhow::Result<Value> {
    get_json(
        "/stock/fund-ownership",
        &[("symbol", symbol), ("limit", &limit.to_string())],
    )
    .await
}
pub async fn ownership(symbol: &str, limit: i64) -> anyhow::Result<Value> {
    get_json(
        "/stock/ownership",
        &[("symbol", symbol), ("limit", &limit.to_string())],
    )
    .await
}
pub async fn company_earnings(symbol: &str, limit: i64) -> anyhow::Result<Value> {
    get_json(
        "/stock/earnings",
        &[("symbol", symbol), ("limit", &limit.to_string())],
    )
    .await
}
pub async fn revenue_estimate(symbol: &str, freq: &str) -> anyhow::Result<Value> {
    get_json(
        "/stock/revenue-estimate",
        &[("symbol", symbol), ("freq", freq)],
    )
    .await
}
pub async fn ebitda_estimate(symbol: &str, freq: &str) -> anyhow::Result<Value> {
    get_json(
        "/stock/ebitda-estimate",
        &[("symbol", symbol), ("freq", freq)],
    )
    .await
}
pub async fn ebit_estimate(symbol: &str, freq: &str) -> anyhow::Result<Value> {
    get_json(
        "/stock/ebit-estimate",
        &[("symbol", symbol), ("freq", freq)],
    )
    .await
}
pub async fn eps_estimate(symbol: &str, freq: &str) -> anyhow::Result<Value> {
    get_json("/stock/eps-estimate", &[("symbol", symbol), ("freq", freq)]).await
}
pub async fn net_income_estimate(symbol: &str, freq: &str) -> anyhow::Result<Value> {
    get_json(
        "/stock/net-income-estimate",
        &[("symbol", symbol), ("freq", freq)],
    )
    .await
}
pub async fn pretax_income_estimate(symbol: &str, freq: &str) -> anyhow::Result<Value> {
    get_json(
        "/stock/pretax-income-estimate",
        &[("symbol", symbol), ("freq", freq)],
    )
    .await
}
pub async fn gross_income_estimate(symbol: &str, freq: &str) -> anyhow::Result<Value> {
    get_json(
        "/stock/gross-income-estimate",
        &[("symbol", symbol), ("freq", freq)],
    )
    .await
}
pub async fn dps_estimate(symbol: &str, freq: &str) -> anyhow::Result<Value> {
    get_json("/stock/dps-estimate", &[("symbol", symbol), ("freq", freq)]).await
}
pub async fn filings(
    symbol: &str,
    from: &str,
    to: &str,
    form: Option<&str>,
) -> anyhow::Result<Value> {
    let mut q = vec![("symbol", symbol), ("from", from), ("to", to)];
    if let Some(f) = form {
        q.push(("form", f));
    }
    get_json("/stock/filings", &q).await
}
pub async fn international_filings(symbol: &str, country: &str) -> anyhow::Result<Value> {
    get_json(
        "/stock/international-filings",
        &[("symbol", symbol), ("country", country)],
    )
    .await
}
pub async fn sec_sentiment(access_number: &str) -> anyhow::Result<Value> {
    get_json(
        "/stock/filings-sentiment",
        &[("accessNumber", access_number)],
    )
    .await
}
pub async fn similarity_index(symbol: &str, freq: &str) -> anyhow::Result<Value> {
    get_json(
        "/stock/similarity-index",
        &[("symbol", symbol), ("freq", freq)],
    )
    .await
}
pub async fn transcripts_list(symbol: &str) -> anyhow::Result<Value> {
    get_json("/stock/transcripts/list", &[("symbol", symbol)]).await
}
pub async fn transcripts(id: &str) -> anyhow::Result<Value> {
    get_json("/stock/transcripts", &[("id", id)]).await
}
pub async fn stock_candles(
    symbol: &str,
    resolution: &str,
    from: &str,
    to: &str,
) -> anyhow::Result<Value> {
    get_json(
        "/stock/candle",
        &[
            ("symbol", symbol),
            ("resolution", resolution),
            ("from", from),
            ("to", to),
        ],
    )
    .await
}
pub async fn stock_tick(symbol: &str, date: &str, limit: i64, skip: i64) -> anyhow::Result<Value> {
    get_json(
        "/stock/tick",
        &[
            ("symbol", symbol),
            ("date", date),
            ("limit", &limit.to_string()),
            ("skip", &skip.to_string()),
        ],
    )
    .await
}
pub async fn stock_nbbo(symbol: &str, date: &str, limit: i64, skip: i64) -> anyhow::Result<Value> {
    get_json(
        "/stock/bbo",
        &[
            ("symbol", symbol),
            ("date", date),
            ("limit", &limit.to_string()),
            ("skip", &skip.to_string()),
        ],
    )
    .await
}
pub async fn last_bid_ask(symbol: &str) -> anyhow::Result<Value> {
    get_json("/stock/bidask", &[("symbol", symbol)]).await
}

// ───── Stock — alternative / regulatory / ESG ─────
pub async fn insider_transactions(symbol: &str, from: &str, to: &str) -> anyhow::Result<Value> {
    get_json(
        "/stock/insider-transactions",
        &[("symbol", symbol), ("from", from), ("to", to)],
    )
    .await
}
pub async fn insider_sentiment(symbol: &str, from: &str, to: &str) -> anyhow::Result<Value> {
    get_json(
        "/stock/insider-sentiment",
        &[("symbol", symbol), ("from", from), ("to", to)],
    )
    .await
}
pub async fn lobbying(symbol: &str, from: &str, to: &str) -> anyhow::Result<Value> {
    get_json(
        "/stock/lobbying",
        &[("symbol", symbol), ("from", from), ("to", to)],
    )
    .await
}
pub async fn usa_spending(symbol: &str, from: &str, to: &str) -> anyhow::Result<Value> {
    get_json(
        "/stock/usa-spending",
        &[("symbol", symbol), ("from", from), ("to", to)],
    )
    .await
}
pub async fn visa_application(symbol: &str, from: &str, to: &str) -> anyhow::Result<Value> {
    get_json(
        "/stock/visa-application",
        &[("symbol", symbol), ("from", from), ("to", to)],
    )
    .await
}
pub async fn uspto_patent(symbol: &str, from: &str, to: &str) -> anyhow::Result<Value> {
    get_json(
        "/stock/uspto-patent",
        &[("symbol", symbol), ("from", from), ("to", to)],
    )
    .await
}
pub async fn supply_chain(symbol: &str) -> anyhow::Result<Value> {
    get_json("/stock/supply-chain", &[("symbol", symbol)]).await
}
pub async fn social_sentiment(symbol: &str, from: &str, to: &str) -> anyhow::Result<Value> {
    get_json(
        "/stock/social-sentiment",
        &[("symbol", symbol), ("from", from), ("to", to)],
    )
    .await
}
pub async fn investment_theme(theme: &str) -> anyhow::Result<Value> {
    get_json("/stock/investment-theme", &[("theme", theme)]).await
}
pub async fn esg_score(symbol: &str) -> anyhow::Result<Value> {
    get_json("/stock/esg", &[("symbol", symbol)]).await
}
pub async fn historical_esg(symbol: &str) -> anyhow::Result<Value> {
    get_json("/stock/historical-esg", &[("symbol", symbol)]).await
}
pub async fn historical_market_cap(symbol: &str, from: &str, to: &str) -> anyhow::Result<Value> {
    get_json(
        "/stock/historical-market-cap",
        &[("symbol", symbol), ("from", from), ("to", to)],
    )
    .await
}
pub async fn historical_employee_count(
    symbol: &str,
    from: &str,
    to: &str,
) -> anyhow::Result<Value> {
    get_json(
        "/stock/historical-employee-count",
        &[("symbol", symbol), ("from", from), ("to", to)],
    )
    .await
}
pub async fn earnings_quality_score(symbol: &str, freq: &str) -> anyhow::Result<Value> {
    get_json(
        "/stock/earnings-quality-score",
        &[("symbol", symbol), ("freq", freq)],
    )
    .await
}
pub async fn revenue_breakdown(symbol: &str) -> anyhow::Result<Value> {
    get_json("/stock/revenue-breakdown", &[("symbol", symbol)]).await
}
pub async fn revenue_breakdown2(symbol: &str) -> anyhow::Result<Value> {
    get_json("/stock/revenue-breakdown2", &[("symbol", symbol)]).await
}
pub async fn presentation(symbol: &str) -> anyhow::Result<Value> {
    get_json("/stock/presentation", &[("symbol", symbol)]).await
}
pub async fn newsroom(symbol: &str, from: &str, to: &str) -> anyhow::Result<Value> {
    get_json(
        "/stock/newsroom",
        &[("symbol", symbol), ("from", from), ("to", to)],
    )
    .await
}
pub async fn congressional_trading(symbol: &str, from: &str, to: &str) -> anyhow::Result<Value> {
    get_json(
        "/stock/congressional-trading",
        &[("symbol", symbol), ("from", from), ("to", to)],
    )
    .await
}
pub async fn price_metric(symbol: &str, date: Option<&str>) -> anyhow::Result<Value> {
    let mut q = vec![("symbol", symbol)];
    if let Some(d) = date {
        q.push(("date", d));
    }
    get_json("/stock/price-metric", &q).await
}
pub async fn earnings_call_live(
    from: &str,
    to: &str,
    symbol: Option<&str>,
) -> anyhow::Result<Value> {
    let mut q = vec![("from", from), ("to", to)];
    if let Some(s) = symbol {
        q.push(("symbol", s));
    }
    get_json("/stock/earnings-call-live", &q).await
}

// ───── News & sentiment ─────
pub async fn news_sentiment(symbol: &str) -> anyhow::Result<Value> {
    get_json("/news-sentiment", &[("symbol", symbol)]).await
}
pub async fn press_releases(symbol: &str, from: &str, to: &str) -> anyhow::Result<Value> {
    get_json(
        "/press-releases",
        &[("symbol", symbol), ("from", from), ("to", to)],
    )
    .await
}

// ───── Scan ─────
pub async fn pattern_recognition(symbol: &str, resolution: &str) -> anyhow::Result<Value> {
    get_json(
        "/scan/pattern",
        &[("symbol", symbol), ("resolution", resolution)],
    )
    .await
}
pub async fn support_resistance(symbol: &str, resolution: &str) -> anyhow::Result<Value> {
    get_json(
        "/scan/support-resistance",
        &[("symbol", symbol), ("resolution", resolution)],
    )
    .await
}
pub async fn aggregate_indicator(symbol: &str, resolution: &str) -> anyhow::Result<Value> {
    get_json(
        "/scan/technical-indicator",
        &[("symbol", symbol), ("resolution", resolution)],
    )
    .await
}
pub async fn technical_indicator(
    symbol: &str,
    resolution: &str,
    from: &str,
    to: &str,
    indicator: &str,
) -> anyhow::Result<Value> {
    get_json(
        "/indicator",
        &[
            ("symbol", symbol),
            ("resolution", resolution),
            ("from", from),
            ("to", to),
            ("indicator", indicator),
        ],
    )
    .await
}

// ───── Forex / Crypto ─────
pub async fn forex_exchanges() -> anyhow::Result<Value> {
    get_json("/forex/exchange", &[]).await
}
pub async fn forex_symbols(exchange: &str) -> anyhow::Result<Value> {
    get_json("/forex/symbol", &[("exchange", exchange)]).await
}
pub async fn forex_rates(base: &str) -> anyhow::Result<Value> {
    get_json("/forex/rates", &[("base", base)]).await
}
pub async fn forex_candles(
    symbol: &str,
    resolution: &str,
    from: &str,
    to: &str,
) -> anyhow::Result<Value> {
    get_json(
        "/forex/candle",
        &[
            ("symbol", symbol),
            ("resolution", resolution),
            ("from", from),
            ("to", to),
        ],
    )
    .await
}
pub async fn crypto_exchanges() -> anyhow::Result<Value> {
    get_json("/crypto/exchange", &[]).await
}
pub async fn crypto_symbols(exchange: &str) -> anyhow::Result<Value> {
    get_json("/crypto/symbol", &[("exchange", exchange)]).await
}
pub async fn crypto_candles(
    symbol: &str,
    resolution: &str,
    from: &str,
    to: &str,
) -> anyhow::Result<Value> {
    get_json(
        "/crypto/candle",
        &[
            ("symbol", symbol),
            ("resolution", resolution),
            ("from", from),
            ("to", to),
        ],
    )
    .await
}
pub async fn crypto_profile(symbol: &str) -> anyhow::Result<Value> {
    get_json("/crypto/profile", &[("symbol", symbol)]).await
}

// ───── Indices ─────
pub async fn indices_constituents(symbol: &str) -> anyhow::Result<Value> {
    get_json("/index/constituents", &[("symbol", symbol)]).await
}
pub async fn indices_hist_constituents(symbol: &str) -> anyhow::Result<Value> {
    get_json("/index/historical-constituents", &[("symbol", symbol)]).await
}

// ───── ETF ─────
pub async fn etf_profile(symbol: &str) -> anyhow::Result<Value> {
    get_json("/etf/profile", &[("symbol", symbol)]).await
}
pub async fn etf_holdings(symbol: &str, skip: i64) -> anyhow::Result<Value> {
    get_json(
        "/etf/holdings",
        &[("symbol", symbol), ("skip", &skip.to_string())],
    )
    .await
}
pub async fn etf_sector(symbol: &str) -> anyhow::Result<Value> {
    get_json("/etf/sector", &[("symbol", symbol)]).await
}
pub async fn etf_country(symbol: &str) -> anyhow::Result<Value> {
    get_json("/etf/country", &[("symbol", symbol)]).await
}
pub async fn etf_allocation(symbol: &str) -> anyhow::Result<Value> {
    get_json("/etf/allocation", &[("symbol", symbol)]).await
}

// ───── Mutual Funds ─────
pub async fn mutual_fund_profile(symbol: &str) -> anyhow::Result<Value> {
    get_json("/mutual-fund/profile", &[("symbol", symbol)]).await
}
pub async fn mutual_fund_holdings(symbol: &str, skip: i64) -> anyhow::Result<Value> {
    get_json(
        "/mutual-fund/holdings",
        &[("symbol", symbol), ("skip", &skip.to_string())],
    )
    .await
}
pub async fn mutual_fund_sector(symbol: &str) -> anyhow::Result<Value> {
    get_json("/mutual-fund/sector", &[("symbol", symbol)]).await
}
pub async fn mutual_fund_country(symbol: &str) -> anyhow::Result<Value> {
    get_json("/mutual-fund/country", &[("symbol", symbol)]).await
}
pub async fn mutual_fund_eet(isin: &str) -> anyhow::Result<Value> {
    get_json("/mutual-fund/eet", &[("isin", isin)]).await
}

// ───── Bonds ─────
pub async fn bond_profile(isin: &str) -> anyhow::Result<Value> {
    get_json("/bond/profile", &[("isin", isin)]).await
}
pub async fn bond_price(isin: &str, from: &str, to: &str) -> anyhow::Result<Value> {
    get_json("/bond/price", &[("isin", isin), ("from", from), ("to", to)]).await
}
pub async fn bond_yield_curve(code: &str) -> anyhow::Result<Value> {
    get_json("/bond/yield-curve", &[("code", code)]).await
}

// ───── Economic / Market ─────
pub async fn economic_codes() -> anyhow::Result<Value> {
    get_json("/economic/code", &[]).await
}
pub async fn economic_data(code: &str) -> anyhow::Result<Value> {
    get_json("/economic", &[("code", code)]).await
}
pub async fn calendar_economic(from: &str, to: &str) -> anyhow::Result<Value> {
    get_json("/calendar/economic", &[("from", from), ("to", to)]).await
}
pub async fn country_list() -> anyhow::Result<Value> {
    get_json("/country", &[]).await
}
pub async fn market_status(exchange: &str) -> anyhow::Result<Value> {
    get_json("/stock/market-status", &[("exchange", exchange)]).await
}
pub async fn market_holiday(exchange: &str) -> anyhow::Result<Value> {
    get_json("/stock/market-holiday", &[("exchange", exchange)]).await
}
pub async fn stock_exchanges() -> anyhow::Result<Value> {
    get_json("/stock/exchange", &[]).await
}
pub async fn sector_metrics(region: &str) -> anyhow::Result<Value> {
    get_json("/sector/metrics", &[("region", region)]).await
}

// ───── Symbol meta / search ─────
pub async fn symbol_lookup(query: &str) -> anyhow::Result<Value> {
    get_json("/search", &[("q", query)]).await
}
pub async fn stock_symbol_list(exchange: &str) -> anyhow::Result<Value> {
    get_json("/stock/symbol", &[("exchange", exchange)]).await
}
pub async fn symbol_change(from: &str, to: &str) -> anyhow::Result<Value> {
    get_json("/ca/symbol-change", &[("from", from), ("to", to)]).await
}
pub async fn isin_change(from: &str, to: &str) -> anyhow::Result<Value> {
    get_json("/ca/isin-change", &[("from", from), ("to", to)]).await
}

// ───── Institutional ─────
pub async fn institutional_profile(cik: &str) -> anyhow::Result<Value> {
    get_json("/institutional/profile", &[("cik", cik)]).await
}
pub async fn institutional_portfolio(cik: &str, from: &str, to: &str) -> anyhow::Result<Value> {
    get_json(
        "/institutional/portfolio",
        &[("cik", cik), ("from", from), ("to", to)],
    )
    .await
}
pub async fn institutional_ownership(symbol: &str, from: &str, to: &str) -> anyhow::Result<Value> {
    get_json(
        "/institutional/ownership",
        &[("symbol", symbol), ("from", from), ("to", to)],
    )
    .await
}

// ───── Miscellaneous / specialty ─────
pub async fn fda_calendar() -> anyhow::Result<Value> {
    get_json("/fda-advisory-committee-calendar", &[]).await
}
pub async fn covid19_us() -> anyhow::Result<Value> {
    get_json("/covid19/us", &[]).await
}
pub async fn bank_branch(symbol: &str) -> anyhow::Result<Value> {
    get_json("/bank-branch", &[("symbol", symbol)]).await
}
pub async fn airline_price_index(airline: &str, from: &str, to: &str) -> anyhow::Result<Value> {
    get_json(
        "/airline/price-index",
        &[("airline", airline), ("from", from), ("to", to)],
    )
    .await
}
