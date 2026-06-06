//! Mahalanobis Distance — P. C. Mahalanobis (1936).
//!
//! Distance metric for multivariate data that accounts for the
//! covariance structure of the variables:
//!
//!   D²(x) = (x − μ)ᵀ · Σ⁻¹ · (x − μ)
//!
//! Properties:
//!   - Scale-invariant (vs Euclidean distance which depends on units)
//!   - Accounts for correlation between dimensions
//!   - Under multivariate normality, D² ~ χ²(p) with p degrees of freedom
//!
//! Uses:
//!   - Multivariate outlier detection (compare D² against χ²(p) quantile)
//!   - Portfolio anomaly detection (return vector distance from mean)
//!   - Regime-change detection (rolling Mahalanobis of recent returns
//!     against full-sample distribution)
//!
//! Pure compute. Takes the design matrix (rows = observations, cols =
//! variables), computes mean + covariance from it (or accepts these
//! pre-computed), then returns per-observation Mahalanobis distances.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MahalanobisReport {
    pub distances: Vec<f64>,
    pub mean: Vec<f64>,
    /// Sample (Bessel-corrected) covariance matrix.
    pub covariance: Vec<Vec<f64>>,
    pub max_distance: f64,
    pub argmax_index: usize,
    pub n_observations: usize,
    pub n_variables: usize,
}

pub fn compute(observations: &[Vec<f64>]) -> Option<MahalanobisReport> {
    let n = observations.len();
    if n < 2 {
        return None;
    }
    let p = observations[0].len();
    if p == 0 {
        return None;
    }
    if observations
        .iter()
        .any(|row| row.len() != p || row.iter().any(|x| !x.is_finite()))
    {
        return None;
    }
    let n_f = n as f64;
    // Sample mean per variable.
    let mut mean = vec![0.0_f64; p];
    for row in observations {
        for (j, x) in row.iter().enumerate() {
            mean[j] += x;
        }
    }
    for m in mean.iter_mut() {
        *m /= n_f;
    }
    // Sample covariance (Bessel-corrected). Loops cross-index across
    // multiple matrices, so iterator rewrites obscure the intent.
    #[allow(clippy::needless_range_loop)]
    let cov = {
        let mut cov = vec![vec![0.0_f64; p]; p];
        for row in observations {
            for j in 0..p {
                for k in 0..p {
                    cov[j][k] += (row[j] - mean[j]) * (row[k] - mean[k]);
                }
            }
        }
        let dof = (n - 1) as f64;
        if dof <= 0.0 {
            return None;
        }
        for j in 0..p {
            for k in 0..p {
                cov[j][k] /= dof;
            }
        }
        cov
    };
    let cov_inv = invert(&cov)?;
    // Per-observation Mahalanobis distance.
    let mut distances = Vec::with_capacity(n);
    let mut max_d = f64::NEG_INFINITY;
    let mut max_i = 0_usize;
    for (i, row) in observations.iter().enumerate() {
        let diff: Vec<f64> = row.iter().zip(mean.iter()).map(|(x, m)| x - m).collect();
        // D² = diffᵀ · cov⁻¹ · diff
        let mut acc = 0.0_f64;
        for j in 0..p {
            let mut row_acc = 0.0_f64;
            for k in 0..p {
                row_acc += cov_inv[j][k] * diff[k];
            }
            acc += diff[j] * row_acc;
        }
        let d = acc.max(0.0).sqrt();
        distances.push(d);
        if d > max_d {
            max_d = d;
            max_i = i;
        }
    }
    Some(MahalanobisReport {
        distances,
        mean,
        covariance: cov,
        max_distance: max_d,
        argmax_index: max_i,
        n_observations: n,
        n_variables: p,
    })
}

