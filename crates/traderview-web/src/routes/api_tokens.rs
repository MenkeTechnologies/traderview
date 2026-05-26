use crate::auth::{generate_pat, AuthUser};
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::routing::{delete, get};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use traderview_db::api_tokens::ApiTokenSummary;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api-tokens", get(list).post(create))
        .route("/api-tokens/:id", delete(revoke))
}

#[derive(Debug, Deserialize)]
struct CreateBody {
    name: String,
    scopes: Option<Vec<String>>,
    expires_at: Option<DateTime<Utc>>,
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
    let (prefix, _secret, wire, hash) = generate_pat()?;
    let row = traderview_db::api_tokens::insert(
        &s.pool, u.id, &body.name, &prefix, &hash, &scopes, body.expires_at,
    ).await.map_err(ApiError::Internal)?;
    Ok(Json(CreateResp { summary: row.into(), token: wire }))
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
