//! Lasso Regression (L1-Regularized OLS) — coordinate-descent fitter.
//!
//! Minimizes:
//!   loss(β) = (1/2N) · Σ (y_i - X_i β)² + α · Σ |β_j|
//!
//! Coordinate descent update for each j (with all other β_k fixed):
//!   z_j = (1/N) · Σ_i x_{i,j} · (y_i - ŷ_i + x_{i,j} · β_j)
//!   norm_j = (1/N) · Σ_i x_{i,j}²
//!   β_j = soft_threshold(z_j, α) / norm_j
//!
//! soft_threshold(z, α) = sign(z) · max(|z| - α, 0). The L1 penalty
//! forces some β_j to exactly zero, producing a sparse solution
//! (variable selection). α controls sparsity vs fit:
//!   - α → 0     reduces to OLS.
//!   - α → ∞     all β = 0.
//!
//! Input is standardized internally (mean-zero, unit-variance per
//! feature) so α is on a comparable scale across features. The
//! returned coefficients are in original-units space; an intercept
//! is fitted but not penalized.
//!
//! Pure compute. Companion to `ridge_regression`,
//! `linear_regression_slope`, `bayesian_regression`.

#![allow(clippy::needless_range_loop)]

#[derive(Debug)]
pub struct Report {
    pub intercept: f64,
    pub coefficients: Vec<f64>,
    pub iterations: u32,
    pub converged: bool,
    pub non_zero_count: usize,
}

pub fn compute(
    x: &[Vec<f64>], y: &[f64],
    alpha: f64, max_iter: u32, tol: f64,
) -> Option<Report> {
    let n = x.len();
    if n < 2 { return None; }
    let p = x[0].len();
    if p < 1 || y.len() != n { return None; }
    if x.iter().any(|row| row.len() != p) { return None; }
    if !alpha.is_finite() || alpha < 0.0 { return None; }
    if !tol.is_finite() || tol <= 0.0 || max_iter == 0 { return None; }
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
    // Center y.
    let y_mean: f64 = y.iter().sum::<f64>() / n as f64;
    let y_centered: Vec<f64> = y.iter().map(|v| v - y_mean).collect();
    // Pre-compute column squared norms (post-standardization: all = 1).
    let mut beta = vec![0.0_f64; p];
    let mut residual = y_centered.clone();
    let mut iters = 0_u32;
    let mut converged = false;
    let nf = n as f64;
    for _ in 0..max_iter {
        iters += 1;
        let mut max_change = 0.0_f64;
        for j in 0..p {
            let old_beta = beta[j];
            // Restore feature j's contribution to residual.
            for i in 0..n { residual[i] += xs[i][j] * old_beta; }
            // Compute z_j = (1/N) Σ x_{i,j} · residual_i
            let z: f64 = (0..n).map(|i| xs[i][j] * residual[i]).sum::<f64>() / nf;
            let new_beta = soft_threshold(z, alpha);
            beta[j] = new_beta;
            let change = (new_beta - old_beta).abs();
            if change > max_change { max_change = change; }
            // Re-subtract new contribution.
            for i in 0..n { residual[i] -= xs[i][j] * new_beta; }
        }
        if max_change < tol { converged = true; break; }
    }
    // De-standardize coefficients.
    let coefficients: Vec<f64> = beta.iter().zip(stds.iter())
        .map(|(b, s)| b / s).collect();
    let intercept = y_mean - coefficients.iter().zip(means.iter())
        .map(|(b, m)| b * m).sum::<f64>();
    let non_zero_count = coefficients.iter().filter(|c| c.abs() > 1e-12).count();
    Some(Report { intercept, coefficients, iterations: iters, converged, non_zero_count })
}

fn soft_threshold(z: f64, alpha: f64) -> f64 {
    let abs_z = z.abs();
    if abs_z <= alpha { 0.0 } else { z.signum() * (abs_z - alpha) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_none() {
        let x = vec![vec![1.0_f64, 2.0]; 5];
        let y = vec![1.0_f64; 5];
        assert!(compute(&x, &y, -1.0, 100, 1e-6).is_none());
        assert!(compute(&x, &y, 0.1, 0, 1e-6).is_none());
        assert!(compute(&x, &y, 0.1, 100, 0.0).is_none());
        assert!(compute(&x, &y[..3], 0.1, 100, 1e-6).is_none());
        let mut bad = x.clone();
        bad[0][0] = f64::NAN;
        assert!(compute(&bad, &y, 0.1, 100, 1e-6).is_none());
    }

    #[test]
    fn zero_alpha_matches_ols_close() {
        // y = 2·x1 + 3·x2 + 1 + noise; with α≈0, coefficients should
        // recover (2, 3) and intercept ≈ 1.
        let n = 100;
        let mut x = Vec::with_capacity(n);
        let mut y = Vec::with_capacity(n);
        for i in 0..n {
            let x1 = i as f64 * 0.1;
            let x2 = (i as f64 * 0.05).sin();
            x.push(vec![x1, x2]);
            y.push(2.0 * x1 + 3.0 * x2 + 1.0);
        }
        let r = compute(&x, &y, 1e-6, 1000, 1e-9).unwrap();
        assert!((r.coefficients[0] - 2.0).abs() < 0.01);
        assert!((r.coefficients[1] - 3.0).abs() < 0.05);
        assert!((r.intercept - 1.0).abs() < 0.1);
    }

    #[test]
    fn large_alpha_zeros_out_coefficients() {
        let n = 50;
        let mut x = Vec::with_capacity(n);
        let mut y = Vec::with_capacity(n);
        for i in 0..n {
            x.push(vec![i as f64 * 0.1, (i as f64 * 0.05).sin()]);
            y.push(2.0 * i as f64 * 0.1);
        }
        let r = compute(&x, &y, 100.0, 1000, 1e-9).unwrap();
        assert_eq!(r.non_zero_count, 0);
        for c in &r.coefficients { assert!(c.abs() < 1e-9); }
    }

    #[test]
    fn convergence_flag_reflects_actual_convergence() {
        let n = 50;
        let mut x = Vec::with_capacity(n);
        let mut y = Vec::with_capacity(n);
        for i in 0..n {
            x.push(vec![i as f64 * 0.1]);
            y.push(2.0 * i as f64 * 0.1);
        }
        let r = compute(&x, &y, 0.01, 10000, 1e-12).unwrap();
        assert!(r.converged);
        assert!(r.iterations < 10000);
    }

    #[test]
    fn output_length_matches_feature_count() {
        let x = vec![vec![1.0_f64, 2.0, 3.0]; 10];
        let y = vec![1.0_f64; 10];
        let r = compute(&x, &y, 0.1, 100, 1e-6).unwrap();
        assert_eq!(r.coefficients.len(), 3);
    }

    #[test]
    fn intercept_matches_y_mean_when_all_coefficients_zero() {
        let n = 50;
        let mut x = Vec::with_capacity(n);
        let mut y = Vec::with_capacity(n);
        for i in 0..n {
            x.push(vec![i as f64 * 0.1, (i as f64 * 0.05).sin()]);
            y.push(5.0 + i as f64 * 0.1);
        }
        let r = compute(&x, &y, 100.0, 1000, 1e-9).unwrap();
        let y_mean: f64 = y.iter().sum::<f64>() / n as f64;
        assert!((r.intercept - y_mean).abs() < 1e-9);
    }
}
