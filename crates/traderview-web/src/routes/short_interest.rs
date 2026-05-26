use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use std::collections::BTreeSet;
use traderview_db::short_interest::{finra_daily, ranked_universe, yahoo_short_stats, FinraDay, ShortStats};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/short/symbol/:sym",    get(symbol))
        .route("/short/finra/:sym",     get(finra))
        .route("/short/ranked",         get(ranked))
}

async fn symbol(_s: State<AppState>, _u: AuthUser, Path(sym): Path<String>)
    -> Result<Json<ShortStats>, ApiError>
{
    Ok(Json(yahoo_short_stats(&sym).await.map_err(ApiError::Internal)?))
}

#[derive(Deserialize)]
struct FinraQ { #[serde(default = "default_days")] days: i64 }
fn default_days() -> i64 { 30 }

async fn finra(_s: State<AppState>, _u: AuthUser, Path(sym): Path<String>, Query(q): Query<FinraQ>)
    -> Result<Json<Vec<FinraDay>>, ApiError>
{
    Ok(Json(finra_daily(&sym, q.days).await.map_err(ApiError::Internal)?))
}

#[derive(Deserialize)]
struct RankedQ { watchlist_id: Option<Uuid> }

async fn ranked(State(s): State<AppState>, user: AuthUser, Query(q): Query<RankedQ>)
    -> Result<Json<Vec<ShortStats>>, ApiError>
{
    let universe: Vec<String> = if let Some(wid) = q.watchlist_id {
        if !traderview_db::watchlists::ensure_owner(&s.pool, user.id, wid).await.map_err(ApiError::Internal)? {
            return Err(ApiError::Forbidden);
        }
        traderview_db::watchlists::symbols(&s.pool, wid).await.map_err(ApiError::Internal)?
    } else {
        let mut all = BTreeSet::new();
        for w in traderview_db::watchlists::list(&s.pool, user.id).await.map_err(ApiError::Internal)? {
            for x in traderview_db::watchlists::symbols(&s.pool, w.id).await.map_err(ApiError::Internal)? {
                all.insert(x);
            }
        }
        all.into_iter().collect()
    };
    Ok(Json(ranked_universe(&universe).await))
}
