//! Oregon SB 608 of 2019 + SB 611 of 2023 Statewide Rent
//! Stabilization Compliance Module.
//!
//! Pure-compute check for landlord compliance with Oregon's
//! statewide rent stabilization law — the **FIRST statewide
//! rent control law in U.S. history**, predating Washington
//! HB 1217 of 2025 by six years. Oregon SB 608 of 2019 (signed
//! by Governor Kate Brown February 28, 2019) was amended by
//! Oregon SB 611 of 2023 (signed by Governor Tina Kotek
//! July 6, 2023) which lowered the general residential cap
//! and the manufactured home park cap. Codified at ORS 90.323
//! (general residential), ORS 90.600 (manufactured home
//! parks), ORS 90.427 (just cause termination).
//!
//! Web research (verified 2026-06-03):
//! - **Oregon SB 608 of 2019** signed by Governor Kate Brown
//!   on **February 28, 2019**; effective immediately. FIRST
//!   statewide rent control law in U.S. history. Amended ORS
//!   90.100, 90.220, 90.323, 90.427, 90.600. Original general
//!   residential cap: 7 % + CPI annually with no absolute
//!   ceiling ([Oregon Legislative Assembly SB 608 Enrolled](https://olis.oregonlegislature.gov/liz/2019R1/Downloads/MeasureDocument/SB608/Enrolled);
//!   [FAQ on Oregon's Rent Control Laws — League of Oregon
//!   Cities](https://www.orcities.org/application/files/9816/9265/7807/FAQ-Oregons-Rent-Control-Laws-Updated8-21-23.pdf)).
//! - **Oregon SB 611 of 2023** signed by Governor Tina Kotek
//!   on **July 6, 2023**; effective immediately. Lowered
//!   general residential cap to **7 % + CPI BUT NO GREATER
//!   THAN 10 %** (whichever is less). Lowered manufactured
//!   home park cap to **6 % maximum** ([Rental Housing Journal
//!   — Local Exceptions Complicate Oregon's Post-SB 611 Rent
//!   Regulations](https://rentalhousingjournal.com/local-exceptions-complicate-oregons-post-sb-611-rent-regulations/)).
//! - **Manufactured Home Park Cap (post-SB 611)**: 6 %
//!   maximum annual rent increase; parks/marinas with **30
//!   OR FEWER spaces are EXEMPT from the cap** under ORS
//!   90.600 amendments.
//! - **First-Year Tenancy Rent Freeze**: landlord may NOT
//!   increase rent during the first year of tenancy. "First
//!   year of occupancy" includes all periods in which ANY
//!   tenant has resided in the unit.
//! - **90-Day Written Notice**: minimum 90-day written notice
//!   before rent increase takes effect (general residential);
//!   7-day notice for week-to-week tenancies.
//! - **15-Year New Construction Exemption**: rent control does
//!   NOT apply to any rental unit when the first certificate
//!   of occupancy for the unit was issued LESS THAN 15 YEARS
//!   from the date of the notice of the rent increase.
//! - **Government Subsidy Exemption**: rent control does NOT
//!   apply where landlord is providing reduced rent to tenant
//!   as part of federal, state, or local program or subsidy.
//! - **Just Cause Termination (ORS 90.427)**: after first year
//!   of tenancy, landlord may only terminate for cause; 90-day
//!   notice required for no-cause termination during first
//!   year; landlord termination for cause during first year
//!   limits next tenant's rent to 7 % + CPI of prior rent.
//! - **DAS Annual Notice**: Oregon Department of Administrative
//!   Services (DAS) publishes the maximum rent increase
//!   percentage by **September 30 each year** based on prior
//!   12-month CPI (West Region all items All Urban Consumers
//!   index per BLS).
//! - **Local Preemption Exception**: Portland City Code 30.01.085
//!   imposes additional 90-day notice for no-cause terminations
//!   even after first year; Milwaukie has parallel ordinance.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const OR_SB_608_SIGNED_DATE_YEAR: u32 = 2019;
pub const OR_SB_608_SIGNED_DATE_MONTH: u32 = 2;
pub const OR_SB_608_SIGNED_DATE_DAY: u32 = 28;
pub const OR_SB_611_SIGNED_DATE_YEAR: u32 = 2023;
pub const OR_SB_611_SIGNED_DATE_MONTH: u32 = 7;
pub const OR_SB_611_SIGNED_DATE_DAY: u32 = 6;
pub const OR_GENERAL_RESIDENTIAL_CAP_BASE_BASIS_POINTS: u64 = 700;
pub const OR_GENERAL_RESIDENTIAL_CAP_CEILING_BASIS_POINTS: u64 = 1_000;
pub const OR_MANUFACTURED_HOME_PARK_CAP_BASIS_POINTS: u64 = 600;
pub const OR_MANUFACTURED_HOME_PARK_EXEMPT_SPACES_THRESHOLD: u32 = 30;
pub const OR_NEW_CONSTRUCTION_EXEMPTION_YEARS: u32 = 15;
pub const OR_NOTICE_DAYS_REQUIRED_GENERAL: u32 = 90;
pub const OR_NOTICE_DAYS_REQUIRED_WEEK_TO_WEEK: u32 = 7;
pub const OR_FIRST_YEAR_TENANCY_MONTHS_FREEZE: u32 = 12;
pub const OR_DAS_ANNUAL_NOTICE_DEADLINE_MONTH: u32 = 9;
pub const OR_DAS_ANNUAL_NOTICE_DEADLINE_DAY: u32 = 30;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyClassification {
    StandardResidentialUnderOrs90_323,
    ManufacturedOrFloatingHomeParkUnderOrs90_600,
    ManufacturedHomeParkWith30OrFewerSpacesExempt,
    NewConstructionWithin15YearsOfFirstCertificateOfOccupancy,
    GovernmentSubsidizedReducedRentExempt,
    PropertyOutsideOregon,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TenancyStatus {
    FirstYearTenancyAnyLeaseType,
    PostFirstYearMonthToMonth,
    PostFirstYearFixedTerm,
    WeekToWeekTenancy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NoticeProvided {
    NoticeProvidedAtOrAbove90DaysGeneral,
    NoticeProvidedBetween7And89DaysGeneral,
    NoticeProvidedAtOrAbove7DaysWeekToWeek,
    NoticeProvidedBelow7Days,
    NoNoticeProvided,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OregonRentMode {
    NotApplicablePropertyOutsideOregon,
    NotApplicableNewConstructionWithin15YearsExemption,
    NotApplicableGovernmentSubsidizedReducedRentExempt,
    NotApplicableManufacturedHomeParkWith30OrFewerSpacesExempt,
    CompliantGeneralResidentialCap7PctPlusCpiUnder10PctCeiling,
    CompliantGeneralResidentialAt10PctCeilingExactlyAllowed,
    CompliantManufacturedHomePark6PctCap,
    Compliant90DayNoticeProvidedGeneralResidential,
    Compliant7DayNoticeProvidedWeekToWeek,
    ViolationFirstYearTenancyRentIncreaseAttempted,
    ViolationGeneralResidentialExceeds7PctPlusCpiOr10PctCap,
    ViolationGeneralResidentialExceeds10PctCeiling,
    ViolationManufacturedHomePark6PctCapExceeded,
    ViolationNoticeBelow90DayMinimumGeneralResidential,
    ViolationWeekToWeekNoticeBelow7DayMinimum,
    ViolationNoNoticeProvided,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub property_classification: PropertyClassification,
    pub tenancy_status: TenancyStatus,
    pub notice_provided: NoticeProvided,
    pub rent_increase_basis_points: u64,
    pub current_cpi_basis_points: u64,
    pub years_since_first_certificate_of_occupancy: u32,
    pub manufactured_home_park_total_spaces: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: OregonRentMode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
    pub allowed_cap_basis_points: u64,
}

pub type RentalOregonSb608Sb611RentStabilizationInput = Input;
pub type RentalOregonSb608Sb611RentStabilizationOutput = Output;
pub type RentalOregonSb608Sb611RentStabilizationResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "Oregon SB 608 of 2019 — FIRST statewide rent control law in U.S. history; signed by Governor Kate Brown on February 28, 2019; effective immediately; amended ORS 90.100, 90.220, 90.323, 90.427, 90.600".to_string(),
        "Oregon SB 611 of 2023 — signed by Governor Tina Kotek on July 6, 2023; effective immediately; lowered general residential cap to 7 % + CPI but no greater than 10 %; lowered manufactured home park cap to 6 %".to_string(),
        "ORS 90.323 — general residential rent cap; 7 % + CPI (West Region all-items All Urban Consumers per BLS) but no greater than 10 %, whichever LESS".to_string(),
        "ORS 90.600 — manufactured home and floating home park rent cap; 6 % maximum annual percentage rent increase post-SB 611; parks/marinas with 30 or fewer spaces EXEMPT from cap".to_string(),
        "ORS 90.427 — just cause termination after first year of tenancy; 90-day notice required for no-cause termination during first year; landlord termination for cause during first year limits next tenant's rent to 7 % + CPI of prior rent".to_string(),
        "First-Year Tenancy Rent Freeze — landlord may NOT raise rent during first year of tenancy; 'first year of occupancy' includes all periods in which ANY tenant has resided in unit".to_string(),
        "90-Day Written Notice — minimum 90-day written notice before rent increase takes effect (general residential); 7-day notice for week-to-week tenancies".to_string(),
        "15-Year New Construction Exemption — rent control does NOT apply to any rental unit when first certificate of occupancy was issued less than 15 years from date of notice of rent increase".to_string(),
        "Government Subsidy Exemption — rent control does NOT apply where landlord providing reduced rent to tenant as part of federal/state/local program or subsidy".to_string(),
        "DAS Annual Notice — Oregon Department of Administrative Services publishes maximum rent increase percentage by September 30 each year based on prior 12-month West Region CPI".to_string(),
        "Portland City Code 30.01.085 + Milwaukie ordinance — local 90-day notice for no-cause terminations even after first year (additive to SB 608 baseline)".to_string(),
        "Oregon League of Cities — FAQ on Oregon's Rent Control Laws comprehensive guide".to_string(),
        "Rental Housing Journal — Local Exceptions Complicate Oregon's Post-SB 611 Rent Regulations".to_string(),
        "Manufactured Housing Communities of Oregon — MHCO rent increase guidance".to_string(),
    ];

    let general_cpi_adjusted_cap_basis_points = OR_GENERAL_RESIDENTIAL_CAP_BASE_BASIS_POINTS
        .saturating_add(input.current_cpi_basis_points)
        .min(OR_GENERAL_RESIDENTIAL_CAP_CEILING_BASIS_POINTS);

    if input.property_classification == PropertyClassification::PropertyOutsideOregon {
        return Output {
            mode: OregonRentMode::NotApplicablePropertyOutsideOregon,
            statutory_basis: "Property outside Oregon; ORS 90.323 / 90.600 inapplicable".to_string(),
            notes: "Property outside Oregon; Oregon SB 608 / SB 611 statewide rent stabilization inapplicable.".to_string(),
            citations,
            allowed_cap_basis_points: 0,
        };
    }

    if input.property_classification
        == PropertyClassification::NewConstructionWithin15YearsOfFirstCertificateOfOccupancy
        && input.years_since_first_certificate_of_occupancy < OR_NEW_CONSTRUCTION_EXEMPTION_YEARS
    {
        return Output {
            mode: OregonRentMode::NotApplicableNewConstructionWithin15YearsExemption,
            statutory_basis: "ORS 90.323 — 15-year new construction exemption".to_string(),
            notes: format!(
                "NOT APPLICABLE: new construction exemption applies; {} years since first certificate of occupancy < 15-year statutory exemption window.",
                input.years_since_first_certificate_of_occupancy
            ),
            citations,
            allowed_cap_basis_points: u64::MAX,
        };
    }

    if input.property_classification
        == PropertyClassification::GovernmentSubsidizedReducedRentExempt
    {
        return Output {
            mode: OregonRentMode::NotApplicableGovernmentSubsidizedReducedRentExempt,
            statutory_basis: "ORS 90.323 — government subsidy exemption".to_string(),
            notes: "NOT APPLICABLE: landlord providing reduced rent as part of federal, state, or local program or subsidy; rent control inapplicable.".to_string(),
            citations,
            allowed_cap_basis_points: u64::MAX,
        };
    }

    if input.property_classification
        == PropertyClassification::ManufacturedHomeParkWith30OrFewerSpacesExempt
        && input.manufactured_home_park_total_spaces <= OR_MANUFACTURED_HOME_PARK_EXEMPT_SPACES_THRESHOLD
    {
        return Output {
            mode: OregonRentMode::NotApplicableManufacturedHomeParkWith30OrFewerSpacesExempt,
            statutory_basis: "ORS 90.600 (post-SB 611) — manufactured home park with 30 or fewer spaces exempt".to_string(),
            notes: format!(
                "NOT APPLICABLE: manufactured home park with {} spaces (≤ 30 statutory threshold) exempt from 6 % cap.",
                input.manufactured_home_park_total_spaces
            ),
            citations,
            allowed_cap_basis_points: u64::MAX,
        };
    }

    if input.tenancy_status == TenancyStatus::FirstYearTenancyAnyLeaseType
        && input.rent_increase_basis_points > 0
    {
        return Output {
            mode: OregonRentMode::ViolationFirstYearTenancyRentIncreaseAttempted,
            statutory_basis: "Oregon SB 608 first-year tenancy rent freeze — no rent increase permitted during first 12 months".to_string(),
            notes: format!(
                "VIOLATION: rent increase of {} basis points attempted during first 12 months of tenancy; Oregon SB 608 first-year rent freeze prohibits ANY rent increase regardless of lease type; 'first year of occupancy' includes all periods any tenant resided in unit.",
                input.rent_increase_basis_points
            ),
            citations,
            allowed_cap_basis_points: 0,
        };
    }

    if input.notice_provided == NoticeProvided::NoNoticeProvided && input.rent_increase_basis_points > 0
    {
        return Output {
            mode: OregonRentMode::ViolationNoNoticeProvided,
            statutory_basis: "Oregon SB 608 — no written notice provided".to_string(),
            notes: "VIOLATION: rent increase attempted with no written notice provided; Oregon SB 608 requires 90-day notice (general residential) or 7-day notice (week-to-week).".to_string(),
            citations,
            allowed_cap_basis_points: 0,
        };
    }

    if input.tenancy_status == TenancyStatus::WeekToWeekTenancy {
        if matches!(
            input.notice_provided,
            NoticeProvided::NoticeProvidedBelow7Days
        ) && input.rent_increase_basis_points > 0
        {
            return Output {
                mode: OregonRentMode::ViolationWeekToWeekNoticeBelow7DayMinimum,
                statutory_basis: "Oregon SB 608 — week-to-week 7-day notice minimum".to_string(),
                notes: "VIOLATION: week-to-week rent increase notice below 7-day statutory minimum.".to_string(),
                citations,
                allowed_cap_basis_points: 0,
            };
        }
    } else if matches!(
        input.notice_provided,
        NoticeProvided::NoticeProvidedBetween7And89DaysGeneral
            | NoticeProvided::NoticeProvidedBelow7Days
    ) && input.rent_increase_basis_points > 0
    {
        return Output {
            mode: OregonRentMode::ViolationNoticeBelow90DayMinimumGeneralResidential,
            statutory_basis: "Oregon SB 608 — 90-day written notice requirement for general residential".to_string(),
            notes: format!(
                "VIOLATION: rent increase notice provided below 90-day statutory minimum for general residential tenancy; provided notice classification: {:?}.",
                input.notice_provided
            ),
            citations,
            allowed_cap_basis_points: 0,
        };
    }

    if input.property_classification
        == PropertyClassification::ManufacturedOrFloatingHomeParkUnderOrs90_600
    {
        if input.rent_increase_basis_points > OR_MANUFACTURED_HOME_PARK_CAP_BASIS_POINTS {
            return Output {
                mode: OregonRentMode::ViolationManufacturedHomePark6PctCapExceeded,
                statutory_basis: "ORS 90.600 (post-SB 611) — manufactured home park 6 % cap".to_string(),
                notes: format!(
                    "VIOLATION: manufactured/floating home park space rent increase of {} basis points exceeds 6 % (600 basis points) cap under ORS 90.600 as amended by SB 611.",
                    input.rent_increase_basis_points
                ),
                citations,
                allowed_cap_basis_points: OR_MANUFACTURED_HOME_PARK_CAP_BASIS_POINTS,
            };
        }
        return Output {
            mode: OregonRentMode::CompliantManufacturedHomePark6PctCap,
            statutory_basis: "ORS 90.600 (post-SB 611) — manufactured home park 6 % cap satisfied".to_string(),
            notes: format!(
                "COMPLIANT: manufactured/floating home park space rent increase of {} basis points within 6 % (600 basis points) statutory cap under ORS 90.600 as amended by SB 611.",
                input.rent_increase_basis_points
            ),
            citations,
            allowed_cap_basis_points: OR_MANUFACTURED_HOME_PARK_CAP_BASIS_POINTS,
        };
    }

    if input.rent_increase_basis_points > OR_GENERAL_RESIDENTIAL_CAP_CEILING_BASIS_POINTS {
        return Output {
            mode: OregonRentMode::ViolationGeneralResidentialExceeds10PctCeiling,
            statutory_basis: "ORS 90.323 (post-SB 611) — 10 % absolute ceiling".to_string(),
            notes: format!(
                "VIOLATION: rent increase of {} basis points exceeds absolute 10 % (1000 basis points) ceiling under ORS 90.323 post-SB 611; 7 % + CPI floor irrelevant — 10 % is the hard ceiling.",
                input.rent_increase_basis_points
            ),
            citations,
            allowed_cap_basis_points: OR_GENERAL_RESIDENTIAL_CAP_CEILING_BASIS_POINTS,
        };
    }

    if input.rent_increase_basis_points > general_cpi_adjusted_cap_basis_points {
        return Output {
            mode: OregonRentMode::ViolationGeneralResidentialExceeds7PctPlusCpiOr10PctCap,
            statutory_basis: "ORS 90.323 — 7 % + CPI OR 10 %, whichever LESS".to_string(),
            notes: format!(
                "VIOLATION: rent increase of {} basis points exceeds applicable cap of {} basis points (7 % + CPI {} bp, capped at 10 % ceiling = whichever lesser).",
                input.rent_increase_basis_points,
                general_cpi_adjusted_cap_basis_points,
                input.current_cpi_basis_points
            ),
            citations,
            allowed_cap_basis_points: general_cpi_adjusted_cap_basis_points,
        };
    }

    if input.rent_increase_basis_points == OR_GENERAL_RESIDENTIAL_CAP_CEILING_BASIS_POINTS {
        return Output {
            mode: OregonRentMode::CompliantGeneralResidentialAt10PctCeilingExactlyAllowed,
            statutory_basis: "ORS 90.323 — 10 % absolute ceiling at boundary".to_string(),
            notes: "COMPLIANT: rent increase exactly at 10 % (1000 basis points) absolute ceiling; ORS 90.323 post-SB 611 allows up to but not exceeding 10 %.".to_string(),
            citations,
            allowed_cap_basis_points: OR_GENERAL_RESIDENTIAL_CAP_CEILING_BASIS_POINTS,
        };
    }

    Output {
        mode: OregonRentMode::CompliantGeneralResidentialCap7PctPlusCpiUnder10PctCeiling,
        statutory_basis: "ORS 90.323 — 7 % + CPI OR 10 %, whichever LESS".to_string(),
        notes: format!(
            "COMPLIANT: rent increase of {} basis points within applicable cap of {} basis points (7 % base + {} bp CPI, capped at 10 % ceiling).",
            input.rent_increase_basis_points,
            general_cpi_adjusted_cap_basis_points,
            input.current_cpi_basis_points
        ),
        citations,
        allowed_cap_basis_points: general_cpi_adjusted_cap_basis_points,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_post_first_year_compliant() -> Input {
        Input {
            property_classification: PropertyClassification::StandardResidentialUnderOrs90_323,
            tenancy_status: TenancyStatus::PostFirstYearMonthToMonth,
            notice_provided: NoticeProvided::NoticeProvidedAtOrAbove90DaysGeneral,
            rent_increase_basis_points: 500,
            current_cpi_basis_points: 300,
            years_since_first_certificate_of_occupancy: 50,
            manufactured_home_park_total_spaces: 0,
        }
    }

    #[test]
    fn property_outside_oregon_not_applicable() {
        let input = Input {
            property_classification: PropertyClassification::PropertyOutsideOregon,
            ..baseline_post_first_year_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, OregonRentMode::NotApplicablePropertyOutsideOregon);
    }

    #[test]
    fn new_construction_within_15_years_not_applicable() {
        let input = Input {
            property_classification:
                PropertyClassification::NewConstructionWithin15YearsOfFirstCertificateOfOccupancy,
            years_since_first_certificate_of_occupancy: 10,
            rent_increase_basis_points: 2_000,
            ..baseline_post_first_year_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            OregonRentMode::NotApplicableNewConstructionWithin15YearsExemption
        );
    }

    #[test]
    fn new_construction_at_exactly_15_years_no_longer_exempt() {
        let input = Input {
            property_classification:
                PropertyClassification::NewConstructionWithin15YearsOfFirstCertificateOfOccupancy,
            years_since_first_certificate_of_occupancy: 15,
            rent_increase_basis_points: 600,
            ..baseline_post_first_year_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            OregonRentMode::CompliantGeneralResidentialCap7PctPlusCpiUnder10PctCeiling
        );
    }

    #[test]
    fn government_subsidized_exempt_not_applicable() {
        let input = Input {
            property_classification: PropertyClassification::GovernmentSubsidizedReducedRentExempt,
            rent_increase_basis_points: 5_000,
            ..baseline_post_first_year_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            OregonRentMode::NotApplicableGovernmentSubsidizedReducedRentExempt
        );
    }

    #[test]
    fn manufactured_park_30_or_fewer_spaces_exempt() {
        let input = Input {
            property_classification:
                PropertyClassification::ManufacturedHomeParkWith30OrFewerSpacesExempt,
            manufactured_home_park_total_spaces: 25,
            rent_increase_basis_points: 1_500,
            ..baseline_post_first_year_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            OregonRentMode::NotApplicableManufacturedHomeParkWith30OrFewerSpacesExempt
        );
    }

    #[test]
    fn post_first_year_general_cap_compliant_baseline() {
        let result = check(&baseline_post_first_year_compliant());
        assert_eq!(
            result.mode,
            OregonRentMode::CompliantGeneralResidentialCap7PctPlusCpiUnder10PctCeiling
        );
    }

    #[test]
    fn post_first_year_at_exactly_10_pct_ceiling_compliant() {
        let input = Input {
            rent_increase_basis_points: 1_000,
            current_cpi_basis_points: 500,
            ..baseline_post_first_year_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            OregonRentMode::CompliantGeneralResidentialAt10PctCeilingExactlyAllowed
        );
    }

    #[test]
    fn post_first_year_above_10_pct_ceiling_violation() {
        let input = Input {
            rent_increase_basis_points: 1_001,
            current_cpi_basis_points: 600,
            ..baseline_post_first_year_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            OregonRentMode::ViolationGeneralResidentialExceeds10PctCeiling
        );
    }

    #[test]
    fn post_first_year_exceeds_7_pct_plus_cpi_violation() {
        let input = Input {
            rent_increase_basis_points: 900,
            current_cpi_basis_points: 100,
            ..baseline_post_first_year_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            OregonRentMode::ViolationGeneralResidentialExceeds7PctPlusCpiOr10PctCap
        );
        assert_eq!(result.allowed_cap_basis_points, 800);
    }

    #[test]
    fn first_year_tenancy_rent_increase_violation() {
        let input = Input {
            tenancy_status: TenancyStatus::FirstYearTenancyAnyLeaseType,
            rent_increase_basis_points: 200,
            ..baseline_post_first_year_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            OregonRentMode::ViolationFirstYearTenancyRentIncreaseAttempted
        );
    }

    #[test]
    fn manufactured_home_park_6_pct_compliant() {
        let input = Input {
            property_classification:
                PropertyClassification::ManufacturedOrFloatingHomeParkUnderOrs90_600,
            rent_increase_basis_points: 600,
            manufactured_home_park_total_spaces: 50,
            ..baseline_post_first_year_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            OregonRentMode::CompliantManufacturedHomePark6PctCap
        );
    }

    #[test]
    fn manufactured_home_park_above_6_pct_violation() {
        let input = Input {
            property_classification:
                PropertyClassification::ManufacturedOrFloatingHomeParkUnderOrs90_600,
            rent_increase_basis_points: 601,
            manufactured_home_park_total_spaces: 50,
            ..baseline_post_first_year_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            OregonRentMode::ViolationManufacturedHomePark6PctCapExceeded
        );
    }

    #[test]
    fn notice_below_90_day_minimum_violation() {
        let input = Input {
            notice_provided: NoticeProvided::NoticeProvidedBetween7And89DaysGeneral,
            ..baseline_post_first_year_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            OregonRentMode::ViolationNoticeBelow90DayMinimumGeneralResidential
        );
    }

    #[test]
    fn week_to_week_notice_below_7_days_violation() {
        let input = Input {
            tenancy_status: TenancyStatus::WeekToWeekTenancy,
            notice_provided: NoticeProvided::NoticeProvidedBelow7Days,
            ..baseline_post_first_year_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            OregonRentMode::ViolationWeekToWeekNoticeBelow7DayMinimum
        );
    }

    #[test]
    fn no_notice_provided_violation() {
        let input = Input {
            notice_provided: NoticeProvided::NoNoticeProvided,
            ..baseline_post_first_year_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, OregonRentMode::ViolationNoNoticeProvided);
    }

    #[test]
    fn citations_pin_sb_608_sb_611_and_ors_chapter_90() {
        let result = check(&baseline_post_first_year_compliant());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("Oregon SB 608 of 2019"));
        assert!(joined.contains("Oregon SB 611 of 2023"));
        assert!(joined.contains("Governor Kate Brown"));
        assert!(joined.contains("February 28, 2019"));
        assert!(joined.contains("Governor Tina Kotek"));
        assert!(joined.contains("July 6, 2023"));
        assert!(joined.contains("FIRST statewide rent control"));
        assert!(joined.contains("ORS 90.323"));
        assert!(joined.contains("ORS 90.600"));
        assert!(joined.contains("ORS 90.427"));
        assert!(joined.contains("7 % + CPI"));
        assert!(joined.contains("10 %"));
        assert!(joined.contains("6 %"));
        assert!(joined.contains("30 or fewer spaces"));
        assert!(joined.contains("first year"));
        assert!(joined.contains("90-day"));
        assert!(joined.contains("7-day"));
        assert!(joined.contains("15 years"));
        assert!(joined.contains("Portland City Code 30.01.085"));
        assert!(joined.contains("Milwaukie"));
        assert!(joined.contains("DAS"));
        assert!(joined.contains("September 30"));
    }

    #[test]
    fn constant_pin_dates_caps_and_thresholds() {
        assert_eq!(OR_SB_608_SIGNED_DATE_YEAR, 2019);
        assert_eq!(OR_SB_608_SIGNED_DATE_MONTH, 2);
        assert_eq!(OR_SB_608_SIGNED_DATE_DAY, 28);
        assert_eq!(OR_SB_611_SIGNED_DATE_YEAR, 2023);
        assert_eq!(OR_SB_611_SIGNED_DATE_MONTH, 7);
        assert_eq!(OR_SB_611_SIGNED_DATE_DAY, 6);
        assert_eq!(OR_GENERAL_RESIDENTIAL_CAP_BASE_BASIS_POINTS, 700);
        assert_eq!(OR_GENERAL_RESIDENTIAL_CAP_CEILING_BASIS_POINTS, 1_000);
        assert_eq!(OR_MANUFACTURED_HOME_PARK_CAP_BASIS_POINTS, 600);
        assert_eq!(OR_MANUFACTURED_HOME_PARK_EXEMPT_SPACES_THRESHOLD, 30);
        assert_eq!(OR_NEW_CONSTRUCTION_EXEMPTION_YEARS, 15);
        assert_eq!(OR_NOTICE_DAYS_REQUIRED_GENERAL, 90);
        assert_eq!(OR_NOTICE_DAYS_REQUIRED_WEEK_TO_WEEK, 7);
        assert_eq!(OR_FIRST_YEAR_TENANCY_MONTHS_FREEZE, 12);
        assert_eq!(OR_DAS_ANNUAL_NOTICE_DEADLINE_MONTH, 9);
        assert_eq!(OR_DAS_ANNUAL_NOTICE_DEADLINE_DAY, 30);
    }

    #[test]
    fn saturating_overflow_defense_extreme_cpi() {
        let input = Input {
            rent_increase_basis_points: 1_000,
            current_cpi_basis_points: u64::MAX,
            ..baseline_post_first_year_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            OregonRentMode::CompliantGeneralResidentialAt10PctCeilingExactlyAllowed
        );
        assert_eq!(result.allowed_cap_basis_points, 1_000);
    }
}
