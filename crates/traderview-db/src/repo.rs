//! Thin repository helpers. Hand-written sqlx, not an ORM.

use sqlx::PgPool;
use traderview_core::{Account, JournalEntry, Trade};
use uuid::Uuid;

/// Insert a row into `users` if it doesn't exist. Returns the id.
pub async fn ensure_local_user(pool: &PgPool) -> anyhow::Result<Uuid> {
    let existing: Option<(Uuid,)> =
        sqlx::query_as("SELECT id FROM users WHERE is_local = true LIMIT 1")
            .fetch_optional(pool)
            .await?;
    if let Some((id,)) = existing {
        return Ok(id);
    }
    let (id,): (Uuid,) = sqlx::query_as(
        "INSERT INTO users (display_name, is_local) VALUES ($1, true) RETURNING id",
    )
    .bind("local")
    .fetch_one(pool)
    .await?;
    Ok(id)
}

pub async fn list_accounts(pool: &PgPool, user_id: Uuid) -> anyhow::Result<Vec<Account>> {
    let rows: Vec<Account> = sqlx::query_as::<_, AccountRow>(
        "SELECT id, user_id, broker, name, base_currency, created_at
           FROM accounts WHERE user_id = $1 ORDER BY created_at ASC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(Into::into)
    .collect();
    Ok(rows)
}

pub async fn list_trades(
    pool: &PgPool,
    account_id: Uuid,
    limit: i64,
    offset: i64,
) -> anyhow::Result<Vec<Trade>> {
    let rows: Vec<Trade> = sqlx::query_as::<_, TradeRow>(
        "SELECT id, account_id, symbol, side, status, opened_at, closed_at,
                qty, entry_avg, exit_avg, gross_pnl, fees, net_pnl
           FROM trades WHERE account_id = $1
          ORDER BY opened_at DESC LIMIT $2 OFFSET $3",
    )
    .bind(account_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(Into::into)
    .collect();
    Ok(rows)
}

pub async fn list_journal_for_day(
    pool: &PgPool,
    user_id: Uuid,
    day: chrono::NaiveDate,
) -> anyhow::Result<Vec<JournalEntry>> {
    let rows: Vec<JournalEntry> = sqlx::query_as::<_, JournalRow>(
        "SELECT id, user_id, trade_id, day, body_md, mood, created_at, updated_at
           FROM journal_entries
          WHERE user_id = $1 AND day = $2
          ORDER BY created_at ASC",
    )
    .bind(user_id)
    .bind(day)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(Into::into)
    .collect();
    Ok(rows)
}

// --- Row adapters --------------------------------------------------------
// sqlx::FromRow can't derive directly on traderview-core types because of
// foreign-enum encodings; lift Postgres rows into local newtypes and convert.

#[derive(sqlx::FromRow)]
struct AccountRow {
    id: Uuid,
    user_id: Uuid,
    broker: String,
    name: String,
    base_currency: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<AccountRow> for Account {
    fn from(r: AccountRow) -> Self {
        Account {
            id: r.id,
            user_id: r.user_id,
            broker: r.broker,
            name: r.name,
            base_currency: r.base_currency,
            created_at: r.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct TradeRow {
    id: Uuid,
    account_id: Uuid,
    symbol: String,
    side: String,
    status: String,
    opened_at: chrono::DateTime<chrono::Utc>,
    closed_at: Option<chrono::DateTime<chrono::Utc>>,
    qty: rust_decimal::Decimal,
    entry_avg: rust_decimal::Decimal,
    exit_avg: Option<rust_decimal::Decimal>,
    gross_pnl: Option<rust_decimal::Decimal>,
    fees: rust_decimal::Decimal,
    net_pnl: Option<rust_decimal::Decimal>,
}

impl From<TradeRow> for Trade {
    fn from(r: TradeRow) -> Self {
        use traderview_core::{TradeSide, TradeStatus};
        Trade {
            id: r.id,
            account_id: r.account_id,
            symbol: r.symbol,
            side: match r.side.as_str() {
                "short" => TradeSide::Short,
                _ => TradeSide::Long,
            },
            status: match r.status.as_str() {
                "closed" => TradeStatus::Closed,
                _ => TradeStatus::Open,
            },
            opened_at: r.opened_at,
            closed_at: r.closed_at,
            qty: r.qty,
            entry_avg: r.entry_avg,
            exit_avg: r.exit_avg,
            gross_pnl: r.gross_pnl,
            fees: r.fees,
            net_pnl: r.net_pnl,
        }
    }
}

#[derive(sqlx::FromRow)]
struct JournalRow {
    id: Uuid,
    user_id: Uuid,
    trade_id: Option<Uuid>,
    day: Option<chrono::NaiveDate>,
    body_md: String,
    mood: Option<i16>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<JournalRow> for JournalEntry {
    fn from(r: JournalRow) -> Self {
        JournalEntry {
            id: r.id,
            user_id: r.user_id,
            trade_id: r.trade_id,
            day: r.day,
            body_md: r.body_md,
            mood: r.mood,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}
