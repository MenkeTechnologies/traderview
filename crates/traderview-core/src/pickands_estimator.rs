//! Pickands Tail Index Estimator (Pickands 1975).
//!
//! For a sample X_{(1)} ≤ … ≤ X_{(n)} sorted ascending, the Pickands
//! shape parameter estimator using the top 4k order statistics is:
//!
//!   ξ̂_P(k) = (1 / ln 2) · ln((X_{(n−k+1)} − X_{(n−2k+1)})
//!                            / (X_{(n−2k+1)} − X_{(n−4k+1)}))
//!
//! Advantages over Hill estimator:
//!   - Works for any sign of ξ (Hill requires ξ > 0)
//!   - Robust to translation: works on raw data not just exceedances
//!
//! Disadvantage: higher variance than Hill for fixed sample size.
//!
//! Range:
//!   - ξ > 0 → Fréchet (heavy tail)
//!   - ξ = 0 → Gumbel (exponential tail)
//!   - ξ < 0 → Weibull (bounded tail)
//!
//! Default k = ⌊n / 4⌋ (uses the top quarter of order stats).
//!
//! Pure compute. Companion to `hill_estimator`, `gpd_tail_fit`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PickandsReport {
    pub k_values: Vec<usize>,
    pub shape_estimates: Vec<f64>,
    pub heuristic_k: usize,
    pub heuristic_shape: f64,
    pub n_observations: usize,
}

pub fn compute(losses: &[f64], k_values: &[usize]) -> Option<PickandsReport> {
    let mut sorted: Vec<f64> = losses.iter().copied()
        .filter(|x| x.is_finite()).collect();
    let n = sorted.len();
    if n < 20 { return None; }
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mut ks = Vec::new();
    let mut shapes = Vec::new();
    for &k in k_values {
        if k < 2 || 4 * k > n { continue; }
        if let Some(xi) = pickands_at_k(&sorted, n, k) {
            ks.push(k);
            shapes.push(xi);
        }
    }
    if ks.is_empty() { return None; }
    let heuristic_k = (n / 4).max(2).min(n / 4);
    let heuristic_shape = pickands_at_k(&sorted, n, heuristic_k)?;
    Some(PickandsReport {
        k_values: ks,
        shape_estimates: shapes,
        heuristic_k,
        heuristic_shape,
        n_observations: n,
    })
}

fn pickands_at_k(sorted: &[f64], n: usize, k: usize) -> Option<f64> {
    if 4 * k > n { return None; }
    // Indices for ascending sorted: X_{(n-k+1)} is at idx n-k.
    let x_n_k1 = sorted[n - k];
    let x_n_2k1 = sorted[n - 2 * k];
    let x_n_4k1 = sorted[n - 4 * k];
    let num = x_n_k1 - x_n_2k1;
    let den = x_n_2k1 - x_n_4k1;
    if den <= 0.0 || num <= 0.0 { return None; }
    let xi = (num / den).ln() / 2.0_f64.ln();
    if !xi.is_finite() { return None; }
    Some(xi)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sim_pareto(n: usize, alpha: f64, seed: u64) -> Vec<f64> {
        let mut state = seed;
        (0..n).map(|_| {
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let u = ((state >> 32) as f64 / u32::MAX as f64).max(1e-12);
            u.powf(-1.0 / alpha)
        }).collect()
    }

    #[test]
    fn too_few_returns_none() {
        assert!(compute(&[1.0; 10], &[2]).is_none());
    }

    #[test]
    fn nan_filtered() {
        let mut losses = sim_pareto(200, 3.0, 1);
        losses.push(f64::NAN);
        let r = compute(&losses, &[10]).unwrap();
        assert_eq!(r.n_observations, 200);
    }

    #[test]
    fn invalid_k_filtered() {
        let losses = sim_pareto(100, 3.0, 1);
        let r = compute(&losses, &[0, 1, 10, 30]).unwrap();
        // k=0, k=1, k=30 (4·30 > 100) all invalid; only k=10 valid.
        assert_eq!(r.k_values, vec![10]);
    }

    #[test]
    fn pareto_yields_positive_shape() {
        // Pareto α = 3 → ξ = 1/α = 0.333 → Pickands recovers positive shape.
        let losses = sim_pareto(2000, 3.0, 42);
        let r = compute(&losses, &[50, 100, 200]).unwrap();
        let any_positive = r.shape_estimates.iter().any(|s| *s > 0.0);
        assert!(any_positive,
            "Pareto sample should yield some positive ξ, got {:?}", r.shape_estimates);
    }

    #[test]
    fn heuristic_uses_quarter_of_sample() {
        let losses = sim_pareto(400, 3.0, 7);
        let r = compute(&losses, &[10]).unwrap();
        assert_eq!(r.heuristic_k, 100);
    }

    #[test]
    fn n_observations_reported() {
        let losses = sim_pareto(150, 2.0, 11);
        let r = compute(&losses, &[10]).unwrap();
        assert_eq!(r.n_observations, 150);
    }
}
