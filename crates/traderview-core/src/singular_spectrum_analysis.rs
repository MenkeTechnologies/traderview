//! Singular Spectrum Analysis (Broomhead-King, Vautard-Ghil) — model-
//! free decomposition of a time series into trend, oscillations, and
//! noise via SVD of the trajectory matrix.
//!
//! Steps:
//!   1. **Embedding**. Form trajectory matrix X (L × K) from sliding
//!      windows of length L over series of length N (K = N - L + 1).
//!   2. **SVD**. Decompose X = Σ_i √λ_i · u_i · v_iᵀ.
//!   3. **Grouping**. Partition indices into trend (largest λ) and
//!      noise (small λ). Typical: top 1 → trend, next pairs → cycles.
//!   4. **Diagonal averaging**. Reconstruct each group's component as a
//!      length-N series by anti-diagonal averaging.
//!
//! Reports the trend (first principal component) and noise (sum of
//! remaining components) reconstructions, plus singular values for
//! eyeballing the spectrum.
//!
//! Naive SVD via Jacobi rotations on the L×L covariance matrix XXᵀ.
//! For series with L > ~100, an iterative SVD would be faster; here
//! we keep L modest.
//!
//! Pure compute. Companion to `wavelet_decomposition_haar`,
//! `kalman_filter_1d`, `savitzky_golay`.

#![allow(clippy::needless_range_loop)]

#[derive(Debug)]
pub struct Report {
    pub trend: Vec<f64>,
    pub noise: Vec<f64>,
    pub singular_values: Vec<f64>,
}

pub fn compute(series: &[f64], window: usize) -> Option<Report> {
    let n = series.len();
    if !(2..=100).contains(&window) || n < 2 * window {
        return None;
    }
    if series.iter().any(|x| !x.is_finite()) {
        return None;
    }
    let l = window;
    let k = n - l + 1;
    // Build XXᵀ (L × L). XXᵀ[i][j] = Σ_t x_{i+t} · x_{j+t} for t = 0..K.
    let mut c = vec![vec![0.0_f64; l]; l];
    for i in 0..l {
        for j in i..l {
            let mut s = 0.0;
            for t in 0..k {
                s += series[i + t] * series[j + t];
            }
            c[i][j] = s;
            c[j][i] = s;
        }
    }
    // Eigen-decompose C via Jacobi rotations: C = U Λ Uᵀ.
    let (eigenvalues, u) = jacobi_eigen(&c, 200);
    // Pair (eigenvalue, eigenvector-column).
    let mut order: Vec<usize> = (0..l).collect();
    order.sort_by(|&a, &b| {
        eigenvalues[b]
            .partial_cmp(&eigenvalues[a])
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let sorted_lambdas: Vec<f64> = order.iter().map(|&i| eigenvalues[i].max(0.0)).collect();
    let singular_values: Vec<f64> = sorted_lambdas.iter().map(|&v| v.sqrt()).collect();
    // For each component, the reconstruction is the rank-1 matrix
    //   X_i = u_i · (Xᵀ u_i)ᵀ
    // diagonally averaged back to a length-N series.
    let trend_idx = order[0];
    let trend = reconstruct(series, l, k, &u, trend_idx);
    // Noise = original series - sum of all reconstructions kept as
    // trend. For minimal complexity, take the residual after trend.
    let noise: Vec<f64> = series
        .iter()
        .zip(trend.iter())
        .map(|(s, t)| s - t)
        .collect();
    Some(Report {
        trend,
        noise,
        singular_values,
    })
}

fn jacobi_eigen(matrix: &[Vec<f64>], max_iter: usize) -> (Vec<f64>, Vec<Vec<f64>>) {
    let n = matrix.len();
    let mut a: Vec<Vec<f64>> = matrix.to_vec();
    let mut v: Vec<Vec<f64>> = (0..n)
        .map(|i| (0..n).map(|j| if i == j { 1.0 } else { 0.0 }).collect())
        .collect();
    for _ in 0..max_iter {
        // Find largest off-diagonal.
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

/// Reconstruct one component via diagonal averaging.
fn reconstruct(series: &[f64], l: usize, k: usize, u: &[Vec<f64>], col: usize) -> Vec<f64> {
    let n = series.len();
    // First compute v_i = Xᵀ u_i / σ_i but normalized. For a rank-1
    // reconstruction X_i = (Xᵀ u_i) · u_iᵀ is enough; we just need the
    // outer product divided so it sums correctly. Use the projection:
    //   X_i[r][c] = u[r] · (Σ_t u[t] · x_{t+c})
    let proj: Vec<f64> = (0..k)
        .map(|c| {
            let mut s = 0.0;
            for t in 0..l {
                s += u[t][col] * series[t + c];
            }
            s
        })
        .collect();
    let mut out = vec![0.0_f64; n];
    let mut counts = vec![0_usize; n];
    for r in 0..l {
        for c in 0..k {
            let idx = r + c;
            out[idx] += u[r][col] * proj[c];
            counts[idx] += 1;
        }
    }
    for i in 0..n {
        if counts[i] > 0 {
            out[i] /= counts[i] as f64;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_none() {
        let s = vec![1.0_f64; 50];
        assert!(compute(&s, 1).is_none());
        assert!(compute(&s, 150).is_none());
        assert!(compute(&s, 30).is_none()); // 2L > N
        let mut s_nan = s.clone();
        s_nan[5] = f64::NAN;
        assert!(compute(&s_nan, 10).is_none());
    }

    #[test]
    fn linear_trend_recovered() {
        // y = 100 + 0.5 · t → trend dominates first component.
        let s: Vec<f64> = (0..100).map(|i| 100.0 + 0.5 * i as f64).collect();
        let r = compute(&s, 20).unwrap();
        // Trend reconstruction should track input closely.
        let resid: f64 = s
            .iter()
            .zip(r.trend.iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum();
        let energy: f64 = s.iter().map(|x| x * x).sum();
        assert!(resid / energy < 0.01);
    }

    #[test]
    fn singular_values_descending() {
        let s: Vec<f64> = (0..100)
            .map(|i| (i as f64 * 0.1).sin() + 0.01 * i as f64)
            .collect();
        let r = compute(&s, 15).unwrap();
        for w in r.singular_values.windows(2) {
            assert!(w[0] >= w[1] - 1e-9);
        }
    }

    #[test]
    fn trend_plus_noise_equals_original() {
        let s: Vec<f64> = (0..100).map(|i| 100.0 + (i as f64 * 0.2).sin()).collect();
        let r = compute(&s, 20).unwrap();
        for i in 0..100 {
            assert!((s[i] - r.trend[i] - r.noise[i]).abs() < 1e-9);
        }
    }

    #[test]
    fn output_lengths_match_input() {
        let s: Vec<f64> = (0..100).map(|i| (i as f64 * 0.3).sin()).collect();
        let r = compute(&s, 20).unwrap();
        assert_eq!(r.trend.len(), 100);
        assert_eq!(r.noise.len(), 100);
        assert_eq!(r.singular_values.len(), 20);
    }

    #[test]
    fn flat_series_yields_flat_trend() {
        let s = vec![5.0_f64; 100];
        let r = compute(&s, 15).unwrap();
        for v in &r.trend {
            assert!((v - 5.0).abs() < 1e-9);
        }
    }
}
