use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use traderview_core::{AssetClass, Execution, OptionType, Side};
use traderview_import::ParsedExecution;
use uuid::Uuid;

pub async fn list_for_account(pool: &PgPool, account_id: Uuid) -> anyhow::Result<Vec<Execution>> {
    let rows: Vec<ExecRow> = sqlx::query_as(
        "SELECT id, account_id, symbol, side::text, qty, price, fee, executed_at, broker_order_id,
                raw, import_id,
                asset_class::text, option_type::text, strike, expiration, multiplier,
                tick_size, tick_value, base_ccy, quote_ccy, pip_size
           FROM executions WHERE account_id = $1 ORDER BY executed_at ASC, id ASC",
    )
    .bind(account_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

pub async fn list_for_trade(pool: &PgPool, trade_id: Uuid) -> anyhow::Result<Vec<Execution>> {
    let rows: Vec<ExecRow> = sqlx::query_as(
        "SELECT e.id, e.account_id, e.symbol, e.side::text, e.qty, e.price, e.fee,
                e.executed_at, e.broker_order_id, e.raw, e.import_id,
                e.asset_class::text, e.option_type::text, e.strike, e.expiration, e.multiplier,
                e.tick_size, e.tick_value, e.base_ccy, e.quote_ccy, e.pip_size
           FROM executions e
           JOIN trade_executions te ON te.execution_id = e.id
          WHERE te.trade_id = $1
          ORDER BY e.executed_at ASC",
    )
    .bind(trade_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

/// Insert a parsed execution. Returns Ok(true) if inserted, Ok(false) if
/// rejected by the dedupe constraint.
pub async fn insert_parsed(
    pool: &PgPool,
    account_id: Uuid,
    import_id: Uuid,
    p: &ParsedExecution,
) -> anyhow::Result<bool> {
    let r = sqlx::query(
        "INSERT INTO executions (
            account_id, symbol, side, qty, price, fee, executed_at,
            broker_order_id, raw, import_id,
            asset_class, option_type, strike, expiration, multiplier,
            tick_size, tick_value, base_ccy, quote_ccy, pip_size
         ) VALUES (
            $1, $2, $3::side_t, $4, $5, $6, $7,
            $8, $9, $10,
            $11::asset_class_t, $12, $13, $14, $15,
            $16, $17, $18, $19, $20
         )
         ON CONFLICT DO NOTHING",
    )
    .bind(account_id)
    .bind(&p.symbol)
    .bind(side_to_pg(p.side))
    .bind(p.qty)
    .bind(p.price)
    .bind(p.fee)
    .bind(p.executed_at)
    .bind(p.broker_order_id.as_deref())
    .bind(&p.raw)
    .bind(import_id)
    .bind(ac_to_pg(p.asset_class))
    .bind(p.option_type.map(option_type_to_pg).map(|s| s as &str))
    .bind(p.strike)
    .bind(p.expiration)
    .bind(p.multiplier)
    .bind(p.tick_size)
    .bind(p.tick_value)
    .bind(p.base_ccy.as_deref())
    .bind(p.quote_ccy.as_deref())
    .bind(p.pip_size)
    .execute(pool)
    .await?;
    Ok(r.rows_affected() > 0)
}

pub async fn insert_manual(
    pool: &PgPool,
    account_id: Uuid,
    p: &ParsedExecution,
) -> anyhow::Result<Uuid> {
    let (id,): (Uuid,) = sqlx::query_as(
        "INSERT INTO executions (
            account_id, symbol, side, qty, price, fee, executed_at,
            broker_order_id, raw, asset_class, option_type, strike, expiration, multiplier,
            tick_size, tick_value, base_ccy, quote_ccy, pip_size
         ) VALUES (
            $1, $2, $3::side_t, $4, $5, $6, $7,
            $8, $9, $10::asset_class_t, $11, $12, $13, $14,
            $15, $16, $17, $18, $19
         ) RETURNING id",
    )
    .bind(account_id)
    .bind(&p.symbol)
    .bind(side_to_pg(p.side))
    .bind(p.qty)
    .bind(p.price)
    .bind(p.fee)
    .bind(p.executed_at)
    .bind(p.broker_order_id.as_deref())
    .bind(&p.raw)
    .bind(ac_to_pg(p.asset_class))
    .bind(p.option_type.map(option_type_to_pg).map(|s| s as &str))
    .bind(p.strike)
    .bind(p.expiration)
    .bind(p.multiplier)
    .bind(p.tick_size)
    .bind(p.tick_value)
    .bind(p.base_ccy.as_deref())
    .bind(p.quote_ccy.as_deref())
    .bind(p.pip_size)
    .fetch_one(pool)
    .await?;
    Ok(id)
}

pub async fn delete(pool: &PgPool, execution_id: Uuid) -> anyhow::Result<bool> {
    let r = sqlx::query("DELETE FROM executions WHERE id = $1")
        .bind(execution_id)
        .execute(pool)
        .await?;
    Ok(r.rows_affected() > 0)
}

fn side_to_pg(s: Side) -> &'static str {
    match s {
        Side::Buy => "buy",
        Side::Sell => "sell",
        Side::Short => "short",
        Side::Cover => "cover",
    }
}

fn ac_to_pg(a: AssetClass) -> &'static str {
    match a {
        AssetClass::Stock => "stock",
        AssetClass::Option => "option",
        AssetClass::Future => "future",
        AssetClass::Forex => "forex",
    }
}

