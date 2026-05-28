//! Hill Estimator — Bruce Hill (1975).
//!
//! Maximum-likelihood estimator of the tail index α for a Pareto-type
//! distribution. Given a sample sorted in descending order
//! X_{(1)} ≥ X_{(2)} ≥ … ≥ X_{(n)}, the Hill estimator using the top
//! k order statistics is:
//!
//!   α̂_H(k) = ( (1/k) · Σ_{i=1..k} log(X_{(i)} / X_{(k+1)}) )⁻¹
//!
//! Properties:
//!   - α̂ < 2 → infinite variance (Pareto-type heavy tail)
//!   - α̂ < 1 → infinite mean
//!   - The reciprocal ξ = 1/α̂ is the extreme-value-theory shape parameter
//!
//! Use cases:
//!   - Tail thickness of return / loss distributions
//!   - VaR / ES validation under heavy tails
//!   - Detecting fat-tail regimes (α̂ around 3–4 typical for equities;
//!     α̂ near 2 = crash-regime)
//!
//! The optimal k is non-trivial (Hill plot inspection or bootstrap).
//! This implementation also reports the estimator across a small range
//! of k to enable visual stability assessment.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HillReport {
    pub n_observations: usize,
    /// k-values evaluated.
    pub k_values: Vec<usize>,
    /// Hill α̂ at each k.
    pub alpha_estimates: Vec<f64>,
    /// Corresponding ξ = 1/α̂.
    pub xi_estimates: Vec<f64>,
    /// Suggested k = ⌊n^(2/3)⌋ heuristic (Drees-Resnick), bounded to
    /// available data; α̂ at that k.
    pub heuristic_k: usize,
    pub heuristic_alpha: f64,
}

/// Computes the Hill tail-index estimator on the absolute values of
/// `losses`. Pass losses as positive numbers (e.g. negate negative
/// returns). Returns None if fewer than `min_n` finite positive
/// observations.
pub fn compute(losses: &[f64], k_values: &[usize]) -> Option<HillReport> {
    let mut sorted: Vec<f64> = losses.iter().copied()
        .filter(|x| x.is_finite() && *x > 0.0)
        .collect();
    let n = sorted.len();
    if n < 10 || k_values.is_empty() { return None; }
    // Descending sort.
    sorted.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
    let mut ks = Vec::new();
    let mut alphas = Vec::new();
    let mut xis = Vec::new();
    for &k in k_values {
        if k < 2 || k >= n { continue; }
        let log_threshold = sorted[k].ln();
        let mean_excess: f64 = (0..k)
            .map(|i| sorted[i].ln() - log_threshold).sum::<f64>() / k as f64;
        if mean_excess <= 0.0 || !mean_excess.is_finite() { continue; }
        ks.push(k);
        alphas.push(1.0 / mean_excess);
        xis.push(mean_excess);
    }
    if ks.is_empty() { return None; }
    // Heuristic optimum k. round (not floor) because n^(2/3) for exact
    // cubes like 1000 lands just below the integer in f64.
    let heuristic_k = ((n as f64).powf(2.0 / 3.0).round() as usize)
        .clamp(2, n.saturating_sub(1));
    let log_threshold = sorted[heuristic_k].ln();
    let mean_excess: f64 = (0..heuristic_k)
        .map(|i| sorted[i].ln() - log_threshold).sum::<f64>() / heuristic_k as f64;
    let heuristic_alpha = if mean_excess > 0.0 { 1.0 / mean_excess } else { f64::NAN };
    Some(HillReport {
        n_observations: n,
        k_values: ks,
        alpha_estimates: alphas,
        xi_estimates: xis,
        heuristic_k,
        heuristic_alpha,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn too_few_returns_none() {
        let losses = vec![1.0_f64; 5];
        assert!(compute(&losses, &[2]).is_none());
    }

    #[test]
    fn empty_k_values_returns_none() {
        let losses = vec![1.0_f64; 100];
        assert!(compute(&losses, &[]).is_none());
    }

    #[test]
    fn invalid_k_filtered_out() {
        let losses: Vec<f64> = (1..=100).map(|i| i as f64).collect();
        // k = 0, 1, 200 all invalid; only k = 10, 20 valid.
        let r = compute(&losses, &[0, 1, 10, 20, 200]).unwrap();
        assert_eq!(r.k_values, vec![10, 20]);
    }

    #[test]
    fn nonpositive_or_nan_observations_filtered() {
        let mut losses: Vec<f64> = (1..=100).map(|i| i as f64).collect();
        losses.extend([-5.0, 0.0, f64::NAN]);
        let r = compute(&losses, &[10]).unwrap();
        // 3 invalid filtered; n = 100.
        assert_eq!(r.n_observations, 100);
    }

    #[test]
    fn pareto_sample_recovers_known_alpha() {
        // Draw Pareto samples X = (1/U)^(1/α) with α = 3.0, then estimate.
        let mut state: u64 = 42;
        let true_alpha = 3.0_f64;
        let n = 5000;
        let losses: Vec<f64> = (0..n).map(|_| {
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let u = ((state >> 32) as f64 / u32::MAX as f64).max(1e-12);
            u.powf(-1.0 / true_alpha)
        }).collect();
        let k_values = vec![50, 100, 200, 500];
        let r = compute(&losses, &k_values).unwrap();
        // At least one estimate should be within 25% of true alpha.
        let close = r.alpha_estimates.iter().any(|a| (a - true_alpha).abs() / true_alpha < 0.25);
        assert!(close, "no Hill estimate close to true α = {true_alpha}: {:?}", r.alpha_estimates);
    }

    #[test]
    fn xi_is_reciprocal_of_alpha() {
        let losses: Vec<f64> = (1..=200).map(|i| i as f64).collect();
        let r = compute(&losses, &[20, 50]).unwrap();
        for (a, x) in r.alpha_estimates.iter().zip(r.xi_estimates.iter()) {
            assert!((a * x - 1.0).abs() < 1e-12);
        }
    }

    #[test]
    fn heuristic_k_uses_n_to_two_thirds() {
        let losses: Vec<f64> = (1..=1000).map(|i| i as f64).collect();
        let r = compute(&losses, &[100]).unwrap();
        // 1000^(2/3) ≈ 100; floor = 100.
        assert_eq!(r.heuristic_k, 100);
    }

    #[test]
    fn alpha_positive_for_strictly_positive_sample() {
        let losses: Vec<f64> = (1..=500).map(|i| i as f64).collect();
        let r = compute(&losses, &[50, 100]).unwrap();
        for a in &r.alpha_estimates {
            assert!(*a > 0.0);
        }
    }
}
