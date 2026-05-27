//! Forced post-trade reflection — trades with |R| >= 2 land in a "needs
//! review" inbox until the user fills the 5-question form.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

const R_THRESHOLD: f64 = 2.0;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct TradeReview {
    pub id: Uuid,
    pub user_id: Uuid,
    pub trade_id: Uuid,
    pub entry_per_plan: Option<bool>,
    pub exit_per_plan: Option<bool>,
    pub would_change: Option<String>,
    pub mood_at_exit: Option<i16>,
    pub setup_tag: Option<String>,
    pub completed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReviewInput {
    pub trade_id: Uuid,
    pub entry_per_plan: Option<bool>,
    pub exit_per_plan: Option<bool>,
    pub would_change: Option<String>,
    pub mood_at_exit: Option<i16>,
    pub setup_tag: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NeedsReviewRow {
    pub trade_id: Uuid,
    pub symbol: String,
    pub net_pnl: f64,
    pub risk_amount: f64,
    pub r_multiple: f64,
    pub closed_at: Option<DateTime<Utc>>,
    pub side: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReviewStats {
    pub total_high_r_trades: i64,
    pub reviewed: i64,
    pub pending: i64,
    pub completion_pct: f64,
    pub last_review_at: Option<DateTime<Utc>>,
}

pub async fn needs_review(pool: &PgPool, user_id: Uuid, account_id: Uuid, limit: i64)
    -> anyhow::Result<Vec<NeedsReviewRow>>
{
    let rows: Vec<(Uuid, String, Decimal, Decimal, Option<DateTime<Utc>>, String)> = sqlx::query_as(
        "SELECT t.id, t.symbol, t.net_pnl, t.risk_amount, t.closed_at, t.side::text
           FROM trades t
           LEFT JOIN trade_reviews r ON r.trade_id = t.id AND r.user_id = $1
          WHERE t.account_id = $2
            AND t.status = 'closed'
            AND t.net_pnl IS NOT NULL
            AND t.risk_amount IS NOT NULL AND t.risk_amount > 0
            AND ABS(t.net_pnl / t.risk_amount) >= $3
            AND r.id IS NULL
          ORDER BY ABS(t.net_pnl / t.risk_amount) DESC
          LIMIT $4",
    ).bind(user_id).bind(account_id).bind(R_THRESHOLD).bind(limit)
     .fetch_all(pool).await?;
    Ok(rows.into_iter().map(|(id, sym, pnl, risk, closed, side)| {
        let net = dec(pnl);
        let r = dec(risk);
        NeedsReviewRow {
            trade_id: id, symbol: sym, net_pnl: net, risk_amount: r,
            r_multiple: if r > 0.0 { net / r } else { 0.0 },
            closed_at: closed, side,
        }
    }).collect())
}

pub async fn stats(pool: &PgPool, user_id: Uuid, account_id: Uuid)
    -> anyhow::Result<ReviewStats>
{
    let (total, reviewed, last): (i64, i64, Option<DateTime<Utc>>) = sqlx::query_as(
        "WITH high_r AS (
            SELECT id FROM trades
             WHERE account_id = $2
               AND status = 'closed'
               AND net_pnl IS NOT NULL
               AND risk_amount IS NOT NULL AND risk_amount > 0
               AND ABS(net_pnl / risk_amount) >= $3
         )
         SELECT
            (SELECT COUNT(*) FROM high_r),
            (SELECT COUNT(*) FROM trade_reviews r
              WHERE r.user_id = $1
                AND r.trade_id IN (SELECT id FROM high_r)),
            (SELECT MAX(completed_at) FROM trade_reviews r
              WHERE r.user_id = $1
                AND r.trade_id IN (SELECT id FROM high_r))",
    ).bind(user_id).bind(account_id).bind(R_THRESHOLD)
     .fetch_one(pool).await?;
    let pct = if total > 0 { reviewed as f64 / total as f64 * 100.0 } else { 0.0 };
    Ok(ReviewStats {
        total_high_r_trades: total, reviewed,
        pending: total - reviewed, completion_pct: pct, last_review_at: last,
    })
}

pub async fn list(pool: &PgPool, user_id: Uuid, limit: i64) -> anyhow::Result<Vec<TradeReview>> {
    Ok(sqlx::query_as(
        "SELECT id, user_id, trade_id, entry_per_plan, exit_per_plan,
                would_change, mood_at_exit, setup_tag, completed_at
           FROM trade_reviews WHERE user_id = $1
          ORDER BY completed_at DESC LIMIT $2",
    ).bind(user_id).bind(limit).fetch_all(pool).await?)
}

pub async fn for_trade(pool: &PgPool, user_id: Uuid, trade_id: Uuid)
    -> anyhow::Result<Option<TradeReview>>
{
    Ok(sqlx::query_as(
        "SELECT id, user_id, trade_id, entry_per_plan, exit_per_plan,
                would_change, mood_at_exit, setup_tag, completed_at
           FROM trade_reviews WHERE user_id = $1 AND trade_id = $2",
    ).bind(user_id).bind(trade_id).fetch_optional(pool).await?)
}

pub async fn upsert(pool: &PgPool, user_id: Uuid, dto: &ReviewInput)
    -> anyhow::Result<TradeReview>
{
    Ok(sqlx::query_as(
        "INSERT INTO trade_reviews
            (user_id, trade_id, entry_per_plan, exit_per_plan,
             would_change, mood_at_exit, setup_tag)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         ON CONFLICT (user_id, trade_id) DO UPDATE SET
            entry_per_plan = EXCLUDED.entry_per_plan,
            exit_per_plan  = EXCLUDED.exit_per_plan,
            would_change   = EXCLUDED.would_change,
            mood_at_exit   = EXCLUDED.mood_at_exit,
            setup_tag      = EXCLUDED.setup_tag,
            completed_at   = now()
         RETURNING id, user_id, trade_id, entry_per_plan, exit_per_plan,
                   would_change, mood_at_exit, setup_tag, completed_at",
    )
    .bind(user_id).bind(dto.trade_id)
    .bind(dto.entry_per_plan).bind(dto.exit_per_plan)
    .bind(dto.would_change.as_deref()).bind(dto.mood_at_exit)
    .bind(dto.setup_tag.as_deref())
    .fetch_one(pool).await?)
}

pub async fn delete(pool: &PgPool, user_id: Uuid, trade_id: Uuid) -> anyhow::Result<bool> {
    Ok(sqlx::query("DELETE FROM trade_reviews WHERE user_id = $1 AND trade_id = $2")
        .bind(user_id).bind(trade_id).execute(pool).await?.rows_affected() > 0)
}

fn dec(d: Decimal) -> f64 { d.to_string().parse().unwrap_or(0.0) }
