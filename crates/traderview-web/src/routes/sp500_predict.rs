//! S&P 500 inclusion predictor route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use traderview_db::sp500_predictor;

pub fn router() -> Router<AppState> {
    Router::new().route("/sp500-predict/scan", get(scan))
}

#[derive(Deserialize)]
struct ScanQ {
    /// Comma-separated symbol list.
    symbols: String,
    #[serde(default)]
    min_market_cap_usd: Option<f64>,
}

async fn scan(
    State(_s): State<AppState>,
    _user: AuthUser,
    Query(q): Query<ScanQ>,
) -> Result<Json<Vec<sp500_predictor::Sp500Score>>, ApiError> {
    let symbols: Vec<String> = q
        .symbols
        .split(',')
        .map(|t| t.trim().to_ascii_uppercase())
        .filter(|t| !t.is_empty())
        .collect();
    if symbols.is_empty() {
        return Ok(Json(Vec::new()));
    }
    let mut criteria = sp500_predictor::Criteria::default();
    if let Some(mc) = q.min_market_cap_usd {
        if mc > 0.0 {
            criteria.min_market_cap_usd = mc;
        }
    }
    Ok(Json(sp500_predictor::scan(&symbols, &criteria).await))
}
