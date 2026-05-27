//! Sentiment-as-a-feed — Reddit r/wallstreetbets + StockTwits pollers.
//!
//! Free public JSON endpoints:
//!   * <https://www.reddit.com/r/wallstreetbets/new.json?limit=100>  (User-Agent required)
//!   * <https://api.stocktwits.com/api/2/streams/symbol/SYM.json>    (rate-limited but no auth)
//!
//! X / Twitter is auth-gated. We expose the schema and source enum value
//! but skip the poller until the user supplies bearer credentials.

use chrono::{DateTime, Duration, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::PgPool;
use std::collections::HashSet;
use std::str::FromStr;
use traderview_core::sentiment::{score_post, Scored};

const UA: &str =
    "traderview/0.1 (github.com/MenkeTechnologies/traderview; ops@menketechnologies.com)";

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent(UA)
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap()
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Mention {
    pub id: uuid::Uuid,
    pub source: String,
    pub external_id: String,
    pub symbol: String,
    pub sentiment: Decimal,
    pub snippet: String,
    pub author: Option<String>,
    pub url: Option<String>,
    pub posted_at: DateTime<Utc>,
    pub fetched_at: DateTime<Utc>,
}

pub async fn insert(pool: &PgPool, source: &str, external_id: &str,
    sc: &Scored, author: Option<&str>, url: Option<&str>, posted_at: DateTime<Utc>)
    -> anyhow::Result<usize>
{
    let snippet: String = sc.text.chars().take(280).collect();
    let sent = Decimal::from_str(&format!("{:.4}", sc.score)).unwrap_or(Decimal::ZERO);
    let mut n = 0;
    for ticker in &sc.tickers {
        let r = sqlx::query(
            "INSERT INTO mentions
                (source, external_id, symbol, sentiment, snippet, author, url, posted_at)
             VALUES ($1::sentiment_source_t, $2, $3, $4, $5, $6, $7, $8)
             ON CONFLICT (source, external_id, symbol) DO NOTHING",
        )
        .bind(source).bind(external_id).bind(ticker).bind(sent).bind(&snippet)
        .bind(author).bind(url).bind(posted_at)
        .execute(pool).await?;
        if r.rows_affected() > 0 { n += 1; }
    }
    Ok(n)
}

// ===========================================================================
// Reddit r/wallstreetbets poller
// ===========================================================================

pub async fn poll_wsb(pool: &PgPool, whitelist: &HashSet<String>) -> anyhow::Result<usize> {
    let url = "https://www.reddit.com/r/wallstreetbets/new.json?limit=100";
    let resp = client().get(url).send().await?;
    if !resp.status().is_success() {
        anyhow::bail!("wsb HTTP {}", resp.status());
    }
    let v: serde_json::Value = resp.json().await?;
    let posts = v["data"]["children"].as_array().cloned().unwrap_or_default();
    let mut inserted = 0;
    for p in posts {
        let d = &p["data"];
        let id = d["id"].as_str().unwrap_or("").to_string();
        if id.is_empty() { continue; }
        let title = d["title"].as_str().unwrap_or("");
        let body  = d["selftext"].as_str().unwrap_or("");
        let combined = format!("{title} {body}");
        let author = d["author"].as_str().map(|s| s.to_string());
        let permalink = d["permalink"].as_str().unwrap_or("");
        let url = format!("https://www.reddit.com{}", permalink);
        let created = d["created_utc"].as_f64()
            .and_then(|ts| chrono::DateTime::from_timestamp(ts as i64, 0))
            .unwrap_or_else(Utc::now);
        let scored = score_post(&combined, whitelist);
        if scored.tickers.is_empty() { continue; }
        inserted += insert(pool, "wsb", &id, &scored, author.as_deref(), Some(&url), created)
            .await.unwrap_or(0);
    }
    Ok(inserted)
}

// ===========================================================================
// StockTwits symbol stream poller
// ===========================================================================

pub async fn poll_stocktwits(pool: &PgPool, symbols: &[String], whitelist: &HashSet<String>)
    -> anyhow::Result<usize>
{
    let mut inserted = 0;
    for sym in symbols {
        let url = format!(
            "https://api.stocktwits.com/api/2/streams/symbol/{sym}.json",
            sym = urlencoding(sym),
        );
        let resp = match client().get(&url).send().await {
            Ok(r) if r.status().is_success() => r,
            Ok(r) => { tracing::debug!(sym = %sym, status = ?r.status(), "stocktwits skip"); continue; }
            Err(e) => { tracing::debug!(sym = %sym, error = ?e, "stocktwits err"); continue; }
        };
        let v: serde_json::Value = match resp.json().await {
            Ok(v) => v,
            Err(_) => continue,
        };
        let msgs = v["messages"].as_array().cloned().unwrap_or_default();
        for m in msgs {
            let id = m["id"].as_i64().map(|x| x.to_string()).unwrap_or_default();
            if id.is_empty() { continue; }
            let body = m["body"].as_str().unwrap_or("");
            let author = m["user"]["username"].as_str().map(|s| s.to_string());
            let created = m["created_at"].as_str()
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(Utc::now);
            // StockTwits exposes user-supplied bullish/bearish flag.
            let mut scored = score_post(body, whitelist);
            // If the symbol we polled isn't in the extracted set (the post
            // may not say $SYM explicitly), inject it — we know the stream.
            if !scored.tickers.iter().any(|t| t == sym) {
                scored.tickers.push(sym.clone());
            }
            if let Some(bias) = m["entities"]["sentiment"]["basic"].as_str() {
                match bias {
                    "Bullish" => scored.score = (scored.score + 0.5).min(1.0),
                    "Bearish" => scored.score = (scored.score - 0.5).max(-1.0),
                    _ => {}
                }
            }
            let url = format!("https://stocktwits.com/message/{}", id);
            inserted += insert(pool, "stocktwits", &id, &scored,
                author.as_deref(), Some(&url), created)
                .await.unwrap_or(0);
        }
    }
    Ok(inserted)
}

fn urlencoding(s: &str) -> String { s.replace('^', "%5E").replace('=', "%3D") }

// ===========================================================================
// Rankings + queries
// ===========================================================================

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct RankedSymbol {
    pub symbol: String,
    pub mention_count: i64,
    pub avg_sentiment: Decimal,
    pub prev_count: i64,
    pub prev_sentiment: Decimal,
    pub count_delta: i64,
    pub sentiment_delta: Decimal,
}

/// Rank tickers by sentiment delta over the trailing `hours` window compared
/// against the prior equal window.
pub async fn ranked(pool: &PgPool, hours: i64, limit: i64) -> anyhow::Result<Vec<RankedSymbol>> {
    let now = Utc::now();
    let cur_start = now - Duration::hours(hours);
    let prev_start = cur_start - Duration::hours(hours);
    let rows: Vec<RankedSymbol> = sqlx::query_as(
        "WITH cur AS (
            SELECT symbol, COUNT(*) AS mention_count, AVG(sentiment) AS avg_sentiment
              FROM mentions WHERE posted_at >= $1 AND posted_at < $2
             GROUP BY symbol
         ),
         prev AS (
            SELECT symbol, COUNT(*) AS prev_count, AVG(sentiment) AS prev_sentiment
              FROM mentions WHERE posted_at >= $3 AND posted_at < $1
             GROUP BY symbol
         )
         SELECT cur.symbol,
                cur.mention_count::int8,
                cur.avg_sentiment::numeric(6,4),
                COALESCE(prev.prev_count, 0)::int8 AS prev_count,
                COALESCE(prev.prev_sentiment, 0)::numeric(6,4) AS prev_sentiment,
                (cur.mention_count - COALESCE(prev.prev_count, 0))::int8 AS count_delta,
                (cur.avg_sentiment - COALESCE(prev.prev_sentiment, 0))::numeric(6,4) AS sentiment_delta
           FROM cur LEFT JOIN prev USING (symbol)
          ORDER BY ABS(cur.avg_sentiment - COALESCE(prev.prev_sentiment, 0)) DESC,
                   cur.mention_count DESC
          LIMIT $4",
    )
    .bind(cur_start).bind(now).bind(prev_start).bind(limit)
    .fetch_all(pool).await?;
    Ok(rows)
}

