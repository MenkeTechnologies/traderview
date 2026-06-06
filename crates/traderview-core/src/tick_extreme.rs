//! NYSE TICK extreme-reading detector.
//!
//! TICK is a real-time market-breadth measure: the number of NYSE stocks
//! whose latest print was on an uptick MINUS those on a downtick. Range
//! typically ±1500, with ±1000 being notable. EXTREME readings of ±1200+
//! often mark short-term turning points (the buying / selling climax).
//!
//! This module takes a TICK series and flags:
//!   - extreme highs: TICK >= `+extreme_threshold` (climactic buying)
//!   - extreme lows:  TICK <= `-extreme_threshold` (climactic selling)
//!   - flips: prior bar extreme on one side, current bar extreme on the
//!     other (rare but powerful reversal cue)
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TickEventKind {
    ExtremeHigh,
    ExtremeLow,
    /// Prior bar was ExtremeHigh and current is ExtremeLow (or vice versa).
    Flip,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TickEvent {
    pub bar_index: usize,
    pub kind: TickEventKind,
    pub tick: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickConfig {
    pub extreme_threshold: f64,
}

impl Default for TickConfig {
    fn default() -> Self {
        Self {
            extreme_threshold: 1200.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TickReport {
    pub events: Vec<TickEvent>,
    pub max_tick: f64,
    pub min_tick: f64,
}

pub fn analyze(tick_series: &[f64], cfg: &TickConfig) -> TickReport {
    let mut report = TickReport::default();
    if tick_series.is_empty() || !cfg.extreme_threshold.is_finite() || cfg.extreme_threshold <= 0.0
    {
        return report;
    }
    report.max_tick = f64::NEG_INFINITY;
    report.min_tick = f64::INFINITY;
    let thr = cfg.extreme_threshold;
    let mut prior_kind: Option<TickEventKind> = None;
    for (i, &v) in tick_series.iter().enumerate() {
        if !v.is_finite() {
            prior_kind = None;
            continue;
        }
        if v > report.max_tick {
            report.max_tick = v;
        }
        if v < report.min_tick {
            report.min_tick = v;
        }
        let cur_kind = if v >= thr {
            Some(TickEventKind::ExtremeHigh)
        } else if v <= -thr {
            Some(TickEventKind::ExtremeLow)
        } else {
            None
        };
        if let Some(k) = cur_kind {
            // Detect flip vs prior bar's extreme.
            if let Some(prev) = prior_kind {
                if (prev == TickEventKind::ExtremeHigh && k == TickEventKind::ExtremeLow)
                    || (prev == TickEventKind::ExtremeLow && k == TickEventKind::ExtremeHigh)
                {
                    report.events.push(TickEvent {
                        bar_index: i,
                        kind: TickEventKind::Flip,
                        tick: v,
                    });
                }
            }
            report.events.push(TickEvent {
                bar_index: i,
                kind: k,
                tick: v,
            });
        }
        prior_kind = cur_kind;
    }
    // Normalize defaults if no finite values seen.
    if !report.max_tick.is_finite() {
        report.max_tick = 0.0;
    }
    if !report.min_tick.is_finite() {
        report.min_tick = 0.0;
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_default() {
        let r = analyze(&[], &TickConfig::default());
        assert!(r.events.is_empty());
        assert_eq!(r.max_tick, 0.0);
    }

    #[test]
    fn invalid_threshold_returns_default() {
        let v = vec![1500.0, -1500.0];
        for cfg in [
            TickConfig {
                extreme_threshold: 0.0,
            },
            TickConfig {
                extreme_threshold: -1.0,
            },
            TickConfig {
                extreme_threshold: f64::NAN,
            },
        ] {
            assert!(analyze(&v, &cfg).events.is_empty());
        }
    }

    #[test]
    fn extreme_high_flagged_when_tick_exceeds_threshold() {
        let v = vec![800.0, 1300.0, 500.0];
        let r = analyze(&v, &TickConfig::default());
        assert!(r
            .events
            .iter()
            .any(|e| e.kind == TickEventKind::ExtremeHigh && e.bar_index == 1));
    }

    #[test]
    fn extreme_low_flagged_at_minus_threshold() {
        let v = vec![-1300.0, 0.0];
        let r = analyze(&v, &TickConfig::default());
        assert!(r.events.iter().any(|e| e.kind == TickEventKind::ExtremeLow));
    }

    #[test]
    fn flip_detected_when_consecutive_bars_swap_extreme_side() {
        // Bar 0 extreme high, bar 1 extreme low → flip at bar 1.
        let v = vec![1300.0, -1300.0];
        let r = analyze(&v, &TickConfig::default());
        let flips: Vec<_> = r
            .events
            .iter()
            .filter(|e| e.kind == TickEventKind::Flip)
            .collect();
        assert_eq!(flips.len(), 1);
        assert_eq!(flips[0].bar_index, 1);
    }

    #[test]
    fn nan_breaks_flip_chain() {
        // Bar 0 high, bar 1 NaN (skipped), bar 2 low → no flip (NaN reset).
        let v = vec![1300.0, f64::NAN, -1300.0];
        let r = analyze(&v, &TickConfig::default());
        let flips: Vec<_> = r
            .events
            .iter()
            .filter(|e| e.kind == TickEventKind::Flip)
            .collect();
        assert!(flips.is_empty());
    }

    #[test]
    fn max_and_min_track_extremes() {
        let v = vec![-500.0, 800.0, -1500.0, 1700.0];
        let r = analyze(&v, &TickConfig::default());
        assert_eq!(r.max_tick, 1700.0);
        assert_eq!(r.min_tick, -1500.0);
    }

    #[test]
    fn no_events_when_tick_stays_within_band() {
        let v = vec![500.0, -500.0, 800.0, -700.0, 1000.0];
        let r = analyze(&v, &TickConfig::default());
        assert!(r.events.is_empty());
    }
}
