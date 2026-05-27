use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::routes::helpers::ensure_trade_owner;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::routing::{delete, get};
use axum::{Json, Router};
use serde::Deserialize;
use traderview_core::Tag;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/tags", get(list).post(create))
        .route("/tags/:id", delete(delete_one))
        .route("/trades/:id/tags", get(for_trade).post(attach))
        .route("/trades/:id/tags/:tag_id", delete(detach))
}

async fn list(State(s): State<AppState>, user: AuthUser) -> Result<Json<Vec<Tag>>, ApiError> {
    Ok(Json(
        traderview_db::tags::list(&s.pool, user.id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

#[derive(Deserialize)]
struct CreateBody {
    name: String,
    #[serde(default = "default_color")]
    color: String,
}
fn default_color() -> String {
    "#00e5ff".into()
}

async fn create(
    State(s): State<AppState>,
    user: AuthUser,
    Json(body): Json<CreateBody>,
) -> Result<Json<Tag>, ApiError> {
    Ok(Json(
        traderview_db::tags::create(&s.pool, user.id, &body.name, &body.color)
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
        traderview_db::tags::delete(&s.pool, user.id, id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

async fn for_trade(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<Tag>>, ApiError> {
    ensure_trade_owner(&s.pool, user.id, id).await?;
    Ok(Json(
        traderview_db::tags::tags_for_trade(&s.pool, id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

#[derive(Deserialize)]
struct AttachBody {
    tag_id: Uuid,
}

async fn attach(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<AttachBody>,
) -> Result<Json<bool>, ApiError> {
    ensure_trade_owner(&s.pool, user.id, id).await?;
    traderview_db::tags::attach_to_trade(&s.pool, id, body.tag_id)
        .await
        .map_err(ApiError::Internal)?;
    Ok(Json(true))
}

async fn detach(
    State(s): State<AppState>,
    user: AuthUser,
    Path((trade_id, tag_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<bool>, ApiError> {
    ensure_trade_owner(&s.pool, user.id, trade_id).await?;
    traderview_db::tags::detach_from_trade(&s.pool, trade_id, tag_id)
        .await
        .map_err(ApiError::Internal)?;
    Ok(Json(true))
}
