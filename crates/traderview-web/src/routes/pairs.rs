//! Pairs cointegration scanner route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use traderview_db::pairs_cointegration;

pub fn router() -> Router<AppState> {
    Router::new().route("/pairs/scan", get(scan))
}

#[derive(Deserialize)]
struct ScanQ {
    /// Comma-separated symbol list.
    symbols: String,
    #[serde(default = "default_days")]
    days: i64,
}
fn default_days() -> i64 {
    180
}

async fn scan(
    State(s): State<AppState>,
    _user: AuthUser,
    Query(q): Query<ScanQ>,
) -> Result<Json<Vec<pairs_cointegration::PairScore>>, ApiError> {
    let symbols: Vec<String> = q
        .symbols
        .split(',')
        .map(|t| t.trim().to_ascii_uppercase())
        .filter(|t| !t.is_empty())
        .collect();
    if symbols.len() < 2 {
        return Ok(Json(Vec::new()));
    }
    Ok(Json(
        pairs_cointegration::scan(&s.pool, &symbols, q.days).await?,
    ))
}