pub async fn feed(pool: &PgPool, limit: i64) -> anyhow::Result<Vec<Mention>> {
    Ok(sqlx::query_as::<_, Mention>(
        "SELECT id, source::text, external_id, symbol, sentiment, snippet,
                author, url, posted_at, fetched_at
           FROM mentions ORDER BY posted_at DESC LIMIT $1",
    )
    .bind(limit)
    .fetch_all(pool).await?)
}

pub async fn for_symbol(pool: &PgPool, symbol: &str, hours: i64, limit: i64)
    -> anyhow::Result<Vec<Mention>>
{
    let from = Utc::now() - Duration::hours(hours);
    Ok(sqlx::query_as::<_, Mention>(
        "SELECT id, source::text, external_id, symbol, sentiment, snippet,
                author, url, posted_at, fetched_at
           FROM mentions WHERE symbol = $1 AND posted_at >= $2
          ORDER BY posted_at DESC LIMIT $3",
    )
    .bind(symbol).bind(from).bind(limit)
    .fetch_all(pool).await?)
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct HourlyBucket {
    pub bucket_hour: DateTime<Utc>,
    pub source: String,
    pub mention_count: i32,
    pub avg_sentiment: Decimal,
}

pub async fn timeseries(pool: &PgPool, symbol: &str, hours: i64) -> anyhow::Result<Vec<HourlyBucket>> {
    let from = Utc::now() - Duration::hours(hours);
    Ok(sqlx::query_as::<_, HourlyBucket>(
        "SELECT date_trunc('hour', posted_at) AS bucket_hour,
                source::text,
                COUNT(*)::int4 AS mention_count,
                AVG(sentiment)::numeric(6,4) AS avg_sentiment
           FROM mentions
          WHERE symbol = $1 AND posted_at >= $2
          GROUP BY bucket_hour, source
          ORDER BY bucket_hour",
    )
    .bind(symbol).bind(from)
    .fetch_all(pool).await?)
}

// ===========================================================================
// Top-of-loop poller: WSB + per-watchlist StockTwits.
// ===========================================================================

pub async fn poll_all(pool: &PgPool) -> (usize, usize) {
    // Build whitelist of known tickers — symbols any user has on a watchlist
    // (cheap; falls back to empty so the WSB cashtag detector still works).
    let symbols: Vec<String> = sqlx::query_scalar(
        "SELECT DISTINCT symbol FROM watchlist_symbols",
    )
    .fetch_all(pool).await.unwrap_or_default();
    let whitelist: HashSet<String> = symbols.iter().cloned().collect();

    let wsb = poll_wsb(pool, &whitelist).await.unwrap_or_else(|e| {
        tracing::warn!(error = ?e, "wsb poll failed"); 0
    });
    let st = poll_stocktwits(pool, &symbols, &whitelist).await.unwrap_or_else(|e| {
        tracing::warn!(error = ?e, "stocktwits poll failed"); 0
    });
    (wsb, st)
}
