//! Bond-tent route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use traderview_db::bond_tent;

pub fn router() -> Router<AppState> {
    Router::new().route("/bond-tent/compute", post(compute))
}

async fn compute(
    State(_s): State<AppState>,
    _user: AuthUser,
    Json(input): Json<bond_tent::BondTentInput>,
) -> Result<Json<bond_tent::BondTentReport>, ApiError> {
    if input.current_age == 0 || input.current_age > 110 {
        return Err(ApiError::BadRequest(
            "current_age must be in [1, 110]".into(),
        ));
    }
    if input.retirement_age < input.current_age || input.retirement_age > 110 {
        return Err(ApiError::BadRequest(
            "retirement_age must be ≥ current_age and ≤ 110".into(),
        ));
    }
    if input.horizon_age < input.current_age || input.horizon_age > 120 {
        return Err(ApiError::BadRequest(
            "horizon_age must be ≥ current_age and ≤ 120".into(),
        ));
    }
    let pcts = [
        ("pre_tent_bond_pct", input.pre_tent_bond_pct),
        ("tent_peak_bond_pct", input.tent_peak_bond_pct),
        ("post_tent_bond_pct", input.post_tent_bond_pct),
    ];
    for (n, v) in pcts {
        if !v.is_finite() || v < 0.0 || v > 100.0 {
            return Err(ApiError::BadRequest(format!("{n} must be in [0, 100]")));
        }
    }
    if input.tent_ramp_years > 50 || input.tent_descent_years > 50 {
        return Err(ApiError::BadRequest(
            "tent_ramp_years and tent_descent_years cap at 50".into(),
        ));
    }
    Ok(Json(bond_tent::compute(&input)))
}
