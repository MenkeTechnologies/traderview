use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use traderview_core::{ForumCategory, ForumPost, ForumThread};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/forum/categories", get(categories))
        .route("/forum/threads", post(create_thread))
        .route("/forum/threads/category/:slug", get(list_in_category))
        .route(
            "/forum/threads/:thread_id/posts",
            get(list_posts).post(create_post),
        )
        .route("/forum/threads/:thread_id/view", post(bump_view))
        .route("/forum/by-slug/:cat_slug/:thread_slug", get(by_slug))
}

async fn categories(State(s): State<AppState>) -> Result<Json<Vec<ForumCategory>>, ApiError> {
    Ok(Json(
        traderview_db::forum::list_categories(&s.pool)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

#[derive(Deserialize)]
struct CreateThreadBody {
    category_id: Uuid,
    title: String,
    body_md: String,
}

#[derive(Serialize)]
struct ThreadCreated {
    thread: ForumThread,
    op_post: ForumPost,
}

async fn create_thread(
    State(s): State<AppState>,
    user: AuthUser,
    Json(body): Json<CreateThreadBody>,
) -> Result<Json<ThreadCreated>, ApiError> {
    let (thread, op_post) = traderview_db::forum::create_thread(
        &s.pool,
        body.category_id,
        user.id,
        &body.title,
        &body.body_md,
    )
    .await
    .map_err(ApiError::Internal)?;
    Ok(Json(ThreadCreated { thread, op_post }))
}

async fn list_in_category(
    State(s): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<Vec<ForumThread>>, ApiError> {
    let cats = traderview_db::forum::list_categories(&s.pool)
        .await
        .map_err(ApiError::Internal)?;
    let cat = cats
        .into_iter()
        .find(|c| c.slug == slug)
        .ok_or(ApiError::NotFound)?;
    Ok(Json(
        traderview_db::forum::list_threads_in(&s.pool, cat.id, 100)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

async fn list_posts(
    State(s): State<AppState>,
    Path(thread_id): Path<Uuid>,
) -> Result<Json<Vec<ForumPost>>, ApiError> {
    Ok(Json(
        traderview_db::forum::list_posts(&s.pool, thread_id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

#[derive(Deserialize)]
struct CreatePostBody {
    body_md: String,
}

async fn create_post(
    State(s): State<AppState>,
    user: AuthUser,
    Path(thread_id): Path<Uuid>,
    Json(body): Json<CreatePostBody>,
) -> Result<Json<ForumPost>, ApiError> {
    Ok(Json(
        traderview_db::forum::create_post(&s.pool, thread_id, user.id, &body.body_md)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

async fn bump_view(
    State(s): State<AppState>,
    Path(thread_id): Path<Uuid>,
) -> Result<Json<bool>, ApiError> {
    traderview_db::forum::bump_thread_view(&s.pool, thread_id)
        .await
        .map_err(ApiError::Internal)?;
    Ok(Json(true))
}

async fn by_slug(
    State(s): State<AppState>,
    Path((cat_slug, thread_slug)): Path<(String, String)>,
) -> Result<Json<ForumThread>, ApiError> {
    Ok(Json(
        traderview_db::forum::thread_by_slug(&s.pool, &cat_slug, &thread_slug)
            .await
            .map_err(ApiError::Internal)?
            .ok_or(ApiError::NotFound)?,
    ))
}
