//! 13F whale-watching:
//!   GET /13f/:cik/diff — holdings diff between the fund's last two
//!   13F-HR filings (new / exited / increased / decreased by shares).

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use traderview_db::thirteen_f::{self, ThirteenFDiff, ThirteenFError};

pub fn router() -> Router<AppState> {
    Router::new().route("/13f/:cik/diff", get(get_diff))
}

async fn get_diff(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(cik): Path<String>,
) -> Result<Json<ThirteenFDiff>, ApiError> {
    let cik = cik.trim().trim_start_matches('0').to_string();
    if cik.is_empty() || cik.len() > 10 || !cik.bytes().all(|b| b.is_ascii_digit()) {
        return Err(ApiError::BadRequest("CIK must be numeric".into()));
    }
    thirteen_f::holdings_diff(&cik).await.map(Json).map_err(|e| match e {
        ThirteenFError::Fetch(msg) => ApiError::Internal(anyhow::anyhow!(msg)),
        other => ApiError::BadRequest(other.to_string()),
    })
}
