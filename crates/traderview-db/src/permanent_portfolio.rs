//! All-Weather / Permanent Portfolio backtest simulator.
//!
//! Backtests 4 canonical passive allocations on cached ETF bars:
//!
//!   * **All-Weather** (Dalio): 40% TLT / 30% VTI / 15% IEF / 7.5% GLD
//!     / 7.5% DBC. Risk-parity-ish across regimes (growth, recession,
//!     inflation, deflation).
//!   * **Permanent Portfolio** (Browne 1981): 25% each — VTI (stocks)
//!     + TLT (long bonds) + GLD (gold) + BIL (cash). One quadrant for
//!     each macro state.
//!   * **60/40 traditional**: 60% VTI + 40% AGG. The benchmark for
//!     boring balanced investing.
//!   * **100% S&P**: 100% SPY. Reference for "what you'd get just
//!     buying the index."
//!
//! Each portfolio is simulated monthly: at period start, rebalance to
//! the target weights using the current prices, compute the realized
//! return over the period from each constituent's price change × its
//! weight. Stats reported: annualised return, annualised vol, Sharpe
//! with 95% CI per Andrew Lo, max DD.

use chrono::{Datelike, Duration, NaiveDate, Utc};
use rust_decimal::prelude::ToPrimitive;
use serde::Serialize;
use sqlx::PgPool;
use traderview_core::BarInterval;

