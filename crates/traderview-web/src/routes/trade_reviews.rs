use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::routes::helpers::ensure_account_owner;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use traderview_db::trade_reviews::{NeedsReviewRow, ReviewInput, ReviewStats, TradeReview};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/trade-reviews", get(list).post(upsert))
        .route("/trade-reviews/needed/:account_id", get(needed))
        .route("/trade-reviews/stats/:account_id", get(stats))
        .route("/trade-reviews/trade/:trade_id", get(for_trade))
        .route("/trade-reviews/trade/:trade_id/delete", post(delete))
}

#[derive(Debug, Deserialize)]
struct Limit {
    limit: Option<i64>,
}

async fn list(
    State(s): State<AppState>,
    u: AuthUser,
    Query(p): Query<Limit>,
) -> Result<Json<Vec<TradeReview>>, ApiError> {
    let limit = p.limit.unwrap_or(50).clamp(1, 500);
    Ok(Json(
        traderview_db::trade_reviews::list(&s.pool, u.id, limit)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

async fn needed(
    State(s): State<AppState>,
    u: AuthUser,
    Path(account_id): Path<Uuid>,
    Query(p): Query<Limit>,
) -> Result<Json<Vec<NeedsReviewRow>>, ApiError> {
    ensure_account_owner(&s, u.id, account_id).await?;
    let limit = p.limit.unwrap_or(50).clamp(1, 500);
    Ok(Json(
        traderview_db::trade_reviews::needs_review(&s.pool, u.id, account_id, limit)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

async fn stats(
    State(s): State<AppState>,
    u: AuthUser,
    Path(account_id): Path<Uuid>,
) -> Result<Json<ReviewStats>, ApiError> {
    ensure_account_owner(&s, u.id, account_id).await?;
    Ok(Json(
        traderview_db::trade_reviews::stats(&s.pool, u.id, account_id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

async fn for_trade(
    State(s): State<AppState>,
    u: AuthUser,
    Path(trade_id): Path<Uuid>,
) -> Result<Json<Option<TradeReview>>, ApiError> {
    Ok(Json(
        traderview_db::trade_reviews::for_trade(&s.pool, u.id, trade_id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

async fn upsert(
    State(s): State<AppState>,
    u: AuthUser,
    Json(body): Json<ReviewInput>,
) -> Result<Json<TradeReview>, ApiError> {
    Ok(Json(
        traderview_db::trade_reviews::upsert(&s.pool, u.id, &body)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

async fn delete(
    State(s): State<AppState>,
    u: AuthUser,
    Path(trade_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let ok = traderview_db::trade_reviews::delete(&s.pool, u.id, trade_id)
        .await
        .map_err(ApiError::Internal)?;
    if !ok {
        return Err(ApiError::NotFound);
    }
    Ok(Json(serde_json::json!({ "deleted": true })))
}
