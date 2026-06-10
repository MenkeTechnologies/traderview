//! Greenblatt magic formula value scorer route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use traderview_db::magic_formula;

pub fn router() -> Router<AppState> {
    Router::new().route("/magic-formula/rank", get(rank))
}

#[derive(Deserialize)]
struct RankQ {
    #[serde(default = "default_max")]
    max_symbols: usize,
}
fn default_max() -> usize {
    50
}

async fn rank(
    State(_s): State<AppState>,
    _user: AuthUser,
    Query(q): Query<RankQ>,
) -> Result<Json<magic_formula::MagicFormulaReport>, ApiError> {
    if !(q.max_symbols >= 1 && q.max_symbols <= 250) {
        return Err(ApiError::BadRequest(
            "max_symbols must be in [1, 250]".into(),
        ));
    }
    let universe = magic_formula::default_universe();
    let symbols: Vec<&str> = universe.iter().copied().collect();
    Ok(Json(
        magic_formula::score_universe(&symbols, q.max_symbols).await,
    ))
}
