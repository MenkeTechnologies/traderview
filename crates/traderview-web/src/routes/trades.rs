use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::routes::helpers::{ensure_account_owner, ensure_trade_owner};
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use rust_decimal::Decimal;
use serde::Deserialize;
use traderview_core::Trade;
use traderview_db::trades::TradeFilter;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/trades", get(list))
        .route("/trades/:id", get(get_one))
        .route("/trades/:id/risk", post(set_risk))
        .route("/trades/rollup", post(rollup))
}

#[derive(Deserialize)]
struct TradesQuery {
    account_id: Uuid,
    #[serde(flatten)]
    filter: TradeFilter,
}

async fn list(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<TradesQuery>,
) -> Result<Json<Vec<Trade>>, ApiError> {
    ensure_account_owner(&s, user.id, q.account_id).await?;
    Ok(Json(
        traderview_db::trades::list_for_account(&s.pool, q.account_id, &q.filter)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

async fn get_one(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Trade>, ApiError> {
    ensure_trade_owner(&s.pool, user.id, id).await?;
    let t = traderview_db::trades::get(&s.pool, id)
        .await
        .map_err(ApiError::Internal)?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(t))
}

#[derive(Deserialize)]
struct RiskBody {
    stop_loss: Option<Decimal>,
    risk_amount: Option<Decimal>,
    initial_target: Option<Decimal>,
}

async fn set_risk(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<RiskBody>,
) -> Result<Json<bool>, ApiError> {
    ensure_trade_owner(&s.pool, user.id, id).await?;
    traderview_db::trades::set_risk_fields(
        &s.pool,
        id,
        body.stop_loss,
        body.risk_amount,
        body.initial_target,
    )
    .await
    .map_err(ApiError::Internal)?;
    Ok(Json(true))
}

#[derive(Deserialize)]
struct RollupQuery {
    account_id: Uuid,
}

async fn rollup(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<RollupQuery>,
) -> Result<Json<usize>, ApiError> {
    ensure_account_owner(&s, user.id, q.account_id).await?;
    Ok(Json(
        traderview_db::trades::rollup_account(&s.pool, q.account_id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}
