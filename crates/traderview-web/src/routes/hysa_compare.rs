//! HYSA compare route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use traderview_db::hysa_compare;

pub fn router() -> Router<AppState> {
    Router::new().route("/hysa-compare/compute", post(compute))
}

async fn compute(
    State(_s): State<AppState>,
    _user: AuthUser,
    Json(input): Json<hysa_compare::HysaCompareInput>,
) -> Result<Json<hysa_compare::HysaCompareReport>, ApiError> {
    if !input.deposit_usd.is_finite() || input.deposit_usd < 0.0 {
        return Err(ApiError::BadRequest("deposit_usd must be ≥ 0".into()));
    }
    if input.months > 600 {
        return Err(ApiError::BadRequest("months cap is 600".into()));
    }
    if input.banks.len() > 50 {
        return Err(ApiError::BadRequest("banks cap is 50".into()));
    }
    for b in &input.banks {
        if !b.apy_pct.is_finite() || b.apy_pct < 0.0 || b.apy_pct > 30.0 {
            return Err(ApiError::BadRequest("apy_pct must be in [0, 30]".into()));
        }
        if !b.monthly_fee_usd.is_finite() || b.monthly_fee_usd < 0.0 {
            return Err(ApiError::BadRequest(
                "monthly_fee_usd must be ≥ 0".into(),
            ));
        }
        if !b.min_balance_usd.is_finite() || b.min_balance_usd < 0.0 {
            return Err(ApiError::BadRequest(
                "min_balance_usd must be ≥ 0".into(),
            ));
        }
    }
    Ok(Json(hysa_compare::compute(&input)))
}
