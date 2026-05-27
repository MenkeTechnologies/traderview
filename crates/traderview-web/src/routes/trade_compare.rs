use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use traderview_db::trade_compare::CompareReport;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new().route("/trade-compare", get(compare))
}

#[derive(Debug, Deserialize)]
struct Params {
    trade_ids: String,
}

async fn compare(
    State(s): State<AppState>,
    u: AuthUser,
    Query(p): Query<Params>,
) -> Result<Json<CompareReport>, ApiError> {
    let ids: Vec<Uuid> = p
        .trade_ids
        .split(',')
        .filter_map(|s| Uuid::parse_str(s.trim()).ok())
        .take(4)
        .collect();
    if ids.len() < 2 {
        return Err(ApiError::BadRequest(
            "need at least 2 valid trade_ids (max 4)".into(),
        ));
    }
    Ok(Json(
        traderview_db::trade_compare::compare(&s.pool, u.id, &ids)
            .await
            .map_err(ApiError::Internal)?,
    ))
}
