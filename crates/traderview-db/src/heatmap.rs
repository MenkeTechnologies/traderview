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

/// Public re-export so other modules (magic_formula, sector_rotation,
/// etc.) can use the same hand-curated S&P universe without each maintaining
/// their own copy.
pub const UNIVERSE_EXPORT: &[(&str, &[&str])] = UNIVERSE;

/// Returns the GICS-ish sector name for `symbol` based on the hand-curated
/// UNIVERSE above, or `None` when the symbol isn't in the top-name list.
/// Used by the portfolio exposure dashboard to bucket positions by sector
/// without needing a paid GICS feed.
pub fn sector_for(symbol: &str) -> Option<&'static str> {
    let upper = symbol.to_ascii_uppercase();
    for (sector, tickers) in UNIVERSE {
        for t in *tickers {
            if t.eq_ignore_ascii_case(&upper) {
                return Some(sector);
            }
        }
    }
    None
}

// Precomputed universe grid. The ~210 universe quote fetches run ONLY
// in the background refresher (background::spawn_refreshers calls
// refresh_universe on interval) — a request never triggers the fan-out.
// The per-user watchlist overlay is small and fetched per request
// against the 60s on-disk quote cache.
static UNIVERSE_CACHE: Lazy<Mutex<Option<Vec<HeatTile>>>> = Lazy::new(|| Mutex::new(None));

/// Fan out the curated universe and park the tiles in the in-process
/// cache. Called by the background refresher; also used inline once on
/// the cold-boot race before the first refresh lands.
pub async fn refresh_universe(pool: &PgPool) -> anyhow::Result<usize> {
    let jobs: Vec<(&'static str, String)> = UNIVERSE
        .iter()
        .flat_map(|(sector, syms)| syms.iter().map(move |s| (*sector, (*s).to_string())))
        .collect();
    let tiles = fetch_tiles(pool, jobs).await;
    let n = tiles.len();
    *UNIVERSE_CACHE.lock().await = Some(tiles);
    Ok(n)
}

pub async fn build(pool: &PgPool, user_id: Uuid) -> anyhow::Result<HeatmapResponse> {
    use std::collections::HashSet;
    // Universe grid from the precomputed cache; compute inline only on
    // the cold-boot race before the first background refresh lands.
    let universe_tiles = {
        let cached = UNIVERSE_CACHE.lock().await.clone();
        match cached {
            Some(t) => t,
            None => {
                refresh_universe(pool).await?;
                UNIVERSE_CACHE.lock().await.clone().unwrap_or_default()
            }
        }
    };

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
    let universe_syms: HashSet<&str> = UNIVERSE
        .iter()
        .flat_map(|(_, syms)| syms.iter().copied())
        .collect();
    let watch_jobs: Vec<(&'static str, String)> = watchlist
        .iter()
        .filter(|s| !universe_syms.contains(s.as_str()))
        .map(|s| ("Watchlist", s.clone()))
        .collect();
    let mut tiles = universe_tiles;
    tiles.extend(fetch_tiles(pool, watch_jobs).await);

    Ok(HeatmapResponse {
        tiles,
        generated_at: Utc::now(),
    })
}

/// Bounded-concurrency quote fan-out: chunks of 16 (Yahoo tolerates
/// ~16 parallel chart requests; unbounded fan-out risks rate limiting).
async fn fetch_tiles(pool: &PgPool, jobs: Vec<(&'static str, String)>) -> Vec<HeatTile> {
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
    tiles
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
