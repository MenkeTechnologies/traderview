use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use traderview_core::{AssetClass, Execution, OptionType, Side};
use traderview_import::ParsedExecution;
use uuid::Uuid;

pub async fn list_for_account(pool: &PgPool, account_id: Uuid) -> anyhow::Result<Vec<Execution>> {
    let rows: Vec<ExecRow> = sqlx::query_as(
        "SELECT id, account_id, symbol, side::text, qty, price, fee, commissions AS commission,
                executed_at, broker_order_id,
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
                e.commissions AS commission,
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
    let r = build_insert_parsed(account_id, import_id, p)
        .execute(pool)
        .await?;
    Ok(r.rows_affected() > 0)
}

/// Transactional variant — used by the import endpoint so a mid-file error
/// rolls back the entire batch instead of leaving stranded rows.
pub async fn insert_parsed_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    account_id: Uuid,
    import_id: Uuid,
    p: &ParsedExecution,
) -> anyhow::Result<bool> {
    let r = build_insert_parsed(account_id, import_id, p)
        .execute(&mut **tx)
        .await?;
    Ok(r.rows_affected() > 0)
}

fn build_insert_parsed<'q>(
    account_id: Uuid,
    import_id: Uuid,
    p: &'q ParsedExecution,
) -> sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments> {
    sqlx::query(
        "INSERT INTO executions (
            account_id, symbol, side, qty, price, fee, commissions, executed_at,
            broker_order_id, raw, import_id,
            asset_class, option_type, strike, expiration, multiplier,
            tick_size, tick_value, base_ccy, quote_ccy, pip_size
         ) VALUES (
            $1, $2, $3::side_t, $4, $5, $6, $7, $8,
            $9, $10, $11,
            $12::asset_class_t, $13::option_type_t, $14, $15, $16,
            $17, $18, $19, $20, $21
         )
         ON CONFLICT DO NOTHING",
    )
    .bind(account_id)
    .bind(&p.symbol)
    .bind(side_to_pg(p.side))
    .bind(p.qty)
    .bind(p.price)
    .bind(p.fee)
    .bind(p.commission)
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
}

pub async fn insert_manual(
    pool: &PgPool,
    account_id: Uuid,
    p: &ParsedExecution,
) -> anyhow::Result<Uuid> {
    // Idempotent on the executions_dedupe_idx UNIQUE
    // (account_id, broker_order_id, executed_at, symbol, side, qty, price)
    // — defense against WS reconnect replays and against ANY pump that
    // emits both partial and final fill events. The dedupe index is
    // partial (broker_order_id IS NOT NULL), so rows without a
    // broker_order_id still insert normally — those use import_id-based
    // dedupe at the importer layer.
    //
    // RETURNING id is wrapped in DISTINCT-ON-conflict semantics: if the
    // INSERT was a no-op (conflict), we fall back to SELECTing the
    // existing row's id so callers keep getting a real Uuid.
    let inserted: Option<(Uuid,)> = sqlx::query_as(
        "INSERT INTO executions (
            account_id, symbol, side, qty, price, fee, commissions, executed_at,
            broker_order_id, raw, asset_class, option_type, strike, expiration, multiplier,
            tick_size, tick_value, base_ccy, quote_ccy, pip_size
         ) VALUES (
            $1, $2, $3::side_t, $4, $5, $6, $7, $8,
            $9, $10, $11::asset_class_t, $12::option_type_t, $13, $14, $15,
            $16, $17, $18, $19, $20
         )
         ON CONFLICT (account_id, broker_order_id, executed_at, symbol, side, qty, price)
            WHERE broker_order_id IS NOT NULL DO NOTHING
         RETURNING id",
    )
    .bind(account_id)
    .bind(&p.symbol)
    .bind(side_to_pg(p.side))
    .bind(p.qty)
    .bind(p.price)
    .bind(p.fee)
    .bind(p.commission)
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
    .fetch_optional(pool)
    .await?;
    if let Some((id,)) = inserted {
        return Ok(id);
    }
    // Conflict path: row already exists. Look it up by the dedupe
    // key and return its id so the caller stays Result<Uuid>.
    let (existing,): (Uuid,) = sqlx::query_as(
        "SELECT id FROM executions
          WHERE account_id = $1 AND broker_order_id = $9
            AND executed_at = $8 AND symbol = $2 AND side = $3::side_t
            AND qty = $4 AND price = $5
          LIMIT 1",
    )
    .bind(account_id)
    .bind(&p.symbol)
    .bind(side_to_pg(p.side))
    .bind(p.qty)
    .bind(p.price)
    .bind(p.fee)
    .bind(p.commission)
    .bind(p.executed_at)
    .bind(p.broker_order_id.as_deref())
    .fetch_one(pool)
    .await?;
    Ok(existing)
}

