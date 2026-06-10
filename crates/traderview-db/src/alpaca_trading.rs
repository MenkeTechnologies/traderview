//! Alpaca Trading API client — REST orders/positions/account + WebSocket
//! trade_updates stream.
//!
//! Separate from `live_ticks.rs` (market-data feed). This module is the
//! BROKER side: places orders, mirrors fills back into `algo_orders` /
//! `algo_fills`. Auth is via `APCA-API-KEY-ID` + `APCA-API-SECRET-KEY`
//! headers (REST) or `{action:"authenticate", key, secret}` (WS).
//!
//! Base URLs:
//!   paper: <https://paper-api.alpaca.markets>
//!   live:  <https://api.alpaca.markets>
//!   ws paper: <wss://paper-api.alpaca.markets/stream>
//!   ws live:  <wss://api.alpaca.markets/stream>
//!
//! Live trading itself is free — the only paid Alpaca subscription is
//! Algo Trader Plus (\$99/mo) for SIP market data.

use chrono::{DateTime, Utc};
use futures_util::{SinkExt, StreamExt};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;
use std::time::Duration;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use uuid::Uuid;

pub const PAPER_BASE_URL: &str = "https://paper-api.alpaca.markets";
pub const LIVE_BASE_URL: &str = "https://api.alpaca.markets";
pub const PAPER_WS_URL: &str = "wss://paper-api.alpaca.markets/stream";
pub const LIVE_WS_URL: &str = "wss://api.alpaca.markets/stream";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrokerMode {
    Paper,
    Live,
}

