//! IRC § 219 — Retirement savings (IRA) contribution deduction.
//!
//! § 219(b)(1) sets the maximum IRA contribution at the lesser of the
//! statutory dollar limit (§ 219(b)(5)) or the taxpayer's earned income.
//! Traditional IRA contributions are above-the-line deductible under
//! § 219(a). Roth IRA contributions are NEVER deductible (§ 408A(c)(1))
//! but the contribution itself phases out by MAGI under § 408A(c)(3).
//!
//! 2026 dollar limits (per IRS Notice 2025-XX):
//!
//! Base contribution limit (§ 219(b)(5)(A)): $7,500 (up from $7,000 for
//! 2024 and 2025).
//!
//! Age 50+ catch-up (§ 219(b)(5)(B)): $1,100 for 2026 (indexed under
//! SECURE 2.0 starting 2024; was statutory $1,000 pre-SECURE-2.0).
//!
//! Traditional IRA deduction phaseout (§ 219(g)) — applies when taxpayer
//! or spouse is covered by a workplace retirement plan:
//!
//! Single/HoH covered by workplace plan: $81,000-$91,000 phaseout.
//!
//! Married filing jointly + taxpayer covered: $129,000-$149,000.
//!
//! Married filing jointly + spouse covered (taxpayer not): $242,000-
//! $252,000 (§ 219(g)(7) widened range for non-active-participant
//! spouse).
//!
//! Married filing separately + covered: $0-$10,000.
//!
//! Roth IRA contribution phaseout (§ 408A(c)(3)):
//!
//! Single/HoH: $153,000-$168,000.
//!
//! MFJ: $242,000-$252,000.
//!
//! MFS: $0-$10,000.
//!
//! Excess contribution (§ 4973) triggers 6% excise tax each year the
//! excess remains in the account.
//!
//! Citations: 26 U.S.C. § 219 (general); § 219(a) (above-the-line
//! deduction); § 219(b)(5)(A) ($7,500 base limit 2026); § 219(b)(5)(B)
//! ($1,100 catch-up 2026); § 219(c) (spousal IRA / Kay Bailey Hutchison
//! rule); § 219(g) (Traditional IRA deduction phaseout when covered by
//! workplace plan); § 408A(c)(3) (Roth IRA contribution phaseout); § 4973
//! (6% excise tax on excess contributions).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContributionType {
    TraditionalIra,
    RothIra,
}

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
pub struct Section219Input {
    pub year: u32,
    pub contribution_type: ContributionType,
    pub filing_status: FilingStatus,
    pub age: u32,
    pub modified_agi_cents: i64,
    pub earned_income_cents: i64,
    pub contributions_cents: i64,
    /// Whether the taxpayer is an active participant in a workplace
    /// retirement plan (drives Traditional IRA deduction phaseout).
    pub covered_by_workplace_plan: bool,
    /// MFJ only: whether the spouse is an active participant. Drives
    /// § 219(g)(7) non-active-participant-spouse widened phaseout.
    pub spouse_covered_by_workplace_plan: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section219Result {
    pub base_contribution_limit_cents: i64,
    pub catch_up_amount_cents: i64,
    pub total_contribution_limit_cents: i64,
    pub earned_income_cap_cents: i64,
    pub effective_contribution_limit_cents: i64,
    pub phaseout_low_cents: i64,
    pub phaseout_high_cents: i64,
    pub in_phaseout_range: bool,
    pub phaseout_reduces_to_cents: i64,
    pub allowed_contribution_cents: i64,
    pub allowed_deduction_cents: i64,
    pub excess_contribution_cents: i64,
    pub excise_tax_under_4973_cents: i64,
    pub citation: &'static str,
    pub note: String,
}

pub fn compute(input: &Section219Input) -> Section219Result {
    let earned = input.earned_income_cents.max(0);
    let magi = input.modified_agi_cents.max(0);
    let contributions = input.contributions_cents.max(0);

    let (base_limit, catch_up) = year_limits(input.year, input.age);
    let total_limit = base_limit + catch_up;
    let earned_income_cap = earned;
    let effective_limit = total_limit.min(earned_income_cap);

    let (phaseout_low, phaseout_high) = phaseout_range(input);

    let no_phaseout = phaseout_low == 0 && phaseout_high == 0;
    let (in_phaseout, reduced_limit) = if no_phaseout || magi <= phaseout_low {
        (false, effective_limit)
    } else if magi >= phaseout_high {
        (true, 0)
    } else {
        let range = phaseout_high - phaseout_low;
        let remaining = phaseout_high - magi;
        let reduced = (effective_limit as i128 * remaining as i128 / range as i128) as i64;
        // § 408A(c)(3)/§ 219(g) round to nearest $10 (simplified to $10 = 1000 cents).
        let rounded = ((reduced + 500) / 1000) * 1000;
        // Statutory minimum $200 once phaseout begins (§ 408A(c)(3)(B)).
        (true, rounded.max(20000).min(effective_limit))
    };

    // For Roth, phaseout reduces CONTRIBUTION itself. For Traditional,
    // phaseout reduces DEDUCTION but contribution can still be made
    // (becomes non-deductible Traditional).
    let (allowed_contribution, allowed_deduction) = match input.contribution_type {
        ContributionType::RothIra => (reduced_limit, 0),
        ContributionType::TraditionalIra => (effective_limit, reduced_limit),
    };

    let excess = (contributions - allowed_contribution).max(0);
    let excise_tax = (excess as i128 * 6 / 100) as i64;

    let note = format!(
        "{:?} {} age {}: base limit {} + catch-up {} = total {} cents. Earned income {} cents. Effective contribution limit = min(total, earned) = {} cents. Phaseout range {}-{} cents. MAGI {} cents → {} {}. Allowed contribution {}; allowed deduction {}; excess {} → § 4973 6% excise {}.",
        input.contribution_type,
        input.year,
        input.age,
        base_limit,
        catch_up,
        total_limit,
        earned,
        effective_limit,
        phaseout_low,
        phaseout_high,
        magi,
        if in_phaseout { "IN-PHASEOUT" } else { "below phaseout" },
        if reduced_limit == 0 && in_phaseout { "(fully phased out)" } else { "" },
        allowed_contribution,
        allowed_deduction,
        excess,
        excise_tax,
    );

    Section219Result {
        base_contribution_limit_cents: base_limit,
        catch_up_amount_cents: catch_up,
        total_contribution_limit_cents: total_limit,
        earned_income_cap_cents: earned_income_cap,
        effective_contribution_limit_cents: effective_limit,
        phaseout_low_cents: phaseout_low,
        phaseout_high_cents: phaseout_high,
        in_phaseout_range: in_phaseout,
        phaseout_reduces_to_cents: reduced_limit,
        allowed_contribution_cents: allowed_contribution,
        allowed_deduction_cents: allowed_deduction,
        excess_contribution_cents: excess,
        excise_tax_under_4973_cents: excise_tax,
        citation:
            "26 U.S.C. § 219(b)(5) base limits + catch-up; § 219(g) Traditional phaseout; § 408A(c)(3) Roth phaseout; § 4973 6% excise on excess",
        note,
    }
}

fn year_limits(year: u32, age: u32) -> (i64, i64) {
    let (base, catch_up) = match year {
        // 2026 per IRS 401(k) limit notice + SECURE 2.0 catch-up index.
        2026 => (750000, 110000),
        // 2024 + 2025 base was $7,000 + $1,000 catch-up.
        2024 | 2025 => (700000, 100000),
        // Conservative fallback uses 2026 amounts.
        _ => (750000, 110000),
    };
    (base, if age >= 50 { catch_up } else { 0 })
}

/// Returns (phaseout_low, phaseout_high) in cents for the applicable
/// phaseout range. Zero/zero means no phaseout applies.
fn phaseout_range(input: &Section219Input) -> (i64, i64) {
    match input.contribution_type {
        ContributionType::RothIra => roth_phaseout(input.year, input.filing_status),
        ContributionType::TraditionalIra => {
            if !input.covered_by_workplace_plan && !input.spouse_covered_by_workplace_plan {
                // Neither covered → no phaseout; fully deductible.
                (0, 0)
            } else if input.covered_by_workplace_plan {
                traditional_active_phaseout(input.year, input.filing_status)
            } else {
                // Spouse covered, taxpayer not — § 219(g)(7) widened range.
                traditional_spouse_only_covered_phaseout(input.year, input.filing_status)
            }
        }
    }
}

fn roth_phaseout(year: u32, fs: FilingStatus) -> (i64, i64) {
    match (year, fs) {
        (2026, FilingStatus::Single | FilingStatus::HeadOfHousehold) => (15300000, 16800000),
        (2026, FilingStatus::MarriedFilingJointly | FilingStatus::QualifyingWidow) => {
            (24200000, 25200000)
        }
        (2026, FilingStatus::MarriedFilingSeparately) => (0, 1000000),
        // 2025 amounts.
        (2025, FilingStatus::Single | FilingStatus::HeadOfHousehold) => (15000000, 16500000),
        (2025, FilingStatus::MarriedFilingJointly | FilingStatus::QualifyingWidow) => {
            (23600000, 24600000)
        }
        (2025, FilingStatus::MarriedFilingSeparately) => (0, 1000000),
        // Default to 2026.
        (_, FilingStatus::Single | FilingStatus::HeadOfHousehold) => (15300000, 16800000),
        (_, FilingStatus::MarriedFilingJointly | FilingStatus::QualifyingWidow) => {
            (24200000, 25200000)
        }
        (_, FilingStatus::MarriedFilingSeparately) => (0, 1000000),
    }
}

fn traditional_active_phaseout(year: u32, fs: FilingStatus) -> (i64, i64) {
    match (year, fs) {
        (2026, FilingStatus::Single | FilingStatus::HeadOfHousehold) => (8100000, 9100000),
        (2026, FilingStatus::MarriedFilingJointly | FilingStatus::QualifyingWidow) => {
            (12900000, 14900000)
        }
        (2026, FilingStatus::MarriedFilingSeparately) => (0, 1000000),
        (2025, FilingStatus::Single | FilingStatus::HeadOfHousehold) => (7900000, 8900000),
        (2025, FilingStatus::MarriedFilingJointly | FilingStatus::QualifyingWidow) => {
            (12600000, 14600000)
        }
        (2025, FilingStatus::MarriedFilingSeparately) => (0, 1000000),
        (_, FilingStatus::Single | FilingStatus::HeadOfHousehold) => (8100000, 9100000),
        (_, FilingStatus::MarriedFilingJointly | FilingStatus::QualifyingWidow) => {
            (12900000, 14900000)
        }
        (_, FilingStatus::MarriedFilingSeparately) => (0, 1000000),
    }
}

fn traditional_spouse_only_covered_phaseout(year: u32, fs: FilingStatus) -> (i64, i64) {
    match (year, fs) {
        // § 219(g)(7) widened range for non-active-participant spouse in MFJ.
        (2026, FilingStatus::MarriedFilingJointly | FilingStatus::QualifyingWidow) => {
            (24200000, 25200000)
        }
        (2025, FilingStatus::MarriedFilingJointly | FilingStatus::QualifyingWidow) => {
            (23600000, 24600000)
        }
        // Other filing statuses don't have a spouse-coverage scenario.
        _ => (0, 0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        year: u32,
        ct: ContributionType,
        fs: FilingStatus,
        age: u32,
        magi: i64,
        earned: i64,
        contributions: i64,
        covered: bool,
        spouse_covered: bool,
    ) -> Section219Input {
        Section219Input {
            year,
            contribution_type: ct,
            filing_status: fs,
            age,
            modified_agi_cents: magi,
            earned_income_cents: earned,
            contributions_cents: contributions,
            covered_by_workplace_plan: covered,
            spouse_covered_by_workplace_plan: spouse_covered,
        }
    }

    #[test]
    fn traditional_2026_under_50_no_workplace_plan_full_deduction() {
        let r = compute(&input(
            2026,
            ContributionType::TraditionalIra,
            FilingStatus::Single,
            40,
            50_000_00,
            50_000_00,
            7_500_00,
            false,
            false,
        ));
        assert_eq!(r.base_contribution_limit_cents, 7_500_00);
        assert_eq!(r.catch_up_amount_cents, 0);
        assert_eq!(r.allowed_contribution_cents, 7_500_00);
        assert_eq!(r.allowed_deduction_cents, 7_500_00);
        assert!(!r.in_phaseout_range);
    }

    #[test]
    fn traditional_2026_age_50_catch_up_added() {
        let r = compute(&input(
            2026,
            ContributionType::TraditionalIra,
            FilingStatus::Single,
            50,
            50_000_00,
            50_000_00,
            8_600_00,
            false,
            false,
        ));
        assert_eq!(r.catch_up_amount_cents, 1_100_00);
        assert_eq!(r.total_contribution_limit_cents, 8_600_00);
    }

    #[test]
    fn traditional_2024_age_50_catch_up_was_1000() {
        // 2024 catch-up was $1,000 (pre-SECURE-2.0 inflation index).
        let r = compute(&input(
            2024,
            ContributionType::TraditionalIra,
            FilingStatus::Single,
            50,
            50_000_00,
            50_000_00,
            8_000_00,
            false,
            false,
        ));
        assert_eq!(r.catch_up_amount_cents, 1_000_00);
        assert_eq!(r.total_contribution_limit_cents, 8_000_00);
    }

    #[test]
    fn traditional_single_covered_above_91k_no_deduction() {
        let r = compute(&input(
            2026,
            ContributionType::TraditionalIra,
            FilingStatus::Single,
            40,
            100_000_00,
            100_000_00,
            7_500_00,
            true,
            false,
        ));
        // Above $91K → deduction = 0; but can still make non-deductible
        // Traditional contribution.
        assert_eq!(r.allowed_contribution_cents, 7_500_00);
        assert_eq!(r.allowed_deduction_cents, 0);
        assert!(r.in_phaseout_range);
    }

    #[test]
    fn traditional_single_covered_at_81k_full_deduction() {
        let r = compute(&input(
            2026,
            ContributionType::TraditionalIra,
            FilingStatus::Single,
            40,
            81_000_00,
            81_000_00,
            7_500_00,
            true,
            false,
        ));
        assert!(!r.in_phaseout_range);
        assert_eq!(r.allowed_deduction_cents, 7_500_00);
    }

    #[test]
    fn traditional_single_covered_at_86k_partial_phaseout() {
        // Midpoint of phaseout = 50% reduction.
        let r = compute(&input(
            2026,
            ContributionType::TraditionalIra,
            FilingStatus::Single,
            40,
            86_000_00,
            86_000_00,
            7_500_00,
            true,
            false,
        ));
        assert!(r.in_phaseout_range);
        // Halfway from $81K to $91K → 50% of $7,500 = $3,750, rounded to nearest $10.
        assert!(r.allowed_deduction_cents > 0);
        assert!(r.allowed_deduction_cents < 7_500_00);
    }

    #[test]
    fn traditional_mfj_covered_above_149k_no_deduction() {
        let r = compute(&input(
            2026,
            ContributionType::TraditionalIra,
            FilingStatus::MarriedFilingJointly,
            40,
            200_000_00,
            200_000_00,
            7_500_00,
            true,
            false,
        ));
        assert_eq!(r.allowed_deduction_cents, 0);
        assert_eq!(r.allowed_contribution_cents, 7_500_00); // can still make non-deductible
    }

    #[test]
    fn traditional_mfj_spouse_only_covered_widened_phaseout() {
        // Taxpayer NOT covered, spouse IS covered. § 219(g)(7) widened
        // phaseout $242K-$252K. At $245K → partial phaseout.
        let r = compute(&input(
            2026,
            ContributionType::TraditionalIra,
            FilingStatus::MarriedFilingJointly,
            40,
            245_000_00,
            100_000_00,
            7_500_00,
            false,
            true,
        ));
        assert!(r.in_phaseout_range);
        assert_eq!(r.phaseout_low_cents, 242_000_00);
        assert_eq!(r.phaseout_high_cents, 252_000_00);
    }

    #[test]
    fn traditional_mfj_neither_covered_no_phaseout() {
        let r = compute(&input(
            2026,
            ContributionType::TraditionalIra,
            FilingStatus::MarriedFilingJointly,
            40,
            500_000_00,
            500_000_00,
            7_500_00,
            false,
            false,
        ));
        assert!(!r.in_phaseout_range);
        assert_eq!(r.allowed_deduction_cents, 7_500_00);
    }

    #[test]
    fn roth_single_2026_below_153k_full_contribution() {
        let r = compute(&input(
            2026,
            ContributionType::RothIra,
            FilingStatus::Single,
            40,
            150_000_00,
            150_000_00,
            7_500_00,
            false,
            false,
        ));
        assert!(!r.in_phaseout_range);
        assert_eq!(r.allowed_contribution_cents, 7_500_00);
    }

    #[test]
    fn roth_single_2026_above_168k_no_contribution() {
        let r = compute(&input(
            2026,
            ContributionType::RothIra,
            FilingStatus::Single,
            40,
            200_000_00,
            200_000_00,
            7_500_00,
            false,
            false,
        ));
        assert!(r.in_phaseout_range);
        assert_eq!(r.allowed_contribution_cents, 0);
        // Excess = full contribution amount.
        assert_eq!(r.excess_contribution_cents, 7_500_00);
        assert_eq!(r.excise_tax_under_4973_cents, 450_00);
    }

    #[test]
    fn roth_mfj_2026_above_252k_no_contribution() {
        let r = compute(&input(
            2026,
            ContributionType::RothIra,
            FilingStatus::MarriedFilingJointly,
            40,
            300_000_00,
            300_000_00,
            7_500_00,
            false,
            false,
        ));
        assert_eq!(r.allowed_contribution_cents, 0);
    }

    #[test]
    fn roth_mfj_2026_in_phaseout_partial() {
        // Midpoint $247K → about 50% allowed = $3,750.
        let r = compute(&input(
            2026,
            ContributionType::RothIra,
            FilingStatus::MarriedFilingJointly,
            40,
            247_000_00,
            247_000_00,
            7_500_00,
            false,
            false,
        ));
        assert!(r.in_phaseout_range);
        assert!(r.allowed_contribution_cents > 0);
        assert!(r.allowed_contribution_cents < 7_500_00);
    }

    #[test]
    fn roth_mfs_2026_zero_to_10k_phaseout() {
        let r = compute(&input(
            2026,
            ContributionType::RothIra,
            FilingStatus::MarriedFilingSeparately,
            40,
            5_000_00,
            5_000_00,
            7_500_00,
            false,
            false,
        ));
        // MFS phaseout is $0-$10K. At $5K → 50% of contribution.
        assert!(r.in_phaseout_range);
    }

    #[test]
    fn roth_mfs_2026_above_10k_no_contribution() {
        let r = compute(&input(
            2026,
            ContributionType::RothIra,
            FilingStatus::MarriedFilingSeparately,
            40,
            15_000_00,
            15_000_00,
            7_500_00,
            false,
            false,
        ));
        assert_eq!(r.allowed_contribution_cents, 0);
    }

    #[test]
    fn earned_income_caps_contribution() {
        // Earned income $3K — caps contribution below the $7,500 statutory limit.
        let r = compute(&input(
            2026,
            ContributionType::TraditionalIra,
            FilingStatus::Single,
            40,
            50_000_00,
            3_000_00,
            7_500_00,
            false,
            false,
        ));
        assert_eq!(r.effective_contribution_limit_cents, 3_000_00);
        assert_eq!(r.allowed_contribution_cents, 3_000_00);
        // Excess = $7,500 - $3,000 = $4,500.
        assert_eq!(r.excess_contribution_cents, 4_500_00);
    }

    #[test]
    fn excess_contribution_triggers_6_percent_excise() {
        // Contribute $9,000 in 2026 → limit $7,500 → $1,500 excess → $90 excise.
        let r = compute(&input(
            2026,
            ContributionType::TraditionalIra,
            FilingStatus::Single,
            40,
            50_000_00,
            50_000_00,
            9_000_00,
            false,
            false,
        ));
        assert_eq!(r.excess_contribution_cents, 1_500_00);
        assert_eq!(r.excise_tax_under_4973_cents, 90_00);
    }

    #[test]
    fn age_49_no_catch_up() {
        let r = compute(&input(
            2026,
            ContributionType::TraditionalIra,
            FilingStatus::Single,
            49,
            50_000_00,
            50_000_00,
            7_500_00,
            false,
            false,
        ));
        assert_eq!(r.catch_up_amount_cents, 0);
    }

    #[test]
    fn age_50_catch_up_boundary() {
        let r = compute(&input(
            2026,
            ContributionType::TraditionalIra,
            FilingStatus::Single,
            50,
            50_000_00,
            50_000_00,
            8_600_00,
            false,
            false,
        ));
        assert_eq!(r.catch_up_amount_cents, 1_100_00);
    }

    #[test]
    fn citations_pin_correct_authorities() {
        let r = compute(&input(
            2026,
            ContributionType::TraditionalIra,
            FilingStatus::Single,
            40,
            50_000_00,
            50_000_00,
            7_500_00,
            false,
            false,
        ));
        assert!(r.citation.contains("§ 219(b)(5)"));
        assert!(r.citation.contains("§ 219(g)"));
        assert!(r.citation.contains("§ 408A(c)(3)"));
        assert!(r.citation.contains("§ 4973"));
    }

    #[test]
    fn negative_inputs_clamped() {
        let r = compute(&input(
            2026,
            ContributionType::TraditionalIra,
            FilingStatus::Single,
            40,
            -100,
            -100,
            -100,
            false,
            false,
        ));
        // Earned income clamped to 0 → no contribution allowed.
        assert_eq!(r.allowed_contribution_cents, 0);
    }

    #[test]
    fn roth_at_high_end_153k_below_phaseout() {
        // $153K is the LOW end — at or below low end means full contribution.
        let r = compute(&input(
            2026,
            ContributionType::RothIra,
            FilingStatus::Single,
            40,
            153_000_00,
            153_000_00,
            7_500_00,
            false,
            false,
        ));
        assert!(!r.in_phaseout_range);
        assert_eq!(r.allowed_contribution_cents, 7_500_00);
    }

    #[test]
    fn roth_at_168k_high_end_fully_phased_out() {
        // $168K is the HIGH end — fully phased out.
        let r = compute(&input(
            2026,
            ContributionType::RothIra,
            FilingStatus::Single,
            40,
            168_000_00,
            168_000_00,
            7_500_00,
            false,
            false,
        ));
        assert!(r.in_phaseout_range);
        assert_eq!(r.allowed_contribution_cents, 0);
    }

    #[test]
    fn traditional_above_phaseout_can_make_nondeductible() {
        // Above phaseout: deduction is zero but contribution still allowed
        // (non-deductible Traditional → backdoor Roth conversion candidate).
        let r = compute(&input(
            2026,
            ContributionType::TraditionalIra,
            FilingStatus::Single,
            40,
            500_000_00,
            500_000_00,
            7_500_00,
            true,
            false,
        ));
        assert_eq!(r.allowed_contribution_cents, 7_500_00);
        assert_eq!(r.allowed_deduction_cents, 0);
    }

    #[test]
    fn roth_distinct_from_traditional_phaseout_behavior() {
        // Same MAGI ($200K), Single, covered. Traditional → no deduction
        // but allowed contribution. Roth → no contribution at all (above $168K).
        let trad = compute(&input(
            2026,
            ContributionType::TraditionalIra,
            FilingStatus::Single,
            40,
            200_000_00,
            200_000_00,
            7_500_00,
            true,
            false,
        ));
        let roth = compute(&input(
            2026,
            ContributionType::RothIra,
            FilingStatus::Single,
            40,
            200_000_00,
            200_000_00,
            7_500_00,
            false,
            false,
        ));
        assert_eq!(trad.allowed_contribution_cents, 7_500_00);
        assert_eq!(trad.allowed_deduction_cents, 0);
        assert_eq!(roth.allowed_contribution_cents, 0);
    }

    #[test]
    fn year_aware_2025_7000_limit() {
        let r = compute(&input(
            2025,
            ContributionType::TraditionalIra,
            FilingStatus::Single,
            40,
            50_000_00,
            50_000_00,
            7_000_00,
            false,
            false,
        ));
        assert_eq!(r.base_contribution_limit_cents, 7_000_00);
    }
}
