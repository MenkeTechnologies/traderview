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
    pub drip: bool,
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
    pub trail_value: Option<Decimal>,
    pub trail_is_pct: Option<bool>,
    pub trail_extreme: Option<Decimal>,
    pub oco_group: Option<Uuid>,
    pub parent_order_id: Option<Uuid>,
    pub filled_price: Option<Decimal>,
    pub filled_qty: Option<Decimal>,
    pub fee: Decimal,
    pub submitted_at: DateTime<Utc>,
    pub filled_at: Option<DateTime<Utc>>,
    pub cancel_at: Option<DateTime<Utc>>,
    pub reject_reason: Option<String>,
    pub plan_note: Option<String>,
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
        "SELECT id, user_id, name, starting_cash, cash, drip, created_at, reset_at
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
         RETURNING id, user_id, name, starting_cash, cash, drip, created_at, reset_at",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?)
}

/// Cap so a runaway client can't mint unbounded accounts.
const MAX_ACCOUNTS_PER_USER: i64 = 12;

/// Create a named paper account (e.g. one per strategy). Names are
/// unique per user (enforced by the 0011 UNIQUE constraint; checked
/// here first for a readable error).
pub async fn create_account(
    pool: &PgPool,
    user_id: Uuid,
    name: &str,
    starting_cash: Decimal,
) -> anyhow::Result<PaperAccount> {
    let name = name.trim();
    if name.is_empty() || name.len() > 60 {
        anyhow::bail!("name must be 1..=60 characters");
    }
    if starting_cash <= Decimal::ZERO {
        anyhow::bail!("starting_cash must be positive");
    }
    let (count,): (i64,) =
        sqlx::query_as("SELECT count(*) FROM paper_accounts WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(pool)
            .await?;
    if count >= MAX_ACCOUNTS_PER_USER {
        anyhow::bail!("account limit reached ({MAX_ACCOUNTS_PER_USER})");
    }
    let taken: Option<(Uuid,)> =
        sqlx::query_as("SELECT id FROM paper_accounts WHERE user_id = $1 AND name = $2")
            .bind(user_id)
            .bind(name)
            .fetch_optional(pool)
            .await?;
    if taken.is_some() {
        anyhow::bail!("an account named '{name}' already exists");
    }
    Ok(sqlx::query_as::<_, PaperAccount>(
        "INSERT INTO paper_accounts (user_id, name, starting_cash, cash)
              VALUES ($1, $2, $3, $3)
         RETURNING id, user_id, name, starting_cash, cash, drip, created_at, reset_at",
    )
    .bind(user_id)
    .bind(name)
    .bind(starting_cash)
    .fetch_one(pool)
    .await?)
}

/// Toggle dividend reinvestment for the account.
pub async fn set_drip(
    pool: &PgPool,
    user_id: Uuid,
    account_id: Uuid,
    enabled: bool,
) -> anyhow::Result<bool> {
    let r = sqlx::query(
        "UPDATE paper_accounts SET drip = $3 WHERE id = $1 AND user_id = $2",
    )
    .bind(account_id)
    .bind(user_id)
    .bind(enabled)
    .execute(pool)
    .await?;
    Ok(r.rows_affected() > 0)
}

pub async fn rename_account(
    pool: &PgPool,
    user_id: Uuid,
    account_id: Uuid,
    name: &str,
) -> anyhow::Result<bool> {
    let name = name.trim();
    if name.is_empty() || name.len() > 60 {
        anyhow::bail!("name must be 1..=60 characters");
    }
    let res = sqlx::query(
        "UPDATE paper_accounts SET name = $3 WHERE id = $1 AND user_id = $2",
    )
    .bind(account_id)
    .bind(user_id)
    .bind(name)
    .execute(pool)
    .await?;
    Ok(res.rows_affected() > 0)
}

/// Delete an account; orders, positions, parent orders, and equity
/// snapshots all cascade (FKs in migrations 0011/0076/0081).
pub async fn delete_account(
    pool: &PgPool,
    user_id: Uuid,
    account_id: Uuid,
) -> anyhow::Result<bool> {
    let res = sqlx::query("DELETE FROM paper_accounts WHERE id = $1 AND user_id = $2")
        .bind(account_id)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(res.rows_affected() > 0)
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
    pub order_type: String, // 'market' | 'limit' | 'stop' | 'trailing'
    pub limit_price: Option<Decimal>,
    pub stop_price: Option<Decimal>,
    /// Trailing stop distance: dollars, or a fraction of the extreme
    /// (e.g. 0.05 = 5%) when trail_is_pct.
    #[serde(default)]
    pub trail_value: Option<Decimal>,
    #[serde(default)]
    pub trail_is_pct: Option<bool>,
    /// Time in force for RESTING orders: 'gtc' (default), 'day'
    /// (expires at the next 16:00 US Eastern close), or 'gtd'
    /// (expires at expire_at). Ignored on orders that fill immediately.
    #[serde(default)]
    pub time_in_force: Option<String>,
    #[serde(default)]
    pub expire_at: Option<DateTime<Utc>>,
    /// Written trade plan; a non-empty note satisfies the
    /// RequirePlanBeforeTrade risk rule and persists with the order.
    #[serde(default)]
    pub plan_note: Option<String>,
}

/// Does this order carry the price/trail its type requires? A
/// trailing trail is dollars > 0, or a fraction in (0, 1) of the
/// ratcheting extreme when trail_is_pct. Market is NOT well-formed
/// for RESTING purposes — it always fills at submit and can never
/// rest. Shared by submit (reject malformed) and replace_order
/// (refuse an amendment that would strand the order).
pub fn order_well_formed(
    order_type: &str,
    limit_price: Option<Decimal>,
    stop_price: Option<Decimal>,
    trail_value: Option<Decimal>,
    trail_is_pct: Option<bool>,
) -> bool {
    let trail_ok = trail_value.is_some_and(|v| {
        v > Decimal::ZERO && (!trail_is_pct.unwrap_or(false) || v < Decimal::ONE)
    });
    matches!(
        (order_type, limit_price, stop_price),
        ("limit", Some(_), _) | ("stop", _, Some(_)) | ("stop_limit", Some(_), Some(_))
    ) || (order_type == "trailing" && trail_ok)
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

/// Stop-limit state machine. Untriggered: the stop must cross first
/// (buy: last >= stop, sell: last <= stop). Once crossed the order is
/// PERMANENTLY a plain limit — a gap through the stop does NOT fill
/// at the gapped price (that protection is the entire point of the
/// type), but a later recovery to limit-or-better does fill, which is
/// why the triggered state must persist rather than re-checking the
/// stop on every tick.
pub fn stop_limit_action(
    side: Side,
    last: Decimal,
    stop: Decimal,
    limit: Decimal,
    already_triggered: bool,
) -> (bool, Option<Decimal>) {
    let crossed = already_triggered
        || match side {
            Side::Buy | Side::Cover => last >= stop,
            Side::Sell | Side::Short => last <= stop,
        };
    if !crossed {
        return (false, None);
    }
    (true, trigger_price("limit", side, last, Some(limit), None))
}

/// Bracket sanity: exits must sit on the correct sides of each other,
/// and a known entry price must sit between them. Long brackets need
/// stop below target; short brackets are the mirror.
pub fn validate_bracket(
    side: Side,
    entry_hint: Option<Decimal>,
    stop_loss: Decimal,
    take_profit: Decimal,
) -> Result<(), &'static str> {
    if !matches!(side, Side::Buy | Side::Short) {
        return Err("bracket entry side must be buy or short");
    }
    let long = matches!(side, Side::Buy);
    let (lo, hi) = if long {
        (stop_loss, take_profit)
    } else {
        (take_profit, stop_loss)
    };
    if lo >= hi {
        return Err(if long {
            "buy bracket needs stop_loss < take_profit"
        } else {
            "short bracket needs take_profit < stop_loss"
        });
    }
    if let Some(e) = entry_hint {
        if e <= lo || e >= hi {
            return Err("entry price must sit between stop and target");
        }
    }
    Ok(())
}

/// Trailing-stop ratchet: fold the latest price into the tracked
/// extreme (high-water for sell/short exits, low-water for buy/cover
/// entries-or-covers) and report whether the retrace from that extreme
/// has reached the trail distance. Returns (new extreme, triggered).
pub fn trail_update(
    side: Side,
    last: Decimal,
    extreme: Decimal,
    trail_value: Decimal,
    trail_is_pct: bool,
) -> (Decimal, bool) {
    let sellish = matches!(side, Side::Sell | Side::Short);
    let new_extreme = if sellish {
        extreme.max(last)
    } else {
        extreme.min(last)
    };
    let trail = if trail_is_pct {
        new_extreme * trail_value
    } else {
        trail_value
    };
    let triggered = if sellish {
        last <= new_extreme - trail
    } else {
        last >= new_extreme + trail
    };
    (new_extreme, triggered)
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

    // One quote resolver for equities AND OCC options — every order
    // type (market/limit/stop/trailing, TIF, brackets) flows through
    // the same path; only the quote source, multiplier, and fee model
    // differ. Expired contracts reject at submit.
    let rq = resolve_quote(pool, &req.symbol).await?;
    let last = rq.last;

    let triggered = if req.order_type == "stop_limit" {
        match (req.stop_price, req.limit_price) {
            (Some(sp), Some(lp)) => stop_limit_action(req.side, last, sp, lp, false).1,
            _ => None,
        }
    } else {
        trigger_price(&req.order_type, req.side, last, req.limit_price, req.stop_price)
    };
    // A stop-limit submitted with the stop already crossed but the
    // limit not satisfiable rests ALREADY TRIGGERED — it's a limit
    // order from birth.
    let stop_triggered = req.order_type == "stop_limit"
        && triggered.is_none()
        && matches!((req.stop_price, req.limit_price),
            (Some(sp), Some(lp)) if stop_limit_action(req.side, last, sp, lp, false).0);

    // Equities get the friction model (slippage + commission + SEC
    // fee); options fill at the chain mid with a flat per-contract
    // commission — bid/ask mid IS the slippage model for options.
    let (fill_price, total_fee_usd) = match triggered {
        None => (None, 0.0),
        Some(p) if rq.is_option => {
            (Some(p), OPTION_COMMISSION_PER_CONTRACT * req.qty.to_string().parse::<f64>().unwrap_or(0.0))
        }
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

    // Untriggered but well-formed limit/stop/trailing orders REST; only
    // orders missing the price/trail their type requires (or an unknown
    // type) reject.
    let well_formed = order_well_formed(
        &req.order_type,
        req.limit_price,
        req.stop_price,
        req.trail_value,
        req.trail_is_pct,
    );
    // A resting trailing stop starts its ratchet at the current price.
    let trail_extreme = (req.order_type == "trailing" && well_formed).then_some(last);
    // Time in force → cancel_at. Only meaningful on orders that rest.
    let cancel_at = match req.time_in_force.as_deref().unwrap_or("gtc") {
        "gtc" => None,
        "day" => Some(traderview_core::holiday_calendar::day_order_expiry(
            Utc::now(),
        )),
        "gtd" => {
            let at = req
                .expire_at
                .ok_or_else(|| anyhow::anyhow!("gtd needs expire_at"))?;
            if at <= Utc::now() {
                anyhow::bail!("expire_at must be in the future");
            }
            Some(at)
        }
        other => anyhow::bail!("unknown time_in_force '{other}'"),
    };
    let mut tx = pool.begin().await?;
    let (status, filled_at, reject) = match (fill_price, well_formed) {
        (Some(_), _) => ("filled", Some(Utc::now()), None),
        (None, true) => ("pending", None, None),
        (None, false) => (
            "rejected",
            None,
            Some("order type needs its limit/stop price or trail value".to_string()),
        ),
    };
    let order: PaperOrder = sqlx::query_as(
        "INSERT INTO paper_orders
            (paper_account_id, symbol, side, qty, order_type, limit_price, stop_price,
             status, filled_price, filled_qty, filled_at, reject_reason,
             trail_value, trail_is_pct, trail_extreme, cancel_at, plan_note, stop_triggered)
         VALUES ($1, $2, $3::side_t, $4, $5::paper_order_type_t, $6, $7,
                 $8::paper_order_status_t, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
         RETURNING id, paper_account_id, symbol, side::text, qty, order_type::text,
                   limit_price, stop_price, status::text,
                   trail_value, trail_is_pct, trail_extreme, oco_group, parent_order_id,
                   filled_price, filled_qty, fee, submitted_at, filled_at, cancel_at, reject_reason, plan_note",
    )
    .bind(account_id).bind(req.symbol.to_uppercase()).bind(side_str)
    .bind(req.qty).bind(&req.order_type).bind(req.limit_price).bind(req.stop_price)
    .bind(status).bind(fill_price).bind(fill_price.map(|_| req.qty))
    .bind(filled_at).bind(reject)
    .bind(req.trail_value).bind(req.trail_is_pct).bind(trail_extreme).bind(cancel_at)
    .bind(req.plan_note.as_deref().map(str::trim).filter(|s| !s.is_empty()))
    .bind(stop_triggered)
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
            rq.multiplier,
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

/// Details of a background fill, for the caller to surface (live feed,
/// notifications) — the user wasn't watching when it happened.
#[derive(Debug, Clone)]
pub struct BackgroundFill {
    pub symbol: String,
    pub side: String,
    pub qty: Decimal,
    pub price: Decimal,
    pub order_type: String,
}

/// One ticker pass over RESTING orders: fill every pending limit/stop
/// whose trigger the current quote satisfies. The status='pending'
/// guard on the claiming UPDATE makes a racing duplicate pass a no-op.
/// Returns the fills executed this pass.
pub async fn check_pending(pool: &PgPool) -> anyhow::Result<Vec<BackgroundFill>> {
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
        trail_value: Option<Decimal>,
        trail_is_pct: Option<bool>,
        trail_extreme: Option<Decimal>,
        oco_group: Option<Uuid>,
        stop_triggered: bool,
    }
    // Expire resting orders whose time in force has lapsed, then any
    // held legs orphaned by a cancelled/expired entry.
    sqlx::query(
        "UPDATE paper_orders
            SET status = 'cancelled', reject_reason = 'expired (time in force)'
          WHERE status IN ('pending', 'held')
            AND cancel_at IS NOT NULL AND cancel_at <= now()",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "UPDATE paper_orders SET status = 'cancelled'
          WHERE status = 'held'
            AND parent_order_id IN (SELECT id FROM paper_orders WHERE status = 'cancelled')",
    )
    .execute(pool)
    .await?;
    // Promote bracket exit legs whose entry has filled: held → pending.
    sqlx::query(
        "UPDATE paper_orders SET status = 'pending'
          WHERE status = 'held'
            AND parent_order_id IN (SELECT id FROM paper_orders WHERE status = 'filled')",
    )
    .execute(pool)
    .await?;
    let rows: Vec<Pending> = sqlx::query_as(
        "SELECT id, paper_account_id, symbol, side::text, qty, order_type::text,
                limit_price, stop_price, trail_value, trail_is_pct, trail_extreme, oco_group,
                stop_triggered
           FROM paper_orders
          WHERE status = 'pending'
          ORDER BY submitted_at
          LIMIT 200",
    )
    .fetch_all(pool)
    .await?;
    let mut filled: Vec<BackgroundFill> = Vec::new();
    for o in rows {
        let Ok(side) = serde_json::from_value::<Side>(serde_json::Value::String(o.side.clone()))
        else {
            continue;
        };
        // Quote failures are transient (rate limit, network) — the
        // order keeps resting and the next pass retries. EXPIRED option
        // contracts are not transient: cancel them with the reason.
        let occ = traderview_core::occ_symbol::parse(&o.symbol);
        let (last, multiplier, is_option) = if let Some(occ) = &occ {
            if occ.expiry < Utc::now().date_naive() {
                sqlx::query(
                    "UPDATE paper_orders
                        SET status = 'cancelled', reject_reason = 'contract expired'
                      WHERE id = $1 AND status = 'pending'",
                )
                .bind(o.id)
                .execute(pool)
                .await
                .ok();
                continue;
            }
            let Ok(Some(p)) = option_quote(occ).await else {
                continue; // no usable chain quote — keep resting
            };
            let Ok(p) = Decimal::try_from(p) else { continue };
            (p, Decimal::from(100), true)
        } else {
            let Ok(quote) = crate::market_data::quote(pool, &o.symbol).await else {
                continue;
            };
            let Ok(last) = Decimal::try_from(quote.price) else {
                continue;
            };
            (last, Decimal::ONE, false)
        };
        let p = if o.order_type == "trailing" {
            let (Some(tv), Some(extreme)) = (o.trail_value, o.trail_extreme) else {
                continue;
            };
            let (new_extreme, triggered) =
                trail_update(side, last, extreme, tv, o.trail_is_pct.unwrap_or(false));
            if !triggered {
                if new_extreme != extreme {
                    sqlx::query("UPDATE paper_orders SET trail_extreme = $2 WHERE id = $1")
                        .bind(o.id)
                        .bind(new_extreme)
                        .execute(pool)
                        .await
                        .ok();
                }
                continue;
            }
            last
        } else if o.order_type == "stop_limit" {
            let (Some(sp), Some(lp)) = (o.stop_price, o.limit_price) else {
                continue;
            };
            let (now_triggered, fill) = stop_limit_action(side, last, sp, lp, o.stop_triggered);
            let Some(p) = fill else {
                // Persist the stop→limit transition so a gap through
                // the stop arms the order even if the process restarts
                // before the limit is ever satisfied.
                if now_triggered && !o.stop_triggered {
                    sqlx::query("UPDATE paper_orders SET stop_triggered = TRUE WHERE id = $1")
                        .bind(o.id)
                        .execute(pool)
                        .await
                        .ok();
                }
                continue;
            };
            p
        } else {
            let Some(p) = trigger_price(&o.order_type, side, last, o.limit_price, o.stop_price)
            else {
                continue;
            };
            p
        };
        let (adjusted, fee) = if is_option {
            (p, OPTION_COMMISSION_PER_CONTRACT * o.qty.to_string().parse::<f64>().unwrap_or(0.0))
        } else {
            frictioned_fill(p, o.qty, side)
        };
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
        apply_fill(&mut tx, o.paper_account_id, &o.symbol, side, o.qty, adjusted, multiplier).await?;
        deduct_fee(&mut tx, o.paper_account_id, fee).await?;
        // OCO: the first leg to fill kills its siblings — atomically
        // with the fill so a crash can't leave both legs live.
        if let Some(group) = o.oco_group {
            sqlx::query(
                "UPDATE paper_orders SET status = 'cancelled'
                  WHERE oco_group = $1 AND id <> $2 AND status IN ('pending', 'held')",
            )
            .bind(group)
            .bind(o.id)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        filled.push(BackgroundFill {
            symbol: o.symbol.clone(),
            side: o.side.clone(),
            qty: o.qty,
            price: adjusted,
            order_type: o.order_type.clone(),
        });
    }
    Ok(filled)
}

/// Cancel a RESTING ('pending') or bracket-held order. Cancelling an
/// entry also cancels its still-held exit legs — they could never
/// activate. Filled is history.
pub async fn cancel_order(pool: &PgPool, user_id: Uuid, order_id: Uuid) -> anyhow::Result<bool> {
    let res = sqlx::query(
        "UPDATE paper_orders o SET status = 'cancelled'
           FROM paper_accounts a
          WHERE o.id = $1 AND o.paper_account_id = a.id
            AND a.user_id = $2 AND o.status IN ('pending', 'held')",
    )
    .bind(order_id)
    .bind(user_id)
    .execute(pool)
    .await?;
    let cancelled = res.rows_affected() > 0;
    if cancelled {
        sqlx::query(
            "UPDATE paper_orders SET status = 'cancelled'
              WHERE parent_order_id = $1 AND status = 'held'",
        )
        .bind(order_id)
        .execute(pool)
        .await?;
    }
    Ok(cancelled)
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReplaceRequest {
    #[serde(default)]
    pub qty: Option<Decimal>,
    #[serde(default)]
    pub limit_price: Option<Decimal>,
    #[serde(default)]
    pub stop_price: Option<Decimal>,
    #[serde(default)]
    pub trail_value: Option<Decimal>,
    #[serde(default)]
    pub trail_is_pct: Option<bool>,
}

/// Cancel/replace in one step: amend a RESTING order's qty/prices
/// without losing the order row or its submitted_at audit trail.
/// None fields keep their current values (no way to clear a price —
/// that would change the order's type). Held bracket legs are
/// deliberately not replaceable: amending a leg's qty would desync it
/// from its entry. The claim UPDATE re-checks status='pending' so a
/// fill racing this call wins cleanly.
pub async fn replace_order(
    pool: &PgPool,
    user_id: Uuid,
    order_id: Uuid,
    req: ReplaceRequest,
) -> anyhow::Result<PaperOrder> {
    let cur: Option<PaperOrder> = sqlx::query_as(
        "SELECT o.id, o.paper_account_id, o.symbol, o.side::text, o.qty, o.order_type::text,
                o.limit_price, o.stop_price, o.status::text,
                o.trail_value, o.trail_is_pct, o.trail_extreme, o.oco_group, o.parent_order_id,
                o.filled_price, o.filled_qty, o.fee, o.submitted_at, o.filled_at, o.cancel_at,
                o.reject_reason, o.plan_note
           FROM paper_orders o JOIN paper_accounts a ON a.id = o.paper_account_id
          WHERE o.id = $1 AND a.user_id = $2",
    )
    .bind(order_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;
    let Some(cur) = cur else {
        anyhow::bail!("order not found");
    };
    if cur.status != "pending" {
        anyhow::bail!("only resting (pending) orders can be replaced");
    }
    let qty = req.qty.unwrap_or(cur.qty);
    if qty <= Decimal::ZERO {
        anyhow::bail!("qty must be positive");
    }
    let limit_price = req.limit_price.or(cur.limit_price);
    let stop_price = req.stop_price.or(cur.stop_price);
    let trail_value = req.trail_value.or(cur.trail_value);
    let trail_is_pct = req.trail_is_pct.or(cur.trail_is_pct);
    if !order_well_formed(&cur.order_type, limit_price, stop_price, trail_value, trail_is_pct) {
        anyhow::bail!("replacement would strand the order: its type still needs its limit/stop price or trail value");
    }
    let updated: Option<PaperOrder> = sqlx::query_as(
        "UPDATE paper_orders
            SET qty = $2, limit_price = $3, stop_price = $4, trail_value = $5, trail_is_pct = $6
          WHERE id = $1 AND status = 'pending'
         RETURNING id, paper_account_id, symbol, side::text, qty, order_type::text,
                   limit_price, stop_price, status::text,
                   trail_value, trail_is_pct, trail_extreme, oco_group, parent_order_id,
                   filled_price, filled_qty, fee, submitted_at, filled_at, cancel_at,
                   reject_reason, plan_note",
    )
    .bind(order_id)
    .bind(qty)
    .bind(limit_price)
    .bind(stop_price)
    .bind(trail_value)
    .bind(trail_is_pct)
    .fetch_optional(pool)
    .await?;
    updated.ok_or_else(|| anyhow::anyhow!("order filled or cancelled while replacing"))
}

#[derive(Debug, Clone, Deserialize)]
pub struct BracketRequest {
    pub symbol: String,
    pub side: Side, // buy or short — the ENTRY direction
    pub qty: Decimal,
    pub entry_type: String, // 'market' | 'limit'
    pub limit_price: Option<Decimal>,
    pub stop_loss: Decimal,
    pub take_profit: Decimal,
}

#[derive(Debug, Clone, Serialize)]
pub struct Bracket {
    pub entry: PaperOrder,
    pub stop: PaperOrder,
    pub target: PaperOrder,
}

/// Submit a bracket: entry through the normal path (market fills now,
/// limit rests), then two exit legs sharing an oco_group — a stop at
/// stop_loss and a limit at take_profit. Legs rest 'held' until the
/// entry fills (the ticker promotes them), or 'pending' immediately
/// when the entry filled on the spot. First leg to fill cancels the
/// other.
pub async fn submit_bracket(
    pool: &PgPool,
    user_id: Uuid,
    account_id: Uuid,
    req: BracketRequest,
) -> anyhow::Result<Bracket> {
    if !matches!(req.entry_type.as_str(), "market" | "limit") {
        anyhow::bail!("entry_type must be 'market' or 'limit'");
    }
    if req.entry_type == "limit" && req.limit_price.is_none() {
        anyhow::bail!("limit entry needs limit_price");
    }
    let entry_hint = (req.entry_type == "limit")
        .then_some(req.limit_price)
        .flatten();
    validate_bracket(req.side, entry_hint, req.stop_loss, req.take_profit)
        .map_err(|e| anyhow::anyhow!(e))?;

    let entry = submit(
        pool,
        user_id,
        account_id,
        OrderRequest {
            symbol: req.symbol.clone(),
            side: req.side,
            qty: req.qty,
            order_type: req.entry_type.clone(),
            limit_price: req.limit_price,
            stop_price: None,
            trail_value: None,
            trail_is_pct: None,
            time_in_force: None,
            expire_at: None,
            plan_note: None,
        },
    )
    .await?;
    if entry.status == "rejected" {
        anyhow::bail!(
            "entry rejected: {}",
            entry.reject_reason.as_deref().unwrap_or("unknown")
        );
    }

    let exit_side = match req.side {
        Side::Buy => "sell",
        _ => "cover",
    };
    let leg_status = if entry.status == "filled" {
        "pending"
    } else {
        "held"
    };
    let group = Uuid::new_v4();
    let symbol = req.symbol.trim().to_uppercase();
    let mut tx = pool.begin().await?;
    let stop = insert_leg(
        &mut tx, account_id, &symbol, exit_side, req.qty,
        "stop", None, Some(req.stop_loss), leg_status, group, Some(entry.id),
    )
    .await?;
    let target = insert_leg(
        &mut tx, account_id, &symbol, exit_side, req.qty,
        "limit", Some(req.take_profit), None, leg_status, group, Some(entry.id),
    )
    .await?;
    tx.commit().await?;
    Ok(Bracket { entry, stop, target })
}

/// Protection sanity for an EXISTING position: derive the bracket
/// entry-side semantics from the position's sign and bound the
/// protected qty by what's held. Returns (entry_side for
/// validate_bracket, exit side string).
pub fn validate_protection(
    pos_qty: Decimal,
    req_qty: Decimal,
) -> Result<(Side, &'static str), &'static str> {
    if req_qty <= Decimal::ZERO {
        return Err("qty must be positive");
    }
    if pos_qty == Decimal::ZERO {
        return Err("no position to protect");
    }
    if req_qty > pos_qty.abs() {
        return Err("qty exceeds position size");
    }
    Ok(if pos_qty > Decimal::ZERO {
        (Side::Buy, "sell")
    } else {
        (Side::Short, "cover")
    })
}

#[derive(Debug, Serialize)]
pub struct Protection {
    pub stop: PaperOrder,
    pub target: PaperOrder,
}

/// Attach an OCO stop/target pair to a position already held — the
/// gap brackets can't cover: a bracket bundles its own entry, but a
/// market buy, a recurring-invest fill, or a DRIP share has no exits.
/// Both legs rest as plain pending orders sharing an oco_group (no
/// parent — there is no entry), so the 5s ticker's first-fill-cancels-
/// sibling machinery applies unchanged. The price relationship is
/// checked by the same validate_bracket as real brackets, with the
/// CURRENT quote as the entry hint — a stop above the market on a
/// long is rejected here, not discovered as an instant fill.
pub async fn attach_protection(
    pool: &PgPool,
    user_id: Uuid,
    account_id: Uuid,
    symbol: &str,
    qty: Decimal,
    stop_loss: Decimal,
    take_profit: Decimal,
) -> anyhow::Result<Protection> {
    let owner: Option<(Uuid,)> = sqlx::query_as("SELECT user_id FROM paper_accounts WHERE id = $1")
        .bind(account_id)
        .fetch_optional(pool)
        .await?;
    if !matches!(owner, Some((u,)) if u == user_id) {
        anyhow::bail!("forbidden");
    }
    let symbol = symbol.trim().to_uppercase();
    let pos: Option<(Decimal,)> = sqlx::query_as(
        "SELECT qty FROM paper_positions WHERE paper_account_id = $1 AND symbol = $2",
    )
    .bind(account_id)
    .bind(&symbol)
    .fetch_optional(pool)
    .await?;
    let pos_qty = pos.map(|(q,)| q).unwrap_or(Decimal::ZERO);
    let (entry_side, exit_side) =
        validate_protection(pos_qty, qty).map_err(|e| anyhow::anyhow!(e))?;
    // Also rejects expired option contracts at attach time.
    let rq = resolve_quote(pool, &symbol).await?;
    validate_bracket(entry_side, Some(rq.last), stop_loss, take_profit)
        .map_err(|e| anyhow::anyhow!(e))?;
    let group = Uuid::new_v4();
    let mut tx = pool.begin().await?;
    let stop = insert_leg(
        &mut tx, account_id, &symbol, exit_side, qty,
        "stop", None, Some(stop_loss), "pending", group, None,
    )
    .await?;
    let target = insert_leg(
        &mut tx, account_id, &symbol, exit_side, qty,
        "limit", Some(take_profit), None, "pending", group, None,
    )
    .await?;
    tx.commit().await?;
    Ok(Protection { stop, target })
}

#[allow(clippy::too_many_arguments)]
async fn insert_leg(
    tx: &mut sqlx::PgConnection,
    account_id: Uuid,
    symbol: &str,
    side: &str,
    qty: Decimal,
    order_type: &str,
    limit_price: Option<Decimal>,
    stop_price: Option<Decimal>,
    status: &str,
    group: Uuid,
    parent: Option<Uuid>,
) -> anyhow::Result<PaperOrder> {
    Ok(sqlx::query_as(
        "INSERT INTO paper_orders
            (paper_account_id, symbol, side, qty, order_type, limit_price, stop_price,
             status, oco_group, parent_order_id)
         VALUES ($1, $2, $3::side_t, $4, $5::paper_order_type_t, $6, $7,
                 $8::paper_order_status_t, $9, $10)
         RETURNING id, paper_account_id, symbol, side::text, qty, order_type::text,
                   limit_price, stop_price, status::text,
                   trail_value, trail_is_pct, trail_extreme, oco_group, parent_order_id,
                   filled_price, filled_qty, fee, submitted_at, filled_at, cancel_at, reject_reason, plan_note",
    )
    .bind(account_id)
    .bind(symbol)
    .bind(side)
    .bind(qty)
    .bind(order_type)
    .bind(limit_price)
    .bind(stop_price)
    .bind(status)
    .bind(group)
    .bind(parent)
    .fetch_one(&mut *tx)
    .await?)
}

/// Chain-quoted price for one OCC contract: mid of bid/ask, last as
/// fallback, None when the contract has no usable price.
pub(crate) async fn option_quote(
    occ: &traderview_core::occ_symbol::OccContract,
) -> anyhow::Result<Option<f64>> {
    let chain = crate::options::chain(&occ.underlying, Some(occ.expiry)).await?;
    let list = if occ.call { &chain.calls } else { &chain.puts };
    Ok(list
        .iter()
        .find(|c| (c.strike - occ.strike).abs() < 1e-6)
        .and_then(|c| traderview_core::occ_symbol::fill_price(c.bid, c.ask, c.last_price)))
}

/// Per-contract commission for paper option fills (retail standard).
const OPTION_COMMISSION_PER_CONTRACT: f64 = 0.65;

#[derive(Debug, Clone, Deserialize)]
pub struct SpreadRequest {
    pub legs: Vec<traderview_core::option_spread::SpreadLeg>,
    /// Spread quantity: each leg fills qty × ratio contracts.
    pub qty: Decimal,
}

#[derive(Debug, Clone, Serialize)]
pub struct SpreadResult {
    pub orders: Vec<PaperOrder>,
    /// Per-share net premium × 100 × qty: positive = credit collected.
    pub net_premium_usd: f64,
    pub total_fees_usd: f64,
}

/// Atomic multi-leg option order: every leg quotes from the chain mid
/// and fills in ONE transaction — a spread never partially fills. Any
/// unquotable or expired leg fails the whole submit (the atomicity IS
/// the feature; legging in is what the single-order ticket is for).
pub async fn submit_spread(
    pool: &PgPool,
    user_id: Uuid,
    account_id: Uuid,
    req: SpreadRequest,
) -> anyhow::Result<SpreadResult> {
    traderview_core::option_spread::validate(&req.legs).map_err(|e| anyhow::anyhow!(e))?;
    if req.qty <= Decimal::ZERO || req.qty > Decimal::from(1000) {
        anyhow::bail!("qty must be in 1..=1000 spreads");
    }
    let owner: Option<(Uuid,)> = sqlx::query_as("SELECT user_id FROM paper_accounts WHERE id = $1")
        .bind(account_id)
        .fetch_optional(pool)
        .await?;
    if !matches!(owner, Some((u,)) if u == user_id) {
        anyhow::bail!("forbidden");
    }
    // Quote every leg BEFORE touching the book — atomicity.
    let mut prices = Vec::with_capacity(req.legs.len());
    for leg in &req.legs {
        let rq = resolve_quote(pool, &leg.symbol).await?;
        prices.push(rq.last.to_string().parse::<f64>().unwrap_or(0.0));
    }
    let per_share = traderview_core::option_spread::net_premium(&req.legs, &prices)
        .ok_or_else(|| anyhow::anyhow!("degenerate leg prices"))?;
    let qty_f = req.qty.to_string().parse::<f64>().unwrap_or(0.0);
    let net_premium_usd = per_share * 100.0 * qty_f;
    let mut total_fees_usd = 0.0;
    let mut tx = pool.begin().await?;
    let mut orders = Vec::with_capacity(req.legs.len());
    let mut first_id: Option<Uuid> = None;
    for (leg, price) in req.legs.iter().zip(&prices) {
        let leg_qty = req.qty * Decimal::from(leg.ratio);
        let side = if leg.buy { Side::Buy } else { Side::Sell };
        let side_str = if leg.buy { "buy" } else { "sell" };
        let price_dec = Decimal::try_from(*price)?;
        let fee = OPTION_COMMISSION_PER_CONTRACT * leg_qty.to_string().parse::<f64>().unwrap_or(0.0);
        total_fees_usd += fee;
        let order: PaperOrder = sqlx::query_as(
            "INSERT INTO paper_orders
                (paper_account_id, symbol, side, qty, order_type,
                 status, filled_price, filled_qty, filled_at, fee, parent_order_id)
             VALUES ($1, $2, $3::side_t, $4, 'market',
                     'filled', $5, $4, now(), $6, $7)
             RETURNING id, paper_account_id, symbol, side::text, qty, order_type::text,
                       limit_price, stop_price, status::text,
                       trail_value, trail_is_pct, trail_extreme, oco_group, parent_order_id,
                       filled_price, filled_qty, fee, submitted_at, filled_at, cancel_at, reject_reason, plan_note",
        )
        .bind(account_id)
        .bind(leg.symbol.trim().to_uppercase())
        .bind(side_str)
        .bind(leg_qty)
        .bind(price_dec)
        .bind(Decimal::try_from(fee)?)
        .bind(first_id) // legs 2..n point at leg 1 for display grouping
        .fetch_one(&mut *tx)
        .await?;
        apply_fill(
            &mut tx,
            account_id,
            &leg.symbol.trim().to_uppercase(),
            side,
            leg_qty,
            price_dec,
            Decimal::from(100),
        )
        .await?;
        if first_id.is_none() {
            first_id = Some(order.id);
        }
        orders.push(order);
    }
    deduct_fee(&mut tx, account_id, total_fees_usd).await?;
    tx.commit().await?;
    Ok(SpreadResult {
        orders,
        net_premium_usd,
        total_fees_usd,
    })
}

#[derive(Debug, Clone, Serialize)]
pub struct SpreadLegQuote {
    pub symbol: String,
    pub mid: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SpreadPreview {
    pub legs: Vec<SpreadLegQuote>,
    pub net_premium_usd: f64,
    pub payoff: traderview_core::spread_payoff::PayoffReport,
}

/// Quote a spread WITHOUT touching the book: per-leg chain mids, net
/// premium, and the expiry payoff profile (max profit/loss,
/// breakevens) over spot ± 30%. Same validation and quote source as
/// submit_spread, so the preview prices what the submit would fill.
pub async fn preview_spread(pool: &PgPool, req: &SpreadRequest) -> anyhow::Result<SpreadPreview> {
    traderview_core::option_spread::validate(&req.legs).map_err(|e| anyhow::anyhow!(e))?;
    if req.qty <= Decimal::ZERO || req.qty > Decimal::from(1000) {
        anyhow::bail!("qty must be in 1..=1000 spreads");
    }
    let qty_f = req.qty.to_string().parse::<f64>().unwrap_or(0.0);
    let mut leg_quotes = Vec::with_capacity(req.legs.len());
    let mut payoff_legs = Vec::with_capacity(req.legs.len());
    let mut prices = Vec::with_capacity(req.legs.len());
    let mut spot = 0.0_f64;
    for leg in &req.legs {
        let occ = traderview_core::occ_symbol::parse(&leg.symbol)
            .ok_or_else(|| anyhow::anyhow!("{} is not an OCC symbol", leg.symbol))?;
        if occ.expiry < Utc::now().date_naive() {
            anyhow::bail!("contract expired {}", occ.expiry);
        }
        let chain = crate::options::chain(&occ.underlying, Some(occ.expiry)).await?;
        spot = chain.spot;
        let list = if occ.call { &chain.calls } else { &chain.puts };
        let mid = list
            .iter()
            .find(|c| (c.strike - occ.strike).abs() < 1e-6)
            .and_then(|c| traderview_core::occ_symbol::fill_price(c.bid, c.ask, c.last_price))
            .ok_or_else(|| anyhow::anyhow!("no usable quote for {}", leg.symbol))?;
        prices.push(mid);
        leg_quotes.push(SpreadLegQuote {
            symbol: leg.symbol.clone(),
            mid,
        });
        payoff_legs.push(traderview_core::spread_payoff::Leg {
            kind: if occ.call {
                traderview_core::spread_payoff::OptionKind::Call
            } else {
                traderview_core::spread_payoff::OptionKind::Put
            },
            strike: occ.strike,
            contracts: (leg.ratio as i64) * if leg.buy { 1 } else { -1 } * qty_f as i64,
            premium_per_share: mid,
        });
    }
    let per_share = traderview_core::option_spread::net_premium(&req.legs, &prices)
        .ok_or_else(|| anyhow::anyhow!("degenerate leg prices"))?;
    let payoff = traderview_core::spread_payoff::payoff(
        &payoff_legs,
        spot * 0.7,
        spot * 1.3,
        120,
        100.0,
    );
    Ok(SpreadPreview {
        legs: leg_quotes,
        net_premium_usd: per_share * 100.0 * qty_f,
        payoff,
    })
}

/// Risk-free rate assumption for paper option greeks. Constant by
/// design — paper greeks are for position awareness, not pricing desks.
const GREEKS_RISK_FREE: f64 = 0.045;

#[derive(Debug, Clone, Serialize)]
pub struct PositionGreeks {
    pub symbol: String,
    pub qty: Decimal,
    pub spot: f64,
    pub iv: Option<f64>,
    /// Position-scaled (× qty × 100); None when the chain has no IV.
    pub delta: Option<f64>,
    pub gamma: Option<f64>,
    pub theta_per_day: Option<f64>,
    pub vega: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AccountOptionGreeks {
    pub positions: Vec<PositionGreeks>,
    /// Sums over positions WITH greeks; positions missing IV are
    /// listed but excluded — and that exclusion is visible per row.
    pub net_delta: f64,
    pub net_gamma: f64,
    pub net_theta_per_day: f64,
    pub net_vega: f64,
}

/// BS greeks for every OCC position on the account, scaled by qty and
/// the 100× multiplier (shorts scale negative naturally). IV and spot
/// come from the chain; a contract the chain doesn't quote IV for is
/// listed with None greeks rather than silently dropped.
pub async fn option_greeks(
    pool: &PgPool,
    user_id: Uuid,
    account_id: Uuid,
) -> anyhow::Result<AccountOptionGreeks> {
    let owner: Option<(Uuid,)> = sqlx::query_as("SELECT user_id FROM paper_accounts WHERE id = $1")
        .bind(account_id)
        .fetch_optional(pool)
        .await?;
    if !matches!(owner, Some((u,)) if u == user_id) {
        anyhow::bail!("forbidden");
    }
    let rows: Vec<(String, Decimal)> = sqlx::query_as(
        "SELECT symbol, qty FROM paper_positions WHERE paper_account_id = $1 ORDER BY symbol",
    )
    .bind(account_id)
    .fetch_all(pool)
    .await?;
    let today = Utc::now().date_naive();
    let mut positions = Vec::new();
    let (mut nd, mut ng, mut nt, mut nv) = (0.0, 0.0, 0.0, 0.0);
    for (symbol, qty) in rows {
        let Some(occ) = traderview_core::occ_symbol::parse(&symbol) else {
            continue; // equities have no option greeks
        };
        let chain = crate::options::chain(&occ.underlying, Some(occ.expiry)).await?;
        let list = if occ.call { &chain.calls } else { &chain.puts };
        let contract = list.iter().find(|c| (c.strike - occ.strike).abs() < 1e-6);
        let iv = contract.and_then(|c| c.implied_vol).filter(|v| *v > 0.0);
        let qty_f: f64 = qty.to_string().parse().unwrap_or(0.0);
        let scale = qty_f * 100.0;
        let g = iv.map(|sigma| {
            let kind = if occ.call {
                traderview_core::greeks::OptKind::Call
            } else {
                traderview_core::greeks::OptKind::Put
            };
            traderview_core::greeks::price_and_greeks(
                kind,
                chain.spot,
                occ.strike,
                traderview_core::occ_symbol::years_to_expiry(occ.expiry, today),
                sigma,
                GREEKS_RISK_FREE,
                0.0,
            )
        });
        if let Some(g) = &g {
            nd += g.delta * scale;
            ng += g.gamma * scale;
            nt += g.theta * scale;
            nv += g.vega * scale;
        }
        positions.push(PositionGreeks {
            symbol,
            qty,
            spot: chain.spot,
            iv,
            delta: g.as_ref().map(|g| g.delta * scale),
            gamma: g.as_ref().map(|g| g.gamma * scale),
            theta_per_day: g.as_ref().map(|g| g.theta * scale),
            vega: g.as_ref().map(|g| g.vega * scale),
        });
    }
    Ok(AccountOptionGreeks {
        positions,
        net_delta: nd,
        net_gamma: ng,
        net_theta_per_day: nt,
        net_vega: nv,
    })
}

/// Settle expired option positions at intrinsic value — cash
/// settlement against the UNDERLYING's spot, the day after expiry so
/// expiry-day trading is never interrupted. ITM pays intrinsic × 100;
/// OTM expires worthless (a close at price 0: realized = −avg × qty ×
/// 100, exactly the premium burned). Long positions sell, shorts
/// cover, both through the normal apply_fill path, with a synthetic
/// filled order row left as the audit trail. Idempotent: a settled
/// position is gone, so the next pass finds nothing. Returns
/// positions settled.
pub async fn settle_expired_options(pool: &PgPool) -> anyhow::Result<usize> {
    let positions: Vec<(Uuid, String, Decimal)> = sqlx::query_as(
        "SELECT paper_account_id, symbol, qty FROM paper_positions",
    )
    .fetch_all(pool)
    .await?;
    let today = Utc::now().date_naive();
    let mut settled = 0usize;
    for (account_id, symbol, qty) in positions {
        let Some(occ) = traderview_core::occ_symbol::parse(&symbol) else {
            continue;
        };
        if occ.expiry >= today {
            continue; // settle strictly AFTER expiry day
        }
        // Spot from the underlying; a failed quote skips this pass
        // (transient) — the position settles on a later pass.
        let Ok(quote) = crate::market_data::quote(pool, &occ.underlying).await else {
            continue;
        };
        let intrinsic =
            traderview_core::occ_symbol::intrinsic(occ.call, occ.strike, quote.price);
        let Ok(price) = Decimal::try_from(intrinsic) else { continue };
        // Closing side: longs sell, shorts cover.
        let (side, side_str) = if qty > Decimal::ZERO {
            (Side::Sell, "sell")
        } else {
            (Side::Cover, "cover")
        };
        let close_qty = qty.abs();
        let mut tx = pool.begin().await?;
        sqlx::query(
            "INSERT INTO paper_orders
                (paper_account_id, symbol, side, qty, order_type,
                 status, filled_price, filled_qty, filled_at, reject_reason)
             VALUES ($1, $2, $3::side_t, $4, 'market',
                     'filled', $5, $4, now(), 'expiry settlement at intrinsic')",
        )
        .bind(account_id)
        .bind(&symbol)
        .bind(side_str)
        .bind(close_qty)
        .bind(price)
        .execute(&mut *tx)
        .await?;
        apply_fill(
            &mut tx,
            account_id,
            &symbol,
            side,
            close_qty,
            price,
            Decimal::from(100),
        )
        .await?;
        tx.commit().await?;
        settled += 1;
        tracing::info!(
            account = %account_id, symbol, intrinsic,
            "expired option settled"
        );
    }
    Ok(settled)
}

struct ResolvedQuote {
    last: Decimal,
    multiplier: Decimal,
    is_option: bool,
}

/// Quote + contract economics for any tradeable symbol: equities from
/// the cached quote at 1x; OCC options from the chain mid at 100x.
/// Expired contracts and quoteless options are hard errors here (the
/// SUBMIT path); the ticker treats those cases as cancel/keep-resting.
async fn resolve_quote(pool: &PgPool, symbol: &str) -> anyhow::Result<ResolvedQuote> {
    if let Some(occ) = traderview_core::occ_symbol::parse(symbol) {
        if occ.expiry < Utc::now().date_naive() {
            anyhow::bail!("contract expired {}", occ.expiry);
        }
        let p = option_quote(&occ)
            .await?
            .ok_or_else(|| anyhow::anyhow!("no usable quote for {symbol} (zero bid/ask and no last)"))?;
        Ok(ResolvedQuote {
            last: Decimal::try_from(p)?,
            multiplier: Decimal::from(100),
            is_option: true,
        })
    } else {
        let q = crate::market_data::quote(pool, symbol).await?;
        Ok(ResolvedQuote {
            last: Decimal::try_from(q.price)?,
            multiplier: Decimal::ONE,
            is_option: false,
        })
    }
}

async fn apply_fill(
    tx: &mut sqlx::PgConnection,
    account_id: Uuid,
    symbol: &str,
    side: Side,
    qty: Decimal,
    price: Decimal,
    multiplier: Decimal,
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
                let realized = (price - cur_avg) * close_qty * direction * multiplier;
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
    let cash_delta = -signed_qty * price * multiplier; // buy decreases cash, sell increases
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
                trail_value, trail_is_pct, trail_extreme, oco_group, parent_order_id,
                filled_price, filled_qty, fee, submitted_at, filled_at, cancel_at, reject_reason, plan_note
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
    fn stop_limit_protects_against_gap_through() {
        // Sell stop 95 / limit 93. At 96: stop not crossed, nothing.
        assert_eq!(stop_limit_action(Side::Sell, d(96), d(95), d(93), false), (false, None));
        // At 94: stop crossed AND 94 >= limit 93 — fills at 94.
        assert_eq!(stop_limit_action(Side::Sell, d(94), d(95), d(93), false), (true, Some(d(94))));
        // Gap to 80: triggered but 80 < 93 — does NOT fill at the
        // gapped price. This refusal is the entire point of the type.
        assert_eq!(stop_limit_action(Side::Sell, d(80), d(95), d(93), false), (true, None));
        // Recovery to 96 with triggered state persisted: now a plain
        // limit, 96 >= 93 fills — even though 96 is back above the
        // stop. Re-checking the stop here would be the bug.
        assert_eq!(stop_limit_action(Side::Sell, d(96), d(95), d(93), true), (true, Some(d(96))));
    }

    #[test]
    fn stop_limit_buy_mirror() {
        // Buy stop 105 / limit 107 (breakout entry with a price cap).
        assert_eq!(stop_limit_action(Side::Buy, d(104), d(105), d(107), false), (false, None));
        assert_eq!(stop_limit_action(Side::Buy, d(106), d(105), d(107), false), (true, Some(d(106))));
        // Gap to 110: triggered, capped — no fill above 107.
        assert_eq!(stop_limit_action(Side::Buy, d(110), d(105), d(107), false), (true, None));
        // Pullback to 106.5 after trigger: fills.
        assert_eq!(stop_limit_action(Side::Buy, d(106) + d(1) / d(2), d(105), d(107), true), (true, Some(d(106) + d(1) / d(2))));
        // Short/Cover route through the same sell/buy arms.
        assert_eq!(stop_limit_action(Side::Short, d(94), d(95), d(93), false), (true, Some(d(94))));
        assert_eq!(stop_limit_action(Side::Cover, d(106), d(105), d(107), false), (true, Some(d(106))));
    }

    #[test]
    fn protection_side_derivation_and_qty_bounds() {
        // Long 100: exits are sells, bracket semantics are Buy-side.
        assert_eq!(validate_protection(d(100), d(100)), Ok((Side::Buy, "sell")));
        assert_eq!(validate_protection(d(100), d(50)), Ok((Side::Buy, "sell")));
        // Short -100: exits are covers, Short-side semantics.
        assert_eq!(validate_protection(d(-100), d(100)), Ok((Side::Short, "cover")));
        // Bounds: more than held, nothing held, non-positive qty.
        assert_eq!(validate_protection(d(100), d(101)), Err("qty exceeds position size"));
        assert_eq!(validate_protection(d(0), d(1)), Err("no position to protect"));
        assert_eq!(validate_protection(d(100), d(0)), Err("qty must be positive"));
    }

    #[test]
    fn well_formed_pins_every_resting_type() {
        // Each type demands exactly its own prices.
        assert!(order_well_formed("limit", Some(d(100)), None, None, None));
        assert!(!order_well_formed("limit", None, Some(d(95)), None, None));
        assert!(order_well_formed("stop", None, Some(d(95)), None, None));
        assert!(!order_well_formed("stop", Some(d(100)), None, None, None));
        assert!(order_well_formed("stop_limit", Some(d(93)), Some(d(95)), None, None));
        assert!(!order_well_formed("stop_limit", Some(d(93)), None, None, None));
        assert!(!order_well_formed("stop_limit", None, Some(d(95)), None, None));
        // Trailing: dollars > 0, or pct strictly inside (0, 1).
        assert!(order_well_formed("trailing", None, None, Some(d(2)), Some(false)));
        assert!(order_well_formed("trailing", None, None, Some(d(1) / d(20)), Some(true)));
        assert!(!order_well_formed("trailing", None, None, Some(d(2)), Some(true)));
        assert!(!order_well_formed("trailing", None, None, Some(d(0)), None));
        // Market never rests; unknown types never pass.
        assert!(!order_well_formed("market", Some(d(1)), Some(d(1)), Some(d(1)), None));
        assert!(!order_well_formed("iceberg", Some(d(1)), Some(d(1)), None, None));
    }

    #[test]
    fn missing_required_price_or_unknown_type_never_triggers() {
        assert_eq!(trigger_price("limit", Side::Buy, d(99), None, None), None);
        assert_eq!(trigger_price("stop", Side::Sell, d(94), None, None), None);
        // Trailing has its own ratchet path — never via trigger_price.
        assert_eq!(trigger_price("trailing", Side::Buy, d(99), Some(d(100)), Some(d(95))), None);
    }

    #[test]
    fn sell_trail_ratchets_up_and_fires_on_dollar_retrace() {
        // $2 trail protecting a long. Walk: 100 → 103 (ratchet) → 101.5
        // (1.5 retrace, holds) → 101 (2.0 retrace, fires).
        let tv = d(2);
        let (e, fired) = trail_update(Side::Sell, d(103), d(100), tv, false);
        assert_eq!((e, fired), (d(103), false));
        let (e, fired) = trail_update(Side::Sell, Decimal::new(1015, 1), e, tv, false);
        assert_eq!((e, fired), (d(103), false));
        let (e, fired) = trail_update(Side::Sell, d(101), e, tv, false);
        assert_eq!((e, fired), (d(103), true));
    }

    #[test]
    fn cover_trail_ratchets_down_and_fires_on_percent_bounce() {
        // 5% trail covering a short. Walk: 80 → 76 (ratchet) → 79
        // (3.95% bounce, holds) → 79.8 (= 76 × 1.05, fires).
        let tv = Decimal::new(5, 2);
        let (e, fired) = trail_update(Side::Cover, d(76), d(80), tv, true);
        assert_eq!((e, fired), (d(76), false));
        let (e, fired) = trail_update(Side::Cover, d(79), e, tv, true);
        assert_eq!((e, fired), (d(76), false));
        let (e, fired) = trail_update(Side::Cover, Decimal::new(798, 1), e, tv, true);
        assert_eq!((e, fired), (d(76), true));
    }

    #[test]
    fn buy_bracket_needs_stop_below_entry_below_target() {
        assert!(validate_bracket(Side::Buy, Some(d(100)), d(95), d(110)).is_ok());
        // Inverted exits.
        assert!(validate_bracket(Side::Buy, None, d(110), d(95)).is_err());
        // Entry outside the bracket.
        assert!(validate_bracket(Side::Buy, Some(d(94)), d(95), d(110)).is_err());
        assert!(validate_bracket(Side::Buy, Some(d(111)), d(95), d(110)).is_err());
    }

    #[test]
    fn short_bracket_is_the_mirror() {
        // Short at 100: profit below (90), stop above (105).
        assert!(validate_bracket(Side::Short, Some(d(100)), d(105), d(90)).is_ok());
        assert!(validate_bracket(Side::Short, None, d(90), d(105)).is_err());
    }

    #[test]
    fn exit_sides_cannot_open_a_bracket() {
        assert!(validate_bracket(Side::Sell, None, d(95), d(110)).is_err());
        assert!(validate_bracket(Side::Cover, None, d(105), d(90)).is_err());
    }

    #[test]
    fn degenerate_equal_stop_and_target_rejected() {
        assert!(validate_bracket(Side::Buy, None, d(100), d(100)).is_err());
    }

    #[test]
    fn trail_extreme_never_loosens() {
        // A sell trail's extreme only moves up — a dip must not lower it.
        let (e, _) = trail_update(Side::Sell, d(98), d(103), d(10), false);
        assert_eq!(e, d(103));
        // A cover trail's extreme only moves down.
        let (e, _) = trail_update(Side::Cover, d(85), d(76), d(20), false);
        assert_eq!(e, d(76));
    }
}
