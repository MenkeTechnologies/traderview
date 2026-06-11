//! Multi-signal confluence dashboard route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{ws::Message, ws::WebSocket, Path, Query, State, WebSocketUpgrade};
use axum::response::Response;
use axum::routing::get;
use axum::{Json, Router};
use chrono::Utc;
use serde::Deserialize;
use traderview_db::confluence;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/confluence/ranked", get(ranked))
        .route("/confluence/events/:symbol", get(events_for_symbol))
        .route("/ws/confluence", get(ws))
}

#[derive(Deserialize)]
struct RankedQ {
    #[serde(default = "default_limit")]
    limit: usize,
    /// Minimum distinct sources a symbol must hit to appear. Default 2.
    #[serde(default = "default_min_sources")]
    min_sources: usize,
}
fn default_limit() -> usize {
    50
}
fn default_min_sources() -> usize {
    2
}

#[derive(Deserialize)]
struct EventsQ {
    #[serde(default = "default_event_limit")]
    limit: usize,
}
fn default_event_limit() -> usize {
    50
}

async fn ranked(
    State(_s): State<AppState>,
    _user: AuthUser,
    Query(q): Query<RankedQ>,
) -> Result<Json<Vec<confluence::ConfluenceRow>>, ApiError> {
    Ok(Json(confluence::global().ranked(
        Utc::now(),
        q.limit,
        q.min_sources,
    )))
}

async fn events_for_symbol(
    State(_s): State<AppState>,
    _user: AuthUser,
    Path(symbol): Path<String>,
    Query(q): Query<EventsQ>,
) -> Result<Json<Vec<confluence::ConfluenceEvent>>, ApiError> {
    Ok(Json(confluence::global().events_for(&symbol, q.limit)))
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
    let store = confluence::global();
    if let Ok(snap) = serde_json::to_string(&store.ranked(Utc::now(), 50, 2)) {
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
                                "{{\"type\":\"event\",\"row\":{j}}}"
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
