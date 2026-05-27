use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::routes::helpers::ensure_account_owner;
use crate::state::AppState;
use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::routing::post;
use axum::{Json, Router};
use traderview_db::csv_wizard::{commit, parse_csv, ColumnMapping, CommitResult, ParsePreview};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/imports/csv-wizard/parse", post(parse))
        .route(
            "/imports/csv-wizard/commit/:account_id",
            post(commit_handler),
        )
}

async fn parse(
    State(_s): State<AppState>,
    _u: AuthUser,
    body: Bytes,
) -> Result<Json<ParsePreview>, ApiError> {
    parse_csv(&body)
        .map(Json)
        .map_err(|e| ApiError::BadRequest(e.to_string()))
}

async fn commit_handler(
    State(s): State<AppState>,
    u: AuthUser,
    Path(account_id): Path<Uuid>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<CommitResult>, ApiError> {
    ensure_account_owner(&s, u.id, account_id).await?;
    // Mapping arrives in a header so the body can stay raw CSV bytes —
    // simpler than multipart and works fine for <2KB JSON.
    let map_hdr = headers
        .get("x-csv-mapping")
        .ok_or_else(|| ApiError::BadRequest("missing X-CSV-Mapping header".into()))?
        .to_str()
        .map_err(|_| ApiError::BadRequest("X-CSV-Mapping must be utf-8".into()))?;
    let mapping: ColumnMapping = serde_json::from_str(map_hdr)
        .map_err(|e| ApiError::BadRequest(format!("X-CSV-Mapping invalid JSON: {e}")))?;
    Ok(Json(
        commit(&s.pool, account_id, &body, &mapping)
            .await
            .map_err(|e| ApiError::BadRequest(e.to_string()))?,
    ))
}
