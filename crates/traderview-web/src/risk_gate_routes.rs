//! Pre-trade risk-gate routes. CRUD on rules + the `evaluate` endpoint
//! that the new-trade UI calls before submitting an order.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use traderview_core::risk_gate::{evaluate, preset_rules, GateDecision, Preset, ProposedTrade, RiskRule};
use traderview_db::risk_rules;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/rules",           get(list_rules).post(create_rule))
        .route("/rules/:id",       delete(delete_rule))
        .route("/rules/:id/toggle", post(toggle_rule))
        .route("/rules/install-preset", post(install_preset))
        .route("/evaluate",        post(evaluate_proposed))
}

// ---------------------------------------------------------------------------
// CRUD on rules
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct ListRulesQuery {
    account_id: Option<Uuid>,
}

async fn list_rules(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<ListRulesQuery>,
) -> Result<Json<Vec<risk_rules::StoredRule>>, ApiError> {
    let rows = risk_rules::list(&s.pool, user.id, q.account_id)
        .await.map_err(ApiError::Internal)?;
    Ok(Json(rows))
}

#[derive(Deserialize)]
struct CreateRuleBody {
    account_id: Option<Uuid>,
    rule: RiskRule,
}

#[derive(Serialize)]
struct CreatedRule { id: Uuid }

async fn create_rule(
    State(s): State<AppState>,
    user: AuthUser,
    Json(body): Json<CreateRuleBody>,
) -> Result<Json<CreatedRule>, ApiError> {
    let id = risk_rules::create(&s.pool, user.id, body.account_id, &body.rule)
        .await.map_err(ApiError::Internal)?;
    Ok(Json(CreatedRule { id }))
}

async fn delete_rule(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let n = risk_rules::delete(&s.pool, user.id, id)
        .await.map_err(ApiError::Internal)?;
    if n == 0 { return Err(ApiError::NotFound); }
    Ok(Json(serde_json::json!({ "deleted": n })))
}

#[derive(Deserialize)]
struct ToggleBody { enabled: bool }

async fn toggle_rule(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<ToggleBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let n = risk_rules::set_enabled(&s.pool, user.id, id, body.enabled)
        .await.map_err(ApiError::Internal)?;
    if n == 0 { return Err(ApiError::NotFound); }
    Ok(Json(serde_json::json!({ "updated": n, "enabled": body.enabled })))
}

// ---------------------------------------------------------------------------
// Install a preset rule pack
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct InstallPresetBody {
    preset: Preset,
    account_id: Option<Uuid>,
}

#[derive(Serialize)]
struct InstalledPreset { inserted: usize }

/// Bulk-insert every rule in the chosen preset. Existing rules are kept;
/// the user can delete them manually if they want a clean slate. We
/// avoid auto-wipe so users don't lose hand-tuned overrides.
async fn install_preset(
    State(s): State<AppState>,
    user: AuthUser,
    Json(body): Json<InstallPresetBody>,
) -> Result<Json<InstalledPreset>, ApiError> {
    let rules = preset_rules(body.preset);
    let n = rules.len();
    for r in rules {
        risk_rules::create(&s.pool, user.id, body.account_id, &r)
            .await.map_err(ApiError::Internal)?;
    }
    Ok(Json(InstalledPreset { inserted: n }))
}

// ---------------------------------------------------------------------------
// Evaluate a proposed trade
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct EvaluateRequest {
    account_id: Uuid,
    proposed: ProposedTrade,
}

async fn evaluate_proposed(
    State(s): State<AppState>,
    user: AuthUser,
    Json(req): Json<EvaluateRequest>,
) -> Result<Json<GateDecision>, ApiError> {
    // Owner-scope check: must own the account being evaluated against.
    let owner: Option<(Uuid,)> = sqlx::query_as(
        "SELECT user_id FROM accounts WHERE id = $1",
    )
    .bind(req.account_id)
    .fetch_optional(&s.pool).await?;
    match owner {
        Some((u,)) if u == user.id => {}
        Some(_) => return Err(ApiError::Forbidden),
        None    => return Err(ApiError::NotFound),
    }

    // Pull only enabled rules that apply to this account (or are global).
    let rows = risk_rules::list(&s.pool, user.id, Some(req.account_id))
        .await.map_err(ApiError::Internal)?;
    let rules: Vec<RiskRule> = rows.into_iter()
        .filter(|r| r.enabled)
        .map(|r| r.rule)
        .collect();

    let ctx = risk_rules::build_context(&s.pool, req.account_id)
        .await.map_err(ApiError::Internal)?;

    let decision = evaluate(&req.proposed, &ctx, &rules, Utc::now());
    Ok(Json(decision))
}
