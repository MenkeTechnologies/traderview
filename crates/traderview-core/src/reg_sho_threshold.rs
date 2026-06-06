//! Reg SHO threshold-list classifier — SEC Rule 203(b)(3).
//!
//! Identifies securities that would be added to a US exchange's
//! Regulation SHO "Threshold Securities List" based on a series of
//! daily fail-to-deliver (FTD) observations.
//!
//! ### SEC rule (17 CFR § 242.203(b)(3))
//!
//! A security qualifies for the threshold list when ALL of:
//!
//! 1. There are **aggregate fails to deliver of ≥ 10,000 shares** at
//!    a registered clearing agency for each of **five consecutive
//!    settlement days**.
//! 2. The level of fails is **≥ 0.5% of the issue's total shares
//!    outstanding** on each of those same five days.
//! 3. The security is published on the threshold list by a Self-
//!    Regulatory Organization (SRO) — for our purposes we just flag
//!    when the qualifying conditions are met.
//!
//! Inclusion forces close-out: brokers/dealers must close out fail-to-
//! deliver positions within 13 settlement days (T+13). Failure to do
//! so triggers § 204 borrow restrictions on the broker. Persistent
//! FTDs are also a well-documented squeeze precursor — Reg SHO list
//! placement was a leading indicator for GME, SAVA, and others.
//!
//! ### Algorithm
//!
//! Walk the daily series in chronological order. Maintain a streak
//! counter that increments when both thresholds are met on a day and
//! resets on a miss. Emit a `ThresholdEvent` on the day the streak
//! reaches 5 consecutive qualifying days.
//!
//! Sources:
//!   * 17 CFR § 242.203(b)(3) (Reg SHO)
//!   * SEC Threshold Securities List FAQ
//!   * NYSE/NASDAQ daily threshold list publications

use serde::{Deserialize, Serialize};

