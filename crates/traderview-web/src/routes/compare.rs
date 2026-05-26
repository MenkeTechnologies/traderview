use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use traderview_db::compare::CompareReport;

pub fn router() -> Router<AppState> {
    Router::new().route("/compare", get(compare_handler))
}

#[derive(Debug, Deserialize)]
struct Params {
    symbols: String,
}

async fn compare_handler(
    State(s): State<AppState>,
    _u: AuthUser,
    Query(p): Query<Params>,
) -> Result<Json<CompareReport>, ApiError> {
    let syms: Vec<String> = p.symbols
        .split(',')
        .map(|s| s.trim().to_uppercase())
        .filter(|s| !s.is_empty())
        .take(4)
        .collect();
    if syms.len() < 2 {
        return Err(ApiError::BadRequest("need at least 2 symbols (comma-separated, max 4)".into()));
    }
    Ok(Json(traderview_db::compare::compare(&s.pool, &syms).await.map_err(ApiError::Internal)?))
}
