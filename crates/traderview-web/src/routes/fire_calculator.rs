//! FIRE retirement calculator route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use traderview_db::fire_calculator;

pub fn router() -> Router<AppState> {
    Router::new().route("/fire-calculator/compute", post(compute))
}

async fn compute(
    State(_s): State<AppState>,
    _user: AuthUser,
    Json(input): Json<fire_calculator::FireInput>,
) -> Result<Json<fire_calculator::FireReport>, ApiError> {
    if input.current_portfolio_usd < 0.0 {
        return Err(ApiError::BadRequest(
            "current_portfolio_usd must be ≥ 0".into(),
        ));
    }
    if input.monthly_contribution_usd < 0.0 {
        return Err(ApiError::BadRequest(
            "monthly_contribution_usd must be ≥ 0".into(),
        ));
    }
    if !(input.expected_annual_return_pct >= -50.0 && input.expected_annual_return_pct <= 50.0) {
        return Err(ApiError::BadRequest(
            "expected_annual_return_pct must be in [-50, 50]".into(),
        ));
    }
    if input.target_net_worth_usd <= 0.0 {
        return Err(ApiError::BadRequest(
            "target_net_worth_usd must be > 0".into(),
        ));
    }
    if !(input.current_age >= 1 && input.current_age <= 110) {
        return Err(ApiError::BadRequest(
            "current_age must be in [1, 110]".into(),
        ));
    }
    if !(input.target_retirement_age > input.current_age && input.target_retirement_age <= 110)
    {
        return Err(ApiError::BadRequest(
            "target_retirement_age must be > current_age and ≤ 110".into(),
        ));
    }
    if !(input.safe_withdrawal_rate_pct > 0.0 && input.safe_withdrawal_rate_pct <= 20.0) {
        return Err(ApiError::BadRequest(
            "safe_withdrawal_rate_pct must be in (0, 20]".into(),
        ));
    }
    Ok(Json(fire_calculator::compute(&input)))
}
