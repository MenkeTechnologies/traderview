use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::routes::helpers::ensure_account_owner;
use crate::state::AppState;
use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use traderview_core::{liquidity, risk, stats, Trade};
use traderview_db::trades::TradeFilter;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/reports/overview", get(overview))
        .route("/reports/by-symbol", get(by_symbol))
        .route("/reports/by-side", get(by_side))
        .route("/reports/by-asset-class", get(by_asset_class))
        .route("/reports/by-day-of-week", get(by_dow))
        .route("/reports/by-hour", get(by_hour))
        .route("/reports/by-hold", get(by_hold))
        .route("/reports/by-month", get(by_month))
        .route("/reports/r-distribution", get(r_distribution))
        .route("/reports/streaks", get(streaks))
        .route("/reports/comparison", get(comparison))
        .route("/reports/exit-efficiency", get(exit_eff))
        .route("/reports/commissions", get(commissions))
        .route("/reports/liquidity", get(liquidity_report))
        .route("/reports/risk", get(risk_report))
        .route("/reports/drawdown", get(drawdown))
        .route("/reports/risk-adjusted", get(risk_adjusted))
        .route("/reports/calendar", get(calendar))
        .route("/stats/summary", get(summary))
        .route("/stats/equity", get(equity))
}

#[derive(Deserialize)]
struct RQ {
    account_id: Uuid,
    #[serde(default)]
    starting_cash: Option<Decimal>,
    /// Optional rolling-window filter (in days). When set, only trades whose
    /// closed_at (falling back to opened_at for still-open trades) is within
    /// the last N days are returned. Powers the dashboard's 30/60/90 toggle.
    #[serde(default)]
    days: Option<i64>,
}

async fn load(s: &AppState, user_id: Uuid, q: &RQ) -> Result<Vec<Trade>, ApiError> {
    ensure_account_owner(s, user_id, q.account_id).await?;
    let f = TradeFilter {
        limit: Some(100_000),
        ..Default::default()
    };
    let mut trades = traderview_db::trades::list_for_account(&s.pool, q.account_id, &f)
        .await
        .map_err(ApiError::Internal)?;
    if let Some(d) = q.days {
        if d > 0 {
            let cutoff = chrono::Utc::now() - chrono::Duration::days(d);
            trades.retain(|t| t.closed_at.unwrap_or(t.opened_at) >= cutoff);
        }
    }
    Ok(trades)
}

async fn overview(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<RQ>,
) -> Result<Json<stats::Summary>, ApiError> {
    let t = load(&s, user.id, &q).await?;
    Ok(Json(stats::summary(&t)))
}

async fn summary(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<RQ>,
) -> Result<Json<stats::Summary>, ApiError> {
    overview(State(s), user, Query(q)).await
}

async fn by_symbol(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<RQ>,
) -> Result<Json<Vec<stats::Bucket>>, ApiError> {
    Ok(Json(stats::by_symbol(
        &load(&s, user.id, &q).await?,
    )))
}
async fn by_side(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<RQ>,
) -> Result<Json<Vec<stats::Bucket>>, ApiError> {
    Ok(Json(stats::by_side(
        &load(&s, user.id, &q).await?,
    )))
}
async fn by_asset_class(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<RQ>,
) -> Result<Json<Vec<stats::Bucket>>, ApiError> {
    Ok(Json(stats::by_asset_class(
        &load(&s, user.id, &q).await?,
    )))
}
async fn by_dow(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<RQ>,
) -> Result<Json<Vec<stats::Bucket>>, ApiError> {
    Ok(Json(stats::by_day_of_week(
        &load(&s, user.id, &q).await?,
    )))
}
async fn by_hour(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<RQ>,
) -> Result<Json<Vec<stats::Bucket>>, ApiError> {
    Ok(Json(stats::by_hour_of_day(
        &load(&s, user.id, &q).await?,
    )))
}
async fn by_hold(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<RQ>,
) -> Result<Json<Vec<stats::Bucket>>, ApiError> {
    Ok(Json(stats::by_hold_bucket(
        &load(&s, user.id, &q).await?,
    )))
}
async fn by_month(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<RQ>,
) -> Result<Json<Vec<stats::Bucket>>, ApiError> {
    Ok(Json(stats::by_month(
        &load(&s, user.id, &q).await?,
    )))
}

