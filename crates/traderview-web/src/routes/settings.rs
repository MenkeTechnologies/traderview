use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::routing::{delete, get};
use axum::{Json, Router};
use serde::Deserialize;
use traderview_core::{FilterSet, UserSettings};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/settings", get(get_settings).post(upsert_settings))
        .route("/filter-sets", get(list_filters).post(save_filter))
        .route("/filter-sets/:id", delete(delete_filter))
}

async fn get_settings(
    State(s): State<AppState>,
    user: AuthUser,
) -> Result<Json<UserSettings>, ApiError> {
    Ok(Json(
        traderview_db::settings::get(&s.pool, user.id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

async fn upsert_settings(
    State(s): State<AppState>,
    user: AuthUser,
    Json(mut body): Json<UserSettings>,
) -> Result<Json<UserSettings>, ApiError> {
    body.user_id = user.id;
    traderview_db::settings::upsert(&s.pool, &body)
        .await
        .map_err(ApiError::Internal)?;
    Ok(Json(body))
}

async fn list_filters(
    State(s): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<FilterSet>>, ApiError> {
    Ok(Json(
        traderview_db::settings::list_filters(&s.pool, user.id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

#[derive(Deserialize)]
struct SaveBody {
    name: String,
    payload: serde_json::Value,
    #[serde(default)]
    is_default: bool,
}

async fn save_filter(
    State(s): State<AppState>,
    user: AuthUser,
    Json(body): Json<SaveBody>,
) -> Result<Json<FilterSet>, ApiError> {
    Ok(Json(
        traderview_db::settings::save_filter(
            &s.pool, user.id, &body.name, &body.payload, body.is_default,
        )
        .await
        .map_err(ApiError::Internal)?,
    ))
}

async fn delete_filter(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<bool>, ApiError> {
    Ok(Json(
        traderview_db::settings::delete_filter(&s.pool, user.id, id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}
