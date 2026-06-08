//! Tastytrade brokerage REST client — session auth + equity orders.
//!
//! API reference (verified 2026-06):
//!   Production: <https://api.tastyworks.com>
//!   Sandbox:    <https://api.cert.tastyworks.com> (the "cert" cluster)
//!   Auth:       POST /sessions → returns session-token; use that as a
//!               raw `Authorization: <token>` header (no Bearer prefix).
//!   Orders:     POST /accounts/{account_number}/orders
//!               JSON body with `order-type`, `time-in-force`,
//!               `price`, `price-effect`, `legs: [...]`.
//!
//! Scope note: this commit ships entry-only orders (single leg). Native
//! bracket support on Tastytrade uses the Complex Orders endpoint
//! (`order-type: "Notional Market"` + child orders) which has a
//! different shape — that's a follow-up. The engine still tracks
//! stop / take-profit state per strategy; until bracket lands, the
//! strategy's exit management is via subsequent submissions.
//!
//! Sources:
//!   <https://developer.tastytrade.com/api-overview/>
//!   <https://developer.tastytrade.com/order-submission/>

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub const SANDBOX_BASE: &str = "https://api.cert.tastyworks.com";
pub const LIVE_BASE: &str = "https://api.tastyworks.com";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TastytradeEnv {
    Sandbox,
    Live,
}

impl TastytradeEnv {
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
pub enum TastytradeError {
    #[error("tastytrade http {status}: {body}")]
    Http { status: u16, body: String },
    #[error("tastytrade auth failed")]
    AuthFailed,
    #[error("tastytrade insufficient buying power")]
    InsufficientBuyingPower,
    #[error("tastytrade invalid request: {0}")]
    InvalidRequest(String),
    #[error("transport: {0}")]
    Transport(#[from] reqwest::Error),
    #[error("decode: {0}")]
    Decode(#[from] serde_json::Error),
}

/// Either an already-minted session token (cheap reuse across calls)
/// or a username/password pair the client will exchange on first use.
#[derive(Debug, Clone)]
pub enum Auth {
    SessionToken(String),
    UserPass { login: String, password: String, remember_me: bool },
}

#[derive(Debug, Clone)]
pub struct TastytradeTrading {
    http: reqwest::Client,
    base: String,
    auth: Auth,
    /// Pre-resolved session token after first use. Held internally so
    /// the place_order path doesn't re-login on every call. NOT thread-
    /// safe across multiple instances; the dispatcher always builds a
    /// fresh client per submit so contention can't happen.
    token: std::sync::Arc<tokio::sync::Mutex<Option<String>>>,
    account_number: String,
}

impl TastytradeTrading {
    pub fn new(
        env: TastytradeEnv,
        auth: Auth,
        account_number: impl Into<String>,
    ) -> Self {
        let initial_token = if let Auth::SessionToken(ref t) = auth {
            Some(t.clone())
        } else {
            None
        };
        Self {
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .expect("reqwest client"),
            base: env.base().to_string(),
            auth,
            token: std::sync::Arc::new(tokio::sync::Mutex::new(initial_token)),
            account_number: account_number.into(),
        }
    }

    /// Test-only — fixed base URL.
    pub fn with_base(
        base: impl Into<String>,
        auth: Auth,
        account_number: impl Into<String>,
    ) -> Self {
        let initial_token = if let Auth::SessionToken(ref t) = auth {
            Some(t.clone())
        } else {
            None
        };
        Self {
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(5))
                .build()
                .expect("reqwest client"),
            base: base.into(),
            auth,
            token: std::sync::Arc::new(tokio::sync::Mutex::new(initial_token)),
            account_number: account_number.into(),
        }
    }

    async fn ensure_token(&self) -> Result<String, TastytradeError> {
        {
            let guard = self.token.lock().await;
            if let Some(t) = guard.as_ref() {
                return Ok(t.clone());
            }
        }
        let (login, password, remember_me) = match &self.auth {
            Auth::SessionToken(_) => unreachable!("token already populated above"),
            Auth::UserPass { login, password, remember_me } => {
                (login.clone(), password.clone(), *remember_me)
            }
        };
        let body = serde_json::json!({
            "login": login,
            "password": password,
            "remember-me": remember_me,
        });
        let resp = self
            .http
            .post(format!("{}/sessions", self.base))
            .header("Accept", "application/json")
            .json(&body)
            .send()
            .await?;
        let status = resp.status();
        let text = resp.text().await?;
        if !status.is_success() {
            if status.as_u16() == 401 {
                return Err(TastytradeError::AuthFailed);
            }
            return Err(TastytradeError::Http {
                status: status.as_u16(),
                body: text,
            });
        }
        let SessionResponse { data } = serde_json::from_str(&text)?;
        let mut guard = self.token.lock().await;
        *guard = Some(data.session_token.clone());
        Ok(data.session_token)
    }

    fn auth_header(&self, req: reqwest::RequestBuilder, token: &str) -> reqwest::RequestBuilder {
        // Tastytrade uses the raw session token in the Authorization
        // header — no 'Bearer' prefix, no Basic encoding.
        req.header("Authorization", token)
            .header("Accept", "application/json")
    }

    async fn handle_status<T: for<'de> Deserialize<'de>>(
        resp: reqwest::Response,
    ) -> Result<T, TastytradeError> {
        let status = resp.status();
        let body = resp.text().await?;
        if status.is_success() {
            return Ok(serde_json::from_str(&body)?);
        }
        match status.as_u16() {
            401 => Err(TastytradeError::AuthFailed),
            403 if body.to_ascii_lowercase().contains("buying power") => {
                Err(TastytradeError::InsufficientBuyingPower)
            }
            400 | 422 => Err(TastytradeError::InvalidRequest(body)),
            _ => Err(TastytradeError::Http { status: status.as_u16(), body }),
        }
    }

    pub async fn place_equity_order(
        &self,
        req: &PlaceEquityOrder,
    ) -> Result<PlaceOrderResponse, TastytradeError> {
        let token = self.ensure_token().await?;
        let url = format!(
            "{}/accounts/{}/orders",
            self.base, self.account_number
        );
        let resp = self
            .auth_header(self.http.post(&url), &token)
            .json(&req.to_json())
            .send()
            .await?;
        Self::handle_status::<PlaceOrderEnvelope>(resp)
            .await
            .map(|e| e.data.order)
    }

    pub async fn cancel_order(&self, order_id: i64) -> Result<(), TastytradeError> {
        let token = self.ensure_token().await?;
        let url = format!(
            "{}/accounts/{}/orders/{order_id}",
            self.base, self.account_number
        );
        let resp = self.auth_header(self.http.delete(&url), &token).send().await?;
        let status = resp.status();
        let body = resp.text().await?;
        if status.is_success() {
            return Ok(());
        }
        match status.as_u16() {
            401 => Err(TastytradeError::AuthFailed),
            422 | 400 => Err(TastytradeError::InvalidRequest(body)),
            _ => Err(TastytradeError::Http { status: status.as_u16(), body }),
        }
    }

    pub async fn get_balances(&self) -> Result<BalancesResponse, TastytradeError> {
        let token = self.ensure_token().await?;
        let url = format!(
            "{}/accounts/{}/balances",
            self.base, self.account_number
        );
        let resp = self.auth_header(self.http.get(&url), &token).send().await?;
        Self::handle_status(resp).await
    }
}

// ─── request shapes ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct PlaceEquityOrder {
    pub symbol: String,
    pub action: EquityAction,
    pub quantity: Decimal,
    pub order_type: OrderType,
    pub time_in_force: TimeInForce,
    /// Required for Limit / Stop Limit. Decimal in dollars.
    pub price: Option<Decimal>,
    /// Required when price is set: "Debit" (we pay) or "Credit" (we receive).
    pub price_effect: Option<PriceEffect>,
    pub stop_trigger: Option<Decimal>,
}

impl PlaceEquityOrder {
    pub fn market(symbol: impl Into<String>, action: EquityAction, qty: Decimal) -> Self {
        Self {
            symbol: symbol.into(),
            action,
            quantity: qty,
            order_type: OrderType::Market,
            time_in_force: TimeInForce::Day,
            price: None,
            price_effect: None,
            stop_trigger: None,
        }
    }

