//! Business-entity CRUD for multi-business Schedule C tracking.
//!
//! Mounted under `/api/businesses`. Every endpoint scopes by `user_id`.
//! Other routes (receipts/dashboard/calendar/tax-rollup) accept a
//! `business_id` query param to filter — `None` means aggregated.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::routing::{get, patch};
use axum::{Json, Router};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_businesses).post(create_business))
        .route("/:id", patch(patch_business).delete(delete_business))
        .route("/:id/set-default", patch(set_default))
}

#[derive(Serialize, sqlx::FromRow)]
struct Business {
    id: Uuid,
    user_id: Uuid,
    name: String,
    ein: Option<String>,
    entity_type: String,
    naics_code: Option<String>,
    principal_addr: Option<String>,
    is_default: bool,
    started_at: Option<NaiveDate>,
    ended_at: Option<NaiveDate>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

async fn list_businesses(
    State(s): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<Business>>, ApiError> {
    let rows: Vec<Business> = sqlx::query_as(
        "SELECT id, user_id, name, ein, entity_type::text AS entity_type,
                naics_code, principal_addr, is_default,
                started_at, ended_at, created_at, updated_at
           FROM businesses
          WHERE user_id = $1
          ORDER BY is_default DESC, name ASC",
    )
    .bind(user.id)
    .fetch_all(&s.pool)
    .await?;
    Ok(Json(rows))
}

#[derive(Deserialize)]
struct CreateBusiness {
    name: String,
    #[serde(default)]
    ein: Option<String>,
    #[serde(default = "default_entity_type")]
    entity_type: String,
    #[serde(default)]
    naics_code: Option<String>,
    #[serde(default)]
    principal_addr: Option<String>,
    #[serde(default)]
    started_at: Option<NaiveDate>,
}
fn default_entity_type() -> String {
    "sole_prop".into()
}

async fn create_business(
    State(s): State<AppState>,
    user: AuthUser,
    Json(body): Json<CreateBusiness>,
) -> Result<Json<Business>, ApiError> {
    let name = body.name.trim();
    if name.is_empty() {
        return Err(ApiError::BadRequest("business name required".into()));
    }
    // If this is the user's first business, mark it as default.
    let existing_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM businesses WHERE user_id = $1")
            .bind(user.id)
            .fetch_one(&s.pool)
            .await?;
    let is_default = existing_count == 0;

    let row: Business = sqlx::query_as(
        "INSERT INTO businesses
            (user_id, name, ein, entity_type, naics_code, principal_addr,
             is_default, started_at)
         VALUES ($1, $2, $3, $4::business_entity_type_t, $5, $6, $7, $8)
         RETURNING id, user_id, name, ein, entity_type::text AS entity_type,
                   naics_code, principal_addr, is_default,
                   started_at, ended_at, created_at, updated_at",
    )
    .bind(user.id)
    .bind(name)
    .bind(body.ein.as_deref())
    .bind(&body.entity_type)
    .bind(body.naics_code.as_deref())
    .bind(body.principal_addr.as_deref())
    .bind(is_default)
    .bind(body.started_at)
    .fetch_one(&s.pool)
    .await?;
    Ok(Json(row))
}

#[derive(Deserialize)]
struct PatchBusiness {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    ein: Option<Option<String>>,
    #[serde(default)]
    entity_type: Option<String>,
    #[serde(default)]
    naics_code: Option<Option<String>>,
    #[serde(default)]
    principal_addr: Option<Option<String>>,
    #[serde(default)]
    started_at: Option<Option<NaiveDate>>,
    #[serde(default)]
    ended_at: Option<Option<NaiveDate>>,
}

async fn patch_business(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<PatchBusiness>,
) -> Result<Json<Business>, ApiError> {
    let exists: Option<Uuid> =
        sqlx::query_scalar("SELECT id FROM businesses WHERE id = $1 AND user_id = $2")
            .bind(id)
            .bind(user.id)
            .fetch_optional(&s.pool)
            .await?;
    if exists.is_none() {
        return Err(ApiError::NotFound);
    }
    let row: Business = sqlx::query_as(
        "UPDATE businesses SET
            name           = COALESCE($3, name),
            ein            = COALESCE($4, ein),
            entity_type    = COALESCE(NULLIF($5, '')::business_entity_type_t, entity_type),
            naics_code     = COALESCE($6, naics_code),
            principal_addr = COALESCE($7, principal_addr),
            started_at     = COALESCE($8, started_at),
            ended_at       = COALESCE($9, ended_at),
            updated_at     = NOW()
         WHERE id = $1 AND user_id = $2
         RETURNING id, user_id, name, ein, entity_type::text AS entity_type,
                   naics_code, principal_addr, is_default,
                   started_at, ended_at, created_at, updated_at",
    )
    .bind(id)
    .bind(user.id)
    .bind(body.name)
    .bind(body.ein.flatten())
    .bind(body.entity_type.unwrap_or_default())
    .bind(body.naics_code.flatten())
    .bind(body.principal_addr.flatten())
    .bind(body.started_at.flatten())
    .bind(body.ended_at.flatten())
    .fetch_one(&s.pool)
    .await?;
    Ok(Json(row))
}

async fn delete_business(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let res = sqlx::query("DELETE FROM businesses WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user.id)
        .execute(&s.pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(ApiError::NotFound);
    }
    Ok(Json(serde_json::json!({"ok": true})))
}

/// Atomically swap the user's default business to the given id.
async fn set_default(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Business>, ApiError> {
    let mut tx = s.pool.begin().await?;
    sqlx::query("UPDATE businesses SET is_default = FALSE WHERE user_id = $1")
        .bind(user.id)
        .execute(&mut *tx)
        .await?;
    let row: Option<Business> = sqlx::query_as(
        "UPDATE businesses SET is_default = TRUE, updated_at = NOW()
          WHERE id = $1 AND user_id = $2
         RETURNING id, user_id, name, ein, entity_type::text AS entity_type,
                   naics_code, principal_addr, is_default,
                   started_at, ended_at, created_at, updated_at",
    )
    .bind(id)
    .bind(user.id)
    .fetch_optional(&mut *tx)
    .await?;
    let row = row.ok_or_else(|| ApiError::NotFound)?;
    tx.commit().await?;
    Ok(Json(row))
}
