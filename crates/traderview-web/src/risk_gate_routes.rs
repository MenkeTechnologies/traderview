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
        .route("/fires",           get(list_fires))
        .route("/fires/by-rule",   get(fires_by_rule))
        .route("/kill-switch",     get(kill_switch_state))
}

#[derive(Deserialize)]
struct ByRuleQuery { #[serde(default = "default_days")] days: i64 }
fn default_days() -> i64 { 30 }

async fn fires_by_rule(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<ByRuleQuery>,
) -> Result<Json<Vec<risk_rules::RuleFireStat>>, ApiError> {
    let rows = risk_rules::fires_by_rule(&s.pool, user.id, q.days)
        .await.map_err(ApiError::Internal)?;
    Ok(Json(rows))
}

/// Lightweight check the topbar polls every 10s. Returns `installed`
/// (is there a kill_switch rule at all?) + `active` (is it enabled?).
/// Used to drive the red 🛑 indicator without pulling every rule.
#[derive(Serialize)]
struct KillState { installed: bool, active: bool }

async fn kill_switch_state(
    State(s): State<AppState>,
    user: AuthUser,
) -> Result<Json<KillState>, ApiError> {
    let rows = risk_rules::list(&s.pool, user.id, None)
        .await.map_err(ApiError::Internal)?;
    let ks = rows.iter().find(|r| matches!(r.rule, RiskRule::KillSwitch));
    Ok(Json(KillState {
        installed: ks.is_some(),
        active: ks.map(|r| r.enabled).unwrap_or(false),
    }))
}

#[derive(Deserialize)]
struct FiresQuery { #[serde(default = "default_fires_limit")] limit: i64 }
fn default_fires_limit() -> i64 { 100 }

async fn list_fires(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<FiresQuery>,
) -> Result<Json<Vec<risk_rules::RiskFire>>, ApiError> {
    let rows = risk_rules::recent_fires(&s.pool, user.id, q.limit)
        .await.map_err(ApiError::Internal)?;
    Ok(Json(rows))
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

    // Persist the fire (only when at least one rule triggered — log_fire
    // short-circuits empty-violation decisions). Background-spawn so we
    // never block the response on an audit-log insert.
    if !decision.violations.is_empty() {
        let pool = s.pool.clone();
        let user_id = user.id;
        let account_id = req.account_id;
        let symbol = req.proposed.symbol.clone();
        let decision_clone = decision.clone();
        tokio::spawn(async move {
            let _ = risk_rules::log_fire(&pool, user_id, Some(account_id), &symbol, &decision_clone).await;
        });
    }

    // If any Block fired, fan-out to every enabled webhook so the user
    // sees the veto in Discord/Slack/etc. Fire-and-forget — never block
    // the response on outbound HTTP.
    if !decision.allow {
        let blocks: Vec<_> = decision.violations.iter()
            .filter(|v| matches!(v.severity, traderview_core::risk_gate::Severity::Block))
            .collect();
        if !blocks.is_empty() {
            let summary = blocks.iter()
                .map(|b| format!("[{}] {}", b.rule, b.message))
                .collect::<Vec<_>>()
                .join("\n");
            let payload = traderview_db::webhooks::AlertPayload {
                title:   format!("Risk Gate vetoed {} entry", req.proposed.symbol),
                message: summary,
                symbol:  Some(req.proposed.symbol.clone()),
                kind:    "risk_gate_block".into(),
                url:     None,
                fired_at: Utc::now(),
            };
            let pool = s.pool.clone();
            let user_id = user.id;
            tokio::spawn(async move {
                traderview_db::webhooks::fan_out_all(&pool, user_id, &payload).await;
            });
        }
    }

    Ok(Json(decision))
}
