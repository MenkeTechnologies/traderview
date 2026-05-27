//! IRC §1411 Net Investment Income Tax (NIIT) calculator.
//!
//! 3.8% surtax on the LESSER of:
//!   (a) net investment income (cap gains, interest, dividends, rents),
//!   (b) MAGI - threshold
//!
//! Thresholds (NOT indexed for inflation — fixed since 2013):
//!   - Single / HoH:                 $200,000
//!   - Married Filing Jointly:       $250,000
//!   - Married Filing Separately:    $125,000
//!
//! Traders with significant gains hit this in addition to their normal
//! cap gains rate. Caller passes NII and MAGI — engine computes the
//! tax owed and the binding leg.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilingStatus {
    Single,
    HeadOfHousehold,
    MarriedFilingJointly,
    MarriedFilingSeparately,
}

impl FilingStatus {
    pub fn niit_threshold(self) -> Decimal {
        match self {
            FilingStatus::Single | FilingStatus::HeadOfHousehold => {
                Decimal::from_str("200000").unwrap()
            }
            FilingStatus::MarriedFilingJointly => Decimal::from_str("250000").unwrap(),
            FilingStatus::MarriedFilingSeparately => Decimal::from_str("125000").unwrap(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NiitInput {
    pub filing_status: FilingStatus,
    pub net_investment_income: Decimal,
    /// Modified Adjusted Gross Income — Form 8960 line 13.
    pub magi: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NiitReport {
    pub threshold: Decimal,
    pub magi_over_threshold: Decimal,
    pub niit_base: Decimal,
    /// 3.8% × niit_base.
    pub tax: Decimal,
    /// Which input bound the calculation.
    pub binding_leg: BindingLeg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BindingLeg {
    /// MAGI was under the threshold → no tax.
    #[default]
    NotSubject,
    /// NII was smaller than MAGI excess → NII binds.
    NetInvestmentIncome,
    /// MAGI excess was smaller than NII → excess binds.
    MagiExcess,
}

pub fn compute(input: &NiitInput) -> NiitReport {
    let rate = Decimal::from_str("0.038").unwrap();
    let threshold = input.filing_status.niit_threshold();
    let magi_excess = (input.magi - threshold).max(Decimal::ZERO);

    if magi_excess.is_zero() {
        return NiitReport {
            threshold,
            magi_over_threshold: Decimal::ZERO,
            niit_base: Decimal::ZERO,
            tax: Decimal::ZERO,
            binding_leg: BindingLeg::NotSubject,
        };
    }
    let nii = input.net_investment_income.max(Decimal::ZERO);
    let (base, leg) = if nii <= magi_excess {
        (nii, BindingLeg::NetInvestmentIncome)
    } else {
        (magi_excess, BindingLeg::MagiExcess)
    };
    let tax = base * rate;
    NiitReport {
        threshold,
        magi_over_threshold: magi_excess,
        niit_base: base,
        tax,
        binding_leg: leg,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    #[test]
    fn under_threshold_no_tax() {
        let r = compute(&NiitInput {
            filing_status: FilingStatus::Single,
            net_investment_income: d("50000"),
            magi: d("180000"),
        });
        assert_eq!(r.tax, Decimal::ZERO);
        assert_eq!(r.binding_leg, BindingLeg::NotSubject);
    }

    #[test]
    fn at_threshold_no_tax() {
        // MAGI exactly at threshold → magi_excess = 0 → no tax.
        let r = compute(&NiitInput {
            filing_status: FilingStatus::Single,
            net_investment_income: d("50000"),
            magi: d("200000"),
        });
        assert_eq!(r.tax, Decimal::ZERO);
    }

    #[test]
    fn nii_smaller_than_excess_nii_binds() {
        // Single, MAGI $300k, NII $40k. Excess = 100k. NII smaller → NII binds.
        let r = compute(&NiitInput {
            filing_status: FilingStatus::Single,
            net_investment_income: d("40000"),
            magi: d("300000"),
        });
        assert_eq!(r.binding_leg, BindingLeg::NetInvestmentIncome);
        assert_eq!(r.niit_base, d("40000"));
        // 40000 × 0.038 = 1520.
        assert_eq!(r.tax, d("1520.000"));
    }

    #[test]
    fn excess_smaller_than_nii_excess_binds() {
        // Single, MAGI $210k (excess 10k), NII $80k. Excess smaller → excess binds.
        let r = compute(&NiitInput {
            filing_status: FilingStatus::Single,
            net_investment_income: d("80000"),
            magi: d("210000"),
        });
        assert_eq!(r.binding_leg, BindingLeg::MagiExcess);
        assert_eq!(r.niit_base, d("10000"));
        assert_eq!(r.tax, d("380.000"));
    }

    #[test]
    fn mfj_threshold_is_250k_not_200k() {
        // MFJ at MAGI $240k — still under 250k threshold → no tax.
        let r = compute(&NiitInput {
            filing_status: FilingStatus::MarriedFilingJointly,
            net_investment_income: d("100000"),
            magi: d("240000"),
        });
        assert_eq!(r.tax, Decimal::ZERO);
    }

    #[test]
    fn mfs_threshold_is_125k() {
        // MFS at MAGI $140k. Excess = 15k. NII = 50k → excess binds.
        let r = compute(&NiitInput {
            filing_status: FilingStatus::MarriedFilingSeparately,
            net_investment_income: d("50000"),
            magi: d("140000"),
        });
        assert_eq!(r.binding_leg, BindingLeg::MagiExcess);
        assert_eq!(r.niit_base, d("15000"));
        assert_eq!(r.tax, d("570.000"));
    }

    #[test]
    fn negative_nii_clamps_to_zero() {
        // Investment loss year → NII negative → treat as zero (NII can
        // never make NIIT base negative).
        let r = compute(&NiitInput {
            filing_status: FilingStatus::Single,
            net_investment_income: d("-10000"),
            magi: d("300000"),
        });
        // NII = 0 → NII binds at 0.
        assert_eq!(r.binding_leg, BindingLeg::NetInvestmentIncome);
        assert_eq!(r.tax, Decimal::ZERO);
    }

    #[test]
    fn high_earner_with_large_nii_pays_full_3_8_pct() {
        // Single, MAGI $500k (excess $300k), NII $100k → NII binds.
        let r = compute(&NiitInput {
            filing_status: FilingStatus::Single,
            net_investment_income: d("100000"),
            magi: d("500000"),
        });
        assert_eq!(r.binding_leg, BindingLeg::NetInvestmentIncome);
        // 100000 × 0.038 = 3800.
        assert_eq!(r.tax, d("3800.000"));
    }

    #[test]
    fn hoh_threshold_matches_single() {
        let r = compute(&NiitInput {
            filing_status: FilingStatus::HeadOfHousehold,
            net_investment_income: d("50000"),
            magi: d("200000"),
        });
        // Same threshold as single → at threshold no tax.
        assert_eq!(r.tax, Decimal::ZERO);
    }
}
