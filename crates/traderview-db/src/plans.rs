use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use traderview_core::{AssetClass, TradePlan, TradeSide};
use uuid::Uuid;

pub struct NewPlan<'a> {
    pub user_id: Uuid,
    pub account_id: Uuid,
    pub symbol: &'a str,
    pub asset_class: AssetClass,
    pub side: TradeSide,
    pub intended_qty: Decimal,
    pub intended_entry: Decimal,
    pub stop_loss: Option<Decimal>,
    pub initial_target: Option<Decimal>,
    pub setup_notes: &'a str,
}

pub async fn create(pool: &PgPool, p: NewPlan<'_>) -> anyhow::Result<TradePlan> {
    let ac = match p.asset_class {
        AssetClass::Stock => "stock",
        AssetClass::Option => "option",
        AssetClass::Future => "future",
        AssetClass::Forex => "forex",
    };
    let s = match p.side {
        TradeSide::Long => "long",
        TradeSide::Short => "short",
    };
    let row: Row = sqlx::query_as(
        "INSERT INTO trade_plans
            (user_id, account_id, symbol, asset_class, side, intended_qty,
             intended_entry, stop_loss, initial_target, setup_notes)
         VALUES ($1, $2, $3, $4::asset_class_t, $5::trade_side_t, $6, $7, $8, $9, $10)
         RETURNING id, user_id, account_id, symbol, asset_class::text, side::text,
                   intended_qty, intended_entry, stop_loss, initial_target, setup_notes,
                   plan_status, linked_trade_id, created_at, filled_at",
    )
    .bind(p.user_id)
    .bind(p.account_id)
    .bind(p.symbol)
    .bind(ac)
    .bind(s)
    .bind(p.intended_qty)
    .bind(p.intended_entry)
    .bind(p.stop_loss)
    .bind(p.initial_target)
    .bind(p.setup_notes)
    .fetch_one(pool)
    .await?;
    Ok(row.into())
}

pub async fn list_pending(pool: &PgPool, user_id: Uuid) -> anyhow::Result<Vec<TradePlan>> {
    let rows: Vec<Row> = sqlx::query_as(
        "SELECT id, user_id, account_id, symbol, asset_class::text, side::text,
                intended_qty, intended_entry, stop_loss, initial_target, setup_notes,
                plan_status, linked_trade_id, created_at, filled_at
           FROM trade_plans
          WHERE user_id = $1 AND plan_status = 'pending'
          ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

pub async fn link_to_trade(
    pool: &PgPool,
    user_id: Uuid,
    plan_id: Uuid,
    trade_id: Uuid,
) -> anyhow::Result<bool> {
    let r = sqlx::query(
        "UPDATE trade_plans SET plan_status = 'filled', linked_trade_id = $3,
                                  filled_at = now(), updated_at = now()
          WHERE id = $1 AND user_id = $2",
    )
    .bind(plan_id)
    .bind(user_id)
    .bind(trade_id)
    .execute(pool)
    .await?;
    Ok(r.rows_affected() > 0)
}

pub async fn abandon(pool: &PgPool, user_id: Uuid, plan_id: Uuid) -> anyhow::Result<bool> {
    let r = sqlx::query(
        "UPDATE trade_plans SET plan_status = 'abandoned', updated_at = now()
          WHERE id = $1 AND user_id = $2",
    )
    .bind(plan_id)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(r.rows_affected() > 0)
}

#[derive(sqlx::FromRow)]
struct Row {
    id: Uuid,
    user_id: Uuid,
    account_id: Uuid,
    symbol: String,
    asset_class: String,
    side: String,
    intended_qty: Decimal,
    intended_entry: Decimal,
    stop_loss: Option<Decimal>,
    initial_target: Option<Decimal>,
    setup_notes: String,
    plan_status: String,
    linked_trade_id: Option<Uuid>,
    created_at: DateTime<Utc>,
    filled_at: Option<DateTime<Utc>>,
}

impl From<Row> for TradePlan {
    fn from(r: Row) -> Self {
        TradePlan {
            id: r.id,
            user_id: r.user_id,
            account_id: r.account_id,
            symbol: r.symbol,
            asset_class: match r.asset_class.as_str() {
                "option" => AssetClass::Option,
                "future" => AssetClass::Future,
                "forex" => AssetClass::Forex,
                _ => AssetClass::Stock,
            },
            side: if r.side == "short" {
                TradeSide::Short
            } else {
                TradeSide::Long
            },
            intended_qty: r.intended_qty,
            intended_entry: r.intended_entry,
            stop_loss: r.stop_loss,
            initial_target: r.initial_target,
            setup_notes: r.setup_notes,
            plan_status: r.plan_status,
            linked_trade_id: r.linked_trade_id,
            created_at: r.created_at,
            filled_at: r.filled_at,
        }
    }
}
