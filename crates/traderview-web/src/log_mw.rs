//! Request/response logging middleware.
//!
//! Logs every request as `method path status duration_ms` at INFO when the
//! response is 2xx/3xx and at WARN/ERROR for 4xx/5xx. For non-2xx responses
//! we buffer the response body so we can include a snippet in the log line —
//! that's what we need to debug "this widget shows nothing" failures.
//!
//! Long bodies are truncated to 4 KB so a single bad request can't fill the
//! log volume.

use axum::{
    body::{to_bytes, Body, Bytes},
    extract::Request,
    http::{Method, StatusCode, Uri},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::time::Instant;

const MAX_BODY_SNIPPET: usize = 4096;
/// Cap the body buffer at 4 MB — large successful CSV exports etc. should
/// never get sniffed (we only buffer error responses below).
const MAX_BODY_BUFFER: usize = 4 * 1024 * 1024;

pub async fn request_response_logger(req: Request, next: Next) -> Response {
    let started = Instant::now();
    let method = req.method().clone();
    let uri = req.uri().clone();

    // Run the handler.
    let response = next.run(req).await;

    let status = response.status();
    let elapsed = started.elapsed();
    let elapsed_ms = elapsed.as_secs_f64() * 1000.0;

    // Happy path — log at INFO without touching the body.
    if status.is_success() || status.is_redirection() {
        tracing::info!(
            method = %method,
            path = %uri.path(),
            status = status.as_u16(),
            elapsed_ms = format!("{elapsed_ms:.1}"),
            "request ok"
        );
        return response;
    }

    // Failure path — sniff the body so the log explains WHY.
    log_failure(method, uri, status, elapsed_ms, response).await
}

async fn log_failure(
    method: Method,
    uri: Uri,
    status: StatusCode,
    elapsed_ms: f64,
    response: Response,
) -> Response {
    let (parts, body) = response.into_parts();
    let bytes = match to_bytes(body, MAX_BODY_BUFFER).await {
        Ok(b) => b,
        Err(e) => {
            // Body read failed — log what we know and pass a synthetic empty
            // body downstream so the client at least gets a status code.
            tracing::error!(
                method = %method,
                path = %uri.path(),
                status = status.as_u16(),
                elapsed_ms = format!("{elapsed_ms:.1}"),
                err = %e,
                "failed to buffer response body for error logging"
            );
            return Response::from_parts(parts, Body::empty()).into_response();
        }
    };
    let snippet = snippet_for_log(&bytes);
    if status.is_server_error() {
        tracing::error!(
            method = %method,
            path = %uri.path(),
            status = status.as_u16(),
            elapsed_ms = format!("{elapsed_ms:.1}"),
            body = %snippet,
            "server error"
        );
    } else {
        tracing::warn!(
            method = %method,
            path = %uri.path(),
            status = status.as_u16(),
            elapsed_ms = format!("{elapsed_ms:.1}"),
            body = %snippet,
            "client error"
        );
    }
    Response::from_parts(parts, Body::from(bytes)).into_response()
}

fn snippet_for_log(bytes: &Bytes) -> String {
    let cap = MAX_BODY_SNIPPET.min(bytes.len());
    let text = String::from_utf8_lossy(&bytes[..cap]).to_string();
    if bytes.len() > MAX_BODY_SNIPPET {
        format!("{text}…[+{} bytes]", bytes.len() - MAX_BODY_SNIPPET)
    } else {
        text
    }
}
