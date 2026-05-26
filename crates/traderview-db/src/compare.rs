//! Side-by-side stock comparison.
//!
//! Pulls Yahoo `quoteSummary` modules + cached daily bars for 2–4 symbols and
//! returns one `CompareRow` per symbol with valuation, profitability, growth,
//! and balance-sheet metrics, plus multi-horizon returns and a 252-bar
//! normalized close series (rebased to 100 at the start of the window) for
//! the relative-strength overlay chart.

use chrono::{Duration, Utc};
use serde::Serialize;
use serde_json::Value;
use sqlx::PgPool;
use traderview_core::BarInterval;

use crate::market_data;

#[derive(Debug, Clone, Serialize)]
pub struct CompareRow {
    pub symbol: String,
    pub name: Option<String>,
    pub sector: Option<String>,
    pub industry: Option<String>,
    pub price: Option<f64>,
    pub market_cap: Option<f64>,
    pub enterprise_value: Option<f64>,

    pub trailing_pe: Option<f64>,
    pub forward_pe: Option<f64>,
    pub peg_ratio: Option<f64>,
    pub price_to_book: Option<f64>,
    pub price_to_sales: Option<f64>,
    pub ev_to_ebitda: Option<f64>,

    pub profit_margin: Option<f64>,
    pub operating_margin: Option<f64>,
    pub return_on_equity: Option<f64>,
    pub return_on_assets: Option<f64>,

    pub revenue_growth: Option<f64>,
    pub earnings_growth: Option<f64>,
    pub revenue_per_share: Option<f64>,
    pub total_cash_per_share: Option<f64>,

    pub debt_to_equity: Option<f64>,
    pub current_ratio: Option<f64>,
    pub quick_ratio: Option<f64>,
    pub free_cashflow: Option<f64>,

    pub dividend_yield: Option<f64>,
    pub payout_ratio: Option<f64>,
    pub beta: Option<f64>,

    pub fifty_two_week_high: Option<f64>,
    pub fifty_two_week_low: Option<f64>,
    pub fifty_day_avg: Option<f64>,
    pub two_hundred_day_avg: Option<f64>,

    pub return_1d: Option<f64>,
    pub return_1w: Option<f64>,
    pub return_1m: Option<f64>,
    pub return_3m: Option<f64>,
    pub return_6m: Option<f64>,
    pub return_1y: Option<f64>,

