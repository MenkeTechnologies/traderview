//! 13F institutional-holdings queries.
//!
//! Read-only query layer over the tables defined in 0033. The EDGAR
//! poller that *populates* these tables lives in a follow-up module —
//! this file just exposes the read paths the UI / API uses:
//!
//!   * `list_managers` (search + pagination)
//!   * `holdings_for_manager` (latest filing, or specific quarter)
//!   * `position_changes_for_manager` (latest vs prior quarter)
//!   * `top_owners_of_symbol` (who owns SYM, ordered by value)
//!   * `top_managers_by_aum`
//!   * `manager_filings` (history per CIK)

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Manager {
    pub id: Uuid,
    pub cik: String,
    pub name: String,
    pub manager_type: String,
    pub state: Option<String>,
    pub aliases: Vec<String>,
    pub notable: bool,
    pub first_seen_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Filing {
    pub id: Uuid,
    pub manager_id: Uuid,
    pub accession_number: String,
    pub form_type: String,
    pub quarter_end: NaiveDate,
    pub filed_at: DateTime<Utc>,
    pub detected_at: DateTime<Utc>,
    pub total_value_usd: Option<Decimal>,
    pub holdings_count: i32,
    pub source_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Holding {
    pub id: Uuid,
    pub filing_id: Uuid,
    pub cusip: String,
    pub symbol: Option<String>,
    pub issuer_name: String,
    pub issuer_class: Option<String>,
    pub put_call: Option<String>,
    pub shares: Decimal,
    pub value_usd: Decimal,
    pub sole_voting: Option<Decimal>,
    pub shared_voting: Option<Decimal>,
    pub none_voting: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct PositionChange {
    pub manager_id: Uuid,
    pub manager_name: String,
    pub cusip: String,
    pub symbol: Option<String>,
    pub issuer_name: String,
    pub current_quarter: NaiveDate,
    pub prior_quarter: Option<NaiveDate>,
    pub shares_now: Decimal,
    pub shares_prior: Decimal,
    pub value_now: Decimal,
    pub value_prior: Decimal,
    pub delta_shares: Decimal,
    pub delta_value: Decimal,
    pub change_type: String, // 'new' | 'increased' | 'decreased' | 'held'
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct SymbolOwner {
    pub manager_id: Uuid,
    pub manager_name: String,
    pub notable: bool,
    pub shares: Decimal,
    pub value_usd: Decimal,
    pub quarter_end: NaiveDate,
}

pub async fn list_managers(
    pool: &PgPool,
    search: Option<&str>,
    notable_only: bool,
    limit: i64,
) -> anyhow::Result<Vec<Manager>> {
    let q = search.unwrap_or("").trim().to_uppercase();
    Ok(sqlx::query_as(
        "SELECT id, cik, name, manager_type::text, state, aliases, notable,
                first_seen_at, last_seen_at
           FROM institutional_managers
          WHERE ($1 = '' OR name ILIKE '%' || $1 || '%' OR cik = $1)
            AND ($2 = FALSE OR notable = TRUE)
          ORDER BY notable DESC, name ASC
          LIMIT $3",
    )
    .bind(q)
    .bind(notable_only)
    .bind(limit)
    .fetch_all(pool)
    .await?)
}

pub async fn manager_by_cik(pool: &PgPool, cik: &str) -> anyhow::Result<Option<Manager>> {
    Ok(sqlx::query_as(
        "SELECT id, cik, name, manager_type::text, state, aliases, notable,
                first_seen_at, last_seen_at
           FROM institutional_managers WHERE cik = $1",
    )
    .bind(cik)
    .fetch_optional(pool)
    .await?)
}

pub async fn manager_filings(
    pool: &PgPool,
    manager_id: Uuid,
    limit: i64,
) -> anyhow::Result<Vec<Filing>> {
    Ok(sqlx::query_as(
        "SELECT id, manager_id, accession_number, form_type, quarter_end,
                filed_at, detected_at, total_value_usd, holdings_count, source_url
           FROM institutional_13f_filings
          WHERE manager_id = $1
          ORDER BY quarter_end DESC, filed_at DESC
          LIMIT $2",
    )
    .bind(manager_id)
    .bind(limit)
    .fetch_all(pool)
    .await?)
}

pub async fn holdings_for_manager_latest(
    pool: &PgPool,
    manager_id: Uuid,
    limit: i64,
) -> anyhow::Result<Vec<Holding>> {
    Ok(sqlx::query_as(
        "SELECT h.id, h.filing_id, h.cusip, h.symbol, h.issuer_name, h.issuer_class,
                h.put_call, h.shares, h.value_usd, h.sole_voting, h.shared_voting,
                h.none_voting
           FROM institutional_holdings h
           JOIN institutional_latest_filings lf ON lf.filing_id = h.filing_id
          WHERE lf.manager_id = $1
          ORDER BY h.value_usd DESC
          LIMIT $2",
    )
    .bind(manager_id)
    .bind(limit)
    .fetch_all(pool)
    .await?)
}

pub async fn holdings_for_filing(
    pool: &PgPool,
    filing_id: Uuid,
    limit: i64,
) -> anyhow::Result<Vec<Holding>> {
    Ok(sqlx::query_as(
        "SELECT id, filing_id, cusip, symbol, issuer_name, issuer_class,
                put_call, shares, value_usd, sole_voting, shared_voting, none_voting
           FROM institutional_holdings
          WHERE filing_id = $1
          ORDER BY value_usd DESC
          LIMIT $2",
    )
    .bind(filing_id)
    .bind(limit)
    .fetch_all(pool)
    .await?)
}

pub async fn position_changes_for_manager(
    pool: &PgPool,
    manager_id: Uuid,
    change_type: Option<&str>,
    limit: i64,
) -> anyhow::Result<Vec<PositionChange>> {
    let ct = change_type.unwrap_or("");
    Ok(sqlx::query_as(
        "SELECT pc.manager_id, m.name AS manager_name, pc.cusip, pc.symbol,
                pc.issuer_name, pc.current_quarter, pc.prior_quarter,
                pc.shares_now, pc.shares_prior, pc.value_now, pc.value_prior,
                pc.delta_shares, pc.delta_value, pc.change_type
           FROM institutional_position_changes pc
           JOIN institutional_managers m ON m.id = pc.manager_id
          WHERE pc.manager_id = $1
            AND ($2 = '' OR pc.change_type = $2)
          ORDER BY ABS(pc.delta_value) DESC
          LIMIT $3",
    )
    .bind(manager_id)
    .bind(ct)
    .bind(limit)
    .fetch_all(pool)
    .await?)
}

pub async fn top_owners_of_symbol(
    pool: &PgPool,
    symbol: &str,
    limit: i64,
) -> anyhow::Result<Vec<SymbolOwner>> {
    let sym = symbol.trim().to_uppercase();
    Ok(sqlx::query_as(
        "SELECT m.id AS manager_id, m.name AS manager_name, m.notable,
                h.shares, h.value_usd, lf.quarter_end
           FROM institutional_holdings h
           JOIN institutional_latest_filings lf ON lf.filing_id = h.filing_id
           JOIN institutional_managers m ON m.id = lf.manager_id
          WHERE h.symbol = $1
          ORDER BY h.value_usd DESC
          LIMIT $2",
    )
    .bind(sym)
    .bind(limit)
    .fetch_all(pool)
    .await?)
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct ManagerAum {
    pub manager_id: Uuid,
    pub manager_name: String,
    pub notable: bool,
    pub aum_usd: Decimal,
    pub holdings_count: i32,
    pub quarter_end: NaiveDate,
}

pub async fn top_managers_by_aum(pool: &PgPool, limit: i64) -> anyhow::Result<Vec<ManagerAum>> {
    Ok(sqlx::query_as(
        "SELECT m.id AS manager_id, m.name AS manager_name, m.notable,
                COALESCE(lf.total_value_usd, 0) AS aum_usd,
                lf.holdings_count, lf.quarter_end
           FROM institutional_managers m
           JOIN institutional_latest_filings lf ON lf.manager_id = m.id
          ORDER BY aum_usd DESC
          LIMIT $1",
    )
    .bind(limit)
    .fetch_all(pool)
    .await?)
}
