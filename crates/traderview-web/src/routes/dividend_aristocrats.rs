//! Dividend Aristocrats / Kings tracker route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use traderview_db::dividend_aristocrats;

pub fn router() -> Router<AppState> {
    Router::new().route("/dividend-aristocrats/rank", get(rank))
}

#[derive(Deserialize)]
struct RankQ {
    #[serde(default = "default_max")]
    max_symbols: usize,
}
fn default_max() -> usize {
    30
}

async fn rank(
    State(_s): State<AppState>,
    _user: AuthUser,
    Query(q): Query<RankQ>,
) -> Result<Json<dividend_aristocrats::AristocratsReport>, ApiError> {
    if !(q.max_symbols >= 1 && q.max_symbols <= 100) {
        return Err(ApiError::BadRequest(
            "max_symbols must be in [1, 100]".into(),
        ));
    }
    Ok(Json(dividend_aristocrats::score_all(q.max_symbols).await))
}
