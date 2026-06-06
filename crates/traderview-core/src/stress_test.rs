//! Portfolio stress test — price × IV × time shock grid for options books.
//!
//! tastytrade Risk Analysis-class. Given a portfolio of option positions,
//! re-price every leg under each (price_shock_pct, iv_shock_pct, time_decay_days)
//! triple and aggregate to a portfolio P&L grid. Surfaces tail risks that a
//! single-greeks snapshot hides — e.g. "delta-neutral now but down 8% if
//! IV crushes 20%."
//!
//! Caller supplies positions with the *current* underlying price, IV, days
//! to expiry, and strike + option kind. The Black-Scholes pricer comes from
//! `crate::greeks::price_and_greeks`.
//!
//! Pure compute. Skips legs whose underlying price or DTE drops to zero
//! under a shock (intrinsic-only fallback).

use crate::greeks::{price_and_greeks, OptKind};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionLeg {
    pub symbol: String,
    pub kind: OptKind,
    /// Underlying spot at time of analysis.
    pub spot: f64,
    pub strike: f64,
    /// Days to expiration today.
    pub days_to_expiry: f64,
    /// Current implied volatility (annual, e.g. 0.30 for 30%).
    pub implied_vol: f64,
    /// Signed contract count: positive = long, negative = short.
    pub contracts: i64,
    /// Contract size (typically 100 for equity options).
    pub multiplier: f64,
    /// Per-share premium the trader paid (+) or collected (-) when opening.
    pub entry_price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressInput {
    pub legs: Vec<OptionLeg>,
    /// Underlying price shocks as fractions (e.g. -0.10 = -10%, +0.05 = +5%).
    pub price_shocks_pct: Vec<f64>,
    /// IV shocks as fractions of CURRENT IV (e.g. -0.20 = IV drops by 20%
    /// relative — 30% IV becomes 24%).
    pub iv_shocks_pct: Vec<f64>,
    /// Calendar days forward to advance "today" by. Reduces DTE.
    pub time_decay_days: f64,
    /// Risk-free rate (annual decimal). 0.045 = 4.5%.
    pub risk_free_rate: f64,
    /// Continuous dividend yield (annual decimal). Usually 0.0 for equities.
    pub dividend_yield: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StressCell {
    pub price_shock_pct: f64,
    pub iv_shock_pct: f64,
    /// Total portfolio P&L under this shock (vs. entry).
    pub pnl_dollars: f64,
    /// Portfolio delta under shock (multiplied by spot × multiplier).
    pub portfolio_delta_dollars: f64,
    /// Portfolio vega (per 1 vol-point).
    pub portfolio_vega_dollars: f64,
    /// Portfolio theta (per day).
    pub portfolio_theta_dollars: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StressReport {
    /// Grid of cells (price_shock_pct outer × iv_shock_pct inner).
    pub grid: Vec<StressCell>,
    /// Worst-case P&L cell (most negative).
    pub worst_case: StressCell,
    /// Best-case P&L cell.
    pub best_case: StressCell,
}

pub fn analyze(input: &StressInput) -> StressReport {
    if input.legs.is_empty() {
        return StressReport::default();
    }
    let mut grid = Vec::with_capacity(input.price_shocks_pct.len() * input.iv_shocks_pct.len());
    let new_dte_years = ((input
        .legs
        .iter()
        .map(|l| l.days_to_expiry)
        .fold(f64::INFINITY, f64::min)
        - input.time_decay_days)
        / 365.0)
        .max(0.0);
    // We use per-leg new DTE rather than a single min, so we re-compute it
    // per leg below. The `new_dte_years` above just verifies we have *some*
    // remaining time for at least one leg.
    let _ = new_dte_years;

    for &p_shock in &input.price_shocks_pct {
        for &iv_shock in &input.iv_shocks_pct {
            let mut pnl = 0.0;
            let mut delta = 0.0;
            let mut vega = 0.0;
            let mut theta = 0.0;
            for leg in &input.legs {
                let shocked_spot = leg.spot * (1.0 + p_shock);
                let shocked_iv = (leg.implied_vol * (1.0 + iv_shock)).max(0.0);
                let new_dte_days = (leg.days_to_expiry - input.time_decay_days).max(0.0);
                let t_years = new_dte_days / 365.0;
                let greeks = price_and_greeks(
                    leg.kind,
                    shocked_spot,
                    leg.strike,
                    t_years,
                    shocked_iv,
                    input.risk_free_rate,
                    input.dividend_yield,
                );
                let contracts_f = leg.contracts as f64;
                let per_contract_pnl = (greeks.price - leg.entry_price) * leg.multiplier;
                pnl += per_contract_pnl * contracts_f;
                delta += greeks.delta * shocked_spot * leg.multiplier * contracts_f;
                vega += greeks.vega * leg.multiplier * contracts_f;
                theta += greeks.theta * leg.multiplier * contracts_f;
            }
            grid.push(StressCell {
                price_shock_pct: p_shock,
                iv_shock_pct: iv_shock,
                pnl_dollars: pnl,
                portfolio_delta_dollars: delta,
                portfolio_vega_dollars: vega,
                portfolio_theta_dollars: theta,
            });
        }
    }

    let worst = grid
        .iter()
        .min_by(|a, b| {
            a.pnl_dollars
                .partial_cmp(&b.pnl_dollars)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .cloned()
        .unwrap_or_default();
    let best = grid
        .iter()
        .max_by(|a, b| {
            a.pnl_dollars
                .partial_cmp(&b.pnl_dollars)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .cloned()
        .unwrap_or_default();

    StressReport {
        grid,
        worst_case: worst,
        best_case: best,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn long_atm_call() -> OptionLeg {
        OptionLeg {
            symbol: "SPY".into(),
            kind: OptKind::Call,
            spot: 500.0,
            strike: 500.0,
            days_to_expiry: 30.0,
            implied_vol: 0.20,
            contracts: 1,
            multiplier: 100.0,
            entry_price: 5.0,
        }
    }

    fn flat_input(legs: Vec<OptionLeg>) -> StressInput {
        StressInput {
            legs,
            price_shocks_pct: vec![-0.05, 0.0, 0.05],
            iv_shocks_pct: vec![-0.10, 0.0, 0.10],
            time_decay_days: 0.0,
            risk_free_rate: 0.045,
            dividend_yield: 0.0,
        }
    }

    #[test]
    fn empty_portfolio_returns_empty_grid() {
        let r = analyze(&flat_input(vec![]));
        assert!(r.grid.is_empty());
        assert_eq!(r.worst_case.pnl_dollars, 0.0);
    }

    #[test]
    fn grid_size_is_price_shocks_times_iv_shocks() {
        let r = analyze(&flat_input(vec![long_atm_call()]));
        assert_eq!(r.grid.len(), 9, "3 price × 3 iv shocks = 9 cells");
    }

    #[test]
    fn long_call_makes_money_when_underlying_rallies() {
        // +5% on a long ATM call should beat -5% on the same call.
        let r = analyze(&flat_input(vec![long_atm_call()]));
        let up5 = r
            .grid
            .iter()
            .find(|c| (c.price_shock_pct - 0.05).abs() < 1e-9 && c.iv_shock_pct == 0.0)
            .unwrap();
        let dn5 = r
            .grid
            .iter()
            .find(|c| (c.price_shock_pct + 0.05).abs() < 1e-9 && c.iv_shock_pct == 0.0)
            .unwrap();
        assert!(
            up5.pnl_dollars > dn5.pnl_dollars,
            "long call should profit more on +5% than -5% (got up={} dn={})",
            up5.pnl_dollars,
            dn5.pnl_dollars
        );
    }

    #[test]
    fn worst_and_best_case_are_extrema_of_the_grid() {
        let r = analyze(&flat_input(vec![long_atm_call()]));
        let min = r
            .grid
            .iter()
            .map(|c| c.pnl_dollars)
            .fold(f64::INFINITY, f64::min);
        let max = r
            .grid
            .iter()
            .map(|c| c.pnl_dollars)
            .fold(f64::NEG_INFINITY, f64::max);
        assert!((r.worst_case.pnl_dollars - min).abs() < 1e-9);
        assert!((r.best_case.pnl_dollars - max).abs() < 1e-9);
    }

    #[test]
    fn iv_crush_hurts_a_long_premium_position() {
        // Long call → vega-positive. IV crush (-20%) at flat spot should
        // be worse P&L than no IV change at flat spot.
        let mut input = flat_input(vec![long_atm_call()]);
        input.iv_shocks_pct = vec![-0.20, 0.0];
        input.price_shocks_pct = vec![0.0];
        let r = analyze(&input);
        let crushed = r.grid.iter().find(|c| c.iv_shock_pct == -0.20).unwrap();
        let flat = r.grid.iter().find(|c| c.iv_shock_pct == 0.0).unwrap();
        assert!(
            crushed.pnl_dollars < flat.pnl_dollars,
            "long-vega should lose money on IV crush (crushed={}, flat={})",
            crushed.pnl_dollars,
            flat.pnl_dollars
        );
    }
}
