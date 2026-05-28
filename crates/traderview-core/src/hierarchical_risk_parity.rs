//! Hierarchical Risk Parity (HRP) — Marcos López de Prado (2016).
//!
//! Outperforms mean-variance and inverse-volatility on out-of-sample
//! data because it doesn't require inverting the covariance matrix
//! (which amplifies estimation noise). The procedure:
//!
//!   1. **Tree clustering** — single-linkage hierarchical clustering on
//!      the asset correlation distance d_ij = √((1 − ρ_ij) / 2).
//!   2. **Quasi-diagonalization** — reorder assets so the most-related
//!      pairs sit next to each other in the covariance matrix.
//!   3. **Recursive bisection** — split clusters in half and allocate
//!      capital inversely proportional to the variance of each half.
//!
//! Pure compute. Returns weights in original-asset order plus the
//! cluster ordering for diagnostics.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HrpReport {
    pub weights: Vec<f64>,
    pub cluster_order: Vec<usize>,
    pub portfolio_variance: f64,
}

pub fn solve(covariance: &[Vec<f64>]) -> Option<HrpReport> {
    let n = covariance.len();
    if n < 2 || covariance.iter().any(|r| r.len() != n) { return None; }
    if covariance.iter().any(|r| r.iter().any(|c| !c.is_finite())) { return None; }
    // Build correlation matrix.
    let stdev: Vec<f64> = (0..n).map(|i| covariance[i][i].max(0.0).sqrt()).collect();
    if stdev.iter().any(|s| *s <= 0.0) { return None; }
    let mut corr = vec![vec![0.0_f64; n]; n];
    for i in 0..n {
        for j in 0..n {
            corr[i][j] = (covariance[i][j] / (stdev[i] * stdev[j])).clamp(-1.0, 1.0);
        }
    }
    // Distance matrix d_ij = √((1 − ρ_ij) / 2).
    let mut dist = vec![vec![0.0_f64; n]; n];
    for i in 0..n {
        for j in 0..n {
            dist[i][j] = ((1.0 - corr[i][j]) / 2.0).max(0.0).sqrt();
        }
    }
    // Single-linkage clustering: build a quasi-diagonal ordering by
    // recursively bisecting at the most-distant pair.
    let cluster_order = quasi_diagonal_order(&dist);
    if cluster_order.len() != n { return None; }
    // Recursive bisection allocation.
    let mut weights = vec![1.0_f64; n];
    bisect_allocate(covariance, &cluster_order, 0, n, &mut weights);
    // Normalize.
    let total: f64 = weights.iter().sum();
    if total <= 0.0 { return None; }
    for w in weights.iter_mut() { *w /= total; }
    // Portfolio variance with HRP weights.
    let mut port_var = 0.0_f64;
    for i in 0..n {
        for j in 0..n {
            port_var += weights[i] * weights[j] * covariance[i][j];
        }
    }
    Some(HrpReport { weights, cluster_order, portfolio_variance: port_var })
}

fn quasi_diagonal_order(dist: &[Vec<f64>]) -> Vec<usize> {
    let n = dist.len();
    if n == 0 { return Vec::new(); }
    // Simple agglomerative single-linkage merge that returns a leaf
    // ordering. We track clusters as lists of indices and at each step
    // merge the two clusters with the smallest min-distance.
    let mut clusters: Vec<Vec<usize>> = (0..n).map(|i| vec![i]).collect();
    while clusters.len() > 1 {
        // Find the pair with minimum single-linkage distance.
        let mut best = (0_usize, 1_usize, f64::INFINITY);
        for i in 0..clusters.len() {
            for j in (i + 1)..clusters.len() {
                let mut d_min = f64::INFINITY;
                for &a in &clusters[i] {
                    for &b in &clusters[j] {
                        if dist[a][b] < d_min { d_min = dist[a][b]; }
                    }
                }
                if d_min < best.2 { best = (i, j, d_min); }
            }
        }
        let mut merged = clusters[best.0].clone();
        merged.extend(clusters[best.1].iter().copied());
        // Remove higher-index first.
        clusters.remove(best.1);
        clusters[best.0] = merged;
    }
    clusters.into_iter().next().unwrap_or_default()
}

