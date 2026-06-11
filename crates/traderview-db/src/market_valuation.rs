//! Market-level valuation gauges + the Chowder dividend screen.
//!
//! * Fed model — S&P 500 forward earnings yield (1/PE via SPY) vs the
//!   10-year Treasury (^TNX). Spread > 0 = equities cheap vs bonds by
//!   this (much-criticized but ubiquitously watched) yardstick.
//! * NH-NL — how many names in the default universe sit within 2% of
//!   their 52-week high vs low; classic breadth confirmation.
//! * Chowder rule — dividend yield + 5-year dividend CAGR ≥ 12
//!   (≥ 8 for utilities/telecom-style slow growers).

use chrono::{Datelike, Duration, Utc};
use serde::Serialize;
use sqlx::PgPool;
use traderview_core::BarInterval;

// ===========================================================================
// Fed model
// ===========================================================================

#[derive(Debug, Clone, Serialize)]
pub struct FedModelReport {
    pub sp500_pe: f64,
    pub earnings_yield_pct: f64,
    pub treasury_10y_pct: f64,
    /// earnings_yield − 10y. Positive = equities cheap on the Fed model.
    pub spread_pct: f64,
    pub verdict: &'static str, // "equities_cheap" | "neutral" | "bonds_cheap"
}

pub async fn fed_model(pool: &PgPool) -> anyhow::Result<FedModelReport> {
    let m = crate::finnhub_rest::metric_all("SPY").await?;
    let pe = m
        .get("metric")
        .and_then(|x| x.get("peTTM"))
        .and_then(|v| v.as_f64())
        .ok_or_else(|| anyhow::anyhow!("no SPY peTTM from finnhub"))?;
    if pe <= 0.0 {
        anyhow::bail!("non-positive SPY PE: {pe}");
    }
    let earnings_yield = 100.0 / pe;
    let yc = crate::vol::yield_curve(pool).await?;
    let ten_y = yc
        .points
        .iter()
        .find(|p| p.symbol == "^TNX")
        .map(|p| p.yield_pct)
        .ok_or_else(|| anyhow::anyhow!("no 10Y yield available"))?;
    let spread = earnings_yield - ten_y;
    let verdict = if spread > 1.0 {
        "equities_cheap"
    } else if spread < -1.0 {
        "bonds_cheap"
    } else {
        "neutral"
    };
    Ok(FedModelReport {
        sp500_pe: pe,
        earnings_yield_pct: earnings_yield,
        treasury_10y_pct: ten_y,
        spread_pct: spread,
        verdict,
    })
}

// ===========================================================================
// New highs / new lows (52-week, default universe)
// ===========================================================================

#[derive(Debug, Clone, Serialize)]
pub struct NhNlReport {
    pub universe_size: usize,
    pub evaluated: usize,
    pub new_highs: Vec<String>,
    pub new_lows: Vec<String>,
    pub nh_nl_diff: i64,
    /// (NH − NL) / evaluated × 100 — breadth thrust style normalization.
    pub nh_nl_pct: f64,
}

/// "New high" = last close within `PROXIMITY_PCT` of the 52-week max;
/// symmetric for lows. Strict equality would flag almost nothing on
/// daily closes.
const PROXIMITY_PCT: f64 = 2.0;

pub async fn nh_nl(pool: &PgPool) -> NhNlReport {
    let to = Utc::now();
    let from = to - Duration::days(370);
    let universe = crate::stock_recommendation::DEFAULT_UNIVERSE;
    let mut new_highs = Vec::new();
    let mut new_lows = Vec::new();
    let mut evaluated = 0usize;
    for sym in universe {
        let Ok(bars) = crate::prices::get_bars(pool, sym, BarInterval::D1, from, to).await else {
            continue;
        };
        if bars.len() < 100 {
            continue;
        }
        let f = |d: rust_decimal::Decimal| -> f64 { d.to_string().parse().unwrap_or(0.0) };
        let last = f(bars.last().expect("non-empty").close);
        let hi = bars.iter().map(|b| f(b.high)).fold(f64::MIN, f64::max);
        let lo = bars
            .iter()
            .map(|b| f(b.low))
            .filter(|v| *v > 0.0)
            .fold(f64::MAX, f64::min);
        if last <= 0.0 || hi <= 0.0 || lo == f64::MAX {
            continue;
        }
        evaluated += 1;
        if (hi - last) / hi * 100.0 <= PROXIMITY_PCT {
            new_highs.push(sym.to_string());
        } else if (last - lo) / last * 100.0 <= PROXIMITY_PCT {
            new_lows.push(sym.to_string());
        }
    }
    let diff = new_highs.len() as i64 - new_lows.len() as i64;
    let pct = if evaluated > 0 {
        diff as f64 / evaluated as f64 * 100.0
    } else {
        0.0
    };
    NhNlReport {
        universe_size: universe.len(),
        evaluated,
        new_highs,
        new_lows,
        nh_nl_diff: diff,
        nh_nl_pct: pct,
    }
}

