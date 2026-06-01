//! IRC § 168(k) — Additional First-Year Depreciation Deduction (bonus
//! depreciation).
//!
//! § 168(k) lets a taxpayer deduct an "additional first-year depreciation
//! deduction" equal to a specified percentage of the adjusted basis of
//! QUALIFIED PROPERTY in the year placed in service. Distinct from
//! `section_179` first-year EXPENSING (which has dollar caps and phaseout)
//! — § 168(k) bonus depreciation has NO dollar cap, no income limitation,
//! and no phaseout. The two work together: §179 first, then §168(k) on
//! the remainder of basis, then MACRS depreciation on what remains.
//!
//! Rate schedule (pre-OBBBA TCJA phasedown):
//!
//! Pre-2018 (pre-TCJA): 50% for most years.
//!
//! 2018 (post-TCJA effective 2017-09-27): 100%.
//!
//! Through 2022: 100%.
//!
//! 2023: 80%. 2024: 60%. 2025 (pre-OBBBA): 40%. 2026 (pre-OBBBA): 20%.
//! 2027+ (pre-OBBBA): 0%.
//!
//! OBBBA restoration to 100% permanent (eff. 2025-01-20):
//!
//! OBBBA permanently restores 100% bonus depreciation for qualified
//! property acquired AND placed in service AFTER 2025-01-19. The TCJA
//! phasedown is eliminated for property meeting both acquisition and
//! placed-in-service dates after the OBBBA effective date.
//!
//! Transition election (§ 168(k) under OBBBA):
//!
//! For the FIRST taxable year ending after 2025-01-19, the taxpayer
//! may ELECT to deduct 40% (or 60% for long-production-period property
//! or certain aircraft) instead of 100%. Election made by the due date
//! including extensions of the federal return for the year including
//! 2025-01-20.
//!
//! Used property: TCJA (2017) made § 168(k) apply to used property when
//! acquired by a taxpayer who has not had a prior use of the property.
//! OBBBA preserves this.
//!
//! Qualified property: § 1245-type tangible property with MACRS recovery
//! period of 20 years or less; computer software (off-the-shelf);
//! qualified film/TV/live theatrical productions; specified plants when
//! planted or grafted; water utility property; qualified improvement
//! property (post-CARES Act 2020 fix making QIP 15-year property
//! eligible for bonus).
//!
//! Citations: 26 U.S.C. § 168(k); § 168(k)(1) (additional first-year
//! deduction); § 168(k)(2) (qualified property definition); § 168(k)(6)
//! (rate schedule); § 168(k)(7) (election out); OBBBA § 70302 (permanent
//! 100% restoration eff. 2025-01-20); IRS Notice 2026-11 (interim guidance
//! on OBBBA bonus depreciation rules).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Section168KInput {
    pub acquisition_year: u32,
    pub acquisition_month: u32,
    pub acquisition_day: u32,
    pub placed_in_service_year: u32,
    pub placed_in_service_month: u32,
    pub placed_in_service_day: u32,
    pub property_cost_cents: i64,
    /// MACRS recovery period in years. Bonus depreciation requires ≤ 20.
    pub macrs_recovery_period_years: u32,
    /// Long-production-period property or certain aircraft. Drives the
    /// transition-election 60% rate (vs 40% standard).
    pub has_long_production_period_or_aircraft: bool,
    /// Whether the taxpayer is electing the OBBBA transition rate (40%
    /// or 60%) instead of 100% for the FYE-after-2025-01-19 election year.
    pub transition_election_made: bool,
    /// Whether the property has been previously used by the taxpayer (or
    /// related parties). Disqualifies under § 168(k)(2)(E)(ii).
    pub prior_use_by_taxpayer: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section168KResult {
    pub qualified_property: bool,
    pub obbba_permanent_100_applies: bool,
    pub transition_election_year: bool,
    pub bonus_depreciation_rate_basis_points: u32,
    pub bonus_depreciation_amount_cents: i64,
    pub citation: &'static str,
    pub note: String,
}

