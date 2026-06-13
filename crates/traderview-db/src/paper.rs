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
    pub cash_apy_pct: Decimal,
    pub borrow_apy_pct: Decimal,
    pub margin_multiplier: Decimal,
    pub margin_apy_pct: Decimal,
    pub auto_liquidate: bool,
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
        "SELECT id, user_id, name, starting_cash, cash, drip, cash_apy_pct, borrow_apy_pct, margin_multiplier, margin_apy_pct, auto_liquidate, created_at, reset_at
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
         RETURNING id, user_id, name, starting_cash, cash, drip, cash_apy_pct, borrow_apy_pct, margin_multiplier, margin_apy_pct, auto_liquidate, created_at, reset_at",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?)
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct CashFlow {
    pub amount: Decimal,
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Deposit (positive) or withdraw (negative). Withdrawals are capped
/// by CASH, not equity — you can't wire out money that's currently
/// stock. The flow is recorded for the statement's flow-aware return.
pub async fn cash_flow(
    pool: &PgPool,
    user_id: Uuid,
    account_id: Uuid,
    amount: Decimal,
    note: Option<&str>,
) -> anyhow::Result<CashFlow> {
    if amount == Decimal::ZERO {
        anyhow::bail!("amount must be nonzero");
    }
    if amount.abs() > Decimal::from(100_000_000) {
        anyhow::bail!("amount exceeds sanity bound");
    }
    let mut tx = pool.begin().await?;
    // Claim guards ownership AND the no-overdraw rule in one UPDATE.
    let updated: Option<(Decimal,)> = sqlx::query_as(
        "UPDATE paper_accounts SET cash = cash + $3
          WHERE id = $1 AND user_id = $2 AND cash + $3 >= 0
        RETURNING cash",
    )
    .bind(account_id)
    .bind(user_id)
    .bind(amount)
    .fetch_optional(&mut *tx)
    .await?;
    if updated.is_none() {
        anyhow::bail!("account not found or withdrawal exceeds cash");
    }
    let row: CashFlow = sqlx::query_as(
        "INSERT INTO paper_cash_flows (paper_account_id, amount, note)
         VALUES ($1, $2, $3)
         RETURNING amount, note, created_at",
    )
    .bind(account_id)
    .bind(amount)
    .bind(note.map(str::trim).filter(|s| !s.is_empty()))
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(row)
}

/// Merge an incoming transferred lot into a destination position,
/// pure. Empty dest takes the incoming basis; same-sign merges
/// weight the average; OPPOSITE signs refuse — netting a long
/// against a short realizes PnL, and realization belongs to trades,
/// not transfers.
pub fn merge_position(
    dest_qty: Decimal,
    dest_avg: Decimal,
    in_qty: Decimal,
    in_avg: Decimal,
) -> Result<(Decimal, Decimal), &'static str> {
    if in_qty == Decimal::ZERO {
        return Err("nothing to merge");
    }
    if dest_qty == Decimal::ZERO {
        return Ok((in_qty, in_avg));
    }
    if (dest_qty > Decimal::ZERO) != (in_qty > Decimal::ZERO) {
        return Err("opposite-sign merge would net positions without realization");
    }
    let qty = dest_qty + in_qty;
    Ok(((qty), (dest_qty * dest_avg + in_qty * in_avg) / qty))
}

/// Move cash between two of the user's accounts atomically — the
/// capital-reallocation op the strategy-per-account layout needs.
/// One transaction: the source debit is overdraw-guarded by CASH
/// (money that is currently stock cannot move), both legs land in
/// paper_cash_flows with cross-referencing notes, and each side's
/// flow-aware returns see it correctly (an outflow here, an inflow
/// there — TWR and Dietz already treat flows as capital, not
/// performance).
pub async fn transfer(
    pool: &PgPool,
    user_id: Uuid,
    from: Uuid,
    to: Uuid,
    amount: Decimal,
) -> anyhow::Result<()> {
    if amount <= Decimal::ZERO {
        anyhow::bail!("amount must be positive");
    }
    if from == to {
        anyhow::bail!("from and to are the same account");
    }
    let mut tx = pool.begin().await?;
    // Names for the cross-referencing notes + ownership of BOTH ends.
    let names: Vec<(Uuid, String)> = sqlx::query_as(
        "SELECT id, name FROM paper_accounts WHERE user_id = $1 AND id IN ($2, $3)",
    )
    .bind(user_id)
    .bind(from)
    .bind(to)
    .fetch_all(&mut *tx)
    .await?;
    let name_of = |id: Uuid| names.iter().find(|(i, _)| *i == id).map(|(_, n)| n.clone());
    let (Some(from_name), Some(to_name)) = (name_of(from), name_of(to)) else {
        anyhow::bail!("account not found");
    };
    let debited = sqlx::query(
        "UPDATE paper_accounts SET cash = cash - $2
          WHERE id = $1 AND cash - $2 >= 0",
    )
    .bind(from)
    .bind(amount)
    .execute(&mut *tx)
    .await?
    .rows_affected();
    if debited == 0 {
        anyhow::bail!("transfer exceeds source cash");
    }
    sqlx::query("UPDATE paper_accounts SET cash = cash + $2 WHERE id = $1")
        .bind(to)
        .bind(amount)
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        "INSERT INTO paper_cash_flows (paper_account_id, amount, note)
         VALUES ($1, $2, $3), ($4, $5, $6)",
    )
    .bind(from)
    .bind(-amount)
    .bind(format!("transfer to {to_name}"))
    .bind(to)
    .bind(amount)
    .bind(format!("transfer from {from_name}"))
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

/// In-kind transfer: move a position (or part) between two of the
/// user's accounts at COST BASIS — no cash moves, no PnL realizes.
/// Both sides get synthetic order rows at the basis so every FIFO
/// reconstruction stays correct: the source's trip closes at exactly
/// its basis (PnL 0 — moving a position is not a gain) and the
/// destination opens at the same basis. The destination merge
/// refuses opposite signs (netting realizes, and realization belongs
/// to trades); the destination's buying power is checked post-state
/// like any fill — an account that can't carry the exposure can't
/// receive it.
pub async fn transfer_position(
    pool: &PgPool,
    user_id: Uuid,
    from: Uuid,
    to: Uuid,
    symbol: &str,
    qty: Decimal,
) -> anyhow::Result<()> {
    if qty <= Decimal::ZERO {
        anyhow::bail!("qty must be positive");
    }
    if from == to {
        anyhow::bail!("from and to are the same account");
    }
    let symbol = symbol.trim().to_uppercase();
    let mut tx = pool.begin().await?;
    let names: Vec<(Uuid, String)> = sqlx::query_as(
        "SELECT id, name FROM paper_accounts WHERE user_id = $1 AND id IN ($2, $3)",
    )
    .bind(user_id)
    .bind(from)
    .bind(to)
    .fetch_all(&mut *tx)
    .await?;
    let name_of = |id: Uuid| names.iter().find(|(i, _)| *i == id).map(|(_, n)| n.clone());
    let (Some(from_name), Some(to_name)) = (name_of(from), name_of(to)) else {
        anyhow::bail!("account not found");
    };
    let src_pos: Option<(Decimal, Decimal)> = sqlx::query_as(
        "SELECT qty, avg_price FROM paper_positions
          WHERE paper_account_id = $1 AND symbol = $2 FOR UPDATE",
    )
    .bind(from)
    .bind(&symbol)
    .fetch_optional(&mut *tx)
    .await?;
    let Some((src_qty, src_avg)) = src_pos else {
        anyhow::bail!("no position in {symbol} to transfer");
    };
    if qty > src_qty.abs() {
        anyhow::bail!("qty exceeds the held position");
    }
    let signed_out = if src_qty > Decimal::ZERO { qty } else { -qty };

    // Destination merge (refuses opposite signs).
    let dest_pos: Option<(Decimal, Decimal)> = sqlx::query_as(
        "SELECT qty, avg_price FROM paper_positions
          WHERE paper_account_id = $1 AND symbol = $2 FOR UPDATE",
    )
    .bind(to)
    .bind(&symbol)
    .fetch_optional(&mut *tx)
    .await?;
    let (dq, da) = dest_pos.unwrap_or((Decimal::ZERO, Decimal::ZERO));
    let (new_dq, new_da) =
        merge_position(dq, da, signed_out, src_avg).map_err(|e| anyhow::anyhow!(e))?;

    // Source decrement / delete.
    let remaining = src_qty - signed_out;
    if remaining.is_zero() {
        sqlx::query("DELETE FROM paper_positions WHERE paper_account_id = $1 AND symbol = $2")
            .bind(from)
            .bind(&symbol)
            .execute(&mut *tx)
            .await?;
    } else {
        sqlx::query(
            "UPDATE paper_positions SET qty = $3, updated_at = now()
              WHERE paper_account_id = $1 AND symbol = $2",
        )
        .bind(from)
        .bind(&symbol)
        .bind(remaining)
        .execute(&mut *tx)
        .await?;
    }
    sqlx::query(
        "INSERT INTO paper_positions (paper_account_id, symbol, qty, avg_price, realized_pnl, updated_at)
              VALUES ($1, $2, $3, $4, 0, now())
         ON CONFLICT (paper_account_id, symbol) DO UPDATE SET
            qty = EXCLUDED.qty, avg_price = EXCLUDED.avg_price, updated_at = now()",
    )
    .bind(to)
    .bind(&symbol)
    .bind(new_dq)
    .bind(new_da)
    .execute(&mut *tx)
    .await?;

    // Synthetic order rows at BASIS so FIFO reconstructions stay
    // correct on both sides (no cash moved — these are bookkeeping
    // fills, like the expiry-settlement rows).
    let (out_side, in_side) = if src_qty > Decimal::ZERO {
        ("sell", "buy")
    } else {
        ("cover", "short")
    };
    sqlx::query(
        "INSERT INTO paper_orders
            (paper_account_id, symbol, side, qty, order_type,
             status, filled_price, filled_qty, filled_at, plan_note)
         VALUES ($1, $2, $3::side_t, $4, 'market', 'filled', $5, $4, now(), $6),
                ($7, $2, $8::side_t, $4, 'market', 'filled', $5, $4, now(), $9)",
    )
    .bind(from)
    .bind(&symbol)
    .bind(out_side)
    .bind(qty)
    .bind(src_avg)
    .bind(format!("in-kind transfer out to {to_name}"))
    .bind(to)
    .bind(in_side)
    .bind(format!("in-kind transfer in from {from_name}"))
    .execute(&mut *tx)
    .await?;

    // Destination buying power, post-state — an account that can't
    // carry the exposure can't receive it.
    let (cash, m): (Decimal, Decimal) = sqlx::query_as(
        "SELECT cash, margin_multiplier FROM paper_accounts WHERE id = $1",
    )
    .bind(to)
    .fetch_one(&mut *tx)
    .await?;
    let books: Vec<(String, Decimal, Decimal)> = sqlx::query_as(
        "SELECT symbol, qty, avg_price FROM paper_positions WHERE paper_account_id = $1",
    )
    .bind(to)
    .fetch_all(&mut *tx)
    .await?;
    let (mut signed_book, mut gross_book) = (Decimal::ZERO, Decimal::ZERO);
    for (sym, q, avg) in &books {
        let mult = if traderview_core::occ_symbol::is_occ(sym) {
            Decimal::from(100)
        } else {
            Decimal::ONE
        };
        signed_book += *q * *avg * mult;
        gross_book += (*q * *avg * mult).abs();
    }
    if !buying_power_ok(cash, signed_book, gross_book, m) {
        anyhow::bail!("destination lacks buying power for the transferred exposure");
    }
    tx.commit().await?;
    Ok(())
}

