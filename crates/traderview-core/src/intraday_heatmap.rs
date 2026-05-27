//! Intraday P&L heatmap by 15-minute buckets.
//!
//! Distinct from sharpe_by_window's hour-of-day analysis — finer-grain
//! 15-minute resolution lets the trader spot SPECIFIC times of day
//! that print money (the 9:50am momo window) vs leak (10:30am chop).
//!
//! Per bucket: trade count, total PnL, avg PnL, win rate.
//!
//! Pure compute.

use chrono::{DateTime, Timelike, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntradayTrade {
    pub when: DateTime<Utc>,
    pub pnl: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BucketStat {
    pub label: String,
    pub hour: u32,
    pub minute: u32,
    pub trade_count: usize,
    pub total_pnl: f64,
    pub avg_pnl: f64,
    pub win_count: usize,
    pub win_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IntradayHeatmapReport {
    /// All 96 quarter-hour buckets across a 24-hour day.
    pub buckets: Vec<BucketStat>,
    /// Most-profitable bucket (by total P&L).
    pub best_bucket_label: Option<String>,
    /// Most-losing bucket.
    pub worst_bucket_label: Option<String>,
}

pub fn build(trades: &[IntradayTrade]) -> IntradayHeatmapReport {
    let mut buckets: [(usize, f64, usize); 96] = [(0, 0.0, 0); 96]; // (count, total_pnl, wins)
    for t in trades {
        let h = t.when.hour();
        let q = t.when.minute() / 15;
        let idx = (h * 4 + q) as usize;
        if idx < 96 {
            buckets[idx].0 += 1;
            buckets[idx].1 += t.pnl;
            if t.pnl > 0.0 {
                buckets[idx].2 += 1;
            }
        }
    }
    let mut report = IntradayHeatmapReport::default();
    let mut best: Option<(String, f64)> = None;
    let mut worst: Option<(String, f64)> = None;
    for (idx, (count, pnl, wins)) in buckets.iter().enumerate() {
        let h = idx as u32 / 4;
        let q = idx as u32 % 4;
        let label = format!("{:02}:{:02}", h, q * 15);
        let avg = if *count > 0 { pnl / *count as f64 } else { 0.0 };
        let wr = if *count > 0 {
            *wins as f64 / *count as f64
        } else {
            0.0
        };
        if *count > 0 {
            if best.as_ref().is_none_or(|(_, p)| *pnl > *p) {
                best = Some((label.clone(), *pnl));
            }
            if worst.as_ref().is_none_or(|(_, p)| *pnl < *p) {
                worst = Some((label.clone(), *pnl));
            }
        }
        report.buckets.push(BucketStat {
            label,
            hour: h,
            minute: q * 15,
            trade_count: *count,
            total_pnl: *pnl,
            avg_pnl: avg,
            win_count: *wins,
            win_rate: wr,
        });
    }
    report.best_bucket_label = best.map(|(l, _)| l);
    report.worst_bucket_label = worst.map(|(l, _)| l);
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn t(h: u32, m: u32, pnl: f64) -> IntradayTrade {
        IntradayTrade {
            when: Utc.with_ymd_and_hms(2026, 5, 27, h, m, 0).unwrap(),
            pnl,
        }
    }

    #[test]
    fn empty_returns_all_zero_buckets() {
        let r = build(&[]);
        assert_eq!(r.buckets.len(), 96);
        for b in &r.buckets {
            assert_eq!(b.trade_count, 0);
        }
        assert!(r.best_bucket_label.is_none());
    }

    #[test]
    fn trades_bucket_into_quarter_hour_slots() {
        let trades = vec![
            t(9, 30, 100.0),
            t(9, 35, 50.0),  // same bucket as 9:30
            t(9, 50, -25.0), // 9:45 bucket
        ];
        let r = build(&trades);
        let nine_thirty = r.buckets.iter().find(|b| b.label == "09:30").unwrap();
        let nine_fortyfive = r.buckets.iter().find(|b| b.label == "09:45").unwrap();
        assert_eq!(nine_thirty.trade_count, 2);
        assert_eq!(nine_thirty.total_pnl, 150.0);
        assert_eq!(nine_fortyfive.trade_count, 1);
    }

    #[test]
    fn win_rate_computed_per_bucket() {
        let trades = vec![t(10, 0, 100.0), t(10, 5, -50.0), t(10, 10, 100.0)];
        let r = build(&trades);
        let bucket = r.buckets.iter().find(|b| b.label == "10:00").unwrap();
        assert_eq!(bucket.trade_count, 3);
        assert!((bucket.win_rate - 2.0 / 3.0).abs() < 1e-9);
    }

    #[test]
    fn best_bucket_identified_by_total_pnl() {
        let trades = vec![
            t(9, 30, 500.0), // best bucket
            t(10, 0, 100.0),
            t(15, 0, 50.0),
        ];
        let r = build(&trades);
        assert_eq!(r.best_bucket_label.as_deref(), Some("09:30"));
    }

    #[test]
    fn worst_bucket_identified_by_lowest_pnl() {
        let trades = vec![
            t(9, 30, 100.0),
            t(10, 0, -500.0), // worst
            t(15, 0, 50.0),
        ];
        let r = build(&trades);
        assert_eq!(r.worst_bucket_label.as_deref(), Some("10:00"));
    }

    #[test]
    fn ninety_six_buckets_total() {
        let r = build(&[]);
        assert_eq!(r.buckets.len(), 96);
        assert_eq!(r.buckets[0].label, "00:00");
        assert_eq!(r.buckets[95].label, "23:45");
    }

    #[test]
    fn minute_rounding_into_15min_buckets() {
        // 9:14 → 9:00 bucket. 9:15 → 9:15 bucket.
        let trades = vec![t(9, 14, 100.0), t(9, 15, 200.0)];
        let r = build(&trades);
        let nine_zero = r.buckets.iter().find(|b| b.label == "09:00").unwrap();
        let nine_fifteen = r.buckets.iter().find(|b| b.label == "09:15").unwrap();
        assert_eq!(nine_zero.trade_count, 1);
        assert_eq!(nine_fifteen.trade_count, 1);
    }
}
