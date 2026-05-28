//! Holt-Winters double exponential smoothing — trend-aware smoother
//! and short-horizon forecaster.
//!
//! Updates two state variables per bar:
//!   level_t = α·y_t + (1−α)·(level_{t−1} + trend_{t−1})
//!   trend_t = β·(level_t − level_{t−1}) + (1−β)·trend_{t−1}
//!   forecast(t+h) = level_t + h · trend_t
//!
//! With (α, β) tuned to (0.3, 0.1) for typical price series. Non-
//! seasonal variant only — adding seasonality would extend to the
//! full triple-exponential form.
//!
//! Pure compute. Returns smoothed level + trend per bar plus an
//! h-step-ahead forecast vector.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HoltWintersReport {
    pub level: Vec<Option<f64>>,
    pub trend: Vec<Option<f64>>,
    pub forecast: Vec<f64>,
}

pub fn compute(
    series: &[f64],
    alpha: f64,
    beta: f64,
    forecast_horizon: usize,
) -> Option<HoltWintersReport> {
    let n = series.len();
    if n < 2
        || !alpha.is_finite() || !(0.0..=1.0).contains(&alpha) || alpha == 0.0
        || !beta.is_finite() || !(0.0..=1.0).contains(&beta) || beta == 0.0
    {
        return None;
    }
    // Seed from the first valid pair of observations.
    let mut idx = 0;
    while idx < n - 1 && !(series[idx].is_finite() && series[idx + 1].is_finite()) {
        idx += 1;
    }
    if idx >= n - 1 { return None; }
    let mut level = series[idx];
    let mut trend = series[idx + 1] - series[idx];
    let mut report = HoltWintersReport {
        level: vec![None; n],
        trend: vec![None; n],
        forecast: Vec::with_capacity(forecast_horizon),
    };
    report.level[idx] = Some(level);
    report.trend[idx] = Some(trend);
    report.level[idx + 1] = Some(level + trend);    // initial smoother step
    report.trend[idx + 1] = Some(trend);
    let mut prev_level = level + trend;
    let mut prev_trend = trend;
    for (i, val) in series.iter().enumerate().skip(idx + 2) {
        if !val.is_finite() {
            // Carry prior state forward, don't update.
            report.level[i] = Some(prev_level);
            report.trend[i] = Some(prev_trend);
            continue;
        }
        level = alpha * val + (1.0 - alpha) * (prev_level + prev_trend);
        trend = beta * (level - prev_level) + (1.0 - beta) * prev_trend;
        report.level[i] = Some(level);
        report.trend[i] = Some(trend);
        prev_level = level;
        prev_trend = trend;
    }
    for h in 1..=forecast_horizon {
        report.forecast.push(prev_level + (h as f64) * prev_trend);
    }
    Some(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_none() {
        assert!(compute(&[1.0], 0.3, 0.1, 5).is_none());
        assert!(compute(&[1.0, 2.0], 0.0, 0.1, 5).is_none());
        assert!(compute(&[1.0, 2.0], 1.5, 0.1, 5).is_none());
        assert!(compute(&[1.0, 2.0], 0.3, 0.0, 5).is_none());
        assert!(compute(&[1.0, 2.0], 0.3, 1.5, 5).is_none());
        assert!(compute(&[1.0, 2.0], f64::NAN, 0.1, 5).is_none());
    }

    #[test]
    fn all_nan_returns_none() {
        let s = vec![f64::NAN; 10];
        assert!(compute(&s, 0.3, 0.1, 5).is_none());
    }

    #[test]
    fn constant_series_yields_zero_trend() {
        let s = vec![100.0; 20];
        let r = compute(&s, 0.3, 0.1, 5).unwrap();
        // After warmup, trend should converge to 0.
        let last_trend = r.trend.iter().rev().find_map(|x| *x).unwrap();
        assert!(last_trend.abs() < 1e-6);
        let last_level = r.level.iter().rev().find_map(|x| *x).unwrap();
        assert!((last_level - 100.0).abs() < 1e-6);
    }

    #[test]
    fn linear_series_recovers_trend_in_steady_state() {
        // y_t = 100 + 2·t → trend should converge to 2.
        let s: Vec<f64> = (0..50).map(|i| 100.0 + 2.0 * i as f64).collect();
        let r = compute(&s, 0.5, 0.5, 5).unwrap();
        let last_trend = r.trend.iter().rev().find_map(|x| *x).unwrap();
        assert!((last_trend - 2.0).abs() < 0.5);
    }

    #[test]
    fn forecast_uses_last_level_plus_trend_times_h() {
        let s: Vec<f64> = (0..30).map(|i| 100.0 + i as f64).collect();
        let r = compute(&s, 0.5, 0.5, 5).unwrap();
        let last_level = r.level.iter().rev().find_map(|x| *x).unwrap();
        let last_trend = r.trend.iter().rev().find_map(|x| *x).unwrap();
        assert_eq!(r.forecast.len(), 5);
        for (h, f) in r.forecast.iter().enumerate() {
            let h_p1 = (h + 1) as f64;
            assert!((f - (last_level + h_p1 * last_trend)).abs() < 1e-9);
        }
    }

    #[test]
    fn zero_horizon_yields_empty_forecast() {
        let s = vec![100.0; 20];
        let r = compute(&s, 0.3, 0.1, 0).unwrap();
        assert!(r.forecast.is_empty());
    }

    #[test]
    fn nan_intermediate_carries_state_forward() {
        let mut s = vec![100.0; 30];
        s[15] = f64::NAN;
        let r = compute(&s, 0.3, 0.1, 0).unwrap();
        assert!(r.level[14].is_some());
        assert!(r.level[15].is_some());
        assert!(r.level[16].is_some());
        // State at 15 carries from 14.
        assert!((r.level[15].unwrap() - r.level[14].unwrap()).abs() < 1e-6);
    }
}
