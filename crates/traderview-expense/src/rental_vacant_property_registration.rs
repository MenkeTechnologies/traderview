//! Multi-Jurisdictional Vacant Property Registration Compliance.
//!
//! Pure-compute check for whether a residential property owner (or
//! mortgagee following foreclosure) has registered a vacant
//! property as required by city / state ordinance, paid the
//! registration fee, and satisfied renewal obligations. Trader-
//! landlord critical because vacant-property registration failures
//! trigger doubled fees, code-enforcement liens, daily penalties,
//! and reputational exposure for portfolio operators.
//!
//! Web research (verified 2026-06-03):
//! - **Chicago Municipal Code § 13-12-125** (owner): owner must
//!   register within 30 days after the building becomes vacant or
//!   within 30 days after assuming ownership of a vacant building.
//!   Registration / renewal fee per registered building = $300;
//!   renewal every 6 months. Fee DOUBLED if registration takes
//!   place as a result of a City identification of violation
//!   rather than voluntary timely compliance. Inspector +
//!   attorney's fees recoverable as a LIEN on the property.
//!   (City of Chicago Department of Buildings — Section
//!   13-12-125; DAWGS Inc. Chicago Vacant Property Ordinances.)
//! - **Chicago Municipal Code § 13-12-126** (mortgagee): mortgagee
//!   of a vacant unregistered building must file within the LATER
//!   of 30 days after the building becomes vacant and unregistered
//!   OR 10 days after default. Initial fee = $700; renewal every
//!   6 months at $300.
//! - **Detroit Vacant Property Registration Ordinance**: owners
//!   must register vacant properties with **BSEED** (Buildings,
//!   Safety Engineering and Environmental Department) within 30
//!   days of vacancy. Owners must maintain the property's exterior
//!   to prevent it from becoming a nuisance or hazard. (Detroit MI
//!   Vacant Property Registration Ordinance Fact Sheet.)
//! - **Common municipal requirements**: 30-day threshold; annual
//!   or semi-annual fee; local agent designation for service of
//!   process; maintenance plan; access for inspection; lien rights
//!   for code-enforcement costs; daily penalties for non-
//!   registration accumulating until cure.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const CHICAGO_13_12_125_OWNER_REGISTRATION_DAYS: u32 = 30;
pub const CHICAGO_13_12_125_OWNER_FEE_DOLLARS: u64 = 300;
pub const CHICAGO_13_12_125_RENEWAL_INTERVAL_MONTHS: u32 = 6;
pub const CHICAGO_13_12_125_CITY_IDENTIFIED_FEE_MULTIPLIER: u64 = 2;
pub const CHICAGO_13_12_126_MORTGAGEE_REGISTRATION_DAYS: u32 = 30;
pub const CHICAGO_13_12_126_MORTGAGEE_INITIAL_FEE_DOLLARS: u64 = 700;
pub const CHICAGO_13_12_126_MORTGAGEE_RENEWAL_FEE_DOLLARS: u64 = 300;
pub const CHICAGO_13_12_126_MORTGAGEE_POST_DEFAULT_DAYS: u32 = 10;
pub const DETROIT_BSEED_REGISTRATION_DAYS: u32 = 30;
pub const DEFAULT_REGISTRATION_DAYS: u32 = 30;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VacantRegistrationJurisdiction {
    ChicagoMunicipalCode,
    DetroitBSEED,
    ClevelandOMC,
    BaltimoreBuildingCode,
    PhiladelphiaVacantPropertyStrategy,
    OtherJurisdictionWithoutVacantRegistration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RegistrationActorRole {
    OwnerOfRecord,
    MortgageeAfterForeclosureOrDefault,
    PropertyManagerOnOwnerBehalf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscoveryMode {
    VoluntaryTimelyCompliance,
    CityIdentifiedViolation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VacantPropertyRegistrationMode {
    NotApplicableUnitNotVacant,
    NotApplicableLessThan30DaysVacant,
    NotApplicableJurisdictionLacksMandate,
    CompliantOwnerRegisteredWithin30DaysAndFeePaid,
    CompliantMortgageeRegisteredAfterForeclosure,
    CompliantRenewalFilingTimely,
    ViolationOwnerFailedToRegisterWithin30Days,
    ViolationDoubleFeeAppliedForCityIdentifiedNonCompliance,
    ViolationMortgageeFailedToRegisterAfterForeclosure,
    ViolationRenewalNotFiledEverySixMonths,
    ViolationLocalAgentNotDesignated,
    ViolationCodeEnforcementLienAccruedFromNonRegistration,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub jurisdiction: VacantRegistrationJurisdiction,
    pub actor_role: RegistrationActorRole,
    pub days_property_has_been_vacant: u32,
    pub days_to_registration_filing: u32,
    pub discovery_mode: DiscoveryMode,
    pub registration_fee_paid_dollars: u64,
    pub local_agent_designated_for_service: bool,
    pub months_since_initial_registration: u32,
    pub renewal_filed_within_required_interval: bool,
    pub code_enforcement_lien_accrued_dollars: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: VacantPropertyRegistrationMode,
    pub required_fee_dollars: u64,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type RentalVacantPropertyRegistrationInput = Input;
pub type RentalVacantPropertyRegistrationOutput = Output;
pub type RentalVacantPropertyRegistrationResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

fn required_initial_fee_for(jurisdiction: VacantRegistrationJurisdiction, role: RegistrationActorRole) -> u64 {
    match (jurisdiction, role) {
        (VacantRegistrationJurisdiction::ChicagoMunicipalCode, RegistrationActorRole::OwnerOfRecord)
        | (VacantRegistrationJurisdiction::ChicagoMunicipalCode, RegistrationActorRole::PropertyManagerOnOwnerBehalf) => {
            CHICAGO_13_12_125_OWNER_FEE_DOLLARS
        }
        (VacantRegistrationJurisdiction::ChicagoMunicipalCode, RegistrationActorRole::MortgageeAfterForeclosureOrDefault) => {
            CHICAGO_13_12_126_MORTGAGEE_INITIAL_FEE_DOLLARS
        }
        _ => CHICAGO_13_12_125_OWNER_FEE_DOLLARS,
    }
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "Chicago Municipal Code § 13-12-125 — owner of vacant building must register within 30 days of vacancy or 30 days of assuming ownership; $300 fee per registered building; renewal every 6 months; DOUBLED for City-identified non-compliance; lien for inspector + attorney fees".to_string(),
        "Chicago Municipal Code § 13-12-126 — mortgagee of vacant unregistered building must register within later of 30 days after vacancy or 10 days after default; $700 initial fee; $300 every-6-month renewal".to_string(),
        "Detroit Vacant Property Registration Ordinance — BSEED (Buildings, Safety Engineering and Environmental Department); 30-day registration after vacancy; exterior maintenance prevents nuisance/hazard".to_string(),
        "Cleveland OMC § 367.131 — Cleveland Department of Building and Housing vacant property registration".to_string(),
        "Baltimore Building Code § 102.5 — vacant house notice + registration".to_string(),
        "Philadelphia Vacant Property Strategy — Department of Licenses and Inspections (L&I) vacant property compliance".to_string(),
        "Common municipal requirements: 30-day threshold; annual or semi-annual fee; local agent for service of process; maintenance plan; access for inspection; lien for code-enforcement costs; daily accrual until cure".to_string(),
    ];

    if input.days_property_has_been_vacant == 0 {
        return Output {
            mode: VacantPropertyRegistrationMode::NotApplicableUnitNotVacant,
            required_fee_dollars: 0,
            statutory_basis: "Property not vacant".to_string(),
            notes: "Property is occupied; vacant-property registration ordinance inapplicable.".to_string(),
            citations,
        };
    }

    if input.days_property_has_been_vacant < DEFAULT_REGISTRATION_DAYS {
        return Output {
            mode: VacantPropertyRegistrationMode::NotApplicableLessThan30DaysVacant,
            required_fee_dollars: 0,
            statutory_basis: "Less than 30 days of vacancy; registration not yet triggered".to_string(),
            notes: format!(
                "Property has been vacant {} days; below 30-day registration trigger. No registration obligation yet.",
                input.days_property_has_been_vacant
            ),
            citations,
        };
    }

    if input.jurisdiction == VacantRegistrationJurisdiction::OtherJurisdictionWithoutVacantRegistration {
        return Output {
            mode: VacantPropertyRegistrationMode::NotApplicableJurisdictionLacksMandate,
            required_fee_dollars: 0,
            statutory_basis: "Jurisdiction lacks vacant-property registration ordinance".to_string(),
            notes: "Jurisdiction does not impose vacant-property registration; no statutory obligation arises.".to_string(),
            citations,
        };
    }

    let required_fee_base = required_initial_fee_for(input.jurisdiction, input.actor_role);
    let required_fee = if input.discovery_mode == DiscoveryMode::CityIdentifiedViolation
        && input.jurisdiction == VacantRegistrationJurisdiction::ChicagoMunicipalCode
        && input.actor_role != RegistrationActorRole::MortgageeAfterForeclosureOrDefault
    {
        required_fee_base.saturating_mul(CHICAGO_13_12_125_CITY_IDENTIFIED_FEE_MULTIPLIER)
    } else {
        required_fee_base
    };

    if input.actor_role == RegistrationActorRole::MortgageeAfterForeclosureOrDefault
        && input.days_to_registration_filing > CHICAGO_13_12_126_MORTGAGEE_REGISTRATION_DAYS
    {
        return Output {
            mode: VacantPropertyRegistrationMode::ViolationMortgageeFailedToRegisterAfterForeclosure,
            required_fee_dollars: required_fee,
            statutory_basis: "Chicago § 13-12-126 — mortgagee 30-day deadline missed".to_string(),
            notes: format!(
                "VIOLATION: Chicago § 13-12-126 requires mortgagee to register within later of 30 days after vacancy + unregistered status OR 10 days after default; mortgagee filed at {} days.",
                input.days_to_registration_filing
            ),
            citations,
        };
    }

    if input.days_to_registration_filing > DEFAULT_REGISTRATION_DAYS {
        if input.discovery_mode == DiscoveryMode::CityIdentifiedViolation
            && input.jurisdiction == VacantRegistrationJurisdiction::ChicagoMunicipalCode
        {
            return Output {
                mode: VacantPropertyRegistrationMode::ViolationDoubleFeeAppliedForCityIdentifiedNonCompliance,
                required_fee_dollars: required_fee,
                statutory_basis: "Chicago § 13-12-125 — fee doubled for City-identified non-compliance".to_string(),
                notes: format!(
                    "VIOLATION: Chicago § 13-12-125 doubles registration fee when registration occurs through City-identified violation rather than voluntary timely compliance. Required fee = ${} (2× base of ${}).",
                    required_fee, required_fee_base
                ),
                citations,
            };
        }
        return Output {
            mode: VacantPropertyRegistrationMode::ViolationOwnerFailedToRegisterWithin30Days,
            required_fee_dollars: required_fee,
            statutory_basis: format!(
                "{:?} requires registration within 30 days of vacancy",
                input.jurisdiction
            ),
            notes: format!(
                "VIOLATION: owner failed to register within 30-day window; filed at {} days after vacancy commencement.",
                input.days_to_registration_filing
            ),
            citations,
        };
    }

    if input.registration_fee_paid_dollars < required_fee {
        return Output {
            mode: VacantPropertyRegistrationMode::ViolationOwnerFailedToRegisterWithin30Days,
            required_fee_dollars: required_fee,
            statutory_basis: "Required registration fee not paid in full".to_string(),
            notes: format!(
                "VIOLATION: actor paid ${} but required fee = ${} (shortfall ${}).",
                input.registration_fee_paid_dollars,
                required_fee,
                required_fee.saturating_sub(input.registration_fee_paid_dollars)
            ),
            citations,
        };
    }

    if !input.local_agent_designated_for_service
        && input.jurisdiction != VacantRegistrationJurisdiction::OtherJurisdictionWithoutVacantRegistration
    {
        return Output {
            mode: VacantPropertyRegistrationMode::ViolationLocalAgentNotDesignated,
            required_fee_dollars: required_fee,
            statutory_basis: "Local agent for service of process required".to_string(),
            notes: "VIOLATION: local agent for service of process not designated; required across vacant-property registration regimes.".to_string(),
            citations,
        };
    }

    if input.months_since_initial_registration >= CHICAGO_13_12_125_RENEWAL_INTERVAL_MONTHS
        && !input.renewal_filed_within_required_interval
    {
        return Output {
            mode: VacantPropertyRegistrationMode::ViolationRenewalNotFiledEverySixMonths,
            required_fee_dollars: CHICAGO_13_12_126_MORTGAGEE_RENEWAL_FEE_DOLLARS,
            statutory_basis: "Chicago § 13-12-125 / § 13-12-126 — every-6-month renewal required".to_string(),
            notes: format!(
                "VIOLATION: {} months have elapsed since initial registration; required 6-month renewal not filed.",
                input.months_since_initial_registration
            ),
            citations,
        };
    }

    if input.code_enforcement_lien_accrued_dollars > 0 {
        return Output {
            mode: VacantPropertyRegistrationMode::ViolationCodeEnforcementLienAccruedFromNonRegistration,
            required_fee_dollars: required_fee,
            statutory_basis: "Inspector + attorney fees recoverable as lien on property".to_string(),
            notes: format!(
                "VIOLATION: code-enforcement lien of ${} accrued from non-registration; recoverable from owner + attached to property.",
                input.code_enforcement_lien_accrued_dollars
            ),
            citations,
        };
    }

    if input.actor_role == RegistrationActorRole::MortgageeAfterForeclosureOrDefault {
        return Output {
            mode: VacantPropertyRegistrationMode::CompliantMortgageeRegisteredAfterForeclosure,
            required_fee_dollars: required_fee,
            statutory_basis: "Chicago § 13-12-126 — mortgagee registration satisfied".to_string(),
            notes: format!(
                "COMPLIANT: mortgagee registered at {} days post-vacancy with ${} initial fee paid.",
                input.days_to_registration_filing, input.registration_fee_paid_dollars
            ),
            citations,
        };
    }

    if input.months_since_initial_registration >= CHICAGO_13_12_125_RENEWAL_INTERVAL_MONTHS
        && input.renewal_filed_within_required_interval
    {
        return Output {
            mode: VacantPropertyRegistrationMode::CompliantRenewalFilingTimely,
            required_fee_dollars: required_fee,
            statutory_basis: "Renewal filed within 6-month interval".to_string(),
            notes: format!(
                "COMPLIANT: renewal filed at {} months (within 6-month interval).",
                input.months_since_initial_registration
            ),
            citations,
        };
    }

    Output {
        mode: VacantPropertyRegistrationMode::CompliantOwnerRegisteredWithin30DaysAndFeePaid,
        required_fee_dollars: required_fee,
        statutory_basis: format!(
            "{:?} 30-day registration window satisfied; fee ${} paid",
            input.jurisdiction, required_fee
        ),
        notes: format!(
            "COMPLIANT: owner registered at {} days after vacancy; required fee = ${}; paid ${}; local agent designated.",
            input.days_to_registration_filing, required_fee, input.registration_fee_paid_dollars
        ),
        citations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_chicago_owner_compliant() -> Input {
        Input {
            jurisdiction: VacantRegistrationJurisdiction::ChicagoMunicipalCode,
            actor_role: RegistrationActorRole::OwnerOfRecord,
            days_property_has_been_vacant: 35,
            days_to_registration_filing: 25,
            discovery_mode: DiscoveryMode::VoluntaryTimelyCompliance,
            registration_fee_paid_dollars: 300,
            local_agent_designated_for_service: true,
            months_since_initial_registration: 0,
            renewal_filed_within_required_interval: false,
            code_enforcement_lien_accrued_dollars: 0,
        }
    }

    #[test]
    fn property_not_vacant_not_applicable() {
        let input = Input {
            days_property_has_been_vacant: 0,
            ..baseline_chicago_owner_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, VacantPropertyRegistrationMode::NotApplicableUnitNotVacant);
    }

    #[test]
    fn less_than_30_days_vacant_not_applicable() {
        let input = Input {
            days_property_has_been_vacant: 25,
            ..baseline_chicago_owner_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, VacantPropertyRegistrationMode::NotApplicableLessThan30DaysVacant);
    }

    #[test]
    fn at_exactly_30_days_vacant_registration_triggered() {
        let input = Input {
            days_property_has_been_vacant: 30,
            ..baseline_chicago_owner_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, VacantPropertyRegistrationMode::CompliantOwnerRegisteredWithin30DaysAndFeePaid);
    }

    #[test]
    fn other_jurisdiction_not_applicable() {
        let input = Input {
            jurisdiction: VacantRegistrationJurisdiction::OtherJurisdictionWithoutVacantRegistration,
            ..baseline_chicago_owner_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, VacantPropertyRegistrationMode::NotApplicableJurisdictionLacksMandate);
    }

    #[test]
    fn chicago_owner_30_day_compliant() {
        let result = check(&baseline_chicago_owner_compliant());
        assert_eq!(result.mode, VacantPropertyRegistrationMode::CompliantOwnerRegisteredWithin30DaysAndFeePaid);
        assert_eq!(result.required_fee_dollars, 300);
    }

    #[test]
    fn chicago_owner_filed_at_31_days_violation() {
        let input = Input {
            days_to_registration_filing: 31,
            ..baseline_chicago_owner_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, VacantPropertyRegistrationMode::ViolationOwnerFailedToRegisterWithin30Days);
    }

    #[test]
    fn chicago_city_identified_double_fee_violation() {
        let input = Input {
            days_to_registration_filing: 40,
            discovery_mode: DiscoveryMode::CityIdentifiedViolation,
            registration_fee_paid_dollars: 300,
            ..baseline_chicago_owner_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, VacantPropertyRegistrationMode::ViolationDoubleFeeAppliedForCityIdentifiedNonCompliance);
        assert_eq!(result.required_fee_dollars, 600);
    }

    #[test]
    fn chicago_mortgagee_30_day_compliant() {
        let input = Input {
            actor_role: RegistrationActorRole::MortgageeAfterForeclosureOrDefault,
            registration_fee_paid_dollars: 700,
            ..baseline_chicago_owner_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, VacantPropertyRegistrationMode::CompliantMortgageeRegisteredAfterForeclosure);
        assert_eq!(result.required_fee_dollars, 700);
    }

    #[test]
    fn chicago_mortgagee_31_days_violation() {
        let input = Input {
            actor_role: RegistrationActorRole::MortgageeAfterForeclosureOrDefault,
            days_to_registration_filing: 31,
            registration_fee_paid_dollars: 700,
            ..baseline_chicago_owner_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, VacantPropertyRegistrationMode::ViolationMortgageeFailedToRegisterAfterForeclosure);
    }

    #[test]
    fn chicago_fee_shortfall_violation() {
        let input = Input {
            registration_fee_paid_dollars: 200,
            ..baseline_chicago_owner_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, VacantPropertyRegistrationMode::ViolationOwnerFailedToRegisterWithin30Days);
        assert_eq!(result.required_fee_dollars, 300);
    }

    #[test]
    fn no_local_agent_designated_violation() {
        let input = Input {
            local_agent_designated_for_service: false,
            ..baseline_chicago_owner_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, VacantPropertyRegistrationMode::ViolationLocalAgentNotDesignated);
    }

    #[test]
    fn renewal_not_filed_after_6_months_violation() {
        let input = Input {
            months_since_initial_registration: 7,
            renewal_filed_within_required_interval: false,
            ..baseline_chicago_owner_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, VacantPropertyRegistrationMode::ViolationRenewalNotFiledEverySixMonths);
    }

    #[test]
    fn renewal_filed_at_6_months_compliant() {
        let input = Input {
            months_since_initial_registration: 6,
            renewal_filed_within_required_interval: true,
            ..baseline_chicago_owner_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, VacantPropertyRegistrationMode::CompliantRenewalFilingTimely);
    }

    #[test]
    fn code_enforcement_lien_accrued_violation() {
        let input = Input {
            code_enforcement_lien_accrued_dollars: 5_000,
            ..baseline_chicago_owner_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, VacantPropertyRegistrationMode::ViolationCodeEnforcementLienAccruedFromNonRegistration);
    }

    #[test]
    fn detroit_bseed_30_day_compliant() {
        let input = Input {
            jurisdiction: VacantRegistrationJurisdiction::DetroitBSEED,
            ..baseline_chicago_owner_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, VacantPropertyRegistrationMode::CompliantOwnerRegisteredWithin30DaysAndFeePaid);
    }

    #[test]
    fn detroit_31_days_violation() {
        let input = Input {
            jurisdiction: VacantRegistrationJurisdiction::DetroitBSEED,
            days_to_registration_filing: 31,
            ..baseline_chicago_owner_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, VacantPropertyRegistrationMode::ViolationOwnerFailedToRegisterWithin30Days);
    }

    #[test]
    fn cleveland_baltimore_philadelphia_30_day_compliant() {
        for jurisdiction in [
            VacantRegistrationJurisdiction::ClevelandOMC,
            VacantRegistrationJurisdiction::BaltimoreBuildingCode,
            VacantRegistrationJurisdiction::PhiladelphiaVacantPropertyStrategy,
        ] {
            let input = Input {
                jurisdiction,
                ..baseline_chicago_owner_compliant()
            };
            let result = check(&input);
            assert_eq!(result.mode, VacantPropertyRegistrationMode::CompliantOwnerRegisteredWithin30DaysAndFeePaid);
        }
    }

    #[test]
    fn citations_pin_chicago_detroit_cleveland_baltimore_philly() {
        let result = check(&baseline_chicago_owner_compliant());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("Chicago Municipal Code § 13-12-125"));
        assert!(joined.contains("Chicago Municipal Code § 13-12-126"));
        assert!(joined.contains("Detroit"));
        assert!(joined.contains("BSEED"));
        assert!(joined.contains("Cleveland OMC"));
        assert!(joined.contains("Baltimore Building Code"));
        assert!(joined.contains("Philadelphia"));
        assert!(joined.contains("$300"));
        assert!(joined.contains("$700"));
        assert!(joined.contains("DOUBLED"));
    }

    #[test]
    fn constant_pin_chicago_fees_and_intervals() {
        assert_eq!(CHICAGO_13_12_125_OWNER_REGISTRATION_DAYS, 30);
        assert_eq!(CHICAGO_13_12_125_OWNER_FEE_DOLLARS, 300);
        assert_eq!(CHICAGO_13_12_125_RENEWAL_INTERVAL_MONTHS, 6);
        assert_eq!(CHICAGO_13_12_125_CITY_IDENTIFIED_FEE_MULTIPLIER, 2);
        assert_eq!(CHICAGO_13_12_126_MORTGAGEE_REGISTRATION_DAYS, 30);
        assert_eq!(CHICAGO_13_12_126_MORTGAGEE_INITIAL_FEE_DOLLARS, 700);
        assert_eq!(CHICAGO_13_12_126_MORTGAGEE_RENEWAL_FEE_DOLLARS, 300);
        assert_eq!(CHICAGO_13_12_126_MORTGAGEE_POST_DEFAULT_DAYS, 10);
        assert_eq!(DETROIT_BSEED_REGISTRATION_DAYS, 30);
        assert_eq!(DEFAULT_REGISTRATION_DAYS, 30);
    }
}
