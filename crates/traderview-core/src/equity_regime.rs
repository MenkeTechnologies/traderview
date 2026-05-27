//! Equity-curve regime detector.
//!
//! Classifies the trader's equity curve into one of four regimes based
//! on rolling slope + variance of the last N periods:
//!
//!   - **Trending Up**: slope > 0 + low residual variance (steady gains)
//!   - **Trending Down**: slope < 0 + low residual variance (steady losses)
//!   - **Choppy**: low slope + high residual variance (no edge)
//!   - **Volatile Up/Down**: directional slope but high variance (lumpy)
//!
//! Lets the dashboard tell the trader "your system has been in a
//! drawdown regime for 12 weeks — consider position-size throttle".
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum EquityRegime {
    TrendingUp,
    TrendingDown,
    VolatileUp,
    VolatileDown,
    #[default]
    Choppy,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RegimeReport {
    pub n: usize,
    pub slope_per_period: f64,
    pub residual_stdev: f64,
    /// R² of the linear fit. Higher = trend more reliable.
    pub r_squared: f64,
    pub regime: EquityRegime,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DetectorConfig {
    /// Min |slope/mean_equity| to count as "trending" rather than Choppy.
    pub trend_slope_pct: f64,
    /// Max residual_stdev/mean_equity to call it "trending" cleanly.
    pub clean_trend_rel_stdev: f64,
}

impl Default for DetectorConfig {
    fn default() -> Self {
        Self {
            trend_slope_pct: 0.001,
            clean_trend_rel_stdev: 0.02,
        }
    }
}

pub fn analyze(equity: &[f64], cfg: &DetectorConfig) -> RegimeReport {
    let mut report = RegimeReport::default();
    let n = equity.len();
    report.n = n;
    if n < 3 {
        return report;
    }
    let mean_t = (n as f64 - 1.0) / 2.0;
    let mean_e: f64 = equity.iter().sum::<f64>() / n as f64;
    let mut num = 0.0;
    let mut den = 0.0;
    for (i, &e) in equity.iter().enumerate() {
        let dt = i as f64 - mean_t;
        num += dt * (e - mean_e);
        den += dt * dt;
    }
    if den == 0.0 {
        return report;
    }
    let slope = num / den;
    let intercept = mean_e - slope * mean_t;
    // R² + residual stdev.
    let mut ss_res = 0.0;
    let mut ss_tot = 0.0;
    for (i, &e) in equity.iter().enumerate() {
        let fit = intercept + slope * i as f64;
        ss_res += (e - fit).powi(2);
        ss_tot += (e - mean_e).powi(2);
    }
    let r_squared = if ss_tot > 0.0 {
        1.0 - ss_res / ss_tot
    } else {
        0.0
    };
    let residual_stdev = (ss_res / n as f64).sqrt();
    let rel_slope = if mean_e > 0.0 { slope / mean_e } else { 0.0 };
    let rel_stdev = if mean_e > 0.0 {
        residual_stdev / mean_e
    } else {
        0.0
    };

    let regime = if rel_slope.abs() < cfg.trend_slope_pct {
        EquityRegime::Choppy
    } else if rel_slope > 0.0 {
        if rel_stdev <= cfg.clean_trend_rel_stdev {
            EquityRegime::TrendingUp
        } else {
            EquityRegime::VolatileUp
        }
    } else {
        if rel_stdev <= cfg.clean_trend_rel_stdev {
            EquityRegime::TrendingDown
        } else {
            EquityRegime::VolatileDown
        }
    };
    report.slope_per_period = slope;
    report.residual_stdev = residual_stdev;
    report.r_squared = r_squared;
    report.regime = regime;
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn under_three_obs_returns_default() {
        let r = analyze(&[10_000.0, 10_500.0], &DetectorConfig::default());
        assert_eq!(r.regime, EquityRegime::Choppy);
    }

    #[test]
    fn steady_uptrend_classified_trending_up() {
        let equity: Vec<f64> = (0..30).map(|i| 10_000.0 + i as f64 * 100.0).collect();
        let r = analyze(&equity, &DetectorConfig::default());
        assert_eq!(r.regime, EquityRegime::TrendingUp);
        assert!(r.slope_per_period > 0.0);
        assert!(r.r_squared > 0.99);
    }

    #[test]
    fn steady_downtrend_classified_trending_down() {
        let equity: Vec<f64> = (0..30).map(|i| 20_000.0 - i as f64 * 100.0).collect();
        let r = analyze(&equity, &DetectorConfig::default());
        assert_eq!(r.regime, EquityRegime::TrendingDown);
        assert!(r.slope_per_period < 0.0);
    }

    #[test]
    fn flat_choppy_classified_choppy() {
        let equity: Vec<f64> = (0..30).map(|i| 10_000.0 + (i % 3) as f64 * 1.0).collect();
        let r = analyze(&equity, &DetectorConfig::default());
        assert_eq!(r.regime, EquityRegime::Choppy);
    }

    #[test]
    fn volatile_uptrend_classified_volatile_up() {
        // Uptrend with significant noise — R² < 1 but slope positive.
        let equity: Vec<f64> = (0..30)
            .map(|i| {
                let trend = 10_000.0 + i as f64 * 100.0;
                let noise = ((i * 73) % 31) as f64 * 100.0 - 1500.0;
                trend + noise
            })
            .collect();
        let r = analyze(&equity, &DetectorConfig::default());
        assert!(
            matches!(
                r.regime,
                EquityRegime::VolatileUp | EquityRegime::TrendingUp
            ),
            "noise should produce VolatileUp or still-clean TrendingUp"
        );
    }

    #[test]
    fn r_squared_near_one_for_perfect_line() {
        let equity: Vec<f64> = (0..30).map(|i| 10_000.0 + i as f64 * 50.0).collect();
        let r = analyze(&equity, &DetectorConfig::default());
        assert!((r.r_squared - 1.0).abs() < 1e-9);
    }

    #[test]
    fn slope_is_per_period() {
        // 30 points, equity grows by exactly 100/period → slope = 100.
        let equity: Vec<f64> = (0..30).map(|i| 10_000.0 + i as f64 * 100.0).collect();
        let r = analyze(&equity, &DetectorConfig::default());
        assert!((r.slope_per_period - 100.0).abs() < 1e-9);
    }

    #[test]
    fn custom_config_changes_regime_thresholds() {
        // Tiny slope (relative to equity) → Choppy under default,
        // but very-loose trend_slope_pct catches it.
        let equity: Vec<f64> = (0..30).map(|i| 10_000.0 + i as f64 * 0.01).collect();
        let lax = DetectorConfig {
            trend_slope_pct: 0.0000001,
            clean_trend_rel_stdev: 1.0,
        };
        let r = analyze(&equity, &lax);
        assert_eq!(r.regime, EquityRegime::TrendingUp);
    }
}
