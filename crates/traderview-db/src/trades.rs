use crate::executions;
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::Deserialize;
use sqlx::PgPool;
use traderview_core::{
    rollup::{rollup, LotMethod, RolledTrade},
    AssetClass, OptionType, Trade, TradeSide, TradeStatus,
};
use uuid::Uuid;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct TradeFilter {
    pub symbol: Option<String>,
    pub status: Option<TradeStatus>,
    pub side: Option<TradeSide>,
    pub asset_class: Option<AssetClass>,
    pub date_from: Option<NaiveDate>,
    pub date_to: Option<NaiveDate>,
    pub tag_id: Option<Uuid>,
    pub min_pnl: Option<Decimal>,
    pub max_pnl: Option<Decimal>,
    pub min_qty: Option<Decimal>,
    pub max_qty: Option<Decimal>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list_for_account(
    pool: &PgPool,
    account_id: Uuid,
    f: &TradeFilter,
) -> anyhow::Result<Vec<Trade>> {
    let mut q = sqlx::QueryBuilder::new(
        "SELECT id, account_id, symbol, side::text, status::text, opened_at, closed_at,
                qty, entry_avg, exit_avg, gross_pnl, fees, net_pnl,
                asset_class::text, option_type::text, strike, expiration, multiplier,
                tick_size, tick_value, base_ccy, quote_ccy, pip_size,
                stop_loss, risk_amount, initial_target, mfe, mae, best_exit_pnl, exit_efficiency
           FROM trades WHERE account_id = ",
    );
    q.push_bind(account_id);
    if let Some(sym) = &f.symbol {
        q.push(" AND symbol = ").push_bind(sym.clone());
    }
    if let Some(status) = f.status {
        let s = match status {
            TradeStatus::Open => "open",
            TradeStatus::Closed => "closed",
        };
        q.push(" AND status = ").push_bind(s).push("::trade_status_t");
    }
    if let Some(side) = f.side {
        let s = match side {
            TradeSide::Long => "long",
            TradeSide::Short => "short",
        };
        q.push(" AND side = ").push_bind(s).push("::trade_side_t");
    }
    if let Some(ac) = f.asset_class {
        let s = ac_to_pg(ac);
        q.push(" AND asset_class = ").push_bind(s).push("::asset_class_t");
    }
    if let Some(d) = f.date_from {
        q.push(" AND opened_at >= ").push_bind(d.and_hms_opt(0, 0, 0).unwrap().and_utc());
    }
    if let Some(d) = f.date_to {
        q.push(" AND opened_at < ")
            .push_bind(d.succ_opt().unwrap_or(d).and_hms_opt(0, 0, 0).unwrap().and_utc());
    }
    if let Some(p) = f.min_pnl {
        q.push(" AND net_pnl >= ").push_bind(p);
    }
    if let Some(p) = f.max_pnl {
        q.push(" AND net_pnl <= ").push_bind(p);
    }
    if let Some(qty) = f.min_qty {
        q.push(" AND qty >= ").push_bind(qty);
    }
    if let Some(qty) = f.max_qty {
        q.push(" AND qty <= ").push_bind(qty);
    }
    if let Some(tag) = f.tag_id {
        q.push(" AND id IN (SELECT trade_id FROM trade_tags WHERE tag_id = ")
            .push_bind(tag)
            .push(")");
    }
    q.push(" ORDER BY opened_at DESC");
    if let Some(l) = f.limit {
        q.push(" LIMIT ").push_bind(l);
    }
    if let Some(o) = f.offset {
        q.push(" OFFSET ").push_bind(o);
    }

    let rows: Vec<TradeRow> = q.build_query_as().fetch_all(pool).await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

pub async fn get(pool: &PgPool, trade_id: Uuid) -> anyhow::Result<Option<Trade>> {
    let row: Option<TradeRow> = sqlx::query_as(
        "SELECT id, account_id, symbol, side::text, status::text, opened_at, closed_at,
                qty, entry_avg, exit_avg, gross_pnl, fees, net_pnl,
                asset_class::text, option_type::text, strike, expiration, multiplier,
                tick_size, tick_value, base_ccy, quote_ccy, pip_size,
                stop_loss, risk_amount, initial_target, mfe, mae, best_exit_pnl, exit_efficiency
           FROM trades WHERE id = $1",
    )
    .bind(trade_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(Into::into))
}

/// Drop all trades+legs for `account_id` and re-derive from current executions.
/// Idempotent. Returns the count of trades emitted.
pub async fn rollup_account(pool: &PgPool, account_id: Uuid) -> anyhow::Result<usize> {
    let execs = executions::list_for_account(pool, account_id).await?;
    let trades = rollup(&execs, LotMethod::Fifo)?;
    let n = trades.len();

    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM trades WHERE account_id = $1")
        .bind(account_id)
        .execute(&mut *tx)
        .await?;
    for rt in &trades {
        insert_rolled(&mut *tx, rt).await?;
    }
    tx.commit().await?;
    Ok(n)
}

async fn insert_rolled(tx: &mut sqlx::PgConnection, rt: &RolledTrade) -> anyhow::Result<()> {
    let t = &rt.trade;
    sqlx::query(
        "INSERT INTO trades (
            id, account_id, symbol, side, status, opened_at, closed_at,
            qty, entry_avg, exit_avg, gross_pnl, fees, net_pnl,
            asset_class, option_type, strike, expiration, multiplier,
            tick_size, tick_value, base_ccy, quote_ccy, pip_size
         ) VALUES (
            $1, $2, $3, $4::trade_side_t, $5::trade_status_t, $6, $7,
            $8, $9, $10, $11, $12, $13,
            $14::asset_class_t, $15, $16, $17, $18,
            $19, $20, $21, $22, $23
         )",
    )
    .bind(t.id)
    .bind(t.account_id)
    .bind(&t.symbol)
    .bind(side_to_pg(t.side))
    .bind(status_to_pg(t.status))
    .bind(t.opened_at)
    .bind(t.closed_at)
    .bind(t.qty)
    .bind(t.entry_avg)
    .bind(t.exit_avg)
    .bind(t.gross_pnl)
    .bind(t.fees)
    .bind(t.net_pnl)
    .bind(ac_to_pg(t.asset_class))
    .bind(t.option_type.map(option_type_to_pg).map(|s| s as &str))
    .bind(t.strike)
    .bind(t.expiration)
    .bind(t.multiplier)
    .bind(t.tick_size)
    .bind(t.tick_value)
    .bind(t.base_ccy.as_deref())
    .bind(t.quote_ccy.as_deref())
    .bind(t.pip_size)
    .execute(&mut *tx)
    .await?;

    for leg in &rt.legs {
        sqlx::query(
            "INSERT INTO trade_executions (trade_id, execution_id, qty_used)
                  VALUES ($1, $2, $3)",
        )
        .bind(leg.trade_id)
        .bind(leg.execution_id)
        .bind(leg.qty_used)
        .execute(&mut *tx)
        .await?;
    }
    Ok(())
}

pub async fn set_risk_fields(
    pool: &PgPool,
    trade_id: Uuid,
    stop_loss: Option<Decimal>,
    risk_amount: Option<Decimal>,
    initial_target: Option<Decimal>,
) -> anyhow::Result<()> {
    sqlx::query(
        "UPDATE trades SET stop_loss = $2, risk_amount = $3, initial_target = $4,
                            updated_at = now() WHERE id = $1",
    )
    .bind(trade_id)
    .bind(stop_loss)
    .bind(risk_amount)
    .bind(initial_target)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn set_excursion(
    pool: &PgPool,
    trade_id: Uuid,
    mfe: Decimal,
    mae: Decimal,
    best_exit_pnl: Decimal,
) -> anyhow::Result<()> {
    let efficiency: Option<Decimal> = sqlx::query_scalar::<_, Option<Decimal>>(
        "SELECT CASE WHEN $2 = 0 THEN NULL ELSE net_pnl / $2 END
           FROM trades WHERE id = $1",
    )
    .bind(trade_id)
    .bind(best_exit_pnl)
    .fetch_optional(pool)
    .await?
    .flatten();

    sqlx::query(
        "UPDATE trades SET mfe = $2, mae = $3, best_exit_pnl = $4, exit_efficiency = $5,
                            updated_at = now() WHERE id = $1",
    )
    .bind(trade_id)
    .bind(mfe)
    .bind(mae)
    .bind(best_exit_pnl)
    .bind(efficiency)
    .execute(pool)
    .await?;
    Ok(())
}

// =================== conversions =====================

fn ac_to_pg(a: AssetClass) -> &'static str {
    match a {
        AssetClass::Stock => "stock",
        AssetClass::Option => "option",
        AssetClass::Future => "future",
        AssetClass::Forex => "forex",
    }
}

fn side_to_pg(s: TradeSide) -> &'static str {
    match s {
        TradeSide::Long => "long",
        TradeSide::Short => "short",
    }
}

