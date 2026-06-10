//! Permanent Portfolio / All-Weather comparison route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use traderview_db::permanent_portfolio;

pub fn router() -> Router<AppState> {
    Router::new().route("/permanent-portfolio/compare", get(compare))
}

#[derive(Deserialize)]
struct CompareQ {
    #[serde(default = "default_days")]
    days_back: i64,
}
fn default_days() -> i64 {
    1825
}

async fn compare(
    State(s): State<AppState>,
    _user: AuthUser,
    Query(q): Query<CompareQ>,
) -> Result<Json<permanent_portfolio::PortfolioComparisonReport>, ApiError> {
    if !(q.days_back >= 365 && q.days_back <= 7300) {
        return Err(ApiError::BadRequest(
            "days_back must be in [365, 7300]".into(),
        ));
    }
    Ok(Json(
        permanent_portfolio::compare(&s.pool, q.days_back).await,
    ))
}
