//! Breakeven Inflation Rate — TIPS-vs-Treasury implied inflation.
//!
//! For a maturity-matched pair of nominal Treasury and TIPS yields:
//!
//!   BEI = nominal_yield − real_yield
//!
//! The breakeven represents the constant rate of inflation at which
//! the TIPS and nominal Treasury would yield the same total return
//! over their life.
//!
//! Adjustments:
//!   - **Inflation risk premium** (IRP): nominal yield contains a
//!     premium for inflation uncertainty; subtract a user-supplied
//!     IRP estimate to get the pure inflation expectation.
//!   - **TIPS liquidity premium** (LP): TIPS are less liquid;
//!     real yield is *upward-biased*. Subtract LP from BEI.
//!
//! Adjusted BEI = BEI − IRP − LP
//!
//! Pure compute. Companion to `nelson_siegel`, `yield_curve_bootstrap`,
//! `cross_currency_basis`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BreakevenInflationReport {
    pub nominal_yield: f64,
    pub real_yield: f64,
    pub breakeven_inflation: f64,
    pub inflation_risk_premium: f64,
    pub liquidity_premium: f64,
    pub adjusted_inflation_expectation: f64,
}

pub fn compute(
    nominal_yield: f64,
    real_yield: f64,
    inflation_risk_premium: f64,
    liquidity_premium: f64,
) -> Option<BreakevenInflationReport> {
    if !nominal_yield.is_finite()
        || !real_yield.is_finite()
        || !inflation_risk_premium.is_finite()
        || !liquidity_premium.is_finite()
    {
        return None;
    }
    let bei = nominal_yield - real_yield;
    let adjusted = bei - inflation_risk_premium - liquidity_premium;
    Some(BreakevenInflationReport {
        nominal_yield,
        real_yield,
        breakeven_inflation: bei,
        inflation_risk_premium,
        liquidity_premium,
        adjusted_inflation_expectation: adjusted,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nan_inputs_return_none() {
        assert!(compute(f64::NAN, 0.01, 0.0, 0.0).is_none());
        assert!(compute(0.04, f64::NAN, 0.0, 0.0).is_none());
        assert!(compute(0.04, 0.01, f64::NAN, 0.0).is_none());
        assert!(compute(0.04, 0.01, 0.0, f64::NAN).is_none());
    }

    #[test]
    fn breakeven_equals_yield_spread() {
        let r = compute(0.045, 0.012, 0.0, 0.0).unwrap();
        assert!((r.breakeven_inflation - (0.045 - 0.012)).abs() < 1e-12);
    }

    #[test]
    fn adjusted_subtracts_irp_and_lp() {
        let r = compute(0.045, 0.012, 0.005, 0.003).unwrap();
        let expected = 0.045 - 0.012 - 0.005 - 0.003;
        assert!((r.adjusted_inflation_expectation - expected).abs() < 1e-12);
    }

    #[test]
    fn negative_breakeven_when_real_above_nominal() {
        // Deflationary regime: real yield above nominal.
        let r = compute(0.02, 0.025, 0.0, 0.0).unwrap();
        assert!(r.breakeven_inflation < 0.0);
    }

    #[test]
    fn zero_adjustments_pass_through() {
        let r = compute(0.045, 0.012, 0.0, 0.0).unwrap();
        assert!((r.adjusted_inflation_expectation - r.breakeven_inflation).abs() < 1e-12);
    }

    #[test]
    fn yields_passed_through_to_report() {
        let r = compute(0.045, 0.012, 0.005, 0.003).unwrap();
        assert_eq!(r.nominal_yield, 0.045);
        assert_eq!(r.real_yield, 0.012);
        assert_eq!(r.inflation_risk_premium, 0.005);
        assert_eq!(r.liquidity_premium, 0.003);
    }
}
