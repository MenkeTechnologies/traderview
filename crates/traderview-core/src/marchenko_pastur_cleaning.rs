//! Marchenko-Pastur Eigenvalue Clipping — denoises a sample covariance
//! matrix by replacing in-noise-bulk eigenvalues with their average,
//! preserving the top "signal" eigenvalues and the trace.
//!
//! Theory: for IID white noise with T observations and N variables and
//! ratio q = N/T, sample-cov eigenvalues fall inside the Marchenko-
//! Pastur bulk [λ_min, λ_max]:
//!
//!   λ_max = σ² · (1 + √q)²
//!   λ_min = σ² · (1 - √q)²
//!
//! Any eigenvalue above λ_max is signal. Eigenvalues inside the bulk
//! are noise — replace them all with their average (preserves trace).
//!
//! Steps:
//!   1. Eigendecompose Σ_sample = U Λ Uᵀ (Jacobi).
//!   2. Estimate σ² as mean of bulk eigenvalues (initial guess: trace/N).
//!      Refine 1 iteration: average eigenvalues ≤ λ_max(σ²).
//!   3. Replace bulk eigenvalues with their average.
//!   4. Reconstruct: Σ_clean = U Λ_clean Uᵀ.
//!
//! Reports the cleaned covariance plus how many eigenvalues were kept
//! as signal vs replaced by bulk.
//!
//! Pure compute. Companion to `ledoit_wolf`, `min_variance_portfolio`,
//! `hierarchical_risk_parity`.

#![allow(clippy::needless_range_loop)]

#[derive(Debug)]
pub struct Report {
    pub cleaned_covariance: Vec<Vec<f64>>,
    pub eigenvalues_signal: Vec<f64>,
    pub bulk_eigenvalue_avg: f64,
    pub signal_count: usize,
    pub bulk_count: usize,
    pub lambda_max: f64,
}

pub fn compute(cov: &[Vec<f64>], num_observations: usize) -> Option<Report> {
    let n = cov.len();
    if n < 2 || num_observations < n { return None; }
    if cov.iter().any(|row| row.len() != n) { return None; }
    if cov.iter().any(|row| row.iter().any(|v| !v.is_finite())) { return None; }
    let (eigenvalues, u) = jacobi_eigen(cov, 200);
    // First pass: estimate σ² from trace.
    let trace: f64 = eigenvalues.iter().map(|v| v.max(0.0)).sum();
    let mut sigma_sq = trace / n as f64;
    let q = n as f64 / num_observations as f64;
    let one_plus_sqrt_q = 1.0 + q.sqrt();
    // One refinement pass: σ² = avg of bulk eigenvalues.
    for _ in 0..2 {
        let lambda_max = sigma_sq * one_plus_sqrt_q * one_plus_sqrt_q;
        let bulk: Vec<f64> = eigenvalues.iter()
            .filter(|&&v| v <= lambda_max && v >= 0.0)
            .copied().collect();
        if bulk.is_empty() { break; }
        sigma_sq = bulk.iter().sum::<f64>() / bulk.len() as f64;
    }
    let lambda_max = sigma_sq * one_plus_sqrt_q * one_plus_sqrt_q;
    // Classify each eigenvalue.
    let mut bulk_indices = Vec::new();
    let mut signal_eigenvalues = Vec::new();
    for (i, &lam) in eigenvalues.iter().enumerate() {
        if lam > lambda_max {
            signal_eigenvalues.push(lam);
        } else {
            bulk_indices.push(i);
        }
    }
    let bulk_avg = if bulk_indices.is_empty() {
        sigma_sq
    } else {
        let s: f64 = bulk_indices.iter().map(|&i| eigenvalues[i].max(0.0)).sum();
        s / bulk_indices.len() as f64
    };
    // Build cleaned eigenvalues, then reconstruct.
    let mut lambda_clean = eigenvalues.clone();
    for &i in &bulk_indices { lambda_clean[i] = bulk_avg; }
    // Σ_clean[i][j] = Σ_k u[i][k] · λ_clean[k] · u[j][k]
    let mut cleaned = vec![vec![0.0_f64; n]; n];
    for i in 0..n {
        for j in i..n {
            let mut s = 0.0;
            for k in 0..n {
                s += u[i][k] * lambda_clean[k] * u[j][k];
            }
            cleaned[i][j] = s;
            cleaned[j][i] = s;
        }
    }
    // Sort signal eigenvalues descending for reporting.
    signal_eigenvalues.sort_by(|a, b| b.partial_cmp(a)
        .unwrap_or(std::cmp::Ordering::Equal));
    Some(Report {
        cleaned_covariance: cleaned,
        signal_count: signal_eigenvalues.len(),
        bulk_count: bulk_indices.len(),
        eigenvalues_signal: signal_eigenvalues,
        bulk_eigenvalue_avg: bulk_avg,
        lambda_max,
    })
}