pub async fn delete(pool: &PgPool, execution_id: Uuid) -> anyhow::Result<bool> {
    let r = sqlx::query("DELETE FROM executions WHERE id = $1")
        .bind(execution_id)
        .execute(pool)
        .await?;
    Ok(r.rows_affected() > 0)
}

/// Edit any subset of mutable fields. None = leave unchanged.
pub async fn update(
    pool: &PgPool,
    id: Uuid,
    side: Option<Side>,
    qty: Option<Decimal>,
    price: Option<Decimal>,
    fee: Option<Decimal>,
    executed_at: Option<DateTime<Utc>>,
) -> anyhow::Result<bool> {
    let r = sqlx::query(
        "UPDATE executions SET
            side = COALESCE($2::side_t, side),
            qty  = COALESCE($3, qty),
            price = COALESCE($4, price),
            fee = COALESCE($5, fee),
            executed_at = COALESCE($6, executed_at)
          WHERE id = $1",
    )
    .bind(id)
    .bind(side.map(side_to_pg))
    .bind(qty)
    .bind(price)
    .bind(fee)
    .bind(executed_at)
    .execute(pool)
    .await?;
    Ok(r.rows_affected() > 0)
}

/// Look up the account that owns an execution. Returns None if not found.
pub async fn account_for(pool: &PgPool, execution_id: Uuid) -> anyhow::Result<Option<Uuid>> {
    let row: Option<(Uuid,)> = sqlx::query_as("SELECT account_id FROM executions WHERE id = $1")
        .bind(execution_id)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(|(a,)| a))
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
    pub commission: Decimal,
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
        let side = match r.side.as_str() {
            "buy" => Side::Buy,
            "sell" => Side::Sell,
            "short" => Side::Short,
            "cover" => Side::Cover,
            other => {
                // The `side_t` ENUM (migration 0001) only accepts the
                // four values above, so this branch is unreachable for
                // rows the DB inserted itself. If we ever see it, schema
                // drift or a hand-edit corrupted the row — log loudly
                // instead of silently relabelling shorts as longs. We
                // still fall through to a value because `From` cannot
                // return Result; the error log surfaces in monitoring
                // before P&L gets corrupted.
                tracing::error!(
                    execution_id = %r.id, account_id = %r.account_id,
                    symbol = %r.symbol, side = other,
                    "executions: unknown side string; defaulting to buy (CORRUPTION SIGNAL)"
                );
                Side::Buy
            }
        };
        Execution {
            id: r.id,
            account_id: r.account_id,
            symbol: r.symbol,
            side,
            qty: r.qty,
            price: r.price,
            fee: r.fee,
            commission: r.commission,
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use serde_json::json;

    // ===========================================================================
    // side_to_pg — every Side variant must produce a stable PG enum string
    // ===========================================================================

    #[test]
    fn side_to_pg_maps_all_four_variants() {
        assert_eq!(side_to_pg(Side::Buy), "buy");
        assert_eq!(side_to_pg(Side::Sell), "sell");
        assert_eq!(side_to_pg(Side::Short), "short");
        assert_eq!(side_to_pg(Side::Cover), "cover");
    }

    #[test]
    fn side_to_pg_output_is_lowercase_matching_pg_enum() {
        // PG enum side_t is lowercase by convention — verify no caps slip in.
        for s in [Side::Buy, Side::Sell, Side::Short, Side::Cover] {
            let v = side_to_pg(s);
            assert!(v.chars().all(|c| c.is_ascii_lowercase()), "{:?} → {}", s, v);
        }
    }

    // ===========================================================================
    // ac_to_pg — every AssetClass variant
    // ===========================================================================

    #[test]
    fn ac_to_pg_maps_all_four_classes() {
        assert_eq!(ac_to_pg(AssetClass::Stock), "stock");
        assert_eq!(ac_to_pg(AssetClass::Option), "option");
        assert_eq!(ac_to_pg(AssetClass::Future), "future");
        assert_eq!(ac_to_pg(AssetClass::Forex), "forex");
    }

    #[test]
    fn ac_to_pg_default_asset_class_maps_to_stock() {
        // Default-derived AssetClass should still map cleanly.
        let a: AssetClass = Default::default();
        assert_eq!(ac_to_pg(a), "stock");
    }

    // ===========================================================================
    // option_type_to_pg
    // ===========================================================================

    #[test]
    fn option_type_to_pg_maps_call_and_put() {
        assert_eq!(option_type_to_pg(OptionType::Call), "call");
        assert_eq!(option_type_to_pg(OptionType::Put), "put");
    }

    // ===========================================================================
    // ExecRow → Execution conversion — text enums map back, fallback to Buy
    // ===========================================================================

    fn row_with_side_and_class(side: &str, ac: &str, ot: Option<&str>) -> ExecRow {
        ExecRow {
            id: Uuid::nil(),
            account_id: Uuid::nil(),
            symbol: "TEST".into(),
            side: side.into(),
            qty: Decimal::from(10),
            price: Decimal::from(100),
            fee: Decimal::ZERO,
            commission: Decimal::ZERO,
            executed_at: Utc.with_ymd_and_hms(2026, 1, 1, 9, 30, 0).unwrap(),
            broker_order_id: None,
            raw: json!({}),
            import_id: None,
            asset_class: ac.into(),
            option_type: ot.map(|s| s.to_string()),
            strike: None,
            expiration: None,
            multiplier: Decimal::ONE,
            tick_size: None,
            tick_value: None,
            base_ccy: None,
            quote_ccy: None,
            pip_size: None,
        }
    }

    #[test]
    fn exec_row_parses_each_side_string_to_enum() {
        for (s, expected) in [
            ("buy", Side::Buy),
            ("sell", Side::Sell),
            ("short", Side::Short),
            ("cover", Side::Cover),
        ] {
            let e: Execution = row_with_side_and_class(s, "stock", None).into();
            assert_eq!(
                e.side, expected,
                "side string {} should map to {:?}",
                s, expected
            );
        }
    }

    #[test]
    fn exec_row_unknown_side_falls_back_to_buy() {
        // Defensive: corrupt PG data must not panic — defaults to Buy.
        let e: Execution = row_with_side_and_class("garbage", "stock", None).into();
        assert_eq!(e.side, Side::Buy);
    }

    #[test]
    fn exec_row_parses_each_asset_class() {
        for (s, expected) in [
            ("stock", AssetClass::Stock),
            ("option", AssetClass::Option),
            ("future", AssetClass::Future),
            ("forex", AssetClass::Forex),
        ] {
            let e: Execution = row_with_side_and_class("buy", s, None).into();
            assert_eq!(e.asset_class, expected);
        }
    }

    #[test]
    fn exec_row_unknown_asset_class_falls_back_to_stock() {
        let e: Execution = row_with_side_and_class("buy", "crypto", None).into();
        assert_eq!(e.asset_class, AssetClass::Stock);
    }

    #[test]
    fn exec_row_option_type_parses_call_and_put_otherwise_none() {
        let e_call: Execution = row_with_side_and_class("buy", "option", Some("call")).into();
        assert_eq!(e_call.option_type, Some(OptionType::Call));
        let e_put: Execution = row_with_side_and_class("buy", "option", Some("put")).into();
        assert_eq!(e_put.option_type, Some(OptionType::Put));
        let e_none: Execution = row_with_side_and_class("buy", "option", None).into();
        assert_eq!(e_none.option_type, None);
        let e_garbage: Execution = row_with_side_and_class("buy", "option", Some("warrant")).into();
        assert_eq!(e_garbage.option_type, None);
    }
}
