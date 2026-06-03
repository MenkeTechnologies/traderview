//! California AB 12 of 2023 Security Deposit Cap Compliance
//! Module (Cal. Civ. Code § 1950.5 as amended effective July
//! 1, 2024).
//!
//! Pure-compute check for landlord compliance with California
//! Assembly Bill 12 of the 2023-2024 Regular Session, which
//! amended Cal. Civ. Code § 1950.5 to reduce the maximum
//! residential security deposit from 2 months' rent (unfurnished)
//! / 3 months' rent (furnished) to **1 month's rent** regardless
//! of furnishing status. Signed by Governor Gavin Newsom on
//! October 11, 2023; effective July 1, 2024. Includes the
//! **small landlord exception** (natural person / natural-person-
//! member LLC with ≤ 2 properties / ≤ 4 dwelling units → 2 months
//! cap) and the **service member override** (military tenants
//! always 1 month regardless of landlord size).
//!
//! Web research (verified 2026-06-03):
//! - **California AB 12 of 2023** (Assembly Bill 12, Haney;
//!   2023-2024 Regular Session) — signed by Governor Gavin
//!   Newsom on **October 11, 2023**; effective **July 1, 2024**
//!   (9-month transition window) ([CA Legislative Information
//!   AB 12](https://leginfo.legislature.ca.gov/faces/billNavClient.xhtml?bill_id=202320240AB12);
//!   [California Security Deposit Limits Effective July 1, 2024
//!   — Brownstein](https://www.bhfs.com/insight/california-security-deposit-limits-effective-july-1-2024/)).
//! - **General Rule (Cal. Civ. Code § 1950.5(c)(1))**: landlord
//!   may not demand or receive security for a rental agreement
//!   for residential property in an amount in excess of **ONE
//!   MONTH'S RENT** regardless of whether the residential
//!   property is unfurnished or furnished.
//! - **Pre-AB 12 Caps (now reduced)**: 2 months' rent for
//!   unfurnished + 3 months' rent for furnished — both
//!   replaced by uniform 1-month cap.
//! - **Small Landlord Exception (Cal. Civ. Code § 1950.5(c)(4))**:
//!   if owner is BOTH (1) a natural person OR a limited liability
//!   corporation in which ALL members are natural persons AND
//!   (2) owns no more than **2 residential rental properties**
//!   collectively including no more than **4 dwelling units
//!   offered for rent**, the maximum security deposit may
//!   exceed 1 month but may NOT exceed **TWO MONTHS' RENT**
//!   regardless of furnished/unfurnished status.
//! - **Service Member Override (Cal. Civ. Code § 1950.5(c)(4)(B))**:
//!   if the tenant is a member of the military service
//!   ("service member" as defined in Cal. Mil. & Vet. Code
//!   § 400), the cap is **STRICTLY ONE MONTH'S RENT regardless
//!   of how many properties the landlord owns** — the small
//!   landlord exception does NOT apply.
//! - **Transition Rule**: AB 12 applies to security deposits
//!   collected on or after **July 1, 2024**. Pre-July 1, 2024
//!   security deposits lawfully collected at higher pre-AB 12
//!   caps remain valid and do NOT need to be partially refunded
//!   mid-tenancy ([Stormoen Law — Landlord-Tenant Maximum
//!   Security Deposit in California](https://stormoenlaw.com/blog/2024/9/9/cq32lqugi4gili56gfg2nmq2obawt5)).
//! - **Trigger Events for New Cap**: cap activates on (1) new
//!   tenancy after original tenant vacates; (2) written lease
//!   renewal; (3) material lease modification.
//! - **Remedies for Excessive Deposit**: tenant may recover
//!   excess deposit + bad-faith retention damages under Cal.
//!   Civ. Code § 1950.5(l) (up to twice the security deposit
//!   amount as statutory damages plus actual damages).

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const CA_AB_12_SIGNED_DATE_YEAR: u32 = 2023;
pub const CA_AB_12_SIGNED_DATE_MONTH: u32 = 10;
pub const CA_AB_12_SIGNED_DATE_DAY: u32 = 11;
pub const CA_AB_12_EFFECTIVE_DATE_YEAR: u32 = 2024;
pub const CA_AB_12_EFFECTIVE_DATE_MONTH: u32 = 7;
pub const CA_AB_12_EFFECTIVE_DATE_DAY: u32 = 1;
pub const CA_AB_12_GENERAL_CAP_MONTHS_RENT: u32 = 1;
pub const CA_AB_12_SMALL_LANDLORD_CAP_MONTHS_RENT: u32 = 2;
pub const CA_AB_12_SMALL_LANDLORD_MAX_PROPERTIES: u32 = 2;
pub const CA_AB_12_SMALL_LANDLORD_MAX_DWELLING_UNITS: u32 = 4;
pub const CA_AB_12_PRE_AB12_UNFURNISHED_CAP_MONTHS_RENT: u32 = 2;
pub const CA_AB_12_PRE_AB12_FURNISHED_CAP_MONTHS_RENT: u32 = 3;
pub const CA_CIV_CODE_1950_5_L_STATUTORY_DAMAGES_MULTIPLIER: u32 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LandlordEntityType {
    NaturalPerson,
    LimitedLiabilityCompanyAllMembersNaturalPersons,
    CorporationOrAnyEntityWithNonNaturalPersonMember,
    PartnershipOrLpWithNonNaturalPersonMember,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TenantClassification {
    StandardResidentialTenant,
    MilitaryServiceMember,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LeaseTriggerEvent {
    PreJuly1_2024LeaseStillInPlace,
    NewTenancyAfterOriginalTenantVacated,
    WrittenLeaseRenewal,
    MaterialLeaseModification,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FurnishingStatus {
    UnfurnishedUnit,
    FurnishedUnit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CaAb12Mode {
    NotApplicablePropertyOutsideCalifornia,
    NotApplicablePreAb12LeaseLawfullyGrandfathered,
    CompliantPostAb12GeneralCapOneMonthRent,
    CompliantSmallLandlordExceptionTwoMonthRentCap,
    CompliantServiceMemberOverrideOneMonthCapRegardlessOfLandlordSize,
    ViolationPostAb12DepositExceedsOneMonthCap,
    ViolationSmallLandlordExceptionDepositExceedsTwoMonthCap,
    ViolationServiceMemberDepositExceedsOneMonthSmallLandlordExceptionInapplicable,
    ViolationSmallLandlordExceptionClaimedButPropertyCountExceedsTwo,
    ViolationSmallLandlordExceptionClaimedButDwellingUnitsExceedsFour,
    ViolationSmallLandlordExceptionClaimedButEntityTypeIneligible,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub property_in_california: bool,
    pub lease_trigger_event: LeaseTriggerEvent,
    pub landlord_entity_type: LandlordEntityType,
    pub landlord_residential_property_count: u32,
    pub landlord_total_dwelling_units_for_rent: u32,
    pub tenant_classification: TenantClassification,
    pub furnishing_status: FurnishingStatus,
    pub monthly_rent_cents: u64,
    pub security_deposit_collected_cents: u64,
    pub small_landlord_exception_claimed: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: CaAb12Mode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
    pub applicable_cap_cents: u64,
}

pub type RentalCaliforniaAb12SecurityDepositCapInput = Input;
pub type RentalCaliforniaAb12SecurityDepositCapOutput = Output;
pub type RentalCaliforniaAb12SecurityDepositCapResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "California AB 12 of 2023 (Haney; 2023-2024 Regular Session) — amends Cal. Civ. Code § 1950.5; signed by Governor Gavin Newsom on October 11, 2023; effective July 1, 2024 (9-month transition window)".to_string(),
        "Cal. Civ. Code § 1950.5(c)(1) — general 1-month rent cap; landlord may not demand or receive security in excess of one month's rent regardless of unfurnished or furnished".to_string(),
        "Cal. Civ. Code § 1950.5(c)(4) — small landlord exception; natural person OR LLC with ALL natural-person members AND ≤ 2 properties / ≤ 4 dwelling units → 2 months' rent cap regardless of furnishing".to_string(),
        "Cal. Civ. Code § 1950.5(c)(4)(B) — service member override; military service member tenant always 1 month cap regardless of landlord size; small landlord exception does NOT apply".to_string(),
        "Cal. Mil. & Vet. Code § 400 — 'service member' definition referenced in § 1950.5(c)(4)(B)".to_string(),
        "Pre-AB 12 caps (now reduced) — 2 months' rent for unfurnished + 3 months' rent for furnished (Cal. Civ. Code § 1950.5 pre-July 1, 2024)".to_string(),
        "Transition rule — AB 12 applies to security deposits collected on or after July 1, 2024; pre-July 1, 2024 lawful deposits at higher caps remain valid; new cap activates on (1) new tenancy after original tenant vacates, (2) written lease renewal, (3) material lease modification".to_string(),
        "Cal. Civ. Code § 1950.5(l) — tenant remedies for excessive deposit; recovery of excess deposit + bad-faith retention up to twice the security deposit as statutory damages plus actual damages".to_string(),
        "Stormoen Law — California Landlord-Tenant Maximum Security Deposit Analysis".to_string(),
        "Brownstein Hyatt Farber Schreck — California Security Deposit Limits Effective July 1, 2024".to_string(),
        "TLD Law — Overview of AB 12 New Security Deposit Limitations for Landlords".to_string(),
        "California Civil Code § 1950.5 — Residential Security Deposit Statute (general framework — itemization, return windows, walk-through, interest treatment)".to_string(),
    ];

    if !input.property_in_california {
        return Output {
            mode: CaAb12Mode::NotApplicablePropertyOutsideCalifornia,
            statutory_basis: "Property outside California; AB 12 / Cal. Civ. Code § 1950.5 inapplicable".to_string(),
            notes: "Property outside California; California AB 12 security deposit cap inapplicable.".to_string(),
            citations,
            applicable_cap_cents: 0,
        };
    }

    if input.lease_trigger_event == LeaseTriggerEvent::PreJuly1_2024LeaseStillInPlace {
        return Output {
            mode: CaAb12Mode::NotApplicablePreAb12LeaseLawfullyGrandfathered,
            statutory_basis: "AB 12 transition rule — pre-July 1, 2024 lawful security deposits grandfathered".to_string(),
            notes: "NOT APPLICABLE: pre-AB 12 lease with lawful security deposit at prior 2-month (unfurnished) or 3-month (furnished) cap; deposit grandfathered until new tenancy, lease renewal, or material lease modification triggers AB 12 cap.".to_string(),
            citations,
            applicable_cap_cents: 0,
        };
    }

    if input.tenant_classification == TenantClassification::MilitaryServiceMember {
        let service_member_cap_cents = input
            .monthly_rent_cents
            .saturating_mul(u64::from(CA_AB_12_GENERAL_CAP_MONTHS_RENT));
        if input.security_deposit_collected_cents > service_member_cap_cents {
            if input.small_landlord_exception_claimed {
                return Output {
                    mode: CaAb12Mode::ViolationServiceMemberDepositExceedsOneMonthSmallLandlordExceptionInapplicable,
                    statutory_basis: "Cal. Civ. Code § 1950.5(c)(4)(B) — service member override; small landlord exception inapplicable".to_string(),
                    notes: format!(
                        "VIOLATION: landlord claimed small landlord exception to collect {} cents from military service member tenant, exceeding 1-month cap of {} cents; § 1950.5(c)(4)(B) service member override applies — strict 1 month regardless of landlord size.",
                        input.security_deposit_collected_cents, service_member_cap_cents
                    ),
                    citations,
                    applicable_cap_cents: service_member_cap_cents,
                };
            }
            return Output {
                mode: CaAb12Mode::ViolationPostAb12DepositExceedsOneMonthCap,
                statutory_basis: "Cal. Civ. Code § 1950.5(c)(4)(B) — service member 1-month cap".to_string(),
                notes: format!(
                    "VIOLATION: deposit of {} cents from military service member tenant exceeds 1-month statutory cap of {} cents under § 1950.5(c)(4)(B).",
                    input.security_deposit_collected_cents, service_member_cap_cents
                ),
                citations,
                applicable_cap_cents: service_member_cap_cents,
            };
        }
        return Output {
            mode: CaAb12Mode::CompliantServiceMemberOverrideOneMonthCapRegardlessOfLandlordSize,
            statutory_basis: "Cal. Civ. Code § 1950.5(c)(4)(B) — service member 1-month cap".to_string(),
            notes: format!(
                "COMPLIANT: deposit of {} cents from military service member tenant within 1-month cap of {} cents under § 1950.5(c)(4)(B); small landlord exception inapplicable.",
                input.security_deposit_collected_cents, service_member_cap_cents
            ),
            citations,
            applicable_cap_cents: service_member_cap_cents,
        };
    }

    if input.small_landlord_exception_claimed {
        if !matches!(
            input.landlord_entity_type,
            LandlordEntityType::NaturalPerson
                | LandlordEntityType::LimitedLiabilityCompanyAllMembersNaturalPersons
        ) {
            return Output {
                mode: CaAb12Mode::ViolationSmallLandlordExceptionClaimedButEntityTypeIneligible,
                statutory_basis: "Cal. Civ. Code § 1950.5(c)(4) — small landlord exception requires natural person or natural-person-member LLC".to_string(),
                notes: format!(
                    "VIOLATION: small landlord exception claimed but landlord entity type ({:?}) is neither natural person nor LLC with all natural-person members; § 1950.5(c)(4) small landlord exception inapplicable.",
                    input.landlord_entity_type
                ),
                citations,
                applicable_cap_cents: input
                    .monthly_rent_cents
                    .saturating_mul(u64::from(CA_AB_12_GENERAL_CAP_MONTHS_RENT)),
            };
        }
        if input.landlord_residential_property_count > CA_AB_12_SMALL_LANDLORD_MAX_PROPERTIES {
            return Output {
                mode: CaAb12Mode::ViolationSmallLandlordExceptionClaimedButPropertyCountExceedsTwo,
                statutory_basis: "Cal. Civ. Code § 1950.5(c)(4) — small landlord exception requires ≤ 2 residential rental properties".to_string(),
                notes: format!(
                    "VIOLATION: small landlord exception claimed but landlord owns {} residential rental properties, exceeding 2-property statutory limit under § 1950.5(c)(4).",
                    input.landlord_residential_property_count
                ),
                citations,
                applicable_cap_cents: input
                    .monthly_rent_cents
                    .saturating_mul(u64::from(CA_AB_12_GENERAL_CAP_MONTHS_RENT)),
            };
        }
        if input.landlord_total_dwelling_units_for_rent > CA_AB_12_SMALL_LANDLORD_MAX_DWELLING_UNITS
        {
            return Output {
                mode: CaAb12Mode::ViolationSmallLandlordExceptionClaimedButDwellingUnitsExceedsFour,
                statutory_basis: "Cal. Civ. Code § 1950.5(c)(4) — small landlord exception requires ≤ 4 dwelling units".to_string(),
                notes: format!(
                    "VIOLATION: small landlord exception claimed but landlord owns {} dwelling units for rent, exceeding 4-unit statutory limit under § 1950.5(c)(4).",
                    input.landlord_total_dwelling_units_for_rent
                ),
                citations,
                applicable_cap_cents: input
                    .monthly_rent_cents
                    .saturating_mul(u64::from(CA_AB_12_GENERAL_CAP_MONTHS_RENT)),
            };
        }
        let small_landlord_cap_cents = input
            .monthly_rent_cents
            .saturating_mul(u64::from(CA_AB_12_SMALL_LANDLORD_CAP_MONTHS_RENT));
        if input.security_deposit_collected_cents > small_landlord_cap_cents {
            return Output {
                mode: CaAb12Mode::ViolationSmallLandlordExceptionDepositExceedsTwoMonthCap,
                statutory_basis: "Cal. Civ. Code § 1950.5(c)(4) — small landlord 2-month cap".to_string(),
                notes: format!(
                    "VIOLATION: deposit of {} cents exceeds 2-month small landlord cap of {} cents under § 1950.5(c)(4).",
                    input.security_deposit_collected_cents, small_landlord_cap_cents
                ),
                citations,
                applicable_cap_cents: small_landlord_cap_cents,
            };
        }
        return Output {
            mode: CaAb12Mode::CompliantSmallLandlordExceptionTwoMonthRentCap,
            statutory_basis: "Cal. Civ. Code § 1950.5(c)(4) — small landlord 2-month cap".to_string(),
            notes: format!(
                "COMPLIANT: deposit of {} cents within 2-month small landlord cap of {} cents under § 1950.5(c)(4); entity type, property count, and dwelling unit count all within statutory limits.",
                input.security_deposit_collected_cents, small_landlord_cap_cents
            ),
            citations,
            applicable_cap_cents: small_landlord_cap_cents,
        };
    }

    let general_cap_cents = input
        .monthly_rent_cents
        .saturating_mul(u64::from(CA_AB_12_GENERAL_CAP_MONTHS_RENT));
    if input.security_deposit_collected_cents > general_cap_cents {
        return Output {
            mode: CaAb12Mode::ViolationPostAb12DepositExceedsOneMonthCap,
            statutory_basis: "Cal. Civ. Code § 1950.5(c)(1) — general 1-month cap".to_string(),
            notes: format!(
                "VIOLATION: deposit of {} cents exceeds 1-month general cap of {} cents under § 1950.5(c)(1) (AB 12 effective July 1, 2024); furnishing status irrelevant.",
                input.security_deposit_collected_cents, general_cap_cents
            ),
            citations,
            applicable_cap_cents: general_cap_cents,
        };
    }
    Output {
        mode: CaAb12Mode::CompliantPostAb12GeneralCapOneMonthRent,
        statutory_basis: "Cal. Civ. Code § 1950.5(c)(1) — general 1-month cap satisfied".to_string(),
        notes: format!(
            "COMPLIANT: deposit of {} cents within 1-month general cap of {} cents under § 1950.5(c)(1); AB 12 effective July 1, 2024 uniform cap regardless of furnishing status.",
            input.security_deposit_collected_cents, general_cap_cents
        ),
        citations,
        applicable_cap_cents: general_cap_cents,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_post_ab12_general_compliant() -> Input {
        Input {
            property_in_california: true,
            lease_trigger_event: LeaseTriggerEvent::NewTenancyAfterOriginalTenantVacated,
            landlord_entity_type: LandlordEntityType::CorporationOrAnyEntityWithNonNaturalPersonMember,
            landlord_residential_property_count: 50,
            landlord_total_dwelling_units_for_rent: 200,
            tenant_classification: TenantClassification::StandardResidentialTenant,
            furnishing_status: FurnishingStatus::UnfurnishedUnit,
            monthly_rent_cents: 250_000,
            security_deposit_collected_cents: 250_000,
            small_landlord_exception_claimed: false,
        }
    }

    #[test]
    fn property_outside_california_not_applicable() {
        let input = Input {
            property_in_california: false,
            ..baseline_post_ab12_general_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, CaAb12Mode::NotApplicablePropertyOutsideCalifornia);
    }

    #[test]
    fn pre_july_2024_lease_grandfathered_not_applicable() {
        let input = Input {
            lease_trigger_event: LeaseTriggerEvent::PreJuly1_2024LeaseStillInPlace,
            security_deposit_collected_cents: 500_000,
            ..baseline_post_ab12_general_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, CaAb12Mode::NotApplicablePreAb12LeaseLawfullyGrandfathered);
    }

    #[test]
    fn post_ab12_general_cap_compliant_at_exactly_one_month() {
        let result = check(&baseline_post_ab12_general_compliant());
        assert_eq!(result.mode, CaAb12Mode::CompliantPostAb12GeneralCapOneMonthRent);
        assert_eq!(result.applicable_cap_cents, 250_000);
    }

    #[test]
    fn post_ab12_general_cap_violation_two_months() {
        let input = Input {
            security_deposit_collected_cents: 500_000,
            ..baseline_post_ab12_general_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, CaAb12Mode::ViolationPostAb12DepositExceedsOneMonthCap);
    }

    #[test]
    fn post_ab12_general_cap_violation_at_one_cent_over() {
        let input = Input {
            security_deposit_collected_cents: 250_001,
            ..baseline_post_ab12_general_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, CaAb12Mode::ViolationPostAb12DepositExceedsOneMonthCap);
    }

    #[test]
    fn post_ab12_furnished_still_one_month_cap() {
        let input = Input {
            furnishing_status: FurnishingStatus::FurnishedUnit,
            security_deposit_collected_cents: 750_000,
            ..baseline_post_ab12_general_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, CaAb12Mode::ViolationPostAb12DepositExceedsOneMonthCap);
    }

    #[test]
    fn small_landlord_natural_person_two_month_cap_compliant() {
        let input = Input {
            landlord_entity_type: LandlordEntityType::NaturalPerson,
            landlord_residential_property_count: 2,
            landlord_total_dwelling_units_for_rent: 4,
            small_landlord_exception_claimed: true,
            security_deposit_collected_cents: 500_000,
            ..baseline_post_ab12_general_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, CaAb12Mode::CompliantSmallLandlordExceptionTwoMonthRentCap);
        assert_eq!(result.applicable_cap_cents, 500_000);
    }

    #[test]
    fn small_landlord_llc_natural_person_members_compliant() {
        let input = Input {
            landlord_entity_type: LandlordEntityType::LimitedLiabilityCompanyAllMembersNaturalPersons,
            landlord_residential_property_count: 1,
            landlord_total_dwelling_units_for_rent: 2,
            small_landlord_exception_claimed: true,
            security_deposit_collected_cents: 500_000,
            ..baseline_post_ab12_general_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, CaAb12Mode::CompliantSmallLandlordExceptionTwoMonthRentCap);
    }

    #[test]
    fn small_landlord_corporation_entity_type_ineligible() {
        let input = Input {
            landlord_entity_type: LandlordEntityType::CorporationOrAnyEntityWithNonNaturalPersonMember,
            landlord_residential_property_count: 1,
            landlord_total_dwelling_units_for_rent: 2,
            small_landlord_exception_claimed: true,
            security_deposit_collected_cents: 500_000,
            ..baseline_post_ab12_general_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            CaAb12Mode::ViolationSmallLandlordExceptionClaimedButEntityTypeIneligible
        );
    }

    #[test]
    fn small_landlord_three_properties_violation() {
        let input = Input {
            landlord_entity_type: LandlordEntityType::NaturalPerson,
            landlord_residential_property_count: 3,
            landlord_total_dwelling_units_for_rent: 3,
            small_landlord_exception_claimed: true,
            security_deposit_collected_cents: 500_000,
            ..baseline_post_ab12_general_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            CaAb12Mode::ViolationSmallLandlordExceptionClaimedButPropertyCountExceedsTwo
        );
    }

    #[test]
    fn small_landlord_five_dwelling_units_violation() {
        let input = Input {
            landlord_entity_type: LandlordEntityType::NaturalPerson,
            landlord_residential_property_count: 2,
            landlord_total_dwelling_units_for_rent: 5,
            small_landlord_exception_claimed: true,
            security_deposit_collected_cents: 500_000,
            ..baseline_post_ab12_general_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            CaAb12Mode::ViolationSmallLandlordExceptionClaimedButDwellingUnitsExceedsFour
        );
    }

    #[test]
    fn small_landlord_at_exactly_4_units_compliant() {
        let input = Input {
            landlord_entity_type: LandlordEntityType::NaturalPerson,
            landlord_residential_property_count: 2,
            landlord_total_dwelling_units_for_rent: 4,
            small_landlord_exception_claimed: true,
            security_deposit_collected_cents: 500_000,
            ..baseline_post_ab12_general_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, CaAb12Mode::CompliantSmallLandlordExceptionTwoMonthRentCap);
    }

    #[test]
    fn small_landlord_two_month_cap_exceeded_violation() {
        let input = Input {
            landlord_entity_type: LandlordEntityType::NaturalPerson,
            landlord_residential_property_count: 1,
            landlord_total_dwelling_units_for_rent: 1,
            small_landlord_exception_claimed: true,
            security_deposit_collected_cents: 500_001,
            ..baseline_post_ab12_general_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            CaAb12Mode::ViolationSmallLandlordExceptionDepositExceedsTwoMonthCap
        );
    }

    #[test]
    fn service_member_override_one_month_compliant() {
        let input = Input {
            tenant_classification: TenantClassification::MilitaryServiceMember,
            landlord_entity_type: LandlordEntityType::NaturalPerson,
            landlord_residential_property_count: 1,
            landlord_total_dwelling_units_for_rent: 1,
            small_landlord_exception_claimed: true,
            security_deposit_collected_cents: 250_000,
            ..baseline_post_ab12_general_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            CaAb12Mode::CompliantServiceMemberOverrideOneMonthCapRegardlessOfLandlordSize
        );
        assert_eq!(result.applicable_cap_cents, 250_000);
    }

    #[test]
    fn service_member_small_landlord_two_month_claim_violation() {
        let input = Input {
            tenant_classification: TenantClassification::MilitaryServiceMember,
            landlord_entity_type: LandlordEntityType::NaturalPerson,
            landlord_residential_property_count: 1,
            landlord_total_dwelling_units_for_rent: 1,
            small_landlord_exception_claimed: true,
            security_deposit_collected_cents: 500_000,
            ..baseline_post_ab12_general_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            CaAb12Mode::ViolationServiceMemberDepositExceedsOneMonthSmallLandlordExceptionInapplicable
        );
    }

    #[test]
    fn service_member_general_one_month_violation() {
        let input = Input {
            tenant_classification: TenantClassification::MilitaryServiceMember,
            security_deposit_collected_cents: 500_000,
            ..baseline_post_ab12_general_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, CaAb12Mode::ViolationPostAb12DepositExceedsOneMonthCap);
    }

    #[test]
    fn written_lease_renewal_triggers_new_cap() {
        let input = Input {
            lease_trigger_event: LeaseTriggerEvent::WrittenLeaseRenewal,
            security_deposit_collected_cents: 500_000,
            ..baseline_post_ab12_general_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, CaAb12Mode::ViolationPostAb12DepositExceedsOneMonthCap);
    }

    #[test]
    fn material_lease_modification_triggers_new_cap() {
        let input = Input {
            lease_trigger_event: LeaseTriggerEvent::MaterialLeaseModification,
            security_deposit_collected_cents: 500_000,
            ..baseline_post_ab12_general_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, CaAb12Mode::ViolationPostAb12DepositExceedsOneMonthCap);
    }

    #[test]
    fn citations_pin_ab12_signing_and_caps() {
        let result = check(&baseline_post_ab12_general_compliant());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("California AB 12 of 2023"));
        assert!(joined.contains("Haney"));
        assert!(joined.contains("Governor Gavin Newsom"));
        assert!(joined.contains("October 11, 2023"));
        assert!(joined.contains("July 1, 2024"));
        assert!(joined.contains("9-month transition"));
        assert!(joined.contains("Cal. Civ. Code § 1950.5"));
        assert!(joined.contains("§ 1950.5(c)(1)"));
        assert!(joined.contains("§ 1950.5(c)(4)"));
        assert!(joined.contains("§ 1950.5(c)(4)(B)"));
        assert!(joined.contains("Cal. Mil. & Vet. Code § 400"));
        assert!(joined.contains("one month's rent"));
        assert!(joined.contains("2 months' rent"));
        assert!(joined.contains("≤ 2 properties"));
        assert!(joined.contains("≤ 4 dwelling units"));
        assert!(joined.contains("service member"));
        assert!(joined.contains("§ 1950.5(l)"));
        assert!(joined.contains("twice the security deposit"));
        assert!(joined.contains("Brownstein"));
        assert!(joined.contains("TLD Law"));
        assert!(joined.contains("Stormoen"));
    }

    #[test]
    fn constant_pin_dates_caps_and_thresholds() {
        assert_eq!(CA_AB_12_SIGNED_DATE_YEAR, 2023);
        assert_eq!(CA_AB_12_SIGNED_DATE_MONTH, 10);
        assert_eq!(CA_AB_12_SIGNED_DATE_DAY, 11);
        assert_eq!(CA_AB_12_EFFECTIVE_DATE_YEAR, 2024);
        assert_eq!(CA_AB_12_EFFECTIVE_DATE_MONTH, 7);
        assert_eq!(CA_AB_12_EFFECTIVE_DATE_DAY, 1);
        assert_eq!(CA_AB_12_GENERAL_CAP_MONTHS_RENT, 1);
        assert_eq!(CA_AB_12_SMALL_LANDLORD_CAP_MONTHS_RENT, 2);
        assert_eq!(CA_AB_12_SMALL_LANDLORD_MAX_PROPERTIES, 2);
        assert_eq!(CA_AB_12_SMALL_LANDLORD_MAX_DWELLING_UNITS, 4);
        assert_eq!(CA_AB_12_PRE_AB12_UNFURNISHED_CAP_MONTHS_RENT, 2);
        assert_eq!(CA_AB_12_PRE_AB12_FURNISHED_CAP_MONTHS_RENT, 3);
        assert_eq!(CA_CIV_CODE_1950_5_L_STATUTORY_DAMAGES_MULTIPLIER, 2);
    }

    #[test]
    fn saturating_overflow_defense_extreme_rent() {
        let input = Input {
            monthly_rent_cents: u64::MAX,
            security_deposit_collected_cents: u64::MAX,
            ..baseline_post_ab12_general_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, CaAb12Mode::CompliantPostAb12GeneralCapOneMonthRent);
    }
}
