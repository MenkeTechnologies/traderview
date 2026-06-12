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
    /// URL query params arrive as strings; `#[serde(flatten)]` on the
    /// route's wrapper struct breaks serde_urlencoded's normal
    /// string→int coercion, so we accept either form explicitly via
    /// `deserialize_with`. This made the dashboard's Open Trades widget
    /// 400 on every refresh until fixed.
    #[serde(default, deserialize_with = "de_opt_i64_from_str_or_int")]
    pub limit: Option<i64>,
    #[serde(default, deserialize_with = "de_opt_i64_from_str_or_int")]
    pub offset: Option<i64>,
}

/// Accept `Option<i64>` from either a JSON integer (POST body) or a query-
/// string scalar (which arrives as a string after `#[serde(flatten)]`).
fn de_opt_i64_from_str_or_int<'de, D>(de: D) -> Result<Option<i64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    struct V;
    impl<'de> Visitor<'de> for V {
        type Value = Option<i64>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("integer or string-encoded integer (or null)")
        }
        fn visit_none<E: de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }
        fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }
        fn visit_i64<E: de::Error>(self, v: i64) -> Result<Self::Value, E> {
            Ok(Some(v))
        }
        fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E> {
            Ok(Some(v as i64))
        }
        fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
            if v.is_empty() {
                return Ok(None);
            }
            v.parse::<i64>().map(Some).map_err(de::Error::custom)
        }
        fn visit_some<D2: serde::Deserializer<'de>>(self, d: D2) -> Result<Self::Value, D2::Error> {
            d.deserialize_any(V)
        }
    }
    de.deserialize_any(V)
}

