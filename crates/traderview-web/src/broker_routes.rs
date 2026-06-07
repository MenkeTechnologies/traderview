//! Broker CRUD for multi-broker trade dashboard filtering.
//!
//! Mounted under `/api/brokers`. Mirrors `business_routes.rs` for
//! consistency: list / create / patch / delete / set-default.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::routing::{get, patch};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_brokers).post(create_broker))
        .route("/:id", patch(patch_broker).delete(delete_broker))
        .route("/:id/set-default", patch(set_default))
}

#[derive(Serialize, sqlx::FromRow)]
struct Broker {
    id: Uuid,
    user_id: Uuid,
    slug: String,
    display_name: String,
    home_url: Option<String>,
    notes: Option<String>,
    is_default: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

async fn list_brokers(
    State(s): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<Broker>>, ApiError> {
    let rows: Vec<Broker> = sqlx::query_as(
        "SELECT id, user_id, slug, display_name, home_url, notes, is_default,
                created_at, updated_at
           FROM brokers
          WHERE user_id = $1
          ORDER BY is_default DESC, display_name ASC",
    )
    .bind(user.id)
    .fetch_all(&s.pool)
    .await?;
    Ok(Json(rows))
}

#[derive(Deserialize)]
struct CreateBroker {
    display_name: String,
    #[serde(default)]
    slug: Option<String>,
    #[serde(default)]
    home_url: Option<String>,
    #[serde(default)]
    notes: Option<String>,
}

fn slug_from(name: &str) -> String {
    let lower = name.trim().to_lowercase();
    let mut out = String::with_capacity(lower.len());
    let mut prev_underscore = false;
    for ch in lower.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
            prev_underscore = false;
        } else if !prev_underscore && !out.is_empty() {
            out.push('_');
            prev_underscore = true;
        }
    }
    while out.ends_with('_') {
        out.pop();
    }
    out
}

async fn create_broker(
    State(s): State<AppState>,
    user: AuthUser,
    Json(body): Json<CreateBroker>,
) -> Result<Json<Broker>, ApiError> {
    let display_name = body.display_name.trim();
    if display_name.is_empty() {
        return Err(ApiError::BadRequest("display_name required".into()));
    }
    let slug = body
        .slug
        .as_deref()
        .map(slug_from)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| slug_from(display_name));
    if slug.is_empty() {
        return Err(ApiError::BadRequest(
            "display_name must contain at least one alphanumeric char".into(),
        ));
    }
    let existing_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM brokers WHERE user_id = $1")
        .bind(user.id)
        .fetch_one(&s.pool)
        .await?;
    let is_default = existing_count == 0;
    let row: Broker = sqlx::query_as(
        "INSERT INTO brokers (user_id, slug, display_name, home_url, notes, is_default)
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (user_id, slug) DO UPDATE SET
            display_name = EXCLUDED.display_name,
            home_url     = EXCLUDED.home_url,
            notes        = EXCLUDED.notes,
            updated_at   = NOW()
         RETURNING id, user_id, slug, display_name, home_url, notes, is_default,
                   created_at, updated_at",
    )
    .bind(user.id)
    .bind(&slug)
    .bind(display_name)
    .bind(body.home_url.as_deref())
    .bind(body.notes.as_deref())
    .bind(is_default)
    .fetch_one(&s.pool)
    .await?;
    Ok(Json(row))
}

#[derive(Deserialize)]
struct PatchBroker {
    #[serde(default)]
    display_name: Option<String>,
    #[serde(default)]
    home_url: Option<Option<String>>,
    #[serde(default)]
    notes: Option<Option<String>>,
}

async fn patch_broker(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<PatchBroker>,
) -> Result<Json<Broker>, ApiError> {
    let exists: Option<Uuid> =
        sqlx::query_scalar("SELECT id FROM brokers WHERE id = $1 AND user_id = $2")
            .bind(id)
            .bind(user.id)
            .fetch_optional(&s.pool)
            .await?;
    if exists.is_none() {
        return Err(ApiError::NotFound);
    }
    let row: Broker = sqlx::query_as(
        "UPDATE brokers SET
            display_name = COALESCE($3, display_name),
            home_url     = COALESCE($4, home_url),
            notes        = COALESCE($5, notes),
            updated_at   = NOW()
         WHERE id = $1 AND user_id = $2
         RETURNING id, user_id, slug, display_name, home_url, notes, is_default,
                   created_at, updated_at",
    )
    .bind(id)
    .bind(user.id)
    .bind(body.display_name)
    .bind(body.home_url.flatten())
    .bind(body.notes.flatten())
    .fetch_one(&s.pool)
    .await?;
    Ok(Json(row))
}

async fn delete_broker(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let res = sqlx::query("DELETE FROM brokers WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user.id)
        .execute(&s.pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(ApiError::NotFound);
    }
    Ok(Json(serde_json::json!({"ok": true})))
}

async fn set_default(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Broker>, ApiError> {
    let mut tx = s.pool.begin().await?;
    sqlx::query("UPDATE brokers SET is_default = FALSE WHERE user_id = $1")
        .bind(user.id)
        .execute(&mut *tx)
        .await?;
    let row: Option<Broker> = sqlx::query_as(
        "UPDATE brokers SET is_default = TRUE, updated_at = NOW()
          WHERE id = $1 AND user_id = $2
         RETURNING id, user_id, slug, display_name, home_url, notes, is_default,
                   created_at, updated_at",
    )
    .bind(id)
    .bind(user.id)
    .fetch_optional(&mut *tx)
    .await?;
    let row = row.ok_or(ApiError::NotFound)?;
    tx.commit().await?;
    Ok(Json(row))
}
