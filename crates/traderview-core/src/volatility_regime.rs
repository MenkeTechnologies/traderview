//! Volatility regime classifier — calm / normal / elevated / crisis.
//!
//! Current realized vol vs. historical distribution gives a regime
//! classification that informs position sizing: trade smaller (or
//! step aside) in crisis regimes, larger in calm. Inverts the dumb
//! "always size by fixed dollar risk" rule that bleeds out in storms.
//!
//! Caller supplies a current realized vol and a history of realized
//! vol from prior comparable windows. Classification is based on
//! percentile within the history:
//!
//!   - **Calm**:     ≤ 25th percentile
//!   - **Normal**:   25-75th percentile
//!   - **Elevated**: 75-95th percentile
//!   - **Crisis**:   ≥ 95th percentile
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum VolRegime {
    Calm,
    #[default]
    Normal,
    Elevated,
    Crisis,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VolRegimeReport {
    pub regime: VolRegime,
    /// Percentile of `current_vol` within `history` (0..=1).
    pub percentile: f64,
    /// Recommended position-size multiplier: 1.5× in Calm, 1.0× Normal,
    /// 0.5× Elevated, 0.0× Crisis. Caller can override the thresholds.
    pub suggested_size_multiplier: f64,
    pub current_vol: f64,
    pub note: String,
}

pub fn classify(current_vol: f64, history: &[f64]) -> VolRegimeReport {
    if history.is_empty() {
        return VolRegimeReport {
            current_vol,
            suggested_size_multiplier: 1.0,
            note: "no historical vol available — defaulting to Normal".into(),
            ..Default::default()
        };
    }
    if !current_vol.is_finite() || current_vol < 0.0 {
        return VolRegimeReport {
            current_vol,
            suggested_size_multiplier: 1.0,
            note: "current vol invalid".into(),
            ..Default::default()
        };
    }
    // Compute percentile as the fraction of history strictly below current_vol.
    let lower = history.iter().filter(|&&h| h < current_vol).count() as f64;
    let percentile = lower / history.len() as f64;
    let regime = if percentile >= 0.95 {
        VolRegime::Crisis
    } else if percentile >= 0.75 {
        VolRegime::Elevated
    } else if percentile <= 0.25 {
        VolRegime::Calm
    } else {
        VolRegime::Normal
    };
    let mult = match regime {
        VolRegime::Calm => 1.5,
        VolRegime::Normal => 1.0,
        VolRegime::Elevated => 0.5,
        VolRegime::Crisis => 0.0,
    };
    let note = match regime {
        VolRegime::Calm => format!(
            "vol at {:.0}th percentile — historically calm",
            percentile * 100.0
        ),
        VolRegime::Normal => format!(
            "vol at {:.0}th percentile — normal regime",
            percentile * 100.0
        ),
        VolRegime::Elevated => format!("vol at {:.0}th percentile — size down", percentile * 100.0),
        VolRegime::Crisis => format!("vol at {:.0}th percentile — step aside", percentile * 100.0),
    };
    VolRegimeReport {
        regime,
        percentile,
        suggested_size_multiplier: mult,
        current_vol,
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_history_returns_default_with_full_size() {
        let r = classify(0.20, &[]);
        assert!(matches!(r.regime, VolRegime::Normal));
        assert_eq!(r.suggested_size_multiplier, 1.0);
        assert!(r.note.contains("no historical"));
    }

    #[test]
    fn current_at_75th_pct_is_elevated() {
        let hist: Vec<f64> = (0..100).map(|i| i as f64 / 100.0).collect();
        // 0.75 is at the 75th percentile within 0..=0.99.
        let r = classify(0.75, &hist);
        assert!(
            matches!(r.regime, VolRegime::Elevated),
            "got {:?} pct={}",
            r.regime,
            r.percentile
        );
        assert_eq!(r.suggested_size_multiplier, 0.5);
    }

    #[test]
    fn current_at_99th_pct_is_crisis_and_zero_size() {
        let hist: Vec<f64> = (0..100).map(|i| i as f64 / 100.0).collect();
        // 0.99 is at the very top — 99% of history is strictly below it.
        let r = classify(0.99, &hist);
        assert!(matches!(r.regime, VolRegime::Crisis));
        assert_eq!(r.suggested_size_multiplier, 0.0);
    }

    #[test]
    fn current_at_lowest_pct_is_calm_and_oversized() {
        let hist: Vec<f64> = (1..=100).map(|i| i as f64 / 100.0).collect();
        // 0.01 is below most — only 0 entries strictly below 0.01 (since hist starts at 0.01).
        let r = classify(0.005, &hist);
        assert!(matches!(r.regime, VolRegime::Calm));
        assert_eq!(r.suggested_size_multiplier, 1.5);
    }

    #[test]
    fn current_at_middle_is_normal() {
        let hist: Vec<f64> = (0..100).map(|i| i as f64 / 100.0).collect();
        // 0.50 is right in the middle.
        let r = classify(0.50, &hist);
        assert!(matches!(r.regime, VolRegime::Normal));
        assert_eq!(r.suggested_size_multiplier, 1.0);
    }

    #[test]
    fn invalid_current_vol_returns_normal_with_warning() {
        let r = classify(f64::NAN, &[0.1, 0.2, 0.3]);
        assert!(matches!(r.regime, VolRegime::Normal));
        assert!(r.note.contains("invalid"));
        let r2 = classify(-0.5, &[0.1, 0.2, 0.3]);
        assert!(matches!(r2.regime, VolRegime::Normal));
        assert!(r2.note.contains("invalid"));
    }
}
