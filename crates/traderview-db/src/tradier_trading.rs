//! Tradier brokerage REST client — equity + OTOCO (bracket) orders.
//!
//! API reference (verified):
//!   Production: <https://api.tradier.com/v1>
//!   Sandbox:    <https://sandbox.tradier.com/v1>
//!   Streaming:  <https://stream.tradier.com/v1>  (HTTP streaming, not raw WS)
//!   Auth:       `Authorization: Bearer <token>` header
//!   Place:      POST /v1/accounts/{account_id}/orders
//!               Content-Type: application/x-www-form-urlencoded
//!               Required form fields: class, symbol, side, quantity, type, duration
//!   Cancel:     DELETE /v1/accounts/{account_id}/orders/{order_id}
//!   Balances:   GET /v1/accounts/{account_id}/balances
//!   Positions:  GET /v1/accounts/{account_id}/positions
//!
//! Sources:
//!   <https://docs.tradier.com/docs/endpoints>
//!   <https://documentation.tradier.com/brokerage-api/trading/place-equity-order>
//!
//! Bracket-equivalent: Tradier's OTOCO ("one-triggers-OCO") order class
//! places an entry order that on fill triggers a pair of opposing
//! take_profit + stop_loss exits. See `OtocoLeg` for the form-array
//! encoding the API requires.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub const SANDBOX_BASE: &str = "https://sandbox.tradier.com/v1";
pub const LIVE_BASE: &str = "https://api.tradier.com/v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TradierEnv {
    Sandbox,
    Live,
}

