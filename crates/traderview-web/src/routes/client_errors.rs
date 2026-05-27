//! Receive browser-side errors and write them to the server log.
//!
//! The frontend POSTs every uncaught error / unhandled rejection / failed
//! `fetch()` to this endpoint so they land in
//! `~/Library/Application Support/traderview/traderview.log` alongside the
//! backend trace — one log to grep when debugging.
//!
//! Authentication intentionally NOT required — boot errors fire before the
//! token is even loaded. The desktop binds to 127.0.0.1 only, so there is
//! no exposure beyond the local app.

use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use serde::Deserialize;

pub fn router() -> Router<AppState> {
    Router::new().route("/client-errors", post(receive))
}

#[derive(Debug, Deserialize)]
struct ClientErr {
    #[serde(default)]
    kind: Option<String>, // "error" | "unhandledrejection" | "api-fail" | "log"
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    stack: Option<String>,
    #[serde(default)]
    src: Option<String>,
    #[serde(default)]
    line: Option<u32>,
    #[serde(default)]
    col: Option<u32>,
    #[serde(default)]
    view: Option<String>,
    #[serde(default)]
    href: Option<String>,
    #[serde(default)]
    ua: Option<String>,
    #[serde(default)]
    extra: Option<serde_json::Value>,
}

async fn receive(
    State(_s): State<AppState>,
    Json(e): Json<ClientErr>,
) -> Result<StatusCode, ApiError> {
    let kind = e.kind.as_deref().unwrap_or("error");
    let level_err = matches!(kind, "error" | "unhandledrejection" | "api-fail");
    let msg = e.message.as_deref().unwrap_or("(no message)");
    if level_err {
        tracing::error!(
            target: "client",
            kind = %kind,
            view = e.view.as_deref().unwrap_or("?"),
            src = e.src.as_deref().unwrap_or("?"),
            line = e.line.unwrap_or(0),
            col = e.col.unwrap_or(0),
            href = e.href.as_deref().unwrap_or("?"),
            ua = e.ua.as_deref().unwrap_or("?"),
            stack = e.stack.as_deref().unwrap_or(""),
            extra = ?e.extra,
            "{msg}"
        );
    } else {
        tracing::info!(
            target: "client",
            kind = %kind,
            view = e.view.as_deref().unwrap_or("?"),
            "{msg}"
        );
    }
    Ok(StatusCode::NO_CONTENT)
}
