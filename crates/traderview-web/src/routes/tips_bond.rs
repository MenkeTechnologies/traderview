//! TIPS bond route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use traderview_db::tips_bond;

pub fn router() -> Router<AppState> {
    Router::new().route("/tips-bond/compute", post(compute))
}

async fn compute(
    State(_s): State<AppState>,
    _user: AuthUser,
    Json(input): Json<tips_bond::TipsInput>,
) -> Result<Json<tips_bond::TipsReport>, ApiError> {
    if !input.face_value_usd.is_finite() || input.face_value_usd < 0.0 {
        return Err(ApiError::BadRequest("face_value_usd must be ≥ 0".into()));
    }
    if !input.real_coupon_rate_pct.is_finite()
        || input.real_coupon_rate_pct < -10.0
        || input.real_coupon_rate_pct > 20.0
    {
        return Err(ApiError::BadRequest(
            "real_coupon_rate_pct must be in [-10, 20]".into(),
        ));
    }
    if input.term_years == 0 || input.term_years > 50 {
        return Err(ApiError::BadRequest(
            "term_years must be in [1, 50]".into(),
        ));
    }
    if !input.annual_cpi_inflation_pct.is_finite()
        || input.annual_cpi_inflation_pct < -20.0
        || input.annual_cpi_inflation_pct > 30.0
    {
        return Err(ApiError::BadRequest(
            "annual_cpi_inflation_pct must be in [-20, 30]".into(),
        ));
    }
    Ok(Json(tips_bond::compute(&input)))
}
