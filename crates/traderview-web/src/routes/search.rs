use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use traderview_db::search::SearchHits;

pub fn router() -> Router<AppState> {
    Router::new().route("/search", get(search))
}

#[derive(Deserialize)]
struct SearchQ {
    q: String,
    #[serde(default = "default_scope")]
    scope: String,
    #[serde(default = "default_limit")]
    limit: i64,
}
fn default_scope() -> String { "all".into() }
fn default_limit() -> i64 { 50 }

async fn search(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<SearchQ>,
) -> Result<Json<SearchHits>, ApiError> {
    let q_str = q.q.trim();
    if q_str.is_empty() {
        return Err(ApiError::BadRequest("q is required".into()));
    }
    Ok(Json(
        traderview_db::search::search(&s.pool, user.id, q_str, &q.scope, q.limit)
            .await
            .map_err(ApiError::Internal)?,
    ))
}
