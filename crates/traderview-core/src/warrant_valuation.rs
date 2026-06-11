//! Warrant valuation — Galai & Schneller (1978) dilution adjustment.
//!
//! A warrant differs from a call: exercise mints M new shares against
//! N outstanding, diluting the very stock it delivers, and the strike
//! proceeds flow into the firm. The classic fixed point:
//!
//!   W = (N / (N + M)) · C(S + (M/N)·W, K, T, r, q, σ)
//!
//! where C is Black-Scholes — the warrant's own value feeds back into
//! the asset value per share. Solved by direct iteration (a
//! contraction here; converges in a handful of steps). M = 0 reduces
//! to a plain call.
//!
//! Pure compute. Companion to `black_scholes`, `convertible_bond`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct WarrantInput {
    pub spot: f64,
    pub strike: f64,
    pub time_to_expiry_years: f64,
    pub risk_free_rate: f64,
    #[serde(default)]
    pub dividend_yield: f64,
    pub volatility: f64,
    /// Shares currently outstanding.
    pub shares_outstanding: f64,
    /// New shares minted if all warrants exercise.
    pub warrants_outstanding: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct WarrantReport {
    pub warrant_value: f64,
    /// The same option priced as a plain call — the dilution-free
    /// upper bound.
    pub plain_call_value: f64,
    /// 1 − W/C: how much the dilution costs.
    pub dilution_discount_pct: f64,
    pub dilution_factor: f64,
    pub iterations: u32,
}

pub fn compute(inp: &WarrantInput) -> Option<WarrantReport> {
    if ![
        inp.spot,
        inp.strike,
        inp.time_to_expiry_years,
        inp.risk_free_rate,
        inp.dividend_yield,
        inp.volatility,
        inp.shares_outstanding,
        inp.warrants_outstanding,
    ]
    .iter()
    .all(|v| v.is_finite())
        || inp.spot <= 0.0
        || inp.strike <= 0.0
        || inp.time_to_expiry_years <= 0.0
        || inp.volatility <= 0.0
        || inp.shares_outstanding <= 0.0
        || inp.warrants_outstanding < 0.0
    {
        return None;
    }
    let bs = |s: f64| {
        crate::black_scholes::call(
            s,
            inp.strike,
            inp.time_to_expiry_years,
            inp.risk_free_rate,
            inp.dividend_yield,
            inp.volatility,
        )
    };
    let plain = bs(inp.spot);
    let n = inp.shares_outstanding;
    let m = inp.warrants_outstanding;
    let factor = n / (n + m);
    let mut w = factor * plain; // first guess
    let mut iterations = 0;
    for _ in 0..200 {
        iterations += 1;
        let next = factor * bs(inp.spot + (m / n) * w);
        if (next - w).abs() < 1e-10 {
            w = next;
            break;
        }
        w = next;
    }
    Some(WarrantReport {
        warrant_value: w,
        plain_call_value: plain,
        dilution_discount_pct: if plain > 0.0 {
            (1.0 - w / plain) * 100.0
        } else {
            0.0
        },
        dilution_factor: factor,
        iterations,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> WarrantInput {
        WarrantInput {
            spot: 100.0,
            strike: 100.0,
            time_to_expiry_years: 1.0,
            risk_free_rate: 0.05,
            dividend_yield: 0.0,
            volatility: 0.2,
            shares_outstanding: 100.0,
            warrants_outstanding: 20.0,
        }
    }

    #[test]
    fn zero_warrants_reduce_to_plain_black_scholes() {
        let mut inp = base();
        inp.warrants_outstanding = 0.0;
        let r = compute(&inp).unwrap();
        assert!((r.warrant_value - r.plain_call_value).abs() < 1e-12);
        assert!(r.dilution_discount_pct.abs() < 1e-9);
        // ATM 1y 5%/20% canonical BS ≈ 10.4506.
        assert!((r.plain_call_value - 10.4506).abs() < 1e-3);
    }

    #[test]
    fn fixed_point_satisfies_the_galai_schneller_equation() {
        let r = compute(&base()).unwrap();
        let lhs = r.warrant_value;
        let rhs = (100.0 / 120.0)
            * crate::black_scholes::call(
                100.0 + (20.0 / 100.0) * r.warrant_value,
                100.0,
                1.0,
                0.05,
                0.0,
                0.2,
            );
        assert!((lhs - rhs).abs() < 1e-8, "{lhs} vs {rhs}");
    }

    #[test]
    fn dilution_discounts_but_proceeds_partially_offset() {
        // The warrant must be worth less than the dilution-free call
        // but more than the naive factor × call (the exercise proceeds
        // raise the per-share asset value).
        let r = compute(&base()).unwrap();
        assert!(r.warrant_value < r.plain_call_value);
        assert!(r.warrant_value > r.dilution_factor * r.plain_call_value);
        assert!(r.dilution_discount_pct > 0.0 && r.dilution_discount_pct < 100.0 / 6.0 + 1e-9);
    }

    #[test]
    fn more_warrants_mean_deeper_discount() {
        let light = compute(&base()).unwrap();
        let mut heavy_inp = base();
        heavy_inp.warrants_outstanding = 50.0;
        let heavy = compute(&heavy_inp).unwrap();
        assert!(heavy.dilution_discount_pct > light.dilution_discount_pct);
        assert!(heavy.warrant_value < light.warrant_value);
    }

    #[test]
    fn hostile_inputs_return_none() {
        let mut bad = base();
        bad.spot = 0.0;
        assert!(compute(&bad).is_none());
        let mut bad = base();
        bad.volatility = 0.0;
        assert!(compute(&bad).is_none());
        let mut bad = base();
        bad.shares_outstanding = 0.0;
        assert!(compute(&bad).is_none());
        let mut bad = base();
        bad.warrants_outstanding = -1.0;
        assert!(compute(&bad).is_none());
        let mut bad = base();
        bad.time_to_expiry_years = f64::NAN;
        assert!(compute(&bad).is_none());
    }
}
