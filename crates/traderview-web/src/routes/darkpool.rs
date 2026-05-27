use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use std::collections::BTreeSet;
use traderview_db::darkpool::{DarkRanked, DarkSeries};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/darkpool/symbol/:sym", get(symbol))
        .route("/darkpool/ranked", get(ranked))
}

#[derive(Deserialize)]
struct Q {
    #[serde(default = "default_days")]
    days: i64,
}
fn default_days() -> i64 {
    30
}

async fn symbol(
    State(s): State<AppState>,
    _u: AuthUser,
    Path(sym): Path<String>,
    Query(q): Query<Q>,
) -> Result<Json<DarkSeries>, ApiError> {
    Ok(Json(
        traderview_db::darkpool::series(&s.pool, &sym, q.days)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

#[derive(Deserialize)]
struct RQ {
    watchlist_id: Option<Uuid>,
    #[serde(default = "default_days")]
    days: i64,
}

async fn ranked(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<RQ>,
) -> Result<Json<Vec<DarkRanked>>, ApiError> {
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
    Ok(Json(
        traderview_db::darkpool::ranked(&s.pool, &universe, q.days).await,
    ))
}
