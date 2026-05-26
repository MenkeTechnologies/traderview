use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use chrono::Utc;
use serde::Deserialize;
use traderview_db::webhooks::{AlertPayload, Webhook};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/webhooks",                 get(list).post(create))
        .route("/webhooks/:id",             delete(delete_one))
        .route("/webhooks/:id/toggle",      post(toggle))
        .route("/webhooks/:id/test",        post(test_fire))
}

async fn list(State(s): State<AppState>, user: AuthUser) -> Result<Json<Vec<Webhook>>, ApiError> {
    Ok(Json(traderview_db::webhooks::list(&s.pool, user.id)
        .await.map_err(ApiError::Internal)?))
}

#[derive(Deserialize)]
struct CreateBody {
    name: String,
    kind: String,       // discord | slack | generic
    url: String,
    secret: Option<String>,
}

async fn create(State(s): State<AppState>, user: AuthUser, Json(b): Json<CreateBody>)
    -> Result<Json<Webhook>, ApiError>
{
    if !["discord", "slack", "generic"].contains(&b.kind.as_str()) {
        return Err(ApiError::BadRequest("kind must be discord|slack|generic".into()));
    }
    Ok(Json(traderview_db::webhooks::create(&s.pool, user.id, &b.name, &b.kind, &b.url, b.secret.as_deref())
        .await.map_err(ApiError::Internal)?))
}

async fn delete_one(State(s): State<AppState>, user: AuthUser, Path(id): Path<Uuid>)
    -> Result<Json<bool>, ApiError>
{
    Ok(Json(traderview_db::webhooks::delete(&s.pool, user.id, id)
        .await.map_err(ApiError::Internal)?))
}

#[derive(Deserialize)]
struct ToggleBody { enabled: bool }

async fn toggle(State(s): State<AppState>, user: AuthUser, Path(id): Path<Uuid>, Json(b): Json<ToggleBody>)
    -> Result<Json<bool>, ApiError>
{
    Ok(Json(traderview_db::webhooks::toggle(&s.pool, user.id, id, b.enabled)
        .await.map_err(ApiError::Internal)?))
}

async fn test_fire(State(s): State<AppState>, user: AuthUser, Path(id): Path<Uuid>)
    -> Result<Json<bool>, ApiError>
{
    let payload = AlertPayload {
        title: "TraderView test alert".into(),
        message: "If you see this, your webhook is wired correctly.".into(),
        symbol: Some("TEST".into()),
        kind: "test".into(),
        url: Some("https://github.com/MenkeTechnologies/traderview".into()),
        fired_at: Utc::now(),
    };
    traderview_db::webhooks::fan_out(&s.pool, user.id, &[id], &payload).await;
    Ok(Json(true))
}
