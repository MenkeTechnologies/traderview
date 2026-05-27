//! Pre-market / overnight futures dashboard.
//!
//! Index futures (ES/NQ/YM/RTY), commodities (GC/SI/CL/NG), crypto (BTC/ETH/SOL),
//! FX/Dollar (DXY + 3 majors). Each card carries:
//!   - last price + % change vs prior close
//!   - 20-day ATR (in %)
//!   - **ATR-normalized magnitude** of the current move (how many ATRs is this?)
//!   - day high / low
//!
//! Plus today's high-importance economic events (from the static calendar).

use chrono::{Duration, NaiveDate, Utc};
use futures_util::future::join_all;
use serde::Serialize;
use sqlx::PgPool;
use traderview_core::BarInterval;

use crate::economy::{self, EconEvent, Importance};
use crate::market_data;

#[derive(Debug, Clone, Serialize)]
pub struct ContractRow {
    pub group: &'static str,
    pub symbol: String,
    pub label: &'static str,
    pub price: f64,
    pub prev_close: Option<f64>,
    pub change_pct: Option<f64>,
    pub atr_pct: Option<f64>,          // 20-day ATR as % of price
    pub atr_multiple: Option<f64>,     // |change_pct| / atr_pct
    pub day_high: Option<f64>,
    pub day_low: Option<f64>,
    pub market_state: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PremarketSnapshot {
    pub contracts: Vec<ContractRow>,
    pub today_events: Vec<EconEvent>,
    pub fetched_at: chrono::DateTime<Utc>,
}

const UNIVERSE: &[(&str, &str, &str)] = &[
    // (group, symbol, label)
    ("Index futures",  "ES=F",       "S&P 500 e-mini"),
    ("Index futures",  "NQ=F",       "Nasdaq 100 e-mini"),
    ("Index futures",  "YM=F",       "Dow e-mini"),
    ("Index futures",  "RTY=F",      "Russell 2000 e-mini"),
    ("Commodities",    "GC=F",       "Gold"),
    ("Commodities",    "SI=F",       "Silver"),
    ("Commodities",    "CL=F",       "Crude WTI"),
    ("Commodities",    "NG=F",       "Natural Gas"),
    ("Crypto",         "BTC-USD",    "Bitcoin"),
    ("Crypto",         "ETH-USD",    "Ethereum"),
    ("Crypto",         "SOL-USD",    "Solana"),
    ("FX",             "DX-Y.NYB",   "Dollar Index"),
    ("FX",             "EURUSD=X",   "EUR/USD"),
    ("FX",             "USDJPY=X",   "USD/JPY"),
    ("FX",             "GBPUSD=X",   "GBP/USD"),
];

pub async fn snapshot(pool: &PgPool) -> anyhow::Result<PremarketSnapshot> {
    // Fetch all 15 symbols concurrently. Serial fetches were observed
    // taking 150+ seconds in production (15 syms × ~10s timeout each on cold
    // Yahoo misses), which blocked the axum handler and starved every other
    // request waiting on the sqlx pool.
    let fetches = UNIVERSE.iter().map(|(group, symbol, label)| {
        let pool = pool.clone();
        async move {
            let q = market_data::quote(&pool, symbol).await.ok();
            let atr_pct = atr20_pct(&pool, symbol).await;
            let atr_multiple = match (q.as_ref().and_then(|q| q.change_pct), atr_pct) {
                (Some(c), Some(a)) if a > 0.0 => Some(c.abs() / a),
                _ => None,
            };
            match q {
                Some(q) => ContractRow {
                    group,
                    symbol: symbol.to_string(),
                    label,
                    price: q.price,
                    prev_close: q.prev_close,
                    change_pct: q.change_pct,
                    atr_pct,
                    atr_multiple,
                    day_high: q.day_high,
                    day_low: q.day_low,
                    market_state: q.market_state,
                },
                None => ContractRow {
                    group,
                    symbol: symbol.to_string(),
                    label,
                    price: 0.0,
                    prev_close: None,
                    change_pct: None,
                    atr_pct: None,
                    atr_multiple: None,
                    day_high: None,
                    day_low: None,
                    market_state: None,
                },
            }
        }
    });
    let contracts: Vec<ContractRow> = join_all(fetches).await;

    let today = Utc::now().date_naive();
    let today_events = economy::upcoming(1, Importance::High)
        .into_iter()
        .filter(|e| e.when_et.date() == today)
        .collect();

    Ok(PremarketSnapshot { contracts, today_events, fetched_at: Utc::now() })
}

/// 20-day ATR (true range) expressed as a percentage of the latest close.
async fn atr20_pct(pool: &PgPool, symbol: &str) -> Option<f64> {
    let to = Utc::now();
    let from = to - Duration::days(60);
    let bars = crate::prices::get_bars(pool, symbol, BarInterval::D1, from, to).await.ok()?;
    let n = bars.len();
    if n < 21 { return None; }
    let closes: Vec<f64> = bars.iter().map(|b| dec(b.close)).collect();
    let highs:  Vec<f64> = bars.iter().map(|b| dec(b.high)).collect();
    let lows:   Vec<f64> = bars.iter().map(|b| dec(b.low)).collect();
    let mut trs = Vec::with_capacity(20);
    for i in (n - 20)..n {
        let tr = (highs[i] - lows[i])
            .max((highs[i] - closes[i - 1]).abs())
            .max((lows[i]  - closes[i - 1]).abs());
        trs.push(tr);
    }
    let atr = trs.iter().sum::<f64>() / 20.0;
    let last = closes[n - 1];
    if last > 0.0 { Some(atr / last * 100.0) } else { None }
}

fn dec(d: rust_decimal::Decimal) -> f64 { d.to_string().parse().unwrap_or(0.0) }

#[allow(dead_code)]
fn _today() -> NaiveDate { Utc::now().date_naive() }
