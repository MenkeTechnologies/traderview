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
        market_cap_change_24h_pct: d["market_cap_change_percentage_24h_usd"]
            .as_f64()
            .unwrap_or(0.0),
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
        hash_rate_thps: None,
        difficulty: None,
        block_height: None,
        mempool_size: None,
        mempool_tx_count: None,
        fetched_at: Utc::now(),
    };
    let c = client();
    if let Ok(s) = c.get("https://blockchain.info/q/hashrate").send().await {
        if let Ok(t) = s.text().await {
            out.hash_rate_thps = t.trim().parse().ok();
        }
    }
    if let Ok(s) = c
        .get("https://blockchain.info/q/getdifficulty")
        .send()
        .await
    {
        if let Ok(t) = s.text().await {
            out.difficulty = t.trim().parse().ok();
        }
    }
    if let Ok(s) = c
        .get("https://blockchain.info/q/getblockcount")
        .send()
        .await
    {
        if let Ok(t) = s.text().await {
            out.block_height = t.trim().parse().ok();
        }
    }
    if let Ok(s) = c
        .get("https://blockchain.info/q/unconfirmedcount")
        .send()
        .await
    {
        if let Ok(t) = s.text().await {
            out.mempool_tx_count = t.trim().parse().ok();
        }
    }
    Ok(out)
}


// ─── OKX perp funding (live) ────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize)]
pub struct FundingSnapshot {
    pub inst_id: String,
    pub spot: f64,
    pub perp: f64,
    /// Raw rate for the venue's CURRENT funding interval.
    pub funding_rate_interval: f64,
    pub interval_hours: f64,
    /// Normalized to the 8h convention the arb calculator uses:
    /// rate × (8 / interval_hours). OKX runs variable intervals.
    pub funding_rate_8h: f64,
    pub next_funding_time_ms: i64,
}

async fn okx_json(url: &str) -> anyhow::Result<serde_json::Value> {
    let v: serde_json::Value = reqwest::Client::builder()
        .user_agent("traderview")
        .timeout(std::time::Duration::from_secs(10))
        .build()?
        .get(url)
        .send()
        .await?
        .json()
        .await?;
    if v.get("code").and_then(|c| c.as_str()) != Some("0") {
        anyhow::bail!("okx error: {}", v.get("msg").and_then(|m| m.as_str()).unwrap_or("?"));
    }
    Ok(v)
}

fn okx_f64(v: &serde_json::Value, path: &[&str]) -> Option<f64> {
    let mut cur = v;
    for p in path {
        cur = cur.get(p)?;
    }
    cur.as_str()?.parse().ok()
}

/// Live funding snapshot for one base asset (e.g. "BTC") from OKX
/// public endpoints — spot ticker, perp ticker, current funding rate.
/// Verified reachable from this deployment; Binance/Bybit are not.
pub async fn funding_snapshot(base: &str) -> anyhow::Result<FundingSnapshot> {
    let base = base.trim().to_uppercase();
    if base.is_empty() || base.len() > 10 || !base.bytes().all(|b| b.is_ascii_alphanumeric()) {
        anyhow::bail!("invalid base asset");
    }
    let spot_id = format!("{base}-USDT");
    let swap_id = format!("{base}-USDT-SWAP");
    let spot = okx_json(&format!(
        "https://www.okx.com/api/v5/market/ticker?instId={spot_id}"
    ))
    .await?;
    let perp = okx_json(&format!(
        "https://www.okx.com/api/v5/market/ticker?instId={swap_id}"
    ))
    .await?;
    let funding = okx_json(&format!(
        "https://www.okx.com/api/v5/public/funding-rate?instId={swap_id}"
    ))
    .await?;
    let d0 = |v: &serde_json::Value| v.get("data").and_then(|d| d.get(0)).cloned();
    let (Some(spot_d), Some(perp_d), Some(fund_d)) = (d0(&spot), d0(&perp), d0(&funding)) else {
        anyhow::bail!("{base}: empty OKX response (unlisted asset?)");
    };
    let spot_px = okx_f64(&spot_d, &["last"]).ok_or_else(|| anyhow::anyhow!("no spot last"))?;
    let perp_px = okx_f64(&perp_d, &["last"]).ok_or_else(|| anyhow::anyhow!("no perp last"))?;
    let rate = okx_f64(&fund_d, &["fundingRate"]).ok_or_else(|| anyhow::anyhow!("no funding rate"))?;
    let t0: i64 = fund_d
        .get("fundingTime")
        .and_then(|x| x.as_str())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let t1: i64 = fund_d
        .get("nextFundingTime")
        .and_then(|x| x.as_str())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    // Interval from the venue's own timestamps; 8h fallback when absent.
    let interval_hours = if t1 > t0 {
        (t1 - t0) as f64 / 3_600_000.0
    } else {
        8.0
    };
    Ok(FundingSnapshot {
        inst_id: swap_id,
        spot: spot_px,
        perp: perp_px,
        funding_rate_interval: rate,
        interval_hours,
        funding_rate_8h: rate * 8.0 / interval_hours,
        next_funding_time_ms: t1,
    })
}


/// The scan universe — major USDT perps on OKX. A curated list like
/// CARRY_UNIVERSE: additions welcome, but every entry must actually
/// list on the venue (unlisted bases just report as errors).
pub const FUNDING_UNIVERSE: &[&str] = &[
    "BTC", "ETH", "SOL", "XRP", "DOGE", "ADA", "AVAX", "LINK", "DOT", "LTC", "BCH", "TON",
];

