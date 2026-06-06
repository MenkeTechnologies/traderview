//! Strategy decay detector — flags accelerating performance degradation.
//!
//! Quant trading systems rarely die quickly; they decay. A formerly-profitable
//! strategy starts producing slightly worse Sharpe quarter over quarter, then
//! falls off a cliff. By the time the trader notices the drawdown, they've
//! given back months of edge.
//!
//! This module takes a rolling-Sharpe series (caller computes via the
//! `sharpe_by_window` module or similar) and flags decay regimes:
//!
//!   - **Healthy**: rolling Sharpe is flat or trending up.
//!   - **Eroding**: Sharpe falling steadily but still positive.
//!   - **Broken**: Sharpe has crossed below the configured threshold OR is
//!     falling at an accelerating rate.
//!
//! Pure compute. Tested via hand-crafted Sharpe trajectories.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DecayVerdict {
    #[default]
    Healthy,
    Eroding,
    Broken,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DecayConfig {
    /// Sharpe below this absolute value triggers "Broken" regardless of slope.
    pub broken_below: f64,
    /// Sharpe slope per period below this triggers "Eroding". Negative.
    pub eroding_slope: f64,
    /// Slope acceleration (second difference) below this triggers "Broken"
    /// even if Sharpe is still above `broken_below`. Negative.
    pub acceleration_floor: f64,
}

