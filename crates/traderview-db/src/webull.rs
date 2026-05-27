//! Webull personal-broker integration (read-only).
//!
//! Personal use only — Webull does not publish an official API. This client
//! talks to the same REST endpoints the Webull mobile / web apps use,
//! authenticated with the user's existing session tokens (DID, access_token,
//! trade_token). Tokens are held in-process only and never written to disk.
//!
//! How a user gets the three tokens:
//!   1. Log in to webull.com in a browser (with MFA / trade pin already done)
//!   2. Open DevTools → Network → click any /api/trade/* request
//!   3. Copy these three request headers:
//!      `did`              → device ID
//!      `access_token`     → session token
//!      `t_token`          → trade-action token (only required for trading)
//!   4. Paste into Settings → Webull → Connect
//!
//! What this module does:
//!   * `positions()` — open positions w/ avg cost, last price, unrealized P/L
//!   * `today_orders()` — orders filled today
//!   * `account_summary()` — net liquidation, cash, day P/L
//!
//! All read-only. Order entry is intentionally **not** implemented.

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, RwLock};

const TRADE_BASE: &str = "https://tradeapi.webullbroker.com";

#[derive(Debug, Clone, Default)]
pub struct Creds {
    pub did: String,
    pub access_token: String,
    pub t_token: Option<String>,
    pub account_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WPosition {
    pub symbol: String,
    pub side: String,       // "long" | "short"
    pub asset_type: String, // "stock" | "option"
    pub qty: f64,
    pub avg_cost: f64,
    pub last_price: f64,
    pub market_value: f64,
    pub unrealized_pnl: f64,
    pub unrealized_pct: f64,
    pub day_pnl: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct WOrder {
    pub order_id: String,
    pub symbol: String,
    pub side: String,
    pub qty: f64,
    pub filled_qty: f64,
    pub avg_fill_price: f64,
    pub status: String,
    pub created_at: Option<DateTime<Utc>>,
    pub filled_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WAccount {
    pub net_liquidation: f64,
    pub cash: f64,
    pub day_pnl: f64,
    pub total_pnl: f64,
    pub buying_power: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct WSnapshot {
    pub account: Option<WAccount>,
    pub positions: Vec<WPosition>,
    pub orders: Vec<WOrder>,
    pub fetched_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct WebullClient {
    creds: Arc<RwLock<Creds>>,
    last: Arc<DashMap<&'static str, WSnapshot>>,
    tx: broadcast::Sender<WSnapshot>,
    started: Arc<tokio::sync::Mutex<bool>>,
}

impl WebullClient {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(64);
        Self {
            creds: Arc::new(RwLock::new(Creds::default())),
            last: Arc::new(DashMap::new()),
            tx,
            started: Arc::new(tokio::sync::Mutex::new(false)),
        }
    }

    pub async fn set_creds(&self, c: Creds) {
        *self.creds.write().await = c;
        self.ensure_poller().await;
    }

    pub async fn has_creds(&self) -> bool {
        let c = self.creds.read().await;
        !c.did.is_empty() && !c.access_token.is_empty()
    }

    pub fn subscribe(&self) -> broadcast::Receiver<WSnapshot> {
        self.tx.subscribe()
    }

    pub fn last_snapshot(&self) -> Option<WSnapshot> {
        self.last.get("last").map(|e| e.value().clone())
    }

    /// Spawn the polling task once. Safe to call repeatedly.
    async fn ensure_poller(&self) {
        let mut started = self.started.lock().await;
        if *started {
            return;
        }
        *started = true;
        let me = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                interval.tick().await;
                if !me.has_creds().await {
                    continue;
                }
                match me.fetch_snapshot().await {
                    Ok(snap) => {
                        me.last.insert("last", snap.clone());
                        let _ = me.tx.send(snap);
                    }
                    Err(e) => tracing::warn!(?e, "webull poll failed"),
                }
            }
        });
    }

    async fn fetch_snapshot(&self) -> anyhow::Result<WSnapshot> {
        let client = self.client();
        let creds = self.creds.read().await.clone();
        let account_id = match &creds.account_id {
            Some(id) => id.clone(),
            None => self.resolve_account_id(&client, &creds).await?,
        };
        let positions = self
            .fetch_positions(&client, &creds, &account_id)
            .await
            .unwrap_or_default();
        let orders = self
            .fetch_today_orders(&client, &creds, &account_id)
            .await
            .unwrap_or_default();
        let account = self.fetch_account(&client, &creds, &account_id).await.ok();
        Ok(WSnapshot {
            account,
            positions,
            orders,
            fetched_at: Utc::now(),
        })
    }

    fn client(&self) -> reqwest::Client {
        reqwest::Client::builder()
            .user_agent("traderview/0.1 (webull-personal)")
            .timeout(Duration::from_secs(10))
            .build()
            .expect("client")
    }

    fn headers(&self, creds: &Creds) -> reqwest::header::HeaderMap {
        let mut h = reqwest::header::HeaderMap::new();
        h.insert(
            "did",
            creds.did.parse().unwrap_or_else(|_| "x".parse().unwrap()),
        );
        h.insert(
            "access_token",
            creds
                .access_token
                .parse()
                .unwrap_or_else(|_| "x".parse().unwrap()),
        );
        if let Some(t) = &creds.t_token {
            if !t.is_empty() {
                h.insert(
                    "t_token",
                    t.parse().unwrap_or_else(|_| "x".parse().unwrap()),
                );
            }
        }
        h.insert("app", "global".parse().unwrap());
        h.insert("platform", "web".parse().unwrap());
        h.insert("device-type", "Web".parse().unwrap());
        h.insert("hl", "en".parse().unwrap());
        h.insert("os", "web".parse().unwrap());
        h.insert("osv", "Mozilla/5.0".parse().unwrap());
        h.insert("ph", "Web".parse().unwrap());
        h.insert("locale", "eng".parse().unwrap());
        h
    }

    async fn resolve_account_id(
        &self,
        client: &reqwest::Client,
        creds: &Creds,
    ) -> anyhow::Result<String> {
        let url = format!("{TRADE_BASE}/api/trading/v1/global/account/list");
        let resp = client.get(&url).headers(self.headers(creds)).send().await?;
        if !resp.status().is_success() {
            anyhow::bail!("account list HTTP {}", resp.status());
        }
        let v: serde_json::Value = resp.json().await?;
        let id = v["data"][0]["secAccountId"]
            .as_i64()
            .map(|x| x.to_string())
            .or_else(|| v["data"][0]["secAccountId"].as_str().map(|s| s.into()))
            .or_else(|| v[0]["secAccountId"].as_str().map(|s| s.into()))
            .ok_or_else(|| anyhow::anyhow!("no account id in response"))?;
        Ok(id)
    }

    async fn fetch_positions(
        &self,
        client: &reqwest::Client,
        creds: &Creds,
        acct: &str,
    ) -> anyhow::Result<Vec<WPosition>> {
        let url = format!("{TRADE_BASE}/api/trading/v1/webull/account/{acct}/positions");
        let resp = client.get(&url).headers(self.headers(creds)).send().await?;
        if !resp.status().is_success() {
            anyhow::bail!("positions HTTP {}", resp.status());
        }
        let v: serde_json::Value = resp.json().await?;
        let arr = v["data"]
            .as_array()
            .or_else(|| v.as_array())
            .cloned()
            .unwrap_or_default();
        Ok(arr
            .into_iter()
            .map(|p| WPosition {
                symbol: p["ticker"]["symbol"]
                    .as_str()
                    .or(p["symbol"].as_str())
                    .unwrap_or("")
                    .into(),
                side: if num(&p["position"]) >= 0.0 {
                    "long".into()
                } else {
                    "short".into()
                },
                asset_type: p["assetType"]
                    .as_str()
                    .unwrap_or("stock")
                    .to_string()
                    .to_lowercase(),
                qty: num(&p["position"]).abs(),
                avg_cost: num(&p["costPrice"]).max(num(&p["avgCost"])),
                last_price: num(&p["lastPrice"]),
                market_value: num(&p["marketValue"]),
                unrealized_pnl: num(&p["unrealizedProfitLoss"])
                    .max(num(&p["unrealizedProfitLossBase"])),
                unrealized_pct: num(&p["unrealizedProfitLossRate"]) * 100.0,
                day_pnl: num(&p["dayProfitLoss"]),
            })
            .collect())
    }

    async fn fetch_today_orders(
        &self,
        client: &reqwest::Client,
        creds: &Creds,
        acct: &str,
    ) -> anyhow::Result<Vec<WOrder>> {
        let url = format!("{TRADE_BASE}/api/trading/v1/webull/order/list?secAccountId={acct}&status=Filled&pageSize=50");
        let resp = client.get(&url).headers(self.headers(creds)).send().await?;
        if !resp.status().is_success() {
            anyhow::bail!("orders HTTP {}", resp.status());
        }
        let v: serde_json::Value = resp.json().await?;
        let arr = v["data"]["items"]
            .as_array()
            .or_else(|| v["data"].as_array())
            .or_else(|| v.as_array())
            .cloned()
            .unwrap_or_default();
        Ok(arr
            .into_iter()
            .map(|o| WOrder {
                order_id: o["orderId"]
                    .as_str()
                    .or(o["id"].as_str())
                    .unwrap_or("")
                    .into(),
                symbol: o["ticker"]["symbol"]
                    .as_str()
                    .or(o["symbol"].as_str())
                    .unwrap_or("")
                    .into(),
                side: o["action"]
                    .as_str()
                    .unwrap_or("")
                    .to_string()
                    .to_lowercase(),
                qty: num(&o["totalQuantity"]),
                filled_qty: num(&o["filledQuantity"]),
                avg_fill_price: num(&o["avgFilledPrice"]).max(num(&o["filledPrice"])),
                status: o["status"].as_str().unwrap_or("").into(),
                created_at: parse_ms(o["placedTime"].as_str()),
                filled_at: parse_ms(o["filledTime"].as_str()),
            })
            .collect())
    }

    async fn fetch_account(
        &self,
        client: &reqwest::Client,
        creds: &Creds,
        acct: &str,
    ) -> anyhow::Result<WAccount> {
        let url = format!("{TRADE_BASE}/api/trading/v1/webull/account/{acct}");
        let resp = client.get(&url).headers(self.headers(creds)).send().await?;
        if !resp.status().is_success() {
            anyhow::bail!("account HTTP {}", resp.status());
        }
        let v: serde_json::Value = resp.json().await?;
        let d = &v["data"];
        Ok(WAccount {
            net_liquidation: num(&d["netLiquidation"]).max(num(&d["totalMarketValue"])),
            cash: num(&d["totalCash"]).max(num(&d["cashBalance"])),
            day_pnl: num(&d["dayProfitLoss"]),
            total_pnl: num(&d["unrealizedProfitLoss"]),
            buying_power: num(&d["dayBuyingPower"]).max(num(&d["overnightBuyingPower"])),
        })
    }
}

impl Default for WebullClient {
    fn default() -> Self {
        Self::new()
    }
}

fn num(v: &serde_json::Value) -> f64 {
    v.as_f64()
        .unwrap_or_else(|| v.as_str().and_then(|s| s.parse().ok()).unwrap_or(0.0))
}
fn parse_ms(s: Option<&str>) -> Option<DateTime<Utc>> {
    let s = s?;
    if let Ok(ms) = s.parse::<i64>() {
        return DateTime::<Utc>::from_timestamp_millis(ms);
    }
    chrono::DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|d| d.with_timezone(&Utc))
}

pub fn global() -> WebullClient {
    static C: once_cell::sync::OnceCell<WebullClient> = once_cell::sync::OnceCell::new();
    C.get_or_init(WebullClient::new).clone()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectRequest {
    pub did: String,
    pub access_token: String,
    #[serde(default)]
    pub t_token: Option<String>,
    #[serde(default)]
    pub account_id: Option<String>,
}
