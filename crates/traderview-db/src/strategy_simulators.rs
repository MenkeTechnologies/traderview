//! Contribution-strategy simulators on REAL price history:
//!
//! * Value Averaging vs DCA — VA targets a value path (target grows by
//!   `target_growth` per month); each month you contribute whatever
//!   closes the gap to the path (selling when above it). DCA invests a
//!   flat amount. Same bars, same months, head-to-head.
//! * CPPI — Constant Proportion Portfolio Insurance: risky allocation
//!   = multiplier × (portfolio − floor), rebalanced monthly, cash leg
//!   earns `cash_rate`. Shows the equity curve + floor breach check.
//! * Dual Momentum (Antonacci GEM) — each month-end, hold the stronger
//!   of US/intl equities by trailing 12m return when either beats
//!   T-bills, else retreat to bonds. Equity curve vs buy-and-hold.
//!
//! All use cached daily bars (`prices::get_bars`), sampling the last
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

// ===========================================================================
// Dual Momentum (Antonacci GEM)
// ===========================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct GemInput {
    /// Backtest span in years (clamped 2..=20).
    pub years: u32,
    /// Momentum lookback in months (clamped 3..=12, GEM canon = 12).
    pub lookback_months: u32,
    /// US equity proxy (GEM canon: SPY).
    pub equity_us: String,
    /// International equity proxy (EFA).
    pub equity_intl: String,
    /// Absolute-momentum hurdle, T-bill proxy (BIL).
    pub tbill: String,
    /// Risk-off sleeve (AGG).
    pub bonds: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct GemMonthRow {
    pub date: chrono::NaiveDate,
    /// Which sleeve GEM held GOING INTO the next month.
    pub holding: String,
    pub gem_value: f64,
    pub buy_hold_value: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct GemReport {
    pub months: usize,
    /// Sleeve the rule selects at the latest month-end.
    pub current_signal: String,
    pub switches: usize,
    pub gem_return_pct: f64,
    pub buy_hold_return_pct: f64,
    pub gem_max_drawdown_pct: f64,
    pub buy_hold_max_drawdown_pct: f64,
    pub rows: Vec<GemMonthRow>,
}

/// Pure GEM walk over aligned monthly closes — unit-testable without a
/// database. `closes[i] = (date, [us, intl, tbill, bonds])`, oldest →
/// newest. `names` are the labels reported in `holding`.
pub fn dual_momentum_core(
    closes: &[(chrono::NaiveDate, [f64; 4])],
    names: &[String; 4],
    lookback: usize,
) -> Result<GemReport, SimError> {
    if closes.len() < lookback + 2 {
        return Err(SimError::Insufficient {
            symbol: names[0].clone(),
            got: closes.len(),
            need: lookback + 2,
        });
    }
    let ret = |asset: usize, i: usize, span: usize| -> f64 {
        let p0 = closes[i - span].1[asset];
        let p1 = closes[i].1[asset];
        if p0 > 0.0 {
            p1 / p0 - 1.0
        } else {
            0.0
        }
    };
    // Sleeve picked at month i: 0 = us, 1 = intl, 3 = bonds.
    let pick = |i: usize| -> usize {
        let (us, intl, tb) = (ret(0, i, lookback), ret(1, i, lookback), ret(2, i, lookback));
        if us.max(intl) > tb {
            if us >= intl {
                0
            } else {
                1
            }
        } else {
            3
        }
    };
    let mut gem = 1.0_f64;
    let mut bh = 1.0_f64;
    let (mut gem_peak, mut bh_peak) = (1.0_f64, 1.0_f64);
    let (mut gem_dd, mut bh_dd) = (0.0_f64, 0.0_f64);
    let mut switches = 0usize;
    let mut prev_sleeve: Option<usize> = None;
    let mut rows = Vec::new();
    for i in lookback..closes.len() - 1 {
        let sleeve = pick(i);
        if let Some(p) = prev_sleeve {
            if p != sleeve {
                switches += 1;
            }
        }
        prev_sleeve = Some(sleeve);
        // Hold the picked sleeve over month i → i+1.
        let growth = {
            let p0 = closes[i].1[sleeve];
            let p1 = closes[i + 1].1[sleeve];
            if p0 > 0.0 {
                p1 / p0
            } else {
                1.0
            }
        };
        gem *= growth;
        let bh_growth = {
            let p0 = closes[i].1[0];
            let p1 = closes[i + 1].1[0];
            if p0 > 0.0 {
                p1 / p0
            } else {
                1.0
            }
        };
        bh *= bh_growth;
        gem_peak = gem_peak.max(gem);
        bh_peak = bh_peak.max(bh);
        gem_dd = gem_dd.max((gem_peak - gem) / gem_peak * 100.0);
        bh_dd = bh_dd.max((bh_peak - bh) / bh_peak * 100.0);
        rows.push(GemMonthRow {
            date: closes[i].0,
            holding: names[sleeve].clone(),
            gem_value: gem,
            buy_hold_value: bh,
        });
    }
    let current = pick(closes.len() - 1);
    Ok(GemReport {
        months: rows.len(),
        current_signal: names[current].clone(),
        switches,
        gem_return_pct: (gem - 1.0) * 100.0,
        buy_hold_return_pct: (bh - 1.0) * 100.0,
        gem_max_drawdown_pct: gem_dd,
        buy_hold_max_drawdown_pct: bh_dd,
        rows,
    })
}

pub async fn dual_momentum(pool: &PgPool, input: &GemInput) -> Result<GemReport, SimError> {
    let years = input.years.clamp(2, 20);
    let lookback = input.lookback_months.clamp(3, 12) as usize;
    let symbols = [
        input.equity_us.trim().to_uppercase(),
        input.equity_intl.trim().to_uppercase(),
        input.tbill.trim().to_uppercase(),
        input.bonds.trim().to_uppercase(),
    ];
    if symbols.iter().any(|s| s.is_empty()) {
        return Err(SimError::BadInput("all four symbols are required"));
    }
    // Fetch monthly closes per sleeve, then inner-join on (year, month)
    // so a missing month in any sleeve drops that month everywhere.
    let mut maps = Vec::with_capacity(4);
    for sym in &symbols {
        let closes = monthly_closes(pool, sym, years).await?;
        let map: std::collections::BTreeMap<(i32, u32), (chrono::NaiveDate, f64)> = closes
            .into_iter()
            .map(|(d, p)| ((d.year(), d.month()), (d, p)))
            .collect();
        maps.push(map);
    }
    let aligned: Vec<(chrono::NaiveDate, [f64; 4])> = maps[0]
        .iter()
        .filter_map(|(key, (date, us))| {
            let intl = maps[1].get(key)?.1;
            let tb = maps[2].get(key)?.1;
            let bd = maps[3].get(key)?.1;
            Some((*date, [*us, intl, tb, bd]))
        })
        .collect();
    dual_momentum_core(&aligned, &symbols, lookback)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn names() -> [String; 4] {
        ["SPY", "EFA", "BIL", "AGG"].map(String::from)
    }

    /// `growth[asset]` = constant monthly growth factor per sleeve.
    fn synthetic(months: usize, growth: [f64; 4]) -> Vec<(NaiveDate, [f64; 4])> {
        let mut prices = [100.0_f64; 4];
        (0..months)
            .map(|i| {
                let date = NaiveDate::from_ymd_opt(2020 + i as i32 / 12, (i % 12) as u32 + 1, 28)
                    .expect("valid date");
                let snap = prices;
                for (p, g) in prices.iter_mut().zip(growth) {
                    *p *= g;
                }
                (date, snap)
            })
            .collect()
    }

    #[test]
    fn gem_holds_strongest_equity_in_bull_regime() {
        // US +2%/mo dominates intl +1%/mo and flat T-bills.
        let closes = synthetic(30, [1.02, 1.01, 1.0, 1.0]);
        let r = dual_momentum_core(&closes, &names(), 12).unwrap();
        assert_eq!(r.current_signal, "SPY");
        assert_eq!(r.switches, 0);
        assert!(r.rows.iter().all(|row| row.holding == "SPY"));
        // Holding the same sleeve as buy-and-hold ⇒ identical curves.
        assert!((r.gem_return_pct - r.buy_hold_return_pct).abs() < 1e-9);
    }

    #[test]
    fn gem_prefers_intl_when_it_outruns_us() {
        let closes = synthetic(30, [1.01, 1.02, 1.0, 1.0]);
        let r = dual_momentum_core(&closes, &names(), 12).unwrap();
        assert_eq!(r.current_signal, "EFA");
        assert!(r.gem_return_pct > r.buy_hold_return_pct);
    }

    #[test]
    fn gem_retreats_to_bonds_in_bear_regime() {
        // Both equities bleed 2%/mo; T-bills flat beat them ⇒ bonds.
        let closes = synthetic(30, [0.98, 0.98, 1.0, 1.003]);
        let r = dual_momentum_core(&closes, &names(), 12).unwrap();
        assert_eq!(r.current_signal, "AGG");
        assert!(r.rows.iter().all(|row| row.holding == "AGG"));
        // GEM sat in slowly-rising bonds while buy-and-hold bled.
        assert!(r.gem_return_pct > 0.0);
        assert!(r.buy_hold_return_pct < 0.0);
        assert!(r.gem_max_drawdown_pct < r.buy_hold_max_drawdown_pct);
    }

    #[test]
    fn gem_counts_regime_switches() {
        // 18 bull months then a hard bear: one switch into bonds once
        // the trailing window flips negative vs T-bills.
        let mut closes = synthetic(18, [1.03, 1.01, 1.0, 1.0]);
        let mut prices = closes.last().map(|(_, p)| *p).expect("non-empty");
        for i in 0..18 {
            let date = NaiveDate::from_ymd_opt(2022, (i % 12) as u32 + 1, 28).expect("valid");
            for (j, p) in prices.iter_mut().enumerate() {
                *p *= [0.95, 0.95, 1.0, 1.0][j];
            }
            closes.push((date, prices));
        }
        let r = dual_momentum_core(&closes, &names(), 12).unwrap();
        assert_eq!(r.current_signal, "AGG");
        assert!(r.switches >= 1, "switches = {}", r.switches);
        assert!(r.rows.first().expect("rows").holding == "SPY");
        assert!(r.rows.last().expect("rows").holding == "AGG");
    }

    #[test]
    fn gem_rejects_short_history() {
        let closes = synthetic(10, [1.01, 1.0, 1.0, 1.0]);
        assert!(dual_momentum_core(&closes, &names(), 12).is_err());
    }
}
