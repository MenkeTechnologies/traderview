//! Savitzky-Golay Polynomial Smoothing Filter (1964).
//!
//! Convolves the signal with coefficients of a least-squares
//! polynomial fit over a sliding window. Preserves peak heights
//! and widths much better than running averages.
//!
//! Hard-coded coefficient tables for the most common configurations
//! (cubic polynomial, smoothing window 5/7/9/11). For arbitrary
//! window/poly-order, callers can use `nadaraya_watson` for the
//! kernel-smoothing analogue.
//!
//! Pure compute. Companion to `nadaraya_watson`, `hodrick_prescott`,
//! `kalman_filter_1d`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SavitzkyGolayReport {
    pub smoothed: Vec<Option<f64>>,
    pub window: usize,
    pub polynomial_order: usize,
}

pub fn compute(
    series: &[f64],
    window: usize,
    polynomial_order: usize,
) -> Option<SavitzkyGolayReport> {
    let n = series.len();
    // Window must be odd and at least 3; poly order must be < window.
    if n < window || window < 3 || window.is_multiple_of(2_usize)
        || polynomial_order >= window {
        return None;
    }
    if series.iter().any(|x| !x.is_finite()) { return None; }
    // Standard Savitzky-Golay coefficients for cubic (order 3) smoothing.
    let coefs: Vec<f64> = match (window, polynomial_order) {
        (5, 3) => vec![-3.0, 12.0, 17.0, 12.0, -3.0],
        (7, 3) => vec![-2.0, 3.0, 6.0, 7.0, 6.0, 3.0, -2.0],
        (9, 3) => vec![-21.0, 14.0, 39.0, 54.0, 59.0, 54.0, 39.0, 14.0, -21.0],
        (11, 3) => vec![-36.0, 9.0, 44.0, 69.0, 84.0, 89.0, 84.0, 69.0, 44.0, 9.0, -36.0],
        // Quadratic (order 2) shares cubic coefficients for symmetric smoothing.
        (5, 2) | (5, 1) => vec![-3.0, 12.0, 17.0, 12.0, -3.0],
        (7, 2) | (7, 1) => vec![-2.0, 3.0, 6.0, 7.0, 6.0, 3.0, -2.0],
        _ => return None,
    };
    let norm: f64 = coefs.iter().sum();
    if norm <= 0.0 { return None; }
    let half = window / 2;
    let mut out = vec![None; n];
    for i in half..(n - half) {
        let mut acc = 0.0_f64;
        for (k, &c) in coefs.iter().enumerate() {
            acc += c * series[i + k - half];
        }
        out[i] = Some(acc / norm);
    }
    Some(SavitzkyGolayReport {
        smoothed: out,
        window,
        polynomial_order,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_window_returns_none() {
        let s = vec![1.0_f64; 20];
        assert!(compute(&s, 2, 1).is_none());
        assert!(compute(&s, 4, 1).is_none());    // even window
        assert!(compute(&s, 5, 10).is_none());   // poly order >= window
    }

    #[test]
    fn nan_returns_none() {
        let s = vec![1.0, f64::NAN, 2.0, 3.0, 4.0];
        assert!(compute(&s, 5, 3).is_none());
    }

    #[test]
    fn unsupported_config_returns_none() {
        let s = vec![1.0_f64; 50];
        // Window 13 with cubic isn't in our table.
        assert!(compute(&s, 13, 3).is_none());
    }

    #[test]
    fn flat_input_yields_flat_output() {
        let s = vec![100.0_f64; 50];
        let r = compute(&s, 7, 3).unwrap();
        for v in r.smoothed.iter().flatten() {
            assert!((v - 100.0).abs() < 1e-9);
        }
    }

    #[test]
    fn linear_input_preserved_exactly() {
        // SG filters exactly preserve polynomial trends up to their order.
        let s: Vec<f64> = (0..30).map(|i| 2.0 * i as f64 + 1.0).collect();
        let r = compute(&s, 7, 3).unwrap();
        // Interior points should match the linear trend.
        for (i, v) in r.smoothed.iter().enumerate().skip(3).take(24) {
            if let Some(vv) = v {
                assert!((vv - s[i]).abs() < 1e-9,
                    "at i={i}: smoothed {} vs input {}", vv, s[i]);
            }
        }
    }

    #[test]
    fn noisy_signal_smoothed_within_input_range() {
        let mut state: u64 = 42;
        let s: Vec<f64> = (0..200).map(|_| {
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            100.0 + ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 10.0
        }).collect();
        let r = compute(&s, 11, 3).unwrap();
        let in_min = s.iter().cloned().fold(f64::INFINITY, f64::min);
        let in_max = s.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        for v in r.smoothed.iter().flatten() {
            assert!(*v >= in_min - 1.0 && *v <= in_max + 1.0);
        }
    }

    #[test]
    fn window_and_order_reported() {
        let s = vec![1.0_f64; 50];
        let r = compute(&s, 9, 3).unwrap();
        assert_eq!(r.window, 9);
        assert_eq!(r.polynomial_order, 3);
    }
}
