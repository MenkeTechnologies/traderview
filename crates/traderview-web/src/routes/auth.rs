use crate::auth::{hash_password, issue_token, verify_password, AuthUser};
use crate::error::ApiError;
use crate::state::{AppMode, AppState};
use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/config", get(config))
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/auth/me", get(me))
}

async fn health() -> &'static str { "ok" }

#[derive(Serialize)]
struct ConfigResponse {
    mode: &'static str,
    version: &'static str,
    brokers: Vec<&'static str>,
}

async fn config(State(s): State<AppState>) -> Json<ConfigResponse> {
    Json(ConfigResponse {
        mode: match s.mode {
            AppMode::Desktop => "desktop",
            AppMode::Web => "web",
        },
        version: env!("CARGO_PKG_VERSION"),
        brokers: traderview_import::supported_sources().to_vec(),
    })
}

#[derive(Deserialize)]
struct RegisterBody {
    email: String,
    password: String,
    #[serde(default)]
    display_name: String,
}

#[derive(Serialize)]
struct TokenResponse {
    token: String,
    user_id: Uuid,
}

async fn register(
    State(s): State<AppState>,
    Json(body): Json<RegisterBody>,
) -> Result<Json<TokenResponse>, ApiError> {
    if s.mode == AppMode::Desktop {
        return Err(ApiError::Forbidden);
    }
    if body.password.len() < 8 {
        return Err(ApiError::BadRequest("password must be >= 8 chars".into()));
    }
    if traderview_db::users::find_by_email(&s.pool, &body.email)
        .await
        .map_err(ApiError::Internal)?
        .is_some()
    {
        return Err(ApiError::Conflict("email already registered".into()));
    }
    let hash = hash_password(&body.password)?;
    let user_id = traderview_db::users::create(&s.pool, &body.email, &hash, &body.display_name)
        .await
        .map_err(ApiError::Internal)?;
    let token = issue_token(&s.jwt_secret, user_id, 24 * 30)?;
    Ok(Json(TokenResponse { token, user_id }))
}

#[derive(Deserialize)]
struct LoginBody {
    email: String,
    password: String,
}

async fn login(
    State(s): State<AppState>,
    Json(body): Json<LoginBody>,
) -> Result<Json<TokenResponse>, ApiError> {
    let row = traderview_db::users::find_by_email(&s.pool, &body.email)
        .await
        .map_err(ApiError::Internal)?
        .ok_or(ApiError::Unauthorized)?;
    let hash = row.password_hash.ok_or(ApiError::Unauthorized)?;
    if !verify_password(&body.password, &hash)? {
        return Err(ApiError::Unauthorized);
    }
    let token = issue_token(&s.jwt_secret, row.user.id, 24 * 30)?;
    Ok(Json(TokenResponse {
        token,
        user_id: row.user.id,
    }))
}

#[derive(Serialize)]
struct MeResponse {
    id: Uuid,
    email: Option<String>,
    display_name: String,
    is_local: bool,
}

async fn me(State(s): State<AppState>, user: AuthUser) -> Result<Json<MeResponse>, ApiError> {
    let u = traderview_db::users::find_by_id(&s.pool, user.id)
        .await
        .map_err(ApiError::Internal)?
        .ok_or(ApiError::Unauthorized)?;
    Ok(Json(MeResponse {
        id: u.id,
        email: u.email,
        display_name: u.display_name,
        is_local: u.is_local,
    }))
}
