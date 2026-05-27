use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::routes::helpers::ensure_trade_owner;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::NaiveDate;
use serde::Deserialize;
use traderview_core::JournalEntry;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/journal/day/:day", get(for_day))
        .route("/journal/trade/:id", get(for_trade))
        .route("/journal/general", get(general))
        .route("/journal", post(create))
        .route("/journal/:id", post(update).delete(delete_one))
}

async fn for_day(
    State(s): State<AppState>,
    user: AuthUser,
    Path(day): Path<NaiveDate>,
) -> Result<Json<Vec<JournalEntry>>, ApiError> {
    Ok(Json(
        traderview_db::journal::list_for_day(&s.pool, user.id, day)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

async fn for_trade(
    State(s): State<AppState>,
    user: AuthUser,
    Path(trade_id): Path<Uuid>,
) -> Result<Json<Vec<JournalEntry>>, ApiError> {
    ensure_trade_owner(&s.pool, user.id, trade_id).await?;
    Ok(Json(
        traderview_db::journal::list_for_trade(&s.pool, user.id, trade_id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

#[derive(Deserialize)]
struct CreateBody {
    trade_id: Option<Uuid>,
    day: Option<NaiveDate>,
    body_md: String,
    mood: Option<i16>,
}

async fn create(
    State(s): State<AppState>,
    user: AuthUser,
    Json(body): Json<CreateBody>,
) -> Result<Json<JournalEntry>, ApiError> {
    // Migration 0009 dropped the trade_id-OR-day CHECK so general notes
    // (both null) are now valid.
    if let Some(tid) = body.trade_id {
        ensure_trade_owner(&s.pool, user.id, tid).await?;
    }
    Ok(Json(
        traderview_db::journal::create(
            &s.pool,
            user.id,
            body.trade_id,
            body.day,
            &body.body_md,
            body.mood,
        )
        .await
        .map_err(ApiError::Internal)?,
    ))
}

async fn general(
    State(s): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<JournalEntry>>, ApiError> {
    Ok(Json(
        traderview_db::journal::list_general(&s.pool, user.id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

#[derive(Deserialize)]
struct UpdateBody {
    body_md: String,
    mood: Option<i16>,
}

async fn update(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateBody>,
) -> Result<Json<bool>, ApiError> {
    Ok(Json(
        traderview_db::journal::update(&s.pool, user.id, id, &body.body_md, body.mood)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

async fn delete_one(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<bool>, ApiError> {
    Ok(Json(
        traderview_db::journal::delete(&s.pool, user.id, id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}
