use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::routes::helpers::ensure_account_owner;
use crate::state::AppState;
use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use traderview_core::{liquidity, risk, stats, AssetClass, Trade, TradeSide};
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
        .route("/reports/by-price", get(by_price))
        .route("/reports/by-tag", get(by_tag))
        .route("/reports/by-duration-coarse", get(by_duration_coarse))
        .route("/reports/by-r-bucket", get(by_r_bucket))
        .route("/reports/by-opening-gap", get(by_opening_gap))
        .route("/reports/by-instrument-volume", get(by_instrument_volume))
        .route("/reports/by-movement", get(by_movement))
        .route("/reports/daily-series", get(daily_series))
        .route("/reports/win-loss-days", get(win_loss_days))
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
        .route("/reports/advanced", get(advanced))
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
    #[serde(default)]
    symbol: Option<String>,
    /// "long" | "short"
    #[serde(default)]
    side: Option<String>,
    /// "stock" | "option" | "future" | "forex"
    #[serde(default)]
    asset_class: Option<String>,
    /// "intraday" | "multiday" — applies to closed trades; hold time computed
    /// from opened_at/closed_at.
    #[serde(default)]
    duration: Option<String>,
    #[serde(default)]
    date_from: Option<NaiveDate>,
    #[serde(default)]
    date_to: Option<NaiveDate>,
    #[serde(default)]
    tag_id: Option<Uuid>,
}

fn parse_side(s: &str) -> Option<TradeSide> {
    match s.to_ascii_lowercase().as_str() {
        "long" => Some(TradeSide::Long),
        "short" => Some(TradeSide::Short),
        _ => None,
    }
}

fn parse_asset(s: &str) -> Option<AssetClass> {
    match s.to_ascii_lowercase().as_str() {
        "stock" => Some(AssetClass::Stock),
        "option" => Some(AssetClass::Option),
        "future" => Some(AssetClass::Future),
        "forex" => Some(AssetClass::Forex),
        _ => None,
    }
}