    fn to_json(&self) -> serde_json::Value {
        let mut leg = serde_json::Map::new();
        leg.insert("symbol".into(), self.symbol.clone().into());
        leg.insert("instrument-type".into(), "Equity".into());
        leg.insert("action".into(), self.action.as_str().into());
        leg.insert("quantity".into(), self.quantity.to_string().into());

        let mut root = serde_json::Map::new();
        root.insert("order-type".into(), self.order_type.as_str().into());
        root.insert("time-in-force".into(), self.time_in_force.as_str().into());
        if let Some(p) = self.price {
            root.insert("price".into(), p.to_string().into());
        }
        if let Some(pe) = self.price_effect {
            root.insert("price-effect".into(), pe.as_str().into());
        }
        if let Some(st) = self.stop_trigger {
            root.insert("stop-trigger".into(), st.to_string().into());
        }
        root.insert("legs".into(), serde_json::Value::Array(vec![leg.into()]));
        root.into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquityAction {
    BuyToOpen,
    SellToClose,
    SellToOpen,
    BuyToClose,
}

impl EquityAction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::BuyToOpen => "Buy to Open",
            Self::SellToClose => "Sell to Close",
            Self::SellToOpen => "Sell to Open",
            Self::BuyToClose => "Buy to Close",
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
            Self::Market => "Market",
            Self::Limit => "Limit",
            Self::Stop => "Stop",
            Self::StopLimit => "Stop Limit",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeInForce {
    Day,
    Gtc,
    Gtd,
    Ioc,
}

impl TimeInForce {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Day => "Day",
            Self::Gtc => "GTC",
            Self::Gtd => "GTD",
            Self::Ioc => "IOC",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PriceEffect {
    Debit,
    Credit,
}

impl PriceEffect {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Debit => "Debit",
            Self::Credit => "Credit",
        }
    }
}

// ─── response shapes ────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct SessionResponse {
    data: SessionData,
}
#[derive(Debug, Deserialize)]
struct SessionData {
    #[serde(rename = "session-token")]
    session_token: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct PlaceOrderEnvelope {
    data: PlaceOrderData,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
struct PlaceOrderData {
    order: PlaceOrderResponse,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlaceOrderResponse {
    pub id: i64,
    pub status: String,
    #[serde(rename = "underlying-symbol", default)]
    pub underlying_symbol: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BalancesResponse {
    pub data: Balances,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Balances {
    #[serde(rename = "account-number", default)]
    pub account_number: Option<String>,
    #[serde(rename = "net-liquidating-value", default)]
    pub net_liquidating_value: Option<Decimal>,
    #[serde(rename = "cash-balance", default)]
    pub cash_balance: Option<Decimal>,
    #[serde(rename = "equity-buying-power", default)]
    pub equity_buying_power: Option<Decimal>,
}
