//! IRC § 30D — Clean Vehicle Credit (formerly Plug-in Electric Drive Motor
//! Vehicle Credit before IRA 2022 redesign).
//!
//! IRA 2022 redesigned § 30D into a **bifurcated $7,500 credit**: $3,750
//! for satisfying the critical-minerals sourcing requirement + $3,750 for
//! satisfying the battery-components sourcing requirement, with a hard
//! income-based cutoff and MSRP caps. Scheduled to run through 2032.
//!
//! **OBBBA termination**: the One Big Beautiful Bill Act of 2025 (P.L.
//! 119-21, signed 2025-07-04) **terminated § 30D for vehicles acquired
//! after September 30, 2025** — accelerating the phaseout by more than 7
//! years. § 25E (used clean vehicle credit, $4,000 / 30% of price) was
//! terminated on the same date.
//!
//! **IRS binding-contract carve-out** (Notice 2025-XX) — if the taxpayer
//! had a **written binding contract** in place AND **made a payment** on
//! or before 2025-09-30, the credit is preserved even if the vehicle is
//! placed in service later. The "acquired" date snaps to the contract +
//! payment date.
//!
//! **Credit amounts (pre-termination)**:
//! - $3,750 if vehicle satisfies critical-minerals test (§ 30D(e)(1)).
//! - $3,750 if vehicle satisfies battery-components test (§ 30D(e)(2)).
//! - Total $7,500 if both; $3,750 if one; $0 if neither.
//!
//! **MSRP caps (§ 30D(f)(11))**:
//! - Cars: $55,000.
//! - SUVs / trucks / vans: $80,000.
//! - Vehicle MSRP exceeding cap → credit = $0.
//!
//! **Income phaseout (§ 30D(f)(10))** — HARD CUTOFF, not gradual. Credit
//! = $0 if MAGI exceeds threshold based on filing status:
//! - Single / MFS / HoH: $150,000 (HoH $225,000 per § 30D(f)(10)(B)(ii)).
//! - MFJ / QW: $300,000.
//!
//! The MAGI test allows the taxpayer to elect the LESSER of current-year
//! or prior-year MAGI under § 30D(f)(10)(B)(iii) — modeled here by
//! whichever value the caller passes.
//!
//! Citations: 26 U.S.C. § 30D; § 30D(e) (credit amounts + bifurcation);
//! § 30D(f)(10) (income phaseout); § 30D(f)(11) (MSRP caps); OBBBA § 70424
//! (eff. 2025-09-30 termination); IRS § 30D OBBBA FAQ (binding-contract
//! carve-out).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VehicleType {
    Car,
    SuvTruckVan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilingStatus {
    Single,
    MarriedFilingJointly,
    MarriedFilingSeparately,
    HeadOfHousehold,
    QualifyingWidow,
}

