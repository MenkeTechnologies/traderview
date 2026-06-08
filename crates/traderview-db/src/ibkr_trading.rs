//! Interactive Brokers Client Portal Web API REST client.
//!
//! IBKR's algo / desktop ecosystem has THREE auth models:
//!   1. Local gateway: user runs IB Gateway or Client Portal Gateway
//!      locally; gateway handles 2FA + session; app POSTs against
//!      `https://localhost:5000/v1/api` with cookies.
//!   2. OAuth 1.0a — older flow, supported but complex.
//!   3. OAuth 2.0 — newer; bearer tokens.
//!
//! This client supports the bearer-token path (option 3) with a
//! user-configurable base URL so option 1 also works — when the user
//! runs the gateway locally, the base URL points at it and the cookie
//! jar takes over (reqwest enables cookies by default).
//!
//! Source: <https://interactivebrokers.github.io/cpwebapi/>

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub const DEFAULT_LOCAL_BASE: &str = "https://localhost:5000/v1/api";

#[derive(Debug, thiserror::Error)]
pub enum IbkrError {
    #[error("ibkr http {status}: {body}")]
    Http { status: u16, body: String },
    #[error("ibkr auth failed (gateway session expired or no token)")]
    AuthFailed,
    #[error("ibkr insufficient buying power")]
    InsufficientBuyingPower,
    #[error("ibkr invalid request: {0}")]
    InvalidRequest(String),
    #[error("transport: {0}")]
    Transport(#[from] reqwest::Error),
    #[error("decode: {0}")]
    Decode(#[from] serde_json::Error),
}

#[derive(Debug, Clone)]
pub struct IbkrTrading {
    http: reqwest::Client,
    base: String,
    /// Optional bearer token. When None we rely on cookie-jar auth
    /// against a local gateway.
    bearer_token: Option<String>,
    account_id: String,
}

impl IbkrTrading {
    pub fn new(
        base: impl Into<String>,
        bearer_token: Option<String>,
        account_id: impl Into<String>,
    ) -> Self {
        Self {
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .cookie_store(true)
                .danger_accept_invalid_certs(true) // local gateway uses self-signed cert
                .build()
                .expect("reqwest client"),
            base: base.into(),
            bearer_token,
            account_id: account_id.into(),
        }
    }

    pub fn with_base(
        base: impl Into<String>,
        bearer_token: Option<String>,
        account_id: impl Into<String>,
    ) -> Self {
        Self {
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(5))
                .cookie_store(true)
                .build()
                .expect("reqwest client"),
            base: base.into(),
            bearer_token,
            account_id: account_id.into(),
        }
    }

    fn auth(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        let req = req.header("Accept", "application/json");
        if let Some(t) = &self.bearer_token {
            req.header("Authorization", format!("Bearer {t}"))
        } else {
            req
        }
    }

    async fn handle_status<T: for<'de> Deserialize<'de>>(
        resp: reqwest::Response,
    ) -> Result<T, IbkrError> {
        let status = resp.status();
        let body = resp.text().await?;
        if status.is_success() {
            return Ok(serde_json::from_str(&body)?);
        }
        match status.as_u16() {
            401 => Err(IbkrError::AuthFailed),
            403 if body.to_ascii_lowercase().contains("buying power") => {
                Err(IbkrError::InsufficientBuyingPower)
            }
            400 | 422 => Err(IbkrError::InvalidRequest(body)),
            _ => Err(IbkrError::Http { status: status.as_u16(), body }),
        }
    }

    /// POST /iserver/account/{accountId}/orders
    /// Body: {"orders": [{conid, side, orderType, quantity, tif, price?}]}
    /// IBKR responses for order placement frequently come back as an
    /// array of `{order_id, local_order_id, order_status, ...}`.
    pub async fn place_order(&self, req: &PlaceOrder) -> Result<Vec<PlaceOrderResponse>, IbkrError> {
        let url = format!(
            "{}/iserver/account/{}/orders",
            self.base, self.account_id
        );
        let body = serde_json::json!({ "orders": [req.to_json()] });
        let resp = self.auth(self.http.post(&url)).json(&body).send().await?;
        Self::handle_status(resp).await
    }

    pub async fn cancel_order(&self, order_id: &str) -> Result<(), IbkrError> {
        let url = format!(
            "{}/iserver/account/{}/order/{order_id}",
            self.base, self.account_id
        );
        let resp = self.auth(self.http.delete(&url)).send().await?;
        let status = resp.status();
        let body = resp.text().await?;
        if status.is_success() {
            return Ok(());
        }
        match status.as_u16() {
            401 => Err(IbkrError::AuthFailed),
            400 | 422 => Err(IbkrError::InvalidRequest(body)),
            _ => Err(IbkrError::Http { status: status.as_u16(), body }),
        }
    }

    /// GET /portfolio/{accountId}/summary — equity, cash, buying power.
    pub async fn get_summary(&self) -> Result<PortfolioSummary, IbkrError> {
        let url = format!(
            "{}/portfolio/{}/summary",
            self.base, self.account_id
        );
        let resp = self.auth(self.http.get(&url)).send().await?;
        Self::handle_status(resp).await
    }

    /// GET /portfolio/{accountId}/positions/0 — first page of positions.
    pub async fn get_positions(&self) -> Result<Vec<Position>, IbkrError> {
        let url = format!(
            "{}/portfolio/{}/positions/0",
            self.base, self.account_id
        );
        let resp = self.auth(self.http.get(&url)).send().await?;
        Self::handle_status(resp).await
    }

    /// GET /iserver/secdef/search?symbol=AAPL — returns matching
    /// contracts. We pick the first STK (stock) result and return its
    /// conid. Strategies fire on symbols; IBKR orders need conids, so
    /// every entry path runs this first.
    pub async fn resolve_stock_conid(&self, symbol: &str) -> Result<i64, IbkrError> {
        let url = format!("{}/iserver/secdef/search", self.base);
        let resp = self
            .auth(self.http.post(&url))
            .json(&serde_json::json!({ "symbol": symbol, "name": false, "secType": "STK" }))
            .send()
            .await?;
        let hits: Vec<SecdefHit> = Self::handle_status(resp).await?;
        // Prefer US listings (conid resolution can return foreign
        // dual-listings). NYSE / NASDAQ exchanges first.
        let pick = hits
            .iter()
            .find(|h| {
                h.description.as_deref().map(|s| {
                    let s = s.to_ascii_uppercase();
                    s.contains("NYSE") || s.contains("NASDAQ") || s.contains("ARCA")
                }).unwrap_or(false)
            })
            .or_else(|| hits.first())
            .ok_or_else(|| IbkrError::InvalidRequest(format!("no contract found for {symbol}")))?;
        pick.conid
            .ok_or_else(|| IbkrError::InvalidRequest(format!("no conid in result for {symbol}")))
    }
}

#[derive(Debug, Clone, Deserialize)]
struct SecdefHit {
    conid: Option<i64>,
    /// IBKR's `description` field carries the exchange — e.g.
    /// "AAPL.NASDAQ.STK" — so we filter on it for US picks.
    description: Option<String>,
}

// ─── request shapes ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct PlaceOrder {
    /// Contract id (conid). IBKR uses integer instrument IDs, not
    /// symbols. Caller is expected to resolve via /iserver/secdef/search.
    pub conid: i64,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub quantity: Decimal,
    pub tif: Tif,
    pub price: Option<Decimal>,
    /// Optional client-side tag for reconciliation.
    pub client_order_id: Option<String>,
}

