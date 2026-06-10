//! Net-worth tracker route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use traderview_db::net_worth_tracker;

pub fn router() -> Router<AppState> {
    Router::new().route("/net-worth-tracker/compute", post(compute))
}

async fn compute(
    State(_s): State<AppState>,
    _user: AuthUser,
    Json(input): Json<net_worth_tracker::NetWorthInput>,
) -> Result<Json<net_worth_tracker::NetWorthReport>, ApiError> {
    if input.assets.len() > 500 {
        return Err(ApiError::BadRequest("assets cap is 500 entries".into()));
    }
    if input.liabilities.len() > 500 {
        return Err(ApiError::BadRequest(
            "liabilities cap is 500 entries".into(),
        ));
    }
    if input.history.len() > 240 {
        return Err(ApiError::BadRequest("history cap is 240 months".into()));
    }
    for it in input.assets.iter().chain(input.liabilities.iter()) {
        if !it.value_usd.is_finite() {
            return Err(ApiError::BadRequest(
                "every line-item value_usd must be finite".into(),
            ));
        }
    }
    Ok(Json(net_worth_tracker::compute(&input)))
}