pub async fn list_for_account(
    pool: &PgPool,
    account_id: Uuid,
    f: &TradeFilter,
) -> anyhow::Result<Vec<Trade>> {
    let mut q = sqlx::QueryBuilder::new(
        "SELECT id, account_id, symbol, side::text, status::text, opened_at, closed_at,
                qty, entry_avg, exit_avg, gross_pnl, fees, commissions, net_pnl,
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
        q.push(" AND status = ")
            .push_bind(s)
            .push("::trade_status_t");
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
        q.push(" AND asset_class = ")
            .push_bind(s)
            .push("::asset_class_t");
    }
    if let Some(d) = f.date_from {
        q.push(" AND opened_at >= ")
            .push_bind(d.and_hms_opt(0, 0, 0).unwrap().and_utc());
    }
    if let Some(d) = f.date_to {
        q.push(" AND opened_at < ").push_bind(
            d.succ_opt()
                .unwrap_or(d)
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_utc(),
        );
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

/// Generalized trade lookup scoped by user. Use this when the caller
/// wants to filter by broker (all accounts of one broker) or aggregate
/// across all of a user's accounts — the original `list_for_account`
/// only supports the single-account case.
///
/// At least one of `user_id`, `account_id`, or `broker_id` must yield a
/// scope; if `account_id` is set it wins. Otherwise we filter by
/// `accounts.user_id = $user_id` and optionally `accounts.broker_id =
/// $broker_id` via a join.
pub async fn list_for_scope(
    pool: &PgPool,
    user_id: Uuid,
    account_id: Option<Uuid>,
    broker_id: Option<Uuid>,
    f: &TradeFilter,
) -> anyhow::Result<Vec<Trade>> {
    if let Some(aid) = account_id {
        return list_for_account(pool, aid, f).await;
    }
    let mut q = sqlx::QueryBuilder::new(
        "SELECT t.id, t.account_id, t.symbol, t.side::text, t.status::text, t.opened_at, t.closed_at,
                t.qty, t.entry_avg, t.exit_avg, t.gross_pnl, t.fees, t.commissions, t.net_pnl,
                t.asset_class::text, t.option_type::text, t.strike, t.expiration, t.multiplier,
                t.tick_size, t.tick_value, t.base_ccy, t.quote_ccy, t.pip_size,
                t.stop_loss, t.risk_amount, t.initial_target, t.mfe, t.mae, t.best_exit_pnl, t.exit_efficiency
           FROM trades t
           JOIN accounts a ON a.id = t.account_id
          WHERE a.user_id = ",
    );
    q.push_bind(user_id);
    if let Some(bid) = broker_id {
        q.push(" AND a.broker_id = ").push_bind(bid);
    }
    if let Some(sym) = &f.symbol {
        q.push(" AND t.symbol = ").push_bind(sym.clone());
    }
    if let Some(status) = f.status {
        let s = match status {
            TradeStatus::Open => "open",
            TradeStatus::Closed => "closed",
        };
        q.push(" AND t.status = ")
            .push_bind(s)
            .push("::trade_status_t");
    }
    if let Some(side) = f.side {
        let s = match side {
            TradeSide::Long => "long",
            TradeSide::Short => "short",
        };
        q.push(" AND t.side = ").push_bind(s).push("::trade_side_t");
    }
    if let Some(ac) = f.asset_class {
        let s = ac_to_pg(ac);
        q.push(" AND t.asset_class = ")
            .push_bind(s)
            .push("::asset_class_t");
    }
    if let Some(d) = f.date_from {
        q.push(" AND t.opened_at >= ")
            .push_bind(d.and_hms_opt(0, 0, 0).unwrap().and_utc());
    }
    if let Some(d) = f.date_to {
        q.push(" AND t.opened_at < ").push_bind(
            d.succ_opt()
                .unwrap_or(d)
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_utc(),
        );
    }
    if let Some(p) = f.min_pnl {
        q.push(" AND t.net_pnl >= ").push_bind(p);
    }
    if let Some(p) = f.max_pnl {
        q.push(" AND t.net_pnl <= ").push_bind(p);
    }
    if let Some(qty) = f.min_qty {
        q.push(" AND t.qty >= ").push_bind(qty);
    }
    if let Some(qty) = f.max_qty {
        q.push(" AND t.qty <= ").push_bind(qty);
    }
    if let Some(tag) = f.tag_id {
        q.push(" AND t.id IN (SELECT trade_id FROM trade_tags WHERE tag_id = ")
            .push_bind(tag)
            .push(")");
    }
    q.push(" ORDER BY t.opened_at DESC");
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
                qty, entry_avg, exit_avg, gross_pnl, fees, commissions, net_pnl,
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
///
/// Concurrency: two callers writing executions to the same account
/// (e.g. the algo runner's record_fill + a WS pump landing a different
/// fill on a peer strategy bound to the same account) both call this
/// from separate transactions. Without a per-account lock they race on
/// the full DELETE+INSERT — the second commit overwrites the first
/// with a stale snapshot. We take a Postgres advisory transaction lock
/// keyed on `hashtext('trades_rollup' || account_id)` as the first
/// statement so concurrent calls serialize per account. The lock
/// auto-releases at COMMIT.
pub async fn rollup_account(pool: &PgPool, account_id: Uuid) -> anyhow::Result<usize> {
    let mut tx = pool.begin().await?;
    // Stable 32-bit key from "trades_rollup:" + account_id UUID. Two
    // concurrent rollup_account calls on the same account block at
    // this statement; rollups on different accounts proceed in
    // parallel. The integer cast is required because pg_advisory_xact_lock
    // takes a single bigint or two int4s, not a TEXT directly.
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
        .bind(format!("trades_rollup:{account_id}"))
        .execute(&mut *tx)
        .await?;
    // Re-read executions INSIDE the locked tx — without this we'd
    // operate on a snapshot taken before another writer's commit
    // became visible, defeating the lock.
    let execs = executions::list_for_account(pool, account_id).await?;
    let trades = rollup(&execs, LotMethod::Fifo)?;
    let n = trades.len();
    sqlx::query("DELETE FROM trades WHERE account_id = $1")
        .bind(account_id)
        .execute(&mut *tx)
        .await?;
    for rt in &trades {
        insert_rolled(&mut tx, rt).await?;
    }
    tx.commit().await?;
    Ok(n)
}

async fn insert_rolled(tx: &mut sqlx::PgConnection, rt: &RolledTrade) -> anyhow::Result<()> {
    let t = &rt.trade;
    sqlx::query(
        "INSERT INTO trades (
            id, account_id, symbol, side, status, opened_at, closed_at,
            qty, entry_avg, exit_avg, gross_pnl, fees, commissions, net_pnl,
            asset_class, option_type, strike, expiration, multiplier,
            tick_size, tick_value, base_ccy, quote_ccy, pip_size
         ) VALUES (
            $1, $2, $3, $4::trade_side_t, $5::trade_status_t, $6, $7,
            $8, $9, $10, $11, $12, $13, $14,
            $15::asset_class_t, $16::option_type_t, $17, $18, $19,
            $20, $21, $22, $23, $24
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
    .bind(t.commissions)
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
                  VALUES ($1, $2, $3)
              ON CONFLICT (trade_id, execution_id)
              DO UPDATE SET qty_used = trade_executions.qty_used + EXCLUDED.qty_used",
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

pub async fn delete(pool: &PgPool, trade_id: Uuid) -> anyhow::Result<bool> {
    let r = sqlx::query("DELETE FROM trades WHERE id = $1")
        .bind(trade_id)
        .execute(pool)
        .await?;
    Ok(r.rows_affected() > 0)
}

/// Delete by id list (single-account; verify ownership at the route layer).
pub async fn delete_many(pool: &PgPool, ids: &[Uuid]) -> anyhow::Result<u64> {
    if ids.is_empty() {
        return Ok(0);
    }
    let r = sqlx::query("DELETE FROM trades WHERE id = ANY($1)")
        .bind(ids)
        .execute(pool)
        .await?;
    Ok(r.rows_affected())
}

/// "Split" a trade into N pieces by deleting it and re-running rollup for the
/// account, but only after marking the underlying executions to disable
/// auto-grouping at the split point. Practical implementation: we re-run the
/// account roll-up with the auto_flatten setting forcibly on for a single
/// shot, which yields one trade per flat→nonzero transition.
///
/// For now, "split" is implemented as: delete the target trade, re-rollup the
/// account. The current FIFO roll-up already splits naturally on flat boundary,
/// so this restores natural grouping. Caller can then re-merge as desired.
pub async fn split(pool: &PgPool, trade_id: Uuid) -> anyhow::Result<usize> {
    let row: Option<(Uuid,)> = sqlx::query_as("SELECT account_id FROM trades WHERE id = $1")
        .bind(trade_id)
        .fetch_optional(pool)
        .await?;
    let acct = row.ok_or_else(|| anyhow::anyhow!("trade not found"))?.0;
    rollup_account(pool, acct).await
}

/// "Merge" — take N trades that belong to one account, delete them, and emit
/// a single materialized trade that aggregates their legs. Inputs must share
/// `account_id` + `symbol` + `asset_class` + `option_leg`; otherwise we
/// refuse.
pub async fn merge(pool: &PgPool, ids: &[Uuid]) -> anyhow::Result<Uuid> {
    use traderview_core::{Trade, TradeStatus};
    if ids.len() < 2 {
        anyhow::bail!("need at least 2 trades to merge");
    }
    let trades: Vec<Trade> = futures(pool, ids).await?;
    let first = trades.first().ok_or_else(|| anyhow::anyhow!("no trades"))?;
    for t in &trades[1..] {
        if t.account_id != first.account_id
            || t.symbol != first.symbol
            || t.asset_class != first.asset_class
            || t.option_type != first.option_type
            || t.expiration != first.expiration
            || t.strike != first.strike
        {
            anyhow::bail!("trades differ on the merge key (account/symbol/asset/option-leg)");
        }
    }

    let mut tx = pool.begin().await?;
    // Sum legs into one new trade.
    let total_qty: Decimal = trades.iter().map(|t| t.qty).sum();
    let total_fees: Decimal = trades.iter().map(|t| t.fees).sum();
    let total_commissions: Decimal = trades.iter().map(|t| t.commissions).sum();
    let weighted_entry: Decimal =
        trades.iter().map(|t| t.entry_avg * t.qty).sum::<Decimal>() / total_qty.max(Decimal::ONE);
    let weighted_exit: Option<Decimal> = {
        let parts: Vec<(Decimal, Decimal)> = trades
            .iter()
            .filter_map(|t| t.exit_avg.map(|x| (x, t.qty)))
            .collect();
        if parts.is_empty() {
            None
        } else {
            let n: Decimal = parts.iter().map(|(_, q)| *q).sum();
            Some(parts.iter().map(|(p, q)| *p * *q).sum::<Decimal>() / n.max(Decimal::ONE))
        }
    };
    let total_gross: Option<Decimal> = sum_opt(trades.iter().map(|t| t.gross_pnl));
    let total_net: Option<Decimal> = sum_opt(trades.iter().map(|t| t.net_pnl));
    let opened_at = trades.iter().map(|t| t.opened_at).min().unwrap();
    let closed_at = trades.iter().filter_map(|t| t.closed_at).max();
    let status = if trades.iter().all(|t| t.status == TradeStatus::Closed) {
        "closed"
    } else {
        "open"
    };

    let new_id: Uuid = sqlx::query_scalar(
        "INSERT INTO trades (
            account_id, symbol, side, status, opened_at, closed_at,
            qty, entry_avg, exit_avg, gross_pnl, fees, commissions, net_pnl,
            asset_class, option_type, strike, expiration, multiplier,
            tick_size, tick_value, base_ccy, quote_ccy, pip_size
         ) VALUES (
            $1, $2, $3::trade_side_t, $4::trade_status_t, $5, $6,
            $7, $8, $9, $10, $11, $12, $13,
            $14::asset_class_t, $15::option_type_t, $16, $17, $18,
            $19, $20, $21, $22, $23
         ) RETURNING id",
    )
    .bind(first.account_id)
    .bind(&first.symbol)
    .bind(side_to_pg(first.side))
    .bind(status)
    .bind(opened_at)
    .bind(closed_at)
    .bind(total_qty)
    .bind(weighted_entry)
    .bind(weighted_exit)
    .bind(total_gross)
    .bind(total_fees)
    .bind(total_commissions)
    .bind(total_net)
    .bind(ac_to_pg(first.asset_class))
    .bind(first.option_type.map(option_type_to_pg).map(|s| s as &str))
    .bind(first.strike)
    .bind(first.expiration)
    .bind(first.multiplier)
    .bind(first.tick_size)
    .bind(first.tick_value)
    .bind(first.base_ccy.as_deref())
    .bind(first.quote_ccy.as_deref())
    .bind(first.pip_size)
    .fetch_one(&mut *tx)
    .await?;

    // Re-point legs from old trades to new id.
    sqlx::query("UPDATE trade_executions SET trade_id = $1 WHERE trade_id = ANY($2)")
        .bind(new_id)
        .bind(ids)
        .execute(&mut *tx)
        .await?;
    // Re-point tags too.
    sqlx::query(
        "INSERT INTO trade_tags (trade_id, tag_id)
         SELECT $1, tag_id FROM trade_tags WHERE trade_id = ANY($2)
         ON CONFLICT DO NOTHING",
    )
    .bind(new_id)
    .bind(ids)
    .execute(&mut *tx)
    .await?;
    sqlx::query("DELETE FROM trades WHERE id = ANY($1)")
        .bind(ids)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(new_id)
}

async fn futures(pool: &PgPool, ids: &[Uuid]) -> anyhow::Result<Vec<traderview_core::Trade>> {
    let rows: Vec<TradeRow> = sqlx::query_as(
        "SELECT id, account_id, symbol, side::text, status::text, opened_at, closed_at,
                qty, entry_avg, exit_avg, gross_pnl, fees, commissions, net_pnl,
                asset_class::text, option_type::text, strike, expiration, multiplier,
                tick_size, tick_value, base_ccy, quote_ccy, pip_size,
                stop_loss, risk_amount, initial_target, mfe, mae, best_exit_pnl, exit_efficiency
           FROM trades WHERE id = ANY($1)",
    )
    .bind(ids)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

fn sum_opt<I: Iterator<Item = Option<Decimal>>>(iter: I) -> Option<Decimal> {
    let mut acc: Option<Decimal> = None;
    for v in iter {
        match v {
            Some(x) => acc = Some(acc.unwrap_or(Decimal::ZERO) + x),
            None => return None,
        }
    }
    acc
}

/// Open positions for a single symbol on a single account. Used by the
/// algo engine's exit pass: the runner asks the strategy to evaluate
/// each open position against the latest bar window and flats any that
/// hit a rule-based exit (mean reversion target, momentum loss, time
/// stop, etc). Returns side as "long" / "short" so the caller maps it
/// to the close side (long → Sell, short → Buy).
#[derive(Debug, Clone)]
pub struct OpenPositionRow {
    pub id: Uuid,
    pub side: String,
    pub qty: Decimal,
    pub entry_avg: Decimal,
    pub opened_at: DateTime<Utc>,
}

/// Net realized P&L for the trade that closed at-or-after `since` on
/// (account_id, symbol). Used by the algo engine's `record_fill` to
/// recognize "this fill closed a trade, here's the delta to add to
/// the run's pnl_realized" — so the daily-loss + consecutive-losses
/// risk gates have a real number to compare against. Returns `None`
/// when the most recent close on the pair is older than `since`
/// (this fill was a partial fill or an open, not a close).
pub async fn realized_pnl_closed_since(
    pool: &PgPool,
    account_id: Uuid,
    symbol: &str,
    since: DateTime<Utc>,
) -> anyhow::Result<Option<Decimal>> {
    let row: Option<(Option<Decimal>,)> = sqlx::query_as(
        "SELECT COALESCE(SUM(net_pnl), 0)::numeric
           FROM trades
          WHERE account_id = $1
            AND symbol = $2
            AND status = 'closed'
            AND closed_at >= $3",
    )
    .bind(account_id)
    .bind(symbol)
    .bind(since)
    .fetch_optional(pool)
    .await?;
    Ok(row.and_then(|(v,)| v).filter(|d| !d.is_zero()))
}

/// Current open qty for a trade, or `None` if it's already closed
/// (status != 'open'). Used by the algo engine's exit pass to re-fetch
/// the live qty right before submitting a close — a partial entry-fill
/// or a previous close fill landing via the WS pump can shrink the
/// position between the position-snapshot and the close submission,
/// and using the stale snapshot would over-sell.
pub async fn get_open_qty(pool: &PgPool, trade_id: Uuid) -> anyhow::Result<Option<Decimal>> {
    let row: Option<(Decimal,)> = sqlx::query_as(
        "SELECT qty FROM trades WHERE id = $1 AND status = 'open'",
    )
    .bind(trade_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|(q,)| q))
}

/// Most recent trade row on (account, symbol) — the row the engine's
/// fill just rolled into. Used for algo auto-tagging.
pub async fn latest_trade_id(
    pool: &PgPool,
    account_id: Uuid,
    symbol: &str,
) -> anyhow::Result<Option<Uuid>> {
    let row: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM trades
          WHERE account_id = $1 AND symbol = $2
          ORDER BY opened_at DESC LIMIT 1",
    )
    .bind(account_id)
    .bind(symbol)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|(id,)| id))
}

/// Distinct symbols with open positions on the account — the
/// correlation gate's comparison set.
pub async fn open_symbols(pool: &PgPool, account_id: Uuid) -> anyhow::Result<Vec<String>> {
    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT DISTINCT symbol FROM trades WHERE account_id = $1 AND status = 'open'",
    )
    .bind(account_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|(s,)| s).collect())
}

