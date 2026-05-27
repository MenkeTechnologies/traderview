//! Post-Earnings-Announcement Drift (PEAD) effect estimator.
//!
//! Bernard & Thomas (1989) showed that stocks with positive earnings
//! surprises continue to drift up (and negative-surprise stocks drift
//! down) for ~60 days post-announcement. This module emits a PEAD
//! score given:
//!
//!   - Standardized Unexpected Earnings (SUE) = (actual_eps - expected_eps) / stdev
//!   - Days since announcement
//!
//! Output: expected residual drift direction + estimated magnitude
//! (decaying linearly to 0 over `drift_window_days`).
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeadInput {
    pub actual_eps: f64,
    pub expected_eps: f64,
    /// Std-dev of analyst forecasts.
    pub forecast_stdev: f64,
    pub days_since_announcement: usize,
    /// Drift window in days (default 60).
    pub drift_window_days: usize,
    /// Max expected drift in pct (default 0.05 = 5%).
    pub max_drift_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PeadReport {
    pub sue: f64,
    pub sue_decile: i32,
    pub expected_drift_pct: f64,
    pub direction: Direction,
    pub days_remaining_in_window: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Direction { Up, Down, #[default] Neutral }

pub fn analyze(input: &PeadInput) -> PeadReport {
    if input.forecast_stdev <= 0.0 {
        return PeadReport::default();
    }
    let sue = (input.actual_eps - input.expected_eps) / input.forecast_stdev;
    // SUE deciles per Bernard & Thomas: roughly mapped from SUE values
    //   |SUE| < 0.5 → middle deciles 4-7 (no drift)
    //   0.5 ≤ |SUE| < 1.5 → outer deciles 2-3, 8-9
    //   |SUE| ≥ 1.5 → extreme deciles 1, 10
    let decile = if sue >= 2.0 { 10 }
        else if sue >= 1.0 { 9 }
        else if sue >= 0.5 { 8 }
        else if sue >  0.0 { 7 }
        else if sue >= -0.5 { 4 }
        else if sue >= -1.0 { 3 }
        else if sue >= -2.0 { 2 }
        else { 1 };
    // Drift magnitude proportional to extremity, decaying to 0 over window.
    let extremity = (sue.abs() / 2.0).min(1.0);
    let progress = if input.drift_window_days > 0 {
        (input.days_since_announcement as f64 / input.drift_window_days as f64).min(1.0)
    } else { 1.0 };
    let remaining_fraction = (1.0 - progress).max(0.0);
    let mag = input.max_drift_pct * extremity * remaining_fraction;
    let direction = if sue > 0.0 { Direction::Up }
        else if sue < 0.0 { Direction::Down }
        else { Direction::Neutral };
    let signed_drift = if matches!(direction, Direction::Down) { -mag } else { mag };
    PeadReport {
        sue,
        sue_decile: decile,
        expected_drift_pct: signed_drift,
        direction,
        days_remaining_in_window:
            input.drift_window_days.saturating_sub(input.days_since_announcement),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(actual: f64, expected: f64, stdev: f64, days: usize) -> PeadInput {
        PeadInput {
            actual_eps: actual,
            expected_eps: expected,
            forecast_stdev: stdev,
            days_since_announcement: days,
            drift_window_days: 60,
            max_drift_pct: 0.05,
        }
    }

    #[test]
    fn zero_stdev_returns_default() {
        let r = analyze(&input(2.0, 1.5, 0.0, 5));
        assert_eq!(r.sue, 0.0);
    }

    #[test]
    fn positive_sue_drift_up() {
        // Beat: SUE = (2.0 - 1.5)/0.2 = 2.5 → top decile.
        let r = analyze(&input(2.0, 1.5, 0.2, 5));
        assert!(r.sue > 2.0);
        assert_eq!(r.sue_decile, 10);
        assert_eq!(r.direction, Direction::Up);
        assert!(r.expected_drift_pct > 0.0);
    }

    #[test]
    fn negative_sue_drift_down() {
        let r = analyze(&input(1.0, 1.5, 0.2, 5));
        assert!(r.sue < 0.0);
        assert_eq!(r.direction, Direction::Down);
        assert!(r.expected_drift_pct < 0.0);
    }

    #[test]
    fn sue_in_middle_low_decile() {
        // |SUE| = 0.5 → decile 8 (just outside middle).
        let r = analyze(&input(1.6, 1.5, 0.2, 5));
        assert_eq!(r.sue_decile, 8);
    }

    #[test]
    fn drift_decays_to_zero_at_window_end() {
        let r = analyze(&input(2.0, 1.5, 0.2, 60));    // exactly at window end
        assert_eq!(r.expected_drift_pct, 0.0);
        assert_eq!(r.days_remaining_in_window, 0);
    }

    #[test]
    fn drift_strongest_at_announcement_day() {
        let day0 = analyze(&input(2.0, 1.5, 0.2, 0));
        let day30 = analyze(&input(2.0, 1.5, 0.2, 30));
        let day50 = analyze(&input(2.0, 1.5, 0.2, 50));
        assert!(day0.expected_drift_pct > day30.expected_drift_pct);
        assert!(day30.expected_drift_pct > day50.expected_drift_pct);
    }

    #[test]
    fn extreme_sue_clamps_at_max_extremity() {
        // SUE = 10 should not produce drift > max_drift_pct.
        let r = analyze(&input(3.0, 1.0, 0.2, 0));
        assert!(r.expected_drift_pct <= 0.05);
    }

    #[test]
    fn beyond_window_zero_drift() {
        let r = analyze(&input(2.0, 1.5, 0.2, 100));
        assert_eq!(r.expected_drift_pct, 0.0);
    }

    #[test]
    fn zero_sue_neutral_direction() {
        let r = analyze(&input(1.5, 1.5, 0.2, 5));
        assert_eq!(r.direction, Direction::Neutral);
        assert_eq!(r.expected_drift_pct, 0.0);
    }
}
