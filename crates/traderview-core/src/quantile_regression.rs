//! Quantile Regression — Koenker & Bassett (1978).
//!
//! Fits y = α + β · x by minimizing the asymmetric check-function loss:
//!
//!   L(α, β) = Σ ρ_τ(y_t − α − β · x_t)
//!   ρ_τ(u) = u · (τ − 𝟙{u < 0})
//!
//! For τ = 0.5 this is least-absolute-deviations (median regression);
//! for τ = 0.05 / 0.95 it produces tail-quantile fits useful for VaR
//! modeling.
//!
//! Uses iteratively-reweighted-least-squares (IRLS) with Huber-style
//! weight clipping. Univariate predictor for clarity.
//!
//! Use cases:
//!   - Conditional VaR regression (τ = 0.01 / 0.05)
//!   - Median-robust beta when residuals are heavy-tailed
//!   - Quantile term-structure of expected returns
//!
//! Pure compute. Companion to `factor_models`, `newey_west`,
//! `white_robust_se`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QuantileRegressionReport {
    pub alpha: f64,
    pub beta: f64,
    pub tau: f64,
    pub objective_value: f64,
    pub iterations: usize,
    pub n_observations: usize,
}

pub fn fit(
    x: &[f64],
    y: &[f64],
    tau: f64,
) -> Option<QuantileRegressionReport> {
    let n = x.len();
    if n < 10 || y.len() != n { return None; }
    if !tau.is_finite() || !(0.001..=0.999).contains(&tau) { return None; }
    if x.iter().any(|v| !v.is_finite()) || y.iter().any(|v| !v.is_finite()) {
        return None;
    }
    // OLS warm start.
    let n_f = n as f64;
    let x_mean: f64 = x.iter().sum::<f64>() / n_f;
    let y_mean: f64 = y.iter().sum::<f64>() / n_f;
    let mut sxx = 0.0_f64;
    let mut sxy = 0.0_f64;
    for i in 0..n {
        sxx += (x[i] - x_mean).powi(2);
        sxy += (x[i] - x_mean) * (y[i] - y_mean);
    }
    if sxx <= 0.0 { return None; }
    let mut beta = sxy / sxx;
    let mut alpha = y_mean - beta * x_mean;
    // IRLS: w_i = (τ − 𝟙{r < 0}) / |r| (clipped to avoid singularity).
    let max_iter = 200;
    let mut last_obj = f64::INFINITY;
    let mut iters = 0;
    for it in 0..max_iter {
        iters = it + 1;
        let mut sum_w = 0.0_f64;
        let mut sum_wx = 0.0_f64;
        let mut sum_wy = 0.0_f64;
        let mut sum_wxx = 0.0_f64;
        let mut sum_wxy = 0.0_f64;
        for i in 0..n {
            let r = y[i] - alpha - beta * x[i];
            let abs_r = r.abs().max(1e-6);
            let psi = if r < 0.0 { -(1.0 - tau) } else { tau };
            let w = psi.abs() / abs_r;
            sum_w += w;
            sum_wx += w * x[i];
            sum_wy += w * y[i];
            sum_wxx += w * x[i] * x[i];
            sum_wxy += w * x[i] * y[i];
        }
        let det = sum_w * sum_wxx - sum_wx * sum_wx;
        if det.abs() < 1e-18 { break; }
        beta = (sum_w * sum_wxy - sum_wx * sum_wy) / det;
        alpha = (sum_wy - beta * sum_wx) / sum_w;
        // Objective: Σ ρ_τ(r).
        let obj: f64 = (0..n).map(|i| {
            let r = y[i] - alpha - beta * x[i];
            r * (tau - if r < 0.0 { 1.0 } else { 0.0 })
        }).sum();
        if (last_obj - obj).abs() < 1e-9 { break; }
        last_obj = obj;
    }
    let obj_final: f64 = (0..n).map(|i| {
        let r = y[i] - alpha - beta * x[i];
        r * (tau - if r < 0.0 { 1.0 } else { 0.0 })
    }).sum();
    Some(QuantileRegressionReport {
        alpha,
        beta,
        tau,
        objective_value: obj_final,
        iterations: iters,
        n_observations: n,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn box_muller(n: usize, seed: u64) -> Vec<f64> {
        let mut state = seed;
        (0..n).map(|_| {
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let u1 = ((state >> 32) as f64 / u32::MAX as f64).max(1e-12);
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let u2 = (state >> 32) as f64 / u32::MAX as f64;
            (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
        }).collect()
    }

    #[test]
    fn too_short_returns_none() {
        assert!(fit(&[1.0; 5], &[1.0; 5], 0.5).is_none());
    }

    #[test]
    fn invalid_tau_returns_none() {
        let x: Vec<f64> = (0..20).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|xi| 2.0 * xi).collect();
        assert!(fit(&x, &y, 0.0).is_none());
        assert!(fit(&x, &y, 1.0).is_none());
        assert!(fit(&x, &y, f64::NAN).is_none());
    }

    #[test]
    fn nan_returns_none() {
        let x: Vec<f64> = (0..20).map(|i| i as f64).collect();
        let mut y: Vec<f64> = x.iter().map(|xi| 2.0 * xi).collect();
        y[5] = f64::NAN;
        assert!(fit(&x, &y, 0.5).is_none());
    }

    #[test]
    fn perfect_linear_fit_recovers_coefficients() {
        let x: Vec<f64> = (0..30).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|xi| 2.0 * xi + 1.0).collect();
        let r = fit(&x, &y, 0.5).unwrap();
        assert!((r.beta - 2.0).abs() < 0.05);
        assert!((r.alpha - 1.0).abs() < 0.5);
    }

    #[test]
    fn median_regression_matches_ols_on_symmetric_noise() {
        // With symmetric noise, median ≈ OLS slope.
        let x: Vec<f64> = (0..200).map(|i| i as f64).collect();
        let noise = box_muller(200, 42);
        let y: Vec<f64> = x.iter().zip(noise.iter()).map(|(xi, n)| 2.0 * xi + n).collect();
        let r = fit(&x, &y, 0.5).unwrap();
        assert!((r.beta - 2.0).abs() < 0.10,
            "median: β should be ≈ 2.0, got {}", r.beta);
    }

    #[test]
    fn tau_recorded() {
        let x: Vec<f64> = (0..30).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|xi| 2.0 * xi).collect();
        let r = fit(&x, &y, 0.25).unwrap();
        assert_eq!(r.tau, 0.25);
    }

    #[test]
    fn n_observations_reported() {
        let x: Vec<f64> = (0..50).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|xi| 2.0 * xi).collect();
        let r = fit(&x, &y, 0.5).unwrap();
        assert_eq!(r.n_observations, 50);
    }

    #[test]
    fn objective_value_non_negative() {
        let x: Vec<f64> = (0..30).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|xi| 2.0 * xi).collect();
        let r = fit(&x, &y, 0.5).unwrap();
        assert!(r.objective_value >= 0.0);
    }
}
