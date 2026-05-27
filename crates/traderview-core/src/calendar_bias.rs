//! Calendar-effect P&L bias scanner.
//!
//! Some traders perform measurably worse on:
//!   - Mondays after weekend (sleepy, fading gap)
//!   - Friday afternoons (rushing for week-end)
//:   - Earnings weeks (trading what they don't know)
//!   - FOMC days (volatile)
//!   - First/last week of the month (rebalance flows)
//!
//! This module buckets per-trade P&L by the calendar flags the trader
//! tagged and exposes per-bucket statistics so they can spot patterns.
//!
//! Pure compute. Caller tags trades with the relevant flags upstream
//! (this module doesn't embed a calendar — that's reference data).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarTaggedTrade {
    pub pnl: f64,
    pub is_monday: bool,
    pub is_friday: bool,
    pub is_earnings_week: bool,
    pub is_fomc_day: bool,
    pub is_first_week_of_month: bool,
    pub is_last_week_of_month: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiasBucket {
    pub label: String,
    pub trade_count: usize,
    pub total_pnl: f64,
    pub avg_pnl: f64,
    pub win_count: usize,
    pub win_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CalendarBiasReport {
    pub buckets: Vec<BiasBucket>,
    /// Buckets where avg_pnl is significantly negative — caller's
    /// "stop trading on these days" recommendations.
    pub problem_buckets: Vec<String>,
}

pub fn analyze(trades: &[CalendarTaggedTrade]) -> CalendarBiasReport {
    let mut report = CalendarBiasReport::default();
    if trades.is_empty() { return report; }
    let collect = |label: &str, filter: fn(&CalendarTaggedTrade) -> bool| -> BiasBucket {
        let filtered: Vec<_> = trades.iter().filter(|t| filter(t)).collect();
        let n = filtered.len();
        if n == 0 {
            return BiasBucket {
                label: label.into(), trade_count: 0,
                total_pnl: 0.0, avg_pnl: 0.0,
                win_count: 0, win_rate: 0.0,
            };
        }
        let total: f64 = filtered.iter().map(|t| t.pnl).sum();
        let wins = filtered.iter().filter(|t| t.pnl > 0.0).count();
        BiasBucket {
            label: label.into(),
            trade_count: n,
            total_pnl: total,
            avg_pnl: total / n as f64,
            win_count: wins,
            win_rate: wins as f64 / n as f64,
        }
    };
    report.buckets = vec![
        collect("Monday",         |t| t.is_monday),
        collect("Friday",         |t| t.is_friday),
        collect("Earnings Week",  |t| t.is_earnings_week),
        collect("FOMC Day",       |t| t.is_fomc_day),
        collect("First Week",     |t| t.is_first_week_of_month),
        collect("Last Week",      |t| t.is_last_week_of_month),
    ];
    // Flag buckets where avg_pnl is negative AND trade count is meaningful.
    for b in &report.buckets {
        if b.trade_count >= 5 && b.avg_pnl < 0.0 {
            report.problem_buckets.push(b.label.clone());
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t_default() -> CalendarTaggedTrade {
        CalendarTaggedTrade {
            pnl: 0.0,
            is_monday: false,
            is_friday: false,
            is_earnings_week: false,
            is_fomc_day: false,
            is_first_week_of_month: false,
            is_last_week_of_month: false,
        }
    }

    fn monday(pnl: f64) -> CalendarTaggedTrade {
        CalendarTaggedTrade { is_monday: true, pnl, ..t_default() }
    }
    fn fomc(pnl: f64) -> CalendarTaggedTrade {
        CalendarTaggedTrade { is_fomc_day: true, pnl, ..t_default() }
    }

    #[test]
    fn empty_returns_default() {
        let r = analyze(&[]);
        assert!(r.buckets.is_empty());
    }

    #[test]
    fn monday_bucket_aggregates_correctly() {
        let trades = vec![monday(100.0), monday(-50.0), monday(200.0)];
        let r = analyze(&trades);
        let mon = r.buckets.iter().find(|b| b.label == "Monday").unwrap();
        assert_eq!(mon.trade_count, 3);
        assert_eq!(mon.total_pnl, 250.0);
        assert!((mon.avg_pnl - 83.333333).abs() < 1e-3);
        assert_eq!(mon.win_count, 2);
        assert!((mon.win_rate - 2.0/3.0).abs() < 1e-9);
    }

    #[test]
    fn problem_bucket_flagged_when_avg_negative_and_n_ge_5() {
        // 5 losing FOMC trades → bucket flagged.
        let trades: Vec<_> = (0..5).map(|_| fomc(-100.0)).collect();
        let r = analyze(&trades);
        assert!(r.problem_buckets.contains(&"FOMC Day".to_string()));
    }

    #[test]
    fn problem_bucket_not_flagged_when_under_5_trades() {
        let trades: Vec<_> = (0..3).map(|_| fomc(-100.0)).collect();
        let r = analyze(&trades);
        assert!(!r.problem_buckets.contains(&"FOMC Day".to_string()),
            "needs ≥ 5 trades for statistical meaningfulness");
    }

    #[test]
    fn positive_avg_pnl_not_flagged() {
        let trades: Vec<_> = (0..10).map(|_| fomc(50.0)).collect();
        let r = analyze(&trades);
        assert!(!r.problem_buckets.contains(&"FOMC Day".to_string()));
    }

    #[test]
    fn buckets_in_consistent_order() {
        let trades = vec![monday(10.0)];
        let r = analyze(&trades);
        let labels: Vec<&str> = r.buckets.iter().map(|b| b.label.as_str()).collect();
        assert_eq!(labels, vec!["Monday", "Friday", "Earnings Week", "FOMC Day", "First Week", "Last Week"]);
    }

    #[test]
    fn empty_bucket_zero_count_zero_pnl() {
        let trades = vec![monday(10.0)];
        let r = analyze(&trades);
        let fri = r.buckets.iter().find(|b| b.label == "Friday").unwrap();
        assert_eq!(fri.trade_count, 0);
        assert_eq!(fri.total_pnl, 0.0);
        assert_eq!(fri.win_rate, 0.0);
    }

    #[test]
    fn trade_can_match_multiple_buckets() {
        // Trade tagged both Monday AND first-week-of-month.
        let t = CalendarTaggedTrade {
            pnl: 100.0,
            is_monday: true,
            is_first_week_of_month: true,
            ..t_default()
        };
        let r = analyze(&[t]);
        let mon = r.buckets.iter().find(|b| b.label == "Monday").unwrap();
        let first = r.buckets.iter().find(|b| b.label == "First Week").unwrap();
        assert_eq!(mon.trade_count, 1);
        assert_eq!(first.trade_count, 1);
    }
}