impl PlaceOrder {
    fn to_json(&self) -> serde_json::Value {
        let mut o = serde_json::Map::new();
        o.insert("conid".into(), self.conid.into());
        o.insert("side".into(), self.side.as_str().into());
        o.insert("orderType".into(), self.order_type.as_str().into());
        o.insert("quantity".into(), self.quantity.to_string().parse::<f64>().unwrap_or(0.0).into());
        o.insert("tif".into(), self.tif.as_str().into());
        if let Some(p) = self.price {
            o.insert("price".into(), p.to_string().parse::<f64>().unwrap_or(0.0).into());
        }
        if let Some(t) = &self.client_order_id {
            o.insert("cOID".into(), t.clone().into());
        }
        o.into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderSide {
    Buy,
    Sell,
}
impl OrderSide {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Buy => "BUY",
            Self::Sell => "SELL",
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
            Self::Market => "MKT",
            Self::Limit => "LMT",
            Self::Stop => "STP",
            Self::StopLimit => "STOP_LIMIT",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tif {
    Day,
    Gtc,
    Opg,
    Ioc,
}
impl Tif {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Day => "DAY",
            Self::Gtc => "GTC",
            Self::Opg => "OPG",
            Self::Ioc => "IOC",
        }
    }
}

// ─── response shapes ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlaceOrderResponse {
    pub order_id: Option<String>,
    pub local_order_id: Option<String>,
    pub order_status: Option<String>,
    /// IBKR sometimes responds with a 'message' / 'messageIds' array
    /// requiring a follow-up confirmation POST. Surface them so the
    /// caller can decide.
    pub message: Option<Vec<String>>,
    #[serde(rename = "messageIds")]
    pub message_ids: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PortfolioSummary {
    #[serde(rename = "netliquidation")]
    pub net_liquidation: Option<SummaryValue>,
    #[serde(rename = "totalcashvalue")]
    pub total_cash_value: Option<SummaryValue>,
    #[serde(rename = "buyingpower")]
    pub buying_power: Option<SummaryValue>,
    #[serde(rename = "availablefunds")]
    pub available_funds: Option<SummaryValue>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SummaryValue {
    pub amount: Option<f64>,
    pub currency: Option<String>,
    pub timestamp: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Position {
    #[serde(rename = "acctId")]
    pub account_id: Option<String>,
    pub conid: Option<i64>,
    #[serde(rename = "contractDesc")]
    pub contract_desc: Option<String>,
    pub position: Option<f64>,
    #[serde(rename = "mktPrice")]
    pub mkt_price: Option<f64>,
    #[serde(rename = "avgPrice")]
    pub avg_price: Option<f64>,
    #[serde(rename = "unrealizedPnl")]
    pub unrealized_pnl: Option<f64>,
}
