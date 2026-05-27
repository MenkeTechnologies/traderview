//! Paper trading simulator — virtual account with market orders that fill
//! immediately against the latest cached quote. Mirrors Warrior Trading's
//! $200k SimTrader (minus the live order book — we fill at last price).

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use traderview_core::Side;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct PaperAccount {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub starting_cash: Decimal,
    pub cash: Decimal,
    pub created_at: DateTime<Utc>,
    pub reset_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct PaperOrder {
    pub id: Uuid,
    pub paper_account_id: Uuid,
    pub symbol: String,
    pub side: String,
    pub qty: Decimal,
    pub order_type: String,
    pub limit_price: Option<Decimal>,
    pub stop_price: Option<Decimal>,
    pub status: String,
    pub filled_price: Option<Decimal>,
    pub filled_qty: Option<Decimal>,
    pub fee: Decimal,
    pub submitted_at: DateTime<Utc>,
    pub filled_at: Option<DateTime<Utc>>,
    pub cancel_at: Option<DateTime<Utc>>,
    pub reject_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct PaperPosition {
    pub paper_account_id: Uuid,
    pub symbol: String,
    pub qty: Decimal,
    pub avg_price: Decimal,
    pub realized_pnl: Decimal,
    pub updated_at: DateTime<Utc>,
}

pub async fn list_accounts(pool: &PgPool, user_id: Uuid) -> anyhow::Result<Vec<PaperAccount>> {
    Ok(sqlx::query_as::<_, PaperAccount>(
        "SELECT id, user_id, name, starting_cash, cash, created_at, reset_at
           FROM paper_accounts WHERE user_id = $1 ORDER BY created_at",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?)
}

pub async fn ensure_default(pool: &PgPool, user_id: Uuid) -> anyhow::Result<PaperAccount> {
    if let Some(a) = list_accounts(pool, user_id).await?.into_iter().next() {
        return Ok(a);
    }
    Ok(sqlx::query_as::<_, PaperAccount>(
        "INSERT INTO paper_accounts (user_id, name, starting_cash, cash)
              VALUES ($1, 'SimTrader', 200000, 200000)
         RETURNING id, user_id, name, starting_cash, cash, created_at, reset_at",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?)
}

pub async fn reset(pool: &PgPool, user_id: Uuid, account_id: Uuid, starting: Decimal) -> anyhow::Result<bool> {
    let mut tx = pool.begin().await?;
    let r = sqlx::query(
        "UPDATE paper_accounts SET starting_cash = $3, cash = $3, reset_at = now()
          WHERE id = $1 AND user_id = $2",
    )
    .bind(account_id).bind(user_id).bind(starting)
    .execute(&mut *tx).await?;
    if r.rows_affected() == 0 {
        return Ok(false);
    }
    sqlx::query("DELETE FROM paper_orders WHERE paper_account_id = $1")
        .bind(account_id).execute(&mut *tx).await?;
    sqlx::query("DELETE FROM paper_positions WHERE paper_account_id = $1")
        .bind(account_id).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(true)
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrderRequest {
    pub symbol: String,
    pub side: Side,
    pub qty: Decimal,
    pub order_type: String,         // 'market' | 'limit' | 'stop' | 'stop_limit'
    pub limit_price: Option<Decimal>,
    pub stop_price: Option<Decimal>,
}

/// Submit + immediately fill a paper order against the latest cached quote.
/// Stop / limit orders fill immediately if the current price satisfies the
/// trigger; otherwise rejected (no order book in this lightweight sim).
pub async fn submit(
    pool: &PgPool,
    user_id: Uuid,
    account_id: Uuid,
    req: OrderRequest,
) -> anyhow::Result<PaperOrder> {
    // Ownership check.
    let owner: Option<(Uuid,)> = sqlx::query_as("SELECT user_id FROM paper_accounts WHERE id = $1")
        .bind(account_id).fetch_optional(pool).await?;
    if !matches!(owner, Some((u,)) if u == user_id) {
        anyhow::bail!("forbidden");
    }

    let quote = crate::market_data::quote(pool, &req.symbol).await?;
    let last = Decimal::try_from(quote.price)?;

    let fill_price = match req.order_type.as_str() {
        "market" => Some(last),
        "limit" => match (req.side, req.limit_price) {
            (Side::Buy | Side::Cover, Some(lp)) if last <= lp => Some(last),
            (Side::Sell | Side::Short, Some(lp)) if last >= lp => Some(last),
            _ => None,
        },
        "stop" => match (req.side, req.stop_price) {
            (Side::Buy | Side::Cover, Some(sp)) if last >= sp => Some(last),
            (Side::Sell | Side::Short, Some(sp)) if last <= sp => Some(last),
            _ => None,
        },
        _ => None,
    };

    let side_str = match req.side {
        Side::Buy => "buy",
        Side::Sell => "sell",
        Side::Short => "short",
        Side::Cover => "cover",
    };

    let mut tx = pool.begin().await?;
    let (status, filled_at, reject) = match fill_price {
        Some(_) => ("filled", Some(Utc::now()), None),
        None => ("rejected", None, Some("limit/stop not triggered at current quote".to_string())),
    };
    let order: PaperOrder = sqlx::query_as(
        "INSERT INTO paper_orders
            (paper_account_id, symbol, side, qty, order_type, limit_price, stop_price,
             status, filled_price, filled_qty, filled_at, reject_reason)
         VALUES ($1, $2, $3::side_t, $4, $5::paper_order_type_t, $6, $7,
                 $8::paper_order_status_t, $9, $10, $11, $12)
         RETURNING id, paper_account_id, symbol, side::text, qty, order_type::text,
                   limit_price, stop_price, status::text,
                   filled_price, filled_qty, fee, submitted_at, filled_at, cancel_at, reject_reason",
    )
    .bind(account_id).bind(&req.symbol.to_uppercase()).bind(side_str)
    .bind(req.qty).bind(&req.order_type).bind(req.limit_price).bind(req.stop_price)
    .bind(status).bind(fill_price).bind(fill_price.map(|_| req.qty))
    .bind(filled_at).bind(reject)
    .fetch_one(&mut *tx)
    .await?;

    if let Some(price) = fill_price {
        apply_fill(&mut tx, account_id, &req.symbol.to_uppercase(), req.side, req.qty, price).await?;
    }
    tx.commit().await?;
    Ok(order)
}

async fn apply_fill(
    tx: &mut sqlx::PgConnection,
    account_id: Uuid,
    symbol: &str,
    side: Side,
    qty: Decimal,
    price: Decimal,
) -> anyhow::Result<()> {
    let signed_qty = match side {
        Side::Buy | Side::Cover => qty,
        Side::Sell | Side::Short => -qty,
    };
    let notional = price * qty;
    let row: Option<(Decimal, Decimal, Decimal)> = sqlx::query_as(
        "SELECT qty, avg_price, realized_pnl FROM paper_positions
          WHERE paper_account_id = $1 AND symbol = $2",
    )
    .bind(account_id).bind(symbol)
    .fetch_optional(&mut *tx).await?;

    let (new_qty, new_avg, new_realized) = match row {
        None => (signed_qty, price, Decimal::ZERO),
        Some((cur_qty, cur_avg, cur_realized)) => {
            let same_sign = (cur_qty > Decimal::ZERO && signed_qty > Decimal::ZERO)
                         || (cur_qty < Decimal::ZERO && signed_qty < Decimal::ZERO);
            let new_q = cur_qty + signed_qty;
            if same_sign || cur_qty.is_zero() {
                // Adding to position — weighted-average.
                let total = cur_avg * cur_qty.abs() + price * signed_qty.abs();
                let avg = if new_q.abs() > Decimal::ZERO { total / new_q.abs() } else { Decimal::ZERO };
                (new_q, avg, cur_realized)
            } else {
                // Reducing or flipping — realize P&L on the part that crosses.
                let close_qty = cur_qty.abs().min(signed_qty.abs());
                let direction = if cur_qty > Decimal::ZERO { Decimal::ONE } else { -Decimal::ONE };
                let realized = (price - cur_avg) * close_qty * direction;
                let avg = if new_q.abs() > Decimal::ZERO {
                    if (cur_qty > Decimal::ZERO) == (new_q > Decimal::ZERO) {
                        cur_avg
                    } else {
                        price
                    }
                } else {
                    Decimal::ZERO
                };
                (new_q, avg, cur_realized + realized)
            }
        }
    };

    if new_qty.is_zero() {
        sqlx::query(
            "DELETE FROM paper_positions WHERE paper_account_id = $1 AND symbol = $2",
        ).bind(account_id).bind(symbol).execute(&mut *tx).await?;
    } else {
        sqlx::query(
            "INSERT INTO paper_positions (paper_account_id, symbol, qty, avg_price, realized_pnl, updated_at)
                  VALUES ($1, $2, $3, $4, $5, now())
             ON CONFLICT (paper_account_id, symbol) DO UPDATE SET
                qty = EXCLUDED.qty, avg_price = EXCLUDED.avg_price,
                realized_pnl = EXCLUDED.realized_pnl, updated_at = now()",
        )
        .bind(account_id).bind(symbol).bind(new_qty).bind(new_avg).bind(new_realized)
        .execute(&mut *tx).await?;
    }

    // Cash impact (no fees in sim).
    let cash_delta = -signed_qty * price;  // buy decreases cash, sell increases
    sqlx::query(
        "UPDATE paper_accounts SET cash = cash + $2 WHERE id = $1",
    )
    .bind(account_id).bind(cash_delta).execute(&mut *tx).await?;

    let _ = notional;
    Ok(())
}

pub async fn list_orders(pool: &PgPool, account_id: Uuid, limit: i64) -> anyhow::Result<Vec<PaperOrder>> {
    Ok(sqlx::query_as::<_, PaperOrder>(
        "SELECT id, paper_account_id, symbol, side::text, qty, order_type::text,
                limit_price, stop_price, status::text,
                filled_price, filled_qty, fee, submitted_at, filled_at, cancel_at, reject_reason
           FROM paper_orders WHERE paper_account_id = $1
          ORDER BY submitted_at DESC LIMIT $2",
    )
    .bind(account_id).bind(limit)
    .fetch_all(pool).await?)
}

pub async fn positions(pool: &PgPool, account_id: Uuid) -> anyhow::Result<Vec<PaperPosition>> {
    Ok(sqlx::query_as::<_, PaperPosition>(
        "SELECT paper_account_id, symbol, qty, avg_price, realized_pnl, updated_at
           FROM paper_positions WHERE paper_account_id = $1 ORDER BY symbol",
    )
    .bind(account_id)
    .fetch_all(pool).await?)
}
