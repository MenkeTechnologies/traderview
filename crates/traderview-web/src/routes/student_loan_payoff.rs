//! Student loan payoff route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use traderview_db::student_loan_payoff;

pub fn router() -> Router<AppState> {
    Router::new().route("/student-loan-payoff/compute", post(compute))
}

async fn compute(
    State(_s): State<AppState>,
    _user: AuthUser,
    Json(input): Json<student_loan_payoff::StudentLoanInput>,
) -> Result<Json<student_loan_payoff::StudentLoanReport>, ApiError> {
    if !input.balance_usd.is_finite() || input.balance_usd < 0.0 {
        return Err(ApiError::BadRequest("balance_usd must be ≥ 0".into()));
    }
    if !input.apr_pct.is_finite() || input.apr_pct < 0.0 || input.apr_pct > 30.0 {
        return Err(ApiError::BadRequest("apr_pct must be in [0, 30]".into()));
    }
    if !input.agi_annual_usd.is_finite() || input.agi_annual_usd < 0.0 {
        return Err(ApiError::BadRequest("agi_annual_usd must be ≥ 0".into()));
    }
    if input.household_size == 0 || input.household_size > 20 {
        return Err(ApiError::BadRequest(
            "household_size must be in [1, 20]".into(),
        ));
    }
    if !input.fpl_multiplier.is_finite()
        || input.fpl_multiplier < 1.0
        || input.fpl_multiplier > 4.0
    {
        return Err(ApiError::BadRequest(
            "fpl_multiplier must be in [1.0, 4.0]".into(),
        ));
    }
    Ok(Json(student_loan_payoff::compute(&input)))
}
