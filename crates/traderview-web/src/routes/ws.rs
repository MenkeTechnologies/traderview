//! Real-time WebSocket fan-out.
//!
//! GET /ws — upgrade to WebSocket. Server pushes serialized `Event`s as JSON
//! text frames. Auth via `?token=` query parameter (browsers can't set
//! Authorization on WS handshakes); when running in Desktop mode the token
//! is optional.
//!
//! Lifecycle: server emits a heartbeat `Ping` every 30s. If the client closes
//! or the channel lags, we shut the socket cleanly.

use crate::auth::decode_token;
use crate::error::ApiError;
use crate::realtime::Event;
use crate::state::{AppMode, AppState};
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use serde::Deserialize;
use std::time::Duration;
use tokio::sync::broadcast::error::RecvError;
use tokio::time::interval;

pub fn router() -> Router<AppState> {
    Router::new().route("/ws", get(ws_handler))
}

#[derive(Debug, Deserialize)]
struct WsParams {
    token: Option<String>,
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(s): State<AppState>,
    Query(p): Query<WsParams>,
) -> Result<Response, ApiError> {
    // Desktop mode: skip auth.
    if matches!(s.mode, AppMode::Web) {
        let tok = p.token.ok_or(ApiError::Unauthorized)?;
        decode_token(&s.jwt_secret, &tok).map_err(|_| ApiError::Unauthorized)?;
    }
    Ok(ws.on_upgrade(move |socket| handle_socket(socket, s)))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    let mut rx = state.hub.subscribe();
    let mut hb = interval(Duration::from_secs(30));
    // First tick fires immediately; skip it so we don't double-emit at connect.
    hb.tick().await;

    // Send a hello so the client knows the channel is up.
    let hello = serde_json::json!({ "type": "hello", "capacity": crate::realtime::CAPACITY });
    if socket.send(Message::Text(hello.to_string())).await.is_err() {
        return;
    }

    loop {
        tokio::select! {
            recv = rx.recv() => match recv {
                Ok(ev) => {
                    let json = match serde_json::to_string(&ev) {
                        Ok(s) => s,
                        Err(_) => continue,
                    };
                    if socket.send(Message::Text(json)).await.is_err() {
                        break;
                    }
                }
                Err(RecvError::Lagged(_)) => {
                    // Slow client — close cleanly.
                    let _ = socket.send(Message::Text(
                        "{\"type\":\"lagged\"}".into())).await;
                    break;
                }
                Err(RecvError::Closed) => break,
            },
            _ = hb.tick() => {
                let ev = Event::Ping { ts: chrono::Utc::now().timestamp() };
                let json = serde_json::to_string(&ev).unwrap_or_default();
                if socket.send(Message::Text(json)).await.is_err() {
                    break;
                }
            }
            inbound = socket.recv() => match inbound {
                Some(Ok(Message::Close(_))) | None => break,
                Some(Ok(_)) => { /* ignore client frames for now */ }
                Some(Err(_)) => break,
            }
        }
    }
}
