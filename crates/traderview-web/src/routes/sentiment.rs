use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use traderview_db::sentiment::{HourlyBucket, Mention, RankedSymbol};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/sentiment/feed",         get(feed))
        .route("/sentiment/ranked",       get(ranked))
        .route("/sentiment/poll",         post(poll_now))
        .route("/sentiment/symbol/:sym",  get(per_symbol))
        .route("/sentiment/series/:sym",  get(series))
}

#[derive(Deserialize)]
struct FeedQ { #[serde(default = "default_feed_limit")] limit: i64 }
fn default_feed_limit() -> i64 { 200 }

async fn feed(State(s): State<AppState>, _user: AuthUser, Query(q): Query<FeedQ>)
    -> Result<Json<Vec<Mention>>, ApiError>
{
    Ok(Json(traderview_db::sentiment::feed(&s.pool, q.limit)
        .await.map_err(ApiError::Internal)?))
}

#[derive(Deserialize)]
struct RankedQ {
    #[serde(default = "default_hours")] hours: i64,
    #[serde(default = "default_rank_limit")] limit: i64,
}
fn default_hours() -> i64 { 1 }
fn default_rank_limit() -> i64 { 50 }

async fn ranked(State(s): State<AppState>, _user: AuthUser, Query(q): Query<RankedQ>)
    -> Result<Json<Vec<RankedSymbol>>, ApiError>
{
    Ok(Json(traderview_db::sentiment::ranked(&s.pool, q.hours, q.limit)
        .await.map_err(ApiError::Internal)?))
}

async fn poll_now(State(s): State<AppState>, _user: AuthUser) -> Result<Json<serde_json::Value>, ApiError> {
    let (wsb, st) = traderview_db::sentiment::poll_all(&s.pool).await;
    Ok(Json(serde_json::json!({"wsb_inserted": wsb, "stocktwits_inserted": st})))
}

#[derive(Deserialize)]
struct SymQ {
    #[serde(default = "default_sym_hours")] hours: i64,
    #[serde(default = "default_sym_limit")] limit: i64,
}
fn default_sym_hours() -> i64 { 24 }
fn default_sym_limit() -> i64 { 100 }

async fn per_symbol(State(s): State<AppState>, _user: AuthUser, Path(sym): Path<String>, Query(q): Query<SymQ>)
    -> Result<Json<Vec<Mention>>, ApiError>
{
    Ok(Json(traderview_db::sentiment::for_symbol(&s.pool, &sym.to_uppercase(), q.hours, q.limit)
        .await.map_err(ApiError::Internal)?))
}

async fn series(State(s): State<AppState>, _user: AuthUser, Path(sym): Path<String>, Query(q): Query<SymQ>)
    -> Result<Json<Vec<HourlyBucket>>, ApiError>
{
    Ok(Json(traderview_db::sentiment::timeseries(&s.pool, &sym.to_uppercase(), q.hours)
        .await.map_err(ApiError::Internal)?))
}
