use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use traderview_db::news::{NewsRow, PollStats};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/news/symbol/:symbol", get(by_symbol))
        .route("/news/recent",          get(recent_global))
        .route("/news/search",          get(search))
        .route("/news/poll-now",        post(poll_now))
        .route("/news/symbol/:symbol/refresh", post(refresh_symbol))
}

#[derive(Debug, Deserialize)]
struct PageQ { limit: Option<i64> }

#[derive(Debug, Deserialize)]
struct SearchQ { q: String, limit: Option<i64> }

async fn by_symbol(
    State(s): State<AppState>,
    _u: AuthUser,
    Path(symbol): Path<String>,
    Query(p): Query<PageQ>,
) -> Result<Json<Vec<NewsRow>>, ApiError> {
    let sym = symbol.to_uppercase();
    let limit = p.limit.unwrap_or(20).clamp(1, 200);
    Ok(Json(traderview_db::news::recent_for_symbol(&s.pool, &sym, limit)
        .await.map_err(ApiError::Internal)?))
}

async fn recent_global(
    State(s): State<AppState>,
    _u: AuthUser,
    Query(p): Query<PageQ>,
) -> Result<Json<Vec<NewsRow>>, ApiError> {
    let limit = p.limit.unwrap_or(40).clamp(1, 200);
    Ok(Json(traderview_db::news::recent_global(&s.pool, limit)
        .await.map_err(ApiError::Internal)?))
}

async fn search(
    State(s): State<AppState>,
    _u: AuthUser,
    Query(p): Query<SearchQ>,
) -> Result<Json<Vec<NewsRow>>, ApiError> {
    if p.q.trim().is_empty() {
        return Err(ApiError::BadRequest("q required".into()));
    }
    let limit = p.limit.unwrap_or(50).clamp(1, 200);
    Ok(Json(traderview_db::news::search(&s.pool, &p.q, limit)
        .await.map_err(ApiError::Internal)?))
}

async fn poll_now(
    State(s): State<AppState>,
    _u: AuthUser,
) -> Result<Json<PollStats>, ApiError> {
    Ok(Json(traderview_db::news::poll_watchlists(&s.pool)
        .await.map_err(ApiError::Internal)?))
}

async fn refresh_symbol(
    State(s): State<AppState>,
    _u: AuthUser,
    Path(symbol): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let sym = symbol.to_uppercase();
    let n = traderview_db::news::fetch_for_symbol(&s.pool, &sym, 20)
        .await.map_err(ApiError::Internal)?;
    Ok(Json(serde_json::json!({ "symbol": sym, "inserted": n })))
}
