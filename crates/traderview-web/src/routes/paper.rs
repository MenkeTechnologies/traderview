use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::Utc;
use rust_decimal::Decimal;
use serde::Deserialize;
use traderview_core::models::{AssetClass, TradeSide};
use traderview_core::risk_gate::{evaluate, ProposedTrade, Severity};
use traderview_db::paper::{OrderRequest, PaperAccount, PaperOrder, PaperPosition};
use traderview_db::risk_rules;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/paper/accounts", get(list).post(ensure_default))
        .route("/paper/accounts/:id/reset", post(reset))
        .route("/paper/accounts/:id/orders", get(orders).post(submit))
        .route("/paper/accounts/:id/positions", get(positions))
}

async fn list(
    State(s): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<PaperAccount>>, ApiError> {
    Ok(Json(
        traderview_db::paper::list_accounts(&s.pool, user.id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

async fn ensure_default(
    State(s): State<AppState>,
    user: AuthUser,
) -> Result<Json<PaperAccount>, ApiError> {
    Ok(Json(
        traderview_db::paper::ensure_default(&s.pool, user.id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

#[derive(Deserialize)]
struct ResetBody {
    starting_cash: Decimal,
}

async fn reset(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(b): Json<ResetBody>,
) -> Result<Json<bool>, ApiError> {
    Ok(Json(
        traderview_db::paper::reset(&s.pool, user.id, id, b.starting_cash)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

#[derive(Deserialize)]
struct OrdersQ {
    #[serde(default = "default_limit")]
    limit: i64,
}
fn default_limit() -> i64 {
    100
}

async fn orders(
    State(s): State<AppState>,
    _user: AuthUser,
    Path(id): Path<Uuid>,
    Query(q): Query<OrdersQ>,
) -> Result<Json<Vec<PaperOrder>>, ApiError> {
    Ok(Json(
        traderview_db::paper::list_orders(&s.pool, id, q.limit)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

async fn submit(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<OrderRequest>,
) -> Result<Json<PaperOrder>, ApiError> {
    // ─── Run the pre-trade Risk Gate against paper orders too ────────────
    // Same rules the live new-trade form enforces. Paper trading is the
    // place to BUILD the habit; gating only live trades would let the
    // user practice rule-breaking. We use the paper account id as the
    // context source so paper-specific equity / today's P&L drive the
    // % checks.
    let proposed = ProposedTrade {
        symbol: req.symbol.clone(),
        // Side mapping: paper OrderRequest uses Side (buy/sell/short/cover);
        // ProposedTrade wants TradeSide. Same mapping as new_trade.js.
        side: match req.side {
            traderview_core::Side::Buy | traderview_core::Side::Sell => TradeSide::Long,
            traderview_core::Side::Short | traderview_core::Side::Cover => TradeSide::Short,
        },
        qty: req.qty,
        // Best-effort entry price for the gate — limit price if set, else
        // stop, else zero (the % rules would just degrade gracefully).
        entry_price: req.limit_price.or(req.stop_price).unwrap_or(Decimal::ZERO),
        stop_loss: req.stop_price,
        asset_class: AssetClass::Stock,
        multiplier: Decimal::ONE,
        tick_size: None,
        tick_value: None,
        has_attached_plan: false,
    };
    let rows = risk_rules::list(&s.pool, user.id, Some(id))
        .await
        .map_err(ApiError::Internal)?;
    let rules: Vec<_> = rows
        .into_iter()
        .filter(|r| r.enabled)
        .map(|r| r.rule)
        .collect();
    if !rules.is_empty() {
        let ctx = risk_rules::build_context(&s.pool, id)
            .await
            .map_err(ApiError::Internal)?;
        let decision = evaluate(&proposed, &ctx, &rules, Utc::now());
        if !decision.allow {
            let msg = decision
                .violations
                .iter()
                .filter(|v| v.severity == Severity::Block)
                .map(|v| format!("[{}] {}", v.rule, v.message))
                .collect::<Vec<_>>()
                .join("; ");
            return Err(ApiError::BadRequest(format!("Risk Gate blocked: {msg}")));
        }
    }

    Ok(Json(
        traderview_db::paper::submit(&s.pool, user.id, id, req)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

async fn positions(
    State(s): State<AppState>,
    _user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<PaperPosition>>, ApiError> {
    Ok(Json(
        traderview_db::paper::positions(&s.pool, id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}
