use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use traderview_db::accounts_overview::OverviewReport;

pub fn router() -> Router<AppState> {
    Router::new().route("/accounts/overview", get(overview))
}

async fn overview(
    State(s): State<AppState>,
    u: AuthUser,
) -> Result<Json<OverviewReport>, ApiError> {
    Ok(Json(
        traderview_db::accounts_overview::report(&s.pool, u.id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}
