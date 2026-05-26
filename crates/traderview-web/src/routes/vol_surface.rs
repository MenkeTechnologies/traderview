use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use traderview_db::vol_surface::VolSurface;

pub fn router() -> Router<AppState> {
    Router::new().route("/vol-surface/:symbol", get(surface))
}

#[derive(Debug, Deserialize)]
struct Params {
    n: Option<usize>,
}

async fn surface(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(symbol): Path<String>,
    Query(p): Query<Params>,
) -> Result<Json<VolSurface>, ApiError> {
    let n = p.n.unwrap_or(8).clamp(1, 16);
    let sym = symbol.to_uppercase();
    Ok(Json(
        traderview_db::vol_surface::surface(&sym, n)
            .await
            .map_err(ApiError::Internal)?,
    ))
}
