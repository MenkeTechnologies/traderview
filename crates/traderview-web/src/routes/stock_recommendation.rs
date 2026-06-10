//! Per-symbol Buy/Sell/Hold recommendation API. Backs the new featured
//! panel on the research view that mirrors stockinvest.us's surface:
//! verdict + 1–5 stars + composite score + 30-day target. Algorithm
//! lives in `traderview_db::stock_recommendation`; this is a thin
//! HTTP shim that handles input validation + error mapping.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use traderview_db::stock_recommendation::{
    compute, RecommendationError, StockRecommendation,
};

pub fn router() -> Router<AppState> {
    Router::new().route(
        "/symbols/:symbol/recommendation",
        get(get_recommendation),
    )
}

async fn get_recommendation(
    State(s): State<AppState>,
    _u: AuthUser,
    Path(symbol): Path<String>,
) -> Result<Json<StockRecommendation>, ApiError> {
    // Normalize the symbol the same way the rest of the per-symbol
    // routes do: uppercase + strip whitespace. Reject anything that
    // looks like a path traversal probe.
    let sym = symbol.trim().to_uppercase();
    if sym.is_empty() || sym.len() > 20 || sym.contains('/') || sym.contains('\\') {
        return Err(ApiError::BadRequest("invalid symbol".into()));
    }
    match compute(&s.pool, &sym).await {
        Ok(r) => Ok(Json(r)),
        Err(RecommendationError::Insufficient { symbol, got, need }) => Err(
            ApiError::BadRequest(format!(
                "not enough price history for {symbol}: have {got}, need {need}"
            )),
        ),
        Err(RecommendationError::InvalidPrice(p)) => Err(ApiError::BadRequest(format!(
            "latest close is non-positive: {p}"
        ))),
        Err(RecommendationError::PriceFetch(e)) => Err(ApiError::Internal(e)),
    }
}
