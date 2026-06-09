//! Hard-to-borrow / squeeze-pressure ranker route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use traderview_db::htb_ranker;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/htb-ranker/ranked", get(ranked))
        .route("/htb-ranker/symbol/:symbol", get(by_symbol))
}

#[derive(Deserialize)]
struct LimitQ {
    #[serde(default = "default_limit")]
    limit: usize,
}
fn default_limit() -> usize {
    50
}

async fn ranked(
    State(_s): State<AppState>,
    _user: AuthUser,
    Query(q): Query<LimitQ>,
) -> Result<Json<Vec<htb_ranker::HtbScore>>, ApiError> {
    Ok(Json(htb_ranker::global().ranked(q.limit)))
}

async fn by_symbol(
    State(_s): State<AppState>,
    _user: AuthUser,
    Path(symbol): Path<String>,
) -> Result<Json<Option<htb_ranker::HtbScore>>, ApiError> {
    Ok(Json(htb_ranker::global().get(&symbol)))
}
