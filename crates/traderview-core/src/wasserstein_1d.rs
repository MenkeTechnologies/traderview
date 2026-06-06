//! 1-D Wasserstein (Earth-Mover) Distance.
//!
//! The Wasserstein-1 distance between two 1D probability distributions
//! is the L¹ distance between their cumulative distribution functions:
//!
//!   W_1(P, Q) = ∫ |F_P(x) − F_Q(x)| dx
//!
//! For empirical samples X = {x_1, …, x_m} and Y = {y_1, …, y_n}
//! sorted ascending, this admits the simple form:
//!
//!   W_1(X, Y) = Σ_k (z_{k+1} − z_k) · |F̂_X(z_k) − F̂_Y(z_k)|
//!
//! where z is the merged sorted union of X and Y.
//!
//! Properties:
//!   - Metric on the space of probability measures with finite mean
//!   - Naturally generalizes to optimal transport in higher dimensions
//!   - More forgiving than KL divergence (well-defined when supports
//!     differ, doesn't blow up to infinity)
//!
//! Use cases:
//!   - Distance between empirical return distributions of two strategies
//!   - Distribution drift detection between training and live data
//!   - Calibration of model-implied vs empirical distributions
//!
//! Pure compute. Companion to `kullback_leibler_divergence`,
//! `kolmogorov_smirnov_2sample`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WassersteinReport {
    pub wasserstein_1: f64,
    pub n_sample_a: usize,
    pub n_sample_b: usize,
}

pub fn compute(sample_a: &[f64], sample_b: &[f64]) -> Option<WassersteinReport> {
    let m = sample_a.len();
    let n = sample_b.len();
    if m == 0 || n == 0 {
        return None;
    }
    if sample_a.iter().any(|x| !x.is_finite()) || sample_b.iter().any(|x| !x.is_finite()) {
        return None;
    }
    let mut a = sample_a.to_vec();
    let mut b = sample_b.to_vec();
    a.sort_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal));
    b.sort_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal));
    // Merge-sort over the union; track per-side cumulative weights.
    let mut i = 0_usize;
    let mut j = 0_usize;
    let mut prev = a[0].min(b[0]);
    let mut f_a = 0.0_f64;
    let mut f_b = 0.0_f64;
    let m_f = m as f64;
    let n_f = n as f64;
    let mut acc = 0.0_f64;
    while i < m || j < n {
        let next = if i == m {
            b[j]
        } else if j == n {
            a[i]
        } else {
            a[i].min(b[j])
        };
        acc += (next - prev) * (f_a - f_b).abs();
        // Advance pointer(s) at this step's value.
        if i < m && a[i] == next {
            i += 1;
            f_a = i as f64 / m_f;
        }
        if j < n && b[j] == next {
            j += 1;
            f_b = j as f64 / n_f;
        }
        prev = next;
    }
    Some(WassersteinReport {
        wasserstein_1: acc,
        n_sample_a: m,
        n_sample_b: n,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_none() {
        assert!(compute(&[], &[1.0, 2.0]).is_none());
        assert!(compute(&[1.0, 2.0], &[]).is_none());
    }

    #[test]
    fn nan_returns_none() {
        assert!(compute(&[1.0, f64::NAN], &[1.0, 2.0]).is_none());
    }

    #[test]
    fn identical_samples_yield_zero_distance() {
        let s = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let r = compute(&s, &s).unwrap();
        assert!(r.wasserstein_1.abs() < 1e-12);
    }

    #[test]
    fn shifted_samples_yield_distance_equal_to_shift() {
        // Both have the same SHAPE, just shifted by 5.
        let a: Vec<f64> = (1..=10).map(|i| i as f64).collect();
        let b: Vec<f64> = (6..=15).map(|i| i as f64).collect();
        let r = compute(&a, &b).unwrap();
        assert!(
            (r.wasserstein_1 - 5.0).abs() < 1e-9,
            "shifted by 5 should yield W₁ ≈ 5, got {}",
            r.wasserstein_1
        );
    }

    #[test]
    fn distance_is_symmetric() {
        let a = vec![1.0, 3.0, 5.0, 7.0];
        let b = vec![2.0, 4.0, 6.0, 8.0];
        let r_ab = compute(&a, &b).unwrap();
        let r_ba = compute(&b, &a).unwrap();
        assert!((r_ab.wasserstein_1 - r_ba.wasserstein_1).abs() < 1e-12);
    }

    #[test]
    fn distance_non_negative_for_all_pairs() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![10.0, 20.0, 30.0];
        let r = compute(&a, &b).unwrap();
        assert!(r.wasserstein_1 >= 0.0);
    }

    #[test]
    fn farther_samples_yield_larger_distance() {
        let a = vec![1.0_f64; 10];
        let near = vec![2.0_f64; 10];
        let far = vec![100.0_f64; 10];
        let r_near = compute(&a, &near).unwrap();
        let r_far = compute(&a, &far).unwrap();
        assert!(r_far.wasserstein_1 > r_near.wasserstein_1);
    }

    #[test]
    fn unequal_sample_sizes_supported() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let r = compute(&a, &b).unwrap();
        assert_eq!(r.n_sample_a, 3);
        assert_eq!(r.n_sample_b, 6);
        assert!(r.wasserstein_1 >= 0.0);
    }
}
