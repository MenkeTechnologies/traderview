use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::routes::helpers::ensure_account_owner;
use crate::state::AppState;
use axum::extract::{Multipart, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use traderview_core::Import;
use traderview_import::{parser_for, sha256_hex};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/imports", get(list).post(upload))
        .route("/imports/sources", get(sources))
}

#[derive(Deserialize)]
struct ListQuery {
    account_id: Uuid,
}

async fn list(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<ListQuery>,
) -> Result<Json<Vec<Import>>, ApiError> {
    ensure_account_owner(&s, user.id, q.account_id).await?;
    Ok(Json(
        traderview_db::imports::list(&s.pool, q.account_id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

#[derive(Serialize)]
struct UploadResponse {
    import_id: Uuid,
    inserted: usize,
    duplicates: usize,
    parsed: usize,
    trades_rolled: usize,
}

async fn upload(
    State(s): State<AppState>,
    user: AuthUser,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, ApiError> {
    let mut account_id: Option<Uuid> = None;
    let mut source = String::new();
    let mut filename = String::new();
    let mut bytes: Vec<u8> = Vec::new();
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::BadRequest(format!("multipart: {e}")))?
    {
        match field.name() {
            Some("account_id") => {
                let v = field
                    .text()
                    .await
                    .map_err(|e| ApiError::BadRequest(format!("account_id: {e}")))?;
                account_id = Some(
                    Uuid::parse_str(&v)
                        .map_err(|e| ApiError::BadRequest(format!("account_id uuid: {e}")))?,
                );
            }
            Some("source") => {
                source = field
                    .text()
                    .await
                    .map_err(|e| ApiError::BadRequest(format!("source: {e}")))?;
            }
            Some("file") => {
                filename = field.file_name().unwrap_or("upload.csv").into();
                bytes = field
                    .bytes()
                    .await
                    .map_err(|e| ApiError::BadRequest(format!("file: {e}")))?
                    .to_vec();
            }
            _ => {}
        }
    }
    let account_id = account_id.ok_or_else(|| ApiError::BadRequest("account_id required".into()))?;
    if source.is_empty() {
        return Err(ApiError::BadRequest("source required".into()));
    }
    if bytes.is_empty() {
        return Err(ApiError::BadRequest("file required".into()));
    }
    ensure_account_owner(&s, user.id, account_id).await?;

    let parser = parser_for(&source)
        .ok_or_else(|| ApiError::BadRequest(format!("unknown broker: {source}")))?;
    let sha = sha256_hex(&bytes);
    let parsed = parser
        .parse(&bytes)
        .map_err(|e| ApiError::BadRequest(format!("parse: {e}")))?;

    let import_row = traderview_db::imports::create(
        &s.pool,
        account_id,
        &source,
        &filename,
        &sha,
        parsed.len() as i32,
    )
    .await
    .map_err(ApiError::Internal)?;

    let mut inserted = 0usize;
    let mut duplicates = 0usize;
    for p in &parsed {
        match traderview_db::executions::insert_parsed(&s.pool, account_id, import_row.id, p).await
        {
            Ok(true) => inserted += 1,
            Ok(false) => duplicates += 1,
            Err(e) => return Err(ApiError::Internal(e)),
        }
    }

    let trades_rolled = traderview_db::trades::rollup_account(&s.pool, account_id)
        .await
        .map_err(ApiError::Internal)?;

    Ok(Json(UploadResponse {
        import_id: import_row.id,
        inserted,
        duplicates,
        parsed: parsed.len(),
        trades_rolled,
    }))
}

#[derive(Serialize)]
struct SourcesResponse {
    sources: Vec<&'static str>,
}

async fn sources() -> Json<SourcesResponse> {
    Json(SourcesResponse {
        sources: traderview_import::supported_sources().to_vec(),
    })
}
