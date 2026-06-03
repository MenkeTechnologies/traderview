use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::routes::helpers::{ensure_account_owner, ensure_trade_owner};
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::routing::{get, patch};
use axum::{Json, Router};
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::Deserialize;
use traderview_core::{AssetClass, Execution, OptionType, Side};
use traderview_import::ParsedExecution;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/executions", get(list).post(create))
        .route("/executions/:id", patch(update).delete(delete_one))
        .route(
            "/trades/:id/executions",
            get(list_for_trade).post(add_to_trade),
        )
}

#[derive(Deserialize)]
struct ListQuery {
    account_id: Uuid,
}

async fn list(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<ListQuery>,
) -> Result<Json<Vec<Execution>>, ApiError> {
    ensure_account_owner(&s, user.id, q.account_id).await?;
    Ok(Json(
        traderview_db::executions::list_for_account(&s.pool, q.account_id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

async fn list_for_trade(
    State(s): State<AppState>,
    user: AuthUser,
    Path(trade_id): Path<Uuid>,
) -> Result<Json<Vec<Execution>>, ApiError> {
    ensure_trade_owner(&s.pool, user.id, trade_id).await?;
    Ok(Json(
        traderview_db::executions::list_for_trade(&s.pool, trade_id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

#[derive(Deserialize)]
struct ManualBody {
    account_id: Uuid,
    symbol: String,
    side: Side,
    qty: Decimal,
    price: Decimal,
    #[serde(default)]
    fee: Decimal,
    #[serde(default)]
    commission: Decimal,
    executed_at: DateTime<Utc>,
    #[serde(default)]
    asset_class: AssetClass,
    #[serde(default)]
    option_type: Option<OptionType>,
    #[serde(default)]
    strike: Option<Decimal>,
    #[serde(default)]
    expiration: Option<NaiveDate>,
    #[serde(default)]
    multiplier: Option<Decimal>,
}

async fn create(
    State(s): State<AppState>,
    user: AuthUser,
    Json(body): Json<ManualBody>,
) -> Result<Json<Uuid>, ApiError> {
    ensure_account_owner(&s, user.id, body.account_id).await?;
    let multiplier = body.multiplier.unwrap_or_else(|| match body.asset_class {
        AssetClass::Option => Decimal::from(100),
        _ => Decimal::ONE,
    });
    let p = ParsedExecution {
        symbol: body.symbol,
        side: body.side,
        qty: body.qty,
        price: body.price,
        fee: body.fee,
        commission: body.commission,
        executed_at: body.executed_at,
        broker_order_id: None,
        raw: serde_json::json!({"source": "manual"}),
        asset_class: body.asset_class,
        option_type: body.option_type,
        strike: body.strike,
        expiration: body.expiration,
        multiplier,
        tick_size: None,
        tick_value: None,
        base_ccy: None,
        quote_ccy: None,
        pip_size: None,
    };
    let id = traderview_db::executions::insert_manual(&s.pool, body.account_id, &p)
        .await
        .map_err(ApiError::Internal)?;
    // Re-derive trades for this account.
    let _ = traderview_db::trades::rollup_account(&s.pool, body.account_id)
        .await
        .map_err(ApiError::Internal)?;
    Ok(Json(id))
}

async fn delete_one(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<bool>, ApiError> {
    let acct = traderview_db::executions::account_for(&s.pool, id)
        .await
        .map_err(ApiError::Internal)?
        .ok_or(ApiError::NotFound)?;
    ensure_account_owner(&s, user.id, acct).await?;
    let r = traderview_db::executions::delete(&s.pool, id)
        .await
        .map_err(ApiError::Internal)?;
    let _ = traderview_db::trades::rollup_account(&s.pool, acct)
        .await
        .map_err(ApiError::Internal)?;
    Ok(Json(r))
}

#[derive(Deserialize)]
struct UpdateBody {
    side: Option<Side>,
    qty: Option<rust_decimal::Decimal>,
    price: Option<rust_decimal::Decimal>,
    fee: Option<rust_decimal::Decimal>,
    executed_at: Option<DateTime<Utc>>,
}

async fn update(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateBody>,
) -> Result<Json<bool>, ApiError> {
    let acct = traderview_db::executions::account_for(&s.pool, id)
        .await
        .map_err(ApiError::Internal)?
        .ok_or(ApiError::NotFound)?;
    ensure_account_owner(&s, user.id, acct).await?;
    let r = traderview_db::executions::update(
        &s.pool,
        id,
        body.side,
        body.qty,
        body.price,
        body.fee,
        body.executed_at,
    )
    .await
    .map_err(ApiError::Internal)?;
    let _ = traderview_db::trades::rollup_account(&s.pool, acct)
        .await
        .map_err(ApiError::Internal)?;
    Ok(Json(r))
}

/// "Add execution to this trade" — for the Manual Execution Editor's
/// `+ Add New` button. We don't pin to one trade; we insert as a fresh
/// execution on the same account and re-roll, which will naturally fold it
/// into whatever trade the FIFO lands on.
#[derive(Deserialize)]
struct AddBody {
    symbol: Option<String>,
    side: Side,
    qty: Decimal,
    price: Decimal,
    #[serde(default)]
    fee: Decimal,
    #[serde(default)]
    commission: Decimal,
    executed_at: DateTime<Utc>,
    #[serde(default)]
    asset_class: AssetClass,
    #[serde(default)]
    option_type: Option<OptionType>,
    #[serde(default)]
    strike: Option<Decimal>,
    #[serde(default)]
    expiration: Option<NaiveDate>,
    #[serde(default)]
    multiplier: Option<Decimal>,
}

async fn add_to_trade(
    State(s): State<AppState>,
    user: AuthUser,
    Path(trade_id): Path<Uuid>,
    Json(body): Json<AddBody>,
) -> Result<Json<Uuid>, ApiError> {
    ensure_trade_owner(&s.pool, user.id, trade_id).await?;
    // Get the trade's account + symbol so the new execution defaults match.
    let trade = traderview_db::trades::get(&s.pool, trade_id)
        .await
        .map_err(ApiError::Internal)?
        .ok_or(ApiError::NotFound)?;
    let symbol = body.symbol.unwrap_or(trade.symbol);
    let multiplier = body.multiplier.unwrap_or(trade.multiplier);
    let p = ParsedExecution {
        symbol,
        side: body.side,
        qty: body.qty,
        price: body.price,
        fee: body.fee,
        commission: body.commission,
        executed_at: body.executed_at,
        broker_order_id: None,
        raw: serde_json::json!({"source": "manual-add"}),
        asset_class: body.asset_class,
        option_type: body.option_type,
        strike: body.strike,
        expiration: body.expiration,
        multiplier,
        tick_size: None,
        tick_value: None,
        base_ccy: None,
        quote_ccy: None,
        pip_size: None,
    };
    let id = traderview_db::executions::insert_manual(&s.pool, trade.account_id, &p)
        .await
        .map_err(ApiError::Internal)?;
    let _ = traderview_db::trades::rollup_account(&s.pool, trade.account_id)
        .await
        .map_err(ApiError::Internal)?;
    Ok(Json(id))
}
