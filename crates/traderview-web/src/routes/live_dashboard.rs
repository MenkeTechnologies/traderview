//! Live Dashboard — fetches the user's broker state DIRECTLY from
//! Alpaca REST and returns the union of (account, positions, orders)
//! in a single call. Does NOT depend on the local `executions` /
//! `trades` pipeline — this is the canonical "what does my broker
//! actually say" surface.
//!
//! Endpoint: `GET /api/live/dashboard?account_id=<uuid>`
//!
//! Sourcing:
//!   - account snapshot     → GET /v2/account
//!   - open positions       → GET /v2/positions
//!   - recent orders (all)  → GET /v2/orders?status=all&limit=100&direction=desc
//!
//! All three are parallel-fetched.  Returns 502 if any call to Alpaca
//! fails.  Credentials come from `data_source_keys::alpaca_creds_plain`
//! (per-user `user_settings` row, env var fallback).
//!
//! Today this is Alpaca-only.  Adding Tradier / IBKR is a follow-up:
//! pick by `accounts.broker` and dispatch to the right client.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::routes::helpers::ensure_account_owner;
use crate::state::AppState;
use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use traderview_db::alpaca_trading::{
    AccountResponse, AlpacaTrading, BrokerMode, OrderResponse, PortfolioHistory, PositionResponse,
};
use traderview_db::data_source_keys;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new().route("/live/dashboard", get(snapshot))
}

#[derive(Deserialize)]
struct LiveQuery {
    account_id: Uuid,
    /// Equity-history window. Accepts Alpaca codes: 1D, 1W, 1M, 3M, 1A, all.
    /// Defaults to "1M". Timeframe is auto-picked: intraday for 1D, daily otherwise.
    #[serde(default)]
    window: Option<String>,
}

#[derive(Serialize)]
struct LiveDashboard {
    account_id: Uuid,
    broker: &'static str,
    mode: &'static str,
    fetched_at: DateTime<Utc>,
    account: Option<AccountResponse>,
    positions: Vec<PositionResponse>,
    orders: Vec<OrderResponse>,
    /// Equity-curve series (`1M` window, `1D` timeframe). `None` if the
    /// portfolio_history fetch failed — frontend renders the rest anyway.
    history: Option<PortfolioHistory>,
    position_count: usize,
    order_count: usize,
    /// Sum of |market_value| across positions (long + short magnitudes).
    total_market_value: f64,
    /// Sum of unrealized_pl across positions.
    total_unrealized_pl: f64,
    /// True if the user's account row is on Alpaca AND we found creds.
    /// False values mean: account belongs to a non-Alpaca broker, or
    /// credentials are missing — caller should surface a setup banner.
    connected: bool,
    /// Human-readable error if `connected` is false.
    note: Option<String>,
}

async fn snapshot(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<LiveQuery>,
) -> Result<Json<LiveDashboard>, ApiError> {
    ensure_account_owner(&s, user.id, q.account_id).await?;

    // Resolve the account's broker. v1: Alpaca only — anything else
    // returns connected=false with a note rather than 4xx.
    let row: Option<(String,)> = sqlx::query_as("SELECT broker FROM accounts WHERE id = $1")
        .bind(q.account_id)
        .fetch_optional(&s.pool)
        .await
        .map_err(|e| ApiError::Internal(e.into()))?;
    let broker = row.map(|(b,)| b).unwrap_or_default();
    if !broker.eq_ignore_ascii_case("alpaca") {
        return Ok(Json(LiveDashboard {
            account_id: q.account_id,
            broker: "alpaca",
            mode: "paper",
            fetched_at: Utc::now(),
            account: None,
            positions: vec![],
            orders: vec![],
            history: None,
            position_count: 0,
            order_count: 0,
            total_market_value: 0.0,
            total_unrealized_pl: 0.0,
            connected: false,
            note: Some(format!(
                "Live Dashboard currently supports Alpaca only (account broker = '{broker}')."
            )),
        }));
    }

    let creds = data_source_keys::alpaca_creds_plain(&s.pool, user.id)
        .await
        .map_err(ApiError::Internal)?;
    let Some((key_id, secret, paper)) = creds else {
        return Ok(Json(LiveDashboard {
            account_id: q.account_id,
            broker: "alpaca",
            mode: "paper",
            fetched_at: Utc::now(),
            account: None,
            positions: vec![],
            orders: vec![],
            history: None,
            position_count: 0,
            order_count: 0,
            total_market_value: 0.0,
            total_unrealized_pl: 0.0,
            connected: false,
            note: Some(
                "No Alpaca credentials configured. Add them in Settings → Brokers → Alpaca."
                    .to_string(),
            ),
        }));
    };
    let mode = if paper { BrokerMode::Paper } else { BrokerMode::Live };
    let client = AlpacaTrading::new(mode, key_id, secret);

    // Pick the equity-history window. Default to 1M; for 1D use intraday
    // bars (5Min) so the curve isn't a single point.
    let window = q.window.as_deref().unwrap_or("1M");
    let (hist_period, hist_timeframe) = match window {
        "1D" => ("1D", "5Min"),
        "1W" => ("1W", "1H"),
        "3M" => ("3M", "1D"),
        "1A" => ("1A", "1D"),
        "all" => ("all", "1D"),
        _ => ("1M", "1D"),
    };

    // Parallel fetch. If any leg fails, surface a structured note rather than
    // 500'ing — the other legs are still useful, and the UI can render partial.
    let (acct_r, pos_r, ord_r, hist_r) = tokio::join!(
        client.get_account(),
        client.list_positions(),
        client.list_orders("all", 100),
        client.get_portfolio_history(hist_period, hist_timeframe),
    );

    let mut note: Option<String> = None;
    let account = match acct_r {
        Ok(a) => Some(a),
        Err(e) => {
            note = Some(format!("account fetch failed: {e}"));
            None
        }
    };
    let positions = match pos_r {
        Ok(p) => p,
        Err(e) => {
            if note.is_none() {
                note = Some(format!("positions fetch failed: {e}"));
            }
            vec![]
        }
    };
    let orders = match ord_r {
        Ok(o) => o,
        Err(e) => {
            if note.is_none() {
                note = Some(format!("orders fetch failed: {e}"));
            }
            vec![]
        }
    };
    let history = match hist_r {
        Ok(h) => Some(h),
        Err(e) => {
            tracing::warn!(error = %e, "portfolio_history fetch failed");
            None
        }
    };

    let total_market_value: f64 = positions
        .iter()
        .filter_map(|p| p.market_value.and_then(|d| d.to_string().parse::<f64>().ok()))
        .map(f64::abs)
        .sum();
    let total_unrealized_pl: f64 = positions
        .iter()
        .filter_map(|p| p.unrealized_pl.and_then(|d| d.to_string().parse::<f64>().ok()))
        .sum();

    Ok(Json(LiveDashboard {
        account_id: q.account_id,
        broker: "alpaca",
        mode: if paper { "paper" } else { "live" },
        fetched_at: Utc::now(),
        position_count: positions.len(),
        order_count: orders.len(),
        account,
        positions,
        orders,
        history,
        total_market_value,
        total_unrealized_pl,
        connected: true,
        note,
    }))
}
