//! Squeeze scanner HTTP + WebSocket endpoints.
//!
//! Routes:
//!   * `GET  /squeeze/candidates` — current top-N catalyst-driven candidates
//!     (the symbols the live-ticks pump is streaming).
//!   * `GET  /squeeze/events?limit=N` — recent squeeze trigger events.
//!   * `GET  /squeeze/config`       — current detector thresholds.
//!   * `POST /squeeze/config`       — update detector thresholds.
//!   * `GET  /ws/squeeze`           — WebSocket of live SqueezeEvents.
//!
//! The candidate aggregator + squeeze detector pumps run in the background
//! out of `bin/server.rs`; these endpoints just surface their state.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{ws::Message, ws::WebSocket, State, WebSocketUpgrade};
use axum::response::Response;
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use traderview_db::candidates;
use traderview_db::squeeze_detector::{self, SqueezeConfig};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/squeeze/candidates", get(candidates_list))
        .route("/squeeze/events", get(events_list))
        .route("/squeeze/config", get(config_get).post(config_set))
        .route("/ws/squeeze", get(ws_handler))
}

async fn candidates_list(
    State(_s): State<AppState>,
    _u: AuthUser,
) -> Result<Json<Vec<candidates::Candidate>>, ApiError> {
    Ok(Json(candidates::global().snapshot()))
}

#[derive(Deserialize)]
struct EventsQuery {
    limit: Option<usize>,
}
async fn events_list(
    State(_s): State<AppState>,
    _u: AuthUser,
    axum::extract::Query(q): axum::extract::Query<EventsQuery>,
) -> Result<Json<Vec<squeeze_detector::SqueezeEvent>>, ApiError> {
    let lim = q.limit.unwrap_or(50).clamp(1, 500);
    Ok(Json(squeeze_detector::global().recent(lim).await))
}

async fn config_get(
    State(_s): State<AppState>,
    _u: AuthUser,
) -> Result<Json<SqueezeConfig>, ApiError> {
    Ok(Json(squeeze_detector::global().get_config().await))
}

async fn config_set(
    State(_s): State<AppState>,
    _u: AuthUser,
    Json(body): Json<SqueezeConfig>,
) -> Result<Json<serde_json::Value>, ApiError> {
    squeeze_detector::global().set_config(body).await;
    Ok(Json(serde_json::json!({ "ok": true })))
}

async fn ws_handler(State(_s): State<AppState>, upgrade: WebSocketUpgrade) -> Response {
    upgrade.on_upgrade(handle_ws)
}

async fn handle_ws(mut socket: WebSocket) {
    let store = squeeze_detector::global();
    // Send a snapshot of recent events on connect so a freshly-loaded view
    // has something to render immediately.
    if let Ok(payload) = serde_json::to_string(&store.recent(50).await) {
        if socket
            .send(Message::Text(format!(
                "{{\"type\":\"snapshot\",\"events\":{payload}}}"
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
                Ok(ev) => {
                    let s = match serde_json::to_string(&ev) {
                        Ok(s) => s,
                        Err(_) => continue,
                    };
                    if socket
                        .send(Message::Text(format!("{{\"type\":\"event\",\"event\":{s}}}")))
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            },
            // Surface client closes promptly so we drop the broadcast slot.
            msg = socket.recv() => match msg {
                Some(Ok(Message::Close(_))) | None => break,
                Some(Err(_)) => break,
                _ => {}
            }
        }
    }
}
