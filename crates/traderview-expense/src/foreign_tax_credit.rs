//! IRS Form 1116 Foreign Tax Credit calculator.
//!
//! Traders holding ADRs / international ETFs / foreign-domiciled stocks
//! collect foreign withholding tax on dividends. That withholding is
//! claimable as a US tax credit (avoiding double taxation) up to the
//! US tax that would have been owed on the same foreign-source income.
//!
//! De minimis rule: if total foreign tax paid ≤ $300 ($600 MFJ) AND all
//! foreign source income is "passive" AND reported on a qualified
//! 1099-DIV/INT, the taxpayer can SKIP Form 1116 entirely and take the
//! full amount as a direct credit on Schedule 3 line 1.
//!
//! Above the de minimis threshold, the credit is LIMITED to:
//!   limit = US_tax_before_credit × (foreign_source_taxable_income / total_taxable_income)
//!
//! This module computes both branches and tells the caller which form
//! treatment applies.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FtcInput {
    pub foreign_tax_paid: Decimal,
    /// Foreign-source taxable income (gross dividends - allocable expenses).
    pub foreign_source_taxable_income: Decimal,
    /// Total taxable income (Form 1040 line 15).
    pub total_taxable_income: Decimal,
    /// US tax before credits (Form 1040 line 16).
    pub us_tax_before_credits: Decimal,
    pub filing_status: FilingStatus,
    /// True if ALL foreign income is passive and reported on
    /// qualified 1099 forms (gates the de minimis simplification).
    pub passive_only: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilingStatus {
    Single,
    HeadOfHousehold,
    MarriedFilingJointly,
    MarriedFilingSeparately,
}

