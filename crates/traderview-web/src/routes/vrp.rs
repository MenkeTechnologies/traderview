//! Volatility-risk-premium (VRP) scanner route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use traderview_db::vrp_scanner;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/vrp/ranked", get(ranked))
        .route("/vrp/symbol/:symbol", get(by_symbol))
        .route("/vrp/refresh/:symbol", get(refresh))
}

#[derive(Deserialize)]
struct RankedQ {
    #[serde(default = "default_dir")]
    direction: String,
    #[serde(default = "default_limit")]
    limit: usize,
}
fn default_dir() -> String {
    "sell".into()
}
fn default_limit() -> usize {
    50
}

async fn ranked(
    State(s): State<AppState>,
    _user: AuthUser,
    Query(q): Query<RankedQ>,
) -> Result<Json<Vec<vrp_scanner::VrpScore>>, ApiError> {
    let store = vrp_scanner::global(s.pool.clone());
    Ok(Json(store.ranked(&q.direction, q.limit)))
}

async fn by_symbol(
    State(s): State<AppState>,
    _user: AuthUser,
    Path(symbol): Path<String>,
) -> Result<Json<Option<vrp_scanner::VrpScore>>, ApiError> {
    let store = vrp_scanner::global(s.pool.clone());
    Ok(Json(store.get(&symbol)))
}

/// Force an immediate refresh of a single symbol (useful for testing
/// the scanner without waiting for the 60-min background tick).
async fn refresh(
    State(s): State<AppState>,
    _user: AuthUser,
    Path(symbol): Path<String>,
) -> Result<Json<Option<vrp_scanner::VrpScore>>, ApiError> {
    let store = vrp_scanner::global(s.pool.clone());
    let row = vrp_scanner::refresh_symbol(&s.pool, &store, &symbol).await?;
    Ok(Json(row))
}
