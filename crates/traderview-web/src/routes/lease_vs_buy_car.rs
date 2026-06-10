//! Lease vs buy car route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use traderview_db::lease_vs_buy_car;

pub fn router() -> Router<AppState> {
    Router::new().route("/lease-vs-buy-car/compute", post(compute))
}

async fn compute(
    State(_s): State<AppState>,
    _user: AuthUser,
    Json(input): Json<lease_vs_buy_car::LeaseVsBuyInput>,
) -> Result<Json<lease_vs_buy_car::LeaseVsBuyReport>, ApiError> {
    let nonneg = [
        ("vehicle_price_usd", input.vehicle_price_usd),
        ("monthly_lease_payment_usd", input.monthly_lease_payment_usd),
        ("drive_off_cost_usd", input.drive_off_cost_usd),
        ("disposition_fee_usd", input.disposition_fee_usd),
        ("down_payment_usd", input.down_payment_usd),
    ];
    for (n, v) in nonneg {
        if !v.is_finite() || v < 0.0 {
            return Err(ApiError::BadRequest(format!("{n} must be ≥ 0 and finite")));
        }
    }
    if !input.sales_tax_pct.is_finite() || input.sales_tax_pct < 0.0 || input.sales_tax_pct > 30.0
    {
        return Err(ApiError::BadRequest(
            "sales_tax_pct must be in [0, 30]".into(),
        ));
    }
    if input.lease_term_months == 0 || input.lease_term_months > 120 {
        return Err(ApiError::BadRequest(
            "lease_term_months must be in [1, 120]".into(),
        ));
    }
    if input.loan_term_months == 0 || input.loan_term_months > 120 {
        return Err(ApiError::BadRequest(
            "loan_term_months must be in [1, 120]".into(),
        ));
    }
    if !input.apr_pct.is_finite() || input.apr_pct < 0.0 || input.apr_pct > 50.0 {
        return Err(ApiError::BadRequest("apr_pct must be in [0, 50]".into()));
    }
    if !input.residual_at_horizon_pct.is_finite()
        || input.residual_at_horizon_pct < 0.0
        || input.residual_at_horizon_pct > 100.0
    {
        return Err(ApiError::BadRequest(
            "residual_at_horizon_pct must be in [0, 100]".into(),
        ));
    }
    if input.analysis_years == 0 || input.analysis_years > 30 {
        return Err(ApiError::BadRequest(
            "analysis_years must be in [1, 30]".into(),
        ));
    }
    if !input.investment_return_pct.is_finite()
        || input.investment_return_pct < -20.0
        || input.investment_return_pct > 30.0
    {
        return Err(ApiError::BadRequest(
            "investment_return_pct must be in [-20, 30]".into(),
        ));
    }
    Ok(Json(lease_vs_buy_car::compute(&input)))
}
