//! Car TCO route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use traderview_db::car_tco;

pub fn router() -> Router<AppState> {
    Router::new().route("/car-tco/compute", post(compute))
}

async fn compute(
    State(_s): State<AppState>,
    _user: AuthUser,
    Json(input): Json<car_tco::CarTcoInput>,
) -> Result<Json<car_tco::CarTcoReport>, ApiError> {
    let nonneg = [
        ("purchase_price_usd", input.purchase_price_usd),
        ("down_payment_usd", input.down_payment_usd),
        ("mpg", input.mpg),
        ("fuel_price_per_gallon_usd", input.fuel_price_per_gallon_usd),
        ("insurance_annual_usd", input.insurance_annual_usd),
        ("maintenance_annual_usd", input.maintenance_annual_usd),
        ("registration_annual_usd", input.registration_annual_usd),
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
    if !input.apr_pct.is_finite() || input.apr_pct < 0.0 || input.apr_pct > 50.0 {
        return Err(ApiError::BadRequest("apr_pct must be in [0, 50]".into()));
    }
    if input.loan_term_months > 120 {
        return Err(ApiError::BadRequest("loan_term_months cap is 120".into()));
    }
    if input.hold_years == 0 || input.hold_years > 30 {
        return Err(ApiError::BadRequest("hold_years must be in [1, 30]".into()));
    }
    if input.annual_miles > 200_000 {
        return Err(ApiError::BadRequest("annual_miles cap is 200,000".into()));
    }
    if !input.residual_pct_after_hold.is_finite()
        || input.residual_pct_after_hold < 0.0
        || input.residual_pct_after_hold > 100.0
    {
        return Err(ApiError::BadRequest(
            "residual_pct_after_hold must be in [0, 100]".into(),
        ));
    }
    Ok(Json(car_tco::compute(&input)))
}
