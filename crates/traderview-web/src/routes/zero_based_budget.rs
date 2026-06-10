//! Zero-based budget route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use traderview_db::zero_based_budget;

pub fn router() -> Router<AppState> {
    Router::new().route("/zero-based-budget/compute", post(compute))
}

async fn compute(
    State(_s): State<AppState>,
    _user: AuthUser,
    Json(input): Json<zero_based_budget::ZeroBasedBudgetInput>,
) -> Result<Json<zero_based_budget::ZeroBasedBudgetReport>, ApiError> {
    if !input.monthly_income_usd.is_finite() || input.monthly_income_usd < 0.0 {
        return Err(ApiError::BadRequest("monthly_income_usd must be ≥ 0".into()));
    }
    if input.categories.len() > 500 {
        return Err(ApiError::BadRequest("categories cap is 500".into()));
    }
    for c in &input.categories {
        if !c.planned_usd.is_finite() || c.planned_usd < 0.0 {
            return Err(ApiError::BadRequest("planned_usd must be ≥ 0".into()));
        }
        if !c.actual_usd.is_finite() || c.actual_usd < 0.0 {
            return Err(ApiError::BadRequest("actual_usd must be ≥ 0".into()));
        }
    }
    Ok(Json(zero_based_budget::compute(&input)))
}
