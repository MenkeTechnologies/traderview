use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::routes::helpers::ensure_account_owner;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use traderview_db::r_distribution::RDistReport;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new().route("/r-distribution/:account_id", get(report))
}

async fn report(
    State(s): State<AppState>,
    u: AuthUser,
    Path(account_id): Path<Uuid>,
) -> Result<Json<RDistReport>, ApiError> {
    ensure_account_owner(&s, u.id, account_id).await?;
    Ok(Json(traderview_db::r_distribution::report(&s.pool, u.id, account_id)
        .await.map_err(ApiError::Internal)?))
}
