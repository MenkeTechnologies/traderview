//! Tax-equivalent yield — compares a tax-free municipal bond to a taxable bond
//! on an after-tax basis.
//!
//! Muni interest is always free of federal tax (and the 3.8% NIIT); it's also
//! free of state tax when the bond is issued in your state. So:
//!
//! ```text
//! muni after-tax    = muni yield × (1 − state, if out-of-state)
//! TEY               = muni after-tax / (1 − combined taxable rate)
//! taxable after-tax = taxable yield × (1 − combined taxable rate)
//! ```
//!
//! TEY is the yield a taxable bond would need to match the muni after tax.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct TeyInput {
    /// Municipal bond yield, percent.
    pub muni_yield_pct: f64,
    /// A taxable bond yield to compare against, percent (0 to skip the verdict).
    #[serde(default)]
    pub taxable_yield_pct: f64,
    /// Marginal federal income-tax rate, percent.
    pub federal_rate_pct: f64,
    /// Marginal state income-tax rate, percent.
    #[serde(default)]
    pub state_rate_pct: f64,
    /// The muni is issued in your state (so it's state-tax-free too).
    #[serde(default)]
    pub in_state: bool,
    /// Net investment income tax (3.8%) applies to the taxable bond's interest.
    #[serde(default)]
    pub niit_applies: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TeyResult {
    /// Combined rate taxing the taxable bond (federal + state + NIIT), percent.
    pub combined_taxable_rate_pct: f64,
    /// Yield a taxable bond must offer to match the muni after tax.
    pub tax_equivalent_yield_pct: f64,
    /// Muni yield net of any state tax (federal/NIIT always exempt).
    pub muni_after_tax_pct: f64,
    /// Taxable bond yield net of the combined rate; `None` if none supplied.
    pub taxable_after_tax_pct: Option<f64>,
    /// "muni", "taxable", or "equal"; `None` if no taxable yield supplied.
    pub verdict: Option<String>,
}

pub fn analyze(input: &TeyInput) -> TeyResult {
    let federal = input.federal_rate_pct / 100.0;
    let state = input.state_rate_pct / 100.0;
    let niit = if input.niit_applies { 0.038 } else { 0.0 };

    let combined = (federal + state + niit).min(0.9999);
    let state_on_muni = if input.in_state { 0.0 } else { state };

    let muni_after_tax = input.muni_yield_pct * (1.0 - state_on_muni);
    let tey = muni_after_tax / (1.0 - combined);

    let (taxable_after_tax, verdict) = if input.taxable_yield_pct > 0.0 {
        let tat = input.taxable_yield_pct * (1.0 - combined);
        let v = if (muni_after_tax - tat).abs() < 1e-9 {
            "equal"
        } else if muni_after_tax > tat {
            "muni"
        } else {
            "taxable"
        };
        (Some(tat), Some(v.to_string()))
    } else {
        (None, None)
    };

    TeyResult {
        combined_taxable_rate_pct: combined * 100.0,
        tax_equivalent_yield_pct: tey,
        muni_after_tax_pct: muni_after_tax,
        taxable_after_tax_pct: taxable_after_tax,
        verdict,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-4
    }

    fn run(muni: f64, taxable: f64, fed: f64, state: f64, in_state: bool, niit: bool) -> TeyResult {
        analyze(&TeyInput {
            muni_yield_pct: muni,
            taxable_yield_pct: taxable,
            federal_rate_pct: fed,
            state_rate_pct: state,
            in_state,
            niit_applies: niit,
        })
    }

    #[test]
    fn tey_basic() {
        // 3.5 / (1 − 0.32) = 5.147059.
        let r = run(3.5, 5.0, 32.0, 0.0, false, false);
        assert!(close(r.tax_equivalent_yield_pct, 5.147059));
    }

    #[test]
    fn taxable_after_tax() {
        let r = run(3.5, 5.0, 32.0, 0.0, false, false);
        assert!(close(r.taxable_after_tax_pct.unwrap(), 3.4));
    }

    #[test]
    fn verdict_muni_wins() {
        // Muni nets 3.5 vs taxable 3.4.
        let r = run(3.5, 5.0, 32.0, 0.0, false, false);
        assert_eq!(r.verdict.unwrap(), "muni");
    }

    #[test]
    fn verdict_taxable_wins() {
        // Taxable 8% nets 5.44 > muni 3.5.
        let r = run(3.5, 8.0, 32.0, 0.0, false, false);
        assert!(close(r.taxable_after_tax_pct.unwrap(), 5.44));
        assert_eq!(r.verdict.unwrap(), "taxable");
    }

    #[test]
    fn in_state_raises_tey() {
        let out = run(3.5, 5.0, 32.0, 5.0, false, false);
        let within = run(3.5, 5.0, 32.0, 5.0, true, false);
        // In-state muni keeps full yield and faces a higher combined rate → higher TEY.
        assert!(within.tax_equivalent_yield_pct > out.tax_equivalent_yield_pct);
        assert!(close(within.tax_equivalent_yield_pct, 3.5 / (1.0 - 0.37)));
    }

    #[test]
    fn out_of_state_taxes_muni() {
        let r = run(3.5, 5.0, 32.0, 5.0, false, false);
        // Out-of-state muni pays state tax: 3.5 × 0.95 = 3.325.
        assert!(close(r.muni_after_tax_pct, 3.325));
    }

    #[test]
    fn niit_raises_combined_rate() {
        let r = run(3.5, 5.0, 32.0, 0.0, false, true);
        assert!(close(r.combined_taxable_rate_pct, 35.8));
        assert!(close(r.tax_equivalent_yield_pct, 3.5 / (1.0 - 0.358)));
    }

    #[test]
    fn zero_federal_no_benefit() {
        let r = run(3.5, 5.0, 0.0, 0.0, false, false);
        assert!(close(r.tax_equivalent_yield_pct, 3.5));
    }
}
