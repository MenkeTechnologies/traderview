use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::routing::{delete, get};
use axum::{Json, Router};
use serde::Deserialize;
use traderview_core::NoteTemplate;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/note-templates", get(list).post(upsert))
        .route("/note-templates/:id", delete(delete_one))
        .route("/note-templates/default", get(default_for))
}

#[derive(Deserialize)]
struct ListQ {
    #[serde(default)]
    scope: Option<String>,
}

async fn list(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<ListQ>,
) -> Result<Json<Vec<NoteTemplate>>, ApiError> {
    Ok(Json(
        traderview_db::note_templates::list(&s.pool, user.id, q.scope.as_deref())
            .await
            .map_err(ApiError::Internal)?,
    ))
}

#[derive(Deserialize)]
struct UpsertBody {
    name: String,
    scope: String,
    body_md: String,
    #[serde(default)]
    is_default: bool,
}

async fn upsert(
    State(s): State<AppState>,
    user: AuthUser,
    Json(body): Json<UpsertBody>,
) -> Result<Json<NoteTemplate>, ApiError> {
    if body.scope != "trade" && body.scope != "journal" {
        return Err(ApiError::BadRequest(
            "scope must be 'trade' or 'journal'".into(),
        ));
    }
    Ok(Json(
        traderview_db::note_templates::upsert(
            &s.pool,
            user.id,
            &body.name,
            &body.scope,
            &body.body_md,
            body.is_default,
        )
        .await
        .map_err(ApiError::Internal)?,
    ))
}

async fn delete_one(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<bool>, ApiError> {
    Ok(Json(
        traderview_db::note_templates::delete(&s.pool, user.id, id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

#[derive(Deserialize)]
struct DefaultQ {
    scope: String,
}

async fn default_for(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<DefaultQ>,
) -> Result<Json<Option<NoteTemplate>>, ApiError> {
    Ok(Json(
        traderview_db::note_templates::default_for(&s.pool, user.id, &q.scope)
            .await
            .map_err(ApiError::Internal)?,
    ))
}
