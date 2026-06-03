//! NYC Local Law 97 of 2019 (Climate Mobilization Act) building
//! greenhouse-gas emissions cap compliance for trader-landlords
//! with NYC inventory.
//!
//! Local Law 97 was enacted in 2019 as part of the NYC Climate
//! Mobilization Act and is codified at NYC Admin Code § 28-320 et
//! seq. Emissions caps phase in starting 2024 with progressive
//! tightening through 2050 to deliver a 40% citywide building-
//! emissions reduction by 2030 and 80% by 2050. Buildings are
//! responsible for ~two-thirds of NYC's total greenhouse-gas
//! footprint, making LL 97 the most aggressive municipal building-
//! emissions regulation in the United States.
//!
//! **Applicability**: buildings with gross floor area greater than
//! **25,000 square feet** (with limited exceptions for industrial
//! buildings, NYCHA, and rent-regulated portions covered separately).
//!
//! **Compliance periods** with emissions intensity limits (kgCO2e
//! per square foot) by occupancy group:
//!
//! - **R-2 multifamily 2024-2029**: 6.75 kgCO2e/sqft
//! - **R-2 multifamily 2030-2034**: 4.07 kgCO2e/sqft
//! - **B business / office 2024-2029**: 8.46 kgCO2e/sqft
//! - **B business / office 2030-2034**: 4.53 kgCO2e/sqft
//!
//! **Penalty**: $268 per metric ton CO2e annual emissions over the
//! building's applicable intensity-limit allowance (NYC Admin Code
//! § 28-320.6 + Rules of City of New York 103-14).
//!
//! **Article 321 alternative compliance pathway**: buildings with
//! more than **35% rent-regulated units** may implement **13
//! prescriptive energy conservation measures** (ECMs) instead of
//! meeting emissions limits directly. Measures include lighting
//! retrofits, heating-system upgrades, insulation, window seals,
//! controls, and similar prescriptive items. Article 321 compliance
//! must be certified by a registered design professional.
//!
//! **Adjustments**: financial hardship adjustment under § 28-320.7,
//! critical facility adjustment under § 28-320.8, special-circumstance
//! adjustment under DOB rulemaking. Approval is discretionary;
//! denials produce full emissions-cap exposure.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[allow(dead_code)]
pub const LL97_BUILDING_SIZE_THRESHOLD_SQFT: u32 = 25_000;
#[allow(dead_code)]
pub const LL97_PENALTY_PER_METRIC_TON_CO2E_CENTS: u64 = 26_800;
#[allow(dead_code)]
pub const LL97_ENACTMENT_YEAR: u32 = 2019;
#[allow(dead_code)]
pub const LL97_FIRST_COMPLIANCE_PERIOD_START_YEAR: u32 = 2024;
#[allow(dead_code)]
pub const LL97_FIRST_COMPLIANCE_PERIOD_END_YEAR: u32 = 2029;
#[allow(dead_code)]
pub const LL97_SECOND_COMPLIANCE_PERIOD_START_YEAR: u32 = 2030;
#[allow(dead_code)]
pub const LL97_SECOND_COMPLIANCE_PERIOD_END_YEAR: u32 = 2034;
#[allow(dead_code)]
pub const LL97_FINAL_TARGET_YEAR: u32 = 2050;
#[allow(dead_code)]
pub const LL97_FINAL_REDUCTION_TARGET_PERCENT: u32 = 80;
#[allow(dead_code)]
pub const LL97_R2_MULTIFAMILY_2024_2029_KG_CO2E_PER_SQFT_X_100: u32 = 675;
#[allow(dead_code)]
pub const LL97_R2_MULTIFAMILY_2030_2034_KG_CO2E_PER_SQFT_X_100: u32 = 407;
#[allow(dead_code)]
pub const LL97_B_BUSINESS_2024_2029_KG_CO2E_PER_SQFT_X_100: u32 = 846;
#[allow(dead_code)]
pub const LL97_B_BUSINESS_2030_2034_KG_CO2E_PER_SQFT_X_100: u32 = 453;
#[allow(dead_code)]
pub const ARTICLE_321_RENT_REGULATED_THRESHOLD_PERCENT: u32 = 35;
#[allow(dead_code)]
pub const ARTICLE_321_PRESCRIPTIVE_ECMS_COUNT: u32 = 13;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OccupancyGroup {
    R2Multifamily,
    BBusinessOffice,
    MMercantile,
    OtherCovered,
    R3OneOrTwoFamilyOrExempt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    ExemptBuildingUnder25000Sqft,
    ExemptOccupancyNotCovered,
    CompliantEmissionsWithinIntensityLimit,
    CompliantArticle321PrescriptiveEcmsRentRegulated,
    CompliantApprovedHardshipAdjustment,
    CompliantApprovedCriticalFacilityAdjustment,
    ViolationEmissionsExceedLimitPenaltyAccrues,
    ViolationArticle321ClaimedButRentRegulatedThresholdNotMet,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub occupancy_group: OccupancyGroup,
    pub gross_floor_area_sqft: u32,
    pub current_year: u32,
    pub annual_emissions_kg_co2e: u64,
    pub rent_regulated_unit_percent: u32,
    pub article_321_prescriptive_ecms_in_place: bool,
    pub approved_hardship_adjustment: bool,
    pub approved_critical_facility_adjustment: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Output {
    pub severity: Severity,
    pub applicable_intensity_limit_kg_co2e_per_sqft_x_100: u32,
    pub annual_emissions_allowance_kg_co2e: u64,
    pub metric_tons_over_limit: u64,
    pub annual_penalty_cents: u64,
    pub notes: Vec<String>,
    pub citations: Vec<String>,
}

pub type ClimateMobilizationActLl97EmissionsInput = Input;
pub type ClimateMobilizationActLl97EmissionsOutput = Output;
pub type ClimateMobilizationActLl97EmissionsResult = Output;

pub fn check(input: &Input) -> Output {
    let mut notes: Vec<String> = Vec::new();
    let citations: Vec<String> = vec![
        "NYC Local Law 97 of 2019 (Climate Mobilization Act)".to_string(),
        "NYC Admin Code § 28-320 et seq. (statutory basis)".to_string(),
        "NYC Admin Code § 28-320.6 (penalty schedule — $268/tCO2e)".to_string(),
        "NYC Admin Code § 28-320.7 (financial hardship adjustment)".to_string(),
        "NYC Admin Code § 28-320.8 (critical facility adjustment)".to_string(),
        "Article 321 (alternative compliance for rent-regulated buildings)".to_string(),
        "1 RCNY § 103-14 (calculation of emission limits for buildings)".to_string(),
        "NYC DOB LL 97 Greenhouse Gas Emissions Reductions page".to_string(),
        "NYC Mayor's Office of Climate & Environmental Justice (MOCEJ) implementation".to_string(),
    ];

    if input.gross_floor_area_sqft <= LL97_BUILDING_SIZE_THRESHOLD_SQFT {
        notes.push(format!(
            "Building {} sqft ≤ {} threshold — exempt from LL 97 emissions cap.",
            input.gross_floor_area_sqft, LL97_BUILDING_SIZE_THRESHOLD_SQFT
        ));
        return Output {
            severity: Severity::ExemptBuildingUnder25000Sqft,
            applicable_intensity_limit_kg_co2e_per_sqft_x_100: 0,
            annual_emissions_allowance_kg_co2e: 0,
            metric_tons_over_limit: 0,
            annual_penalty_cents: 0,
            notes,
            citations,
        };
    }

    if matches!(
        input.occupancy_group,
        OccupancyGroup::R3OneOrTwoFamilyOrExempt
    ) {
        notes.push("Occupancy group R-3 (one- or two-family home) or otherwise exempt — outside LL 97 scope.".to_string());
        return Output {
            severity: Severity::ExemptOccupancyNotCovered,
            applicable_intensity_limit_kg_co2e_per_sqft_x_100: 0,
            annual_emissions_allowance_kg_co2e: 0,
            metric_tons_over_limit: 0,
            annual_penalty_cents: 0,
            notes,
            citations,
        };
    }

    if input.approved_critical_facility_adjustment {
        notes.push("DOB-approved § 28-320.8 critical facility adjustment in effect — emissions-limit exposure suspended.".to_string());
        return Output {
            severity: Severity::CompliantApprovedCriticalFacilityAdjustment,
            applicable_intensity_limit_kg_co2e_per_sqft_x_100: 0,
            annual_emissions_allowance_kg_co2e: 0,
            metric_tons_over_limit: 0,
            annual_penalty_cents: 0,
            notes,
            citations,
        };
    }

    if input.approved_hardship_adjustment {
        notes.push("DOB-approved § 28-320.7 financial hardship adjustment in effect — emissions-limit exposure modified.".to_string());
        return Output {
            severity: Severity::CompliantApprovedHardshipAdjustment,
            applicable_intensity_limit_kg_co2e_per_sqft_x_100: 0,
            annual_emissions_allowance_kg_co2e: 0,
            metric_tons_over_limit: 0,
            annual_penalty_cents: 0,
            notes,
            citations,
        };
    }

    if input.article_321_prescriptive_ecms_in_place {
        if input.rent_regulated_unit_percent <= ARTICLE_321_RENT_REGULATED_THRESHOLD_PERCENT {
            notes.push(format!(
                "Article 321 claimed but rent-regulated units {}% ≤ {}% threshold — not eligible.",
                input.rent_regulated_unit_percent,
                ARTICLE_321_RENT_REGULATED_THRESHOLD_PERCENT
            ));
            return Output {
                severity: Severity::ViolationArticle321ClaimedButRentRegulatedThresholdNotMet,
                applicable_intensity_limit_kg_co2e_per_sqft_x_100: 0,
                annual_emissions_allowance_kg_co2e: 0,
                metric_tons_over_limit: 0,
                annual_penalty_cents: 0,
                notes,
                citations,
            };
        }
        notes.push(format!(
            "Article 321 alternative compliance: {}% rent-regulated > {}% threshold; {} prescriptive ECMs in place.",
            input.rent_regulated_unit_percent,
            ARTICLE_321_RENT_REGULATED_THRESHOLD_PERCENT,
            ARTICLE_321_PRESCRIPTIVE_ECMS_COUNT
        ));
        return Output {
            severity: Severity::CompliantArticle321PrescriptiveEcmsRentRegulated,
            applicable_intensity_limit_kg_co2e_per_sqft_x_100: 0,
            annual_emissions_allowance_kg_co2e: 0,
            metric_tons_over_limit: 0,
            annual_penalty_cents: 0,
            notes,
            citations,
        };
    }

    let intensity_x_100 = compute_intensity_limit(input.occupancy_group, input.current_year);
    let allowance_kg = (input.gross_floor_area_sqft as u64)
        .saturating_mul(intensity_x_100 as u64)
        / 100;

    if input.annual_emissions_kg_co2e <= allowance_kg {
        notes.push(format!(
            "Annual emissions {} kgCO2e ≤ allowance {} kgCO2e (intensity {}.{:02} kgCO2e/sqft × {} sqft).",
            input.annual_emissions_kg_co2e,
            allowance_kg,
            intensity_x_100 / 100,
            intensity_x_100 % 100,
            input.gross_floor_area_sqft
        ));
        return Output {
            severity: Severity::CompliantEmissionsWithinIntensityLimit,
            applicable_intensity_limit_kg_co2e_per_sqft_x_100: intensity_x_100,
            annual_emissions_allowance_kg_co2e: allowance_kg,
            metric_tons_over_limit: 0,
            annual_penalty_cents: 0,
            notes,
            citations,
        };
    }

    let excess_kg = input.annual_emissions_kg_co2e.saturating_sub(allowance_kg);
    let excess_metric_tons = excess_kg / 1_000;
    let penalty = LL97_PENALTY_PER_METRIC_TON_CO2E_CENTS.saturating_mul(excess_metric_tons);

    notes.push(format!(
        "Emissions {} kgCO2e exceed allowance {} kgCO2e by {} metric tons; annual penalty ${}.",
        input.annual_emissions_kg_co2e,
        allowance_kg,
        excess_metric_tons,
        penalty / 100
    ));
    Output {
        severity: Severity::ViolationEmissionsExceedLimitPenaltyAccrues,
        applicable_intensity_limit_kg_co2e_per_sqft_x_100: intensity_x_100,
        annual_emissions_allowance_kg_co2e: allowance_kg,
        metric_tons_over_limit: excess_metric_tons,
        annual_penalty_cents: penalty,
        notes,
        citations,
    }
}

fn compute_intensity_limit(occupancy: OccupancyGroup, year: u32) -> u32 {
    let first_period = (LL97_FIRST_COMPLIANCE_PERIOD_START_YEAR
        ..=LL97_FIRST_COMPLIANCE_PERIOD_END_YEAR)
        .contains(&year);
    let second_period = (LL97_SECOND_COMPLIANCE_PERIOD_START_YEAR
        ..=LL97_SECOND_COMPLIANCE_PERIOD_END_YEAR)
        .contains(&year);
    match (occupancy, first_period, second_period) {
        (OccupancyGroup::R2Multifamily, true, _) => {
            LL97_R2_MULTIFAMILY_2024_2029_KG_CO2E_PER_SQFT_X_100
        }
        (OccupancyGroup::R2Multifamily, _, true) => {
            LL97_R2_MULTIFAMILY_2030_2034_KG_CO2E_PER_SQFT_X_100
        }
        (OccupancyGroup::BBusinessOffice, true, _) => {
            LL97_B_BUSINESS_2024_2029_KG_CO2E_PER_SQFT_X_100
        }
        (OccupancyGroup::BBusinessOffice, _, true) => {
            LL97_B_BUSINESS_2030_2034_KG_CO2E_PER_SQFT_X_100
        }
        _ => LL97_B_BUSINESS_2024_2029_KG_CO2E_PER_SQFT_X_100,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_r2_2026() -> Input {
        Input {
            occupancy_group: OccupancyGroup::R2Multifamily,
            gross_floor_area_sqft: 50_000,
            current_year: 2026,
            annual_emissions_kg_co2e: 300_000,
            rent_regulated_unit_percent: 0,
            article_321_prescriptive_ecms_in_place: false,
            approved_hardship_adjustment: false,
            approved_critical_facility_adjustment: false,
        }
    }

    #[test]
    fn r2_50000_sqft_2026_within_675_intensity_compliant() {
        let out = check(&base_r2_2026());
        assert_eq!(
            out.severity,
            Severity::CompliantEmissionsWithinIntensityLimit
        );
        assert_eq!(out.applicable_intensity_limit_kg_co2e_per_sqft_x_100, 675);
        assert_eq!(out.annual_emissions_allowance_kg_co2e, 337_500);
        assert_eq!(out.annual_penalty_cents, 0);
    }

    #[test]
    fn r2_50000_sqft_2026_above_675_intensity_violation_with_penalty() {
        let mut i = base_r2_2026();
        i.annual_emissions_kg_co2e = 500_000;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationEmissionsExceedLimitPenaltyAccrues
        );
        assert_eq!(out.metric_tons_over_limit, 162);
        assert_eq!(out.annual_penalty_cents, 4_341_600);
    }

    #[test]
    fn r2_50000_sqft_2030_tightens_to_407_intensity() {
        let mut i = base_r2_2026();
        i.current_year = 2030;
        let out = check(&i);
        assert_eq!(out.applicable_intensity_limit_kg_co2e_per_sqft_x_100, 407);
    }

    #[test]
    fn r2_2030_with_2024_compliant_emissions_now_violation() {
        let mut i = base_r2_2026();
        i.current_year = 2030;
        i.annual_emissions_kg_co2e = 300_000;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationEmissionsExceedLimitPenaltyAccrues
        );
        assert_eq!(out.metric_tons_over_limit, 96);
    }

    #[test]
    fn b_business_office_2024_846_intensity_higher_than_r2() {
        let mut i = base_r2_2026();
        i.occupancy_group = OccupancyGroup::BBusinessOffice;
        let out = check(&i);
        assert_eq!(out.applicable_intensity_limit_kg_co2e_per_sqft_x_100, 846);
    }

    #[test]
    fn b_business_office_2030_drops_to_453_intensity() {
        let mut i = base_r2_2026();
        i.occupancy_group = OccupancyGroup::BBusinessOffice;
        i.current_year = 2030;
        let out = check(&i);
        assert_eq!(out.applicable_intensity_limit_kg_co2e_per_sqft_x_100, 453);
    }

    #[test]
    fn building_under_25000_sqft_exempt() {
        let mut i = base_r2_2026();
        i.gross_floor_area_sqft = 25_000;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExemptBuildingUnder25000Sqft);
    }

    #[test]
    fn building_at_25001_sqft_just_over_threshold_not_exempt() {
        let mut i = base_r2_2026();
        i.gross_floor_area_sqft = 25_001;
        i.annual_emissions_kg_co2e = 100_000;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::CompliantEmissionsWithinIntensityLimit
        );
    }

    #[test]
    fn r3_one_or_two_family_exempt() {
        let mut i = base_r2_2026();
        i.occupancy_group = OccupancyGroup::R3OneOrTwoFamilyOrExempt;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExemptOccupancyNotCovered);
    }

    #[test]
    fn article_321_with_50_pct_rent_regulated_compliant() {
        let mut i = base_r2_2026();
        i.article_321_prescriptive_ecms_in_place = true;
        i.rent_regulated_unit_percent = 50;
        i.annual_emissions_kg_co2e = 1_000_000;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::CompliantArticle321PrescriptiveEcmsRentRegulated
        );
    }

    #[test]
    fn article_321_with_exactly_35_pct_rent_regulated_not_eligible() {
        let mut i = base_r2_2026();
        i.article_321_prescriptive_ecms_in_place = true;
        i.rent_regulated_unit_percent = 35;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationArticle321ClaimedButRentRegulatedThresholdNotMet
        );
    }

    #[test]
    fn article_321_with_36_pct_rent_regulated_eligible() {
        let mut i = base_r2_2026();
        i.article_321_prescriptive_ecms_in_place = true;
        i.rent_regulated_unit_percent = 36;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::CompliantArticle321PrescriptiveEcmsRentRegulated
        );
    }

    #[test]
    fn approved_hardship_adjustment_overrides_emissions_limit() {
        let mut i = base_r2_2026();
        i.approved_hardship_adjustment = true;
        i.annual_emissions_kg_co2e = 1_000_000;
        let out = check(&i);
        assert_eq!(out.severity, Severity::CompliantApprovedHardshipAdjustment);
    }

    #[test]
    fn approved_critical_facility_adjustment_overrides_emissions_limit() {
        let mut i = base_r2_2026();
        i.approved_critical_facility_adjustment = true;
        i.annual_emissions_kg_co2e = 1_000_000;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::CompliantApprovedCriticalFacilityAdjustment
        );
    }

    #[test]
    fn citations_pin_ll97_admin_code_28_320_subsections() {
        let out = check(&base_r2_2026());
        assert!(out.citations.iter().any(|c| c.contains("Local Law 97 of 2019")));
        assert!(out.citations.iter().any(|c| c.contains("§ 28-320.6")));
        assert!(out.citations.iter().any(|c| c.contains("§ 28-320.7")));
        assert!(out.citations.iter().any(|c| c.contains("§ 28-320.8")));
    }

    #[test]
    fn citations_pin_article_321_and_rcny_103_14() {
        let out = check(&base_r2_2026());
        assert!(out.citations.iter().any(|c| c.contains("Article 321")));
        assert!(out.citations.iter().any(|c| c.contains("§ 103-14")));
    }

    #[test]
    fn constant_pin_25000_sqft_threshold() {
        assert_eq!(LL97_BUILDING_SIZE_THRESHOLD_SQFT, 25_000);
    }

    #[test]
    fn constant_pin_268_per_ton_penalty() {
        assert_eq!(LL97_PENALTY_PER_METRIC_TON_CO2E_CENTS, 26_800);
    }

    #[test]
    fn constant_pin_r2_first_period_675() {
        assert_eq!(LL97_R2_MULTIFAMILY_2024_2029_KG_CO2E_PER_SQFT_X_100, 675);
    }

    #[test]
    fn constant_pin_r2_second_period_407() {
        assert_eq!(LL97_R2_MULTIFAMILY_2030_2034_KG_CO2E_PER_SQFT_X_100, 407);
    }

    #[test]
    fn constant_pin_b_business_first_period_846() {
        assert_eq!(LL97_B_BUSINESS_2024_2029_KG_CO2E_PER_SQFT_X_100, 846);
    }

    #[test]
    fn constant_pin_article_321_35_pct_threshold() {
        assert_eq!(ARTICLE_321_RENT_REGULATED_THRESHOLD_PERCENT, 35);
    }

    #[test]
    fn constant_pin_article_321_13_ecms_count() {
        assert_eq!(ARTICLE_321_PRESCRIPTIVE_ECMS_COUNT, 13);
    }

    #[test]
    fn constant_pin_2050_final_target_year() {
        assert_eq!(LL97_FINAL_TARGET_YEAR, 2050);
    }

    #[test]
    fn constant_pin_80_pct_final_reduction() {
        assert_eq!(LL97_FINAL_REDUCTION_TARGET_PERCENT, 80);
    }

    #[test]
    fn very_large_excess_emissions_saturating_no_overflow() {
        let mut i = base_r2_2026();
        i.annual_emissions_kg_co2e = u64::MAX;
        let out = check(&i);
        assert!(out.metric_tons_over_limit > 0);
        assert!(out.annual_penalty_cents > 0);
    }
}
