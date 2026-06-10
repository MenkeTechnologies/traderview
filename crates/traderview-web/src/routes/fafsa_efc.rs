//! FAFSA / SAI estimator route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use traderview_db::fafsa_efc;

pub fn router() -> Router<AppState> {
    Router::new().route("/fafsa-efc/compute", post(compute))
}

async fn compute(
    State(_s): State<AppState>,
    _user: AuthUser,
    Json(input): Json<fafsa_efc::FafsaInput>,
) -> Result<Json<fafsa_efc::FafsaReport>, ApiError> {
    let nonneg = [
        ("parent_agi_usd", input.parent_agi_usd),
        ("parent_assets_usd", input.parent_assets_usd),
        ("student_agi_usd", input.student_agi_usd),
        ("student_assets_usd", input.student_assets_usd),
    ];
    for (n, v) in nonneg {
        if !v.is_finite() || v < 0.0 {
            return Err(ApiError::BadRequest(format!("{n} must be ≥ 0 and finite")));
        }
    }
    if input.household_size == 0 || input.household_size > 20 {
        return Err(ApiError::BadRequest(
            "household_size must be in [1, 20]".into(),
        ));
    }
    if input.dependents_in_college == 0 || input.dependents_in_college > 10 {
        return Err(ApiError::BadRequest(
            "dependents_in_college must be in [1, 10]".into(),
        ));
    }
    Ok(Json(fafsa_efc::compute(&input)))
}
