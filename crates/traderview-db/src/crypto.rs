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
    funding_scan_for(FUNDING_UNIVERSE.iter().map(|s| s.to_string()).collect()).await
}

/// Same sweep over a CALLER-CHOSEN universe — the 12 majors miss alt
/// perps. Bases are validated and deduped; invalid tokens land in
/// `failed` by name rather than being silently dropped; capped at 20
/// (each base is ~4 venue calls).
pub async fn funding_scan_for(bases: Vec<String>) -> FundingScan {
    let mut seen = std::collections::BTreeSet::new();
    let mut valid = Vec::new();
    let mut failed = Vec::new();
    for b in bases {
        let b = b.trim().to_uppercase();
        if b.is_empty() || !seen.insert(b.clone()) {
            continue;
        }
        if b.len() > 10 || !b.bytes().all(|c| c.is_ascii_alphanumeric()) {
            failed.push(format!("{b}: invalid base asset"));
        } else if valid.len() < 20 {
            valid.push(b);
        } else {
            failed.push(format!("{b}: over the 20-base cap"));
        }
    }
    let futs = valid.iter().map(|base| async move {
        let (snap, hist) =
            futures_util::future::join(funding_snapshot(base), funding_history(base, 30)).await;
        (base.to_string(), snap, hist)
    });
    let results = futures_util::future::join_all(futs).await;
    let mut rows = Vec::new();
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


/// OI × price quadrant — the classic positioning read. Both rising:
/// new money agreeing with the move. Price up on falling OI: shorts
/// closing, not buyers arriving. Price down on rising OI: new shorts
/// pressing. Both falling: longs liquidating. Changes inside ±EPS%
/// are noise, read as flat.
pub fn oi_price_quadrant(price_chg_pct: f64, oi_chg_pct: f64) -> &'static str {
    const EPS: f64 = 0.05;
    let (p, o) = (price_chg_pct, oi_chg_pct);
    if p.abs() < EPS || o.abs() < EPS {
        "flat"
    } else if p > 0.0 && o > 0.0 {
        "new_longs"
    } else if p > 0.0 {
        "short_covering"
    } else if o > 0.0 {
        "new_shorts"
    } else {
        "long_liquidation"
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct LiquidationPressure {
    /// Span covered by the venue's recent-liquidations feed, oldest
    /// record to now — the feed is "most recent N", not a fixed
    /// window, so the span is reported rather than assumed.
    pub window_minutes: i64,
    pub longs_liquidated: usize,
    pub shorts_liquidated: usize,
    /// Σ contracts × ctVal × bankruptcy price — USD terms.
    pub long_notional_usd: f64,
    pub short_notional_usd: f64,
}

/// Aggregate the venue's recent filled liquidations. rows = (is_long,
/// contracts, bankruptcy_px) per order; ct_val converts contracts to
/// base units. Heavy long liquidations = a cascade down; heavy short
/// = a squeeze up.
pub fn liquidation_pressure(
    rows: &[(bool, f64, f64)],
    ct_val: f64,
    oldest_ts_ms: i64,
    now_ms: i64,
) -> LiquidationPressure {
    let (mut lc, mut sc, mut ln, mut sn) = (0usize, 0usize, 0.0, 0.0);
    for (long, sz, px) in rows {
        let notional = sz * ct_val * px;
        if *long {
            lc += 1;
            ln += notional;
        } else {
            sc += 1;
            sn += notional;
        }
    }
    LiquidationPressure {
        window_minutes: ((now_ms - oldest_ts_ms).max(0)) / 60_000,
        longs_liquidated: lc,
        shorts_liquidated: sc,
        long_notional_usd: ln,
        short_notional_usd: sn,
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Positioning {
    pub inst_id: String,
    pub price_last: f64,
    pub price_chg_24h_pct: f64,
    /// Open interest across the USDT swap, USD terms.
    pub oi_usd: f64,
    /// 24h OI change from the venue's daily OI series (all {base}
    /// contracts, a broader scope than the single swap's level — the
    /// DIRECTION is the signal, not the level match). None when the
    /// series is too short.
    pub oi_chg_24h_pct: Option<f64>,
    pub quadrant: Option<&'static str>,
    /// Long/short ACCOUNT ratio (1.4 = 40% more long accounts than
    /// short) — retail positioning, newest hour.
    pub long_short_ratio: Option<f64>,
    pub long_short_ratio_24h_ago: Option<f64>,
    /// Taker buy share of the last 24 hourly buckets — aggressor flow.
    pub taker_buy_share_24h_pct: Option<f64>,
    pub funding_rate: Option<f64>,
    /// Recent forced liquidations — who is being carried out. None
    /// when either the liquidation feed or the contract spec is dark.
    pub liquidations: Option<LiquidationPressure>,
}

fn rubik_rows(v: &serde_json::Value) -> Vec<Vec<f64>> {
    v.get("data")
        .and_then(|d| d.as_array())
        .map(|rows| {
            rows.iter()
                .filter_map(|r| {
                    r.as_array().map(|xs| {
                        xs.iter()
                            .filter_map(|x| x.as_str().and_then(|s| s.parse().ok()))
                            .collect()
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Live positioning dashboard: who is positioned where, fetched
/// concurrently. Ticker + open interest are required; the rubik
/// stats (ratio, taker flow, OI history) and funding are enrichment
/// — each degrades to None independently so one dark endpoint
/// doesn't kill the read.
pub async fn positioning(base: &str) -> anyhow::Result<Positioning> {
    let base = base.trim().to_uppercase();
    if base.is_empty() || base.len() > 10 || !base.bytes().all(|b| b.is_ascii_alphanumeric()) {
        anyhow::bail!("invalid base asset");
    }
    let inst = format!("{base}-USDT-SWAP");
    let u_ticker = format!("https://www.okx.com/api/v5/market/ticker?instId={inst}");
    let u_oi = format!("https://www.okx.com/api/v5/public/open-interest?instId={inst}");
    let u_oi_hist = format!(
        "https://www.okx.com/api/v5/rubik/stat/contracts/open-interest-volume?ccy={base}&period=1D"
    );
    let u_ls = format!(
        "https://www.okx.com/api/v5/rubik/stat/contracts/long-short-account-ratio?ccy={base}&period=1H"
    );
    let u_taker = format!(
        "https://www.okx.com/api/v5/rubik/stat/taker-volume?ccy={base}&instType=CONTRACTS&period=1H"
    );
    let u_funding = format!("https://www.okx.com/api/v5/public/funding-rate?instId={inst}");
    let u_liq = format!(
        "https://www.okx.com/api/v5/public/liquidation-orders?instType=SWAP&uly={base}-USDT&state=filled"
    );
    let u_spec = format!("https://www.okx.com/api/v5/public/instruments?instType=SWAP&instId={inst}");
    let (ticker, oi, oi_hist, ls, taker, funding, liq, spec) = futures_util::join!(
        okx_json(&u_ticker),
        okx_json(&u_oi),
        okx_json(&u_oi_hist),
        okx_json(&u_ls),
        okx_json(&u_taker),
        okx_json(&u_funding),
        okx_json(&u_liq),
        okx_json(&u_spec),
    );
    let ticker = ticker?;
    let last = okx_f64(&ticker, &["data", "0", "last"])
        .ok_or_else(|| anyhow::anyhow!("no last price"))?;
    let open24h = okx_f64(&ticker, &["data", "0", "open24h"])
        .ok_or_else(|| anyhow::anyhow!("no open24h"))?;
    let price_chg_24h_pct = (last / open24h - 1.0) * 100.0;
    let oi = oi?;
    let oi_usd = okx_f64(&oi, &["data", "0", "oiUsd"])
        .ok_or_else(|| anyhow::anyhow!("no open interest"))?;
    // Daily OI series, newest first: [ts, oiUsd, volUsd].
    let oi_chg_24h_pct = oi_hist.ok().map(|v| rubik_rows(&v)).and_then(|rows| {
        let now = rows.first().and_then(|r| r.get(1)).copied()?;
        let prev = rows.get(1).and_then(|r| r.get(1)).copied()?;
        (prev != 0.0).then(|| (now / prev - 1.0) * 100.0)
    });
    let ls_rows = ls.ok().map(|v| rubik_rows(&v)).unwrap_or_default();
    let long_short_ratio = ls_rows.first().and_then(|r| r.get(1)).copied();
    let long_short_ratio_24h_ago = ls_rows.get(24).and_then(|r| r.get(1)).copied();
    // Taker volume, newest first: [ts, sellVol, buyVol] per the venue
    // docs — sell FIRST. Sum the last 24 hourly buckets.
    let taker_buy_share_24h_pct = taker.ok().map(|v| rubik_rows(&v)).and_then(|rows| {
        let (mut sell, mut buy) = (0.0, 0.0);
        for r in rows.iter().take(24) {
            sell += r.get(1).copied().unwrap_or(0.0);
            buy += r.get(2).copied().unwrap_or(0.0);
        }
        (sell + buy > 0.0).then(|| buy / (sell + buy) * 100.0)
    });
    Ok(Positioning {
        inst_id: inst,
        price_last: last,
        price_chg_24h_pct,
        oi_usd,
        quadrant: oi_chg_24h_pct.map(|o| oi_price_quadrant(price_chg_24h_pct, o)),
        oi_chg_24h_pct,
        long_short_ratio,
        long_short_ratio_24h_ago,
        taker_buy_share_24h_pct,
        funding_rate: funding.ok().and_then(|v| okx_f64(&v, &["data", "0", "fundingRate"])),
        liquidations: (|| {
            let ct_val = okx_f64(&spec.ok()?, &["data", "0", "ctVal"]).filter(|v| *v > 0.0)?;
            let liq = liq.ok()?;
            let details = liq
                .get("data")
                .and_then(|d| d.as_array())
                .and_then(|arr| arr.first())
                .and_then(|r| r.get("details"))
                .and_then(|d| d.as_array())?;
            let mut rows = Vec::new();
            let mut oldest = i64::MAX;
            for o in details {
                let long = o.get("posSide").and_then(|v| v.as_str()) == Some("long");
                let sz: f64 = o.get("sz").and_then(|v| v.as_str()).and_then(|s| s.parse().ok())?;
                let px: f64 = o.get("bkPx").and_then(|v| v.as_str()).and_then(|s| s.parse().ok())?;
                let ts: i64 = o.get("ts").and_then(|v| v.as_str()).and_then(|s| s.parse().ok())?;
                oldest = oldest.min(ts);
                rows.push((long, sz, px));
            }
            (!rows.is_empty()).then(|| {
                liquidation_pressure(&rows, ct_val, oldest, chrono::Utc::now().timestamp_millis())
            })
        })(),
    })
}

#[cfg(test)]
mod positioning_tests {
    use super::*;

    #[test]
    fn liquidation_pressure_splits_and_converts() {
        // Two long liqs (1.04 + 3.8 contracts) + one short, ctVal
        // 0.01 — the live-observed BTC swap spec. Notional = sz ×
        // 0.01 × bankruptcy px.
        let rows = [
            (true, 1.04, 62_912.5),
            (true, 3.8, 62_810.1),
            (false, 2.0, 63_000.0),
        ];
        let p = liquidation_pressure(&rows, 0.01, 1_781_246_333_927, 1_781_246_933_927);
        assert_eq!(p.longs_liquidated, 2);
        assert_eq!(p.shorts_liquidated, 1);
        assert!((p.long_notional_usd - (1.04 * 0.01 * 62_912.5 + 3.8 * 0.01 * 62_810.1)).abs() < 1e-6);
        assert!((p.short_notional_usd - 2.0 * 0.01 * 63_000.0).abs() < 1e-6);
        // 600 000 ms span = 10 minutes.
        assert_eq!(p.window_minutes, 10);
    }

    #[test]
    fn quadrant_pins_all_five_reads() {
        assert_eq!(oi_price_quadrant(2.0, 3.0), "new_longs");
        assert_eq!(oi_price_quadrant(2.0, -3.0), "short_covering");
        assert_eq!(oi_price_quadrant(-2.0, 3.0), "new_shorts");
        assert_eq!(oi_price_quadrant(-2.0, -3.0), "long_liquidation");
        // Sub-noise moves on EITHER axis read flat, both boundaries.
        assert_eq!(oi_price_quadrant(0.01, 5.0), "flat");
        assert_eq!(oi_price_quadrant(5.0, -0.01), "flat");
    }
}


/// Expiry date from an OKX dated-futures instId tail ("BTC-USDT-260626"
/// → 2026-06-26). The tail is YYMMDD; anything else is None.
pub fn parse_okx_expiry(inst_id: &str) -> Option<chrono::NaiveDate> {
    let tail = inst_id.rsplit('-').next()?;
    if tail.len() != 6 || !tail.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let yy: i32 = tail[..2].parse().ok()?;
    let mm: u32 = tail[2..4].parse().ok()?;
    let dd: u32 = tail[4..6].parse().ok()?;
    chrono::NaiveDate::from_ymd_opt(2000 + yy, mm, dd)
}

/// Simple (non-compounded) annualized basis: (fut/spot − 1) × 365/days,
/// in percent. None when inputs can't produce a meaningful number —
/// non-positive prices, or zero/negative days (an expiring-today
/// contract's basis annualizes to noise, not signal).
pub fn annualized_basis_pct(spot: f64, fut: f64, days_to_expiry: i64) -> Option<f64> {
    if spot <= 0.0 || fut <= 0.0 || days_to_expiry <= 0 {
        return None;
    }
    Some((fut / spot - 1.0) * 365.0 / days_to_expiry as f64 * 100.0)
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CarryLeg {
    pub inst_id: String,
    pub expiry: chrono::NaiveDate,
    pub days_to_expiry: i64,
    pub fut_price: f64,
    /// Raw basis to expiry, percent.
    pub basis_pct: f64,
    /// Simple annualization — the comparable carry number.
    pub annualized_pct: f64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CarryBasis {
    pub spot: f64,
    pub legs: Vec<CarryLeg>,
    /// Dated contracts listed but unusable (no parsable expiry,
    /// expiring today, bad price) — listed, not silently dropped.
    pub skipped: Vec<String>,
}

/// Live cash-and-carry sheet: spot vs every dated future on the
/// underlying. Positive annualized basis = contango — buy spot,
/// short the future, capture the convergence; the perp version of
/// this trade is the funding-arb tool, this is the locked-term one.
pub async fn carry_basis(base: &str) -> anyhow::Result<CarryBasis> {
    let base = base.trim().to_uppercase();
    if base.is_empty() || base.len() > 10 || !base.bytes().all(|b| b.is_ascii_alphanumeric()) {
        anyhow::bail!("invalid base asset");
    }
    let u_spot = format!("https://www.okx.com/api/v5/market/ticker?instId={base}-USDT");
    let u_futs = format!("https://www.okx.com/api/v5/market/tickers?instType=FUTURES&uly={base}-USDT");
    let (spot_v, futs_v) = futures_util::join!(okx_json(&u_spot), okx_json(&u_futs));
    let spot = okx_f64(&spot_v?, &["data", "0", "last"])
        .filter(|p| *p > 0.0)
        .ok_or_else(|| anyhow::anyhow!("no spot price"))?;
    let futs = futs_v?;
    let today = chrono::Utc::now().date_naive();
    let (mut legs, mut skipped) = (Vec::new(), Vec::new());
    for row in futs.get("data").and_then(|d| d.as_array()).into_iter().flatten() {
        let inst_id = row.get("instId").and_then(|v| v.as_str()).unwrap_or_default().to_string();
        let last = row.get("last").and_then(|v| v.as_str()).and_then(|s| s.parse::<f64>().ok());
        let leg = parse_okx_expiry(&inst_id).and_then(|expiry| {
            let days = (expiry - today).num_days();
            let fut = last?;
            let annualized = annualized_basis_pct(spot, fut, days)?;
            Some(CarryLeg {
                inst_id: inst_id.clone(),
                expiry,
                days_to_expiry: days,
                fut_price: fut,
                basis_pct: (fut / spot - 1.0) * 100.0,
                annualized_pct: annualized,
            })
        });
        match leg {
            Some(l) => legs.push(l),
            None => skipped.push(inst_id),
        }
    }
    legs.sort_by_key(|l| l.days_to_expiry);
    Ok(CarryBasis { spot, legs, skipped })
}

#[cfg(test)]
mod carry_tests {
    use super::*;

    #[test]
    fn expiry_parse_and_basis_annualization() {
        // Live-observed shape: BTC-USDT-260626 → 2026-06-26.
        assert_eq!(
            parse_okx_expiry("BTC-USDT-260626"),
            chrono::NaiveDate::from_ymd_opt(2026, 6, 26)
        );
        // Perps and spot ids have no date tail.
        assert_eq!(parse_okx_expiry("BTC-USDT-SWAP"), None);
        assert_eq!(parse_okx_expiry("BTC-USDT"), None);
        // 1% basis over 73 days = 5% annualized, exactly 365/73 = 5.
        let a = annualized_basis_pct(100.0, 101.0, 73).unwrap();
        assert!((a - 5.0).abs() < 1e-9);
        // Backwardation annualizes negative.
        assert!(annualized_basis_pct(100.0, 99.0, 73).unwrap() < 0.0);
        // Expiring today / bad prices: None, not infinity or noise.
        assert_eq!(annualized_basis_pct(100.0, 101.0, 0), None);
        assert_eq!(annualized_basis_pct(0.0, 101.0, 30), None);
    }
}


/// Parse an OKX option instId tail: "BTC-USD_UM-260619-63500-P" →
/// (expiry, strike, is_call). The family prefix varies (BTC-USD vs
/// BTC-USD_UM), so parse from the END like the OCC parser does.
pub fn parse_okx_option_id(inst_id: &str) -> Option<(chrono::NaiveDate, f64, bool)> {
    let mut parts = inst_id.rsplit('-');
    let right = match parts.next()? {
        "C" => true,
        "P" => false,
        _ => return None,
    };
    let strike: f64 = parts.next()?.parse().ok()?;
    let date = parts.next()?;
    if date.len() != 6 || !date.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let yy: i32 = date[..2].parse().ok()?;
    let mm: u32 = date[2..4].parse().ok()?;
    let dd: u32 = date[4..6].parse().ok()?;
    let expiry = chrono::NaiveDate::from_ymd_opt(2000 + yy, mm, dd)?;
    (strike > 0.0).then_some((expiry, strike, right))
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct VolExpiry {
    pub expiry: chrono::NaiveDate,
    pub days: i64,
    /// Mark IV of the contract whose |delta| sits closest to 0.50.
    pub atm_iv_pct: f64,
    /// 25-delta risk reversal: call IV − put IV at |delta| ≈ 0.25.
    /// Positive = upside bid (calls richer), negative = downside fear.
    /// None when either wing has no contract within 0.10 of the
    /// target delta — a skew read off the wrong strikes is worse
    /// than no read.
    pub rr25_pct: Option<f64>,
    pub contracts: usize,
}

/// Per-expiry vol summary from (expiry, days, |delta|, is_call,
/// mark_iv) rows. Pure; rows with non-finite IV/delta are the
/// caller's to filter.
pub fn vol_surface(rows: &[(chrono::NaiveDate, i64, f64, bool, f64)]) -> Vec<VolExpiry> {
    let mut by_expiry: std::collections::BTreeMap<chrono::NaiveDate, Vec<&(chrono::NaiveDate, i64, f64, bool, f64)>> =
        Default::default();
    for r in rows {
        by_expiry.entry(r.0).or_default().push(r);
    }
    by_expiry
        .into_iter()
        .filter_map(|(expiry, legs)| {
            let days = legs.first()?.1;
            let atm = legs
                .iter()
                .min_by(|a, b| (a.2 - 0.5).abs().total_cmp(&(b.2 - 0.5).abs()))?;
            let wing = |call: bool| {
                legs.iter()
                    .filter(|l| l.3 == call)
                    .min_by(|a, b| (a.2 - 0.25).abs().total_cmp(&(b.2 - 0.25).abs()))
                    .filter(|l| (l.2 - 0.25).abs() <= 0.10)
            };
            let rr25_pct = match (wing(true), wing(false)) {
                (Some(c), Some(p)) => Some((c.4 - p.4) * 100.0),
                _ => None,
            };
            Some(VolExpiry {
                expiry,
                days,
                atm_iv_pct: atm.4 * 100.0,
                rr25_pct,
                contracts: legs.len(),
            })
        })
        .collect()
}

/// Live vol surface for {base}-USD options: ATM IV term structure +
/// 25-delta risk reversal per expiry, from the venue's opt-summary
/// (mark vol + BS delta per contract).
pub async fn crypto_vol_surface(base: &str) -> anyhow::Result<Vec<VolExpiry>> {
    let base = base.trim().to_uppercase();
    if base.is_empty() || base.len() > 10 || !base.bytes().all(|b| b.is_ascii_alphanumeric()) {
        anyhow::bail!("invalid base asset");
    }
    let u = format!("https://www.okx.com/api/v5/public/opt-summary?uly={base}-USD");
    let v = okx_json(&u).await?;
    let today = chrono::Utc::now().date_naive();
    let mut rows = Vec::new();
    for o in v.get("data").and_then(|d| d.as_array()).into_iter().flatten() {
        let inst = o.get("instId").and_then(|x| x.as_str()).unwrap_or_default();
        let Some((expiry, strike, call)) = parse_okx_option_id(inst) else {
            continue;
        };
        let _ = strike;
        let iv: f64 = match o.get("markVol").and_then(|x| x.as_str()).and_then(|s| s.parse().ok()) {
            Some(x) if x > 0.0 => x,
            _ => continue,
        };
        let delta: f64 = match o.get("deltaBS").and_then(|x| x.as_str()).and_then(|s| s.parse().ok()) {
            Some(x) => x,
            None => continue,
        };
        let days = (expiry - today).num_days();
        if days <= 0 {
            continue;
        }
        rows.push((expiry, days, delta.abs(), call, iv));
    }
    if rows.is_empty() {
        anyhow::bail!("no live option marks for {base}-USD");
    }
    Ok(vol_surface(&rows))
}

#[cfg(test)]
mod vol_tests {
    use super::*;

    #[test]
    fn option_id_parse_and_surface_reads() {
        // Live-observed id shape.
        let (exp, k, call) = parse_okx_option_id("BTC-USD_UM-260619-63500-P").unwrap();
        assert_eq!(exp, chrono::NaiveDate::from_ymd_opt(2026, 6, 19).unwrap());
        assert_eq!(k, 63_500.0);
        assert!(!call);
        assert!(parse_okx_option_id("BTC-USD-SWAP").is_none());

        let e = chrono::NaiveDate::from_ymd_opt(2026, 6, 19).unwrap();
        // ATM = |delta| nearest 0.50; RR25 = call IV − put IV at the
        // 0.25 wings: 42% − 48% = −6 points (downside fear).
        let rows = vec![
            (e, 7, 0.50, true, 0.40),
            (e, 7, 0.26, true, 0.42),
            (e, 7, 0.24, false, 0.48),
            (e, 7, 0.10, true, 0.55),
        ];
        let s = vol_surface(&rows);
        assert_eq!(s.len(), 1);
        assert!((s[0].atm_iv_pct - 40.0).abs() < 1e-9);
        assert!((s[0].rr25_pct.unwrap() + 6.0).abs() < 1e-9);
        // A wing with nothing near 0.25 delta REFUSES the skew read.
        let rows = vec![(e, 7, 0.50, true, 0.40), (e, 7, 0.45, false, 0.44)];
        let s = vol_surface(&rows);
        assert!(s[0].rr25_pct.is_none());
    }
}


/// Close-to-close annualized realized vol from daily closes (oldest
/// first), √365 — crypto trades every day, not 252. Sample stdev of
/// log returns; None under 10 returns (a week of data stated as an
/// annualized vol is noise wearing units).
pub fn annualized_realized_vol(closes_oldest_first: &[f64]) -> Option<f64> {
    if closes_oldest_first.len() < 11 || closes_oldest_first.iter().any(|c| *c <= 0.0) {
        return None;
    }
    let rets: Vec<f64> = closes_oldest_first
        .windows(2)
        .map(|w| (w[1] / w[0]).ln())
        .collect();
    let n = rets.len() as f64;
    let mean = rets.iter().sum::<f64>() / n;
    let var = rets.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (n - 1.0);
    Some(var.sqrt() * 365.0_f64.sqrt())
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CryptoVrp {
    /// 30-day close-to-close realized vol, annualized √365.
    pub realized_vol_pct: f64,
    /// Front ATM implied (nearest expiry ≥ 7 days — sub-week ATM IV
    /// is pin risk and event noise, not a vol level).
    pub implied_vol_pct: f64,
    pub implied_expiry: chrono::NaiveDate,
    pub iv_over_rv: f64,
    /// Volatility-points spread (IV − RV) — what the seller is paid.
    pub premium_pct: f64,
}

/// Live variance-risk-premium read: the venue's option marks vs the
/// asset's own recent movement. IV/RV > 1 = options rich (the
/// documented seller edge, on average); < 1 = options cheap.
pub async fn crypto_vrp(base: &str) -> anyhow::Result<CryptoVrp> {
    let base_up = base.trim().to_uppercase();
    if base_up.is_empty() || base_up.len() > 10 || !base_up.bytes().all(|b| b.is_ascii_alphanumeric()) {
        anyhow::bail!("invalid base asset");
    }
    let u_candles =
        format!("https://www.okx.com/api/v5/market/candles?instId={base_up}-USDT&bar=1D&limit=31");
    let (candles, surface) = futures_util::join!(okx_json(&u_candles), crypto_vol_surface(&base_up));
    // Candles arrive NEWEST first: [ts, o, h, l, c, ...].
    let mut closes: Vec<f64> = candles?
        .get("data")
        .and_then(|d| d.as_array())
        .into_iter()
        .flatten()
        .filter_map(|row| row.get(4).and_then(|c| c.as_str()).and_then(|s| s.parse().ok()))
        .collect();
    closes.reverse();
    let realized = annualized_realized_vol(&closes)
        .ok_or_else(|| anyhow::anyhow!("not enough daily closes for a realized vol"))?;
    let surface = surface?;
    let front = surface
        .iter()
        .filter(|e| e.days >= 7)
        .min_by_key(|e| e.days)
        .ok_or_else(|| anyhow::anyhow!("no option expiry ≥ 7 days out"))?;
    let implied = front.atm_iv_pct / 100.0;
    Ok(CryptoVrp {
        realized_vol_pct: realized * 100.0,
        implied_vol_pct: front.atm_iv_pct,
        implied_expiry: front.expiry,
        iv_over_rv: implied / realized,
        premium_pct: front.atm_iv_pct - realized * 100.0,
    })
}

#[cfg(test)]
mod vrp_tests {
    use super::*;

    #[test]
    fn realized_vol_pins_v365_and_floor() {
        // Alternating ±1% daily moves: log returns ±ln(1.01)/ln(0.99…)
        // — compute the expected value directly from the same series
        // rather than asserting a magic constant.
        let mut closes = vec![100.0_f64];
        for i in 0..30 {
            let last = *closes.last().unwrap();
            closes.push(if i % 2 == 0 { last * 1.01 } else { last * 0.99 });
        }
        let rets: Vec<f64> = closes.windows(2).map(|w| (w[1] / w[0]).ln()).collect();
        let n = rets.len() as f64;
        let mean = rets.iter().sum::<f64>() / n;
        let var = rets.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (n - 1.0);
        let expect = var.sqrt() * 365.0_f64.sqrt();
        let got = annualized_realized_vol(&closes).unwrap();
        assert!((got - expect).abs() < 1e-12);
        // ~19% annualized for ±1% dailies at √365 — sanity band, not
        // a hand-waved constant: 0.01 × √365 ≈ 0.191.
        assert!((got - 0.191).abs() < 0.01);
        // Sample-size floor and bad inputs.
        assert!(annualized_realized_vol(&closes[..10]).is_none());
        assert!(annualized_realized_vol(&[100.0, 0.0, 100.0]).is_none());
    }
}


/// Is this a crypto spot pair the paper engine can quote? Strictly
/// BASE-USDT / BASE-USD, uppercase alphanumeric base 2..=10 — narrow
/// on purpose: equities can't collide with the dash form, and OCC
/// symbols have no dash.
pub fn is_crypto_pair(symbol: &str) -> bool {
    let Some((base, quote)) = symbol.split_once('-') else {
        return false;
    };
    matches!(quote, "USDT" | "USD")
        && (2..=10).contains(&base.len())
        && base.bytes().all(|b| b.is_ascii_uppercase() || b.is_ascii_digit())
}

/// Spot last with a 5s in-process cache — the paper engine's resting-
/// order ticker re-quotes every 5s per order, and hammering the venue
/// once per order per tick for the same instId is rude and slow.
pub async fn spot_quote_cached(symbol: &str) -> anyhow::Result<f64> {
    use std::time::{Duration, Instant};
    static CACHE: once_cell::sync::Lazy<tokio::sync::Mutex<std::collections::HashMap<String, (Instant, f64)>>> =
        once_cell::sync::Lazy::new(|| tokio::sync::Mutex::new(Default::default()));
    {
        let g = CACHE.lock().await;
        if let Some((t, p)) = g.get(symbol) {
            if t.elapsed() < Duration::from_secs(5) {
                return Ok(*p);
            }
        }
    }
    let u = format!("https://www.okx.com/api/v5/market/ticker?instId={symbol}");
    let p = okx_f64(&okx_json(&u).await?, &["data", "0", "last"])
        .filter(|p| *p > 0.0)
        .ok_or_else(|| anyhow::anyhow!("no spot quote for {symbol}"))?;
    CACHE.lock().await.insert(symbol.to_string(), (Instant::now(), p));
    Ok(p)
}

#[cfg(test)]
mod pair_tests {
    use super::*;

    #[test]
    fn crypto_pair_detection_is_narrow() {
        assert!(is_crypto_pair("BTC-USDT"));
        assert!(is_crypto_pair("ETH-USD"));
        assert!(is_crypto_pair("1INCH-USDT"));
        // Equities, OCC, perps, lowercase, weird quotes: all out.
        assert!(!is_crypto_pair("AAPL"));
        assert!(!is_crypto_pair("AAPL270115C00190000"));
        assert!(!is_crypto_pair("BTC-USDT-SWAP"));
        assert!(!is_crypto_pair("btc-usdt"));
        assert!(!is_crypto_pair("BTC-EUR"));
        assert!(!is_crypto_pair("B-USD"));
    }
}