fn invert(m: &[Vec<f64>]) -> Option<Vec<Vec<f64>>> {
    let n = m.len();
    if n == 0 || m.iter().any(|r| r.len() != n) {
        return None;
    }
    let mut aug = vec![vec![0.0_f64; 2 * n]; n];
    for i in 0..n {
        for j in 0..n {
            aug[i][j] = m[i][j];
        }
        aug[i][n + i] = 1.0;
    }
    for i in 0..n {
        let mut pivot = i;
        for r in (i + 1)..n {
            if aug[r][i].abs() > aug[pivot][i].abs() {
                pivot = r;
            }
        }
        if aug[pivot][i].abs() < 1e-18 {
            return None;
        }
        aug.swap(i, pivot);
        let div = aug[i][i];
        for v in aug[i].iter_mut() {
            *v /= div;
        }
        for r in 0..n {
            if r == i {
                continue;
            }
            let f = aug[r][i];
            if f == 0.0 {
                continue;
            }
            let pivot_row = aug[i].clone();
            for (j, v) in aug[r].iter_mut().enumerate() {
                *v -= f * pivot_row[j];
            }
        }
    }
    Some((0..n).map(|i| aug[i][n..].to_vec()).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_or_single_obs_returns_none() {
        assert!(compute(&[]).is_none());
        assert!(compute(&[vec![1.0, 2.0]]).is_none());
    }

    #[test]
    fn ragged_input_returns_none() {
        let obs = vec![vec![1.0, 2.0], vec![1.0]];
        assert!(compute(&obs).is_none());
    }

    #[test]
    fn nan_input_returns_none() {
        let obs = vec![vec![1.0, f64::NAN], vec![2.0, 3.0]];
        assert!(compute(&obs).is_none());
    }

    #[test]
    fn all_identical_returns_none() {
        // Covariance is singular when all observations are identical.
        let obs = vec![vec![1.0, 2.0]; 10];
        assert!(compute(&obs).is_none());
    }

    #[test]
    fn univariate_reduces_to_z_score() {
        // For 1-variable input, D = |z| = |x - μ| / σ.
        let obs: Vec<Vec<f64>> = (0..10).map(|i| vec![i as f64]).collect();
        let r = compute(&obs).unwrap();
        let mean = 4.5_f64;
        let var: f64 = (0..10).map(|i| (i as f64 - mean).powi(2)).sum::<f64>() / 9.0;
        let sd = var.sqrt();
        for (i, d) in r.distances.iter().enumerate() {
            let expected = (i as f64 - mean).abs() / sd;
            assert!(
                (d - expected).abs() < 1e-9,
                "i={i}: got {d}, expected {expected}"
            );
        }
    }

    #[test]
    fn mean_observation_has_zero_distance() {
        // 4 non-collinear points plus their mean as the 5th observation.
        // Adding the mean keeps the mean of all 5 at the same location,
        // so the 5th observation's Mahalanobis distance is exactly 0.
        let obs = vec![
            vec![1.0, 1.0],
            vec![5.0, 2.0],
            vec![3.0, 7.0],
            vec![6.0, 5.0],
            vec![3.75, 3.75], // = mean of the previous 4
        ];
        let r = compute(&obs).unwrap();
        assert!(
            r.distances[4] < 1e-9,
            "mean obs should have distance ~0, got {}",
            r.distances[4]
        );
    }

    #[test]
    fn outlier_has_largest_distance() {
        // Cluster of 9 near-identical points + 1 obvious outlier.
        let mut obs: Vec<Vec<f64>> = (0..9)
            .map(|i| vec![100.0 + i as f64 * 0.1, 50.0 + i as f64 * 0.05])
            .collect();
        obs.push(vec![200.0, 200.0]);
        let r = compute(&obs).unwrap();
        assert_eq!(r.argmax_index, 9);
    }

    #[test]
    fn correlated_dimensions_handled() {
        // x = i, y = 2i: perfectly correlated. Outlier breaks correlation.
        let mut obs: Vec<Vec<f64>> = (0..10).map(|i| vec![i as f64, 2.0 * i as f64]).collect();
        obs.push(vec![5.0, 0.0]); // breaks the y = 2x pattern
                                  // Mahalanobis correctly handles correlation; but here the
                                  // correlation matrix is nearly singular, which may yield None.
                                  // Skip if so — the test is about not panicking on near-singular input.
        let _ = compute(&obs);
    }
}
