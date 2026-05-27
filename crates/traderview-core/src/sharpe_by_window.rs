//! Sharpe-by-time-window analyzer.
//!
//! Slices realized R-multiples (or P&L) into time buckets — hour-of-day,
//! day-of-week, week-of-month, or custom calendar windows — and computes
//! a mini-Sharpe per bucket. Identifies windows where the trader has
//! a real edge vs windows that are noise.
//!
//! Pure compute. Annualization factor is parametric so the same engine
//! works for intraday R-buckets (factor = 252) and per-trade buckets
//! (factor = trades_per_year). Sharpe = mean / stdev × sqrt(factor).

use chrono::{DateTime, Datelike, Timelike, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TradeReturn {
    pub when: DateTime<Utc>,
    /// Per-trade return — caller normalizes (R-multiple, %, dollars).
    /// All buckets compute on whatever unit caller passes; aggregation
    /// is unit-agnostic.
    pub r: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Bucket {
    HourOfDay,
    DayOfWeek,
    /// Calendar month (Jan, Feb, ...).
    MonthOfYear,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowStats {
    /// Bucket label: "09", "Mon", "Jan", ...
    pub label: String,
    pub trade_count: usize,
    pub mean_r: f64,
    pub stdev_r: f64,
    /// mean / stdev × sqrt(annualization). NaN-safe (returns 0 if no spread).
    pub sharpe: f64,
    /// Total of all R values in the bucket — useful for "where am I
    /// actually making the money" view independent of Sharpe.
    pub sum_r: f64,
}

pub fn by(returns: &[TradeReturn], bucket: Bucket, annualization: f64) -> Vec<WindowStats> {
    let mut groups: std::collections::BTreeMap<u32, Vec<f64>> = Default::default();
    for r in returns {
        let key = bucket_key(r.when, bucket);
        groups.entry(key).or_default().push(r.r);
    }
    let mut out: Vec<WindowStats> = groups.into_iter()
        .map(|(k, vals)| stats_for(label_for(k, bucket), vals, annualization))
        .collect();
    // Sort by chronological key already (BTreeMap), but for week labels we
    // want Mon..Sun rather than 0..6 which is already the case; for hour
    // the key IS the sort order; for month the key IS the order. So
    // BTreeMap insertion order is fine — no extra sort.
    out.sort_by(|a, b| a.label.cmp(&b.label));   // alpha within bucket type
    out
}

fn bucket_key(t: DateTime<Utc>, bucket: Bucket) -> u32 {
    match bucket {
        Bucket::HourOfDay   => t.hour(),
        Bucket::DayOfWeek   => t.weekday().num_days_from_monday(),
        Bucket::MonthOfYear => t.month(),
    }
}

fn label_for(key: u32, bucket: Bucket) -> String {
    match bucket {
        Bucket::HourOfDay   => format!("{:02}", key),
        Bucket::DayOfWeek   => match key {
            0 => "Mon", 1 => "Tue", 2 => "Wed", 3 => "Thu",
            4 => "Fri", 5 => "Sat", 6 => "Sun", _ => "?",
        }.into(),
        Bucket::MonthOfYear => match key {
            1 => "Jan", 2 => "Feb", 3 => "Mar", 4 => "Apr",
            5 => "May", 6 => "Jun", 7 => "Jul", 8 => "Aug",
            9 => "Sep", 10 => "Oct", 11 => "Nov", 12 => "Dec",
            _ => "?",
        }.into(),
    }
}

fn stats_for(label: String, vals: Vec<f64>, annualization: f64) -> WindowStats {
    let n = vals.len();
    if n == 0 {
        return WindowStats {
            label, trade_count: 0, mean_r: 0.0, stdev_r: 0.0, sharpe: 0.0, sum_r: 0.0,
        };
    }
    let sum: f64 = vals.iter().sum();
    let mean = sum / n as f64;
    let var = vals.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n as f64;
    let stdev = var.sqrt();
    let sharpe = if stdev == 0.0 { 0.0 } else { mean / stdev * annualization.sqrt() };
    WindowStats {
        label,
        trade_count: n,
        mean_r: mean,
        stdev_r: stdev,
        sharpe,
        sum_r: sum,
    }
}

/// Convenience: also expose by-day-of-week as a typed weekday lookup
/// so the dashboard can iterate Mon..Sun in display order without
/// alpha-sorting "Fri" before "Mon".
pub fn day_of_week_ordered(returns: &[TradeReturn], annualization: f64) -> [WindowStats; 7] {
    let mut by_day: [Vec<f64>; 7] = Default::default();
    for r in returns {
        let idx = r.when.weekday().num_days_from_monday() as usize;
        by_day[idx].push(r.r);
    }
    let labels = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
    std::array::from_fn(|i| stats_for(labels[i].into(), by_day[i].clone(), annualization))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn at(y: i32, m: u32, d: u32, h: u32, r: f64) -> TradeReturn {
        TradeReturn {
            when: Utc.with_ymd_and_hms(y, m, d, h, 0, 0).unwrap(),
            r,
        }
    }

    #[test]
    fn empty_input_returns_empty() {
        let out = by(&[], Bucket::HourOfDay, 252.0);
        assert!(out.is_empty());
    }

    #[test]
    fn hour_of_day_buckets_collapse_same_hour_across_days() {
        let trades = vec![
            at(2026, 5, 1, 9, 1.0),   // 09:00 → +1R
            at(2026, 5, 2, 9, 1.0),   // 09:00 → +1R
            at(2026, 5, 1, 15, -1.0), // 15:00 → -1R
        ];
        let out = by(&trades, Bucket::HourOfDay, 252.0);
        assert_eq!(out.len(), 2);
        let nine = out.iter().find(|s| s.label == "09").unwrap();
        let fifteen = out.iter().find(|s| s.label == "15").unwrap();
        assert_eq!(nine.trade_count, 2);
        assert_eq!(nine.mean_r, 1.0);
        assert_eq!(fifteen.trade_count, 1);
        assert_eq!(fifteen.mean_r, -1.0);
    }

    #[test]
    fn sharpe_zero_when_stdev_is_zero() {
        // All same value — stdev = 0 → sharpe protected from div0.
        let trades = vec![
            at(2026, 5, 1, 10, 0.5),
            at(2026, 5, 2, 10, 0.5),
            at(2026, 5, 3, 10, 0.5),
        ];
        let out = by(&trades, Bucket::HourOfDay, 252.0);
        let s = &out[0];
        assert_eq!(s.stdev_r, 0.0);
        assert_eq!(s.sharpe, 0.0, "div-by-zero guarded");
        assert_eq!(s.mean_r, 0.5);
    }

    #[test]
    fn sharpe_positive_for_positive_edge() {
        // Mean=1, stdev>0, annualization=252 → sqrt(252) ≈ 15.87.
        let trades = vec![
            at(2026, 5, 1, 10,  2.0),
            at(2026, 5, 2, 10,  1.0),
            at(2026, 5, 3, 10,  0.0),
        ];
        let out = by(&trades, Bucket::HourOfDay, 252.0);
        let s = &out[0];
        assert!(s.sharpe > 0.0);
        // Mean = 1, var = ((1)^2 + 0 + (1)^2)/3 = 2/3, stdev ≈ 0.8165.
        // Sharpe = 1 / 0.8165 × sqrt(252) ≈ 19.44.
        assert!((s.sharpe - (1.0 / (2.0f64/3.0).sqrt() * 252.0_f64.sqrt())).abs() < 1e-6);
    }

    #[test]
    fn day_of_week_ordered_returns_array_in_mon_to_sun_order() {
        let trades = vec![
            at(2026, 5, 4, 10,  1.0),  // 2026-05-04 = Mon
            at(2026, 5, 5, 10, -1.0),  // Tue
            at(2026, 5, 6, 10,  2.0),  // Wed
        ];
        let arr = day_of_week_ordered(&trades, 252.0);
        assert_eq!(arr[0].label, "Mon");
        assert_eq!(arr[0].mean_r, 1.0);
        assert_eq!(arr[1].label, "Tue");
        assert_eq!(arr[1].mean_r, -1.0);
        assert_eq!(arr[2].label, "Wed");
        assert_eq!(arr[2].mean_r, 2.0);
        // Days without trades come through with zero counts.
        assert_eq!(arr[4].label, "Fri");
        assert_eq!(arr[4].trade_count, 0);
    }

    #[test]
    fn month_of_year_buckets_use_three_letter_labels() {
        let trades = vec![
            at(2026, 1, 5, 10,  1.0),
            at(2026, 6, 5, 10, -1.0),
            at(2026, 12, 5, 10,  3.0),
        ];
        let out = by(&trades, Bucket::MonthOfYear, 12.0);
        assert_eq!(out.len(), 3);
        let dec = out.iter().find(|s| s.label == "Dec").unwrap();
        assert_eq!(dec.mean_r, 3.0);
    }

    #[test]
    fn sum_r_independent_of_sharpe() {
        // Useful sanity: sum_r is just total P&L per bucket, no annualization.
        let trades = vec![
            at(2026, 5, 1, 10,  2.0),
            at(2026, 5, 2, 10,  3.0),
        ];
        let out = by(&trades, Bucket::HourOfDay, 252.0);
        assert_eq!(out[0].sum_r, 5.0);
    }
}
