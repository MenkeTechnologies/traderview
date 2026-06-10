//! Savings rate route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use traderview_db::savings_rate;

pub fn router() -> Router<AppState> {
    Router::new().route("/savings-rate/compute", post(compute))
}

async fn compute(
    State(_s): State<AppState>,
    _user: AuthUser,
    Json(input): Json<savings_rate::SavingsRateInput>,
) -> Result<Json<savings_rate::SavingsRateReport>, ApiError> {
    let fields = [
        ("gross_annual_income_usd", input.gross_annual_income_usd),
        ("net_annual_income_usd", input.net_annual_income_usd),
        ("annual_expenses_usd", input.annual_expenses_usd),
        ("annual_savings_usd", input.annual_savings_usd),
    ];
    for (n, v) in fields {
        if !v.is_finite() || v < 0.0 {
            return Err(ApiError::BadRequest(format!("{n} must be ≥ 0 and finite")));
        }
    }
    if !(input.expected_real_return_pct >= -10.0 && input.expected_real_return_pct <= 20.0) {
        return Err(ApiError::BadRequest(
            "expected_real_return_pct must be in [-10, 20]".into(),
        ));
    }
    if !(input.safe_withdrawal_rate_pct > 0.0 && input.safe_withdrawal_rate_pct <= 20.0) {
        return Err(ApiError::BadRequest(
            "safe_withdrawal_rate_pct must be in (0, 20]".into(),
        ));
    }
    Ok(Json(savings_rate::compute(&input)))
}
