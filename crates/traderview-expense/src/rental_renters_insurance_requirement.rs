//! Renter's Insurance Requirement Lease Provision Compliance.
//!
//! Pure-compute check for whether a landlord-imposed renter's
//! insurance requirement is enforceable under jurisdictional rules
//! governing residential lease provisions. All 50 U.S. states
//! permit landlords to require renter's insurance contractually,
//! but the requirement must be (1) IN WRITING in the lease, (2)
//! unambiguous, and (3) at a coverage level not so excessive as
//! to be effectively prohibitive of obtaining affordable policies.
//!
//! Web research (verified 2026-06-03):
//! - **General rule**: renter's insurance is NOT legally required
//!   by any state; landlords may require it contractually. All
//!   50 states permit contractual requirement. (Progressive
//!   Renters Insurance Requirements by State; Travelers Insurance;
//!   Allstate; MoneyGeek "Can a Landlord Require Renters
//!   Insurance".)
//! - **California**: landlords permitted to require renter's
//!   insurance as part of lease agreement so long as requirement
//!   is **clearly stated in writing before the lease is signed or
//!   renewed**. (Cost U Less Direct, California Landlords Renters
//!   Insurance Requirements.)
//! - **Texas, Florida, Ohio**: routine landlord requirement;
//!   courts uphold unambiguous lease language. (Money Geek.)
//! - **Standard coverage industry-norm**: $100,000 liability
//!   coverage + $10,000-$30,000 personal property coverage; courts
//!   have upheld these as reasonable. Courts will NOT enforce
//!   coverage amounts so excessive that they effectively prohibit
//!   tenants from finding affordable policies. (Steadily,
//!   Insurance.com.)
//! - **Additional insured naming**: typical lease clauses require
//!   tenant to name landlord as **additional insured** on the
//!   policy and provide proof of coverage before move-in / lease
//!   renewal.
//! - **Verbal agreements NOT sufficient** for enforcement; written
//!   lease provision required. (The Credit People.)
//! - **Mid-lease imposition**: a landlord cannot unilaterally add
//!   a renter's insurance requirement mid-lease without a written
//!   lease amendment signed by tenant; doing so is unenforceable.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const RENTERS_INSURANCE_INDUSTRY_STANDARD_LIABILITY_DOLLARS: u64 = 100_000;
pub const RENTERS_INSURANCE_INDUSTRY_STANDARD_PERSONAL_PROPERTY_DOLLARS: u64 = 10_000;
pub const RENTERS_INSURANCE_HIGH_END_PERSONAL_PROPERTY_DOLLARS: u64 = 30_000;
pub const RENTERS_INSURANCE_EXCESSIVE_LIABILITY_THRESHOLD_DOLLARS: u64 = 1_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InsuranceJurisdiction {
    California,
    Texas,
    Florida,
    NewYork,
    Ohio,
    OtherStatePermittingContractualRequirement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LeaseRequirementFormality {
    NoRequirementInLease,
    WrittenLeaseClauseClearAndUnambiguous,
    WrittenLeaseClauseAmbiguous,
    VerbalRequirementOnly,
    AttemptedMidLeaseImpositionWithoutAmendment,
    AttemptedMidLeaseImpositionWithSignedAmendment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdditionalInsuredStatus {
    LandlordNamedAdditionalInsuredProofProvided,
    LandlordNamedButProofNotProvided,
    LandlordNotNamedButPolicyInForce,
    NoPolicyInForce,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RentersInsuranceRequirementMode {
    NotApplicableNoLeaseRequirement,
    CompliantWrittenRequirementWithStandardCoverageAndProof,
    CompliantWrittenRequirementWithAdditionalInsuredNamingAndProof,
    CompliantMidLeaseAmendmentSignedByTenant,
    ViolationVerbalRequirementOnlyNotInLease,
    ViolationAmbiguousLeaseClauseNotEnforceable,
    ViolationMidLeaseImpositionWithoutAmendment,
    ViolationExcessiveLiabilityRequirementProhibitive,
    ViolationTenantFailedToMaintainPolicy,
    ViolationLandlordRequiredProofNotProvidedByTenant,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub jurisdiction: InsuranceJurisdiction,
    pub lease_requirement_formality: LeaseRequirementFormality,
    pub required_liability_coverage_dollars: u64,
    pub required_personal_property_coverage_dollars: u64,
    pub landlord_additional_insured_naming_required_in_lease: bool,
    pub additional_insured_status: AdditionalInsuredStatus,
    pub tenant_currently_maintains_policy: bool,
    pub tenant_provided_proof_of_insurance: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: RentersInsuranceRequirementMode,
    pub enforceability_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type RentalRentersInsuranceRequirementInput = Input;
pub type RentalRentersInsuranceRequirementOutput = Output;
pub type RentalRentersInsuranceRequirementResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "All 50 states permit landlord-imposed renter's insurance requirement contractually (Progressive Renters Insurance Requirements by State; Travelers Insurance; Allstate)".to_string(),
        "California — requirement must be CLEARLY STATED IN WRITING before lease is signed or renewed (Cost U Less Direct, California Landlords Renters Insurance Requirements)".to_string(),
        "Texas / Florida / Ohio — courts uphold unambiguous lease language requiring renter's insurance (MoneyGeek Can a Landlord Require Renters Insurance)".to_string(),
        "Industry standard $100,000 liability + $10,000–$30,000 personal property upheld as reasonable; excessive amounts effectively prohibitive of affordable policies NOT enforceable (Steadily Can a Landlord Require Renters Insurance)".to_string(),
        "Verbal renter's insurance requirements NOT enforceable; must be in written lease (The Credit People Landlord Mandate Renters Insurance)".to_string(),
        "Mid-lease imposition requires SIGNED LEASE AMENDMENT by tenant; unilateral imposition unenforceable".to_string(),
        "Typical lease clause: 'Tenant must maintain renters insurance with minimum $10,000 personal-property coverage and $100,000 liability coverage, naming landlord as additional insured.' (MoneyGeek)".to_string(),
        "HUD does NOT require renter's insurance for Section 8 voucher tenants by federal regulation; Public Housing Authorities may not impose unenforceable requirements".to_string(),
    ];

    if input.lease_requirement_formality == LeaseRequirementFormality::NoRequirementInLease {
        return Output {
            mode: RentersInsuranceRequirementMode::NotApplicableNoLeaseRequirement,
            enforceability_basis: "No renter's insurance requirement in lease".to_string(),
            notes: "Lease contains no renter's insurance requirement; no obligation on tenant. Landlord cannot enforce an unspecified obligation.".to_string(),
            citations,
        };
    }

    if input.lease_requirement_formality == LeaseRequirementFormality::VerbalRequirementOnly {
        return Output {
            mode: RentersInsuranceRequirementMode::ViolationVerbalRequirementOnlyNotInLease,
            enforceability_basis: "Verbal requirement; no written lease clause".to_string(),
            notes: format!(
                "VIOLATION: requirement communicated verbally only. Jurisdiction = {:?} requires written lease clause; verbal agreements not enforceable for insurance requirement.",
                input.jurisdiction
            ),
            citations,
        };
    }

    if input.lease_requirement_formality == LeaseRequirementFormality::WrittenLeaseClauseAmbiguous {
        return Output {
            mode: RentersInsuranceRequirementMode::ViolationAmbiguousLeaseClauseNotEnforceable,
            enforceability_basis: "Ambiguous lease clause; courts construe against drafting landlord".to_string(),
            notes: format!(
                "VIOLATION: written lease clause is ambiguous. Contra proferentem rule construes ambiguity against drafting landlord. Jurisdiction = {:?} requires unambiguous lease language for enforcement.",
                input.jurisdiction
            ),
            citations,
        };
    }

    if input.lease_requirement_formality
        == LeaseRequirementFormality::AttemptedMidLeaseImpositionWithoutAmendment
    {
        return Output {
            mode: RentersInsuranceRequirementMode::ViolationMidLeaseImpositionWithoutAmendment,
            enforceability_basis: "Mid-lease imposition without signed amendment".to_string(),
            notes: format!(
                "VIOLATION: landlord attempted to impose renter's insurance requirement mid-lease without written amendment signed by tenant. Unilateral lease modification unenforceable in jurisdiction = {:?}; requires signed amendment.",
                input.jurisdiction
            ),
            citations,
        };
    }

    if input.required_liability_coverage_dollars
        >= RENTERS_INSURANCE_EXCESSIVE_LIABILITY_THRESHOLD_DOLLARS
    {
        return Output {
            mode: RentersInsuranceRequirementMode::ViolationExcessiveLiabilityRequirementProhibitive,
            enforceability_basis: "Excessive liability requirement effectively prohibitive".to_string(),
            notes: format!(
                "VIOLATION: required liability coverage = ${} is at or above $1,000,000 threshold; courts decline to enforce coverage amounts so excessive that they effectively prohibit tenants from finding affordable policies. Industry standard reasonable minimum = $100,000.",
                input.required_liability_coverage_dollars
            ),
            citations,
        };
    }

    if !input.tenant_currently_maintains_policy {
        return Output {
            mode: RentersInsuranceRequirementMode::ViolationTenantFailedToMaintainPolicy,
            enforceability_basis: "Enforceable lease requirement violated by tenant".to_string(),
            notes: "VIOLATION: tenant failed to maintain renter's insurance policy as required by enforceable written lease clause. Default under lease; landlord may pursue lease remedies (cure notice, eviction).".to_string(),
            citations,
        };
    }

    if input.landlord_additional_insured_naming_required_in_lease
        && input.additional_insured_status
            == AdditionalInsuredStatus::LandlordNotNamedButPolicyInForce
    {
        return Output {
            mode: RentersInsuranceRequirementMode::ViolationTenantFailedToMaintainPolicy,
            enforceability_basis: "Policy in force but landlord not named additional insured per lease".to_string(),
            notes: "VIOLATION: lease requires landlord named as additional insured; tenant has policy but landlord not named. Tenant in technical default under enforceable lease clause; cure notice + proof of additional-insured endorsement required.".to_string(),
            citations,
        };
    }

    if !input.tenant_provided_proof_of_insurance {
        return Output {
            mode: RentersInsuranceRequirementMode::ViolationLandlordRequiredProofNotProvidedByTenant,
            enforceability_basis: "Policy may be in force but proof not provided".to_string(),
            notes: "VIOLATION: tenant maintains policy but did not provide proof (declarations page / certificate of insurance) to landlord. Most lease clauses make proof-of-coverage a prerequisite to occupancy; tenant in technical default until proof provided.".to_string(),
            citations,
        };
    }

    if input.lease_requirement_formality
        == LeaseRequirementFormality::AttemptedMidLeaseImpositionWithSignedAmendment
    {
        return Output {
            mode: RentersInsuranceRequirementMode::CompliantMidLeaseAmendmentSignedByTenant,
            enforceability_basis: "Mid-lease addition cured by signed amendment".to_string(),
            notes: format!(
                "COMPLIANT: mid-lease addition of renter's insurance requirement cured by written amendment signed by tenant. Effective in jurisdiction = {:?}. Required liability = ${}; personal property = ${}; tenant maintains policy + proof provided.",
                input.jurisdiction,
                input.required_liability_coverage_dollars,
                input.required_personal_property_coverage_dollars
            ),
            citations,
        };
    }

    if input.landlord_additional_insured_naming_required_in_lease
        && matches!(
            input.additional_insured_status,
            AdditionalInsuredStatus::LandlordNamedAdditionalInsuredProofProvided
        )
    {
        return Output {
            mode: RentersInsuranceRequirementMode::CompliantWrittenRequirementWithAdditionalInsuredNamingAndProof,
            enforceability_basis: "Written lease requirement satisfied with additional-insured naming".to_string(),
            notes: format!(
                "COMPLIANT: tenant maintains policy + landlord named as additional insured + proof provided. Jurisdiction = {:?}; required liability = ${}; required personal property = ${}.",
                input.jurisdiction,
                input.required_liability_coverage_dollars,
                input.required_personal_property_coverage_dollars
            ),
            citations,
        };
    }

    Output {
        mode: RentersInsuranceRequirementMode::CompliantWrittenRequirementWithStandardCoverageAndProof,
        enforceability_basis: "Written lease requirement satisfied with proof of coverage".to_string(),
        notes: format!(
            "COMPLIANT: tenant maintains policy at required levels (liability = ${}; personal property = ${}); proof provided. Jurisdiction = {:?}.",
            input.required_liability_coverage_dollars,
            input.required_personal_property_coverage_dollars,
            input.jurisdiction
        ),
        citations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_california_standard_compliant() -> Input {
        Input {
            jurisdiction: InsuranceJurisdiction::California,
            lease_requirement_formality:
                LeaseRequirementFormality::WrittenLeaseClauseClearAndUnambiguous,
            required_liability_coverage_dollars: 100_000,
            required_personal_property_coverage_dollars: 10_000,
            landlord_additional_insured_naming_required_in_lease: false,
            additional_insured_status: AdditionalInsuredStatus::LandlordNotNamedButPolicyInForce,
            tenant_currently_maintains_policy: true,
            tenant_provided_proof_of_insurance: true,
        }
    }

    #[test]
    fn california_standard_written_requirement_compliant() {
        let result = check(&baseline_california_standard_compliant());
        assert_eq!(result.mode, RentersInsuranceRequirementMode::CompliantWrittenRequirementWithStandardCoverageAndProof);
    }

    #[test]
    fn additional_insured_naming_with_proof_compliant() {
        let input = Input {
            landlord_additional_insured_naming_required_in_lease: true,
            additional_insured_status:
                AdditionalInsuredStatus::LandlordNamedAdditionalInsuredProofProvided,
            ..baseline_california_standard_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, RentersInsuranceRequirementMode::CompliantWrittenRequirementWithAdditionalInsuredNamingAndProof);
    }

    #[test]
    fn no_requirement_in_lease_not_applicable() {
        let input = Input {
            lease_requirement_formality: LeaseRequirementFormality::NoRequirementInLease,
            ..baseline_california_standard_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            RentersInsuranceRequirementMode::NotApplicableNoLeaseRequirement
        );
    }

    #[test]
    fn verbal_requirement_only_violation() {
        let input = Input {
            lease_requirement_formality: LeaseRequirementFormality::VerbalRequirementOnly,
            ..baseline_california_standard_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            RentersInsuranceRequirementMode::ViolationVerbalRequirementOnlyNotInLease
        );
    }

    #[test]
    fn ambiguous_lease_clause_violation() {
        let input = Input {
            lease_requirement_formality: LeaseRequirementFormality::WrittenLeaseClauseAmbiguous,
            ..baseline_california_standard_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            RentersInsuranceRequirementMode::ViolationAmbiguousLeaseClauseNotEnforceable
        );
        assert!(result.notes.contains("Contra proferentem"));
    }

    #[test]
    fn mid_lease_imposition_without_amendment_violation() {
        let input = Input {
            lease_requirement_formality:
                LeaseRequirementFormality::AttemptedMidLeaseImpositionWithoutAmendment,
            ..baseline_california_standard_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            RentersInsuranceRequirementMode::ViolationMidLeaseImpositionWithoutAmendment
        );
    }

    #[test]
    fn mid_lease_imposition_with_signed_amendment_compliant() {
        let input = Input {
            lease_requirement_formality:
                LeaseRequirementFormality::AttemptedMidLeaseImpositionWithSignedAmendment,
            ..baseline_california_standard_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            RentersInsuranceRequirementMode::CompliantMidLeaseAmendmentSignedByTenant
        );
    }

    #[test]
    fn excessive_1m_liability_requirement_violation() {
        let input = Input {
            required_liability_coverage_dollars: 1_000_000,
            ..baseline_california_standard_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            RentersInsuranceRequirementMode::ViolationExcessiveLiabilityRequirementProhibitive
        );
    }

    #[test]
    fn excessive_2m_liability_requirement_violation() {
        let input = Input {
            required_liability_coverage_dollars: 2_000_000,
            ..baseline_california_standard_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            RentersInsuranceRequirementMode::ViolationExcessiveLiabilityRequirementProhibitive
        );
    }

    #[test]
    fn standard_100k_liability_compliant() {
        let result = check(&baseline_california_standard_compliant());
        assert_eq!(result.mode, RentersInsuranceRequirementMode::CompliantWrittenRequirementWithStandardCoverageAndProof);
    }

    #[test]
    fn at_999_999_below_threshold_still_compliant() {
        let input = Input {
            required_liability_coverage_dollars: 999_999,
            ..baseline_california_standard_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, RentersInsuranceRequirementMode::CompliantWrittenRequirementWithStandardCoverageAndProof);
    }

    #[test]
    fn tenant_no_policy_violation() {
        let input = Input {
            tenant_currently_maintains_policy: false,
            ..baseline_california_standard_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            RentersInsuranceRequirementMode::ViolationTenantFailedToMaintainPolicy
        );
    }

    #[test]
    fn tenant_policy_but_landlord_not_named_when_required_violation() {
        let input = Input {
            landlord_additional_insured_naming_required_in_lease: true,
            additional_insured_status: AdditionalInsuredStatus::LandlordNotNamedButPolicyInForce,
            ..baseline_california_standard_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            RentersInsuranceRequirementMode::ViolationTenantFailedToMaintainPolicy
        );
        assert!(result.notes.contains("additional insured"));
    }

    #[test]
    fn tenant_policy_but_no_proof_provided_violation() {
        let input = Input {
            tenant_provided_proof_of_insurance: false,
            ..baseline_california_standard_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            RentersInsuranceRequirementMode::ViolationLandlordRequiredProofNotProvidedByTenant
        );
    }

    #[test]
    fn texas_florida_ohio_uphold_unambiguous_clause_compliant() {
        for jurisdiction in [
            InsuranceJurisdiction::Texas,
            InsuranceJurisdiction::Florida,
            InsuranceJurisdiction::Ohio,
        ] {
            let input = Input {
                jurisdiction,
                ..baseline_california_standard_compliant()
            };
            let result = check(&input);
            assert_eq!(result.mode, RentersInsuranceRequirementMode::CompliantWrittenRequirementWithStandardCoverageAndProof);
        }
    }

    #[test]
    fn new_york_other_state_compliant_with_standard_coverage() {
        for jurisdiction in [
            InsuranceJurisdiction::NewYork,
            InsuranceJurisdiction::OtherStatePermittingContractualRequirement,
        ] {
            let input = Input {
                jurisdiction,
                ..baseline_california_standard_compliant()
            };
            let result = check(&input);
            assert_eq!(result.mode, RentersInsuranceRequirementMode::CompliantWrittenRequirementWithStandardCoverageAndProof);
        }
    }

    #[test]
    fn citations_pin_industry_and_jurisdictional_sources() {
        let result = check(&baseline_california_standard_compliant());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("All 50 states"));
        assert!(joined.contains("California"));
        assert!(joined.contains("CLEARLY STATED IN WRITING"));
        assert!(joined.contains("Texas / Florida / Ohio"));
        assert!(joined.contains("$100,000 liability"));
        assert!(joined.contains("Verbal renter's insurance requirements NOT enforceable"));
        assert!(joined.contains("Mid-lease imposition requires SIGNED LEASE AMENDMENT"));
        assert!(joined.contains("HUD does NOT require"));
    }

    #[test]
    fn constant_pin_industry_standard_thresholds() {
        assert_eq!(
            RENTERS_INSURANCE_INDUSTRY_STANDARD_LIABILITY_DOLLARS,
            100_000
        );
        assert_eq!(
            RENTERS_INSURANCE_INDUSTRY_STANDARD_PERSONAL_PROPERTY_DOLLARS,
            10_000
        );
        assert_eq!(RENTERS_INSURANCE_HIGH_END_PERSONAL_PROPERTY_DOLLARS, 30_000);
        assert_eq!(
            RENTERS_INSURANCE_EXCESSIVE_LIABILITY_THRESHOLD_DOLLARS,
            1_000_000
        );
    }
}
