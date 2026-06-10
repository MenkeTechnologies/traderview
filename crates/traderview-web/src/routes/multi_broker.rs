//! Multi-broker position aggregation + kill-switch routes.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use traderview_db::multi_broker;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/multi-broker/positions", get(positions))
        .route("/multi-broker/kill-switch", post(kill_switch))
        .route("/multi-broker/kill-switch/log", get(kill_log))
}

#[derive(Serialize)]
struct KillSwitchAuditRow {
    id: i64,
    fired_at: chrono::DateTime<chrono::Utc>,
    brokers_attempted: String,
    cancelled_orders: i32,
    closed_positions: i32,
    reason: Option<String>,
    error_count: i32,
}

async fn kill_log(
    State(s): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<KillSwitchAuditRow>>, ApiError> {
    let rows: Vec<(
        i64,
        chrono::DateTime<chrono::Utc>,
        String,
        i32,
        i32,
        Option<String>,
        i32,
    )> = sqlx::query_as(
        "SELECT id, fired_at, brokers_attempted, cancelled_orders,
                closed_positions, reason, error_count
           FROM multi_broker_kill_switch_audit
          WHERE user_id = $1
          ORDER BY fired_at DESC
          LIMIT 100",
    )
    .bind(user.id)
    .fetch_all(&s.pool)
    .await?;
    Ok(Json(
        rows.into_iter()
            .map(|(id, fired_at, brokers, c, p, r, e)| KillSwitchAuditRow {
                id,
                fired_at,
                brokers_attempted: brokers,
                cancelled_orders: c,
                closed_positions: p,
                reason: r,
                error_count: e,
            })
            .collect(),
    ))
}

async fn positions(
    State(s): State<AppState>,
    user: AuthUser,
) -> Result<Json<multi_broker::MultiBrokerReport>, ApiError> {
    Ok(Json(multi_broker::fetch_all(&s.pool, user.id).await?))
}

/// Kill-switch payload. The `confirm_token` must equal the literal
/// string `KILL-ALL-NOW` — a deliberate friction-point so an
/// accidental POST can't trigger the destructive action. The UI's
/// `tConfirm` dialog asks the user to type the token explicitly.
#[derive(Deserialize)]
struct KillSwitchBody {
    confirm_token: String,
    #[serde(default)]
    reason: Option<String>,
}

#[derive(Serialize)]
struct KillSwitchResponse {
    brokers_attempted: Vec<String>,
    cancelled_orders: usize,
    closed_positions: usize,
    per_broker: Vec<multi_broker::KillSwitchBrokerOutcome>,
    errors: Vec<multi_broker::BrokerError>,
}

async fn kill_switch(
    State(s): State<AppState>,
    user: AuthUser,
    Json(body): Json<KillSwitchBody>,
) -> Result<Json<KillSwitchResponse>, ApiError> {
    const EXPECTED: &str = "KILL-ALL-NOW";
    if body.confirm_token.trim() != EXPECTED {
        return Err(ApiError::BadRequest(format!(
            "confirm_token must equal {EXPECTED:?} — refusing to fire kill-switch"
        )));
    }
    let r = multi_broker::kill_all_for_user(&s.pool, user.id).await?;
    // Audit insert MUST surface failures — silently dropping the row
    // after a destructive cancel/flat across all brokers is a
    // compliance disaster. tracing::error so monitoring picks it up,
    // then continue: the broker-side action already happened, so the
    // handler still reports what was cancelled. We do NOT bubble the
    // audit error to the caller because that would suggest the kill
    // didn't fire when it did.
    if let Err(e) = sqlx::query(
        "INSERT INTO multi_broker_kill_switch_audit
            (user_id, brokers_attempted, cancelled_orders, closed_positions, reason, error_count)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(user.id)
    .bind(r.brokers_attempted.join(","))
    .bind(r.cancelled_orders as i32)
    .bind(r.closed_positions as i32)
    .bind(&body.reason)
    .bind(r.errors.len() as i32)
    .execute(&s.pool)
    .await
    {
        tracing::error!(
            user_id = %user.id, error = %e,
            cancelled = r.cancelled_orders, closed = r.closed_positions,
            "multi_broker kill_switch audit INSERT failed; destructive action completed but audit row missing"
        );
    }
    // Partial-failure status: any per-broker error means the caller's
    // monitoring should trip. Return 207 Multi-Status semantics via
    // ApiError so the response body still carries the structured
    // per-broker result.
    if !r.errors.is_empty() {
        let body = serde_json::to_string(&KillSwitchResponse {
            brokers_attempted: r.brokers_attempted.clone(),
            cancelled_orders: r.cancelled_orders,
            closed_positions: r.closed_positions,
            per_broker: r.per_broker.clone(),
            errors: r.errors.clone(),
        })
        .unwrap_or_else(|_| "{\"errors\":\"serialization failed\"}".into());
        return Err(ApiError::BadRequest(format!(
            "partial broker failure (207 Multi-Status): {body}"
        )));
    }
    Ok(Json(KillSwitchResponse {
        brokers_attempted: r.brokers_attempted,
        cancelled_orders: r.cancelled_orders,
        closed_positions: r.closed_positions,
        per_broker: r.per_broker,
        errors: r.errors,
    }))
}
