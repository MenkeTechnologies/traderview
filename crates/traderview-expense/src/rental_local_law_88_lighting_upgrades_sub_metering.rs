//! NYC Local Law 88 of 2009 Lighting Upgrades & Sub-Metering
//! compliance for trader-landlords with NYC commercial or mixed-use
//! inventory.
//!
//! Local Law 88 of 2009 is part of the **Greener, Greater Buildings
//! Plan (GGBP)** along with Local Law 84 (annual benchmarking),
//! Local Law 87 (energy audits), and Local Law 97 (emissions cap).
//! Enacted December 28, 2009. Codified at NYC Admin Code § 28-310
//! (lighting upgrades) and § 28-311 (sub-metering).
//!
//! **Two distinct compliance vectors**:
//!
//! 1. **§ 28-310 Lighting Upgrades** — lighting systems in covered
//!    buildings must comply with current NYC Energy Conservation
//!    Code (NYCECC) standards for new systems by **January 1, 2025**.
//!
//! 2. **§ 28-311 Electrical Sub-Metering** — electrical sub-meters
//!    must be installed in each covered tenant space by January 1,
//!    2025; building owners must provide monthly billing statements
//!    to tenants showing electrical consumption and charges.
//!
//! **Coverage**: originally buildings > 50,000 sqft; subsequently
//! amended to include buildings **25,000+ square feet**. Sub-
//! metering specifically applies to non-residential tenant spaces
//! **greater than 5,000 gross square feet**, OR floors > 5,000
//! sqft with two or more tenants.
//!
//! **Exemptions**: occupancy group R-2 (multifamily residential)
//! and R-3 (one- and two-family) dwelling units; tenant spaces
//! where electrical consumption is already measured by a dedicated
//! utility meter.
//!
//! **Monthly tenant billing statements** (§ 28-311.4): building
//! owners must provide tenants with monthly statements showing
//! electrical consumption measured by the submeter and the
//! electrical charges billed. For shared submeters covering
//! multiple tenants, statements must include total consumption for
//! the metered area and each tenant's percentage of that area.
//!
//! **Filing requirements** (§ 28-311.4): owners must file lighting
//! upgrade and submeter installation compliance reports with NYC
//! DOB by **May 1, 2025** along with a **$115 filing fee** (covers
//! both lighting and submetering reports). Reports uploaded via
//! BEAM (Building Energy Audit Management) system.
//!
//! **Penalty schedule** (NYC Admin Code § 28-311.5):
//!
//! - **$500** per covered tenant space without a sub-meter,
//!   assessed annually until all required submeters are installed.
//! - **$1,500** for failure to file lighting upgrade report,
//!   assessed annually until compliance.
//! - **$1,500** for failure to file submeter installation report,
//!   assessed annually until compliance.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[allow(dead_code)]
pub const LL88_ENACTMENT_YEAR: u32 = 2009;
#[allow(dead_code)]
pub const LL88_BUILDING_SIZE_THRESHOLD_SQFT: u32 = 25_000;
#[allow(dead_code)]
pub const LL88_TENANT_SPACE_THRESHOLD_SQFT: u32 = 5_000;
#[allow(dead_code)]
pub const LL88_COMPLIANCE_DEADLINE_YEAR: u32 = 2025;
#[allow(dead_code)]
pub const LL88_COMPLIANCE_DEADLINE_MONTH: u32 = 1;
#[allow(dead_code)]
pub const LL88_COMPLIANCE_DEADLINE_DAY: u32 = 1;
#[allow(dead_code)]
pub const LL88_REPORT_FILING_DEADLINE_MONTH: u32 = 5;
#[allow(dead_code)]
pub const LL88_REPORT_FILING_DEADLINE_DAY: u32 = 1;
#[allow(dead_code)]
pub const LL88_FILING_FEE_CENTS: u64 = 11_500;
#[allow(dead_code)]
pub const LL88_NO_SUBMETER_PENALTY_CENTS_PER_TENANT_SPACE_PER_YEAR: u64 = 50_000;
#[allow(dead_code)]
pub const LL88_REPORT_NOT_FILED_PENALTY_CENTS_PER_REPORT_PER_YEAR: u64 = 150_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OccupancyGroup {
    R2MultifamilyResidential,
    R3OneOrTwoFamily,
    Commercial,
    MixedUse,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    ExemptBuildingUnder25000Sqft,
    ExemptR2OrR3OccupancyOnly,
    CompliantLightingAndSubMeteringComplete,
    ViolationLightingNotUpgradedByJan1_2025,
    ViolationSubMeterNotInstalledByJan1_2025,
    ViolationLightingReportNotFiledByMay1_2025,
    ViolationSubMeteringReportNotFiledByMay1_2025,
    ViolationMonthlyTenantBillingNotProvided,
    AggravatedViolationMultipleCompoundingFailures,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub gross_floor_area_sqft: u32,
    pub building_occupancy_group: OccupancyGroup,
    pub has_commercial_tenant_spaces_over_5000_sqft: bool,
    pub count_of_qualifying_tenant_spaces_without_submeter: u32,
    pub lighting_upgraded_by_jan_1_2025: bool,
    pub sub_meters_installed_by_jan_1_2025: bool,
    pub lighting_report_filed_by_may_1_2025: bool,
    pub sub_metering_report_filed_by_may_1_2025: bool,
    pub monthly_tenant_billing_provided: bool,
    pub current_year: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Output {
    pub severity: Severity,
    pub compliant: bool,
    pub total_annual_penalty_cents: u64,
    pub filing_fee_required_cents: u64,
    pub notes: Vec<String>,
    pub citations: Vec<String>,
}

pub type LocalLaw88LightingUpgradesSubMeteringInput = Input;
pub type LocalLaw88LightingUpgradesSubMeteringOutput = Output;
pub type LocalLaw88LightingUpgradesSubMeteringResult = Output;

pub fn check(input: &Input) -> Output {
    let mut notes: Vec<String> = Vec::new();
    let citations: Vec<String> = vec![
        "NYC Local Law 88 of 2009 (Lighting Upgrades & Sub-Metering)".to_string(),
        "NYC Admin Code § 28-310 (lighting upgrades)".to_string(),
        "NYC Admin Code § 28-311 (sub-metering)".to_string(),
        "NYC Admin Code § 28-311.4 (monthly tenant billing + reporting)".to_string(),
        "NYC Admin Code § 28-311.5 (penalty schedule)".to_string(),
        "NYC Energy Conservation Code (NYCECC) — current standards for new lighting systems"
            .to_string(),
        "Greener, Greater Buildings Plan (GGBP)".to_string(),
        "NYC DOB BEAM (Building Energy Audit Management) system".to_string(),
        "NYC DOB Local Law 88 User Guide (Sub-Metering Article 311)".to_string(),
    ];

    if input.gross_floor_area_sqft < LL88_BUILDING_SIZE_THRESHOLD_SQFT {
        notes.push(format!(
            "Building gross floor area {} sqft < {} sqft threshold — exempt from LL 88.",
            input.gross_floor_area_sqft, LL88_BUILDING_SIZE_THRESHOLD_SQFT
        ));
        return Output {
            severity: Severity::ExemptBuildingUnder25000Sqft,
            compliant: true,
            total_annual_penalty_cents: 0,
            filing_fee_required_cents: 0,
            notes,
            citations,
        };
    }

    if matches!(
        input.building_occupancy_group,
        OccupancyGroup::R2MultifamilyResidential | OccupancyGroup::R3OneOrTwoFamily
    ) && !input.has_commercial_tenant_spaces_over_5000_sqft
    {
        notes.push("Occupancy group R-2 (multifamily) or R-3 (1-2 family) with no qualifying commercial tenant spaces — exempt from LL 88 sub-metering requirement.".to_string());
        return Output {
            severity: Severity::ExemptR2OrR3OccupancyOnly,
            compliant: true,
            total_annual_penalty_cents: 0,
            filing_fee_required_cents: 0,
            notes,
            citations,
        };
    }

    let mut violations_count: u32 = 0;
    let mut annual_penalty: u64 = 0;

    if !input.lighting_upgraded_by_jan_1_2025 && input.current_year >= LL88_COMPLIANCE_DEADLINE_YEAR
    {
        violations_count += 1;
        annual_penalty =
            annual_penalty.saturating_add(LL88_REPORT_NOT_FILED_PENALTY_CENTS_PER_REPORT_PER_YEAR);
        notes.push(format!(
            "Lighting upgrade not completed by January 1, {} deadline — § 28-310 violation.",
            LL88_COMPLIANCE_DEADLINE_YEAR
        ));
    }

    if !input.sub_meters_installed_by_jan_1_2025
        && input.has_commercial_tenant_spaces_over_5000_sqft
        && input.current_year >= LL88_COMPLIANCE_DEADLINE_YEAR
    {
        violations_count += 1;
        annual_penalty = annual_penalty.saturating_add(
            LL88_NO_SUBMETER_PENALTY_CENTS_PER_TENANT_SPACE_PER_YEAR
                .saturating_mul(input.count_of_qualifying_tenant_spaces_without_submeter as u64),
        );
        notes.push(format!(
            "Sub-meters not installed in {} qualifying commercial tenant spaces > {} sqft by January 1, {} — § 28-311 violation; $500/space/year.",
            input.count_of_qualifying_tenant_spaces_without_submeter,
            LL88_TENANT_SPACE_THRESHOLD_SQFT,
            LL88_COMPLIANCE_DEADLINE_YEAR
        ));
    }

    if !input.lighting_report_filed_by_may_1_2025
        && input.current_year >= LL88_COMPLIANCE_DEADLINE_YEAR
    {
        violations_count += 1;
        annual_penalty =
            annual_penalty.saturating_add(LL88_REPORT_NOT_FILED_PENALTY_CENTS_PER_REPORT_PER_YEAR);
        notes.push(format!(
            "Lighting upgrade compliance report not filed by May 1, {} — $1,500/year penalty.",
            LL88_COMPLIANCE_DEADLINE_YEAR
        ));
    }

    if !input.sub_metering_report_filed_by_may_1_2025
        && input.has_commercial_tenant_spaces_over_5000_sqft
        && input.current_year >= LL88_COMPLIANCE_DEADLINE_YEAR
    {
        violations_count += 1;
        annual_penalty =
            annual_penalty.saturating_add(LL88_REPORT_NOT_FILED_PENALTY_CENTS_PER_REPORT_PER_YEAR);
        notes.push(format!(
            "Sub-metering installation compliance report not filed by May 1, {} — $1,500/year penalty.",
            LL88_COMPLIANCE_DEADLINE_YEAR
        ));
    }

    if !input.monthly_tenant_billing_provided
        && input.has_commercial_tenant_spaces_over_5000_sqft
        && input.sub_meters_installed_by_jan_1_2025
    {
        violations_count += 1;
        notes.push("Monthly tenant billing statements not provided — § 28-311.4 violation; tenants entitled to monthly consumption and charge breakdown.".to_string());
    }

    if violations_count == 0 {
        notes.push(format!(
            "Full LL 88 compliance: lighting upgraded to NYCECC standards + sub-meters installed (where required) + reports filed by May 1, {} + monthly tenant billing provided (where required). Filing fee ${}.",
            LL88_COMPLIANCE_DEADLINE_YEAR,
            LL88_FILING_FEE_CENTS / 100
        ));
        return Output {
            severity: Severity::CompliantLightingAndSubMeteringComplete,
            compliant: true,
            total_annual_penalty_cents: 0,
            filing_fee_required_cents: LL88_FILING_FEE_CENTS,
            notes,
            citations,
        };
    }

    if violations_count >= 2 {
        return Output {
            severity: Severity::AggravatedViolationMultipleCompoundingFailures,
            compliant: false,
            total_annual_penalty_cents: annual_penalty,
            filing_fee_required_cents: LL88_FILING_FEE_CENTS,
            notes,
            citations,
        };
    }

    let severity = if !input.lighting_upgraded_by_jan_1_2025 {
        Severity::ViolationLightingNotUpgradedByJan1_2025
    } else if !input.sub_meters_installed_by_jan_1_2025 {
        Severity::ViolationSubMeterNotInstalledByJan1_2025
    } else if !input.lighting_report_filed_by_may_1_2025 {
        Severity::ViolationLightingReportNotFiledByMay1_2025
    } else if !input.sub_metering_report_filed_by_may_1_2025 {
        Severity::ViolationSubMeteringReportNotFiledByMay1_2025
    } else {
        Severity::ViolationMonthlyTenantBillingNotProvided
    };

    Output {
        severity,
        compliant: false,
        total_annual_penalty_cents: annual_penalty,
        filing_fee_required_cents: LL88_FILING_FEE_CENTS,
        notes,
        citations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_compliant() -> Input {
        Input {
            gross_floor_area_sqft: 100_000,
            building_occupancy_group: OccupancyGroup::Commercial,
            has_commercial_tenant_spaces_over_5000_sqft: true,
            count_of_qualifying_tenant_spaces_without_submeter: 0,
            lighting_upgraded_by_jan_1_2025: true,
            sub_meters_installed_by_jan_1_2025: true,
            lighting_report_filed_by_may_1_2025: true,
            sub_metering_report_filed_by_may_1_2025: true,
            monthly_tenant_billing_provided: true,
            current_year: 2026,
        }
    }

    #[test]
    fn fully_compliant_baseline() {
        let out = check(&base_compliant());
        assert_eq!(
            out.severity,
            Severity::CompliantLightingAndSubMeteringComplete
        );
        assert!(out.compliant);
        assert_eq!(out.filing_fee_required_cents, 11_500);
    }

    #[test]
    fn building_under_25000_sqft_exempt() {
        let mut i = base_compliant();
        i.gross_floor_area_sqft = 24_999;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExemptBuildingUnder25000Sqft);
    }

    #[test]
    fn building_at_exactly_25000_sqft_covered() {
        let mut i = base_compliant();
        i.gross_floor_area_sqft = 25_000;
        let out = check(&i);
        assert!(out.compliant);
    }

    #[test]
    fn r2_multifamily_without_commercial_tenants_exempt() {
        let mut i = base_compliant();
        i.building_occupancy_group = OccupancyGroup::R2MultifamilyResidential;
        i.has_commercial_tenant_spaces_over_5000_sqft = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExemptR2OrR3OccupancyOnly);
    }

    #[test]
    fn r3_one_two_family_exempt() {
        let mut i = base_compliant();
        i.building_occupancy_group = OccupancyGroup::R3OneOrTwoFamily;
        i.has_commercial_tenant_spaces_over_5000_sqft = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExemptR2OrR3OccupancyOnly);
    }

    #[test]
    fn mixed_use_r2_with_commercial_tenants_covered() {
        let mut i = base_compliant();
        i.building_occupancy_group = OccupancyGroup::R2MultifamilyResidential;
        i.has_commercial_tenant_spaces_over_5000_sqft = true;
        let out = check(&i);
        assert!(out.compliant);
    }

    #[test]
    fn lighting_not_upgraded_violation() {
        let mut i = base_compliant();
        i.lighting_upgraded_by_jan_1_2025 = false;
        i.has_commercial_tenant_spaces_over_5000_sqft = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationLightingNotUpgradedByJan1_2025
        );
    }

    #[test]
    fn sub_meter_not_installed_3_spaces_1500_penalty() {
        let mut i = base_compliant();
        i.sub_meters_installed_by_jan_1_2025 = false;
        i.count_of_qualifying_tenant_spaces_without_submeter = 3;
        i.monthly_tenant_billing_provided = false;
        let out = check(&i);
        assert_eq!(out.total_annual_penalty_cents, 150_000);
    }

    #[test]
    fn sub_meter_not_installed_10_spaces_5000_penalty() {
        let mut i = base_compliant();
        i.sub_meters_installed_by_jan_1_2025 = false;
        i.count_of_qualifying_tenant_spaces_without_submeter = 10;
        i.monthly_tenant_billing_provided = false;
        let out = check(&i);
        assert_eq!(out.total_annual_penalty_cents, 500_000);
    }

    #[test]
    fn lighting_report_not_filed_1500_penalty() {
        let mut i = base_compliant();
        i.lighting_report_filed_by_may_1_2025 = false;
        i.has_commercial_tenant_spaces_over_5000_sqft = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationLightingReportNotFiledByMay1_2025
        );
        assert_eq!(out.total_annual_penalty_cents, 150_000);
    }

    #[test]
    fn sub_metering_report_not_filed_1500_penalty() {
        let mut i = base_compliant();
        i.sub_metering_report_filed_by_may_1_2025 = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationSubMeteringReportNotFiledByMay1_2025
        );
        assert_eq!(out.total_annual_penalty_cents, 150_000);
    }

    #[test]
    fn monthly_tenant_billing_not_provided_violation() {
        let mut i = base_compliant();
        i.monthly_tenant_billing_provided = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationMonthlyTenantBillingNotProvided
        );
    }

    #[test]
    fn aggravated_multiple_compounding_failures() {
        let mut i = base_compliant();
        i.lighting_upgraded_by_jan_1_2025 = false;
        i.sub_meters_installed_by_jan_1_2025 = false;
        i.count_of_qualifying_tenant_spaces_without_submeter = 5;
        i.lighting_report_filed_by_may_1_2025 = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::AggravatedViolationMultipleCompoundingFailures
        );
        assert_eq!(out.total_annual_penalty_cents, 550_000);
    }

    #[test]
    fn pre_2025_compliance_year_no_penalty_assessed() {
        let mut i = base_compliant();
        i.current_year = 2024;
        i.lighting_upgraded_by_jan_1_2025 = false;
        i.sub_meters_installed_by_jan_1_2025 = false;
        let out = check(&i);
        assert_eq!(out.total_annual_penalty_cents, 0);
    }

    #[test]
    fn citations_pin_ll88_admin_code_28_310_311() {
        let out = check(&base_compliant());
        assert!(out
            .citations
            .iter()
            .any(|c| c.contains("Local Law 88 of 2009")));
        assert!(out.citations.iter().any(|c| c.contains("§ 28-310")));
        assert!(out.citations.iter().any(|c| c.contains("§ 28-311")));
        assert!(out.citations.iter().any(|c| c.contains("§ 28-311.4")));
        assert!(out.citations.iter().any(|c| c.contains("§ 28-311.5")));
    }

    #[test]
    fn citations_pin_nycecc_ggbp_beam() {
        let out = check(&base_compliant());
        assert!(out.citations.iter().any(|c| c.contains("NYCECC")));
        assert!(out
            .citations
            .iter()
            .any(|c| c.contains("Greener, Greater Buildings Plan")));
        assert!(out.citations.iter().any(|c| c.contains("BEAM")));
    }

    #[test]
    fn constant_pin_25000_building_threshold() {
        assert_eq!(LL88_BUILDING_SIZE_THRESHOLD_SQFT, 25_000);
    }

    #[test]
    fn constant_pin_5000_tenant_space_threshold() {
        assert_eq!(LL88_TENANT_SPACE_THRESHOLD_SQFT, 5_000);
    }

    #[test]
    fn constant_pin_115_filing_fee() {
        assert_eq!(LL88_FILING_FEE_CENTS, 11_500);
    }

    #[test]
    fn constant_pin_500_per_tenant_space_penalty() {
        assert_eq!(
            LL88_NO_SUBMETER_PENALTY_CENTS_PER_TENANT_SPACE_PER_YEAR,
            50_000
        );
    }

    #[test]
    fn constant_pin_1500_per_report_penalty() {
        assert_eq!(
            LL88_REPORT_NOT_FILED_PENALTY_CENTS_PER_REPORT_PER_YEAR,
            150_000
        );
    }

    #[test]
    fn constant_pin_jan_1_2025_deadline_and_may_1_filing() {
        assert_eq!(LL88_COMPLIANCE_DEADLINE_YEAR, 2025);
        assert_eq!(LL88_COMPLIANCE_DEADLINE_MONTH, 1);
        assert_eq!(LL88_COMPLIANCE_DEADLINE_DAY, 1);
        assert_eq!(LL88_REPORT_FILING_DEADLINE_MONTH, 5);
        assert_eq!(LL88_REPORT_FILING_DEADLINE_DAY, 1);
    }

    #[test]
    fn very_large_tenant_space_count_no_overflow() {
        let mut i = base_compliant();
        i.sub_meters_installed_by_jan_1_2025 = false;
        i.count_of_qualifying_tenant_spaces_without_submeter = u32::MAX;
        i.monthly_tenant_billing_provided = false;
        let out = check(&i);
        assert!(out.total_annual_penalty_cents > 0);
    }
}