// ===========================================================================
// Chowder rule
// ===========================================================================

#[derive(Debug, Clone, Serialize)]
pub struct ChowderReport {
    pub symbol: String,
    pub dividend_yield_pct: f64,
    pub dividend_cagr_5y_pct: f64,
    pub chowder_number: f64,
    /// 12 normally; 8 when yield ≥ 3% (slow-grower track per the rule).
    pub threshold: f64,
    pub passes: bool,
    pub annual_dividends: Vec<(i32, f64)>,
}

#[derive(Debug, thiserror::Error)]
pub enum ChowderError {
    #[error("dividend fetch failed: {0}")]
    Fetch(anyhow::Error),
    #[error("not enough dividend history for {symbol}: {years} years")]
    Insufficient { symbol: String, years: usize },
    #[error("no dividend yield available for {symbol}")]
    NoYield { symbol: String },
}

pub async fn chowder(symbol: &str) -> Result<ChowderReport, ChowderError> {
    let to = Utc::now().date_naive();
    let from = to - Duration::days(366 * 6 + 30);
    let divs = crate::finnhub_rest::stock_dividends(
        symbol,
        &from.format("%Y-%m-%d").to_string(),
        &to.format("%Y-%m-%d").to_string(),
    )
    .await
    .map_err(ChowderError::Fetch)?;
    // Sum dividends per calendar year. Finnhub returns an array of
    // {payDate|date, amount}.
    let mut per_year: std::collections::BTreeMap<i32, f64> = std::collections::BTreeMap::new();
    if let Some(arr) = divs.as_array() {
        for d in arr {
            let date_str = d
                .get("payDate")
                .or_else(|| d.get("date"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let amount = d.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0);
            if let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                *per_year.entry(date.year()).or_insert(0.0) += amount;
            }
        }
    }
    // Drop the (incomplete) current year; need 5 complete years + base.
    let this_year = to.year();
    per_year.remove(&this_year);
    let annual: Vec<(i32, f64)> = per_year.into_iter().collect();
    if annual.len() < 5 {
        return Err(ChowderError::Insufficient {
            symbol: symbol.to_string(),
            years: annual.len(),
        });
    }
    let recent = &annual[annual.len() - 5..];
    let first = recent.first().expect("len 5").1;
    let last = recent.last().expect("len 5").1;
    let cagr = if first > 0.0 {
        ((last / first).powf(1.0 / 4.0) - 1.0) * 100.0
    } else {
        0.0
    };
    // Yield from metric_all.
    let m = crate::finnhub_rest::metric_all(symbol)
        .await
        .map_err(ChowderError::Fetch)?;
    let yield_pct = m
        .get("metric")
        .and_then(|x| {
            x.get("currentDividendYieldTTM")
                .or_else(|| x.get("dividendYieldIndicatedAnnual"))
        })
        .and_then(|v| v.as_f64())
        .ok_or_else(|| ChowderError::NoYield {
            symbol: symbol.to_string(),
        })?;
    let chowder_number = yield_pct + cagr;
    // Per the rule: high-yield slow growers (≥3% yield) clear at 8;
    // everything else needs 12.
    let threshold = if yield_pct >= 3.0 { 8.0 } else { 12.0 };
    Ok(ChowderReport {
        symbol: symbol.to_string(),
        dividend_yield_pct: yield_pct,
        dividend_cagr_5y_pct: cagr,
        chowder_number,
        threshold,
        passes: chowder_number >= threshold,
        annual_dividends: recent.to_vec(),
    })
}