impl BrokerMode {
    pub fn from_str_lossy(s: &str) -> Self {
        if s == "alpaca_live" {
            Self::Live
        } else {
            Self::Paper
        }
    }
    fn rest_base(self) -> &'static str {
        match self {
            Self::Paper => PAPER_BASE_URL,
            Self::Live => LIVE_BASE_URL,
        }
    }
    fn ws_url(self) -> &'static str {
        match self {
            Self::Paper => PAPER_WS_URL,
            Self::Live => LIVE_WS_URL,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AlpacaError {
    #[error("alpaca http error {status}: {body}")]
    Http { status: u16, body: String },
    #[error("alpaca insufficient buying power")]
    InsufficientBuyingPower,
    #[error("alpaca invalid request: {0}")]
    InvalidRequest(String),
    #[error("alpaca auth failed")]
    AuthFailed,
    #[error("transport: {0}")]
    Transport(#[from] reqwest::Error),
    #[error("ws: {0}")]
    Ws(#[from] tokio_tungstenite::tungstenite::Error),
    #[error("decode: {0}")]
    Decode(#[from] serde_json::Error),
}

/// Configuration for a single Alpaca account. Clone is cheap; share
/// across the engine + WS listener so they hit the same base URL.
#[derive(Debug, Clone)]
pub struct AlpacaTrading {
    http: reqwest::Client,
    rest_base: String,
    ws_url: String,
    key_id: String,
    secret: String,
}

impl AlpacaTrading {
    pub fn new(mode: BrokerMode, key_id: impl Into<String>, secret: impl Into<String>) -> Self {
        Self {
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .expect("reqwest client"),
            rest_base: mode.rest_base().to_string(),
            ws_url: mode.ws_url().to_string(),
            key_id: key_id.into(),
            secret: secret.into(),
        }
    }

    /// Test-only constructor that overrides the REST base URL (for wiremock).
    /// WS URL stays unset — tests that need WS construct it separately.
    pub fn with_rest_base(
        rest_base: impl Into<String>,
        key_id: impl Into<String>,
        secret: impl Into<String>,
    ) -> Self {
        Self {
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(5))
                .build()
                .expect("reqwest client"),
            rest_base: rest_base.into(),
            ws_url: String::new(),
            key_id: key_id.into(),
            secret: secret.into(),
        }
    }

    fn auth_headers(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        req.header("APCA-API-KEY-ID", &self.key_id)
            .header("APCA-API-SECRET-KEY", &self.secret)
    }

    async fn handle_status<T: for<'de> Deserialize<'de>>(
        resp: reqwest::Response,
    ) -> Result<T, AlpacaError> {
        let status = resp.status();
        let body = resp.text().await?;
        if status.is_success() {
            return Ok(serde_json::from_str(&body)?);
        }
        match status.as_u16() {
            401 | 403 => {
                // Alpaca returns 403 for both auth failure AND
                // insufficient buying power; the body's `message`
                // disambiguates.
                if body.to_ascii_lowercase().contains("buying power")
                    || body.to_ascii_lowercase().contains("insufficient")
                {
                    Err(AlpacaError::InsufficientBuyingPower)
                } else {
                    Err(AlpacaError::AuthFailed)
                }
            }
            422 => Err(AlpacaError::InvalidRequest(body)),
            _ => Err(AlpacaError::Http {
                status: status.as_u16(),
                body,
            }),
        }
    }

    // ─── orders ────────────────────────────────────────────────────────

    pub async fn place_order(&self, req: &PlaceOrderRequest) -> Result<OrderResponse, AlpacaError> {
        let url = format!("{}/v2/orders", self.rest_base);
        let resp = self
            .auth_headers(self.http.post(&url))
            .json(req)
            .send()
            .await?;
        Self::handle_status(resp).await
    }

    /// Alpaca bulk endpoint — cancels every open order on the account.
    /// Returns the JSON body verbatim (server-shaped); empty Vec on
    /// success when there's nothing open. Some terminal states come
    /// back as 207 Multi-Status — caller treats success-or-partial as
    /// "fine" since we'll re-list afterward to surface what's still
    /// outstanding.
    pub async fn cancel_all_orders(&self) -> Result<(), AlpacaError> {
        let url = format!("{}/v2/orders", self.rest_base);
        let resp = self.auth_headers(self.http.delete(&url)).send().await?;
        let status = resp.status();
        if status.is_success() || status.as_u16() == 207 {
            return Ok(());
        }
        let body = resp.text().await?;
        Err(AlpacaError::Http {
            status: status.as_u16(),
            body,
        })
    }

    /// Alpaca bulk endpoint — issues a flat-everything close. Pass
    /// `cancel_orders = true` to also cancel open orders in the same
    /// call (recommended for kill-switch).
    pub async fn close_all_positions(&self, cancel_orders: bool) -> Result<(), AlpacaError> {
        let url = format!(
            "{}/v2/positions?cancel_orders={}",
            self.rest_base, cancel_orders
        );
        let resp = self.auth_headers(self.http.delete(&url)).send().await?;
        let status = resp.status();
        if status.is_success() || status.as_u16() == 207 {
            return Ok(());
        }
        let body = resp.text().await?;
        Err(AlpacaError::Http {
            status: status.as_u16(),
            body,
        })
    }

    pub async fn cancel_order(&self, broker_order_id: &str) -> Result<(), AlpacaError> {
        let url = format!("{}/v2/orders/{}", self.rest_base, broker_order_id);
        let resp = self.auth_headers(self.http.delete(&url)).send().await?;
        let status = resp.status();
        // Alpaca returns 204 No Content on success; some terminal states
        // (already filled) come back as 422 — caller treats as "fine".
        if status.as_u16() == 204 || status.is_success() {
            return Ok(());
        }
        let body = resp.text().await?;
        if status.as_u16() == 422 {
            return Err(AlpacaError::InvalidRequest(body));
        }
        Err(AlpacaError::Http {
            status: status.as_u16(),
            body,
        })
    }

    /// List open orders (status=open). Used to count what the bulk
    /// `cancel_all_orders` / `close_all_positions(true)` call will act
    /// on, so the kill-switch result can report a precise number.
    pub async fn list_open_orders(&self) -> Result<Vec<OrderResponse>, AlpacaError> {
        let url = format!("{}/v2/orders?status=open", self.rest_base);
        let resp = self.auth_headers(self.http.get(&url)).send().await?;
        Self::handle_status(resp).await
    }

    /// All orders matching `status` ("open" / "closed" / "all"), most recent first.
    /// `limit` is hard-clamped to 500 (Alpaca's API max).
    pub async fn list_orders(
        &self,
        status: &str,
        limit: u32,
    ) -> Result<Vec<OrderResponse>, AlpacaError> {
        let lim = limit.min(500);
        let url = format!(
            "{}/v2/orders?status={}&limit={}&direction=desc",
            self.rest_base, status, lim
        );
        let resp = self.auth_headers(self.http.get(&url)).send().await?;
        Self::handle_status(resp).await
    }

    pub async fn get_order(&self, broker_order_id: &str) -> Result<OrderResponse, AlpacaError> {
        let url = format!("{}/v2/orders/{}", self.rest_base, broker_order_id);
        let resp = self.auth_headers(self.http.get(&url)).send().await?;
        Self::handle_status(resp).await
    }

    pub async fn get_order_by_client_id(
        &self,
        client_order_id: Uuid,
    ) -> Result<OrderResponse, AlpacaError> {
        let url = format!(
            "{}/v2/orders:by_client_order_id?client_order_id={}",
            self.rest_base, client_order_id
        );
        let resp = self.auth_headers(self.http.get(&url)).send().await?;
        Self::handle_status(resp).await
    }

    // ─── positions / account ───────────────────────────────────────────

    pub async fn list_positions(&self) -> Result<Vec<PositionResponse>, AlpacaError> {
        let url = format!("{}/v2/positions", self.rest_base);
        let resp = self.auth_headers(self.http.get(&url)).send().await?;
        Self::handle_status(resp).await
    }

    /// Historical portfolio equity series. `period` accepts Alpaca's
    /// codes: "1D", "1W", "1M", "3M", "1A", "all". `timeframe` accepts
    /// "1Min", "5Min", "15Min", "1H", "1D". Returns aligned arrays —
    /// timestamps (unix seconds), equity, profit/loss, p/l %.
    pub async fn get_portfolio_history(
        &self,
        period: &str,
        timeframe: &str,
    ) -> Result<PortfolioHistory, AlpacaError> {
        let url = format!(
            "{}/v2/account/portfolio/history?period={}&timeframe={}",
            self.rest_base, period, timeframe
        );
        let resp = self.auth_headers(self.http.get(&url)).send().await?;
        Self::handle_status(resp).await
    }

    pub async fn get_account(&self) -> Result<AccountResponse, AlpacaError> {
        let url = format!("{}/v2/account", self.rest_base);
        let resp = self.auth_headers(self.http.get(&url)).send().await?;
        Self::handle_status(resp).await
    }

    // ─── trade updates WebSocket ───────────────────────────────────────

    /// Connect to the trade_updates stream and pump events to `handler`
    /// until the stream closes or `handler` returns `Err`. Caller is
    /// expected to wrap this in a reconnect loop the same way
    /// `live_ticks.rs` does for market data.
    pub async fn trade_updates_stream<F, Fut>(&self, mut handler: F) -> Result<(), AlpacaError>
    where
        F: FnMut(TradeUpdateEvent) -> Fut + Send,
        Fut: std::future::Future<Output = Result<(), AlpacaError>> + Send,
    {
        let url = if self.ws_url.is_empty() {
            return Err(AlpacaError::InvalidRequest("ws_url not configured".into()));
        } else {
            self.ws_url.clone()
        };
        let (ws, _) = tokio_tungstenite::connect_async(&url).await?;
        let (mut tx, mut rx) = ws.split();

        // 1. Auth frame (note: WS uses "action":"authenticate", REST uses headers).
        let auth = serde_json::json!({
            "action": "authenticate",
            "key": self.key_id,
            "secret": self.secret,
        })
        .to_string();
        tx.send(WsMessage::Text(auth)).await?;

        // 2. Subscribe to trade_updates.
        let sub = serde_json::json!({
            "action": "listen",
            "data": {"streams": ["trade_updates"]},
        })
        .to_string();
        tx.send(WsMessage::Text(sub)).await?;

        // 3. Pump loop.
        while let Some(msg) = rx.next().await {
            let frame = match msg? {
                WsMessage::Text(t) => t,
                WsMessage::Binary(b) => String::from_utf8_lossy(&b).into_owned(),
                WsMessage::Close(_) => break,
                _ => continue,
            };
            let parsed: WsEnvelope = match serde_json::from_str(&frame) {
                Ok(v) => v,
                Err(_) => continue,
            };
            // Only the "trade_updates" stream produces fillable events.
            // Auth / listen acks come through as stream=="authorization"
            // / "listening" — log + skip.
            if parsed.stream.as_deref() == Some("authorization") {
                let status = parsed
                    .data
                    .as_ref()
                    .and_then(|d| d.get("status"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if status == "unauthorized" {
                    return Err(AlpacaError::AuthFailed);
                }
                continue;
            }
            if parsed.stream.as_deref() != Some("trade_updates") {
                continue;
            }
            let Some(data) = parsed.data else { continue };
            let event: TradeUpdateEvent = match serde_json::from_value(data) {
                Ok(v) => v,
                Err(e) => {
                    tracing::warn!(error = %e, "alpaca trade_updates decode failed");
                    continue;
                }
            };
            handler(event).await?;
        }
        Ok(())
    }
}

// ─── REST request/response DTOs ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct PlaceOrderRequest {
    pub symbol: String,
    /// Either `qty` or `notional` must be set. `qty` accepts fractionals
    /// for market+day TIF; everything else needs whole shares.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qty: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notional: Option<Decimal>,
    pub side: String, // "buy" | "sell"
    #[serde(rename = "type")]
    pub order_type: String, // "market" | "limit" | "stop" | "stop_limit" | "trailing_stop"
    pub time_in_force: String, // "day" | "gtc" | "ioc" | "fok" | "opg" | "cls"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_price: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_price: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trail_price: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trail_percent: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extended_hours: Option<bool>,
    pub client_order_id: Uuid,
    /// "simple" (default) | "bracket" | "oco" | "oto"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_class: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub take_profit: Option<TakeProfitLeg>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_loss: Option<StopLossLeg>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TakeProfitLeg {
    pub limit_price: Decimal,
}

#[derive(Debug, Clone, Serialize)]
pub struct StopLossLeg {
    pub stop_price: Decimal,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_price: Option<Decimal>,
}

impl PlaceOrderRequest {
    /// Convenience: simple market order, day TIF.
    pub fn market(symbol: impl Into<String>, side: &str, qty: Decimal, coid: Uuid) -> Self {
        Self {
            symbol: symbol.into(),
            qty: Some(qty),
            notional: None,
            side: side.into(),
            order_type: "market".into(),
            time_in_force: "day".into(),
            limit_price: None,
            stop_price: None,
            trail_price: None,
            trail_percent: None,
            extended_hours: None,
            client_order_id: coid,
            order_class: None,
            take_profit: None,
            stop_loss: None,
        }
    }

    /// Extended-hours LIMIT entry — pre-market (4:00-9:30 ET) or
    /// after-hours (16:00-20:00 ET). Alpaca's extended-hours rules:
    ///   * `order_type` MUST be `limit` (market rejected)
    ///   * `time_in_force` MUST be `day`
    ///   * `extended_hours: true`
    ///   * `order_class` MUST be omitted (no bracket/OCO/OTO)
    ///
    /// So the engine submits entry-only; the user manages exit
    /// manually OR the risk circuit breakers / metrics dashboard
    /// surface the open position for human review the next session.
    pub fn extended_hours_limit(
        symbol: impl Into<String>,
        side: &str,
        qty: Decimal,
        coid: Uuid,
        limit_price: Decimal,
    ) -> Self {
        Self {
            symbol: symbol.into(),
            qty: Some(qty),
            notional: None,
            side: side.into(),
            order_type: "limit".into(),
            time_in_force: "day".into(),
            limit_price: Some(limit_price),
            stop_price: None,
            trail_price: None,
            trail_percent: None,
            extended_hours: Some(true),
            client_order_id: coid,
            order_class: None,
            take_profit: None,
            stop_loss: None,
        }
    }

    /// Crypto simple-market — Alpaca's crypto venue uses the same
    /// /v2/orders endpoint as equities BUT rejects bracket / OCO /
    /// OTO order classes (no native bracket on crypto). Symbol must
    /// be `BASE/QUOTE` (e.g. `BTC/USD`); fractional qty allowed.
    /// Exit logic stays in the strategy (evaluate_exit) or the user
    /// manages it manually.
    pub fn crypto_market(symbol: impl Into<String>, side: &str, qty: Decimal, coid: Uuid) -> Self {
        Self {
            symbol: symbol.into(),
            qty: Some(qty),
            notional: None,
            side: side.into(),
            order_type: "market".into(),
            // Crypto markets 24/7 — gtc lets the order persist past
            // a session close without re-submitting.
            time_in_force: "gtc".into(),
            limit_price: None,
            stop_price: None,
            trail_price: None,
            trail_percent: None,
            extended_hours: None,
            client_order_id: coid,
            order_class: None,
            take_profit: None,
            stop_loss: None,
        }
    }

    /// Convenience: native bracket with market entry + take_profit + stop_loss.
    pub fn bracket_market(
        symbol: impl Into<String>,
        side: &str,
        qty: Decimal,
        coid: Uuid,
        take_profit: Decimal,
        stop_loss: Decimal,
    ) -> Self {
        Self {
            symbol: symbol.into(),
            qty: Some(qty),
            notional: None,
            side: side.into(),
            order_type: "market".into(),
            time_in_force: "day".into(),
            limit_price: None,
            stop_price: None,
            trail_price: None,
            trail_percent: None,
            extended_hours: None,
            client_order_id: coid,
            order_class: Some("bracket".into()),
            take_profit: Some(TakeProfitLeg {
                limit_price: take_profit,
            }),
            stop_loss: Some(StopLossLeg {
                stop_price: stop_loss,
                limit_price: None,
            }),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OrderResponse {
    pub id: String,
    pub client_order_id: String,
    pub status: String,
    pub symbol: String,
    pub side: String,
    #[serde(rename = "type")]
    pub order_type: String,
    pub qty: Option<Decimal>,
    pub filled_qty: Option<Decimal>,
    pub filled_avg_price: Option<Decimal>,
    pub limit_price: Option<Decimal>,
    pub stop_price: Option<Decimal>,
    pub time_in_force: String,
    pub order_class: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PositionResponse {
    pub symbol: String,
    pub qty: Decimal,
    pub side: String,
    pub avg_entry_price: Decimal,
    pub market_value: Option<Decimal>,
    pub cost_basis: Option<Decimal>,
    pub unrealized_pl: Option<Decimal>,
    pub current_price: Option<Decimal>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AccountResponse {
    pub id: String,
    pub status: String,
    pub cash: Decimal,
    pub equity: Decimal,
    pub buying_power: Decimal,
    pub portfolio_value: Decimal,
    pub daytrade_count: Option<i64>,
    pub pattern_day_trader: Option<bool>,
}

/// Alpaca's `/v2/account/portfolio/history` payload. All arrays are
/// aligned by index. Equity is in account currency. `timestamp` is
/// seconds since epoch. `base_value` is the equity at the start of
/// the requested window — useful for relative P/L if the absolute
/// equity series is short or sparse.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PortfolioHistory {
    pub timestamp: Vec<i64>,
    pub equity: Vec<Option<f64>>,
    pub profit_loss: Vec<Option<f64>>,
    pub profit_loss_pct: Vec<Option<f64>>,
    pub base_value: Option<f64>,
    pub timeframe: String,
}

// ─── WebSocket envelope + trade update payload ──────────────────────────────

#[derive(Debug, Deserialize)]
struct WsEnvelope {
    stream: Option<String>,
    data: Option<Json>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TradeUpdateEvent {
    /// "new" | "fill" | "partial_fill" | "canceled" | "expired" |
    /// "done_for_day" | "replaced" | "rejected" | "pending_cancel" |
    /// "pending_replace" | "stopped" | "suspended" | "calculated"
    pub event: String,
    /// Fill price for fill/partial_fill events.
    pub price: Option<Decimal>,
    /// Fill quantity for fill/partial_fill events.
    pub qty: Option<Decimal>,
    /// Position size after this event.
    pub position_qty: Option<Decimal>,
    /// Echoed order — `client_order_id` here is OUR Uuid string.
    pub order: OrderResponse,
    /// ISO 8601 timestamp of the event itself.
    pub timestamp: Option<DateTime<Utc>>,
}
