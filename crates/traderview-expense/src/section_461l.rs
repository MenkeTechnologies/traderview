//! IRC §461(l) — Limitation on excess business losses of noncorporate
//! taxpayers.
//!
//! **Completes the loss-limitation cascade** for individual, trust, and
//! estate taxpayers:
//!
//!   1. §704(d) — outside basis limit (`section_704d`)
//!   2. §465  — at-risk amount limit (`section_465`)
//!   3. §469  — passive activity loss limit (`section_469`)
//!   4. **§461(l)** — excess business loss limit (THIS module)
//!
//! Each prior limit must be applied before §461(l). The "net business
//! loss" that §461(l) limits is the loss SURVIVING all earlier limits.
//!
//! **§461(l)(1)** disallows the portion of a noncorporate taxpayer's
//! net business loss that exceeds a statutory threshold. The
//! disallowed portion becomes a §172 NOL carryforward for subsequent
//! years (no carryback).
//!
//! **§461(l)(3) threshold amounts** (inflation-adjusted, but re-
//! indexed by the One Big Beautiful Bill Act of 2025):
//!
//! | Tax year | Single   | MFJ      |
//! |----------|----------|----------|
//! | 2021     | $262,000 | $524,000 |
//! | 2022     | $270,000 | $540,000 |
//! | 2023     | $289,000 | $578,000 |
//! | 2024     | $305,000 | $610,000 |
//! | 2025     | $313,000 | $626,000 |
//! | **2026** | **$256,000** | **$512,000** |
//!
//! The 2026 thresholds dropped from 2025 because OBBBA (signed mid-2025)
//! re-indexed back toward the TCJA-original $250k/$500k 2018 base. The
//! same act also made §461(l) **permanent** — eliminating the prior
//! 2028 sunset.
//!
//! **§461(l)(2)** application: applies AFTER §704(d), §465, §469
//! limits. The "net business loss" is what remains. Corporate
//! taxpayers (C-corp) are NOT subject to §461(l).
//!
//! **2018-2020 CARES Act suspension**: §461(l) was suspended for tax
//! years 2018-2020 (CARES Act §2304). First effective year of the
//! limitation is 2021.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilingStatus {
    Single,
    /// MFJ — Married Filing Jointly (statutory 2× single threshold).
    MarriedFilingJointly,
    /// HOH / MFS — treated as Single for §461(l) threshold (the
    /// statute references single by default).
    HeadOfHouseholdOrMfs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaxpayerKind {
    /// Individual, trust, or estate (subject to §461(l)).
    Noncorporate,
    /// C-corporation (NOT subject to §461(l)).
    Ccorporation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section461lInput {
    pub tax_year: i32,
    pub filing_status: FilingStatus,
    pub taxpayer_kind: TaxpayerKind,
    /// Aggregate deductions from the taxpayer's trades or businesses
    /// after applying §704(d), §465, §469 limits. Positive number.
    pub aggregate_business_deductions_after_prior_limits: Decimal,
    /// Aggregate gross income / gain from trades or businesses.
    pub aggregate_business_income: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section461lResult {
    /// True if §461(l) applies (noncorporate + post-2020 tax year).
    pub applies: bool,
    /// Threshold for the taxpayer (single or MFJ) in the input year.
    pub threshold_dollars: Decimal,
    /// Net business loss = deductions - income. Zero if income exceeds
    /// deductions (no loss).
    pub net_business_loss: Decimal,
    /// Excess business loss disallowed under §461(l) — net loss minus
    /// threshold. Becomes §172 NOL carryforward.
    pub excess_business_loss_disallowed: Decimal,
    /// Loss actually deductible against other income this year.
    pub allowed_loss_deduction: Decimal,
    /// True if the excess-loss limit binds (net loss > threshold).
    pub limit_binding: bool,
    pub note: String,
}

/// §461(l)(3) inflation-adjusted threshold for single filer. MFJ is 2×.
fn threshold_for_year(year: i32) -> Decimal {
    use rust_decimal::Decimal as D;
    let single_str: &str = match year {
        2021 => "262000",
        2022 => "270000",
        2023 => "289000",
        2024 => "305000",
        2025 => "313000",
        2026 => "256000", // post-OBBBA re-indexing
        // For 2027+ and pre-2021 fallback, use a conservative estimate.
        // Caller responsible for refreshing once IRS publishes future
        // year amounts.
        y if y > 2026 => "256000",
        _ => "0", // CARES Act suspension years (2018-2020) and earlier
    };
    single_str.parse::<D>().unwrap_or(Decimal::ZERO)
}

pub fn compute(input: &Section461lInput) -> Section461lResult {
    // §461(l) applies only to noncorporate taxpayers in tax years 2021+
    // (CARES Act suspended for 2018-2020).
    let applies =
        matches!(input.taxpayer_kind, TaxpayerKind::Noncorporate) && input.tax_year >= 2021;

    if !applies {
        return Section461lResult {
            applies: false,
            threshold_dollars: Decimal::ZERO,
            net_business_loss: Decimal::ZERO,
            excess_business_loss_disallowed: Decimal::ZERO,
            allowed_loss_deduction: input.aggregate_business_deductions_after_prior_limits,
            limit_binding: false,
            note: if matches!(input.taxpayer_kind, TaxpayerKind::Ccorporation) {
                "§461(l) does NOT apply — C-corporation; corporate loss treatment under §172 only"
                    .into()
            } else if input.tax_year < 2021 {
                format!(
                    "§461(l) suspended for {} (CARES Act §2304 suspended 2018-2020 + statute first effective 2021)",
                    input.tax_year
                )
            } else {
                String::new()
            },
        };
    }

    // Single threshold; MFJ doubles.
    let single_threshold = threshold_for_year(input.tax_year);
    let threshold = match input.filing_status {
        FilingStatus::MarriedFilingJointly => single_threshold * Decimal::from(2),
        FilingStatus::Single | FilingStatus::HeadOfHouseholdOrMfs => single_threshold,
    };

    // Net business loss = deductions - income (clamped at zero — no
    // limitation if net gain).
    let net_business_loss = (input.aggregate_business_deductions_after_prior_limits
        - input.aggregate_business_income)
        .max(Decimal::ZERO);

    let excess = (net_business_loss - threshold).max(Decimal::ZERO);
    let allowed = input.aggregate_business_deductions_after_prior_limits - excess;
    let limit_binds = net_business_loss > threshold;

    let status_phrase = match input.filing_status {
        FilingStatus::Single => "single",
        FilingStatus::MarriedFilingJointly => "MFJ",
        FilingStatus::HeadOfHouseholdOrMfs => "HOH/MFS",
    };

    let note = if limit_binds {
        format!(
            "§461(l) BINDS for {} year {}: net business loss ${} > threshold ${}; ${} disallowed and becomes §172 NOL carryforward",
            status_phrase,
            input.tax_year,
            net_business_loss.round_dp(2),
            threshold.round_dp(2),
            excess.round_dp(2)
        )
    } else {
        format!(
            "§461(l) satisfied for {} year {}: net business loss ${} ≤ threshold ${} → full loss allowed",
            status_phrase,
            input.tax_year,
            net_business_loss.round_dp(2),
            threshold.round_dp(2)
        )
    };

    Section461lResult {
        applies: true,
        threshold_dollars: threshold,
        net_business_loss,
        excess_business_loss_disallowed: excess,
        allowed_loss_deduction: allowed,
        limit_binding: limit_binds,
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn base() -> Section461lInput {
        Section461lInput {
            tax_year: 2026,
            filing_status: FilingStatus::Single,
            taxpayer_kind: TaxpayerKind::Noncorporate,
            aggregate_business_deductions_after_prior_limits: dec!(400_000),
            aggregate_business_income: Decimal::ZERO,
        }
    }

    #[test]
    fn single_2026_threshold_256k() {
        // 2026 single: $256k threshold per OBBBA re-indexing.
        let r = compute(&base());
        assert_eq!(r.threshold_dollars, dec!(256_000));
    }

    #[test]
    fn mfj_2026_threshold_512k() {
        let mut i = base();
        i.filing_status = FilingStatus::MarriedFilingJointly;
        let r = compute(&i);
        assert_eq!(r.threshold_dollars, dec!(512_000));
    }

    #[test]
    fn single_2026_400k_loss_excess_144k_disallowed() {
        // $400k loss - $256k threshold = $144k EBL disallowed.
        let r = compute(&base());
        assert_eq!(r.net_business_loss, dec!(400_000));
        assert_eq!(r.excess_business_loss_disallowed, dec!(144_000));
        assert_eq!(r.allowed_loss_deduction, dec!(256_000));
        assert!(r.limit_binding);
    }

    #[test]
    fn mfj_2026_700k_loss_excess_188k_disallowed() {
        // MFJ $700k loss - $512k threshold = $188k disallowed.
        let mut i = base();
        i.filing_status = FilingStatus::MarriedFilingJointly;
        i.aggregate_business_deductions_after_prior_limits = dec!(700_000);
        let r = compute(&i);
        assert_eq!(r.threshold_dollars, dec!(512_000));
        assert_eq!(r.excess_business_loss_disallowed, dec!(188_000));
        assert_eq!(r.allowed_loss_deduction, dec!(512_000));
    }

    #[test]
    fn obbba_re_indexing_2025_vs_2026_delta() {
        // OBBBA dropped 2026 single from $313k → $256k (delta $57k).
        // MFJ from $626k → $512k (delta $114k).
        let single_2025 = threshold_for_year(2025);
        let single_2026 = threshold_for_year(2026);
        assert_eq!(single_2025, dec!(313_000));
        assert_eq!(single_2026, dec!(256_000));
        assert_eq!(single_2025 - single_2026, dec!(57_000));
        // MFJ delta is 2× single delta.
        assert_eq!(
            (single_2025 - single_2026) * Decimal::from(2),
            dec!(114_000)
        );
    }

    #[test]
    fn loss_below_threshold_no_limit_binding() {
        // $200k loss < $256k single threshold → no EBL, full deduction.
        let mut i = base();
        i.aggregate_business_deductions_after_prior_limits = dec!(200_000);
        let r = compute(&i);
        assert!(!r.limit_binding);
        assert_eq!(r.excess_business_loss_disallowed, Decimal::ZERO);
        assert_eq!(r.allowed_loss_deduction, dec!(200_000));
    }

    #[test]
    fn threshold_exact_boundary_no_limit_binding() {
        // Loss exactly = threshold → not > threshold → not binding.
        let mut i = base();
        i.aggregate_business_deductions_after_prior_limits = dec!(256_000);
        let r = compute(&i);
        assert!(!r.limit_binding);
        assert_eq!(r.excess_business_loss_disallowed, Decimal::ZERO);
    }

    #[test]
    fn threshold_one_dollar_over_binds() {
        let mut i = base();
        i.aggregate_business_deductions_after_prior_limits = dec!(256_001);
        let r = compute(&i);
        assert!(r.limit_binding);
        assert_eq!(r.excess_business_loss_disallowed, dec!(1));
    }

    #[test]
    fn net_gain_no_loss_no_excess() {
        // Business income $100k, deductions $50k → net gain, no loss.
        let mut i = base();
        i.aggregate_business_deductions_after_prior_limits = dec!(50_000);
        i.aggregate_business_income = dec!(100_000);
        let r = compute(&i);
        assert_eq!(r.net_business_loss, Decimal::ZERO);
        assert!(!r.limit_binding);
    }

    #[test]
    fn c_corp_not_subject_to_section_461l() {
        let mut i = base();
        i.taxpayer_kind = TaxpayerKind::Ccorporation;
        let r = compute(&i);
        assert!(!r.applies);
        assert!(r.note.contains("C-corporation"));
    }

    #[test]
    fn pre_2021_year_suspended_under_cares() {
        // 2020 was suspended by CARES Act §2304.
        let mut i = base();
        i.tax_year = 2020;
        let r = compute(&i);
        assert!(!r.applies);
        assert!(r.note.contains("CARES Act"));
        assert!(r.note.contains("suspended"));
    }

    #[test]
    fn pre_2018_no_limitation_existed() {
        // §461(l) was enacted in TCJA (effective 2018) → no limit pre-2018.
        let mut i = base();
        i.tax_year = 2017;
        let r = compute(&i);
        assert!(!r.applies);
    }

    #[test]
    fn cares_suspension_years_all_pinned() {
        // 2018, 2019, 2020 all suspended.
        for year in [2018, 2019, 2020] {
            let mut i = base();
            i.tax_year = year;
            let r = compute(&i);
            assert!(!r.applies, "year {year} should have §461(l) suspended");
        }
    }

    #[test]
    fn first_effective_year_2021_applies() {
        let mut i = base();
        i.tax_year = 2021;
        let r = compute(&i);
        assert!(r.applies);
        assert_eq!(r.threshold_dollars, dec!(262_000));
    }

    #[test]
    fn historical_thresholds_pinned() {
        // Pin each year's known threshold value (single, MFJ = 2×).
        let cases = [
            (2021, dec!(262_000)),
            (2022, dec!(270_000)),
            (2023, dec!(289_000)),
            (2024, dec!(305_000)),
            (2025, dec!(313_000)),
            (2026, dec!(256_000)),
        ];
        for (year, expected) in cases {
            let mut i = base();
            i.tax_year = year;
            let r = compute(&i);
            assert_eq!(r.threshold_dollars, expected, "year {year}");
        }
    }

    #[test]
    fn hoh_mfs_uses_single_threshold() {
        // HOH and MFS use the single-filer threshold per the statute
        // (which doesn't carve out separate amounts for these statuses).
        let mut i = base();
        i.filing_status = FilingStatus::HeadOfHouseholdOrMfs;
        let r = compute(&i);
        assert_eq!(r.threshold_dollars, dec!(256_000));
    }

    #[test]
    fn excess_becomes_nol_per_172_note() {
        let r = compute(&base());
        assert!(r.note.contains("§172"));
        assert!(r.note.contains("NOL"));
    }

    #[test]
    fn very_large_loss_no_precision_loss() {
        // $1B loss + $700M income = $300M net loss. MFJ 2026 = $512k
        // threshold. EBL = $299.488M.
        let mut i = base();
        i.filing_status = FilingStatus::MarriedFilingJointly;
        i.aggregate_business_deductions_after_prior_limits = dec!(1_000_000_000);
        i.aggregate_business_income = dec!(700_000_000);
        let r = compute(&i);
        assert_eq!(r.net_business_loss, dec!(300_000_000));
        assert_eq!(r.excess_business_loss_disallowed, dec!(299_488_000));
    }

    #[test]
    fn future_year_uses_2026_fallback() {
        // 2027+ uses the 2026 threshold as a conservative fallback
        // (caller responsible for refreshing once IRS publishes).
        let mut i = base();
        i.tax_year = 2027;
        let r = compute(&i);
        assert_eq!(r.threshold_dollars, dec!(256_000));
    }

    #[test]
    fn loss_just_below_mfj_threshold_full_allowance() {
        // MFJ 2026: $511,999 net loss → just under $512k threshold.
        let mut i = base();
        i.filing_status = FilingStatus::MarriedFilingJointly;
        i.aggregate_business_deductions_after_prior_limits = dec!(511_999);
        let r = compute(&i);
        assert!(!r.limit_binding);
        assert_eq!(r.allowed_loss_deduction, dec!(511_999));
    }

    #[test]
    fn note_describes_binding_path_with_dollar_figures() {
        let r = compute(&base());
        assert!(r.note.contains("§461(l) BINDS"));
        assert!(r.note.contains("$256000") || r.note.contains("256000"));
        assert!(r.note.contains("$144000") || r.note.contains("144000"));
    }

    #[test]
    fn note_describes_satisfied_path() {
        let mut i = base();
        i.aggregate_business_deductions_after_prior_limits = dec!(100_000);
        let r = compute(&i);
        assert!(r.note.contains("§461(l) satisfied"));
    }
}
