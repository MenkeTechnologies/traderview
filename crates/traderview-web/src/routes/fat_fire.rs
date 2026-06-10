//! Fat FIRE route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use traderview_db::fat_fire;

pub fn router() -> Router<AppState> {
    Router::new().route("/fat-fire/compute", post(compute))
}

async fn compute(
    State(_s): State<AppState>,
    _user: AuthUser,
    Json(input): Json<fat_fire::FatFireInput>,
) -> Result<Json<fat_fire::FatFireReport>, ApiError> {
    if !input.current_nw_usd.is_finite() || input.current_nw_usd < 0.0 {
        return Err(ApiError::BadRequest("current_nw_usd must be ≥ 0".into()));
    }
    if !input.annual_expenses_usd.is_finite() || input.annual_expenses_usd < 0.0 {
        return Err(ApiError::BadRequest(
            "annual_expenses_usd must be ≥ 0".into(),
        ));
    }
    if !input.safe_withdrawal_rate_pct.is_finite()
        || input.safe_withdrawal_rate_pct <= 0.0
        || input.safe_withdrawal_rate_pct > 20.0
    {
        return Err(ApiError::BadRequest(
            "safe_withdrawal_rate_pct must be in (0, 20]".into(),
        ));
    }
    if !input.expected_real_return_pct.is_finite()
        || input.expected_real_return_pct < -10.0
        || input.expected_real_return_pct > 20.0
    {
        return Err(ApiError::BadRequest(
            "expected_real_return_pct must be in [-10, 20]".into(),
        ));
    }
    if !input.monthly_contribution_usd.is_finite() || input.monthly_contribution_usd < 0.0 {
        return Err(ApiError::BadRequest(
            "monthly_contribution_usd must be ≥ 0".into(),
        ));
    }
    Ok(Json(fat_fire::compute(&input)))
}
