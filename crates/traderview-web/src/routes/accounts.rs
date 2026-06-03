use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::routes::helpers::ensure_account_owner;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use traderview_core::Account;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/accounts", get(list).post(create))
        .route("/accounts/:id", delete(delete_one))
        .route("/accounts/:id/rebuild-trades", post(rebuild_trades))
}

#[derive(Serialize)]
struct RebuildResp {
    trades_rolled: usize,
}

/// Force a FIFO rebuild of the trades table from current executions for the
/// given account. Used to recover after a migration that deduped executions
/// (e.g. 0037_executions_dedupe_no_order_id) but left the materialized
/// trades table referencing the inflated pre-dedup count.
async fn rebuild_trades(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<RebuildResp>, ApiError> {
    ensure_account_owner(&s, user.id, id).await?;
    let n = traderview_db::trades::rollup_account(&s.pool, id)
        .await
        .map_err(ApiError::Internal)?;
    Ok(Json(RebuildResp { trades_rolled: n }))
}

async fn list(State(s): State<AppState>, user: AuthUser) -> Result<Json<Vec<Account>>, ApiError> {
    Ok(Json(
        traderview_db::accounts::list(&s.pool, user.id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

#[derive(Deserialize)]
struct CreateBody {
    broker: String,
    name: String,
    #[serde(default = "default_ccy")]
    base_currency: String,
}
fn default_ccy() -> String {
    "USD".into()
}

async fn create(
    State(s): State<AppState>,
    user: AuthUser,
    Json(body): Json<CreateBody>,
) -> Result<Json<Account>, ApiError> {
    Ok(Json(
        traderview_db::accounts::create(
            &s.pool,
            user.id,
            &body.broker,
            &body.name,
            &body.base_currency,
        )
        .await
        .map_err(ApiError::Internal)?,
    ))
}

async fn delete_one(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<bool>, ApiError> {
    Ok(Json(
        traderview_db::accounts::delete(&s.pool, user.id, id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}
