use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use traderview_db::markets::MarketsSnapshot;

pub fn router() -> Router<AppState> {
    Router::new().route("/markets/snapshot", get(snapshot))
}

/// 60-second in-process cache. The dashboard polls this on every load.
static CACHE: once_cell::sync::Lazy<Arc<Mutex<Option<(Instant, MarketsSnapshot)>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(None)));

async fn snapshot(State(_s): State<AppState>) -> Result<Json<MarketsSnapshot>, ApiError> {
    {
        let guard = CACHE.lock().await;
        if let Some((ts, snap)) = guard.as_ref() {
            if ts.elapsed() < Duration::from_secs(60) {
                return Ok(Json(snap.clone()));
            }
        }
    }
    let fresh = traderview_db::markets::snapshot()
        .await
        .map_err(ApiError::Internal)?;
    let mut guard = CACHE.lock().await;
    *guard = Some((Instant::now(), fresh.clone()));
    Ok(Json(fresh))
}
