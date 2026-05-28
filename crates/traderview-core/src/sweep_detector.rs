//! Sweep detector — flags trades that hit multiple exchanges
//! near-simultaneously, the signature of an aggressive routed sweep
//! order (institutional or smart-money).
//!
//! Definition (matches the typical broker / smart-router behavior):
//!   A "sweep" is a cluster of prints within `time_window_ms` that:
//!     1. Hit ≥ `min_venues` distinct exchanges (MIC codes)
//!     2. Total size ≥ `min_aggregate_size`
//!     3. All on the same side (caller pre-classifies via tick rule)
//!
//! Returns aggregated cluster events: window start/end timestamps, total
//! size, venue count, dominant side, and member tick indices.
//!
//! Pure compute.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SweepSide {
    Buy,
    Sell,
    Mixed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutedTick {
    pub ts: DateTime<Utc>,
    pub price: f64,
    pub size: f64,
    /// Market identifier code (e.g. "ARCA", "NSDQ", "IEX", "BATS").
    pub venue: String,
    pub side: SweepSide,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SweepConfig {
    pub time_window_ms: i64,
    pub min_venues: usize,
    pub min_aggregate_size: f64,
}

impl Default for SweepConfig {
    fn default() -> Self {
        Self { time_window_ms: 500, min_venues: 3, min_aggregate_size: 1000.0 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SweepEvent {
    pub start_ts: DateTime<Utc>,
    pub end_ts: DateTime<Utc>,
    pub start_index: usize,
    pub end_index: usize,
    pub venue_count: usize,
    pub print_count: usize,
    pub total_size: f64,
    pub side: SweepSide,
    pub min_price: f64,
    pub max_price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SweepReport {
    pub events: Vec<SweepEvent>,
}

pub fn detect(ticks: &[RoutedTick], cfg: &SweepConfig) -> SweepReport {
    let mut report = SweepReport::default();
    if ticks.len() < 2 || cfg.time_window_ms <= 0 || cfg.min_venues < 2 || cfg.min_aggregate_size <= 0.0 {
        return report;
    }
    // Sliding window: walk right pointer; advance left while window > time_window_ms.
    let mut left = 0usize;
    let window = Duration::milliseconds(cfg.time_window_ms);
    let mut last_end = 0i64;
    let mut last_event_end: Option<DateTime<Utc>> = None;
    for right in 0..ticks.len() {
        while left < right && (ticks[right].ts - ticks[left].ts) > window {
            left += 1;
        }
        let cluster = &ticks[left..=right];
        let venues: HashSet<&str> = cluster.iter().map(|t| t.venue.as_str()).collect();
        if venues.len() < cfg.min_venues {
            continue;
        }
        let total_size: f64 = cluster.iter().map(|t| t.size.max(0.0)).sum();
        if total_size < cfg.min_aggregate_size {
            continue;
        }
        // Determine dominant side (all-same vs mixed).
        let mut buys = 0;
        let mut sells = 0;
        for t in cluster {
            match t.side {
                SweepSide::Buy => buys += 1,
                SweepSide::Sell => sells += 1,
                SweepSide::Mixed => {}
            }
        }
        let side = if sells == 0 && buys > 0 {
            SweepSide::Buy
        } else if buys == 0 && sells > 0 {
            SweepSide::Sell
        } else {
            SweepSide::Mixed
        };
        // Coalesce overlapping/adjacent windows so we report one event
        // per real sweep rather than one per tick within it.
        let start_ts = ticks[left].ts;
        let end_ts = ticks[right].ts;
        if let Some(prev_end) = last_event_end {
            if start_ts <= prev_end {
                // Extend the last event.
                if let Some(last) = report.events.last_mut() {
                    last.end_ts = end_ts;
                    last.end_index = right;
                    last.print_count = right - last.start_index + 1;
                    last.total_size = ticks[last.start_index..=right]
                        .iter().map(|t| t.size.max(0.0)).sum();
                    let v: HashSet<&str> = ticks[last.start_index..=right]
                        .iter().map(|t| t.venue.as_str()).collect();
                    last.venue_count = v.len();
                    last.max_price = ticks[last.start_index..=right]
                        .iter().filter(|t| t.price.is_finite())
                        .map(|t| t.price).fold(f64::NEG_INFINITY, f64::max);
                    last.min_price = ticks[last.start_index..=right]
                        .iter().filter(|t| t.price.is_finite())
                        .map(|t| t.price).fold(f64::INFINITY, f64::min);
                    last_event_end = Some(end_ts);
                    continue;
                }
            }
        }
        let min_price = cluster.iter().filter(|t| t.price.is_finite())
            .map(|t| t.price).fold(f64::INFINITY, f64::min);
        let max_price = cluster.iter().filter(|t| t.price.is_finite())
            .map(|t| t.price).fold(f64::NEG_INFINITY, f64::max);
        report.events.push(SweepEvent {
            start_ts, end_ts,
            start_index: left, end_index: right,
            venue_count: venues.len(),
            print_count: cluster.len(),
            total_size,
            side,
            min_price, max_price,
        });
        last_event_end = Some(end_ts);
        last_end = right as i64;
    }
    let _ = last_end;
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn t(ms: i64, price: f64, size: f64, venue: &str, side: SweepSide) -> RoutedTick {
        RoutedTick {
            ts: Utc.timestamp_millis_opt(ms).unwrap(),
            price, size,
            venue: venue.into(),
            side,
        }
    }

    #[test]
    fn empty_or_single_returns_empty() {
        assert!(detect(&[], &SweepConfig::default()).events.is_empty());
        let r = detect(&[t(0, 100.0, 1000.0, "ARCA", SweepSide::Buy)], &SweepConfig::default());
        assert!(r.events.is_empty());
    }

    #[test]
    fn invalid_config_returns_empty() {
        let ticks = vec![t(0, 100.0, 500.0, "ARCA", SweepSide::Buy); 5];
        for cfg in [
            SweepConfig { time_window_ms: 0, ..Default::default() },
            SweepConfig { time_window_ms: -1, ..Default::default() },
            SweepConfig { min_venues: 1, ..Default::default() },
            SweepConfig { min_aggregate_size: 0.0, ..Default::default() },
        ] {
            assert!(detect(&ticks, &cfg).events.is_empty());
        }
    }

    #[test]
    fn single_venue_doesnt_qualify() {
        // 5 prints in 100ms all on ARCA — not a multi-venue sweep.
        let ticks: Vec<RoutedTick> = (0..5).map(|i|
            t(i * 20, 100.0, 500.0, "ARCA", SweepSide::Buy)
        ).collect();
        let r = detect(&ticks, &SweepConfig::default());
        assert!(r.events.is_empty());
    }

    #[test]
    fn multi_venue_burst_flagged() {
        // 3 venues, 1.5k aggregate, within 200ms — should fire.
        let ticks = vec![
            t(0,   100.0, 600.0, "ARCA", SweepSide::Buy),
            t(50,  100.1, 500.0, "NSDQ", SweepSide::Buy),
            t(120, 100.2, 400.0, "BATS", SweepSide::Buy),
        ];
        let r = detect(&ticks, &SweepConfig::default());
        assert_eq!(r.events.len(), 1);
        assert_eq!(r.events[0].venue_count, 3);
        assert_eq!(r.events[0].side, SweepSide::Buy);
        assert!((r.events[0].total_size - 1500.0).abs() < 1e-9);
    }

    #[test]
    fn mixed_side_classified_as_mixed() {
        let ticks = vec![
            t(0,   100.0, 600.0, "ARCA", SweepSide::Buy),
            t(50,  100.0, 500.0, "NSDQ", SweepSide::Sell),
            t(120, 100.0, 400.0, "BATS", SweepSide::Buy),
        ];
        let r = detect(&ticks, &SweepConfig::default());
        assert_eq!(r.events[0].side, SweepSide::Mixed);
    }

    #[test]
    fn cluster_outside_time_window_doesnt_fire() {
        // 3 venues but spread over 2 seconds — beyond 500ms window.
        let ticks = vec![
            t(0,    100.0, 600.0, "ARCA", SweepSide::Buy),
            t(700,  100.0, 500.0, "NSDQ", SweepSide::Buy),
            t(1500, 100.0, 400.0, "BATS", SweepSide::Buy),
        ];
        let r = detect(&ticks, &SweepConfig::default());
        assert!(r.events.is_empty());
    }

    #[test]
    fn overlapping_windows_coalesce_to_one_event() {
        // 4 ticks all within 200ms; should produce ONE event, not multiple
        // overlapping sweep events as the window slides.
        let ticks = vec![
            t(0,   100.0, 500.0, "ARCA", SweepSide::Buy),
            t(50,  100.1, 400.0, "NSDQ", SweepSide::Buy),
            t(100, 100.2, 300.0, "BATS", SweepSide::Buy),
            t(150, 100.3, 600.0, "IEX",  SweepSide::Buy),
        ];
        let r = detect(&ticks, &SweepConfig::default());
        assert_eq!(r.events.len(), 1, "expected coalesced single event");
        assert_eq!(r.events[0].print_count, 4);
    }
}
