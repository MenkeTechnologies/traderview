use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use traderview_db::earnings_iv::{EarningsIvHit, EarningsIvReport};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/iv/scan",              get(scan))
        .route("/iv/symbols/:symbol",   get(symbol))
}

#[derive(Deserialize)]
struct ScanQ {
    watchlist_id: Option<Uuid>,
    #[serde(default = "default_horizon")]
    horizon_days: i64,
    #[serde(default = "default_limit")]
    limit: usize,
}
fn default_horizon() -> i64 { 7 }
fn default_limit() -> usize { 50 }

async fn scan(State(s): State<AppState>, user: AuthUser, Query(q): Query<ScanQ>)
    -> Result<Json<Vec<EarningsIvHit>>, ApiError>
{
    Ok(Json(traderview_db::earnings_iv::scan(
        &s.pool, user.id, q.watchlist_id, q.horizon_days, q.limit,
    ).await.map_err(ApiError::Internal)?))
}

async fn symbol(State(s): State<AppState>, _user: AuthUser, Path(symbol): Path<String>)
    -> Result<Json<EarningsIvReport>, ApiError>
{
    Ok(Json(traderview_db::earnings_iv::report(&s.pool, &symbol)
        .await.map_err(ApiError::Internal)?))
}
