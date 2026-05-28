//! Distance Correlation — Székely, Rizzo & Bakirov (2007).
//!
//! Measures dependence between two random variables, with the property
//! that it is ZERO if and only if the variables are independent
//! (unlike Pearson, which only captures LINEAR dependence and can be
//! zero for highly nonlinear-but-dependent pairs).
//!
//! Definition for samples (X_1, Y_1), …, (X_n, Y_n):
//!
//!   a_{ij} = |X_i − X_j|,  b_{ij} = |Y_i − Y_j|
//!   ā_i. = mean over j,    ā.. = grand mean
//!   A_{ij} = a_{ij} − ā_i. − ā_.j + ā..
//!   (B same for b)
//!
//!   dCov² = (1/n²) · Σ A_{ij} · B_{ij}
//!   dVarX² = (1/n²) · Σ A_{ij}²
//!   dVarY² = (1/n²) · Σ B_{ij}²
//!   dCor   = √(dCov² / √(dVarX² · dVarY²))
//!
//! Range [0, 1]: 0 = independent, 1 = perfect dependence.
//!
//! O(n²) memory; intended for moderate sample sizes (n ≤ ~1000).
//!
//! Pure compute. Companion to `spearman_correlation`, `realized_correlation`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DistanceCorrelationReport {
    pub distance_correlation: f64,
    pub distance_covariance: f64,
    pub distance_variance_x: f64,
    pub distance_variance_y: f64,
    pub n_observations: usize,
}

pub fn compute(x: &[f64], y: &[f64]) -> Option<DistanceCorrelationReport> {
    let n = x.len();
    if n < 5 || y.len() != n { return None; }
    if x.iter().any(|v| !v.is_finite()) || y.iter().any(|v| !v.is_finite()) {
        return None;
    }
    // Pairwise absolute-distance matrices.
    let mut a = vec![vec![0.0_f64; n]; n];
    let mut b = vec![vec![0.0_f64; n]; n];
    for i in 0..n {
        for j in 0..n {
            a[i][j] = (x[i] - x[j]).abs();
            b[i][j] = (y[i] - y[j]).abs();
        }
    }
    // Row, column, grand means.
    let n_f = n as f64;
    let a_row: Vec<f64> = a.iter().map(|r| r.iter().sum::<f64>() / n_f).collect();
    let a_col: Vec<f64> = (0..n).map(|j| {
        (0..n).map(|i| a[i][j]).sum::<f64>() / n_f
    }).collect();
    let a_grand: f64 = a.iter().flatten().sum::<f64>() / (n_f * n_f);
    let b_row: Vec<f64> = b.iter().map(|r| r.iter().sum::<f64>() / n_f).collect();
    let b_col: Vec<f64> = (0..n).map(|j| {
        (0..n).map(|i| b[i][j]).sum::<f64>() / n_f
    }).collect();
    let b_grand: f64 = b.iter().flatten().sum::<f64>() / (n_f * n_f);
    let mut cov_sq = 0.0_f64;
    let mut var_x_sq = 0.0_f64;
    let mut var_y_sq = 0.0_f64;
    for i in 0..n {
        for j in 0..n {
            let a_centered = a[i][j] - a_row[i] - a_col[j] + a_grand;
            let b_centered = b[i][j] - b_row[i] - b_col[j] + b_grand;
            cov_sq += a_centered * b_centered;
            var_x_sq += a_centered * a_centered;
            var_y_sq += b_centered * b_centered;
        }
    }
    let denom = n_f * n_f;
    cov_sq /= denom;
    var_x_sq /= denom;
    var_y_sq /= denom;
    let dcov = cov_sq.max(0.0).sqrt();
    let dvar_x = var_x_sq.max(0.0).sqrt();
    let dvar_y = var_y_sq.max(0.0).sqrt();
    let dcor_denom = (dvar_x * dvar_y).sqrt();
    let dcor = if dcor_denom > 0.0 { (dcov / dcor_denom).clamp(0.0, 1.0) } else { 0.0 };
    Some(DistanceCorrelationReport {
        distance_correlation: dcor,
        distance_covariance: dcov,
        distance_variance_x: dvar_x,
        distance_variance_y: dvar_y,
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
    fn too_short_or_mismatched_returns_none() {
        assert!(compute(&[1.0; 3], &[1.0; 3]).is_none());
        assert!(compute(&[1.0; 5], &[1.0; 4]).is_none());
    }

    #[test]
    fn nan_returns_none() {
        assert!(compute(&[1.0, f64::NAN, 2.0, 3.0, 4.0],
            &[1.0, 2.0, 3.0, 4.0, 5.0]).is_none());
    }

    #[test]
    fn linear_relation_yields_positive_distance_correlation() {
        let x: Vec<f64> = (1..=50).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|xi| 2.0 * xi + 1.0).collect();
        let r = compute(&x, &y).unwrap();
        assert!((r.distance_correlation - 1.0).abs() < 1e-6,
            "linear: dCor should be 1.0, got {}", r.distance_correlation);
    }

    #[test]
    fn nonlinear_dependence_detected() {
        // y = x² captured by distance correlation but missed by Pearson on
        // symmetric-around-zero x. dCor for this case is ~0.49 (well above
        // chance) — Pearson would be ~0.
        let x: Vec<f64> = (-25..=25).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|xi| xi * xi).collect();
        let r = compute(&x, &y).unwrap();
        assert!(r.distance_correlation > 0.3,
            "y = x² should have meaningful dCor, got {}", r.distance_correlation);
        // Sanity check: Pearson is ~0 for this case.
        let mean_x: f64 = x.iter().sum::<f64>() / x.len() as f64;
        let mean_y: f64 = y.iter().sum::<f64>() / y.len() as f64;
        let pearson_num: f64 = x.iter().zip(y.iter())
            .map(|(xi, yi)| (xi - mean_x) * (yi - mean_y)).sum();
        assert!(pearson_num.abs() < 1.0,
            "Pearson should be ~0 by symmetry, got {pearson_num}");
    }

    #[test]
    fn independent_samples_yield_low_distance_correlation() {
        let x = box_muller(200, 42);
        let y = box_muller(200, 13);
        let r = compute(&x, &y).unwrap();
        assert!(r.distance_correlation < 0.20,
            "independent samples: dCor should be small, got {}", r.distance_correlation);
    }

    #[test]
    fn distance_correlation_in_unit_range() {
        let x = box_muller(50, 1);
        let y = box_muller(50, 2);
        let r = compute(&x, &y).unwrap();
        assert!((0.0..=1.0).contains(&r.distance_correlation));
    }

    #[test]
    fn flat_input_yields_zero_correlation() {
        let x = vec![1.0_f64; 30];
        let y = box_muller(30, 1);
        let r = compute(&x, &y).unwrap();
        // dVar_x = 0 → dCor = 0 (by implementation).
        assert_eq!(r.distance_correlation, 0.0);
    }

    #[test]
    fn n_observations_reported() {
        let x = box_muller(50, 1);
        let y = box_muller(50, 2);
        let r = compute(&x, &y).unwrap();
        assert_eq!(r.n_observations, 50);
    }
}
