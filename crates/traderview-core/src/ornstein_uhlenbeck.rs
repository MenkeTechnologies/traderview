//! Ornstein-Uhlenbeck mean-reversion fit.
//!
//! Continuous-time:  dx_t = θ(μ − x_t) dt + σ dW_t
//! Discrete-time AR(1) representation (dt = 1 unit):
//!
//!   x_t = α + β · x_{t−1} + ε_t
//!
//! where
//!   β = e^{−θ}
//!   α = μ · (1 − β)
//!   stdev(ε) = σ · √((1 − e^{−2θ}) / (2θ))
//!
//! Returns:
//!   - mu      = long-run mean
//!   - theta   = mean-reversion speed (per unit step)
//!   - sigma   = diffusion intensity
//!   - half_life = ln(2) / θ
//!   - residuals = x_t − (α + β · x_{t−1})
//!
//! Pure compute. Useful for sizing pairs trades on cointegrated
//! residual spreads — the half-life sets the expected holding period
//! and σ sets the entry/exit z-score bands.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OuReport {
    pub mu: f64,
    pub theta: f64,
    pub sigma: f64,
    pub half_life: f64,
    pub residuals: Vec<f64>,
}

pub fn fit(series: &[f64]) -> Option<OuReport> {
    let n = series.len();
    if n < 4 {
        return None;
    }
    if series.iter().any(|x| !x.is_finite()) {
        return None;
    }
    // Build (x_{t−1}, x_t) pairs and run OLS.
    let m = n - 1;
    let mean_x = series[..m].iter().sum::<f64>() / m as f64;
    let mean_y = series[1..].iter().sum::<f64>() / m as f64;
    let mut sxy = 0.0_f64;
    let mut sxx = 0.0_f64;
    for i in 0..m {
        let dx = series[i] - mean_x;
        let dy = series[i + 1] - mean_y;
        sxy += dx * dy;
        sxx += dx * dx;
    }
    if sxx <= 0.0 {
        return None;
    }
    let beta = sxy / sxx;
    let alpha = mean_y - beta * mean_x;
    if !(0.0..1.0).contains(&beta) {
        // β ≥ 1 → no reversion. β < 0 → flip oscillation (not OU).
        return None;
    }
    let theta = -beta.ln();
    if !theta.is_finite() || theta <= 0.0 {
        return None;
    }
    let mu = alpha / (1.0 - beta);
    // Residuals + σ̂.
    let mut residuals = Vec::with_capacity(m);
    for i in 0..m {
        residuals.push(series[i + 1] - alpha - beta * series[i]);
    }
    let var_resid: f64 = residuals.iter().map(|r| r * r).sum::<f64>() / (m as f64 - 2.0).max(1.0);
    // σ² · (1 − e^{−2θ}) / (2θ) = var(residual)
    // → σ² = var(resid) · 2θ / (1 − e^{−2θ})
    let factor = (1.0 - (-2.0 * theta).exp()) / (2.0 * theta);
    let sigma2 = var_resid / factor.max(1e-18);
    let sigma = sigma2.max(0.0).sqrt();
    let half_life = std::f64::consts::LN_2 / theta;
    Some(OuReport {
        mu,
        theta,
        sigma,
        half_life,
        residuals,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_none() {
        assert!(fit(&[]).is_none());
    }

    #[test]
    fn nan_returns_none() {
        let mut s = vec![1.0; 100];
        s[5] = f64::NAN;
        assert!(fit(&s).is_none());
    }

    #[test]
    fn flat_series_returns_none() {
        let s = vec![5.0; 100];
        assert!(fit(&s).is_none());
    }

    #[test]
    fn random_walk_yields_no_strong_reversion() {
        // Pure random walk has β = 1 in the limit. Finite samples produce
        // β slightly < 1 by chance, so a fit may succeed — but the
        // half-life will be large vs the mean-reverting case (which has
        // half-life ≈ 1 with β = 0.5). We assert the half-life is at
        // least 10× larger than the typical mean-reverting fit (HL≈1).
        let mut s = vec![0.0_f64];
        let mut state = 42u64;
        for _ in 0..2_000 {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let u = ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 0.05;
            s.push(s.last().unwrap() + u);
        }
        match fit(&s) {
            None => {} // strong-reversion guard rejected: also fine
            Some(report) => {
                assert!(
                    report.half_life > 10.0,
                    "random walk half-life should be long, got {}",
                    report.half_life
                );
            }
        }
    }

    #[test]
    fn mean_reverting_ar1_recovers_mu_and_theta() {
        // True process: x_t = 0.5 · x_{t−1} + 5 + ε  →  μ = 10, β = 0.5,
        // θ = −ln(0.5) ≈ 0.693, half-life = 1.
        let mut s = vec![10.0_f64];
        let mut state = 999u64;
        for _ in 0..2_000 {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let u = ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 0.5;
            let prev = *s.last().unwrap();
            s.push(0.5 * prev + 5.0 + u);
        }
        let report = fit(&s).expect("populated");
        assert!(
            (report.mu - 10.0).abs() < 0.2,
            "μ should be ≈ 10, got {}",
            report.mu
        );
        assert!(
            (report.theta - 0.693).abs() < 0.05,
            "θ should be ≈ ln(2) ≈ 0.693, got {}",
            report.theta
        );
        assert!((report.half_life - 1.0).abs() < 0.1);
        assert!(report.sigma > 0.0);
    }

    #[test]
    fn higher_beta_yields_longer_half_life() {
        let mut slow = vec![10.0_f64];
        let mut state = 7u64;
        for _ in 0..2_000 {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let u = ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 0.5;
            let prev = *slow.last().unwrap();
            slow.push(0.95 * prev + 0.5 + u);
        }
        let r_slow = fit(&slow).expect("populated");
        let mut fast = vec![10.0_f64];
        let mut state = 7u64;
        for _ in 0..2_000 {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let u = ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 0.5;
            let prev = *fast.last().unwrap();
            fast.push(0.3 * prev + 7.0 + u);
        }
        let r_fast = fit(&fast).expect("populated");
        assert!(
            r_slow.half_life > r_fast.half_life,
            "β=0.95 should have longer HL ({}) than β=0.3 ({})",
            r_slow.half_life,
            r_fast.half_life
        );
    }

    #[test]
    fn residuals_have_length_n_minus_one() {
        let mut s = vec![10.0_f64];
        let mut state = 1u64;
        for _ in 0..100 {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let u = ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 0.5;
            let prev = *s.last().unwrap();
            s.push(0.5 * prev + 5.0 + u);
        }
        let report = fit(&s).expect("populated");
        assert_eq!(report.residuals.len(), s.len() - 1);
    }
}
