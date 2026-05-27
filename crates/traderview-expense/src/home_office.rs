//! Home office deduction — Form 8829 + simplified method.
//!
//! Two methods, IRS lets you pick higher per year:
//!
//! 1. **Simplified.** $5 per sqft × business-use sqft, capped at 300 sqft.
//!    Maximum deduction = $1,500. No depreciation recapture on sale.
//! 2. **Actual (Form 8829).** business-use % × (mortgage interest + property
//!    tax + utilities + insurance + repairs + depreciation). Higher
//!    paperwork, depreciation recapture on sale.
//!
//! We compute both and surface the larger one. The simplified-method cap
//! catches users with offices over 300 sqft — they get the cap, not
//! sqft × $5.
//!
//! Requirement (IRC §280A): regular and exclusive use as principal place
//! of business. We don't enforce this — user attests via the form.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct HomeOfficeInput {
    pub business_use_sqft: Decimal,
    pub total_home_sqft: Decimal,
    /// Actual-method inputs. Annual numbers (12 months).
    pub annual_mortgage_interest: Decimal,
    pub annual_property_tax: Decimal,
    pub annual_utilities: Decimal,
    pub annual_insurance: Decimal,
    pub annual_repairs: Decimal,
    pub annual_depreciation: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HomeOfficeReport {
    /// $5 × sqft, capped at $1500 (300 sqft × $5).
    pub simplified_deduction: Decimal,
    /// (business_sqft / total_sqft) × total annual expenses.
    pub actual_deduction: Decimal,
    /// Business-use % as a Decimal in `[0,1]`. 0 if `total_home_sqft <= 0`.
    pub business_pct: Decimal,
    /// Whichever is larger — recommended choice.
    pub recommended_deduction: Decimal,
    pub recommended_method: String,         // "simplified" | "actual"
}

const SIMPLIFIED_RATE_PER_SQFT: &str = "5";
const SIMPLIFIED_CAP_SQFT: i64 = 300;

pub fn compute(input: &HomeOfficeInput) -> HomeOfficeReport {
    let mut r = HomeOfficeReport::default();

    if input.business_use_sqft <= Decimal::ZERO {
        return r;
    }

    // ----- simplified method -----
    let rate = Decimal::from_str(SIMPLIFIED_RATE_PER_SQFT).unwrap();
    let cap = Decimal::from(SIMPLIFIED_CAP_SQFT);
    let counted_sqft = input.business_use_sqft.min(cap);
    r.simplified_deduction = counted_sqft * rate;

    // ----- actual method -----
    if input.total_home_sqft > Decimal::ZERO {
        r.business_pct = input.business_use_sqft / input.total_home_sqft;
        // Clamp to 100% just in case input is malformed (office > home).
        if r.business_pct > Decimal::ONE {
            r.business_pct = Decimal::ONE;
        }
        let annual = input.annual_mortgage_interest
            + input.annual_property_tax
            + input.annual_utilities
            + input.annual_insurance
            + input.annual_repairs
            + input.annual_depreciation;
        r.actual_deduction = annual * r.business_pct;
    }

    // ----- recommend the higher -----
    if r.actual_deduction > r.simplified_deduction {
        r.recommended_deduction = r.actual_deduction;
        r.recommended_method = "actual".into();
    } else {
        r.recommended_deduction = r.simplified_deduction;
        r.recommended_method = "simplified".into();
    }

    r
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> Decimal { Decimal::from_str(s).unwrap() }

    fn input(sqft: &str, total: &str) -> HomeOfficeInput {
        HomeOfficeInput {
            business_use_sqft: d(sqft),
            total_home_sqft: d(total),
            annual_mortgage_interest: Decimal::ZERO,
            annual_property_tax: Decimal::ZERO,
            annual_utilities: Decimal::ZERO,
            annual_insurance: Decimal::ZERO,
            annual_repairs: Decimal::ZERO,
            annual_depreciation: Decimal::ZERO,
        }
    }

    #[test]
    fn zero_sqft_returns_empty_report() {
        let r = compute(&input("0", "2000"));
        assert_eq!(r.simplified_deduction, Decimal::ZERO);
        assert_eq!(r.actual_deduction, Decimal::ZERO);
        assert_eq!(r.recommended_deduction, Decimal::ZERO);
    }

    #[test]
    fn simplified_under_cap() {
        // 100 sqft × $5 = $500.
        let r = compute(&input("100", "2000"));
        assert_eq!(r.simplified_deduction, d("500"));
        assert_eq!(r.recommended_method, "simplified");
    }

    #[test]
    fn simplified_at_cap() {
        // Exactly 300 sqft × $5 = $1500.
        let r = compute(&input("300", "2000"));
        assert_eq!(r.simplified_deduction, d("1500"));
    }

    #[test]
    fn simplified_caps_at_300_sqft() {
        // 500 sqft would naively be $2500 but the cap is 300 × $5 = $1500.
        let r = compute(&input("500", "2000"));
        assert_eq!(r.simplified_deduction, d("1500"),
            "simplified-method deduction must cap at $1,500");
    }

    #[test]
    fn actual_method_uses_business_percentage() {
        // 200 sqft / 2000 sqft = 10% business use.
        // Annual expenses $30k → deduction $3,000.
        let mut i = input("200", "2000");
        i.annual_mortgage_interest = d("12000");
        i.annual_property_tax = d("4000");
        i.annual_utilities = d("6000");
        i.annual_insurance = d("2000");
        i.annual_repairs = d("3000");
        i.annual_depreciation = d("3000");
        let r = compute(&i);
        assert_eq!(r.business_pct, d("0.1"));
        assert_eq!(r.actual_deduction, d("3000.0"));
    }

    #[test]
    fn recommends_actual_when_larger() {
        let mut i = input("200", "2000");   // simplified = $1000
        i.annual_utilities = d("50000");    // 10% × 50k = $5,000
        let r = compute(&i);
        assert_eq!(r.recommended_method, "actual");
        assert_eq!(r.recommended_deduction, d("5000.0"));
    }

    #[test]
    fn recommends_simplified_when_actual_is_smaller() {
        let mut i = input("200", "2000");   // simplified = $1000
        i.annual_utilities = d("5000");     // 10% × 5k = $500
        let r = compute(&i);
        assert_eq!(r.recommended_method, "simplified");
        assert_eq!(r.recommended_deduction, d("1000"));
    }

    #[test]
    fn business_pct_clamps_to_100_percent_on_malformed_input() {
        // Office larger than home — shouldn't crash, just cap at 100%.
        let mut i = input("3000", "2000");  // office > home
        i.annual_utilities = d("1000");
        let r = compute(&i);
        assert_eq!(r.business_pct, Decimal::ONE);
        assert_eq!(r.actual_deduction, d("1000"));
    }

    #[test]
    fn total_sqft_zero_skips_actual_method() {
        // User entered no total home sqft — can't compute %.
        let i = input("100", "0");
        let r = compute(&i);
        assert_eq!(r.actual_deduction, Decimal::ZERO);
        // Simplified still works.
        assert_eq!(r.simplified_deduction, d("500"));
    }
}
