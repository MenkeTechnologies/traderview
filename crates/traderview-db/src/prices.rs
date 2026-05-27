//! Price-bars cache + yfinance fetcher.
//!
//! On `get_bars`, return the cached range; if it doesn't cover the requested
//! window, fetch from Yahoo Finance, upsert into `price_bars`, and re-query.
//!
//! Yahoo v8 chart endpoint:
//!   `https://query1.finance.yahoo.com/v8/finance/chart/{symbol}`
//!     `?period1={epoch}&period2={epoch}&interval={1m,5m,15m,1h,1d,1wk}`
//!
//! Public endpoint — no API key. Subject to rate-limits; the cache amortizes.

use chrono::{DateTime, TimeZone, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::str::FromStr;
use traderview_core::{BarInterval, PriceBar};

const YAHOO_BASE: &str = "https://query1.finance.yahoo.com/v8/finance/chart/";
const USER_AGENT: &str =
    "Mozilla/5.0 (compatible; traderview/0.1; +https://github.com/MenkeTechnologies/traderview)";

pub async fn get_bars(
    pool: &PgPool,
    symbol: &str,
    interval: BarInterval,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> anyhow::Result<Vec<PriceBar>> {
    if !is_range_cached(pool, symbol, interval, from, to).await? {
        match fetch_yahoo(symbol, interval, from, to).await {
            Ok(bars) => {
                upsert(pool, &bars).await?;
                log_fetch(pool, symbol, interval, from, to, bars.len() as i32).await?;
            }
            Err(e) => {
                tracing::warn!(
                    ?e,
                    symbol,
                    ?interval,
                    "yahoo fetch failed; serving cached only"
                );
            }
        }
    }
    read_bars(pool, symbol, interval, from, to).await
}

async fn read_bars(
    pool: &PgPool,
    symbol: &str,
    interval: BarInterval,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> anyhow::Result<Vec<PriceBar>> {
    type PriceBarRow = (
        String,
        String,
        DateTime<Utc>,
        Decimal,
        Decimal,
        Decimal,
        Decimal,
        Decimal,
        String,
    );
    let rows: Vec<PriceBarRow> = sqlx::query_as(
        "SELECT symbol, interval::text, bar_time, open, high, low, close, volume, source
               FROM price_bars
              WHERE symbol = $1 AND interval = $2::bar_interval_t
                AND bar_time BETWEEN $3 AND $4
              ORDER BY bar_time ASC",
    )
    .bind(symbol)
    .bind(interval.label())
    .bind(from)
    .bind(to)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(
            |(symbol, iv, bar_time, open, high, low, close, volume, source)| PriceBar {
                symbol,
                interval: parse_interval(&iv),
                bar_time,
                open,
                high,
                low,
                close,
                volume,
                source,
            },
        )
        .collect())
}

async fn is_range_cached(
    pool: &PgPool,
    symbol: &str,
    interval: BarInterval,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> anyhow::Result<bool> {
    let row: Option<(bool,)> = sqlx::query_as(
        "SELECT TRUE FROM price_fetch_log
          WHERE symbol = $1 AND interval = $2::bar_interval_t
            AND range_start <= $3 AND range_end >= $4 LIMIT 1",
    )
    .bind(symbol)
    .bind(interval.label())
    .bind(from)
    .bind(to)
    .fetch_optional(pool)
    .await?;
    Ok(row.is_some())
}

async fn log_fetch(
    pool: &PgPool,
    symbol: &str,
    interval: BarInterval,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
    bar_count: i32,
) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO price_fetch_log (symbol, interval, range_start, range_end, bar_count)
              VALUES ($1, $2::bar_interval_t, $3, $4, $5)
         ON CONFLICT DO NOTHING",
    )
    .bind(symbol)
    .bind(interval.label())
    .bind(from)
    .bind(to)
    .bind(bar_count)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn upsert(pool: &PgPool, bars: &[PriceBar]) -> anyhow::Result<()> {
    if bars.is_empty() {
        return Ok(());
    }
    let mut tx = pool.begin().await?;
    for b in bars {
        sqlx::query(
            "INSERT INTO price_bars
                (symbol, interval, bar_time, open, high, low, close, volume, source)
             VALUES ($1, $2::bar_interval_t, $3, $4, $5, $6, $7, $8, $9)
             ON CONFLICT (symbol, interval, bar_time) DO UPDATE
                SET open = EXCLUDED.open, high = EXCLUDED.high,
                    low = EXCLUDED.low, close = EXCLUDED.close,
                    volume = EXCLUDED.volume, source = EXCLUDED.source,
                    fetched_at = now()",
        )
        .bind(&b.symbol)
        .bind(b.interval.label())
        .bind(b.bar_time)
        .bind(b.open)
        .bind(b.high)
        .bind(b.low)
        .bind(b.close)
        .bind(b.volume)
        .bind(&b.source)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}

fn parse_interval(s: &str) -> BarInterval {
    match s {
        "1m" => BarInterval::M1,
        "5m" => BarInterval::M5,
        "15m" => BarInterval::M15,
        "1h" => BarInterval::H1,
        "1w" => BarInterval::W1,
        _ => BarInterval::D1,
    }
}

fn yahoo_interval(iv: BarInterval) -> &'static str {
    match iv {
        BarInterval::M1 => "1m",
        BarInterval::M5 => "5m",
        BarInterval::M15 => "15m",
        BarInterval::H1 => "60m",
        BarInterval::D1 => "1d",
        BarInterval::W1 => "1wk",
    }
}

async fn fetch_yahoo(
    symbol: &str,
    interval: BarInterval,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> anyhow::Result<Vec<PriceBar>> {
    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(std::time::Duration::from_secs(10))
        .build()?;
    let url = format!(
        "{base}{sym}?period1={p1}&period2={p2}&interval={iv}&events=div%2Csplit&includeAdjustedClose=true",
        base = YAHOO_BASE,
        sym = symbol,
        p1 = from.timestamp(),
        p2 = to.timestamp(),
        iv = yahoo_interval(interval),
    );
    let resp = client.get(&url).send().await?;
    if !resp.status().is_success() {
        anyhow::bail!("yahoo HTTP {}", resp.status());
    }
    let raw: YahooResponse = resp.json().await?;
    let result = raw
        .chart
        .result
        .and_then(|mut v| v.pop())
        .ok_or_else(|| anyhow::anyhow!("yahoo: empty result"))?;
    let timestamps = result.timestamp.unwrap_or_default();
    let quote = result
        .indicators
        .quote
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("yahoo: no quote"))?;

    let mut out = Vec::with_capacity(timestamps.len());
    for (i, ts) in timestamps.iter().enumerate() {
        let bar_time = Utc
            .timestamp_opt(*ts, 0)
            .single()
            .ok_or_else(|| anyhow::anyhow!("bad ts"))?;
        let open = quote
            .open
            .get(i)
            .and_then(|x| *x)
            .and_then(|x| Decimal::from_str(&x.to_string()).ok());
        let high = quote
            .high
            .get(i)
            .and_then(|x| *x)
            .and_then(|x| Decimal::from_str(&x.to_string()).ok());
        let low = quote
            .low
            .get(i)
            .and_then(|x| *x)
            .and_then(|x| Decimal::from_str(&x.to_string()).ok());
        let close = quote
            .close
            .get(i)
            .and_then(|x| *x)
            .and_then(|x| Decimal::from_str(&x.to_string()).ok());
        let volume = quote
            .volume
            .get(i)
            .and_then(|x| *x)
            .map(Decimal::from)
            .unwrap_or(Decimal::ZERO);
        if let (Some(open), Some(high), Some(low), Some(close)) = (open, high, low, close) {
            out.push(PriceBar {
                symbol: symbol.into(),
                interval,
                bar_time,
                open,
                high,
                low,
                close,
                volume,
                source: "yfinance".into(),
            });
        }
    }
    Ok(out)
}

#[derive(serde::Deserialize)]
struct YahooResponse {
    chart: YahooChart,
}
#[derive(serde::Deserialize)]
struct YahooChart {
    result: Option<Vec<YahooResult>>,
}
#[derive(serde::Deserialize)]
struct YahooResult {
    timestamp: Option<Vec<i64>>,
    indicators: YahooIndicators,
}
#[derive(serde::Deserialize)]
struct YahooIndicators {
    quote: Vec<YahooQuote>,
}
#[derive(serde::Deserialize)]
struct YahooQuote {
    open: Vec<Option<f64>>,
    high: Vec<Option<f64>>,
    low: Vec<Option<f64>>,
    close: Vec<Option<f64>>,
    #[serde(default)]
    volume: Vec<Option<i64>>,
}
