//! Credit utilization route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use traderview_db::credit_utilization;

pub fn router() -> Router<AppState> {
    Router::new().route("/credit-utilization/compute", post(compute))
}

async fn compute(
    State(_s): State<AppState>,
    _user: AuthUser,
    Json(input): Json<credit_utilization::CreditUtilizationInput>,
) -> Result<Json<credit_utilization::CreditUtilizationReport>, ApiError> {
    if input.cards.len() > 100 {
        return Err(ApiError::BadRequest("cards cap is 100".into()));
    }
    for c in &input.cards {
        if !c.balance_usd.is_finite() || c.balance_usd < 0.0 {
            return Err(ApiError::BadRequest("balance_usd must be ≥ 0".into()));
        }
        if !c.limit_usd.is_finite() || c.limit_usd < 0.0 {
            return Err(ApiError::BadRequest("limit_usd must be ≥ 0".into()));
        }
    }
    Ok(Json(credit_utilization::compute(&input)))
}
