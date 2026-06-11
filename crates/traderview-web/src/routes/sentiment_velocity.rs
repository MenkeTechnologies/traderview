//! Sentiment-velocity detector route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{ws::Message, ws::WebSocket, Query, State, WebSocketUpgrade};
use axum::response::Response;
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use traderview_db::sentiment_velocity;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/sentiment-velocity/recent", get(latest))
        .route("/ws/sentiment-velocity", get(ws))
}

#[derive(Deserialize)]
struct LimitQ {
    #[serde(default = "default_limit")]
    limit: usize,
}
fn default_limit() -> usize {
    100
}

async fn latest(
    State(s): State<AppState>,
    _user: AuthUser,
    Query(q): Query<LimitQ>,
) -> Result<Json<Vec<sentiment_velocity::VelocityEvent>>, ApiError> {
    let store = sentiment_velocity::global(s.pool.clone());
    Ok(Json(store.latest(q.limit)))
}

async fn ws(
    State(s): State<AppState>,
    Query(tq): Query<crate::auth::WsTokenQuery>,
    upgrade: WebSocketUpgrade,
) -> Result<Response, ApiError> {
    crate::auth::require_ws_auth(&s, tq.token.as_deref())?;
    let store = sentiment_velocity::global(s.pool.clone());
    Ok(upgrade.on_upgrade(move |socket| handle_ws(socket, store)))
}

async fn handle_ws(mut socket: WebSocket, store: sentiment_velocity::VelocityStore) {
    if let Ok(snap) = serde_json::to_string(&store.latest(100)) {
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
                Ok(ev) => {
                    if let Ok(j) = serde_json::to_string(&ev) {
                        if socket
                            .send(Message::Text(format!(
                                "{{\"type\":\"velocity\",\"row\":{j}}}"
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
