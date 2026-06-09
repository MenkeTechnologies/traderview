//! Scanner backtest route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use traderview_db::scanner_backtest;

pub fn router() -> Router<AppState> {
    Router::new().route("/scanner-backtest/pead", get(pead))
}

#[derive(Deserialize)]
struct DaysQ {
    #[serde(default = "default_days")]
    days: i64,
}
fn default_days() -> i64 {
    365
}

async fn pead(
    State(s): State<AppState>,
    _user: AuthUser,
    Query(q): Query<DaysQ>,
) -> Result<Json<scanner_backtest::BacktestResult>, ApiError> {
    Ok(Json(
        scanner_backtest::backtest_pead(&s.pool, q.days).await?,
    ))
}
