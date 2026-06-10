//! CAPE / Shiller P/E indicator route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use traderview_db::cape_indicator;

pub fn router() -> Router<AppState> {
    Router::new().route("/cape-indicator/score", get(score))
}

#[derive(Deserialize)]
struct ScoreQ {
    /// CAPE value to score. Defaults to latest known embedded value.
    #[serde(default)]
    value: Option<f64>,
}

async fn score(
    State(_s): State<AppState>,
    _user: AuthUser,
    Query(q): Query<ScoreQ>,
) -> Result<Json<cape_indicator::CapeScore>, ApiError> {
    let value = q.value.unwrap_or_else(cape_indicator::latest_known_value);
    if !(value > 0.0 && value < 100.0) {
        return Err(ApiError::BadRequest(
            "CAPE value must be in (0, 100)".into(),
        ));
    }
    Ok(Json(cape_indicator::score(value)))
}
