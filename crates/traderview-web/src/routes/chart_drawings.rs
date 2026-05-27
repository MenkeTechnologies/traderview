use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::routing::{delete, get};
use axum::{Json, Router};
use traderview_db::chart_drawings::{ChartDrawing, DrawingInput};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/chart-drawings/:symbol",
            get(list).post(create).delete(delete_all),
        )
        .route("/chart-drawings/by-id/:id", delete(delete_one))
}

async fn list(
    State(s): State<AppState>,
    u: AuthUser,
    Path(symbol): Path<String>,
) -> Result<Json<Vec<ChartDrawing>>, ApiError> {
    let sym = symbol.to_uppercase();
    Ok(Json(
        traderview_db::chart_drawings::list_for_symbol(&s.pool, u.id, &sym)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

async fn create(
    State(s): State<AppState>,
    u: AuthUser,
    Path(symbol): Path<String>,
    Json(body): Json<DrawingInput>,
) -> Result<Json<ChartDrawing>, ApiError> {
    let sym = symbol.to_uppercase();
    Ok(Json(
        traderview_db::chart_drawings::create(&s.pool, u.id, &sym, &body)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

async fn delete_one(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let ok = traderview_db::chart_drawings::delete(&s.pool, u.id, id)
        .await
        .map_err(ApiError::Internal)?;
    Ok(Json(serde_json::json!({ "deleted": ok })))
}

async fn delete_all(
    State(s): State<AppState>,
    u: AuthUser,
    Path(symbol): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let sym = symbol.to_uppercase();
    let n = traderview_db::chart_drawings::delete_all_for_symbol(&s.pool, u.id, &sym)
        .await
        .map_err(ApiError::Internal)?;
    Ok(Json(serde_json::json!({ "deleted_count": n })))
}
