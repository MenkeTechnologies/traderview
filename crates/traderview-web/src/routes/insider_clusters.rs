//! Insider cluster scoring route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use chrono::Utc;
use serde::Deserialize;
use traderview_db::{insider_clusters, insider_stream};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/insider-clusters/ranked", get(ranked))
        .route("/insider-clusters/symbol/:symbol", get(by_symbol))
}

#[derive(Deserialize)]
struct LimitQ {
    #[serde(default = "default_limit")]
    limit: usize,
}
fn default_limit() -> usize {
    50
}

async fn ranked(
    State(_s): State<AppState>,
    _user: AuthUser,
    Query(q): Query<LimitQ>,
) -> Result<Json<Vec<insider_clusters::InsiderCluster>>, ApiError> {
    let store = insider_stream::global();
    Ok(Json(insider_clusters::ranked(&store, q.limit, Utc::now())))
}

async fn by_symbol(
    State(_s): State<AppState>,
    _user: AuthUser,
    Path(symbol): Path<String>,
) -> Result<Json<Option<insider_clusters::InsiderCluster>>, ApiError> {
    let store = insider_stream::global();
    Ok(Json(insider_clusters::for_symbol(
        &store,
        &symbol,
        Utc::now(),
    )))
}
