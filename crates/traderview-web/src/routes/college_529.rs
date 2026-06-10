//! 529 college planner route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use traderview_db::college_529;

pub fn router() -> Router<AppState> {
    Router::new().route("/college-529/compute", post(compute))
}

async fn compute(
    State(_s): State<AppState>,
    _user: AuthUser,
    Json(input): Json<college_529::College529Input>,
) -> Result<Json<college_529::College529Report>, ApiError> {
    if input.child_age_years > 25 || input.college_start_age > 30 {
        return Err(ApiError::BadRequest(
            "child_age_years ≤ 25 and college_start_age ≤ 30".into(),
        ));
    }
    if !input.annual_cost_today_usd.is_finite() || input.annual_cost_today_usd < 0.0 {
        return Err(ApiError::BadRequest(
            "annual_cost_today_usd must be ≥ 0".into(),
        ));
    }
    if !input.tuition_inflation_pct.is_finite()
        || input.tuition_inflation_pct < -10.0
        || input.tuition_inflation_pct > 30.0
    {
        return Err(ApiError::BadRequest(
            "tuition_inflation_pct must be in [-10, 30]".into(),
        ));
    }
    if input.years_in_college == 0 || input.years_in_college > 10 {
        return Err(ApiError::BadRequest(
            "years_in_college must be in [1, 10]".into(),
        ));
    }
    if !input.current_529_balance_usd.is_finite() || input.current_529_balance_usd < 0.0 {
        return Err(ApiError::BadRequest(
            "current_529_balance_usd must be ≥ 0".into(),
        ));
    }
    if !input.expected_annual_return_pct.is_finite()
        || input.expected_annual_return_pct < -20.0
        || input.expected_annual_return_pct > 30.0
    {
        return Err(ApiError::BadRequest(
            "expected_annual_return_pct must be in [-20, 30]".into(),
        ));
    }
    if !input.current_monthly_contribution_usd.is_finite()
        || input.current_monthly_contribution_usd < 0.0
    {
        return Err(ApiError::BadRequest(
            "current_monthly_contribution_usd must be ≥ 0".into(),
        ));
    }
    Ok(Json(college_529::compute(&input)))
}
