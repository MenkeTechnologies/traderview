//! Personal cash-flow statement route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use traderview_db::personal_cash_flow;

pub fn router() -> Router<AppState> {
    Router::new().route("/personal-cash-flow/compute", post(compute))
}

async fn compute(
    State(_s): State<AppState>,
    _user: AuthUser,
    Json(input): Json<personal_cash_flow::CashFlowInput>,
) -> Result<Json<personal_cash_flow::CashFlowReport>, ApiError> {
    if input.rows.len() > 1000 {
        return Err(ApiError::BadRequest("rows cap is 1000".into()));
    }
    for r in &input.rows {
        if !r.amount_usd.is_finite() || r.amount_usd < 0.0 {
            return Err(ApiError::BadRequest(
                "every row amount_usd must be ≥ 0 and finite".into(),
            ));
        }
        match r.category.as_str() {
            "operating" | "investing" | "financing" => {}
            _ => {
                return Err(ApiError::BadRequest(
                    "category must be operating / investing / financing".into(),
                ));
            }
        }
        match r.direction.as_str() {
            "inflow" | "outflow" => {}
            _ => {
                return Err(ApiError::BadRequest(
                    "direction must be inflow / outflow".into(),
                ));
            }
        }
    }
    Ok(Json(personal_cash_flow::compute(&input)))
}
