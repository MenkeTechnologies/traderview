//! Monte Carlo Value-at-Risk — full-revaluation MC simulation of
//! portfolio returns from a parametric joint return distribution.
//!
//! Default: multivariate Gaussian with provided mean vector + Cholesky
//! decomposition of the covariance matrix. Per simulation:
//!
//!   z ~ N(0, I)
//!   r = μ + L · z         where L · L' = Σ
//!   pnl = Σ w_i · r_i
//!
//! VaR_α = − quantile(pnl, 1 − α)
//! ES_α  = − mean(pnl | pnl ≤ VaR_α)
//!
//! Pure compute. Companion to `conditional_var`, `component_var`,
//! `cholesky`, `expected_shortfall_contribution`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MonteCarloVarReport {
    pub var_at_confidence: f64,
    pub expected_shortfall_at_confidence: f64,
    pub portfolio_mean: f64,
    pub portfolio_volatility: f64,
    pub n_simulations: usize,
    pub confidence: f64,
}

pub fn simulate(
    weights: &[f64],
    mean_returns: &[f64],
    cholesky_lower: &[Vec<f64>],
    confidence: f64,
    n_simulations: usize,
    seed: u64,
) -> Option<MonteCarloVarReport> {
    let n = weights.len();
    if n == 0
        || mean_returns.len() != n
        || cholesky_lower.len() != n
        || cholesky_lower.iter().any(|r| r.len() != n)
        || n_simulations < 100
        || !confidence.is_finite()
        || !(0.5..1.0).contains(&confidence)
    {
        return None;
    }
    if weights.iter().any(|x| !x.is_finite())
        || mean_returns.iter().any(|x| !x.is_finite())
        || cholesky_lower
            .iter()
            .any(|r| r.iter().any(|x| !x.is_finite()))
    {
        return None;
    }
    let mut state = seed;
    let mut pnl_samples = Vec::with_capacity(n_simulations);
    let mut z = vec![0.0_f64; n];
    let mut r = vec![0.0_f64; n];
    for _ in 0..n_simulations {
        for slot in z.iter_mut() {
            *slot = standard_normal(&mut state);
        }
        for i in 0..n {
            let mut acc = mean_returns[i];
            for (j, zj) in z.iter().enumerate().take(i + 1) {
                acc += cholesky_lower[i][j] * zj;
            }
            r[i] = acc;
        }
        let pnl: f64 = weights.iter().zip(r.iter()).map(|(w, ri)| w * ri).sum();
        pnl_samples.push(pnl);
    }
    let mut sorted = pnl_samples.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let quantile_idx = ((1.0 - confidence) * n_simulations as f64).floor() as usize;
    let quantile_idx = quantile_idx.min(n_simulations - 1);
    let var = -sorted[quantile_idx];
    // ES = − mean(pnl ≤ VaR_quantile), i.e. average of the worst tail.
    let tail_size = (quantile_idx + 1).max(1);
    let es_sum: f64 = sorted[..tail_size].iter().sum();
    let es = -es_sum / tail_size as f64;
    let mean_pnl: f64 = pnl_samples.iter().sum::<f64>() / n_simulations as f64;
    let var_pnl: f64 = pnl_samples
        .iter()
        .map(|p| (p - mean_pnl).powi(2))
        .sum::<f64>()
        / (n_simulations - 1) as f64;
    Some(MonteCarloVarReport {
        var_at_confidence: var,
        expected_shortfall_at_confidence: es,
        portfolio_mean: mean_pnl,
        portfolio_volatility: var_pnl.sqrt(),
        n_simulations,
        confidence,
    })
}

fn standard_normal(state: &mut u64) -> f64 {
    // Polar Box-Muller using a single uniform pair per call.
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    let u1 = ((*state >> 32) as f64 / u32::MAX as f64).max(1e-12);
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    let u2 = (*state >> 32) as f64 / u32::MAX as f64;
    (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_none() {
        let w = vec![0.5, 0.5];
        let m = vec![0.0, 0.0];
        let l = vec![vec![0.1, 0.0], vec![0.0, 0.1]];
        assert!(simulate(&[], &[], &[], 0.95, 1000, 42).is_none());
        assert!(simulate(&w, &m, &l, 1.0, 1000, 42).is_none());
        assert!(simulate(&w, &m, &l, 0.95, 50, 42).is_none());
        assert!(simulate(&w, &m, &l, f64::NAN, 1000, 42).is_none());
    }

    #[test]
    fn univariate_normal_matches_analytic_var() {
        // Single-asset portfolio: σ = 0.10, μ = 0, w = 1.
        let w = vec![1.0];
        let m = vec![0.0];
        let l = vec![vec![0.10]];
        let r = simulate(&w, &m, &l, 0.95, 10_000, 42).unwrap();
        // VaR_95 = z_0.95 · σ ≈ 1.645 · 0.10 = 0.164.
        assert!(
            (r.var_at_confidence - 0.164).abs() < 0.03,
            "VaR ≈ 0.164, got {}",
            r.var_at_confidence
        );
    }

    #[test]
    fn higher_confidence_yields_higher_var() {
        let w = vec![1.0];
        let m = vec![0.0];
        let l = vec![vec![0.10]];
        let r_95 = simulate(&w, &m, &l, 0.95, 10_000, 42).unwrap();
        let r_99 = simulate(&w, &m, &l, 0.99, 10_000, 42).unwrap();
        assert!(r_99.var_at_confidence > r_95.var_at_confidence);
    }

    #[test]
    fn es_exceeds_var() {
        let w = vec![0.5, 0.5];
        let m = vec![0.0, 0.0];
        let l = vec![vec![0.10, 0.0], vec![0.05, 0.087]]; // ρ ≈ 0.5
        let r = simulate(&w, &m, &l, 0.95, 5_000, 42).unwrap();
        assert!(
            r.expected_shortfall_at_confidence >= r.var_at_confidence,
            "ES {} should ≥ VaR {}",
            r.expected_shortfall_at_confidence,
            r.var_at_confidence
        );
    }

    #[test]
    fn deterministic_for_fixed_seed() {
        let w = vec![1.0];
        let m = vec![0.0];
        let l = vec![vec![0.10]];
        let r1 = simulate(&w, &m, &l, 0.95, 1000, 42).unwrap();
        let r2 = simulate(&w, &m, &l, 0.95, 1000, 42).unwrap();
        assert_eq!(r1.var_at_confidence, r2.var_at_confidence);
    }

    #[test]
    fn n_simulations_reported() {
        let w = vec![1.0];
        let m = vec![0.0];
        let l = vec![vec![0.10]];
        let r = simulate(&w, &m, &l, 0.95, 5000, 42).unwrap();
        assert_eq!(r.n_simulations, 5000);
    }
}
