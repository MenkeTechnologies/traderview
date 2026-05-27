//! Tax-equivalent yield calculator.
//!
//! For a tax-free municipal bond at yield `Y`, the equivalent taxable
//! yield required to net the same after-tax income is:
//!
//!   TEY = Y / (1 - marginal_tax_rate)
//!
//! For triple-tax-exempt munis (federal + state + local for in-state
//! residents) use the combined marginal rate.
//!
//! Inverse: for a TAXABLE yield `Y_tax`, the equivalent tax-free yield
//! that nets the same after-tax income is:
//!
//!   Y_tax_free = Y_tax × (1 - tax_rate)
//!
//! Pure compute.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeyInput {
    pub federal_marginal_rate: Decimal,
    pub state_marginal_rate: Decimal,
    pub niit_rate: Decimal,
    /// True for in-state munis (state tax also exempt).
    pub in_state_muni: bool,
    /// True if income subject to NIIT.
    pub niit_applies: bool,
}

impl TeyInput {
    /// Combined marginal rate applicable to the comparison.
    pub fn combined_rate(&self) -> Decimal {
        let mut total = self.federal_marginal_rate;
        if self.in_state_muni { total += self.state_marginal_rate; }
        if self.niit_applies { total += self.niit_rate; }
        total
    }
}

/// Given a tax-FREE yield, return the equivalent TAXABLE yield required
/// to net the same after-tax income.
pub fn tey_from_muni(muni_yield: Decimal, input: &TeyInput) -> Decimal {
    let denom = Decimal::ONE - input.combined_rate();
    if denom <= Decimal::ZERO {
        return Decimal::ZERO;    // rate ≥ 100% — undefined
    }
    muni_yield / denom
}

/// Given a TAXABLE yield, return the equivalent tax-FREE yield that
/// nets the same after-tax income.
pub fn after_tax_yield(taxable_yield: Decimal, input: &TeyInput) -> Decimal {
    taxable_yield * (Decimal::ONE - input.combined_rate())
}

/// Standalone helper: which is better given both yields?
pub fn better(taxable: Decimal, muni: Decimal, input: &TeyInput) -> BetterChoice {
    let muni_tey = tey_from_muni(muni, input);
    if muni_tey > taxable { BetterChoice::Muni }
    else if muni_tey < taxable { BetterChoice::Taxable }
    else { BetterChoice::Tie }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BetterChoice { Muni, Taxable, Tie }

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> Decimal { Decimal::from_str(s).unwrap() }

    fn fed_24() -> TeyInput {
        TeyInput {
            federal_marginal_rate: d("0.24"),
            state_marginal_rate: d("0.05"),
            niit_rate: d("0.038"),
            in_state_muni: false,
            niit_applies: false,
        }
    }

    #[test]
    fn federal_only_24pct_combined_rate() {
        assert_eq!(fed_24().combined_rate(), d("0.24"));
    }

    #[test]
    fn in_state_muni_adds_state_rate() {
        let i = TeyInput { in_state_muni: true, ..fed_24() };
        // 0.24 + 0.05 = 0.29.
        assert_eq!(i.combined_rate(), d("0.29"));
    }

    #[test]
    fn niit_adds_3_8_pct() {
        let i = TeyInput { niit_applies: true, ..fed_24() };
        assert_eq!(i.combined_rate(), d("0.278"));
    }

    #[test]
    fn tey_3pct_muni_24pct_federal_is_3_95pct() {
        // 0.03 / (1 - 0.24) = 0.03 / 0.76 ≈ 0.03947.
        let result = tey_from_muni(d("0.03"), &fed_24());
        let expected = d("0.03") / d("0.76");
        assert_eq!(result, expected);
    }

    #[test]
    fn after_tax_yield_5pct_taxable_24pct_federal_is_3_8pct() {
        // 0.05 × (1 - 0.24) = 0.038.
        let result = after_tax_yield(d("0.05"), &fed_24());
        assert_eq!(result, d("0.0380"));
    }

    #[test]
    fn better_picks_muni_when_tey_exceeds_taxable() {
        // Muni 3% @ 24% federal → TEY = 3.95%. Taxable 3.5% → muni wins.
        let choice = better(d("0.035"), d("0.03"), &fed_24());
        assert_eq!(choice, BetterChoice::Muni);
    }

    #[test]
    fn better_picks_taxable_when_taxable_exceeds_tey() {
        // Muni 3% → TEY = 3.95%. Taxable 4.5% → taxable wins.
        let choice = better(d("0.045"), d("0.03"), &fed_24());
        assert_eq!(choice, BetterChoice::Taxable);
    }

    #[test]
    fn tey_returns_zero_when_combined_rate_ge_one() {
        let crazy = TeyInput {
            federal_marginal_rate: d("1.0"),
            ..fed_24()
        };
        assert_eq!(tey_from_muni(d("0.03"), &crazy), Decimal::ZERO);
    }

    #[test]
    fn high_earner_with_state_niit_tey_higher() {
        // 35% fed + 9% state + 3.8% NIIT, in-state muni → combined 47.8%.
        let high = TeyInput {
            federal_marginal_rate: d("0.35"),
            state_marginal_rate: d("0.09"),
            niit_rate: d("0.038"),
            in_state_muni: true,
            niit_applies: true,
        };
        let tey = tey_from_muni(d("0.03"), &high);
        // 0.03 / (1 - 0.478) = 0.03 / 0.522 ≈ 0.0575.
        let expected = d("0.03") / d("0.522");
        assert_eq!(tey, expected);
        // The higher the combined rate, the higher the TEY.
        assert!(tey > tey_from_muni(d("0.03"), &fed_24()));
    }
}
