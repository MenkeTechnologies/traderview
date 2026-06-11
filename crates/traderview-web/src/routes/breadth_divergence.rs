//! Breadth-divergence detector route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{ws::Message, ws::WebSocket, Query, State, WebSocketUpgrade};
use axum::response::Response;
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use traderview_db::breadth_divergence;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/breadth-divergence/recent", get(latest))
        .route("/breadth-divergence/window", get(window))
        .route("/breadth-divergence/current", get(current))
        .route("/ws/breadth-divergence", get(ws))
}

#[derive(Deserialize)]
struct LimitQ {
    #[serde(default = "default_limit")]
    limit: usize,
}
fn default_limit() -> usize {
    100
}

#[derive(Serialize)]
struct CurrentRegime {
    regime: Option<String>,
}

async fn latest(
    State(s): State<AppState>,
    _user: AuthUser,
    Query(q): Query<LimitQ>,
) -> Result<Json<Vec<breadth_divergence::DivergenceEvent>>, ApiError> {
    let store = breadth_divergence::global(s.pool.clone());
    Ok(Json(store.latest(q.limit)))
}

async fn window(
    State(s): State<AppState>,
    _user: AuthUser,
) -> Result<Json<Vec<breadth_divergence::Sample>>, ApiError> {
    let store = breadth_divergence::global(s.pool.clone());
    Ok(Json(store.window_snapshot().await))
}

async fn current(
    State(s): State<AppState>,
    _user: AuthUser,
) -> Result<Json<CurrentRegime>, ApiError> {
    let store = breadth_divergence::global(s.pool.clone());
    Ok(Json(CurrentRegime {
        regime: store.current_regime().await.map(|k| k.as_str().into()),
    }))
}

async fn ws(
    State(s): State<AppState>,
    Query(tq): Query<crate::auth::WsTokenQuery>,
    upgrade: WebSocketUpgrade,
) -> Result<Response, ApiError> {
    crate::auth::require_ws_auth(&s, tq.token.as_deref())?;
    let store = breadth_divergence::global(s.pool.clone());
    Ok(upgrade.on_upgrade(move |socket| handle_ws(socket, store)))
}

async fn handle_ws(mut socket: WebSocket, store: breadth_divergence::DivergenceStore) {
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
                                "{{\"type\":\"divergence\",\"row\":{j}}}"
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
