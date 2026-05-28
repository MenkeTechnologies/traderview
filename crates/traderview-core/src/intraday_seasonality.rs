//! Intraday Seasonality — average bar-by-bar return by time-of-day.
//!
//! For each minute-of-day bucket aggregates:
//!   - mean log-return
//!   - std dev of returns
//!   - hit rate (% positive returns)
//!   - sample count
//!
//! Useful for detecting open-drift, midday-reversion, last-hour drift,
//! and similar U-shape volume / return patterns.
//!
//! Caller supplies bars tagged by minute-of-day (0..1440). Returns per
//! bar are computed within-day only (no overnight returns to avoid
//! mixing with gap effects).
//!
//! Pure compute. Companion to `monthly_seasonality`,
//! `holiday_seasonality`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct IntradayBar {
    pub date: u32,             // YYYYMMDD encoding
    pub minute_of_day: u16,    // 0..1440
    pub close: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct MinuteStats {
    pub minute_of_day: u16,
    pub mean_return: f64,
    pub std_return: f64,
    pub hit_rate: f64,
    pub sample_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IntradaySeasonalityReport {
    pub by_minute: Vec<MinuteStats>,
    pub n_observations: usize,
}

pub fn compute(bars: &[IntradayBar]) -> Option<IntradaySeasonalityReport> {
    if bars.len() < 2 { return None; }
    if bars.iter().any(|b| !b.close.is_finite() || b.close <= 0.0
        || b.minute_of_day >= 1440) {
        return None;
    }
    // Group by date, then compute per-bar within-day returns.
    let mut grouped: std::collections::BTreeMap<u32, Vec<(u16, f64)>> =
        std::collections::BTreeMap::new();
    for b in bars {
        grouped.entry(b.date).or_default().push((b.minute_of_day, b.close));
    }
    let mut returns_by_minute: std::collections::BTreeMap<u16, Vec<f64>> =
        std::collections::BTreeMap::new();
    for (_, day) in grouped.iter_mut() {
        day.sort_by_key(|(m, _)| *m);
        for w in day.windows(2) {
            let (_, prev) = w[0];
            let (m_cur, cur) = w[1];
            if prev > 0.0 && cur > 0.0 {
                let r = (cur / prev).ln();
                returns_by_minute.entry(m_cur).or_default().push(r);
            }
        }
    }
    let mut report = IntradaySeasonalityReport {
        by_minute: Vec::new(),
        n_observations: bars.len(),
    };
    for (m, returns) in returns_by_minute {
        let n_f = returns.len() as f64;
        let mean: f64 = returns.iter().sum::<f64>() / n_f;
        let var: f64 = returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / n_f;
        let std = var.max(0.0).sqrt();
        let positive = returns.iter().filter(|r| **r > 0.0).count();
        report.by_minute.push(MinuteStats {
            minute_of_day: m,
            mean_return: mean,
            std_return: std,
            hit_rate: positive as f64 / n_f,
            sample_count: returns.len() as u32,
        });
    }
    Some(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(date: u32, minute: u16, close: f64) -> IntradayBar {
        IntradayBar { date, minute_of_day: minute, close }
    }

    #[test]
    fn empty_returns_none() {
        assert!(compute(&[]).is_none());
        assert!(compute(&[b(20200101, 0, 100.0)]).is_none());
    }

    #[test]
    fn nan_or_invalid_minute_returns_none() {
        assert!(compute(&[b(20200101, 0, f64::NAN), b(20200101, 1, 100.0)]).is_none());
        assert!(compute(&[b(20200101, 2000, 100.0), b(20200101, 1, 100.0)]).is_none());
    }

    #[test]
    fn flat_intraday_yields_zero_returns() {
        let bars: Vec<_> = (0..10).map(|m| b(20200101, m as u16, 100.0)).collect();
        let r = compute(&bars).unwrap();
        for s in &r.by_minute {
            assert!(s.mean_return.abs() < 1e-9);
        }
    }

    #[test]
    fn within_day_returns_only() {
        // Two days, no overnight returns leak in.
        let bars = vec![
            b(20200101, 0, 100.0),
            b(20200101, 1, 101.0),    // ret = ln(1.01) for minute 1
            b(20200102, 0, 200.0),    // new day, no return from prior close 101
            b(20200102, 1, 202.0),    // ret = ln(1.01) for minute 1 also
        ];
        let r = compute(&bars).unwrap();
        let m1 = r.by_minute.iter().find(|s| s.minute_of_day == 1).unwrap();
        // Both days contribute one return each at minute 1.
        assert_eq!(m1.sample_count, 2);
        assert!((m1.mean_return - (101.0_f64 / 100.0).ln()).abs() < 1e-9);
    }

    #[test]
    fn output_count_matches_distinct_minutes() {
        let bars = vec![
            b(20200101, 0, 100.0),
            b(20200101, 30, 101.0),
            b(20200101, 60, 102.0),
        ];
        let r = compute(&bars).unwrap();
        assert_eq!(r.by_minute.len(), 2);    // returns at minute 30 and 60
    }
}
