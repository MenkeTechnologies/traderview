//! Dividend capture / arbitrage scanner.
//!
//! Dividend capture is the textbook strategy: buy a stock the day
//! before ex-dividend, collect the dividend, sell after the price
//! recovers. Empirically the ex-date price drop captures only ~60-80%
//! of the dividend (the remainder is the long-side retention edge —
//! attributable to tax-clientele effects and short-term price
//! microstructure). Annualised across 4 quarterly cycles, the
//! retained portion compounds into a defensible edge for liquid,
//! moderate-yield names where the short side is also feasible.
//!
//! This module surfaces capture candidates by combining:
//!
//!   1. `market_data::dividends(symbol)` — Yahoo trailing-4-quarter
//!      total + current price → annualised yield.
//!   2. `short_interest::finnhub_short_stats(symbol)` — short_pct_float
//!      mapped to a coarse **borrow-cost proxy** (no paid IB SLB feed):
//!        * <5%   → 0.25% (easy-to-borrow)
//!        * 5-15% → 2.0%  (general collateral up to mildly hard)
//!        * 15-30% → 8.0% (hard-to-borrow)
//!        * >30%  → 25.0% (extreme HTB)
//!   3. Pure compute combines them into:
//!        * `long_capture_edge_pct` = `annual_yield × LONG_RETENTION_PCT`
//!          (retention defaults to 30%, conservative vs the 60-80%
//!          drop convention)
//!        * `short_arb_edge_pct` = `(1 - LONG_RETENTION_PCT) × annual_yield
//!          − borrow_proxy − tx_friction`
//!      The two variants together let the user see whether a name is
//!      better for the long-only capture or the short-hedged arb.
//!
//! Ranked by `max(long_capture, short_arb)` descending so the best
//! per-name variant surfaces first. Names with negative edges on
//! both sides are filtered.

use chrono::{DateTime, Duration, Utc};
use rust_decimal::prelude::ToPrimitive;
use serde::Serialize;
use sqlx::PgPool;

use crate::market_data;
use crate::short_interest;

const MAX_DAYS_TO_EX: i64 = 45;
const LONG_RETENTION_PCT: f64 = 0.30;
const TX_FRICTION_PCT: f64 = 0.05; // 0.05% round-trip — generous for retail.

#[derive(Debug, Clone, Serialize)]
pub struct DividendCaptureRow {
    pub symbol: String,
    pub price: f64,
    pub trailing_4q_dividend: f64,
    pub annual_yield_pct: f64,
    pub ex_dividend_date: Option<chrono::NaiveDate>,
    pub days_to_ex: Option<i64>,
    pub short_pct_float: Option<f64>,
    pub borrow_cost_proxy_pct: f64,
    /// Annualised retained-dividend long-capture edge after expected
    /// price drop. Positive means buying pre-ex / selling post-ex has
    /// positive EV.
    pub long_capture_edge_pct: f64,
    /// Annualised short-side edge: capture the (1 - retention) × yield
    /// price drop, pay the borrow + tx friction.
    pub short_arb_edge_pct: f64,
    /// `max(long_capture_edge, short_arb_edge)` — the better of the two
    /// variants. Used as the rank key.
    pub best_edge_pct: f64,
    pub observed_at: DateTime<Utc>,
}

// ─── Pure compute ──────────────────────────────────────────────────────────

/// Map a short_pct_float (0-1 fraction) to a borrow cost proxy in
/// annualised percent. Conservative — real IB SLB rates can be either
/// side of these tiers, but this captures the structural shape.
pub fn borrow_cost_proxy(short_pct_float: Option<f64>) -> f64 {
    match short_pct_float {
        Some(p) if p.is_finite() && p > 0.0 => {
            let pct = p * 100.0;
            if pct < 5.0 {
                0.25
            } else if pct < 15.0 {
                2.0
            } else if pct < 30.0 {
                8.0
            } else {
                25.0
            }
        }
        _ => 0.25, // Unknown → assume easy-to-borrow.
    }
}

