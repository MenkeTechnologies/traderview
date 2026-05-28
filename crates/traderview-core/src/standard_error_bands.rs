//! Standard Error Bands — Jon Andersen (TASC, 1996).
//!
//! Smoothed linear-regression channel:
//!
//!   reg_line_t = endpoint of OLS fit over period bars
//!   se_t       = standard error of the regression = sqrt(SSE / (n-2))
//!   midline    = SMA(reg_line, smoothing)
//!   se_smooth  = SMA(se,        smoothing)
//!   upper      = midline + multiplier · se_smooth
//!   lower      = midline - multiplier · se_smooth
//!
//! Unlike Bollinger Bands (which use stdev), SEB uses *regression
//! residual* stdev — so a strong trending market gives narrow bands
//! (price tracks the line) while choppy/range markets widen them.
//!
//! Pure compute. Defaults: period = 21, smoothing = 3, multiplier = 2.0.
//! Companion to `bollinger_band_width`, `linear_regression_channel`,
//! `starc_bands`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StandardErrorBandsReport {
    pub midline: Vec<Option<f64>>,
    pub upper: Vec<Option<f64>>,
    pub lower: Vec<Option<f64>>,
    pub period: usize,
    pub smoothing: usize,
    pub multiplier: f64,
}

pub fn compute(
    closes: &[f64],
    period: usize,
    smoothing: usize,
    multiplier: f64,
) -> StandardErrorBandsReport {
    let n = closes.len();
    let mut report = StandardErrorBandsReport {
        midline: vec![None; n],
        upper: vec![None; n],
        lower: vec![None; n],
        period,
        smoothing,
        multiplier,
    };
    if period < 3 || smoothing < 1
        || !multiplier.is_finite() || multiplier <= 0.0
        || n < period + smoothing { return report; }
    if closes.iter().any(|x| !x.is_finite()) { return report; }
    let p_f = period as f64;
    let x_mean = (p_f - 1.0) / 2.0;
    let x_var: f64 = (0..period).map(|i| {
        let dx = i as f64 - x_mean;
        dx * dx
    }).sum();
    let mut reg_line = vec![None; n];
    let mut se = vec![None; n];
    for i in (period - 1)..n {
        let win = &closes[i + 1 - period..=i];
        let y_mean: f64 = win.iter().sum::<f64>() / p_f;
        let mut sxy = 0.0_f64;
        for (k, &y) in win.iter().enumerate() {
            sxy += (k as f64 - x_mean) * (y - y_mean);
        }
        let slope = sxy / x_var;
        let intercept = y_mean - slope * x_mean;
        let y_hat_last = intercept + slope * (p_f - 1.0);
        let mut sse = 0.0_f64;
        for (k, &y) in win.iter().enumerate() {
            let y_hat = intercept + slope * k as f64;
            sse += (y - y_hat).powi(2);
        }
        reg_line[i] = Some(y_hat_last);
        // Standard error of regression: sqrt(SSE / (n - 2)).
        let se_val = if period > 2 { (sse / (p_f - 2.0)).max(0.0).sqrt() } else { 0.0 };
        se[i] = Some(se_val);
    }
    let midline = sma_opt(&reg_line, smoothing);
    let se_smooth = sma_opt(&se, smoothing);
    for i in 0..n {
        if let (Some(m), Some(s)) = (midline[i], se_smooth[i]) {
            report.midline[i] = Some(m);
            report.upper[i] = Some(m + multiplier * s);
            report.lower[i] = Some(m - multiplier * s);
        }
    }
    report
}

fn sma_opt(series: &[Option<f64>], period: usize) -> Vec<Option<f64>> {
    let n = series.len();
    let mut out = vec![None; n];
    if period == 0 || n < period { return out; }
    let p_f = period as f64;
    for i in (period - 1)..n {
        let win = &series[i + 1 - period..=i];
        if win.iter().any(|x| x.is_none()) { continue; }
        let s: f64 = win.iter().filter_map(|x| *x).sum();
        out[i] = Some(s / p_f);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_empty() {
        let c = vec![100.0_f64; 50];
        let r = compute(&c, 2, 3, 2.0);
        assert!(r.midline.iter().all(|x| x.is_none()));
        let r2 = compute(&c, 21, 3, 0.0);
        assert!(r2.midline.iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_empty() {
        let mut c = vec![100.0_f64; 50];
        c[5] = f64::NAN;
        let r = compute(&c, 21, 3, 2.0);
        assert!(r.midline.iter().all(|x| x.is_none()));
    }

    #[test]
    fn perfect_linear_trend_collapses_bands() {
        // Perfect fit → SSE = 0 → SE = 0 → bands collapse on midline.
        let c: Vec<f64> = (0..100).map(|i| i as f64).collect();
        let r = compute(&c, 21, 3, 2.0);
        let last = 99;
        let u = r.upper[last].unwrap();
        let l = r.lower[last].unwrap();
        assert!((u - l).abs() < 1e-6,
            "perfect trend should collapse SE bands, got width {}", u - l);
    }

    #[test]
    fn flat_market_collapses_bands() {
        let c = vec![100.0_f64; 50];
        let r = compute(&c, 21, 3, 2.0);
        for i in 30..50 {
            if let (Some(u), Some(l)) = (r.upper[i], r.lower[i]) {
                assert!((u - l).abs() < 1e-9);
            }
        }
    }

    #[test]
    fn noisy_input_widens_bands() {
        let mut state: u64 = 42;
        let c: Vec<f64> = (0..100).map(|_| {
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let r = (state >> 32) as u32 as f64 / u32::MAX as f64;
            100.0 + (r - 0.5) * 20.0
        }).collect();
        let r = compute(&c, 21, 3, 2.0);
        let last = 99;
        let u = r.upper[last].unwrap();
        let l = r.lower[last].unwrap();
        assert!(u > l + 1.0,
            "noisy input should produce nonzero band width, got {}", u - l);
    }

    #[test]
    fn upper_above_lower_always() {
        let mut state: u64 = 42;
        let c: Vec<f64> = (0..100).map(|i| {
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let r = (state >> 32) as u32 as f64 / u32::MAX as f64;
            100.0 + i as f64 * 0.5 + (r - 0.5) * 5.0
        }).collect();
        let r = compute(&c, 21, 3, 2.0);
        for i in 30..100 {
            if let (Some(u), Some(l)) = (r.upper[i], r.lower[i]) {
                assert!(u >= l - 1e-9);
            }
        }
    }

    #[test]
    fn output_lengths_match_input() {
        let c = vec![100.0_f64; 50];
        let r = compute(&c, 21, 3, 2.0);
        assert_eq!(r.midline.len(), 50);
        assert_eq!(r.upper.len(), 50);
    }
}
