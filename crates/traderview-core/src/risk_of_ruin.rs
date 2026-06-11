//! Analytic risk of ruin — closed-form complement to the Monte-Carlo
//! equity simulator (`monte_carlo`).
//!
//! Model each trade as a two-outcome walk in risk units: lose 1 unit
//! with probability q = 1 − p, win `payoff_ratio` units with
//! probability p. Ruin = cumulative P/L reaching −U units, where
//! U = capital / risk_per_trade.
//!
//! The classical characteristic-equation method: find z₀ ∈ (0, 1) with
//!
//!   p·z₀^R + q·z₀^{−1} = 1        (R = payoff_ratio)
//!
//! then RoR = z₀^U. For R = 1 this reduces to the textbook gambler's
//! ruin z₀ = q/p. Systems with non-positive expectancy are ruined with
//! probability 1 given enough trades.
//!
//! Pure compute. z₀ found by bisection (the z = 1 root is excluded).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct RuinInput {
    /// Win probability per trade, (0, 1).
    pub win_probability: f64,
    /// Units won per win for every 1 unit risked (R multiple).
    pub payoff_ratio: f64,
    /// Account capital, $.
    pub capital: f64,
    /// Dollar risk per trade (the 1-unit loss).
    pub risk_per_trade: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct RuinReport {
    /// Capital expressed in 1-loss risk units.
    pub risk_units: f64,
    /// Per-trade expectancy in risk units: p·R − q.
    pub expectancy_r: f64,
    /// Root of the characteristic equation (1.0 for non-positive edge).
    pub z0: f64,
    /// Probability of ever drawing the account down to zero.
    pub risk_of_ruin: f64,
    /// Full-Kelly fraction p − q/R (clamped at 0).
    pub kelly_fraction: f64,
}

pub fn compute(inp: &RuinInput) -> Option<RuinReport> {
    if !inp.win_probability.is_finite()
        || !(inp.win_probability > 0.0 && inp.win_probability < 1.0)
        || !inp.payoff_ratio.is_finite()
        || inp.payoff_ratio <= 0.0
        || !inp.capital.is_finite()
        || inp.capital <= 0.0
        || !inp.risk_per_trade.is_finite()
        || inp.risk_per_trade <= 0.0
        || inp.risk_per_trade > inp.capital
    {
        return None;
    }
    let p = inp.win_probability;
    let q = 1.0 - p;
    let r = inp.payoff_ratio;
    let units = inp.capital / inp.risk_per_trade;
    let expectancy = p * r - q;
    let kelly = (p - q / r).max(0.0);
    if expectancy <= 0.0 {
        // Negative or zero edge: ruin is certain in the limit.
        return Some(RuinReport {
            risk_units: units,
            expectancy_r: expectancy,
            z0: 1.0,
            risk_of_ruin: 1.0,
            kelly_fraction: kelly,
        });
    }
    // f(z) = p·z^R + q/z − 1: f(0⁺) → +∞, f(1) = 0 with f'(1) > 0 for
    // positive edge, so the interior root has f < 0 just left of 1.
    let f = |z: f64| p * z.powf(r) + q / z - 1.0;
    let (mut lo, mut hi) = (1e-12, 1.0 - 1e-12);
    if f(hi) >= 0.0 {
        // Numerical corner: edge so small the bracket collapses.
        return Some(RuinReport {
            risk_units: units,
            expectancy_r: expectancy,
            z0: 1.0,
            risk_of_ruin: 1.0,
            kelly_fraction: kelly,
        });
    }
    for _ in 0..200 {
        let mid = (lo + hi) / 2.0;
        if f(mid) > 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let z0 = (lo + hi) / 2.0;
    Some(RuinReport {
        risk_units: units,
        expectancy_r: expectancy,
        z0,
        risk_of_ruin: z0.powf(units).clamp(0.0, 1.0),
        kelly_fraction: kelly,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inp(p: f64, r: f64, capital: f64, risk: f64) -> RuinInput {
        RuinInput {
            win_probability: p,
            payoff_ratio: r,
            capital,
            risk_per_trade: risk,
        }
    }

    #[test]
    fn even_payoff_reduces_to_textbook_gamblers_ruin() {
        // p = 0.6, R = 1 ⇒ z₀ = q/p = 2/3; U = 10 ⇒ RoR = (2/3)^10.
        let rep = compute(&inp(0.6, 1.0, 10_000.0, 1_000.0)).unwrap();
        assert!((rep.z0 - 2.0 / 3.0).abs() < 1e-9, "{}", rep.z0);
        let want = (2.0_f64 / 3.0).powi(10);
        assert!((rep.risk_of_ruin - want).abs() < 1e-9);
        assert!((rep.expectancy_r - 0.2).abs() < 1e-12);
        assert!((rep.kelly_fraction - 0.2).abs() < 1e-12);
    }

    #[test]
    fn unequal_payoff_root_matches_hand_solved_quadratic() {
        // p=0.4, R=2: 0.4z³ − z + 0.6 = 0 factors as (z−1)(0.4z² +
        // 0.4z − 0.6) ⇒ z₀ = (−0.4 + √1.12)/0.8.
        let rep = compute(&inp(0.4, 2.0, 20_000.0, 1_000.0)).unwrap();
        let want = (-0.4 + 1.12_f64.sqrt()) / 0.8;
        assert!((rep.z0 - want).abs() < 1e-9, "{} vs {want}", rep.z0);
        assert!((rep.risk_of_ruin - want.powi(20)).abs() < 1e-9);
    }

    #[test]
    fn no_edge_means_certain_ruin() {
        let coin = compute(&inp(0.5, 1.0, 10_000.0, 100.0)).unwrap();
        assert_eq!(coin.risk_of_ruin, 1.0);
        let negative = compute(&inp(0.4, 1.0, 10_000.0, 100.0)).unwrap();
        assert_eq!(negative.risk_of_ruin, 1.0);
    }

    #[test]
    fn ruin_falls_with_more_capital_and_better_odds() {
        let small = compute(&inp(0.55, 1.0, 5_000.0, 1_000.0)).unwrap();
        let large = compute(&inp(0.55, 1.0, 50_000.0, 1_000.0)).unwrap();
        assert!(large.risk_of_ruin < small.risk_of_ruin);
        let better = compute(&inp(0.65, 1.0, 5_000.0, 1_000.0)).unwrap();
        assert!(better.risk_of_ruin < small.risk_of_ruin);
    }

    #[test]
    fn fractional_units_supported() {
        // 2.5% risk ⇒ 40 units; RoR = (q/p)^40 at R=1.
        let rep = compute(&inp(0.55, 1.0, 10_000.0, 250.0)).unwrap();
        let z = 0.45_f64 / 0.55;
        assert!((rep.risk_of_ruin - z.powf(40.0)).abs() < 1e-9);
    }

    #[test]
    fn hostile_inputs_return_none() {
        assert!(compute(&inp(0.0, 1.0, 10_000.0, 100.0)).is_none());
        assert!(compute(&inp(1.0, 1.0, 10_000.0, 100.0)).is_none());
        assert!(compute(&inp(0.6, 0.0, 10_000.0, 100.0)).is_none());
        assert!(compute(&inp(0.6, 1.0, 0.0, 100.0)).is_none());
        assert!(compute(&inp(0.6, 1.0, 100.0, 200.0)).is_none()); // risk > capital
        assert!(compute(&inp(f64::NAN, 1.0, 10_000.0, 100.0)).is_none());
    }
}
