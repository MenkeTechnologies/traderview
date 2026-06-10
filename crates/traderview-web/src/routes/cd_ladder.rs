//! CD ladder route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use traderview_db::cd_ladder;

pub fn router() -> Router<AppState> {
    Router::new().route("/cd-ladder/compute", post(compute))
}

async fn compute(
    State(_s): State<AppState>,
    _user: AuthUser,
    Json(input): Json<cd_ladder::CdLadderInput>,
) -> Result<Json<cd_ladder::CdLadderReport>, ApiError> {
    if !input.total_principal_usd.is_finite() || input.total_principal_usd < 0.0 {
        return Err(ApiError::BadRequest(
            "total_principal_usd must be ≥ 0".into(),
        ));
    }
    if input.rungs == 0 || input.rungs > 30 {
        return Err(ApiError::BadRequest("rungs must be in [1, 30]".into()));
    }
    if input.term_years_per_rung == 0 || input.term_years_per_rung > 30 {
        return Err(ApiError::BadRequest(
            "term_years_per_rung must be in [1, 30]".into(),
        ));
    }
    for v in &input.per_rung_apy_pct {
        if !v.is_finite() || *v < 0.0 || *v > 30.0 {
            return Err(ApiError::BadRequest(
                "per_rung_apy_pct entries must be in [0, 30]".into(),
            ));
        }
    }
    if let Some(v) = input.flat_apy_pct {
        if !v.is_finite() || v < 0.0 || v > 30.0 {
            return Err(ApiError::BadRequest(
                "flat_apy_pct must be in [0, 30]".into(),
            ));
        }
    }
    Ok(Json(cd_ladder::compute(&input)))
}
