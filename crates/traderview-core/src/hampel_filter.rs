//! Hampel Filter — robust rolling outlier detection (Frank Hampel, 1974).
//!
//! For each point in a series, compute the rolling-window MEDIAN and
//! MEDIAN ABSOLUTE DEVIATION (MAD). Flag the point as an outlier if
//! it deviates from the median by more than `k_sigma · 1.4826 · MAD`,
//! where 1.4826 is the scale factor that makes MAD a consistent
//! estimator of σ under Gaussian noise.
//!
//!   median_t = median of x[t − half_window .. t + half_window]
//!   mad_t    = median |x_i − median_t|
//!   sigma_t  = 1.4826 · mad_t
//!   outlier_t = |x_t − median_t| > k_sigma · sigma_t
//!
//! The filtered series replaces outliers with the local median.
//!
//! Use cases:
//!   - Pre-process price/return series before fitting models
//!   - Cleanse market-data feeds (fat-finger detection)
//!   - Detect flash crashes / quote stuffing artifacts
//!
//! Pure compute. Companion to `realized_volatility`, `nadaraya_watson`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HampelReport {
    pub filtered_series: Vec<f64>,
    pub is_outlier: Vec<bool>,
    pub local_median: Vec<f64>,
    pub local_mad: Vec<f64>,
    pub n_outliers: usize,
}

pub fn compute(
    series: &[f64],
    half_window: usize,
    k_sigma: f64,
) -> Option<HampelReport> {
    if series.is_empty() || half_window == 0 || !k_sigma.is_finite() || k_sigma <= 0.0 {
        return None;
    }
    if series.iter().any(|x| !x.is_finite()) { return None; }
    let n = series.len();
    let mut filtered = series.to_vec();
    let mut is_outlier = vec![false; n];
    let mut local_med = vec![0.0_f64; n];
    let mut local_mad = vec![0.0_f64; n];
    let mut count = 0_usize;
    for i in 0..n {
        let lo = i.saturating_sub(half_window);
        let hi = (i + half_window + 1).min(n);
        let win: Vec<f64> = series[lo..hi].to_vec();
        let med = median(&win);
        let dev: Vec<f64> = win.iter().map(|x| (x - med).abs()).collect();
        let mad = median(&dev);
        let sigma = 1.4826 * mad;
        local_med[i] = med;
        local_mad[i] = mad;
        // MAD = 0 typically means the bulk of the window is constant; in
        // that case any nonzero deviation IS an outlier (the only
        // non-bulk value). Treat zero MAD as "tolerate exact median only".
        let is_out = if sigma > 0.0 {
            (series[i] - med).abs() > k_sigma * sigma
        } else {
            (series[i] - med).abs() > 1e-12
        };
        if is_out {
            filtered[i] = med;
            is_outlier[i] = true;
            count += 1;
        }
    }
    Some(HampelReport {
        filtered_series: filtered,
        is_outlier,
        local_median: local_med,
        local_mad,
        n_outliers: count,
    })
}

fn median(values: &[f64]) -> f64 {
    let mut sorted: Vec<f64> = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = sorted.len();
    if n.is_multiple_of(2_usize) {
        (sorted[n / 2 - 1] + sorted[n / 2]) / 2.0
    } else {
        sorted[n / 2]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_none() {
        assert!(compute(&[], 5, 3.0).is_none());
    }

    #[test]
    fn invalid_params_return_none() {
        let s = vec![1.0_f64; 20];
        assert!(compute(&s, 0, 3.0).is_none());
        assert!(compute(&s, 5, 0.0).is_none());
        assert!(compute(&s, 5, -1.0).is_none());
        assert!(compute(&s, 5, f64::NAN).is_none());
    }

    #[test]
    fn nan_input_returns_none() {
        let s = vec![1.0, f64::NAN, 2.0];
        assert!(compute(&s, 5, 3.0).is_none());
    }

    #[test]
    fn flat_input_has_no_outliers() {
        let s = vec![100.0_f64; 30];
        let r = compute(&s, 5, 3.0).unwrap();
        assert_eq!(r.n_outliers, 0);
        for o in &r.is_outlier {
            assert!(!o);
        }
    }

    #[test]
    fn single_outlier_detected() {
        let mut s: Vec<f64> = (0..30).map(|_| 100.0_f64).collect();
        s[15] = 500.0;    // gross outlier
        let r = compute(&s, 5, 3.0).unwrap();
        assert!(r.is_outlier[15]);
        // Filtered value at that index = local median.
        assert!((r.filtered_series[15] - 100.0).abs() < 1e-9);
    }

    #[test]
    fn fat_finger_replaced_by_local_median() {
        let mut s: Vec<f64> = (0..50).map(|i| 100.0 + (i as f64 * 0.1).sin()).collect();
        s[20] = 1000.0;
        let r = compute(&s, 5, 3.0).unwrap();
        assert!(r.is_outlier[20]);
        // Filtered value should be near the surrounding price.
        assert!((r.filtered_series[20] - 100.0).abs() < 2.0);
    }

    #[test]
    fn multiple_outliers_all_flagged() {
        let mut s: Vec<f64> = (0..50).map(|_| 100.0_f64).collect();
        s[10] = 500.0;
        s[25] = -200.0;
        s[40] = 800.0;
        let r = compute(&s, 5, 3.0).unwrap();
        assert!(r.is_outlier[10]);
        assert!(r.is_outlier[25]);
        assert!(r.is_outlier[40]);
        assert_eq!(r.n_outliers, 3);
    }

    #[test]
    fn higher_k_sigma_flags_fewer_outliers() {
        let mut s: Vec<f64> = (0..30).map(|_| 100.0_f64).collect();
        s[15] = 120.0;    // moderate outlier (~20σ-equivalent for flat noise)
        let strict = compute(&s, 5, 3.0).unwrap();
        let loose = compute(&s, 5, 30.0).unwrap();
        assert!(strict.n_outliers >= loose.n_outliers);
    }

    #[test]
    fn output_lengths_match_input() {
        let s: Vec<f64> = (0..50).map(|i| (i as f64).sin()).collect();
        let r = compute(&s, 5, 3.0).unwrap();
        assert_eq!(r.filtered_series.len(), 50);
        assert_eq!(r.is_outlier.len(), 50);
        assert_eq!(r.local_median.len(), 50);
        assert_eq!(r.local_mad.len(), 50);
    }
}
