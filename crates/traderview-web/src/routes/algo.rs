//! REST surface for momentum algo trading.
//!
//! CRUD for strategies + lifecycle endpoints (start/stop run, engage /
//! release kill switch). The actual engine tick pump (subscribe to live
//! tick bus, evaluate strategy, dispatch to broker) is NOT mounted here
//! — it's an in-process tokio task wired separately in `bin/server.rs`
//! when broker_mode != internal_sim.
//!
//! All write endpoints require the standard `AuthUser` extractor.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use serde::Deserialize;
use traderview_db::algo::{
    AlgoFill, AlgoOrder, AlgoRun, AlgoStrategy, AlgoStrategyInput, KillSwitchAudit,
};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/algo/strategies", get(list_strategies).post(create_strategy))
        .route(
            "/algo/strategies/:id",
            put(update_strategy).delete(delete_strategy),
        )
        .route("/algo/strategies/:id/kill-switch", post(post_kill_switch))
        .route("/algo/strategies/:id/kill-history", get(get_kill_history))
        .route("/algo/strategies/:id/runs", get(list_runs).post(start_run))
        .route("/algo/strategies/:id/stop", post(stop_run))
        .route("/algo/runs/:id/orders", get(list_orders))
        .route("/algo/orders/:id/fills", get(list_fills))
}

// ─── strategy CRUD ──────────────────────────────────────────────────────────

