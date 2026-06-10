//! Tax-loss harvesting scanner route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use traderview_db::tax_loss_harvest;

pub fn router() -> Router<AppState> {
    Router::new().route("/paper-tax-loss-harvest/scan", get(scan))
}

#[derive(Deserialize)]
struct ScanQ {
    #[serde(default = "default_rate")]
    marginal_rate_pct: f64,
    #[serde(default = "default_threshold")]
    min_loss_threshold_pct: f64,
}
fn default_rate() -> f64 {
    35.0
}
fn default_threshold() -> f64 {
    5.0
}

async fn scan(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<ScanQ>,
) -> Result<Json<tax_loss_harvest::HarvestReport>, ApiError> {
    if !(q.marginal_rate_pct >= 0.0 && q.marginal_rate_pct <= 60.0) {
        return Err(ApiError::BadRequest(
            "marginal_rate_pct must be in [0, 60]".into(),
        ));
    }
    if !(q.min_loss_threshold_pct >= 0.0 && q.min_loss_threshold_pct <= 100.0) {
        return Err(ApiError::BadRequest(
            "min_loss_threshold_pct must be in [0, 100]".into(),
        ));
    }
    Ok(Json(
        tax_loss_harvest::scan(
            &s.pool,
            user.id,
            q.marginal_rate_pct,
            q.min_loss_threshold_pct,
        )
        .await?,
    ))
}
