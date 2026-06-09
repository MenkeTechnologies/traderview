//! Multi-broker position aggregation + kill-switch routes.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use traderview_db::multi_broker;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/multi-broker/positions", get(positions))
        .route("/multi-broker/kill-switch", post(kill_switch))
}

async fn positions(
    State(s): State<AppState>,
    user: AuthUser,
) -> Result<Json<multi_broker::MultiBrokerReport>, ApiError> {
    Ok(Json(multi_broker::fetch_all(&s.pool, user.id).await?))
}

/// Kill-switch payload. The `confirm_token` must equal the literal
/// string `KILL-ALL-NOW` — a deliberate friction-point so an
/// accidental POST can't trigger the destructive action. The UI's
/// `tConfirm` dialog asks the user to type the token explicitly.
#[derive(Deserialize)]
struct KillSwitchBody {
    confirm_token: String,
}

#[derive(Serialize)]
struct KillSwitchResult {
    cancelled_orders: usize,
    closed_positions: usize,
    errors: Vec<multi_broker::BrokerError>,
    note: String,
}

async fn kill_switch(
    State(_s): State<AppState>,
    _user: AuthUser,
    Json(body): Json<KillSwitchBody>,
) -> Result<Json<KillSwitchResult>, ApiError> {
    const EXPECTED: &str = "KILL-ALL-NOW";
    if body.confirm_token.trim() != EXPECTED {
        return Err(ApiError::BadRequest(format!(
            "confirm_token must equal {EXPECTED:?} — refusing to fire kill-switch"
        )));
    }
    // The actual order-cancel / position-close fan-out wiring lives in
    // a follow-up — this stub returns 0/0 and a note explaining the
    // safety posture. The token-gate is in place so the frontend
    // dialog + audit-log flow can be built and tested without yet
    // wiring to live broker APIs.
    Ok(Json(KillSwitchResult {
        cancelled_orders: 0,
        closed_positions: 0,
        errors: Vec::new(),
        note: "kill-switch armed but no live broker wiring yet — \
               cancel/close fan-out lands in a follow-up commit"
            .into(),
    }))
}
