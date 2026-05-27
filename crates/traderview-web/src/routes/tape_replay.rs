use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use traderview_db::tape_replay::TapeReplay;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new().route("/tape-replay/:trade_id", get(build))
}

async fn build(
    State(s): State<AppState>,
    u: AuthUser,
    Path(trade_id): Path<Uuid>,
) -> Result<Json<TapeReplay>, ApiError> {
    Ok(Json(
        traderview_db::tape_replay::build(&s.pool, u.id, trade_id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}