impl TradierEnv {
    pub fn from_broker_mode(mode: &str) -> Self {
        match mode {
            "live" => Self::Live,
            _ => Self::Sandbox,
        }
    }
    fn base(self) -> &'static str {
        match self {
            Self::Sandbox => SANDBOX_BASE,
            Self::Live => LIVE_BASE,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TradierError {
    #[error("tradier http {status}: {body}")]
    Http { status: u16, body: String },
    #[error("tradier auth failed")]
    AuthFailed,
    #[error("tradier insufficient buying power")]
    InsufficientBuyingPower,
    #[error("tradier invalid request: {0}")]
    InvalidRequest(String),
    #[error("transport: {0}")]
    Transport(#[from] reqwest::Error),
    #[error("decode: {0}")]
    Decode(#[from] serde_json::Error),
}

#[derive(Debug, Clone)]
pub struct TradierTrading {
    http: reqwest::Client,
    base: String,
    token: String,
    account_id: String,
}

impl TradierTrading {
    pub fn new(env: TradierEnv, token: impl Into<String>, account_id: impl Into<String>) -> Self {
        Self {
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .expect("reqwest client"),
            base: env.base().to_string(),
            token: token.into(),
            account_id: account_id.into(),
        }
    }

    /// Test-only constructor — wiremock injects a mock base URL.
    pub fn with_base(base: impl Into<String>, token: impl Into<String>, account_id: impl Into<String>) -> Self {
        Self {
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(5))
                .build()
                .expect("reqwest client"),
            base: base.into(),
            token: token.into(),
            account_id: account_id.into(),
        }
    }

    fn auth(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        req.header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/json")
    }

    async fn handle_status<T: for<'de> Deserialize<'de>>(
        resp: reqwest::Response,
    ) -> Result<T, TradierError> {
        let status = resp.status();
        let body = resp.text().await?;
        if status.is_success() {
            return Ok(serde_json::from_str(&body)?);
        }
        match status.as_u16() {
            401 => Err(TradierError::AuthFailed),
            403 if body.to_ascii_lowercase().contains("buying power") => {
                Err(TradierError::InsufficientBuyingPower)
            }
            400 | 422 => Err(TradierError::InvalidRequest(body)),
            _ => Err(TradierError::Http { status: status.as_u16(), body }),
        }
    }

    /// Place a simple equity market or limit order.
    pub async fn place_equity_order(
        &self,
        req: &PlaceEquityOrder,
    ) -> Result<PlaceOrderResponse, TradierError> {
        let url = format!("{}/accounts/{}/orders", self.base, self.account_id);
        let resp = self
            .auth(self.http.post(&url))
            .form(&req.to_form())
            .send()
            .await?;
        Self::handle_status::<PlaceOrderEnvelope>(resp)
            .await
            .map(|e| e.order)
    }

    /// Place an OTOCO bracket: entry + take_profit + stop_loss using
    /// Tradier's native multi-leg class. The exit pair becomes an OCO
    /// after the entry fills, mirroring Alpaca's `bracket` semantics.
    pub async fn place_otoco_bracket(
        &self,
        bracket: &OtocoBracket,
    ) -> Result<PlaceOrderResponse, TradierError> {
        let url = format!("{}/accounts/{}/orders", self.base, self.account_id);
        let resp = self
            .auth(self.http.post(&url))
            .form(&bracket.to_form())
            .send()
            .await?;
        Self::handle_status::<PlaceOrderEnvelope>(resp)
            .await
            .map(|e| e.order)
    }

    pub async fn cancel_order(&self, order_id: i64) -> Result<(), TradierError> {
        let url = format!(
            "{}/accounts/{}/orders/{order_id}",
            self.base, self.account_id
        );
        let resp = self.auth(self.http.delete(&url)).send().await?;
        let status = resp.status();
        let body = resp.text().await?;
        if status.is_success() {
            return Ok(());
        }
        match status.as_u16() {
            401 => Err(TradierError::AuthFailed),
            422 | 400 => Err(TradierError::InvalidRequest(body)),
            _ => Err(TradierError::Http { status: status.as_u16(), body }),
        }
    }

    pub async fn get_balances(&self) -> Result<BalancesResponse, TradierError> {
        let url = format!("{}/accounts/{}/balances", self.base, self.account_id);
        let resp = self.auth(self.http.get(&url)).send().await?;
        Self::handle_status(resp).await
    }

    pub async fn get_positions(&self) -> Result<PositionsResponse, TradierError> {
        let url = format!("{}/accounts/{}/positions", self.base, self.account_id);
        let resp = self.auth(self.http.get(&url)).send().await?;
        Self::handle_status(resp).await
    }
}

// ─── request shapes ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct PlaceEquityOrder {
    pub symbol: String,
    pub side: EquitySide,
    pub quantity: Decimal,
    pub order_type: OrderType,
    pub duration: Duration_,
    pub price: Option<Decimal>,
    pub stop: Option<Decimal>,
    pub tag: Option<String>,
}

impl PlaceEquityOrder {
    pub fn market(symbol: impl Into<String>, side: EquitySide, qty: Decimal) -> Self {
        Self {
            symbol: symbol.into(),
            side,
            quantity: qty,
            order_type: OrderType::Market,
            duration: Duration_::Day,
            price: None,
            stop: None,
            tag: None,
        }
    }

    fn to_form(&self) -> Vec<(&'static str, String)> {
        let mut v: Vec<(&'static str, String)> = vec![
            ("class", "equity".into()),
            ("symbol", self.symbol.clone()),
            ("side", self.side.as_str().into()),
            ("quantity", self.quantity.to_string()),
            ("type", self.order_type.as_str().into()),
            ("duration", self.duration.as_str().into()),
        ];
        if let Some(p) = self.price {
            v.push(("price", p.to_string()));
        }
        if let Some(s) = self.stop {
            v.push(("stop", s.to_string()));
        }
        if let Some(t) = &self.tag {
            v.push(("tag", t.clone()));
        }
        v
    }
}

/// OTOCO = one-triggers-OCO. After the parent (leg 0) fills, the two
/// exit legs (take_profit + stop_loss) become an OCO pair.
#[derive(Debug, Clone)]
pub struct OtocoBracket {
    pub symbol: String,
    pub entry_side: EquitySide,
    pub exit_side: EquitySide,
    pub quantity: Decimal,
    pub take_profit_price: Decimal,
    pub stop_loss_price: Decimal,
    pub duration: Duration_,
    pub tag: Option<String>,
}

impl OtocoBracket {
    fn to_form(&self) -> Vec<(&'static str, String)> {
        // Tradier OTOCO form encoding: class=otoco + indexed leg arrays.
        // Leg 0 = parent (market entry); leg 1 = take-profit limit;
        // leg 2 = stop-loss stop. Each leg carries its own
        // symbol/quantity/side/type/duration/price/stop suffix.
        let mut v: Vec<(&'static str, String)> = vec![
            ("class", "otoco".into()),
            ("duration", self.duration.as_str().into()),
            ("symbol[0]", self.symbol.clone()),
            ("side[0]", self.entry_side.as_str().into()),
            ("quantity[0]", self.quantity.to_string()),
            ("type[0]", "market".into()),
            ("symbol[1]", self.symbol.clone()),
            ("side[1]", self.exit_side.as_str().into()),
            ("quantity[1]", self.quantity.to_string()),
            ("type[1]", "limit".into()),
            ("price[1]", self.take_profit_price.to_string()),
            ("symbol[2]", self.symbol.clone()),
            ("side[2]", self.exit_side.as_str().into()),
            ("quantity[2]", self.quantity.to_string()),
            ("type[2]", "stop".into()),
            ("stop[2]", self.stop_loss_price.to_string()),
        ];
        if let Some(t) = &self.tag {
            v.push(("tag", t.clone()));
        }
        v
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquitySide {
    Buy,
    Sell,
    SellShort,
    BuyToCover,
}

impl EquitySide {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Buy => "buy",
            Self::Sell => "sell",
            Self::SellShort => "sell_short",
            Self::BuyToCover => "buy_to_cover",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderType {
    Market,
    Limit,
    Stop,
    StopLimit,
}

impl OrderType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Market => "market",
            Self::Limit => "limit",
            Self::Stop => "stop",
            Self::StopLimit => "stop_limit",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Duration_ {
    Day,
    Gtc,
    Pre,
    Post,
}

impl Duration_ {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Day => "day",
            Self::Gtc => "gtc",
            Self::Pre => "pre",
            Self::Post => "post",
        }
    }
}

// ─── response shapes ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Serialize)]
struct PlaceOrderEnvelope {
    order: PlaceOrderResponse,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlaceOrderResponse {
    pub id: i64,
    pub status: String,
    /// Optional — Tradier sometimes echoes the partner_id back.
    #[serde(default)]
    pub partner_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BalancesResponse {
    pub balances: Balances,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Balances {
    pub account_number: Option<String>,
    pub account_type: Option<String>,
    pub total_equity: Option<Decimal>,
    pub total_cash: Option<Decimal>,
    pub option_buying_power: Option<Decimal>,
    pub stock_buying_power: Option<Decimal>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PositionsResponse {
    pub positions: Option<PositionsList>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum PositionsList {
    Empty(String),
    Single { position: Position },
    Many { position: Vec<Position> },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Position {
    pub cost_basis: Decimal,
    pub date_acquired: Option<String>,
    pub id: Option<i64>,
    pub quantity: Decimal,
    pub symbol: String,
}
