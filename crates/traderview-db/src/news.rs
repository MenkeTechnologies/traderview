//! News ingestion + search.
//!
//! Live path: `fetch_for_symbol()` hits Yahoo's v1/finance/search endpoint
//! (already in `market_data::news`), scores each headline via
//! `traderview_core::sentiment::score`, upserts into `news_items`.
//!
//! Read path: `recent_for_symbol`, `recent_global`, `search` (tsvector + ts_rank).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::market_data;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct NewsRow {
    pub id: uuid::Uuid,
    pub symbol: String,
    pub uuid: Option<String>,
    pub title: String,
    pub publisher: Option<String>,
    pub link: Option<String>,
    pub thumbnail: Option<String>,
    pub sentiment: Option<f32>,
    pub published_at: Option<DateTime<Utc>>,
    pub fetched_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollStats {
    pub fetched: usize,
    pub inserted: u64,
    pub symbols_polled: usize,
}

pub async fn fetch_for_symbol(pool: &PgPool, symbol: &str, count: usize) -> anyhow::Result<u64> {
    let items = market_data::news(symbol, count).await.unwrap_or_default();
    let mut inserted = 0u64;
    for n in items {
        let Some(title) = n.title else { continue };
        let sentiment = Some(traderview_core::sentiment::score(&title) as f32);
        let published = n
            .provider_publish_time
            .and_then(|ts| chrono::DateTime::from_timestamp(ts, 0));
        // ON CONFLICT DO NOTHING handles both unique indexes.
        let r = sqlx::query(
            "INSERT INTO news_items
                (symbol, uuid, title, publisher, link, thumbnail, sentiment, published_at)
              VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
             ON CONFLICT DO NOTHING",
        )
        .bind(symbol)
        .bind(n.uuid.as_deref())
        .bind(&title)
        .bind(n.publisher.as_deref())
        .bind(n.link.as_deref())
        .bind(n.thumbnail.as_deref())
        .bind(sentiment)
        .bind(published)
        .execute(pool)
        .await?;
        inserted += r.rows_affected();
    }
    Ok(inserted)
}

/// Poll all distinct watchlist symbols across all users.
pub async fn poll_watchlists(pool: &PgPool) -> anyhow::Result<PollStats> {
    let symbols: Vec<String> = sqlx::query_scalar(
        "SELECT DISTINCT symbol FROM watchlist_symbols ORDER BY symbol LIMIT 100",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();
    let mut inserted = 0u64;
    let mut fetched = 0usize;
    for s in &symbols {
        if let Ok(n) = fetch_for_symbol(pool, s, 10).await {
            inserted += n;
            fetched += 10;
        }
        // tiny politeness delay between Yahoo calls
        tokio::time::sleep(std::time::Duration::from_millis(120)).await;
    }
    Ok(PollStats {
        fetched,
        inserted,
        symbols_polled: symbols.len(),
    })
}

pub async fn recent_for_symbol(
    pool: &PgPool,
    symbol: &str,
    limit: i64,
) -> anyhow::Result<Vec<NewsRow>> {
    Ok(sqlx::query_as(
        "SELECT id, symbol, uuid, title, publisher, link, thumbnail, sentiment,
                published_at, fetched_at
           FROM news_items
          WHERE symbol = $1
          ORDER BY COALESCE(published_at, fetched_at) DESC
          LIMIT $2",
    )
    .bind(symbol)
    .bind(limit)
    .fetch_all(pool)
    .await?)
}

pub async fn recent_global(pool: &PgPool, limit: i64) -> anyhow::Result<Vec<NewsRow>> {
    Ok(sqlx::query_as(
        "SELECT id, symbol, uuid, title, publisher, link, thumbnail, sentiment,
                published_at, fetched_at
           FROM news_items
          ORDER BY COALESCE(published_at, fetched_at) DESC
          LIMIT $1",
    )
    .bind(limit)
    .fetch_all(pool)
    .await?)
}

pub async fn search(pool: &PgPool, q: &str, limit: i64) -> anyhow::Result<Vec<NewsRow>> {
    // websearch_to_tsquery handles user input like AAPL OR -lawsuit safely.
    Ok(sqlx::query_as(
        "SELECT id, symbol, uuid, title, publisher, link, thumbnail, sentiment,
                published_at, fetched_at
           FROM news_items
          WHERE search_tsv @@ websearch_to_tsquery('english', $1)
          ORDER BY ts_rank(search_tsv, websearch_to_tsquery('english', $1)) DESC,
                   COALESCE(published_at, fetched_at) DESC
          LIMIT $2",
    )
    .bind(q)
    .bind(limit)
    .fetch_all(pool)
    .await?)
}
