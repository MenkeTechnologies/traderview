//! Home maintenance route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use traderview_db::home_maintenance;

pub fn router() -> Router<AppState> {
    Router::new().route("/home-maintenance/compute", post(compute))
}

async fn compute(
    State(_s): State<AppState>,
    _user: AuthUser,
    Json(input): Json<home_maintenance::HomeMaintenanceInput>,
) -> Result<Json<home_maintenance::HomeMaintenanceReport>, ApiError> {
    if !input.home_value_usd.is_finite() || input.home_value_usd < 0.0 {
        return Err(ApiError::BadRequest("home_value_usd must be ≥ 0".into()));
    }
    if !input.general_pct_of_value.is_finite()
        || input.general_pct_of_value < 0.0
        || input.general_pct_of_value > 10.0
    {
        return Err(ApiError::BadRequest(
            "general_pct_of_value must be in [0, 10]".into(),
        ));
    }
    if !(1900..=2200).contains(&input.current_year) {
        return Err(ApiError::BadRequest(
            "current_year must be in [1900, 2200]".into(),
        ));
    }
    if input.systems.len() > 100 {
        return Err(ApiError::BadRequest("systems cap is 100".into()));
    }
    for s in &input.systems {
        if !(1800..=2200).contains(&s.install_year) {
            return Err(ApiError::BadRequest(
                "install_year must be in [1800, 2200]".into(),
            ));
        }
        if s.expected_life_years == 0 || s.expected_life_years > 200 {
            return Err(ApiError::BadRequest(
                "expected_life_years must be in [1, 200]".into(),
            ));
        }
        if !s.replacement_cost_usd.is_finite() || s.replacement_cost_usd < 0.0 {
            return Err(ApiError::BadRequest(
                "replacement_cost_usd must be ≥ 0".into(),
            ));
        }
    }
    Ok(Json(home_maintenance::compute(&input)))
}
