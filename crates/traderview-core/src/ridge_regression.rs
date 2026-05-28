//! Ridge Regression (L2-Regularized OLS) — closed-form Tikhonov-
//! regularized least squares.
//!
//! Minimizes:
//!   loss(β) = (1/N) · Σ (y_i - X_i β)² + α · Σ β_j²
//!
//! Closed form on standardized features:
//!   β = (XᵀX + N·α·I)⁻¹ · Xᵀy
//!
//! L2 penalty shrinks coefficients toward zero (without forcing
//! sparsity, unlike Lasso). Stabilizes the solution when columns of X
//! are correlated (multicollinearity) where OLS would be ill-
//! conditioned.
//!
//! Features are standardized internally (mean-zero, unit-variance).
//! An intercept is fitted in original-units space (un-penalized).
//!
//! Pure compute. Companion to `lasso_regression_coordinate_descent`,
//! `linear_regression_slope`, `bayesian_regression`, `ledoit_wolf`.

#![allow(clippy::needless_range_loop)]

#[derive(Debug)]
pub struct Report {
    pub intercept: f64,
    pub coefficients: Vec<f64>,
    pub r_squared: f64,
}

pub fn compute(x: &[Vec<f64>], y: &[f64], alpha: f64) -> Option<Report> {
    let n = x.len();
    if n < 2 { return None; }
    let p = x[0].len();
    if p < 1 || y.len() != n { return None; }
    if x.iter().any(|row| row.len() != p) { return None; }
    if !alpha.is_finite() || alpha < 0.0 { return None; }
    if x.iter().any(|row| row.iter().any(|v| !v.is_finite())) { return None; }
    if y.iter().any(|v| !v.is_finite()) { return None; }
    // Standardize features.
    let mut means = vec![0.0_f64; p];
    let mut stds = vec![1.0_f64; p];
    for j in 0..p {
        let m: f64 = (0..n).map(|i| x[i][j]).sum::<f64>() / n as f64;
        means[j] = m;
        let var: f64 = (0..n).map(|i| (x[i][j] - m).powi(2)).sum::<f64>() / n as f64;
        stds[j] = var.max(1e-18).sqrt();
    }
    let mut xs = vec![vec![0.0_f64; p]; n];
    for i in 0..n {
        for j in 0..p {
            xs[i][j] = (x[i][j] - means[j]) / stds[j];
        }
    }
    let y_mean: f64 = y.iter().sum::<f64>() / n as f64;
    let y_centered: Vec<f64> = y.iter().map(|v| v - y_mean).collect();
    // XᵀX + N·α·I
    let mut xtx = vec![vec![0.0_f64; p]; p];
    for j in 0..p {
        for k in j..p {
            let s: f64 = (0..n).map(|i| xs[i][j] * xs[i][k]).sum();
            xtx[j][k] = s;
            xtx[k][j] = s;
        }
    }
    for j in 0..p { xtx[j][j] += n as f64 * alpha; }
    let xty: Vec<f64> = (0..p).map(|j|
        (0..n).map(|i| xs[i][j] * y_centered[i]).sum()
    ).collect();
    let beta_std = solve_linear_system(xtx, xty)?;
    let coefficients: Vec<f64> = beta_std.iter().zip(stds.iter())
        .map(|(b, s)| b / s).collect();
    let intercept = y_mean - coefficients.iter().zip(means.iter())
        .map(|(b, m)| b * m).sum::<f64>();
    // R² = 1 - SSE/SST.
    let y_hat: Vec<f64> = (0..n).map(|i| {
        intercept + (0..p).map(|j| coefficients[j] * x[i][j]).sum::<f64>()
    }).collect();
    let sse: f64 = y.iter().zip(y_hat.iter()).map(|(a, b)| (a - b).powi(2)).sum();
    let sst: f64 = y.iter().map(|a| (a - y_mean).powi(2)).sum();
    let r_squared = if sst > 1e-18 { 1.0 - sse / sst } else { 0.0 };
    Some(Report { intercept, coefficients, r_squared })
}

