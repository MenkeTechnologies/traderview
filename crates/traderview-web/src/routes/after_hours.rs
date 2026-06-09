//! After-hours mover scanner route.
//!
//! Snapshot endpoint returns the top gainers / losers in either the
//! `pre` or `post` session, ranked by signed change_pct vs the prior
//! RTH close. The WebSocket pushes every per-symbol state update as the
//! tape classifier folds new trades into the AfterHoursStore.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{ws::Message, ws::WebSocket, Query, State, WebSocketUpgrade};
use axum::response::Response;
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use traderview_db::after_hours;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/after-hours/movers", get(movers))
        .route("/after-hours/snapshot", get(snapshot))
        .route("/ws/after-hours", get(ws))
}

#[derive(Deserialize)]
struct MoversQ {
    /// `pre` or `post` — defaults to whichever session is currently live.
    #[serde(default)]
    session: Option<String>,
    /// `gainers` (default) or `losers`.
    #[serde(default)]
    direction: Option<String>,
    #[serde(default = "default_limit")]
    limit: usize,
    /// Filter out rows with `|change_pct| < min_pct`. Default 1.0%.
    #[serde(default = "default_min_pct")]
    min_pct: f64,
}

fn default_limit() -> usize {
    50
}
fn default_min_pct() -> f64 {
    1.0
}

#[derive(Deserialize)]
struct SnapshotQ {
    #[serde(default = "default_snapshot_limit")]
    limit: usize,
}
fn default_snapshot_limit() -> usize {
    200
}

fn parse_session(s: Option<&str>) -> after_hours::Session {
    match s.map(str::to_ascii_lowercase).as_deref() {
        Some("pre") => after_hours::Session::Pre,
        Some("post") => after_hours::Session::Post,
        Some("rth") => after_hours::Session::Rth,
        Some("closed") => after_hours::Session::Closed,
        // No explicit session → default to POST. The intended UX is
        // "open the page after 4pm and see today's after-hours movers";
        // POST is the larger of the two sessions on a typical earnings
        // day. The UI sends an explicit `session=pre` when the user
        // toggles, so the default only fires on first page load.
        _ => after_hours::Session::Post,
    }
}

async fn movers(
    State(_s): State<AppState>,
    _user: AuthUser,
    Query(q): Query<MoversQ>,
) -> Result<Json<Vec<after_hours::AfterHoursState>>, ApiError> {
    let session = parse_session(q.session.as_deref());
    let gainers = !matches!(q.direction.as_deref(), Some("losers"));
    let rows = after_hours::global().movers(session, gainers, q.limit, q.min_pct);
    Ok(Json(rows))
}

async fn snapshot(
    State(_s): State<AppState>,
    _user: AuthUser,
    Query(q): Query<SnapshotQ>,
) -> Result<Json<Vec<after_hours::AfterHoursState>>, ApiError> {
    Ok(Json(after_hours::global().snapshot(q.limit)))
}

async fn ws(State(_s): State<AppState>, upgrade: WebSocketUpgrade) -> Response {
    upgrade.on_upgrade(handle_ws)
}

async fn handle_ws(mut socket: WebSocket) {
    let store = after_hours::global();
    // Hydrate with current snapshot before streaming deltas.
    if let Ok(snap) = serde_json::to_string(&store.snapshot(200)) {
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
                Ok(row) => {
                    if let Ok(j) = serde_json::to_string(&row) {
                        if socket
                            .send(Message::Text(format!(
                                "{{\"type\":\"update\",\"row\":{j}}}"
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