fn jacobi_eigen(matrix: &[Vec<f64>], max_iter: usize) -> (Vec<f64>, Vec<Vec<f64>>) {
    let n = matrix.len();
    let mut a: Vec<Vec<f64>> = matrix.to_vec();
    let mut v: Vec<Vec<f64>> = (0..n).map(|i|
        (0..n).map(|j| if i == j { 1.0 } else { 0.0 }).collect()
    ).collect();
    for _ in 0..max_iter {
        let mut p = 0; let mut q = 1; let mut max = 0.0_f64;
        for i in 0..n {
            for j in i + 1..n {
                let abs_ij = a[i][j].abs();
                if abs_ij > max { max = abs_ij; p = i; q = j; }
            }
        }
        if max < 1e-12 { break; }
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
        let m = vec![vec![1.0_f64, 0.0], vec![0.0, 1.0]];
        assert!(compute(&[], 100).is_none());
        let single = vec![vec![1.0_f64]];
        assert!(compute(&single, 100).is_none());
        assert!(compute(&m, 1).is_none());      // T < N
        let bad = vec![vec![1.0_f64, 0.0], vec![0.0]];
        assert!(compute(&bad, 100).is_none());
        let mut nan = m.clone();
        nan[0][0] = f64::NAN;
        assert!(compute(&nan, 100).is_none());
    }

    #[test]
    fn identity_cleans_to_identity() {
        let n = 5;
        let mut m = vec![vec![0.0_f64; n]; n];
        for i in 0..n { m[i][i] = 1.0; }
        let r = compute(&m, 500).unwrap();
        // All eigenvalues equal 1 — all in bulk, replaced by avg=1.
        for i in 0..n {
            for j in 0..n {
                let expected = if i == j { 1.0 } else { 0.0 };
                assert!((r.cleaned_covariance[i][j] - expected).abs() < 1e-9);
            }
        }
    }

    #[test]
    fn trace_preserved() {
        // Random-ish cov. Cleaning preserves trace (sum of eigenvalues).
        let m = vec![
            vec![0.04, 0.01, 0.005],
            vec![0.01, 0.09, 0.02],
            vec![0.005, 0.02, 0.16],
        ];
        let r = compute(&m, 200).unwrap();
        let original_trace: f64 = (0..3).map(|i| m[i][i]).sum();
        let cleaned_trace: f64 = (0..3).map(|i| r.cleaned_covariance[i][i]).sum();
        assert!((original_trace - cleaned_trace).abs() < 1e-9);
    }

    #[test]
    fn signal_eigenvalue_preserved() {
        // Build a cov with one large eigenvalue + small bulk.
        // Diagonal with one big variance and tiny others.
        let m = vec![
            vec![10.0, 0.0, 0.0, 0.0, 0.0],
            vec![0.0, 0.01, 0.0, 0.0, 0.0],
            vec![0.0, 0.0, 0.01, 0.0, 0.0],
            vec![0.0, 0.0, 0.0, 0.01, 0.0],
            vec![0.0, 0.0, 0.0, 0.0, 0.01],
        ];
        let r = compute(&m, 100).unwrap();
        assert_eq!(r.signal_count, 1);
        assert!((r.eigenvalues_signal[0] - 10.0).abs() < 1e-9);
    }

    #[test]
    fn signal_and_bulk_counts_sum_to_n() {
        let m = vec![
            vec![0.04, 0.01, 0.005],
            vec![0.01, 0.09, 0.02],
            vec![0.005, 0.02, 0.16],
        ];
        let r = compute(&m, 100).unwrap();
        assert_eq!(r.signal_count + r.bulk_count, 3);
    }

    #[test]
    fn cleaned_cov_is_symmetric() {
        let m = vec![
            vec![0.04, 0.01, 0.005],
            vec![0.01, 0.09, 0.02],
            vec![0.005, 0.02, 0.16],
        ];
        let r = compute(&m, 100).unwrap();
        for i in 0..3 {
            for j in 0..3 {
                assert!((r.cleaned_covariance[i][j] - r.cleaned_covariance[j][i]).abs() < 1e-9);
            }
        }
    }
}