async fn r_distribution(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<RQ>,
) -> Result<Json<stats::RMultipleDistribution>, ApiError> {
    Ok(Json(stats::r_distribution(
        &load(&s, user.id, &q).await?,
    )))
}
async fn streaks(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<RQ>,
) -> Result<Json<Vec<stats::Streak>>, ApiError> {
    Ok(Json(stats::streaks(
        &load(&s, user.id, &q).await?,
    )))
}
async fn comparison(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<RQ>,
) -> Result<Json<stats::Comparison>, ApiError> {
    Ok(Json(stats::comparison(
        &load(&s, user.id, &q).await?,
    )))
}
async fn exit_eff(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<RQ>,
) -> Result<Json<stats::ExitEfficiency>, ApiError> {
    Ok(Json(stats::exit_efficiency(
        &load(&s, user.id, &q).await?,
    )))
}
async fn commissions(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<RQ>,
) -> Result<Json<stats::CommissionReport>, ApiError> {
    Ok(Json(stats::commissions(
        &load(&s, user.id, &q).await?,
    )))
}

async fn calendar(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<RQ>,
) -> Result<Json<Vec<stats::CalendarCell>>, ApiError> {
    Ok(Json(stats::calendar(
        &load(&s, user.id, &q).await?,
    )))
}

async fn equity(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<RQ>,
) -> Result<Json<Vec<stats::EquityPoint>>, ApiError> {
    let trades = load(&s, user.id, &q).await?;
    let cash = q.starting_cash.unwrap_or(Decimal::ZERO);
    Ok(Json(stats::equity_curve(&trades, cash)))
}

async fn drawdown(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<RQ>,
) -> Result<Json<stats::MaxDrawdown>, ApiError> {
    let trades = load(&s, user.id, &q).await?;
    let cash = q.starting_cash.unwrap_or(Decimal::ZERO);
    let eq = stats::equity_curve(&trades, cash);
    Ok(Json(stats::max_drawdown(&eq)))
}

async fn risk_adjusted(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<RQ>,
) -> Result<Json<stats::RiskAdjusted>, ApiError> {
    let trades = load(&s, user.id, &q).await?;
    let cash = q.starting_cash.unwrap_or(Decimal::ZERO);
    let eq = stats::equity_curve(&trades, cash);
    Ok(Json(stats::risk_adjusted(&eq)))
}

async fn risk_report(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<RQ>,
) -> Result<Json<risk::RiskSummary>, ApiError> {
    Ok(Json(risk::risk_summary(
        load(&s, user.id, &q).await?.iter(),
    )))
}

#[derive(Deserialize)]
struct LiquidityQ {
    account_id: Uuid,
    /// Optional `symbol1:1000000,symbol2:500000` ADV overrides.
    #[serde(default)]
    adv: Option<String>,
}

#[derive(Serialize)]
struct LiquidityResponse {
    report: liquidity::LiquidityReport,
}

async fn liquidity_report(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<LiquidityQ>,
) -> Result<Json<LiquidityResponse>, ApiError> {
    ensure_account_owner(&s, user.id, q.account_id).await?;
    let f = TradeFilter {
        limit: Some(100_000),
        ..Default::default()
    };
    let trades = traderview_db::trades::list_for_account(&s.pool, q.account_id, &f)
        .await
        .map_err(ApiError::Internal)?;
    let mut adv: HashMap<String, Decimal> = HashMap::new();
    if let Some(s) = q.adv {
        for part in s.split(',') {
            if let Some((sym, v)) = part.split_once(':') {
                if let Ok(d) = v.parse::<Decimal>() {
                    adv.insert(sym.trim().to_string(), d);
                }
            }
        }
    }
    Ok(Json(LiquidityResponse {
        report: liquidity::liquidity(&trades, &adv),
    }))
}
