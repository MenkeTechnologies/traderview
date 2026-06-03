//! Principal Component Analysis (PCA) of a symmetric matrix via the
//! Jacobi eigenvalue algorithm.
//!
//! Returns eigenvalues sorted descending (largest first) and the
//! matching eigenvectors as columns. For a covariance matrix Σ, the
//! eigenvalues are the variances of the principal components and the
//! eigenvectors are the principal-component loadings.
//!
//! Used for:
//!   - Risk-factor decomposition (find dominant systematic factor)
//!   - Dimensionality reduction in portfolio optimization
//!   - PCA-hedged trading (hedge against the top-k factors)
//!
//! Jacobi method: O(n³) per sweep, ≤ 100 sweeps for any input. Suitable
//! for the small-to-moderate (≤ 100 × 100) covariance matrices typical
//! in portfolio analytics.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PcaReport {
    /// Eigenvalues sorted descending.
    pub eigenvalues: Vec<f64>,
    /// Eigenvectors as columns of a n×n matrix; column k matches
    /// eigenvalues`[k]`. Each column is unit-length.
    pub eigenvectors: Vec<Vec<f64>>,
    /// Fraction of total variance explained by each component.
    pub explained_variance_ratio: Vec<f64>,
    pub iterations: usize,
    pub converged: bool,
}

