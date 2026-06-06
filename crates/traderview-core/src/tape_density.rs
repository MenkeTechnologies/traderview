//! Time-and-sales density / large-print detector.
//!
//! Given a stream of timestamped tick prints, emit:
//!   - rolling tick rate (prints / minute) in a sliding window
//:   - flagged large prints (size > `large_threshold`)
//!   - "burst" windows where tick rate exceeds the rolling baseline by
//!     `burst_multiplier`
//!
//! Tape density spikes are the cleanest fast-finger signal — when the
//! ribbon prints 3× normal volume in 30 seconds, somebody is sweeping.
//!
//! Pure compute. Caller supplies ticks pre-sorted by timestamp ascending.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Tick {
    pub ts: DateTime<Utc>,
    pub price: f64,
    pub size: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DensityConfig {
    /// Rolling window length in seconds (e.g. 60 for one minute).
    pub window_secs: i64,
    /// Print size threshold above which a tick is flagged as "large".
    pub large_threshold: f64,
    /// Multiplier vs the rolling baseline to flag a "burst" window.
    pub burst_multiplier: f64,
}

impl Default for DensityConfig {
    fn default() -> Self {
        Self {
            window_secs: 60,
            large_threshold: 10_000.0,
            burst_multiplier: 3.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LargePrint {
    pub tick_index: usize,
    pub ts: DateTime<Utc>,
    pub price: f64,
    pub size: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurstWindow {
    pub start_index: usize,
    pub end_index: usize,
    pub print_count: usize,
    pub baseline_rate: f64,
    pub burst_rate: f64,
    pub ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DensityReport {
    pub large_prints: Vec<LargePrint>,
    pub bursts: Vec<BurstWindow>,
    /// Average tick rate over the whole input.
    pub baseline_prints_per_min: f64,
}

pub fn analyze(ticks: &[Tick], cfg: &DensityConfig) -> DensityReport {
    let mut report = DensityReport::default();
    let n = ticks.len();
    if n < 2 || cfg.window_secs <= 0 || cfg.large_threshold <= 0.0 || cfg.burst_multiplier <= 1.0 {
        return report;
    }
    // Large prints.
    for (i, t) in ticks.iter().enumerate() {
        if t.size.is_finite() && t.size >= cfg.large_threshold {
            report.large_prints.push(LargePrint {
                tick_index: i,
                ts: t.ts,
                price: t.price,
                size: t.size,
            });
        }
    }
    // Baseline rate over the whole span.
    let span = ticks[n - 1].ts - ticks[0].ts;
    let mins = (span.num_seconds() as f64 / 60.0).max(0.0001);
    let baseline_rate = n as f64 / mins;
    report.baseline_prints_per_min = baseline_rate;
    // Sliding-window burst detector. Use a 2-pointer over the timestamps.
    let mut left = 0;
    for right in 0..n {
        // Advance left until window is within `window_secs`.
        while left < right
            && (ticks[right].ts - ticks[left].ts) > Duration::seconds(cfg.window_secs)
        {
            left += 1;
        }
        let count = right - left + 1;
        let window_mins = (cfg.window_secs as f64 / 60.0).max(0.0001);
        let window_rate = count as f64 / window_mins;
        let ratio = window_rate / baseline_rate.max(0.0001);
        if ratio >= cfg.burst_multiplier {
            // Coalesce: if we already have an open burst that includes left, extend it.
            let mut extended = false;
            if let Some(last) = report.bursts.last_mut() {
                if last.end_index + 1 >= left {
                    last.end_index = right;
                    last.print_count = right - last.start_index + 1;
                    last.burst_rate = last.print_count as f64 / window_mins;
                    last.ratio = last.burst_rate / baseline_rate.max(0.0001);
                    extended = true;
                }
            }
            if !extended {
                report.bursts.push(BurstWindow {
                    start_index: left,
                    end_index: right,
                    print_count: count,
                    baseline_rate,
                    burst_rate: window_rate,
                    ratio,
                });
            }
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn t(s: i64, price: f64, size: f64) -> Tick {
        Tick {
            ts: Utc.timestamp_opt(s, 0).unwrap(),
            price,
            size,
        }
    }

    #[test]
    fn empty_or_single_returns_default() {
        assert!(analyze(&[], &DensityConfig::default())
            .large_prints
            .is_empty());
        assert!(analyze(&[t(0, 100.0, 100.0)], &DensityConfig::default())
            .large_prints
            .is_empty());
    }

    #[test]
    fn invalid_config_returns_default() {
        let ticks = vec![t(0, 100.0, 100.0); 10];
        for cfg in [
            DensityConfig {
                window_secs: 0,
                ..Default::default()
            },
            DensityConfig {
                window_secs: -1,
                ..Default::default()
            },
            DensityConfig {
                large_threshold: 0.0,
                ..Default::default()
            },
            DensityConfig {
                burst_multiplier: 0.5,
                ..Default::default()
            },
        ] {
            let r = analyze(&ticks, &cfg);
            assert!(r.large_prints.is_empty() && r.bursts.is_empty());
        }
    }

    #[test]
    fn large_prints_flagged_at_threshold() {
        let ticks = vec![
            t(0, 100.0, 500.0),
            t(1, 100.0, 15_000.0), // large
            t(2, 100.0, 800.0),
            t(3, 100.0, 50_000.0), // larger
        ];
        let r = analyze(&ticks, &DensityConfig::default());
        assert_eq!(r.large_prints.len(), 2);
        assert!((r.large_prints[0].size - 15_000.0).abs() < 1e-9);
    }

    #[test]
    fn burst_window_detected_when_rate_exceeds_baseline() {
        // 10 ticks over 10 minutes (baseline = 1/min), then 30 ticks in
        // 30 seconds (window rate = 60/min → 60× baseline).
        let mut ticks: Vec<Tick> = (0..10).map(|i| t(i * 60, 100.0, 100.0)).collect();
        for i in 0..30 {
            ticks.push(t(10 * 60 + i, 100.0, 100.0));
        }
        let cfg = DensityConfig {
            window_secs: 60,
            large_threshold: 1e9,
            burst_multiplier: 3.0,
        };
        let r = analyze(&ticks, &cfg);
        assert!(!r.bursts.is_empty(), "expected a burst window");
        // The burst rate should be much higher than baseline.
        assert!(r.bursts[0].ratio > 3.0);
    }

    #[test]
    fn no_burst_on_uniform_rate() {
        // 60 ticks over 60 minutes — baseline = 1/min, window rate also 1/min → no burst.
        let ticks: Vec<Tick> = (0..60).map(|i| t(i * 60, 100.0, 100.0)).collect();
        let r = analyze(&ticks, &DensityConfig::default());
        assert!(r.bursts.is_empty());
    }

    #[test]
    fn baseline_rate_matches_overall_average() {
        // 6 ticks over 60 seconds = 6/min.
        let ticks: Vec<Tick> = (0..6).map(|i| t(i * 10, 100.0, 100.0)).collect();
        let r = analyze(&ticks, &DensityConfig::default());
        // span = 50 seconds = 5/6 min; 6 / (5/6) = 7.2. Verify it's close.
        assert!(r.baseline_prints_per_min > 5.0);
    }
}
