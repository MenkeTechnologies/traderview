//! Emergency-fund readiness route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use traderview_db::emergency_fund;

pub fn router() -> Router<AppState> {
    Router::new().route("/emergency-fund/compute", post(compute))
}

async fn compute(
    State(_s): State<AppState>,
    _user: AuthUser,
    Json(input): Json<emergency_fund::EmergencyFundInput>,
) -> Result<Json<emergency_fund::EmergencyFundReport>, ApiError> {
    if input.monthly_expenses_usd < 0.0 {
        return Err(ApiError::BadRequest(
            "monthly_expenses_usd must be ≥ 0".into(),
        ));
    }
    if input.current_fund_usd < 0.0 {
        return Err(ApiError::BadRequest("current_fund_usd must be ≥ 0".into()));
    }
    if !(input.target_months > 0.0 && input.target_months <= 60.0) {
        return Err(ApiError::BadRequest(
            "target_months must be in (0, 60]".into(),
        ));
    }
    if input.monthly_contribution_usd < 0.0 {
        return Err(ApiError::BadRequest(
            "monthly_contribution_usd must be ≥ 0".into(),
        ));
    }
    Ok(Json(emergency_fund::compute(&input)))
}
