use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::routes::helpers::ensure_trade_owner;
use crate::state::AppState;
use axum::body::Body;
use axum::extract::{Multipart, Path, State};
use axum::http::header;
use axum::response::Response;
use axum::routing::{delete, get};
use axum::{Json, Router};
use traderview_core::Screenshot;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/trades/:id/screenshots", get(list).post(upload))
        .route("/screenshots/:id", delete(delete_one))
        .route("/screenshots/:id/bytes", get(get_bytes))
}

async fn list(
    State(s): State<AppState>,
    user: AuthUser,
    Path(trade_id): Path<Uuid>,
) -> Result<Json<Vec<Screenshot>>, ApiError> {
    ensure_trade_owner(&s.pool, user.id, trade_id).await?;
    Ok(Json(
        traderview_db::screenshots::list_for_trade(&s.pool, trade_id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

async fn upload(
    State(s): State<AppState>,
    user: AuthUser,
    Path(trade_id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<Json<Screenshot>, ApiError> {
    ensure_trade_owner(&s.pool, user.id, trade_id).await?;
    let mut filename = String::new();
    let mut mime = String::new();
    let mut bytes: Vec<u8> = Vec::new();
    let mut caption = String::new();
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::BadRequest(format!("multipart: {e}")))?
    {
        match field.name() {
            Some("file") => {
                filename = field.file_name().unwrap_or("upload").into();
                mime = field
                    .content_type()
                    .unwrap_or("application/octet-stream")
                    .into();
                bytes = field
                    .bytes()
                    .await
                    .map_err(|e| ApiError::BadRequest(format!("file: {e}")))?
                    .to_vec();
            }
            Some("caption") => {
                caption = field
                    .text()
                    .await
                    .map_err(|e| ApiError::BadRequest(format!("caption: {e}")))?;
            }
            _ => {}
        }
    }
    if bytes.is_empty() {
        return Err(ApiError::BadRequest("no `file` part in multipart body".into()));
    }
    Ok(Json(
        traderview_db::screenshots::create(
            &s.pool, user.id, Some(trade_id), None, &filename, &mime, &bytes, &caption,
        )
        .await
        .map_err(ApiError::Internal)?,
    ))
}

async fn get_bytes(
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Response, ApiError> {
    let (mime, bytes) = traderview_db::screenshots::get_bytes(&s.pool, id)
        .await
        .map_err(ApiError::Internal)?
        .ok_or(ApiError::NotFound)?;
    Ok(Response::builder()
        .header(header::CONTENT_TYPE, mime)
        .header(header::CACHE_CONTROL, "private, max-age=604800")
        .body(Body::from(bytes))
        .unwrap())
}

async fn delete_one(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<bool>, ApiError> {
    Ok(Json(
        traderview_db::screenshots::delete(&s.pool, user.id, id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}
