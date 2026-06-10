//! Envelope budgeting route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use traderview_db::envelope_budget;

pub fn router() -> Router<AppState> {
    Router::new().route("/envelope-budget/compute", post(compute))
}

async fn compute(
    State(_s): State<AppState>,
    _user: AuthUser,
    Json(input): Json<envelope_budget::EnvelopeBudgetInput>,
) -> Result<Json<envelope_budget::EnvelopeBudgetReport>, ApiError> {
    if input.envelopes.len() > 500 {
        return Err(ApiError::BadRequest("envelopes cap is 500".into()));
    }
    for env in &input.envelopes {
        if !env.period_allotment_usd.is_finite() || env.period_allotment_usd < 0.0 {
            return Err(ApiError::BadRequest(
                "period_allotment_usd must be ≥ 0".into(),
            ));
        }
        if !env.starting_balance_usd.is_finite() {
            return Err(ApiError::BadRequest(
                "starting_balance_usd must be finite".into(),
            ));
        }
        if !env.spent_this_period_usd.is_finite() || env.spent_this_period_usd < 0.0 {
            return Err(ApiError::BadRequest(
                "spent_this_period_usd must be ≥ 0".into(),
            ));
        }
    }
    Ok(Json(envelope_budget::compute(&input)))
}
