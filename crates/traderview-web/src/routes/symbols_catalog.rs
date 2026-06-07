//! Symbol catalog endpoints — drive the global frontend autocomplete.
//!
//!  * `GET  /api/symbols/list?seed_if_empty=true`
//!     – returns every row in the `symbols` table. When the flag is
//!       set and the table is empty, runs a Finnhub seed first so the
//!       caller gets data on first ever request without an extra round
//!       trip. Failures during auto-seed are logged but never fail
//!       the response — an empty list is still useful (graceful).
//!  * `POST /api/symbols/seed?exchange=US`
//!     – idempotent re-seed. Use to refresh descriptions or pull a new
//!       exchange (e.g. `?exchange=TO` for TSX).

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use traderview_db::symbols::{count, list_all, seed_from_finnhub, Symbol};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/symbols/list", get(list))
        .route("/symbols/seed", post(seed))
}

#[derive(Deserialize)]
struct ListQ {
    #[serde(default)]
    seed_if_empty: bool,
}

async fn list(
    State(s): State<AppState>,
    _user: AuthUser,
    Query(q): Query<ListQ>,
) -> Result<Json<Vec<Symbol>>, ApiError> {
    if q.seed_if_empty {
        let n = count(&s.pool).await.map_err(ApiError::Internal)?;
        if n == 0 {
            if let Err(e) = seed_from_finnhub(&s.pool, "US").await {
                // Don't fail the read — empty list is still graceful.
                tracing::warn!(error = %e, "symbol catalog seed failed on first load");
            }
        }
    }
    Ok(Json(list_all(&s.pool).await.map_err(ApiError::Internal)?))
}

#[derive(Deserialize)]
struct SeedQ {
    #[serde(default = "default_exchange")]
    exchange: String,
}
fn default_exchange() -> String {
    "US".into()
}

#[derive(serde::Serialize)]
struct SeedResp {
    inserted: usize,
    exchange: String,
}

async fn seed(
    State(s): State<AppState>,
    _user: AuthUser,
    Query(q): Query<SeedQ>,
) -> Result<Json<SeedResp>, ApiError> {
    let n = seed_from_finnhub(&s.pool, &q.exchange)
        .await
        .map_err(ApiError::Internal)?;
    Ok(Json(SeedResp {
        inserted: n,
        exchange: q.exchange,
    }))
}
