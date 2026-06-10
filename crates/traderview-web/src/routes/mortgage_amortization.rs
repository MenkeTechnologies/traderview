//! Mortgage amortization route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use traderview_db::mortgage_amortization;

pub fn router() -> Router<AppState> {
    Router::new().route("/mortgage-amortization/compute", post(compute))
}

async fn compute(
    State(_s): State<AppState>,
    _user: AuthUser,
    Json(input): Json<mortgage_amortization::MortgageInput>,
) -> Result<Json<mortgage_amortization::MortgageReport>, ApiError> {
    let fields = [
        ("home_price_usd", input.home_price_usd),
        ("down_payment_usd", input.down_payment_usd),
        ("annual_property_tax_usd", input.annual_property_tax_usd),
        ("annual_insurance_usd", input.annual_insurance_usd),
        ("monthly_hoa_usd", input.monthly_hoa_usd),
        ("extra_principal_usd", input.extra_principal_usd),
    ];
    for (n, v) in fields {
        if !v.is_finite() || v < 0.0 {
            return Err(ApiError::BadRequest(format!("{n} must be ≥ 0 and finite")));
        }
    }
    if !input.apr_pct.is_finite() || input.apr_pct < 0.0 || input.apr_pct > 30.0 {
        return Err(ApiError::BadRequest("apr_pct must be in [0, 30]".into()));
    }
    if input.term_months == 0 || input.term_months > 600 {
        return Err(ApiError::BadRequest(
            "term_months must be in [1, 600]".into(),
        ));
    }
    if !input.pmi_annual_rate_pct.is_finite()
        || input.pmi_annual_rate_pct < 0.0
        || input.pmi_annual_rate_pct > 10.0
    {
        return Err(ApiError::BadRequest(
            "pmi_annual_rate_pct must be in [0, 10]".into(),
        ));
    }
    Ok(Json(mortgage_amortization::compute(&input)))
}