impl FilingStatus {
    pub fn de_minimis_threshold(self) -> Decimal {
        use std::str::FromStr;
        match self {
            FilingStatus::MarriedFilingJointly => Decimal::from_str("600").unwrap(),
            _ => Decimal::from_str("300").unwrap(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FtcReport {
    pub form_treatment: FormTreatment,
    /// The credit limit per the §904 fraction. NOT applicable in the
    /// de minimis branch (full foreign tax paid is creditable there).
    pub limit_per_904: Decimal,
    /// The actual credit allowed = min(foreign_tax_paid, limit_per_904)
    /// — or `foreign_tax_paid` directly in the de minimis branch.
    pub credit_allowed: Decimal,
    /// Foreign tax paid that EXCEEDS the credit limit. Under §904(c) these
    /// carry back 1 year, forward 10. Caller reports this on Form 1116
    /// Schedule B (carryover schedule).
    pub excess_carryover: Decimal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FormTreatment {
    /// Under de minimis — claim directly on Sched 3 line 1, no Form 1116.
    DirectCreditDeMinimis,
    /// Must file Form 1116 to compute the limit.
    Form1116Required,
}

pub fn compute(input: &FtcInput) -> FtcReport {
    let threshold = input.filing_status.de_minimis_threshold();
    let de_minimis = input.passive_only && input.foreign_tax_paid <= threshold;
    if de_minimis {
        return FtcReport {
            form_treatment: FormTreatment::DirectCreditDeMinimis,
            limit_per_904: Decimal::ZERO,    // not computed in this branch
            credit_allowed: input.foreign_tax_paid,
            excess_carryover: Decimal::ZERO,
        };
    }
    // Full Form 1116 computation.
    let limit = if input.total_taxable_income.is_zero() {
        Decimal::ZERO
    } else {
        input.us_tax_before_credits
            * input.foreign_source_taxable_income
            / input.total_taxable_income
    };
    let credit = limit.min(input.foreign_tax_paid);
    let excess = if input.foreign_tax_paid > limit {
        input.foreign_tax_paid - limit
    } else {
        Decimal::ZERO
    };
    FtcReport {
        form_treatment: FormTreatment::Form1116Required,
        limit_per_904: limit,
        credit_allowed: credit,
        excess_carryover: excess,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn d(s: &str) -> Decimal { Decimal::from_str(s).unwrap() }

    fn baseline() -> FtcInput {
        FtcInput {
            foreign_tax_paid: d("250"),
            foreign_source_taxable_income: d("3000"),
            total_taxable_income: d("150000"),
            us_tax_before_credits: d("25000"),
            filing_status: FilingStatus::Single,
            passive_only: true,
        }
    }

    #[test]
    fn de_minimis_single_under_300_skips_form_1116() {
        let r = compute(&baseline());
        assert_eq!(r.form_treatment, FormTreatment::DirectCreditDeMinimis);
        assert_eq!(r.credit_allowed, d("250"));
        assert_eq!(r.excess_carryover, Decimal::ZERO);
    }

    #[test]
    fn de_minimis_mfj_under_600_skips_form_1116() {
        let i = FtcInput {
            foreign_tax_paid: d("550"),
            filing_status: FilingStatus::MarriedFilingJointly,
            ..baseline()
        };
        let r = compute(&i);
        assert_eq!(r.form_treatment, FormTreatment::DirectCreditDeMinimis);
        assert_eq!(r.credit_allowed, d("550"));
    }

    #[test]
    fn over_threshold_forces_form_1116() {
        // Single, $500 foreign tax > $300.
        let i = FtcInput { foreign_tax_paid: d("500"), ..baseline() };
        let r = compute(&i);
        assert_eq!(r.form_treatment, FormTreatment::Form1116Required);
        // Limit = 25000 × 3000 / 150000 = 500. Credit = min(500, 500) = 500.
        assert_eq!(r.limit_per_904, d("500"));
        assert_eq!(r.credit_allowed, d("500"));
        assert_eq!(r.excess_carryover, Decimal::ZERO);
    }

    #[test]
    fn over_threshold_with_excess_records_carryover() {
        // Foreign tax $800 > limit $500 → $300 excess carryover.
        let i = FtcInput { foreign_tax_paid: d("800"), ..baseline() };
        let r = compute(&i);
        assert_eq!(r.limit_per_904, d("500"));
        assert_eq!(r.credit_allowed, d("500"));
        assert_eq!(r.excess_carryover, d("300"));
    }

    #[test]
    fn non_passive_income_forces_form_1116_even_under_threshold() {
        let i = FtcInput {
            foreign_tax_paid: d("250"),    // under $300
            passive_only: false,           // but not passive
            ..baseline()
        };
        let r = compute(&i);
        assert_eq!(r.form_treatment, FormTreatment::Form1116Required,
            "non-passive income disqualifies de minimis even under threshold");
    }

    #[test]
    fn zero_total_taxable_income_yields_zero_limit() {
        let i = FtcInput {
            foreign_tax_paid: d("500"),
            total_taxable_income: Decimal::ZERO,
            ..baseline()
        };
        let r = compute(&i);
        assert_eq!(r.form_treatment, FormTreatment::Form1116Required);
        assert_eq!(r.limit_per_904, Decimal::ZERO);
        assert_eq!(r.credit_allowed, Decimal::ZERO);
        assert_eq!(r.excess_carryover, d("500"),
            "all foreign tax carries over when no US tax to credit against");
    }

    #[test]
    fn mfs_uses_300_threshold_not_600() {
        // Married filing separately gets the $300 threshold, NOT $600.
        let i = FtcInput {
            foreign_tax_paid: d("400"),
            filing_status: FilingStatus::MarriedFilingSeparately,
            ..baseline()
        };
        let r = compute(&i);
        assert_eq!(r.form_treatment, FormTreatment::Form1116Required,
            "MFS uses $300 threshold like Single, not MFJ $600");
    }

    #[test]
    fn limit_uses_section_904_fraction() {
        // foreign/total income ratio = 30k/300k = 10% of US tax.
        let i = FtcInput {
            foreign_tax_paid: d("5000"),
            foreign_source_taxable_income: d("30000"),
            total_taxable_income: d("300000"),
            us_tax_before_credits: d("60000"),
            filing_status: FilingStatus::Single,
            passive_only: true,
        };
        let r = compute(&i);
        // limit = 60000 × 30000 / 300000 = 6000.
        assert_eq!(r.limit_per_904, d("6000"));
        // foreign tax 5000 < limit 6000 → full credit, no carryover.
        assert_eq!(r.credit_allowed, d("5000"));
        assert_eq!(r.excess_carryover, Decimal::ZERO);
    }
}
