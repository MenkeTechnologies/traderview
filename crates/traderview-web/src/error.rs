use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("unauthorized")]
    Unauthorized,
    #[error("forbidden")]
    Forbidden,
    #[error("not found")]
    NotFound,
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("conflict: {0}")]
    Conflict(String),
    /// PAT rate-limit exhausted. Emits 429 + Retry-After + X-RateLimit-*.
    #[error("rate limited: retry in {retry_after_secs}s")]
    RateLimited {
        limit: u32,
        remaining: u32,
        retry_after_secs: u64,
        reset_epoch: i64,
    },
    #[error("database: {0}")]
    Db(#[from] sqlx::Error),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("internal: {0}")]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        if let ApiError::RateLimited {
            limit,
            remaining,
            retry_after_secs,
            reset_epoch,
        } = self
        {
            let body = Json(json!({
                "error": "rate limited",
                "limit": limit,
                "remaining": remaining,
                "retry_after_secs": retry_after_secs,
                "reset_epoch": reset_epoch,
            }));
            let mut resp = (StatusCode::TOO_MANY_REQUESTS, body).into_response();
            let h = resp.headers_mut();
            if let Ok(v) = HeaderValue::from_str(&retry_after_secs.to_string()) {
                h.insert("retry-after", v);
            }
            if let Ok(v) = HeaderValue::from_str(&limit.to_string()) {
                h.insert("x-ratelimit-limit", v);
            }
            if let Ok(v) = HeaderValue::from_str(&remaining.to_string()) {
                h.insert("x-ratelimit-remaining", v);
            }
            if let Ok(v) = HeaderValue::from_str(&reset_epoch.to_string()) {
                h.insert("x-ratelimit-reset", v);
            }
            return resp;
        }
        let (status, msg) = match &self {
            ApiError::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            ApiError::Forbidden => (StatusCode::FORBIDDEN, self.to_string()),
            ApiError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            ApiError::BadRequest(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            ApiError::Conflict(_) => (StatusCode::CONFLICT, self.to_string()),
            ApiError::Db(e) => {
                tracing::error!(error = %e, "db error");
                (StatusCode::INTERNAL_SERVER_ERROR, "database error".into())
            }
            ApiError::Io(e) => {
                tracing::error!(error = %e, "io error");
                (StatusCode::INTERNAL_SERVER_ERROR, "io error".into())
            }
            ApiError::Internal(e) => {
                tracing::error!(error = %e, "internal error");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal error".into())
            }
            ApiError::RateLimited { .. } => unreachable!(), // handled above
        };
        (status, Json(json!({ "error": msg }))).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;

    async fn body_string(resp: Response) -> (StatusCode, String) {
        let status = resp.status();
        let bytes = to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
        (status, String::from_utf8_lossy(&bytes).to_string())
    }

    // ── Status-code mapping per variant ───────────────────────────────────

    #[test]
    fn unauthorized_returns_401() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let (status, body) = rt.block_on(body_string(ApiError::Unauthorized.into_response()));
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert!(body.contains("unauthorized"));
    }

    #[test]
    fn forbidden_returns_403() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let (status, body) = rt.block_on(body_string(ApiError::Forbidden.into_response()));
        assert_eq!(status, StatusCode::FORBIDDEN);
        assert!(body.contains("forbidden"));
    }

    #[test]
    fn not_found_returns_404() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let (status, _) = rt.block_on(body_string(ApiError::NotFound.into_response()));
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[test]
    fn bad_request_returns_400_with_message_passthrough() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let err = ApiError::BadRequest("symbol is required".into());
        let (status, body) = rt.block_on(body_string(err.into_response()));
        assert_eq!(status, StatusCode::BAD_REQUEST);
        // The exact user-supplied message must reach the client — otherwise
        // /api/calc/* etc. lose their explanatory error context.
        assert!(body.contains("symbol is required"));
    }

    #[test]
    fn conflict_returns_409_with_message() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let err = ApiError::Conflict("watchlist exists".into());
        let (status, body) = rt.block_on(body_string(err.into_response()));
        assert_eq!(status, StatusCode::CONFLICT);
        assert!(body.contains("watchlist exists"));
    }

    // ── Rate-limit special path: 429 + Retry-After + X-RateLimit-* ────────

    #[test]
    fn rate_limited_emits_429_with_all_rate_headers() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let err = ApiError::RateLimited {
            limit: 100,
            remaining: 0,
            retry_after_secs: 30,
            reset_epoch: 1700000000,
        };
        let resp = err.into_response();
        let status = resp.status();
        let headers = resp.headers().clone();
        assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
        // All four headers documented in the doc-comment must be present.
        assert_eq!(headers.get("retry-after").unwrap(), "30");
        assert_eq!(headers.get("x-ratelimit-limit").unwrap(), "100");
        assert_eq!(headers.get("x-ratelimit-remaining").unwrap(), "0");
        assert_eq!(headers.get("x-ratelimit-reset").unwrap(), "1700000000");
        let (_, body) = rt.block_on(body_string(resp));
        assert!(body.contains("\"rate limited\""));
        assert!(body.contains("\"retry_after_secs\":30"));
    }

    // ── Internal errors don't leak chain to the wire ──────────────────────

    #[test]
    fn internal_error_returns_500_without_leaking_chain() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let err = ApiError::Internal(anyhow::anyhow!("DB connection string: postgres://secret"));
        let (status, body) = rt.block_on(body_string(err.into_response()));
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        // The chain (which may contain secrets) goes to the log, NOT to the
        // client; the wire only sees the generic "internal error" message.
        assert!(body.contains("internal error"));
        assert!(!body.contains("postgres://secret"));
    }

    #[test]
    fn db_error_returns_500_with_generic_message() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        // sqlx::Error::RowNotFound is the cheapest variant to fabricate.
        let err: ApiError = sqlx::Error::RowNotFound.into();
        let (status, body) = rt.block_on(body_string(err.into_response()));
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert!(body.contains("database error"));
    }

    // ── From-impls preserve the variant identity ──────────────────────────

    #[test]
    fn io_error_converts_via_from_and_returns_500() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "missing");
        let err: ApiError = io_err.into();
        let (status, body) = rt.block_on(body_string(err.into_response()));
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert!(body.contains("io error"));
    }
}
