//! NYC Local Law 152 of 2016 periodic gas piping inspection
//! compliance for trader-landlords with NYC building inventory.
//!
//! Enacted in response to the March 12, 2014 East Harlem gas
//! explosion at 1644-1646 Park Avenue that killed 8 people and
//! collapsed two buildings due to undetected gas-piping deterioration.
//!
//! **Operative rule** (NYC Admin Code § 28-318): exposed gas piping
//! systems in ALL NYC buildings — EXCEPT one- and two-family homes
//! (Occupancy Group R-3) — must be inspected every **4 years** by a
//! Licensed Master Plumber (LMP) or qualified individual working
//! for an LMP.
//!
//! **Community-district rotating schedule** (1 RCNY § 103-10):
//!
//! - Districts 1, 3, 10: 2020, 2024, 2028, 2032 ...
//! - Districts 2, 5, 7, 13, 18: 2021, 2025, 2029, 2033 ...
//! - Districts 4, 6, 8, 9, 16: 2022, 2026, 2030, 2034 ...
//! - Districts 11, 12, 14, 15, 17: 2023, 2027, 2031, 2035 ...
//!
//! **Filing workflow**:
//!
//! - LMP delivers Gas Piping System Periodic Inspection Report
//!   (GPS1) to building owner within **30 days** of inspection.
//! - Owner submits Gas Piping System Periodic Inspection
//!   Certification (GPS2) to DOB within **60 days** of inspection.
//! - If conditions need correction: follow-up certification within
//!   **120 days** of inspection confirming all conditions corrected,
//!   OR within **180 days** with DOB-approved extension.
//!
//! **Unsafe condition workflow**: if LMP identifies any unsafe or
//! hazardous condition, LMP must IMMEDIATELY (1) call 911, (2)
//! notify building owner, (3) notify utility providing gas service,
//! (4) notify NYC DOB. For immediately hazardous conditions, LMP
//! may advise gas shutoff to section or entire building as a
//! priority for life safety.
//!
//! **Penalties** (1 RCNY § 102-01):
//!
//! - Missed GPS2 filing: **$10,000** fine + **$10,000 additional
//!   per year** of continuing noncompliance.
//! - Cycle 2 and beyond non-certification: **$1,500** for 1-3 family
//!   homes (R-3 buildings still subject to a separate certification
//!   requirement); **$5,000** for all other buildings.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[allow(dead_code)]
pub const LL152_ENACTMENT_YEAR: u32 = 2016;
#[allow(dead_code)]
pub const LL152_INSPECTION_CYCLE_YEARS: u32 = 4;
#[allow(dead_code)]
pub const LL152_GPS1_DELIVERY_DAYS_TO_OWNER: u32 = 30;
#[allow(dead_code)]
pub const LL152_GPS2_FILING_DAYS_TO_DOB: u32 = 60;
#[allow(dead_code)]
pub const LL152_CORRECTION_CERTIFICATION_DAYS: u32 = 120;
#[allow(dead_code)]
pub const LL152_CORRECTION_EXTENSION_DAYS: u32 = 180;
#[allow(dead_code)]
pub const LL152_MISSED_FILING_INITIAL_PENALTY_CENTS: u64 = 1_000_000;
#[allow(dead_code)]
pub const LL152_MISSED_FILING_ANNUAL_PENALTY_CENTS: u64 = 1_000_000;
#[allow(dead_code)]
pub const LL152_CYCLE_2_RESIDENTIAL_PENALTY_CENTS: u64 = 150_000;
#[allow(dead_code)]
pub const LL152_CYCLE_2_OTHER_PENALTY_CENTS: u64 = 500_000;
#[allow(dead_code)]
pub const EAST_HARLEM_EXPLOSION_FATALITY_COUNT: u32 = 8;
#[allow(dead_code)]
pub const EAST_HARLEM_EXPLOSION_YEAR: u32 = 2014;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OccupancyGroup {
    R3OneOrTwoFamily,
    R1Multifamily,
    R2Multifamily,
    OtherCommercial,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    ExemptOneOrTwoFamilyR3,
    CompliantInspectionFiledOnTimeNoUnsafeConditions,
    CompliantInspectionFiledWithCorrectionsCertifiedWithin120Days,
    CompliantInspectionFiledWithCorrectionExtension180Days,
    ViolationLmpNotLicensedMasterPlumber,
    ViolationGps1NotDeliveredWithin30Days,
    ViolationGps2NotFiledWithin60Days,
    ViolationCorrectionsNotCertifiedWithin120Or180Days,
    ViolationUnsafeConditionGasNotShutOff,
    ViolationCommunityDistrictDeadlineMissed,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub building_occupancy_group: OccupancyGroup,
    pub community_district: u32,
    pub current_year: u32,
    pub inspection_performed_in_district_year: bool,
    pub lmp_licensed_master_plumber: bool,
    pub gps1_delivered_to_owner_within_30_days: bool,
    pub gps2_filed_to_dob_within_60_days: bool,
    pub unsafe_condition_identified: bool,
    pub gas_shutoff_advised_if_unsafe: bool,
    pub owner_corrected_conditions: bool,
    pub days_to_correction_certification: u32,
    pub dob_approved_extension: bool,
    pub cycle_number: u32,
    pub years_since_missed_filing: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Output {
    pub severity: Severity,
    pub compliant: bool,
    pub total_penalty_cents: u64,
    pub corrective_action_required: bool,
    pub inspection_required: bool,
    pub assigned_district_inspection_year: u32,
    pub notes: Vec<String>,
    pub citations: Vec<String>,
}

pub type GasPipingInspectionLocalLaw152Input = Input;
pub type GasPipingInspectionLocalLaw152Output = Output;
pub type GasPipingInspectionLocalLaw152Result = Output;

pub fn check(input: &Input) -> Output {
    let mut notes: Vec<String> = Vec::new();
    let citations: Vec<String> = vec![
        "NYC Local Law 152 of 2016 (periodic gas piping inspection)".to_string(),
        "NYC Admin Code § 28-318 (statutory basis)".to_string(),
        "1 RCNY § 103-10 (gas piping inspection rules)".to_string(),
        "1 RCNY § 102-01 (penalty schedule)".to_string(),
        "NYC DOB Gas Piping Inspection Forms GPS1 and GPS2".to_string(),
        "East Harlem gas explosion (March 12, 2014) — 1644-1646 Park Avenue — 8 fatalities — precipitating incident for LL 152".to_string(),
    ];

    let assigned_district_year = compute_district_year(input.community_district);

    if matches!(
        input.building_occupancy_group,
        OccupancyGroup::R3OneOrTwoFamily
    ) {
        notes.push(
            "Occupancy Group R-3 (one- or two-family home) — exempt from LL 152 inspection regime."
                .to_string(),
        );
        return Output {
            severity: Severity::ExemptOneOrTwoFamilyR3,
            compliant: true,
            total_penalty_cents: 0,
            corrective_action_required: false,
            inspection_required: false,
            assigned_district_inspection_year: assigned_district_year,
            notes,
            citations,
        };
    }

    if !input.inspection_performed_in_district_year && input.current_year >= assigned_district_year
    {
        notes.push(format!(
            "Community District {} assigned inspection year {}; no inspection performed. Missed district deadline.",
            input.community_district, assigned_district_year
        ));
        let mut penalty = LL152_MISSED_FILING_INITIAL_PENALTY_CENTS;
        penalty = penalty.saturating_add(
            LL152_MISSED_FILING_ANNUAL_PENALTY_CENTS
                .saturating_mul(input.years_since_missed_filing as u64),
        );
        return Output {
            severity: Severity::ViolationCommunityDistrictDeadlineMissed,
            compliant: false,
            total_penalty_cents: penalty,
            corrective_action_required: false,
            inspection_required: true,
            assigned_district_inspection_year: assigned_district_year,
            notes,
            citations,
        };
    }

    if !input.lmp_licensed_master_plumber {
        notes.push("Inspection not performed by Licensed Master Plumber (LMP) — per se LL 152 procedural violation.".to_string());
        return Output {
            severity: Severity::ViolationLmpNotLicensedMasterPlumber,
            compliant: false,
            total_penalty_cents: 0,
            corrective_action_required: true,
            inspection_required: true,
            assigned_district_inspection_year: assigned_district_year,
            notes,
            citations,
        };
    }

    if input.unsafe_condition_identified && !input.gas_shutoff_advised_if_unsafe {
        notes.push("Unsafe/hazardous gas-piping condition identified but LMP did not advise gas shutoff — life-safety violation; mandatory immediate 911 + utility + DOB notification per LL 152.".to_string());
        return Output {
            severity: Severity::ViolationUnsafeConditionGasNotShutOff,
            compliant: false,
            total_penalty_cents: 0,
            corrective_action_required: true,
            inspection_required: true,
            assigned_district_inspection_year: assigned_district_year,
            notes,
            citations,
        };
    }

    if !input.gps1_delivered_to_owner_within_30_days {
        notes.push(format!(
            "GPS1 not delivered to owner within {} days of inspection — LMP procedural violation.",
            LL152_GPS1_DELIVERY_DAYS_TO_OWNER
        ));
        return Output {
            severity: Severity::ViolationGps1NotDeliveredWithin30Days,
            compliant: false,
            total_penalty_cents: 0,
            corrective_action_required: true,
            inspection_required: false,
            assigned_district_inspection_year: assigned_district_year,
            notes,
            citations,
        };
    }

    if !input.gps2_filed_to_dob_within_60_days {
        notes.push(format!(
            "GPS2 not filed with DOB within {} days of inspection — ${} initial penalty + ${}/year continuing.",
            LL152_GPS2_FILING_DAYS_TO_DOB,
            LL152_MISSED_FILING_INITIAL_PENALTY_CENTS / 100,
            LL152_MISSED_FILING_ANNUAL_PENALTY_CENTS / 100
        ));
        let mut penalty = LL152_MISSED_FILING_INITIAL_PENALTY_CENTS;
        penalty = penalty.saturating_add(
            LL152_MISSED_FILING_ANNUAL_PENALTY_CENTS
                .saturating_mul(input.years_since_missed_filing as u64),
        );
        return Output {
            severity: Severity::ViolationGps2NotFiledWithin60Days,
            compliant: false,
            total_penalty_cents: penalty,
            corrective_action_required: false,
            inspection_required: false,
            assigned_district_inspection_year: assigned_district_year,
            notes,
            citations,
        };
    }

    if input.owner_corrected_conditions {
        let within_120 =
            input.days_to_correction_certification <= LL152_CORRECTION_CERTIFICATION_DAYS;
        let within_180_with_extension = input.days_to_correction_certification
            <= LL152_CORRECTION_EXTENSION_DAYS
            && input.dob_approved_extension;
        if within_120 {
            notes.push(format!(
                "Corrective certification filed within {} days of inspection.",
                LL152_CORRECTION_CERTIFICATION_DAYS
            ));
            return Output {
                severity: Severity::CompliantInspectionFiledWithCorrectionsCertifiedWithin120Days,
                compliant: true,
                total_penalty_cents: 0,
                corrective_action_required: false,
                inspection_required: false,
                assigned_district_inspection_year: assigned_district_year,
                notes,
                citations,
            };
        }
        if within_180_with_extension {
            notes.push(format!(
                "Corrective certification filed within {} days under DOB-approved extension.",
                LL152_CORRECTION_EXTENSION_DAYS
            ));
            return Output {
                severity: Severity::CompliantInspectionFiledWithCorrectionExtension180Days,
                compliant: true,
                total_penalty_cents: 0,
                corrective_action_required: false,
                inspection_required: false,
                assigned_district_inspection_year: assigned_district_year,
                notes,
                citations,
            };
        }
        let cycle_2_penalty = if matches!(
            input.building_occupancy_group,
            OccupancyGroup::R1Multifamily | OccupancyGroup::R2Multifamily
        ) && input.cycle_number >= 2
        {
            LL152_CYCLE_2_RESIDENTIAL_PENALTY_CENTS
        } else if input.cycle_number >= 2 {
            LL152_CYCLE_2_OTHER_PENALTY_CENTS
        } else {
            0
        };
        notes.push(format!(
            "Corrective certification {} days exceeds 120/180-day windows; Cycle {} non-certification penalty ${}.",
            input.days_to_correction_certification,
            input.cycle_number,
            cycle_2_penalty / 100
        ));
        return Output {
            severity: Severity::ViolationCorrectionsNotCertifiedWithin120Or180Days,
            compliant: false,
            total_penalty_cents: cycle_2_penalty,
            corrective_action_required: true,
            inspection_required: false,
            assigned_district_inspection_year: assigned_district_year,
            notes,
            citations,
        };
    }

    notes.push("LL 152 compliant: inspection performed in-cycle by LMP; GPS1 + GPS2 filed timely; no unsafe conditions identified.".to_string());
    Output {
        severity: Severity::CompliantInspectionFiledOnTimeNoUnsafeConditions,
        compliant: true,
        total_penalty_cents: 0,
        corrective_action_required: false,
        inspection_required: false,
        assigned_district_inspection_year: assigned_district_year,
        notes,
        citations,
    }
}

fn compute_district_year(district: u32) -> u32 {
    match district {
        1 | 3 | 10 => 2024,
        2 | 5 | 7 | 13 | 18 => 2025,
        4 | 6 | 8 | 9 | 16 => 2026,
        11 | 12 | 14 | 15 | 17 => 2027,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_compliant_district_4() -> Input {
        Input {
            building_occupancy_group: OccupancyGroup::OtherCommercial,
            community_district: 4,
            current_year: 2026,
            inspection_performed_in_district_year: true,
            lmp_licensed_master_plumber: true,
            gps1_delivered_to_owner_within_30_days: true,
            gps2_filed_to_dob_within_60_days: true,
            unsafe_condition_identified: false,
            gas_shutoff_advised_if_unsafe: false,
            owner_corrected_conditions: false,
            days_to_correction_certification: 0,
            dob_approved_extension: false,
            cycle_number: 2,
            years_since_missed_filing: 0,
        }
    }

    #[test]
    fn district_4_compliant_inspection_no_unsafe() {
        let out = check(&base_compliant_district_4());
        assert_eq!(
            out.severity,
            Severity::CompliantInspectionFiledOnTimeNoUnsafeConditions
        );
        assert_eq!(out.assigned_district_inspection_year, 2026);
        assert!(out.compliant);
    }

    #[test]
    fn r3_one_or_two_family_exempt() {
        let mut i = base_compliant_district_4();
        i.building_occupancy_group = OccupancyGroup::R3OneOrTwoFamily;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExemptOneOrTwoFamilyR3);
    }

    #[test]
    fn district_1_assigned_year_2024() {
        let mut i = base_compliant_district_4();
        i.community_district = 1;
        let out = check(&i);
        assert_eq!(out.assigned_district_inspection_year, 2024);
    }

    #[test]
    fn district_2_assigned_year_2025() {
        let mut i = base_compliant_district_4();
        i.community_district = 2;
        let out = check(&i);
        assert_eq!(out.assigned_district_inspection_year, 2025);
    }

    #[test]
    fn district_11_assigned_year_2027() {
        let mut i = base_compliant_district_4();
        i.community_district = 11;
        let out = check(&i);
        assert_eq!(out.assigned_district_inspection_year, 2027);
    }

    #[test]
    fn missed_district_deadline_no_inspection_penalty() {
        let mut i = base_compliant_district_4();
        i.inspection_performed_in_district_year = false;
        i.years_since_missed_filing = 2;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationCommunityDistrictDeadlineMissed
        );
        assert_eq!(out.total_penalty_cents, 3_000_000);
    }

    #[test]
    fn non_lmp_inspector_violation() {
        let mut i = base_compliant_district_4();
        i.lmp_licensed_master_plumber = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ViolationLmpNotLicensedMasterPlumber);
    }

    #[test]
    fn unsafe_condition_no_gas_shutoff_violation() {
        let mut i = base_compliant_district_4();
        i.unsafe_condition_identified = true;
        i.gas_shutoff_advised_if_unsafe = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationUnsafeConditionGasNotShutOff
        );
    }

    #[test]
    fn unsafe_condition_with_shutoff_advised_proceeds() {
        let mut i = base_compliant_district_4();
        i.unsafe_condition_identified = true;
        i.gas_shutoff_advised_if_unsafe = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::CompliantInspectionFiledOnTimeNoUnsafeConditions
        );
    }

    #[test]
    fn gps1_not_delivered_within_30_days_violation() {
        let mut i = base_compliant_district_4();
        i.gps1_delivered_to_owner_within_30_days = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationGps1NotDeliveredWithin30Days
        );
    }

    #[test]
    fn gps2_not_filed_within_60_days_violation() {
        let mut i = base_compliant_district_4();
        i.gps2_filed_to_dob_within_60_days = false;
        i.years_since_missed_filing = 0;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ViolationGps2NotFiledWithin60Days);
        assert_eq!(out.total_penalty_cents, 1_000_000);
    }

    #[test]
    fn gps2_not_filed_2_years_penalty_30000() {
        let mut i = base_compliant_district_4();
        i.gps2_filed_to_dob_within_60_days = false;
        i.years_since_missed_filing = 2;
        let out = check(&i);
        assert_eq!(out.total_penalty_cents, 3_000_000);
    }

    #[test]
    fn corrections_within_120_days_compliant() {
        let mut i = base_compliant_district_4();
        i.owner_corrected_conditions = true;
        i.days_to_correction_certification = 100;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::CompliantInspectionFiledWithCorrectionsCertifiedWithin120Days
        );
    }

    #[test]
    fn corrections_at_exactly_120_days_compliant() {
        let mut i = base_compliant_district_4();
        i.owner_corrected_conditions = true;
        i.days_to_correction_certification = 120;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::CompliantInspectionFiledWithCorrectionsCertifiedWithin120Days
        );
    }

    #[test]
    fn corrections_121_days_without_extension_violation() {
        let mut i = base_compliant_district_4();
        i.owner_corrected_conditions = true;
        i.days_to_correction_certification = 121;
        i.dob_approved_extension = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationCorrectionsNotCertifiedWithin120Or180Days
        );
    }

    #[test]
    fn corrections_180_days_with_extension_compliant() {
        let mut i = base_compliant_district_4();
        i.owner_corrected_conditions = true;
        i.days_to_correction_certification = 180;
        i.dob_approved_extension = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::CompliantInspectionFiledWithCorrectionExtension180Days
        );
    }

    #[test]
    fn corrections_181_days_with_extension_violation() {
        let mut i = base_compliant_district_4();
        i.owner_corrected_conditions = true;
        i.days_to_correction_certification = 181;
        i.dob_approved_extension = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationCorrectionsNotCertifiedWithin120Or180Days
        );
    }

    #[test]
    fn cycle_2_residential_uncorrected_penalty_1500() {
        let mut i = base_compliant_district_4();
        i.building_occupancy_group = OccupancyGroup::R1Multifamily;
        i.owner_corrected_conditions = true;
        i.days_to_correction_certification = 200;
        i.cycle_number = 2;
        let out = check(&i);
        assert_eq!(out.total_penalty_cents, 150_000);
    }

    #[test]
    fn cycle_2_commercial_uncorrected_penalty_5000() {
        let mut i = base_compliant_district_4();
        i.building_occupancy_group = OccupancyGroup::OtherCommercial;
        i.owner_corrected_conditions = true;
        i.days_to_correction_certification = 200;
        i.cycle_number = 2;
        let out = check(&i);
        assert_eq!(out.total_penalty_cents, 500_000);
    }

    #[test]
    fn citations_pin_ll152_admin_code_28_318_rcny_103_10() {
        let out = check(&base_compliant_district_4());
        assert!(out
            .citations
            .iter()
            .any(|c| c.contains("Local Law 152 of 2016")));
        assert!(out.citations.iter().any(|c| c.contains("§ 28-318")));
        assert!(out.citations.iter().any(|c| c.contains("1 RCNY § 103-10")));
        assert!(out.citations.iter().any(|c| c.contains("1 RCNY § 102-01")));
    }

    #[test]
    fn citations_pin_east_harlem_explosion_2014() {
        let out = check(&base_compliant_district_4());
        assert!(out.citations.iter().any(|c| c.contains("East Harlem")));
        assert!(out.citations.iter().any(|c| c.contains("March 12, 2014")));
        assert!(out.citations.iter().any(|c| c.contains("8 fatalities")));
    }

    #[test]
    fn constant_pin_ll152_4_year_cycle() {
        assert_eq!(LL152_INSPECTION_CYCLE_YEARS, 4);
    }

    #[test]
    fn constant_pin_gps1_30_day_delivery() {
        assert_eq!(LL152_GPS1_DELIVERY_DAYS_TO_OWNER, 30);
    }

    #[test]
    fn constant_pin_gps2_60_day_filing() {
        assert_eq!(LL152_GPS2_FILING_DAYS_TO_DOB, 60);
    }

    #[test]
    fn constant_pin_120_day_correction() {
        assert_eq!(LL152_CORRECTION_CERTIFICATION_DAYS, 120);
    }

    #[test]
    fn constant_pin_180_day_correction_with_extension() {
        assert_eq!(LL152_CORRECTION_EXTENSION_DAYS, 180);
    }

    #[test]
    fn constant_pin_10000_initial_penalty() {
        assert_eq!(LL152_MISSED_FILING_INITIAL_PENALTY_CENTS, 1_000_000);
    }

    #[test]
    fn constant_pin_east_harlem_8_fatalities() {
        assert_eq!(EAST_HARLEM_EXPLOSION_FATALITY_COUNT, 8);
    }
}
