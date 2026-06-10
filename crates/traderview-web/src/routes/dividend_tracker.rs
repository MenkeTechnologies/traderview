//! Dividend total-return tracker route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use traderview_db::dividend_tracker;

pub fn router() -> Router<AppState> {
    Router::new().route("/dividend-tracker/report", get(report))
}

async fn report(
    State(s): State<AppState>,
    user: AuthUser,
) -> Result<Json<dividend_tracker::DividendPortfolioReport>, ApiError> {
    Ok(Json(
        dividend_tracker::compute_report(&s.pool, user.id).await?,
    ))
}
