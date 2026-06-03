//! EV charging accommodation compliance framework for residential rentals.
//!
//! Most states with significant electric vehicle adoption have enacted "right-to-charge"
//! statutes that prohibit landlords + condo associations from unreasonably restricting
//! tenant installation of EV charging stations. The federal Inflation Reduction Act of
//! 2022 expanded the § 30C alternative fuel vehicle refueling property credit (federal
//! tax credit for landlords + tenants installing EV charging). State right-to-charge
//! frameworks typically require landlord approval of tenant-paid EV charger installation
//! subject to reasonable safety, aesthetic, and reimbursement conditions.
//!
//! Jurisdictional grid:
//!
//! - CA Cal. Civ. Code § 1947.6 (AB 2565 + AB 2863): landlord must approve tenant
//!   request to install EV charging at tenant cost; applies to leases executed,
//!   renewed, or extended after July 1, 2015. Tenant must agree to: comply with
//!   landlord installation/use/maintenance/removal requirements; provide complete
//!   documents showing cost and scope; written description of work; pay all
//!   landlord costs before work commences; pay electrical costs; pay damage,
//!   maintenance, repair, removal, replacement costs. Liability insurance + UL
//!   listing typically required.
//! - IL 765 ILCS 1085 Electric Vehicle Charging Act (effective Jan 1, 2024):
//!   tenant has right to install EV charging system at tenant's expense subject
//!   to conditions. New construction (single-family + multi-unit) post-Jan 1, 2024
//!   must include EV-capable parking spaces.
//! - CO C.R.S. § 38-12-601: tenant may install Level 1 or Level 2 EV charging
//!   system at tenant expense. Landlord may NOT charge fee for placement or use
//!   (except reimbursement for actual electricity cost or reasonable access fee).
//!   Landlord may require bona-fide safety requirements + 30-day registration +
//!   reasonable aesthetic provisions.
//! - FL Fla. Stat. § 718.113(8): condominium owners (NOT tenants of rentals)
//!   have right to install EV charging in assigned or limited common element
//!   parking space. Electricity must be separately metered. Targets condo
//!   associations rather than residential landlords.
//! - HI Haw. Rev. Stat. § 196-7.5: lease/instrument provisions prohibiting EV
//!   charging at multi-family residential dwelling or townhouse parking stalls
//!   are VOID and unenforceable.
//! - NY (RPL § 234 + state energy law amendments): HOA / condo association
//!   restrictions on EV charging in assigned parking space prohibited.
//! - CT P.A. 21-191; OR ORS 90.222; MD § 11-111.4 condo + similar tenant
//!   protections in other right-to-charge states (15+ jurisdictions).
//! - DEFAULT: no statewide right-to-charge statute; landlord may impose
//!   reasonable lease restrictions on EV charging installation; tenant may
//!   negotiate as part of lease terms.
//!
//! Federal complement: § 30C alternative fuel vehicle refueling property credit
//! provides up to 30% credit (max $1,000/$30,000 residential/commercial) for EV
//! charging infrastructure installed at residential and business properties; IRA
//! 2022 (Pub. L. 117-169) extended through 2032.
//!
//! Citations (verified per WebSearch 2026-06-02):
//! - leginfo.legislature.ca.gov/faces/codes_displaySection.xhtml?lawCode=CIV&sectionNum=1947.6
//! - ilga.gov/legislation/ilcs/ilcs3.asp?ActID=4407&ChapterID=62
//! - codes.findlaw.com/co/title-38-property-real-and-personal/co-rev-st-sect-38-12-601/
//! - flsenate.gov/laws/statutes/2021/718.113
//! - pluginamerica.org/policy/right-to-charge-policies/

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    California,
    Illinois,
    Colorado,
    Florida,
    Hawaii,
    NewYork,
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LandlordResponse {
    /// Granted approval with reasonable conditions.
    GrantedApprovalWithReasonableConditions,
    /// Outright denied without basis.
    OutrightDeniedWithoutBasis,
    /// Imposed unreasonable conditions (excessive fees, prohibitively expensive
    /// modifications, aesthetic restrictions not bona-fide safety).
    UnreasonableConditionsImposed,
    /// Failed to respond within statutory window.
    NoTimelyResponseGiven,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TenantCompliance {
    /// Tenant agreed to pay all costs + UL listing + liability insurance + statutory
    /// conditions.
    AgreedToAllStatutoryConditions,
    /// Tenant refused to comply with bona-fide safety or cost conditions.
    RefusedBonaFideSafetyOrCostConditions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    CompliantLandlordApprovalWithReasonableConditions,
    OutrightDenialViolatesRightToChargeStatute,
    UnreasonableConditionsViolation,
    NoTimelyResponseViolatesStatutoryWindow,
    TenantRefusedBonaFideSafetyOrCostConditionsDeniedReasonably,
    FloridaCondoOnlyRentalNotProtected,
    DefaultJurisdictionNoRightToChargeStatute,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub jurisdiction: Jurisdiction,
    pub landlord_response: LandlordResponse,
    pub tenant_compliance: TenantCompliance,
    pub tenant_actual_damages_cents: u64,
}

pub type RentalEvChargingAccommodationInput = Input;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub severity: Severity,
    pub estimated_landlord_exposure_cents: u64,
    pub note: String,
}

