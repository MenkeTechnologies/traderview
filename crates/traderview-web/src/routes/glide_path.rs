//! Glide path route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use traderview_db::glide_path;

pub fn router() -> Router<AppState> {
    Router::new().route("/glide-path/compute", post(compute))
}

async fn compute(
    State(_s): State<AppState>,
    _user: AuthUser,
    Json(input): Json<glide_path::GlidePathInput>,
) -> Result<Json<glide_path::GlidePathReport>, ApiError> {
    if input.current_age == 0 || input.current_age > 110 {
        return Err(ApiError::BadRequest(
            "current_age must be in [1, 110]".into(),
        ));
    }
    if input.retirement_age < input.current_age || input.retirement_age > 110 {
        return Err(ApiError::BadRequest(
            "retirement_age must be ≥ current_age and ≤ 110".into(),
        ));
    }
    if input.landing_age < input.retirement_age || input.landing_age > 120 {
        return Err(ApiError::BadRequest(
            "landing_age must be ≥ retirement_age and ≤ 120".into(),
        ));
    }
    if input.horizon_age < input.current_age || input.horizon_age > 120 {
        return Err(ApiError::BadRequest(
            "horizon_age must be ≥ current_age and ≤ 120".into(),
        ));
    }
    for (n, v) in [
        ("start_stock_pct", input.start_stock_pct),
        ("retire_stock_pct", input.retire_stock_pct),
        ("landing_stock_pct", input.landing_stock_pct),
    ] {
        if !v.is_finite() || v < 0.0 || v > 100.0 {
            return Err(ApiError::BadRequest(format!("{n} must be in [0, 100]")));
        }
    }
    Ok(Json(glide_path::compute(&input)))
}
