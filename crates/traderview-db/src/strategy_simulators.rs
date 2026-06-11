//! Contribution-strategy simulators on REAL price history:
//!
//! * Value Averaging vs DCA — VA targets a value path (target grows by
//!   `target_growth` per month); each month you contribute whatever
//!   closes the gap to the path (selling when above it). DCA invests a
//!   flat amount. Same bars, same months, head-to-head.
//! * CPPI — Constant Proportion Portfolio Insurance: risky allocation
//!   = multiplier × (portfolio − floor), rebalanced monthly, cash leg
//!   earns `cash_rate`. Shows the equity curve + floor breach check.
//!
//! Both use cached daily bars (`prices::get_bars`), sampling the last
//! close of each calendar month.

use chrono::{Datelike, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use traderview_core::BarInterval;

const MIN_MONTHS: usize = 12;

#[derive(Debug, thiserror::Error)]
pub enum SimError {
    #[error("not enough monthly closes for {symbol}: got {got}, need {need}")]
    Insufficient {
        symbol: String,
        got: usize,
        need: usize,
    },
    #[error("price fetch failed: {0}")]
    PriceFetch(anyhow::Error),
    #[error("invalid input: {0}")]
    BadInput(&'static str),
}

/// Last close of each calendar month, oldest→newest.
async fn monthly_closes(
    pool: &PgPool,
    symbol: &str,
    years: u32,
) -> Result<Vec<(chrono::NaiveDate, f64)>, SimError> {
    let to = Utc::now();
    let from = to - Duration::days(366 * years as i64 + 31);
    let bars = crate::prices::get_bars(pool, symbol, BarInterval::D1, from, to)
        .await
        .map_err(SimError::PriceFetch)?;
    let mut by_month: std::collections::BTreeMap<(i32, u32), (chrono::NaiveDate, f64)> =
        std::collections::BTreeMap::new();
    for b in &bars {
        let d = b.bar_time.date_naive();
        let close: f64 = b.close.to_string().parse().unwrap_or(0.0);
        if close > 0.0 {
            by_month.insert((d.year(), d.month()), (d, close));
        }
    }
    let out: Vec<_> = by_month.into_values().collect();
    if out.len() < MIN_MONTHS {
        return Err(SimError::Insufficient {
            symbol: symbol.to_string(),
            got: out.len(),
            need: MIN_MONTHS,
        });
    }
    Ok(out)
}

// ===========================================================================
// Value Averaging vs DCA
// ===========================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct VaInput {
    pub symbol: String,
    /// Backtest span in years (clamped 1..=20).
    pub years: u32,
    /// Monthly DCA contribution AND the monthly step of the VA target
    /// path, $.
    pub monthly_amount: f64,
    /// VA target path growth, %/month on the cumulative target
    /// (0.5 ≈ 6%/yr expectation).
    pub target_growth_pct_monthly: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct VaMonthRow {
    pub date: chrono::NaiveDate,
    pub price: f64,
    pub va_contribution: f64,
    pub va_value: f64,
    pub dca_value: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct VaReport {
    pub symbol: String,
    pub months: usize,
    pub va_total_contributed: f64,
    pub va_final_value: f64,
    pub va_return_pct: f64,
    pub dca_total_contributed: f64,
    pub dca_final_value: f64,
    pub dca_return_pct: f64,
    /// Positive = value averaging beat DCA on money-weighted return.
    pub va_edge_pct: f64,
    pub rows: Vec<VaMonthRow>,
}

pub async fn value_averaging(pool: &PgPool, input: &VaInput) -> Result<VaReport, SimError> {
    if input.monthly_amount <= 0.0 {
        return Err(SimError::BadInput("monthly_amount must be positive"));
    }
    let years = input.years.clamp(1, 20);
    let closes = monthly_closes(pool, &input.symbol, years).await?;
    let g = input.target_growth_pct_monthly / 100.0;

    let mut va_shares = 0.0_f64;
    let mut va_contributed = 0.0_f64;
    let mut dca_shares = 0.0_f64;
    let mut dca_contributed = 0.0_f64;
    let mut target = 0.0_f64;
    let mut rows = Vec::with_capacity(closes.len());
    for (i, (date, price)) in closes.iter().enumerate() {
        // VA: target path step t = amount × t, compounded at g.
        target = (target + input.monthly_amount) * (1.0 + g);
        let held = va_shares * price;
        let gap = target - held; // buy when below path, sell when above
        va_shares += gap / price;
        va_contributed += gap.max(0.0); // money-in only; sells return cash
        // DCA: flat buy.
        dca_shares += input.monthly_amount / price;
        dca_contributed += input.monthly_amount;
        let _ = i;
        rows.push(VaMonthRow {
            date: *date,
            price: *price,
            va_contribution: gap,
            va_value: va_shares * price,
            dca_value: dca_shares * price,
        });
    }
    let last_price = closes.last().map(|(_, p)| *p).unwrap_or(0.0);
    let va_final = va_shares * last_price;
    let dca_final = dca_shares * last_price;
    let pct = |fin: f64, contrib: f64| {
        if contrib > 0.0 {
            (fin - contrib) / contrib * 100.0
        } else {
            0.0
        }
    };
    let va_ret = pct(va_final, va_contributed);
    let dca_ret = pct(dca_final, dca_contributed);
    Ok(VaReport {
        symbol: input.symbol.clone(),
        months: closes.len(),
        va_total_contributed: va_contributed,
        va_final_value: va_final,
        va_return_pct: va_ret,
        dca_total_contributed: dca_contributed,
        dca_final_value: dca_final,
        dca_return_pct: dca_ret,
        va_edge_pct: va_ret - dca_ret,
        rows,
    })
}

// ===========================================================================
// CPPI
// ===========================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct CppiInput {
    pub symbol: String,
    pub years: u32,
    pub initial_capital: f64,
    /// Protected floor as a fraction of initial capital (e.g. 0.8).
    pub floor_fraction: f64,
    /// CPPI multiplier (3-6 typical).
    pub multiplier: f64,
    /// Annual cash yield % on the safe leg.
    pub cash_rate_pct: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CppiMonthRow {
    pub date: chrono::NaiveDate,
    pub portfolio: f64,
    pub risky_allocation: f64,
    pub cushion: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CppiReport {
    pub symbol: String,
    pub months: usize,
    pub final_value: f64,
    pub total_return_pct: f64,
    pub buy_and_hold_return_pct: f64,
    pub max_drawdown_pct: f64,
    pub floor_value: f64,
    pub floor_breached: bool,
    pub rows: Vec<CppiMonthRow>,
}

pub async fn cppi(pool: &PgPool, input: &CppiInput) -> Result<CppiReport, SimError> {
    if input.initial_capital <= 0.0 {
        return Err(SimError::BadInput("initial_capital must be positive"));
    }
    if !(0.0..1.0).contains(&input.floor_fraction) {
        return Err(SimError::BadInput("floor_fraction must be in [0, 1)"));
    }
    if input.multiplier <= 0.0 {
        return Err(SimError::BadInput("multiplier must be positive"));
    }
    let years = input.years.clamp(1, 20);
    let closes = monthly_closes(pool, &input.symbol, years).await?;
    let floor = input.initial_capital * input.floor_fraction;
    let monthly_cash = input.cash_rate_pct / 100.0 / 12.0;

    let mut portfolio = input.initial_capital;
    let mut peak = portfolio;
    let mut max_dd = 0.0_f64;
    let mut breached = false;
    let mut rows = Vec::with_capacity(closes.len());
    for w in closes.windows(2) {
        let (date, p0) = w[0];
        let (_, p1) = w[1];
        let cushion = (portfolio - floor).max(0.0);
        let risky = (input.multiplier * cushion).min(portfolio);
        let safe = portfolio - risky;
        let risky_ret = if p0 > 0.0 { p1 / p0 - 1.0 } else { 0.0 };
        portfolio = risky * (1.0 + risky_ret) + safe * (1.0 + monthly_cash);
        if portfolio < floor {
            breached = true;
        }
        peak = peak.max(portfolio);
        if peak > 0.0 {
            max_dd = max_dd.max((peak - portfolio) / peak * 100.0);
        }
        rows.push(CppiMonthRow {
            date,
            portfolio,
            risky_allocation: risky,
            cushion,
        });
    }
    let first_price = closes.first().map(|(_, p)| *p).unwrap_or(1.0);
    let last_price = closes.last().map(|(_, p)| *p).unwrap_or(1.0);
    let bh_ret = (last_price / first_price - 1.0) * 100.0;
    Ok(CppiReport {
        symbol: input.symbol.clone(),
        months: closes.len(),
        final_value: portfolio,
        total_return_pct: (portfolio / input.initial_capital - 1.0) * 100.0,
        buy_and_hold_return_pct: bh_ret,
        max_drawdown_pct: max_dd,
        floor_value: floor,
        floor_breached: breached,
        rows,
    })
}
