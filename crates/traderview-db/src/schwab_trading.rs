//! Schwab Trader API REST client (TD Ameritrade replacement).
//!
//! After Schwab finished migrating TD Ameritrade clients (Sep 2024),
//! the legacy TD API was retired. The replacement is the Schwab
//! Developer Portal "Trader API" at <https://api.schwabapi.com/trader/v1>.
//! Auth is OAuth 2.0:
//!   - Authorization-code flow on first connect (user logs in via
//!     web; redirect with `code` → exchange for access + refresh token).
//!   - Access token: 30-min TTL.
//!   - Refresh token: 7-day TTL; rotated on each refresh.
//!
//! This client receives the tokens after the user runs through the
//! OAuth flow in Settings (separate flow, not implemented here). On
//! a 401 we attempt one refresh via `refresh_access_token()` and
//! retry; if that also fails we surface AuthFailed so the UI prompts
//! a re-login. The refreshed pair is returned through a tokio
//! `Mutex<Tokens>` so concurrent callers share the rotation.
//!
//! Sources:
//!   - <https://developer.schwab.com/products/trader-api--individual>
//!   - <https://developer.schwab.com/user-guides/apis-and-apps/authentication>

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

pub const TRADER_BASE: &str = "https://api.schwabapi.com/trader/v1";
pub const AUTH_BASE: &str = "https://api.schwabapi.com/v1";

#[derive(Debug, thiserror::Error)]
pub enum SchwabError {
    #[error("schwab http {status}: {body}")]
    Http { status: u16, body: String },
    #[error("schwab auth failed (refresh token expired or revoked)")]
    AuthFailed,
    #[error("schwab insufficient buying power")]
    InsufficientBuyingPower,
    #[error("schwab invalid request: {0}")]
    InvalidRequest(String),
    #[error("transport: {0}")]
    Transport(#[from] reqwest::Error),
    #[error("decode: {0}")]
    Decode(#[from] serde_json::Error),
}

/// OAuth tokens. The dispatcher hands us the pair; `refresh_token`
/// gets rotated on every refresh call, so we keep them behind a
/// `Mutex` so concurrent callers don't race the rotation. The caller
/// is responsible for persisting the rotated pair (a `TokenCallback`
/// is invoked after every successful refresh).
#[derive(Debug, Clone)]
pub struct Tokens {
    pub access_token: String,
    pub refresh_token: String,
}

pub type TokenCallback =
    Arc<dyn Fn(Tokens) + Send + Sync + 'static>;

#[derive(Clone)]
pub struct SchwabTrading {
    http: reqwest::Client,
    trader_base: String,
    auth_base: String,
    tokens: Arc<Mutex<Tokens>>,
    /// `client_id:client_secret` for the refresh-token grant. Schwab
    /// uses HTTP Basic on the `/v1/oauth/token` endpoint.
    client_id: String,
    client_secret: String,
    account_hash: String,
    /// Optional callback fired with the new (access, refresh) pair
    /// every time a refresh succeeds — the web layer hooks this to
    /// persist the rotated refresh_token back to user_settings.
    on_token_refresh: Option<TokenCallback>,
}

impl std::fmt::Debug for SchwabTrading {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SchwabTrading")
            .field("trader_base", &self.trader_base)
            .field("auth_base", &self.auth_base)
            .field("account_hash", &self.account_hash)
            .field("on_token_refresh", &self.on_token_refresh.is_some())
            .finish()
    }
}

impl SchwabTrading {
    pub fn new(
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
        tokens: Tokens,
        account_hash: impl Into<String>,
    ) -> Self {
        Self {
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .expect("reqwest client"),
            trader_base: TRADER_BASE.into(),
            auth_base: AUTH_BASE.into(),
            tokens: Arc::new(Mutex::new(tokens)),
            client_id: client_id.into(),
            client_secret: client_secret.into(),
            account_hash: account_hash.into(),
            on_token_refresh: None,
        }
    }

    /// Test-friendly constructor pinning both bases.
    pub fn with_bases(
        trader_base: impl Into<String>,
        auth_base: impl Into<String>,
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
        tokens: Tokens,
        account_hash: impl Into<String>,
    ) -> Self {
        Self {
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(5))
                .build()
                .expect("reqwest client"),
            trader_base: trader_base.into(),
            auth_base: auth_base.into(),
            tokens: Arc::new(Mutex::new(tokens)),
            client_id: client_id.into(),
            client_secret: client_secret.into(),
            account_hash: account_hash.into(),
            on_token_refresh: None,
        }
    }

    pub fn on_token_refresh(mut self, cb: TokenCallback) -> Self {
        self.on_token_refresh = Some(cb);
        self
    }

    async fn current_access_token(&self) -> String {
        self.tokens.lock().await.access_token.clone()
    }

