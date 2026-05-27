use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use traderview_db::sector_rotation::RotationReport;

pub fn router() -> Router<AppState> {
    Router::new().route("/sector-rotation", get(report))
}

async fn report(
    State(s): State<AppState>,
    _u: AuthUser,
) -> Result<Json<RotationReport>, ApiError> {
    Ok(Json(traderview_db::sector_rotation::report(&s.pool)
        .await.map_err(ApiError::Internal)?))
}
