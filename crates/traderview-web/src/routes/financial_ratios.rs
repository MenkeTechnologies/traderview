//! Personal financial ratios route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use traderview_db::financial_ratios;

pub fn router() -> Router<AppState> {
    Router::new().route("/financial-ratios/compute", post(compute))
}

async fn compute(
    State(_s): State<AppState>,
    _user: AuthUser,
    Json(input): Json<financial_ratios::RatiosInput>,
) -> Result<Json<financial_ratios::RatioReport>, ApiError> {
    let fields = [
        ("gross_monthly_income_usd", input.gross_monthly_income_usd),
        ("total_monthly_expenses_usd", input.total_monthly_expenses_usd),
        ("monthly_debt_payments_usd", input.monthly_debt_payments_usd),
        ("monthly_housing_payment_usd", input.monthly_housing_payment_usd),
        ("liquid_assets_usd", input.liquid_assets_usd),
        ("emergency_fund_balance_usd", input.emergency_fund_balance_usd),
        ("total_assets_usd", input.total_assets_usd),
        ("total_liabilities_usd", input.total_liabilities_usd),
        ("retirement_assets_usd", input.retirement_assets_usd),
    ];
    for (name, v) in fields {
        if !v.is_finite() || v < 0.0 {
            return Err(ApiError::BadRequest(format!(
                "{name} must be ≥ 0 and finite"
            )));
        }
    }
    Ok(Json(financial_ratios::compute(&input)))
}
