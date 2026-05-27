//! JWT auth + argon2 password hashing + axum middleware.

use crate::error::ApiError;
use crate::state::{AppMode, AppState};
use argon2::password_hash::{rand_core::OsRng, PasswordHasher, PasswordVerifier, SaltString};
use argon2::{Argon2, PasswordHash};
use axum::extract::{FromRequestParts, State};
use axum::http::request::Parts;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub exp: i64,
    pub iat: i64,
}

pub fn hash_password(plain: &str) -> Result<String, ApiError> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(plain.as_bytes(), &salt)
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("argon2: {e}")))?
        .to_string();
    Ok(hash)
}

pub fn verify_password(plain: &str, hash: &str) -> Result<bool, ApiError> {
    let parsed = PasswordHash::new(hash)
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("bad hash: {e}")))?;
    Ok(Argon2::default()
        .verify_password(plain.as_bytes(), &parsed)
        .is_ok())
}

pub fn issue_token(secret: &[u8], user_id: Uuid, ttl_hours: i64) -> Result<String, ApiError> {
    let now = Utc::now();
    let claims = Claims {
        sub: user_id,
        iat: now.timestamp(),
        exp: (now + Duration::hours(ttl_hours)).timestamp(),
    };
    encode(&Header::new(Algorithm::HS256), &claims, &EncodingKey::from_secret(secret))
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("jwt encode: {e}")))
}

pub fn decode_token(secret: &[u8], token: &str) -> Result<Claims, ApiError> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret),
        &Validation::new(Algorithm::HS256),
    )
    .map_err(|_| ApiError::Unauthorized)?;
    Ok(data.claims)
}

/// Extractor: resolves the current user.
///
/// * In `Desktop` mode, falls back to the unique `is_local = true` user so the
///   WebView never has to deal with auth.
/// * In `Web` mode, requires a valid `Authorization: Bearer <jwt>` header.
pub struct AuthUser {
    pub id: Uuid,
}

#[axum::async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    AppState: axum::extract::FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let State(app): State<AppState> =
            State::from_request_parts(parts, state).await.map_err(|_| ApiError::Unauthorized)?;

        if let Some(header) = parts.headers.get(axum::http::header::AUTHORIZATION) {
            if let Ok(s) = header.to_str() {
                if let Some(tok) = s.strip_prefix("Bearer ") {
                    // Personal Access Token path: pat_<24>_<32>
                    if let Some(rest) = tok.strip_prefix("pat_") {
                        let user_id = verify_pat(&app, rest).await?;
                        return Ok(AuthUser { id: user_id });
                    }
                    let claims = decode_token(&app.jwt_secret, tok)?;
                    return Ok(AuthUser { id: claims.sub });
                }
            }
        }

        if app.mode == AppMode::Desktop {
            let id = traderview_db::users::ensure_local(&app.pool)
                .await
                .map_err(ApiError::Internal)?;
            return Ok(AuthUser { id });
        }

        Err(ApiError::Unauthorized)
    }
}

// ===========================================================================
// Personal Access Token helpers
// ===========================================================================

const PAT_PREFIX_LEN: usize = 24;
const PAT_SECRET_LEN: usize = 32;

/// Generate a new (prefix, secret, wire_token, hash) tuple. Caller persists
/// `prefix` + `hash`; returns `wire_token` to the user exactly once.
pub fn generate_pat() -> Result<(String, String, String, String), ApiError> {
    use rand::distributions::Alphanumeric;
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let prefix: String = (&mut rng).sample_iter(Alphanumeric).take(PAT_PREFIX_LEN).map(char::from).collect();
    let secret: String = (&mut rng).sample_iter(Alphanumeric).take(PAT_SECRET_LEN).map(char::from).collect();
    let wire = format!("pat_{}_{}", prefix, secret);
    let hash = hash_password(&format!("{}_{}", prefix, secret))?;
    Ok((prefix, secret, wire, hash))
}

/// Verify a `Bearer pat_<rest>` token and return the owning user id. Bumps
/// last_used_at on success.
pub async fn verify_pat(app: &AppState, rest: &str) -> Result<Uuid, ApiError> {
    // rest = "<24 prefix>_<32 secret>"
    let mut split = rest.splitn(2, '_');
    let prefix = split.next().ok_or(ApiError::Unauthorized)?;
    let secret = split.next().ok_or(ApiError::Unauthorized)?;
    if prefix.len() != PAT_PREFIX_LEN || secret.len() != PAT_SECRET_LEN {
        return Err(ApiError::Unauthorized);
    }
    let row = traderview_db::api_tokens::find_active_by_prefix(&app.pool, prefix)
        .await.map_err(ApiError::Internal)?
        .ok_or(ApiError::Unauthorized)?;
    let candidate = format!("{}_{}", prefix, secret);
    let ok = verify_password(&candidate, &row.hash)?;
    if !ok { return Err(ApiError::Unauthorized); }
    // Rate-limit enforcement BEFORE bumping usage — throttled requests
    // shouldn't count against the visible use_count.
    let cap = row.rate_limit_per_min.max(1) as u32;
    let rl = crate::rate_limit::check_and_consume(row.id, cap);
    if !rl.allowed {
        return Err(ApiError::RateLimited {
            limit: rl.limit,
            remaining: rl.remaining,
            retry_after_secs: rl.retry_after_secs,
            reset_epoch: rl.reset_epoch,
        });
    }
    // Fire-and-forget the usage bump — don't block the request on it.
    let pool = app.pool.clone();
    let id = row.id;
    tokio::spawn(async move {
        let _ = traderview_db::api_tokens::bump_usage(&pool, id).await;
    });
    Ok(row.user_id)
}