async fn list_strategies(
    State(s): State<AppState>,
    u: AuthUser,
) -> Result<Json<Vec<AlgoStrategy>>, ApiError> {
    Ok(Json(
        traderview_db::algo::list_strategies(&s.pool, u.id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

const VALID_STRATEGY_TYPES: &[&str] = &[
    "momentum", "mean_reversion", "orb", "donchian_trend", "bb_squeeze",
    "ttm_squeeze", "vwap_scalp", "supertrend", "heikin_ashi_trend",
    "connors_rsi2", "order_block_sweep", "pead", "pairs",
];

async fn user_owns_account(
    pool: &sqlx::PgPool,
    user_id: Uuid,
    account_id: Uuid,
) -> Result<bool, ApiError> {
    let row: Option<(Uuid,)> =
        sqlx::query_as("SELECT id FROM accounts WHERE id = $1 AND user_id = $2")
            .bind(account_id)
            .bind(user_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| ApiError::Internal(e.into()))?;
    Ok(row.is_some())
}

async fn create_strategy(
    State(s): State<AppState>,
    u: AuthUser,
    Json(body): Json<AlgoStrategyInput>,
) -> Result<Json<AlgoStrategy>, ApiError> {
    if body.name.trim().is_empty() {
        return Err(ApiError::BadRequest("name required".into()));
    }
    if !VALID_STRATEGY_TYPES.contains(&body.strategy_type.as_str()) {
        return Err(ApiError::BadRequest(format!(
            "unknown strategy_type {}; expected one of: {}",
            body.strategy_type,
            VALID_STRATEGY_TYPES.join(", "),
        )));
    }
    // Every strategy must be bound to a real broker account so fills
    // flow into the standard executions → trades pipeline. Refuse the
    // save outright instead of letting the row sit inert and confuse
    // the user about why nothing happens.
    let account_id = body
        .account_id
        .ok_or_else(|| ApiError::BadRequest(
            "account_id required — pick a broker account before saving (Settings → Accounts to add one)".into(),
        ))?;
    if !user_owns_account(&s.pool, u.id, account_id).await? {
        return Err(ApiError::BadRequest("account_id does not belong to you".into()));
    }
    // Engine code enforces paper_locked_until at submit-time, but the
    // route also rejects naked alpaca_live at create-time so the UI
    // doesn't show a misleading "saved" toast for a config the engine
    // will refuse to run.
    if body.broker_mode == "alpaca_live" {
        return Err(ApiError::BadRequest(
            "new strategies must start in internal_sim or alpaca_paper; switch to alpaca_live after the 30-day paper-lock expires".into(),
        ));
    }
    let broker_mode = body.broker_mode.clone();
    let created = traderview_db::algo::create_strategy(&s.pool, u.id, body)
        .await
        .map_err(ApiError::Internal)?;
    maybe_hot_spawn_pump(&s, u.id, &broker_mode).await;
    Ok(Json(created))
}

/// If the strategy's broker_mode points at Alpaca, ensure a
/// trade_updates pump exists for (user, paper-or-live). Idempotent —
/// no-op if one is already running for that tuple. Errors don't fail
/// the route; the pump just won't push WS events until restart.
async fn maybe_hot_spawn_pump(state: &AppState, user_id: Uuid, broker_mode: &str) {
    let paper = match broker_mode {
        "alpaca_paper" => true,
        "alpaca_live" => false,
        _ => return,
    };
    let sink = state.build_engine_event_sink();
    let spawned = traderview_db::alpaca_pump::ensure_pump_for(
        state.alpaca_pumps.clone(),
        state.pool.clone(),
        user_id,
        paper,
        Some(sink),
    )
    .await;
    if spawned {
        tracing::info!(user_id = %user_id, paper, "alpaca pump hot-spawned");
    }
}

async fn update_strategy(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<AlgoStrategyInput>,
) -> Result<Json<AlgoStrategy>, ApiError> {
    if !VALID_STRATEGY_TYPES.contains(&body.strategy_type.as_str()) {
        return Err(ApiError::BadRequest(format!(
            "unknown strategy_type {}; expected one of: {}",
            body.strategy_type,
            VALID_STRATEGY_TYPES.join(", "),
        )));
    }
    let account_id = body
        .account_id
        .ok_or_else(|| ApiError::BadRequest("account_id required".into()))?;
    if !user_owns_account(&s.pool, u.id, account_id).await? {
        return Err(ApiError::BadRequest("account_id does not belong to you".into()));
    }
    // Allow alpaca_live on update only if existing paper_locked_until
    // has expired. Engine still re-checks at submit.
    if body.broker_mode == "alpaca_live" {
        let existing = traderview_db::algo::get_strategy(&s.pool, u.id, id)
            .await
            .map_err(ApiError::Internal)?
            .ok_or(ApiError::NotFound)?;
        if chrono::Utc::now() <= existing.paper_locked_until {
            return Err(ApiError::BadRequest(format!(
                "strategy is paper-locked until {}; cannot promote to alpaca_live yet",
                existing.paper_locked_until
            )));
        }
    }
    let broker_mode = body.broker_mode.clone();
    let updated = traderview_db::algo::update_strategy(&s.pool, u.id, id, body)
        .await
        .map_err(ApiError::Internal)?
        .ok_or(ApiError::NotFound)?;
    maybe_hot_spawn_pump(&s, u.id, &broker_mode).await;
    Ok(Json(updated))
}

async fn delete_strategy(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let ok = traderview_db::algo::delete_strategy(&s.pool, u.id, id)
        .await
        .map_err(ApiError::Internal)?;
    if !ok {
        return Err(ApiError::NotFound);
    }
    Ok(Json(serde_json::json!({ "deleted": true })))
}

// ─── kill switch ────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct KillSwitchBody {
    engaged: bool,
    #[serde(default)]
    reason: Option<String>,
}

async fn post_kill_switch(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<KillSwitchBody>,
) -> Result<Json<AlgoStrategy>, ApiError> {
    traderview_db::algo::set_kill_switch(&s.pool, u.id, id, body.engaged, body.reason)
        .await
        .map_err(ApiError::Internal)?
        .map(Json)
        .ok_or(ApiError::NotFound)
}

async fn get_kill_history(
    State(s): State<AppState>,
    _u: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<KillSwitchAudit>>, ApiError> {
    Ok(Json(
        traderview_db::algo::kill_switch_history(&s.pool, id, 100)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

// ─── run lifecycle ──────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct StopRunBody {
    #[serde(default = "default_stop_reason")]
    reason: String,
}

fn default_stop_reason() -> String { "user".into() }

async fn start_run(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<AlgoRun>, ApiError> {
    let strategy = traderview_db::algo::get_strategy(&s.pool, u.id, id)
        .await
        .map_err(ApiError::Internal)?
        .ok_or(ApiError::NotFound)?;
    if strategy.kill_switch {
        return Err(ApiError::BadRequest(
            "kill switch engaged — release it before starting a run".into(),
        ));
    }
    if !strategy.enabled {
        return Err(ApiError::BadRequest("strategy is disabled".into()));
    }
    // account_id is NOT NULL since migration 0056 — strategy must be
    // bound. No defensive check needed; the row could not have been
    // saved otherwise.
    if let Some(open) = traderview_db::algo::get_open_run(&s.pool, id)
        .await
        .map_err(ApiError::Internal)?
    {
        // Idempotent: re-starting a strategy with an open run returns
        // that run instead of erroring — UI re-mounting shouldn't break.
        return Ok(Json(open));
    }
    Ok(Json(
        traderview_db::algo::start_run(&s.pool, id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

async fn stop_run(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<StopRunBody>,
) -> Result<Json<AlgoRun>, ApiError> {
    // Ownership check — get_strategy returns None for cross-user access.
    traderview_db::algo::get_strategy(&s.pool, u.id, id)
        .await
        .map_err(ApiError::Internal)?
        .ok_or(ApiError::NotFound)?;
    let open = traderview_db::algo::get_open_run(&s.pool, id)
        .await
        .map_err(ApiError::Internal)?
        .ok_or_else(|| ApiError::BadRequest("no open run for this strategy".into()))?;
    traderview_db::algo::stop_run(&s.pool, open.id, &body.reason, None, None)
        .await
        .map_err(ApiError::Internal)?
        .map(Json)
        .ok_or(ApiError::NotFound)
}

#[derive(Debug, Deserialize)]
struct ListRunsQuery {
    #[serde(default = "default_limit")]
    limit: i64,
}

fn default_limit() -> i64 { 25 }

async fn list_runs(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
    Query(q): Query<ListRunsQuery>,
) -> Result<Json<Vec<AlgoRun>>, ApiError> {
    // Ownership check.
    traderview_db::algo::get_strategy(&s.pool, u.id, id)
        .await
        .map_err(ApiError::Internal)?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(
        traderview_db::algo::list_runs(&s.pool, id, q.limit.clamp(1, 200))
            .await
            .map_err(ApiError::Internal)?,
    ))
}

async fn list_orders(
    State(s): State<AppState>,
    _u: AuthUser,
    Path(id): Path<Uuid>,
    Query(q): Query<ListRunsQuery>,
) -> Result<Json<Vec<AlgoOrder>>, ApiError> {
    Ok(Json(
        traderview_db::algo::list_orders(&s.pool, id, q.limit.clamp(1, 500))
            .await
            .map_err(ApiError::Internal)?,
    ))
}

async fn list_fills(
    State(s): State<AppState>,
    _u: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<AlgoFill>>, ApiError> {
    Ok(Json(
        traderview_db::algo::list_fills(&s.pool, id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}
