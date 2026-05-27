//! HTTP routes. Single-file for now; split per-resource once any handler
//! grows past ~80 lines.

use crate::auth::{hash_password, issue_token, verify_password, AuthUser};
use crate::error::ApiError;
use crate::state::{AppMode, AppState};
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use traderview_core::{stats, JournalEntry, Trade};
use traderview_db::repo;
use uuid::Uuid;

pub fn api_router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/config", get(config))
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/auth/me", get(me))
        .route("/accounts", get(list_accounts))
        .route("/trades", get(list_trades))
        .route("/stats/summary", get(stats_summary))
        .route("/stats/equity", get(stats_equity))
        .route("/journal/:day", get(journal_for_day))
        .nest("/expense", crate::expense_routes::router())
}

// --- health / config -----------------------------------------------------

async fn health() -> &'static str {
    "ok"
}

#[derive(Serialize)]
struct ConfigResponse {
    mode: &'static str,
}

async fn config(State(s): State<AppState>) -> Json<ConfigResponse> {
    Json(ConfigResponse {
        mode: match s.mode {
            AppMode::Desktop => "desktop",
            AppMode::Web => "web",
        },
    })
}

// --- auth ----------------------------------------------------------------

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
    let hash = hash_password(&body.password)?;
    let existing: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM users WHERE lower(email) = lower($1) LIMIT 1",
    )
    .bind(&body.email)
    .fetch_optional(&s.pool)
    .await?;
    if existing.is_some() {
        return Err(ApiError::Conflict("email already registered".into()));
    }
    let (user_id,): (Uuid,) = sqlx::query_as(
        "INSERT INTO users (email, password_hash, display_name)
              VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(&body.email)
    .bind(&hash)
    .bind(&body.display_name)
    .fetch_one(&s.pool)
    .await?;
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
    let row: Option<(Uuid, Option<String>)> = sqlx::query_as(
        "SELECT id, password_hash FROM users WHERE lower(email) = lower($1) LIMIT 1",
    )
    .bind(&body.email)
    .fetch_optional(&s.pool)
    .await?;
    let (user_id, hash) = row.ok_or(ApiError::Unauthorized)?;
    let hash = hash.ok_or(ApiError::Unauthorized)?;
    if !verify_password(&body.password, &hash)? {
        return Err(ApiError::Unauthorized);
    }
    let token = issue_token(&s.jwt_secret, user_id, 24 * 30)?;
    Ok(Json(TokenResponse { token, user_id }))
}

#[derive(Serialize)]
struct MeResponse {
    id: Uuid,
    email: Option<String>,
    display_name: String,
    is_local: bool,
}

async fn me(State(s): State<AppState>, user: AuthUser) -> Result<Json<MeResponse>, ApiError> {
    let row: (Uuid, Option<String>, String, bool) = sqlx::query_as(
        "SELECT id, email, display_name, is_local FROM users WHERE id = $1",
    )
    .bind(user.id)
    .fetch_one(&s.pool)
    .await?;
    Ok(Json(MeResponse {
        id: row.0,
        email: row.1,
        display_name: row.2,
        is_local: row.3,
    }))
}

// --- accounts / trades ---------------------------------------------------

async fn list_accounts(
    State(s): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<traderview_core::Account>>, ApiError> {
    Ok(Json(
        repo::list_accounts(&s.pool, user.id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

#[derive(Deserialize)]
struct TradesQuery {
    account_id: Uuid,
    #[serde(default = "default_limit")]
    limit: i64,
    #[serde(default)]
    offset: i64,
}
fn default_limit() -> i64 {
    50
}

async fn list_trades(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<TradesQuery>,
) -> Result<Json<Vec<Trade>>, ApiError> {
    ensure_account_owner(&s, user.id, q.account_id).await?;
    Ok(Json(
        repo::list_trades(&s.pool, q.account_id, q.limit, q.offset)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

// --- stats ---------------------------------------------------------------

#[derive(Deserialize)]
struct StatsQuery {
    account_id: Uuid,
}

async fn stats_summary(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<StatsQuery>,
) -> Result<Json<stats::Summary>, ApiError> {
    ensure_account_owner(&s, user.id, q.account_id).await?;
    let trades = repo::list_trades(&s.pool, q.account_id, 100_000, 0)
        .await
        .map_err(ApiError::Internal)?;
    Ok(Json(stats::summary(&trades)))
}

async fn stats_equity(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<StatsQuery>,
) -> Result<Json<Vec<stats::EquityPoint>>, ApiError> {
    ensure_account_owner(&s, user.id, q.account_id).await?;
    let trades = repo::list_trades(&s.pool, q.account_id, 100_000, 0)
        .await
        .map_err(ApiError::Internal)?;
    Ok(Json(stats::equity_curve(&trades)))
}

// --- journal -------------------------------------------------------------

async fn journal_for_day(
    State(s): State<AppState>,
    user: AuthUser,
    Path(day): Path<NaiveDate>,
) -> Result<Json<Vec<JournalEntry>>, ApiError> {
    Ok(Json(
        repo::list_journal_for_day(&s.pool, user.id, day)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

// --- helpers -------------------------------------------------------------

async fn ensure_account_owner(
    s: &AppState,
    user_id: Uuid,
    account_id: Uuid,
) -> Result<(), ApiError> {
    let row: Option<(Uuid,)> =
        sqlx::query_as("SELECT user_id FROM accounts WHERE id = $1")
            .bind(account_id)
            .fetch_optional(&s.pool)
            .await?;
    match row {
        Some((owner,)) if owner == user_id => Ok(()),
        Some(_) => Err(ApiError::Forbidden),
        None => Err(ApiError::NotFound),
    }
}