pub fn compute(input: &Section168KInput) -> Section168KResult {
    let cost = input.property_cost_cents.max(0);

    // Qualified property: MACRS recovery ≤ 20 years AND no prior use.
    let qualified = input.macrs_recovery_period_years <= 20 && !input.prior_use_by_taxpayer;
    if !qualified {
        return Section168KResult {
            qualified_property: false,
            obbba_permanent_100_applies: false,
            transition_election_year: false,
            bonus_depreciation_rate_basis_points: 0,
            bonus_depreciation_amount_cents: 0,
            citation: "26 U.S.C. § 168(k)(2) — qualified property requires MACRS recovery period ≤ 20 years AND no prior use by taxpayer",
            note: format!(
                "Property not qualified: MACRS recovery {} years (need ≤ 20) or prior use by taxpayer = {}.",
                input.macrs_recovery_period_years, input.prior_use_by_taxpayer
            ),
        };
    }

    let placed_after_obbba = is_after_2025_01_19(
        input.placed_in_service_year,
        input.placed_in_service_month,
        input.placed_in_service_day,
    );
    let acquired_after_obbba = is_after_2025_01_19(
        input.acquisition_year,
        input.acquisition_month,
        input.acquisition_day,
    );

    let obbba_permanent = placed_after_obbba && acquired_after_obbba;
    let transition_election_year = placed_after_obbba
        && acquired_after_obbba
        && fye_contains_january_2025(input.placed_in_service_year);

    let rate_bps: u32 = if obbba_permanent {
        if transition_election_year && input.transition_election_made {
            if input.has_long_production_period_or_aircraft {
                6000
            } else {
                4000
            }
        } else {
            10000
        }
    } else {
        // Pre-OBBBA schedule. Use placed-in-service year.
        match input.placed_in_service_year {
            y if y < 2018 => 5000,  // Pre-TCJA 50%
            y if (2018..=2022).contains(&y) => 10000, // TCJA 100%
            2023 => 8000,
            2024 => 6000,
            2025 => 4000, // Pre-OBBBA phasedown rate for property placed before 2025-01-20
            2026 => 2000,
            _ => 0, // 2027+ pre-OBBBA = 0
        }
    };

    let bonus_amount = (cost as i128 * rate_bps as i128 / 10_000) as i64;

    let citation = if obbba_permanent {
        if transition_election_year && input.transition_election_made {
            "OBBBA § 70302 + § 168(k)(7) — transition election permits 40% (60% long-production/aircraft) for FYE after 2025-01-19"
        } else {
            "26 U.S.C. § 168(k) + OBBBA § 70302 — PERMANENT 100% bonus depreciation for qualified property acquired AND placed in service after 2025-01-19"
        }
    } else {
        "26 U.S.C. § 168(k)(6) — TCJA phasedown rate schedule (pre-OBBBA): 100% (2018-2022), 80% (2023), 60% (2024), 40% (2025), 20% (2026), 0% (2027+)"
    };

    let note = format!(
        "Property cost = {} cents. MACRS recovery period {} years (≤ 20 OK). Placed in service {}-{:02}-{:02}. Acquisition {}-{:02}-{:02}. {}. Bonus rate = {} bps ({}%). Bonus depreciation = {} cents.",
        cost,
        input.macrs_recovery_period_years,
        input.placed_in_service_year,
        input.placed_in_service_month,
        input.placed_in_service_day,
        input.acquisition_year,
        input.acquisition_month,
        input.acquisition_day,
        if obbba_permanent {
            if transition_election_year && input.transition_election_made {
                "OBBBA transition election made"
            } else {
                "OBBBA permanent 100% applies"
            }
        } else {
            "Pre-OBBBA TCJA phasedown rate applies"
        },
        rate_bps,
        rate_bps / 100,
        bonus_amount,
    );

    Section168KResult {
        qualified_property: true,
        obbba_permanent_100_applies: obbba_permanent
            && !(transition_election_year && input.transition_election_made),
        transition_election_year,
        bonus_depreciation_rate_basis_points: rate_bps,
        bonus_depreciation_amount_cents: bonus_amount,
        citation,
        note,
    }
}

fn is_after_2025_01_19(year: u32, month: u32, day: u32) -> bool {
    match year.cmp(&2025) {
        std::cmp::Ordering::Greater => true,
        std::cmp::Ordering::Less => false,
        std::cmp::Ordering::Equal => match month.cmp(&1) {
            std::cmp::Ordering::Greater => true,
            std::cmp::Ordering::Less => false,
            std::cmp::Ordering::Equal => day > 19,
        },
    }
}