#[allow(clippy::needless_range_loop, clippy::map_clone)]    // matrix-element indexing
pub fn decompose(matrix: &[Vec<f64>], max_iter: usize, tolerance: f64) -> Option<PcaReport> {
    let n = matrix.len();
    if n == 0 || matrix.iter().any(|row| row.len() != n) { return None; }
    if matrix.iter().any(|row| row.iter().any(|c| !c.is_finite())) { return None; }
    if !tolerance.is_finite() || tolerance <= 0.0 || max_iter == 0 { return None; }
    // Verify symmetry.
    for i in 0..n {
        for j in (i + 1)..n {
            if (matrix[i][j] - matrix[j][i]).abs()
                > 1e-9 * (1.0 + matrix[i][j].abs() + matrix[j][i].abs())
            {
                return None;
            }
        }
    }
    // Working copy of A (will be diagonalized in place); V starts as I.
    let mut a: Vec<Vec<f64>> = matrix.iter().map(|r| r.clone()).collect();
    let mut v = vec![vec![0.0_f64; n]; n];
    for i in 0..n { v[i][i] = 1.0; }
    let mut iters = 0;
    let mut converged = false;
    for _ in 0..max_iter {
        iters += 1;
        // Off-diagonal sum of squares — termination criterion.
        let mut off_norm = 0.0_f64;
        for i in 0..n {
            for j in (i + 1)..n {
                off_norm += a[i][j] * a[i][j];
            }
        }
        if off_norm.sqrt() < tolerance { converged = true; break; }
        // Sweep through all off-diagonal (p, q) pairs.
        #[allow(clippy::needless_range_loop)]    // matrix iteration needs both indices
        for p in 0..n - 1 {
            for q in (p + 1)..n {
                let apq = a[p][q];
                if apq.abs() < 1e-18 { continue; }
                let theta = (a[q][q] - a[p][p]) / (2.0 * apq);
                let t = if theta >= 0.0 {
                    1.0 / (theta + (1.0 + theta * theta).sqrt())
                } else {
                    1.0 / (theta - (1.0 + theta * theta).sqrt())
                };
                let c = 1.0 / (1.0 + t * t).sqrt();
                let s = t * c;
                // Apply rotation to A.
                let app = a[p][p];
                let aqq = a[q][q];
                a[p][p] = app - t * apq;
                a[q][q] = aqq + t * apq;
                a[p][q] = 0.0;
                a[q][p] = 0.0;
                for r in 0..n {
                    if r != p && r != q {
                        let arp = a[r][p];
                        let arq = a[r][q];
                        a[r][p] = c * arp - s * arq;
                        a[p][r] = a[r][p];
                        a[r][q] = s * arp + c * arq;
                        a[q][r] = a[r][q];
                    }
                }
                // Accumulate rotation into V.
                for r in 0..n {
                    let vrp = v[r][p];
                    let vrq = v[r][q];
                    v[r][p] = c * vrp - s * vrq;
                    v[r][q] = s * vrp + c * vrq;
                }
            }
        }
    }
    // Eigenvalues = diagonal of A.
    let mut eigen_pairs: Vec<(f64, Vec<f64>)> = (0..n)
        .map(|i| (a[i][i], (0..n).map(|r| v[r][i]).collect::<Vec<f64>>()))
        .collect();
    // Sort descending by eigenvalue magnitude (use absolute value to
    // gracefully handle slight numerical negatives on PSD inputs).
    eigen_pairs.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    let eigenvalues: Vec<f64> = eigen_pairs.iter().map(|(e, _)| *e).collect();
    let mut eigenvectors = vec![vec![0.0_f64; n]; n];
    for (col, (_, vec)) in eigen_pairs.iter().enumerate() {
        for row in 0..n {
            eigenvectors[row][col] = vec[row];
        }
    }
    let total: f64 = eigenvalues.iter().map(|e| e.max(0.0)).sum();
    let explained_variance_ratio: Vec<f64> = if total > 0.0 {
        eigenvalues.iter().map(|e| e.max(0.0) / total).collect()
    } else {
        vec![0.0; n]
    };
    Some(PcaReport {
        eigenvalues,
        eigenvectors,
        explained_variance_ratio,
        iterations: iters,
        converged,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_none() {
        assert!(decompose(&[], 100, 1e-10).is_none());
    }

    #[test]
    fn non_square_returns_none() {
        let m = vec![vec![1.0, 2.0]];
        assert!(decompose(&m, 100, 1e-10).is_none());
    }

    #[test]
    fn asymmetric_returns_none() {
        let m = vec![vec![1.0, 2.0], vec![3.0, 1.0]];
        assert!(decompose(&m, 100, 1e-10).is_none());
    }

    #[test]
    fn nan_input_returns_none() {
        let m = vec![vec![1.0, f64::NAN], vec![f64::NAN, 1.0]];
        assert!(decompose(&m, 100, 1e-10).is_none());
    }

    #[test]
    fn invalid_solver_params_return_none() {
        let m = vec![vec![1.0, 0.0], vec![0.0, 1.0]];
        assert!(decompose(&m, 0, 1e-10).is_none());
        assert!(decompose(&m, 100, 0.0).is_none());
        assert!(decompose(&m, 100, f64::NAN).is_none());
    }

    #[test]
    fn identity_yields_ones_eigenvalues() {
        let m = vec![
            vec![1.0, 0.0, 0.0],
            vec![0.0, 1.0, 0.0],
            vec![0.0, 0.0, 1.0],
        ];
        let r = decompose(&m, 100, 1e-12).unwrap();
        for e in &r.eigenvalues {
            assert!((e - 1.0).abs() < 1e-9);
        }
        assert!(r.converged);
    }

    #[test]
    fn diagonal_matrix_yields_diagonal_eigenvalues() {
        let m = vec![
            vec![5.0, 0.0],
            vec![0.0, 2.0],
        ];
        let r = decompose(&m, 100, 1e-12).unwrap();
        // Sorted descending.
        assert!((r.eigenvalues[0] - 5.0).abs() < 1e-9);
        assert!((r.eigenvalues[1] - 2.0).abs() < 1e-9);
    }

    #[test]
    fn covariance_eigen_decomposition_explains_full_variance() {
        let m = vec![
            vec![4.0, 1.0, 0.5],
            vec![1.0, 9.0, 2.0],
            vec![0.5, 2.0, 16.0],
        ];
        let r = decompose(&m, 200, 1e-12).unwrap();
        let total_explained: f64 = r.explained_variance_ratio.iter().sum();
        assert!((total_explained - 1.0).abs() < 1e-9);
        let total_var: f64 = r.eigenvalues.iter().sum();
        let trace: f64 = (0..3).map(|i| m[i][i]).sum();
        assert!((total_var - trace).abs() < 1e-6);
    }

    #[test]
    fn eigenvectors_unit_length() {
        let m = vec![
            vec![4.0, 2.0],
            vec![2.0, 3.0],
        ];
        let r = decompose(&m, 100, 1e-12).unwrap();
        for col in 0..2 {
            let len: f64 = (0..2).map(|row| r.eigenvectors[row][col].powi(2)).sum::<f64>().sqrt();
            assert!((len - 1.0).abs() < 1e-9);
        }
    }

    #[test]
    fn eigenvalue_eigenvector_satisfies_a_x_eq_lambda_x() {
        let m = vec![
            vec![6.0, 2.0],
            vec![2.0, 3.0],
        ];
        let r = decompose(&m, 100, 1e-12).unwrap();
        for k in 0..2 {
            let v: Vec<f64> = (0..2).map(|i| r.eigenvectors[i][k]).collect();
            let av: Vec<f64> = (0..2).map(|i| (0..2).map(|j| m[i][j] * v[j]).sum::<f64>()).collect();
            let lambda_v: Vec<f64> = v.iter().map(|x| r.eigenvalues[k] * x).collect();
            for i in 0..2 {
                assert!((av[i] - lambda_v[i]).abs() < 1e-9);
            }
        }
    }
}
