//! Linear Regression Channel — Gilbert Raff.
//!
//! Fits an OLS line to the last `period` closes, then draws parallel
//! lines `n_stdev` standard deviations of the residuals above and
//! below. Final-bar value of each line is returned.
//!
//! Returned series:
//!   - slope      : slope of the fitted line per bar
//!   - regression : y-value of the fitted line at bar i
//!   - upper_band : regression + n_stdev · residual_stdev
//!   - lower_band : regression - n_stdev · residual_stdev
//!   - r_squared  : coefficient of determination over the window
//!
//! All values reported at the END of each rolling window (no
//! repainting). Bars before the first complete window are None.
//!
//! Pure compute. Companion to `linear_regression_slope` (if shipped),
//! `chande_trend_index`, `r_squared` style trend-strength indicators.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LinearRegressionChannelReport {
    pub slope: Vec<Option<f64>>,
    pub intercept: Vec<Option<f64>>,
    pub regression: Vec<Option<f64>>,
    pub upper_band: Vec<Option<f64>>,
    pub lower_band: Vec<Option<f64>>,
    pub r_squared: Vec<Option<f64>>,
    pub period: usize,
    pub n_stdev: f64,
}

pub fn compute(
    closes: &[f64],
    period: usize,
    n_stdev: f64,
) -> LinearRegressionChannelReport {
    let n = closes.len();
    let mut report = LinearRegressionChannelReport {
        slope: vec![None; n],
        intercept: vec![None; n],
        regression: vec![None; n],
        upper_band: vec![None; n],
        lower_band: vec![None; n],
        r_squared: vec![None; n],
        period,
        n_stdev,
    };
    if period < 3 || n < period
        || !n_stdev.is_finite() || n_stdev <= 0.0 { return report; }
    if closes.iter().any(|x| !x.is_finite()) { return report; }
    let p_f = period as f64;
    // Precomputed x statistics for x = 0..period-1.
    let x_mean = (p_f - 1.0) / 2.0;
    let x_var: f64 = (0..period).map(|i| {
        let dx = i as f64 - x_mean;
        dx * dx
    }).sum();
    for i in (period - 1)..n {
        let win = &closes[i + 1 - period..=i];
        let y_mean: f64 = win.iter().sum::<f64>() / p_f;
        let mut sxy = 0.0_f64;
        for (k, &y) in win.iter().enumerate() {
            let dx = k as f64 - x_mean;
            sxy += dx * (y - y_mean);
        }
        let slope = sxy / x_var;
        let intercept = y_mean - slope * x_mean;
        // y at last bar in window (x = period - 1).
        let y_hat_last = intercept + slope * (p_f - 1.0);
        // Residual stdev + R² over the window.
        let mut sse = 0.0_f64;
        let mut sst = 0.0_f64;
        for (k, &y) in win.iter().enumerate() {
            let y_hat = intercept + slope * k as f64;
            sse += (y - y_hat).powi(2);
            sst += (y - y_mean).powi(2);
        }
        let resid_stdev = (sse / p_f).sqrt();
        report.slope[i] = Some(slope);
        report.intercept[i] = Some(intercept);
        report.regression[i] = Some(y_hat_last);
        report.upper_band[i] = Some(y_hat_last + n_stdev * resid_stdev);
        report.lower_band[i] = Some(y_hat_last - n_stdev * resid_stdev);
        report.r_squared[i] = Some(if sst > 0.0 { 1.0 - sse / sst } else { 1.0 });
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_params_return_all_none() {
        let c = vec![100.0_f64; 50];
        assert!(compute(&c, 2, 2.0).slope.iter().all(|x| x.is_none()));
        assert!(compute(&c, 10, 0.0).slope.iter().all(|x| x.is_none()));
        assert!(compute(&c, 10, f64::NAN).slope.iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_all_none() {
        let mut c = vec![100.0_f64; 50];
        c[5] = f64::NAN;
        let r = compute(&c, 20, 2.0);
        assert!(r.slope.iter().all(|x| x.is_none()));
    }

    #[test]
    fn perfect_linear_uptrend_yields_unit_slope() {
        // closes = 1, 2, 3, ... with slope 1.
        let c: Vec<f64> = (0..50).map(|i| i as f64).collect();
        let r = compute(&c, 20, 2.0);
        let last_slope = r.slope[49].unwrap();
        assert!((last_slope - 1.0).abs() < 1e-9);
        // R² = 1 for perfect fit.
        let last_r2 = r.r_squared[49].unwrap();
        assert!((last_r2 - 1.0).abs() < 1e-9);
        // Residual stdev = 0 → bands collapse on regression.
        assert!((r.upper_band[49].unwrap() - r.lower_band[49].unwrap()).abs() < 1e-9);
    }

    #[test]
    fn flat_market_yields_zero_slope() {
        let c = vec![100.0_f64; 50];
        let r = compute(&c, 20, 2.0);
        let last = r.slope[49].unwrap();
        assert!(last.abs() < 1e-12);
        // R² for flat data is conventionally 1 (zero variance, zero error).
        assert!((r.r_squared[49].unwrap() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn downtrend_yields_negative_slope() {
        let c: Vec<f64> = (0..50).map(|i| 100.0 - i as f64).collect();
        let r = compute(&c, 20, 2.0);
        assert!(r.slope[49].unwrap() < 0.0);
    }

    #[test]
    fn bands_widen_with_noise() {
        // Noisy series → R² < 1, bands wider than zero.
        let mut state: u64 = 42;
        let c: Vec<f64> = (0..100).map(|i| {
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let noise = ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 10.0;
            100.0 + i as f64 * 0.5 + noise
        }).collect();
        let r = compute(&c, 20, 2.0);
        let last = 99;
        assert!(r.upper_band[last].unwrap() > r.regression[last].unwrap());
        assert!(r.lower_band[last].unwrap() < r.regression[last].unwrap());
        assert!(r.r_squared[last].unwrap() < 1.0);
    }

    #[test]
    fn output_lengths_match_input() {
        let c = vec![100.0_f64; 50];
        let r = compute(&c, 20, 2.0);
        assert_eq!(r.slope.len(), 50);
        assert_eq!(r.upper_band.len(), 50);
        assert_eq!(r.r_squared.len(), 50);
    }
}
