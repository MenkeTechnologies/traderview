//! IRC § 530 — Coverdell Education Savings Accounts (ESA).
//!
//! § 530 creates tax-favored education savings accounts for designated
//! beneficiaries. Contributions are NOT deductible but earnings grow
//! tax-free and qualified withdrawals (for K-12 and post-secondary
//! education expenses) are tax-free. Distinct from § 529 plans (which
//! got their own K-12 expansion to $20K under OBBBA effective 2026 but
//! are administered separately under § 529).
//!
//! Contribution limit (§ 530(b)(1)(A)(ii)): **$2,000 per beneficiary
//! per year**. This figure is STATUTORY and has NOT been increased
//! since 2002. The limit is AGGREGATE — total of all contributors'
//! contributions to any ESA for the same beneficiary cannot exceed
//! $2,000.
//!
//! Beneficiary age limit (§ 530(b)(1)(A)(i)): contributions only
//! permitted while beneficiary is under age 18. Account must be
//! distributed by age 30 (§ 530(d)(8)). **Special-needs exception**:
//! both age limits are waived for special-needs beneficiaries
//! (§ 530(d)(7)).
//!
//! MAGI phaseout (§ 530(c)) over $15K window for unmarried filers /
//! $30K for joint:
//!
//! Single / HoH / QW: **$95,000-$110,000**.
//!
//! Married filing jointly / QW: **$190,000-$220,000**.
//!
//! Excess contribution (§ 4973) — 6% excise tax on excess imposed on
//! the BENEFICIARY (not the contributor) each year the excess remains
//! in the account.
//!
//! Citations: 26 U.S.C. § 530; § 530(b)(1)(A)(ii) ($2,000 statutory
//! limit); § 530(b)(1)(A)(i) (under-age-18 contribution age limit);
//! § 530(c) (MAGI phaseout); § 530(d)(7) (special-needs waiver);
//! § 530(d)(8) (age-30 distribution requirement); § 4973 (6% excise
//! tax on excess contributions).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilingStatus {
    Single,
    HeadOfHousehold,
    MarriedFilingJointly,
    MarriedFilingSeparately,
    QualifyingWidow,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section530Input {
    pub year: u32,
    pub contributor_filing_status: FilingStatus,
    pub contributor_modified_agi_cents: i64,
    pub beneficiary_age_years: u32,
    pub beneficiary_special_needs: bool,
    /// Total contributions to all Coverdell ESAs for this beneficiary
    /// from ALL contributors. Drives the §4973 excess test (the cap is
    /// aggregate across all contributors, not per-contributor).
    pub aggregate_contributions_for_beneficiary_cents: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section530Result {
    pub statutory_limit_cents: i64,
    pub phaseout_low_cents: i64,
    pub phaseout_high_cents: i64,
    pub in_phaseout_range: bool,
    pub contributor_max_contribution_cents: i64,
    pub beneficiary_eligible_for_contributions: bool,
    pub excess_contribution_cents: i64,
    pub excise_tax_under_4973_cents: i64,
    pub citation: &'static str,
    pub note: String,
}

pub fn compute(input: &Section530Input) -> Section530Result {
    let aggregate = input.aggregate_contributions_for_beneficiary_cents.max(0);
    let magi = input.contributor_modified_agi_cents.max(0);

    // Statutory cap (since 2002): $2,000.
    let cap = 200000;

    // Beneficiary age limit: under 18 unless special needs.
    let age_eligible = input.beneficiary_age_years < 18 || input.beneficiary_special_needs;

    let (low, high) = phaseout_range(input.contributor_filing_status);

    let (in_phaseout, contributor_max) = if !age_eligible {
        (false, 0)
    } else if magi <= low {
        (false, cap)
    } else if magi >= high {
        (true, 0)
    } else {
        let range = high - low;
        let remaining = high - magi;
        let reduced = ((cap as i128) * (remaining as i128) / (range as i128)) as i64;
        (true, reduced.max(0))
    };

    let excess = (aggregate - cap).max(0);
    let excise_tax = ((excess as i128) * 6 / 100) as i64;

    let note = format!(
        "Coverdell ESA for {} (beneficiary age {}, special-needs = {}). Statutory cap = $2,000. MAGI = {} cents; phaseout {} cents to {} cents ({:?}). {} contributor max = {} cents. Aggregate contributions {} cents; excess = {} cents → § 4973 6% excise tax = {} cents (imposed on beneficiary).",
        input.year,
        input.beneficiary_age_years,
        input.beneficiary_special_needs,
        magi,
        low,
        high,
        input.contributor_filing_status,
        if in_phaseout { "IN-PHASEOUT" } else { "below phaseout" },
        contributor_max,
        aggregate,
        excess,
        excise_tax,
    );

    Section530Result {
        statutory_limit_cents: cap,
        phaseout_low_cents: low,
        phaseout_high_cents: high,
        in_phaseout_range: in_phaseout,
        contributor_max_contribution_cents: contributor_max,
        beneficiary_eligible_for_contributions: age_eligible,
        excess_contribution_cents: excess,
        excise_tax_under_4973_cents: excise_tax,
        citation:
            "26 U.S.C. § 530(b)(1)(A)(ii) — $2,000 statutory annual contribution limit per beneficiary (unchanged since 2002); § 530(c) MAGI phaseout; § 530(b)(1)(A)(i) under-age-18 limit + § 530(d)(7) special-needs waiver; § 4973 6% excise tax on excess",
        note,
    }
}

fn phaseout_range(fs: FilingStatus) -> (i64, i64) {
    match fs {
        FilingStatus::MarriedFilingJointly | FilingStatus::QualifyingWidow => (19000000, 22000000),
        _ => (9500000, 11000000),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        year: u32,
        fs: FilingStatus,
        magi: i64,
        beneficiary_age: u32,
        special_needs: bool,
        aggregate: i64,
    ) -> Section530Input {
        Section530Input {
            year,
            contributor_filing_status: fs,
            contributor_modified_agi_cents: magi,
            beneficiary_age_years: beneficiary_age,
            beneficiary_special_needs: special_needs,
            aggregate_contributions_for_beneficiary_cents: aggregate,
        }
    }

    #[test]
    fn single_under_95k_full_2000_contribution() {
        let r = compute(&input(2026, FilingStatus::Single, 50_000_00, 10, false, 0));
        assert_eq!(r.contributor_max_contribution_cents, 2_000_00);
        assert!(!r.in_phaseout_range);
        assert!(r.beneficiary_eligible_for_contributions);
    }

    #[test]
    fn single_at_95k_boundary_full() {
        let r = compute(&input(2026, FilingStatus::Single, 95_000_00, 10, false, 0));
        assert_eq!(r.contributor_max_contribution_cents, 2_000_00);
        assert!(!r.in_phaseout_range);
    }

    #[test]
    fn single_at_110k_boundary_zero() {
        let r = compute(&input(2026, FilingStatus::Single, 110_000_00, 10, false, 0));
        assert_eq!(r.contributor_max_contribution_cents, 0);
        assert!(r.in_phaseout_range);
    }

    #[test]
    fn single_midpoint_102500_half() {
        let r = compute(&input(2026, FilingStatus::Single, 102_500_00, 10, false, 0));
        assert_eq!(r.contributor_max_contribution_cents, 1_000_00);
        assert!(r.in_phaseout_range);
    }

    #[test]
    fn mfj_under_190k_full() {
        let r = compute(&input(
            2026,
            FilingStatus::MarriedFilingJointly,
            150_000_00,
            10,
            false,
            0,
        ));
        assert_eq!(r.contributor_max_contribution_cents, 2_000_00);
        assert_eq!(r.phaseout_low_cents, 190_000_00);
        assert_eq!(r.phaseout_high_cents, 220_000_00);
    }

    #[test]
    fn mfj_at_220k_zero() {
        let r = compute(&input(
            2026,
            FilingStatus::MarriedFilingJointly,
            220_000_00,
            10,
            false,
            0,
        ));
        assert_eq!(r.contributor_max_contribution_cents, 0);
    }

    #[test]
    fn beneficiary_age_18_not_eligible_unless_special_needs() {
        let r = compute(&input(2026, FilingStatus::Single, 50_000_00, 18, false, 0));
        assert!(!r.beneficiary_eligible_for_contributions);
        assert_eq!(r.contributor_max_contribution_cents, 0);
    }

    #[test]
    fn beneficiary_age_17_eligible() {
        let r = compute(&input(2026, FilingStatus::Single, 50_000_00, 17, false, 0));
        assert!(r.beneficiary_eligible_for_contributions);
        assert_eq!(r.contributor_max_contribution_cents, 2_000_00);
    }

    #[test]
    fn beneficiary_age_30_special_needs_still_eligible() {
        // § 530(d)(7) waives age limit for special-needs beneficiary.
        let r = compute(&input(2026, FilingStatus::Single, 50_000_00, 30, true, 0));
        assert!(r.beneficiary_eligible_for_contributions);
    }

    #[test]
    fn excess_contribution_triggers_6_percent_excise() {
        // $3,000 contributed vs $2,000 cap → $1,000 excess → $60 excise.
        let r = compute(&input(
            2026,
            FilingStatus::Single,
            50_000_00,
            10,
            false,
            3_000_00,
        ));
        assert_eq!(r.excess_contribution_cents, 1_000_00);
        assert_eq!(r.excise_tax_under_4973_cents, 60_00);
    }

    #[test]
    fn no_excess_at_cap_boundary() {
        let r = compute(&input(
            2026,
            FilingStatus::Single,
            50_000_00,
            10,
            false,
            2_000_00,
        ));
        assert_eq!(r.excess_contribution_cents, 0);
    }

    #[test]
    fn one_cent_above_cap_triggers_excise() {
        let r = compute(&input(
            2026,
            FilingStatus::Single,
            50_000_00,
            10,
            false,
            2_000_01,
        ));
        assert_eq!(r.excess_contribution_cents, 1);
        // 6% × 1 cent = 0 cents (rounded).
        assert_eq!(r.excise_tax_under_4973_cents, 0);
    }

    #[test]
    fn statutory_2000_cap_does_not_inflation_adjust() {
        // Both 2025 and 2026 (and forever) use same $2,000 statutory cap.
        let r_2025 = compute(&input(2025, FilingStatus::Single, 50_000_00, 10, false, 0));
        let r_2026 = compute(&input(2026, FilingStatus::Single, 50_000_00, 10, false, 0));
        let r_2030 = compute(&input(2030, FilingStatus::Single, 50_000_00, 10, false, 0));
        assert_eq!(r_2025.statutory_limit_cents, 2_000_00);
        assert_eq!(r_2026.statutory_limit_cents, 2_000_00);
        assert_eq!(r_2030.statutory_limit_cents, 2_000_00);
        assert_eq!(r_2025.contributor_max_contribution_cents, 2_000_00);
        assert_eq!(r_2026.contributor_max_contribution_cents, 2_000_00);
        assert_eq!(r_2030.contributor_max_contribution_cents, 2_000_00);
    }

    #[test]
    fn mfj_phaseout_midpoint_205k_half() {
        // MFJ midpoint $205K of $190K-$220K → 50% × $2000 = $1000.
        let r = compute(&input(
            2026,
            FilingStatus::MarriedFilingJointly,
            205_000_00,
            10,
            false,
            0,
        ));
        assert!(r.in_phaseout_range);
        assert_eq!(r.contributor_max_contribution_cents, 1_000_00);
    }

    #[test]
    fn hoh_uses_single_phaseout_range() {
        let r = compute(&input(
            2026,
            FilingStatus::HeadOfHousehold,
            100_000_00,
            10,
            false,
            0,
        ));
        assert_eq!(r.phaseout_low_cents, 95_000_00);
        assert_eq!(r.phaseout_high_cents, 110_000_00);
    }

    #[test]
    fn qw_uses_mfj_phaseout_range() {
        let r = compute(&input(
            2026,
            FilingStatus::QualifyingWidow,
            200_000_00,
            10,
            false,
            0,
        ));
        assert_eq!(r.phaseout_low_cents, 190_000_00);
        assert_eq!(r.phaseout_high_cents, 220_000_00);
    }

    #[test]
    fn mfs_uses_single_range() {
        // MFS gets single-status treatment (NOT half of MFJ — same as
        // section_221 student-loan deduction).
        let r = compute(&input(
            2026,
            FilingStatus::MarriedFilingSeparately,
            100_000_00,
            10,
            false,
            0,
        ));
        assert_eq!(r.phaseout_low_cents, 95_000_00);
        assert_eq!(r.phaseout_high_cents, 110_000_00);
    }

    #[test]
    fn citations_pin_correct_authorities() {
        let r = compute(&input(2026, FilingStatus::Single, 50_000_00, 10, false, 0));
        assert!(r.citation.contains("§ 530(b)(1)(A)(ii)"));
        assert!(r.citation.contains("$2,000"));
        assert!(r.citation.contains("unchanged since 2002"));
        assert!(r.citation.contains("§ 530(c)"));
        assert!(r.citation.contains("§ 530(d)(7)"));
        assert!(r.citation.contains("§ 4973"));
    }

    #[test]
    fn negative_inputs_clamped() {
        let r = compute(&input(2026, FilingStatus::Single, -1, 10, false, -1));
        assert_eq!(r.excess_contribution_cents, 0);
    }

    #[test]
    fn beneficiary_18_with_special_needs_eligible() {
        let r = compute(&input(2026, FilingStatus::Single, 50_000_00, 18, true, 0));
        assert!(r.beneficiary_eligible_for_contributions);
    }

    #[test]
    fn high_excess_high_excise() {
        // $10K contribution → $8K excess → 6% × $8K = $480.
        let r = compute(&input(
            2026,
            FilingStatus::Single,
            50_000_00,
            10,
            false,
            10_000_00,
        ));
        assert_eq!(r.excess_contribution_cents, 8_000_00);
        assert_eq!(r.excise_tax_under_4973_cents, 480_00);
    }

    #[test]
    fn worked_example_grandparent_contribution_phaseout() {
        // Grandparent at $105K MAGI single — midpoint of phaseout range.
        // $105K is 2/3 of way through $95K-$110K → $2000 × 5/15 = $666.67.
        let r = compute(&input(2026, FilingStatus::Single, 105_000_00, 10, false, 0));
        assert!(r.in_phaseout_range);
        // Phaseout reduction: (110 - 105)/15 = 5/15 = 1/3 → contributor
        // max = $2000 × 5/15 = $666.66 → 66666 cents (truncated).
        let expected = (2_000_00_i128 * 5_000_00 / 15_000_00) as i64;
        assert_eq!(r.contributor_max_contribution_cents, expected);
    }
}
