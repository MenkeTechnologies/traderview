//! Rolling Z-score detector.
//!
//! For a price series, emit per-bar Z-score relative to the rolling
//! window mean + stdev. Used as a generic mean-reversion entry filter
//! (in addition to pair-trade specific use cases).
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ZPoint {
    pub price: f64,
    pub window_mean: f64,
    pub window_stdev: f64,
    pub z_score: f64,
}

pub fn compute(series: &[f64], window: usize) -> Vec<ZPoint> {
    let n = series.len();
    let mut out = vec![ZPoint::default(); n];
    if n < window || window == 0 {
        return out;
    }
    for i in (window - 1)..n {
        let w = &series[(i + 1 - window)..=i];
        let mean = w.iter().sum::<f64>() / window as f64;
        let var = w.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / window as f64;
        let std = var.sqrt();
        let price = series[i];
        let z = if std > 0.0 { (price - mean) / std } else { 0.0 };
        out[i] = ZPoint {
            price,
            window_mean: mean,
            window_stdev: std,
            z_score: z,
        };
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_empty() {
        assert!(compute(&[], 20).is_empty());
    }

    #[test]
    fn series_shorter_than_window_emits_zeros() {
        let out = compute(&[1.0, 2.0, 3.0], 10);
        for p in &out {
            assert_eq!(p.z_score, 0.0);
        }
    }

    #[test]
    fn z_score_zero_at_window_mean() {
        // 5 bars all at 100 → window mean = 100, stdev = 0, z forced to 0.
        let out = compute(&[100.0; 5], 5);
        assert_eq!(out[4].z_score, 0.0);
    }

    #[test]
    fn z_score_two_for_two_sigma_deviation() {
        // 5 bars at [10, 20, 30, 40, 50] → mean=30, var = ((20)^2+(10)^2+0+(10)^2+(20)^2)/5
        // = (400+100+0+100+400)/5 = 1000/5 = 200. std = √200 ≈ 14.14.
        // Latest value 50 → z = (50-30)/14.14 ≈ 1.41.
        let out = compute(&[10.0, 20.0, 30.0, 40.0, 50.0], 5);
        let expected = (50.0 - 30.0) / 200.0_f64.sqrt();
        assert!((out[4].z_score - expected).abs() < 1e-9);
    }

    #[test]
    fn rolling_window_advances_with_each_bar() {
        // 8 bars, window 3 → 6 points have non-zero z (indices 2..=7).
        let series: Vec<f64> = (1..=8).map(|i| i as f64).collect();
        let out = compute(&series, 3);
        for p in &out[..2] {
            assert_eq!(p.z_score, 0.0); // pre-warmup
        }
        for (i, p) in out.iter().enumerate().take(8).skip(2) {
            // Each window of [i-2, i-1, i] = arithmetic progression → window_mean = i.
            assert!((p.window_mean - i as f64).abs() < 1e-9);
        }
    }

    #[test]
    fn flat_series_z_score_zero_throughout() {
        let out = compute(&[5.0; 20], 5);
        for p in &out[4..] {
            assert_eq!(p.z_score, 0.0);
        }
    }

    #[test]
    fn outlier_at_end_produces_high_z() {
        let mut series: Vec<f64> = (0..20).map(|i| 100.0 + (i % 2) as f64).collect();
        series[19] = 200.0; // outlier
        let out = compute(&series, 10);
        assert!(
            out[19].z_score.abs() > 2.0,
            "outlier should produce |z| > 2, got {}",
            out[19].z_score
        );
    }
}
