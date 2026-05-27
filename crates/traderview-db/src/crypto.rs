//! Crypto market data via CoinGecko's free public API (no auth).
//!
//! Endpoints:
//!   * <https://api.coingecko.com/api/v3/coins/markets?vs_currency=usd&per_page=100>
//!   * <https://api.coingecko.com/api/v3/global>

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

const UA: &str = "traderview/0.1 (github.com/MenkeTechnologies/traderview)";

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent(UA)
        .timeout(std::time::Duration::from_secs(12))
        .build()
        .unwrap()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinRow {
    pub id: String,
    pub symbol: String,
    pub name: String,
    #[serde(rename = "image")]
    pub image: Option<String>,
    pub current_price: Option<f64>,
    pub market_cap: Option<f64>,
    pub market_cap_rank: Option<i32>,
    pub total_volume: Option<f64>,
    pub price_change_percentage_24h: Option<f64>,
    pub price_change_percentage_7d_in_currency: Option<f64>,
    pub circulating_supply: Option<f64>,
    pub total_supply: Option<f64>,
    pub max_supply: Option<f64>,
    pub ath: Option<f64>,
    pub ath_date: Option<String>,
    pub ath_change_percentage: Option<f64>,
    pub atl: Option<f64>,
}

pub async fn top(n: u32) -> anyhow::Result<Vec<CoinRow>> {
    let url = format!(
        "https://api.coingecko.com/api/v3/coins/markets\
         ?vs_currency=usd&order=market_cap_desc&per_page={n}&page=1\
         &sparkline=false&price_change_percentage=24h,7d"
    );
    let resp = client().get(&url).send().await?;
    if !resp.status().is_success() {
        anyhow::bail!("coingecko HTTP {}", resp.status());
    }
    Ok(resp.json().await?)
}

#[derive(Debug, Clone, Serialize)]
pub struct Global {
    pub total_market_cap_usd: f64,
    pub total_volume_usd: f64,
    pub market_cap_change_24h_pct: f64,
    pub btc_dominance: f64,
    pub eth_dominance: f64,
    pub active_cryptocurrencies: i32,
    pub upcoming_icos: i32,
    pub markets: i32,
    pub fetched_at: DateTime<Utc>,
}

pub async fn global() -> anyhow::Result<Global> {
    let url = "https://api.coingecko.com/api/v3/global";
    let resp = client().get(url).send().await?;
    if !resp.status().is_success() {
        anyhow::bail!("coingecko global HTTP {}", resp.status());
    }
    let v: serde_json::Value = resp.json().await?;
    let d = &v["data"];
    Ok(Global {
        total_market_cap_usd: d["total_market_cap"]["usd"].as_f64().unwrap_or(0.0),
        total_volume_usd: d["total_volume"]["usd"].as_f64().unwrap_or(0.0),
        market_cap_change_24h_pct: d["market_cap_change_percentage_24h_usd"].as_f64().unwrap_or(0.0),
        btc_dominance: d["market_cap_percentage"]["btc"].as_f64().unwrap_or(0.0),
        eth_dominance: d["market_cap_percentage"]["eth"].as_f64().unwrap_or(0.0),
        active_cryptocurrencies: d["active_cryptocurrencies"].as_i64().unwrap_or(0) as i32,
        upcoming_icos: d["upcoming_icos"].as_i64().unwrap_or(0) as i32,
        markets: d["markets"].as_i64().unwrap_or(0) as i32,
        fetched_at: Utc::now(),
    })
}

#[derive(Debug, Clone, Serialize)]
pub struct OnChainBtc {
    pub hash_rate_thps: Option<f64>,
    pub difficulty: Option<f64>,
    pub block_height: Option<i64>,
    pub mempool_size: Option<i64>,
    pub mempool_tx_count: Option<i64>,
    pub fetched_at: DateTime<Utc>,
}

/// Bitcoin on-chain stats via blockchain.com (no auth).
pub async fn btc_onchain() -> anyhow::Result<OnChainBtc> {
    let mut out = OnChainBtc {
        hash_rate_thps: None, difficulty: None, block_height: None,
        mempool_size: None, mempool_tx_count: None,
        fetched_at: Utc::now(),
    };
    let c = client();
    if let Ok(s) = c.get("https://blockchain.info/q/hashrate").send().await {
        if let Ok(t) = s.text().await { out.hash_rate_thps = t.trim().parse().ok(); }
    }
    if let Ok(s) = c.get("https://blockchain.info/q/getdifficulty").send().await {
        if let Ok(t) = s.text().await { out.difficulty = t.trim().parse().ok(); }
    }
    if let Ok(s) = c.get("https://blockchain.info/q/getblockcount").send().await {
        if let Ok(t) = s.text().await { out.block_height = t.trim().parse().ok(); }
    }
    if let Ok(s) = c.get("https://blockchain.info/q/unconfirmedcount").send().await {
        if let Ok(t) = s.text().await { out.mempool_tx_count = t.trim().parse().ok(); }
    }
    Ok(out)
}
