//! Washington HB 1217 of 2025 Statewide Rent Stabilization
//! Compliance Module.
//!
//! Pure-compute check for landlord compliance with Washington
//! state's HB 1217 statewide rent stabilization law signed by
//! Governor Bob Ferguson on May 7, 2025 and effective
//! immediately. First statewide rent cap law in Washington
//! history; amends RCW 59.18 Residential Landlord-Tenant Act
//! and RCW 59.20 Manufactured / Mobile Home Landlord-Tenant
//! Act.
//!
//! Web research (verified 2026-06-03):
//! - **Washington HB 1217 of 2025** (Engrossed Substitute House
//!   Bill 1217; "Concerning housing stability for tenants
//!   subject to the residential landlord-tenant act and the
//!   manufactured / mobile home landlord-tenant act by limiting
//!   rent and fee increases, requiring notice of rent and fee
//!   increases, limiting fees and deposits, establishing a
//!   landlord resource center and associated services,
//!   authorizing tenant lease termination, creating parity
//!   between lease types, and providing for attorney general
//!   enforcement"). Signed by Governor Bob Ferguson on
//!   **May 7, 2025**; effective **immediately on signing**.
//!   ([HB 1217 Washington State Legislature Bill Summary 2025](https://app.leg.wa.gov/billsummary?billnumber=1217&year=2025);
//!   [Washington Enacts Statewide Rent Control — Stoel Rives LLP](https://www.stoel.com/insights/publications/washington-enacts-statewide-rent-control-key-rules-now-in-effect)).
//! - **General Rent Cap**: after the first year of tenancy,
//!   landlords may not increase rent in any 12-month period by
//!   more than **7 % + CPI** or **10 %**, whichever is **LESS**
//!   ([HB 1217 Landlord Resource Center — WA Dept of Commerce](https://www.commerce.wa.gov/housing-policy/hb1217-landlord-resource-center/)).
//! - **Manufactured / Mobile Home Park Space Rent Cap**: **5 %
//!   maximum annual percentage rent increase** (RCW 59.20
//!   amendments). Owners of manufactured/mobile homes who rent
//!   park space pay the lower cap.
//! - **First-Year Tenancy Rent Freeze**: landlord may NOT
//!   raise rent **in any amount during the first 12 months of
//!   tenancy** regardless of lease type (month-to-month or
//!   fixed term) ([North City Law — HB 1217 Tenant Rights
//!   Lawyer](https://www.northcitylaw.com/practice-areas/tenant/hb-1217-tenant-rights-lawyer-was-your-rent-increase-unlawful/)).
//! - **Notice Requirement**: minimum **90 days written notice**
//!   before rent increase (increased from prior 60-day RCW
//!   59.18.140 notice requirement); city ordinances may impose
//!   longer notice.
//! - **New Construction Exemption**: residential dwelling units
//!   are exempt for **12 years following issuance of the first
//!   certificate of occupancy** for the applicable dwelling
//!   unit. Manufactured/mobile home parks have separate
//!   construction-exemption analysis.
//! - **Sunset**: most provisions expire **July 1, 2040**
//!   (15-year sunset clause). Cap structure terminates absent
//!   legislative reauthorization.
//! - **Single-Family Home Exemption — NOT in Final Law**:
//!   Senate version proposed exemption for single-family homes
//!   not owned by corporations and 10 % + CPI cap; both
//!   provisions stripped in conference committee April 27,
//!   2025 ([SJA Property Management — HB 1217 Exemptions
//!   Analysis](https://propertymanagersseattle.com/rental-property-exempt-from-hb-1217/)).
//! - **Excessive Increase Civil Action**: tenant may bring
//!   civil action to recover damages (actual + statutory) plus
//!   attorney fees for any rent increase exceeding the cap.
//!   Washington Attorney General has independent enforcement
//!   authority.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const WA_HB_1217_SIGNED_DATE_YEAR: u32 = 2025;
pub const WA_HB_1217_SIGNED_DATE_MONTH: u32 = 5;
pub const WA_HB_1217_SIGNED_DATE_DAY: u32 = 7;
pub const WA_HB_1217_GENERAL_RENT_CAP_BASE_BASIS_POINTS: u64 = 700;
pub const WA_HB_1217_GENERAL_RENT_CAP_CEILING_BASIS_POINTS: u64 = 1_000;
pub const WA_HB_1217_MANUFACTURED_HOME_PARK_CAP_BASIS_POINTS: u64 = 500;
pub const WA_HB_1217_NOTICE_DAYS_REQUIRED: u32 = 90;
pub const WA_HB_1217_FIRST_YEAR_TENANCY_MONTHS_FREEZE: u32 = 12;
pub const WA_HB_1217_NEW_CONSTRUCTION_EXEMPTION_YEARS: u32 = 12;
pub const WA_HB_1217_SUNSET_DATE_YEAR: u32 = 2040;
pub const WA_HB_1217_SUNSET_DATE_MONTH: u32 = 7;
pub const WA_HB_1217_SUNSET_DATE_DAY: u32 = 1;
pub const WA_HB_1217_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyClassification {
    StandardResidentialDwellingUnitUnderRcw59_18,
    ManufacturedOrMobileHomeParkSpaceUnderRcw59_20,
    NewConstructionWithin12YearsOfCertificateOfOccupancy,
    PropertyOutsideWashington,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TenancyStatus {
    FirstYearTenancyMonthToMonth,
    FirstYearTenancyFixedTerm,
    PostFirstYearMonthToMonth,
    PostFirstYearFixedTerm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NoticeProvided {
    NoticeProvided90DaysOrMore,
    NoticeProvided60To89Days,
    NoticeProvidedLessThan60Days,
    NoNoticeProvided,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WashingtonHb1217Mode {
    NotApplicablePropertyOutsideWashington,
    NotApplicableNewConstructionWithin12YearsExemption,
    CompliantPostFirstYearGeneralCap7PctPlusCpiUnder10Pct,
    CompliantPostFirstYearAt10PctCeilingExactlyAllowed,
    CompliantManufacturedHomePark5PctCap,
    Compliant90DayNoticeProvided,
    ViolationFirstYearTenancyRentIncreaseAttempted,
    ViolationPostFirstYearIncreaseExceeds7PctPlusCpiOr10PctCap,
    ViolationPostFirstYearIncreaseExceeds10PctCeiling,
    ViolationManufacturedHomePark5PctCapExceeded,
    ViolationNoticeBelow90DayMinimum,
    ViolationNoNoticeProvided,
    ViolationPostSunsetIncreaseClaimedHb1217AfterJuly1_2040,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub property_classification: PropertyClassification,
    pub tenancy_status: TenancyStatus,
    pub notice_provided: NoticeProvided,
    pub rent_increase_basis_points: u64,
    pub current_cpi_basis_points: u64,
    pub years_since_first_certificate_of_occupancy: u32,
    pub increase_takes_effect_after_july_1_2040: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: WashingtonHb1217Mode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
    pub allowed_cap_basis_points: u64,
}

pub type RentalWashingtonHb1217RentStabilizationInput = Input;
pub type RentalWashingtonHb1217RentStabilizationOutput = Output;
pub type RentalWashingtonHb1217RentStabilizationResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "Washington HB 1217 of 2025 (Engrossed Substitute House Bill 1217; statewide rent stabilization) — signed by Governor Bob Ferguson on May 7, 2025; effective immediately".to_string(),
        "Washington RCW 59.18 (Residential Landlord-Tenant Act) — amended by HB 1217 to add 7 % + CPI OR 10 % rent cap (whichever LESS) for post-first-year tenancies".to_string(),
        "Washington RCW 59.20 (Manufactured / Mobile Home Landlord-Tenant Act) — amended by HB 1217 to impose 5 % maximum annual percentage rent increase on manufactured/mobile home park space rentals".to_string(),
        "HB 1217 first-year tenancy rent freeze — landlord may NOT raise rent in any amount during first 12 months of tenancy regardless of lease type (month-to-month or fixed term)".to_string(),
        "HB 1217 90-day written notice requirement — increased from prior 60-day RCW 59.18.140 requirement; city ordinances may impose longer notice".to_string(),
        "HB 1217 new construction exemption — residential dwelling units exempt for 12 years following issuance of first certificate of occupancy".to_string(),
        "HB 1217 sunset — most provisions expire July 1, 2040 (15-year sunset; cap structure terminates absent legislative reauthorization)".to_string(),
        "Single-family home exemption NOT in final law — Senate version proposed exemption for non-corporate single-family homes plus 10 % + CPI cap; both provisions stripped in conference committee April 27, 2025".to_string(),
        "Washington Attorney General independent enforcement authority + tenant private civil action with actual + statutory damages and attorney fees for excessive increases".to_string(),
        "HB 1217 Landlord Resource Center administered by Washington Department of Commerce (RCW 43.330) for landlord compliance assistance".to_string(),
        "Stoel Rives LLP HB 1217 Analysis — comprehensive landlord obligations under new statute".to_string(),
        "North City Law HB 1217 Tenant Rights Lawyer Practice Area — first-year freeze + 90-day notice analysis".to_string(),
    ];

    let general_cpi_adjusted_cap_basis_points = WA_HB_1217_GENERAL_RENT_CAP_BASE_BASIS_POINTS
        .saturating_add(input.current_cpi_basis_points)
        .min(WA_HB_1217_GENERAL_RENT_CAP_CEILING_BASIS_POINTS);

    if input.property_classification == PropertyClassification::PropertyOutsideWashington {
        return Output {
            mode: WashingtonHb1217Mode::NotApplicablePropertyOutsideWashington,
            statutory_basis: "Property outside Washington state; HB 1217 inapplicable".to_string(),
            notes: "Property outside Washington state; Washington HB 1217 statewide rent stabilization inapplicable.".to_string(),
            citations,
            allowed_cap_basis_points: 0,
        };
    }

    if input.increase_takes_effect_after_july_1_2040 {
        return Output {
            mode: WashingtonHb1217Mode::ViolationPostSunsetIncreaseClaimedHb1217AfterJuly1_2040,
            statutory_basis: "HB 1217 sunset July 1, 2040 — cap structure terminates absent reauthorization".to_string(),
            notes: "VIOLATION: rent increase claims HB 1217 application after July 1, 2040 sunset date; statutory cap structure terminated absent legislative reauthorization.".to_string(),
            citations,
            allowed_cap_basis_points: 0,
        };
    }

    if input.property_classification
        == PropertyClassification::NewConstructionWithin12YearsOfCertificateOfOccupancy
        && input.years_since_first_certificate_of_occupancy
            < WA_HB_1217_NEW_CONSTRUCTION_EXEMPTION_YEARS
    {
        return Output {
            mode: WashingtonHb1217Mode::NotApplicableNewConstructionWithin12YearsExemption,
            statutory_basis: "HB 1217 new construction exemption — first 12 years following certificate of occupancy".to_string(),
            notes: format!(
                "NOT APPLICABLE: new construction exemption applies; {} years since first certificate of occupancy < 12-year statutory exemption window.",
                input.years_since_first_certificate_of_occupancy
            ),
            citations,
            allowed_cap_basis_points: u64::MAX,
        };
    }

    if matches!(
        input.tenancy_status,
        TenancyStatus::FirstYearTenancyMonthToMonth | TenancyStatus::FirstYearTenancyFixedTerm
    ) && input.rent_increase_basis_points > 0
    {
        return Output {
            mode: WashingtonHb1217Mode::ViolationFirstYearTenancyRentIncreaseAttempted,
            statutory_basis: "HB 1217 first-year tenancy rent freeze — no rent increase permitted during first 12 months regardless of lease type".to_string(),
            notes: format!(
                "VIOLATION: rent increase of {} basis points attempted during first 12 months of tenancy; HB 1217 first-year rent freeze prohibits ANY rent increase regardless of month-to-month or fixed-term lease.",
                input.rent_increase_basis_points
            ),
            citations,
            allowed_cap_basis_points: 0,
        };
    }

    if input.notice_provided == NoticeProvided::NoNoticeProvided
        && input.rent_increase_basis_points > 0
    {
        return Output {
            mode: WashingtonHb1217Mode::ViolationNoNoticeProvided,
            statutory_basis: "HB 1217 90-day written notice requirement — no notice = void increase".to_string(),
            notes: "VIOLATION: rent increase attempted with no written notice provided; HB 1217 requires minimum 90-day written notice before rent increase takes effect.".to_string(),
            citations,
            allowed_cap_basis_points: 0,
        };
    }

    if matches!(
        input.notice_provided,
        NoticeProvided::NoticeProvided60To89Days | NoticeProvided::NoticeProvidedLessThan60Days
    ) && input.rent_increase_basis_points > 0
    {
        return Output {
            mode: WashingtonHb1217Mode::ViolationNoticeBelow90DayMinimum,
            statutory_basis: "HB 1217 90-day written notice requirement".to_string(),
            notes: format!(
                "VIOLATION: rent increase notice provided below 90-day statutory minimum; HB 1217 increased prior 60-day RCW 59.18.140 notice requirement to 90 days; provided notice classification: {:?}.",
                input.notice_provided
            ),
            citations,
            allowed_cap_basis_points: 0,
        };
    }

    if input.property_classification
        == PropertyClassification::ManufacturedOrMobileHomeParkSpaceUnderRcw59_20
    {
        if input.rent_increase_basis_points > WA_HB_1217_MANUFACTURED_HOME_PARK_CAP_BASIS_POINTS {
            return Output {
                mode: WashingtonHb1217Mode::ViolationManufacturedHomePark5PctCapExceeded,
                statutory_basis: "HB 1217 RCW 59.20 manufactured / mobile home park 5 % cap".to_string(),
                notes: format!(
                    "VIOLATION: manufactured/mobile home park space rent increase of {} basis points exceeds 5 % (500 basis points) cap under RCW 59.20 as amended by HB 1217.",
                    input.rent_increase_basis_points
                ),
                citations,
                allowed_cap_basis_points: WA_HB_1217_MANUFACTURED_HOME_PARK_CAP_BASIS_POINTS,
            };
        }
        return Output {
            mode: WashingtonHb1217Mode::CompliantManufacturedHomePark5PctCap,
            statutory_basis: "HB 1217 RCW 59.20 manufactured / mobile home park 5 % cap satisfied".to_string(),
            notes: format!(
                "COMPLIANT: manufactured/mobile home park space rent increase of {} basis points within 5 % (500 basis points) statutory cap under RCW 59.20 as amended by HB 1217.",
                input.rent_increase_basis_points
            ),
            citations,
            allowed_cap_basis_points: WA_HB_1217_MANUFACTURED_HOME_PARK_CAP_BASIS_POINTS,
        };
    }

    if input.rent_increase_basis_points > WA_HB_1217_GENERAL_RENT_CAP_CEILING_BASIS_POINTS {
        return Output {
            mode: WashingtonHb1217Mode::ViolationPostFirstYearIncreaseExceeds10PctCeiling,
            statutory_basis: "HB 1217 RCW 59.18 — 10 % absolute ceiling regardless of CPI".to_string(),
            notes: format!(
                "VIOLATION: rent increase of {} basis points exceeds absolute 10 % (1000 basis points) ceiling under HB 1217 RCW 59.18; 7 % + CPI floor irrelevant — 10 % is the hard ceiling.",
                input.rent_increase_basis_points
            ),
            citations,
            allowed_cap_basis_points: WA_HB_1217_GENERAL_RENT_CAP_CEILING_BASIS_POINTS,
        };
    }

    if input.rent_increase_basis_points > general_cpi_adjusted_cap_basis_points {
        return Output {
            mode: WashingtonHb1217Mode::ViolationPostFirstYearIncreaseExceeds7PctPlusCpiOr10PctCap,
            statutory_basis: "HB 1217 RCW 59.18 — 7 % + CPI OR 10 %, whichever LESS".to_string(),
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

    if input.rent_increase_basis_points == WA_HB_1217_GENERAL_RENT_CAP_CEILING_BASIS_POINTS {
        return Output {
            mode: WashingtonHb1217Mode::CompliantPostFirstYearAt10PctCeilingExactlyAllowed,
            statutory_basis: "HB 1217 RCW 59.18 — 10 % absolute ceiling at boundary".to_string(),
            notes: "COMPLIANT: rent increase exactly at 10 % (1000 basis points) absolute ceiling; HB 1217 allows up to but not exceeding 10 %.".to_string(),
            citations,
            allowed_cap_basis_points: WA_HB_1217_GENERAL_RENT_CAP_CEILING_BASIS_POINTS,
        };
    }

    Output {
        mode: WashingtonHb1217Mode::CompliantPostFirstYearGeneralCap7PctPlusCpiUnder10Pct,
        statutory_basis: "HB 1217 RCW 59.18 — 7 % + CPI OR 10 %, whichever LESS".to_string(),
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
            property_classification:
                PropertyClassification::StandardResidentialDwellingUnitUnderRcw59_18,
            tenancy_status: TenancyStatus::PostFirstYearMonthToMonth,
            notice_provided: NoticeProvided::NoticeProvided90DaysOrMore,
            rent_increase_basis_points: 500,
            current_cpi_basis_points: 300,
            years_since_first_certificate_of_occupancy: 50,
            increase_takes_effect_after_july_1_2040: false,
        }
    }

    #[test]
    fn property_outside_washington_not_applicable() {
        let input = Input {
            property_classification: PropertyClassification::PropertyOutsideWashington,
            ..baseline_post_first_year_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            WashingtonHb1217Mode::NotApplicablePropertyOutsideWashington
        );
    }

    #[test]
    fn new_construction_within_12_years_not_applicable() {
        let input = Input {
            property_classification:
                PropertyClassification::NewConstructionWithin12YearsOfCertificateOfOccupancy,
            years_since_first_certificate_of_occupancy: 5,
            rent_increase_basis_points: 1_500,
            ..baseline_post_first_year_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            WashingtonHb1217Mode::NotApplicableNewConstructionWithin12YearsExemption
        );
    }

    #[test]
    fn new_construction_at_exactly_12_years_no_longer_exempt() {
        let input = Input {
            property_classification:
                PropertyClassification::NewConstructionWithin12YearsOfCertificateOfOccupancy,
            years_since_first_certificate_of_occupancy: 12,
            rent_increase_basis_points: 600,
            ..baseline_post_first_year_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            WashingtonHb1217Mode::CompliantPostFirstYearGeneralCap7PctPlusCpiUnder10Pct
        );
    }

    #[test]
    fn post_first_year_general_cap_compliant_baseline() {
        let result = check(&baseline_post_first_year_compliant());
        assert_eq!(
            result.mode,
            WashingtonHb1217Mode::CompliantPostFirstYearGeneralCap7PctPlusCpiUnder10Pct
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
            WashingtonHb1217Mode::CompliantPostFirstYearAt10PctCeilingExactlyAllowed
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
            WashingtonHb1217Mode::ViolationPostFirstYearIncreaseExceeds10PctCeiling
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
            WashingtonHb1217Mode::ViolationPostFirstYearIncreaseExceeds7PctPlusCpiOr10PctCap
        );
        assert_eq!(result.allowed_cap_basis_points, 800);
    }

    #[test]
    fn post_first_year_high_cpi_capped_at_10_pct() {
        let input = Input {
            rent_increase_basis_points: 950,
            current_cpi_basis_points: 1_500,
            ..baseline_post_first_year_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            WashingtonHb1217Mode::CompliantPostFirstYearGeneralCap7PctPlusCpiUnder10Pct
        );
        assert_eq!(result.allowed_cap_basis_points, 1_000);
    }

    #[test]
    fn first_year_month_to_month_rent_increase_violation() {
        let input = Input {
            tenancy_status: TenancyStatus::FirstYearTenancyMonthToMonth,
            rent_increase_basis_points: 200,
            ..baseline_post_first_year_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            WashingtonHb1217Mode::ViolationFirstYearTenancyRentIncreaseAttempted
        );
    }

    #[test]
    fn first_year_fixed_term_rent_increase_violation() {
        let input = Input {
            tenancy_status: TenancyStatus::FirstYearTenancyFixedTerm,
            rent_increase_basis_points: 100,
            ..baseline_post_first_year_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            WashingtonHb1217Mode::ViolationFirstYearTenancyRentIncreaseAttempted
        );
    }

    #[test]
    fn manufactured_home_park_5_pct_compliant() {
        let input = Input {
            property_classification:
                PropertyClassification::ManufacturedOrMobileHomeParkSpaceUnderRcw59_20,
            rent_increase_basis_points: 500,
            ..baseline_post_first_year_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            WashingtonHb1217Mode::CompliantManufacturedHomePark5PctCap
        );
    }

    #[test]
    fn manufactured_home_park_above_5_pct_violation() {
        let input = Input {
            property_classification:
                PropertyClassification::ManufacturedOrMobileHomeParkSpaceUnderRcw59_20,
            rent_increase_basis_points: 501,
            ..baseline_post_first_year_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            WashingtonHb1217Mode::ViolationManufacturedHomePark5PctCapExceeded
        );
    }

    #[test]
    fn notice_below_90_day_minimum_violation() {
        let input = Input {
            notice_provided: NoticeProvided::NoticeProvided60To89Days,
            ..baseline_post_first_year_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            WashingtonHb1217Mode::ViolationNoticeBelow90DayMinimum
        );
    }

    #[test]
    fn no_notice_provided_violation() {
        let input = Input {
            notice_provided: NoticeProvided::NoNoticeProvided,
            ..baseline_post_first_year_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, WashingtonHb1217Mode::ViolationNoNoticeProvided);
    }

    #[test]
    fn post_sunset_july_1_2040_violation() {
        let input = Input {
            increase_takes_effect_after_july_1_2040: true,
            ..baseline_post_first_year_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            WashingtonHb1217Mode::ViolationPostSunsetIncreaseClaimedHb1217AfterJuly1_2040
        );
    }

    #[test]
    fn citations_pin_hb_1217_signing_caps_and_sunset() {
        let result = check(&baseline_post_first_year_compliant());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("Washington HB 1217 of 2025"));
        assert!(joined.contains("Governor Bob Ferguson"));
        assert!(joined.contains("May 7, 2025"));
        assert!(joined.contains("RCW 59.18"));
        assert!(joined.contains("RCW 59.20"));
        assert!(joined.contains("7 % + CPI OR 10 %"));
        assert!(joined.contains("5 %"));
        assert!(joined.contains("12 months"));
        assert!(joined.contains("90-day"));
        assert!(joined.contains("12 years"));
        assert!(joined.contains("July 1, 2040"));
        assert!(joined.contains("15-year"));
        assert!(joined.contains("Single-family home exemption NOT in final law"));
        assert!(joined.contains("Attorney General"));
        assert!(joined.contains("Stoel Rives"));
        assert!(joined.contains("North City Law"));
    }

    #[test]
    fn constant_pin_caps_dates_and_thresholds() {
        assert_eq!(WA_HB_1217_SIGNED_DATE_YEAR, 2025);
        assert_eq!(WA_HB_1217_SIGNED_DATE_MONTH, 5);
        assert_eq!(WA_HB_1217_SIGNED_DATE_DAY, 7);
        assert_eq!(WA_HB_1217_GENERAL_RENT_CAP_BASE_BASIS_POINTS, 700);
        assert_eq!(WA_HB_1217_GENERAL_RENT_CAP_CEILING_BASIS_POINTS, 1_000);
        assert_eq!(WA_HB_1217_MANUFACTURED_HOME_PARK_CAP_BASIS_POINTS, 500);
        assert_eq!(WA_HB_1217_NOTICE_DAYS_REQUIRED, 90);
        assert_eq!(WA_HB_1217_FIRST_YEAR_TENANCY_MONTHS_FREEZE, 12);
        assert_eq!(WA_HB_1217_NEW_CONSTRUCTION_EXEMPTION_YEARS, 12);
        assert_eq!(WA_HB_1217_SUNSET_DATE_YEAR, 2040);
        assert_eq!(WA_HB_1217_SUNSET_DATE_MONTH, 7);
        assert_eq!(WA_HB_1217_SUNSET_DATE_DAY, 1);
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
            WashingtonHb1217Mode::CompliantPostFirstYearAt10PctCeilingExactlyAllowed
        );
        assert_eq!(result.allowed_cap_basis_points, 1_000);
    }
}
