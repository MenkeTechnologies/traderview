use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use rust_decimal::Decimal;
use serde::Deserialize;
use traderview_db::paper::{OrderRequest, PaperAccount, PaperOrder, PaperPosition};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/paper/accounts",            get(list).post(ensure_default))
        .route("/paper/accounts/:id/reset",  post(reset))
        .route("/paper/accounts/:id/orders", get(orders).post(submit))
        .route("/paper/accounts/:id/positions", get(positions))
}

async fn list(State(s): State<AppState>, user: AuthUser) -> Result<Json<Vec<PaperAccount>>, ApiError> {
    Ok(Json(traderview_db::paper::list_accounts(&s.pool, user.id)
        .await.map_err(ApiError::Internal)?))
}

async fn ensure_default(State(s): State<AppState>, user: AuthUser) -> Result<Json<PaperAccount>, ApiError> {
    Ok(Json(traderview_db::paper::ensure_default(&s.pool, user.id)
        .await.map_err(ApiError::Internal)?))
}

#[derive(Deserialize)]
struct ResetBody { starting_cash: Decimal }

async fn reset(State(s): State<AppState>, user: AuthUser, Path(id): Path<Uuid>, Json(b): Json<ResetBody>)
    -> Result<Json<bool>, ApiError>
{
    Ok(Json(traderview_db::paper::reset(&s.pool, user.id, id, b.starting_cash)
        .await.map_err(ApiError::Internal)?))
}

#[derive(Deserialize)]
struct OrdersQ { #[serde(default = "default_limit")] limit: i64 }
fn default_limit() -> i64 { 100 }

async fn orders(State(s): State<AppState>, _user: AuthUser, Path(id): Path<Uuid>, Query(q): Query<OrdersQ>)
    -> Result<Json<Vec<PaperOrder>>, ApiError>
{
    Ok(Json(traderview_db::paper::list_orders(&s.pool, id, q.limit)
        .await.map_err(ApiError::Internal)?))
}

async fn submit(State(s): State<AppState>, user: AuthUser, Path(id): Path<Uuid>, Json(req): Json<OrderRequest>)
    -> Result<Json<PaperOrder>, ApiError>
{
    Ok(Json(traderview_db::paper::submit(&s.pool, user.id, id, req)
        .await.map_err(ApiError::Internal)?))
}

async fn positions(State(s): State<AppState>, _user: AuthUser, Path(id): Path<Uuid>)
    -> Result<Json<Vec<PaperPosition>>, ApiError>
{
    Ok(Json(traderview_db::paper::positions(&s.pool, id)
        .await.map_err(ApiError::Internal)?))
}
