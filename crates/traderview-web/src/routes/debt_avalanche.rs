//! Debt avalanche planner route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use traderview_db::debt_avalanche;

pub fn router() -> Router<AppState> {
    Router::new().route("/debt-avalanche/compute", post(compute))
}

async fn compute(
    State(_s): State<AppState>,
    _user: AuthUser,
    Json(input): Json<debt_avalanche::DebtAvalancheInput>,
) -> Result<Json<debt_avalanche::DebtAvalancheReport>, ApiError> {
    if input.debts.len() > 100 {
        return Err(ApiError::BadRequest("debts cap is 100".into()));
    }
    if !input.extra_payment_usd.is_finite() || input.extra_payment_usd < 0.0 {
        return Err(ApiError::BadRequest(
            "extra_payment_usd must be ≥ 0".into(),
        ));
    }
    for d in &input.debts {
        if !d.balance_usd.is_finite() || d.balance_usd < 0.0 {
            return Err(ApiError::BadRequest("balance_usd must be ≥ 0".into()));
        }
        if !d.apr_pct.is_finite() || d.apr_pct < 0.0 || d.apr_pct > 100.0 {
            return Err(ApiError::BadRequest("apr_pct must be in [0, 100]".into()));
        }
        if !d.min_payment_usd.is_finite() || d.min_payment_usd < 0.0 {
            return Err(ApiError::BadRequest(
                "min_payment_usd must be ≥ 0".into(),
            ));
        }
    }
    Ok(Json(debt_avalanche::compute(&input)))
}
