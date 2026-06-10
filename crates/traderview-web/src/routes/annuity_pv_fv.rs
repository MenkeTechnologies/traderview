//! Annuity PV/FV route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use traderview_db::annuity_pv_fv;

pub fn router() -> Router<AppState> {
    Router::new().route("/annuity-pv-fv/compute", post(compute))
}

async fn compute(
    State(_s): State<AppState>,
    _user: AuthUser,
    Json(input): Json<annuity_pv_fv::AnnuityInput>,
) -> Result<Json<annuity_pv_fv::AnnuityReport>, ApiError> {
    if !input.payment_per_period_usd.is_finite() || input.payment_per_period_usd < 0.0 {
        return Err(ApiError::BadRequest(
            "payment_per_period_usd must be ≥ 0".into(),
        ));
    }
    if !input.annual_rate_pct.is_finite() || input.annual_rate_pct < -50.0
        || input.annual_rate_pct > 50.0
    {
        return Err(ApiError::BadRequest(
            "annual_rate_pct must be in [-50, 50]".into(),
        ));
    }
    if input.periods_per_year == 0 || input.periods_per_year > 365 {
        return Err(ApiError::BadRequest(
            "periods_per_year must be in [1, 365]".into(),
        ));
    }
    if !input.years.is_finite() || input.years < 0.0 || input.years > 100.0 {
        return Err(ApiError::BadRequest(
            "years must be in [0, 100]".into(),
        ));
    }
    if input.annuity_kind != "ordinary" && input.annuity_kind != "due" {
        return Err(ApiError::BadRequest(
            "annuity_kind must be 'ordinary' or 'due'".into(),
        ));
    }
    Ok(Json(annuity_pv_fv::compute(&input)))
}