    /// POST {auth_base}/oauth/token — refresh-token grant. Updates the
    /// shared Tokens lock and fires the persistence callback.
    pub async fn refresh_access_token(&self) -> Result<Tokens, SchwabError> {
        let refresh_token = self.tokens.lock().await.refresh_token.clone();
        let url = format!("{}/oauth/token", self.auth_base);
        let resp = self
            .http
            .post(&url)
            .basic_auth(&self.client_id, Some(&self.client_secret))
            .form(&[
                ("grant_type", "refresh_token"),
                ("refresh_token", refresh_token.as_str()),
            ])
            .send()
            .await?;
        let status = resp.status();
        let body = resp.text().await?;
        if !status.is_success() {
            // refresh-token rotation hard failure → user needs to
            // re-run the OAuth flow.
            if status.as_u16() == 400 || status.as_u16() == 401 {
                return Err(SchwabError::AuthFailed);
            }
            return Err(SchwabError::Http { status: status.as_u16(), body });
        }
        #[derive(Deserialize)]
        struct RefreshResp {
            access_token: String,
            refresh_token: Option<String>,
        }
        let parsed: RefreshResp = serde_json::from_str(&body)?;
        let new_refresh = parsed.refresh_token.unwrap_or(refresh_token);
        let new = Tokens {
            access_token: parsed.access_token,
            refresh_token: new_refresh,
        };
        {
            let mut guard = self.tokens.lock().await;
            *guard = new.clone();
        }
        if let Some(cb) = &self.on_token_refresh {
            cb(new.clone());
        }
        Ok(new)
    }

    async fn handle_status<T: for<'de> Deserialize<'de>>(
        resp: reqwest::Response,
    ) -> Result<T, SchwabError> {
        let status = resp.status();
        let body = resp.text().await?;
        if status.is_success() {
            // Some Schwab endpoints (DELETE on /orders) return 200 OK
            // with empty body. Caller asks for `()` via empty struct.
            if body.trim().is_empty() {
                return Ok(serde_json::from_str("null")?);
            }
            return Ok(serde_json::from_str(&body)?);
        }
        match status.as_u16() {
            401 => Err(SchwabError::AuthFailed),
            403 if body.to_ascii_lowercase().contains("buying power") => {
                Err(SchwabError::InsufficientBuyingPower)
            }
            400 | 422 => Err(SchwabError::InvalidRequest(body)),
            _ => Err(SchwabError::Http { status: status.as_u16(), body }),
        }
    }

    /// POST /trader/v1/accounts/{accountHash}/orders. On 401 we attempt
    /// ONE refresh + retry. The single-leg market order JSON shape is
    /// canonical per Schwab docs: orderType + session + duration +
    /// orderStrategyType + orderLegCollection.
    pub async fn place_order(&self, req: &PlaceOrder) -> Result<PlaceOrderResponse, SchwabError> {
        let url = format!("{}/accounts/{}/orders", self.trader_base, self.account_hash);
        let body = req.to_json();
        let first = self.send_post(&url, &body).await;
        let resp = match first {
            Err(SchwabError::AuthFailed) => {
                // One refresh + retry.
                self.refresh_access_token().await?;
                self.send_post(&url, &body).await?
            }
            other => other?,
        };
        // Schwab returns 201 Created with an empty body + a Location
        // header `/orders/{id}` for placements. send_post stripped that
        // header into resp.status/headers but we lost it inside
        // handle_status. The unified handle_status expects a body, so
        // we do the placement-specific decode here.
        Ok(resp)
    }

