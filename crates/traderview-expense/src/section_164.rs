//! IRC § 164 — State and Local Tax (SALT) deduction cap.
//!
//! TCJA capped the SALT itemized deduction at **$10,000 ($5,000 MFS)**
//! for tax years 2018-2025. The One Big Beautiful Bill Act (OBBBA, eff.
//! 2025-01-01) **temporarily expanded the cap** through 2029, with an
//! automatic sunset back to the TCJA $10,000 cap in 2030.
//!
//! **Expanded cap mechanics (§ 164(b)(6) as amended by OBBBA § 70413)**:
//! - 2025 base cap: **$40,000** joint / single / HoH ($20,000 MFS).
//! - 2026-2029: cap rises by **1% per year** (compounded).
//! - **High-income phaseout** begins at **$500,000 MAGI** joint / single /
//!   HoH ($250,000 MFS) in 2025; threshold also rises 1% annually through
//!   2029. Reduction = **30%** of the excess over the threshold.
//! - **Statutory floor**: every taxpayer is guaranteed at least
//!   **$10,000 ($5,000 MFS)** even if the phaseout would otherwise
//!   reduce the cap below that amount.
//! - **2030 sunset**: cap snaps back to TCJA $10,000 ($5,000 MFS) with no
//!   phaseout. No further extensions written into OBBBA.
//!
//! Citations: 26 U.S.C. § 164; § 164(b)(6) (SALT cap, as amended); OBBBA
//! § 70413 (eff. 2025-01-01, expanded cap + phaseout); IRC § 164(b)(6)(B)
//! (2030 reversion).
//!
//! Out of scope: pass-through-entity (PTET) workaround (state-level
//! elections that re-characterize state income tax as a deductible
//! business expense at the entity level). PTET planning is a state-by-
//! state field handled by the user; this module just computes the federal
//! itemized cap.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilingStatus {
    Single,
    MarriedFilingJointly,
    MarriedFilingSeparately,
    HeadOfHousehold,
    QualifyingWidow,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section164Input {
    pub year: u32,
    pub filing_status: FilingStatus,
    /// Total state + local tax paid (income + property + sales) — the
    /// uncapped pre-§ 164(b)(6) amount.
    pub salt_paid_cents: i64,
    /// Modified AGI for the §164(b)(6)(C) phaseout test.
    pub modified_agi_cents: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section164Result {
    /// Base cap (pre-phaseout) for the year + filing status.
    pub base_cap_cents: i64,
    /// MAGI threshold for the 30% phaseout to begin.
    pub phaseout_threshold_cents: i64,
    /// Amount the cap is reduced by under §164(b)(6)(C) phaseout.
    pub phaseout_reduction_cents: i64,
    /// Statutory floor — guaranteed minimum deduction regardless of
    /// phaseout. $10K ($5K MFS).
    pub statutory_floor_cents: i64,
    /// Reduced cap = max(floor, base_cap − phaseout_reduction).
    pub reduced_cap_cents: i64,
    /// Final allowed deduction = min(salt_paid, reduced_cap).
    pub allowed_deduction_cents: i64,
    /// Amount of SALT paid that is BLOCKED by the cap (lost to federal).
    pub blocked_by_cap_cents: i64,
    pub citation: &'static str,
    pub note: String,
}

pub fn compute(input: &Section164Input) -> Section164Result {
    let salt_paid = input.salt_paid_cents.max(0);
    let magi = input.modified_agi_cents.max(0);
    let is_mfs = matches!(input.filing_status, FilingStatus::MarriedFilingSeparately);

    // Pre-OBBBA TCJA cap (2018-2024).
    if input.year < 2025 {
        let cap = if is_mfs { 500000 } else { 1000000 };
        let allowed = salt_paid.min(cap);
        return Section164Result {
            base_cap_cents: cap,
            phaseout_threshold_cents: 0,
            phaseout_reduction_cents: 0,
            statutory_floor_cents: cap,
            reduced_cap_cents: cap,
            allowed_deduction_cents: allowed,
            blocked_by_cap_cents: salt_paid - allowed,
            citation:
                "26 U.S.C. § 164(b)(6) (TCJA) — $10,000 / $5,000 MFS SALT cap (tax years 2018-2024)",
            note: format!(
                "TCJA SALT cap = {} cents for {:?}. No phaseout. Allowed deduction = {} cents.",
                cap, input.filing_status, allowed
            ),
        };
    }

    // 2030 sunset — TCJA cap restored, no phaseout.
    if input.year >= 2030 {
        let cap = if is_mfs { 500000 } else { 1000000 };
        let allowed = salt_paid.min(cap);
        return Section164Result {
            base_cap_cents: cap,
            phaseout_threshold_cents: 0,
            phaseout_reduction_cents: 0,
            statutory_floor_cents: cap,
            reduced_cap_cents: cap,
            allowed_deduction_cents: allowed,
            blocked_by_cap_cents: salt_paid - allowed,
            citation: "26 U.S.C. § 164(b)(6)(B) — OBBBA SALT-cap expansion sunset; reverts to TCJA $10,000 / $5,000 MFS cap for 2030+",
            note: format!(
                "Post-OBBBA SALT-cap sunset: 2030+ reverts to TCJA $10,000 cap ({} cents for {:?}). Allowed = {} cents.",
                cap, input.filing_status, allowed
            ),
        };
    }

    // OBBBA expanded cap window 2025-2029.
    let (base_cap, threshold, floor) = obbba_year_amounts(input.year, is_mfs);
    let excess = (magi - threshold).max(0);
    // §164(b)(6)(C) phaseout: 30% × excess MAGI.
    let phaseout_reduction = (excess as i128 * 30 / 100) as i64;
    let reduced_cap = (base_cap - phaseout_reduction).max(floor);
    let allowed = salt_paid.min(reduced_cap);

    let note = format!(
        "OBBBA expanded SALT cap for {} ({:?}): base cap = {} cents, MAGI = {}, phaseout threshold = {} cents, 30% × excess = {} cents, reduced cap = max({} floor, {} − {}) = {} cents. Allowed deduction = min(salt_paid {}, cap {}) = {} cents.",
        input.year,
        input.filing_status,
        base_cap,
        magi,
        threshold,
        phaseout_reduction,
        floor,
        base_cap,
        phaseout_reduction,
        reduced_cap,
        salt_paid,
        reduced_cap,
        allowed,
    );

    Section164Result {
        base_cap_cents: base_cap,
        phaseout_threshold_cents: threshold,
        phaseout_reduction_cents: phaseout_reduction,
        statutory_floor_cents: floor,
        reduced_cap_cents: reduced_cap,
        allowed_deduction_cents: allowed,
        blocked_by_cap_cents: salt_paid - allowed,
        citation: "26 U.S.C. § 164(b)(6) as amended by OBBBA § 70413 — expanded SALT cap with 30% high-income phaseout above $500,000 ($250,000 MFS), $10,000 ($5,000 MFS) statutory floor, 1% annual cap+threshold increase through 2029, 2030 sunset to TCJA",
        note,
    }
}

/// Returns (base_cap, phaseout_threshold, statutory_floor) for OBBBA years
/// 2025-2029. Compounding the 1% annual increase: year_value = base × 1.01^(year-2025).
fn obbba_year_amounts(year: u32, is_mfs: bool) -> (i64, i64, i64) {
    let years_after_2025 = (year - 2025) as i32;
    let multiplier = compound_one_percent(years_after_2025);
    let (base_cap_2025, threshold_2025, floor) = if is_mfs {
        (2000000, 25000000, 500000)
    } else {
        (4000000, 50000000, 1000000)
    };
    let cap = ((base_cap_2025 as i128 * multiplier) / 10_000) as i64;
    let threshold = ((threshold_2025 as i128 * multiplier) / 10_000) as i64;
    (cap, threshold, floor)
}

/// Compounds a 1% annual rate over `years` years, returning the multiplier
/// in basis-points × 100 (so 10000 = 1.0000). Uses fixed-point i128 math
/// to avoid float drift across years.
fn compound_one_percent(years: i32) -> i128 {
    // 1.01^years, scaled by 10_000 for 4-decimal precision.
    let mut multiplier: i128 = 10_000;
    for _ in 0..years {
        multiplier = multiplier * 101 / 100;
    }
    multiplier
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(year: u32, status: FilingStatus, salt_paid: i64, magi: i64) -> Section164Input {
        Section164Input {
            year,
            filing_status: status,
            salt_paid_cents: salt_paid,
            modified_agi_cents: magi,
        }
    }

    #[test]
    fn tcja_2024_cap_10k() {
        let r = compute(&input(2024, FilingStatus::Single, 5000000, 10000000));
        assert_eq!(r.allowed_deduction_cents, 1000000);
        assert_eq!(r.blocked_by_cap_cents, 4000000);
        assert!(r.citation.contains("TCJA"));
    }

    #[test]
    fn tcja_2024_mfs_cap_5k() {
        let r = compute(&input(
            2024,
            FilingStatus::MarriedFilingSeparately,
            3000000,
            10000000,
        ));
        assert_eq!(r.allowed_deduction_cents, 500000);
    }

    #[test]
    fn obbba_2025_single_under_threshold_full_cap() {
        let r = compute(&input(2025, FilingStatus::Single, 5000000, 40000000));
        assert_eq!(r.base_cap_cents, 4000000);
        assert_eq!(r.phaseout_threshold_cents, 50000000);
        assert_eq!(r.phaseout_reduction_cents, 0);
        assert_eq!(r.reduced_cap_cents, 4000000);
        assert_eq!(r.allowed_deduction_cents, 4000000);
    }

    #[test]
    fn obbba_2025_at_500k_boundary_no_reduction() {
        let r = compute(&input(2025, FilingStatus::Single, 5000000, 50000000));
        assert_eq!(r.phaseout_reduction_cents, 0);
        assert_eq!(r.reduced_cap_cents, 4000000);
    }

    #[test]
    fn obbba_2025_one_dollar_above_threshold_30_cents_reduction() {
        let r = compute(&input(2025, FilingStatus::Single, 5000000, 50000000 + 100));
        assert_eq!(r.phaseout_reduction_cents, 30);
        assert_eq!(r.reduced_cap_cents, 4000000 - 30);
    }

    #[test]
    fn obbba_2025_high_magi_phaseout_to_floor() {
        // MAGI $1M, threshold $500K, excess $500K × 30% = $150K reduction.
        // Base cap $40K − $150K = -$110K → clamped at $10K floor.
        let r = compute(&input(2025, FilingStatus::Single, 5000000, 1_00000000));
        assert_eq!(r.phaseout_reduction_cents, 15000000);
        assert_eq!(r.reduced_cap_cents, 1000000);
        assert_eq!(r.allowed_deduction_cents, 1000000);
    }

    #[test]
    fn obbba_2025_statutory_floor_protects_high_income() {
        // Even at MAGI $10M, the $10K floor binds.
        let r = compute(&input(2025, FilingStatus::Single, 10000000, 10_00000000));
        assert_eq!(r.allowed_deduction_cents, 1000000);
        assert_eq!(r.statutory_floor_cents, 1000000);
    }

    #[test]
    fn obbba_2025_mfs_half_amounts() {
        let r = compute(&input(
            2025,
            FilingStatus::MarriedFilingSeparately,
            2500000,
            24000000,
        ));
        assert_eq!(r.base_cap_cents, 2000000);
        assert_eq!(r.phaseout_threshold_cents, 25000000);
        assert_eq!(r.statutory_floor_cents, 500000);
        assert_eq!(r.allowed_deduction_cents, 2000000);
    }

    #[test]
    fn obbba_2026_cap_rises_one_percent() {
        let r = compute(&input(2026, FilingStatus::Single, 5000000, 10000000));
        // 40,000 * 1.01 = 40,400.
        assert_eq!(r.base_cap_cents, 4040000);
        assert_eq!(r.phaseout_threshold_cents, 50500000);
    }

    #[test]
    fn obbba_2027_compounds() {
        let r = compute(&input(2027, FilingStatus::Single, 5000000, 10000000));
        // 40,000 * 1.01^2 = 40,804.
        assert_eq!(r.base_cap_cents, 4080400);
    }

    #[test]
    fn obbba_2029_final_year() {
        let r = compute(&input(2029, FilingStatus::Single, 5000000, 10000000));
        // 40,000 * 1.01^4 = 41,624.1604 → integer cents 41,624_16 = 4_162_416 cents
        let expected_cap = (40_000_00_i128 * compound_one_percent(4) / 10_000) as i64;
        assert_eq!(r.base_cap_cents, expected_cap);
    }

    #[test]
    fn obbba_2030_sunsets_to_tcja_10k() {
        let r = compute(&input(2030, FilingStatus::Single, 5000000, 40000000));
        assert_eq!(r.base_cap_cents, 1000000);
        assert_eq!(r.allowed_deduction_cents, 1000000);
        assert_eq!(r.phaseout_reduction_cents, 0);
        assert!(r.citation.contains("sunset"));
        assert!(r.citation.contains("§ 164(b)(6)(B)"));
    }

    #[test]
    fn obbba_2030_mfs_sunsets_to_5k() {
        let r = compute(&input(
            2030,
            FilingStatus::MarriedFilingSeparately,
            5000000,
            40000000,
        ));
        assert_eq!(r.allowed_deduction_cents, 500000);
    }

    #[test]
    fn obbba_2031_still_tcja() {
        let r = compute(&input(2031, FilingStatus::Single, 5000000, 40000000));
        assert_eq!(r.allowed_deduction_cents, 1000000);
    }

    #[test]
    fn salt_paid_under_cap_no_blocking() {
        let r = compute(&input(2025, FilingStatus::Single, 500000, 10000000));
        assert_eq!(r.allowed_deduction_cents, 500000);
        assert_eq!(r.blocked_by_cap_cents, 0);
    }

    #[test]
    fn salt_paid_exactly_cap_no_blocking() {
        let r = compute(&input(2025, FilingStatus::Single, 4000000, 10000000));
        assert_eq!(r.allowed_deduction_cents, 4000000);
        assert_eq!(r.blocked_by_cap_cents, 0);
    }

    #[test]
    fn phaseout_partial_reduction_above_floor() {
        // MAGI $550K → excess $50K × 30% = $15K reduction. Base $40K − $15K
        // = $25K. SALT paid $30K → allowed $25K.
        let r = compute(&input(2025, FilingStatus::Single, 3000000, 55000000));
        assert_eq!(r.phaseout_reduction_cents, 1500000);
        assert_eq!(r.reduced_cap_cents, 2500000);
        assert_eq!(r.allowed_deduction_cents, 2500000);
    }

    #[test]
    fn mfs_phaseout_at_250k_threshold() {
        // MAGI $300K MFS → excess $50K × 30% = $15K reduction. Base $20K
        // − $15K = $5K (= floor). Allowed $5K.
        let r = compute(&input(
            2025,
            FilingStatus::MarriedFilingSeparately,
            2000000,
            30000000,
        ));
        assert_eq!(r.phaseout_reduction_cents, 1500000);
        assert_eq!(r.reduced_cap_cents, 500000);
    }

    #[test]
    fn mfj_uses_500k_threshold_not_250k() {
        let r = compute(&input(
            2025,
            FilingStatus::MarriedFilingJointly,
            4000000,
            45000000,
        ));
        assert_eq!(r.phaseout_threshold_cents, 50000000);
        assert_eq!(r.phaseout_reduction_cents, 0);
    }

    #[test]
    fn citations_pin_correct_authorities() {
        let r_2024 = compute(&input(2024, FilingStatus::Single, 5000000, 40000000));
        assert!(r_2024.citation.contains("TCJA"));

        let r_2025 = compute(&input(2025, FilingStatus::Single, 5000000, 40000000));
        assert!(r_2025.citation.contains("§ 70413"));
        assert!(r_2025.citation.contains("OBBBA"));
        assert!(r_2025.citation.contains("$500,000"));

        let r_2030 = compute(&input(2030, FilingStatus::Single, 5000000, 40000000));
        assert!(r_2030.citation.contains("sunset"));
    }

    #[test]
    fn blocked_by_cap_equals_salt_paid_minus_allowed() {
        for year in [2024, 2025, 2026, 2030] {
            let r = compute(&input(year, FilingStatus::Single, 5000000, 10000000));
            assert_eq!(r.blocked_by_cap_cents, 5000000 - r.allowed_deduction_cents);
        }
    }

    #[test]
    fn negative_inputs_clamped() {
        let r = compute(&input(2025, FilingStatus::Single, -100, -200));
        assert_eq!(r.allowed_deduction_cents, 0);
    }

    #[test]
    fn compound_growth_strictly_monotonic_across_obbba_years() {
        let caps: Vec<i64> = (2025..=2029)
            .map(|y| compute(&input(y, FilingStatus::Single, 5000000, 10000000)).base_cap_cents)
            .collect();
        for w in caps.windows(2) {
            assert!(w[1] > w[0], "year-over-year cap should grow: {:?}", caps);
        }
    }
}
