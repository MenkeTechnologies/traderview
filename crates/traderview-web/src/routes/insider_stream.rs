//! Real-time insider Form 4 stream route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{ws::Message, ws::WebSocket, Path, Query, State, WebSocketUpgrade};
use axum::response::Response;
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use traderview_db::insider_stream;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/insider-stream/recent", get(latest))
        .route("/insider-stream/symbol/:symbol", get(latest_for_symbol))
        .route("/insider-stream/top-buys", get(top_buys))
        .route("/ws/insider-stream", get(ws))
}

#[derive(Deserialize)]
struct LimitQ {
    #[serde(default = "default_limit")]
    limit: usize,
}
fn default_limit() -> usize {
    200
}

#[derive(Deserialize)]
struct TopBuysQ {
    #[serde(default = "default_days")]
    days: i64,
    #[serde(default = "default_top_limit")]
    limit: usize,
}
fn default_days() -> i64 {
    30
}
fn default_top_limit() -> usize {
    25
}

#[derive(Serialize)]
struct TopBuyRow {
    symbol: String,
    total_dollars: f64,
    transaction_count: usize,
}

async fn latest(
    State(_s): State<AppState>,
    _user: AuthUser,
    Query(q): Query<LimitQ>,
) -> Result<Json<Vec<insider_stream::InsiderEvent>>, ApiError> {
    Ok(Json(insider_stream::global().latest(q.limit)))
}

async fn latest_for_symbol(
    State(_s): State<AppState>,
    _user: AuthUser,
    Path(symbol): Path<String>,
    Query(q): Query<LimitQ>,
) -> Result<Json<Vec<insider_stream::InsiderEvent>>, ApiError> {
    Ok(Json(insider_stream::global().latest_for(&symbol, q.limit)))
}

async fn top_buys(
    State(_s): State<AppState>,
    _user: AuthUser,
    Query(q): Query<TopBuysQ>,
) -> Result<Json<Vec<TopBuyRow>>, ApiError> {
    let rows = insider_stream::global()
        .top_buys(q.days, q.limit)
        .into_iter()
        .map(|(symbol, total_dollars, transaction_count)| TopBuyRow {
            symbol,
            total_dollars,
            transaction_count,
        })
        .collect();
    Ok(Json(rows))
}

async fn ws(State(_s): State<AppState>, upgrade: WebSocketUpgrade) -> Response {
    upgrade.on_upgrade(handle_ws)
}

async fn handle_ws(mut socket: WebSocket) {
    let store = insider_stream::global();
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
                Ok(ev) => {
                    if let Ok(j) = serde_json::to_string(&ev) {
                        if socket
                            .send(Message::Text(format!(
                                "{{\"type\":\"insider\",\"row\":{j}}}"
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