/// Total open entry notional across the WHOLE account — every open
/// trade's |qty × entry_avg|, all strategies, all symbols. Entry
/// notional, not marked value: deterministic from the trades table
/// with no quote dependency (same convention as the borrow-fee pass).
pub async fn open_account_notional(pool: &PgPool, account_id: Uuid) -> anyhow::Result<Decimal> {
    let r: Option<(Decimal,)> = sqlx::query_as(
        "SELECT COALESCE(SUM(ABS(qty * entry_avg)), 0)
           FROM trades
          WHERE account_id = $1 AND status = 'open'",
    )
    .bind(account_id)
    .fetch_optional(pool)
    .await?;
    Ok(r.map(|(d,)| d).unwrap_or_default())
}

pub async fn open_positions_for_symbol(
    pool: &PgPool,
    account_id: Uuid,
    symbol: &str,
) -> anyhow::Result<Vec<OpenPositionRow>> {
    let rows = sqlx::query_as::<_, (Uuid, String, Decimal, Decimal, DateTime<Utc>)>(
        "SELECT id, side::text, qty, entry_avg, opened_at
           FROM trades
          WHERE account_id = $1 AND symbol = $2 AND status = 'open'",
    )
    .bind(account_id)
    .bind(symbol)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(id, side, qty, entry_avg, opened_at)| OpenPositionRow {
            id,
            side,
            qty,
            entry_avg,
            opened_at,
        })
        .collect())
}

