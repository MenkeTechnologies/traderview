//! Backtest → Monte Carlo bridge: resample a strategy's ACTUAL trade
//! PnLs (with replacement) into thousands of alternate orderings and
//! report the resulting ending-equity / max-drawdown distributions.
//!
//! Why: a single backtest shows ONE ordering of the trades; its max
//! drawdown is one draw from a distribution. Resequencing answers
//! "what drawdown should I budget for if these same trades arrive in a
//! different order" — Pardo's standard post-backtest step.
//!
//! The shared monte_carlo core treats each drawn value as a DOLLAR
//! equity increment (`equity += r`), so this bridge feeds per-trade
//! dollar PnL, not R-multiples.

use crate::monte_carlo::{simulate, McConfig, McReport};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McBridgeParams {
    pub n_curves: usize,
    /// Trades per synthetic curve; None = the backtest's own count.
    pub trades_per_curve: Option<usize>,
    /// Ruin = equity falling to this fraction of start (0.5 = half).
    pub ruin_fraction: f64,
    pub seed: u64,
}

impl Default for McBridgeParams {
    fn default() -> Self {
        Self {
            n_curves: 1000,
            trades_per_curve: None,
            ruin_fraction: 0.5,
            seed: 42,
        }
    }
}

/// Resample trade PnLs into the MC engine. Non-finite PnLs are
/// dropped; None when nothing usable remains or params are degenerate.
pub fn from_pnls(pnls: &[f64], start_equity: f64, p: &McBridgeParams) -> Option<McReport> {
    let clean: Vec<f64> = pnls.iter().copied().filter(|v| v.is_finite()).collect();
    if clean.is_empty() || start_equity <= 0.0 || !(0.0..1.0).contains(&p.ruin_fraction) {
        return None;
    }
    let cfg = McConfig {
        n_curves: p.n_curves,
        trades_per_curve: p.trades_per_curve.unwrap_or(clean.len()),
        start_equity,
        ruin_threshold: start_equity * p.ruin_fraction,
        seed: p.seed,
    };
    simulate(&clean, &cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constant_winner_pins_exact_ending_equity() {
        // Every draw is +100: ending equity is exact regardless of the
        // RNG path — start + 100 × trades_per_curve.
        let p = McBridgeParams {
            n_curves: 50,
            trades_per_curve: Some(20),
            ..McBridgeParams::default()
        };
        let r = from_pnls(&[100.0], 10_000.0, &p).unwrap();
        assert_eq!(r.ending_equity_p05, 12_000.0);
        assert_eq!(r.ending_equity_p95, 12_000.0);
        assert_eq!(r.probability_of_ruin, 0.0);
        assert_eq!(r.probability_profitable, 1.0);
    }

    #[test]
    fn constant_loser_hits_ruin_with_certainty() {
        // -600 per trade from 10k with ruin at 50%: equity crosses
        // 5,000 by trade 9 on every curve.
        let p = McBridgeParams {
            n_curves: 50,
            trades_per_curve: Some(10),
            ..McBridgeParams::default()
        };
        let r = from_pnls(&[-600.0], 10_000.0, &p).unwrap();
        assert_eq!(r.probability_of_ruin, 1.0);
        assert_eq!(r.probability_profitable, 0.0);
    }

    #[test]
    fn same_seed_reproduces_distribution_exactly() {
        let pnls = [250.0, -120.0, 90.0, -300.0, 410.0];
        let p = McBridgeParams::default();
        let a = from_pnls(&pnls, 50_000.0, &p).unwrap();
        let b = from_pnls(&pnls, 50_000.0, &p).unwrap();
        assert_eq!(a.ending_equity_p50, b.ending_equity_p50);
        assert_eq!(a.max_drawdown_p95, b.max_drawdown_p95);
        // A different seed moves the median (sanity that seed matters).
        let c = from_pnls(&pnls, 50_000.0, &McBridgeParams { seed: 7, ..p }).unwrap();
        assert!(c.ending_equity_p50 != a.ending_equity_p50 || c.max_drawdown_p95 != a.max_drawdown_p95);
    }

    #[test]
    fn degenerate_inputs_are_none() {
        let p = McBridgeParams::default();
        assert!(from_pnls(&[], 10_000.0, &p).is_none());
        assert!(from_pnls(&[f64::NAN], 10_000.0, &p).is_none());
        assert!(from_pnls(&[100.0], 0.0, &p).is_none());
        assert!(from_pnls(
            &[100.0],
            10_000.0,
            &McBridgeParams { ruin_fraction: 1.5, ..McBridgeParams::default() }
        )
        .is_none());
    }
}
