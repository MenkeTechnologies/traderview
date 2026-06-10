//! Auto loan route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use traderview_db::auto_loan;

pub fn router() -> Router<AppState> {
    Router::new().route("/auto-loan/compute", post(compute))
}

async fn compute(
    State(_s): State<AppState>,
    _user: AuthUser,
    Json(input): Json<auto_loan::AutoLoanInput>,
) -> Result<Json<auto_loan::AutoLoanReport>, ApiError> {
    if !input.vehicle_price_usd.is_finite() || input.vehicle_price_usd < 0.0 {
        return Err(ApiError::BadRequest("vehicle_price_usd must be ≥ 0".into()));
    }
    if !input.down_payment_usd.is_finite() || input.down_payment_usd < 0.0 {
        return Err(ApiError::BadRequest("down_payment_usd must be ≥ 0".into()));
    }
    if !input.trade_in_credit_usd.is_finite() || input.trade_in_credit_usd < 0.0 {
        return Err(ApiError::BadRequest(
            "trade_in_credit_usd must be ≥ 0".into(),
        ));
    }
    if !input.sales_tax_pct.is_finite() || input.sales_tax_pct < 0.0 || input.sales_tax_pct > 30.0
    {
        return Err(ApiError::BadRequest(
            "sales_tax_pct must be in [0, 30]".into(),
        ));
    }
    if !input.apr_pct.is_finite() || input.apr_pct < 0.0 || input.apr_pct > 50.0 {
        return Err(ApiError::BadRequest("apr_pct must be in [0, 50]".into()));
    }
    if input.term_months == 0 || input.term_months > 120 {
        return Err(ApiError::BadRequest(
            "term_months must be in [1, 120]".into(),
        ));
    }
    Ok(Json(auto_loan::compute(&input)))
}