fn option_type_to_pg(o: OptionType) -> &'static str {
    match o {
        OptionType::Call => "call",
        OptionType::Put => "put",
    }
}

#[derive(sqlx::FromRow)]
pub struct ExecRow {
    pub id: Uuid,
    pub account_id: Uuid,
    pub symbol: String,
    pub side: String,
    pub qty: Decimal,
    pub price: Decimal,
    pub fee: Decimal,
    pub executed_at: DateTime<Utc>,
    pub broker_order_id: Option<String>,
    pub raw: serde_json::Value,
    pub import_id: Option<Uuid>,
    pub asset_class: String,
    pub option_type: Option<String>,
    pub strike: Option<Decimal>,
    pub expiration: Option<NaiveDate>,
    pub multiplier: Decimal,
    pub tick_size: Option<Decimal>,
    pub tick_value: Option<Decimal>,
    pub base_ccy: Option<String>,
    pub quote_ccy: Option<String>,
    pub pip_size: Option<Decimal>,
}

impl From<ExecRow> for Execution {
    fn from(r: ExecRow) -> Self {
        Execution {
            id: r.id,
            account_id: r.account_id,
            symbol: r.symbol,
            side: match r.side.as_str() {
                "buy" => Side::Buy,
                "sell" => Side::Sell,
                "short" => Side::Short,
                "cover" => Side::Cover,
                _ => Side::Buy,
            },
            qty: r.qty,
            price: r.price,
            fee: r.fee,
            executed_at: r.executed_at,
            broker_order_id: r.broker_order_id,
            raw: r.raw,
            import_id: r.import_id,
            asset_class: match r.asset_class.as_str() {
                "option" => AssetClass::Option,
                "future" => AssetClass::Future,
                "forex" => AssetClass::Forex,
                _ => AssetClass::Stock,
            },
            option_type: r.option_type.and_then(|s| match s.as_str() {
                "call" => Some(OptionType::Call),
                "put" => Some(OptionType::Put),
                _ => None,
            }),
            strike: r.strike,
            expiration: r.expiration,
            multiplier: r.multiplier,
            tick_size: r.tick_size,
            tick_value: r.tick_value,
            base_ccy: r.base_ccy,
            quote_ccy: r.quote_ccy,
            pip_size: r.pip_size,
        }
    }
}