/// Minimum daily FTD shares per SEC Rule 203(b)(3)(i).
pub const MIN_DAILY_FTD_SHARES: u64 = 10_000;
/// Minimum daily FTD as fraction of total outstanding per Rule 203(b)(3)(ii).
pub const MIN_FTD_PCT_OUTSTANDING: f64 = 0.005;
/// Consecutive settlement days required for threshold inclusion.
pub const CONSECUTIVE_DAYS_REQUIRED: u32 = 5;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FtdDay {
    /// Settlement day index (monotonic integer). Caller is responsible
    /// for using settlement days (not calendar days) — the analyzer
    /// assumes consecutive integers are consecutive settlement days.
    pub day: i64,
    pub ftd_shares: u64,
    pub shares_outstanding: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct ThresholdEvent {
    /// Day on which the security crossed the 5-day threshold (5th
    /// consecutive qualifying day).
    pub trigger_day: i64,
    /// First day of the qualifying streak.
    pub streak_start_day: i64,
    /// Highest single-day FTD shares observed during the streak.
    pub peak_ftd_shares: u64,
    /// Highest single-day FTD-as-pct-of-outstanding during the streak.
    pub peak_ftd_pct: f64,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct DayClassification {
    pub day: i64,
    pub meets_share_floor: bool,
    pub meets_pct_floor: bool,
    pub qualifies: bool,
    pub streak_length_after_day: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ThresholdReport {
    pub events: Vec<ThresholdEvent>,
    pub day_breakdown: Vec<DayClassification>,
}

pub fn classify(series: &[FtdDay]) -> ThresholdReport {
    let mut report = ThresholdReport::default();
    let mut streak_len: u32 = 0;
    let mut streak_start: Option<i64> = None;
    let mut peak_shares: u64 = 0;
    let mut peak_pct: f64 = 0.0;

    for d in series {
        let pct = if d.shares_outstanding == 0 {
            0.0
        } else {
            d.ftd_shares as f64 / d.shares_outstanding as f64
        };
        let meets_shares = d.ftd_shares >= MIN_DAILY_FTD_SHARES;
        let meets_pct = pct >= MIN_FTD_PCT_OUTSTANDING;
        let qualifies = meets_shares && meets_pct;

        if qualifies {
            if streak_len == 0 {
                streak_start = Some(d.day);
                peak_shares = 0;
                peak_pct = 0.0;
            }
            streak_len += 1;
            peak_shares = peak_shares.max(d.ftd_shares);
            if pct > peak_pct {
                peak_pct = pct;
            }
            if streak_len == CONSECUTIVE_DAYS_REQUIRED {
                // Emit a threshold event on the day the streak hit 5.
                report.events.push(ThresholdEvent {
                    trigger_day: d.day,
                    streak_start_day: streak_start.expect("streak start set"),
                    peak_ftd_shares: peak_shares,
                    peak_ftd_pct: peak_pct,
                });
            }
        } else {
            streak_len = 0;
            streak_start = None;
            peak_shares = 0;
            peak_pct = 0.0;
        }

        report.day_breakdown.push(DayClassification {
            day: d.day,
            meets_share_floor: meets_shares,
            meets_pct_floor: meets_pct,
            qualifies,
            streak_length_after_day: streak_len,
        });
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(day: i64, ftd: u64, oas: u64) -> FtdDay {
        FtdDay {
            day,
            ftd_shares: ftd,
            shares_outstanding: oas,
        }
    }

    #[test]
    fn empty_series_no_events() {
        let r = classify(&[]);
        assert!(r.events.is_empty());
        assert!(r.day_breakdown.is_empty());
    }

    #[test]
    fn under_share_floor_never_qualifies() {
        // 9,000 shares each day — below 10,000 floor regardless of pct.
        let series = vec![
            day(0, 9_000, 1_000_000),
            day(1, 9_000, 1_000_000),
            day(2, 9_000, 1_000_000),
            day(3, 9_000, 1_000_000),
            day(4, 9_000, 1_000_000),
            day(5, 9_000, 1_000_000),
        ];
        let r = classify(&series);
        assert!(r.events.is_empty());
        for d in &r.day_breakdown {
            assert!(!d.qualifies);
        }
    }

    #[test]
    fn under_pct_floor_never_qualifies() {
        // 100k shares but 1 billion outstanding → 0.01% << 0.5%.
        let series: Vec<FtdDay> = (0..10).map(|d| day(d, 100_000, 1_000_000_000)).collect();
        let r = classify(&series);
        assert!(r.events.is_empty());
    }

    #[test]
    fn five_consecutive_qualifying_days_triggers_event_on_day_5() {
        // 100k FTDs / 10M outstanding = 1% → above both thresholds.
        let series: Vec<FtdDay> = (0..5).map(|d| day(d, 100_000, 10_000_000)).collect();
        let r = classify(&series);
        assert_eq!(r.events.len(), 1);
        let ev = r.events[0];
        assert_eq!(ev.trigger_day, 4); // 5th day = index 4
        assert_eq!(ev.streak_start_day, 0);
        assert_eq!(ev.peak_ftd_shares, 100_000);
        assert!((ev.peak_ftd_pct - 0.01).abs() < 1e-9);
    }

    #[test]
    fn streak_resets_on_a_miss_no_event() {
        // 4 qualifying, 1 missing, 4 more qualifying → streak never hits 5.
        let mut series: Vec<FtdDay> = Vec::new();
        for d in 0..4 {
            series.push(day(d, 100_000, 10_000_000));
        }
        series.push(day(4, 1_000, 10_000_000)); // miss (under shares floor)
        for d in 5..9 {
            series.push(day(d, 100_000, 10_000_000));
        }
        let r = classify(&series);
        assert!(r.events.is_empty());
    }

    #[test]
    fn streak_resets_then_qualifies_emits_event() {
        // 3 qualifying, 1 miss, 5 qualifying → one event on day 9.
        let mut series: Vec<FtdDay> = Vec::new();
        for d in 0..3 {
            series.push(day(d, 100_000, 10_000_000));
        }
        series.push(day(3, 500, 10_000_000)); // miss
        for d in 4..9 {
            series.push(day(d, 100_000, 10_000_000));
        }
        let r = classify(&series);
        assert_eq!(r.events.len(), 1);
        assert_eq!(r.events[0].trigger_day, 8);
        assert_eq!(r.events[0].streak_start_day, 4);
    }

    #[test]
    fn long_streak_emits_event_on_day_5_only_not_subsequent() {
        // 10 qualifying days. Event emitted on 5th day; days 6-10
        // continue the streak but don't emit additional events.
        let series: Vec<FtdDay> = (0..10).map(|d| day(d, 100_000, 10_000_000)).collect();
        let r = classify(&series);
        assert_eq!(r.events.len(), 1);
        assert_eq!(r.events[0].trigger_day, 4);
        // Day 10's streak counter should be 10 in the breakdown.
        assert_eq!(r.day_breakdown[9].streak_length_after_day, 10);
    }

    #[test]
    fn peak_metrics_tracked_during_streak() {
        // Mixed levels across qualifying days — peak should report
        // the highest values seen.
        let series = vec![
            day(0, 100_000, 10_000_000), // 1%
            day(1, 200_000, 10_000_000), // 2%
            day(2, 150_000, 10_000_000), // 1.5%
            day(3, 500_000, 10_000_000), // 5% ← peak
            day(4, 250_000, 10_000_000), // 2.5%
        ];
        let r = classify(&series);
        assert_eq!(r.events.len(), 1);
        assert_eq!(r.events[0].peak_ftd_shares, 500_000);
        assert!((r.events[0].peak_ftd_pct - 0.05).abs() < 1e-9);
    }

    #[test]
    fn zero_shares_outstanding_treated_as_pct_zero() {
        // Defensive: 0 shares outstanding can't qualify on pct.
        let series: Vec<FtdDay> = (0..10).map(|d| day(d, 100_000, 0)).collect();
        let r = classify(&series);
        assert!(r.events.is_empty());
    }

    #[test]
    fn boundary_at_exactly_thresholds_qualifies() {
        // 10,000 shares (exactly) and 0.5% (exactly) → meets both.
        let series: Vec<FtdDay> = (0..5)
            .map(|d| day(d, 10_000, 2_000_000)) // 0.5%
            .collect();
        let r = classify(&series);
        assert_eq!(r.events.len(), 1);
    }

    #[test]
    fn day_breakdown_classifies_each_day_independently() {
        let series = vec![
            day(0, 100_000, 10_000_000),
            day(1, 1_000, 10_000_000),
            day(2, 100_000, 100_000_000), // pct = 0.1% < 0.5%
            day(3, 100_000, 10_000_000),
        ];
        let r = classify(&series);
        assert!(r.day_breakdown[0].qualifies);
        assert!(!r.day_breakdown[1].qualifies);
        assert!(!r.day_breakdown[1].meets_share_floor);
        assert!(!r.day_breakdown[2].qualifies);
        assert!(r.day_breakdown[2].meets_share_floor);
        assert!(!r.day_breakdown[2].meets_pct_floor);
        assert!(r.day_breakdown[3].qualifies);
    }
}
