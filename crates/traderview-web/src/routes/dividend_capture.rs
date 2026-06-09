//! Dividend capture / arb scanner route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use traderview_db::dividend_capture;

pub fn router() -> Router<AppState> {
    Router::new().route("/dividend-capture/scan", get(scan))
}

#[derive(Deserialize)]
struct ScanQ {
    symbols: String,
}

async fn scan(
    State(s): State<AppState>,
    _user: AuthUser,
    Query(q): Query<ScanQ>,
) -> Result<Json<Vec<dividend_capture::DividendCaptureRow>>, ApiError> {
    let symbols: Vec<String> = q
        .symbols
        .split(',')
        .map(|t| t.trim().to_ascii_uppercase())
        .filter(|t| !t.is_empty())
        .collect();
    if symbols.is_empty() {
        return Ok(Json(Vec::new()));
    }
    Ok(Json(dividend_capture::scan(&s.pool, &symbols).await))
}
