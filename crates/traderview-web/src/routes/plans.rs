use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::routes::helpers::{ensure_account_owner, ensure_trade_owner};
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use rust_decimal::Decimal;
use serde::Deserialize;
use traderview_core::{AssetClass, TradePlan, TradeSide};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/plans", get(list).post(create))
        .route("/plans/:id/link/:trade_id", post(link))
        .route("/plans/:id", delete(abandon))
}

async fn list(
    State(s): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<TradePlan>>, ApiError> {
    Ok(Json(
        traderview_db::plans::list_pending(&s.pool, user.id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

#[derive(Deserialize)]
struct CreateBody {
    account_id: Uuid,
    symbol: String,
    asset_class: AssetClass,
    side: TradeSide,
    intended_qty: Decimal,
    intended_entry: Decimal,
    stop_loss: Option<Decimal>,
    initial_target: Option<Decimal>,
    #[serde(default)]
    setup_notes: String,
}

async fn create(
    State(s): State<AppState>,
    user: AuthUser,
    Json(body): Json<CreateBody>,
) -> Result<Json<TradePlan>, ApiError> {
    ensure_account_owner(&s, user.id, body.account_id).await?;
    Ok(Json(
        traderview_db::plans::create(
            &s.pool,
            traderview_db::plans::NewPlan {
                user_id: user.id,
                account_id: body.account_id,
                symbol: &body.symbol,
                asset_class: body.asset_class,
                side: body.side,
                intended_qty: body.intended_qty,
                intended_entry: body.intended_entry,
                stop_loss: body.stop_loss,
                initial_target: body.initial_target,
                setup_notes: &body.setup_notes,
            },
        )
        .await
        .map_err(ApiError::Internal)?,
    ))
}

async fn link(
    State(s): State<AppState>,
    user: AuthUser,
    Path((plan_id, trade_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<bool>, ApiError> {
    ensure_trade_owner(&s.pool, user.id, trade_id).await?;
    Ok(Json(
        traderview_db::plans::link_to_trade(&s.pool, user.id, plan_id, trade_id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

async fn abandon(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<bool>, ApiError> {
    Ok(Json(
        traderview_db::plans::abandon(&s.pool, user.id, id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}
