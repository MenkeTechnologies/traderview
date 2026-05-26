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
