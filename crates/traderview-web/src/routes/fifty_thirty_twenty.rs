//! 50/30/20 budget rule route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use traderview_db::fifty_thirty_twenty;

pub fn router() -> Router<AppState> {
    Router::new().route("/fifty-thirty-twenty/compute", post(compute))
}

async fn compute(
    State(_s): State<AppState>,
    _user: AuthUser,
    Json(input): Json<fifty_thirty_twenty::FiftyThirtyTwentyInput>,
) -> Result<Json<fifty_thirty_twenty::FiftyThirtyTwentyReport>, ApiError> {
    if !input.net_monthly_income_usd.is_finite() || input.net_monthly_income_usd < 0.0 {
        return Err(ApiError::BadRequest(
            "net_monthly_income_usd must be ≥ 0".into(),
        ));
    }
    if input.rows.len() > 500 {
        return Err(ApiError::BadRequest("rows cap is 500".into()));
    }
    for r in &input.rows {
        if !r.amount_usd.is_finite() || r.amount_usd < 0.0 {
            return Err(ApiError::BadRequest("amount_usd must be ≥ 0".into()));
        }
    }
    Ok(Json(fifty_thirty_twenty::compute(&input)))
}
