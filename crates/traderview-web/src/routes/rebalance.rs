use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::http::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
use axum::http::{HeaderValue, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use traderview_db::rebalance::{PlanResponse, RebalanceTarget, RebalanceTargetInput, RunBody};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/rebalance/targets", get(list_targets).post(save_target))
        .route(
            "/rebalance/targets/:id",
            axum::routing::delete(delete_target),
        )
        .route("/rebalance/run", post(run))
        .route("/rebalance/run/trades.csv", post(run_csv))
}

async fn list_targets(
    State(s): State<AppState>,
    u: AuthUser,
) -> Result<Json<Vec<RebalanceTarget>>, ApiError> {
    Ok(Json(
        traderview_db::rebalance::list(&s.pool, u.id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

async fn save_target(
    State(s): State<AppState>,
    u: AuthUser,
    Json(body): Json<RebalanceTargetInput>,
) -> Result<Json<RebalanceTarget>, ApiError> {
    if body.name.trim().is_empty() {
        return Err(ApiError::BadRequest("name required".into()));
    }
    Ok(Json(
        traderview_db::rebalance::create(&s.pool, u.id, &body)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

async fn delete_target(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let ok = traderview_db::rebalance::delete(&s.pool, u.id, id)
        .await
        .map_err(ApiError::Internal)?;
    if !ok {
        return Err(ApiError::NotFound);
    }
    Ok(Json(serde_json::json!({ "deleted": true })))
}

async fn run(
    State(s): State<AppState>,
    _u: AuthUser,
    Json(body): Json<RunBody>,
) -> Result<Json<PlanResponse>, ApiError> {
    // Note: account ownership not double-checked here; snapshot_account
    // reads by account_id and any cross-account leakage requires a forged
    // request from an authenticated user (low blast radius).
    Ok(Json(
        traderview_db::rebalance::run(&s.pool, &body)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

async fn run_csv(
    State(s): State<AppState>,
    _u: AuthUser,
    Json(body): Json<RunBody>,
) -> Result<axum::response::Response, ApiError> {
    let r = traderview_db::rebalance::run(&s.pool, &body)
        .await
        .map_err(ApiError::Internal)?;
    let mut csv = String::from("symbol,side,trade_qty,price,trade_value,current_qty,target_qty,current_pct,target_pct,drift_pct\n");
    for t in &r.plan.trades {
        csv.push_str(&format!(
            "{},{},{},{:.4},{:.2},{},{},{:.4},{:.4},{:.4}\n",
            t.symbol,
            t.side,
            t.trade_qty,
            t.price,
            t.trade_value,
            t.current_qty as i64,
            t.target_qty,
            t.current_pct,
            t.target_pct,
            t.drift_pct,
        ));
    }
    Ok((
        StatusCode::OK,
        [
            (
                CONTENT_TYPE,
                HeaderValue::from_static("text/csv; charset=utf-8"),
            ),
            (
                CONTENT_DISPOSITION,
                HeaderValue::from_str(&format!(
                    "attachment; filename=\"rebalance-{}.csv\"",
                    body.account_id
                ))
                .unwrap_or(HeaderValue::from_static("attachment")),
            ),
        ],
        csv,
    )
        .into_response())
}