/// Whether the calendar year's fiscal year ending could contain
/// 2025-01-19 — simplified as placed-in-service-year = 2025 for our
/// purposes.
fn fye_contains_january_2025(year: u32) -> bool {
    year == 2025
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        acq_year: u32,
        acq_month: u32,
        acq_day: u32,
        pis_year: u32,
        pis_month: u32,
        pis_day: u32,
        cost: i64,
        macrs_years: u32,
        long_production: bool,
        transition_election: bool,
        prior_use: bool,
    ) -> Section168KInput {
        Section168KInput {
            acquisition_year: acq_year,
            acquisition_month: acq_month,
            acquisition_day: acq_day,
            placed_in_service_year: pis_year,
            placed_in_service_month: pis_month,
            placed_in_service_day: pis_day,
            property_cost_cents: cost,
            macrs_recovery_period_years: macrs_years,
            has_long_production_period_or_aircraft: long_production,
            transition_election_made: transition_election,
            prior_use_by_taxpayer: prior_use,
        }
    }

    #[test]
    fn obbba_2026_100_percent_permanent() {
        // Acquired + placed in service 2026 → OBBBA permanent 100%.
        let r = compute(&input(
            2026, 6, 1, 2026, 6, 1, 100_000_00, 5, false, false, false,
        ));
        assert!(r.qualified_property);
        assert!(r.obbba_permanent_100_applies);
        assert_eq!(r.bonus_depreciation_rate_basis_points, 10000);
        assert_eq!(r.bonus_depreciation_amount_cents, 100_000_00);
    }

    #[test]
    fn pre_obbba_2024_60_percent() {
        let r = compute(&input(
            2024, 6, 1, 2024, 6, 1, 100_000_00, 5, false, false, false,
        ));
        assert_eq!(r.bonus_depreciation_rate_basis_points, 6000);
        assert_eq!(r.bonus_depreciation_amount_cents, 60_000_00);
        assert!(!r.obbba_permanent_100_applies);
    }

    #[test]
    fn pre_obbba_2023_80_percent() {
        let r = compute(&input(
            2023, 6, 1, 2023, 6, 1, 100_000_00, 5, false, false, false,
        ));
        assert_eq!(r.bonus_depreciation_rate_basis_points, 8000);
    }

    #[test]
    fn pre_obbba_2025_phasedown_40_percent() {
        // Placed in service before 2025-01-20 → pre-OBBBA 40% rate.
        let r = compute(&input(
            2024, 12, 1, 2025, 1, 15, 100_000_00, 5, false, false, false,
        ));
        assert_eq!(r.bonus_depreciation_rate_basis_points, 4000);
        assert!(!r.obbba_permanent_100_applies);
    }

    #[test]
    fn at_2025_01_19_boundary_pre_obbba() {
        let r = compute(&input(
            2024, 12, 1, 2025, 1, 19, 100_000_00, 5, false, false, false,
        ));
        assert_eq!(r.bonus_depreciation_rate_basis_points, 4000);
        assert!(!r.obbba_permanent_100_applies);
    }

    #[test]
    fn at_2025_01_20_obbba_permanent_100_applies() {
        // Acquired and placed in service exactly 2025-01-20.
        let r = compute(&input(
            2025, 1, 20, 2025, 1, 20, 100_000_00, 5, false, false, false,
        ));
        assert!(r.obbba_permanent_100_applies);
        assert_eq!(r.bonus_depreciation_rate_basis_points, 10000);
    }

    #[test]
    fn obbba_transition_election_40_percent_2025() {
        // 2025 transition election — taxpayer elects 40% over 100%.
        let r = compute(&input(
            2025, 6, 1, 2025, 6, 1, 100_000_00, 5, false, true, false,
        ));
        assert!(r.transition_election_year);
        assert_eq!(r.bonus_depreciation_rate_basis_points, 4000);
        assert_eq!(r.bonus_depreciation_amount_cents, 40_000_00);
    }

    #[test]
    fn obbba_transition_election_60_percent_long_production() {
        let r = compute(&input(
            2025, 6, 1, 2025, 6, 1, 100_000_00, 5, true, true, false,
        ));
        assert_eq!(r.bonus_depreciation_rate_basis_points, 6000);
        assert_eq!(r.bonus_depreciation_amount_cents, 60_000_00);
    }

    #[test]
    fn obbba_no_transition_election_2025_full_100() {
        // 2025 acquired + placed-in-service post-cutoff WITHOUT election
        // → permanent 100%.
        let r = compute(&input(
            2025, 6, 1, 2025, 6, 1, 100_000_00, 5, false, false, false,
        ));
        assert_eq!(r.bonus_depreciation_rate_basis_points, 10000);
    }

    #[test]
    fn transition_election_not_available_for_2026() {
        // Transition election is only for FYE-after-2025-01-19, not 2026+.
        let r = compute(&input(
            2026, 6, 1, 2026, 6, 1, 100_000_00, 5, false, true, false,
        ));
        // Even with transition_election_made=true, 2026 doesn't qualify
        // → permanent 100% applies.
        assert!(!r.transition_election_year);
        assert_eq!(r.bonus_depreciation_rate_basis_points, 10000);
    }

    #[test]
    fn prior_use_disqualifies() {
        let r = compute(&input(
            2026, 6, 1, 2026, 6, 1, 100_000_00, 5, false, false, true,
        ));
        assert!(!r.qualified_property);
        assert_eq!(r.bonus_depreciation_amount_cents, 0);
        assert!(r.citation.contains("prior use"));
    }

    #[test]
    fn macrs_25_year_period_disqualifies() {
        // Recovery > 20 years → not qualified.
        let r = compute(&input(
            2026, 6, 1, 2026, 6, 1, 100_000_00, 25, false, false, false,
        ));
        assert!(!r.qualified_property);
        assert_eq!(r.bonus_depreciation_amount_cents, 0);
    }

    #[test]
    fn macrs_20_year_period_qualifies() {
        let r = compute(&input(
            2026, 6, 1, 2026, 6, 1, 100_000_00, 20, false, false, false,
        ));
        assert!(r.qualified_property);
    }

    #[test]
    fn pre_2018_50_percent_rate() {
        let r = compute(&input(
            2016, 6, 1, 2016, 6, 1, 100_000_00, 5, false, false, false,
        ));
        assert_eq!(r.bonus_depreciation_rate_basis_points, 5000);
    }

    #[test]
    fn tcja_2018_2022_100_percent() {
        for year in [2018, 2019, 2020, 2021, 2022] {
            let r = compute(&input(
                year, 6, 1, year, 6, 1, 100_000_00, 5, false, false, false,
            ));
            assert_eq!(
                r.bonus_depreciation_rate_basis_points, 10000,
                "year {} should be 100%",
                year
            );
        }
    }

    #[test]
    fn pre_obbba_2026_would_have_been_20_pct() {
        // Acquired BEFORE 2025-01-20 placed in service 2026 → pre-OBBBA
        // phasedown 20%.
        let r = compute(&input(
            2024, 12, 1, 2026, 6, 1, 100_000_00, 5, false, false, false,
        ));
        // Not both after cutoff → pre-OBBBA rate applies.
        assert!(!r.obbba_permanent_100_applies);
        assert_eq!(r.bonus_depreciation_rate_basis_points, 2000);
    }

    #[test]
    fn pre_obbba_2027_zero() {
        let r = compute(&input(
            2024, 12, 1, 2027, 6, 1, 100_000_00, 5, false, false, false,
        ));
        assert_eq!(r.bonus_depreciation_rate_basis_points, 0);
        assert_eq!(r.bonus_depreciation_amount_cents, 0);
    }

    #[test]
    fn obbba_2027_back_to_100_permanent() {
        // 2027 with OBBBA permanent rule + both dates after cutoff → 100%.
        let r = compute(&input(
            2027, 1, 1, 2027, 1, 1, 100_000_00, 5, false, false, false,
        ));
        assert!(r.obbba_permanent_100_applies);
        assert_eq!(r.bonus_depreciation_rate_basis_points, 10000);
    }

    #[test]
    fn acquisition_before_cutoff_placed_after_does_not_get_obbba() {
        // Acquired 2024-12-01, placed in service 2025-06-01. Both must
        // be after 2025-01-19 for OBBBA permanent — not satisfied here.
        let r = compute(&input(
            2024, 12, 1, 2025, 6, 1, 100_000_00, 5, false, false, false,
        ));
        assert!(!r.obbba_permanent_100_applies);
    }

    #[test]
    fn placed_before_cutoff_acquired_after_does_not_get_obbba() {
        // Acquired 2025-02-01 (after cutoff), placed in service 2025-01-15
        // (before cutoff) — both dates must be after 2025-01-19.
        let r = compute(&input(
            2025, 2, 1, 2025, 1, 15, 100_000_00, 5, false, false, false,
        ));
        assert!(!r.obbba_permanent_100_applies);
    }

    #[test]
    fn negative_cost_clamped() {
        let r = compute(&input(
            2026, 6, 1, 2026, 6, 1, -100, 5, false, false, false,
        ));
        assert_eq!(r.bonus_depreciation_amount_cents, 0);
    }

    #[test]
    fn citations_pin_correct_authorities() {
        let obbba = compute(&input(
            2026, 6, 1, 2026, 6, 1, 100_000_00, 5, false, false, false,
        ));
        assert!(obbba.citation.contains("OBBBA § 70302"));
        assert!(obbba.citation.contains("PERMANENT 100%"));

        let transition = compute(&input(
            2025, 6, 1, 2025, 6, 1, 100_000_00, 5, false, true, false,
        ));
        assert!(transition.citation.contains("transition election"));

        let phasedown = compute(&input(
            2024, 6, 1, 2024, 6, 1, 100_000_00, 5, false, false, false,
        ));
        assert!(phasedown.citation.contains("phasedown"));

        let not_qualified = compute(&input(
            2026, 6, 1, 2026, 6, 1, 100_000_00, 25, false, false, false,
        ));
        assert!(not_qualified.citation.contains("§ 168(k)(2)"));
    }
}
