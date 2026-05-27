//! Goal tracker — target return + max-DD limit + current progress.
//!
//! A trader's discipline is reinforced by tracking measurable goals.
//! This module evaluates current performance against:
//!   - Target annualized return
//!   - Max drawdown (kill-switch limit)
//!   - Progress toward the period's profit target
//!   - On-pace estimate at current run rate
//!
//! Pure compute.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goals {
    pub period_start_equity: f64,
    pub target_pct_return: f64,
    /// Period max DD as fraction of starting equity. Breach → kill switch.
    pub max_dd_pct: f64,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProgressReport {
    pub current_equity: f64,
    pub peak_equity: f64,
    pub current_dd_pct: f64,
    pub current_pct_return: f64,
    pub target_pct_return: f64,
    /// Fraction of target hit (1.0 = met, > 1.0 = exceeded).
    pub pct_of_target: f64,
    /// Days into the period.
    pub days_elapsed: i64,
    pub days_total: i64,
    /// Annualized run-rate extrapolated from progress so far.
    pub annualized_pace: f64,
    /// True when current_dd_pct has exceeded the max_dd_pct guardrail.
    pub kill_switch_breached: bool,
    pub on_pace: OnPace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum OnPace {
    AheadOfPace,
    OnPace,
    BehindPace,
    /// Period not yet started or has ended.
    #[default]
    OutOfPeriod,
}

pub fn evaluate(goals: &Goals, equity_history: &[f64], today: NaiveDate) -> ProgressReport {
    let mut report = ProgressReport {
        target_pct_return: goals.target_pct_return,
        ..Default::default()
    };
    if equity_history.is_empty() {
        return report;
    }
    let current = *equity_history.last().unwrap();
    let peak = equity_history
        .iter()
        .cloned()
        .fold(f64::NEG_INFINITY, f64::max);
    let dd = if peak <= 0.0 {
        0.0
    } else {
        (peak - current) / peak
    };
    report.current_equity = current;
    report.peak_equity = peak;
    report.current_dd_pct = dd;
    if goals.period_start_equity > 0.0 {
        report.current_pct_return =
            (current - goals.period_start_equity) / goals.period_start_equity;
        report.pct_of_target = if goals.target_pct_return > 0.0 {
            report.current_pct_return / goals.target_pct_return
        } else {
            0.0
        };
    }
    report.kill_switch_breached = dd > goals.max_dd_pct;
    let total_days = (goals.period_end - goals.period_start).num_days().max(1);
    let elapsed = (today - goals.period_start).num_days();
    report.days_total = total_days;
    report.days_elapsed = elapsed;
    if elapsed <= 0 || elapsed > total_days {
        report.on_pace = OnPace::OutOfPeriod;
        return report;
    }
    let pct_period_elapsed = elapsed as f64 / total_days as f64;
    report.annualized_pace = if elapsed > 0 {
        report.current_pct_return * 365.0 / elapsed as f64
    } else {
        0.0
    };
    let target_fraction_today = goals.target_pct_return * pct_period_elapsed;
    let buffer = 0.10 * goals.target_pct_return.abs();
    report.on_pace = if report.current_pct_return > target_fraction_today + buffer {
        OnPace::AheadOfPace
    } else if report.current_pct_return < target_fraction_today - buffer {
        OnPace::BehindPace
    } else {
        OnPace::OnPace
    };
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    fn goals() -> Goals {
        Goals {
            period_start_equity: 100_000.0,
            target_pct_return: 0.30, // 30% target
            max_dd_pct: 0.10,        // 10% max DD
            period_start: d(2026, 1, 1),
            period_end: d(2026, 12, 31),
        }
    }

    #[test]
    fn empty_history_returns_zeros() {
        let r = evaluate(&goals(), &[], d(2026, 6, 30));
        assert_eq!(r.current_equity, 0.0);
        assert_eq!(r.current_pct_return, 0.0);
    }

    #[test]
    fn on_target_at_30_pct_mid_year() {
        // After ~half the year, equity at 115k → 15% return ≈ on-pace for 30% annual.
        let r = evaluate(
            &goals(),
            &[100_000.0, 105_000.0, 110_000.0, 115_000.0],
            d(2026, 6, 30),
        );
        assert!((r.current_pct_return - 0.15).abs() < 1e-9);
        assert_eq!(r.on_pace, OnPace::OnPace);
    }

    #[test]
    fn ahead_of_pace_when_well_above_proportional_target() {
        // At mid-year, equity at 130k → 30% already achieved.
        let r = evaluate(&goals(), &[100_000.0, 130_000.0], d(2026, 6, 30));
        assert_eq!(r.on_pace, OnPace::AheadOfPace);
        assert!(r.pct_of_target > 0.9);
    }

    #[test]
    fn behind_pace_when_well_below_proportional_target() {
        let r = evaluate(&goals(), &[100_000.0, 102_000.0], d(2026, 6, 30));
        assert_eq!(r.on_pace, OnPace::BehindPace);
    }

    #[test]
    fn kill_switch_breached_at_15pct_dd() {
        // Peak 120k, current 100k → DD = 16.7% > 10% limit.
        let r = evaluate(&goals(), &[100_000.0, 120_000.0, 100_000.0], d(2026, 6, 30));
        assert!(r.kill_switch_breached);
        assert!(r.current_dd_pct > 0.10);
    }

    #[test]
    fn kill_switch_not_breached_within_dd_limit() {
        // Peak 110k, current 105k → DD = 4.5% < 10%.
        let r = evaluate(&goals(), &[100_000.0, 110_000.0, 105_000.0], d(2026, 6, 30));
        assert!(!r.kill_switch_breached);
    }

    #[test]
    fn out_of_period_when_today_before_start() {
        let r = evaluate(&goals(), &[100_000.0, 110_000.0], d(2025, 12, 31));
        assert_eq!(r.on_pace, OnPace::OutOfPeriod);
    }

    #[test]
    fn out_of_period_when_today_after_end() {
        let r = evaluate(&goals(), &[100_000.0, 130_000.0], d(2027, 1, 1));
        assert_eq!(r.on_pace, OnPace::OutOfPeriod);
    }

    #[test]
    fn annualized_pace_extrapolates_from_partial_year() {
        // Half-year, 15% gain → annualized = 30%.
        let r = evaluate(&goals(), &[100_000.0, 115_000.0], d(2026, 7, 1));
        let half_year_days = (d(2026, 7, 1) - d(2026, 1, 1)).num_days() as f64;
        let expected = 0.15 * 365.0 / half_year_days;
        assert!((r.annualized_pace - expected).abs() < 1e-9);
    }
}
