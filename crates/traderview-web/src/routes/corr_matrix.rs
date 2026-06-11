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
        .route("/correlation/regime", get(regime))
}

#[derive(Debug, Deserialize)]
struct RegimeQ {
    a: String,
    b: String,
    /// Rolling window in trading days (clamped 10..=252).
    window: Option<usize>,
    years: Option<u32>,
}

/// Rolling-correlation regime detector between two symbols — flags the
/// bars where a hedge pair flips coupled/neutral/inverse.
async fn regime(
    State(s): State<AppState>,
    _u: AuthUser,
    Query(q): Query<RegimeQ>,
) -> Result<Json<traderview_core::correlation_regime::CorrelationRegimeReport>, ApiError> {
    let a = q.a.trim().to_uppercase();
    let b = q.b.trim().to_uppercase();
    if a.is_empty() || b.is_empty() || a.len() > 20 || b.len() > 20 {
        return Err(ApiError::BadRequest("invalid symbols".into()));
    }
    let window = q.window.unwrap_or(63).clamp(10, 252);
    traderview_db::strategy_calculators::correlation_regime(
        &s.pool,
        &a,
        &b,
        window,
        q.years.unwrap_or(5),
    )
    .await
    .map(Json)
    .map_err(|e| match e {
        traderview_db::strategy_calculators::TomError::PriceFetch(inner) => {
            ApiError::Internal(inner)
        }
        other => ApiError::BadRequest(other.to_string()),
    })
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
