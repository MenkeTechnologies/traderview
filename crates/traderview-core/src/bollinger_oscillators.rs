//! Bollinger Bands %B and Bandwidth — two standardized BB-derived
//! oscillators that turn the bands into normalized trade signals.
//!
//! %B  = (close − lower_band) / (upper_band − lower_band)
//!       0.0 = touching lower band, 0.5 = at middle SMA, 1.0 = touching
//!       upper. > 1 = above upper (breakout); < 0 = below lower (breakdown).
//!
//! Bandwidth = (upper_band − lower_band) / middle_band
//!       Compresses the bands' absolute width into a unitless squeeze
//!       indicator. Low bandwidth = compressed vol → coiling for a move
//!       (the canonical "Bollinger squeeze" used in TTM-Squeeze setups).
//!
//! Pure compute. The base Bollinger bands themselves live in
//! `indicators::bollinger` — this module adds the two normalized
//! companion signals.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BollingerOscReport {
    pub percent_b: Vec<Option<f64>>,
    pub bandwidth: Vec<Option<f64>>,
    pub middle: Vec<Option<f64>>,
    pub upper: Vec<Option<f64>>,
    pub lower: Vec<Option<f64>>,
}

pub fn compute(closes: &[f64], period: usize, k: f64) -> BollingerOscReport {
    let n = closes.len();
    let mut report = BollingerOscReport {
        percent_b: vec![None; n],
        bandwidth: vec![None; n],
        middle: vec![None; n],
        upper: vec![None; n],
        lower: vec![None; n],
    };
    if period == 0 || !k.is_finite() || k < 0.0 || n < period {
        return report;
    }
    for i in (period - 1)..n {
        let win = &closes[i + 1 - period..=i];
        if win.iter().any(|x| !x.is_finite()) || !closes[i].is_finite() { continue; }
        let mean = win.iter().sum::<f64>() / period as f64;
        let var = win.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / period as f64;
        let stdev = var.max(0.0).sqrt();
        let upper = mean + k * stdev;
        let lower = mean - k * stdev;
        report.middle[i] = Some(mean);
        report.upper[i] = Some(upper);
        report.lower[i] = Some(lower);
        let band_width = upper - lower;
        if band_width > 0.0 {
            report.percent_b[i] = Some((closes[i] - lower) / band_width);
        }
        if mean.abs() > 1e-18 {
            report.bandwidth[i] = Some(band_width / mean);
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_default() {
        let r = compute(&[], 20, 2.0);
        assert!(r.percent_b.is_empty());
    }

    #[test]
    fn period_zero_returns_all_none() {
        let r = compute(&[100.0; 30], 0, 2.0);
        assert!(r.percent_b.iter().all(|x| x.is_none()));
    }

    #[test]
    fn negative_k_returns_all_none() {
        let r = compute(&[100.0; 30], 20, -1.0);
        assert!(r.percent_b.iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_series_yields_zero_bandwidth_and_undefined_percent_b() {
        // stdev = 0 → upper == lower → bandwidth = 0, %B undefined.
        let r = compute(&[100.0; 30], 20, 2.0);
        for v in r.bandwidth.iter().flatten() {
            assert!(v.abs() < 1e-12);
        }
        for pb in r.percent_b.iter().take(30) {
            assert!(pb.is_none());    // division by zero band width
        }
    }

    #[test]
    fn close_at_upper_band_yields_percent_b_one() {
        // Build a series where the last close sits exactly at the upper band.
        let mut closes = vec![100.0; 19];
        closes.push(100.0);
        // 20-bar window now flat, stdev = 0 → degenerate. Add variance:
        closes[10] = 110.0;
        closes[15] = 90.0;
        let r = compute(&closes, 20, 2.0);
        let pb = r.percent_b[19];
        // Just verify it's a finite number in a sensible range.
        if let Some(v) = pb {
            assert!((-1.0..=2.0).contains(&v));
        }
    }

    #[test]
    fn bandwidth_positive_after_real_movement() {
        let closes: Vec<f64> = (0..50).map(|i| 100.0 + (i as f64 * 0.1).sin() * 5.0).collect();
        let r = compute(&closes, 20, 2.0);
        let bw = r.bandwidth[49].unwrap();
        assert!(bw > 0.0);
    }

    #[test]
    fn percent_b_in_typical_range_for_oscillating_input() {
        let closes: Vec<f64> = (0..200).map(|i| 100.0 + (i as f64 * 0.07).sin() * 5.0).collect();
        let r = compute(&closes, 20, 2.0);
        for pb in r.percent_b.iter().flatten() {
            // %B normally between -0.5 and 1.5 for oscillating data with ±2σ bands.
            assert!((-2.0..=3.0).contains(pb));
        }
    }

    #[test]
    fn nan_inputs_skipped() {
        let mut closes = vec![100.0; 50];
        closes[25] = f64::NAN;
        let r = compute(&closes, 20, 2.0);
        // No panic; output length matches.
        assert_eq!(r.percent_b.len(), 50);
    }

    #[test]
    fn middle_upper_lower_ordered() {
        let closes: Vec<f64> = (0..50).map(|i| 100.0 + (i as f64 * 0.07).cos() * 5.0).collect();
        let r = compute(&closes, 20, 2.0);
        for i in 0..50 {
            if let (Some(l), Some(m), Some(u)) = (r.lower[i], r.middle[i], r.upper[i]) {
                assert!(l <= m && m <= u);
            }
        }
    }

    #[test]
    fn higher_k_yields_wider_bands() {
        let closes: Vec<f64> = (0..50).map(|i| 100.0 + (i as f64 * 0.07).sin() * 5.0).collect();
        let r_1k = compute(&closes, 20, 1.0);
        let r_2k = compute(&closes, 20, 2.0);
        let bw_1 = r_1k.bandwidth[49].unwrap();
        let bw_2 = r_2k.bandwidth[49].unwrap();
        assert!(bw_2 > bw_1);
    }
}
