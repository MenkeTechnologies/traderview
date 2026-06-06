//! Principal-Component Yield-Curve Decomposition (Litterman-Scheinkman
//! 1991) — extracts level, slope, and curvature factors from a time
//! series of yield curves via PCA on first-difference yield changes.
//!
//! Classic empirical result: across G7 sovereign curves, the first 3
//! principal components explain >95% of variance:
//!   PC1 (~80%) ≈ level   (parallel shift)
//!   PC2 (~10%) ≈ slope   (steepening / flattening)
//!   PC3 (~3%)  ≈ curvature (butterfly / twist)
//!
//! Steps:
//!   1. Build `n × t` matrix of yield-change vectors over T periods
//!      at N tenors.
//!   2. Center each tenor column.
//!   3. Compute covariance C = (1/T) · ΔY · ΔYᵀ.
//!   4. Jacobi eigendecomposition → eigenvalues (variance per factor)
//!      and eigenvectors (loadings: each column is one factor's
//!      tenor-loadings vector).
//!   5. Variance-explained = λ_i / Σ λ.
//!
//! Returns the top-k factors (loadings + variance explained).
//!
//! Pure compute. Companion to `nelson_siegel_svensson`, `pca`,
//! `key_rate_duration`.

#![allow(clippy::needless_range_loop)]

#[derive(Debug)]
pub struct Report {
    pub loadings: Vec<Vec<f64>>,
    pub variance_explained: Vec<f64>,
    pub cumulative_variance: Vec<f64>,
    pub eigenvalues: Vec<f64>,
}

