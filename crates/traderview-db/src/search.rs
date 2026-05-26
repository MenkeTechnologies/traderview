//! Full-text + symbol-trigram search across trades, journal, and forum posts.

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct SearchHits {
    pub query: String,
    pub trades: Vec<TradeHit>,
    pub journal: Vec<JournalHit>,
    pub forum: Vec<ForumHit>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct TradeHit {
    pub id: Uuid,
    pub account_id: Uuid,
    pub symbol: String,
    pub side: String,
    pub status: String,
    pub opened_at: DateTime<Utc>,
    pub net_pnl: Option<Decimal>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct JournalHit {
    pub id: Uuid,
    pub trade_id: Option<Uuid>,
    pub day: Option<NaiveDate>,
    pub snippet: String,
    pub created_at: DateTime<Utc>,
    pub rank: f32,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ForumHit {
    pub post_id: Uuid,
    pub thread_id: Uuid,
    pub thread_title: String,
    pub thread_slug: String,
    pub category_slug: String,
    pub snippet: String,
    pub created_at: DateTime<Utc>,
    pub rank: f32,
}

pub async fn search(
    pool: &PgPool,
    user_id: Uuid,
    q: &str,
    scope: &str,
    limit: i64,
) -> anyhow::Result<SearchHits> {
    let trades = if scope == "all" || scope == "trades" {
        sqlx::query_as::<_, TradeHit>(
            "SELECT t.id, t.account_id, t.symbol, t.side::text, t.status::text,
                    t.opened_at, t.net_pnl
               FROM trades t
               JOIN accounts a ON a.id = t.account_id
              WHERE a.user_id = $1
                AND (t.symbol ILIKE '%' || $2 || '%' OR t.symbol % $2)
              ORDER BY t.opened_at DESC
              LIMIT $3",
        )
        .bind(user_id)
        .bind(q)
        .bind(limit)
        .fetch_all(pool)
        .await?
    } else {
        Vec::new()
    };

    let journal = if scope == "all" || scope == "journal" {
        sqlx::query_as::<_, JournalHit>(
            "SELECT id, trade_id, day,
                    ts_headline('english', body_md, plainto_tsquery('english', $2),
                                'MaxFragments=1, MaxWords=20, MinWords=5') AS snippet,
                    created_at,
                    ts_rank(body_tsv, plainto_tsquery('english', $2)) AS rank
               FROM journal_entries
              WHERE user_id = $1
                AND body_tsv @@ plainto_tsquery('english', $2)
              ORDER BY rank DESC, created_at DESC
              LIMIT $3",
        )
        .bind(user_id)
        .bind(q)
        .bind(limit)
        .fetch_all(pool)
        .await?
    } else {
        Vec::new()
    };

    let forum = if scope == "all" || scope == "forum" {
        sqlx::query_as::<_, ForumHit>(
            "SELECT p.id AS post_id, t.id AS thread_id, t.title AS thread_title,
                    t.slug AS thread_slug, c.slug AS category_slug,
                    ts_headline('english', p.body_md, plainto_tsquery('english', $1),
                                'MaxFragments=1, MaxWords=20, MinWords=5') AS snippet,
                    p.created_at,
                    ts_rank(p.body_tsv, plainto_tsquery('english', $1)) AS rank
               FROM forum_posts p
               JOIN forum_threads t ON t.id = p.thread_id
               JOIN forum_categories c ON c.id = t.category_id
              WHERE p.body_tsv @@ plainto_tsquery('english', $1)
              ORDER BY rank DESC, p.created_at DESC
              LIMIT $2",
        )
        .bind(q)
        .bind(limit)
        .fetch_all(pool)
        .await?
    } else {
        Vec::new()
    };

    Ok(SearchHits {
        query: q.into(),
        trades,
        journal,
        forum,
    })
}