    pub normalized_closes: Vec<NormalizedPoint>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NormalizedPoint {
    pub day: String,           // YYYY-MM-DD
    pub value: f64,            // rebased to 100 at series start
}

#[derive(Debug, Clone, Serialize)]
pub struct CompareReport {
    pub rows: Vec<CompareRow>,
    pub fetched_at: chrono::DateTime<Utc>,
}

pub async fn compare(pool: &PgPool, symbols: &[String]) -> anyhow::Result<CompareReport> {
    // Sequential is fine — Yahoo rate-limits, and 2-4 symbols × ~500ms is acceptable.
    let mut rows = Vec::with_capacity(symbols.len());
    for s in symbols {
        if let Ok(row) = build_row(pool, s.clone()).await {
            rows.push(row);
        }
    }
    Ok(CompareReport { rows, fetched_at: Utc::now() })
}

async fn build_row(pool: &PgPool, sym: String) -> anyhow::Result<CompareRow> {
    let qs = market_data::fundamentals(&sym).await.unwrap_or(Value::Null);
    let quote = market_data::quote(pool, &sym).await.ok();
    let bars = {
        let to = Utc::now();
        let from = to - Duration::days(400);
        crate::prices::get_bars(pool, &sym, BarInterval::D1, from, to).await.unwrap_or_default()
    };

    let sd  = &qs["summaryDetail"];
    let fd  = &qs["financialData"];
    let ks  = &qs["defaultKeyStatistics"];
    let ap  = &qs["assetProfile"];
    let pr  = &qs["price"];

    let price = quote.as_ref().map(|q| q.price).or_else(|| f(&pr["regularMarketPrice"]));

    // Multi-horizon returns from cached bars.
    let closes: Vec<f64> = bars.iter().map(|b| dec(b.close)).collect();
    let return_at = |n: usize| -> Option<f64> {
        if closes.len() > n {
            let p = closes[closes.len() - 1 - n];
            let c = *closes.last().unwrap();
            if p > 0.0 { Some((c - p) / p * 100.0) } else { None }
        } else { None }
    };

    // 252-bar normalized series (rebased to 100 at the first bar in the window).
    let window: Vec<&_> = bars.iter().rev().take(252).collect();
    let mut window: Vec<_> = window.into_iter().rev().collect();
    let base = window.first().map(|b| dec(b.close)).filter(|x| *x > 0.0);
    let normalized_closes = match base {
        Some(b0) => window.drain(..).map(|b| NormalizedPoint {
            day: b.bar_time.format("%Y-%m-%d").to_string(),
            value: dec(b.close) / b0 * 100.0,
        }).collect(),
        None => Vec::new(),
    };

    Ok(CompareRow {
        symbol: sym.clone(),
        name:     s(&pr["longName"]).or_else(|| s(&pr["shortName"])),
        sector:   s(&ap["sector"]),
        industry: s(&ap["industry"]),
        price,
        market_cap:       f(&sd["marketCap"]).or_else(|| f(&pr["marketCap"])),
        enterprise_value: f(&ks["enterpriseValue"]),

        trailing_pe:      f(&sd["trailingPE"]).or_else(|| f(&ks["trailingPE"])),
        forward_pe:       f(&sd["forwardPE"]).or_else(|| f(&ks["forwardPE"])),
        peg_ratio:        f(&ks["pegRatio"]),
        price_to_book:    f(&ks["priceToBook"]),
        price_to_sales:   f(&sd["priceToSalesTrailing12Months"]),
        ev_to_ebitda:     f(&ks["enterpriseToEbitda"]),

        profit_margin:    f(&fd["profitMargins"]).or_else(|| f(&ks["profitMargins"])),
        operating_margin: f(&fd["operatingMargins"]),
        return_on_equity: f(&fd["returnOnEquity"]),
        return_on_assets: f(&fd["returnOnAssets"]),

        revenue_growth:   f(&fd["revenueGrowth"]),
        earnings_growth:  f(&fd["earningsGrowth"]),
        revenue_per_share: f(&fd["revenuePerShare"]),
        total_cash_per_share: f(&fd["totalCashPerShare"]),

        debt_to_equity:   f(&fd["debtToEquity"]),
        current_ratio:    f(&fd["currentRatio"]),
        quick_ratio:      f(&fd["quickRatio"]),
        free_cashflow:    f(&fd["freeCashflow"]),

        dividend_yield:   f(&sd["dividendYield"]),
        payout_ratio:     f(&sd["payoutRatio"]),
        beta:             f(&sd["beta"]).or_else(|| f(&ks["beta"])),

        fifty_two_week_high: f(&sd["fiftyTwoWeekHigh"]),
        fifty_two_week_low:  f(&sd["fiftyTwoWeekLow"]),
        fifty_day_avg:       f(&sd["fiftyDayAverage"]),
        two_hundred_day_avg: f(&sd["twoHundredDayAverage"]),

        return_1d: return_at(1),
        return_1w: return_at(5),
        return_1m: return_at(21),
        return_3m: return_at(63),
        return_6m: return_at(126),
        return_1y: return_at(252),

        normalized_closes,
    })
}

/// Extract `raw` field from a Yahoo `{raw, fmt, longFmt}` object, or treat
/// a bare number as the value. Returns None on miss.
fn f(v: &Value) -> Option<f64> {
    if let Some(n) = v.as_f64() { return Some(n); }
    if let Some(n) = v.as_i64() { return Some(n as f64); }
    if let Some(n) = v["raw"].as_f64() { return Some(n); }
    if let Some(n) = v["raw"].as_i64() { return Some(n as f64); }
    None
}

fn s(v: &Value) -> Option<String> { v.as_str().map(|x| x.to_string()) }

fn dec(d: rust_decimal::Decimal) -> f64 { d.to_string().parse().unwrap_or(0.0) }
