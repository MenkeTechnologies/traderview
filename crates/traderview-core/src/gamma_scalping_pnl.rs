//! Gamma Scalping P&L Simulator.
//!
//! For a long-gamma, delta-hedged options position rebalanced
//! discretely, the running P&L decomposes into three components:
//!
//!   1. **Theta decay** = − Θ · Δt per period (Θ < 0 for long options)
//!   2. **Gamma scalping** = ½ · Γ · (ΔS)² per period (always ≥ 0 for long Γ)
//!   3. **Re-hedge transaction cost** = |Δw| · spot · tc_pct
//!
//! Net per-period P&L:
//!
//!   pnl_t = gamma · (ΔS_t)² / 2 − theta_per_day + transaction_cost
//!
//! Useful for:
//!   - Sizing required realized vol vs implied to break even on long gamma
//!   - Evaluating gamma-scalping strategy economics at given transaction costs
//!   - Optimal re-hedge frequency (more frequent = more friction, less variance)
//!
//! Pure compute. Companion to `greeks_profile`, `second_order_greeks`,
//! `iv_solver`, `bipower_variation`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GammaScalpingReport {
    pub per_step_pnl: Vec<f64>,
    pub cumulative_pnl: Vec<f64>,
    pub total_pnl: f64,
    pub total_gamma_pnl: f64,
    pub total_theta_pnl: f64,
    pub total_transaction_cost: f64,
    pub realized_vol_annualized: f64,
    pub breakeven_iv_annualized: f64,
    pub n_steps: usize,
}

