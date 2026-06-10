//! I-bond route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use traderview_db::i_bond;

pub fn router() -> Router<AppState> {
    Router::new().route("/i-bond/compute", post(compute))
}

async fn compute(
    State(_s): State<AppState>,
    _user: AuthUser,
    Json(input): Json<i_bond::IBondInput>,
) -> Result<Json<i_bond::IBondReport>, ApiError> {
    if !input.purchase_amount_usd.is_finite() || input.purchase_amount_usd < 0.0 {
        return Err(ApiError::BadRequest(
            "purchase_amount_usd must be ≥ 0".into(),
        ));
    }
    if !input.fixed_rate_pct.is_finite() || input.fixed_rate_pct < -10.0
        || input.fixed_rate_pct > 20.0
    {
        return Err(ApiError::BadRequest(
            "fixed_rate_pct must be in [-10, 20]".into(),
        ));
    }
    if !input.semi_annual_inflation_pct.is_finite()
        || input.semi_annual_inflation_pct < -20.0
        || input.semi_annual_inflation_pct > 20.0
    {
        return Err(ApiError::BadRequest(
            "semi_annual_inflation_pct must be in [-20, 20]".into(),
        ));
    }
    if input.holding_period_months > 600 {
        return Err(ApiError::BadRequest(
            "holding_period_months cap is 600".into(),
        ));
    }
    Ok(Json(i_bond::compute(&input)))
}
