//! EV vs ICE route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use traderview_db::ev_vs_ice;

pub fn router() -> Router<AppState> {
    Router::new().route("/ev-vs-ice/compute", post(compute))
}

async fn compute(
    State(_s): State<AppState>,
    _user: AuthUser,
    Json(input): Json<ev_vs_ice::EvVsIceInput>,
) -> Result<Json<ev_vs_ice::EvVsIceReport>, ApiError> {
    if input.hold_years == 0 || input.hold_years > 30 {
        return Err(ApiError::BadRequest("hold_years must be in [1, 30]".into()));
    }
    if input.annual_miles > 200_000 {
        return Err(ApiError::BadRequest("annual_miles cap is 200,000".into()));
    }
    Ok(Json(ev_vs_ice::compute(&input)))
}
