//! Permutation Entropy (Bandt-Pompe 2002) — complexity measure based on
//! the relative ordering of consecutive samples.
//!
//!   PE(m) = −Σ_π p(π) · log(p(π))
//!
//! where π ranges over the m! possible ordinal patterns (permutations
//! of {0..m−1}) and p(π) is the empirical frequency of each pattern in
//! length-m windows of the series.
//!
//! Normalized form: PE_norm = PE / log(m!) ∈ [0, 1].
//!
//! Compared to `sample_entropy`:
//!   - Faster (O(N·m) vs O(N²))
//!   - Invariant to monotonic transformations (only ranks matter)
//!   - Distinguishes regular vs noisy series cleanly
//!
//! Pure compute. Order m typically 3..7; we accept 2..8.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PermutationEntropyReport {
    pub permutation_entropy: f64,
    pub normalized_entropy: f64,
    pub order: usize,
    pub n_patterns: usize,
    pub n_observations: usize,
}

pub fn compute(series: &[f64], order: usize) -> Option<PermutationEntropyReport> {
    let n = series.len();
    if !(2..=8).contains(&order) || n < order + 1 {
        return None;
    }
    if series.iter().any(|x| !x.is_finite()) { return None; }
    // Count ordinal patterns.
    let mut counts: std::collections::HashMap<Vec<usize>, usize> =
        std::collections::HashMap::new();
    let mut n_windows = 0_usize;
    for start in 0..=(n - order) {
        let window = &series[start..start + order];
        // Get ordinal pattern (indices sorted by value).
        let mut indexed: Vec<(usize, f64)> = window.iter().enumerate()
            .map(|(i, v)| (i, *v)).collect();
        indexed.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        let pattern: Vec<usize> = indexed.iter().map(|(i, _)| *i).collect();
        *counts.entry(pattern).or_insert(0) += 1;
        n_windows += 1;
    }
    if n_windows == 0 { return None; }
    let n_f = n_windows as f64;
    let pe: f64 = counts.values()
        .map(|&c| {
            let p = c as f64 / n_f;
            -p * p.ln()
        })
        .sum();
    // Normalize by log(m!).
    let m_fact: u64 = (1..=order as u64).product();
    let max_entropy = (m_fact as f64).ln();
    let normalized = if max_entropy > 0.0 { pe / max_entropy } else { 0.0 };
    Some(PermutationEntropyReport {
        permutation_entropy: pe,
        normalized_entropy: normalized,
        order,
        n_patterns: counts.len(),
        n_observations: n,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn too_short_returns_none() {
        assert!(compute(&[0.01; 3], 5).is_none());
    }

    #[test]
    fn invalid_order_returns_none() {
        let s = vec![0.01; 100];
        assert!(compute(&s, 1).is_none());
        assert!(compute(&s, 9).is_none());
    }

    #[test]
    fn nan_input_returns_none() {
        let mut s = vec![0.01; 100];
        s[5] = f64::NAN;
        assert!(compute(&s, 3).is_none());
    }

    #[test]
    fn monotonically_increasing_yields_zero_entropy() {
        // Every window has the same ordinal pattern (sorted ascending) →
        // single pattern → PE = 0.
        let s: Vec<f64> = (0..100).map(|i| i as f64).collect();
        let r = compute(&s, 3).unwrap();
        assert_eq!(r.n_patterns, 1);
        assert!(r.permutation_entropy.abs() < 1e-12);
        assert!(r.normalized_entropy.abs() < 1e-12);
    }

    #[test]
    fn random_series_yields_high_normalized_entropy() {
        let mut state: u64 = 42;
        let s: Vec<f64> = (0..1_000).map(|_| {
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (state >> 32) as f64 / u32::MAX as f64
        }).collect();
        let r = compute(&s, 4).unwrap();
        // Should hit all 24 patterns with roughly uniform probability.
        assert!(r.normalized_entropy > 0.9,
            "random series should give normalized PE > 0.9, got {}",
            r.normalized_entropy);
    }

    #[test]
    fn invariant_to_monotonic_transformation() {
        // PE depends only on rank order — multiplying by a positive
        // constant or adding doesn't change it.
        let mut state: u64 = 99;
        let s: Vec<f64> = (0..500).map(|_| {
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (state >> 32) as f64 / u32::MAX as f64
        }).collect();
        let scaled: Vec<f64> = s.iter().map(|x| 5.0 * x + 100.0).collect();
        let r1 = compute(&s, 3).unwrap();
        let r2 = compute(&scaled, 3).unwrap();
        assert!((r1.permutation_entropy - r2.permutation_entropy).abs() < 1e-9);
    }

    #[test]
    fn normalized_entropy_in_unit_range() {
        let s: Vec<f64> = (0..200).map(|i| (i as f64 * 0.07).sin()).collect();
        let r = compute(&s, 3).unwrap();
        assert!((0.0..=1.0).contains(&r.normalized_entropy));
    }

    #[test]
    fn higher_order_yields_more_possible_patterns() {
        let mut state: u64 = 7;
        let s: Vec<f64> = (0..500).map(|_| {
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (state >> 32) as f64 / u32::MAX as f64
        }).collect();
        let r3 = compute(&s, 3).unwrap();
        let r5 = compute(&s, 5).unwrap();
        assert!(r5.n_patterns >= r3.n_patterns);
    }
}