#[derive(Debug, Clone, serde::Serialize)]
pub struct FundingScanRow {
    pub base: String,
    pub funding_rate_8h: f64,
    pub funding_apr_pct: f64,
    pub basis_pct: f64,
    pub spot: f64,
    pub perp: f64,
    pub interval_hours: f64,
    /// Realized-funding regime over the last ~30 intervals — ranks a
    /// steady carry above an equally-rich one-interval spike. None
    /// when the history fetch failed (the row still lists).
    pub persistence: Option<FundingPersistence>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct FundingScan {
    /// Ranked by |APR| descending — the richest carry first, either
    /// direction (negative funding pays the long side).
    pub rows: Vec<FundingScanRow>,
    /// Bases that failed to quote — reported, never silently dropped.
    pub failed: Vec<String>,
}

/// Concurrent sweep of the funding universe. The snapshot and the
/// ~30-interval history fetch run concurrently per base (one join_all
/// over both, not a second sequential sweep); a failed history
/// degrades that row's persistence to None rather than failing it.
pub async fn funding_scan() -> FundingScan {
    let futs = FUNDING_UNIVERSE.iter().map(|base| async move {
        let (snap, hist) =
            futures_util::future::join(funding_snapshot(base), funding_history(base, 30)).await;
        (base.to_string(), snap, hist)
    });
    let results = futures_util::future::join_all(futs).await;
    let mut rows = Vec::new();
    let mut failed = Vec::new();
    for (base, res, hist) in results {
        match res {
            Ok(s) => rows.push(FundingScanRow {
                base,
                funding_rate_8h: s.funding_rate_8h,
                funding_apr_pct: s.funding_rate_8h * 3.0 * 365.0 * 100.0,
                basis_pct: (s.perp / s.spot - 1.0) * 100.0,
                spot: s.spot,
                perp: s.perp,
                interval_hours: s.interval_hours,
                persistence: hist.ok().as_deref().and_then(funding_persistence),
            }),
            Err(e) => failed.push(format!("{base}: {e}")),
        }
    }
    rows.sort_by(|a, b| b.funding_apr_pct.abs().total_cmp(&a.funding_apr_pct.abs()));
    FundingScan { rows, failed }
}


#[derive(Debug, Clone, serde::Serialize)]
pub struct FundingPersistence {
    /// Realized intervals examined (venue-capped).
    pub intervals: usize,
    /// Mean realized rate per interval (raw, not 8h-normalized — the
    /// trend question is sign and stability, not APR units).
    pub mean_rate: f64,
    /// Share of intervals whose sign matches the LATEST rate — the
    /// regime question: 0.9 = persistent, 0.5 = coin-flip noise.
    pub same_sign_as_latest_pct: f64,
}

/// Persistence stats over realized funding rates (newest first, as
/// the venue returns them). None for an empty history. Zero-rate
/// intervals count as matching nothing (they pay nobody).
pub fn funding_persistence(rates_newest_first: &[f64]) -> Option<FundingPersistence> {
    let latest = *rates_newest_first.first()?;
    let n = rates_newest_first.len();
    let mean_rate = rates_newest_first.iter().sum::<f64>() / n as f64;
    let same = rates_newest_first
        .iter()
        .filter(|r| latest != 0.0 && r.signum() == latest.signum() && **r != 0.0)
        .count();
    Some(FundingPersistence {
        intervals: n,
        mean_rate,
        same_sign_as_latest_pct: same as f64 / n as f64,
    })
}

/// Realized funding history from OKX, newest first.
pub async fn funding_history(base: &str, limit: u32) -> anyhow::Result<Vec<f64>> {
    let base = base.trim().to_uppercase();
    if base.is_empty() || base.len() > 10 || !base.bytes().all(|b| b.is_ascii_alphanumeric()) {
        anyhow::bail!("invalid base asset");
    }
    let v = okx_json(&format!(
        "https://www.okx.com/api/v5/public/funding-rate-history?instId={base}-USDT-SWAP&limit={}",
        limit.clamp(1, 100)
    ))
    .await?;
    let rows = v
        .get("data")
        .and_then(|d| d.as_array())
        .cloned()
        .unwrap_or_default();
    Ok(rows
        .iter()
        .filter_map(|r| {
            r.get("realizedRate")
                .or_else(|| r.get("fundingRate"))
                .and_then(|x| x.as_str())
                .and_then(|s| s.parse().ok())
        })
        .collect())
}

#[cfg(test)]
mod funding_tests {
    use super::*;

    #[test]
    fn persistence_pins_regime_vs_noise() {
        // 4/5 negative matching a negative latest: persistent.
        let p = funding_persistence(&[-0.0002, -0.0001, 0.0001, -0.0003, -0.0002]).unwrap();
        assert_eq!(p.intervals, 5);
        assert!((p.same_sign_as_latest_pct - 0.8).abs() < 1e-12);
        assert!((p.mean_rate + 0.00014).abs() < 1e-9);
        // Alternating signs: half match — coin-flip, not a regime.
        let p = funding_persistence(&[0.0001, -0.0001, 0.0001, -0.0001]).unwrap();
        assert!((p.same_sign_as_latest_pct - 0.5).abs() < 1e-12);
        // Zero latest matches nothing; empty history is None.
        let p = funding_persistence(&[0.0, 0.0001]).unwrap();
        assert_eq!(p.same_sign_as_latest_pct, 0.0);
        assert!(funding_persistence(&[]).is_none());
    }
}
