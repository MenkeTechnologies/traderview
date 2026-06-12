//! Market gamma regime tracker route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::Utc;
use traderview_db::market_gamma_regime;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/market-gamma/report", get(report))
        // POST — the frontend sends POST and the handler mutates the
        // rolling history. As a GET it 405'd and "Refresh Now" was a
        // silent no-op.
        .route("/market-gamma/refresh", post(refresh))
}

async fn report(
    State(_s): State<AppState>,
    _user: AuthUser,
) -> Result<Json<Option<market_gamma_regime::MarketGammaReport>>, ApiError> {
    Ok(Json(market_gamma_regime::global().report(Utc::now())))
}

/// Force an immediate snapshot — useful for the frontend's "refresh
/// now" button when the user doesn't want to wait the 30-min background
/// tick. Adds the result to the rolling history.
async fn refresh(
    State(_s): State<AppState>,
    _user: AuthUser,
) -> Result<Json<Option<market_gamma_regime::GexSnapshot>>, ApiError> {
    let snap = market_gamma_regime::fetch_snapshot().await;
    if let Some(s) = &snap {
        market_gamma_regime::global().upsert(s.clone());
    }
    Ok(Json(snap))
}
