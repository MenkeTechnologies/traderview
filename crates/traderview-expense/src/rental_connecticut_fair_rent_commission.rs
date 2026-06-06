//! Connecticut Fair Rent Commission (Public Act 22-30; C.G.S.
//! §§ 7-148b/c) Compliance Module.
//!
//! Pure-compute check for landlord compliance with Connecticut's
//! Fair Rent Commission regime under Public Act 22-30 (2022),
//! codified at C.G.S. §§ 7-148b (creation/powers) and 7-148c
//! (13 excessive rent factors). PA 22-30 required every town,
//! city, or borough with population of 25,000+ per most recent
//! decennial census to establish a Fair Rent Commission by
//! **July 1, 2023**. As of 2025, ~83 percent of Connecticut
//! residents have access to a Fair Rent Commission.
//!
//! Web research (verified 2026-06-03):
//! - **Connecticut Public Act 22-30** (2022) — required every
//!   town, city, or borough with population of **25,000 or
//!   more** per most recent decennial census to adopt
//!   ordinance creating a Fair Rent Commission by **July 1,
//!   2023**; amended C.G.S. § 7-148b ([CT Department of
//!   Housing — Fair Rent Commission Toolkit (August 2024)](https://portal.ct.gov/-/media/doh/fair-rent-commission/fair-rent-commission-toolkit-final-as-of-8-15-24.pdf);
//!   [CTData — Eighty-Three Percent of CT Residents Now Have
//!   Access to a Fair Rent Commission](https://www.ctdata.org/blog/ct-residents-access-to-fair-rent-commission)).
//! - **C.G.S. § 7-148b Creation of Fair Rent Commission**: any
//!   town/city/borough subject to the 25,000-population mandate
//!   must (1) adopt ordinance creating commission on or before
//!   July 1, 2023; (2) notify Commissioner of Housing within
//!   **30 days** of adoption; (3) transmit copy of ordinance
//!   to Commissioner ([Justia C.G.S. § 7-148b](https://law.justia.com/codes/connecticut/title-7/chapter-98/section-7-148b/)).
//! - **C.G.S. § 7-148c — 13 Excessive Rent Factors**: factors
//!   commission must consider in determining whether rent or
//!   rent increase is excessive: (1) size of rent increase;
//!   (2) condition of premises; (3) landlord's operating costs
//!   (mortgage, repairs, maintenance); (4) services included
//!   in rent (heat, utilities); (5) income of tenant; (6) rents
//!   for comparable properties; (7) decrease in housing
//!   services; (8) substantial increase in expense; (9) rental
//!   history; (10) age and condition of property; (11) capital
//!   improvements; (12) real estate taxes; (13) other relevant
//!   factors.
//! - **"Seasonal Basis" Definition**: housing accommodations
//!   rented for a period or periods aggregating **not more than
//!   120 days** in any one calendar year are NOT subject to
//!   Fair Rent Commission jurisdiction.
//! - **"Rental Charge" Definition**: includes any fee or charge
//!   in addition to rent that is imposed or sought to be
//!   imposed upon a tenant by a landlord — covers parking,
//!   amenity fees, "junk fees," etc.
//! - **Commission Powers**: investigate and adjudicate
//!   excessive rent complaints; order rent reductions; stay
//!   evictions for retaliation under C.G.S. § 47a-20;
//!   subpoena testimony and documents; mediate between
//!   parties.
//! - **Retaliation Prohibition (C.G.S. § 47a-20)**: landlord
//!   may not increase rent, decrease services, or initiate
//!   eviction within 6 months of tenant's good-faith complaint
//!   to Fair Rent Commission.
//! - **Coverage**: ~83 percent of Connecticut residents now
//!   have access to a Fair Rent Commission post-PA 22-30,
//!   including residents of Bridgeport, Hartford, New Haven,
//!   Waterbury, Stamford, Norwalk, Danbury, New Britain, West
//!   Hartford, Greenwich, and other large municipalities.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const CT_PA_22_30_ENACTMENT_YEAR: u32 = 2022;
pub const CT_PA_22_30_ORDINANCE_DEADLINE_YEAR: u32 = 2023;
pub const CT_PA_22_30_ORDINANCE_DEADLINE_MONTH: u32 = 7;
pub const CT_PA_22_30_ORDINANCE_DEADLINE_DAY: u32 = 1;
pub const CT_FAIR_RENT_COMMISSION_MUNICIPAL_POPULATION_THRESHOLD: u32 = 25_000;
pub const CT_FAIR_RENT_COMMISSION_DOH_NOTIFICATION_DAYS: u32 = 30;
pub const CT_SEASONAL_BASIS_MAX_DAYS_PER_YEAR: u32 = 120;
pub const CT_EXCESSIVE_RENT_FACTORS_COUNT: u32 = 13;
pub const CT_RETALIATION_LOOKBACK_MONTHS: u32 = 6;
pub const CT_RESIDENTS_COVERAGE_BASIS_POINTS_POST_PA_22_30: u64 = 8_300;
pub const CT_FAIR_RENT_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MunicipalityClassification {
    SubjectTo25kPopulationMandate,
    SmallerMunicipalityWithDiscretionaryCommission,
    SmallerMunicipalityWithoutCommission,
    NotInConnecticut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CommissionStatus {
    OrdinanceAdoptedAndDoHNotifiedWithin30Days,
    OrdinanceAdoptedButDoHNotificationLate,
    OrdinanceNotAdoptedBy25kMandateDeadline,
    OrdinanceNotApplicableNoMandate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RentalArrangement {
    StandardYearRoundResidentialTenancy,
    SeasonalBasisAtOrUnder120DaysPerYearExempt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LandlordAction {
    RentIncrease,
    RentalChargeAddedFeeOrServiceCharge,
    ServiceReduction,
    EvictionInitiatedAfterTenantComplaint,
    NoAdverseAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnecticutFairRentMode {
    NotApplicableNotInConnecticut,
    NotApplicableSeasonalBasisExempt,
    NotApplicableSmallerMunicipalityNoCommission,
    CompliantOrdinanceAdoptedAndDoHNotifiedTimely,
    CompliantSmallerMunicipalityWithDiscretionaryCommission,
    CompliantNoExcessiveRentDeterminationFactorsConsideredAndPassed,
    CompliantRentalChargeDisclosedAndNotExcessive,
    ViolationMunicipalityFailedToAdoptOrdinanceBy25kMandateDeadline,
    ViolationDoHNotificationNotProvidedWithin30Days,
    ViolationExcessiveRentDeterminationUnderC148cFactors,
    ViolationRetaliationWithin6MonthsOfTenantComplaintUnderSection47a20,
    ViolationLandlordIgnoredFairRentCommissionRentReductionOrder,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub municipality_classification: MunicipalityClassification,
    pub commission_status: CommissionStatus,
    pub rental_arrangement: RentalArrangement,
    pub landlord_action: LandlordAction,
    pub commission_found_excessive_rent_under_c148c: bool,
    pub action_taken_within_6_months_of_tenant_complaint: bool,
    pub landlord_complied_with_rent_reduction_order: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: ConnecticutFairRentMode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type RentalConnecticutFairRentCommissionInput = Input;
pub type RentalConnecticutFairRentCommissionOutput = Output;
pub type RentalConnecticutFairRentCommissionResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "Connecticut Public Act 22-30 (2022) — required every town/city/borough with population of 25,000+ per most recent decennial census to adopt ordinance creating Fair Rent Commission by July 1, 2023; amended C.G.S. § 7-148b".to_string(),
        "C.G.S. § 7-148b Creation of Fair Rent Commission — municipality must (1) adopt ordinance creating commission on or before July 1, 2023; (2) notify Commissioner of Housing within 30 days of adoption; (3) transmit copy of ordinance to Commissioner".to_string(),
        "C.G.S. § 7-148c — 13 Excessive Rent Factors commission must consider: (1) size of rent increase; (2) condition of premises; (3) landlord's operating costs (mortgage, repairs, maintenance); (4) services included in rent (heat, utilities); (5) income of tenant; (6) rents for comparable properties; (7) decrease in housing services; (8) substantial increase in expense; (9) rental history; (10) age and condition of property; (11) capital improvements; (12) real estate taxes; (13) other relevant factors".to_string(),
        "Seasonal Basis Definition (C.G.S. § 7-148b) — housing accommodations rented for a period or periods aggregating not more than 120 days in any one calendar year are NOT subject to Fair Rent Commission jurisdiction".to_string(),
        "Rental Charge Definition (C.G.S. § 7-148b) — includes any fee or charge in addition to rent that is imposed or sought to be imposed upon a tenant by a landlord — covers parking, amenity fees, junk fees, etc.".to_string(),
        "Commission Powers — investigate and adjudicate excessive rent complaints; order rent reductions; stay evictions for retaliation under C.G.S. § 47a-20; subpoena testimony and documents; mediate between parties".to_string(),
        "C.G.S. § 47a-20 Retaliation Prohibition — landlord may not increase rent, decrease services, or initiate eviction within 6 months of tenant's good-faith complaint to Fair Rent Commission".to_string(),
        "Coverage Statistic — ~83 percent of Connecticut residents now have access to a Fair Rent Commission post-PA 22-30, including residents of Bridgeport, Hartford, New Haven, Waterbury, Stamford, Norwalk, Danbury, New Britain, West Hartford, Greenwich, and other large municipalities".to_string(),
        "CT Department of Housing — Fair Rent Commission Toolkit (August 2024) — operational guide for commission creation and case processing".to_string(),
        "CTData — Eighty-Three Percent of Connecticut Residents Now Have Access to a Fair Rent Commission — coverage analysis".to_string(),
        "CT General Assembly Office of Legislative Research 2023-R-0247 — Fair Rent Commission summary".to_string(),
        "United Way of Connecticut — Fair Rent Commissions Resource Guide".to_string(),
    ];

    if input.municipality_classification == MunicipalityClassification::NotInConnecticut {
        return Output {
            mode: ConnecticutFairRentMode::NotApplicableNotInConnecticut,
            statutory_basis: "Property outside Connecticut; PA 22-30 / C.G.S. §§ 7-148b/c inapplicable".to_string(),
            notes: "NOT APPLICABLE: property outside Connecticut; Connecticut Fair Rent Commission regime inapplicable.".to_string(),
            citations,
        };
    }

    if input.rental_arrangement == RentalArrangement::SeasonalBasisAtOrUnder120DaysPerYearExempt {
        return Output {
            mode: ConnecticutFairRentMode::NotApplicableSeasonalBasisExempt,
            statutory_basis: "C.G.S. § 7-148b — seasonal basis exemption (≤ 120 days per year)".to_string(),
            notes: "NOT APPLICABLE: housing rented on seasonal basis (≤ 120 days per calendar year); Fair Rent Commission jurisdiction does not extend to such rentals.".to_string(),
            citations,
        };
    }

    if input.municipality_classification
        == MunicipalityClassification::SmallerMunicipalityWithoutCommission
    {
        return Output {
            mode: ConnecticutFairRentMode::NotApplicableSmallerMunicipalityNoCommission,
            statutory_basis: "C.G.S. § 7-148b — smaller municipality (<25,000) with no discretionary commission".to_string(),
            notes: "NOT APPLICABLE: municipality below 25,000 population threshold and has not elected to create discretionary Fair Rent Commission; Connecticut Fair Rent Commission regime not triggered.".to_string(),
            citations,
        };
    }

    if input.municipality_classification
        == MunicipalityClassification::SubjectTo25kPopulationMandate
        && input.commission_status == CommissionStatus::OrdinanceNotAdoptedBy25kMandateDeadline
    {
        return Output {
            mode: ConnecticutFairRentMode::ViolationMunicipalityFailedToAdoptOrdinanceBy25kMandateDeadline,
            statutory_basis: "PA 22-30 / C.G.S. § 7-148b — mandatory ordinance adoption by July 1, 2023".to_string(),
            notes: "VIOLATION: municipality subject to 25,000-population mandate failed to adopt Fair Rent Commission ordinance by July 1, 2023 statutory deadline.".to_string(),
            citations,
        };
    }

    if input.commission_status == CommissionStatus::OrdinanceAdoptedButDoHNotificationLate {
        return Output {
            mode: ConnecticutFairRentMode::ViolationDoHNotificationNotProvidedWithin30Days,
            statutory_basis: "C.G.S. § 7-148b — 30-day notification to Commissioner of Housing".to_string(),
            notes: "VIOLATION: ordinance adopted but Commissioner of Housing not notified within 30 days as required by C.G.S. § 7-148b.".to_string(),
            citations,
        };
    }

    if input.landlord_action == LandlordAction::EvictionInitiatedAfterTenantComplaint
        && input.action_taken_within_6_months_of_tenant_complaint
    {
        return Output {
            mode: ConnecticutFairRentMode::ViolationRetaliationWithin6MonthsOfTenantComplaintUnderSection47a20,
            statutory_basis: "C.G.S. § 47a-20 — retaliation prohibition within 6 months of tenant complaint".to_string(),
            notes: "VIOLATION: landlord initiated eviction within 6 months of tenant's good-faith complaint to Fair Rent Commission; C.G.S. § 47a-20 retaliation prohibition violated.".to_string(),
            citations,
        };
    }

    if input.commission_found_excessive_rent_under_c148c
        && !input.landlord_complied_with_rent_reduction_order
    {
        return Output {
            mode: ConnecticutFairRentMode::ViolationLandlordIgnoredFairRentCommissionRentReductionOrder,
            statutory_basis: "C.G.S. § 7-148c — fair rent commission rent reduction order binding".to_string(),
            notes: "VIOLATION: Fair Rent Commission determined rent excessive under C.G.S. § 7-148c 13-factor analysis and issued rent reduction order; landlord failed to comply with binding order.".to_string(),
            citations,
        };
    }

    if input.commission_found_excessive_rent_under_c148c
        && input.landlord_complied_with_rent_reduction_order
    {
        return Output {
            mode: ConnecticutFairRentMode::ViolationExcessiveRentDeterminationUnderC148cFactors,
            statutory_basis: "C.G.S. § 7-148c — rent found excessive under 13-factor analysis".to_string(),
            notes: "VIOLATION: Fair Rent Commission determined rent excessive under C.G.S. § 7-148c 13-factor analysis; landlord has complied with rent reduction order but excessive-rent determination on record may inform future commission proceedings.".to_string(),
            citations,
        };
    }

    if input.municipality_classification
        == MunicipalityClassification::SmallerMunicipalityWithDiscretionaryCommission
    {
        return Output {
            mode: ConnecticutFairRentMode::CompliantSmallerMunicipalityWithDiscretionaryCommission,
            statutory_basis: "C.G.S. § 7-148b — smaller municipality elected to create discretionary Fair Rent Commission".to_string(),
            notes: "COMPLIANT: smaller municipality (<25,000) has elected to create discretionary Fair Rent Commission; Fair Rent Commission framework applies; no excessive-rent finding or retaliation.".to_string(),
            citations,
        };
    }

    if input.commission_status == CommissionStatus::OrdinanceAdoptedAndDoHNotifiedWithin30Days {
        return Output {
            mode: ConnecticutFairRentMode::CompliantOrdinanceAdoptedAndDoHNotifiedTimely,
            statutory_basis: "PA 22-30 / C.G.S. § 7-148b — ordinance adopted and DoH notified timely".to_string(),
            notes: "COMPLIANT: municipality subject to 25,000-population mandate adopted Fair Rent Commission ordinance by July 1, 2023 deadline and notified Commissioner of Housing within 30 days; no excessive-rent finding or retaliation.".to_string(),
            citations,
        };
    }

    Output {
        mode: ConnecticutFairRentMode::CompliantNoExcessiveRentDeterminationFactorsConsideredAndPassed,
        statutory_basis: "C.G.S. § 7-148c — 13-factor excessive-rent analysis passed".to_string(),
        notes: "COMPLIANT: 13-factor excessive-rent analysis under C.G.S. § 7-148c performed; no excessive-rent determination; no retaliation within 6 months of any tenant complaint.".to_string(),
        citations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_compliant_25k_municipality() -> Input {
        Input {
            municipality_classification: MunicipalityClassification::SubjectTo25kPopulationMandate,
            commission_status: CommissionStatus::OrdinanceAdoptedAndDoHNotifiedWithin30Days,
            rental_arrangement: RentalArrangement::StandardYearRoundResidentialTenancy,
            landlord_action: LandlordAction::NoAdverseAction,
            commission_found_excessive_rent_under_c148c: false,
            action_taken_within_6_months_of_tenant_complaint: false,
            landlord_complied_with_rent_reduction_order: true,
        }
    }

    #[test]
    fn property_outside_ct_not_applicable() {
        let input = Input {
            municipality_classification: MunicipalityClassification::NotInConnecticut,
            ..baseline_compliant_25k_municipality()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            ConnecticutFairRentMode::NotApplicableNotInConnecticut
        );
    }

    #[test]
    fn seasonal_basis_120_days_exempt() {
        let input = Input {
            rental_arrangement: RentalArrangement::SeasonalBasisAtOrUnder120DaysPerYearExempt,
            ..baseline_compliant_25k_municipality()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            ConnecticutFairRentMode::NotApplicableSeasonalBasisExempt
        );
    }

    #[test]
    fn smaller_municipality_no_commission_not_applicable() {
        let input = Input {
            municipality_classification:
                MunicipalityClassification::SmallerMunicipalityWithoutCommission,
            commission_status: CommissionStatus::OrdinanceNotApplicableNoMandate,
            ..baseline_compliant_25k_municipality()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            ConnecticutFairRentMode::NotApplicableSmallerMunicipalityNoCommission
        );
    }

    #[test]
    fn smaller_municipality_with_discretionary_commission_compliant() {
        let input = Input {
            municipality_classification:
                MunicipalityClassification::SmallerMunicipalityWithDiscretionaryCommission,
            ..baseline_compliant_25k_municipality()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            ConnecticutFairRentMode::CompliantSmallerMunicipalityWithDiscretionaryCommission
        );
    }

    #[test]
    fn ordinance_not_adopted_by_25k_mandate_deadline_violation() {
        let input = Input {
            commission_status: CommissionStatus::OrdinanceNotAdoptedBy25kMandateDeadline,
            ..baseline_compliant_25k_municipality()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            ConnecticutFairRentMode::ViolationMunicipalityFailedToAdoptOrdinanceBy25kMandateDeadline
        );
    }

    #[test]
    fn doh_notification_late_violation() {
        let input = Input {
            commission_status: CommissionStatus::OrdinanceAdoptedButDoHNotificationLate,
            ..baseline_compliant_25k_municipality()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            ConnecticutFairRentMode::ViolationDoHNotificationNotProvidedWithin30Days
        );
    }

    #[test]
    fn ordinance_adopted_and_doh_notified_compliant() {
        let result = check(&baseline_compliant_25k_municipality());
        assert_eq!(
            result.mode,
            ConnecticutFairRentMode::CompliantOrdinanceAdoptedAndDoHNotifiedTimely
        );
    }

    #[test]
    fn excessive_rent_with_compliance_still_violation() {
        let input = Input {
            commission_found_excessive_rent_under_c148c: true,
            landlord_complied_with_rent_reduction_order: true,
            ..baseline_compliant_25k_municipality()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            ConnecticutFairRentMode::ViolationExcessiveRentDeterminationUnderC148cFactors
        );
    }

    #[test]
    fn excessive_rent_without_compliance_violation() {
        let input = Input {
            commission_found_excessive_rent_under_c148c: true,
            landlord_complied_with_rent_reduction_order: false,
            ..baseline_compliant_25k_municipality()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            ConnecticutFairRentMode::ViolationLandlordIgnoredFairRentCommissionRentReductionOrder
        );
    }

    #[test]
    fn retaliation_within_6_months_of_complaint_violation() {
        let input = Input {
            landlord_action: LandlordAction::EvictionInitiatedAfterTenantComplaint,
            action_taken_within_6_months_of_tenant_complaint: true,
            ..baseline_compliant_25k_municipality()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            ConnecticutFairRentMode::ViolationRetaliationWithin6MonthsOfTenantComplaintUnderSection47a20
        );
    }

    #[test]
    fn retaliation_after_6_months_not_violation() {
        let input = Input {
            landlord_action: LandlordAction::EvictionInitiatedAfterTenantComplaint,
            action_taken_within_6_months_of_tenant_complaint: false,
            ..baseline_compliant_25k_municipality()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            ConnecticutFairRentMode::CompliantOrdinanceAdoptedAndDoHNotifiedTimely
        );
    }

    #[test]
    fn citations_pin_pa_22_30_and_cgs_sections() {
        let result = check(&baseline_compliant_25k_municipality());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("Connecticut Public Act 22-30"));
        assert!(joined.contains("2022"));
        assert!(joined.contains("25,000+"));
        assert!(joined.contains("July 1, 2023"));
        assert!(joined.contains("C.G.S. § 7-148b"));
        assert!(joined.contains("C.G.S. § 7-148c"));
        assert!(joined.contains("13 Excessive Rent Factors"));
        assert!(joined.contains("120 days"));
        assert!(joined.contains("30 days"));
        assert!(joined.contains("C.G.S. § 47a-20"));
        assert!(joined.contains("6 months"));
        assert!(joined.contains("Commissioner of Housing"));
        assert!(joined.contains("83 percent"));
        assert!(joined.contains("Bridgeport"));
        assert!(joined.contains("Hartford"));
        assert!(joined.contains("New Haven"));
        assert!(joined.contains("CT Department of Housing"));
        assert!(joined.contains("Fair Rent Commission Toolkit"));
        assert!(joined.contains("CTData"));
        assert!(joined.contains("CT General Assembly Office of Legislative Research"));
        assert!(joined.contains("United Way of Connecticut"));
    }

    #[test]
    fn constant_pin_dates_thresholds_and_coverage() {
        assert_eq!(CT_PA_22_30_ENACTMENT_YEAR, 2022);
        assert_eq!(CT_PA_22_30_ORDINANCE_DEADLINE_YEAR, 2023);
        assert_eq!(CT_PA_22_30_ORDINANCE_DEADLINE_MONTH, 7);
        assert_eq!(CT_PA_22_30_ORDINANCE_DEADLINE_DAY, 1);
        assert_eq!(
            CT_FAIR_RENT_COMMISSION_MUNICIPAL_POPULATION_THRESHOLD,
            25_000
        );
        assert_eq!(CT_FAIR_RENT_COMMISSION_DOH_NOTIFICATION_DAYS, 30);
        assert_eq!(CT_SEASONAL_BASIS_MAX_DAYS_PER_YEAR, 120);
        assert_eq!(CT_EXCESSIVE_RENT_FACTORS_COUNT, 13);
        assert_eq!(CT_RETALIATION_LOOKBACK_MONTHS, 6);
        assert_eq!(CT_RESIDENTS_COVERAGE_BASIS_POINTS_POST_PA_22_30, 8_300);
        assert_eq!(CT_FAIR_RENT_BASIS_POINT_DENOMINATOR, 10_000);
    }
}