impl FilingStatus {
    fn magi_threshold_cents(self) -> i64 {
        match self {
            FilingStatus::MarriedFilingJointly | FilingStatus::QualifyingWidow => 30000000,
            FilingStatus::HeadOfHousehold => 22500000,
            FilingStatus::Single | FilingStatus::MarriedFilingSeparately => 15000000,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section30DInput {
    pub acquisition_year: u32,
    pub acquisition_month: u32,
    pub acquisition_day: u32,
    /// Whether the taxpayer had a written binding contract in place AND
    /// made a payment on or before 2025-09-30. Preserves the credit even
    /// if the vehicle is placed in service later.
    pub binding_contract_with_payment_before_2025_09_30: bool,
    pub vehicle_type: VehicleType,
    pub msrp_cents: i64,
    /// Modified AGI for § 30D(f)(10) phaseout. Caller may pass the lesser
    /// of current-year or prior-year per § 30D(f)(10)(B)(iii) election.
    pub modified_agi_cents: i64,
    pub filing_status: FilingStatus,
    /// § 30D(e)(1) critical-minerals sourcing test ($3,750).
    pub meets_critical_minerals_test: bool,
    /// § 30D(e)(2) battery-components sourcing test ($3,750).
    pub meets_battery_components_test: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section30DResult {
    pub acquisition_after_obbba_termination: bool,
    pub binding_contract_carve_out_applied: bool,
    pub msrp_cap_cents: i64,
    pub msrp_under_cap: bool,
    pub magi_threshold_cents: i64,
    pub magi_under_threshold: bool,
    pub critical_minerals_amount_cents: i64,
    pub battery_components_amount_cents: i64,
    pub credit_amount_cents: i64,
    pub credit_eligible: bool,
    pub citation: &'static str,
    pub note: String,
}

pub fn compute(input: &Section30DInput) -> Section30DResult {
    let msrp_cap = match input.vehicle_type {
        VehicleType::Car => 5500000,
        VehicleType::SuvTruckVan => 8000000,
    };
    let magi_threshold = input.filing_status.magi_threshold_cents();
    let msrp_under_cap = input.msrp_cents <= msrp_cap;
    let magi_under_threshold = input.modified_agi_cents <= magi_threshold;

    // OBBBA termination: vehicles acquired after 2025-09-30. The cutoff
    // is a calendar date — strict greater-than. Binding contract + payment
    // on or before the cutoff preserves eligibility.
    let after_termination = is_after_2025_09_30(
        input.acquisition_year,
        input.acquisition_month,
        input.acquisition_day,
    );
    let carve_out = input.binding_contract_with_payment_before_2025_09_30;
    let blocked_by_termination = after_termination && !carve_out;

    if blocked_by_termination {
        return Section30DResult {
            acquisition_after_obbba_termination: true,
            binding_contract_carve_out_applied: false,
            msrp_cap_cents: msrp_cap,
            msrp_under_cap,
            magi_threshold_cents: magi_threshold,
            magi_under_threshold,
            critical_minerals_amount_cents: 0,
            battery_components_amount_cents: 0,
            credit_amount_cents: 0,
            credit_eligible: false,
            citation:
                "26 U.S.C. § 30D + OBBBA § 70424 — credit TERMINATED for vehicles acquired after 2025-09-30",
            note: format!(
                "Vehicle acquired {}-{:02}-{:02} after the OBBBA § 70424 termination date 2025-09-30. No binding-contract carve-out. Credit unavailable.",
                input.acquisition_year, input.acquisition_month, input.acquisition_day
            ),
        };
    }

    if !magi_under_threshold {
        return Section30DResult {
            acquisition_after_obbba_termination: after_termination,
            binding_contract_carve_out_applied: carve_out && after_termination,
            msrp_cap_cents: msrp_cap,
            msrp_under_cap,
            magi_threshold_cents: magi_threshold,
            magi_under_threshold: false,
            critical_minerals_amount_cents: 0,
            battery_components_amount_cents: 0,
            credit_amount_cents: 0,
            credit_eligible: false,
            citation: "26 U.S.C. § 30D(f)(10) — credit unavailable when MAGI exceeds threshold (HARD CUTOFF, not gradual)",
            note: format!(
                "MAGI {} cents exceeds the § 30D(f)(10) threshold {} cents for {:?}. Credit hard-cutoff to $0 (taxpayer may elect lesser of current or prior year MAGI per § 30D(f)(10)(B)(iii)).",
                input.modified_agi_cents, magi_threshold, input.filing_status
            ),
        };
    }

    if !msrp_under_cap {
        return Section30DResult {
            acquisition_after_obbba_termination: after_termination,
            binding_contract_carve_out_applied: carve_out && after_termination,
            msrp_cap_cents: msrp_cap,
            msrp_under_cap: false,
            magi_threshold_cents: magi_threshold,
            magi_under_threshold,
            critical_minerals_amount_cents: 0,
            battery_components_amount_cents: 0,
            credit_amount_cents: 0,
            credit_eligible: false,
            citation: "26 U.S.C. § 30D(f)(11) — MSRP exceeds applicable cap ($55,000 cars / $80,000 SUVs+trucks+vans)",
            note: format!(
                "MSRP {} cents exceeds the {:?} cap {} cents. Credit unavailable per § 30D(f)(11).",
                input.msrp_cents, input.vehicle_type, msrp_cap
            ),
        };
    }

    let critical_minerals = if input.meets_critical_minerals_test { 375000 } else { 0 };
    let battery_components = if input.meets_battery_components_test { 375000 } else { 0 };
    let credit = critical_minerals + battery_components;

    Section30DResult {
        acquisition_after_obbba_termination: after_termination,
        binding_contract_carve_out_applied: carve_out && after_termination,
        msrp_cap_cents: msrp_cap,
        msrp_under_cap,
        magi_threshold_cents: magi_threshold,
        magi_under_threshold,
        critical_minerals_amount_cents: critical_minerals,
        battery_components_amount_cents: battery_components,
        credit_amount_cents: credit,
        credit_eligible: credit > 0,
        citation:
            "26 U.S.C. § 30D(e) — bifurcated $3,750 critical-minerals + $3,750 battery-components credit (pre-OBBBA termination 2025-09-30)",
        note: format!(
            "Critical-minerals test {}: ${}. Battery-components test {}: ${}. Total credit = {} cents.{}",
            if input.meets_critical_minerals_test { "MET" } else { "NOT MET" },
            critical_minerals / 100,
            if input.meets_battery_components_test { "MET" } else { "NOT MET" },
            battery_components / 100,
            credit,
            if carve_out && after_termination {
                " IRS binding-contract carve-out preserved eligibility despite acquisition date after 2025-09-30."
            } else {
                ""
            },
        ),
    }
}

fn is_after_2025_09_30(year: u32, month: u32, day: u32) -> bool {
    match year.cmp(&2025) {
        std::cmp::Ordering::Greater => true,
        std::cmp::Ordering::Less => false,
        std::cmp::Ordering::Equal => match month.cmp(&9) {
            std::cmp::Ordering::Greater => true,
            std::cmp::Ordering::Less => false,
            std::cmp::Ordering::Equal => day > 30,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        year: u32,
        month: u32,
        day: u32,
        carve_out: bool,
        vt: VehicleType,
        msrp: i64,
        magi: i64,
        fs: FilingStatus,
        crit: bool,
        battery: bool,
    ) -> Section30DInput {
        Section30DInput {
            acquisition_year: year,
            acquisition_month: month,
            acquisition_day: day,
            binding_contract_with_payment_before_2025_09_30: carve_out,
            vehicle_type: vt,
            msrp_cents: msrp,
            modified_agi_cents: magi,
            filing_status: fs,
            meets_critical_minerals_test: crit,
            meets_battery_components_test: battery,
        }
    }

    #[test]
    fn full_7500_credit_for_eligible_vehicle() {
        let r = compute(&input(
            2024, 6, 1, false, VehicleType::Car, 40_000_00, 100_000_00,
            FilingStatus::Single, true, true,
        ));
        assert_eq!(r.credit_amount_cents, 7_500_00);
        assert!(r.credit_eligible);
    }

    #[test]
    fn half_credit_critical_minerals_only() {
        let r = compute(&input(
            2024, 6, 1, false, VehicleType::Car, 40_000_00, 100_000_00,
            FilingStatus::Single, true, false,
        ));
        assert_eq!(r.credit_amount_cents, 3_750_00);
    }

    #[test]
    fn half_credit_battery_components_only() {
        let r = compute(&input(
            2024, 6, 1, false, VehicleType::Car, 40_000_00, 100_000_00,
            FilingStatus::Single, false, true,
        ));
        assert_eq!(r.credit_amount_cents, 3_750_00);
    }

    #[test]
    fn no_credit_neither_test_met() {
        let r = compute(&input(
            2024, 6, 1, false, VehicleType::Car, 40_000_00, 100_000_00,
            FilingStatus::Single, false, false,
        ));
        assert_eq!(r.credit_amount_cents, 0);
        assert!(!r.credit_eligible);
    }

    #[test]
    fn obbba_termination_acquired_after_2025_09_30() {
        let r = compute(&input(
            2025, 10, 1, false, VehicleType::Car, 40_000_00, 100_000_00,
            FilingStatus::Single, true, true,
        ));
        assert!(r.acquisition_after_obbba_termination);
        assert_eq!(r.credit_amount_cents, 0);
        assert!(r.citation.contains("OBBBA § 70424"));
        assert!(r.citation.contains("TERMINATED"));
    }

    #[test]
    fn at_2025_09_30_boundary_still_eligible() {
        // Exactly on the cutoff date — not "after" → still eligible.
        let r = compute(&input(
            2025, 9, 30, false, VehicleType::Car, 40_000_00, 100_000_00,
            FilingStatus::Single, true, true,
        ));
        assert!(!r.acquisition_after_obbba_termination);
        assert_eq!(r.credit_amount_cents, 7_500_00);
    }

    #[test]
    fn one_day_after_cutoff_terminated() {
        let r = compute(&input(
            2025, 10, 1, false, VehicleType::Car, 40_000_00, 100_000_00,
            FilingStatus::Single, true, true,
        ));
        assert!(r.acquisition_after_obbba_termination);
        assert_eq!(r.credit_amount_cents, 0);
    }

    #[test]
    fn binding_contract_carve_out_preserves_credit_after_cutoff() {
        let r = compute(&input(
            2025, 11, 15, true, VehicleType::Car, 40_000_00, 100_000_00,
            FilingStatus::Single, true, true,
        ));
        assert!(r.acquisition_after_obbba_termination);
        assert!(r.binding_contract_carve_out_applied);
        assert_eq!(r.credit_amount_cents, 7_500_00);
        assert!(r.note.contains("binding-contract carve-out"));
    }

    #[test]
    fn no_carve_out_no_payment_blocks_post_cutoff() {
        // After cutoff + no binding contract → blocked.
        let r = compute(&input(
            2025, 11, 15, false, VehicleType::Car, 40_000_00, 100_000_00,
            FilingStatus::Single, true, true,
        ));
        assert_eq!(r.credit_amount_cents, 0);
        assert!(!r.binding_contract_carve_out_applied);
    }

    #[test]
    fn magi_over_150k_single_no_credit() {
        let r = compute(&input(
            2024, 6, 1, false, VehicleType::Car, 40_000_00, 200_000_00,
            FilingStatus::Single, true, true,
        ));
        assert_eq!(r.credit_amount_cents, 0);
        assert!(r.citation.contains("§ 30D(f)(10)"));
        assert!(r.citation.contains("HARD CUTOFF"));
    }

    #[test]
    fn magi_at_150k_single_boundary_eligible() {
        let r = compute(&input(
            2024, 6, 1, false, VehicleType::Car, 40_000_00, 150_000_00,
            FilingStatus::Single, true, true,
        ));
        assert!(r.magi_under_threshold);
        assert_eq!(r.credit_amount_cents, 7_500_00);
    }

    #[test]
    fn magi_at_150k_plus_one_cent_blocks() {
        let r = compute(&input(
            2024, 6, 1, false, VehicleType::Car, 40_000_00, 150_000_00 + 1, FilingStatus::Single, true, true,
        ));
        assert_eq!(r.credit_amount_cents, 0);
    }

    #[test]
    fn mfj_uses_300k_threshold() {
        let r = compute(&input(
            2024, 6, 1, false, VehicleType::Car, 40_000_00, 280_000_00,
            FilingStatus::MarriedFilingJointly, true, true,
        ));
        assert_eq!(r.magi_threshold_cents, 300_000_00);
        assert_eq!(r.credit_amount_cents, 7_500_00);
    }

    #[test]
    fn hoh_uses_225k_threshold() {
        let r = compute(&input(
            2024, 6, 1, false, VehicleType::Car, 40_000_00, 200_000_00,
            FilingStatus::HeadOfHousehold, true, true,
        ));
        assert_eq!(r.magi_threshold_cents, 225_000_00);
        assert_eq!(r.credit_amount_cents, 7_500_00);
    }

    #[test]
    fn mfs_uses_150k_threshold_not_300k() {
        // MFS gets the SAME treatment as Single, not half of MFJ.
        let r = compute(&input(
            2024, 6, 1, false, VehicleType::Car, 40_000_00, 200_000_00,
            FilingStatus::MarriedFilingSeparately, true, true,
        ));
        assert_eq!(r.magi_threshold_cents, 150_000_00);
        assert_eq!(r.credit_amount_cents, 0);
    }

    #[test]
    fn car_msrp_over_55k_blocks_credit() {
        let r = compute(&input(
            2024, 6, 1, false, VehicleType::Car, 56_000_00, 100_000_00,
            FilingStatus::Single, true, true,
        ));
        assert_eq!(r.credit_amount_cents, 0);
        assert!(!r.msrp_under_cap);
        assert!(r.citation.contains("§ 30D(f)(11)"));
    }

    #[test]
    fn car_msrp_at_55k_boundary_eligible() {
        let r = compute(&input(
            2024, 6, 1, false, VehicleType::Car, 55_000_00, 100_000_00,
            FilingStatus::Single, true, true,
        ));
        assert!(r.msrp_under_cap);
        assert_eq!(r.credit_amount_cents, 7_500_00);
    }

    #[test]
    fn suv_msrp_up_to_80k_eligible() {
        let r = compute(&input(
            2024, 6, 1, false, VehicleType::SuvTruckVan, 80_000_00, 100_000_00,
            FilingStatus::Single, true, true,
        ));
        assert_eq!(r.msrp_cap_cents, 80_000_00);
        assert_eq!(r.credit_amount_cents, 7_500_00);
    }

    #[test]
    fn suv_msrp_over_80k_blocks_credit() {
        let r = compute(&input(
            2024, 6, 1, false, VehicleType::SuvTruckVan, 81_000_00, 100_000_00,
            FilingStatus::Single, true, true,
        ));
        assert_eq!(r.credit_amount_cents, 0);
    }

    #[test]
    fn pre_termination_2024_eligible_with_all_conditions() {
        let r = compute(&input(
            2024, 1, 15, false, VehicleType::Car, 40_000_00, 100_000_00,
            FilingStatus::Single, true, true,
        ));
        assert_eq!(r.credit_amount_cents, 7_500_00);
        assert!(!r.acquisition_after_obbba_termination);
    }

    #[test]
    fn termination_check_order_independent_of_other_blocks() {
        // High MSRP + post-termination — termination should fire first.
        let r = compute(&input(
            2026, 1, 1, false, VehicleType::Car, 100_000_00, 100_000_00,
            FilingStatus::Single, true, true,
        ));
        assert!(r.citation.contains("TERMINATED"));
    }

    #[test]
    fn full_credit_breakdown_matches_components() {
        let r = compute(&input(
            2024, 6, 1, false, VehicleType::Car, 40_000_00, 100_000_00,
            FilingStatus::Single, true, true,
        ));
        assert_eq!(r.critical_minerals_amount_cents, 3_750_00);
        assert_eq!(r.battery_components_amount_cents, 3_750_00);
        assert_eq!(
            r.credit_amount_cents,
            r.critical_minerals_amount_cents + r.battery_components_amount_cents
        );
    }

    #[test]
    fn citations_pin_correct_authorities() {
        let pre = compute(&input(
            2024, 6, 1, false, VehicleType::Car, 40_000_00, 100_000_00,
            FilingStatus::Single, true, true,
        ));
        assert!(pre.citation.contains("§ 30D(e)"));

        let post = compute(&input(
            2026, 1, 1, false, VehicleType::Car, 40_000_00, 100_000_00,
            FilingStatus::Single, true, true,
        ));
        assert!(post.citation.contains("OBBBA § 70424"));

        let high_magi = compute(&input(
            2024, 6, 1, false, VehicleType::Car, 40_000_00, 200_000_00,
            FilingStatus::Single, true, true,
        ));
        assert!(high_magi.citation.contains("§ 30D(f)(10)"));

        let high_msrp = compute(&input(
            2024, 6, 1, false, VehicleType::Car, 60_000_00, 100_000_00,
            FilingStatus::Single, true, true,
        ));
        assert!(high_msrp.citation.contains("§ 30D(f)(11)"));
    }

    #[test]
    fn year_boundary_2024_pre_termination_2026_post() {
        let r_2024 = compute(&input(
            2024, 12, 31, false, VehicleType::Car, 40_000_00, 100_000_00,
            FilingStatus::Single, true, true,
        ));
        let r_2026 = compute(&input(
            2026, 1, 1, false, VehicleType::Car, 40_000_00, 100_000_00,
            FilingStatus::Single, true, true,
        ));
        assert_eq!(r_2024.credit_amount_cents, 7_500_00);
        assert_eq!(r_2026.credit_amount_cents, 0);
    }

    #[test]
    fn date_boundary_2025_09_29_vs_30_vs_oct_1() {
        let day29 = compute(&input(
            2025, 9, 29, false, VehicleType::Car, 40_000_00, 100_000_00,
            FilingStatus::Single, true, true,
        ));
        let day30 = compute(&input(
            2025, 9, 30, false, VehicleType::Car, 40_000_00, 100_000_00,
            FilingStatus::Single, true, true,
        ));
        let oct1 = compute(&input(
            2025, 10, 1, false, VehicleType::Car, 40_000_00, 100_000_00,
            FilingStatus::Single, true, true,
        ));
        assert_eq!(day29.credit_amount_cents, 7_500_00);
        assert_eq!(day30.credit_amount_cents, 7_500_00);
        assert_eq!(oct1.credit_amount_cents, 0);
    }
}
