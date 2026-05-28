//! Cholesky decomposition — lower-triangular L such that A = L · Lᵀ
//! for a symmetric positive-definite matrix A.
//!
//! Used in Monte Carlo correlated-draw generation: given uncorrelated
//! standard normals z, the vector L · z has covariance A. Also used
//! in numerically stable matrix-inverse routines.
//!
//! Algorithm: classic Cholesky-Banachiewicz recursion. O(n³) time.
//! Returns None when A is not positive-definite (any diagonal ≤ 0 in
//! the factored matrix flags this).
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CholeskyReport {
    /// Lower-triangular factor (zeros above diagonal).
    pub l: Vec<Vec<f64>>,
    /// Product of diagonal entries (≡ √det(A)).
    pub sqrt_determinant: f64,
}

#[allow(clippy::needless_range_loop)]    // matrix iteration uses both i,j indices for symmetric access
pub fn decompose(a: &[Vec<f64>]) -> Option<CholeskyReport> {
    let n = a.len();
    if n == 0 || a.iter().any(|row| row.len() != n) { return None; }
    if a.iter().any(|row| row.iter().any(|c| !c.is_finite())) { return None; }
    // Verify symmetry (within float tolerance).
    for i in 0..n {
        for j in (i + 1)..n {
            if (a[i][j] - a[j][i]).abs() > 1e-9 * (1.0 + a[i][j].abs() + a[j][i].abs()) {
                return None;
            }
        }
    }
    let mut l = vec![vec![0.0_f64; n]; n];
    for i in 0..n {
        for j in 0..=i {
            let mut sum = 0.0_f64;
            for k in 0..j {
                sum += l[i][k] * l[j][k];
            }
            if i == j {
                let diag = a[i][i] - sum;
                if diag <= 0.0 || !diag.is_finite() {
                    return None;    // not positive-definite
                }
                l[i][j] = diag.sqrt();
            } else {
                if l[j][j].abs() < 1e-18 { return None; }
                l[i][j] = (a[i][j] - sum) / l[j][j];
            }
        }
    }
    let sqrt_det: f64 = (0..n).map(|i| l[i][i]).product();
    Some(CholeskyReport { l, sqrt_determinant: sqrt_det })
}

/// Multiply L · z to produce correlated draws from uncorrelated z.
#[allow(clippy::needless_range_loop)]    // matrix product needs both indices for triangle iteration
pub fn multiply(l: &[Vec<f64>], z: &[f64]) -> Option<Vec<f64>> {
    let n = l.len();
    if z.len() != n { return None; }
    if l.iter().any(|row| row.len() != n) { return None; }
    let mut out = vec![0.0_f64; n];
    for i in 0..n {
        for j in 0..=i {
            out[i] += l[i][j] * z[j];
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_none() {
        assert!(decompose(&[]).is_none());
    }

    #[test]
    fn non_square_returns_none() {
        assert!(decompose(&[vec![1.0, 0.5]]).is_none());
    }

    #[test]
    fn asymmetric_returns_none() {
        let a = vec![vec![1.0, 0.5], vec![0.7, 1.0]];
        assert!(decompose(&a).is_none());
    }

    #[test]
    fn non_positive_definite_returns_none() {
        // Indefinite matrix.
        let a = vec![vec![1.0, 2.0], vec![2.0, 1.0]];
        assert!(decompose(&a).is_none());
    }

    #[test]
    fn nan_input_returns_none() {
        let a = vec![vec![1.0, f64::NAN], vec![f64::NAN, 1.0]];
        assert!(decompose(&a).is_none());
    }

    #[test]
    fn identity_yields_identity() {
        let a = vec![
            vec![1.0, 0.0, 0.0],
            vec![0.0, 1.0, 0.0],
            vec![0.0, 0.0, 1.0],
        ];
        let r = decompose(&a).unwrap();
        for i in 0..3 {
            for j in 0..3 {
                let expected = if i == j { 1.0 } else { 0.0 };
                assert!((r.l[i][j] - expected).abs() < 1e-12);
            }
        }
        assert!((r.sqrt_determinant - 1.0).abs() < 1e-12);
    }

    #[test]
    fn diagonal_yields_sqrt_diagonal() {
        let a = vec![
            vec![4.0, 0.0],
            vec![0.0, 9.0],
        ];
        let r = decompose(&a).unwrap();
        assert!((r.l[0][0] - 2.0).abs() < 1e-12);
        assert!((r.l[1][1] - 3.0).abs() < 1e-12);
        assert!((r.sqrt_determinant - 6.0).abs() < 1e-12);
    }

    #[test]
    #[allow(clippy::needless_range_loop)]    // matrix-equality check needs both indices
    fn ll_t_equals_a() {
        let a = vec![
            vec![4.0, 12.0, -16.0],
            vec![12.0, 37.0, -43.0],
            vec![-16.0, -43.0, 98.0],
        ];
        let r = decompose(&a).unwrap();
        for i in 0..3 {
            for j in 0..3 {
                let mut s = 0.0;
                for k in 0..3 {
                    s += r.l[i][k] * r.l[j][k];
                }
                assert!((s - a[i][j]).abs() < 1e-9,
                    "L·Lᵀ mismatch at ({i},{j}): got {s} expected {}", a[i][j]);
            }
        }
    }

    #[test]
    fn upper_triangle_is_zero() {
        let a = vec![
            vec![4.0, 2.0],
            vec![2.0, 5.0],
        ];
        let r = decompose(&a).unwrap();
        assert_eq!(r.l[0][1], 0.0);
    }

    #[test]
    fn multiply_with_correlated_draws() {
        // 2-asset covariance σ² = (4, 9), ρ = 0.5.
        let a = vec![
            vec![4.0, 3.0],
            vec![3.0, 9.0],
        ];
        let r = decompose(&a).unwrap();
        let z = vec![1.0, 1.0];
        let out = multiply(&r.l, &z).unwrap();
        assert_eq!(out.len(), 2);
        assert!(out.iter().all(|x| x.is_finite()));
    }

    #[test]
    fn multiply_dim_mismatch_returns_none() {
        let l = vec![vec![1.0, 0.0], vec![0.5, 0.8]];
        assert!(multiply(&l, &[1.0]).is_none());
    }
}
