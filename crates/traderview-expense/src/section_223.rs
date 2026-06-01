//! IRC § 223 — Health Savings Accounts (HSAs).
//!
//! § 223 provides triple tax-advantaged accounts: contributions are
//! deductible above-the-line under § 223(a); earnings grow tax-free;
//! withdrawals for qualified medical expenses are tax-free. To
//! contribute, the taxpayer must be covered by a "High Deductible Health
//! Plan" (HDHP) meeting three statutory tests under § 223(c)(2):
//! minimum annual deductible, capped out-of-pocket maximum, and no
//! payment for non-preventive care before the deductible is met.
//!
//! 2026 inflation-adjusted amounts (Rev. Proc. 2025-19):
//!
//! Contribution limits (§ 223(b)(2)): $4,400 self-only HDHP coverage;
//! $8,750 family HDHP coverage.
//!
//! Catch-up for age 55+ (§ 223(b)(3)): additional $1,000. This figure
//! is STATUTORY and does NOT adjust for inflation. Combined with the
//! base limit yields $5,400 self-only or $9,750 family for age-55+
//! account holders.
//!
//! HDHP self-only thresholds (§ 223(c)(2)): minimum deductible $1,700;
//! maximum in-network out-of-pocket $8,500.
//!
//! HDHP family thresholds (§ 223(c)(2)): minimum deductible $3,400;
//! maximum in-network out-of-pocket $17,000.
//!
//! Excess contribution (§ 4973) is subject to a 6% excise tax each
//! year the excess remains in the account (not modeled here — caller
//! should apply via § 4973 module).
//!
//! Citations: 26 U.S.C. § 223; § 223(a) (deduction); § 223(b)(2)
//! (contribution limits); § 223(b)(3) ($1,000 catch-up); § 223(c)(2)
//! (HDHP definition); § 4973 (6% excise tax on excess contributions);
//! Rev. Proc. 2025-19 (2026 inflation-adjusted amounts).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoverageType {
    SelfOnly,
    Family,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section223Input {
    pub year: u32,
    pub coverage_type: CoverageType,
    pub age: u32,
    pub contributions_cents: i64,
    pub hdhp_deductible_cents: i64,
    pub hdhp_out_of_pocket_max_cents: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section223Result {
    pub hdhp_eligible: bool,
    pub hdhp_minimum_deductible_cents: i64,
    pub hdhp_maximum_out_of_pocket_cents: i64,
    pub base_contribution_limit_cents: i64,
    pub catch_up_amount_cents: i64,
    pub total_contribution_limit_cents: i64,
    pub allowed_deduction_cents: i64,
    pub excess_contribution_cents: i64,
    pub excise_tax_under_4973_cents: i64,
    pub citation: &'static str,
    pub note: String,
}

pub fn compute(input: &Section223Input) -> Section223Result {
    let contributions = input.contributions_cents.max(0);
    let deductible = input.hdhp_deductible_cents.max(0);
    let oop_max = input.hdhp_out_of_pocket_max_cents.max(0);

    let (min_deductible, max_oop, base_limit) = thresholds(input.year, input.coverage_type);
    let hdhp_eligible = deductible >= min_deductible && oop_max <= max_oop && oop_max > 0;

    if !hdhp_eligible {
        return Section223Result {
            hdhp_eligible: false,
            hdhp_minimum_deductible_cents: min_deductible,
            hdhp_maximum_out_of_pocket_cents: max_oop,
            base_contribution_limit_cents: 0,
            catch_up_amount_cents: 0,
            total_contribution_limit_cents: 0,
            allowed_deduction_cents: 0,
            excess_contribution_cents: contributions,
            excise_tax_under_4973_cents: (contributions as i128 * 6 / 100) as i64,
            citation: "26 U.S.C. § 223(c)(2) — HSA contributions require HDHP coverage; plan must meet minimum deductible AND maximum out-of-pocket tests",
            note: format!(
                "Plan does NOT qualify as HDHP for {} {:?} coverage. Required: deductible ≥ {} cents, OOP max ≤ {} cents. Plan: deductible {}, OOP max {}. All ${} of contributions are EXCESS subject to 6% § 4973 excise tax.",
                input.year, input.coverage_type, min_deductible, max_oop, deductible, oop_max, contributions / 100
            ),
        };
    }

    let catch_up = if input.age >= 55 { 100000 } else { 0 };
    let total_limit = base_limit + catch_up;
    let allowed_deduction = contributions.min(total_limit);
    let excess = (contributions - total_limit).max(0);
    let excise_tax = (excess as i128 * 6 / 100) as i64;

    let note = format!(
        "HDHP qualifies: deductible {} ≥ minimum {}; OOP max {} ≤ maximum {}. Base limit {} + catch-up (age {}{}) = total limit {}. Contributions {} → allowed deduction {}, excess {} subject to 6% § 4973 excise tax = {}.",
        deductible,
        min_deductible,
        oop_max,
        max_oop,
        base_limit,
        input.age,
        if input.age >= 55 { " ≥ 55, +$1,000" } else { ", no catch-up" },
        total_limit,
        contributions,
        allowed_deduction,
        excess,
        excise_tax,
    );

    Section223Result {
        hdhp_eligible: true,
        hdhp_minimum_deductible_cents: min_deductible,
        hdhp_maximum_out_of_pocket_cents: max_oop,
        base_contribution_limit_cents: base_limit,
        catch_up_amount_cents: catch_up,
        total_contribution_limit_cents: total_limit,
        allowed_deduction_cents: allowed_deduction,
        excess_contribution_cents: excess,
        excise_tax_under_4973_cents: excise_tax,
        citation:
            "26 U.S.C. § 223(a)/(b)(2)/(b)(3)/(c)(2) — HSA deduction with base + catch-up limits + HDHP thresholds; § 4973 6% excise tax on excess",
        note,
    }
}

/// Returns (HDHP minimum deductible, HDHP max OOP, base contribution
/// limit) for the year and coverage type. 2026 amounts per Rev. Proc.
/// 2025-19; 2025 amounts per Rev. Proc. 2024-25.
fn thresholds(year: u32, coverage: CoverageType) -> (i64, i64, i64) {
    match (year, coverage) {
        // 2026 inflation-adjusted (Rev. Proc. 2025-19).
        (2026, CoverageType::SelfOnly) => (170000, 850000, 440000),
        (2026, CoverageType::Family) => (340000, 1700000, 875000),
        // 2025 (Rev. Proc. 2024-25).
        (2025, CoverageType::SelfOnly) => (165000, 830000, 430000),
        (2025, CoverageType::Family) => (330000, 1660000, 855000),
        // Fallback for unsupported years — model uses 2026.
        (_, CoverageType::SelfOnly) => (170000, 850000, 440000),
        (_, CoverageType::Family) => (340000, 1700000, 875000),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        year: u32,
        coverage: CoverageType,
        age: u32,
        contributions: i64,
        deductible: i64,
        oop_max: i64,
    ) -> Section223Input {
        Section223Input {
            year,
            coverage_type: coverage,
            age,
            contributions_cents: contributions,
            hdhp_deductible_cents: deductible,
            hdhp_out_of_pocket_max_cents: oop_max,
        }
    }

    #[test]
    fn self_only_2026_under_55_full_limit() {
        // 2026 self-only base $4,400. Age 40 → no catch-up.
        let r = compute(&input(
            2026,
            CoverageType::SelfOnly,
            40,
            4_400_00,
            2_000_00,
            7_000_00,
        ));
        assert!(r.hdhp_eligible);
        assert_eq!(r.base_contribution_limit_cents, 4_400_00);
        assert_eq!(r.catch_up_amount_cents, 0);
        assert_eq!(r.total_contribution_limit_cents, 4_400_00);
        assert_eq!(r.allowed_deduction_cents, 4_400_00);
        assert_eq!(r.excess_contribution_cents, 0);
    }

    #[test]
    fn self_only_2026_age_55_catch_up_added() {
        // 2026 self-only + age 55 catch-up: $4,400 + $1,000 = $5,400.
        let r = compute(&input(
            2026,
            CoverageType::SelfOnly,
            55,
            5_400_00,
            2_000_00,
            7_000_00,
        ));
        assert_eq!(r.catch_up_amount_cents, 1_000_00);
        assert_eq!(r.total_contribution_limit_cents, 5_400_00);
        assert_eq!(r.allowed_deduction_cents, 5_400_00);
    }

    #[test]
    fn family_2026_under_55_full_limit() {
        // 2026 family base $8,750.
        let r = compute(&input(
            2026,
            CoverageType::Family,
            40,
            8_750_00,
            4_000_00,
            14_000_00,
        ));
        assert!(r.hdhp_eligible);
        assert_eq!(r.base_contribution_limit_cents, 8_750_00);
        assert_eq!(r.total_contribution_limit_cents, 8_750_00);
    }

    #[test]
    fn family_2026_age_55_catch_up() {
        // Family + age 55 catch-up: $8,750 + $1,000 = $9,750.
        let r = compute(&input(
            2026,
            CoverageType::Family,
            65,
            9_750_00,
            4_000_00,
            14_000_00,
        ));
        assert_eq!(r.total_contribution_limit_cents, 9_750_00);
        assert_eq!(r.allowed_deduction_cents, 9_750_00);
    }

    #[test]
    fn deductible_below_minimum_disqualifies() {
        // 2026 self-only min $1,700. Plan deductible $1,500.
        let r = compute(&input(
            2026,
            CoverageType::SelfOnly,
            40,
            4_400_00,
            1_500_00,
            7_000_00,
        ));
        assert!(!r.hdhp_eligible);
        assert_eq!(r.allowed_deduction_cents, 0);
        // All contributions excess, 6% × $4,400 = $264 excise.
        assert_eq!(r.excess_contribution_cents, 4_400_00);
        assert_eq!(r.excise_tax_under_4973_cents, 264_00);
    }

    #[test]
    fn deductible_at_1700_boundary_qualifies() {
        let r = compute(&input(
            2026,
            CoverageType::SelfOnly,
            40,
            4_400_00,
            1_700_00,
            7_000_00,
        ));
        assert!(r.hdhp_eligible);
    }

    #[test]
    fn oop_max_above_8500_disqualifies() {
        // 2026 self-only max OOP $8,500. Plan OOP $9,000 → disqualifies.
        let r = compute(&input(
            2026,
            CoverageType::SelfOnly,
            40,
            4_400_00,
            2_000_00,
            9_000_00,
        ));
        assert!(!r.hdhp_eligible);
    }

    #[test]
    fn oop_max_at_8500_boundary_qualifies() {
        let r = compute(&input(
            2026,
            CoverageType::SelfOnly,
            40,
            4_400_00,
            2_000_00,
            8_500_00,
        ));
        assert!(r.hdhp_eligible);
    }

    #[test]
    fn excess_contribution_triggers_6_percent_excise() {
        // 2026 self-only limit $4,400. Contribute $5,000 → $600 excess.
        // 6% × $600 = $36.
        let r = compute(&input(
            2026,
            CoverageType::SelfOnly,
            40,
            5_000_00,
            2_000_00,
            7_000_00,
        ));
        assert_eq!(r.excess_contribution_cents, 600_00);
        assert_eq!(r.excise_tax_under_4973_cents, 36_00);
    }

    #[test]
    fn excess_contribution_age_55_calculated_against_higher_limit() {
        // Age 55 → $5,400 limit. Contribute $6,000 → $600 excess.
        let r = compute(&input(
            2026,
            CoverageType::SelfOnly,
            55,
            6_000_00,
            2_000_00,
            7_000_00,
        ));
        assert_eq!(r.excess_contribution_cents, 600_00);
    }

    #[test]
    fn under_contribution_no_excess() {
        let r = compute(&input(
            2026,
            CoverageType::SelfOnly,
            40,
            2_000_00,
            2_000_00,
            7_000_00,
        ));
        assert_eq!(r.allowed_deduction_cents, 2_000_00);
        assert_eq!(r.excess_contribution_cents, 0);
    }

    #[test]
    fn family_2026_inflation_adjusted_8750() {
        let r = compute(&input(
            2026,
            CoverageType::Family,
            40,
            8_750_00,
            4_000_00,
            14_000_00,
        ));
        assert_eq!(r.base_contribution_limit_cents, 8_750_00);
    }

    #[test]
    fn self_only_2025_4300() {
        // 2025 self-only base was $4,300.
        let r = compute(&input(
            2025,
            CoverageType::SelfOnly,
            40,
            4_300_00,
            2_000_00,
            7_000_00,
        ));
        assert_eq!(r.base_contribution_limit_cents, 4_300_00);
        assert_eq!(r.hdhp_minimum_deductible_cents, 1_650_00);
    }

    #[test]
    fn family_2025_8550() {
        let r = compute(&input(
            2025,
            CoverageType::Family,
            40,
            8_550_00,
            4_000_00,
            14_000_00,
        ));
        assert_eq!(r.base_contribution_limit_cents, 8_550_00);
    }

    #[test]
    fn catch_up_1000_does_not_inflation_adjust() {
        // The $1,000 catch-up is statutory and never adjusts.
        let r2025 = compute(&input(
            2025,
            CoverageType::SelfOnly,
            55,
            5_300_00,
            2_000_00,
            7_000_00,
        ));
        let r2026 = compute(&input(
            2026,
            CoverageType::SelfOnly,
            55,
            5_400_00,
            2_000_00,
            7_000_00,
        ));
        assert_eq!(r2025.catch_up_amount_cents, 1_000_00);
        assert_eq!(r2026.catch_up_amount_cents, 1_000_00);
    }

    #[test]
    fn age_54_no_catch_up() {
        let r = compute(&input(
            2026,
            CoverageType::SelfOnly,
            54,
            4_400_00,
            2_000_00,
            7_000_00,
        ));
        assert_eq!(r.catch_up_amount_cents, 0);
    }

    #[test]
    fn age_55_boundary_catch_up_applies() {
        let r = compute(&input(
            2026,
            CoverageType::SelfOnly,
            55,
            5_400_00,
            2_000_00,
            7_000_00,
        ));
        assert_eq!(r.catch_up_amount_cents, 1_000_00);
    }

    #[test]
    fn zero_oop_max_disqualifies() {
        // OOP max = 0 doesn't represent a valid HDHP — disqualify.
        let r = compute(&input(
            2026,
            CoverageType::SelfOnly,
            40,
            4_400_00,
            2_000_00,
            0,
        ));
        assert!(!r.hdhp_eligible);
    }

    #[test]
    fn negative_inputs_clamped() {
        let r = compute(&input(2026, CoverageType::SelfOnly, 40, -1, -1, -1));
        assert!(!r.hdhp_eligible);
        assert_eq!(r.excess_contribution_cents, 0);
    }

    #[test]
    fn citations_pin_correct_authorities() {
        let r = compute(&input(
            2026,
            CoverageType::SelfOnly,
            55,
            5_400_00,
            2_000_00,
            7_000_00,
        ));
        assert!(r.citation.contains("§ 223(a)"));
        assert!(r.citation.contains("(b)(2)"));
        assert!(r.citation.contains("(b)(3)"));
        assert!(r.citation.contains("(c)(2)"));
        assert!(r.citation.contains("§ 4973"));

        let disqualified = compute(&input(
            2026,
            CoverageType::SelfOnly,
            40,
            5_000_00,
            1_500_00,
            7_000_00,
        ));
        assert!(disqualified.citation.contains("§ 223(c)(2)"));
    }

    #[test]
    fn note_includes_excise_tax_amount() {
        let r = compute(&input(
            2026,
            CoverageType::SelfOnly,
            40,
            5_000_00,
            2_000_00,
            7_000_00,
        ));
        assert!(r.note.contains("6% § 4973 excise tax"));
        assert!(r.note.contains("3600"));
    }

    #[test]
    fn family_min_deductible_double_self_only() {
        // Family min deductible should be roughly 2x self-only.
        let r_so = compute(&input(2026, CoverageType::SelfOnly, 40, 0, 0, 0));
        let r_f = compute(&input(2026, CoverageType::Family, 40, 0, 0, 0));
        assert_eq!(r_so.hdhp_minimum_deductible_cents, 1_700_00);
        assert_eq!(r_f.hdhp_minimum_deductible_cents, 3_400_00);
    }

    #[test]
    fn family_oop_max_double_self_only() {
        let r_so = compute(&input(2026, CoverageType::SelfOnly, 40, 0, 0, 0));
        let r_f = compute(&input(2026, CoverageType::Family, 40, 0, 0, 0));
        assert_eq!(r_so.hdhp_maximum_out_of_pocket_cents, 8_500_00);
        assert_eq!(r_f.hdhp_maximum_out_of_pocket_cents, 17_000_00);
    }

    #[test]
    fn worked_example_high_income_trader_catch_up() {
        // High-income trader, age 60, family HDHP, contributes max $9,750.
        // Above-the-line deduction = $9,750 → tax savings at 37% = $3,607.50.
        let r = compute(&input(
            2026,
            CoverageType::Family,
            60,
            9_750_00,
            4_500_00,
            16_000_00,
        ));
        assert!(r.hdhp_eligible);
        assert_eq!(r.allowed_deduction_cents, 9_750_00);
        assert_eq!(r.catch_up_amount_cents, 1_000_00);
        assert_eq!(r.excess_contribution_cents, 0);
    }
}
