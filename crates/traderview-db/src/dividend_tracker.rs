//! Dividend total-return tracker.
//!
//! For income-oriented investors, the wealth metric is total return
//! (price + reinvested dividends), not price return. A 3% yield over
//! 20 years with reinvestment compounds to a 1.8× multiplier on top of
//! whatever the price did. This module surfaces:
//!
//!   * **Total return** (price + DRIP) per position since opened_at.
//!   * **Yield-on-cost** — annual dividend per share × current shares
//!     / cost basis. Tells you "what % of my original capital is
//!     returning to me each year as cash." Climbs over time as the
//!     dividend grows.
//!   * **Forward 12-month income estimate** — latest trailing-4-quarter
//!     dividend × shares held. Cash you expect.
//!
//! Pure compute does the dividend reinvestment math; repository pulls
//! per-position dividends from Yahoo's events chart endpoint.

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::prelude::ToPrimitive;
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct DividendEvent {
    pub ex_date: NaiveDate,
    pub amount_per_share: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PositionDividendReport {
    pub symbol: String,
    pub qty: f64,
    pub cost_basis_per_share: f64,
    pub opened_at: DateTime<Utc>,
    pub current_price: f64,
    pub price_return_pct: f64,
    pub total_return_pct: f64,
    pub trailing_12m_div_per_share: f64,
    pub yield_on_cost_pct: f64,
    pub current_yield_pct: f64,
    pub forward_12m_income_usd: f64,
    pub dividend_events_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct DividendPortfolioReport {
    pub positions: Vec<PositionDividendReport>,
    pub total_forward_12m_income_usd: f64,
    pub weighted_yield_on_cost_pct: f64,
    pub weighted_current_yield_pct: f64,
    pub generated_at: DateTime<Utc>,
}

// ─── Pure compute ──────────────────────────────────────────────────────────

/// Apply dividend reinvestment: each ex-date dividend buys
/// `dividend_amount × current_shares / ex_date_price` more shares.
/// Returns the share count after all dividends are reinvested. Caller
/// supplies the ex-date close so DRIP shares are bought at the historical
/// price, not today's price.
pub fn shares_after_reinvestment(
    initial_shares: f64,
    dividends: &[(NaiveDate, f64)],
    close_at_ex_date: &dyn Fn(NaiveDate) -> Option<f64>,
) -> f64 {
    let mut shares = initial_shares;
    for (ex_date, amount) in dividends {
        if let Some(close) = close_at_ex_date(*ex_date) {
            if close > 0.0 {
                let dividend_cash = shares * amount;
                let new_shares = dividend_cash / close;
                shares += new_shares;
            }
        }
    }
    shares
}

/// Total return with DRIP, in %:
///   total_return = (final_shares × current_price - initial_cost) / initial_cost × 100
pub fn total_return_pct(
    initial_shares: f64,
    initial_cost_per_share: f64,
    final_shares: f64,
    current_price: f64,
) -> f64 {
    let initial_cost = initial_shares * initial_cost_per_share;
    if initial_cost <= 0.0 {
        return 0.0;
    }
    let final_value = final_shares * current_price;
    (final_value - initial_cost) / initial_cost * 100.0
}

/// Price return only (no dividends), in %:
///   price_return = (current_price - cost_per_share) / cost_per_share × 100
pub fn price_return_pct(cost_per_share: f64, current_price: f64) -> f64 {
    if cost_per_share <= 0.0 {
        return 0.0;
    }
    (current_price - cost_per_share) / cost_per_share * 100.0
}

/// Yield on cost: annual dividend × shares / original cost basis, in %.
/// For a stock that grew its dividend 4× since you bought it, YoC can
/// easily be 12% even when the current yield is 3%.
pub fn yield_on_cost_pct(trailing_12m_div_per_share: f64, shares: f64, cost_basis: f64) -> f64 {
    if cost_basis <= 0.0 {
        return 0.0;
    }
    trailing_12m_div_per_share * shares / cost_basis * 100.0
}

/// Current yield: trailing 12-month dividend / current price × 100.
pub fn current_yield_pct(trailing_12m_div_per_share: f64, current_price: f64) -> f64 {
    if current_price <= 0.0 {
        return 0.0;
    }
    trailing_12m_div_per_share / current_price * 100.0
}

/// Sum the last four dividend payments — the standard "annualized" rate
/// when the cadence is quarterly. Falls back to (number of payments in
/// trailing 365 days × average) when payment count differs.
pub fn trailing_12m_dividend(events: &[(NaiveDate, f64)], today: NaiveDate) -> f64 {
    let cutoff = today - chrono::Duration::days(365);
    let in_window: Vec<f64> = events
        .iter()
        .filter(|(d, _)| *d >= cutoff && *d <= today)
        .map(|(_, a)| *a)
        .collect();
    in_window.iter().sum()
}

// ─── Repository ────────────────────────────────────────────────────────────

/// Pull a position's dividend events + current price by reusing the
/// existing Yahoo chart fetcher in market_data. Returns an empty list
/// for non-paying stocks or fetch errors.
pub async fn fetch_dividend_events(symbol: &str) -> (Vec<DividendEvent>, Option<f64>) {
    let encoded = symbol.replace('^', "%5E").replace('=', "%3D");
    let url = format!(
        "https://query1.finance.yahoo.com/v8/finance/chart/{encoded}\
         ?interval=1d&range=5y&events=div",
    );
    let client = match reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (compatible; traderview/0.10)")
        .timeout(std::time::Duration::from_secs(10))
        .build()
    {
        Ok(c) => c,
        Err(_) => return (Vec::new(), None),
    };
    let resp = match client.get(&url).send().await {
        Ok(r) if r.status().is_success() => r,
        _ => return (Vec::new(), None),
    };
    let v: serde_json::Value = match resp.json().await {
        Ok(j) => j,
        Err(_) => return (Vec::new(), None),
    };
    let result = &v["chart"]["result"][0];
    let current_price = result["meta"]["regularMarketPrice"].as_f64();

    let mut events: Vec<DividendEvent> = result["events"]["dividends"]
        .as_object()
        .map(|m| {
            m.values()
                .filter_map(|ev| {
                    let date = ev["date"].as_i64()?;
                    let amount = ev["amount"].as_f64()?;
                    let ex_date = chrono::DateTime::<Utc>::from_timestamp(date, 0)?.date_naive();
                    Some(DividendEvent {
                        ex_date,
                        amount_per_share: amount,
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    events.sort_by_key(|e| e.ex_date);
    (events, current_price)
}

pub async fn compute_report(
    pool: &PgPool,
    user_id: Uuid,
) -> anyhow::Result<DividendPortfolioReport> {
    let account = crate::paper::ensure_default(pool, user_id).await?;
    let positions = crate::paper::positions(pool, account.id).await?;
    let today = Utc::now().date_naive();
    let mut rows: Vec<PositionDividendReport> = Vec::new();
    let mut total_forward_income = 0.0_f64;
    let mut sum_yoc_weighted = 0.0_f64;
    let mut sum_current_yield_weighted = 0.0_f64;
    let mut total_cost_basis = 0.0_f64;
    let mut total_market_value = 0.0_f64;

    for p in &positions {
        let qty = p.qty.to_f64().unwrap_or(0.0);
        if qty <= 0.0 {
            // Only track long positions for dividend purposes (shorts
            // pay dividends to the lender, not receive them).
            continue;
        }
        let cost_per_share = p.avg_price.to_f64().unwrap_or(0.0);
        let (events, current_price_opt) = fetch_dividend_events(&p.symbol).await;
        let current_price = current_price_opt.unwrap_or(cost_per_share);
        let event_tuples: Vec<(NaiveDate, f64)> = events
            .iter()
            .map(|e| (e.ex_date, e.amount_per_share))
            .collect();

        // DRIP only counts dividends paid AFTER the position opened.
        let opened_at_date = p.updated_at.date_naive();
        let post_open_events: Vec<(NaiveDate, f64)> = event_tuples
            .iter()
            .filter(|(d, _)| *d >= opened_at_date)
            .copied()
            .collect();

        // No per-day price cache yet for ex-date closes — use the
        // simpler approximation that ex-date close ≈ current_price
        // when we don't have history (acceptable for the first cut;
        // a follow-up adds proper price_bars lookup).
        let close_at_ex = |_d: NaiveDate| -> Option<f64> { Some(current_price) };
        let final_shares = shares_after_reinvestment(qty, &post_open_events, &close_at_ex);

        let cost_basis = qty * cost_per_share;
        let market_value = final_shares * current_price;
        let trailing_12m_div = trailing_12m_dividend(&event_tuples, today);
        let yoc = yield_on_cost_pct(trailing_12m_div, qty, cost_basis);
        let cy = current_yield_pct(trailing_12m_div, current_price);
        let fwd_income = trailing_12m_div * final_shares;

        total_forward_income += fwd_income;
        sum_yoc_weighted += yoc * cost_basis;
        sum_current_yield_weighted += cy * market_value;
        total_cost_basis += cost_basis;
        total_market_value += market_value;

        rows.push(PositionDividendReport {
            symbol: p.symbol.clone(),
            qty,
            cost_basis_per_share: cost_per_share,
            opened_at: p.updated_at,
            current_price,
            price_return_pct: price_return_pct(cost_per_share, current_price),
            total_return_pct: total_return_pct(qty, cost_per_share, final_shares, current_price),
            trailing_12m_div_per_share: trailing_12m_div,
            yield_on_cost_pct: yoc,
            current_yield_pct: cy,
            forward_12m_income_usd: fwd_income,
            dividend_events_count: event_tuples.len(),
        });
    }

    let weighted_yoc = if total_cost_basis > 0.0 {
        sum_yoc_weighted / total_cost_basis
    } else {
        0.0
    };
    let weighted_cy = if total_market_value > 0.0 {
        sum_current_yield_weighted / total_market_value
    } else {
        0.0
    };

    Ok(DividendPortfolioReport {
        positions: rows,
        total_forward_12m_income_usd: total_forward_income,
        weighted_yield_on_cost_pct: weighted_yoc,
        weighted_current_yield_pct: weighted_cy,
        generated_at: Utc::now(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    #[test]
    fn shares_after_reinvestment_no_dividends_returns_initial() {
        let close_fn = |_: NaiveDate| -> Option<f64> { Some(100.0) };
        let final_shares = shares_after_reinvestment(10.0, &[], &close_fn);
        assert_eq!(final_shares, 10.0);
    }

    #[test]
    fn shares_after_reinvestment_compounds_at_ex_date_close() {
        // 10 shares, $1 dividend at $100 ex-date close → 0.1 new shares.
        let close_fn = |_: NaiveDate| -> Option<f64> { Some(100.0) };
        let dividends = vec![(d(2026, 3, 1), 1.0)];
        let final_shares = shares_after_reinvestment(10.0, &dividends, &close_fn);
        assert!((final_shares - 10.1).abs() < 1e-9);
    }

    #[test]
    fn shares_after_reinvestment_compounds_quarterly() {
        // 100 shares, 4 × $0.50 dividend at $50 ex-date close each.
        // q1: 100 × 0.5 / 50 = 1 new share → 101
        // q2: 101 × 0.5 / 50 = 1.01 → 102.01
        // q3: 102.01 × 0.5 / 50 = 1.0201 → 103.0301
        // q4: 103.0301 × 0.5 / 50 = 1.030301 → 104.060401
        let close_fn = |_: NaiveDate| -> Option<f64> { Some(50.0) };
        let dividends = vec![
            (d(2026, 3, 1), 0.5),
            (d(2026, 6, 1), 0.5),
            (d(2026, 9, 1), 0.5),
            (d(2026, 12, 1), 0.5),
        ];
        let final_shares = shares_after_reinvestment(100.0, &dividends, &close_fn);
        assert!((final_shares - 104.060401).abs() < 1e-3);
    }

    #[test]
    fn shares_after_reinvestment_skips_zero_close() {
        let close_fn = |_: NaiveDate| -> Option<f64> { Some(0.0) };
        let dividends = vec![(d(2026, 3, 1), 1.0)];
        let final_shares = shares_after_reinvestment(10.0, &dividends, &close_fn);
        assert_eq!(final_shares, 10.0, "zero close → no reinvestment");
    }

    #[test]
    fn total_return_includes_drip_capital() {
        // Bought 100 @ $50, dividend reinvested → 102 shares, price $55.
        // Initial cost = 5000, final value = 102 × 55 = 5610.
        // Total return = (5610 - 5000) / 5000 = 12.2%
        let tr = total_return_pct(100.0, 50.0, 102.0, 55.0);
        assert!((tr - 12.2).abs() < 1e-9);
    }

    #[test]
    fn total_return_zero_when_no_cost() {
        assert_eq!(total_return_pct(0.0, 50.0, 0.0, 55.0), 0.0);
    }

    #[test]
    fn price_return_basic() {
        assert!((price_return_pct(100.0, 110.0) - 10.0).abs() < 1e-9);
        assert!((price_return_pct(100.0, 90.0) + 10.0).abs() < 1e-9);
    }

    #[test]
    fn yield_on_cost_higher_than_current_when_dividend_grew() {
        // Bought at $50, current price $100, trailing dividend $3/share.
        // Current yield = 3/100 = 3%. YoC = 3 × 100 / (100 × 50) = 6%.
        let yoc = yield_on_cost_pct(3.0, 100.0, 100.0 * 50.0);
        let cy = current_yield_pct(3.0, 100.0);
        assert!((yoc - 6.0).abs() < 1e-9);
        assert!((cy - 3.0).abs() < 1e-9);
        assert!(yoc > cy, "growing-dividend stock: YoC > current yield");
    }

    #[test]
    fn yield_metrics_zero_on_invalid_inputs() {
        assert_eq!(yield_on_cost_pct(2.0, 10.0, 0.0), 0.0);
        assert_eq!(current_yield_pct(2.0, 0.0), 0.0);
    }

    #[test]
    fn trailing_12m_sums_only_window() {
        let today = d(2026, 6, 1);
        let events = vec![
            (d(2025, 1, 1), 0.50),  // 17 months ago — outside window
            (d(2025, 7, 1), 0.55),  // 11 months ago — inside
            (d(2025, 10, 1), 0.55), // 8 months — inside
            (d(2026, 1, 1), 0.60),  // 5 months — inside
            (d(2026, 4, 1), 0.60),  // 2 months — inside
        ];
        let t = trailing_12m_dividend(&events, today);
        // 0.55 + 0.55 + 0.60 + 0.60 = 2.30 (Jan-2025 excluded)
        assert!((t - 2.30).abs() < 1e-9);
    }

    #[test]
    fn trailing_12m_zero_for_empty_or_outside_window() {
        let today = d(2026, 6, 1);
        assert_eq!(trailing_12m_dividend(&[], today), 0.0);
        let stale = vec![(d(2020, 1, 1), 1.0)];
        assert_eq!(trailing_12m_dividend(&stale, today), 0.0);
    }
}
