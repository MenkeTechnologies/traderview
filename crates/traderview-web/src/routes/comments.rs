use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::routing::{delete, get};
use axum::{Json, Router};
use serde::Deserialize;
use traderview_core::Comment;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/shared/:slug/comments", get(list).post(create))
        .route("/comments/:id", delete(delete_one))
}

async fn list(
    State(s): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<Vec<Comment>>, ApiError> {
    let share = traderview_db::shares::by_slug(&s.pool, &slug)
        .await
        .map_err(ApiError::Internal)?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(
        traderview_db::comments::list_for_share(&s.pool, share.id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

#[derive(Deserialize)]
struct CreateBody {
    parent_id: Option<Uuid>,
    body_md: String,
}

async fn create(
    State(s): State<AppState>,
    user: AuthUser,
    Path(slug): Path<String>,
    Json(body): Json<CreateBody>,
) -> Result<Json<Comment>, ApiError> {
    let share = traderview_db::shares::by_slug(&s.pool, &slug)
        .await
        .map_err(ApiError::Internal)?
        .ok_or(ApiError::NotFound)?;
    if !share.is_public {
        return Err(ApiError::Forbidden);
    }
    Ok(Json(
        traderview_db::comments::create(&s.pool, share.id, user.id, body.parent_id, &body.body_md)
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
        traderview_db::comments::delete(&s.pool, user.id, id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}