impl Default for DecayConfig {
    fn default() -> Self {
        Self {
            broken_below: 0.5,
            eroding_slope: -0.05,
            acceleration_floor: -0.10,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DecayReport {
    pub verdict: DecayVerdict,
    /// Linear slope of the rolling-Sharpe series across the entire window.
    pub slope_per_period: f64,
    /// Second difference (slope of the slope) — negative = accelerating decay.
    pub acceleration: f64,
    /// Latest Sharpe value (the rightmost point of the series).
    pub latest_sharpe: f64,
    /// Peak Sharpe + the index where it occurred.
    pub peak_sharpe: f64,
    pub peak_index: usize,
    pub note: String,
}

pub fn analyze(rolling_sharpe: &[f64], cfg: &DecayConfig) -> DecayReport {
    let n = rolling_sharpe.len();
    if n == 0 {
        return DecayReport {
            note: "empty input".into(),
            ..Default::default()
        };
    }
    let latest = rolling_sharpe[n - 1];
    let mut peak = f64::NEG_INFINITY;
    let mut peak_idx = 0;
    for (i, &s) in rolling_sharpe.iter().enumerate() {
        if s > peak {
            peak = s;
            peak_idx = i;
        }
    }
    if n < 3 {
        return DecayReport {
            verdict: if latest < cfg.broken_below {
                DecayVerdict::Broken
            } else {
                DecayVerdict::Healthy
            },
            latest_sharpe: latest,
            peak_sharpe: peak,
            peak_index: peak_idx,
            note: "n < 3 — slope undefined, verdict from latest only".into(),
            ..Default::default()
        };
    }
    // Linear-regression slope of Sharpe vs index across the full window.
    let mean_x = (n as f64 - 1.0) / 2.0;
    let mean_y: f64 = rolling_sharpe.iter().sum::<f64>() / n as f64;
    let (mut num, mut den) = (0.0, 0.0);
    for (i, &y) in rolling_sharpe.iter().enumerate() {
        let dx = i as f64 - mean_x;
        num += dx * (y - mean_y);
        den += dx * dx;
    }
    let slope = if den > 0.0 { num / den } else { 0.0 };
    // Acceleration = slope of the slope. Split the series in two halves and
    // compare. Negative = decay is speeding up.
    let mid = n / 2;
    let first_slope = simple_slope(&rolling_sharpe[..mid]);
    let second_slope = simple_slope(&rolling_sharpe[mid..]);
    let acceleration = second_slope - first_slope;

    let verdict = if latest < cfg.broken_below || acceleration < cfg.acceleration_floor {
        DecayVerdict::Broken
    } else if slope < cfg.eroding_slope {
        DecayVerdict::Eroding
    } else {
        DecayVerdict::Healthy
    };
    let note = match verdict {
        DecayVerdict::Broken if latest < cfg.broken_below => format!(
            "latest Sharpe {latest:.2} is below the {:.2} floor",
            cfg.broken_below
        ),
        DecayVerdict::Broken => format!(
            "decay is accelerating — acceleration {acceleration:.3} past the {:.3} threshold",
            cfg.acceleration_floor
        ),
        DecayVerdict::Eroding => format!("rolling Sharpe falling at {slope:.3} per period"),
        DecayVerdict::Healthy => "rolling Sharpe trend is healthy".into(),
    };
    DecayReport {
        verdict,
        slope_per_period: slope,
        acceleration,
        latest_sharpe: latest,
        peak_sharpe: peak,
        peak_index: peak_idx,
        note,
    }
}

fn simple_slope(s: &[f64]) -> f64 {
    let n = s.len();
    if n < 2 {
        return 0.0;
    }
    let mean_x = (n as f64 - 1.0) / 2.0;
    let mean_y: f64 = s.iter().sum::<f64>() / n as f64;
    let (mut num, mut den) = (0.0, 0.0);
    for (i, &y) in s.iter().enumerate() {
        let dx = i as f64 - mean_x;
        num += dx * (y - mean_y);
        den += dx * dx;
    }
    if den > 0.0 {
        num / den
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_returns_default_with_note() {
        let r = analyze(&[], &DecayConfig::default());
        assert!(matches!(r.verdict, DecayVerdict::Healthy));
        assert_eq!(r.latest_sharpe, 0.0);
        assert!(r.note.contains("empty"));
    }

    #[test]
    fn flat_healthy_series_classifies_as_healthy() {
        let r = analyze(&[1.5, 1.5, 1.5, 1.5, 1.5, 1.5], &DecayConfig::default());
        assert!(matches!(r.verdict, DecayVerdict::Healthy));
        assert!(r.slope_per_period.abs() < 1e-9);
        assert!((r.peak_sharpe - 1.5).abs() < 1e-9);
    }

    #[test]
    fn sharpe_below_broken_floor_is_broken() {
        // Slope is upward but the LATEST value is below the floor.
        let r = analyze(&[0.1, 0.2, 0.3, 0.4], &DecayConfig::default());
        assert!(matches!(r.verdict, DecayVerdict::Broken));
        assert!(r.note.contains("below"));
    }

    #[test]
    fn slow_steady_erosion_is_eroding() {
        // Drops 0.1 each period — well above the broken floor (0.5) at the end.
        let r = analyze(
            &[2.0, 1.8, 1.6, 1.4, 1.2, 1.0, 0.8],
            &DecayConfig::default(),
        );
        assert!(
            matches!(r.verdict, DecayVerdict::Eroding),
            "expected Eroding for steady decline, got {:?} (slope={})",
            r.verdict,
            r.slope_per_period
        );
        assert!(r.slope_per_period < 0.0);
    }

    #[test]
    fn accelerating_decay_classifies_as_broken() {
        // First half: 2.0 → 1.9 (slow). Second half: 1.9 → 0.6 (cliff).
        let r = analyze(
            &[2.0, 1.99, 1.98, 1.97, 1.9, 1.5, 1.0, 0.6],
            &DecayConfig::default(),
        );
        assert!(
            matches!(r.verdict, DecayVerdict::Broken),
            "expected Broken for cliff, got {:?}",
            r.verdict
        );
        // Acceleration must be more negative than the floor.
        assert!(r.acceleration < -0.10);
    }

    #[test]
    fn peak_is_tracked_with_index() {
        let r = analyze(&[1.0, 2.0, 1.5, 1.2, 1.0], &DecayConfig::default());
        assert!((r.peak_sharpe - 2.0).abs() < 1e-9);
        assert_eq!(r.peak_index, 1);
    }

    #[test]
    fn rising_sharpe_is_healthy() {
        let r = analyze(&[1.0, 1.1, 1.2, 1.3, 1.4, 1.5], &DecayConfig::default());
        assert!(matches!(r.verdict, DecayVerdict::Healthy));
        assert!(r.slope_per_period > 0.0);
    }
}
