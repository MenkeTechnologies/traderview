use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::routes::helpers::{ensure_account_owner, ensure_trade_owner};
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use traderview_core::Trade;
use traderview_db::trades::TradeFilter;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/trades", get(list))
        .route("/trades/:id", get(get_one).delete(delete_one))
        .route("/trades/:id/risk", post(set_risk))
        .route("/trades/:id/split", post(split))
        .route("/trades/rollup", post(rollup))
        .route("/trades/merge", post(merge))
        .route("/trades/bulk", post(bulk))
        .route("/trades/close-expired-options", post(close_expired_options))
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

// ---- delete -----------------------------------------------------

async fn delete_one(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<bool>, ApiError> {
    ensure_trade_owner(&s.pool, user.id, id).await?;
    Ok(Json(
        traderview_db::trades::delete(&s.pool, id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

// ---- split / merge / close expired -----------------------------------------

async fn split(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<usize>, ApiError> {
    ensure_trade_owner(&s.pool, user.id, id).await?;
    Ok(Json(
        traderview_db::trades::split(&s.pool, id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

#[derive(Deserialize)]
struct MergeBody {
    trade_ids: Vec<Uuid>,
}

async fn merge(
    State(s): State<AppState>,
    user: AuthUser,
    Json(body): Json<MergeBody>,
) -> Result<Json<Uuid>, ApiError> {
    for id in &body.trade_ids {
        ensure_trade_owner(&s.pool, user.id, *id).await?;
    }
    Ok(Json(
        traderview_db::trades::merge(&s.pool, &body.trade_ids)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

async fn close_expired_options(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<RollupQuery>,
) -> Result<Json<usize>, ApiError> {
    ensure_account_owner(&s, user.id, q.account_id).await?;
    Ok(Json(
        traderview_db::trades::close_expired_options(&s.pool, q.account_id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

// ---- bulk actions -----------------------------------------------------

#[derive(Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
enum BulkAction {
    Delete,
    AddTag {
        tag_id: Uuid,
    },
    RemoveTag {
        tag_id: Uuid,
    },
    SetRisk {
        stop_loss: Option<Decimal>,
        risk_amount: Option<Decimal>,
        initial_target: Option<Decimal>,
    },
    Merge,
    Split,
    Share {
        is_public: bool,
    },
}

#[derive(Deserialize)]
struct BulkBody {
    trade_ids: Vec<Uuid>,
    #[serde(flatten)]
    action: BulkAction,
}

#[derive(Serialize)]
struct BulkResult {
    affected: u64,
    note: Option<String>,
}

async fn bulk(
    State(s): State<AppState>,
    user: AuthUser,
    Json(body): Json<BulkBody>,
) -> Result<Json<BulkResult>, ApiError> {
    for id in &body.trade_ids {
        ensure_trade_owner(&s.pool, user.id, *id).await?;
    }
    let n = match body.action {
        BulkAction::Delete => traderview_db::trades::delete_many(&s.pool, &body.trade_ids)
            .await
            .map_err(ApiError::Internal)?,
        BulkAction::AddTag { tag_id } => {
            for id in &body.trade_ids {
                traderview_db::tags::attach_to_trade(&s.pool, *id, tag_id)
                    .await
                    .map_err(ApiError::Internal)?;
            }
            body.trade_ids.len() as u64
        }
        BulkAction::RemoveTag { tag_id } => {
            for id in &body.trade_ids {
                traderview_db::tags::detach_from_trade(&s.pool, *id, tag_id)
                    .await
                    .map_err(ApiError::Internal)?;
            }
            body.trade_ids.len() as u64
        }
        BulkAction::SetRisk {
            stop_loss,
            risk_amount,
            initial_target,
        } => {
            for id in &body.trade_ids {
                traderview_db::trades::set_risk_fields(
                    &s.pool,
                    *id,
                    stop_loss,
                    risk_amount,
                    initial_target,
                )
                .await
                .map_err(ApiError::Internal)?;
            }
            body.trade_ids.len() as u64
        }
        BulkAction::Merge => {
            traderview_db::trades::merge(&s.pool, &body.trade_ids)
                .await
                .map_err(ApiError::Internal)?;
            body.trade_ids.len() as u64
        }
        BulkAction::Split => {
            for id in &body.trade_ids {
                traderview_db::trades::split(&s.pool, *id)
                    .await
                    .map_err(ApiError::Internal)?;
            }
            body.trade_ids.len() as u64
        }
        BulkAction::Share { is_public } => {
            for id in &body.trade_ids {
                traderview_db::shares::create(&s.pool, *id, user.id, is_public, true, true)
                    .await
                    .map_err(ApiError::Internal)?;
            }
            body.trade_ids.len() as u64
        }
    };
    Ok(Json(BulkResult {
        affected: n,
        note: None,
    }))
}
