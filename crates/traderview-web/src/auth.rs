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
    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret),
    )
    .map_err(|e| ApiError::Internal(anyhow::anyhow!("jwt encode: {e}")))
}

/// Query-string token shape used by every WS upgrade handler. The
/// browser can't set Authorization on a WS handshake, so the token
/// rides as `?token=<jwt>` instead.
#[derive(Debug, Deserialize)]
pub struct WsTokenQuery {
    pub token: Option<String>,
}

/// Gate a WebSocket upgrade. Returns Ok when the request is authorized
/// (valid `?token=<jwt>` in Web mode, or Desktop mode where the local
/// user is always present). Returns `ApiError::Unauthorized` otherwise.
///
/// Use this from EVERY `async fn ws(...)` upgrade handler — the
/// per-route WS handlers (uoa_stream, halts, insider_stream, etc.)
/// historically omitted any check and were callable by
/// unauthenticated clients.
pub fn require_ws_auth(state: &AppState, token: Option<&str>) -> Result<(), ApiError> {
    if matches!(state.mode, AppMode::Desktop) {
        return Ok(());
    }
    let tok = token.ok_or(ApiError::Unauthorized)?;
    decode_token(&state.jwt_secret, tok).map(|_| ())
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
#[derive(Clone, Copy)]
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
        let State(app): State<AppState> = State::from_request_parts(parts, state)
            .await
            .map_err(|_| ApiError::Unauthorized)?;

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

        // Query-param token: for file downloads (CSV/HTML) where the browser
        // can't attach an Authorization header to <a download>. Same secret;
        // only used when the header path didn't match. Hand-decodes %xx since
        // we don't want to drag in a urlencoding dep just for this one path.
        //
        // GATED TO GET (and HEAD/OPTIONS) ONLY. A token placed in a URL
        // leaks to browser history, referer headers, reverse-proxy logs,
        // and copy/paste — so it must not authorize mutating requests.
        // Without this gate a leaked download URL would let an attacker
        // POST/PATCH/DELETE from anywhere they could exfiltrate it.
        let method = &parts.method;
        if matches!(
            *method,
            axum::http::Method::GET | axum::http::Method::HEAD | axum::http::Method::OPTIONS
        ) {
            if let Some(q) = parts.uri.query() {
                for kv in q.split('&') {
                    if let Some(tok) = kv.strip_prefix("token=") {
                        let raw = percent_decode_simple(tok);
                        if let Some(rest) = raw.strip_prefix("pat_") {
                            let user_id = verify_pat(&app, rest).await?;
                            return Ok(AuthUser { id: user_id });
                        }
                        if let Ok(claims) = decode_token(&app.jwt_secret, &raw) {
                            return Ok(AuthUser { id: claims.sub });
                        }
                    }
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

/// Minimal percent-decoder for `?token=...` query values. Handles `%XX`
/// (case-insensitive) and `+` → space. Returns the input unchanged on
/// malformed escapes — bad tokens fail JWT/PAT verification downstream.
fn percent_decode_simple(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'%' && i + 2 < bytes.len() {
            let hi = (bytes[i + 1] as char).to_digit(16);
            let lo = (bytes[i + 2] as char).to_digit(16);
            if let (Some(h), Some(l)) = (hi, lo) {
                out.push(((h << 4) | l) as u8);
                i += 3;
                continue;
            }
        }
        out.push(if b == b'+' { b' ' } else { b });
        i += 1;
    }
    String::from_utf8(out).unwrap_or_else(|_| s.to_string())
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
    let prefix: String = (&mut rng)
        .sample_iter(Alphanumeric)
        .take(PAT_PREFIX_LEN)
        .map(char::from)
        .collect();
    let secret: String = (&mut rng)
        .sample_iter(Alphanumeric)
        .take(PAT_SECRET_LEN)
        .map(char::from)
        .collect();
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
        .await
        .map_err(ApiError::Internal)?
        .ok_or(ApiError::Unauthorized)?;
    let candidate = format!("{}_{}", prefix, secret);
    let ok = verify_password(&candidate, &row.hash)?;
    if !ok {
        return Err(ApiError::Unauthorized);
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    // ===========================================================================
    // percent_decode_simple — query-token decoder used by file downloads
    // ===========================================================================

    #[test]
    fn percent_decode_handles_basic_escapes() {
        assert_eq!(percent_decode_simple("hello%20world"), "hello world");
        assert_eq!(percent_decode_simple("a%2Bb%3Dc"), "a+b=c");
        assert_eq!(percent_decode_simple("plus+sign"), "plus sign");
    }

    #[test]
    fn percent_decode_passes_through_malformed_escapes() {
        // %XX with non-hex chars is left as-is — downstream JWT/PAT verify
        // will reject; we don't want to crash on garbage input.
        assert_eq!(percent_decode_simple("oops%GG"), "oops%GG");
        assert_eq!(percent_decode_simple("trail%"), "trail%");
        assert_eq!(percent_decode_simple("trail%2"), "trail%2");
    }

    #[test]
    fn percent_decode_preserves_unescaped_jwt_chars() {
        // Real JWTs contain only [A-Za-z0-9._-] so nothing should change.
        let jwt = "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJ4In0.signature_part";
        assert_eq!(percent_decode_simple(jwt), jwt);
    }

    // ===========================================================================
    // hash_password / verify_password — Argon2 round-trip
    // ===========================================================================

    #[test]
    fn hashing_produces_argon2_phc_format() {
        let h = hash_password("hunter2").unwrap();
        // Argon2 PHC strings always start with "$argon2".
        assert!(h.starts_with("$argon2"), "got {}", h);
    }

    #[test]
    fn hashing_is_salted_so_same_password_yields_different_hash() {
        let h1 = hash_password("samepw").unwrap();
        let h2 = hash_password("samepw").unwrap();
        assert_ne!(h1, h2, "argon2 must use a fresh salt per hash");
    }

    #[test]
    fn verify_returns_true_for_correct_password() {
        let h = hash_password("correct horse battery staple").unwrap();
        assert!(verify_password("correct horse battery staple", &h).unwrap());
    }

    #[test]
    fn verify_returns_false_for_wrong_password() {
        let h = hash_password("hunter2").unwrap();
        assert!(!verify_password("hunter3", &h).unwrap());
    }

    #[test]
    fn verify_returns_false_for_empty_password_against_real_hash() {
        let h = hash_password("nonempty").unwrap();
        assert!(!verify_password("", &h).unwrap());
    }

    #[test]
    fn verify_errors_on_malformed_hash_string() {
        // PasswordHash::new fails parse → Internal error.
        let err = verify_password("any", "not-a-real-hash");
        assert!(err.is_err());
    }

    #[test]
    fn hash_password_accepts_empty_string() {
        // Argon2 doesn't require non-empty input; we mirror that here.
        let h = hash_password("").unwrap();
        assert!(h.starts_with("$argon2"));
        assert!(verify_password("", &h).unwrap());
    }

    // ===========================================================================
    // issue_token / decode_token — JWT round-trip
    // ===========================================================================

    #[test]
    fn issued_token_decodes_back_to_same_user_id() {
        let secret = b"a-very-secret-key-only-for-tests";
        let id = Uuid::new_v4();
        let tok = issue_token(secret, id, 1).unwrap();
        let claims = decode_token(secret, &tok).unwrap();
        assert_eq!(claims.sub, id);
    }

    #[test]
    fn issued_token_includes_exp_in_the_future() {
        let secret = b"another-secret";
        let id = Uuid::new_v4();
        let before = Utc::now().timestamp();
        let tok = issue_token(secret, id, 24).unwrap();
        let claims = decode_token(secret, &tok).unwrap();
        // exp should be roughly 24h ahead — at minimum 23.5h to leave slack.
        assert!(claims.exp - before >= 23 * 3600 + 1800);
        assert!(claims.exp - before <= 25 * 3600);
        assert!(claims.iat >= before);
    }

    #[test]
    fn decode_rejects_token_signed_with_different_secret() {
        let id = Uuid::new_v4();
        let tok = issue_token(b"secret-A", id, 1).unwrap();
        let err = decode_token(b"secret-B", &tok);
        assert!(matches!(err, Err(ApiError::Unauthorized)));
    }

    #[test]
    fn decode_rejects_garbage_token() {
        let err = decode_token(b"any", "definitely-not-a-jwt");
        assert!(matches!(err, Err(ApiError::Unauthorized)));
    }

    #[test]
    fn decode_rejects_expired_token() {
        // Negative TTL → exp in the past → jsonwebtoken Validation rejects it.
        let secret = b"secret";
        let id = Uuid::new_v4();
        let tok = issue_token(secret, id, -1).unwrap();
        let err = decode_token(secret, &tok);
        assert!(matches!(err, Err(ApiError::Unauthorized)));
    }

    // ===========================================================================
    // generate_pat — shape, prefix length, hash verifies
    // ===========================================================================

    #[test]
    fn generate_pat_returns_24_char_prefix_and_32_char_secret() {
        let (prefix, secret, _, _) = generate_pat().unwrap();
        assert_eq!(prefix.len(), PAT_PREFIX_LEN);
        assert_eq!(secret.len(), PAT_SECRET_LEN);
    }

    #[test]
    fn generate_pat_wire_token_is_pat_underscore_format() {
        let (prefix, secret, wire, _) = generate_pat().unwrap();
        let expected = format!("pat_{}_{}", prefix, secret);
        assert_eq!(wire, expected);
        assert!(wire.starts_with("pat_"));
    }

    #[test]
    fn generate_pat_hash_verifies_against_prefix_secret_concatenation() {
        let (prefix, secret, _, hash) = generate_pat().unwrap();
        let candidate = format!("{}_{}", prefix, secret);
        assert!(verify_password(&candidate, &hash).unwrap());
    }

    #[test]
    fn generate_pat_hash_rejects_wire_token_directly() {
        // The hash is keyed on `{prefix}_{secret}` — NOT the full `pat_...`
        // wire form. Verifies the contract documented in verify_pat().
        let (_, _, wire, hash) = generate_pat().unwrap();
        assert!(!verify_password(&wire, &hash).unwrap());
    }

    #[test]
    fn generate_pat_uses_only_alphanumeric_characters() {
        let (prefix, secret, _, _) = generate_pat().unwrap();
        assert!(prefix.chars().all(|c| c.is_ascii_alphanumeric()));
        assert!(secret.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn generate_pat_produces_unique_tokens_across_calls() {
        // 56 alnum chars from rand — collision probability is astronomically low.
        let (p1, s1, _, _) = generate_pat().unwrap();
        let (p2, s2, _, _) = generate_pat().unwrap();
        assert_ne!((p1, s1), (p2, s2));
    }
}
