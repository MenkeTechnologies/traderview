//! Personal balance sheet route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use traderview_db::personal_balance_sheet;

pub fn router() -> Router<AppState> {
    Router::new().route("/personal-balance-sheet/compute", post(compute))
}

async fn compute(
    State(_s): State<AppState>,
    _user: AuthUser,
    Json(input): Json<personal_balance_sheet::BalanceSheetInput>,
) -> Result<Json<personal_balance_sheet::BalanceSheetReport>, ApiError> {
    if input.assets.len() > 500 || input.liabilities.len() > 500 {
        return Err(ApiError::BadRequest(
            "assets / liabilities each capped at 500".into(),
        ));
    }
    for it in &input.assets {
        if !it.value_usd.is_finite() {
            return Err(ApiError::BadRequest("asset value_usd must be finite".into()));
        }
    }
    for it in &input.liabilities {
        if !it.value_usd.is_finite() {
            return Err(ApiError::BadRequest(
                "liability value_usd must be finite".into(),
            ));
        }
    }
    Ok(Json(personal_balance_sheet::compute(&input)))
}
