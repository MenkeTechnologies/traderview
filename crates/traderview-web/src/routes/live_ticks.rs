use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{ws::Message, ws::WebSocket, State, WebSocketUpgrade};
use axum::response::Response;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use traderview_db::live_ticks;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/ticks/snapshot", get(snapshot))
        .route("/ticks/configure", post(configure))
        .route("/ws/ticks", get(ws))
}

async fn snapshot(
    State(_s): State<AppState>,
    _user: AuthUser,
) -> Result<Json<Vec<live_ticks::SymbolState>>, ApiError> {
    Ok(Json(live_ticks::global().snapshot()))
}

#[derive(Deserialize)]
struct ConfigureBody {
    /// Finnhub API key. Set once per session; persisted on the server only
    /// for the lifetime of the process (never written to disk).
    #[serde(default)]
    api_key: Option<String>,
    /// Universe of symbols to subscribe to.
    #[serde(default)]
    symbols: Vec<String>,
}

async fn configure(
    State(_s): State<AppState>,
    _user: AuthUser,
    Json(body): Json<ConfigureBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let store = live_ticks::global();
    if let Some(k) = body.api_key {
        if !k.is_empty() {
            store.set_api_key(k).await;
        }
    }
    if !body.symbols.is_empty() {
        store
            .set_symbols(body.symbols.clone())
            .await
            .map_err(ApiError::Internal)?;
    }
    Ok(Json(serde_json::json!({
        "ok": true,
        "has_key": store.has_key().await,
        "subscribed": body.symbols.len(),
    })))
}

async fn ws(State(_s): State<AppState>, upgrade: WebSocketUpgrade) -> Response {
    upgrade.on_upgrade(handle_ws)
}

async fn handle_ws(mut socket: WebSocket) {
    let store = live_ticks::global();
    if let Ok(snap) = serde_json::to_string(&store.snapshot()) {
        if socket
            .send(Message::Text(format!(
                "{{\"type\":\"snapshot\",\"states\":{snap}}}"
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
                Ok(s) => {
                    if let Ok(j) = serde_json::to_string(&s) {
                        if socket
                            .send(Message::Text(format!("{{\"type\":\"tick\",\"state\":{j}}}")))
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                }
                Err(_) => break,
            },
            msg = socket.recv() => match msg {
                Some(Ok(Message::Close(_))) | None => break,
                _ => {}
            }
        }
    }
}