pub type RentalEvChargingAccommodationOutput = Output;
pub type RentalEvChargingAccommodationResult = Output;

const TYPICAL_EV_CHARGING_TORT_BASELINE_CENTS: u64 = 1_500_000;
const TYPICAL_RIGHT_TO_CHARGE_CIVIL_PENALTY_CENTS: u64 = 1_000_000;

#[must_use]
pub fn check(input: &Input) -> Output {
    if matches!(input.jurisdiction, Jurisdiction::Florida) {
        return Output {
            severity: Severity::FloridaCondoOnlyRentalNotProtected,
            estimated_landlord_exposure_cents: 0,
            note: "Florida Stat. § 718.113(8) right-to-charge applies to CONDOMINIUM OWNERS \
                   only, NOT to tenants of residential rentals. Florida does NOT have a \
                   statewide right-to-charge statute for tenant-installed EV charging in \
                   rental properties. Landlord may impose lease restrictions on EV charging \
                   installation; tenant may negotiate as part of lease terms. Verify any \
                   municipal ordinances (Miami-Dade, Orange County, Hillsborough County) \
                   imposing landlord obligations. Federal § 30C alternative fuel vehicle \
                   refueling property credit available regardless (up to 30%, max $1,000 \
                   residential / $30,000 commercial; IRA 2022 Pub. L. 117-169 extended \
                   through 2032)."
                .to_string(),
        };
    }

    if matches!(input.jurisdiction, Jurisdiction::Default) {
        return Output {
            severity: Severity::DefaultJurisdictionNoRightToChargeStatute,
            estimated_landlord_exposure_cents: 0,
            note: "Default jurisdiction without identified right-to-charge statute. Landlord \
                   may impose reasonable lease restrictions on EV charging installation. \
                   Tenant may negotiate as part of lease terms. Verify state law: 15+ \
                   right-to-charge jurisdictions exist (CA + IL + CO + HI + NY + CT + OR + \
                   MD + others). Federal § 30C alternative fuel vehicle refueling property \
                   credit available regardless (up to 30%, max $1,000 residential / $30,000 \
                   commercial; IRA 2022 Pub. L. 117-169 extended through 2032)."
                .to_string(),
        };
    }

    if matches!(
        input.tenant_compliance,
        TenantCompliance::RefusedBonaFideSafetyOrCostConditions
    ) {
        return Output {
            severity: Severity::TenantRefusedBonaFideSafetyOrCostConditionsDeniedReasonably,
            estimated_landlord_exposure_cents: 0,
            note: format!(
                "Tenant refused to comply with bona-fide safety or cost conditions imposed \
                 by landlord (UL listing requirement + licensed-electrician installation + \
                 liability-insurance certificate + payment of installation costs upfront). \
                 {} Landlord denial is REASONABLE under statute. Tenant has no right-to-\
                 charge claim absent agreeing to standard conditions.",
                statute_citation(input.jurisdiction)
            ),
        };
    }

    match input.landlord_response {
        LandlordResponse::GrantedApprovalWithReasonableConditions => Output {
            severity: Severity::CompliantLandlordApprovalWithReasonableConditions,
            estimated_landlord_exposure_cents: 0,
            note: format!(
                "Compliant: landlord approved EV charging installation with reasonable \
                 conditions. {} Standard reasonable conditions: tenant pays installation \
                 + electrical metering + landlord oversight costs upfront; UL listing + \
                 licensed-electrician installation + liability insurance certificate; \
                 reasonable aesthetic compliance. Federal § 30C alternative fuel vehicle \
                 refueling property credit may offset 30% of cost (max $1,000 residential / \
                 $30,000 commercial; IRA 2022 extended through 2032).",
                statute_citation(input.jurisdiction)
            ),
        },
        LandlordResponse::OutrightDeniedWithoutBasis => {
            let exposure = input
                .tenant_actual_damages_cents
                .saturating_add(TYPICAL_RIGHT_TO_CHARGE_CIVIL_PENALTY_CENTS)
                .saturating_add(TYPICAL_EV_CHARGING_TORT_BASELINE_CENTS);
            Output {
                severity: Severity::OutrightDenialViolatesRightToChargeStatute,
                estimated_landlord_exposure_cents: exposure,
                note: format!(
                    "RIGHT-TO-CHARGE VIOLATION: outright denial without statutory basis. {} \
                     Tenant entitled to injunctive relief compelling approval + civil \
                     penalty + actual damages + attorney fees + emotional-distress \
                     baseline. Estimated exposure ${} = tenant damages (${}) + civil \
                     penalty estimate (${}) + tort baseline (${}).",
                    statute_citation(input.jurisdiction),
                    exposure / 100,
                    input.tenant_actual_damages_cents / 100,
                    TYPICAL_RIGHT_TO_CHARGE_CIVIL_PENALTY_CENTS / 100,
                    TYPICAL_EV_CHARGING_TORT_BASELINE_CENTS / 100
                ),
            }
        }
        LandlordResponse::UnreasonableConditionsImposed => {
            let exposure = input
                .tenant_actual_damages_cents
                .saturating_add(TYPICAL_RIGHT_TO_CHARGE_CIVIL_PENALTY_CENTS);
            Output {
                severity: Severity::UnreasonableConditionsViolation,
                estimated_landlord_exposure_cents: exposure,
                note: format!(
                    "UNREASONABLE CONDITIONS VIOLATION: landlord imposed conditions beyond \
                     bona-fide safety + reasonable cost reimbursement. {} Excessive fees, \
                     prohibitively expensive modifications, aesthetic restrictions not tied \
                     to legitimate safety basis, or onerous insurance requirements all \
                     violate right-to-charge framework. Estimated exposure ${}.",
                    statute_citation(input.jurisdiction),
                    exposure / 100
                ),
            }
        }
        LandlordResponse::NoTimelyResponseGiven => {
            let exposure = input
                .tenant_actual_damages_cents
                .saturating_add(TYPICAL_RIGHT_TO_CHARGE_CIVIL_PENALTY_CENTS);
            Output {
                severity: Severity::NoTimelyResponseViolatesStatutoryWindow,
                estimated_landlord_exposure_cents: exposure,
                note: format!(
                    "NO TIMELY RESPONSE: landlord failed to respond to tenant request within \
                     statutory window. {} Failure-to-respond typically treated as deemed \
                     approval (CA Civ. Code § 1947.6 + parallel provisions) OR as denial \
                     subject to right-to-charge enforcement. Estimated exposure ${}.",
                    statute_citation(input.jurisdiction),
                    exposure / 100
                ),
            }
        }
    }
}

