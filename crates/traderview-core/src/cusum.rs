//! CUSUM (Cumulative Sum) change-point detector — Page-Hinkley.
//!
//! Detects when a series' mean shifts by more than a threshold amount.
//! Originally a statistical quality-control tool, applied in trading to
//! flag regime changes: a CUSUM-positive event on log-returns is the
//! moment a market quietly transitioned from drift up to drift down.
//!
//! Algorithm: maintain two running sums `g_pos` and `g_neg` that
//! accumulate the signed gap from a reference mean. When either exceeds
//! a threshold (multiplied by stdev for scale invariance), emit a
//! change event and reset.
//!
//! Pure compute. Caller supplies a (reference_mean, threshold_stdevs)
//! pair or accepts the defaults.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeDirection {
    Up,
    Down,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ChangeEvent {
    pub bar_index: usize,
    pub direction: ChangeDirection,
    pub cusum_value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CusumConfig {
    /// Pre-computed mean of the in-control series (e.g. trailing
    /// 100-bar mean of log returns).
    pub reference_mean: f64,
    /// Pre-computed stdev for normalization. Threshold = `threshold_stdevs × this`.
    pub reference_stdev: f64,
    /// How many stdevs the cumulative sum must travel before triggering.
    pub threshold_stdevs: f64,
    /// Slack (drift parameter): subtracted from each observation's
    /// contribution to make the test less sensitive to small noise.
    /// Typically 0.5 × stdev. 0.0 disables.
    pub slack: f64,
}

impl Default for CusumConfig {
    fn default() -> Self {
        Self {
            reference_mean: 0.0,
            reference_stdev: 1.0,
            threshold_stdevs: 5.0,
            slack: 0.5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CusumReport {
    pub events: Vec<ChangeEvent>,
    pub final_g_pos: f64,
    pub final_g_neg: f64,
    pub n_events: usize,
}

pub fn detect(series: &[f64], cfg: &CusumConfig) -> CusumReport {
    if series.is_empty() || cfg.reference_stdev <= 0.0 {
        return CusumReport::default();
    }
    let threshold = cfg.threshold_stdevs * cfg.reference_stdev;
    let mut g_pos = 0.0_f64;
    let mut g_neg = 0.0_f64;
    let mut events = Vec::new();
    for (i, &x) in series.iter().enumerate() {
        let dev = x - cfg.reference_mean;
        g_pos = (g_pos + dev - cfg.slack).max(0.0);
        g_neg = (g_neg - dev - cfg.slack).max(0.0);
        if g_pos > threshold {
            events.push(ChangeEvent {
                bar_index: i,
                direction: ChangeDirection::Up,
                cusum_value: g_pos,
            });
            g_pos = 0.0;
        }
        if g_neg > threshold {
            events.push(ChangeEvent {
                bar_index: i,
                direction: ChangeDirection::Down,
                cusum_value: g_neg,
            });
            g_neg = 0.0;
        }
    }
    let n_events = events.len();
    CusumReport {
        events,
        final_g_pos: g_pos,
        final_g_neg: g_neg,
        n_events,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_returns_default() {
        let r = detect(&[], &CusumConfig::default());
        assert!(r.events.is_empty());
    }

    #[test]
    fn series_around_mean_emits_no_events() {
        // Series of values ±0.5 around mean 0 — should never breach threshold (5σ).
        let series: Vec<f64> = (0..100)
            .map(|i| if i % 2 == 0 { 0.5 } else { -0.5 })
            .collect();
        let r = detect(&series, &CusumConfig::default());
        assert!(r.events.is_empty(), "small variance shouldn't trigger");
    }

    #[test]
    fn sustained_upward_drift_triggers_up_event() {
        // Mean = 0, but series persistently above mean → cumulative sum builds.
        let series: Vec<f64> = (0..30).map(|_| 1.0).collect(); // every sample is +1
        let cfg = CusumConfig {
            reference_mean: 0.0,
            reference_stdev: 1.0,
            threshold_stdevs: 3.0,
            slack: 0.0,
        };
        let r = detect(&series, &cfg);
        assert!(!r.events.is_empty(), "sustained drift should fire");
        // First event should be Up.
        assert!(matches!(r.events[0].direction, ChangeDirection::Up));
        // First fire after about 4 bars (cumulative sum = 4 > threshold 3).
        assert!(r.events[0].bar_index <= 5);
    }

    #[test]
    fn sustained_downward_drift_triggers_down_event() {
        let series: Vec<f64> = (0..30).map(|_| -1.0).collect();
        let cfg = CusumConfig {
            reference_mean: 0.0,
            reference_stdev: 1.0,
            threshold_stdevs: 3.0,
            slack: 0.0,
        };
        let r = detect(&series, &cfg);
        assert!(!r.events.is_empty());
        assert!(matches!(r.events[0].direction, ChangeDirection::Down));
    }

    #[test]
    fn slack_filters_small_drifts() {
        // Series of +0.3 each step. With slack=0.5, each step contributes
        // (0.3 - 0.5) = -0.2 → g_pos stays at zero. No events.
        let series: Vec<f64> = (0..30).map(|_| 0.3).collect();
        let cfg = CusumConfig {
            reference_mean: 0.0,
            reference_stdev: 1.0,
            threshold_stdevs: 3.0,
            slack: 0.5,
        };
        let r = detect(&series, &cfg);
        assert!(
            r.events.is_empty(),
            "slack 0.5 should eat the 0.3 drift, got {} events",
            r.events.len()
        );
    }

    #[test]
    fn reset_after_event_so_re_triggers_only_after_new_buildup() {
        // 30 bars of +1 should fire MULTIPLE events because the sum resets after each.
        let series: Vec<f64> = (0..30).map(|_| 1.0).collect();
        let cfg = CusumConfig {
            reference_mean: 0.0,
            reference_stdev: 1.0,
            threshold_stdevs: 3.0,
            slack: 0.0,
        };
        let r = detect(&series, &cfg);
        // 30 bars / ~4 bars per event ≈ 7-8 events.
        assert!(
            r.events.len() >= 5,
            "expected ≥ 5 events, got {}",
            r.events.len()
        );
    }

    #[test]
    fn zero_stdev_returns_default_without_panic() {
        let r = detect(
            &[1.0, 2.0, 3.0],
            &CusumConfig {
                reference_mean: 0.0,
                reference_stdev: 0.0,
                threshold_stdevs: 3.0,
                slack: 0.0,
            },
        );
        assert!(r.events.is_empty());
    }
}
