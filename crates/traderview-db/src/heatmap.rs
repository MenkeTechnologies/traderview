//! Sector heatmap — Finviz-style colored grid of S&P 500 names by sector.
//! Each tile = one symbol sized by market cap, colored by today's % change.
//!
//! Built-in universe is a curated subset (top ~150 by market cap across all
//! 11 GICS sectors). The user's watchlist symbols are merged in as a
//! "Watchlist" pseudo-sector so personal positions show up alongside the
//! benchmark grid.

use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use serde::Serialize;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct HeatTile {
    pub symbol: String,
    pub sector: &'static str,
    pub price: f64,
    pub change_pct: f64,
    pub market_cap: Option<f64>, // rough weight; falls back to 1.0 if unknown
    pub fetched_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HeatmapResponse {
    pub tiles: Vec<HeatTile>,
    pub generated_at: DateTime<Utc>,
}

/// Top names per sector. Hand-curated — covers ~80% of S&P market cap with
/// a manageable number of API calls.
const UNIVERSE: &[(&str, &[&str])] = &[
    (
        "Technology",
        &[
            "AAPL", "MSFT", "NVDA", "GOOG", "META", "AVGO", "ORCL", "ADBE", "CRM", "CSCO", "AMD",
            "INTC", "QCOM", "TXN", "INTU", "NOW", "IBM", "AMAT", "MU", "LRCX", "PANW", "CRWD",
            "SNPS", "CDNS", "KLAC", "ADI", "MRVL", "FTNT", "ANET", "PLTR",
        ],
    ),
    (
        "Communication Services",
        &[
            "META", "GOOGL", "NFLX", "TMUS", "CMCSA", "DIS", "VZ", "T", "CHTR", "EA", "TTWO",
            "WBD", "ROKU", "SPOT", "SNAP", "PINS",
        ],
    ),
    (
        "Consumer Discretionary",
        &[
            "AMZN", "TSLA", "HD", "MCD", "NKE", "SBUX", "TJX", "LOW", "BKNG", "CMG", "ABNB", "MAR",
            "ORLY", "HLT", "F", "GM", "RCL", "DHI", "LEN", "ROST",
        ],
    ),
    (
        "Consumer Staples",
        &[
            "WMT", "COST", "PG", "KO", "PEP", "PM", "MO", "MDLZ", "CL", "KMB", "EL", "TGT", "STZ",
            "KR", "SYY", "HSY", "KDP", "GIS",
        ],
    ),
    (
        "Financials",
        &[
            "JPM", "V", "MA", "BAC", "WFC", "GS", "MS", "C", "AXP", "SCHW", "BLK", "BX", "SPGI",
            "PGR", "MMC", "COF", "TFC", "USB", "CME", "ICE", "PYPL", "BRK-B",
        ],
    ),
    (
        "Healthcare",
        &[
            "LLY", "UNH", "JNJ", "ABBV", "MRK", "TMO", "ABT", "DHR", "PFE", "ISRG", "AMGN", "MDT",
            "BMY", "SYK", "ELV", "CVS", "GILD", "REGN", "VRTX", "BSX", "ZTS", "CI", "HUM",
        ],
    ),
    (
        "Industrials",
        &[
            "GE", "RTX", "CAT", "UNP", "HON", "BA", "UPS", "DE", "LMT", "ADP", "ETN", "NOC", "WM",
            "ITW", "CSX", "FDX", "NSC", "EMR", "GD", "MMM", "WDAY", "PH",
        ],
    ),
    (
        "Energy",
        &[
            "XOM", "CVX", "COP", "EOG", "OXY", "SLB", "MPC", "PSX", "VLO", "WMB", "EPD", "ET",
            "KMI", "PXD", "DVN", "HES", "FANG", "MRO",
        ],
    ),
    (
        "Utilities",
        &[
            "NEE", "SO", "DUK", "AEP", "SRE", "D", "EXC", "XEL", "PCG", "ED", "PEG", "ETR", "WEC",
            "ES", "EIX", "FE", "AWK", "AEE",
        ],
    ),
    (
        "Real Estate",
        &[
            "PLD", "AMT", "EQIX", "CCI", "PSA", "WELL", "O", "SPG", "DLR", "CBRE", "AVB", "EQR",
            "VTR", "BXP", "ESS", "ARE", "UDR",
        ],
    ),
    (
        "Materials",
        &[
            "LIN", "SHW", "APD", "ECL", "FCX", "NEM", "NUE", "DOW", "DD", "CTVA", "VMC", "MLM",
            "PPG", "STLD", "BALL", "CF", "ALB",
        ],
    ),
];

pub async fn build(pool: &PgPool, user_id: Uuid) -> anyhow::Result<HeatmapResponse> {
    // Per-user in-process cache. Building the grid fans out ~210 quote fetches;
    // even with the 60s on-disk quote cache that's noticeable work on every
    // visit. Cache the whole response per user for `CACHE_TTL` so repeat loads
    // (and the auto-refresh poll) return instantly.
    //
    // Single-flight: the per-user lock is held for the entire build, so N
    // concurrent requests for the same user trigger exactly one fan-out.
    let entry = {
        let mut map = CACHE.lock().await;
        map.entry(user_id)
            .or_insert_with(|| Arc::new(Mutex::new(None)))
            .clone()
    };
    let mut cache = entry.lock().await;
    if let Some((stored_at, resp)) = cache.as_ref() {
        if stored_at.elapsed() < CACHE_TTL {
            return Ok(resp.clone());
        }
    }

    let resp = build_uncached(pool, user_id).await?;
    *cache = Some((Instant::now(), resp.clone()));
    Ok(resp)
}

// In-process heatmap cache, keyed by user (watchlist differs per user). Each
// user gets their own inner lock so a slow build for one user doesn't block
// cache hits for another. 60s freshness matches the on-disk quote cache.
const CACHE_TTL: Duration = Duration::from_secs(60);
#[allow(clippy::type_complexity)]
static CACHE: Lazy<Mutex<HashMap<Uuid, Arc<Mutex<Option<(Instant, HeatmapResponse)>>>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

async fn build_uncached(pool: &PgPool, user_id: Uuid) -> anyhow::Result<HeatmapResponse> {
    use std::collections::HashSet;
    // Watchlist symbols get pinned to a "Watchlist" pseudo-sector so they
    // always render even if not in the curated universe.
    let mut watchlist: HashSet<String> = HashSet::new();
    if let Ok(lists) = crate::watchlists::list(pool, user_id).await {
        for w in lists {
            if let Ok(syms) = crate::watchlists::symbols(pool, w.id).await {
                for s in syms {
                    watchlist.insert(s);
                }
            }
        }
    }

    // Flatten the universe (and de-duped watchlist) into one job list so the
    // ~210 quote fetches run concurrently. A serial loop here blocks the
    // request for a minute+ on a cold cache — see `market_data::quotes`.
    let universe_syms: HashSet<&str> = UNIVERSE
        .iter()
        .flat_map(|(_, syms)| syms.iter().copied())
        .collect();
    let mut jobs: Vec<(&'static str, String)> = Vec::new();
    for (sector, syms) in UNIVERSE {
        for sym in *syms {
            jobs.push((sector, (*sym).to_string()));
        }
    }
    for sym in &watchlist {
        if !universe_syms.contains(sym.as_str()) {
            jobs.push(("Watchlist", sym.clone()));
        }
    }

    // Bounded concurrency: fetch in chunks of 16 (Yahoo tolerates ~16 parallel
    // chart requests; unbounded fan-out risks rate limiting).
    let mut tiles: Vec<HeatTile> = Vec::new();
    for chunk in jobs.chunks(16) {
        let futs = chunk.iter().map(|(sector, sym)| {
            let pool = pool.clone();
            let sym = sym.clone();
            let sector: &'static str = sector;
            async move { tile_for(&pool, &sym, sector).await }
        });
        tiles.extend(
            futures_util::future::join_all(futs)
                .await
                .into_iter()
                .flatten(),
        );
    }

    Ok(HeatmapResponse {
        tiles,
        generated_at: Utc::now(),
    })
}

async fn tile_for(pool: &PgPool, sym: &str, sector: &'static str) -> Option<HeatTile> {
    let q = crate::market_data::quote(pool, sym).await.ok()?;
    Some(HeatTile {
        symbol: sym.into(),
        sector,
        price: q.price,
        change_pct: q.change_pct.unwrap_or(0.0),
        market_cap: None, // optional; fundamentals fetch is slow — surface from cached snapshot later
        fetched_at: q.fetched_at,
    })
}
