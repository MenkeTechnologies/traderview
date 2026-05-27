use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::routing::{delete, get};
use axum::{Json, Router};
use serde::Deserialize;
use traderview_db::hotkeys::Hotkey;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/hotkeys", get(list).post(upsert))
        .route("/hotkeys/:id", delete(delete_one))
}

async fn list(State(s): State<AppState>, user: AuthUser) -> Result<Json<Vec<Hotkey>>, ApiError> {
    Ok(Json(traderview_db::hotkeys::list(&s.pool, user.id)
        .await.map_err(ApiError::Internal)?))
}

#[derive(Deserialize)]
struct UpsertBody {
    name: String,
    combo: String,
    action: String,
    #[serde(default)]
    payload: serde_json::Value,
}

async fn upsert(State(s): State<AppState>, user: AuthUser, Json(b): Json<UpsertBody>)
    -> Result<Json<Hotkey>, ApiError>
{
    Ok(Json(traderview_db::hotkeys::upsert(
        &s.pool, user.id, &b.name, &b.combo, &b.action, &b.payload,
    ).await.map_err(ApiError::Internal)?))
}

async fn delete_one(State(s): State<AppState>, user: AuthUser, Path(id): Path<Uuid>)
    -> Result<Json<bool>, ApiError>
{
    Ok(Json(traderview_db::hotkeys::delete(&s.pool, user.id, id)
        .await.map_err(ApiError::Internal)?))
}
