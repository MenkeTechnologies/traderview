use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use rust_decimal::Decimal;
use serde::Deserialize;
use traderview_db::disclosures::{Disclosure, PollResult, Watcher};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/disclosures", get(list))
        .route("/disclosures/poll", post(poll_now))
        .route(
            "/disclosures/watchers",
            get(list_watchers).post(create_watcher),
        )
        .route("/disclosures/watchers/:id", delete(delete_watcher))
}

#[derive(Deserialize)]
struct ListQ {
    kind: Option<String>,
    symbol: Option<String>,
    #[serde(default = "default_limit")]
    limit: i64,
}
fn default_limit() -> i64 {
    200
}

async fn list(
    State(s): State<AppState>,
    _user: AuthUser,
    Query(q): Query<ListQ>,
) -> Result<Json<Vec<Disclosure>>, ApiError> {
    Ok(Json(
        traderview_db::disclosures::list(&s.pool, q.kind.as_deref(), q.symbol.as_deref(), q.limit)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

async fn poll_now(
    State(s): State<AppState>,
    _user: AuthUser,
) -> Result<Json<PollResult>, ApiError> {
    Ok(Json(traderview_db::disclosures::poll_all(&s.pool).await))
}

async fn list_watchers(
    State(s): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<Watcher>>, ApiError> {
    Ok(Json(
        traderview_db::disclosures::list_watchers(&s.pool, user.id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

#[derive(Deserialize)]
struct CreateBody {
    name: String,
    #[serde(default = "default_kinds")]
    kinds: Vec<String>,
    symbols: Option<Vec<String>>,
    filers: Option<Vec<String>>,
    min_amount_usd: Option<Decimal>,
    #[serde(default = "default_sound")]
    sound: String,
}
fn default_kinds() -> Vec<String> {
    vec![
        "insider_form4".into(),
        "senate_stock".into(),
        "house_stock".into(),
    ]
}
fn default_sound() -> String {
    "bell".into()
}

async fn create_watcher(
    State(s): State<AppState>,
    user: AuthUser,
    Json(b): Json<CreateBody>,
) -> Result<Json<Watcher>, ApiError> {
    Ok(Json(
        traderview_db::disclosures::create_watcher(
            &s.pool,
            traderview_db::disclosures::NewWatcher {
                user_id: user.id,
                name: &b.name,
                kinds: &b.kinds,
                symbols: b.symbols.as_deref(),
                filers: b.filers.as_deref(),
                min_amount_usd: b.min_amount_usd,
                sound: &b.sound,
            },
        )
        .await
        .map_err(ApiError::Internal)?,
    ))
}

async fn delete_watcher(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<bool>, ApiError> {
    Ok(Json(
        traderview_db::disclosures::delete_watcher(&s.pool, user.id, id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}
