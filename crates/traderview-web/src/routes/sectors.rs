use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use traderview_db::sectors::Sector;

pub fn router() -> Router<AppState> {
    Router::new().route("/sectors", get(list))
}

async fn list(State(s): State<AppState>, _user: AuthUser) -> Result<Json<Vec<Sector>>, ApiError> {
    Ok(Json(traderview_db::sectors::ranked(&s.pool).await.map_err(ApiError::Internal)?))
}