/// Pure: given the raw facts, compute the capture row. `None` returned
/// when neither the long nor the short variant offers positive edge
/// (caller filters these out of the surface).
pub fn compute_row(
    symbol: &str,
    price: Option<f64>,
    trailing_4q_dividend: Option<f64>,
    ex_dividend_date: Option<chrono::NaiveDate>,
    short_pct_float: Option<f64>,
    now: chrono::NaiveDate,
) -> Option<DividendCaptureRow> {
    let price = price.filter(|p| p.is_finite() && *p > 0.0)?;
    let div = trailing_4q_dividend.filter(|d| d.is_finite() && *d > 0.0)?;
    let annual_yield_pct = div / price * 100.0;
    let days_to_ex = ex_dividend_date.map(|d| (d - now).num_days());
    let borrow_cost_proxy_pct = borrow_cost_proxy(short_pct_float);

    let long_edge = annual_yield_pct * LONG_RETENTION_PCT - TX_FRICTION_PCT;
    let short_edge =
        annual_yield_pct * (1.0 - LONG_RETENTION_PCT) - borrow_cost_proxy_pct - TX_FRICTION_PCT;
    let best = long_edge.max(short_edge);
    if best <= 0.0 {
        return None;
    }
    Some(DividendCaptureRow {
        symbol: symbol.to_ascii_uppercase(),
        price,
        trailing_4q_dividend: div,
        annual_yield_pct,
        ex_dividend_date,
        days_to_ex,
        short_pct_float,
        borrow_cost_proxy_pct,
        long_capture_edge_pct: long_edge,
        short_arb_edge_pct: short_edge,
        best_edge_pct: best,
        observed_at: Utc::now(),
    })
}

// ─── Repository / fetch ────────────────────────────────────────────────────

fn parse_yahoo_dividends(
    body: &serde_json::Value,
) -> (Option<f64>, Option<f64>, Option<chrono::NaiveDate>) {
    let raw = |path: &[&str]| -> Option<f64> {
        let mut cur = body;
        for k in path {
            cur = cur.get(k)?;
        }
        cur.as_f64()
            .or_else(|| cur.get("raw").and_then(|r| r.as_f64()))
    };
    let price = raw(&["summaryDetail", "regularMarketPrice"]);
    let div = raw(&["summaryDetail", "dividendRate"]);
    let ex_ts = raw(&["summaryDetail", "exDividendDate"])
        .or_else(|| raw(&["calendarEvents", "exDividendDate"]));
    let ex_date =
        ex_ts.and_then(|t| DateTime::<Utc>::from_timestamp(t as i64, 0).map(|d| d.date_naive()));
    (price, div, ex_date)
}

/// Best-effort: fetch dividend + short stats and compute the row for
/// one symbol. Returns None if the symbol pays no dividend, has no
/// upcoming ex-date within MAX_DAYS_TO_EX, or yields no positive-edge
/// variant.
pub async fn refresh_symbol(
    pool: &PgPool,
    symbol: &str,
) -> anyhow::Result<Option<DividendCaptureRow>> {
    let body = market_data::dividends(symbol).await?;
    let (price, div, ex_date) = parse_yahoo_dividends(&body);
    let today = Utc::now().date_naive();
    if let Some(d) = ex_date {
        let dte = (d - today).num_days();
        if dte < -2 || dte > MAX_DAYS_TO_EX {
            // Too far past or too far in the future to be actionable.
            // Allow up to -2 days because Yahoo's exDividendDate lags by
            // a day on some feeds.
            return Ok(None);
        }
    }
    // Short stats are best-effort — if Finnhub fails or no key, borrow
    // proxy defaults to the easy-to-borrow tier.
    let short_pct_float = match short_interest::finnhub_short_stats(symbol).await {
        Ok(s) => s.short_pct_float,
        Err(e) => {
            tracing::debug!(?e, symbol, "div_capture: short stats fetch failed");
            None
        }
    };
    Ok(compute_row(
        symbol,
        price,
        div,
        ex_date,
        short_pct_float,
        today,
    ))
}

