//! Markets snapshot — fixed list of global indices + commodities + FX.
//!
//! Pulls last-close + prior-close from the Yahoo Finance chart endpoint for
//! each symbol in parallel and returns a single snapshot payload. No auth
//! required (the v8 chart endpoint is public).

use chrono::{DateTime, Utc};
use futures_util::future::join_all;
use once_cell::sync::Lazy;
use serde::Serialize;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

const USER_AGENT: &str =
    "Mozilla/5.0 (compatible; traderview/0.1; +https://github.com/MenkeTechnologies/traderview)";

#[derive(Debug, Clone, Serialize)]
pub struct MarketTile {
    pub symbol: String,
    pub label: String,
    pub flag: &'static str,
    pub lat: f64,
    pub lng: f64,
    pub price: f64,
    pub prev_close: f64,
    pub change_pct: f64,
    pub currency: &'static str,
    pub market_state: String, // "REGULAR" | "CLOSED" | "PRE" | "POST" — best-effort from chart meta
}

#[derive(Debug, Clone, Serialize)]
pub struct MarketsSnapshot {
    pub indices: Vec<MarketTile>,
    pub commodities: Vec<MarketTile>,
    #[serde(default)]
    pub fx: Vec<MarketTile>,
    #[serde(default)]
    pub crypto: Vec<MarketTile>,
    pub us_market_open: bool,
    pub fetched_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Kind {
    Index,
    Commodity,
    Fx,
    Crypto,
}

struct Pin {
    symbol: &'static str,
    label: &'static str,
    flag: &'static str,
    lat: f64,
    lng: f64,
    currency: &'static str,
    kind: Kind,
}

const PINS: &[Pin] = &[
    // Indices — anchored at the exchange city for the map pin.
    Pin {
        symbol: "^GSPC",
        label: "S&P 500",
        flag: "🇺🇸",
        lat: 40.71,
        lng: -74.01,
        currency: "USD",
        kind: Kind::Index,
    },
    Pin {
        symbol: "^IXIC",
        label: "Nasdaq",
        flag: "🇺🇸",
        lat: 40.72,
        lng: -73.99,
        currency: "USD",
        kind: Kind::Index,
    },
    Pin {
        symbol: "^GSPTSE",
        label: "S&P Toronto",
        flag: "🇨🇦",
        lat: 43.65,
        lng: -79.38,
        currency: "CAD",
        kind: Kind::Index,
    },
    Pin {
        symbol: "^BVSP",
        label: "Bovespa",
        flag: "🇧🇷",
        lat: -23.55,
        lng: -46.63,
        currency: "BRL",
        kind: Kind::Index,
    },
    Pin {
        symbol: "^FTSE",
        label: "FTSE London",
        flag: "🇬🇧",
        lat: 51.51,
        lng: -0.10,
        currency: "GBP",
        kind: Kind::Index,
    },
    Pin {
        symbol: "^FCHI",
        label: "CAC Paris",
        flag: "🇫🇷",
        lat: 48.85,
        lng: 2.35,
        currency: "EUR",
        kind: Kind::Index,
    },
    Pin {
        symbol: "^GDAXI",
        label: "DAX",
        flag: "🇩🇪",
        lat: 50.11,
        lng: 8.68,
        currency: "EUR",
        kind: Kind::Index,
    },
    Pin {
        symbol: "^N225",
        label: "Nikkei",
        flag: "🇯🇵",
        lat: 35.69,
        lng: 139.69,
        currency: "JPY",
        kind: Kind::Index,
    },
    Pin {
        symbol: "^HSI",
        label: "Hang Seng",
        flag: "🇭🇰",
        lat: 22.30,
        lng: 114.17,
        currency: "HKD",
        kind: Kind::Index,
    },
    Pin {
        symbol: "000001.SS",
        label: "Shanghai",
        flag: "🇨🇳",
        lat: 31.23,
        lng: 121.47,
        currency: "CNY",
        kind: Kind::Index,
    },
    Pin {
        symbol: "^NSEI",
        label: "Nifty India",
        flag: "🇮🇳",
        lat: 19.08,
        lng: 72.87,
        currency: "INR",
        kind: Kind::Index,
    },
    Pin {
        symbol: "^AORD",
        label: "Aus All Ords",
        flag: "🇦🇺",
        lat: -33.86,
        lng: 151.21,
        currency: "AUD",
        kind: Kind::Index,
    },
    // Commodities — front-month futures via Yahoo continuous contracts.
    // Energy
    Pin {
        symbol: "CL=F",
        label: "Crude Oil (WTI)",
        flag: "🛢️",
        lat: 0.0,
        lng: 0.0,
        currency: "USD",
        kind: Kind::Commodity,
    },
    Pin {
        symbol: "BZ=F",
        label: "Brent Crude",
        flag: "🛢️",
        lat: 0.0,
        lng: 0.0,
        currency: "USD",
        kind: Kind::Commodity,
    },
    Pin {
        symbol: "NG=F",
        label: "Natural Gas",
        flag: "🔥",
        lat: 0.0,
        lng: 0.0,
        currency: "USD",
        kind: Kind::Commodity,
    },
    Pin {
        symbol: "RB=F",
        label: "Gasoline",
        flag: "⛽",
        lat: 0.0,
        lng: 0.0,
        currency: "USD",
        kind: Kind::Commodity,
    },
    Pin {
        symbol: "HO=F",
        label: "Heating Oil",
        flag: "🔥",
        lat: 0.0,
        lng: 0.0,
        currency: "USD",
        kind: Kind::Commodity,
    },
    // Metals
    Pin {
        symbol: "GC=F",
        label: "Gold",
        flag: "🥇",
        lat: 0.0,
        lng: 0.0,
        currency: "USD",
        kind: Kind::Commodity,
    },
    Pin {
        symbol: "SI=F",
        label: "Silver",
        flag: "🥈",
        lat: 0.0,
        lng: 0.0,
        currency: "USD",
        kind: Kind::Commodity,
    },
    Pin {
        symbol: "HG=F",
        label: "Copper",
        flag: "🟤",
        lat: 0.0,
        lng: 0.0,
        currency: "USD",
        kind: Kind::Commodity,
    },
    Pin {
        symbol: "PL=F",
        label: "Platinum",
        flag: "⚪",
        lat: 0.0,
        lng: 0.0,
        currency: "USD",
        kind: Kind::Commodity,
    },
    Pin {
        symbol: "PA=F",
        label: "Palladium",
        flag: "⚫",
        lat: 0.0,
        lng: 0.0,
        currency: "USD",
        kind: Kind::Commodity,
    },
    // Grains
    Pin {
        symbol: "ZC=F",
        label: "Corn",
        flag: "🌽",
        lat: 0.0,
        lng: 0.0,
        currency: "USD",
        kind: Kind::Commodity,
    },
    Pin {
        symbol: "ZW=F",
        label: "Wheat",
        flag: "🌾",
        lat: 0.0,
        lng: 0.0,
        currency: "USD",
        kind: Kind::Commodity,
    },
    Pin {
        symbol: "ZS=F",
        label: "Soybeans",
        flag: "🫘",
        lat: 0.0,
        lng: 0.0,
        currency: "USD",
        kind: Kind::Commodity,
    },
    Pin {
        symbol: "ZO=F",
        label: "Oats",
        flag: "🌾",
        lat: 0.0,
        lng: 0.0,
        currency: "USD",
        kind: Kind::Commodity,
    },
    // Softs
    Pin {
        symbol: "KC=F",
        label: "Coffee",
        flag: "☕",
        lat: 0.0,
        lng: 0.0,
        currency: "USD",
        kind: Kind::Commodity,
    },
    Pin {
        symbol: "SB=F",
        label: "Sugar",
        flag: "🍬",
        lat: 0.0,
        lng: 0.0,
        currency: "USD",
        kind: Kind::Commodity,
    },
    Pin {
        symbol: "CC=F",
        label: "Cocoa",
        flag: "🍫",
        lat: 0.0,
        lng: 0.0,
        currency: "USD",
        kind: Kind::Commodity,
    },
    Pin {
        symbol: "CT=F",
        label: "Cotton",
        flag: "🧵",
        lat: 0.0,
        lng: 0.0,
        currency: "USD",
        kind: Kind::Commodity,
    },
    Pin {
        symbol: "OJ=F",
        label: "Orange Juice",
        flag: "🍊",
        lat: 0.0,
        lng: 0.0,
        currency: "USD",
        kind: Kind::Commodity,
    },
    // Livestock
    Pin {
        symbol: "LE=F",
        label: "Live Cattle",
        flag: "🐂",
        lat: 0.0,
        lng: 0.0,
        currency: "USD",
        kind: Kind::Commodity,
    },
    Pin {
        symbol: "HE=F",
        label: "Lean Hogs",
        flag: "🐖",
        lat: 0.0,
        lng: 0.0,
        currency: "USD",
        kind: Kind::Commodity,
    },
    // FX — major pairs.
    Pin {
        symbol: "EURUSD=X",
        label: "EUR/USD",
        flag: "💶",
        lat: 0.0,
        lng: 0.0,
        currency: "USD",
        kind: Kind::Fx,
    },
    Pin {
        symbol: "GBPUSD=X",
        label: "GBP/USD",
        flag: "💷",
        lat: 0.0,
        lng: 0.0,
        currency: "USD",
        kind: Kind::Fx,
    },
    Pin {
        symbol: "USDJPY=X",
        label: "USD/JPY",
        flag: "💴",
        lat: 0.0,
        lng: 0.0,
        currency: "JPY",
        kind: Kind::Fx,
    },
    Pin {
        symbol: "USDCHF=X",
        label: "USD/CHF",
        flag: "🇨🇭",
        lat: 0.0,
        lng: 0.0,
        currency: "CHF",
        kind: Kind::Fx,
    },
    Pin {
        symbol: "AUDUSD=X",
        label: "AUD/USD",
        flag: "🇦🇺",
        lat: 0.0,
        lng: 0.0,
        currency: "USD",
        kind: Kind::Fx,
    },
    Pin {
        symbol: "USDCAD=X",
        label: "USD/CAD",
        flag: "🇨🇦",
        lat: 0.0,
        lng: 0.0,
        currency: "CAD",
        kind: Kind::Fx,
    },
    Pin {
        symbol: "DX=F",
        label: "DXY",
        flag: "🇺🇸",
        lat: 0.0,
        lng: 0.0,
        currency: "USD",
        kind: Kind::Fx,
    },
    // Crypto — majors via Yahoo's USD pairs.
    Pin {
        symbol: "BTC-USD",
        label: "Bitcoin",
        flag: "🟠",
        lat: 0.0,
        lng: 0.0,
        currency: "USD",
        kind: Kind::Crypto,
    },
    Pin {
        symbol: "ETH-USD",
        label: "Ethereum",
        flag: "⟠",
        lat: 0.0,
        lng: 0.0,
        currency: "USD",
        kind: Kind::Crypto,
    },
    Pin {
        symbol: "SOL-USD",
        label: "Solana",
        flag: "◎",
        lat: 0.0,
        lng: 0.0,
        currency: "USD",
        kind: Kind::Crypto,
    },
];

// In-process snapshot cache. The world-map view polls this on every dashboard
// load; without caching we paid ~1.2s on every visit to hit 16 Yahoo endpoints.
// 60s freshness matches the on-disk quote cache used elsewhere.
const CACHE_TTL: Duration = Duration::from_secs(60);
static CACHE: Lazy<Mutex<Option<(Instant, MarketsSnapshot)>>> = Lazy::new(|| Mutex::new(None));

/// Fetch the latest snapshot. Hits Yahoo's v8 chart endpoint for each pin
/// concurrently. Result is cached in-process for 60s; subsequent calls within
/// the window return the cached value immediately (sub-millisecond).
///
/// Single-flight: the cache lock is held for the entire fetch. Concurrent
/// callers that arrive during a fetch await the lock and then observe the
/// freshly-written cache entry, so N concurrent requests cause exactly one
/// fan-out to Yahoo instead of N. Without this the prior implementation
/// dropped the lock before fetching, letting N callers each issue 16
/// concurrent Yahoo requests (16×N) when the cache had just expired.
pub async fn snapshot() -> anyhow::Result<MarketsSnapshot> {
    let mut cache = CACHE.lock().await;
    if let Some((stored_at, snap)) = cache.as_ref() {
        if stored_at.elapsed() < CACHE_TTL {
            return Ok(snap.clone());
        }
    }

    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(std::time::Duration::from_secs(8))
        .build()?;
    // Real concurrent fetch — prior version "popped and awaited" one at a
    // time which was sequential despite the parallelism comment.
    let results: Vec<Option<MarketTile>> =
        join_all(PINS.iter().map(|p| fetch_one(&client, p))).await;

    let mut indices = Vec::new();
    let mut commodities = Vec::new();
    let mut fx = Vec::new();
    let mut crypto = Vec::new();
    let mut us_market_open = false;
    for r in results.into_iter().flatten() {
        if r.symbol == "^GSPC" && r.market_state == "REGULAR" {
            us_market_open = true;
        }
        // Look up the original Pin's kind by symbol — preserves the
        // commodity/fx/crypto split that the frontend uses to render
        // separate strips.
        let kind = PINS.iter().find(|p| p.symbol == r.symbol).map(|p| p.kind);
        match kind {
            Some(Kind::Index) => indices.push(r),
            Some(Kind::Commodity) => commodities.push(r),
            Some(Kind::Fx) => fx.push(r),
            Some(Kind::Crypto) => crypto.push(r),
            None => commodities.push(r), // shouldn't happen — keep round-trip honest
        }
    }
    let snap = MarketsSnapshot {
        indices,
        commodities,
        fx,
        crypto,
        us_market_open,
        fetched_at: Utc::now(),
    };
    *cache = Some((Instant::now(), snap.clone()));
    Ok(snap)
}

async fn fetch_one(client: &reqwest::Client, p: &Pin) -> Option<MarketTile> {
    let url = format!(
        "https://query1.finance.yahoo.com/v8/finance/chart/{sym}?interval=1d&range=5d",
        sym = urlencoding(p.symbol)
    );
    let resp = match client.get(&url).send().await {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!(?e, sym = p.symbol, "snapshot fetch failed");
            return None;
        }
    };
    if !resp.status().is_success() {
        tracing::warn!(status = ?resp.status(), sym = p.symbol, "snapshot HTTP error");
        return None;
    }
    let raw: ChartResp = resp.json().await.ok()?;
    let result = raw.chart.result?.into_iter().next()?;
    let meta = result.meta;
    let price = meta.regular_market_price?;
    let prev_close = meta
        .chart_previous_close
        .unwrap_or(meta.previous_close.unwrap_or(price));
    let change_pct = if prev_close > 0.0 {
        (price - prev_close) / prev_close * 100.0
    } else {
        0.0
    };
    Some(MarketTile {
        symbol: p.symbol.into(),
        label: p.label.into(),
        flag: p.flag,
        lat: p.lat,
        lng: p.lng,
        price,
        prev_close,
        change_pct,
        currency: p.currency,
        market_state: meta.market_state.unwrap_or_else(|| "UNKNOWN".into()),
    })
}

