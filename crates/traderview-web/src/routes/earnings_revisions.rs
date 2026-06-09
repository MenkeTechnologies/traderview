//! Earnings revision tracker route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use traderview_db::earnings_revisions;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/earnings-revisions/scan", get(scan))
        .route("/earnings-revisions/symbol/:symbol", get(by_symbol))
}

#[derive(Deserialize)]
struct ScanQ {
    symbols: String,
}

async fn scan(
    State(_s): State<AppState>,
    _user: AuthUser,
    Query(q): Query<ScanQ>,
) -> Result<Json<Vec<earnings_revisions::RevisionMetrics>>, ApiError> {
    let symbols: Vec<String> = q
        .symbols
        .split(',')
        .map(|t| t.trim().to_ascii_uppercase())
        .filter(|t| !t.is_empty())
        .collect();
    if symbols.is_empty() {
        return Ok(Json(Vec::new()));
    }
    Ok(Json(earnings_revisions::scan(&symbols).await))
}

async fn by_symbol(
    State(_s): State<AppState>,
    _user: AuthUser,
    Path(symbol): Path<String>,
) -> Result<Json<Vec<earnings_revisions::RevisionMetrics>>, ApiError> {
    Ok(Json(earnings_revisions::for_symbol(&symbol).await))
}