    async fn send_post(
        &self,
        url: &str,
        body: &serde_json::Value,
    ) -> Result<PlaceOrderResponse, SchwabError> {
        let token = self.current_access_token().await;
        let resp = self
            .http
            .post(url)
            .bearer_auth(&token)
            .header("Accept", "application/json")
            .json(body)
            .send()
            .await?;
        let status = resp.status();
        // Order placements return 201 + Location header carrying the order id.
        if status.as_u16() == 201 {
            let order_id = resp
                .headers()
                .get("Location")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.rsplit('/').next())
                .map(String::from);
            return Ok(PlaceOrderResponse {
                order_id,
                status: Some("Accepted".into()),
            });
        }
        let body = resp.text().await?;
        if status.is_success() {
            // 200 happens for some replay / replace paths.
            return Ok(PlaceOrderResponse {
                order_id: None,
                status: Some("Accepted".into()),
            });
        }
        match status.as_u16() {
            401 => Err(SchwabError::AuthFailed),
            403 if body.to_ascii_lowercase().contains("buying power") => {
                Err(SchwabError::InsufficientBuyingPower)
            }
            400 | 422 => Err(SchwabError::InvalidRequest(body)),
            _ => Err(SchwabError::Http { status: status.as_u16(), body }),
        }
    }

    /// DELETE /trader/v1/accounts/{accountHash}/orders/{orderId}.
    pub async fn cancel_order(&self, order_id: &str) -> Result<(), SchwabError> {
        let url = format!(
            "{}/accounts/{}/orders/{order_id}",
            self.trader_base, self.account_hash
        );
        let token = self.current_access_token().await;
        let resp = self.http.delete(&url).bearer_auth(&token).send().await?;
        let status = resp.status();
        if status.is_success() {
            return Ok(());
        }
        match status.as_u16() {
            401 => Err(SchwabError::AuthFailed),
            _ => Err(SchwabError::Http {
                status: status.as_u16(),
                body: resp.text().await.unwrap_or_default(),
            }),
        }
    }

    /// Read the current access token from the shared Tokens lock —
    /// needed by the streamer pump to assemble the WS LOGIN payload.
    pub async fn current_access_token_public(&self) -> String {
        self.current_access_token().await
    }

    /// GET /trader/v1/userPreference — returns streamerInfo (WS URL,
    /// client correl id, customer id) used by the streaming pump.
    pub async fn get_user_preference(&self) -> Result<String, SchwabError> {
        let url = format!("{}/userPreference", self.trader_base);
        let token = self.current_access_token().await;
        let resp = self
            .http
            .get(&url)
            .bearer_auth(&token)
            .header("Accept", "application/json")
            .send()
            .await?;
        let status = resp.status();
        let body = resp.text().await?;
        if status.is_success() {
            return Ok(body);
        }
        match status.as_u16() {
            401 => Err(SchwabError::AuthFailed),
            _ => Err(SchwabError::Http { status: status.as_u16(), body }),
        }
    }

    /// GET /trader/v1/accounts/{accountHash} — balances + positions.
    pub async fn get_account(&self) -> Result<AccountSummary, SchwabError> {
        let url = format!("{}/accounts/{}", self.trader_base, self.account_hash);
        let token = self.current_access_token().await;
        let resp = self
            .http
            .get(&url)
            .bearer_auth(&token)
            .header("Accept", "application/json")
            .send()
            .await?;
        Self::handle_status(resp).await
    }
}

// ─── request shapes ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct PlaceOrder {
    pub symbol: String,
    pub instruction: Instruction,
    pub order_type: OrderType,
    pub quantity: Decimal,
    pub duration: Duration_,
    pub session: Session,
    pub price: Option<Decimal>,
    /// Optional client-side tag persisted in `orderLegCollection[0].comment`.
    pub comment: Option<String>,
}

impl PlaceOrder {
    fn to_json(&self) -> serde_json::Value {
        let qty: f64 = self.quantity.to_string().parse().unwrap_or(0.0);
        let mut o = serde_json::json!({
            "orderType": self.order_type.as_str(),
            "session": self.session.as_str(),
            "duration": self.duration.as_str(),
            "orderStrategyType": "SINGLE",
            "orderLegCollection": [{
                "instruction": self.instruction.as_str(),
                "quantity": qty,
                "instrument": {
                    "symbol": self.symbol,
                    "assetType": "EQUITY"
                }
            }]
        });
        if let Some(p) = self.price {
            o["price"] = p.to_string().parse::<f64>().unwrap_or(0.0).into();
        }
        if let Some(c) = &self.comment {
            // Schwab's order strategy JSON tucks the tag on the leg.
            o["orderLegCollection"][0]["comment"] = c.clone().into();
        }
        o
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instruction {
    Buy,
    Sell,
    SellShort,
    BuyToCover,
}
impl Instruction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Buy => "BUY",
            Self::Sell => "SELL",
            Self::SellShort => "SELL_SHORT",
            Self::BuyToCover => "BUY_TO_COVER",
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
            Self::Market => "MARKET",
            Self::Limit => "LIMIT",
            Self::Stop => "STOP",
            Self::StopLimit => "STOP_LIMIT",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Duration_ {
    Day,
    GoodTillCancel,
    FillOrKill,
    ImmediateOrCancel,
}
impl Duration_ {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Day => "DAY",
            Self::GoodTillCancel => "GOOD_TILL_CANCEL",
            Self::FillOrKill => "FILL_OR_KILL",
            Self::ImmediateOrCancel => "IMMEDIATE_OR_CANCEL",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Session {
    Normal,
    Am,
    Pm,
    Seamless,
}
impl Session {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Normal => "NORMAL",
            Self::Am => "AM",
            Self::Pm => "PM",
            Self::Seamless => "SEAMLESS",
        }
    }
}

// ─── response shapes ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlaceOrderResponse {
    /// Pulled from the Location response header — Schwab returns
    /// `Location: /orders/{accountHash}/{orderId}` on 201.
    pub order_id: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountSummary {
    pub securities_account: Option<SecuritiesAccount>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SecuritiesAccount {
    pub account_number: Option<String>,
    pub r#type: Option<String>,
    pub current_balances: Option<Balances>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Balances {
    pub liquidation_value: Option<f64>,
    pub cash_balance: Option<f64>,
    pub buying_power: Option<f64>,
}
