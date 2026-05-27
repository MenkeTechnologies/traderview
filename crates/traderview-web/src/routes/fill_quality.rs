use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::routes::helpers::ensure_account_owner;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use traderview_db::fill_quality::FillQualityReport;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new().route("/fill-quality/:account_id", get(report))
}

async fn report(
    State(s): State<AppState>,
    u: AuthUser,
    Path(account_id): Path<Uuid>,
) -> Result<Json<FillQualityReport>, ApiError> {
    ensure_account_owner(&s, u.id, account_id).await?;
    Ok(Json(
        traderview_db::fill_quality::report(&s.pool, u.id, account_id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}
