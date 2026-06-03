//! D.C. Tenant Opportunity to Purchase Act (TOPA) compliance for
//! trader-landlords selling rental property in Washington, D.C.
//!
//! TOPA was enacted in 1980 (D.C. Law 3-86) as part of the Rental
//! Housing Conversion and Sale Act, codified at D.C. Code
//! § 42-3401.01 et seq. with the operative tenant-purchase provisions
//! at § 42-3404.02 et seq. and the civil-action provisions at
//! § 42-3405.03. Implementing regulations in 14 DCMR.
//!
//! TOPA is the most aggressive municipal tenant-purchase right in
//! the United States: when a landlord sells residential rental
//! property, tenants get a statutory right of first refusal to
//! purchase the property at the terms offered by a third-party
//! buyer, with extended negotiation and financing windows.
//!
//! **Coverage tiers** (post-2018 amendments):
//!
//! - **Single-family homes**: EXEMPT as of July 3, 2018 under the
//!   TOPA Single-Family Home Exemption Amendment Act of 2017.
//!   Single-family dwellings with an Accessory Dwelling Unit are
//!   also exempted.
//! - **2-4 unit buildings**: covered IF owned by a corporate
//!   business OR an individual owner who owns multiple DC
//!   properties. Otherwise exempt. **90-day negotiation period**
//!   after Statement of Interest. Extension of 1 day for each day
//!   landlord fails to deliver required information; 15-day
//!   extension if landlord enters into third-party contract during
//!   negotiation.
//! - **5+ unit buildings**: full TOPA applies. **45-day Cooling-Off
//!   Period** after offer of sale (or 30 days for Statement of
//!   Interest if tenant association already exists). **120-day
//!   negotiation period** following Statement of Interest. **120-
//!   240 day financing window** depending on lender needs.
//!
//! **5+ unit tenant organization registration** under D.C. Code
//! § 42-3404.11: tenant organization may NOT assign rights to a
//! third party during first 45 days unless registration application
//! has been submitted to DHCD AND tenant organization has completed
//! certified TOPA-rights training.
//!
//! **Civil enforcement** under D.C. Code § 42-3405.03: any aggrieved
//! owner, tenant, or tenant organization may seek civil enforcement
//! in law or equity; prevailing party entitled to costs and
//! reasonable attorney fees. § 42-3509.01 housing-code-violation
//! civil penalties may apply separately.
//!
//! **RENTAL Act of 2025** (Rebalancing Expectations for Neighbors,
//! Tenants, and Landlords Act of 2025): passed by D.C. Council
//! September 17, 2025; effective December 31, 2025. Significantly
//! amends TOPA but does not eliminate the core right of first
//! refusal regime — primarily shortens windows and clarifies
//! exemptions.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[allow(dead_code)]
pub const TOPA_SINGLE_FAMILY_EXEMPTION_EFFECTIVE_YEAR: u32 = 2018;
#[allow(dead_code)]
pub const TOPA_2_TO_4_UNIT_THRESHOLD_LOW: u32 = 2;
#[allow(dead_code)]
pub const TOPA_2_TO_4_UNIT_THRESHOLD_HIGH: u32 = 4;
#[allow(dead_code)]
pub const TOPA_5_PLUS_UNIT_THRESHOLD: u32 = 5;
#[allow(dead_code)]
pub const TOPA_2_TO_4_NEGOTIATION_DAYS: u32 = 90;
#[allow(dead_code)]
pub const TOPA_5_PLUS_NEGOTIATION_DAYS: u32 = 120;
#[allow(dead_code)]
pub const TOPA_5_PLUS_COOLING_OFF_NO_ASSOC_DAYS: u32 = 45;
#[allow(dead_code)]
pub const TOPA_5_PLUS_STATEMENT_OF_INTEREST_EXISTING_ASSOC_DAYS: u32 = 30;
#[allow(dead_code)]
pub const TOPA_FINANCING_DAYS_MIN: u32 = 120;
#[allow(dead_code)]
pub const TOPA_FINANCING_DAYS_MAX: u32 = 240;
#[allow(dead_code)]
pub const TOPA_THIRD_PARTY_CONTRACT_EXTENSION_DAYS: u32 = 15;
#[allow(dead_code)]
pub const RENTAL_ACT_2025_EFFECTIVE_YEAR: u32 = 2025;
#[allow(dead_code)]
pub const RENTAL_ACT_2025_EFFECTIVE_MONTH: u32 = 12;
#[allow(dead_code)]
pub const RENTAL_ACT_2025_EFFECTIVE_DAY: u32 = 31;
#[allow(dead_code)]
pub const TOPA_ORIGINAL_ENACTMENT_YEAR: u32 = 1980;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyType {
    SingleFamilyHome,
    TwoToFourUnit,
    FivePlusUnit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    ExemptSingleFamilyHomePost2018,
    Exempt2To4UnitNonCorporateSingleProperty,
    Compliant5PlusUnitOfferOfSaleProvidedFullProcess,
    Compliant2To4UnitNegotiationPeriodActive,
    CompliantRightOfFirstRefusalRespectedOnThirdPartyContract,
    ViolationFailedToProvideOfferOfSale,
    ViolationFailedToObserveCoolingOffPeriod45Days,
    ViolationFailedToHonorRightOfFirstRefusalOnThirdPartyContract,
    ViolationTenantOrganizationAssignedRightsWithoutDhcdRegistration,
    ViolationLandlordTimelineNotHonoredCivilCauseOfAction,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub property_type: PropertyType,
    pub corporate_owner_or_multi_property_owner: bool,
    pub offer_of_sale_provided_to_tenants: bool,
    pub days_since_offer_of_sale: u32,
    pub tenant_association_pre_existing: bool,
    pub statement_of_interest_filed: bool,
    pub days_in_cooling_off_period_so_far: u32,
    pub third_party_contract_signed: bool,
    pub right_of_first_refusal_provided: bool,
    pub tenant_organization_registered_with_dhcd: bool,
    pub tenant_organization_assigned_rights_within_45_days: bool,
    pub current_year: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Output {
    pub severity: Severity,
    pub applicable_negotiation_days: u32,
    pub statement_of_interest_window_days: u32,
    pub topa_applies: bool,
    pub civil_cause_of_action_exposure: bool,
    pub notes: Vec<String>,
    pub citations: Vec<String>,
}

pub type DcTopaInput = Input;
pub type DcTopaOutput = Output;
pub type DcTopaResult = Output;

pub fn check(input: &Input) -> Output {
    let mut notes: Vec<String> = Vec::new();
    let citations: Vec<String> = vec![
        "D.C. Code § 42-3401.01 et seq. (Rental Housing Conversion and Sale Act of 1980 — TOPA)".to_string(),
        "D.C. Code § 42-3404.02 (tenant opportunity to purchase — operative provision)".to_string(),
        "D.C. Code § 42-3404.11 (tenant organization registration with DHCD)".to_string(),
        "D.C. Code § 42-3405.03 (civil cause of action — attorney fees + costs)".to_string(),
        "D.C. Code § 42-3509.01 (housing-code-violation civil penalty)".to_string(),
        "14 DCMR § 4300 et seq. (TOPA implementing regulations)".to_string(),
        "D.C. Law 3-86 (Tenant Opportunity to Purchase Act of 1980 — original enactment)".to_string(),
        "TOPA Single-Family Home Exemption Amendment Act of 2017 (eff. July 3, 2018)".to_string(),
        "RENTAL Act of 2025 (passed Sep 17, 2025; eff. Dec 31, 2025)".to_string(),
        "DHCD Tenant Opportunity to Purchase Assistance program".to_string(),
        "DC Office of the Tenant Advocate (OTA) TOPA guidance".to_string(),
    ];

    if matches!(input.property_type, PropertyType::SingleFamilyHome) {
        notes.push(format!(
            "Single-family home — exempt under TOPA Single-Family Home Exemption Amendment Act of 2017 (eff. July 3, {}).",
            TOPA_SINGLE_FAMILY_EXEMPTION_EFFECTIVE_YEAR
        ));
        return Output {
            severity: Severity::ExemptSingleFamilyHomePost2018,
            applicable_negotiation_days: 0,
            statement_of_interest_window_days: 0,
            topa_applies: false,
            civil_cause_of_action_exposure: false,
            notes,
            citations,
        };
    }

    if matches!(input.property_type, PropertyType::TwoToFourUnit)
        && !input.corporate_owner_or_multi_property_owner
    {
        notes.push("2-4 unit building owned by non-corporate single-property owner — exempt from TOPA under post-2018 amendments.".to_string());
        return Output {
            severity: Severity::Exempt2To4UnitNonCorporateSingleProperty,
            applicable_negotiation_days: 0,
            statement_of_interest_window_days: 0,
            topa_applies: false,
            civil_cause_of_action_exposure: false,
            notes,
            citations,
        };
    }

    if !input.offer_of_sale_provided_to_tenants {
        notes.push("Landlord failed to provide TOPA Offer of Sale — per se § 42-3404.02 violation; tenants entitled to civil cause of action under § 42-3405.03.".to_string());
        return Output {
            severity: Severity::ViolationFailedToProvideOfferOfSale,
            applicable_negotiation_days: 0,
            statement_of_interest_window_days: 0,
            topa_applies: true,
            civil_cause_of_action_exposure: true,
            notes,
            citations,
        };
    }

    let (negotiation_days, statement_window_days) = match input.property_type {
        PropertyType::TwoToFourUnit => (TOPA_2_TO_4_NEGOTIATION_DAYS, TOPA_2_TO_4_NEGOTIATION_DAYS),
        PropertyType::FivePlusUnit => {
            let statement_window = if input.tenant_association_pre_existing {
                TOPA_5_PLUS_STATEMENT_OF_INTEREST_EXISTING_ASSOC_DAYS
            } else {
                TOPA_5_PLUS_COOLING_OFF_NO_ASSOC_DAYS
            };
            (TOPA_5_PLUS_NEGOTIATION_DAYS, statement_window)
        }
        PropertyType::SingleFamilyHome => unreachable!(),
    };

    if matches!(input.property_type, PropertyType::FivePlusUnit)
        && input.tenant_organization_assigned_rights_within_45_days
        && !input.tenant_organization_registered_with_dhcd
    {
        notes.push("Tenant organization in 5+ unit building assigned rights within 45-day cooling-off period without DHCD registration — § 42-3404.11 violation.".to_string());
        return Output {
            severity: Severity::ViolationTenantOrganizationAssignedRightsWithoutDhcdRegistration,
            applicable_negotiation_days: negotiation_days,
            statement_of_interest_window_days: statement_window_days,
            topa_applies: true,
            civil_cause_of_action_exposure: true,
            notes,
            citations,
        };
    }

    if matches!(input.property_type, PropertyType::FivePlusUnit)
        && !input.tenant_association_pre_existing
        && input.days_in_cooling_off_period_so_far < TOPA_5_PLUS_COOLING_OFF_NO_ASSOC_DAYS
        && input.third_party_contract_signed
    {
        notes.push(format!(
            "Landlord signed third-party contract during {}-day cooling-off period — violation of § 42-3404.02 right of first refusal.",
            TOPA_5_PLUS_COOLING_OFF_NO_ASSOC_DAYS
        ));
        return Output {
            severity: Severity::ViolationFailedToObserveCoolingOffPeriod45Days,
            applicable_negotiation_days: negotiation_days,
            statement_of_interest_window_days: statement_window_days,
            topa_applies: true,
            civil_cause_of_action_exposure: true,
            notes,
            citations,
        };
    }

    if input.third_party_contract_signed && !input.right_of_first_refusal_provided {
        notes.push("Landlord signed third-party contract without honoring tenant right of first refusal at third-party terms — per se § 42-3404.02 violation.".to_string());
        return Output {
            severity: Severity::ViolationFailedToHonorRightOfFirstRefusalOnThirdPartyContract,
            applicable_negotiation_days: negotiation_days,
            statement_of_interest_window_days: statement_window_days,
            topa_applies: true,
            civil_cause_of_action_exposure: true,
            notes,
            citations,
        };
    }

    if input.third_party_contract_signed && input.right_of_first_refusal_provided {
        notes.push("Right of first refusal properly provided to tenants on third-party contract terms.".to_string());
        return Output {
            severity: Severity::CompliantRightOfFirstRefusalRespectedOnThirdPartyContract,
            applicable_negotiation_days: negotiation_days,
            statement_of_interest_window_days: statement_window_days,
            topa_applies: true,
            civil_cause_of_action_exposure: false,
            notes,
            citations,
        };
    }

    if matches!(input.property_type, PropertyType::FivePlusUnit) {
        notes.push(format!(
            "TOPA 5+ unit process active: {}-day {} window + {}-day negotiation.",
            statement_window_days,
            if input.tenant_association_pre_existing { "Statement of Interest" } else { "cooling-off" },
            negotiation_days
        ));
        return Output {
            severity: Severity::Compliant5PlusUnitOfferOfSaleProvidedFullProcess,
            applicable_negotiation_days: negotiation_days,
            statement_of_interest_window_days: statement_window_days,
            topa_applies: true,
            civil_cause_of_action_exposure: false,
            notes,
            citations,
        };
    }

    notes.push(format!(
        "TOPA 2-4 unit negotiation period active ({}-day window).",
        negotiation_days
    ));
    Output {
        severity: Severity::Compliant2To4UnitNegotiationPeriodActive,
        applicable_negotiation_days: negotiation_days,
        statement_of_interest_window_days: statement_window_days,
        topa_applies: true,
        civil_cause_of_action_exposure: false,
        notes,
        citations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_5_plus_unit_compliant() -> Input {
        Input {
            property_type: PropertyType::FivePlusUnit,
            corporate_owner_or_multi_property_owner: true,
            offer_of_sale_provided_to_tenants: true,
            days_since_offer_of_sale: 50,
            tenant_association_pre_existing: true,
            statement_of_interest_filed: true,
            days_in_cooling_off_period_so_far: 30,
            third_party_contract_signed: false,
            right_of_first_refusal_provided: false,
            tenant_organization_registered_with_dhcd: true,
            tenant_organization_assigned_rights_within_45_days: false,
            current_year: 2026,
        }
    }

    #[test]
    fn five_plus_unit_full_process_compliant() {
        let out = check(&base_5_plus_unit_compliant());
        assert_eq!(
            out.severity,
            Severity::Compliant5PlusUnitOfferOfSaleProvidedFullProcess
        );
        assert!(out.topa_applies);
        assert_eq!(out.applicable_negotiation_days, 120);
        assert_eq!(out.statement_of_interest_window_days, 30);
    }

    #[test]
    fn single_family_home_exempt_post_2018() {
        let mut i = base_5_plus_unit_compliant();
        i.property_type = PropertyType::SingleFamilyHome;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExemptSingleFamilyHomePost2018);
        assert!(!out.topa_applies);
    }

    #[test]
    fn two_to_four_unit_non_corporate_single_property_exempt() {
        let mut i = base_5_plus_unit_compliant();
        i.property_type = PropertyType::TwoToFourUnit;
        i.corporate_owner_or_multi_property_owner = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::Exempt2To4UnitNonCorporateSingleProperty
        );
    }

    #[test]
    fn two_to_four_unit_corporate_owner_covered_with_90_day() {
        let mut i = base_5_plus_unit_compliant();
        i.property_type = PropertyType::TwoToFourUnit;
        i.corporate_owner_or_multi_property_owner = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::Compliant2To4UnitNegotiationPeriodActive
        );
        assert_eq!(out.applicable_negotiation_days, 90);
    }

    #[test]
    fn failed_to_provide_offer_of_sale_violation() {
        let mut i = base_5_plus_unit_compliant();
        i.offer_of_sale_provided_to_tenants = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationFailedToProvideOfferOfSale
        );
        assert!(out.civil_cause_of_action_exposure);
    }

    #[test]
    fn third_party_contract_with_refusal_compliant() {
        let mut i = base_5_plus_unit_compliant();
        i.third_party_contract_signed = true;
        i.right_of_first_refusal_provided = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::CompliantRightOfFirstRefusalRespectedOnThirdPartyContract
        );
    }

    #[test]
    fn third_party_contract_without_refusal_violation() {
        let mut i = base_5_plus_unit_compliant();
        i.third_party_contract_signed = true;
        i.right_of_first_refusal_provided = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationFailedToHonorRightOfFirstRefusalOnThirdPartyContract
        );
        assert!(out.civil_cause_of_action_exposure);
    }

    #[test]
    fn five_plus_existing_association_30_day_statement_window() {
        let mut i = base_5_plus_unit_compliant();
        i.tenant_association_pre_existing = true;
        let out = check(&i);
        assert_eq!(out.statement_of_interest_window_days, 30);
    }

    #[test]
    fn five_plus_no_association_45_day_cooling_off_window() {
        let mut i = base_5_plus_unit_compliant();
        i.tenant_association_pre_existing = false;
        i.days_in_cooling_off_period_so_far = 50;
        let out = check(&i);
        assert_eq!(out.statement_of_interest_window_days, 45);
    }

    #[test]
    fn cooling_off_period_violation_third_party_contract_within_45_days() {
        let mut i = base_5_plus_unit_compliant();
        i.tenant_association_pre_existing = false;
        i.days_in_cooling_off_period_so_far = 30;
        i.third_party_contract_signed = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationFailedToObserveCoolingOffPeriod45Days
        );
    }

    #[test]
    fn tenant_org_assigned_rights_without_dhcd_registration_violation() {
        let mut i = base_5_plus_unit_compliant();
        i.tenant_organization_assigned_rights_within_45_days = true;
        i.tenant_organization_registered_with_dhcd = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationTenantOrganizationAssignedRightsWithoutDhcdRegistration
        );
    }

    #[test]
    fn citations_pin_topa_dc_code_subsections() {
        let out = check(&base_5_plus_unit_compliant());
        assert!(out.citations.iter().any(|c| c.contains("§ 42-3401.01")));
        assert!(out.citations.iter().any(|c| c.contains("§ 42-3404.02")));
        assert!(out.citations.iter().any(|c| c.contains("§ 42-3404.11")));
        assert!(out.citations.iter().any(|c| c.contains("§ 42-3405.03")));
        assert!(out.citations.iter().any(|c| c.contains("§ 42-3509.01")));
    }

    #[test]
    fn citations_pin_d_c_law_3_86_and_rental_act_2025() {
        let out = check(&base_5_plus_unit_compliant());
        assert!(out.citations.iter().any(|c| c.contains("D.C. Law 3-86")));
        assert!(out.citations.iter().any(|c| c.contains("RENTAL Act of 2025")));
        assert!(out.citations.iter().any(|c| c.contains("Sep 17, 2025")));
    }

    #[test]
    fn citations_pin_single_family_exemption_2018() {
        let out = check(&base_5_plus_unit_compliant());
        assert!(out.citations.iter().any(|c| c.contains("Single-Family Home Exemption")));
        assert!(out.citations.iter().any(|c| c.contains("July 3, 2018")));
    }

    #[test]
    fn constant_pin_90_day_2_4_negotiation() {
        assert_eq!(TOPA_2_TO_4_NEGOTIATION_DAYS, 90);
    }

    #[test]
    fn constant_pin_120_day_5_plus_negotiation() {
        assert_eq!(TOPA_5_PLUS_NEGOTIATION_DAYS, 120);
    }

    #[test]
    fn constant_pin_45_day_cooling_off() {
        assert_eq!(TOPA_5_PLUS_COOLING_OFF_NO_ASSOC_DAYS, 45);
    }

    #[test]
    fn constant_pin_30_day_existing_assoc_statement_window() {
        assert_eq!(TOPA_5_PLUS_STATEMENT_OF_INTEREST_EXISTING_ASSOC_DAYS, 30);
    }

    #[test]
    fn constant_pin_5_plus_unit_threshold() {
        assert_eq!(TOPA_5_PLUS_UNIT_THRESHOLD, 5);
    }

    #[test]
    fn constant_pin_15_day_third_party_extension() {
        assert_eq!(TOPA_THIRD_PARTY_CONTRACT_EXTENSION_DAYS, 15);
    }

    #[test]
    fn constant_pin_120_240_day_financing_window() {
        assert_eq!(TOPA_FINANCING_DAYS_MIN, 120);
        assert_eq!(TOPA_FINANCING_DAYS_MAX, 240);
    }

    #[test]
    fn constant_pin_single_family_exemption_2018_year() {
        assert_eq!(TOPA_SINGLE_FAMILY_EXEMPTION_EFFECTIVE_YEAR, 2018);
    }

    #[test]
    fn constant_pin_rental_act_2025_effective_date() {
        assert_eq!(RENTAL_ACT_2025_EFFECTIVE_YEAR, 2025);
        assert_eq!(RENTAL_ACT_2025_EFFECTIVE_MONTH, 12);
        assert_eq!(RENTAL_ACT_2025_EFFECTIVE_DAY, 31);
    }

    #[test]
    fn constant_pin_topa_1980_original_enactment() {
        assert_eq!(TOPA_ORIGINAL_ENACTMENT_YEAR, 1980);
    }
}
