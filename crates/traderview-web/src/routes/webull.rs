use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{ws::Message, ws::WebSocket, Query, State, WebSocketUpgrade};
use axum::response::Response;
use axum::routing::{get, post};
use axum::{Json, Router};
use traderview_db::webull;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/webull/connect", post(connect))
        .route("/webull/snapshot", get(snapshot))
        .route("/ws/webull", get(ws))
}

async fn connect(
    State(_s): State<AppState>,
    _user: AuthUser,
    Json(body): Json<webull::ConnectRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let client = webull::global();
    client
        .set_creds(webull::Creds {
            did: body.did,
            access_token: body.access_token,
            t_token: body.t_token,
            account_id: body.account_id,
        })
        .await;
    Ok(Json(
        serde_json::json!({"ok": true, "has_creds": client.has_creds().await}),
    ))
}

async fn snapshot(
    State(_s): State<AppState>,
    _user: AuthUser,
) -> Result<Json<Option<webull::WSnapshot>>, ApiError> {
    Ok(Json(webull::global().last_snapshot()))
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
    let client = webull::global();
    if let Some(snap) = client.last_snapshot() {
        if let Ok(j) = serde_json::to_string(&snap) {
            if socket
                .send(Message::Text(format!(
                    "{{\"type\":\"snapshot\",\"snap\":{j}}}"
                )))
                .await
                .is_err()
            {
                return;
            }
        }
    }
    let mut rx = client.subscribe();
    loop {
        tokio::select! {
            recv = rx.recv() => match recv {
                Ok(s) => {
                    if let Ok(j) = serde_json::to_string(&s) {
                        if socket.send(Message::Text(format!("{{\"type\":\"snapshot\",\"snap\":{j}}}")))
                            .await.is_err() { break; }
                    }
                }
                // Same Lagged/Closed split as halts.rs: don't drop the
                // socket on a transient slow-consumer hiccup; emit a
                // `lagged` frame and keep streaming.
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                    let _ = socket
                        .send(Message::Text("{\"type\":\"lagged\"}".into()))
                        .await;
                    continue;
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            },
            msg = socket.recv() => match msg {
                Some(Ok(Message::Close(_))) | None => break,
                _ => {}
            }
        }
    }
}
