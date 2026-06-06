//! Net Investment Income Tax (NIIT) — IRC § 1411 (3.8% surtax).
//!
//! 3.8% surtax on the **lesser** of:
//!   1. Net investment income (NII), or
//!   2. The excess of modified adjusted gross income (MAGI) over the
//!      filing-status threshold.
//!
//! Thresholds (NOT inflation-adjusted — set by statute in 2013):
//!   * Single / HoH:                                  $200,000
//!   * MFJ / QSS:                                     $250,000
//!   * MFS:                                           $125,000
//!
//! Net investment income includes:
//!   * Interest, dividends (ordinary + qualified), capital gains
//!   * Rental/royalty income from passive activities (IRC § 469)
//!   * Non-qualified annuity income
//!   * Income from a trade or business of trading financial instruments
//!
//! NII excludes:
//!   * Wages, self-employment income (subject to SE/Medicare instead)
//!   * Distributions from qualified retirement plans
//!   * Tax-exempt interest
//!
//! For this engine, MAGI ≈ AGI (foreign earned income exclusion is the
//! only common add-back and we don't model FEIE in v1).
//!
//! Sources:
//!   * IRC § 1411
//!   * 26 CFR § 1.1411-1 et seq.
//!   * IRS Form 8960 instructions (Rev. 2024)

use crate::engine::FilingStatus;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct NiitInput {
    /// Net investment income (already netted of allocable deductions).
    pub net_investment_income: Decimal,
    /// Modified adjusted gross income. For most filers this equals AGI.
    pub magi: Decimal,
    pub status: FilingStatus,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct NiitResult {
    /// MAGI threshold for this filing status.
    pub threshold: Decimal,
    /// max(0, MAGI - threshold).
    pub excess_magi: Decimal,
    /// min(NII, excess_magi) — the base to which 3.8% is applied.
    pub taxable_base: Decimal,
    /// 3.8% × taxable_base, rounded to cents.
    pub tax: Decimal,
}

/// Per-status MAGI threshold (statutory, no inflation adjustment).
///
/// QSS (qualifying surviving spouse) shares the MFJ threshold — IRC §
/// 1411(b)(1). The engine's `FilingStatus` enum collapses QSS into
/// `Mfj` per Form 1040 box 4, so this matches.
pub fn threshold(status: FilingStatus) -> Decimal {
    match status {
        FilingStatus::Single | FilingStatus::Hoh => Decimal::from(200_000),
        FilingStatus::Mfj => Decimal::from(250_000),
        FilingStatus::Mfs => Decimal::from(125_000),
    }
}

pub fn compute(input: NiitInput) -> NiitResult {
    let threshold = threshold(input.status);
    let excess_magi = (input.magi - threshold).max(Decimal::ZERO);
    let nii = input.net_investment_income.max(Decimal::ZERO);
    let taxable_base = nii.min(excess_magi);
    let rate: Decimal = "0.038".parse().unwrap();
    let tax = (taxable_base * rate).round_dp(2);
    NiitResult {
        threshold,
        excess_magi,
        taxable_base,
        tax,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(n: i64) -> Decimal {
        Decimal::from(n)
    }

    #[test]
    fn under_threshold_owes_zero() {
        let r = compute(NiitInput {
            net_investment_income: d(50_000),
            magi: d(180_000),
            status: FilingStatus::Single,
        });
        assert_eq!(r.excess_magi, Decimal::ZERO);
        assert_eq!(r.taxable_base, Decimal::ZERO);
        assert_eq!(r.tax, Decimal::ZERO);
    }

    #[test]
    fn nii_smaller_than_excess_magi_uses_nii() {
        // Single, MAGI = $300k → excess = $100k. NII = $20k.
        // NIIT = 3.8% × $20k = $760.
        let r = compute(NiitInput {
            net_investment_income: d(20_000),
            magi: d(300_000),
            status: FilingStatus::Single,
        });
        assert_eq!(r.excess_magi, d(100_000));
        assert_eq!(r.taxable_base, d(20_000));
        assert_eq!(r.tax, "760.00".parse::<Decimal>().unwrap());
    }

    #[test]
    fn excess_magi_smaller_than_nii_uses_excess() {
        // Single, MAGI = $220k → excess = $20k. NII = $50k.
        // NIIT = 3.8% × $20k = $760.
        let r = compute(NiitInput {
            net_investment_income: d(50_000),
            magi: d(220_000),
            status: FilingStatus::Single,
        });
        assert_eq!(r.taxable_base, d(20_000));
        assert_eq!(r.tax, "760.00".parse::<Decimal>().unwrap());
    }

    #[test]
    fn mfj_threshold_is_250k_not_400k() {
        // MFJ threshold is $250k, NOT 2x single. Common mistake.
        let r = compute(NiitInput {
            net_investment_income: d(100_000),
            magi: d(350_000),
            status: FilingStatus::Mfj,
        });
        assert_eq!(r.threshold, d(250_000));
        assert_eq!(r.excess_magi, d(100_000));
        assert_eq!(r.taxable_base, d(100_000));
        assert_eq!(r.tax, "3800.00".parse::<Decimal>().unwrap());
    }

    #[test]
    fn mfs_threshold_is_125k() {
        let r = compute(NiitInput {
            net_investment_income: d(20_000),
            magi: d(150_000),
            status: FilingStatus::Mfs,
        });
        assert_eq!(r.threshold, d(125_000));
        assert_eq!(r.excess_magi, d(25_000));
        assert_eq!(r.taxable_base, d(20_000));
        assert_eq!(r.tax, "760.00".parse::<Decimal>().unwrap());
    }

    #[test]
    fn hoh_threshold_matches_single() {
        let r = compute(NiitInput {
            net_investment_income: d(10_000),
            magi: d(210_000),
            status: FilingStatus::Hoh,
        });
        assert_eq!(r.threshold, d(200_000));
        assert_eq!(r.excess_magi, d(10_000));
        assert_eq!(r.taxable_base, d(10_000));
        assert_eq!(r.tax, "380.00".parse::<Decimal>().unwrap());
    }

    #[test]
    fn negative_nii_clamped_to_zero() {
        // Capital losses can drive NII negative — NIIT can't be negative.
        let r = compute(NiitInput {
            net_investment_income: d(-5_000),
            magi: d(300_000),
            status: FilingStatus::Single,
        });
        assert_eq!(r.taxable_base, Decimal::ZERO);
        assert_eq!(r.tax, Decimal::ZERO);
    }

    #[test]
    fn at_threshold_exactly_owes_zero() {
        let r = compute(NiitInput {
            net_investment_income: d(50_000),
            magi: d(200_000),
            status: FilingStatus::Single,
        });
        assert_eq!(r.excess_magi, Decimal::ZERO);
        assert_eq!(r.tax, Decimal::ZERO);
    }

    #[test]
    fn rounds_to_two_decimal_places() {
        // 3.8% × $13,333 = $506.654 → $506.65 (banker's rounding).
        let r = compute(NiitInput {
            net_investment_income: d(13_333),
            magi: d(300_000),
            status: FilingStatus::Single,
        });
        assert_eq!(r.tax, "506.65".parse::<Decimal>().unwrap());
    }

    #[test]
    fn mfj_at_threshold_exactly_owes_zero() {
        let r = compute(NiitInput {
            net_investment_income: d(50_000),
            magi: d(250_000),
            status: FilingStatus::Mfj,
        });
        assert_eq!(r.threshold, d(250_000));
        assert_eq!(r.excess_magi, Decimal::ZERO);
    }
}