/// Scan a symbol list. Returns rows newest-first, ranked by
/// `best_edge_pct` descending.
pub async fn scan(pool: &PgPool, symbols: &[String]) -> Vec<DividendCaptureRow> {
    let mut rows: Vec<DividendCaptureRow> = Vec::new();
    for sym in symbols {
        match refresh_symbol(pool, sym).await {
            Ok(Some(r)) => rows.push(r),
            Ok(None) => {}
            Err(e) => {
                tracing::debug!(?e, symbol = %sym, "div_capture: refresh failed");
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(120)).await;
    }
    rows.sort_by(|a, b| {
        b.best_edge_pct
            .partial_cmp(&a.best_edge_pct)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> chrono::NaiveDate {
        chrono::NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    #[test]
    fn borrow_proxy_tiers() {
        assert_eq!(borrow_cost_proxy(Some(0.02)), 0.25); // 2% → easy
        assert_eq!(borrow_cost_proxy(Some(0.10)), 2.0); // 10% → general collateral
        assert_eq!(borrow_cost_proxy(Some(0.20)), 8.0); // 20% → HTB
        assert_eq!(borrow_cost_proxy(Some(0.40)), 25.0); // 40% → extreme HTB
        assert_eq!(borrow_cost_proxy(None), 0.25); // unknown → default
    }

    #[test]
    fn compute_row_emits_long_edge_for_high_yield_easy_borrow() {
        // 5% annual yield, easy-to-borrow (1% short_pct).
        // long_edge = 5 × 0.30 − 0.05 = 1.45
        // short_edge = 5 × 0.70 − 0.25 − 0.05 = 3.20
        // best = 3.20 (short side wins for easy-borrow / high-yield)
        let r = compute_row(
            "AAA",
            Some(100.0),
            Some(5.0),
            Some(d(2026, 7, 1)),
            Some(0.01),
            d(2026, 6, 9),
        )
        .expect("should emit");
        assert!((r.annual_yield_pct - 5.0).abs() < 1e-9);
        assert!((r.long_capture_edge_pct - 1.45).abs() < 1e-9);
        assert!((r.short_arb_edge_pct - 3.20).abs() < 1e-9);
        assert!((r.best_edge_pct - 3.20).abs() < 1e-9);
        assert_eq!(r.days_to_ex, Some(22));
    }

    #[test]
    fn compute_row_filters_when_no_edge() {
        // Tiny yield (0.1%), huge borrow cost → neither variant positive.
        // long_edge = 0.1 × 0.30 − 0.05 = -0.02
        // short_edge = 0.1 × 0.70 − 25 − 0.05 = -24.98
        let r = compute_row(
            "ZERO",
            Some(100.0),
            Some(0.1),
            Some(d(2026, 7, 1)),
            Some(0.40), // extreme HTB
            d(2026, 6, 9),
        );
        assert!(r.is_none());
    }

    #[test]
    fn compute_row_filters_zero_price_or_dividend() {
        let now = d(2026, 6, 9);
        assert!(compute_row("X", Some(0.0), Some(1.0), None, None, now).is_none());
        assert!(compute_row("X", Some(100.0), Some(0.0), None, None, now).is_none());
        assert!(compute_row("X", None, Some(1.0), None, None, now).is_none());
        assert!(compute_row("X", Some(100.0), None, None, None, now).is_none());
    }

    #[test]
    fn compute_row_handles_missing_ex_date() {
        // No ex-date → row still computes (we surface annual yield) but
        // days_to_ex stays None.
        let r = compute_row(
            "NOEX",
            Some(100.0),
            Some(5.0),
            None,
            Some(0.01),
            d(2026, 6, 9),
        )
        .expect("should emit");
        assert!(r.days_to_ex.is_none());
        assert!(r.ex_dividend_date.is_none());
    }

    #[test]
    fn short_arb_negative_when_borrow_too_expensive() {
        // 4% yield + extreme HTB (25% borrow). long_edge = 4 × 0.30 − 0.05 = 1.15.
        // short_edge = 4 × 0.70 − 25 − 0.05 = -22.25. Long edge wins.
        let r = compute_row(
            "HTB",
            Some(100.0),
            Some(4.0),
            Some(d(2026, 7, 1)),
            Some(0.40),
            d(2026, 6, 9),
        )
        .expect("long-only edge should still emit");
        assert!(r.long_capture_edge_pct > 0.0);
        assert!(r.short_arb_edge_pct < 0.0);
        assert!((r.best_edge_pct - r.long_capture_edge_pct).abs() < 1e-9);
    }

    #[test]
    fn parse_yahoo_dividends_extracts_raw_fields() {
        let body = serde_json::json!({
            "summaryDetail": {
                "regularMarketPrice": { "raw": 100.0 },
                "dividendRate": { "raw": 2.5 },
                "exDividendDate": { "raw": 1_780_000_000_i64 }
            }
        });
        let (p, d, e) = parse_yahoo_dividends(&body);
        assert_eq!(p, Some(100.0));
        assert_eq!(d, Some(2.5));
        assert!(e.is_some());
    }

    #[test]
    fn parse_yahoo_dividends_handles_bare_numbers() {
        let body = serde_json::json!({
            "summaryDetail": {
                "regularMarketPrice": 100.0,
                "dividendRate": 2.5
            }
        });
        let (p, d, e) = parse_yahoo_dividends(&body);
        assert_eq!(p, Some(100.0));
        assert_eq!(d, Some(2.5));
        assert!(e.is_none());
    }

    #[test]
    fn parse_yahoo_dividends_returns_none_on_empty() {
        let body = serde_json::json!({});
        let (p, d, e) = parse_yahoo_dividends(&body);
        assert!(p.is_none());
        assert!(d.is_none());
        assert!(e.is_none());
    }
}
