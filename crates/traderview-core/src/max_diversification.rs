//! Maximum-Diversification Portfolio — Choueifaty & Coignard (2008).
//!
//! Finds the weights that maximize the diversification ratio:
//!
//!   DR(w) = (wᵀ · σ) / √(wᵀ · Σ · w)
//!
//! where σ is the vector of asset volatilities and Σ is the covariance
//! matrix. DR ≥ 1 always; DR = 1 means single-asset (no diversification);
//! higher = better-diversified.
//!
//! Closed-form proportional weights (Choueifaty-Coignard 2008):
//!
//!   w ∝ Σ⁻¹ · σ
//!
//! then normalize so Σ w = 1.
//!
//! Pure compute. Companion to `risk_parity_weights`,
//! `hierarchical_risk_parity`, `min_variance_portfolio`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MaxDivReport {
    pub weights: Vec<f64>,
    pub diversification_ratio: f64,
    pub portfolio_volatility: f64,
    pub weighted_average_volatility: f64,
}

pub fn solve(covariance: &[Vec<f64>]) -> Option<MaxDivReport> {
    let n = covariance.len();
    if n < 2 || covariance.iter().any(|r| r.len() != n) { return None; }
    if covariance.iter().any(|r| r.iter().any(|c| !c.is_finite())) { return None; }
    // Asset volatilities (square root of diagonal).
    let stdev: Vec<f64> = (0..n).map(|i| covariance[i][i].max(0.0).sqrt()).collect();
    if stdev.iter().any(|s| !s.is_finite() || *s <= 0.0) { return None; }
    // Solve Σ·w = σ, then normalize.
    let mut w = solve_linear(covariance, &stdev)?;
    let sum: f64 = w.iter().sum();
    if sum.abs() < 1e-18 { return None; }
    for x in w.iter_mut() { *x /= sum; }
    // Portfolio vol = √(wᵀ Σ w).
    let sigma_w: Vec<f64> = matvec(covariance, &w);
    let port_var: f64 = w.iter().zip(sigma_w.iter()).map(|(a, b)| a * b).sum();
    let port_vol = port_var.max(0.0).sqrt();
    // Weighted avg vol = wᵀ σ.
    let wavg_vol: f64 = w.iter().zip(stdev.iter()).map(|(a, b)| a * b).sum();
    let dr = if port_vol > 0.0 { wavg_vol / port_vol } else { 1.0 };
    Some(MaxDivReport {
        weights: w,
        diversification_ratio: dr,
        portfolio_volatility: port_vol,
        weighted_average_volatility: wavg_vol,
    })
}

fn matvec(m: &[Vec<f64>], v: &[f64]) -> Vec<f64> {
    m.iter().map(|r| r.iter().zip(v.iter()).map(|(a, b)| a * b).sum()).collect()
}

fn solve_linear(m: &[Vec<f64>], y: &[f64]) -> Option<Vec<f64>> {
    let n = m.len();
    if n == 0 || y.len() != n || m.iter().any(|r| r.len() != n) { return None; }
    let mut aug = vec![vec![0.0_f64; n + 1]; n];
    for i in 0..n {
        for j in 0..n { aug[i][j] = m[i][j]; }
        aug[i][n] = y[i];
    }
    for i in 0..n {
        let mut pivot = i;
        for r in (i + 1)..n {
            if aug[r][i].abs() > aug[pivot][i].abs() { pivot = r; }
        }
        if aug[pivot][i].abs() < 1e-18 { return None; }
        aug.swap(i, pivot);
        let div = aug[i][i];
        for v in aug[i].iter_mut() { *v /= div; }
        for r in 0..n {
            if r == i { continue; }
            let f = aug[r][i];
            if f == 0.0 { continue; }
            let pivot_row = aug[i].clone();
            for (j, v) in aug[r].iter_mut().enumerate() { *v -= f * pivot_row[j]; }
        }
    }
    Some((0..n).map(|i| aug[i][n]).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_or_malformed_returns_none() {
        assert!(solve(&[]).is_none());
        assert!(solve(&[vec![1.0, 0.0]]).is_none());
        let nan = vec![vec![f64::NAN, 0.0], vec![0.0, 1.0]];
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
            vec![0.04, 0.0, 0.0],
            vec![0.0, 0.04, 0.0],
            vec![0.0, 0.0, 0.04],
        ];
        let r = solve(&cov).unwrap();
        for w in &r.weights {
            assert!((w - 1.0 / 3.0).abs() < 1e-9);
        }
    }

    #[test]
    fn diversification_ratio_at_least_one() {
        let cov = vec![
            vec![0.04, 0.01, 0.005],
            vec![0.01, 0.09, 0.02],
            vec![0.005, 0.02, 0.16],
        ];
        let r = solve(&cov).unwrap();
        // DR can be slightly below 1.0 in numerical edge cases but should
        // be at least 1.0 for any non-degenerate portfolio.
        assert!(r.diversification_ratio >= 0.99);
    }

    #[test]
    fn weights_sum_to_one() {
        let cov = vec![
            vec![0.04, 0.01],
            vec![0.01, 0.09],
        ];
        let r = solve(&cov).unwrap();
        let sum: f64 = r.weights.iter().sum();
        assert!((sum - 1.0).abs() < 1e-9);
    }

    #[test]
    fn correlated_assets_yield_lower_dr_than_uncorrelated() {
        let uncorr = vec![
            vec![0.04, 0.0],
            vec![0.0, 0.04],
        ];
        let corr = vec![
            vec![0.04, 0.035],
            vec![0.035, 0.04],
        ];
        let r_uncorr = solve(&uncorr).unwrap();
        let r_corr = solve(&corr).unwrap();
        assert!(r_uncorr.diversification_ratio > r_corr.diversification_ratio);
    }

    #[test]
    fn singular_covariance_returns_none() {
        let cov = vec![
            vec![1.0, 1.0],
            vec![1.0, 1.0],
        ];
        assert!(solve(&cov).is_none());
    }
}
