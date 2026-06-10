//! Mortgage refinance route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use traderview_db::mortgage_refinance;

pub fn router() -> Router<AppState> {
    Router::new().route("/mortgage-refinance/compute", post(compute))
}

async fn compute(
    State(_s): State<AppState>,
    _user: AuthUser,
    Json(input): Json<mortgage_refinance::RefinanceInput>,
) -> Result<Json<mortgage_refinance::RefinanceReport>, ApiError> {
    let fields = [
        ("current_balance_usd", input.current_balance_usd),
        ("closing_costs_usd", input.closing_costs_usd),
        ("cash_out_usd", input.cash_out_usd),
    ];
    for (n, v) in fields {
        if !v.is_finite() || v < 0.0 {
            return Err(ApiError::BadRequest(format!("{n} must be ≥ 0 and finite")));
        }
    }
    if !input.current_apr_pct.is_finite()
        || input.current_apr_pct < 0.0
        || input.current_apr_pct > 30.0
    {
        return Err(ApiError::BadRequest(
            "current_apr_pct must be in [0, 30]".into(),
        ));
    }
    if !input.new_apr_pct.is_finite() || input.new_apr_pct < 0.0 || input.new_apr_pct > 30.0 {
        return Err(ApiError::BadRequest("new_apr_pct must be in [0, 30]".into()));
    }
    if input.current_remaining_months == 0 || input.current_remaining_months > 600 {
        return Err(ApiError::BadRequest(
            "current_remaining_months must be in [1, 600]".into(),
        ));
    }
    if input.new_term_months == 0 || input.new_term_months > 600 {
        return Err(ApiError::BadRequest(
            "new_term_months must be in [1, 600]".into(),
        ));
    }
    if input.planning_horizon_months > 600 {
        return Err(ApiError::BadRequest(
            "planning_horizon_months must be ≤ 600".into(),
        ));
    }
    Ok(Json(mortgage_refinance::compute(&input)))
}