pub fn compute(curves: &[Vec<f64>], top_k: usize) -> Option<Report> {
    let t = curves.len();
    if t < 5 {
        return None;
    }
    let n = curves[0].len();
    if n < 2 || top_k == 0 || top_k > n {
        return None;
    }
    if curves.iter().any(|row| row.len() != n) {
        return None;
    }
    if curves.iter().any(|row| row.iter().any(|v| !v.is_finite())) {
        return None;
    }
    // Build yield-change matrix: ΔY_{t,k} = curves[t+1][k] - curves[t][k].
    let m = t - 1;
    let mut dy = vec![vec![0.0_f64; n]; m];
    for ti in 0..m {
        for k in 0..n {
            dy[ti][k] = curves[ti + 1][k] - curves[ti][k];
        }
    }
    // Center per-tenor (subtract column mean).
    let mut means = vec![0.0_f64; n];
    for k in 0..n {
        let s: f64 = (0..m).map(|ti| dy[ti][k]).sum();
        means[k] = s / m as f64;
    }
    for ti in 0..m {
        for k in 0..n {
            dy[ti][k] -= means[k];
        }
    }
    // Covariance matrix C[i][j] = (1/m) Σ_t dy[t][i] · dy[t][j].
    let mut cov = vec![vec![0.0_f64; n]; n];
    for i in 0..n {
        for j in i..n {
            let s: f64 = (0..m).map(|ti| dy[ti][i] * dy[ti][j]).sum();
            cov[i][j] = s / m as f64;
            cov[j][i] = cov[i][j];
        }
    }
    let (eigenvalues, u) = jacobi_eigen(&cov, 200);
    // Sort by descending eigenvalue.
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&a, &b| {
        eigenvalues[b]
            .partial_cmp(&eigenvalues[a])
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let total: f64 = eigenvalues
        .iter()
        .map(|v| v.max(0.0))
        .sum::<f64>()
        .max(1e-18);
    let mut loadings = Vec::with_capacity(top_k);
    let mut variance_explained = Vec::with_capacity(top_k);
    let mut cumulative_variance = Vec::with_capacity(top_k);
    let mut cum = 0.0_f64;
    let mut sorted_eigen = Vec::with_capacity(top_k);
    for &idx in order.iter().take(top_k) {
        let factor: Vec<f64> = (0..n).map(|i| u[i][idx]).collect();
        let lambda = eigenvalues[idx].max(0.0);
        sorted_eigen.push(lambda);
        let frac = lambda / total;
        cum += frac;
        loadings.push(factor);
        variance_explained.push(frac);
        cumulative_variance.push(cum);
    }
    Some(Report {
        loadings,
        variance_explained,
        cumulative_variance,
        eigenvalues: sorted_eigen,
    })
}

fn jacobi_eigen(matrix: &[Vec<f64>], max_iter: usize) -> (Vec<f64>, Vec<Vec<f64>>) {
    let n = matrix.len();
    let mut a: Vec<Vec<f64>> = matrix.to_vec();
    let mut v: Vec<Vec<f64>> = (0..n)
        .map(|i| (0..n).map(|j| if i == j { 1.0 } else { 0.0 }).collect())
        .collect();
    for _ in 0..max_iter {
        let mut p = 0;
        let mut q = 1;
        let mut max = 0.0_f64;
        for i in 0..n {
            for j in i + 1..n {
                let abs_ij = a[i][j].abs();
                if abs_ij > max {
                    max = abs_ij;
                    p = i;
                    q = j;
                }
            }
        }
        if max < 1e-12 {
            break;
        }
        let app = a[p][p];
        let aqq = a[q][q];
        let apq = a[p][q];
        let theta = (aqq - app) / (2.0 * apq);
        let t = if theta >= 0.0 {
            1.0 / (theta + (1.0 + theta * theta).sqrt())
        } else {
            1.0 / (theta - (1.0 + theta * theta).sqrt())
        };
        let c = 1.0 / (1.0 + t * t).sqrt();
        let s = t * c;
        for i in 0..n {
            let aip = a[i][p];
            let aiq = a[i][q];
            a[i][p] = c * aip - s * aiq;
            a[i][q] = s * aip + c * aiq;
            let vip = v[i][p];
            let viq = v[i][q];
            v[i][p] = c * vip - s * viq;
            v[i][q] = s * vip + c * viq;
        }
        for j in 0..n {
            let apj = a[p][j];
            let aqj = a[q][j];
            a[p][j] = c * apj - s * aqj;
            a[q][j] = s * apj + c * aqj;
        }
    }
    let eigenvalues: Vec<f64> = (0..n).map(|i| a[i][i]).collect();
    (eigenvalues, v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_none() {
        let c = vec![vec![1.0, 2.0]; 10];
        assert!(compute(&c, 0).is_none());
        assert!(compute(&c, 5).is_none()); // top_k > n
        let short = vec![vec![1.0, 2.0]; 3];
        assert!(compute(&short, 1).is_none());
        let bad_shape = vec![vec![1.0, 2.0], vec![1.0]];
        assert!(compute(&bad_shape, 1).is_none());
        let mut bad = vec![vec![1.0, 2.0]; 10];
        bad[0][0] = f64::NAN;
        assert!(compute(&bad, 1).is_none());
    }

    #[test]
    fn parallel_shifts_dominate_first_pc() {
        // Each period: parallel shift across all tenors.
        let mut c = Vec::new();
        let base = [2.0_f64, 2.5, 3.0, 3.2, 3.5];
        for t in 0..50 {
            let shift = (t as f64 * 0.1).sin() * 0.5;
            c.push(base.iter().map(|y| y + shift).collect());
        }
        let r = compute(&c, 3).unwrap();
        // First factor should explain almost all variance.
        assert!(r.variance_explained[0] > 0.95);
        // And first factor's loadings should all be same sign (parallel shift).
        let signs: Vec<f64> = r.loadings[0].iter().map(|x| x.signum()).collect();
        assert!(signs.iter().all(|&s| s == signs[0]));
    }

    #[test]
    fn variance_explained_sums_to_one() {
        let mut c = Vec::new();
        for t in 0..50 {
            c.push(vec![
                2.0 + (t as f64 * 0.1).sin() * 0.3,
                2.5 + (t as f64 * 0.2).cos() * 0.4,
                3.0 + (t as f64 * 0.05).sin() * 0.2,
                3.5,
            ]);
        }
        let r = compute(&c, 4).unwrap();
        let sum: f64 = r.variance_explained.iter().sum();
        assert!((sum - 1.0).abs() < 1e-6);
    }

    #[test]
    fn variance_explained_monotone_decreasing() {
        let mut c = Vec::new();
        for t in 0..30 {
            c.push(vec![
                (t as f64 * 0.1).sin(),
                (t as f64 * 0.15).cos(),
                (t as f64 * 0.2).sin(),
            ]);
        }
        let r = compute(&c, 3).unwrap();
        for w in r.variance_explained.windows(2) {
            assert!(w[0] >= w[1] - 1e-9);
        }
    }

    #[test]
    fn cumulative_variance_reaches_one_at_full_rank() {
        let mut c = Vec::new();
        for t in 0..30 {
            c.push(vec![
                (t as f64 * 0.1).sin(),
                (t as f64 * 0.15).cos(),
                (t as f64 * 0.2).sin(),
            ]);
        }
        let r = compute(&c, 3).unwrap();
        assert!((r.cumulative_variance[2] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn output_dims_match_request() {
        let mut c = Vec::new();
        for _ in 0..30 {
            c.push(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        }
        // Need some variation for non-degenerate PCA.
        for t in 0..30 {
            for k in 0..5 {
                c[t][k] += (t as f64 * 0.1 + k as f64).sin() * 0.05;
            }
        }
        let r = compute(&c, 3).unwrap();
        assert_eq!(r.loadings.len(), 3);
        assert_eq!(r.loadings[0].len(), 5);
        assert_eq!(r.variance_explained.len(), 3);
    }
}