/// Gauss-Jordan elimination — returns None on singular system.
fn solve_linear_system(mut a: Vec<Vec<f64>>, mut b: Vec<f64>) -> Option<Vec<f64>> {
    let n = b.len();
    for i in 0..n {
        let mut pivot = i;
        let mut pmax = a[i][i].abs();
        for k in i + 1..n {
            if a[k][i].abs() > pmax { pmax = a[k][i].abs(); pivot = k; }
        }
        if pmax < 1e-15 { return None; }
        a.swap(i, pivot);
        b.swap(i, pivot);
        let pv = a[i][i];
        for j in 0..n { a[i][j] /= pv; }
        b[i] /= pv;
        for k in 0..n {
            if k != i && a[k][i].abs() > 0.0 {
                let factor = a[k][i];
                for j in 0..n { a[k][j] -= factor * a[i][j]; }
                b[k] -= factor * b[i];
            }
        }
    }
    Some(b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_none() {
        let x = vec![vec![1.0_f64, 2.0]; 5];
        let y = vec![1.0_f64; 5];
        assert!(compute(&x, &y, -1.0).is_none());
        assert!(compute(&x, &y[..3], 0.1).is_none());
        let mut bad = x.clone();
        bad[0][0] = f64::NAN;
        assert!(compute(&bad, &y, 0.1).is_none());
    }

    #[test]
    fn zero_alpha_recovers_ols_coefficients() {
        // y = 2·x1 + 3·x2 + 1, α=0 → exact OLS recovery.
        let n = 100;
        let mut x = Vec::with_capacity(n);
        let mut y = Vec::with_capacity(n);
        for i in 0..n {
            let x1 = i as f64 * 0.1;
            let x2 = (i as f64 * 0.05).sin();
            x.push(vec![x1, x2]);
            y.push(2.0 * x1 + 3.0 * x2 + 1.0);
        }
        let r = compute(&x, &y, 0.0).unwrap();
        assert!((r.coefficients[0] - 2.0).abs() < 1e-6);
        assert!((r.coefficients[1] - 3.0).abs() < 1e-6);
        assert!((r.intercept - 1.0).abs() < 1e-6);
    }

    #[test]
    fn large_alpha_shrinks_coefficients_toward_zero() {
        let n = 100;
        let mut x = Vec::with_capacity(n);
        let mut y = Vec::with_capacity(n);
        for i in 0..n {
            let x1 = i as f64 * 0.1;
            x.push(vec![x1]);
            y.push(2.0 * x1 + 1.0);
        }
        let small = compute(&x, &y, 0.01).unwrap();
        let large = compute(&x, &y, 100.0).unwrap();
        assert!(large.coefficients[0].abs() < small.coefficients[0].abs());
    }

    #[test]
    fn r_squared_near_one_for_clean_signal() {
        let n = 100;
        let mut x = Vec::with_capacity(n);
        let mut y = Vec::with_capacity(n);
        for i in 0..n {
            let x1 = i as f64 * 0.1;
            x.push(vec![x1]);
            y.push(2.0 * x1 + 1.0);
        }
        let r = compute(&x, &y, 0.0).unwrap();
        assert!((r.r_squared - 1.0).abs() < 1e-6);
    }

    #[test]
    fn r_squared_in_unit_interval() {
        let n = 50;
        let mut x = Vec::with_capacity(n);
        let mut y = Vec::with_capacity(n);
        for i in 0..n {
            x.push(vec![(i as f64 * 0.3).sin(), (i as f64 * 0.1).cos()]);
            y.push((i as f64 * 0.2).sin());
        }
        let r = compute(&x, &y, 0.1).unwrap();
        assert!(r.r_squared <= 1.0 + 1e-9);
    }

    #[test]
    fn output_length_matches_feature_count() {
        let x = vec![vec![1.0_f64, 2.0, 3.0]; 10];
        let y = vec![1.0_f64; 10];
        let r = compute(&x, &y, 0.1).unwrap();
        assert_eq!(r.coefficients.len(), 3);
    }
}