#[derive(Debug, Clone, Serialize)]
pub struct PortfolioConfig {
    pub name: &'static str,
    pub allocations: Vec<(&'static str, f64)>,
}

pub fn all_weather() -> PortfolioConfig {
    PortfolioConfig {
        name: "all_weather",
        allocations: vec![
            ("TLT", 0.40),
            ("VTI", 0.30),
            ("IEF", 0.15),
            ("GLD", 0.075),
            ("DBC", 0.075),
        ],
    }
}

pub fn permanent_portfolio() -> PortfolioConfig {
    PortfolioConfig {
        name: "permanent_portfolio",
        allocations: vec![("VTI", 0.25), ("TLT", 0.25), ("GLD", 0.25), ("BIL", 0.25)],
    }
}

pub fn sixty_forty() -> PortfolioConfig {
    PortfolioConfig {
        name: "60_40",
        allocations: vec![("VTI", 0.60), ("AGG", 0.40)],
    }
}

pub fn pure_sp500() -> PortfolioConfig {
    PortfolioConfig {
        name: "100_sp500",
        allocations: vec![("SPY", 1.00)],
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PortfolioStats {
    pub name: String,
    pub allocations: Vec<(String, f64)>,
    pub n_months: usize,
    pub annualised_return_pct: f64,
    pub annualised_vol_pct: f64,
    pub annualised_sharpe: f64,
    pub sharpe_se: f64,
    pub sharpe_ci_lo_95: f64,
    pub sharpe_ci_hi_95: f64,
    pub max_drawdown_pct: f64,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PortfolioComparisonReport {
    pub portfolios: Vec<PortfolioStats>,
    pub errors: Vec<String>,
}

// ─── Pure compute ──────────────────────────────────────────────────────────

fn add_months(d: NaiveDate, n: u32) -> NaiveDate {
    let mut year = d.year();
    let mut month = d.month() as i32;
    month += n as i32;
    while month > 12 {
        month -= 12;
        year += 1;
    }
    NaiveDate::from_ymd_opt(year, month as u32, d.day()).unwrap_or(d)
}

fn close_at_or_after(closes: &[(NaiveDate, f64)], target: NaiveDate) -> Option<f64> {
    closes.iter().find(|(d, _)| *d >= target).map(|(_, c)| *c)
}

/// Simulate monthly returns for one portfolio. At each period start,
/// rebalance to target weights at current prices; at period end, sum
/// weight × per-asset return. Skips periods where any constituent is
/// missing data.
pub fn simulate_portfolio(
    closes_by_symbol: &[(String, Vec<(NaiveDate, f64)>)],
    allocations: &[(&str, f64)],
    start: NaiveDate,
    end: NaiveDate,
) -> Vec<f64> {
    let mut returns: Vec<f64> = Vec::new();
    let mut period_start = start;
    while period_start < end {
        let period_end = add_months(period_start, 1);
        let mut realized = 0.0_f64;
        let mut covered = 0.0_f64;
        for (sym, weight) in allocations {
            let Some((_, closes)) = closes_by_symbol.iter().find(|(s, _)| s == sym) else {
                continue;
            };
            let Some(p_start) = close_at_or_after(closes, period_start) else {
                continue;
            };
            let Some(p_end) = close_at_or_after(closes, period_end) else {
                continue;
            };
            if p_start <= 0.0 || p_end <= 0.0 {
                continue;
            }
            let asset_return = (p_end / p_start - 1.0) * 100.0;
            realized += weight * asset_return;
            covered += weight;
        }
        if covered >= 0.99 {
            // Only count periods where we had data for ~all of the portfolio.
            returns.push(realized);
        }
        period_start = period_end;
    }
    returns
}

/// Same shape as sector_rotation_strategy::aggregate_stats — duplicated
/// here to keep the dep-graph clean (sector_rotation is its own module).
pub fn aggregate_stats(monthly_returns: &[f64]) -> (f64, f64, f64, f64, f64, f64, f64) {
    let n = monthly_returns.len();
    if n < 2 {
        return (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    }
    let mean = monthly_returns.iter().sum::<f64>() / n as f64;
    let var = monthly_returns
        .iter()
        .map(|r| (r - mean).powi(2))
        .sum::<f64>()
        / (n - 1) as f64;
    let stdev = var.sqrt();
    let annualised_return = (1.0 + mean / 100.0).powi(12) - 1.0;
    let annualised_return_pct = annualised_return * 100.0;
    let annualised_vol_pct = stdev * 12.0_f64.sqrt();
    let annualised_sharpe = if annualised_vol_pct > 0.0 {
        annualised_return_pct / annualised_vol_pct
    } else {
        0.0
    };
    let sharpe_se = ((1.0 + 0.5 * annualised_sharpe * annualised_sharpe) / n as f64).sqrt();
    let ci_lo = annualised_sharpe - 1.96 * sharpe_se;
    let ci_hi = annualised_sharpe + 1.96 * sharpe_se;
    let mut cum = 0.0_f64;
    let mut peak = f64::NEG_INFINITY;
    let mut max_dd = 0.0_f64;
    for r in monthly_returns {
        cum += r;
        if cum > peak {
            peak = cum;
        }
        let dd = peak - cum;
        if dd > max_dd {
            max_dd = dd;
        }
    }
    (
        annualised_return_pct,
        annualised_vol_pct,
        annualised_sharpe,
        sharpe_se,
        ci_lo,
        ci_hi,
        max_dd,
    )
}

// ─── Repository ────────────────────────────────────────────────────────────

async fn fetch_closes(pool: &PgPool, symbol: &str, days: i64) -> Vec<(NaiveDate, f64)> {
    let to = Utc::now();
    let from = to - Duration::days(days);
    crate::prices::get_bars(pool, symbol, BarInterval::D1, from, to)
        .await
        .unwrap_or_default()
        .into_iter()
        .filter_map(|b| b.close.to_f64().map(|c| (b.bar_time.date_naive(), c)))
        .collect()
}

pub async fn compare(pool: &PgPool, days_back: i64) -> PortfolioComparisonReport {
    let portfolios = vec![
        all_weather(),
        permanent_portfolio(),
        sixty_forty(),
        pure_sp500(),
    ];
    // Union of all symbols across portfolios.
    let mut symbols: Vec<&'static str> = Vec::new();
    for p in &portfolios {
        for (s, _) in &p.allocations {
            if !symbols.contains(s) {
                symbols.push(*s);
            }
        }
    }
    let mut closes_by_symbol: Vec<(String, Vec<(NaiveDate, f64)>)> = Vec::new();
    let mut errors: Vec<String> = Vec::new();
    for s in &symbols {
        let closes = fetch_closes(pool, s, days_back).await;
        if closes.is_empty() {
            errors.push(format!("{s}: no cached bars"));
        } else {
            closes_by_symbol.push(((*s).into(), closes));
        }
    }
    let today = Utc::now().date_naive();
    let start = today - Duration::days(days_back);
    let mut stats: Vec<PortfolioStats> = Vec::new();
    for p in &portfolios {
        let returns = simulate_portfolio(&closes_by_symbol, &p.allocations, start, today);
        let (ann_ret, ann_vol, sharpe, sharpe_se, ci_lo, ci_hi, max_dd) = aggregate_stats(&returns);
        let allocations: Vec<(String, f64)> = p
            .allocations
            .iter()
            .map(|(s, w)| ((*s).into(), *w))
            .collect();
        let note = if returns.is_empty() {
            Some("no months simulated — check ETF coverage in cached bars".into())
        } else {
            None
        };
        stats.push(PortfolioStats {
            name: p.name.into(),
            allocations,
            n_months: returns.len(),
            annualised_return_pct: ann_ret,
            annualised_vol_pct: ann_vol,
            annualised_sharpe: sharpe,
            sharpe_se,
            sharpe_ci_lo_95: ci_lo,
            sharpe_ci_hi_95: ci_hi,
            max_drawdown_pct: max_dd,
            note,
        });
    }
    stats.sort_by(|a, b| {
        b.annualised_sharpe
            .partial_cmp(&a.annualised_sharpe)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    PortfolioComparisonReport {
        portfolios: stats,
        errors,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    fn flat_closes(start: NaiveDate, n: usize, price: f64) -> Vec<(NaiveDate, f64)> {
        (0..n)
            .map(|i| (start + Duration::days(i as i64), price))
            .collect()
    }

    #[test]
    fn all_weather_allocations_sum_to_one() {
        let cfg = all_weather();
        let total: f64 = cfg.allocations.iter().map(|(_, w)| w).sum();
        assert!((total - 1.0).abs() < 1e-9);
    }

    #[test]
    fn permanent_portfolio_allocations_sum_to_one() {
        let cfg = permanent_portfolio();
        let total: f64 = cfg.allocations.iter().map(|(_, w)| w).sum();
        assert!((total - 1.0).abs() < 1e-9);
    }

    #[test]
    fn sixty_forty_allocations_sum_to_one() {
        let cfg = sixty_forty();
        let total: f64 = cfg.allocations.iter().map(|(_, w)| w).sum();
        assert!((total - 1.0).abs() < 1e-9);
    }

    #[test]
    fn pure_sp500_is_single_holding() {
        let cfg = pure_sp500();
        assert_eq!(cfg.allocations.len(), 1);
        assert_eq!(cfg.allocations[0].0, "SPY");
    }

    #[test]
    fn simulate_flat_portfolio_zero_returns() {
        // All constituents at flat prices → zero return each month.
        let closes_a = flat_closes(d(2026, 1, 1), 365, 100.0);
        let closes_b = flat_closes(d(2026, 1, 1), 365, 50.0);
        let universe = vec![("A".to_string(), closes_a), ("B".to_string(), closes_b)];
        let r = simulate_portfolio(
            &universe,
            &[("A", 0.5), ("B", 0.5)],
            d(2026, 1, 1),
            d(2026, 12, 1),
        );
        for v in &r {
            assert!(v.abs() < 1e-9);
        }
    }

    #[test]
    fn simulate_skips_periods_when_coverage_incomplete() {
        // One constituent has data, other doesn't.
        let closes_a = flat_closes(d(2026, 1, 1), 365, 100.0);
        let universe = vec![("A".to_string(), closes_a)];
        // Allocation references B which has no data.
        let r = simulate_portfolio(
            &universe,
            &[("A", 0.5), ("B", 0.5)],
            d(2026, 1, 1),
            d(2026, 12, 1),
        );
        // Coverage 0.5 < 0.99 → all periods skipped.
        assert!(r.is_empty());
    }

    #[test]
    fn aggregate_stats_zero_on_empty() {
        let (a, b, c, d, e, f, g) = aggregate_stats(&[]);
        assert_eq!((a, b, c, d, e, f, g), (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0));
    }

    #[test]
    fn aggregate_stats_basic_sharpe() {
        let returns = vec![1.0, 0.5, 1.5, 0.5, 1.0, 0.5, 1.5, 0.5, 1.0, 0.5, 1.5, 0.5];
        let (ann_ret, ann_vol, sharpe, _, _, _, _) = aggregate_stats(&returns);
        assert!(ann_ret > 0.0);
        assert!(ann_vol > 0.0);
        assert!(sharpe > 0.0);
    }
}
