//! Catalyst → price-action correlation route.
//!
//! Snapshot endpoint returns recent locked-in correlations newest-first
//! (optionally filtered to one symbol). The WebSocket pushes every
//! freshly-emitted correlation as the correlator fires it.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{ws::Message, ws::WebSocket, Path, Query, State, WebSocketUpgrade};
use axum::response::Response;
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use traderview_db::catalyst_correlator;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/catalyst-correlations", get(latest))
        .route(
            "/catalyst-correlations/symbol/:symbol",
            get(latest_for_symbol),
        )
        .route("/ws/catalyst-correlations", get(ws))
}

#[derive(Deserialize)]
struct LatestQ {
    #[serde(default = "default_limit")]
    limit: usize,
}
fn default_limit() -> usize {
    200
}

async fn latest(
    State(_s): State<AppState>,
    _user: AuthUser,
    Query(q): Query<LatestQ>,
) -> Result<Json<Vec<catalyst_correlator::Correlation>>, ApiError> {
    Ok(Json(catalyst_correlator::global().latest(q.limit)))
}

async fn latest_for_symbol(
    State(_s): State<AppState>,
    _user: AuthUser,
    Path(symbol): Path<String>,
    Query(q): Query<LatestQ>,
) -> Result<Json<Vec<catalyst_correlator::Correlation>>, ApiError> {
    Ok(Json(
        catalyst_correlator::global().latest_for(&symbol, q.limit),
    ))
}

async fn ws(
    State(s): State<AppState>,
    Query(tq): Query<crate::auth::WsTokenQuery>,
    upgrade: WebSocketUpgrade,
) -> Result<Response, ApiError> {
    crate::auth::require_ws_auth(&s, tq.token.as_deref())?;
    Ok(upgrade.on_upgrade(handle_ws))
}

async fn handle_ws(mut socket: WebSocket) {
    let store = catalyst_correlator::global();
    if let Ok(snap) = serde_json::to_string(&store.latest(200)) {
        if socket
            .send(Message::Text(format!(
                "{{\"type\":\"snapshot\",\"rows\":{snap}}}"
            )))
            .await
            .is_err()
        {
            return;
        }
    }
    let mut rx = store.subscribe();
    loop {
        tokio::select! {
            recv = rx.recv() => match recv {
                Ok(c) => {
                    if let Ok(j) = serde_json::to_string(&c) {
                        if socket
                            .send(Message::Text(format!(
                                "{{\"type\":\"correlation\",\"row\":{j}}}"
                            )))
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                Err(_) => break,
            },
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
        }
    }
}
