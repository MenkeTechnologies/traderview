//! IPO lockup expiration tracker route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use traderview_db::ipo_lockups;

pub fn router() -> Router<AppState> {
    Router::new().route("/ipo-lockups/upcoming", get(upcoming))
}

async fn upcoming(
    State(_s): State<AppState>,
    _user: AuthUser,
) -> Result<Json<Vec<ipo_lockups::LockupExpiry>>, ApiError> {
    Ok(Json(ipo_lockups::upcoming().await?))
}