pub fn simulate(
    spot_path: &[f64],
    gamma: f64,
    theta_per_step: f64,
    transaction_cost_pct: f64,
    steps_per_year: f64,
) -> Option<GammaScalpingReport> {
    let n = spot_path.len();
    if n < 2 { return None; }
    if !gamma.is_finite() || !theta_per_step.is_finite()
        || !transaction_cost_pct.is_finite() || transaction_cost_pct < 0.0
        || !steps_per_year.is_finite() || steps_per_year <= 0.0
        || spot_path.iter().any(|s| !s.is_finite() || *s <= 0.0)
    {
        return None;
    }
    let mut per_step = Vec::with_capacity(n - 1);
    let mut cumulative = Vec::with_capacity(n - 1);
    let mut gamma_total = 0.0_f64;
    let mut theta_total = 0.0_f64;
    let mut tc_total = 0.0_f64;
    let mut cum = 0.0_f64;
    let mut sum_sq_returns = 0.0_f64;
    for i in 1..n {
        let ds = spot_path[i] - spot_path[i - 1];
        let log_return = (spot_path[i] / spot_path[i - 1]).ln();
        sum_sq_returns += log_return * log_return;
        let gamma_pnl = 0.5 * gamma * ds * ds;
        let theta_pnl = theta_per_step;
        // Re-hedge size = |Γ · ΔS| shares; transaction cost = |Δshares| · spot · tc%.
        let rehedge_shares = (gamma * ds).abs();
        let tc = rehedge_shares * spot_path[i] * transaction_cost_pct;
        let pnl = gamma_pnl + theta_pnl - tc;
        per_step.push(pnl);
        cum += pnl;
        cumulative.push(cum);
        gamma_total += gamma_pnl;
        theta_total += theta_pnl;
        tc_total += tc;
    }
    let n_steps = (n - 1) as f64;
    let realized_var_per_step = sum_sq_returns / n_steps;
    let realized_vol_ann = (realized_var_per_step * steps_per_year).max(0.0).sqrt();
    // Breakeven IV: realized vol at which total gamma PnL exactly offsets
    // theta + transaction costs. From gamma_pnl ≈ ½ · Γ · σ² · S² · Δt:
    let cost_per_step = (-theta_per_step + tc_total / n_steps).max(0.0);
    let mid_spot: f64 = spot_path.iter().sum::<f64>() / n as f64;
    let breakeven_var_per_step = if gamma > 0.0 {
        2.0 * cost_per_step / (gamma * mid_spot * mid_spot)
    } else { 0.0 };
    let breakeven_iv = (breakeven_var_per_step * steps_per_year).max(0.0).sqrt();
    Some(GammaScalpingReport {
        per_step_pnl: per_step,
        cumulative_pnl: cumulative,
        total_pnl: cum,
        total_gamma_pnl: gamma_total,
        total_theta_pnl: theta_total,
        total_transaction_cost: tc_total,
        realized_vol_annualized: realized_vol_ann,
        breakeven_iv_annualized: breakeven_iv,
        n_steps: (n - 1),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn too_short_returns_none() {
        assert!(simulate(&[100.0], 0.01, -0.05, 0.0001, 252.0).is_none());
    }

    #[test]
    fn invalid_inputs_return_none() {
        assert!(simulate(&[100.0, 101.0], 0.01, -0.05, -0.001, 252.0).is_none());
        assert!(simulate(&[100.0, 101.0], 0.01, -0.05, 0.0001, 0.0).is_none());
        assert!(simulate(&[100.0, -1.0], 0.01, -0.05, 0.0001, 252.0).is_none());
        assert!(simulate(&[100.0, f64::NAN], 0.01, -0.05, 0.0001, 252.0).is_none());
    }

    #[test]
    fn flat_spot_yields_zero_gamma_pnl() {
        let spots = vec![100.0_f64; 50];
        let r = simulate(&spots, 0.01, -0.05, 0.0, 252.0).unwrap();
        assert!(r.total_gamma_pnl.abs() < 1e-12);
        // P&L is pure theta drain.
        assert!((r.total_pnl - r.total_theta_pnl).abs() < 1e-9);
    }

    #[test]
    fn realized_vol_matches_log_return_estimate() {
        let spots: Vec<f64> = (0..50).map(|i| 100.0 * (1.0 + 0.001 * i as f64)).collect();
        let r = simulate(&spots, 0.01, -0.05, 0.0, 252.0).unwrap();
        // Manual check.
        let log_returns: Vec<f64> = (1..spots.len()).map(|i| (spots[i] / spots[i - 1]).ln()).collect();
        let var_per_step: f64 = log_returns.iter().map(|x| x * x).sum::<f64>()
            / log_returns.len() as f64;
        let expected = (var_per_step * 252.0).sqrt();
        assert!((r.realized_vol_annualized - expected).abs() < 1e-9);
    }

    #[test]
    fn long_gamma_positive_realized_yields_positive_gamma_pnl() {
        let mut state: u64 = 42;
        let spots: Vec<f64> = (0..200).scan(100.0_f64, |s, _| {
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let r = ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 0.02;
            *s *= 1.0 + r;
            Some(*s)
        }).collect();
        let r = simulate(&spots, 0.01, 0.0, 0.0, 252.0).unwrap();
        assert!(r.total_gamma_pnl > 0.0);
    }

    #[test]
    fn transaction_cost_subtracts_from_pnl() {
        let mut state: u64 = 11;
        let spots: Vec<f64> = (0..100).scan(100.0_f64, |s, _| {
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let r = ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 0.02;
            *s *= 1.0 + r;
            Some(*s)
        }).collect();
        let no_tc = simulate(&spots, 0.01, 0.0, 0.0, 252.0).unwrap();
        let with_tc = simulate(&spots, 0.01, 0.0, 0.001, 252.0).unwrap();
        assert!(with_tc.total_pnl < no_tc.total_pnl);
        assert!(with_tc.total_transaction_cost > 0.0);
    }

    #[test]
    fn breakeven_iv_positive_when_costs_exist() {
        let spots = vec![100.0_f64; 30];
        let r = simulate(&spots, 0.01, -0.05, 0.001, 252.0).unwrap();
        assert!(r.breakeven_iv_annualized > 0.0);
    }

    #[test]
    fn cumulative_pnl_length_matches_per_step() {
        let spots = vec![100.0_f64; 30];
        let r = simulate(&spots, 0.01, -0.05, 0.0, 252.0).unwrap();
        assert_eq!(r.per_step_pnl.len(), 29);
        assert_eq!(r.cumulative_pnl.len(), 29);
        assert_eq!(r.n_steps, 29);
    }
}
