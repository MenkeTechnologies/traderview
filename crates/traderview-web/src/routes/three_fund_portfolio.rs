//! Three-fund portfolio route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use traderview_db::three_fund_portfolio;

pub fn router() -> Router<AppState> {
    Router::new().route("/three-fund-portfolio/compute", post(compute))
}

async fn compute(
    State(_s): State<AppState>,
    _user: AuthUser,
    Json(input): Json<three_fund_portfolio::ThreeFundInput>,
) -> Result<Json<three_fund_portfolio::ThreeFundReport>, ApiError> {
    if input.age == 0 || input.age > 110 {
        return Err(ApiError::BadRequest("age must be in [1, 110]".into()));
    }
    let nonneg = [
        ("current_us_stocks_usd", input.current_us_stocks_usd),
        ("current_intl_stocks_usd", input.current_intl_stocks_usd),
        ("current_bonds_usd", input.current_bonds_usd),
    ];
    for (n, v) in nonneg {
        if !v.is_finite() || v < 0.0 {
            return Err(ApiError::BadRequest(format!("{n} must be ≥ 0 and finite")));
        }
    }
    if !input.us_within_stocks_pct.is_finite()
        || input.us_within_stocks_pct < 0.0
        || input.us_within_stocks_pct > 100.0
    {
        return Err(ApiError::BadRequest(
            "us_within_stocks_pct must be in [0, 100]".into(),
        ));
    }
    Ok(Json(three_fund_portfolio::compute(&input)))
}
