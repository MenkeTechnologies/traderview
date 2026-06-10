//! Pension lump vs annuity route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use traderview_db::pension_lump_vs_annuity;

pub fn router() -> Router<AppState> {
    Router::new().route("/pension-lump-vs-annuity/compute", post(compute))
}

async fn compute(
    State(_s): State<AppState>,
    _user: AuthUser,
    Json(input): Json<pension_lump_vs_annuity::PensionInput>,
) -> Result<Json<pension_lump_vs_annuity::PensionReport>, ApiError> {
    if !input.lump_sum_usd.is_finite() || input.lump_sum_usd < 0.0 {
        return Err(ApiError::BadRequest("lump_sum_usd must be ≥ 0".into()));
    }
    if !input.monthly_annuity_usd.is_finite() || input.monthly_annuity_usd < 0.0 {
        return Err(ApiError::BadRequest(
            "monthly_annuity_usd must be ≥ 0".into(),
        ));
    }
    if input.current_age < 1 || input.current_age > 110 {
        return Err(ApiError::BadRequest(
            "current_age must be in [1, 110]".into(),
        ));
    }
    if input.life_expectancy_age <= input.current_age || input.life_expectancy_age > 120 {
        return Err(ApiError::BadRequest(
            "life_expectancy_age must be > current_age and ≤ 120".into(),
        ));
    }
    if !input.expected_real_return_pct.is_finite()
        || input.expected_real_return_pct < -20.0
        || input.expected_real_return_pct > 30.0
    {
        return Err(ApiError::BadRequest(
            "expected_real_return_pct must be in [-20, 30]".into(),
        ));
    }
    Ok(Json(pension_lump_vs_annuity::compute(&input)))
}
