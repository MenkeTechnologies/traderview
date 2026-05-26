use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use serde::Deserialize;
use traderview_core::Mentorship;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/mentorships", post(request))
        .route("/mentorships/mentors", get(mentors_of))
        .route("/mentorships/mentees", get(mentees_of))
        .route("/mentorships/:id/accept", post(accept))
        .route("/mentorships/:id", delete(revoke))
}

#[derive(Deserialize)]
struct RequestBody {
    mentor_id: Uuid,
    #[serde(default = "default_scope")]
    scope: String,
}
fn default_scope() -> String { "read".into() }

async fn request(
    State(s): State<AppState>,
    user: AuthUser,
    Json(body): Json<RequestBody>,
) -> Result<Json<Mentorship>, ApiError> {
    if body.mentor_id == user.id {
        return Err(ApiError::BadRequest("cannot mentor yourself".into()));
    }
    Ok(Json(
        traderview_db::mentorships::request(&s.pool, body.mentor_id, user.id, &body.scope)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

async fn accept(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<bool>, ApiError> {
    Ok(Json(
        traderview_db::mentorships::accept(&s.pool, user.id, id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

async fn revoke(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<bool>, ApiError> {
    Ok(Json(
        traderview_db::mentorships::revoke(&s.pool, user.id, id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

async fn mentors_of(
    State(s): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<Mentorship>>, ApiError> {
    Ok(Json(
        traderview_db::mentorships::mentors_of(&s.pool, user.id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

async fn mentees_of(
    State(s): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<Mentorship>>, ApiError> {
    Ok(Json(
        traderview_db::mentorships::mentees_of(&s.pool, user.id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}
