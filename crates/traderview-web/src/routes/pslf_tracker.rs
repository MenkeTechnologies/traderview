//! PSLF tracker route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use traderview_db::pslf_tracker;

pub fn router() -> Router<AppState> {
    Router::new().route("/pslf-tracker/compute", post(compute))
}

async fn compute(
    State(_s): State<AppState>,
    _user: AuthUser,
    Json(input): Json<pslf_tracker::PslfInput>,
) -> Result<Json<pslf_tracker::PslfReport>, ApiError> {
    if input.qualifying_payments_made > 200 {
        return Err(ApiError::BadRequest(
            "qualifying_payments_made cap is 200".into(),
        ));
    }
    if !input.current_balance_usd.is_finite() || input.current_balance_usd < 0.0 {
        return Err(ApiError::BadRequest(
            "current_balance_usd must be ≥ 0".into(),
        ));
    }
    if !input.apr_pct.is_finite() || input.apr_pct < 0.0 || input.apr_pct > 30.0 {
        return Err(ApiError::BadRequest("apr_pct must be in [0, 30]".into()));
    }
    if !input.monthly_payment_usd.is_finite() || input.monthly_payment_usd < 0.0 {
        return Err(ApiError::BadRequest(
            "monthly_payment_usd must be ≥ 0".into(),
        ));
    }
    Ok(Json(pslf_tracker::compute(&input)))
}
