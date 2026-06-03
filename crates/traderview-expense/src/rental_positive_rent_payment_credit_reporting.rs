//! Residential Positive Rent Payment Credit Reporting Compliance.
//!
//! Pure-compute check for landlord compliance with California
//! AB 2747 (effective Jan 1, 2025; key requirements begin Apr 1,
//! 2025) and similar state statutes mandating that covered
//! landlords OFFER tenants the option to have positive rental
//! payment information reported to at least one nationwide
//! consumer reporting agency.
//!
//! Web research (verified 2026-06-03):
//! - **California AB 2747 (Assembly Bill 2747 of 2023-2024
//!   session)**: codified at Cal. Civ. Code § 1954.06 (added).
//!   Effective January 1, 2025; key offer requirements begin
//!   April 1, 2025. Requires covered landlords to OFFER tenants
//!   the option of having positive rental payment information
//!   reported to at least one nationwide consumer reporting
//!   agency. Cal. Legislative Information AB 2747 Bill Text;
//!   HBR Rentals AB 2747; FrontLobby California Rent Reporting
//!   Law guide.
//! - **Small-landlord exemption**: a landlord of a residential
//!   rental building with **15 or fewer** dwelling units is
//!   EXEMPT unless specified conditions met. Exemption
//!   DISAPPEARS if landlord (a) owns MORE THAN ONE residential
//!   rental building AND (b) is a REIT, corporation, or LLC with
//!   at least one corporate member.
//! - **Effective-date matrix**:
//!   - Leases entered into on/after Apr 1, 2025: offer made at
//!     time of lease agreement AND at least once annually
//!     thereafter.
//!   - Leases outstanding as of Jan 1, 2025 (pre-existing): offer
//!     made no later than Apr 1, 2025 AND at least once annually
//!     thereafter.
//! - **Fee cap**: landlord may charge the tenant who ELECTS
//!   positive reporting the LESSER of **$10 per month** OR the
//!   landlord's actual cost. If landlord incurs no actual cost
//!   (e.g., free reporting service), no fee may be charged.
//! - **Qualifying CRA**: report must go to at least one
//!   NATIONWIDE consumer reporting agency (Equifax, Experian,
//!   TransUnion).

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const POSITIVE_RENT_REPORTING_CA_AB_2747_EFFECTIVE_YEAR: u32 = 2025;
pub const POSITIVE_RENT_REPORTING_CA_AB_2747_EFFECTIVE_MONTH: u32 = 1;
pub const POSITIVE_RENT_REPORTING_CA_AB_2747_EFFECTIVE_DAY: u32 = 1;
pub const POSITIVE_RENT_REPORTING_CA_OFFER_DEADLINE_YEAR: u32 = 2025;
pub const POSITIVE_RENT_REPORTING_CA_OFFER_DEADLINE_MONTH: u32 = 4;
pub const POSITIVE_RENT_REPORTING_CA_OFFER_DEADLINE_DAY: u32 = 1;
pub const POSITIVE_RENT_REPORTING_CA_FEE_CAP_DOLLARS_PER_MONTH: u64 = 10;
pub const POSITIVE_RENT_REPORTING_CA_SMALL_LANDLORD_UNIT_THRESHOLD: u32 = 15;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    California,
    OtherStateWithoutPositiveRentReportingMandate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LandlordOwnershipStructure {
    IndividualOrLlcWithoutCorporateMember,
    Reit,
    Corporation,
    LlcWithCorporateMember,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LeaseTiming {
    NewLeaseOnOrAfterApril1_2025,
    PreExistingLeaseOutstandingJanuary1_2025,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TenantElection {
    TenantNotYetOffered,
    TenantDeclinedOffer,
    TenantElectedAndReportingActive,
    TenantElectedButReportingNotActivated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PositiveRentReportingMode {
    NotApplicableExemptSmallLandlord,
    NotApplicableJurisdictionLacksMandate,
    CompliantOfferMadeAtLeaseInceptionAndAnnual,
    CompliantTenantDeclined,
    CompliantTenantElectedAndReportingActive,
    ViolationOfferNotMadeAtLeaseInception,
    ViolationAnnualOfferNotProvided,
    ViolationOfferToPreExistingTenantsMissedApril1_2025,
    ViolationFeeExceedsLesserOf10DollarsOrActualCost,
    ViolationFeeChargedWhenLandlordIncursNoActualCost,
    ViolationNoQualifyingNationwideConsumerReportingAgency,
    ViolationSmallLandlordExemptionInvalidlyClaimed,
    ViolationTenantElectedButReportingNotActivated,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub jurisdiction: Jurisdiction,
    pub unit_count_in_building: u32,
    pub landlord_owns_multiple_buildings: bool,
    pub landlord_ownership_structure: LandlordOwnershipStructure,
    pub lease_timing: LeaseTiming,
    pub offer_made_at_lease_inception: bool,
    pub offer_made_annually: bool,
    pub offer_to_pre_existing_tenant_made_by_april_1_2025: bool,
    pub tenant_election: TenantElection,
    pub monthly_fee_charged_dollars: u64,
    pub landlord_actual_cost_per_month_dollars: u64,
    pub reporting_to_nationwide_cra: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: PositiveRentReportingMode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type RentalPositiveRentPaymentCreditReportingInput = Input;
pub type RentalPositiveRentPaymentCreditReportingOutput = Output;
pub type RentalPositiveRentPaymentCreditReportingResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

fn small_landlord_exemption_applies(input: &Input) -> bool {
    if input.unit_count_in_building > POSITIVE_RENT_REPORTING_CA_SMALL_LANDLORD_UNIT_THRESHOLD {
        return false;
    }
    if input.landlord_owns_multiple_buildings
        && matches!(
            input.landlord_ownership_structure,
            LandlordOwnershipStructure::Reit
                | LandlordOwnershipStructure::Corporation
                | LandlordOwnershipStructure::LlcWithCorporateMember
        )
    {
        return false;
    }
    true
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "Cal. AB 2747 (2023-2024 session) — added Cal. Civ. Code § 1954.06; effective January 1, 2025; key offer requirements begin April 1, 2025".to_string(),
        "Cal. Civ. Code § 1954.06 — landlord must OFFER each tenant option of positive rental payment information reporting to at least one nationwide consumer reporting agency".to_string(),
        "Cal. Civ. Code § 1954.06 small-landlord exemption — building with 15 or fewer dwelling units; exemption DISAPPEARS if landlord owns more than one residential rental building AND is REIT, corporation, or LLC with at least one corporate member".to_string(),
        "Cal. Civ. Code § 1954.06 effective-date matrix — new leases on/after Apr 1, 2025: offer at lease + annually; pre-existing leases outstanding Jan 1, 2025: offer no later than Apr 1, 2025 + annually".to_string(),
        "Cal. Civ. Code § 1954.06 fee cap — LESSER of $10 per month OR actual landlord cost; no fee if landlord incurs no actual cost".to_string(),
        "Cal. Civ. Code § 1954.06 — reporting to at least one nationwide consumer reporting agency (Equifax / Experian / TransUnion)".to_string(),
    ];

    if input.jurisdiction == Jurisdiction::OtherStateWithoutPositiveRentReportingMandate {
        return Output {
            mode: PositiveRentReportingMode::NotApplicableJurisdictionLacksMandate,
            statutory_basis: "None — jurisdiction lacks positive rent reporting mandate".to_string(),
            notes: "Jurisdiction does not impose positive rent reporting offer obligation; landlord may voluntarily offer reporting without statutory mandate.".to_string(),
            citations,
        };
    }

    if small_landlord_exemption_applies(input) {
        return Output {
            mode: PositiveRentReportingMode::NotApplicableExemptSmallLandlord,
            statutory_basis: "Cal. Civ. Code § 1954.06 — small-landlord exemption applies".to_string(),
            notes: format!(
                "Small-landlord exemption applies: building has {} dwelling units (≤ 15 threshold). Landlord ownership structure = {:?}; owns multiple buildings = {}.",
                input.unit_count_in_building, input.landlord_ownership_structure, input.landlord_owns_multiple_buildings
            ),
            citations,
        };
    }

    if input.unit_count_in_building <= POSITIVE_RENT_REPORTING_CA_SMALL_LANDLORD_UNIT_THRESHOLD
        && input.landlord_owns_multiple_buildings
        && !matches!(
            input.landlord_ownership_structure,
            LandlordOwnershipStructure::Reit
                | LandlordOwnershipStructure::Corporation
                | LandlordOwnershipStructure::LlcWithCorporateMember
        )
    {
        return Output {
            mode: PositiveRentReportingMode::NotApplicableExemptSmallLandlord,
            statutory_basis: "Cal. Civ. Code § 1954.06 — small-landlord exemption applies".to_string(),
            notes: "Small-landlord exemption applies: ≤ 15 units and individual/non-corporate-LLC owner; corporate-multi-building disqualifier not met.".to_string(),
            citations,
        };
    }

    if input.lease_timing == LeaseTiming::NewLeaseOnOrAfterApril1_2025 && !input.offer_made_at_lease_inception {
        return Output {
            mode: PositiveRentReportingMode::ViolationOfferNotMadeAtLeaseInception,
            statutory_basis: "Cal. Civ. Code § 1954.06 — offer required at lease inception for new leases on/after April 1, 2025".to_string(),
            notes: "VIOLATION: new lease entered on/after April 1, 2025; offer of positive rent reporting NOT made at time of lease agreement.".to_string(),
            citations,
        };
    }

    if input.lease_timing == LeaseTiming::PreExistingLeaseOutstandingJanuary1_2025
        && !input.offer_to_pre_existing_tenant_made_by_april_1_2025
    {
        return Output {
            mode: PositiveRentReportingMode::ViolationOfferToPreExistingTenantsMissedApril1_2025,
            statutory_basis: "Cal. Civ. Code § 1954.06 — offer to pre-existing tenants required no later than April 1, 2025".to_string(),
            notes: "VIOLATION: lease was outstanding as of January 1, 2025; offer of positive rent reporting NOT made by the April 1, 2025 statutory deadline.".to_string(),
            citations,
        };
    }

    if !input.offer_made_annually {
        return Output {
            mode: PositiveRentReportingMode::ViolationAnnualOfferNotProvided,
            statutory_basis: "Cal. Civ. Code § 1954.06 — annual offer requirement".to_string(),
            notes: "VIOLATION: landlord failed to provide the required annual re-offer of positive rent reporting.".to_string(),
            citations,
        };
    }

    if input.landlord_actual_cost_per_month_dollars == 0 && input.monthly_fee_charged_dollars > 0 {
        return Output {
            mode: PositiveRentReportingMode::ViolationFeeChargedWhenLandlordIncursNoActualCost,
            statutory_basis: "Cal. Civ. Code § 1954.06 — no fee permitted when landlord incurs no actual cost".to_string(),
            notes: format!(
                "VIOLATION: landlord charged ${} per month but incurs $0 actual cost. Statute prohibits fee when landlord has no actual cost for the reporting service.",
                input.monthly_fee_charged_dollars
            ),
            citations,
        };
    }

    let allowed_fee = POSITIVE_RENT_REPORTING_CA_FEE_CAP_DOLLARS_PER_MONTH
        .min(input.landlord_actual_cost_per_month_dollars);

    if input.monthly_fee_charged_dollars > allowed_fee {
        return Output {
            mode: PositiveRentReportingMode::ViolationFeeExceedsLesserOf10DollarsOrActualCost,
            statutory_basis: "Cal. Civ. Code § 1954.06 — fee exceeds lesser of $10/month or actual cost".to_string(),
            notes: format!(
                "VIOLATION: landlord charges ${} per month; statutory cap = lesser of $10 OR actual cost (${}) = ${}. Excess of ${}.",
                input.monthly_fee_charged_dollars,
                input.landlord_actual_cost_per_month_dollars,
                allowed_fee,
                input.monthly_fee_charged_dollars.saturating_sub(allowed_fee)
            ),
            citations,
        };
    }

    if input.tenant_election == TenantElection::TenantElectedAndReportingActive && !input.reporting_to_nationwide_cra {
        return Output {
            mode: PositiveRentReportingMode::ViolationNoQualifyingNationwideConsumerReportingAgency,
            statutory_basis: "Cal. Civ. Code § 1954.06 — reporting must go to at least one NATIONWIDE consumer reporting agency".to_string(),
            notes: "VIOLATION: tenant elected reporting; landlord activated reporting but NOT to a nationwide CRA (Equifax / Experian / TransUnion).".to_string(),
            citations,
        };
    }

    if input.tenant_election == TenantElection::TenantElectedButReportingNotActivated {
        return Output {
            mode: PositiveRentReportingMode::ViolationTenantElectedButReportingNotActivated,
            statutory_basis: "Cal. Civ. Code § 1954.06 — landlord failed to activate reporting after tenant election".to_string(),
            notes: "VIOLATION: tenant elected positive rent reporting; landlord failed to activate reporting service. Statutory obligation requires reporting once tenant elects.".to_string(),
            citations,
        };
    }

    match input.tenant_election {
        TenantElection::TenantDeclinedOffer => Output {
            mode: PositiveRentReportingMode::CompliantTenantDeclined,
            statutory_basis: "Cal. Civ. Code § 1954.06 — offer satisfied; tenant declined".to_string(),
            notes: "COMPLIANT: landlord made statutorily required offer; tenant declined. No reporting obligation arises.".to_string(),
            citations,
        },
        TenantElection::TenantElectedAndReportingActive => Output {
            mode: PositiveRentReportingMode::CompliantTenantElectedAndReportingActive,
            statutory_basis: "Cal. Civ. Code § 1954.06 — offer satisfied; tenant elected; reporting to nationwide CRA active".to_string(),
            notes: format!(
                "COMPLIANT: offer made + tenant elected + reporting active to nationwide CRA. Fee = ${}/mo (≤ statutory cap of ${}).",
                input.monthly_fee_charged_dollars, allowed_fee
            ),
            citations,
        },
        _ => Output {
            mode: PositiveRentReportingMode::CompliantOfferMadeAtLeaseInceptionAndAnnual,
            statutory_basis: "Cal. Civ. Code § 1954.06 — offer + annual re-offer satisfied; tenant not yet elected".to_string(),
            notes: format!(
                "COMPLIANT: offer made at lease inception + annual re-offer satisfied. Tenant election status = {:?}.",
                input.tenant_election
            ),
            citations,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_california_compliant_new_lease() -> Input {
        Input {
            jurisdiction: Jurisdiction::California,
            unit_count_in_building: 50,
            landlord_owns_multiple_buildings: false,
            landlord_ownership_structure: LandlordOwnershipStructure::IndividualOrLlcWithoutCorporateMember,
            lease_timing: LeaseTiming::NewLeaseOnOrAfterApril1_2025,
            offer_made_at_lease_inception: true,
            offer_made_annually: true,
            offer_to_pre_existing_tenant_made_by_april_1_2025: true,
            tenant_election: TenantElection::TenantElectedAndReportingActive,
            monthly_fee_charged_dollars: 5,
            landlord_actual_cost_per_month_dollars: 5,
            reporting_to_nationwide_cra: true,
        }
    }

    #[test]
    fn other_jurisdiction_not_applicable() {
        let input = Input {
            jurisdiction: Jurisdiction::OtherStateWithoutPositiveRentReportingMandate,
            ..baseline_california_compliant_new_lease()
        };
        let result = check(&input);
        assert_eq!(result.mode, PositiveRentReportingMode::NotApplicableJurisdictionLacksMandate);
    }

    #[test]
    fn small_landlord_individual_exempt() {
        let input = Input {
            unit_count_in_building: 10,
            ..baseline_california_compliant_new_lease()
        };
        let result = check(&input);
        assert_eq!(result.mode, PositiveRentReportingMode::NotApplicableExemptSmallLandlord);
    }

    #[test]
    fn small_landlord_at_exactly_15_units_exempt() {
        let input = Input {
            unit_count_in_building: 15,
            ..baseline_california_compliant_new_lease()
        };
        let result = check(&input);
        assert_eq!(result.mode, PositiveRentReportingMode::NotApplicableExemptSmallLandlord);
    }

    #[test]
    fn small_landlord_at_16_units_not_exempt() {
        let input = Input {
            unit_count_in_building: 16,
            ..baseline_california_compliant_new_lease()
        };
        let result = check(&input);
        assert_eq!(result.mode, PositiveRentReportingMode::CompliantTenantElectedAndReportingActive);
    }

    #[test]
    fn small_landlord_corporate_multi_building_loses_exemption() {
        let input = Input {
            unit_count_in_building: 10,
            landlord_owns_multiple_buildings: true,
            landlord_ownership_structure: LandlordOwnershipStructure::Corporation,
            ..baseline_california_compliant_new_lease()
        };
        let result = check(&input);
        assert_eq!(result.mode, PositiveRentReportingMode::CompliantTenantElectedAndReportingActive);
    }

    #[test]
    fn small_landlord_reit_multi_building_loses_exemption() {
        let input = Input {
            unit_count_in_building: 10,
            landlord_owns_multiple_buildings: true,
            landlord_ownership_structure: LandlordOwnershipStructure::Reit,
            ..baseline_california_compliant_new_lease()
        };
        let result = check(&input);
        assert_eq!(result.mode, PositiveRentReportingMode::CompliantTenantElectedAndReportingActive);
    }

    #[test]
    fn small_landlord_llc_with_corporate_member_multi_building_loses_exemption() {
        let input = Input {
            unit_count_in_building: 10,
            landlord_owns_multiple_buildings: true,
            landlord_ownership_structure: LandlordOwnershipStructure::LlcWithCorporateMember,
            ..baseline_california_compliant_new_lease()
        };
        let result = check(&input);
        assert_eq!(result.mode, PositiveRentReportingMode::CompliantTenantElectedAndReportingActive);
    }

    #[test]
    fn small_landlord_individual_multi_building_still_exempt() {
        let input = Input {
            unit_count_in_building: 10,
            landlord_owns_multiple_buildings: true,
            landlord_ownership_structure: LandlordOwnershipStructure::IndividualOrLlcWithoutCorporateMember,
            ..baseline_california_compliant_new_lease()
        };
        let result = check(&input);
        assert_eq!(result.mode, PositiveRentReportingMode::NotApplicableExemptSmallLandlord);
    }

    #[test]
    fn new_lease_offer_not_made_at_inception_violation() {
        let input = Input {
            offer_made_at_lease_inception: false,
            ..baseline_california_compliant_new_lease()
        };
        let result = check(&input);
        assert_eq!(result.mode, PositiveRentReportingMode::ViolationOfferNotMadeAtLeaseInception);
    }

    #[test]
    fn pre_existing_lease_april_1_2025_deadline_missed_violation() {
        let input = Input {
            lease_timing: LeaseTiming::PreExistingLeaseOutstandingJanuary1_2025,
            offer_to_pre_existing_tenant_made_by_april_1_2025: false,
            ..baseline_california_compliant_new_lease()
        };
        let result = check(&input);
        assert_eq!(result.mode, PositiveRentReportingMode::ViolationOfferToPreExistingTenantsMissedApril1_2025);
    }

    #[test]
    fn annual_re_offer_not_provided_violation() {
        let input = Input {
            offer_made_annually: false,
            ..baseline_california_compliant_new_lease()
        };
        let result = check(&input);
        assert_eq!(result.mode, PositiveRentReportingMode::ViolationAnnualOfferNotProvided);
    }

    #[test]
    fn fee_charged_when_zero_actual_cost_violation() {
        let input = Input {
            landlord_actual_cost_per_month_dollars: 0,
            monthly_fee_charged_dollars: 5,
            ..baseline_california_compliant_new_lease()
        };
        let result = check(&input);
        assert_eq!(result.mode, PositiveRentReportingMode::ViolationFeeChargedWhenLandlordIncursNoActualCost);
    }

    #[test]
    fn fee_exceeds_10_dollar_cap_violation() {
        let input = Input {
            monthly_fee_charged_dollars: 15,
            landlord_actual_cost_per_month_dollars: 20,
            ..baseline_california_compliant_new_lease()
        };
        let result = check(&input);
        assert_eq!(result.mode, PositiveRentReportingMode::ViolationFeeExceedsLesserOf10DollarsOrActualCost);
        assert!(result.notes.contains("Excess of $5"));
    }

    #[test]
    fn fee_exceeds_actual_cost_when_below_10_dollar_cap_violation() {
        let input = Input {
            monthly_fee_charged_dollars: 7,
            landlord_actual_cost_per_month_dollars: 3,
            ..baseline_california_compliant_new_lease()
        };
        let result = check(&input);
        assert_eq!(result.mode, PositiveRentReportingMode::ViolationFeeExceedsLesserOf10DollarsOrActualCost);
    }

    #[test]
    fn fee_at_exactly_10_dollar_cap_compliant_when_actual_cost_higher() {
        let input = Input {
            monthly_fee_charged_dollars: 10,
            landlord_actual_cost_per_month_dollars: 12,
            ..baseline_california_compliant_new_lease()
        };
        let result = check(&input);
        assert_eq!(result.mode, PositiveRentReportingMode::CompliantTenantElectedAndReportingActive);
    }

    #[test]
    fn tenant_declined_compliant() {
        let input = Input {
            tenant_election: TenantElection::TenantDeclinedOffer,
            ..baseline_california_compliant_new_lease()
        };
        let result = check(&input);
        assert_eq!(result.mode, PositiveRentReportingMode::CompliantTenantDeclined);
    }

    #[test]
    fn tenant_elected_but_reporting_not_activated_violation() {
        let input = Input {
            tenant_election: TenantElection::TenantElectedButReportingNotActivated,
            ..baseline_california_compliant_new_lease()
        };
        let result = check(&input);
        assert_eq!(result.mode, PositiveRentReportingMode::ViolationTenantElectedButReportingNotActivated);
    }

    #[test]
    fn tenant_elected_no_nationwide_cra_violation() {
        let input = Input {
            reporting_to_nationwide_cra: false,
            ..baseline_california_compliant_new_lease()
        };
        let result = check(&input);
        assert_eq!(result.mode, PositiveRentReportingMode::ViolationNoQualifyingNationwideConsumerReportingAgency);
    }

    #[test]
    fn citations_pin_ab_2747_and_civ_code_1954_06() {
        let result = check(&baseline_california_compliant_new_lease());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("Cal. AB 2747"));
        assert!(joined.contains("Cal. Civ. Code § 1954.06"));
        assert!(joined.contains("January 1, 2025"));
        assert!(joined.contains("April 1, 2025"));
        assert!(joined.contains("15 or fewer dwelling units"));
        assert!(joined.contains("$10 per month"));
        assert!(joined.contains("Equifax / Experian / TransUnion"));
    }

    #[test]
    fn constant_pin_ab_2747_dates_and_thresholds() {
        assert_eq!(POSITIVE_RENT_REPORTING_CA_AB_2747_EFFECTIVE_YEAR, 2025);
        assert_eq!(POSITIVE_RENT_REPORTING_CA_AB_2747_EFFECTIVE_MONTH, 1);
        assert_eq!(POSITIVE_RENT_REPORTING_CA_AB_2747_EFFECTIVE_DAY, 1);
        assert_eq!(POSITIVE_RENT_REPORTING_CA_OFFER_DEADLINE_YEAR, 2025);
        assert_eq!(POSITIVE_RENT_REPORTING_CA_OFFER_DEADLINE_MONTH, 4);
        assert_eq!(POSITIVE_RENT_REPORTING_CA_OFFER_DEADLINE_DAY, 1);
        assert_eq!(POSITIVE_RENT_REPORTING_CA_FEE_CAP_DOLLARS_PER_MONTH, 10);
        assert_eq!(POSITIVE_RENT_REPORTING_CA_SMALL_LANDLORD_UNIT_THRESHOLD, 15);
    }
}
