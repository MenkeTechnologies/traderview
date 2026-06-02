//! IRC § 1411 — Net Investment Income Tax (NIIT). 3.8%
//! surtax on net investment income (NII) for individuals,
//! estates, and trusts when modified adjusted gross income
//! (MAGI) exceeds statutory thresholds. Trader-critical: any
//! high-income trader (single MAGI > $200K; MFJ MAGI > $250K)
//! pays 3.8% on interest + dividends + capital gains +
//! passive rental income + royalties + non-qualified
//! annuity income. § 469(c)(7) real estate professional
//! exception carries through to § 1411 NIIT.
//!
//! Distinct from siblings `section_469` (passive activity
//! rules), `section_1256` (60/40 contract treatment), and
//! `section_865` (sourcing of income).
//!
//! **§ 1411(a)(1) tax computation** — NIIT = 3.8% × LESSER
//! of:
//! - Net investment income (NII), OR
//! - Excess of MAGI over applicable threshold.
//!
//! **§ 1411(b) MAGI thresholds (NOT indexed for inflation;
//! same since 2013 ACA enactment)**:
//! - Single / Head of Household: **$200,000**
//! - Married Filing Jointly / Qualifying Surviving Spouse:
//!   **$250,000**
//! - Married Filing Separately: **$125,000**
//!
//! **§ 1411(c)(1) net investment income categories**:
//! - Interest
//! - Dividends
//! - Capital gains
//! - Rental income (passive)
//! - Royalty income
//! - Non-qualified annuity income
//!
//! **§ 1411(c)(1)(B) deductions** — investment expenses +
//! state income tax allocable to investment income.
//!
//! **§ 1411(c)(2) trade or business carve-outs** — income
//! from trade or business in which taxpayer materially
//! participates (other than trade or business of trading
//! financial instruments / commodities) is EXCLUDED from
//! NII.
//!
//! **§ 1411(c)(5) qualified retirement plan distributions
//! exception** — distributions from qualified retirement
//! plans + IRAs are EXCLUDED from NII.
//!
//! **§ 469(c)(7) real estate professional carve-out** — if
//! taxpayer (a) performs ≥ 750 hours per year in real
//! property trades or businesses AND (b) more than HALF of
//! personal services in real property trades, rental income
//! may be treated as ACTIVE and excluded from NII.
//!
//! **One Big Beautiful Bill Act of 2025 (OBBBA)** — did NOT
//! repeal, amend, or modify § 1411; 3.8% rate + thresholds +
//! categories + retirement-plan exception all remain
//! identical to 2013 form.
//!
//! Citations: 26 USC § 1411(a)(1), (b)(1)-(3), (c)(1)(A)-
//! (B), (c)(2), (c)(5); IRS Form 8960 (2025); IRS Topic
//! 559; § 469(c)(7) real estate professional; Pub. L. 119-21
//! (OBBBA 2025; § 1411 unchanged).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FilingStatus {
    /// § 1411(b)(2) — $200,000 threshold.
    Single,
    /// § 1411(b)(2) — $200,000 threshold.
    HeadOfHousehold,
    /// § 1411(b)(1) — $250,000 threshold.
    MarriedFilingJointly,
    /// § 1411(b)(1) — $250,000 threshold.
    QualifyingSurvivingSpouse,
    /// § 1411(b)(3) — $125,000 threshold.
    MarriedFilingSeparately,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section1411Input {
    pub filing_status: FilingStatus,
    /// Modified adjusted gross income (MAGI) in cents.
    pub magi_cents: i64,
    /// Net investment income (NII) before § 469(c)(7) carve-
    /// out, in cents.
    pub net_investment_income_cents: i64,
    /// Rental income included in NII, in cents (subject to §
    /// 469(c)(7) real-estate-professional carve-out).
    pub rental_income_in_nii_cents: i64,
    /// Whether taxpayer qualifies as real estate
    /// professional under § 469(c)(7).
    pub real_estate_professional: bool,
    /// Qualified retirement plan distributions in NII, in
    /// cents (always excluded under § 1411(c)(5)).
    pub qualified_retirement_distributions_cents: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section1411Result {
    pub niit_owed_cents: i64,
    pub applicable_threshold_cents: i64,
    pub magi_excess_over_threshold_cents: i64,
    pub adjusted_nii_cents: i64,
    pub threshold_engaged: bool,
    pub real_estate_professional_carve_out_engaged: bool,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section1411Input) -> Section1411Result {
    let threshold: i64 = match input.filing_status {
        FilingStatus::Single | FilingStatus::HeadOfHousehold => 20_000_000_000,
        FilingStatus::MarriedFilingJointly | FilingStatus::QualifyingSurvivingSpouse => {
            25_000_000_000
        }
        FilingStatus::MarriedFilingSeparately => 12_500_000_000,
    };

    let magi = input.magi_cents.max(0);
    let nii_raw = input.net_investment_income_cents.max(0);
    let rental_in_nii = input.rental_income_in_nii_cents.max(0).min(nii_raw);
    let retirement_distributions = input
        .qualified_retirement_distributions_cents
        .max(0)
        .min(nii_raw);

    let rep_carve_out_engaged = input.real_estate_professional;
    let rental_excluded = if rep_carve_out_engaged {
        rental_in_nii
    } else {
        0
    };

    let adjusted_nii = nii_raw
        .saturating_sub(rental_excluded)
        .saturating_sub(retirement_distributions);

    let magi_excess = magi.saturating_sub(threshold).max(0);
    let threshold_engaged = magi > threshold;

    let niit_base = adjusted_nii.min(magi_excess);
    let niit_owed = if threshold_engaged {
        niit_base.saturating_mul(38) / 1000
    } else {
        0
    };

    let notes: Vec<String> = vec![
        "26 USC § 1411(a)(1) — NIIT = 3.8% × LESSER of (1) net investment income or (2) excess of MAGI over applicable threshold; tax owed only when MAGI exceeds threshold"
            .to_string(),
        "26 USC § 1411(b) MAGI thresholds (NOT indexed for inflation, same since 2013): Single/HoH $200,000; MFJ/QSS $250,000; MFS $125,000"
            .to_string(),
        "26 USC § 1411(c)(1) NII categories: interest + dividends + capital gains + passive rental income + royalties + non-qualified annuity income; § 1411(c)(2) trade or business carve-out for material participation; § 1411(c)(5) qualified retirement distributions EXCLUDED"
            .to_string(),
        "26 USC § 469(c)(7) real estate professional carve-out — if taxpayer performs ≥ 750 hours per year in real property trades or businesses AND more than half of personal services in real property, rental income may be treated as ACTIVE and excluded from NII"
            .to_string(),
        "Pub. L. 119-21 (One Big Beautiful Bill Act of 2025) — did NOT repeal, amend, or modify § 1411; 3.8% rate + thresholds + categories + retirement-plan exception remain identical to 2013 form"
            .to_string(),
    ];

    Section1411Result {
        niit_owed_cents: niit_owed,
        applicable_threshold_cents: threshold,
        magi_excess_over_threshold_cents: magi_excess,
        adjusted_nii_cents: adjusted_nii,
        threshold_engaged,
        real_estate_professional_carve_out_engaged: rep_carve_out_engaged
            && rental_excluded > 0,
        citation: "26 USC § 1411(a)(1), (b)(1)-(3), (c)(1)(A)-(B), (c)(2), (c)(5); IRS Form 8960 (2025); IRS Topic 559; § 469(c)(7); Pub. L. 119-21",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn single_high_income() -> Section1411Input {
        Section1411Input {
            filing_status: FilingStatus::Single,
            magi_cents: 30_000_000_000,
            net_investment_income_cents: 5_000_000_000,
            rental_income_in_nii_cents: 0,
            real_estate_professional: false,
            qualified_retirement_distributions_cents: 0,
        }
    }

    fn mfj_high_income() -> Section1411Input {
        Section1411Input {
            filing_status: FilingStatus::MarriedFilingJointly,
            magi_cents: 35_000_000_000,
            net_investment_income_cents: 6_000_000_000,
            rental_income_in_nii_cents: 0,
            real_estate_professional: false,
            qualified_retirement_distributions_cents: 0,
        }
    }

    #[test]
    fn single_below_200k_no_tax() {
        let mut i = single_high_income();
        i.magi_cents = 19_999_999_999;
        let r = check(&i);
        assert_eq!(r.niit_owed_cents, 0);
        assert!(!r.threshold_engaged);
    }

    #[test]
    fn single_at_200k_boundary_no_tax() {
        let mut i = single_high_income();
        i.magi_cents = 20_000_000_000;
        let r = check(&i);
        assert!(!r.threshold_engaged);
        assert_eq!(r.niit_owed_cents, 0);
    }

    #[test]
    fn single_over_200k_engages_niit() {
        let r = check(&single_high_income());
        assert!(r.threshold_engaged);
        assert_eq!(r.applicable_threshold_cents, 20_000_000_000);
        assert_eq!(r.magi_excess_over_threshold_cents, 10_000_000_000);
        assert_eq!(r.adjusted_nii_cents, 5_000_000_000);
        assert_eq!(r.niit_owed_cents, 190_000_000);
    }

    #[test]
    fn mfj_at_250k_boundary_no_tax() {
        let mut i = mfj_high_income();
        i.magi_cents = 25_000_000_000;
        let r = check(&i);
        assert!(!r.threshold_engaged);
        assert_eq!(r.niit_owed_cents, 0);
    }

    #[test]
    fn mfj_over_250k_engages_niit() {
        let r = check(&mfj_high_income());
        assert!(r.threshold_engaged);
        assert_eq!(r.applicable_threshold_cents, 25_000_000_000);
        assert_eq!(r.magi_excess_over_threshold_cents, 10_000_000_000);
        assert_eq!(r.adjusted_nii_cents, 6_000_000_000);
        assert_eq!(r.niit_owed_cents, 228_000_000);
    }

    #[test]
    fn mfs_at_125k_boundary_no_tax() {
        let mut i = single_high_income();
        i.filing_status = FilingStatus::MarriedFilingSeparately;
        i.magi_cents = 12_500_000_000;
        let r = check(&i);
        assert!(!r.threshold_engaged);
        assert_eq!(r.applicable_threshold_cents, 12_500_000_000);
        assert_eq!(r.niit_owed_cents, 0);
    }

    #[test]
    fn mfs_over_125k_engages_niit() {
        let mut i = single_high_income();
        i.filing_status = FilingStatus::MarriedFilingSeparately;
        i.magi_cents = 15_000_000_000;
        let r = check(&i);
        assert!(r.threshold_engaged);
        assert_eq!(r.applicable_threshold_cents, 12_500_000_000);
        assert_eq!(r.magi_excess_over_threshold_cents, 2_500_000_000);
    }

    #[test]
    fn nii_less_than_magi_excess_uses_nii() {
        let mut i = single_high_income();
        i.magi_cents = 100_000_000_000;
        i.net_investment_income_cents = 1_000_000_000;
        let r = check(&i);
        assert_eq!(r.niit_owed_cents, 38_000_000);
    }

    #[test]
    fn magi_excess_less_than_nii_uses_magi_excess() {
        let mut i = single_high_income();
        i.magi_cents = 20_100_000_000;
        i.net_investment_income_cents = 10_000_000_000;
        let r = check(&i);
        assert_eq!(r.magi_excess_over_threshold_cents, 100_000_000);
        assert_eq!(r.niit_owed_cents, 3_800_000);
    }

    #[test]
    fn real_estate_professional_excludes_rental_from_nii() {
        let mut i = single_high_income();
        i.real_estate_professional = true;
        i.rental_income_in_nii_cents = 2_000_000_000;
        let r = check(&i);
        assert_eq!(r.adjusted_nii_cents, 3_000_000_000);
        assert!(r.real_estate_professional_carve_out_engaged);
    }

    #[test]
    fn non_real_estate_professional_keeps_rental_in_nii() {
        let mut i = single_high_income();
        i.real_estate_professional = false;
        i.rental_income_in_nii_cents = 2_000_000_000;
        let r = check(&i);
        assert_eq!(r.adjusted_nii_cents, 5_000_000_000);
        assert!(!r.real_estate_professional_carve_out_engaged);
    }

    #[test]
    fn retirement_distributions_always_excluded() {
        let mut i = single_high_income();
        i.qualified_retirement_distributions_cents = 1_000_000_000;
        let r = check(&i);
        assert_eq!(r.adjusted_nii_cents, 4_000_000_000);
    }

    #[test]
    fn citation_pins_all_subsections() {
        let r = check(&single_high_income());
        assert!(r.citation.contains("§ 1411(a)(1)"));
        assert!(r.citation.contains("(b)(1)-(3)"));
        assert!(r.citation.contains("(c)(1)(A)-(B)"));
        assert!(r.citation.contains("(c)(2)"));
        assert!(r.citation.contains("(c)(5)"));
        assert!(r.citation.contains("Form 8960"));
        assert!(r.citation.contains("§ 469(c)(7)"));
        assert!(r.citation.contains("Pub. L. 119-21"));
    }

    #[test]
    fn note_pins_3_8_percent_formula() {
        let r = check(&single_high_income());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 1411(a)(1)") && n.contains("3.8%") && n.contains("LESSER")));
    }

    #[test]
    fn note_pins_thresholds() {
        let r = check(&single_high_income());
        assert!(r.notes.iter().any(|n| n.contains("$200,000")
            && n.contains("$250,000")
            && n.contains("$125,000")
            && n.contains("NOT indexed")));
    }

    #[test]
    fn note_pins_categories() {
        let r = check(&single_high_income());
        assert!(r.notes.iter().any(|n| n.contains("§ 1411(c)(1)")
            && n.contains("interest")
            && n.contains("dividends")
            && n.contains("capital gains")
            && n.contains("retirement distributions EXCLUDED")));
    }

    #[test]
    fn note_pins_real_estate_professional_469c7() {
        let r = check(&single_high_income());
        assert!(r.notes.iter().any(|n| n.contains("§ 469(c)(7)")
            && n.contains("750 hours")
            && n.contains("half")
            && n.contains("ACTIVE")));
    }

    #[test]
    fn note_pins_obbba_2025_unchanged() {
        let r = check(&single_high_income());
        assert!(r.notes.iter().any(|n| n.contains("Pub. L. 119-21")
            && n.contains("did NOT repeal")
            && n.contains("3.8% rate")));
    }

    #[test]
    fn filing_status_threshold_truth_table() {
        for (status, exp_threshold) in [
            (FilingStatus::Single, 20_000_000_000_i64),
            (FilingStatus::HeadOfHousehold, 20_000_000_000),
            (FilingStatus::MarriedFilingJointly, 25_000_000_000),
            (FilingStatus::QualifyingSurvivingSpouse, 25_000_000_000),
            (FilingStatus::MarriedFilingSeparately, 12_500_000_000),
        ] {
            let mut i = single_high_income();
            i.filing_status = status;
            let r = check(&i);
            assert_eq!(r.applicable_threshold_cents, exp_threshold);
        }
    }

    #[test]
    fn defensive_negative_magi_clamped() {
        let mut i = single_high_income();
        i.magi_cents = -100_000_000;
        let r = check(&i);
        assert!(!r.threshold_engaged);
        assert_eq!(r.niit_owed_cents, 0);
    }

    #[test]
    fn defensive_negative_nii_clamped() {
        let mut i = single_high_income();
        i.net_investment_income_cents = -1_000_000_000;
        let r = check(&i);
        assert_eq!(r.adjusted_nii_cents, 0);
        assert_eq!(r.niit_owed_cents, 0);
    }

    #[test]
    fn defensive_negative_rental_clamped() {
        let mut i = single_high_income();
        i.real_estate_professional = true;
        i.rental_income_in_nii_cents = -1_000_000_000;
        let r = check(&i);
        assert_eq!(r.adjusted_nii_cents, 5_000_000_000);
    }

    #[test]
    fn defensive_overflow_saturating() {
        let mut i = single_high_income();
        i.magi_cents = i64::MAX - 1000;
        i.net_investment_income_cents = 100_000_000;
        let r = check(&i);
        assert!(r.threshold_engaged);
    }

    #[test]
    fn rental_carve_out_capped_at_nii() {
        let mut i = single_high_income();
        i.real_estate_professional = true;
        i.rental_income_in_nii_cents = 100_000_000_000;
        let r = check(&i);
        assert_eq!(r.adjusted_nii_cents, 0);
    }

    #[test]
    fn rep_uniquely_excludes_rental_invariant() {
        let mut i_rep = single_high_income();
        i_rep.real_estate_professional = true;
        i_rep.rental_income_in_nii_cents = 2_000_000_000;
        let r_rep = check(&i_rep);
        assert!(r_rep.real_estate_professional_carve_out_engaged);

        let mut i_non = single_high_income();
        i_non.real_estate_professional = false;
        i_non.rental_income_in_nii_cents = 2_000_000_000;
        let r_non = check(&i_non);
        assert!(!r_non.real_estate_professional_carve_out_engaged);
    }

    #[test]
    fn mfj_uniquely_highest_threshold_invariant() {
        let mut i_mfj = single_high_income();
        i_mfj.filing_status = FilingStatus::MarriedFilingJointly;
        let r_mfj = check(&i_mfj);
        assert_eq!(r_mfj.applicable_threshold_cents, 25_000_000_000);

        let mut i_single = single_high_income();
        i_single.filing_status = FilingStatus::Single;
        let r_single = check(&i_single);
        assert!(r_mfj.applicable_threshold_cents > r_single.applicable_threshold_cents);

        let mut i_mfs = single_high_income();
        i_mfs.filing_status = FilingStatus::MarriedFilingSeparately;
        let r_mfs = check(&i_mfs);
        assert!(r_mfj.applicable_threshold_cents > r_mfs.applicable_threshold_cents);
    }

    #[test]
    fn mfs_uniquely_lowest_threshold_invariant() {
        let mut i_mfs = single_high_income();
        i_mfs.filing_status = FilingStatus::MarriedFilingSeparately;
        let r_mfs = check(&i_mfs);
        assert_eq!(r_mfs.applicable_threshold_cents, 12_500_000_000);

        for status in [
            FilingStatus::Single,
            FilingStatus::HeadOfHousehold,
            FilingStatus::MarriedFilingJointly,
            FilingStatus::QualifyingSurvivingSpouse,
        ] {
            let mut i = single_high_income();
            i.filing_status = status;
            let r = check(&i);
            assert!(r.applicable_threshold_cents > r_mfs.applicable_threshold_cents);
        }
    }

    #[test]
    fn three_eight_percent_precision_at_round_amount() {
        let mut i = single_high_income();
        i.magi_cents = 30_000_000_000;
        i.net_investment_income_cents = 1_000_000_000;
        let r = check(&i);
        assert_eq!(r.niit_owed_cents, 38_000_000);
    }

    #[test]
    fn rep_and_retirement_both_excluded_compounding() {
        let mut i = single_high_income();
        i.real_estate_professional = true;
        i.rental_income_in_nii_cents = 1_000_000_000;
        i.qualified_retirement_distributions_cents = 1_000_000_000;
        let r = check(&i);
        assert_eq!(r.adjusted_nii_cents, 3_000_000_000);
    }

    #[test]
    fn nii_zero_no_niit_even_above_threshold() {
        let mut i = single_high_income();
        i.net_investment_income_cents = 0;
        let r = check(&i);
        assert!(r.threshold_engaged);
        assert_eq!(r.adjusted_nii_cents, 0);
        assert_eq!(r.niit_owed_cents, 0);
    }
}