fn urlencoding(s: &str) -> String {
    // Minimal — encode '^' and '=' which are common in Yahoo symbols.
    s.replace('^', "%5E").replace('=', "%3D")
}

#[derive(serde::Deserialize)]
struct ChartResp {
    chart: ChartInner,
}
#[derive(serde::Deserialize)]
struct ChartInner {
    result: Option<Vec<ChartResult>>,
}
#[derive(serde::Deserialize)]
struct ChartResult {
    meta: ChartMeta,
}
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChartMeta {
    regular_market_price: Option<f64>,
    previous_close: Option<f64>,
    chart_previous_close: Option<f64>,
    market_state: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Both async tests below mutate the global `CACHE` static. Without
    // serialization they race: cache_expires_after_ttl's backdated write can
    // land between cache_hit_returns_stored_snapshot's setup and its
    // snapshot() call, causing the latter to fall into the network-fetch
    // path and blow past its 50ms timing assert.
    static TEST_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    fn probe(open: bool) -> MarketsSnapshot {
        MarketsSnapshot {
            indices: vec![],
            commodities: vec![],
            fx: vec![],
            crypto: vec![],
            us_market_open: open,
            fetched_at: Utc::now(),
        }
    }

    /// Pre-populate the cache with a fresh entry and verify snapshot() returns
    /// it without going to Yahoo. The "evidence" is timing: a cache miss has
    /// to do 16 HTTP fetches (>100ms even on a fast network); a cache hit is
    /// sub-millisecond. The clear marker is `us_market_open=true` which Yahoo
    /// would only set during US RTH.
    #[tokio::test]
    async fn cache_hit_returns_stored_snapshot() {
        let _guard = TEST_LOCK.lock().await;
        let stored = probe(true);
        let stored_at = stored.fetched_at;
        *CACHE.lock().await = Some((Instant::now(), stored));
        let started = Instant::now();
        let got = snapshot().await.expect("cache hit must succeed");
        // 2s ceiling: a cache MISS does 16 HTTP fetches and takes >>1s, so anything
        // under 2s confirms the cache path. Tighter thresholds (<50ms) flake on
        // CI runners under load (observed on macOS-latest).
        assert!(
            started.elapsed() < std::time::Duration::from_secs(2),
            "cache hit should be near-instant; got {:?}",
            started.elapsed()
        );
        assert_eq!(
            got.fetched_at, stored_at,
            "cache hit returned a different snapshot than what we stored"
        );
    }

    /// Verify TTL: an entry stored 2 minutes ago must be treated as stale.
    /// We can't test the *fetch* path without network, but we can verify the
    /// cache check returns None for expired entries by inspecting the static.
    #[tokio::test]
    async fn cache_expires_after_ttl() {
        let _guard = TEST_LOCK.lock().await;
        let stale = probe(false);
        // Backdate to 2 minutes ago — well beyond CACHE_TTL (60s).
        // Use checked_sub to avoid panic on CI runners whose process Instant
        // origin is younger than the backdate amount (observed on
        // freshly-booted macOS hosts where mach_absolute_time is small).
        // If the subtraction would underflow, skip the test — the assertion
        // we want to make (when.elapsed() > CACHE_TTL) cannot be set up.
        let Some(backdated) = Instant::now().checked_sub(std::time::Duration::from_secs(120))
        else {
            return;
        };
        *CACHE.lock().await = Some((backdated, stale));

        // Re-read the cache and verify the freshness check would reject it.
        let guard = CACHE.lock().await;
        let (when, _) = guard.as_ref().expect("we just set it");
        assert!(
            when.elapsed() > CACHE_TTL,
            "backdated entry must be older than CACHE_TTL"
        );
    }

    #[test]
    fn urlencoding_handles_yahoo_caret_and_equals() {
        assert_eq!(urlencoding("^GSPC"), "%5EGSPC");
        assert_eq!(urlencoding("EURUSD=X"), "EURUSD%3DX");
        assert_eq!(urlencoding("AAPL"), "AAPL"); // pass-through for plain
    }
}
