//! Rent vs buy route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use traderview_db::rent_vs_buy;

pub fn router() -> Router<AppState> {
    Router::new().route("/rent-vs-buy/compute", post(compute))
}

async fn compute(
    State(_s): State<AppState>,
    _user: AuthUser,
    Json(input): Json<rent_vs_buy::RentVsBuyInput>,
) -> Result<Json<rent_vs_buy::RentVsBuyReport>, ApiError> {
    let nonneg = [
        ("home_price_usd", input.home_price_usd),
        ("insurance_annual_usd", input.insurance_annual_usd),
        ("monthly_hoa_usd", input.monthly_hoa_usd),
        ("monthly_rent_usd", input.monthly_rent_usd),
        ("renter_insurance_annual_usd", input.renter_insurance_annual_usd),
    ];
    for (n, v) in nonneg {
        if !v.is_finite() || v < 0.0 {
            return Err(ApiError::BadRequest(format!("{n} must be ≥ 0 and finite")));
        }
    }
    let pcts = [
        ("down_payment_pct", input.down_payment_pct, 0.0, 100.0),
        ("mortgage_apr_pct", input.mortgage_apr_pct, 0.0, 30.0),
        ("closing_costs_pct", input.closing_costs_pct, 0.0, 20.0),
        ("property_tax_annual_pct", input.property_tax_annual_pct, 0.0, 10.0),
        ("maintenance_annual_pct", input.maintenance_annual_pct, 0.0, 10.0),
        ("home_appreciation_pct", input.home_appreciation_pct, -20.0, 30.0),
        ("selling_costs_pct", input.selling_costs_pct, 0.0, 20.0),
        ("rent_inflation_pct", input.rent_inflation_pct, -20.0, 30.0),
        ("investment_return_pct", input.investment_return_pct, -20.0, 30.0),
    ];
    for (n, v, lo, hi) in pcts {
        if !v.is_finite() || v < lo || v > hi {
            return Err(ApiError::BadRequest(format!(
                "{n} must be in [{lo}, {hi}]"
            )));
        }
    }
    if input.mortgage_term_months == 0 || input.mortgage_term_months > 600 {
        return Err(ApiError::BadRequest(
            "mortgage_term_months must be in [1, 600]".into(),
        ));
    }
    if input.horizon_years > 60 {
        return Err(ApiError::BadRequest("horizon_years cap is 60".into()));
    }
    Ok(Json(rent_vs_buy::compute(&input)))
}
