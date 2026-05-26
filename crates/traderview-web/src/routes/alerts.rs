use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use rust_decimal::Decimal;
use serde::Deserialize;
use traderview_db::alerts::AlertRule;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/alerts", get(list).post(create))
        .route("/alerts/:id", delete(delete_one))
        .route("/alerts/:id/toggle", post(toggle))
        .route("/alerts/:id/fired", post(fired))
}

async fn list(State(s): State<AppState>, user: AuthUser) -> Result<Json<Vec<AlertRule>>, ApiError> {
    Ok(Json(traderview_db::alerts::list(&s.pool, user.id)
        .await.map_err(ApiError::Internal)?))
}

#[derive(Deserialize)]
struct CreateBody {
    symbol: String,
    trigger: String,
    threshold: Option<Decimal>,
    #[serde(default = "default_sound")]
    sound: String,
    voice_text: Option<String>,
}
fn default_sound() -> String { "bell".into() }

async fn create(State(s): State<AppState>, user: AuthUser, Json(b): Json<CreateBody>)
    -> Result<Json<AlertRule>, ApiError>
{
    Ok(Json(traderview_db::alerts::create(
        &s.pool, user.id, &b.symbol, &b.trigger, b.threshold, &b.sound, b.voice_text.as_deref(),
    ).await.map_err(ApiError::Internal)?))
}

async fn delete_one(State(s): State<AppState>, user: AuthUser, Path(id): Path<Uuid>)
    -> Result<Json<bool>, ApiError>
{
    Ok(Json(traderview_db::alerts::delete(&s.pool, user.id, id)
        .await.map_err(ApiError::Internal)?))
}

#[derive(Deserialize)]
struct ToggleBody { enabled: bool }

async fn toggle(State(s): State<AppState>, user: AuthUser, Path(id): Path<Uuid>, Json(b): Json<ToggleBody>)
    -> Result<Json<bool>, ApiError>
{
    Ok(Json(traderview_db::alerts::toggle(&s.pool, user.id, id, b.enabled)
        .await.map_err(ApiError::Internal)?))
}

async fn fired(State(s): State<AppState>, _user: AuthUser, Path(id): Path<Uuid>)
    -> Result<Json<bool>, ApiError>
{
    traderview_db::alerts::mark_fired(&s.pool, id).await.map_err(ApiError::Internal)?;
    Ok(Json(true))
}
