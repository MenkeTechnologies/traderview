use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::routes::helpers::ensure_account_owner;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use chrono::{Duration, Utc};
use serde::Deserialize;
use traderview_core::discipline_score::{score as compute_score, DisciplineScore, ScoreInputs};
use traderview_db::discipline::DisciplineReport;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/discipline/:account_id", get(report))
        .route("/discipline/:account_id/score", get(score_route))
}

async fn report(
    State(s): State<AppState>,
    u: AuthUser,
    Path(account_id): Path<Uuid>,
) -> Result<Json<DisciplineReport>, ApiError> {
    ensure_account_owner(&s, u.id, account_id).await?;
    Ok(Json(
        traderview_db::discipline::report(&s.pool, u.id, account_id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

#[derive(Deserialize)]
struct ScoreQuery {
    #[serde(default = "default_score_days")]
    days: i64,
}
fn default_score_days() -> i64 {
    7
}

/// Unified discipline score = post-trade rule evals + pre-trade Risk Gate
/// fires, condensed to a single 0-100 number + letter grade.
async fn score_route(
    State(s): State<AppState>,
    u: AuthUser,
    Path(account_id): Path<Uuid>,
    Query(q): Query<ScoreQuery>,
) -> Result<Json<DisciplineScore>, ApiError> {
    ensure_account_owner(&s, u.id, account_id).await?;
    let days = q.days.clamp(1, 365);

    // Post-trade signals from the existing discipline report. We sum the
    // per-trade rule evaluations into the ScoreInputs shape.
    let report = traderview_db::discipline::report(&s.pool, u.id, account_id)
        .await
        .map_err(ApiError::Internal)?;
    let trades = report.rule_evals.len() as u32;
    let trades_with_stop = report.rule_evals.iter().filter(|e| e.stop_set).count() as u32;
    let trades_stop_honored = report.rule_evals.iter().filter(|e| e.stop_honored).count() as u32;
    let trades_qty_within_plan = report.rule_evals.iter().filter(|e| e.qty_within).count() as u32;
    let trades_direction_matched = report
        .rule_evals
        .iter()
        .filter(|e| e.direction_match)
        .count() as u32;

    // Pre-trade signals from the risk_fires table, scoped to the window.
    let cutoff = Utc::now() - Duration::days(days);
    let recent_fires = traderview_db::risk_rules::recent_fires(&s.pool, u.id, 500)
        .await
        .map_err(ApiError::Internal)?;
    let mut gate_warnings = 0u32;
    let mut gate_blocks = 0u32;
    for f in &recent_fires {
        if f.fired_at < cutoff {
            continue;
        }
        // Scope to the account if the fire was account-scoped; user-global
        // fires (account_id IS NULL) always count.
        if let Some(ac) = f.account_id {
            if ac != account_id {
                continue;
            }
        }
        for v in &f.decision.violations {
            match v.severity {
                traderview_core::risk_gate::Severity::Warning => gate_warnings += 1,
                traderview_core::risk_gate::Severity::Block => gate_blocks += 1,
            }
        }
    }

    let inputs = ScoreInputs {
        trades,
        trades_with_stop,
        trades_stop_honored,
        trades_qty_within_plan,
        trades_direction_matched,
        gate_warnings,
        gate_blocks,
    };
    Ok(Json(compute_score(&inputs)))
}
