use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::routes::helpers::ensure_account_owner;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use chrono::{Datelike, Utc};
use serde::Deserialize;
use traderview_db::tax_lots::{compute, LotMethod, TaxReport};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new().route("/tax-lots/:account_id", get(report))
}

#[derive(Debug, Deserialize)]
struct Params {
    year: Option<i32>,
    method: Option<LotMethod>,
}

async fn report(
    State(s): State<AppState>,
    u: AuthUser,
    Path(account_id): Path<Uuid>,
    Query(p): Query<Params>,
) -> Result<Json<TaxReport>, ApiError> {
    ensure_account_owner(&s, u.id, account_id).await?;
    let year = p.year.unwrap_or_else(|| Utc::now().year());
    let method = p.method.unwrap_or(LotMethod::Fifo);
    Ok(Json(
        compute(&s.pool, account_id, year, method)
            .await
            .map_err(ApiError::Internal)?,
    ))
}
