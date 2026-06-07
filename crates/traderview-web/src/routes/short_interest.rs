use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use std::collections::BTreeSet;
use traderview_db::short_interest::{
    finnhub_short_stats, finra_daily, ranked_universe, FinraDay, ShortStats,
};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/short/symbol/:sym", get(symbol))
        .route("/short/finra/:sym", get(finra))
        .route("/short/ranked", get(ranked))
}

async fn symbol(
    _s: State<AppState>,
    _u: AuthUser,
    Path(sym): Path<String>,
) -> Result<Json<ShortStats>, ApiError> {
    // Finnhub backend — Yahoo's `quoteSummary` was crumb-locked and
    // 401'ing since late 2023, so we route everything through
    // Finnhub's `/stock/short-interest` + `/stock/metric` pair. The
    // fallback envelope (all-None) covers tier restrictions and
    // network issues without 500'ing.
    let stats = finnhub_short_stats(&sym).await.unwrap_or_else(|e| {
        tracing::warn!(symbol = %sym, error = %e, "finnhub short_stats failed; returning empty");
        ShortStats {
            symbol: sym.to_uppercase(),
            shares_short: None,
            shares_short_prior: None,
            short_ratio: None,
            short_pct_float: None,
            short_pct_outstanding: None,
            float: None,
            change_pct: None,
            fetched_at: chrono::Utc::now(),
        }
    });
    Ok(Json(stats))
}

#[derive(Deserialize)]
struct FinraQ {
    #[serde(default = "default_days")]
    days: i64,
}
fn default_days() -> i64 {
    30
}

async fn finra(
    _s: State<AppState>,
    _u: AuthUser,
    Path(sym): Path<String>,
    Query(q): Query<FinraQ>,
) -> Result<Json<Vec<FinraDay>>, ApiError> {
    Ok(Json(
        finra_daily(&sym, q.days)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

#[derive(Deserialize)]
struct RankedQ {
    watchlist_id: Option<Uuid>,
}

async fn ranked(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<RankedQ>,
) -> Result<Json<Vec<ShortStats>>, ApiError> {
    let universe: Vec<String> = if let Some(wid) = q.watchlist_id {
        if !traderview_db::watchlists::ensure_owner(&s.pool, user.id, wid)
            .await
            .map_err(ApiError::Internal)?
        {
            return Err(ApiError::Forbidden);
        }
        traderview_db::watchlists::symbols(&s.pool, wid)
            .await
            .map_err(ApiError::Internal)?
    } else {
        let mut all = BTreeSet::new();
        for w in traderview_db::watchlists::list(&s.pool, user.id)
            .await
            .map_err(ApiError::Internal)?
        {
            for x in traderview_db::watchlists::symbols(&s.pool, w.id)
                .await
                .map_err(ApiError::Internal)?
            {
                all.insert(x);
            }
        }
        all.into_iter().collect()
    };
    Ok(Json(ranked_universe(&universe).await))
}
