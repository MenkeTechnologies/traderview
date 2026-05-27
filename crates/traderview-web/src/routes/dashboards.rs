use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use traderview_db::dashboards::{Dashboard, DashboardInput};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/dashboards", get(list).post(create))
        .route("/dashboards/:id", get(one).put(update).delete(remove))
}

async fn list(
    State(s): State<AppState>,
    u: AuthUser,
) -> Result<Json<Vec<Dashboard>>, ApiError> {
    Ok(Json(traderview_db::dashboards::list_for_user(&s.pool, u.id)
        .await.map_err(ApiError::Internal)?))
}

async fn one(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Dashboard>, ApiError> {
    traderview_db::dashboards::get(&s.pool, u.id, id)
        .await.map_err(ApiError::Internal)?
        .map(Json).ok_or(ApiError::NotFound)
}

async fn create(
    State(s): State<AppState>,
    u: AuthUser,
    Json(body): Json<DashboardInput>,
) -> Result<Json<Dashboard>, ApiError> {
    if body.name.trim().is_empty() {
        return Err(ApiError::BadRequest("name required".into()));
    }
    Ok(Json(traderview_db::dashboards::create(&s.pool, u.id, &body)
        .await.map_err(ApiError::Internal)?))
}

async fn update(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<DashboardInput>,
) -> Result<Json<Dashboard>, ApiError> {
    traderview_db::dashboards::update(&s.pool, u.id, id, &body)
        .await.map_err(ApiError::Internal)?
        .map(Json).ok_or(ApiError::NotFound)
}

async fn remove(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let ok = traderview_db::dashboards::delete(&s.pool, u.id, id)
        .await.map_err(ApiError::Internal)?;
    if !ok { return Err(ApiError::NotFound); }
    Ok(Json(serde_json::json!({ "deleted": true })))
}

