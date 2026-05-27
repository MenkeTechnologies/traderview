use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use traderview_db::goals::{Goal, GoalInput, GoalProgress};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/goals", get(list).post(create))
        .route("/goals/:id", axum::routing::delete(delete).put(update))
        .route("/goals/:id/progress", get(progress))
}

async fn list(State(s): State<AppState>, u: AuthUser) -> Result<Json<Vec<Goal>>, ApiError> {
    Ok(Json(
        traderview_db::goals::list(&s.pool, u.id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

async fn create(
    State(s): State<AppState>,
    u: AuthUser,
    Json(body): Json<GoalInput>,
) -> Result<Json<Goal>, ApiError> {
    if body.name.trim().is_empty() {
        return Err(ApiError::BadRequest("name required".into()));
    }
    if body.end_date < body.start_date {
        return Err(ApiError::BadRequest(
            "end_date must be >= start_date".into(),
        ));
    }
    Ok(Json(
        traderview_db::goals::create(&s.pool, u.id, &body)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

async fn update(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<GoalInput>,
) -> Result<Json<Goal>, ApiError> {
    traderview_db::goals::update(&s.pool, u.id, id, &body)
        .await
        .map_err(ApiError::Internal)?
        .map(Json)
        .ok_or(ApiError::NotFound)
}

async fn delete(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let ok = traderview_db::goals::delete(&s.pool, u.id, id)
        .await
        .map_err(ApiError::Internal)?;
    if !ok {
        return Err(ApiError::NotFound);
    }
    Ok(Json(serde_json::json!({ "deleted": true })))
}

async fn progress(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<GoalProgress>, ApiError> {
    Ok(Json(
        traderview_db::goals::progress(&s.pool, u.id, id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}
