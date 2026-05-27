use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use traderview_core::position_size::{
    self, FixedFractionalParams, Inputs, KellyParams, RBasedParams, Report, Side,
};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new().route("/position-size", post(compute)).route(
        "/position-size/account/:account_id/winrate",
        get(account_winrate),
    )
}

#[derive(Debug, Deserialize)]
struct Body {
    side: Side,
    entry: f64,
    stop: f64,
    equity: f64,
    #[serde(default)]
    correlation_drag: f64,
    #[serde(default)]
    max_position_pct: f64,
    fixed_fractional: Option<FixedFractionalParams>,
    r_based: Option<RBasedParams>,
    kelly: Option<KellyParams>,
    /// "fixed_fractional" | "r_based" | "kelly"
    recommended_method: Option<String>,
}

async fn compute(
    State(_s): State<AppState>,
    _u: AuthUser,
    Json(b): Json<Body>,
) -> Result<Json<Report>, ApiError> {
    let inputs = Inputs {
        side: b.side,
        entry: b.entry,
        stop: b.stop,
        equity: b.equity,
        correlation_drag: b.correlation_drag,
        max_position_pct: b.max_position_pct,
    };
    Ok(Json(position_size::report(
        inputs,
        b.fixed_fractional,
        b.r_based,
        b.kelly,
        b.recommended_method.as_deref(),
    )))
}

#[derive(Debug, Serialize)]
struct WinRateResp {
    samples: i64,
    wins: i64,
    losses: i64,
    win_rate: f64,
    avg_win: f64,
    avg_loss: f64,
    /// Suggested KellyParams for the calculator.
    kelly: KellyParams,
}

async fn account_winrate(
    State(s): State<AppState>,
    u: AuthUser,
    axum::extract::Path(account_id): axum::extract::Path<Uuid>,
) -> Result<Json<WinRateResp>, ApiError> {
    // Confirm account ownership (matches helpers::ensure_account_owner).
    let row: Option<(Uuid,)> = sqlx::query_as("SELECT user_id FROM accounts WHERE id = $1")
        .bind(account_id)
        .fetch_optional(&s.pool)
        .await
        .map_err(ApiError::Db)?;
    match row {
        Some((owner,)) if owner == u.id => {}
        Some(_) => return Err(ApiError::Forbidden),
        None => return Err(ApiError::NotFound),
    }
    // Aggregate closed trades.
    let stats: (Option<i64>, Option<i64>, Option<f64>, Option<f64>) = sqlx::query_as(
        "SELECT
            COUNT(*) FILTER (WHERE net_pnl > 0),
            COUNT(*) FILTER (WHERE net_pnl < 0),
            AVG(net_pnl) FILTER (WHERE net_pnl > 0)::float8,
            AVG(net_pnl) FILTER (WHERE net_pnl < 0)::float8
           FROM trades
          WHERE account_id = $1 AND status = 'closed' AND net_pnl IS NOT NULL",
    )
    .bind(account_id)
    .fetch_one(&s.pool)
    .await
    .map_err(ApiError::Db)?;
    let wins = stats.0.unwrap_or(0);
    let losses = stats.1.unwrap_or(0);
    let total = wins + losses;
    let win_rate = if total > 0 {
        wins as f64 / total as f64
    } else {
        0.0
    };
    let avg_win = stats.2.unwrap_or(0.0);
    let avg_loss = stats.3.unwrap_or(0.0).abs();
    Ok(Json(WinRateResp {
        samples: total,
        wins,
        losses,
        win_rate,
        avg_win,
        avg_loss,
        kelly: KellyParams {
            win_rate,
            avg_win: if avg_win > 0.0 { avg_win } else { 1.0 },
            avg_loss: if avg_loss > 0.0 { avg_loss } else { 1.0 },
            fractional_kelly: 0.5,
        },
    }))
}
