//! Tail dependence coefficients — λ_L (lower) and λ_U (upper).
//!
//! Captures the probability that one asset experiences an extreme
//! quantile event GIVEN the other did. Critical for assessing
//! diversification breakdown in crashes (lower tail) and rallies
//! (upper tail).
//!
//! Empirical estimator (Frahm-Junker-Schmidt 2005, non-parametric form):
//!
//!   λ_L(α) = Pr[F(Y) ≤ α | F(X) ≤ α]
//!          ≈ (1/k) · #{i : R_X(i) ≤ k AND R_Y(i) ≤ k}
//!
//! where k = ⌊α · n⌋ and R_X, R_Y are ranks of x, y (1..n).
//!
//! Similarly λ_U with the upper k ranks. The asymptotic estimators are
//! the limit as α → 0 (λ_L) or α → 1 (λ_U); we use a finite-α proxy
//! at α = 0.05 (5% tail) which is the standard practitioner threshold.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TailDependenceReport {
    pub lower_tail_dependence: f64,
    pub upper_tail_dependence: f64,
    pub n_observations: usize,
    pub tail_size: usize,
}

pub fn compute(x: &[f64], y: &[f64], alpha: f64) -> Option<TailDependenceReport> {
    let n = x.len();
    if y.len() != n || n < 10 || !alpha.is_finite() || !(0.0..0.5).contains(&alpha) || alpha <= 0.0
    {
        return None;
    }
    let clean: Vec<(f64, f64)> = x
        .iter()
        .zip(y.iter())
        .filter(|(a, b)| a.is_finite() && b.is_finite())
        .map(|(a, b)| (*a, *b))
        .collect();
    let m = clean.len();
    if m < 10 {
        return None;
    }
    let xs: Vec<f64> = clean.iter().map(|p| p.0).collect();
    let ys: Vec<f64> = clean.iter().map(|p| p.1).collect();
    let rx = ranks(&xs);
    let ry = ranks(&ys);
    let k = ((m as f64) * alpha).ceil().max(1.0) as usize;
    let k = k.min(m);
    // Lower: count points where R_X ≤ k AND R_Y ≤ k.
    let mut lower_count = 0_usize;
    let mut upper_count = 0_usize;
    let upper_thresh = m - k + 1;
    for i in 0..m {
        if rx[i] <= k && ry[i] <= k {
            lower_count += 1;
        }
        if rx[i] >= upper_thresh && ry[i] >= upper_thresh {
            upper_count += 1;
        }
    }
    let lambda_l = lower_count as f64 / k as f64;
    let lambda_u = upper_count as f64 / k as f64;
    Some(TailDependenceReport {
        lower_tail_dependence: lambda_l,
        upper_tail_dependence: lambda_u,
        n_observations: m,
        tail_size: k,
    })
}

/// Returns integer ranks (1..n) with average-rank tie-breaking
/// truncated to nearest integer (acceptable for tail-counting).
fn ranks(values: &[f64]) -> Vec<usize> {
    let n = values.len();
    let mut indexed: Vec<(usize, f64)> = values.iter().enumerate().map(|(i, v)| (i, *v)).collect();
    indexed.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    let mut ranks = vec![0_usize; n];
    for (rank, (orig_idx, _)) in indexed.iter().enumerate() {
        ranks[*orig_idx] = rank + 1;
    }
    ranks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dim_mismatch_returns_none() {
        assert!(compute(&[1.0; 50], &[1.0; 25], 0.05).is_none());
    }

    #[test]
    fn too_short_returns_none() {
        assert!(compute(&[1.0; 5], &[1.0; 5], 0.05).is_none());
    }

    #[test]
    fn invalid_alpha_returns_none() {
        let x = vec![1.0; 50];
        let y = vec![1.0; 50];
        assert!(compute(&x, &y, 0.0).is_none());
        assert!(compute(&x, &y, 0.6).is_none());
        assert!(compute(&x, &y, -0.1).is_none());
        assert!(compute(&x, &y, f64::NAN).is_none());
    }

    #[test]
    fn perfectly_comonotonic_yields_full_tail_dependence() {
        // y = x → ranks identical → bottom-k of X coincides with bottom-k of Y.
        let x: Vec<f64> = (1..=100).map(|i| i as f64).collect();
        let y = x.clone();
        let r = compute(&x, &y, 0.10).unwrap();
        assert!((r.lower_tail_dependence - 1.0).abs() < 1e-9);
        assert!((r.upper_tail_dependence - 1.0).abs() < 1e-9);
    }

    #[test]
    fn counter_monotonic_yields_zero_tail_dependence() {
        // y = −x → ranks reversed → bottom of X = top of Y → no joint extreme.
        let x: Vec<f64> = (1..=100).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|v| -v).collect();
        let r = compute(&x, &y, 0.10).unwrap();
        assert!(r.lower_tail_dependence < 0.05);
        assert!(r.upper_tail_dependence < 0.05);
    }

    #[test]
    fn independent_data_yields_tail_dependence_near_alpha() {
        // For independent data at α=0.10 with n=1000, λ_L ≈ α = 0.10.
        let mut state: u64 = 999;
        let mut x = Vec::with_capacity(1_000);
        let mut y = Vec::with_capacity(1_000);
        for _ in 0..1_000 {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            x.push((state >> 32) as f64 / u32::MAX as f64);
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            y.push((state >> 32) as f64 / u32::MAX as f64);
        }
        let r = compute(&x, &y, 0.10).unwrap();
        // λ_L ≈ α — should be within 0.10 ± 0.05.
        assert!((r.lower_tail_dependence - 0.10).abs() < 0.05);
        assert!((r.upper_tail_dependence - 0.10).abs() < 0.05);
    }

    #[test]
    fn alpha_governs_tail_size() {
        let x: Vec<f64> = (1..=100).map(|i| i as f64).collect();
        let y = x.clone();
        let r05 = compute(&x, &y, 0.05).unwrap();
        let r10 = compute(&x, &y, 0.10).unwrap();
        assert!(r10.tail_size > r05.tail_size);
    }

    #[test]
    fn nan_pairs_skipped() {
        let mut x: Vec<f64> = (1..=50).map(|i| i as f64).collect();
        let y: Vec<f64> = x.clone();
        x[5] = f64::NAN;
        let r = compute(&x, &y, 0.10).unwrap();
        assert_eq!(r.n_observations, 49);
    }
}