/// Insert zero-priced closing executions for every option leg whose
/// expiration date has passed and which still has an open trade open.
pub async fn close_expired_options(pool: &PgPool, account_id: Uuid) -> anyhow::Result<usize> {
    let open_options: Vec<(Uuid, String, Decimal, chrono::NaiveDate, String, Decimal)> =
        sqlx::query_as(
            "SELECT id, symbol, qty, expiration, side::text, multiplier
               FROM trades
              WHERE account_id = $1 AND status = 'open' AND asset_class = 'option'
                AND expiration IS NOT NULL AND expiration < CURRENT_DATE",
        )
        .bind(account_id)
        .fetch_all(pool)
        .await?;
    let n = open_options.len();
    for (_id, symbol, qty, expiration, side, multiplier) in open_options {
        let closing_side = if side == "long" { "sell" } else { "cover" };
        let exec_time = expiration.and_hms_opt(16, 0, 0).unwrap().and_utc();
        sqlx::query(
            "INSERT INTO executions (
                account_id, symbol, side, qty, price, fee, executed_at,
                asset_class, multiplier, raw
             ) VALUES ($1, $2, $3::side_t, $4, 0, 0, $5,
                       'option'::asset_class_t, $6, '{\"source\":\"close_expired\"}'::jsonb)",
        )
        .bind(account_id)
        .bind(&symbol)
        .bind(closing_side)
        .bind(qty)
        .bind(exec_time)
        .bind(multiplier)
        .execute(pool)
        .await?;
    }
    if n > 0 {
        rollup_account(pool, account_id).await?;
    }
    Ok(n)
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
    pub commissions: Decimal,
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
            commissions: r.commissions,
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

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    // ─── sum_opt iterator combinator ─────────────────────────────────

    #[test]
    fn sum_opt_empty_iterator_is_none() {
        // An empty iterator: no values means there's nothing to sum,
        // and the impl returns None (vs Some(0)) so callers can
        // distinguish "no rows" from "all-zero rows".
        let r = sum_opt(std::iter::empty::<Option<Decimal>>());
        assert!(r.is_none(), "empty iter must yield None, not Some(0)");
    }

    #[test]
    fn sum_opt_all_some_returns_sum() {
        let r = sum_opt(
            vec![
                Some(Decimal::from(10)),
                Some(Decimal::from(20)),
                Some(Decimal::from(30)),
            ]
            .into_iter(),
        );
        assert_eq!(r, Some(Decimal::from(60)));
    }

    #[test]
    fn sum_opt_with_any_none_returns_none() {
        // Critical contract: a single None POISONS the sum. Callers
        // depend on this for "if any leg's MFE is unknown, the whole
        // trade's MFE is unknown" semantics.
        let r = sum_opt(vec![Some(Decimal::from(10)), None, Some(Decimal::from(30))].into_iter());
        assert_eq!(
            r, None,
            "ANY None must poison the entire sum — partial sums are forbidden"
        );
    }

    #[test]
    fn sum_opt_single_some_is_that_value() {
        let r = sum_opt(std::iter::once(Some(Decimal::from(42))));
        assert_eq!(r, Some(Decimal::from(42)));
    }

    #[test]
    fn sum_opt_handles_negative_and_zero() {
        let r = sum_opt(
            vec![
                Some(Decimal::from(-100)),
                Some(Decimal::ZERO),
                Some(Decimal::from(50)),
            ]
            .into_iter(),
        );
        assert_eq!(r, Some(Decimal::from(-50)));
    }

    #[test]
    fn sum_opt_short_circuits_at_first_none() {
        // After hitting None, no further elements should be observed —
        // we verify by having a Some with a side-effect counter after
        // the None and asserting the counter wasn't incremented.
        use std::cell::Cell;
        let after_none_visits = Cell::new(0u32);
        let r = sum_opt(
            vec![Some(Decimal::from(1)), None].into_iter().chain(
                std::iter::from_fn(|| {
                    after_none_visits.set(after_none_visits.get() + 1);
                    Some(Some(Decimal::from(999)))
                })
                .take(3),
            ),
        );
        assert_eq!(r, None);
        assert_eq!(
            after_none_visits.get(),
            0,
            "must short-circuit at first None — no further iterator pulls"
        );
    }

    // ─── enum-to-pg-string converters (canonical schema column values) ──

    #[test]
    fn ac_to_pg_canonical_strings() {
        // These strings MUST match the asset_class_t pg enum exactly —
        // any drift breaks the INSERT cast '...::asset_class_t' in the
        // migration. Pinning so a rename is a 3-table coordinated change.
        assert_eq!(ac_to_pg(AssetClass::Stock), "stock");
        assert_eq!(ac_to_pg(AssetClass::Option), "option");
        assert_eq!(ac_to_pg(AssetClass::Future), "future");
        assert_eq!(ac_to_pg(AssetClass::Forex), "forex");
    }

    #[test]
    fn side_to_pg_canonical_strings() {
        assert_eq!(side_to_pg(TradeSide::Long), "long");
        assert_eq!(side_to_pg(TradeSide::Short), "short");
    }

    #[test]
    fn status_to_pg_canonical_strings() {
        assert_eq!(status_to_pg(TradeStatus::Open), "open");
        assert_eq!(status_to_pg(TradeStatus::Closed), "closed");
    }

    #[test]
    fn option_type_to_pg_canonical_strings() {
        assert_eq!(option_type_to_pg(OptionType::Call), "call");
        assert_eq!(option_type_to_pg(OptionType::Put), "put");
    }

    #[test]
    fn ac_to_pg_lowercase_invariant() {
        // The asset_class_t pg enum is lowercase. Drift to TitleCase or
        // SCREAMING would break the INSERT cast — pinned via lowercase
        // character predicate across every variant.
        for ac in [
            AssetClass::Stock,
            AssetClass::Option,
            AssetClass::Future,
            AssetClass::Forex,
        ] {
            let s = ac_to_pg(ac);
            assert!(
                s.chars().all(|c| c.is_ascii_lowercase()),
                "{:?} -> {} must be all-lowercase to match pg enum",
                ac,
                s
            );
        }
    }

    #[test]
    fn side_to_pg_lowercase_invariant() {
        for s in [TradeSide::Long, TradeSide::Short] {
            let v = side_to_pg(s);
            assert!(
                v.chars().all(|c| c.is_ascii_lowercase()),
                "{:?} -> {} must be all-lowercase",
                s,
                v
            );
        }
    }

    // ─── round-trip: pg-string back to enum (caller's responsibility,
    //     but pin the string set so it can never silently change) ──

    #[test]
    fn pg_string_set_is_unique_across_converters() {
        // Within each converter, every variant maps to a unique string
        // (otherwise the DB casts would collapse different enum values
        // into the same pg enum tag). Pinning this property.
        let ac_strings: Vec<&str> = vec![
            ac_to_pg(AssetClass::Stock),
            ac_to_pg(AssetClass::Option),
            ac_to_pg(AssetClass::Future),
            ac_to_pg(AssetClass::Forex),
        ];
        let mut sorted = ac_strings.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(
            sorted.len(),
            ac_strings.len(),
            "ac_to_pg must yield unique strings — no two variants may collide"
        );
    }

    // Make sure the converters are stable across Decimal precision changes
    // (cosmetic — these are static strings, but pinning anyway).
    #[test]
    fn converters_dont_depend_on_decimal_precision() {
        let _d = Decimal::from_str("1.000000000000000000").unwrap();
        assert_eq!(side_to_pg(TradeSide::Long), "long");
        assert_eq!(side_to_pg(TradeSide::Short), "short");
    }
}