fn bisect_allocate(cov: &[Vec<f64>], order: &[usize], lo: usize, hi: usize, weights: &mut [f64]) {
    if hi - lo <= 1 { return; }
    let mid = lo + (hi - lo) / 2;
    let left = &order[lo..mid];
    let right = &order[mid..hi];
    let var_l = cluster_variance(cov, left);
    let var_r = cluster_variance(cov, right);
    if var_l + var_r <= 0.0 { return; }
    let alpha = 1.0 - var_l / (var_l + var_r);    // smaller variance gets larger weight
    for &i in left { weights[i] *= alpha; }
    for &i in right { weights[i] *= 1.0 - alpha; }
    bisect_allocate(cov, order, lo, mid, weights);
    bisect_allocate(cov, order, mid, hi, weights);
}

fn cluster_variance(cov: &[Vec<f64>], cluster: &[usize]) -> f64 {
    let n = cluster.len();
    if n == 0 { return 0.0; }
    // Inverse-volatility weights within cluster as the seed weights.
    let inv_vols: Vec<f64> = cluster.iter()
        .map(|&i| 1.0 / cov[i][i].max(1e-18).sqrt())
        .collect();
    let total: f64 = inv_vols.iter().sum();
    let w: Vec<f64> = inv_vols.iter().map(|x| x / total).collect();
    let mut sum = 0.0_f64;
    for (a, &i) in cluster.iter().enumerate() {
        for (b, &j) in cluster.iter().enumerate() {
            sum += w[a] * w[b] * cov[i][j];
        }
    }
    sum
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_or_malformed_returns_none() {
        assert!(solve(&[]).is_none());
        let bad = vec![vec![1.0, 0.5]];
        assert!(solve(&bad).is_none());
        let nan = vec![vec![f64::NAN; 2]; 2];
        assert!(solve(&nan).is_none());
    }

    #[test]
    fn zero_variance_returns_none() {
        let bad = vec![vec![0.0, 0.0], vec![0.0, 0.0]];
        assert!(solve(&bad).is_none());
    }

    #[test]
    fn equal_uncorrelated_assets_yield_equal_weights() {
        let cov = vec![
            vec![0.04, 0.0, 0.0, 0.0],
            vec![0.0, 0.04, 0.0, 0.0],
            vec![0.0, 0.0, 0.04, 0.0],
            vec![0.0, 0.0, 0.0, 0.04],
        ];
        let r = solve(&cov).unwrap();
        for w in &r.weights {
            assert!((w - 0.25).abs() < 0.01,
                "expected equal weights, got {:?}", r.weights);
        }
    }

    #[test]
    fn higher_vol_asset_gets_lower_weight() {
        let cov = vec![
            vec![0.01, 0.0, 0.0],
            vec![0.0, 0.04, 0.0],
            vec![0.0, 0.0, 0.16],
        ];
        let r = solve(&cov).unwrap();
        // Lower vol asset = larger weight.
        assert!(r.weights[0] > r.weights[1]);
        assert!(r.weights[1] > r.weights[2]);
    }

    #[test]
    fn weights_sum_to_one() {
        let cov = vec![
            vec![0.04, 0.01, 0.005, 0.0],
            vec![0.01, 0.09, 0.02, 0.0],
            vec![0.005, 0.02, 0.16, 0.01],
            vec![0.0, 0.0, 0.01, 0.25],
        ];
        let r = solve(&cov).unwrap();
        let sum: f64 = r.weights.iter().sum();
        assert!((sum - 1.0).abs() < 1e-9);
    }

    #[test]
    fn cluster_order_contains_all_indices() {
        let cov = vec![
            vec![0.04, 0.01],
            vec![0.01, 0.09],
        ];
        let r = solve(&cov).unwrap();
        assert_eq!(r.cluster_order.len(), 2);
        let mut sorted = r.cluster_order.clone();
        sorted.sort();
        assert_eq!(sorted, vec![0, 1]);
    }

    #[test]
    fn portfolio_variance_finite_and_positive() {
        let cov = vec![
            vec![0.04, 0.01, 0.005],
            vec![0.01, 0.09, 0.02],
            vec![0.005, 0.02, 0.16],
        ];
        let r = solve(&cov).unwrap();
        assert!(r.portfolio_variance > 0.0);
    }
}
