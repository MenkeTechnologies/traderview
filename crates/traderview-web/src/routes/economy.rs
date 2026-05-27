use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use traderview_db::economy::{upcoming, EconEvent, Importance};

pub fn router() -> Router<AppState> {
    Router::new().route("/economy/calendar", get(calendar))
}

#[derive(Deserialize)]
struct Q {
    #[serde(default = "default_days")]
    days: i64,
    #[serde(default)]
    importance: Option<String>,
}
fn default_days() -> i64 {
    60
}

async fn calendar(
    _s: State<AppState>,
    _u: AuthUser,
    Query(q): Query<Q>,
) -> Result<Json<Vec<EconEvent>>, ApiError> {
    let min = match q.importance.as_deref() {
        Some("high") => Importance::High,
        Some("medium") => Importance::Medium,
        _ => Importance::Low,
    };
    Ok(Json(upcoming(q.days, min)))
}
