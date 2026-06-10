//! Paper-account rebalancer routes.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use traderview_db::paper_rebalance;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/paper-rebalance/targets",
            get(list_targets).post(create_target),
        )
        .route("/paper-rebalance/targets/:id", delete(delete_target))
        .route("/paper-rebalance/plan/:id", post(plan))
}

async fn list_targets(
    State(s): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<paper_rebalance::PaperRebalanceTarget>>, ApiError> {
    Ok(Json(paper_rebalance::list(&s.pool, user.id).await?))
}

async fn create_target(
    State(s): State<AppState>,
    user: AuthUser,
    Json(dto): Json<paper_rebalance::PaperRebalanceTargetInput>,
) -> Result<Json<paper_rebalance::PaperRebalanceTarget>, ApiError> {
    Ok(Json(paper_rebalance::upsert(&s.pool, user.id, &dto).await?))
}

async fn delete_target(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let deleted = paper_rebalance::delete(&s.pool, user.id, id).await?;
    Ok(Json(serde_json::json!({"deleted": deleted})))
}

async fn plan(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<paper_rebalance::PaperRebalancePlan>, ApiError> {
    paper_rebalance::plan(&s.pool, user.id, id)
        .await?
        .map(Json)
        .ok_or(ApiError::NotFound)
}
