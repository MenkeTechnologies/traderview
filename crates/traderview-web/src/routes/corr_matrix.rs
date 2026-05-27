use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use traderview_db::corr_matrix::CorrMatrix;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/correlation/watchlist/:wid", get(for_watchlist))
        .route("/correlation/symbols", get(for_symbols))
}

#[derive(Debug, Deserialize)]
struct WlQ {
    days: Option<i64>,
}

async fn for_watchlist(
    State(s): State<AppState>,
    u: AuthUser,
    Path(wid): Path<Uuid>,
    Query(p): Query<WlQ>,
) -> Result<Json<CorrMatrix>, ApiError> {
    let days = p.days.unwrap_or(90).clamp(30, 730);
    Ok(Json(
        traderview_db::corr_matrix::for_watchlist(&s.pool, u.id, wid, days)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

#[derive(Debug, Deserialize)]
struct SymsQ {
    symbols: String,
    days: Option<i64>,
}

async fn for_symbols(
    State(s): State<AppState>,
    _u: AuthUser,
    Query(p): Query<SymsQ>,
) -> Result<Json<CorrMatrix>, ApiError> {
    let syms: Vec<String> = p
        .symbols
        .split(',')
        .map(|x| x.trim().to_uppercase())
        .filter(|x| !x.is_empty())
        .take(50)
        .collect();
    if syms.len() < 2 {
        return Err(ApiError::BadRequest(
            "need at least 2 symbols (comma-separated)".into(),
        ));
    }
    let days = p.days.unwrap_or(90).clamp(30, 730);
    Ok(Json(
        traderview_db::corr_matrix::for_symbols(&s.pool, &syms, days)
            .await
            .map_err(ApiError::Internal)?,
    ))
}
