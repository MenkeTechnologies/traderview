use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use traderview_db::market_data::QuoteSnapshot;
use traderview_db::watchlists::Watchlist;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/watchlists", get(list).post(create))
        .route("/watchlists/:id", post(rename).delete(delete_one))
        .route("/watchlists/:id/symbols", get(symbols).post(add_symbol))
        .route("/watchlists/:id/symbols/:symbol", delete(remove_symbol))
        .route("/watchlists/:id/quotes", get(quotes))
}

async fn list(State(s): State<AppState>, user: AuthUser) -> Result<Json<Vec<Watchlist>>, ApiError> {
    Ok(Json(
        traderview_db::watchlists::list(&s.pool, user.id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

#[derive(Deserialize)]
struct CreateBody {
    name: String,
}

async fn create(
    State(s): State<AppState>,
    user: AuthUser,
    Json(body): Json<CreateBody>,
) -> Result<Json<Watchlist>, ApiError> {
    Ok(Json(
        traderview_db::watchlists::create(&s.pool, user.id, &body.name)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

#[derive(Deserialize)]
struct RenameBody {
    name: String,
}

async fn rename(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<RenameBody>,
) -> Result<Json<bool>, ApiError> {
    Ok(Json(
        traderview_db::watchlists::rename(&s.pool, user.id, id, &body.name)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

async fn delete_one(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<bool>, ApiError> {
    Ok(Json(
        traderview_db::watchlists::delete(&s.pool, user.id, id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

async fn ensure_owner(s: &AppState, user_id: Uuid, id: Uuid) -> Result<(), ApiError> {
    if traderview_db::watchlists::ensure_owner(&s.pool, user_id, id)
        .await
        .map_err(ApiError::Internal)?
    {
        Ok(())
    } else {
        Err(ApiError::Forbidden)
    }
}

async fn symbols(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<String>>, ApiError> {
    ensure_owner(&s, user.id, id).await?;
    Ok(Json(
        traderview_db::watchlists::symbols(&s.pool, id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

#[derive(Deserialize)]
struct AddSymBody {
    symbol: String,
}

async fn add_symbol(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<AddSymBody>,
) -> Result<Json<bool>, ApiError> {
    ensure_owner(&s, user.id, id).await?;
    traderview_db::watchlists::add_symbol(&s.pool, id, &body.symbol)
        .await
        .map_err(ApiError::Internal)?;
    push_watchlist_subs_to_live_ticks(&s.pool).await;
    Ok(Json(true))
}

async fn remove_symbol(
    State(s): State<AppState>,
    user: AuthUser,
    Path((id, symbol)): Path<(Uuid, String)>,
) -> Result<Json<bool>, ApiError> {
    ensure_owner(&s, user.id, id).await?;
    let removed = traderview_db::watchlists::remove_symbol(&s.pool, id, &symbol)
        .await
        .map_err(ApiError::Internal)?;
    push_watchlist_subs_to_live_ticks(&s.pool).await;
    Ok(Json(removed))
}

/// Push the union of every user's watchlist symbols into the live-tick
/// subscription set. Best-effort — failures get logged but don't fail
/// the user's HTTP request. Called after every watchlist mutation so
/// the WS workers stay in sync without the candidates/scanner loop.
async fn push_watchlist_subs_to_live_ticks(pool: &sqlx::PgPool) {
    let live = traderview_db::live_ticks::global();
    if !live.has_any_provider().await {
        // No WS provider configured yet — set_symbols would be a no-op,
        // skip the DB round-trip.
        return;
    }
    let symbols = match traderview_db::watchlists::all_distinct_symbols(pool).await {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!(error = %e, "all_distinct_symbols failed; live ticks may lag");
            return;
        }
    };
    if let Err(e) = live.set_symbols(symbols).await {
        tracing::warn!(error = %e, "set_symbols from watchlist mutation failed");
    }
}

#[derive(Serialize)]
struct QuotesResp {
    symbols: Vec<String>,
    quotes: Vec<QuoteSnapshot>,
}

async fn quotes(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<QuotesResp>, ApiError> {
    ensure_owner(&s, user.id, id).await?;
    let syms = traderview_db::watchlists::symbols(&s.pool, id)
        .await
        .map_err(ApiError::Internal)?;
    let qs = traderview_db::market_data::quotes(&s.pool, &syms).await;
    Ok(Json(QuotesResp {
        symbols: syms,
        quotes: qs,
    }))
}
