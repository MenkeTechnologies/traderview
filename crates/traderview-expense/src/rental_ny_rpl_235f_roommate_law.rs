//! NY Real Property Law § 235-f Roommate Law (Unlawful Restrictions
//! on Occupancy) compliance for trader-landlords with NY State
//! residential rental inventory.
//!
//! Enacted 1983 as a tenant-protection statute prohibiting landlord
//! restrictions on tenant rights to share residential premises with
//! immediate family, additional occupants, and dependent children.
//! Codified at NY RPL § 235-f.
//!
//! **§ 235-f(1) definitions**:
//!
//! - **Tenant**: person occupying or entitled to occupy residential
//!   rental premises who is either party to the lease or rental
//!   agreement OR a statutory tenant under the emergency housing
//!   rent control law, NYC rent and rehabilitation law, or Article
//!   7-C of the Multiple Dwelling Law (Loft Law).
//! - **Occupant**: a person, other than a tenant or a member of a
//!   tenant's immediate family, occupying a premises with the
//!   consent of the tenant or tenants.
//!
//! **§ 235-f(2) single-tenant lease occupancy permitted**: any
//! lease entered into by ONE tenant shall be construed to permit
//! occupancy by:
//!
//! 1. The tenant;
//! 2. Immediate family of the tenant;
//! 3. **ONE additional occupant**;
//! 4. Dependent children of the occupant;
//!
//! PROVIDED that the tenant or the tenant's spouse occupies the
//! premises as their primary residence.
//!
//! **§ 235-f(3) multi-tenant lease occupancy permitted**: any
//! lease entered into by TWO OR MORE tenants shall be construed to
//! permit occupancy by tenants, immediate family of tenants,
//! occupants, and dependent children of occupants, PROVIDED that
//! the total number of tenants and occupants (EXCLUDING occupants'
//! dependent children) does not exceed the number of tenants
//! specified in the current lease or rental agreement, AND at
//! least one tenant or tenant's spouse occupies as primary
//! residence.
//!
//! **§ 235-f(4) tenant notice requirement**: tenant shall inform
//! landlord of the name of any occupant within **30 days** following
//! the commencement of occupancy OR within 30 days following a
//! request by the landlord.
//!
//! **§ 235-f(5) limited occupant rights**: no occupant nor
//! occupant's dependent child shall, without express written
//! permission of the landlord, acquire any right to continued
//! occupancy in the event that the tenant vacates the premises,
//! NOR acquire any other rights of tenancy.
//!
//! **§ 235-f(6) unlawful lease restrictions**: it is unlawful for a
//! landlord to restrict occupancy of residential premises, by
//! express lease terms or otherwise, to a tenant or tenants or to
//! such tenants and immediate family. Any such restriction in a
//! lease or rental agreement is UNENFORCEABLE as against public
//! policy.
//!
//! **§ 235-f(7) waiver void**: any provision of a lease purporting
//! to waive a provision of § 235-f shall be NULL AND VOID.
//!
//! **§ 235-f(8) Multiple Dwelling Law and rent stabilization
//! interaction**: nothing in § 235-f shall be construed to limit
//! the right of the landlord to enforce occupancy standards
//! contained in MDL § 4(7) (overcrowding) or rent stabilization
//! laws regarding succession rights of family members.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[allow(dead_code)]
pub const RPL_235F_OCCUPANT_NOTICE_DAYS: u32 = 30;
#[allow(dead_code)]
pub const RPL_235F_SINGLE_TENANT_MAX_ADDITIONAL_OCCUPANTS: u32 = 1;
#[allow(dead_code)]
pub const RPL_235F_ORIGINAL_ENACTMENT_YEAR: u32 = 1983;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LeaseType {
    SingleTenant,
    MultiTenant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    CompliantSingleTenantWith1OccupantAndDependents,
    CompliantMultiTenantWithinNumericCap,
    CompliantOccupantNoticeProvidedWithin30Days,
    ViolationLeaseClauseRestrictingOccupancyUnenforceable,
    ViolationLandlordSoughtEvictionForUnlistedOccupant,
    ViolationSingleTenantExceedsOneAdditionalOccupant,
    ViolationMultiTenantLeaseExceedsNumericCap,
    ViolationNoTenantPrimaryResidence,
    ViolationOccupantNoticeNotProvidedWithin30Days,
    ViolationWaiverOfRpl235fNullAndVoid,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub lease_type: LeaseType,
    pub number_of_tenants_in_lease: u32,
    pub number_of_additional_occupants: u32,
    pub number_of_occupants_dependent_children: u32,
    pub number_of_tenants_immediate_family: u32,
    pub tenant_or_spouse_uses_as_primary_residence: bool,
    pub lease_contains_occupancy_restriction_clause: bool,
    pub lease_waiver_of_235f_attempted: bool,
    pub occupant_name_provided_to_landlord: bool,
    pub days_since_occupant_commenced: u32,
    pub landlord_seeking_eviction_for_occupant: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Output {
    pub severity: Severity,
    pub compliant: bool,
    pub maximum_permitted_occupants_excluding_dependent_children: u32,
    pub lease_clause_enforceable: bool,
    pub notes: Vec<String>,
    pub citations: Vec<String>,
}

pub type NyRpl235FRoommateLawInput = Input;
pub type NyRpl235FRoommateLawOutput = Output;
pub type NyRpl235FRoommateLawResult = Output;

pub fn check(input: &Input) -> Output {
    let mut notes: Vec<String> = Vec::new();
    let citations: Vec<String> = vec![
        "NY Real Property Law § 235-f (Unlawful Restrictions on Occupancy — Roommate Law)".to_string(),
        "NY RPL § 235-f(1) (definitions — tenant + occupant)".to_string(),
        "NY RPL § 235-f(2) (single-tenant lease — 1 additional occupant + dependent children permitted)".to_string(),
        "NY RPL § 235-f(3) (multi-tenant lease — numeric cap on tenants + occupants)".to_string(),
        "NY RPL § 235-f(4) (30-day occupant notice requirement)".to_string(),
        "NY RPL § 235-f(5) (occupant has no tenancy rights without landlord's express written permission)".to_string(),
        "NY RPL § 235-f(6) (unlawful lease restrictions on occupancy)".to_string(),
        "NY RPL § 235-f(7) (waiver of § 235-f null and void)".to_string(),
        "NY RPL § 235-f(8) (MDL § 4(7) overcrowding cross-reference)".to_string(),
        "NY MDL § 4(7) (Multiple Dwelling Law overcrowding standard)".to_string(),
        "NY RPL Article 7-C (Loft Law — statutory tenant cross-reference)".to_string(),
        "NY Emergency Housing Rent Control Law (statutory tenant cross-reference)".to_string(),
    ];

    if input.lease_waiver_of_235f_attempted {
        notes.push("Lease provision purporting to waive § 235-f — null and void under § 235-f(7); cannot be enforced against tenant.".to_string());
        return Output {
            severity: Severity::ViolationWaiverOfRpl235fNullAndVoid,
            compliant: false,
            maximum_permitted_occupants_excluding_dependent_children: 0,
            lease_clause_enforceable: false,
            notes,
            citations,
        };
    }

    if input.lease_contains_occupancy_restriction_clause {
        notes.push("Lease clause restricting occupancy beyond § 235-f rights — unenforceable as against public policy under § 235-f(6).".to_string());
        return Output {
            severity: Severity::ViolationLeaseClauseRestrictingOccupancyUnenforceable,
            compliant: false,
            maximum_permitted_occupants_excluding_dependent_children: 0,
            lease_clause_enforceable: false,
            notes,
            citations,
        };
    }

    if !input.tenant_or_spouse_uses_as_primary_residence {
        notes.push("Tenant or tenant's spouse does not occupy premises as primary residence — § 235-f(2)/(3) occupancy rights not available.".to_string());
        return Output {
            severity: Severity::ViolationNoTenantPrimaryResidence,
            compliant: false,
            maximum_permitted_occupants_excluding_dependent_children: 0,
            lease_clause_enforceable: true,
            notes,
            citations,
        };
    }

    if input.occupant_name_provided_to_landlord
        && input.days_since_occupant_commenced > RPL_235F_OCCUPANT_NOTICE_DAYS
    {
        notes.push(format!(
            "Tenant did not inform landlord of occupant name within {}-day window — § 235-f(4) violation.",
            RPL_235F_OCCUPANT_NOTICE_DAYS
        ));
        return Output {
            severity: Severity::ViolationOccupantNoticeNotProvidedWithin30Days,
            compliant: false,
            maximum_permitted_occupants_excluding_dependent_children: 0,
            lease_clause_enforceable: true,
            notes,
            citations,
        };
    }

    match input.lease_type {
        LeaseType::SingleTenant => {
            if input.number_of_additional_occupants
                > RPL_235F_SINGLE_TENANT_MAX_ADDITIONAL_OCCUPANTS
            {
                notes.push(format!(
                    "Single-tenant lease: {} additional occupants > {} statutory maximum under § 235-f(2).",
                    input.number_of_additional_occupants,
                    RPL_235F_SINGLE_TENANT_MAX_ADDITIONAL_OCCUPANTS
                ));
                let severity = if input.landlord_seeking_eviction_for_occupant {
                    Severity::ViolationLandlordSoughtEvictionForUnlistedOccupant
                } else {
                    Severity::ViolationSingleTenantExceedsOneAdditionalOccupant
                };
                return Output {
                    severity,
                    compliant: false,
                    maximum_permitted_occupants_excluding_dependent_children: 1,
                    lease_clause_enforceable: true,
                    notes,
                    citations,
                };
            }
            notes.push(format!(
                "§ 235-f(2) single-tenant compliance: 1 tenant + {} immediate family + {} additional occupant + {} dependent children of occupant permitted.",
                input.number_of_tenants_immediate_family,
                input.number_of_additional_occupants,
                input.number_of_occupants_dependent_children
            ));
            Output {
                severity: Severity::CompliantSingleTenantWith1OccupantAndDependents,
                compliant: true,
                maximum_permitted_occupants_excluding_dependent_children: 1,
                lease_clause_enforceable: true,
                notes,
                citations,
            }
        }
        LeaseType::MultiTenant => {
            let tenants_plus_occupants_excluding_dep_children = input
                .number_of_tenants_in_lease
                .saturating_add(input.number_of_additional_occupants);
            if tenants_plus_occupants_excluding_dep_children > input.number_of_tenants_in_lease {
                notes.push(format!(
                    "Multi-tenant lease: {} tenants + {} occupants ({} total excluding dependent children) > {} tenants specified in lease — § 235-f(3) violation.",
                    input.number_of_tenants_in_lease,
                    input.number_of_additional_occupants,
                    tenants_plus_occupants_excluding_dep_children,
                    input.number_of_tenants_in_lease
                ));
                let severity = if input.landlord_seeking_eviction_for_occupant {
                    Severity::ViolationLandlordSoughtEvictionForUnlistedOccupant
                } else {
                    Severity::ViolationMultiTenantLeaseExceedsNumericCap
                };
                return Output {
                    severity,
                    compliant: false,
                    maximum_permitted_occupants_excluding_dependent_children:
                        input.number_of_tenants_in_lease,
                    lease_clause_enforceable: true,
                    notes,
                    citations,
                };
            }
            notes.push(format!(
                "§ 235-f(3) multi-tenant compliance: {} tenants + {} occupants (within numeric cap of {} lease tenants); dependent children excluded from cap.",
                input.number_of_tenants_in_lease,
                input.number_of_additional_occupants,
                input.number_of_tenants_in_lease
            ));
            Output {
                severity: Severity::CompliantMultiTenantWithinNumericCap,
                compliant: true,
                maximum_permitted_occupants_excluding_dependent_children:
                    input.number_of_tenants_in_lease,
                lease_clause_enforceable: true,
                notes,
                citations,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_single_tenant_compliant() -> Input {
        Input {
            lease_type: LeaseType::SingleTenant,
            number_of_tenants_in_lease: 1,
            number_of_additional_occupants: 1,
            number_of_occupants_dependent_children: 2,
            number_of_tenants_immediate_family: 0,
            tenant_or_spouse_uses_as_primary_residence: true,
            lease_contains_occupancy_restriction_clause: false,
            lease_waiver_of_235f_attempted: false,
            occupant_name_provided_to_landlord: true,
            days_since_occupant_commenced: 15,
            landlord_seeking_eviction_for_occupant: false,
        }
    }

    #[test]
    fn single_tenant_with_1_occupant_compliant() {
        let out = check(&base_single_tenant_compliant());
        assert_eq!(
            out.severity,
            Severity::CompliantSingleTenantWith1OccupantAndDependents
        );
        assert!(out.compliant);
    }

    #[test]
    fn single_tenant_with_2_occupants_violation() {
        let mut i = base_single_tenant_compliant();
        i.number_of_additional_occupants = 2;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationSingleTenantExceedsOneAdditionalOccupant
        );
    }

    #[test]
    fn multi_tenant_within_numeric_cap_compliant() {
        let mut i = base_single_tenant_compliant();
        i.lease_type = LeaseType::MultiTenant;
        i.number_of_tenants_in_lease = 3;
        i.number_of_additional_occupants = 0;
        let out = check(&i);
        assert_eq!(out.severity, Severity::CompliantMultiTenantWithinNumericCap);
    }

    #[test]
    fn multi_tenant_exceeds_numeric_cap_violation() {
        let mut i = base_single_tenant_compliant();
        i.lease_type = LeaseType::MultiTenant;
        i.number_of_tenants_in_lease = 2;
        i.number_of_additional_occupants = 2;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationMultiTenantLeaseExceedsNumericCap
        );
    }

    #[test]
    fn lease_clause_restricting_occupancy_unenforceable() {
        let mut i = base_single_tenant_compliant();
        i.lease_contains_occupancy_restriction_clause = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationLeaseClauseRestrictingOccupancyUnenforceable
        );
        assert!(!out.lease_clause_enforceable);
    }

    #[test]
    fn waiver_of_235f_null_and_void() {
        let mut i = base_single_tenant_compliant();
        i.lease_waiver_of_235f_attempted = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ViolationWaiverOfRpl235fNullAndVoid);
    }

    #[test]
    fn no_tenant_primary_residence_violation() {
        let mut i = base_single_tenant_compliant();
        i.tenant_or_spouse_uses_as_primary_residence = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ViolationNoTenantPrimaryResidence);
    }

    #[test]
    fn occupant_notice_within_30_days_compliant() {
        let mut i = base_single_tenant_compliant();
        i.days_since_occupant_commenced = 30;
        let out = check(&i);
        assert!(out.compliant);
    }

    #[test]
    fn occupant_notice_at_exactly_30_days_compliant() {
        let mut i = base_single_tenant_compliant();
        i.days_since_occupant_commenced = 30;
        let out = check(&i);
        assert!(out.compliant);
    }

    #[test]
    fn occupant_notice_31_days_violation() {
        let mut i = base_single_tenant_compliant();
        i.occupant_name_provided_to_landlord = true;
        i.days_since_occupant_commenced = 31;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationOccupantNoticeNotProvidedWithin30Days
        );
    }

    #[test]
    fn landlord_seeking_eviction_for_excess_occupant_violation() {
        let mut i = base_single_tenant_compliant();
        i.number_of_additional_occupants = 2;
        i.landlord_seeking_eviction_for_occupant = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationLandlordSoughtEvictionForUnlistedOccupant
        );
    }

    #[test]
    fn dependent_children_not_counted_against_cap() {
        let mut i = base_single_tenant_compliant();
        i.number_of_occupants_dependent_children = 5;
        let out = check(&i);
        assert!(out.compliant);
    }

    #[test]
    fn citations_pin_rpl_235f_subsections() {
        let out = check(&base_single_tenant_compliant());
        assert!(out.citations.iter().any(|c| c.contains("§ 235-f(1)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 235-f(2)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 235-f(3)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 235-f(4)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 235-f(5)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 235-f(6)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 235-f(7)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 235-f(8)")));
    }

    #[test]
    fn citations_pin_mdl_4_7_overcrowding_and_loft_law() {
        let out = check(&base_single_tenant_compliant());
        assert!(out.citations.iter().any(|c| c.contains("MDL § 4(7)")));
        assert!(out.citations.iter().any(|c| c.contains("Article 7-C")));
    }

    #[test]
    fn constant_pin_30_day_occupant_notice() {
        assert_eq!(RPL_235F_OCCUPANT_NOTICE_DAYS, 30);
    }

    #[test]
    fn constant_pin_1_additional_occupant_single_tenant_max() {
        assert_eq!(RPL_235F_SINGLE_TENANT_MAX_ADDITIONAL_OCCUPANTS, 1);
    }

    #[test]
    fn constant_pin_1983_original_enactment() {
        assert_eq!(RPL_235F_ORIGINAL_ENACTMENT_YEAR, 1983);
    }
}