pub async fn list_cash_flows(
    pool: &PgPool,
    user_id: Uuid,
    account_id: Uuid,
    limit: i64,
) -> anyhow::Result<Vec<CashFlow>> {
    Ok(sqlx::query_as(
        "SELECT f.amount, f.note, f.created_at
           FROM paper_cash_flows f
           JOIN paper_accounts a ON a.id = f.paper_account_id
          WHERE f.paper_account_id = $1 AND a.user_id = $2
          ORDER BY f.created_at DESC LIMIT $3",
    )
    .bind(account_id)
    .bind(user_id)
    .bind(limit)
    .fetch_all(pool)
    .await?)
}

/// Buying-power constraint on ENTRY-BASIS books (deterministic — no
/// quote dependency, the borrow-fee convention): equity = cash +
/// signed book value; the account is inside its power when equity is
/// positive and gross exposure ≤ multiplier × equity. m = 1 is a
/// cash account (longs bounded by cash, shorts at 2|S| ≤ cash —
/// conservative, in the spirit of Reg-T's 150% short requirement);
/// m = 2 is Reg-T initial margin. Entry basis, not marked value: a
/// stated approximation, the same one the borrow-fee pass makes.
pub fn buying_power_ok(
    cash: Decimal,
    signed_book: Decimal,
    gross_book: Decimal,
    multiplier: Decimal,
) -> bool {
    let m = multiplier.max(Decimal::ONE);
    let equity = cash + signed_book;
    if gross_book.is_zero() {
        // No positions: only a non-negative cash balance is required
        // (withdrawals already guard overdraw; fees can graze zero).
        return cash >= Decimal::ZERO || equity > Decimal::ZERO;
    }
    equity > Decimal::ZERO && gross_book <= m * equity
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
         RETURNING id, user_id, name, starting_cash, cash, drip, cash_apy_pct, borrow_apy_pct, margin_multiplier, margin_apy_pct, auto_liquidate, created_at, reset_at",
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
    /// Time in force: 'gtc' (default), 'day' (expires at the next 16:00
    /// US Eastern close), 'gtd' (expires at expire_at), or 'ioc'/'fok'
    /// (immediate-or-cancel / fill-or-kill — fill at submit if
    /// marketable, otherwise cancel instead of resting). gtc/day/gtd are
    /// ignored on orders that fill immediately.
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
        ("limit", Some(_), _)
            | ("stop", _, Some(_))
            | ("stop_limit", Some(_), Some(_))
            | ("moc", _, _)
            | ("loc", Some(_), _)
    ) || (order_type == "trailing" && trail_ok)
        // trailing_stop_limit: a valid trail, plus an optional limit
        // offset in stop_price (absent = 0 = fill at the trail level);
        // a negative offset is malformed.
        || (order_type == "trailing_stop_limit"
            && trail_ok
            && stop_price.is_none_or(|o| o >= Decimal::ZERO))
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OnCloseAction {
    /// Before the stamped close — keep resting.
    Wait,
    Fill(Decimal),
    /// LOC whose limit wasn't met at the close — an on-close order
    /// does not survive into the next session.
    Cancel,
}