fn statute_citation(jurisdiction: Jurisdiction) -> &'static str {
    match jurisdiction {
        Jurisdiction::California => {
            "CA Cal. Civ. Code § 1947.6 (AB 2565 + AB 2863) — right-to-charge applies to \
             leases executed, renewed, or extended after July 1, 2015."
        }
        Jurisdiction::Illinois => {
            "IL Electric Vehicle Charging Act (765 ILCS 1085) — effective January 1, 2024 \
             + new-construction EV-capable parking-space mandate."
        }
        Jurisdiction::Colorado => {
            "CO C.R.S. § 38-12-601 — Level 1 or Level 2 EV charging at tenant expense; \
             landlord may not charge fee except actual electricity cost or reasonable \
             access fee; safety + 30-day registration + reasonable aesthetic conditions \
             permitted."
        }
        Jurisdiction::Hawaii => {
            "HI Haw. Rev. Stat. § 196-7.5 — lease/instrument provisions prohibiting EV \
             charging at multi-family residential dwelling or townhouse parking stalls \
             are VOID and unenforceable."
        }
        Jurisdiction::NewYork => {
            "NY RPL § 234 + state energy law amendments — HOA / condo association \
             restrictions on EV charging in assigned parking space prohibited."
        }
        Jurisdiction::Florida => {
            "FL Fla. Stat. § 718.113(8) — applies to CONDOMINIUM OWNERS only; renters \
             not protected by statewide right-to-charge."
        }
        Jurisdiction::Default => "No identified right-to-charge statute.",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_ca() -> Input {
        Input {
            jurisdiction: Jurisdiction::California,
            landlord_response: LandlordResponse::GrantedApprovalWithReasonableConditions,
            tenant_compliance: TenantCompliance::AgreedToAllStatutoryConditions,
            tenant_actual_damages_cents: 5_000_00,
        }
    }

    #[test]
    fn california_compliant_approval_with_reasonable_conditions() {
        let input = base_ca();
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::CompliantLandlordApprovalWithReasonableConditions
        );
        assert!(output.note.contains("§ 1947.6"));
        assert!(output.note.contains("§ 30C"));
    }

    #[test]
    fn california_outright_denial_violation() {
        let mut input = base_ca();
        input.landlord_response = LandlordResponse::OutrightDeniedWithoutBasis;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::OutrightDenialViolatesRightToChargeStatute
        );
        // $5K + $10K + $15K = $30K
        assert_eq!(output.estimated_landlord_exposure_cents, 30_000_00);
    }

    #[test]
    fn california_unreasonable_conditions_violation() {
        let mut input = base_ca();
        input.landlord_response = LandlordResponse::UnreasonableConditionsImposed;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::UnreasonableConditionsViolation
        );
        // $5K + $10K = $15K
        assert_eq!(output.estimated_landlord_exposure_cents, 15_000_00);
    }

    #[test]
    fn california_no_timely_response_violation() {
        let mut input = base_ca();
        input.landlord_response = LandlordResponse::NoTimelyResponseGiven;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::NoTimelyResponseViolatesStatutoryWindow
        );
    }

    #[test]
    fn tenant_refused_safety_conditions_denial_reasonable() {
        let mut input = base_ca();
        input.tenant_compliance = TenantCompliance::RefusedBonaFideSafetyOrCostConditions;
        input.landlord_response = LandlordResponse::OutrightDeniedWithoutBasis;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::TenantRefusedBonaFideSafetyOrCostConditionsDeniedReasonably
        );
        assert_eq!(output.estimated_landlord_exposure_cents, 0);
    }

    #[test]
    fn illinois_765_ilcs_1085_effective_january_1_2024() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::Illinois;
        let output = check(&input);
        assert!(output.note.contains("765 ILCS 1085"));
        assert!(output.note.contains("January 1, 2024"));
    }

    #[test]
    fn colorado_38_12_601_level_1_or_2_at_tenant_expense() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::Colorado;
        let output = check(&input);
        assert!(output.note.contains("§ 38-12-601"));
        assert!(output.note.contains("Level 1 or Level 2"));
    }

    #[test]
    fn hawaii_196_7_5_void_and_unenforceable() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::Hawaii;
        let output = check(&input);
        assert!(output.note.contains("§ 196-7.5"));
        assert!(output.note.contains("VOID and unenforceable"));
    }

    #[test]
    fn new_york_rpl_234_hoa_condo_restrictions_prohibited() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::NewYork;
        let output = check(&input);
        assert!(output.note.contains("RPL § 234"));
    }

    #[test]
    fn florida_condo_only_rental_not_protected() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::Florida;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::FloridaCondoOnlyRentalNotProtected
        );
        assert!(output.note.contains("§ 718.113(8)"));
        assert!(output.note.contains("CONDOMINIUM OWNERS only"));
    }

    #[test]
    fn default_jurisdiction_no_right_to_charge_statute() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::Default;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::DefaultJurisdictionNoRightToChargeStatute
        );
        assert!(output.note.contains("15+"));
    }

    #[test]
    fn typical_ev_charging_tort_baseline_constant_pins_15000() {
        assert_eq!(TYPICAL_EV_CHARGING_TORT_BASELINE_CENTS, 1_500_000);
    }

    #[test]
    fn typical_right_to_charge_civil_penalty_constant_pins_10000() {
        assert_eq!(TYPICAL_RIGHT_TO_CHARGE_CIVIL_PENALTY_CENTS, 1_000_000);
    }

    #[test]
    fn very_large_damages_no_overflow() {
        let mut input = base_ca();
        input.landlord_response = LandlordResponse::OutrightDeniedWithoutBasis;
        input.tenant_actual_damages_cents = u64::MAX;
        let output = check(&input);
        assert_eq!(output.estimated_landlord_exposure_cents, u64::MAX);
    }

    #[test]
    fn zero_damages_uses_baseline_penalty_plus_tort() {
        let mut input = base_ca();
        input.landlord_response = LandlordResponse::OutrightDeniedWithoutBasis;
        input.tenant_actual_damages_cents = 0;
        let output = check(&input);
        // $0 + $10K + $15K = $25K
        assert_eq!(output.estimated_landlord_exposure_cents, 25_000_00);
    }

    #[test]
    fn note_pins_section_30c_alternative_fuel_credit() {
        let input = base_ca();
        let output = check(&input);
        assert!(output.note.contains("§ 30C"));
        assert!(output.note.contains("$1,000"));
        assert!(output.note.contains("$30,000"));
        assert!(output.note.contains("IRA 2022"));
    }

    #[test]
    fn note_pins_irs_2022_extended_through_2032() {
        let input = base_ca();
        let output = check(&input);
        assert!(output.note.contains("2032"));
    }

    #[test]
    fn ca_lease_july_1_2015_threshold_pinned() {
        let input = base_ca();
        let output = check(&input);
        assert!(output.note.contains("July 1, 2015"));
    }
}
