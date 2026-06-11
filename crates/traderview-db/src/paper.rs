//! Paper trading simulator — virtual account filled against the latest
//! cached quote. Mirrors Warrior Trading's $200k SimTrader (minus the
//! live order book — we fill at last price). Market orders fill
//! immediately; untriggered limit/stop orders REST as 'pending' and the
//! background ticker fills them when the quote crosses their trigger.

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

pub async fn reset(
    pool: &PgPool,
    user_id: Uuid,
    account_id: Uuid,
    starting: Decimal,
) -> anyhow::Result<bool> {
    let mut tx = pool.begin().await?;
    let r = sqlx::query(
        "UPDATE paper_accounts SET starting_cash = $3, cash = $3, reset_at = now()
          WHERE id = $1 AND user_id = $2",
    )
    .bind(account_id)
    .bind(user_id)
    .bind(starting)
    .execute(&mut *tx)
    .await?;
    if r.rows_affected() == 0 {
        return Ok(false);
    }
    sqlx::query("DELETE FROM paper_orders WHERE paper_account_id = $1")
        .bind(account_id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM paper_positions WHERE paper_account_id = $1")
        .bind(account_id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(true)
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrderRequest {
    pub symbol: String,
    pub side: Side,
    pub qty: Decimal,
    pub order_type: String, // 'market' | 'limit' | 'stop' | 'stop_limit'
    pub limit_price: Option<Decimal>,
    pub stop_price: Option<Decimal>,
}

/// Price at which an order triggers against the current quote: market
/// always at last; limit when last is at-or-better than the limit;
/// stop when last has crossed the stop. None = does not trigger now
/// (a well-formed limit/stop RESTS as 'pending'; malformed rejects).
pub fn trigger_price(
    order_type: &str,
    side: Side,
    last: Decimal,
    limit_price: Option<Decimal>,
    stop_price: Option<Decimal>,
) -> Option<Decimal> {
    match order_type {
        "market" => Some(last),
        "limit" => match (side, limit_price) {
            (Side::Buy | Side::Cover, Some(lp)) if last <= lp => Some(last),
            (Side::Sell | Side::Short, Some(lp)) if last >= lp => Some(last),
            _ => None,
        },
        "stop" => match (side, stop_price) {
            (Side::Buy | Side::Cover, Some(sp)) if last >= sp => Some(last),
            (Side::Sell | Side::Short, Some(sp)) if last <= sp => Some(last),
            _ => None,
        },
        _ => None,
    }
}

/// Apply the baseline-equity friction model to a triggered price:
/// returns (adjusted fill price, total commission + SEC fee in USD).
fn frictioned_fill(price: Decimal, qty: Decimal, side: Side) -> (Decimal, f64) {
    let cfg = crate::friction::FrictionConfig::baseline_equity();
    let fill_side = match side {
        Side::Buy => crate::friction::FillSide::BuyOpen,
        Side::Sell => crate::friction::FillSide::SellClose,
        Side::Short => crate::friction::FillSide::SellOpen,
        Side::Cover => crate::friction::FillSide::BuyClose,
    };
    let price_f64 = price.to_string().parse::<f64>().unwrap_or(0.0);
    let qty_f64 = qty.to_string().parse::<f64>().unwrap_or(0.0);
    let f = crate::friction::apply_fill_friction(price_f64, qty_f64, fill_side, cfg);
    (
        Decimal::try_from(f.fill_price).unwrap_or(price),
        f.commission_usd + f.sec_fee_usd,
    )
}

/// Submit a paper order against the latest cached quote. Market (and
/// already-triggered limit/stop) orders fill immediately; well-formed
/// limit/stop orders that don't trigger REST as 'pending' and are
/// filled by the background ticker when the quote crosses. Malformed
/// orders (missing the price their type needs) reject.
pub async fn submit(
    pool: &PgPool,
    user_id: Uuid,
    account_id: Uuid,
    req: OrderRequest,
) -> anyhow::Result<PaperOrder> {
    // Ownership check.
    let owner: Option<(Uuid,)> = sqlx::query_as("SELECT user_id FROM paper_accounts WHERE id = $1")
        .bind(account_id)
        .fetch_optional(pool)
        .await?;
    if !matches!(owner, Some((u,)) if u == user_id) {
        anyhow::bail!("forbidden");
    }

    let quote = crate::market_data::quote(pool, &req.symbol).await?;
    let last = Decimal::try_from(quote.price)?;

    let triggered = trigger_price(&req.order_type, req.side, last, req.limit_price, req.stop_price);

    // Apply friction to the trigger price so paper fills track live
    // execution: buyer pays slippage up, seller receives slippage down,
    // commission + SEC fee charged separately.
    let (fill_price, total_fee_usd) = match triggered {
        None => (None, 0.0),
        Some(p) => {
            let (adjusted, fee) = frictioned_fill(p, req.qty, req.side);
            (Some(adjusted), fee)
        }
    };

    let side_str = match req.side {
        Side::Buy => "buy",
        Side::Sell => "sell",
        Side::Short => "short",
        Side::Cover => "cover",
    };

    // Untriggered but well-formed limit/stop orders REST; only orders
    // missing the price their type requires (or an unknown type) reject.
    let well_formed = matches!(
        (req.order_type.as_str(), req.limit_price, req.stop_price),
        ("limit", Some(_), _) | ("stop", _, Some(_))
    );
    let mut tx = pool.begin().await?;
    let (status, filled_at, reject) = match (fill_price, well_formed) {
        (Some(_), _) => ("filled", Some(Utc::now()), None),
        (None, true) => ("pending", None, None),
        (None, false) => (
            "rejected",
            None,
            Some("order type needs its limit/stop price".to_string()),
        ),
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
    .bind(account_id).bind(req.symbol.to_uppercase()).bind(side_str)
    .bind(req.qty).bind(&req.order_type).bind(req.limit_price).bind(req.stop_price)
    .bind(status).bind(fill_price).bind(fill_price.map(|_| req.qty))
    .bind(filled_at).bind(reject)
    .fetch_one(&mut *tx)
    .await?;

    if let Some(price) = fill_price {
        apply_fill(
            &mut tx,
            account_id,
            &req.symbol.to_uppercase(),
            req.side,
            req.qty,
            price,
        )
        .await?;
        // Commission + SEC fee deducted from cash on top of the
        // already-friction-adjusted fill_price. Fees go negative on
        // cash regardless of side.
        deduct_fee(&mut tx, account_id, total_fee_usd).await?;
    }
    tx.commit().await?;
    Ok(order)
}

async fn deduct_fee(
    tx: &mut sqlx::PgConnection,
    account_id: Uuid,
    total_fee_usd: f64,
) -> anyhow::Result<()> {
    if total_fee_usd > 0.0 {
        if let Ok(fee_dec) = Decimal::try_from(total_fee_usd) {
            sqlx::query("UPDATE paper_accounts SET cash = cash - $2 WHERE id = $1")
                .bind(account_id)
                .bind(fee_dec)
                .execute(&mut *tx)
                .await?;
        }
    }
    Ok(())
}

/// One ticker pass over RESTING orders: fill every pending limit/stop
/// whose trigger the current quote satisfies. The status='pending'
/// guard on the claiming UPDATE makes a racing duplicate pass a no-op.
/// Returns the number of orders filled.
pub async fn check_pending(pool: &PgPool) -> anyhow::Result<usize> {
    #[derive(sqlx::FromRow)]
    struct Pending {
        id: Uuid,
        paper_account_id: Uuid,
        symbol: String,
        side: String,
        qty: Decimal,
        order_type: String,
        limit_price: Option<Decimal>,
        stop_price: Option<Decimal>,
    }
    let rows: Vec<Pending> = sqlx::query_as(
        "SELECT id, paper_account_id, symbol, side::text, qty, order_type::text,
                limit_price, stop_price
           FROM paper_orders
          WHERE status = 'pending'
          ORDER BY submitted_at
          LIMIT 200",
    )
    .fetch_all(pool)
    .await?;
    let mut filled = 0usize;
    for o in rows {
        let Ok(side) = serde_json::from_value::<Side>(serde_json::Value::String(o.side.clone()))
        else {
            continue;
        };
        // Quote failures are transient (rate limit, network) — the
        // order keeps resting and the next pass retries.
        let Ok(quote) = crate::market_data::quote(pool, &o.symbol).await else {
            continue;
        };
        let Ok(last) = Decimal::try_from(quote.price) else {
            continue;
        };
        let Some(p) = trigger_price(&o.order_type, side, last, o.limit_price, o.stop_price)
        else {
            continue;
        };
        let (adjusted, fee) = frictioned_fill(p, o.qty, side);
        let mut tx = pool.begin().await?;
        let claimed = sqlx::query(
            "UPDATE paper_orders
                SET status = 'filled', filled_price = $2, filled_qty = qty, filled_at = now()
              WHERE id = $1 AND status = 'pending'",
        )
        .bind(o.id)
        .bind(adjusted)
        .execute(&mut *tx)
        .await?
        .rows_affected();
        if claimed == 0 {
            tx.rollback().await?;
            continue;
        }
        apply_fill(&mut tx, o.paper_account_id, &o.symbol, side, o.qty, adjusted).await?;
        deduct_fee(&mut tx, o.paper_account_id, fee).await?;
        tx.commit().await?;
        filled += 1;
    }
    Ok(filled)
}

/// Cancel a RESTING order. Only 'pending' cancels; filled is history.
pub async fn cancel_order(pool: &PgPool, user_id: Uuid, order_id: Uuid) -> anyhow::Result<bool> {
    let res = sqlx::query(
        "UPDATE paper_orders o SET status = 'cancelled'
           FROM paper_accounts a
          WHERE o.id = $1 AND o.paper_account_id = a.id
            AND a.user_id = $2 AND o.status = 'pending'",
    )
    .bind(order_id)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(res.rows_affected() > 0)
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
    // FOR UPDATE locks the row for the duration of this tx so two
    // concurrent fills on the same (paper_account_id, symbol) serialize
    // — without it the SELECT-Rust-compute-INSERT race silently lost
    // one fill's qty delta (last writer of the ON CONFLICT DO UPDATE
    // wins with its pre-conflict snapshot). The lock is row-level and
    // releases at COMMIT.
    let row: Option<(Decimal, Decimal, Decimal)> = sqlx::query_as(
        "SELECT qty, avg_price, realized_pnl FROM paper_positions
          WHERE paper_account_id = $1 AND symbol = $2
          FOR UPDATE",
    )
    .bind(account_id)
    .bind(symbol)
    .fetch_optional(&mut *tx)
    .await?;

    let (new_qty, new_avg, new_realized) = match row {
        None => (signed_qty, price, Decimal::ZERO),
        Some((cur_qty, cur_avg, cur_realized)) => {
            let same_sign = (cur_qty > Decimal::ZERO && signed_qty > Decimal::ZERO)
                || (cur_qty < Decimal::ZERO && signed_qty < Decimal::ZERO);
            let new_q = cur_qty + signed_qty;
            if same_sign || cur_qty.is_zero() {
                // Adding to position — weighted-average.
                let total = cur_avg * cur_qty.abs() + price * signed_qty.abs();
                let avg = if new_q.abs() > Decimal::ZERO {
                    total / new_q.abs()
                } else {
                    Decimal::ZERO
                };
                (new_q, avg, cur_realized)
            } else {
                // Reducing or flipping — realize P&L on the part that crosses.
                let close_qty = cur_qty.abs().min(signed_qty.abs());
                let direction = if cur_qty > Decimal::ZERO {
                    Decimal::ONE
                } else {
                    -Decimal::ONE
                };
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
        sqlx::query("DELETE FROM paper_positions WHERE paper_account_id = $1 AND symbol = $2")
            .bind(account_id)
            .bind(symbol)
            .execute(&mut *tx)
            .await?;
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
    let cash_delta = -signed_qty * price; // buy decreases cash, sell increases
    sqlx::query("UPDATE paper_accounts SET cash = cash + $2 WHERE id = $1")
        .bind(account_id)
        .bind(cash_delta)
        .execute(&mut *tx)
        .await?;

    let _ = notional;
    Ok(())
}

pub async fn list_orders(
    pool: &PgPool,
    account_id: Uuid,
    limit: i64,
) -> anyhow::Result<Vec<PaperOrder>> {
    Ok(sqlx::query_as::<_, PaperOrder>(
        "SELECT id, paper_account_id, symbol, side::text, qty, order_type::text,
                limit_price, stop_price, status::text,
                filled_price, filled_qty, fee, submitted_at, filled_at, cancel_at, reject_reason
           FROM paper_orders WHERE paper_account_id = $1
          ORDER BY submitted_at DESC LIMIT $2",
    )
    .bind(account_id)
    .bind(limit)
    .fetch_all(pool)
    .await?)
}

pub async fn positions(pool: &PgPool, account_id: Uuid) -> anyhow::Result<Vec<PaperPosition>> {
    Ok(sqlx::query_as::<_, PaperPosition>(
        "SELECT paper_account_id, symbol, qty, avg_price, realized_pnl, updated_at
           FROM paper_positions WHERE paper_account_id = $1 ORDER BY symbol",
    )
    .bind(account_id)
    .fetch_all(pool)
    .await?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(v: i64) -> Decimal {
        Decimal::from(v)
    }

    #[test]
    fn market_always_triggers_at_last() {
        assert_eq!(trigger_price("market", Side::Buy, d(100), None, None), Some(d(100)));
    }

    #[test]
    fn buy_limit_triggers_at_or_below_rests_above() {
        // Buy limit 100: last 99 fills at 99 (price improvement), last 101 rests.
        assert_eq!(trigger_price("limit", Side::Buy, d(99), Some(d(100)), None), Some(d(99)));
        assert_eq!(trigger_price("limit", Side::Buy, d(100), Some(d(100)), None), Some(d(100)));
        assert_eq!(trigger_price("limit", Side::Buy, d(101), Some(d(100)), None), None);
    }

    #[test]
    fn sell_limit_triggers_at_or_above_rests_below() {
        assert_eq!(trigger_price("limit", Side::Sell, d(101), Some(d(100)), None), Some(d(101)));
        assert_eq!(trigger_price("limit", Side::Sell, d(99), Some(d(100)), None), None);
    }

    #[test]
    fn sell_stop_triggers_when_price_falls_through() {
        // Sell stop 95 (protective): last 94 triggers, last 96 rests.
        assert_eq!(trigger_price("stop", Side::Sell, d(94), None, Some(d(95))), Some(d(94)));
        assert_eq!(trigger_price("stop", Side::Sell, d(96), None, Some(d(95))), None);
    }

    #[test]
    fn buy_stop_triggers_when_price_rises_through() {
        // Buy stop 105 (breakout entry): last 106 triggers, last 104 rests.
        assert_eq!(trigger_price("stop", Side::Buy, d(106), None, Some(d(105))), Some(d(106)));
        assert_eq!(trigger_price("stop", Side::Buy, d(104), None, Some(d(105))), None);
    }

    #[test]
    fn short_and_cover_mirror_sell_and_buy() {
        // Short = sell-side trigger, cover = buy-side trigger.
        assert_eq!(trigger_price("limit", Side::Short, d(101), Some(d(100)), None), Some(d(101)));
        assert_eq!(trigger_price("stop", Side::Cover, d(106), None, Some(d(105))), Some(d(106)));
    }

    #[test]
    fn missing_required_price_or_unknown_type_never_triggers() {
        assert_eq!(trigger_price("limit", Side::Buy, d(99), None, None), None);
        assert_eq!(trigger_price("stop", Side::Sell, d(94), None, None), None);
        assert_eq!(trigger_price("trailing", Side::Buy, d(99), Some(d(100)), Some(d(95))), None);
    }
}
