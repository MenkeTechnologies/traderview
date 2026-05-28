//! Theil-Sen Estimator — non-parametric robust linear-regression
//! slope/intercept that breaks down at 29% outlier contamination
//! (vs OLS's 0%).
//!
//! Algorithm:
//!   slope     = median over all pairs i<j of (y_j - y_i) / (x_j - x_i)
//!   intercept = median over all i of (y_i - slope · x_i)
//!
//! Pairs with x_j == x_i are skipped (vertical line undefined).
//!
//! Complexity is O(n²) pairs — fine for n in the few hundreds, slow
//! for larger n. For larger data use Siegel's repeated-medians
//! variant (out of scope; this module ships canonical Theil-Sen).
//!
//! Pure compute. Companion to `linear_regression_slope`,
//! `ridge_regression`, `kendall_tau`.

#![allow(clippy::needless_range_loop)]

#[derive(Debug)]
pub struct Report {
    pub slope: f64,
    pub intercept: f64,
    pub n_pairs: usize,
}

pub fn compute(x: &[f64], y: &[f64]) -> Option<Report> {
    let n = x.len();
    if n < 3 || y.len() != n { return None; }
    if x.iter().chain(y.iter()).any(|v| !v.is_finite()) { return None; }
    let mut slopes = Vec::with_capacity(n * (n - 1) / 2);
    for i in 0..n {
        for j in i + 1..n {
            let dx = x[j] - x[i];
            if dx == 0.0 { continue; }
            slopes.push((y[j] - y[i]) / dx);
        }
    }
    if slopes.is_empty() { return None; }
    let slope = median(&mut slopes);
    let mut intercepts: Vec<f64> = (0..n).map(|i| y[i] - slope * x[i]).collect();
    let intercept = median(&mut intercepts);
    Some(Report { slope, intercept, n_pairs: slopes.len() })
}

fn median(v: &mut [f64]) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = v.len();
    if n.is_multiple_of(2) {
        0.5 * (v[n / 2 - 1] + v[n / 2])
    } else {
        v[n / 2]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_none() {
        let x = vec![1.0_f64; 2];
        let y = vec![1.0_f64; 2];
        assert!(compute(&x, &y).is_none());
        let x10 = vec![1.0_f64; 10];
        let y10 = vec![1.0_f64; 10];
        assert!(compute(&x10, &y10[..5]).is_none());
        let mut nan = y10.clone();
        nan[0] = f64::NAN;
        assert!(compute(&x10, &nan).is_none());
        // All x identical → no valid pair.
        let flat_x = vec![5.0_f64; 10];
        let y = vec![1.0_f64; 10];
        assert!(compute(&flat_x, &y).is_none());
    }

    #[test]
    fn perfect_line_recovered() {
        // y = 2·x + 3 → slope 2, intercept 3.
        let x: Vec<f64> = (0..20).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|xi| 2.0 * xi + 3.0).collect();
        let r = compute(&x, &y).unwrap();
        assert!((r.slope - 2.0).abs() < 1e-12);
        assert!((r.intercept - 3.0).abs() < 1e-12);
    }

    #[test]
    fn robust_against_outliers() {
        // Clean line + 25% wild outliers. OLS would tilt; Theil-Sen
        // recovers slope ≈ 2 because it medians over pairs.
        let mut x: Vec<f64> = (0..20).map(|i| i as f64).collect();
        let mut y: Vec<f64> = x.iter().map(|xi| 2.0 * xi + 1.0).collect();
        // 5 outliers (25% of 20).
        for i in [3, 7, 11, 14, 18] {
            y[i] += 100.0;
        }
        let r = compute(&x, &y).unwrap();
        // Slope should still be close to 2.
        assert!((r.slope - 2.0).abs() < 0.5);
        // Keep `x` mutable-binding warning quiet without changing semantics.
        x.push(0.0); let _ = x;
    }

    #[test]
    fn zero_slope_for_flat_y() {
        let x: Vec<f64> = (0..15).map(|i| i as f64).collect();
        let y = vec![42.0_f64; 15];
        let r = compute(&x, &y).unwrap();
        assert!(r.slope.abs() < 1e-12);
        assert!((r.intercept - 42.0).abs() < 1e-12);
    }

    #[test]
    fn pair_count_equals_n_choose_2_when_all_x_distinct() {
        let x: Vec<f64> = (0..10).map(|i| i as f64).collect();
        let y = vec![1.0_f64; 10];
        let r = compute(&x, &y).unwrap();
        assert_eq!(r.n_pairs, 10 * 9 / 2);
    }

    #[test]
    fn duplicate_x_pairs_are_skipped() {
        // 5 distinct x values, duplicated → only 5C2 = 10 valid pairs
        // (one per (i,j) with x_j > x_i).
        let x = vec![0.0, 0.0, 1.0, 1.0, 2.0];
        let y = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let r = compute(&x, &y).unwrap();
        // Pairs with dx != 0: (0,2)(0,3)(0,4)(1,2)(1,3)(1,4)(2,4)(3,4) = 8.
        assert_eq!(r.n_pairs, 8);
    }
}
