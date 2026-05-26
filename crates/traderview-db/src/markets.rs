//! Markets snapshot — fixed list of global indices + commodities + FX.
//!
//! Pulls last-close + prior-close from the Yahoo Finance chart endpoint for
//! each symbol in parallel and returns a single snapshot payload. No auth
//! required (the v8 chart endpoint is public).

use chrono::{DateTime, Utc};
use serde::Serialize;

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
    pub market_state: String,   // "REGULAR" | "CLOSED" | "PRE" | "POST" — best-effort from chart meta
}

#[derive(Debug, Clone, Serialize)]
pub struct MarketsSnapshot {
    pub indices: Vec<MarketTile>,
    pub commodities: Vec<MarketTile>,
    pub us_market_open: bool,
    pub fetched_at: DateTime<Utc>,
}

struct Pin {
    symbol: &'static str,
    label: &'static str,
    flag: &'static str,
    lat: f64,
    lng: f64,
    currency: &'static str,
    is_index: bool,
}

const PINS: &[Pin] = &[
    // Indices — anchored at the exchange city.
    Pin { symbol: "^GSPC",   label: "S&P 500",      flag: "🇺🇸", lat: 40.71,  lng: -74.01, currency: "USD", is_index: true },
    Pin { symbol: "^IXIC",   label: "Nasdaq",       flag: "🇺🇸", lat: 40.72,  lng: -73.99, currency: "USD", is_index: true },
    Pin { symbol: "^GSPTSE", label: "S&P Toronto",  flag: "🇨🇦", lat: 43.65,  lng: -79.38, currency: "CAD", is_index: true },
    Pin { symbol: "^BVSP",   label: "Bovespa",      flag: "🇧🇷", lat: -23.55, lng: -46.63, currency: "BRL", is_index: true },
    Pin { symbol: "^FTSE",   label: "FTSE London",  flag: "🇬🇧", lat: 51.51,  lng:  -0.10, currency: "GBP", is_index: true },
    Pin { symbol: "^FCHI",   label: "CAC Paris",    flag: "🇫🇷", lat: 48.85,  lng:   2.35, currency: "EUR", is_index: true },
    Pin { symbol: "^GDAXI",  label: "DAX",          flag: "🇩🇪", lat: 50.11,  lng:   8.68, currency: "EUR", is_index: true },
    Pin { symbol: "^N225",   label: "Nikkei",       flag: "🇯🇵", lat: 35.69,  lng: 139.69, currency: "JPY", is_index: true },
    Pin { symbol: "^HSI",    label: "Hang Seng",    flag: "🇭🇰", lat: 22.30,  lng: 114.17, currency: "HKD", is_index: true },
    Pin { symbol: "000001.SS", label: "Shanghai",   flag: "🇨🇳", lat: 31.23,  lng: 121.47, currency: "CNY", is_index: true },
    Pin { symbol: "^NSEI",   label: "Nifty India",  flag: "🇮🇳", lat: 19.08,  lng:  72.87, currency: "INR", is_index: true },
    Pin { symbol: "^AORD",   label: "Aus All Ords", flag: "🇦🇺", lat: -33.86, lng: 151.21, currency: "AUD", is_index: true },
    // Commodities + FX — no map pin, shown in the strip.
    Pin { symbol: "EURUSD=X", label: "EUR/USD",   flag: "💶", lat: 0.0, lng: 0.0, currency: "USD", is_index: false },
    Pin { symbol: "BTC-USD",  label: "Bitcoin",   flag: "🟠", lat: 0.0, lng: 0.0, currency: "USD", is_index: false },
    Pin { symbol: "CL=F",     label: "Crude Oil", flag: "🛢️", lat: 0.0, lng: 0.0, currency: "USD", is_index: false },
    Pin { symbol: "GC=F",     label: "Gold",      flag: "🥇", lat: 0.0, lng: 0.0, currency: "USD", is_index: false },
];

/// Fetch the latest snapshot. Hits Yahoo's v8 chart endpoint for each pin in
/// parallel — typical end-to-end latency ~1 second for ~16 symbols.
pub async fn snapshot() -> anyhow::Result<MarketsSnapshot> {
    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(std::time::Duration::from_secs(8))
        .build()?;
    let futs = PINS.iter().map(|p| fetch_one(&client, p));
    let results = futures_collect(futs).await;

    let mut indices = Vec::new();
    let mut commodities = Vec::new();
    let mut us_market_open = false;
    for r in results.into_iter().flatten() {
        if r.symbol == "^GSPC" && r.market_state == "REGULAR" {
            us_market_open = true;
        }
        if PINS.iter().find(|p| p.symbol == r.symbol).map(|p| p.is_index).unwrap_or(false) {
            indices.push(r);
        } else {
            commodities.push(r);
        }
    }
    Ok(MarketsSnapshot {
        indices,
        commodities,
        us_market_open,
        fetched_at: Utc::now(),
    })
}

// Tiny vendored helper so we don't add the `futures` crate just for join_all.
async fn futures_collect<I, F, T>(iter: I) -> Vec<T>
where
    I: IntoIterator<Item = F>,
    F: std::future::Future<Output = T>,
{
    let mut tasks: Vec<F> = iter.into_iter().collect();
    let mut out: Vec<T> = Vec::with_capacity(tasks.len());
    while let Some(t) = tasks.pop() {
        out.push(t.await);
    }
    // We pushed in reverse — re-reverse to preserve input order.
    out.reverse();
    out
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
    let prev_close = meta.chart_previous_close.unwrap_or(meta.previous_close.unwrap_or(price));
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
struct ChartResp { chart: ChartInner }
#[derive(serde::Deserialize)]
struct ChartInner { result: Option<Vec<ChartResult>> }
#[derive(serde::Deserialize)]
struct ChartResult { meta: ChartMeta }
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChartMeta {
    regular_market_price: Option<f64>,
    previous_close: Option<f64>,
    chart_previous_close: Option<f64>,
    market_state: Option<String>,
}