/// MOC/LOC evaluation at a tick. The close itself is stamped into the
/// order at submit (day_order_expiry — holiday/DST aware), so this is
/// pure: before it, Wait; at/after it, MOC fills at last, LOC fills
/// at limit-or-better and cancels otherwise.
pub fn on_close_action(
    order_type: &str,
    side: Side,
    last: Decimal,
    limit_price: Option<Decimal>,
    now: DateTime<Utc>,
    trigger_at: DateTime<Utc>,
) -> OnCloseAction {
    if now < trigger_at {
        return OnCloseAction::Wait;
    }
    match order_type {
        "moc" => OnCloseAction::Fill(last),
        "loc" => match trigger_price("limit", side, last, limit_price, None) {
            Some(p) => OnCloseAction::Fill(p),
            None => OnCloseAction::Cancel,
        },
        _ => OnCloseAction::Wait,
    }
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

/// The frozen limit price when a trailing-stop-limit triggers. The
/// trailing stop sits `trail` below the high-water (sell/short) or above
/// the low-water (buy/cover); on trigger the order converts to a limit
/// at that stop level, moved `limit_offset` further toward the market so
/// a small retrace past the stop still fills while a hard gap rests.
/// Offset 0 fills exactly at the trailing stop level.
pub fn trailing_limit_on_trigger(
    side: Side,
    extreme: Decimal,
    trail_value: Decimal,
    trail_is_pct: bool,
    limit_offset: Decimal,
) -> Decimal {
    let trail = if trail_is_pct {
        extreme * trail_value
    } else {
        trail_value
    };
    if matches!(side, Side::Sell | Side::Short) {
        extreme - trail - limit_offset
    } else {
        extreme + trail + limit_offset
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
/// Status for an order that did NOT fill at submit. IOC/FOK orders that
/// aren't immediately marketable are CANCELLED rather than left to rest
/// — the paper engine fills fully or not at all, so fill-or-kill and
/// immediate-or-cancel collapse to one rule here. A malformed resting
/// order is rejected (malformed beats IOC); otherwise it rests as
/// pending. Returns (status, reject_reason).
fn unfilled_status(
    well_formed: bool,
    immediate_or_cancel: bool,
    tif: &str,
) -> (&'static str, Option<String>) {
    match (well_formed, immediate_or_cancel) {
        (false, _) => (
            "rejected",
            Some("order type needs its limit/stop price or trail value".to_string()),
        ),
        (true, true) => (
            "cancelled",
            Some(format!("{}: not immediately marketable", tif.to_uppercase())),
        ),
        (true, false) => ("pending", None),
    }
}

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
        Some(p) if rq.is_crypto => {
            // Taker fee on notional; the price itself is unadjusted —
            // the fee IS crypto's friction model.
            let notional = (p * req.qty).to_string().parse::<f64>().unwrap_or(0.0);
            (Some(p), notional * CRYPTO_TAKER_FEE_PCT / 100.0)
        }
        Some(p) if rq.is_forex => {
            // Spread cost in pips; price unadjusted — the spread IS
            // forex's friction model.
            (Some(p), forex_spread_fee(&req.symbol, req.qty))
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
    // A resting trailing stop (plain or stop-limit) starts its ratchet
    // at the current price.
    let trail_extreme = (matches!(req.order_type.as_str(), "trailing" | "trailing_stop_limit")
        && well_formed)
        .then_some(last);
    // On-close orders carry their fill time, stamped at submit.
    let trigger_at = matches!(req.order_type.as_str(), "moc" | "loc")
        .then(|| traderview_core::holiday_calendar::day_order_expiry(Utc::now()));
    // Time in force → cancel_at. Only meaningful on orders that rest.
    // IOC/FOK never rest: they resolve at submit (fill if marketable,
    // else cancel), so they carry no cancel_at.
    let tif = req.time_in_force.as_deref().unwrap_or("gtc");
    let immediate_or_cancel = matches!(tif, "ioc" | "fok");
    let cancel_at = match tif {
        "gtc" | "ioc" | "fok" => None,
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
    // MOC/LOC manage their own lifecycle: a DAY cancel_at lands at
    // exactly 16:00 — the expire sweep runs BEFORE the fill pass and
    // would cancel the order at its own trigger instant.
    let cancel_at = if trigger_at.is_some() { None } else { cancel_at };
    let mut tx = pool.begin().await?;
    let (status, filled_at, reject) = match fill_price {
        Some(_) => ("filled", Some(Utc::now()), None),
        None => {
            let (s, r) = unfilled_status(well_formed, immediate_or_cancel, tif);
            (s, None, r)
        }
    };
    let order: PaperOrder = sqlx::query_as(
        "INSERT INTO paper_orders
            (paper_account_id, symbol, side, qty, order_type, limit_price, stop_price,
             status, filled_price, filled_qty, filled_at, reject_reason,
             trail_value, trail_is_pct, trail_extreme, cancel_at, plan_note, stop_triggered, trigger_at)
         VALUES ($1, $2, $3::side_t, $4, $5::paper_order_type_t, $6, $7,
                 $8::paper_order_status_t, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
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
    .bind(trigger_at)
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
        trigger_at: Option<DateTime<Utc>>,
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
    // Promote bracket exit legs whose entry has filled: held →
    // pending. Trailing legs also get their ratchet seeded from the
    // ENTRY's fill price — without it the trailing branch would skip
    // them forever (it requires an extreme to ratchet from).
    sqlx::query(
        "UPDATE paper_orders o
            SET status = 'pending',
                trail_extreme = CASE WHEN o.order_type = 'trailing'
                                     THEN COALESCE(o.trail_extreme, p.filled_price)
                                     ELSE o.trail_extreme END
           FROM paper_orders p
          WHERE o.parent_order_id = p.id
            AND o.status = 'held' AND p.status = 'filled'",
    )
    .execute(pool)
    .await?;
    let rows: Vec<Pending> = sqlx::query_as(
        "SELECT id, paper_account_id, symbol, side::text, qty, order_type::text,
                limit_price, stop_price, trail_value, trail_is_pct, trail_extreme, oco_group,
                stop_triggered, trigger_at
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
        let (last, multiplier, is_option) = if crate::crypto::is_crypto_pair(&o.symbol) {
            let Ok(p) = crate::crypto::spot_quote_cached(&o.symbol).await else {
                continue; // transient venue failure — keep resting
            };
            let Ok(p) = Decimal::try_from(p) else { continue };
            (p, Decimal::ONE, false)
        } else if let Some(occ) = &occ {
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
        } else if o.order_type == "trailing_stop_limit" {
            let (Some(tv), Some(extreme)) = (o.trail_value, o.trail_extreme) else {
                continue;
            };
            if o.stop_triggered {
                // Already converted to a static limit at the frozen
                // level; fill if the market is at/through it, else rest.
                let Some(lp) = o.limit_price else { continue };
                let Some(p) = trigger_price("limit", side, last, Some(lp), None) else {
                    continue;
                };
                p
            } else {
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
                // Trigger: freeze the trailing stop into a limit (the
                // stop level moved stop_price toward the market) and
                // persist the conversion, so a gap past the limit keeps
                // resting instead of slipping to market.
                let offset = o.stop_price.unwrap_or(Decimal::ZERO);
                let frozen = trailing_limit_on_trigger(
                    side,
                    new_extreme,
                    tv,
                    o.trail_is_pct.unwrap_or(false),
                    offset,
                );
                sqlx::query(
                    "UPDATE paper_orders
                        SET stop_triggered = TRUE, limit_price = $2, trail_extreme = $3
                      WHERE id = $1",
                )
                .bind(o.id)
                .bind(frozen)
                .bind(new_extreme)
                .execute(pool)
                .await
                .ok();
                let Some(p) = trigger_price("limit", side, last, Some(frozen), None) else {
                    continue;
                };
                p
            }
        } else if o.order_type == "moc" || o.order_type == "loc" {
            let Some(trigger_at) = o.trigger_at else {
                continue; // malformed row — leave it for inspection
            };
            match on_close_action(&o.order_type, side, last, o.limit_price, Utc::now(), trigger_at)
            {
                OnCloseAction::Wait => continue,
                OnCloseAction::Fill(p) => p,
                OnCloseAction::Cancel => {
                    sqlx::query(
                        "UPDATE paper_orders
                            SET status = 'cancelled', reject_reason = 'limit not met at close'
                          WHERE id = $1 AND status = 'pending'",
                    )
                    .bind(o.id)
                    .execute(pool)
                    .await
                    .ok();
                    continue;
                }
            }
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
        } else if crate::crypto::is_crypto_pair(&o.symbol) {
            let notional = (p * o.qty).to_string().parse::<f64>().unwrap_or(0.0);
            (p, notional * CRYPTO_TAKER_FEE_PCT / 100.0)
        } else if crate::forex::is_forex_pair(&o.symbol) {
            (p, forex_spread_fee(&o.symbol, o.qty))
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
        if let Err(e) = apply_fill(&mut tx, o.paper_account_id, &o.symbol, side, o.qty, adjusted, multiplier).await {
            // Buying power (or any fill-invariant) failure: roll the
            // claim back and CANCEL the order with the reason — a
            // resting order the account can no longer afford must not
            // wedge the ticker retrying forever.
            tx.rollback().await?;
            sqlx::query(
                "UPDATE paper_orders SET status = 'cancelled', reject_reason = $2
                  WHERE id = $1 AND status = 'pending'",
            )
            .bind(o.id)
            .bind(e.to_string())
            .execute(pool)
            .await
            .ok();
            continue;
        }
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
    pub entry_type: String, // 'market' | 'limit' | 'stop' | 'stop_limit'
    pub limit_price: Option<Decimal>,
    /// Entry trigger for stop / stop_limit entries — the breakout
    /// bracket: a buy stop above resistance with the exits attached.
    #[serde(default)]
    pub stop_price: Option<Decimal>,
    /// Fixed stop leg. Exactly one of stop_loss / trail_value.
    #[serde(default)]
    pub stop_loss: Option<Decimal>,
    /// Trailing stop leg instead of a fixed stop: dollars, or a
    /// fraction of the ratcheting extreme when trail_is_pct.
    #[serde(default)]
    pub trail_value: Option<Decimal>,
    #[serde(default)]
    pub trail_is_pct: Option<bool>,
    pub take_profit: Decimal,
}

/// Validation for the TRAILING bracket variant: the target must sit
/// on the profit side of a known entry hint (the fixed-stop ordering
/// check has no fixed stop to order against), and the trail itself
/// must be well-formed. Pure.
pub fn validate_trailing_bracket(
    side: Side,
    entry_hint: Option<Decimal>,
    take_profit: Decimal,
    trail_value: Option<Decimal>,
    trail_is_pct: Option<bool>,
) -> Result<(), &'static str> {
    if !matches!(side, Side::Buy | Side::Short) {
        return Err("bracket entry side must be buy or short");
    }
    if !order_well_formed("trailing", None, None, trail_value, trail_is_pct) {
        return Err("trail must be dollars > 0, or a fraction in (0, 1) when trail_is_pct");
    }
    if let Some(e) = entry_hint {
        let ok = match side {
            Side::Buy => take_profit > e,
            _ => take_profit < e,
        };
        if !ok {
            return Err("take_profit must be on the profit side of the entry");
        }
    }
    Ok(())
}

/// Entry hint for bracket price-relationship validation, pure. Market
/// has no knowable entry price (the fill quote is the hint's job at
/// fill time); limit anchors at its limit; stop and stop_limit anchor
/// at the STOP — the trigger is where the position comes alive, and a
/// stop_limit's limit only bounds slippage past it.
pub fn bracket_entry_hint(
    entry_type: &str,
    limit_price: Option<Decimal>,
    stop_price: Option<Decimal>,
) -> Result<Option<Decimal>, &'static str> {
    match entry_type {
        "market" => Ok(None),
        "limit" => limit_price.map(Some).ok_or("limit entry needs limit_price"),
        "stop" => stop_price.map(Some).ok_or("stop entry needs stop_price"),
        "stop_limit" => match (stop_price, limit_price) {
            (Some(sp), Some(_)) => Ok(Some(sp)),
            _ => Err("stop_limit entry needs stop_price and limit_price"),
        },
        _ => Err("entry_type must be market, limit, stop, or stop_limit"),
    }
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
#[derive(Debug, Clone, Deserialize)]
pub struct ScaleRequest {
    pub symbol: String,
    pub side: Side,
    pub total_qty: Decimal,
    pub price_low: Decimal,
    pub price_high: Decimal,
    pub rungs: usize,
    /// Whole-number share quantities (equities); false splits fractional
    /// size (crypto / FX, where fractional is native).
    #[serde(default)]
    pub whole_units: bool,
    #[serde(default)]
    pub time_in_force: Option<String>,
    #[serde(default)]
    pub expire_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScaleResult {
    /// The limit orders created, ordered low price to high.
    pub orders: Vec<PaperOrder>,
    /// Rungs the risk gate rejected (e.g. buying power ran out on the
    /// later, larger-cumulative rungs); the accepted rungs still rest.
    pub rejected: usize,
}

/// Submit a scale (ladder) order: `rungs` evenly-priced limit orders
/// across `[price_low, price_high]`. Each rung is an independent limit
/// order through the normal submit path, so the Risk Gate and buying
/// power apply per rung and the ladder simply stops filling when capital
/// runs out — no special-case engine code. 2..=50 rungs.
pub async fn submit_scale(
    pool: &PgPool,
    user_id: Uuid,
    account_id: Uuid,
    req: ScaleRequest,
) -> anyhow::Result<ScaleResult> {
    if !(2..=50).contains(&req.rungs) {
        anyhow::bail!("rungs must be between 2 and 50");
    }
    let to_f = |d: Decimal| d.to_string().parse::<f64>().unwrap_or(0.0);
    let rungs = traderview_core::scale_order::ladder(
        to_f(req.total_qty),
        to_f(req.price_low),
        to_f(req.price_high),
        req.rungs,
        req.whole_units,
    )
    .ok_or_else(|| anyhow::anyhow!("invalid scale parameters (check qty, band, and rungs)"))?;

    let mut orders = Vec::with_capacity(rungs.len());
    let mut rejected = 0;
    for r in rungs {
        let order = submit(
            pool,
            user_id,
            account_id,
            OrderRequest {
                symbol: req.symbol.clone(),
                side: req.side,
                qty: Decimal::try_from(r.qty)?,
                order_type: "limit".into(),
                limit_price: Some(Decimal::try_from(r.price)?.round_dp(6)),
                stop_price: None,
                trail_value: None,
                trail_is_pct: None,
                time_in_force: req.time_in_force.clone(),
                expire_at: req.expire_at,
                plan_note: None,
            },
        )
        .await?;
        if order.status == "rejected" {
            rejected += 1;
        }
        orders.push(order);
    }
    Ok(ScaleResult { orders, rejected })
}

pub async fn submit_bracket(
    pool: &PgPool,
    user_id: Uuid,
    account_id: Uuid,
    req: BracketRequest,
) -> anyhow::Result<Bracket> {
    let entry_hint = bracket_entry_hint(&req.entry_type, req.limit_price, req.stop_price)
        .map_err(|e| anyhow::anyhow!(e))?;
    match (req.stop_loss, req.trail_value) {
        (Some(sl), None) => validate_bracket(req.side, entry_hint, sl, req.take_profit)
            .map_err(|e| anyhow::anyhow!(e))?,
        (None, Some(_)) => validate_trailing_bracket(
            req.side, entry_hint, req.take_profit, req.trail_value, req.trail_is_pct,
        )
        .map_err(|e| anyhow::anyhow!(e))?,
        _ => anyhow::bail!("exactly one of stop_loss / trail_value"),
    }

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
            stop_price: req.stop_price,
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
    // Trailing stop legs start their ratchet at the ENTRY fill when
    // it already happened; held legs get the extreme stamped at
    // promotion from the parent's fill price.
    let stop = match (req.stop_loss, req.trail_value) {
        (Some(sl), _) => {
            insert_leg(
                &mut tx, account_id, &symbol, exit_side, req.qty,
                "stop", None, Some(sl), leg_status, group, Some(entry.id), None, None, None,
            )
            .await?
        }
        (None, tv) => {
            insert_leg(
                &mut tx, account_id, &symbol, exit_side, req.qty,
                "trailing", None, None, leg_status, group, Some(entry.id),
                tv, req.trail_is_pct, entry.filled_price,
            )
            .await?
        }
    };
    let target = insert_leg(
        &mut tx, account_id, &symbol, exit_side, req.qty,
        "limit", Some(req.take_profit), None, leg_status, group, Some(entry.id), None, None, None,
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
        "stop", None, Some(stop_loss), "pending", group, None, None, None, None,
    )
    .await?;
    let target = insert_leg(
        &mut tx, account_id, &symbol, exit_side, qty,
        "limit", Some(take_profit), None, "pending", group, None, None, None, None,
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
    trail_value: Option<Decimal>,
    trail_is_pct: Option<bool>,
    trail_extreme: Option<Decimal>,
) -> anyhow::Result<PaperOrder> {
    Ok(sqlx::query_as(
        "INSERT INTO paper_orders
            (paper_account_id, symbol, side, qty, order_type, limit_price, stop_price,
             status, oco_group, parent_order_id, trail_value, trail_is_pct, trail_extreme)
         VALUES ($1, $2, $3::side_t, $4, $5::paper_order_type_t, $6, $7,
                 $8::paper_order_status_t, $9, $10, $11, $12, $13)
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
    .bind(trail_value)
    .bind(trail_is_pct)
    .bind(trail_extreme)
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

/// Roll sanity, pure: both legs must be OCC contracts on the SAME
/// underlying (rolling into a different name is two trades, not a
/// roll), the target must differ from the source (strike, expiry, or
/// both may change — out-rolls, up/down-rolls, and diagonals are all
/// rolls), and qty is bounded by the held position. Returns
/// (close_leg_is_buy, open_leg_is_buy): a LONG position closes by
/// selling and opens the new contract by buying; a short position is
/// the mirror.
pub fn validate_roll(
    from: &str,
    to: &str,
    pos_qty: Decimal,
    qty: Decimal,
) -> Result<(bool, bool), &'static str> {
    let f = traderview_core::occ_symbol::parse(from).ok_or("from is not an OCC contract")?;
    let t = traderview_core::occ_symbol::parse(to).ok_or("to is not an OCC contract")?;
    if f.underlying != t.underlying {
        return Err("roll must stay on the same underlying");
    }
    if from == to {
        return Err("target contract is the same as the source");
    }
    if qty <= Decimal::ZERO {
        return Err("qty must be positive");
    }
    if pos_qty == Decimal::ZERO {
        return Err("no position in the source contract");
    }
    if qty > pos_qty.abs() {
        return Err("qty exceeds position size");
    }
    // Long: sell to close, buy to open. Short: buy to close, sell to open.
    Ok(if pos_qty > Decimal::ZERO { (false, true) } else { (true, false) })
}

/// Roll an option position: close (part of) the held contract and
/// open another on the same underlying ATOMICALLY through the spread
/// path — every leg quotes before the book is touched, so a roll can
/// never close the old contract and then fail to open the new one.
/// Returns the spread result; net_premium_usd > 0 means the roll
/// collected a credit.
pub async fn roll_position(
    pool: &PgPool,
    user_id: Uuid,
    account_id: Uuid,
    from: &str,
    to: &str,
    qty: Decimal,
) -> anyhow::Result<SpreadResult> {
    let owner: Option<(Uuid,)> = sqlx::query_as("SELECT user_id FROM paper_accounts WHERE id = $1")
        .bind(account_id)
        .fetch_optional(pool)
        .await?;
    if !matches!(owner, Some((u,)) if u == user_id) {
        anyhow::bail!("forbidden");
    }
    let from = from.trim().to_uppercase();
    let to = to.trim().to_uppercase();
    let pos: Option<(Decimal,)> = sqlx::query_as(
        "SELECT qty FROM paper_positions WHERE paper_account_id = $1 AND symbol = $2",
    )
    .bind(account_id)
    .bind(&from)
    .fetch_optional(pool)
    .await?;
    let pos_qty = pos.map(|(q,)| q).unwrap_or(Decimal::ZERO);
    let (close_buy, open_buy) =
        validate_roll(&from, &to, pos_qty, qty).map_err(|e| anyhow::anyhow!(e))?;
    submit_spread(
        pool,
        user_id,
        account_id,
        SpreadRequest {
            legs: vec![
                traderview_core::option_spread::SpreadLeg { symbol: from, buy: close_buy, ratio: 1 },
                traderview_core::option_spread::SpreadLeg { symbol: to, buy: open_buy, ratio: 1 },
            ],
            qty,
        },
    )
    .await
}

/// Covered-call sanity, pure: the option must be an OCC CALL (a
/// covered put is a different structure with different risk), and
/// each contract covers exactly 100 shares. Returns (underlying,
/// shares_to_buy).
pub fn validate_covered_call(
    call_symbol: &str,
    contracts: u32,
) -> Result<(String, Decimal), &'static str> {
    let occ = traderview_core::occ_symbol::parse(call_symbol)
        .ok_or("not an OCC option symbol")?;
    if !occ.call {
        return Err("covered call needs a CALL leg");
    }
    if contracts == 0 || contracts > 1000 {
        return Err("contracts must be in 1..=1000");
    }
    Ok((occ.underlying, Decimal::from(contracts as i64 * 100)))
}

#[derive(Debug, Clone, Serialize)]
pub struct CoveredCallResult {
    pub stock_order: PaperOrder,
    pub call_order: PaperOrder,
    /// Stock cost − call premium collected (positive = net cash out).
    pub net_debit_usd: f64,
}

/// Buy-write in ONE transaction: 100×contracts shares + the short
/// call, both quoted BEFORE the book is touched (the spread
/// convention — a covered call can never end up half-built). The call
/// credit posts FIRST so the buying-power check inside each fill sees
/// the premium before the stock debit. Both legs ride the normal fill
/// machinery: friction on the shares, per-contract commission on the
/// call, audit rows, the lot book.
pub async fn covered_call(
    pool: &PgPool,
    user_id: Uuid,
    account_id: Uuid,
    call_symbol: &str,
    contracts: u32,
) -> anyhow::Result<CoveredCallResult> {
    let owner: Option<(Uuid,)> = sqlx::query_as("SELECT user_id FROM paper_accounts WHERE id = $1")
        .bind(account_id)
        .fetch_optional(pool)
        .await?;
    if !matches!(owner, Some((u,)) if u == user_id) {
        anyhow::bail!("forbidden");
    }
    let call_symbol = call_symbol.trim().to_uppercase();
    let (underlying, shares) =
        validate_covered_call(&call_symbol, contracts).map_err(|e| anyhow::anyhow!(e))?;
    // Quote BOTH legs before touching the book.
    let stock_q = resolve_quote(pool, &underlying).await?;
    let call_q = resolve_quote(pool, &call_symbol).await?;
    let contracts_dec = Decimal::from(contracts as i64);
    let (stock_px, stock_fee) = frictioned_fill(stock_q.last, shares, Side::Buy);
    let call_px = call_q.last;
    let call_fee = OPTION_COMMISSION_PER_CONTRACT * contracts as f64;

    let mut tx = pool.begin().await?;
    let group = Uuid::new_v4();
    // Credit leg first: the BP check inside each fill is post-state,
    // and premium-before-stock is the order that reflects the
    // structure's real margin profile.
    let call_order = insert_leg(
        &mut tx, account_id, &call_symbol, "short", contracts_dec,
        "market", None, None, "filled", group, None, None, None, None,
    )
    .await?;
    sqlx::query(
        "UPDATE paper_orders SET filled_price = $2, filled_qty = qty, filled_at = now(),
                plan_note = 'covered call' WHERE id = $1",
    )
    .bind(call_order.id)
    .bind(call_px)
    .execute(&mut *tx)
    .await?;
    apply_fill(&mut tx, account_id, &call_symbol, Side::Short, contracts_dec, call_px, Decimal::from(100)).await?;
    deduct_fee(&mut tx, account_id, call_fee).await?;

    let stock_order = insert_leg(
        &mut tx, account_id, &underlying, "buy", shares,
        "market", None, None, "filled", group, None, None, None, None,
    )
    .await?;
    sqlx::query(
        "UPDATE paper_orders SET filled_price = $2, filled_qty = qty, filled_at = now(),
                plan_note = 'covered call' WHERE id = $1",
    )
    .bind(stock_order.id)
    .bind(stock_px)
    .execute(&mut *tx)
    .await?;
    apply_fill(&mut tx, account_id, &underlying, Side::Buy, shares, stock_px, Decimal::ONE).await?;
    deduct_fee(&mut tx, account_id, stock_fee).await?;
    tx.commit().await?;

    use rust_decimal::prelude::ToPrimitive;
    let net_debit_usd = (stock_px * shares).to_f64().unwrap_or(0.0)
        - (call_px * contracts_dec * Decimal::from(100)).to_f64().unwrap_or(0.0)
        + stock_fee
        + call_fee;
    Ok(CoveredCallResult { stock_order, call_order, net_debit_usd })
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
pub async fn preview_spread(_pool: &PgPool, req: &SpreadRequest) -> anyhow::Result<SpreadPreview> {
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
/// Early-exercise sanity, pure: only LONG positions exercise (a
/// short is assigned, not exercised — that right belongs to the
/// other side), contracts bounded by the holding, contract not yet
/// expired (expiry settlement owns that). Returns the share delta
/// per the right: a call BUYS 100/contract at strike, a put SELLS
/// (possibly opening a short — buying power judges that downstream).
pub fn validate_exercise(
    occ: &traderview_core::occ_symbol::OccContract,
    pos_qty: Decimal,
    contracts: Decimal,
    today: chrono::NaiveDate,
) -> Result<Side, &'static str> {
    if pos_qty <= Decimal::ZERO {
        return Err("only long option positions can be exercised");
    }
    if contracts <= Decimal::ZERO || contracts > pos_qty {
        return Err("contracts must be positive and within the holding");
    }
    if occ.expiry < today {
        return Err("contract expired — expiry settlement handles it");
    }
    Ok(if occ.call { Side::Buy } else { Side::Sell })
}

/// Assignment sanity, pure — the mirror of validate_exercise: only
/// SHORT positions are assigned (a long holds the right, the short
/// carries the obligation), contracts bounded by |holding|, expired
/// contracts belong to expiry settlement. The share leg mirrors too:
/// an assigned CALL writer DELIVERS (sells at strike, possibly
/// opening a short); an assigned PUT writer takes delivery (buys).
pub fn validate_assignment(
    occ: &traderview_core::occ_symbol::OccContract,
    pos_qty: Decimal,
    contracts: Decimal,
    today: chrono::NaiveDate,
) -> Result<Side, &'static str> {
    if pos_qty >= Decimal::ZERO {
        return Err("only short option positions can be assigned");
    }
    if contracts <= Decimal::ZERO || contracts > pos_qty.abs() {
        return Err("contracts must be positive and within the holding");
    }
    if occ.expiry < today {
        return Err("contract expired — expiry settlement handles it");
    }
    Ok(if occ.call { Side::Sell } else { Side::Buy })
}

#[derive(Debug, Clone, Serialize)]
pub struct ExerciseResult {
    pub option_order: PaperOrder,
    pub stock_order: PaperOrder,
    /// Premium realized as a LOSS on the option close (it closes at
    /// $0 — the option is consumed; its value moves into the share
    /// basis at strike, so total economics are exact).
    pub shares: Decimal,
    pub strike: f64,
}

/// Exercise a long American option early: the option closes at $0
/// (premium burn realized — the value is delivered as shares at
/// strike, not as option proceeds) and the shares land at STRIKE:
/// calls buy 100/contract, puts sell 100/contract (possibly opening
/// a short). One transaction; OTM exercise is allowed — it's the
/// holder's right and occasionally rational around dividends — but
/// the order note states the intrinsic so the history shows what it
/// was worth.
pub async fn exercise(
    pool: &PgPool,
    user_id: Uuid,
    account_id: Uuid,
    symbol: &str,
    contracts: Decimal,
) -> anyhow::Result<ExerciseResult> {
    let owner: Option<(Uuid,)> = sqlx::query_as("SELECT user_id FROM paper_accounts WHERE id = $1")
        .bind(account_id)
        .fetch_optional(pool)
        .await?;
    if !matches!(owner, Some((u,)) if u == user_id) {
        anyhow::bail!("forbidden");
    }
    let symbol = symbol.trim().to_uppercase();
    let occ = traderview_core::occ_symbol::parse(&symbol)
        .ok_or_else(|| anyhow::anyhow!("not an OCC option symbol"))?;
    let pos: Option<(Decimal,)> = sqlx::query_as(
        "SELECT qty FROM paper_positions WHERE paper_account_id = $1 AND symbol = $2",
    )
    .bind(account_id)
    .bind(&symbol)
    .fetch_optional(pool)
    .await?;
    let pos_qty = pos.map(|(q,)| q).unwrap_or(Decimal::ZERO);
    let today = Utc::now().date_naive();
    let stock_side = validate_exercise(&occ, pos_qty, contracts, today)
        .map_err(|e| anyhow::anyhow!(e))?;
    // Intrinsic for the audit note (exercise itself prices at strike).
    let intrinsic = match crate::market_data::quote(pool, &occ.underlying).await {
        Ok(q) => traderview_core::occ_symbol::intrinsic(occ.call, occ.strike, q.price),
        Err(_) => 0.0,
    };
    let shares = contracts * Decimal::from(100);
    let strike = Decimal::try_from(occ.strike)?;
    let note = format!("early exercise (intrinsic ${intrinsic:.2}/sh)");

    let mut tx = pool.begin().await?;
    let group = Uuid::new_v4();
    let option_order = insert_leg(
        &mut tx, account_id, &symbol, "sell", contracts,
        "market", None, None, "filled", group, None, None, None, None,
    )
    .await?;
    sqlx::query(
        "UPDATE paper_orders SET filled_price = 0, filled_qty = qty, filled_at = now(),
                plan_note = $2 WHERE id = $1",
    )
    .bind(option_order.id)
    .bind(&note)
    .execute(&mut *tx)
    .await?;
    apply_fill(&mut tx, account_id, &symbol, Side::Sell, contracts, Decimal::ZERO, Decimal::from(100)).await?;

    let stock_side_str = match stock_side {
        Side::Buy => "buy",
        _ => "sell",
    };
    let stock_order = insert_leg(
        &mut tx, account_id, &occ.underlying, stock_side_str, shares,
        "market", None, None, "filled", group, None, None, None, None,
    )
    .await?;
    sqlx::query(
        "UPDATE paper_orders SET filled_price = $2, filled_qty = qty, filled_at = now(),
                plan_note = $3 WHERE id = $1",
    )
    .bind(stock_order.id)
    .bind(strike)
    .bind(&note)
    .execute(&mut *tx)
    .await?;
    apply_fill(&mut tx, account_id, &occ.underlying, stock_side, shares, strike, Decimal::ONE).await?;
    tx.commit().await?;

    Ok(ExerciseResult { option_order, stock_order, shares, strike: occ.strike })
}

/// Practice being assigned on a short option: the obligation settles
/// — the short option closes at $0 (the premium was collected at
/// open and is now fully earned-and-spent through the share leg) and
/// shares move at STRIKE per the right. Manual by design: the sim
/// has no counterparty to decide assignment for you, and inventing
/// one would be fabrication — the digest's assignment-risk warning
/// says when a real one likely would.
pub async fn assign(
    pool: &PgPool,
    user_id: Uuid,
    account_id: Uuid,
    symbol: &str,
    contracts: Decimal,
) -> anyhow::Result<ExerciseResult> {
    let owner: Option<(Uuid,)> = sqlx::query_as("SELECT user_id FROM paper_accounts WHERE id = $1")
        .bind(account_id)
        .fetch_optional(pool)
        .await?;
    if !matches!(owner, Some((u,)) if u == user_id) {
        anyhow::bail!("forbidden");
    }
    let symbol = symbol.trim().to_uppercase();
    let occ = traderview_core::occ_symbol::parse(&symbol)
        .ok_or_else(|| anyhow::anyhow!("not an OCC option symbol"))?;
    let pos: Option<(Decimal,)> = sqlx::query_as(
        "SELECT qty FROM paper_positions WHERE paper_account_id = $1 AND symbol = $2",
    )
    .bind(account_id)
    .bind(&symbol)
    .fetch_optional(pool)
    .await?;
    let pos_qty = pos.map(|(q,)| q).unwrap_or(Decimal::ZERO);
    let today = Utc::now().date_naive();
    let stock_side = validate_assignment(&occ, pos_qty, contracts, today)
        .map_err(|e| anyhow::anyhow!(e))?;
    let intrinsic = match crate::market_data::quote(pool, &occ.underlying).await {
        Ok(q) => traderview_core::occ_symbol::intrinsic(occ.call, occ.strike, q.price),
        Err(_) => 0.0,
    };
    let shares = contracts * Decimal::from(100);
    let strike = Decimal::try_from(occ.strike)?;
    let note = format!("assignment (intrinsic ${intrinsic:.2}/sh)");

    let mut tx = pool.begin().await?;
    let group = Uuid::new_v4();
    let option_order = insert_leg(
        &mut tx, account_id, &symbol, "cover", contracts,
        "market", None, None, "filled", group, None, None, None, None,
    )
    .await?;
    sqlx::query(
        "UPDATE paper_orders SET filled_price = 0, filled_qty = qty, filled_at = now(),
                plan_note = $2 WHERE id = $1",
    )
    .bind(option_order.id)
    .bind(&note)
    .execute(&mut *tx)
    .await?;
    apply_fill(&mut tx, account_id, &symbol, Side::Cover, contracts, Decimal::ZERO, Decimal::from(100)).await?;

    let stock_side_str = match stock_side {
        Side::Buy => "buy",
        _ => "sell",
    };
    let stock_order = insert_leg(
        &mut tx, account_id, &occ.underlying, stock_side_str, shares,
        "market", None, None, "filled", group, None, None, None, None,
    )
    .await?;
    sqlx::query(
        "UPDATE paper_orders SET filled_price = $2, filled_qty = qty, filled_at = now(),
                plan_note = $3 WHERE id = $1",
    )
    .bind(stock_order.id)
    .bind(strike)
    .bind(&note)
    .execute(&mut *tx)
    .await?;
    apply_fill(&mut tx, account_id, &occ.underlying, stock_side, shares, strike, Decimal::ONE).await?;
    tx.commit().await?;

    Ok(ExerciseResult { option_order, stock_order, shares, strike: occ.strike })
}

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
    is_crypto: bool,
    is_forex: bool,
}

/// Quote + contract economics for any tradeable symbol: equities from
/// the cached quote at 1x; OCC options from the chain mid at 100x.
/// Expired contracts and quoteless options are hard errors here (the
/// SUBMIT path); the ticker treats those cases as cancel/keep-resting.
/// Spot price for any engine-quotable non-option symbol: crypto
/// pairs from the venue (5s cache), everything else from the cached
/// equity quote. The auto-invest and rebalance passes share it so
/// "$100 of BTC weekly" prices through the same seam the ticket uses.
pub async fn simple_spot(pool: &PgPool, symbol: &str) -> anyhow::Result<f64> {
    if crate::crypto::is_crypto_pair(symbol) {
        crate::crypto::spot_quote_cached(symbol).await
    } else {
        Ok(crate::market_data::quote(pool, symbol).await?.price)
    }
}

#[derive(Debug, Serialize)]
pub struct StopSuggestion {
    pub last: f64,
    pub atr: f64,
    pub period: usize,
    pub bars_used: usize,
    /// Long protection: stop 2×ATR below last, target 3×ATR above —
    /// a 1.5R structure by construction.
    pub stop_long: f64,
    pub target_long: f64,
    pub stop_short: f64,
    pub target_short: f64,
}

/// Volatility-scaled stop/target suggestion from daily bars (the
/// bars store serves equities AND crypto pairs through one seam).
/// Wilder ATR(14) via the shared core indicator; refuses under 15
/// bars — an ATR off a week of data is noise wearing units.
pub async fn stop_suggestion(pool: &PgPool, symbol: &str) -> anyhow::Result<StopSuggestion> {
    use rust_decimal::prelude::ToPrimitive;
    const PERIOD: usize = 14;
    let symbol = symbol.trim().to_uppercase();
    let to = Utc::now();
    let from = to - chrono::Duration::days(60);
    let bars = crate::prices::get_bars(pool, &symbol, traderview_core::BarInterval::D1, from, to)
        .await?;
    if bars.len() < PERIOD + 1 {
        anyhow::bail!("only {} daily bars — need at least {}", bars.len(), PERIOD + 1);
    }
    let f = |d: Decimal| d.to_f64().unwrap_or(0.0);
    let highs: Vec<f64> = bars.iter().map(|b| f(b.high)).collect();
    let lows: Vec<f64> = bars.iter().map(|b| f(b.low)).collect();
    let closes: Vec<f64> = bars.iter().map(|b| f(b.close)).collect();
    let atr = traderview_core::indicators::atr(&highs, &lows, &closes, PERIOD)
        .last()
        .copied()
        .flatten()
        .filter(|a| *a > 0.0)
        .ok_or_else(|| anyhow::anyhow!("no ATR from {} bars", bars.len()))?;
    let last = simple_spot(pool, &symbol).await?;
    Ok(StopSuggestion {
        last,
        atr,
        period: PERIOD,
        bars_used: bars.len(),
        stop_long: last - 2.0 * atr,
        target_long: last + 3.0 * atr,
        stop_short: last + 2.0 * atr,
        target_short: last - 3.0 * atr,
    })
}

/// Crypto taker fee, percent of notional — the taker fee IS the
/// friction model for crypto (no per-share slippage tier, no SEC fee).
pub const CRYPTO_TAKER_FEE_PCT: f64 = 0.1;

/// Round-trip-equivalent spread charged per FX fill, in pips. Retail
/// majors trade ~0.1–1 pip on ECN; 0.5 is a conservative all-in
/// assumption. The spread IS forex's friction model — no commission,
/// no SEC fee, no per-share slippage tier.
pub const FOREX_SPREAD_PIPS: f64 = 0.5;

/// Spread cost for an FX fill: `spread_pips × pip_size × |units|`, in
/// the quote currency. The USD-denominated engine treats it as USD —
/// exact for USD-quoted majors (EURUSD, GBPUSD), an approximation for
/// USD-base and cross pairs whose pip is denominated elsewhere.
fn forex_spread_fee(symbol: &str, qty: Decimal) -> f64 {
    let units = qty.to_string().parse::<f64>().unwrap_or(0.0).abs();
    FOREX_SPREAD_PIPS * crate::forex::pip_size(symbol) * units
}

async fn resolve_quote(pool: &PgPool, symbol: &str) -> anyhow::Result<ResolvedQuote> {
    if traderview_core::occ_symbol::is_occ(symbol) {
        // handled below
    } else if crate::crypto::is_crypto_pair(symbol) {
        // Crypto spot: live venue last with the 5s cache; fractional
        // qty is native here.
        let p = crate::crypto::spot_quote_cached(symbol).await?;
        return Ok(ResolvedQuote {
            last: Decimal::try_from(p)?,
            multiplier: Decimal::ONE,
            is_option: false,
            is_crypto: true,
            is_forex: false,
        });
    } else if crate::forex::is_forex_pair(symbol) {
        // FX spot: same Yahoo-backed quote seam as equities (the `=X`
        // translation lives in market_data::quote). Multiplier 1 —
        // qty is base-currency units; the spread is the friction model.
        let q = crate::market_data::quote(pool, symbol).await?;
        return Ok(ResolvedQuote {
            last: Decimal::try_from(q.price)?,
            multiplier: Decimal::ONE,
            is_option: false,
            is_crypto: false,
            is_forex: true,
        });
    }
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
            is_crypto: false,
            is_forex: false,
        })
    } else {
        let q = crate::market_data::quote(pool, symbol).await?;
        Ok(ResolvedQuote {
            last: Decimal::try_from(q.price)?,
            multiplier: Decimal::ONE,
            is_option: false,
            is_crypto: false,
            is_forex: false,
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

    // Buying power — ONE enforcement point covering every fill path
    // (ticket, brackets, spreads, rolls, background fills). Post-state
    // validation: the tx rolls back everything above on violation.
    let (cash, m): (Decimal, Decimal) = sqlx::query_as(
        "SELECT cash, margin_multiplier FROM paper_accounts WHERE id = $1",
    )
    .bind(account_id)
    .fetch_one(&mut *tx)
    .await?;
    let books: Vec<(String, Decimal, Decimal)> = sqlx::query_as(
        "SELECT symbol, qty, avg_price FROM paper_positions WHERE paper_account_id = $1",
    )
    .bind(account_id)
    .fetch_all(&mut *tx)
    .await?;
    let (mut signed_book, mut gross_book) = (Decimal::ZERO, Decimal::ZERO);
    for (sym, q, avg) in &books {
        let mult = if traderview_core::occ_symbol::is_occ(sym) {
            Decimal::from(100)
        } else {
            Decimal::ONE
        };
        signed_book += *q * *avg * mult;
        gross_book += (*q * *avg * mult).abs();
    }
    if !buying_power_ok(cash, signed_book, gross_book, m) {
        anyhow::bail!(
            "insufficient buying power: gross exposure ${gross_book} exceeds {m}x entry-basis equity"
        );
    }

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
    fn dq(s: &str) -> Decimal {
        s.parse().unwrap()
    }

    #[test]
    fn forex_spread_fee_standard_lot_eurusd() {
        // 0.5 pip × 0.0001 pip-size × 100_000 units = $5.00 on a
        // standard lot of a USD-quoted major.
        let fee = forex_spread_fee("EURUSD", d(100_000));
        assert!((fee - 5.0).abs() < 1e-9, "got {fee}");
    }

    #[test]
    fn forex_spread_fee_jpy_pair_uses_two_decimal_pip() {
        // USDJPY pip is 0.01: 0.5 × 0.01 × 10_000 = $50 (quote-ccy).
        let fee = forex_spread_fee("USDJPY", d(10_000));
        assert!((fee - 50.0).abs() < 1e-9, "got {fee}");
    }

    #[test]
    fn forex_spread_fee_is_sign_independent() {
        // A short (negative qty) costs the same spread as a long.
        assert_eq!(
            forex_spread_fee("GBPUSD", d(50_000)),
            forex_spread_fee("GBPUSD", d(-50_000))
        );
    }

    #[test]
    fn ioc_fok_cancel_when_not_marketable() {
        // Well-formed but unfilled IOC/FOK → cancelled, not resting.
        let (s, r) = unfilled_status(true, true, "ioc");
        assert_eq!(s, "cancelled");
        assert!(r.unwrap().contains("IOC"));
        let (s, r) = unfilled_status(true, true, "fok");
        assert_eq!(s, "cancelled");
        assert!(r.unwrap().contains("FOK"));
    }

    #[test]
    fn gtc_unfilled_rests_pending() {
        let (s, r) = unfilled_status(true, false, "gtc");
        assert_eq!(s, "pending");
        assert!(r.is_none());
    }

    #[test]
    fn malformed_beats_ioc() {
        // A malformed IOC is rejected for malformation, not cancelled.
        let (s, r) = unfilled_status(false, true, "ioc");
        assert_eq!(s, "rejected");
        assert!(r.unwrap().contains("needs its limit/stop"));
    }

    #[test]
    fn trailing_stop_limit_freezes_below_stop_for_sell() {
        // High-water 100, $2 trail → stop at 98; $0.50 offset moves the
        // limit to 97.50, the most slippage the order will concede.
        let lp = trailing_limit_on_trigger(Side::Sell, d(100), d(2), false, dq("0.5"));
        assert_eq!(lp, dq("97.5"));
        // Offset 0 → fill exactly at the trail stop level (98).
        assert_eq!(trailing_limit_on_trigger(Side::Sell, d(100), d(2), false, d(0)), d(98));
    }

    #[test]
    fn trailing_stop_limit_freezes_above_stop_for_buy() {
        // Low-water 100, $2 trail → stop at 102; $0.50 offset → 102.50.
        let lp = trailing_limit_on_trigger(Side::Cover, d(100), d(2), false, dq("0.5"));
        assert_eq!(lp, dq("102.5"));
    }

    #[test]
    fn trailing_stop_limit_percentage_trail() {
        // 5% trail off high-water 100 → stop at 95; $0.25 offset → 94.75.
        let lp = trailing_limit_on_trigger(Side::Short, d(100), dq("0.05"), true, dq("0.25"));
        assert_eq!(lp, dq("94.75"));
    }

    #[test]
    fn trailing_stop_limit_is_well_formed() {
        // Needs a valid trail; the limit offset (stop_price) is optional
        // and defaults to 0 (fill at the trail level) when absent.
        assert!(order_well_formed("trailing_stop_limit", None, None, Some(d(2)), Some(false)));
        assert!(order_well_formed("trailing_stop_limit", None, Some(d(0)), Some(d(2)), Some(false)));
        assert!(order_well_formed("trailing_stop_limit", None, Some(dq("0.5")), Some(d(2)), Some(false)));
        // A bad trail, or a negative offset, is malformed.
        assert!(!order_well_formed("trailing_stop_limit", None, Some(d(0)), None, None));
        assert!(!order_well_formed("trailing_stop_limit", None, Some(d(-1)), Some(d(2)), Some(false)));
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
    fn merge_position_weights_and_refuses_netting() {
        // Empty dest: incoming basis carries over exactly.
        assert_eq!(merge_position(d(0), d(0), d(100), d(50)), Ok((d(100), d(50))));
        // Same-sign: weighted — 100@50 + 50@80 = 150@60.
        assert_eq!(merge_position(d(100), d(50), d(50), d(80)), Ok((d(150), d(60))));
        // Shorts merge with shorts the same way.
        assert_eq!(merge_position(d(-100), d(50), d(-50), d(80)), Ok((d(-150), d(60))));
        // Opposite signs refuse: netting realizes PnL, and
        // realization belongs to trades, not transfers.
        assert_eq!(
            merge_position(d(100), d(50), d(-50), d(80)),
            Err("opposite-sign merge would net positions without realization")
        );
        assert_eq!(merge_position(d(100), d(50), d(0), d(80)), Err("nothing to merge"));
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
    fn roll_validation_pins_sides_and_bounds() {
        let from = "AAPL260117C00190000";
        let to_strike = "AAPL260117C00200000";
        let to_expiry = "AAPL260618C00190000";
        let other = "MSFT260117C00190000";
        // Long 5: sell to close, buy to open — strike roll and
        // calendar roll both valid.
        assert_eq!(validate_roll(from, to_strike, d(5), d(5)), Ok((false, true)));
        assert_eq!(validate_roll(from, to_expiry, d(5), d(2)), Ok((false, true)));
        // Short -5: the mirror.
        assert_eq!(validate_roll(from, to_strike, d(-5), d(5)), Ok((true, false)));
        // Different underlying is two trades, not a roll.
        assert_eq!(
            validate_roll(from, other, d(5), d(5)),
            Err("roll must stay on the same underlying")
        );
        // Same contract, nothing held, oversize, equity symbols.
        assert_eq!(validate_roll(from, from, d(5), d(5)), Err("target contract is the same as the source"));
        assert_eq!(validate_roll(from, to_strike, d(0), d(1)), Err("no position in the source contract"));
        assert_eq!(validate_roll(from, to_strike, d(5), d(6)), Err("qty exceeds position size"));
        assert_eq!(validate_roll("AAPL", to_strike, d(5), d(5)), Err("from is not an OCC contract"));
    }

    #[test]
    fn on_close_waits_fills_and_cancels() {
        use chrono::TimeZone;
        let close = Utc.with_ymd_and_hms(2026, 6, 12, 20, 0, 0).unwrap();
        let before = close - chrono::Duration::minutes(1);
        // Before the stamped close: both types rest.
        assert_eq!(on_close_action("moc", Side::Buy, d(100), None, before, close), OnCloseAction::Wait);
        assert_eq!(on_close_action("loc", Side::Buy, d(100), Some(d(99)), before, close), OnCloseAction::Wait);
        // At the close: MOC takes last unconditionally.
        assert_eq!(on_close_action("moc", Side::Sell, d(100), None, close, close), OnCloseAction::Fill(d(100)));
        // LOC buy fills at limit-or-better, CANCELS when through it —
        // an on-close order does not survive to the next session.
        assert_eq!(on_close_action("loc", Side::Buy, d(98), Some(d(99)), close, close), OnCloseAction::Fill(d(98)));
        assert_eq!(on_close_action("loc", Side::Buy, d(100), Some(d(99)), close, close), OnCloseAction::Cancel);
        // Sell mirror.
        assert_eq!(on_close_action("loc", Side::Sell, d(100), Some(d(99)), close, close), OnCloseAction::Fill(d(100)));
        assert_eq!(on_close_action("loc", Side::Sell, d(98), Some(d(99)), close, close), OnCloseAction::Cancel);
        // LOC with no limit price is malformed — cancels at the close
        // rather than filling at an unintended price.
        assert_eq!(on_close_action("loc", Side::Buy, d(98), None, close, close), OnCloseAction::Cancel);
    }

    #[test]
    fn bracket_entry_hint_pins_all_types() {
        // Market: no knowable entry price — hint None, prices checked
        // against the live quote at fill.
        assert_eq!(bracket_entry_hint("market", None, None), Ok(None));
        // Limit anchors at its limit; missing price is the caller's bug.
        assert_eq!(bracket_entry_hint("limit", Some(d(100)), None), Ok(Some(d(100))));
        assert_eq!(bracket_entry_hint("limit", None, None), Err("limit entry needs limit_price"));
        // Stop entries anchor at the STOP — where the position comes
        // alive. stop_limit also anchors at the stop, not its limit.
        assert_eq!(bracket_entry_hint("stop", None, Some(d(105))), Ok(Some(d(105))));
        assert_eq!(bracket_entry_hint("stop", None, None), Err("stop entry needs stop_price"));
        assert_eq!(bracket_entry_hint("stop_limit", Some(d(107)), Some(d(105))), Ok(Some(d(105))));
        assert_eq!(
            bracket_entry_hint("stop_limit", None, Some(d(105))),
            Err("stop_limit entry needs stop_price and limit_price")
        );
        assert_eq!(
            bracket_entry_hint("trailing", None, None),
            Err("entry_type must be market, limit, stop, or stop_limit")
        );
    }

    #[test]
    fn exercise_validation_pins() {
        // 2027 expiries — "today" below is mid-2026 and the first
        // version of this test used Jan-2026 contracts that were
        // ALREADY EXPIRED relative to its own clock (the validator
        // caught the fixture, not the reverse).
        let occ = traderview_core::occ_symbol::parse("AAPL270115C00190000").unwrap();
        let put = traderview_core::occ_symbol::parse("AAPL270115P00190000").unwrap();
        let today = chrono::NaiveDate::from_ymd_opt(2026, 6, 12).unwrap();
        // Long call exercises into a BUY at strike; long put into a SELL.
        assert_eq!(validate_exercise(&occ, d(5), d(2), today), Ok(Side::Buy));
        assert_eq!(validate_exercise(&put, d(5), d(5), today), Ok(Side::Sell));
        // Shorts are assigned, not exercised; over-holding rejects.
        assert_eq!(
            validate_exercise(&occ, d(-5), d(1), today),
            Err("only long option positions can be exercised")
        );
        assert_eq!(
            validate_exercise(&occ, d(5), d(6), today),
            Err("contracts must be positive and within the holding")
        );
        // Expired contracts belong to expiry settlement.
        let late = chrono::NaiveDate::from_ymd_opt(2027, 1, 16).unwrap();
        assert_eq!(
            validate_exercise(&occ, d(5), d(1), late),
            Err("contract expired — expiry settlement handles it")
        );
        // Expiry DAY itself still exercises (American right runs
        // through the close).
        let on_expiry = chrono::NaiveDate::from_ymd_opt(2027, 1, 15).unwrap();
        assert_eq!(validate_exercise(&occ, d(5), d(1), on_expiry), Ok(Side::Buy));
    }

    #[test]
    fn assignment_validation_mirrors_exercise() {
        let call = traderview_core::occ_symbol::parse("AAPL270115C00190000").unwrap();
        let put = traderview_core::occ_symbol::parse("AAPL270115P00190000").unwrap();
        let today = chrono::NaiveDate::from_ymd_opt(2026, 6, 12).unwrap();
        // Assigned call writer DELIVERS (sells); assigned put writer
        // takes delivery (buys) — the exact mirror of exercise sides.
        assert_eq!(validate_assignment(&call, d(-5), d(2), today), Ok(Side::Sell));
        assert_eq!(validate_assignment(&put, d(-5), d(5), today), Ok(Side::Buy));
        // Longs hold the right; they are never assigned.
        assert_eq!(
            validate_assignment(&call, d(5), d(1), today),
            Err("only short option positions can be assigned")
        );
        assert_eq!(
            validate_assignment(&call, d(-5), d(6), today),
            Err("contracts must be positive and within the holding")
        );
        let late = chrono::NaiveDate::from_ymd_opt(2027, 1, 16).unwrap();
        assert_eq!(
            validate_assignment(&call, d(-5), d(1), late),
            Err("contract expired — expiry settlement handles it")
        );
    }

    #[test]
    fn covered_call_validation_pins() {
        // Live-pinned fixture format from the roll tests.
        let call = "AAPL260117C00190000";
        let put = "AAPL260117P00190000";
        let (und, shares) = validate_covered_call(call, 3).unwrap();
        assert_eq!(und, "AAPL");
        assert_eq!(shares, d(300));
        assert_eq!(validate_covered_call(put, 1), Err("covered call needs a CALL leg"));
        assert_eq!(validate_covered_call("AAPL", 1), Err("not an OCC option symbol"));
        assert_eq!(validate_covered_call(call, 0), Err("contracts must be in 1..=1000"));
    }

    #[test]
    fn buying_power_pins_cash_regt_and_shorts() {
        let bp = |c: i64, s: i64, g: i64, m: i64| {
            buying_power_ok(d(c), d(s), d(g), d(m))
        };
        // Cash account (m=1): a long book bounded by remaining cash —
        // $100k cash, buy $60k: cash 40k, book 60k, gross 60k ≤ 1×100k.
        assert!(bp(40_000, 60_000, 60_000, 1));
        // Overbuy on m=1: $120k of stock on $100k — cash −20k,
        // equity 100k, gross 120k > 100k: rejected.
        assert!(!bp(-20_000, 120_000, 120_000, 1));
        // Reg-T (m=2) allows exactly that: 120k ≤ 2×100k.
        assert!(bp(-20_000, 120_000, 120_000, 2));
        // Reg-T boundary: $200k of stock on $100k is exactly 2×equity.
        assert!(bp(-100_000, 200_000, 200_000, 2));
        assert!(!bp(-100_001, 200_001, 200_001, 2));
        // Shorts: proceeds raise cash, signed book is negative.
        // $100k cash + short $40k: cash 140k, signed −40k, equity
        // 100k, gross 40k ≤ 2×100k fine on Reg-T; m=1 needs
        // 2|S| ≤ cash → 40k ≤ 100k−40k... gross 40k ≤ 1×100k: ok.
        assert!(bp(140_000, -40_000, 40_000, 2));
        // Blown account: equity ≤ 0 fails regardless of multiplier.
        assert!(!bp(10_000, -50_000, 50_000, 4));
        // Empty book: plain cash sanity.
        assert!(bp(100_000, 0, 0, 1));
        assert!(!bp(-1, 0, 0, 4));
        // Long: target above the entry hint; trail $2 valid.
        assert!(validate_trailing_bracket(Side::Buy, Some(d(100)), d(110), Some(d(2)), Some(false)).is_ok());
        // Target on the wrong side rejects, both directions.
        assert_eq!(
            validate_trailing_bracket(Side::Buy, Some(d(100)), d(95), Some(d(2)), None),
            Err("take_profit must be on the profit side of the entry")
        );
        assert_eq!(
            validate_trailing_bracket(Side::Short, Some(d(100)), d(105), Some(d(2)), None),
            Err("take_profit must be on the profit side of the entry")
        );
        // Short with target below passes; market entry (no hint) only
        // checks the trail itself.
        assert!(validate_trailing_bracket(Side::Short, Some(d(100)), d(90), Some(d(2)), None).is_ok());
        assert!(validate_trailing_bracket(Side::Buy, None, d(1), Some(d(2)), None).is_ok());
        // Bad trails: zero, and a pct ≥ 1.
        assert!(validate_trailing_bracket(Side::Buy, None, d(110), Some(d(0)), None).is_err());
        assert!(validate_trailing_bracket(Side::Buy, None, d(110), Some(d(2)), Some(true)).is_err());
        // Sell/cover are not entry sides.
        assert!(validate_trailing_bracket(Side::Sell, None, d(110), Some(d(2)), None).is_err());
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