fn status_to_pg(s: TradeStatus) -> &'static str {
    match s {
        TradeStatus::Open => "open",
        TradeStatus::Closed => "closed",
    }
}

fn option_type_to_pg(o: OptionType) -> &'static str {
    match o {
        OptionType::Call => "call",
        OptionType::Put => "put",
    }
}

#[derive(sqlx::FromRow)]
pub struct TradeRow {
    pub id: Uuid,
    pub account_id: Uuid,
    pub symbol: String,
    pub side: String,
    pub status: String,
    pub opened_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub qty: Decimal,
    pub entry_avg: Decimal,
    pub exit_avg: Option<Decimal>,
    pub gross_pnl: Option<Decimal>,
    pub fees: Decimal,
    pub net_pnl: Option<Decimal>,
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
    pub stop_loss: Option<Decimal>,
    pub risk_amount: Option<Decimal>,
    pub initial_target: Option<Decimal>,
    pub mfe: Option<Decimal>,
    pub mae: Option<Decimal>,
    pub best_exit_pnl: Option<Decimal>,
    pub exit_efficiency: Option<Decimal>,
}

impl From<TradeRow> for Trade {
    fn from(r: TradeRow) -> Self {
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
            stop_loss: r.stop_loss,
            risk_amount: r.risk_amount,
            initial_target: r.initial_target,
            mfe: r.mfe,
            mae: r.mae,
            best_exit_pnl: r.best_exit_pnl,
            exit_efficiency: r.exit_efficiency,
        }
    }
}
