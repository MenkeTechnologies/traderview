//! Faber-style sector momentum rotation strategy.
//!
//! Mebane Faber's "A Quantitative Approach to Tactical Asset Allocation"
//! (2007, updated 2013). For each month:
//!
//!   1. Score 11 sector ETFs by N-month total return (price momentum).
//!   2. Pick top K by momentum.
//!   3. Hold equal-weighted for one month.
//!   4. Rebalance monthly.
//!
//! Backtested: ~10-12% annualised return with lower drawdown than buy-and-
//! hold S&P. Edge persists because most retail investors don't follow a
//! systematic monthly rebalance rule and emotional drift dominates.
//!
//! This module simulates the strategy on cached price_bars and reports:
//!   * Per-month sector picks
//!   * Realized monthly returns
//!   * Annualized Sharpe + 95% CI per Andrew Lo
//!   * Max drawdown
//!   * Current month's top picks (live ranking)

use chrono::{Datelike, Duration, NaiveDate, Utc};
use rust_decimal::prelude::ToPrimitive;
use serde::Serialize;
use sqlx::PgPool;
use traderview_core::BarInterval;

/// Sector SPDR ETFs covering 11 GICS sectors.
pub const SECTOR_ETFS: &[&str] = &[
    "XLK", "XLF", "XLV", "XLE", "XLY", "XLP", "XLI", "XLB", "XLU", "XLRE", "XLC",
];

