use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::routes::helpers::ensure_account_owner;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use traderview_db::live_positions::LiveSnapshot;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new().route("/live-positions/:account_id", get(snapshot))
}

async fn snapshot(
    State(s): State<AppState>,
    u: AuthUser,
    Path(account_id): Path<Uuid>,
) -> Result<Json<LiveSnapshot>, ApiError> {
    ensure_account_owner(&s, u.id, account_id).await?;
    Ok(Json(traderview_db::live_positions::snapshot(&s.pool, account_id)
        .await.map_err(ApiError::Internal)?))
}
