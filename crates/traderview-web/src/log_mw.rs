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

#[cfg(test)]
mod tests {
    use super::*;

    // ── snippet_for_log: error-body truncation for log lines ──────────────

    #[test]
    fn short_body_passes_through_unchanged() {
        let b = Bytes::from_static(b"{\"error\":\"bad request\"}");
        let s = snippet_for_log(&b);
        assert_eq!(s, r#"{"error":"bad request"}"#);
    }

    #[test]
    fn empty_body_yields_empty_string() {
        let b = Bytes::new();
        assert_eq!(snippet_for_log(&b), "");
    }

    #[test]
    fn body_at_exact_cap_is_not_truncated() {
        // The cap is the inclusive boundary — bodies of exactly MAX_BODY_SNIPPET
        // bytes must NOT get the truncation suffix because there's nothing left
        // to drop. Off-by-one here would mislead log readers.
        let b = Bytes::from(vec![b'x'; MAX_BODY_SNIPPET]);
        let s = snippet_for_log(&b);
        assert_eq!(s.len(), MAX_BODY_SNIPPET);
        assert!(!s.contains("…[+"));
    }

    #[test]
    fn oversized_body_truncates_and_reports_overflow() {
        let extra = 1234usize;
        let total = MAX_BODY_SNIPPET + extra;
        let b = Bytes::from(vec![b'a'; total]);
        let s = snippet_for_log(&b);
        // Suffix format is "…[+N bytes]" — the count must equal exact overflow.
        assert!(
            s.ends_with(&format!("…[+{extra} bytes]")),
            "suffix wrong: {s}"
        );
        // Visible body content is exactly MAX_BODY_SNIPPET bytes of 'a'.
        let prefix = &s[..MAX_BODY_SNIPPET];
        assert!(prefix.bytes().all(|c| c == b'a'));
    }

    #[test]
    fn invalid_utf8_does_not_panic_and_uses_replacement_chars() {
        // 4xx/5xx bodies may include binary or mis-encoded data — must never
        // panic the logger middleware.
        let b = Bytes::from_static(&[0xff, 0xfe, 0xfd, b'!']);
        let s = snippet_for_log(&b);
        // from_utf8_lossy emits U+FFFD for each bad byte.
        assert!(s.contains('\u{FFFD}'));
        assert!(s.ends_with('!'));
    }

    #[test]
    fn cap_constants_match_documented_limits() {
        // The README and module docstring promise 4 KB snippet / 4 MB buffer
        // caps. If anyone tunes these by accident, the log format changes for
        // every error in production — this pins the contract.
        assert_eq!(MAX_BODY_SNIPPET, 4096);
        assert_eq!(MAX_BODY_BUFFER, 4 * 1024 * 1024);
    }

    #[test]
    fn truncation_preserves_leading_bytes_not_trailing() {
        // The first bytes of a JSON error body are typically the most useful
        // ({"error":"..."}). Snipped should keep the head, drop the tail.
        let mut bytes = Vec::with_capacity(MAX_BODY_SNIPPET + 100);
        bytes.extend_from_slice(b"{\"error\":\"first\"");
        bytes.extend(std::iter::repeat(b'X').take(MAX_BODY_SNIPPET));
        let b = Bytes::from(bytes);
        let s = snippet_for_log(&b);
        assert!(s.starts_with("{\"error\":\"first\""));
    }
}