#[derive(Debug, Clone, Serialize)]
pub struct SectorMomentum {
    pub symbol: String,
    pub momentum_pct: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct MonthlyPick {
    pub period_end: NaiveDate,
    pub picks: Vec<String>,
    pub realized_return_pct: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct StrategyReport {
    pub lookback_months: u32,
    pub top_k: u32,
    pub monthly_picks: Vec<MonthlyPick>,
    pub current_momentum_ranking: Vec<SectorMomentum>,
    pub annualised_return_pct: f64,
    pub annualised_vol_pct: f64,
    pub annualised_sharpe: f64,
    pub sharpe_se: f64,
    pub sharpe_ci_lo_95: f64,
    pub sharpe_ci_hi_95: f64,
    pub max_drawdown_pct: f64,
    pub n_months: usize,
}

// ─── Pure compute ──────────────────────────────────────────────────────────

/// Find close on or after `target_date`. Returns None if no bar exists.
fn close_at_or_after(closes: &[(NaiveDate, f64)], target: NaiveDate) -> Option<(NaiveDate, f64)> {
    closes.iter().find(|(d, _)| *d >= target).copied()
}

/// Compute price momentum: (close_t / close_{t - N months} - 1) × 100.
/// Returns None if either close is missing.
pub fn momentum_pct(
    closes: &[(NaiveDate, f64)],
    as_of: NaiveDate,
    lookback_months: u32,
) -> Option<f64> {
    let lookback_date = subtract_months(as_of, lookback_months);
    // Reject when our earliest data starts AFTER the lookback date —
    // that means we don't actually have N months of history and the
    // momentum number would be misleading.
    let first_date = closes.first().map(|(d, _)| *d)?;
    if first_date > lookback_date {
        return None;
    }
    let (_, start) = close_at_or_after(closes, lookback_date)?;
    let (_, end) = close_at_or_after(closes, as_of)?;
    if start <= 0.0 || end <= 0.0 {
        return None;
    }
    Some((end / start - 1.0) * 100.0)
}

fn subtract_months(d: NaiveDate, n: u32) -> NaiveDate {
    let mut year = d.year();
    let mut month = d.month() as i32;
    month -= n as i32;
    while month <= 0 {
        month += 12;
        year -= 1;
    }
    NaiveDate::from_ymd_opt(year, month as u32, d.day()).unwrap_or(d)
}

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

/// Pick top K symbols by momentum descending. Returns symbol names.
pub fn pick_top_k(scores: &[SectorMomentum], k: usize) -> Vec<String> {
    let mut sorted = scores.to_vec();
    sorted.sort_by(|a, b| {
        b.momentum_pct
            .partial_cmp(&a.momentum_pct)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    sorted.into_iter().take(k).map(|s| s.symbol).collect()
}

/// Simulate the rotation strategy. Given per-ETF close series, walks
/// month-by-month from earliest available data + lookback_months
/// forward to today, picks top K each month, holds equal-weighted for
/// one month, accrues realized return.
///
/// Returns the per-month realized returns and the picks made for each
/// rebalance. Empty input → empty output.
pub fn simulate_strategy(
    closes_by_etf: &[(String, Vec<(NaiveDate, f64)>)],
    lookback_months: u32,
    top_k: u32,
    start: NaiveDate,
    end: NaiveDate,
) -> Vec<MonthlyPick> {
    if closes_by_etf.is_empty() || top_k == 0 {
        return Vec::new();
    }
    let mut out: Vec<MonthlyPick> = Vec::new();
    let mut period_start = start;
    while period_start < end {
        let period_end = add_months(period_start, 1);
        // Rank sectors by momentum AS OF period_start.
        let mut scores: Vec<SectorMomentum> = Vec::new();
        for (sym, closes) in closes_by_etf {
            if let Some(m) = momentum_pct(closes, period_start, lookback_months) {
                scores.push(SectorMomentum {
                    symbol: sym.clone(),
                    momentum_pct: m,
                });
            }
        }
        if scores.is_empty() {
            period_start = period_end;
            continue;
        }
        let picks = pick_top_k(&scores, top_k as usize);
        // Realized return = avg of picked sectors' returns over the holding month.
        let mut returns: Vec<f64> = Vec::new();
        for pick in &picks {
            let Some(closes) = closes_by_etf
                .iter()
                .find(|(s, _)| s == pick)
                .map(|(_, c)| c)
            else {
                continue;
            };
            let Some((_, p_start)) = close_at_or_after(closes, period_start) else {
                continue;
            };
            let Some((_, p_end)) = close_at_or_after(closes, period_end) else {
                continue;
            };
            if p_start <= 0.0 || p_end <= 0.0 {
                continue;
            }
            returns.push((p_end / p_start - 1.0) * 100.0);
        }
        let realized = if returns.is_empty() {
            0.0
        } else {
            returns.iter().sum::<f64>() / returns.len() as f64
        };
        out.push(MonthlyPick {
            period_end,
            picks,
            realized_return_pct: realized,
        });
        period_start = period_end;
    }
    out
}

/// Aggregate strategy stats from the per-month realized returns.
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

async fn fetch_etf_closes(pool: &PgPool, symbol: &str, days: i64) -> Vec<(NaiveDate, f64)> {
    let to = Utc::now();
    let from = to - Duration::days(days);
    crate::prices::get_bars(pool, symbol, BarInterval::D1, from, to)
        .await
        .unwrap_or_default()
        .into_iter()
        .filter_map(|b| b.close.to_f64().map(|c| (b.bar_time.date_naive(), c)))
        .collect()
}

pub async fn run_strategy(
    pool: &PgPool,
    days_back: i64,
    lookback_months: u32,
    top_k: u32,
) -> anyhow::Result<StrategyReport> {
    let mut closes_by_etf: Vec<(String, Vec<(NaiveDate, f64)>)> = Vec::new();
    for sym in SECTOR_ETFS {
        let closes = fetch_etf_closes(pool, sym, days_back + (lookback_months as i64 * 31)).await;
        if !closes.is_empty() {
            closes_by_etf.push((sym.to_string(), closes));
        }
    }
    let today = Utc::now().date_naive();
    let start = today - Duration::days(days_back);
    // Step forward to first valid month-start that has lookback data
    // available across the universe.
    let first_data_start = add_months(start, lookback_months);
    let picks = simulate_strategy(
        &closes_by_etf,
        lookback_months,
        top_k,
        first_data_start,
        today,
    );
    let monthly_returns: Vec<f64> = picks.iter().map(|p| p.realized_return_pct).collect();
    let (ann_ret, ann_vol, sharpe, sharpe_se, ci_lo, ci_hi, max_dd) =
        aggregate_stats(&monthly_returns);

    // Current ranking — what would we pick this month.
    let mut current_scores: Vec<SectorMomentum> = Vec::new();
    for (sym, closes) in &closes_by_etf {
        if let Some(m) = momentum_pct(closes, today, lookback_months) {
            current_scores.push(SectorMomentum {
                symbol: sym.clone(),
                momentum_pct: m,
            });
        }
    }
    current_scores.sort_by(|a, b| {
        b.momentum_pct
            .partial_cmp(&a.momentum_pct)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(StrategyReport {
        lookback_months,
        top_k,
        n_months: monthly_returns.len(),
        monthly_picks: picks,
        current_momentum_ranking: current_scores,
        annualised_return_pct: ann_ret,
        annualised_vol_pct: ann_vol,
        annualised_sharpe: sharpe,
        sharpe_se,
        sharpe_ci_lo_95: ci_lo,
        sharpe_ci_hi_95: ci_hi,
        max_drawdown_pct: max_dd,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    fn linear_closes(start: NaiveDate, days: usize, daily_pct: f64) -> Vec<(NaiveDate, f64)> {
        let mut p = 100.0_f64;
        (0..days)
            .map(|i| {
                let row = (start + Duration::days(i as i64), p);
                p *= 1.0 + daily_pct / 100.0;
                row
            })
            .collect()
    }

    #[test]
    fn subtract_months_basic() {
        assert_eq!(subtract_months(d(2026, 6, 15), 3), d(2026, 3, 15));
    }

    #[test]
    fn subtract_months_year_wrap() {
        assert_eq!(subtract_months(d(2026, 2, 15), 6), d(2025, 8, 15));
    }

    #[test]
    fn add_months_basic() {
        assert_eq!(add_months(d(2026, 1, 15), 3), d(2026, 4, 15));
    }

    #[test]
    fn add_months_year_wrap() {
        assert_eq!(add_months(d(2026, 11, 15), 3), d(2027, 2, 15));
    }

    #[test]
    fn momentum_returns_some_for_full_history() {
        let closes = linear_closes(d(2026, 1, 1), 365, 0.05);
        let m = momentum_pct(&closes, d(2026, 9, 1), 6);
        assert!(m.is_some());
        assert!(m.unwrap() > 0.0, "rising series → positive momentum");
    }

    #[test]
    fn momentum_returns_none_when_lookback_predates_data() {
        let closes = linear_closes(d(2026, 1, 1), 60, 0.05);
        // as_of - 6 months = 2025-08 — before data starts
        let m = momentum_pct(&closes, d(2026, 2, 1), 6);
        assert!(m.is_none());
    }

    #[test]
    fn pick_top_k_returns_highest_momentum_first() {
        let scores = vec![
            SectorMomentum {
                symbol: "A".into(),
                momentum_pct: 5.0,
            },
            SectorMomentum {
                symbol: "B".into(),
                momentum_pct: 15.0,
            },
            SectorMomentum {
                symbol: "C".into(),
                momentum_pct: 10.0,
            },
        ];
        let picks = pick_top_k(&scores, 2);
        assert_eq!(picks, vec!["B", "C"]);
    }

    #[test]
    fn pick_top_k_caps_at_input_size() {
        let scores = vec![SectorMomentum {
            symbol: "A".into(),
            momentum_pct: 5.0,
        }];
        let picks = pick_top_k(&scores, 5);
        assert_eq!(picks, vec!["A"]);
    }

    #[test]
    fn simulate_strategy_handles_empty_universe() {
        let picks = simulate_strategy(&[], 6, 3, d(2026, 1, 1), d(2026, 12, 1));
        assert!(picks.is_empty());
    }

    #[test]
    fn simulate_strategy_walks_month_by_month() {
        // 1 ETF, 1 year of rising data. 6m lookback, top-1. Should rotate
        // monthly with positive returns each period.
        let history = linear_closes(d(2025, 1, 1), 730, 0.05);
        let universe = vec![("A".to_string(), history)];
        let picks = simulate_strategy(&universe, 6, 1, d(2025, 7, 1), d(2026, 6, 1));
        assert!(!picks.is_empty());
        for p in &picks {
            assert_eq!(p.picks.len(), 1);
            assert_eq!(p.picks[0], "A");
            assert!(
                p.realized_return_pct > 0.0,
                "rising series → positive returns"
            );
        }
    }

    #[test]
    fn aggregate_stats_zero_on_insufficient_data() {
        let (a, b, c, d, e, f, g) = aggregate_stats(&[]);
        assert_eq!((a, b, c, d, e, f, g), (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0));
        let (a, b, c, d, e, f, g) = aggregate_stats(&[1.0]);
        assert_eq!((a, b, c, d, e, f, g), (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0));
    }

    #[test]
    fn aggregate_stats_computes_sharpe_with_ci() {
        // 12 months of +1% each → annualised ~12.68%, vol = 0, edge case.
        // Use slight variance to avoid the zero-stdev branch.
        let returns: Vec<f64> = (0..24)
            .map(|i| if i % 2 == 0 { 1.5 } else { 0.5 })
            .collect();
        let (_, ann_vol, sharpe, sharpe_se, _, _, _) = aggregate_stats(&returns);
        assert!(ann_vol > 0.0);
        assert!(sharpe > 0.0);
        assert!(sharpe_se > 0.0);
    }
}
