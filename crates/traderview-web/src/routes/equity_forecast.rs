use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::routes::helpers::ensure_account_owner;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use rust_decimal::Decimal;
use serde::Deserialize;
use traderview_core::equity_forecast::{forecast, ForecastInput, ForecastReport};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new().route("/equity-forecast", post(run))
}

#[derive(Debug, Deserialize)]
struct Body {
    account_id: Uuid,
    starting_equity: f64,
    risk_pct_per_trade: f64,
    num_trades: usize,
    num_paths: usize,
    seed: Option<u64>,
    ruin_threshold_pct: Option<f64>,
}

async fn run(
    State(s): State<AppState>,
    u: AuthUser,
    Json(b): Json<Body>,
) -> Result<Json<ForecastReport>, ApiError> {
    ensure_account_owner(&s, u.id, b.account_id).await?;
    // Pull historical R sample from closed trades with risk_amount set.
    let rows: Vec<(Decimal, Decimal)> = sqlx::query_as(
        "SELECT net_pnl, risk_amount FROM trades
          WHERE account_id = $1 AND status = 'closed'
            AND net_pnl IS NOT NULL
            AND risk_amount IS NOT NULL AND risk_amount > 0",
    )
    .bind(b.account_id)
    .fetch_all(&s.pool)
    .await
    .map_err(ApiError::Db)?;
    if rows.is_empty() {
        return Err(ApiError::BadRequest(
            "no closed trades with risk_amount set on this account".into(),
        ));
    }
    let r_samples: Vec<f64> = rows
        .iter()
        .map(|(pnl, risk)| dec(*pnl) / dec(*risk))
        .collect();
    let input = ForecastInput {
        r_samples,
        starting_equity: b.starting_equity,
        risk_pct_per_trade: b.risk_pct_per_trade,
        num_trades: b.num_trades,
        num_paths: b.num_paths,
        seed: b.seed,
        ruin_threshold_pct: b.ruin_threshold_pct,
    };
    Ok(Json(forecast(&input)))
}

fn dec(d: Decimal) -> f64 {
    d.to_string().parse().unwrap_or(0.0)
}