async fn load(s: &AppState, user_id: Uuid, q: &RQ) -> Result<Vec<Trade>, ApiError> {
    ensure_account_owner(s, user_id, q.account_id).await?;
    let f = TradeFilter {
        symbol: q.symbol.clone().filter(|x| !x.is_empty()),
        side: q.side.as_deref().and_then(parse_side),
        asset_class: q.asset_class.as_deref().and_then(parse_asset),
        date_from: q.date_from,
        date_to: q.date_to,
        tag_id: q.tag_id,
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
    if let Some(dur) = q.duration.as_deref() {
        let dur = dur.to_ascii_lowercase();
        // Intraday = open + close on the same calendar day (UTC). Multiday = different.
        trades.retain(|t| {
            let Some(close) = t.closed_at else {
                return dur != "intraday" && dur != "multiday";
            };
            let same = close.date_naive() == t.opened_at.date_naive();
            match dur.as_str() {
                "intraday" => same,
                "multiday" => !same,
                _ => true,
            }
        });
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
    Ok(Json(stats::by_symbol(&load(&s, user.id, &q).await?)))
}
async fn by_side(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<RQ>,
) -> Result<Json<Vec<stats::Bucket>>, ApiError> {
    Ok(Json(stats::by_side(&load(&s, user.id, &q).await?)))
}
async fn by_asset_class(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<RQ>,
) -> Result<Json<Vec<stats::Bucket>>, ApiError> {
    Ok(Json(stats::by_asset_class(&load(&s, user.id, &q).await?)))
}
async fn by_dow(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<RQ>,
) -> Result<Json<Vec<stats::Bucket>>, ApiError> {
    Ok(Json(stats::by_day_of_week(&load(&s, user.id, &q).await?)))
}
async fn by_hour(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<RQ>,
) -> Result<Json<Vec<stats::Bucket>>, ApiError> {
    Ok(Json(stats::by_hour_of_day(&load(&s, user.id, &q).await?)))
}
async fn by_hold(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<RQ>,
) -> Result<Json<Vec<stats::Bucket>>, ApiError> {
    Ok(Json(stats::by_hold_bucket(&load(&s, user.id, &q).await?)))
}
async fn by_month(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<RQ>,
) -> Result<Json<Vec<stats::Bucket>>, ApiError> {
    Ok(Json(stats::by_month(&load(&s, user.id, &q).await?)))
}
async fn by_price(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<RQ>,
) -> Result<Json<Vec<stats::Bucket>>, ApiError> {
    Ok(Json(stats::by_price_bucket(&load(&s, user.id, &q).await?)))
}
async fn by_duration_coarse(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<RQ>,
) -> Result<Json<Vec<stats::Bucket>>, ApiError> {
    Ok(Json(stats::by_duration_coarse(
        &load(&s, user.id, &q).await?,
    )))
}
async fn by_r_bucket(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<RQ>,
) -> Result<Json<Vec<stats::Bucket>>, ApiError> {
    Ok(Json(stats::by_r_bucket(&load(&s, user.id, &q).await?)))
}

/// Look up prior-day closes for each trade from `price_bars` (1d interval).
/// Trades whose symbol has no cached prior daily bar are excluded from
/// the bucketing. Cache-only — does NOT trigger Yahoo fetches; if the
/// price bar table is empty the response is an empty bucket set.
async fn by_opening_gap(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<RQ>,
) -> Result<Json<Vec<stats::Bucket>>, ApiError> {
    let trades = load(&s, user.id, &q).await?;
    type Row = (Uuid, Option<Decimal>);
    let mut prior_by_trade: HashMap<Uuid, Decimal> = HashMap::new();
    for t in &trades {
        let row: Option<Row> = sqlx::query_as(
            "SELECT $1::uuid, (
                SELECT close FROM price_bars
                WHERE symbol = $2 AND interval = '1d'::bar_interval_t
                  AND bar_time < $3
                ORDER BY bar_time DESC LIMIT 1
            )",
        )
        .bind(t.id)
        .bind(&t.symbol)
        .bind(t.opened_at)
        .fetch_optional(&s.pool)
        .await
        .map_err(|e| ApiError::Internal(e.into()))?;
        if let Some((id, Some(prior))) = row {
            prior_by_trade.insert(id, prior);
        }
    }
    Ok(Json(stats::by_opening_gap(&trades, &prior_by_trade)))
}

/// Average daily volume per unique symbol, computed over the most recent
/// 60 cached daily bars. Cache-only.
async fn by_instrument_volume(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<RQ>,
) -> Result<Json<Vec<stats::Bucket>>, ApiError> {
    let trades = load(&s, user.id, &q).await?;
    let symbols: Vec<String> = trades
        .iter()
        .map(|t| t.symbol.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    type Row = (String, Option<Decimal>);
    let rows: Vec<Row> = sqlx::query_as(
        "WITH recent AS (
            SELECT symbol, volume,
                   ROW_NUMBER() OVER (PARTITION BY symbol ORDER BY bar_time DESC) AS rn
              FROM price_bars
             WHERE interval = '1d'::bar_interval_t
               AND symbol = ANY($1)
         )
         SELECT symbol, AVG(volume)::numeric
           FROM recent
          WHERE rn <= 60
          GROUP BY symbol",
    )
    .bind(&symbols)
    .fetch_all(&s.pool)
    .await
    .map_err(|e| ApiError::Internal(e.into()))?;
    let adv: HashMap<String, Decimal> = rows
        .into_iter()
        .filter_map(|(sym, v)| v.map(|d| (sym, d)))
        .collect();
    Ok(Json(stats::by_instrument_volume(&trades, &adv)))
}

/// Average intraday range as a percent of close per unique symbol, over
/// the most recent 60 cached daily bars. Cache-only.
async fn by_movement(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<RQ>,
) -> Result<Json<Vec<stats::Bucket>>, ApiError> {
    let trades = load(&s, user.id, &q).await?;
    let symbols: Vec<String> = trades
        .iter()
        .map(|t| t.symbol.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    type Row = (String, Option<f64>);
    let rows: Vec<Row> = sqlx::query_as(
        "WITH recent AS (
            SELECT symbol, high, low, close,
                   ROW_NUMBER() OVER (PARTITION BY symbol ORDER BY bar_time DESC) AS rn
              FROM price_bars
             WHERE interval = '1d'::bar_interval_t
               AND symbol = ANY($1)
         )
         SELECT symbol, AVG(((high - low) / NULLIF(close, 0)) * 100)::float8
           FROM recent
          WHERE rn <= 60
          GROUP BY symbol",
    )
    .bind(&symbols)
    .fetch_all(&s.pool)
    .await
    .map_err(|e| ApiError::Internal(e.into()))?;
    let range_pct: HashMap<String, f64> = rows
        .into_iter()
        .filter_map(|(sym, v)| v.map(|x| (sym, x)))
        .collect();
    Ok(Json(stats::by_movement(&trades, &range_pct)))
}
async fn daily_series(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<RQ>,
) -> Result<Json<Vec<stats::DailySeriesPoint>>, ApiError> {
    Ok(Json(stats::daily_series(&load(&s, user.id, &q).await?)))
}
async fn win_loss_days(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<RQ>,
) -> Result<Json<stats::WinLossDays>, ApiError> {
    Ok(Json(stats::win_loss_days(&load(&s, user.id, &q).await?)))
}

async fn r_distribution(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<RQ>,
) -> Result<Json<stats::RMultipleDistribution>, ApiError> {
    Ok(Json(stats::r_distribution(&load(&s, user.id, &q).await?)))
}
async fn streaks(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<RQ>,
) -> Result<Json<Vec<stats::Streak>>, ApiError> {
    Ok(Json(stats::streaks(&load(&s, user.id, &q).await?)))
}
async fn comparison(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<RQ>,
) -> Result<Json<stats::Comparison>, ApiError> {
    Ok(Json(stats::comparison(&load(&s, user.id, &q).await?)))
}
async fn exit_eff(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<RQ>,
) -> Result<Json<stats::ExitEfficiency>, ApiError> {
    Ok(Json(stats::exit_efficiency(&load(&s, user.id, &q).await?)))
}
async fn commissions(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<RQ>,
) -> Result<Json<stats::CommissionReport>, ApiError> {
    Ok(Json(stats::commissions(&load(&s, user.id, &q).await?)))
}

async fn by_tag(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<RQ>,
) -> Result<Json<Vec<stats::Bucket>>, ApiError> {
    let trades = load(&s, user.id, &q).await?;
    // Fetch tags-for-each-trade in a single query, then bucket client-side
    // since the trade-list is already in memory.
    let mut map: std::collections::HashMap<Uuid, Vec<String>> = std::collections::HashMap::new();
    if !trades.is_empty() {
        let ids: Vec<Uuid> = trades.iter().map(|t| t.id).collect();
        let rows: Vec<(Uuid, String)> = sqlx::query_as(
            "SELECT tt.trade_id, t.name
               FROM trade_tags tt JOIN tags t ON t.id = tt.tag_id
              WHERE tt.trade_id = ANY($1)",
        )
        .bind(&ids)
        .fetch_all(&s.pool)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!(e)))?;
        for (tid, name) in rows {
            map.entry(tid).or_default().push(name);
        }
    }
    Ok(Json(stats::by_tag(&trades, &map)))
}

async fn advanced(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<RQ>,
) -> Result<Json<stats::Advanced>, ApiError> {
    let trades = load(&s, user.id, &q).await?;
    let cash = q.starting_cash.unwrap_or(Decimal::ZERO);
    Ok(Json(stats::advanced(&trades, cash)))
}

async fn calendar(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<RQ>,
) -> Result<Json<Vec<stats::CalendarCell>>, ApiError> {
    Ok(Json(stats::calendar(&load(&s, user.id, &q).await?)))
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
