//! Isotonic Regression via Pool-Adjacent-Violators (PAVA).
//!
//! Fits a monotonically non-decreasing function ŷ to (x_i, y_i)
//! observations under the constraint:
//!
//!   ŷ_1 ≤ ŷ_2 ≤ … ≤ ŷ_n
//!
//! by minimizing Σ w_i · (ŷ_i − y_i)². The PAVA algorithm walks the
//! sorted-by-x sequence and pools any consecutive pair that violates
//! monotonicity, replacing them with their weighted mean. Linear time.
//!
//! Use cases:
//!   - Probability-output calibration (Platt scaling alternative)
//!   - Constrained price-impact curve fitting
//!   - Score-to-probability remapping
//!
//! Set `decreasing = true` to fit a monotonically non-INCREASING
//! function instead.
//!
//! Pure compute. Companion to `nadaraya_watson`, `factor_neutralization`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IsotonicReport {
    pub fitted_values: Vec<f64>,
    pub n_observations: usize,
    pub decreasing: bool,
}

pub fn fit(x: &[f64], y: &[f64], decreasing: bool) -> Option<IsotonicReport> {
    let n = x.len();
    if n < 2 || y.len() != n { return None; }
    if x.iter().any(|v| !v.is_finite()) || y.iter().any(|v| !v.is_finite()) {
        return None;
    }
    // Sort by x.
    let mut idx: Vec<usize> = (0..n).collect();
    idx.sort_by(|a, b| x[*a].partial_cmp(&x[*b]).unwrap_or(std::cmp::Ordering::Equal));
    let sign = if decreasing { -1.0 } else { 1.0 };
    let sorted_y: Vec<f64> = idx.iter().map(|i| sign * y[*i]).collect();
    // PAVA. Maintain stack of (cumulative sum, cumulative weight, end-index).
    let mut stack: Vec<(f64, f64, usize)> = Vec::new();
    for (i, &yi) in sorted_y.iter().enumerate() {
        let mut sum = yi;
        let mut weight = 1.0_f64;
        // Pool while previous block's mean > current block's mean.
        while let Some(&(s_prev, w_prev, _)) = stack.last() {
            if s_prev / w_prev > sum / weight {
                stack.pop();
                sum += s_prev;
                weight += w_prev;
            } else {
                break;
            }
        }
        stack.push((sum, weight, i));
    }
    // Expand the stack back into per-observation fitted values (in sorted order).
    let mut sorted_fitted = vec![0.0_f64; n];
    let mut prev_end = 0_usize;
    for (sum, weight, end) in &stack {
        let mean = sum / weight;
        for slot in sorted_fitted.iter_mut().take(end + 1).skip(prev_end) { *slot = mean; }
        prev_end = end + 1;
    }
    // Unsort back to original order; un-flip sign if decreasing.
    let mut fitted = vec![0.0_f64; n];
    for (sort_pos, orig_idx) in idx.iter().enumerate() {
        fitted[*orig_idx] = sign * sorted_fitted[sort_pos];
    }
    Some(IsotonicReport { fitted_values: fitted, n_observations: n, decreasing })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn too_short_returns_none() {
        assert!(fit(&[1.0], &[1.0], false).is_none());
    }

    #[test]
    fn mismatched_returns_none() {
        assert!(fit(&[1.0, 2.0], &[1.0], false).is_none());
    }

    #[test]
    fn nan_returns_none() {
        assert!(fit(&[1.0, f64::NAN], &[1.0, 2.0], false).is_none());
    }

    #[test]
    fn monotone_increasing_input_unchanged() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let r = fit(&x, &y, false).unwrap();
        for (orig, fitted) in y.iter().zip(r.fitted_values.iter()) {
            assert!((orig - fitted).abs() < 1e-12);
        }
    }

    #[test]
    fn violation_pooled_to_mean() {
        // y has a single violator: 1, 3, 2, 4, 5 — pool middle pair.
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![1.0, 3.0, 2.0, 4.0, 5.0];
        let r = fit(&x, &y, false).unwrap();
        // Indices 1 and 2 should be pooled to mean (3+2)/2 = 2.5.
        assert!((r.fitted_values[1] - 2.5).abs() < 1e-12);
        assert!((r.fitted_values[2] - 2.5).abs() < 1e-12);
        assert!((r.fitted_values[0] - 1.0).abs() < 1e-12);
        assert!((r.fitted_values[3] - 4.0).abs() < 1e-12);
        assert!((r.fitted_values[4] - 5.0).abs() < 1e-12);
    }

    #[test]
    fn fitted_is_monotone_after_violations() {
        let x: Vec<f64> = (1..=20).map(|i| i as f64).collect();
        let y: Vec<f64> = (0..20).map(|i| (i as f64).sin() * 5.0 + i as f64 * 0.5).collect();
        let r = fit(&x, &y, false).unwrap();
        for w in r.fitted_values.windows(2) {
            assert!(w[1] >= w[0] - 1e-9);
        }
    }

    #[test]
    fn decreasing_fit_monotone_non_increasing() {
        let x: Vec<f64> = (0..20).map(|i| i as f64).collect();
        let y: Vec<f64> = (0..20).map(|i| (i as f64).sin() * 3.0 - i as f64 * 0.2).collect();
        let r = fit(&x, &y, true).unwrap();
        for w in r.fitted_values.windows(2) {
            assert!(w[1] <= w[0] + 1e-9);
        }
    }

    #[test]
    fn fitted_length_matches_input() {
        let x: Vec<f64> = (0..50).map(|i| i as f64).collect();
        let y: Vec<f64> = (0..50).map(|i| (i as f64).cos()).collect();
        let r = fit(&x, &y, false).unwrap();
        assert_eq!(r.fitted_values.len(), 50);
    }
}
