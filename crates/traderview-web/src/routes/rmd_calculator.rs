//! RMD calculator route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use traderview_db::rmd_calculator;

pub fn router() -> Router<AppState> {
    Router::new().route("/rmd-calculator/compute", post(compute))
}

async fn compute(
    State(_s): State<AppState>,
    _user: AuthUser,
    Json(input): Json<rmd_calculator::RmdInput>,
) -> Result<Json<rmd_calculator::RmdReport>, ApiError> {
    if !(1900..=2100).contains(&input.birth_year) {
        return Err(ApiError::BadRequest(
            "birth_year must be in [1900, 2100]".into(),
        ));
    }
    if input.current_age == 0 || input.current_age > 120 {
        return Err(ApiError::BadRequest(
            "current_age must be in [1, 120]".into(),
        ));
    }
    if !input.balance_usd.is_finite() || input.balance_usd < 0.0 {
        return Err(ApiError::BadRequest("balance_usd must be ≥ 0".into()));
    }
    if !input.expected_annual_return_pct.is_finite()
        || input.expected_annual_return_pct < -20.0
        || input.expected_annual_return_pct > 30.0
    {
        return Err(ApiError::BadRequest(
            "expected_annual_return_pct must be in [-20, 30]".into(),
        ));
    }
    if input.project_years > 60 {
        return Err(ApiError::BadRequest("project_years cap is 60".into()));
    }
    Ok(Json(rmd_calculator::compute(&input)))
}
