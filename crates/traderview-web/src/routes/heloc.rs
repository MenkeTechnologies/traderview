//! HELOC route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use traderview_db::heloc;

pub fn router() -> Router<AppState> {
    Router::new().route("/heloc/compute", post(compute))
}

async fn compute(
    State(_s): State<AppState>,
    _user: AuthUser,
    Json(input): Json<heloc::HelocInput>,
) -> Result<Json<heloc::HelocReport>, ApiError> {
    if !input.line_size_usd.is_finite() || input.line_size_usd < 0.0 {
        return Err(ApiError::BadRequest("line_size_usd must be ≥ 0".into()));
    }
    if !input.current_balance_usd.is_finite() || input.current_balance_usd < 0.0 {
        return Err(ApiError::BadRequest(
            "current_balance_usd must be ≥ 0".into(),
        ));
    }
    if !input.variable_apr_pct.is_finite()
        || input.variable_apr_pct < 0.0
        || input.variable_apr_pct > 30.0
    {
        return Err(ApiError::BadRequest(
            "variable_apr_pct must be in [0, 30]".into(),
        ));
    }
    if input.draw_period_months == 0 || input.draw_period_months > 360 {
        return Err(ApiError::BadRequest(
            "draw_period_months must be in [1, 360]".into(),
        ));
    }
    if input.repayment_period_months == 0 || input.repayment_period_months > 480 {
        return Err(ApiError::BadRequest(
            "repayment_period_months must be in [1, 480]".into(),
        ));
    }
    if !input.draw_phase_min_pct.is_finite()
        || input.draw_phase_min_pct < 0.0
        || input.draw_phase_min_pct > 10.0
    {
        return Err(ApiError::BadRequest(
            "draw_phase_min_pct must be in [0, 10]".into(),
        ));
    }
    if !input.monthly_voluntary_principal_usd.is_finite()
        || input.monthly_voluntary_principal_usd < 0.0
    {
        return Err(ApiError::BadRequest(
            "monthly_voluntary_principal_usd must be ≥ 0".into(),
        ));
    }
    Ok(Json(heloc::compute(&input)))
}
