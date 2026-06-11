use crate::auth::{hash_password, issue_token, verify_password, AuthUser};
use crate::error::ApiError;
use crate::rate_limit;
use crate::state::{AppMode, AppState};
use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Stable per-email rate-limit key. The rate_limit module is keyed by
/// Uuid; we derive a deterministic Uuid from `auth-login:{email}` via
/// the workspace's default DefaultHasher so repeated attempts on the
/// same email map to the same bucket. Two SipHash rounds (different
/// initial state via the prefix bytes) fill the 16 byte buffer.
/// Switch to per-IP if a future middleware exposes the client address.
fn login_rate_key(email: &str) -> Uuid {
    use std::hash::{Hash, Hasher};
    let normalized = format!("auth-login:{}", email.trim().to_ascii_lowercase());
    let mut h1 = std::collections::hash_map::DefaultHasher::new();
    normalized.hash(&mut h1);
    let hi = h1.finish();
    let mut h2 = std::collections::hash_map::DefaultHasher::new();
    // Distinct prefix → distinct SipHash sequence for the low 64 bits.
    "lo:".hash(&mut h2);
    normalized.hash(&mut h2);
    let lo = h2.finish();
    let mut bytes = [0u8; 16];
    bytes[..8].copy_from_slice(&hi.to_be_bytes());
    bytes[8..].copy_from_slice(&lo.to_be_bytes());
    Uuid::from_bytes(bytes)
}

/// Per-email login/register attempts allowed per minute. 10 is enough
/// for legitimate password fumbles + a forgotten-password retry burst;
/// far below the surface needed for credential-stuffing to work.
const LOGIN_RATE_PER_MIN: u32 = 10;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/config", get(config))
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/auth/me", get(me))
}

async fn health() -> &'static str {
    "ok"
}

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
    // Throttle per-email to prevent registration spam / enumeration
    // probes via "is this email already taken" reconnaissance.
    if !rate_limit::check_and_consume(login_rate_key(&body.email), LOGIN_RATE_PER_MIN).allowed {
        return Err(ApiError::BadRequest(
            "too many register attempts; slow down and try again in a minute".into(),
        ));
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
    // Throttle per-email so credential-stuffing has to find a new
    // address every 10 attempts/minute. The bucket exists per-process
    // (DashMap), so this defends a single host; a multi-host fleet
    // would need a Redis or DB-backed limiter — out of scope here.
    if !rate_limit::check_and_consume(login_rate_key(&body.email), LOGIN_RATE_PER_MIN).allowed {
        return Err(ApiError::Unauthorized);
    }
    // Timing-oracle defense: always run a verify_password (against a
    // dummy hash on the unknown-email branch) so the wall-clock cost
    // of "user not found" matches "user found, wrong password". The
    // dummy hash is for the literal string "x" so a real password
    // would obviously fail to verify.
    let row = traderview_db::users::find_by_email(&s.pool, &body.email)
        .await
        .map_err(ApiError::Internal)?;
    let (hash, real_user_id) = match row {
        Some(r) => (r.password_hash, Some(r.user.id)),
        None => (None, None),
    };
    let hash_str: String = hash.unwrap_or_else(dummy_password_hash);
    let ok = verify_password(&body.password, &hash_str)?;
    if !ok || real_user_id.is_none() {
        return Err(ApiError::Unauthorized);
    }
    let user_id = real_user_id.expect("checked Some above");
    let token = issue_token(&s.jwt_secret, user_id, 24 * 30)?;
    Ok(Json(TokenResponse { token, user_id }))
}

/// A constant Argon2 hash used to consume CPU on the unknown-email
/// path of `login` so an attacker can't distinguish "user exists" vs
/// "user doesn't exist" by wall-clock timing. Generated with
/// password = "x" using the same params as `hash_password`.
fn dummy_password_hash() -> String {
    // OWASP-recommended Argon2id params (the same hash_password uses).
    // Hardcoded so we don't pay the cost of regenerating on every
    // unknown-email login; only the verify_password CPU work matters.
    "$argon2id$v=19$m=19456,t=2,p=1$VGltZUNvbnN0YW50RHVtbXk$\
     OmJ4+IkjB10MtKp2sBPGCYr0Y6tSUyKM3FvjW1Ag0bU"
        .replace(['\n', ' '], "")
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
