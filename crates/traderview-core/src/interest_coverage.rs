//! Interest- and fixed-charge-coverage ratios — corporate solvency from the
//! income statement (distinct from DSCR, which covers property debt service
//! including principal).
//!
//! ```text
//! EBITDA                  = EBIT + depreciation & amortization
//! Times interest earned   = EBIT / interest expense
//! EBITDA interest coverage = EBITDA / interest expense
//! Fixed-charge coverage   = (EBIT + lease) / (interest + lease)
//! ```
//!
//! Times-interest-earned above ~2.5 is generally comfortable; below ~1.5 the
//! firm is straining to cover its interest.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct CoverageInput {
    /// Earnings before interest and taxes.
    pub ebit_usd: f64,
    /// Depreciation and amortization (added back for EBITDA).
    #[serde(default)]
    pub depreciation_amortization_usd: f64,
    /// Interest expense for the period.
    pub interest_expense_usd: f64,
    /// Lease/rental payments, the fixed charge for FCCR. Optional.
    #[serde(default)]
    pub lease_payments_usd: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CoverageResult {
    /// EBIT + D&A.
    pub ebitda_usd: f64,
    /// EBIT / interest; `None` if interest ≤ 0.
    pub times_interest_earned: Option<f64>,
    /// EBITDA / interest; `None` if interest ≤ 0.
    pub ebitda_interest_coverage: Option<f64>,
    /// (EBIT + lease) / (interest + lease); `None` if that denominator ≤ 0.
    pub fixed_charge_coverage: Option<f64>,
    /// Whether EBIT covers interest at least once (TIE ≥ 1).
    pub covers_interest: bool,
}

pub fn analyze(input: &CoverageInput) -> CoverageResult {
    let ebitda = input.ebit_usd + input.depreciation_amortization_usd;

    let times_interest_earned = if input.interest_expense_usd > 0.0 {
        Some(input.ebit_usd / input.interest_expense_usd)
    } else {
        None
    };
    let ebitda_interest_coverage = if input.interest_expense_usd > 0.0 {
        Some(ebitda / input.interest_expense_usd)
    } else {
        None
    };

    let fccr_denominator = input.interest_expense_usd + input.lease_payments_usd;
    let fixed_charge_coverage = if fccr_denominator > 0.0 {
        Some((input.ebit_usd + input.lease_payments_usd) / fccr_denominator)
    } else {
        None
    };

    let covers_interest = times_interest_earned.map(|t| t >= 1.0).unwrap_or(false);

    CoverageResult {
        ebitda_usd: ebitda,
        times_interest_earned,
        ebitda_interest_coverage,
        fixed_charge_coverage,
        covers_interest,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6
    }

    fn run(ebit: f64, da: f64, interest: f64, lease: f64) -> CoverageResult {
        analyze(&CoverageInput {
            ebit_usd: ebit,
            depreciation_amortization_usd: da,
            interest_expense_usd: interest,
            lease_payments_usd: lease,
        })
    }

    #[test]
    fn ebitda_is_ebit_plus_da() {
        let r = run(500.0, 100.0, 200.0, 50.0);
        assert!(close(r.ebitda_usd, 600.0));
    }

    #[test]
    fn times_interest_earned() {
        // 500 / 200 = 2.5.
        let r = run(500.0, 100.0, 200.0, 50.0);
        assert!(close(r.times_interest_earned.unwrap(), 2.5));
    }

    #[test]
    fn ebitda_coverage() {
        // 600 / 200 = 3.0.
        let r = run(500.0, 100.0, 200.0, 50.0);
        assert!(close(r.ebitda_interest_coverage.unwrap(), 3.0));
    }

    #[test]
    fn fixed_charge_coverage() {
        // (500 + 50) / (200 + 50) = 550 / 250 = 2.2.
        let r = run(500.0, 100.0, 200.0, 50.0);
        assert!(close(r.fixed_charge_coverage.unwrap(), 2.2));
    }

    #[test]
    fn covers_interest_true_when_tie_ge_1() {
        let r = run(500.0, 100.0, 200.0, 0.0);
        assert!(r.covers_interest);
    }

    #[test]
    fn does_not_cover_when_ebit_below_interest() {
        let r = run(150.0, 50.0, 200.0, 0.0);
        assert!(close(r.times_interest_earned.unwrap(), 0.75));
        assert!(!r.covers_interest);
    }

    #[test]
    fn zero_interest_guards_ratios() {
        let r = run(500.0, 100.0, 0.0, 0.0);
        assert!(r.times_interest_earned.is_none());
        assert!(r.ebitda_interest_coverage.is_none());
        assert!(!r.covers_interest);
        // FCCR denominator is also zero with no lease.
        assert!(r.fixed_charge_coverage.is_none());
    }

    #[test]
    fn fccr_defined_by_lease_even_without_interest() {
        // No interest but lease present → FCCR still defined.
        let r = run(500.0, 100.0, 0.0, 50.0);
        assert!(r.times_interest_earned.is_none());
        assert!(close(r.fixed_charge_coverage.unwrap(), (500.0 + 50.0) / 50.0));
    }
}
