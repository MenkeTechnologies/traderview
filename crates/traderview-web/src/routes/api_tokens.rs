use crate::auth::{generate_pat, AuthUser};
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::routing::{delete, get, patch};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use traderview_db::api_tokens::ApiTokenSummary;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api-tokens", get(list).post(create))
        .route("/api-tokens/:id", delete(revoke))
        .route("/api-tokens/:id/rate-limit", patch(set_rate_limit))
}

#[derive(Debug, Deserialize)]
struct CreateBody {
    name: String,
    scopes: Option<Vec<String>>,
    expires_at: Option<DateTime<Utc>>,
    rate_limit_per_min: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct RateLimitBody {
    rate_limit_per_min: i32,
}

#[derive(Debug, Serialize)]
struct CreateResp {
    summary: ApiTokenSummary,
    /// Plain wire token — shown ONCE; never recoverable afterwards.
    token: String,
}

async fn list(
    State(s): State<AppState>,
    u: AuthUser,
) -> Result<Json<Vec<ApiTokenSummary>>, ApiError> {
    Ok(Json(traderview_db::api_tokens::list_for_user(&s.pool, u.id)
        .await.map_err(ApiError::Internal)?))
}

async fn create(
    State(s): State<AppState>,
    u: AuthUser,
    Json(body): Json<CreateBody>,
) -> Result<Json<CreateResp>, ApiError> {
    if body.name.trim().is_empty() {
        return Err(ApiError::BadRequest("name required".into()));
    }
    let scopes = body.scopes.unwrap_or_else(|| vec!["read".into()]);
    for sc in &scopes {
        if !matches!(sc.as_str(), "read" | "write" | "admin") {
            return Err(ApiError::BadRequest(format!("unknown scope: {}", sc)));
        }
    }
    if let Some(rl) = body.rate_limit_per_min {
        if !(1..=10_000).contains(&rl) {
            return Err(ApiError::BadRequest("rate_limit_per_min must be 1..=10000".into()));
        }
    }
    let (prefix, _secret, wire, hash) = generate_pat()?;
    let row = traderview_db::api_tokens::insert(
        &s.pool,
        traderview_db::api_tokens::NewToken {
            user_id: u.id,
            name: &body.name,
            prefix: &prefix,
            hash: &hash,
            scopes: &scopes,
            expires_at: body.expires_at,
            rate_limit_per_min: body.rate_limit_per_min,
        },
    ).await.map_err(ApiError::Internal)?;
    Ok(Json(CreateResp { summary: row.into(), token: wire }))
}

async fn set_rate_limit(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<RateLimitBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let ok = traderview_db::api_tokens::set_rate_limit(&s.pool, u.id, id, body.rate_limit_per_min)
        .await.map_err(ApiError::Internal)?;
    if !ok { return Err(ApiError::NotFound); }
    Ok(Json(serde_json::json!({ "ok": true, "rate_limit_per_min": body.rate_limit_per_min })))
}

async fn revoke(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let ok = traderview_db::api_tokens::revoke(&s.pool, u.id, id)
        .await.map_err(ApiError::Internal)?;
    if !ok { return Err(ApiError::NotFound); }
    Ok(Json(serde_json::json!({ "revoked": true })))
}
