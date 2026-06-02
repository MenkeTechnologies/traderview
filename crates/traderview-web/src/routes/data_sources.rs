//! Per-user market-data provider credentials — Finnhub, Alpaca.
//!
//! `GET /data-sources`  returns the current state with secrets masked as `"***"`.
//! `POST /data-sources` upserts. Secret fields containing `"***"` or empty
//! string are interpreted as "leave the column alone" so the UI can submit
//! the form without re-typing the key.
//!
//! When the Finnhub key changes, the live-ticks WebSocket loop's in-memory
//! key slot is updated too so the live scanner picks up the new credential
//! without a process restart.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use traderview_db::data_source_keys::{self, DataSourceKeysDto};

pub fn router() -> Router<AppState> {
    Router::new().route("/data-sources", get(get_keys).post(set_keys))
}

async fn get_keys(
    State(s): State<AppState>,
    u: AuthUser,
) -> Result<Json<DataSourceKeysDto>, ApiError> {
    Ok(Json(
        data_source_keys::get(&s.pool, u.id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

async fn set_keys(
    State(s): State<AppState>,
    u: AuthUser,
    Json(body): Json<DataSourceKeysDto>,
) -> Result<Json<serde_json::Value>, ApiError> {
    data_source_keys::set(&s.pool, u.id, &body)
        .await
        .map_err(ApiError::Internal)?;
    // Best-effort refresh of the in-memory live-ticks key so the WS loop
    // picks up the new value without a process restart. Falls back to the
    // DB+env resolver — if neither yields a key, leave the slot untouched.
    if let Ok(Some(k)) = data_source_keys::finnhub_key_plain(&s.pool, u.id).await {
        traderview_db::live_ticks::global().set_api_key(k).await;
    }
    Ok(Json(serde_json::json!({ "ok": true })))
}
