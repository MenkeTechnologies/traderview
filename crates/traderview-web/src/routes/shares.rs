use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::routes::helpers::ensure_trade_owner;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::routing::{delete, get};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use traderview_core::{Trade, TradeShare};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/shares", get(list_mine).post(create))
        .route("/shares/public", get(list_public))
        .route("/shares/:id", delete(delete_one))
        .route("/shared/:slug", get(view_public))
}

#[derive(Deserialize)]
struct CreateBody {
    trade_id: Uuid,
    #[serde(default = "default_true")]
    is_public: bool,
    #[serde(default = "default_true")]
    show_notes: bool,
    #[serde(default = "default_true")]
    show_screenshots: bool,
}
fn default_true() -> bool {
    true
}

async fn create(
    State(s): State<AppState>,
    user: AuthUser,
    Json(body): Json<CreateBody>,
) -> Result<Json<TradeShare>, ApiError> {
    ensure_trade_owner(&s.pool, user.id, body.trade_id).await?;
    Ok(Json(
        traderview_db::shares::create(
            &s.pool,
            body.trade_id,
            user.id,
            body.is_public,
            body.show_notes,
            body.show_screenshots,
        )
        .await
        .map_err(ApiError::Internal)?,
    ))
}

async fn list_mine(
    State(s): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<TradeShare>>, ApiError> {
    Ok(Json(
        traderview_db::shares::list_for_owner(&s.pool, user.id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

async fn list_public(State(s): State<AppState>) -> Result<Json<Vec<TradeShare>>, ApiError> {
    Ok(Json(
        traderview_db::shares::list_public(&s.pool, 100)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

#[derive(Serialize)]
struct PublicView {
    share: TradeShare,
    trade: Trade,
}

async fn view_public(
    State(s): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<PublicView>, ApiError> {
    let share = traderview_db::shares::by_slug(&s.pool, &slug)
        .await
        .map_err(ApiError::Internal)?
        .ok_or(ApiError::NotFound)?;
    if !share.is_public {
        return Err(ApiError::Forbidden);
    }
    let trade = traderview_db::trades::get(&s.pool, share.trade_id)
        .await
        .map_err(ApiError::Internal)?
        .ok_or(ApiError::NotFound)?;
    traderview_db::shares::bump_view(&s.pool, &slug)
        .await
        .map_err(ApiError::Internal)?;
    Ok(Json(PublicView { share, trade }))
}

async fn delete_one(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<bool>, ApiError> {
    Ok(Json(
        traderview_db::shares::delete(&s.pool, user.id, id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}
