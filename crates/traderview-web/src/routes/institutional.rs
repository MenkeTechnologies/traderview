//! 13F institutional-holdings routes — read-only.
//!
//! All endpoints are PUBLIC behind the existing auth middleware (no
//! ownership filter — 13F filings are public SEC disclosures). The
//! poller that populates the tables runs out of band; these routes
//! just expose the read paths.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use traderview_db::institutional::{
    holdings_for_filing, holdings_for_manager_latest, list_managers, manager_by_cik,
    manager_filings, position_changes_for_manager, top_managers_by_aum, top_owners_of_symbol,
    Filing, Holding, Manager, ManagerAum, PositionChange, SymbolOwner,
};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/institutional/managers", get(list_managers_route))
        .route(
            "/institutional/managers/by-cik/:cik",
            get(manager_by_cik_route),
        )
        .route(
            "/institutional/managers/:id/filings",
            get(manager_filings_route),
        )
        .route(
            "/institutional/managers/:id/holdings",
            get(holdings_latest_route),
        )
        .route(
            "/institutional/managers/:id/changes",
            get(position_changes_route),
        )
        .route(
            "/institutional/filings/:id/holdings",
            get(holdings_for_filing_route),
        )
        .route(
            "/institutional/symbols/:symbol/owners",
            get(top_owners_route),
        )
        .route("/institutional/top-managers", get(top_managers_route))
}

#[derive(Deserialize)]
struct ManagerListQuery {
    search: Option<String>,
    notable: Option<bool>,
    limit: Option<i64>,
}

async fn list_managers_route(
    State(s): State<AppState>,
    _u: AuthUser,
    Query(q): Query<ManagerListQuery>,
) -> Result<Json<Vec<Manager>>, ApiError> {
    let limit = q.limit.unwrap_or(100).clamp(1, 1000);
    Ok(Json(
        list_managers(
            &s.pool,
            q.search.as_deref(),
            q.notable.unwrap_or(false),
            limit,
        )
        .await
        .map_err(ApiError::Internal)?,
    ))
}

async fn manager_by_cik_route(
    State(s): State<AppState>,
    _u: AuthUser,
    Path(cik): Path<String>,
) -> Result<Json<Manager>, ApiError> {
    manager_by_cik(&s.pool, &cik)
        .await
        .map_err(ApiError::Internal)?
        .map(Json)
        .ok_or(ApiError::NotFound)
}

#[derive(Deserialize)]
struct LimitQuery {
    limit: Option<i64>,
}

async fn manager_filings_route(
    State(s): State<AppState>,
    _u: AuthUser,
    Path(id): Path<Uuid>,
    Query(q): Query<LimitQuery>,
) -> Result<Json<Vec<Filing>>, ApiError> {
    let limit = q.limit.unwrap_or(20).clamp(1, 200);
    Ok(Json(
        manager_filings(&s.pool, id, limit)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

async fn holdings_latest_route(
    State(s): State<AppState>,
    _u: AuthUser,
    Path(id): Path<Uuid>,
    Query(q): Query<LimitQuery>,
) -> Result<Json<Vec<Holding>>, ApiError> {
    let limit = q.limit.unwrap_or(500).clamp(1, 5000);
    Ok(Json(
        holdings_for_manager_latest(&s.pool, id, limit)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

#[derive(Deserialize)]
struct ChangesQuery {
    change_type: Option<String>, // new | increased | decreased | held
    limit: Option<i64>,
}

async fn position_changes_route(
    State(s): State<AppState>,
    _u: AuthUser,
    Path(id): Path<Uuid>,
    Query(q): Query<ChangesQuery>,
) -> Result<Json<Vec<PositionChange>>, ApiError> {
    if let Some(ct) = q.change_type.as_deref() {
        if !ct.is_empty() && !matches!(ct, "new" | "increased" | "decreased" | "held") {
            return Err(ApiError::BadRequest(format!("invalid change_type: {ct}")));
        }
    }
    let limit = q.limit.unwrap_or(100).clamp(1, 1000);
    Ok(Json(
        position_changes_for_manager(&s.pool, id, q.change_type.as_deref(), limit)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

async fn holdings_for_filing_route(
    State(s): State<AppState>,
    _u: AuthUser,
    Path(id): Path<Uuid>,
    Query(q): Query<LimitQuery>,
) -> Result<Json<Vec<Holding>>, ApiError> {
    let limit = q.limit.unwrap_or(500).clamp(1, 5000);
    Ok(Json(
        holdings_for_filing(&s.pool, id, limit)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

async fn top_owners_route(
    State(s): State<AppState>,
    _u: AuthUser,
    Path(symbol): Path<String>,
    Query(q): Query<LimitQuery>,
) -> Result<Json<Vec<SymbolOwner>>, ApiError> {
    if symbol.trim().is_empty() {
        return Err(ApiError::BadRequest("symbol required".into()));
    }
    let limit = q.limit.unwrap_or(50).clamp(1, 500);
    Ok(Json(
        top_owners_of_symbol(&s.pool, &symbol, limit)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

async fn top_managers_route(
    State(s): State<AppState>,
    _u: AuthUser,
    Query(q): Query<LimitQuery>,
) -> Result<Json<Vec<ManagerAum>>, ApiError> {
    let limit = q.limit.unwrap_or(50).clamp(1, 500);
    Ok(Json(
        top_managers_by_aum(&s.pool, limit)
            .await
            .map_err(ApiError::Internal)?,
    ))
}
