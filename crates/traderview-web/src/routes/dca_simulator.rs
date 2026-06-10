//! DCA scheduler simulator route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use traderview_db::dca_simulator;

pub fn router() -> Router<AppState> {
    Router::new().route("/dca-simulator/run", get(run))
}

#[derive(Deserialize)]
struct RunQ {
    symbol: String,
    contribution_usd: f64,
    frequency: dca_simulator::DcaFrequency,
    #[serde(default = "default_days")]
    days_back: i64,
}
fn default_days() -> i64 {
    1825
}

async fn run(
    State(s): State<AppState>,
    _user: AuthUser,
    Query(q): Query<RunQ>,
) -> Result<Json<Option<dca_simulator::DcaResult>>, ApiError> {
    if !(q.contribution_usd > 0.0 && q.contribution_usd <= 1_000_000.0) {
        return Err(ApiError::BadRequest(
            "contribution_usd must be in (0, 1_000_000]".into(),
        ));
    }
    if !(q.days_back >= 30 && q.days_back <= 7300) {
        return Err(ApiError::BadRequest(
            "days_back must be in [30, 7300]".into(),
        ));
    }
    Ok(Json(
        dca_simulator::run(
            &s.pool,
            &q.symbol,
            q.contribution_usd,
            q.frequency,
            q.days_back,
        )
        .await?,
    ))
}
