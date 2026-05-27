//! Sector heatmap — Finviz-style colored grid of S&P 500 names by sector.
//! Each tile = one symbol sized by market cap, colored by today's % change.
//!
//! Built-in universe is a curated subset (top ~150 by market cap across all
//! 11 GICS sectors). The user's watchlist symbols are merged in as a
//! "Watchlist" pseudo-sector so personal positions show up alongside the
//! benchmark grid.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct HeatTile {
    pub symbol: String,
    pub sector: &'static str,
    pub price: f64,
    pub change_pct: f64,
    pub market_cap: Option<f64>,   // rough weight; falls back to 1.0 if unknown
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
    ("Technology", &[
        "AAPL","MSFT","NVDA","GOOG","META","AVGO","ORCL","ADBE","CRM","CSCO",
        "AMD","INTC","QCOM","TXN","INTU","NOW","IBM","AMAT","MU","LRCX",
        "PANW","CRWD","SNPS","CDNS","KLAC","ADI","MRVL","FTNT","ANET","PLTR",
    ]),
    ("Communication Services", &[
        "META","GOOGL","NFLX","TMUS","CMCSA","DIS","VZ","T","CHTR","EA",
        "TTWO","WBD","ROKU","SPOT","SNAP","PINS",
    ]),
    ("Consumer Discretionary", &[
        "AMZN","TSLA","HD","MCD","NKE","SBUX","TJX","LOW","BKNG","CMG",
        "ABNB","MAR","ORLY","HLT","F","GM","RCL","DHI","LEN","ROST",
    ]),
    ("Consumer Staples", &[
        "WMT","COST","PG","KO","PEP","PM","MO","MDLZ","CL","KMB",
        "EL","TGT","STZ","KR","SYY","HSY","KDP","GIS",
    ]),
    ("Financials", &[
        "JPM","V","MA","BAC","WFC","GS","MS","C","AXP","SCHW",
        "BLK","BX","SPGI","PGR","MMC","COF","TFC","USB","CME","ICE",
        "PYPL","BRK-B",
    ]),
    ("Healthcare", &[
        "LLY","UNH","JNJ","ABBV","MRK","TMO","ABT","DHR","PFE","ISRG",
        "AMGN","MDT","BMY","SYK","ELV","CVS","GILD","REGN","VRTX","BSX",
        "ZTS","CI","HUM",
    ]),
    ("Industrials", &[
        "GE","RTX","CAT","UNP","HON","BA","UPS","DE","LMT","ADP",
        "ETN","NOC","WM","ITW","CSX","FDX","NSC","EMR","GD","MMM",
        "WDAY","PH",
    ]),
    ("Energy", &[
        "XOM","CVX","COP","EOG","OXY","SLB","MPC","PSX","VLO","WMB",
        "EPD","ET","KMI","PXD","DVN","HES","FANG","MRO",
    ]),
    ("Utilities", &[
        "NEE","SO","DUK","AEP","SRE","D","EXC","XEL","PCG","ED",
        "PEG","ETR","WEC","ES","EIX","FE","AWK","AEE",
    ]),
    ("Real Estate", &[
        "PLD","AMT","EQIX","CCI","PSA","WELL","O","SPG","DLR","CBRE",
        "AVB","EQR","VTR","BXP","ESS","ARE","UDR",
    ]),
    ("Materials", &[
        "LIN","SHW","APD","ECL","FCX","NEM","NUE","DOW","DD","CTVA",
        "VMC","MLM","PPG","STLD","BALL","CF","ALB",
    ]),
];

pub async fn build(pool: &PgPool, user_id: Uuid) -> anyhow::Result<HeatmapResponse> {
    use std::collections::HashSet;
    // Watchlist symbols get pinned to a "Watchlist" pseudo-sector so they
    // always render even if not in the curated universe.
    let mut watchlist: HashSet<String> = HashSet::new();
    if let Ok(lists) = crate::watchlists::list(pool, user_id).await {
        for w in lists {
            if let Ok(syms) = crate::watchlists::symbols(pool, w.id).await {
                for s in syms { watchlist.insert(s); }
            }
        }
    }

    let mut tiles = Vec::new();
    for (sector, syms) in UNIVERSE {
        for sym in *syms {
            if let Some(t) = tile_for(pool, sym, sector).await {
                tiles.push(t);
            }
        }
    }
    // Watchlist sector — only add symbols not already represented above.
    let already: HashSet<String> = tiles.iter().map(|t| t.symbol.clone()).collect();
    for sym in &watchlist {
        if already.contains(sym) { continue; }
        if let Some(t) = tile_for(pool, sym, "Watchlist").await {
            tiles.push(t);
        }
    }

    Ok(HeatmapResponse { tiles, generated_at: Utc::now() })
}

async fn tile_for(pool: &PgPool, sym: &str, sector: &'static str) -> Option<HeatTile> {
    let q = crate::market_data::quote(pool, sym).await.ok()?;
    Some(HeatTile {
        symbol: sym.into(),
        sector,
        price: q.price,
        change_pct: q.change_pct.unwrap_or(0.0),
        market_cap: None,   // optional; fundamentals fetch is slow — surface from cached snapshot later
        fetched_at: q.fetched_at,
    })
}
