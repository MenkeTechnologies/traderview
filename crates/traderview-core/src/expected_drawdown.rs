//! Expected Drawdown via Monte Carlo on a parametric return process.
//!
//! Simulates `n_paths` price paths under a normal return distribution
//! and reports the distribution of maximum drawdowns:
//!
//!   path: P_t+1 = P_t · exp(μ + σ · z), z ~ N(0,1)
//!   mdd: max((HWM_t − P_t) / HWM_t) over t = 1..horizon
//!
//! Outputs:
//!   - E[MDD]                 (Expected Drawdown)
//!   - median MDD
//!   - 5th / 95th percentile of MDD distribution
//!   - VaR_95 of MDD (95th percentile = 5%-tail-loss MDD)
//!
//! Use cases:
//!   - Sizing position to keep expected MDD under a budget
//!   - Pre-trade simulation of strategy maximum drawdown
//!   - Stress-test risk reporting
//!
//! Pure compute. Companion to `rolling_drawdown`, `monte_carlo_var`,
//! `pain_index`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExpectedDrawdownReport {
    pub expected_drawdown: f64,
    pub median_drawdown: f64,
    pub drawdown_5th_percentile: f64,
    pub drawdown_95th_percentile: f64,
    pub max_simulated_drawdown: f64,
    pub n_paths: usize,
    pub horizon: usize,
}

pub fn simulate(
    drift_per_period: f64,
    vol_per_period: f64,
    horizon: usize,
    n_paths: usize,
    seed: u64,
) -> Option<ExpectedDrawdownReport> {
    if !drift_per_period.is_finite()
        || !vol_per_period.is_finite() || vol_per_period <= 0.0
        || horizon < 2 || n_paths < 100 {
        return None;
    }
    let mut state = seed;
    let mut mdds = Vec::with_capacity(n_paths);
    for _ in 0..n_paths {
        let mut price = 1.0_f64;
        let mut hwm = 1.0_f64;
        let mut max_dd = 0.0_f64;
        for _ in 0..horizon {
            let z = standard_normal(&mut state);
            price *= (drift_per_period + vol_per_period * z).exp();
            if price > hwm { hwm = price; }
            let dd = (hwm - price) / hwm;
            if dd > max_dd { max_dd = dd; }
        }
        mdds.push(max_dd);
    }
    mdds.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n_f = n_paths as f64;
    let mean: f64 = mdds.iter().sum::<f64>() / n_f;
    let q = |p: f64| {
        let idx = ((p * n_f).floor() as usize).min(n_paths - 1);
        mdds[idx]
    };
    Some(ExpectedDrawdownReport {
        expected_drawdown: mean,
        median_drawdown: q(0.50),
        drawdown_5th_percentile: q(0.05),
        drawdown_95th_percentile: q(0.95),
        max_simulated_drawdown: *mdds.last().unwrap(),
        n_paths,
        horizon,
    })
}

fn standard_normal(state: &mut u64) -> f64 {
    *state = state.wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    let u1 = ((*state >> 32) as f64 / u32::MAX as f64).max(1e-12);
    *state = state.wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    let u2 = (*state >> 32) as f64 / u32::MAX as f64;
    (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_none() {
        assert!(simulate(0.0, 0.0, 100, 1000, 42).is_none());
        assert!(simulate(0.0, 0.01, 1, 1000, 42).is_none());
        assert!(simulate(0.0, 0.01, 100, 50, 42).is_none());
        assert!(simulate(f64::NAN, 0.01, 100, 1000, 42).is_none());
    }

    #[test]
    fn higher_vol_yields_larger_expected_drawdown() {
        let low = simulate(0.0, 0.01, 250, 1000, 42).unwrap();
        let high = simulate(0.0, 0.05, 250, 1000, 42).unwrap();
        assert!(high.expected_drawdown > low.expected_drawdown);
    }

    #[test]
    fn quantiles_ordered_correctly() {
        let r = simulate(0.0, 0.02, 250, 1000, 42).unwrap();
        assert!(r.drawdown_5th_percentile <= r.median_drawdown);
        assert!(r.median_drawdown <= r.drawdown_95th_percentile);
        assert!(r.drawdown_95th_percentile <= r.max_simulated_drawdown);
    }

    #[test]
    fn drawdown_non_negative() {
        let r = simulate(0.001, 0.02, 250, 500, 7).unwrap();
        assert!(r.expected_drawdown >= 0.0);
        assert!(r.median_drawdown >= 0.0);
    }

    #[test]
    fn deterministic_for_fixed_seed() {
        let r1 = simulate(0.001, 0.02, 100, 200, 42).unwrap();
        let r2 = simulate(0.001, 0.02, 100, 200, 42).unwrap();
        assert_eq!(r1.expected_drawdown, r2.expected_drawdown);
    }

    #[test]
    fn n_paths_and_horizon_reported() {
        let r = simulate(0.0, 0.02, 100, 500, 42).unwrap();
        assert_eq!(r.n_paths, 500);
        assert_eq!(r.horizon, 100);
    }
}
