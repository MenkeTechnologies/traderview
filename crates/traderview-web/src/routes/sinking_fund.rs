//! Sinking-fund planner route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use traderview_db::sinking_fund;

pub fn router() -> Router<AppState> {
    Router::new().route("/sinking-fund/compute", post(compute))
}

async fn compute(
    State(_s): State<AppState>,
    _user: AuthUser,
    Json(input): Json<sinking_fund::SinkingFundInput>,
) -> Result<Json<sinking_fund::SinkingFundReport>, ApiError> {
    if input.goals.len() > 500 {
        return Err(ApiError::BadRequest("goals cap is 500".into()));
    }
    for g in &input.goals {
        if !g.target_usd.is_finite() || g.target_usd < 0.0 {
            return Err(ApiError::BadRequest("target_usd must be ≥ 0".into()));
        }
        if !g.current_balance_usd.is_finite() || g.current_balance_usd < 0.0 {
            return Err(ApiError::BadRequest(
                "current_balance_usd must be ≥ 0".into(),
            ));
        }
        if !g.target_date_months.is_finite() || g.target_date_months < 0.0 {
            return Err(ApiError::BadRequest(
                "target_date_months must be ≥ 0".into(),
            ));
        }
        if !g.monthly_contribution_usd.is_finite() || g.monthly_contribution_usd < 0.0 {
            return Err(ApiError::BadRequest(
                "monthly_contribution_usd must be ≥ 0".into(),
            ));
        }
    }
    Ok(Json(sinking_fund::compute(&input)))
}
