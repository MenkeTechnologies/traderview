//! Keltner Channels — Chester Keltner.
//!
//! Per bar:
//!   middle = N-period EMA of close
//!   upper  = middle + multiplier × N-period ATR
//!   lower  = middle - multiplier × N-period ATR
//!
//! Like Bollinger Bands but with ATR width instead of stdev — smoother
//! and less reactive to single-bar spikes. The base of the TTM Squeeze
//! signal (BB inside KC = volatility contraction).
//!
//! Pure compute. Caller pre-computes EMA + ATR series and passes them.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct KeltnerPoint {
    pub middle: f64,
    pub upper: f64,
    pub lower: f64,
}

pub fn compute(ema: &[f64], atr: &[f64], multiplier: f64) -> Vec<KeltnerPoint> {
    if ema.len() != atr.len() { return vec![]; }
    ema.iter().zip(atr).map(|(&m, &a)| KeltnerPoint {
        middle: m,
        upper: m + multiplier * a,
        lower: m - multiplier * a,
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_empty() {
        assert!(compute(&[], &[], 1.5).is_empty());
    }

    #[test]
    fn length_mismatch_returns_empty() {
        assert!(compute(&[1.0, 2.0], &[1.0], 1.5).is_empty());
    }

    #[test]
    fn middle_equals_input_ema() {
        let out = compute(&[100.0, 101.0], &[1.0, 1.0], 1.5);
        assert_eq!(out[0].middle, 100.0);
        assert_eq!(out[1].middle, 101.0);
    }

    #[test]
    fn upper_above_middle_by_multiplier_times_atr() {
        let out = compute(&[100.0], &[2.0], 1.5);
        // upper = 100 + 1.5 × 2 = 103.
        assert_eq!(out[0].upper, 103.0);
    }

    #[test]
    fn lower_below_middle_by_multiplier_times_atr() {
        let out = compute(&[100.0], &[2.0], 1.5);
        assert_eq!(out[0].lower, 97.0);
    }

    #[test]
    fn bands_symmetric_around_middle() {
        let out = compute(&[50.0, 100.0], &[1.0, 5.0], 2.0);
        for p in &out {
            let up_offset = p.upper - p.middle;
            let down_offset = p.middle - p.lower;
            assert!((up_offset - down_offset).abs() < 1e-12);
        }
    }

    #[test]
    fn larger_multiplier_wider_bands() {
        let tight = compute(&[100.0], &[1.0], 1.0);
        let wide  = compute(&[100.0], &[1.0], 3.0);
        let tight_range = tight[0].upper - tight[0].lower;
        let wide_range  = wide[0].upper  - wide[0].lower;
        assert!(wide_range > tight_range);
        // Wide is exactly 3× tight here.
        assert!((wide_range - tight_range * 3.0).abs() < 1e-9);
    }

    #[test]
    fn zero_atr_collapses_bands_to_middle() {
        let out = compute(&[100.0], &[0.0], 1.5);
        assert_eq!(out[0].upper, 100.0);
        assert_eq!(out[0].lower, 100.0);
    }
}
