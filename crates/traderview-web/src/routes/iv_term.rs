//! IV term-structure scanner route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use traderview_db::iv_term_structure;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/iv-term/ranked", get(ranked))
        .route("/iv-term/symbol/:symbol", get(by_symbol))
        .route("/iv-term/refresh/:symbol", get(refresh))
}

#[derive(Deserialize)]
struct LimitQ {
    #[serde(default = "default_limit")]
    limit: usize,
}
fn default_limit() -> usize {
    30
}

async fn ranked(
    State(_s): State<AppState>,
    _user: AuthUser,
    Query(q): Query<LimitQ>,
) -> Result<Json<Vec<iv_term_structure::TermStructure>>, ApiError> {
    Ok(Json(iv_term_structure::global().ranked(q.limit)))
}

async fn by_symbol(
    State(_s): State<AppState>,
    _user: AuthUser,
    Path(symbol): Path<String>,
) -> Result<Json<Option<iv_term_structure::TermStructure>>, ApiError> {
    Ok(Json(iv_term_structure::global().get(&symbol)))
}

/// Force-refresh one symbol — useful for testing without waiting for
/// the 4-hour background tick.
async fn refresh(
    State(_s): State<AppState>,
    _user: AuthUser,
    Path(symbol): Path<String>,
) -> Result<Json<Option<iv_term_structure::TermStructure>>, ApiError> {
    let row = iv_term_structure::fetch_for(&symbol).await;
    if let Some(r) = &row {
        iv_term_structure::global().upsert(r.clone());
    }
    Ok(Json(row))
}
