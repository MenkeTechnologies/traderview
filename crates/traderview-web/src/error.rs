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
    #[error("internal: {0}")]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        if let ApiError::RateLimited { limit, remaining, retry_after_secs, reset_epoch } = self {
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
            ApiError::Internal(e) => {
                tracing::error!(error = %e, "internal error");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal error".into())
            }
            ApiError::RateLimited { .. } => unreachable!(), // handled above
        };
        (status, Json(json!({ "error": msg }))).into_response()
    }
}
