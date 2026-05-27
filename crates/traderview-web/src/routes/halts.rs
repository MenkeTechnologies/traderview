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
fn default_limit() -> usize { 100 }

async fn latest(
    State(_s): State<AppState>,
    _user: AuthUser,
    Query(q): Query<LatestQ>,
) -> Result<Json<Vec<halts::Halt>>, ApiError> {
    Ok(Json(halts::global().latest(q.limit)))
}

async fn ws(
    State(_s): State<AppState>,
    upgrade: WebSocketUpgrade,
) -> Response {
    upgrade.on_upgrade(handle_ws)
}

async fn handle_ws(mut socket: WebSocket) {
    let store = halts::global();
    // Send snapshot first.
    if let Ok(snap) = serde_json::to_string(&store.latest(50)) {
        if socket.send(Message::Text(format!("{{\"type\":\"snapshot\",\"halts\":{snap}}}")))
            .await.is_err() { return; }
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
