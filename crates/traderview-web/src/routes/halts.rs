use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{ws::Message, ws::WebSocket, Query, State, WebSocketUpgrade};
use axum::response::Response;
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use traderview_db::halts;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/halts", get(latest))
        .route("/ws/halts", get(ws))
}

#[derive(Deserialize)]
struct LatestQ {
    #[serde(default = "default_limit")]
    limit: usize,
}
fn default_limit() -> usize {
    100
}

async fn latest(
    State(_s): State<AppState>,
    _user: AuthUser,
    Query(q): Query<LatestQ>,
) -> Result<Json<Vec<halts::Halt>>, ApiError> {
    Ok(Json(halts::global().latest(q.limit)))
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
    let store = halts::global();
    // Send snapshot first.
    if let Ok(snap) = serde_json::to_string(&store.latest(50)) {
        if socket
            .send(Message::Text(format!(
                "{{\"type\":\"snapshot\",\"halts\":{snap}}}"
            )))
            .await
            .is_err()
        {
            return;
        }
    }
    // Then stream updates.
    let mut rx = store.subscribe();
    loop {
        tokio::select! {
            recv = rx.recv() => match recv {
                Ok(h) => {
                    if let Ok(j) = serde_json::to_string(&h) {
                        if socket.send(Message::Text(format!("{{\"type\":\"halt\",\"halt\":{j}}}")))
                            .await.is_err() { break; }
                    }
                }
                // Slow consumer: drop the lag and resync on next event.
                // Sending a `lagged` frame lets the client decide whether
                // to re-fetch the snapshot. Previously we broke the
                // socket on Lagged, forcing a full reconnect over a
                // transient hiccup.
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                    let _ = socket
                        .send(Message::Text("{\"type\":\"lagged\"}".into()))
                        .await;
                    continue;
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
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
