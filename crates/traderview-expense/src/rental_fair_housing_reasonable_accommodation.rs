//! Fair Housing Act reasonable-accommodation / reasonable-modification compliance.
//!
//! The federal Fair Housing Act (FHA) at 42 U.S.C. § 3604(f)(3)(B) makes it unlawful
//! for housing providers to refuse to make reasonable accommodations in rules,
//! policies, practices, or services when such accommodations are necessary to afford a
//! person with a disability equal opportunity to use and enjoy a dwelling. § 3604(f)(3)
//! (A) parallels with REASONABLE MODIFICATIONS — physical changes to the premises at
//! the tenant's expense for which the landlord cannot require restoration if normal
//! wear-and-tear. The HUD/DOJ Joint Statement on Reasonable Accommodations (May 17,
//! 2004) and HUD/DOJ Joint Statement on Reasonable Modifications (March 5, 2008)
//! establish the federal framework. State laws (CA FEHA, NY HRL, NJ LAD, MA ch. 151B)
//! provide PARALLEL protections that can exceed federal minimums.
//!
//! 2026 federal-policy reversal: HUD's May 22, 2026 internal memorandum permanently
//! cancelled prior HUD guidance and instructed agency staff to stop pursuing complaints
//! involving emotional support animals that have not been individually trained for
//! disability-related work or tasks. STATE LAWS continue to apply at higher floor.
//!
//! Key statutes:
//!
//! - 42 U.S.C. § 3604(f)(3)(B) reasonable accommodation (provider expense unless
//!   undue financial / administrative burden).
//! - 42 U.S.C. § 3604(f)(3)(A) reasonable modification (tenant expense; landlord
//!   cannot require restoration for normal wear).
//! - HUD/DOJ Joint Statement on Reasonable Accommodations (May 17, 2004).
//! - HUD/DOJ Joint Statement on Reasonable Modifications (March 5, 2008).
//! - ADA Titles II + III for common areas in privately-owned multi-family housing.
//! - CA FEHA (Cal. Gov. Code §§ 12927(c)(1) + 12955) — state protection survives 2026
//!   federal reversal. CA AB-468 (effective Jan 1, 2022) requires 30-day LMHP
//!   relationship before ESA letter issued.
//! - NY State HRL (NY Exec. Law § 296(5)(c)) + NYC HRL (NYC Admin. Code § 8-107(5)).
//! - NJ LAD (N.J.S.A. 10:5-12.4) reasonable accommodation duty.
//! - MA Gen. L. ch. 151B § 4(7A) reasonable accommodation duty.
//!
//! Citations (verified per WebSearch 2026-06-02):
//! - hud.gov/sites/documents/huddojstatement.pdf (HUD/DOJ 2004 Joint Statement)
//! - hud.gov/sites/dfiles/PIH/documents/HCV_Guidebook-Chapter_Fair-Housing_April-2025.pdf
//! - animallaw.info/sites/default/files/HUD%20FHEO%20Assistance%20Animals%20Notice%202020.pdf
//! - pettable.com/blog/california-esa-law-ab468

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    /// FHA + HUD federal floor only (post-May-22-2026 narrowed ESA framework).
    FederalFhaOnlyPost2026EsaReversal,
    /// California FEHA — survives federal reversal; AB-468 30-day LMHP rule.
    California,
    /// NY State HRL + NYC HRL.
    NewYork,
    NewJersey,
    Massachusetts,
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccommodationRequestType {
    /// Service animal individually trained for disability-related work (ADA standard).
    ServiceAnimalAdaStandard,
    /// Emotional support animal — comfort/companionship; no training requirement
    /// under pre-2026 FHA framework; California FEHA still recognizes.
    EmotionalSupportAnimal,
    /// Live-in aide for caregiver support.
    LiveInAideCaregiverSupport,
    /// Accessible parking-space assignment.
    AccessibleParkingSpaceAssignment,
    /// Lease-term modification (e.g., transfer to ground-floor unit, early lease
    /// termination for hospitalization).
    LeaseTermsModification,
    /// Physical modification to dwelling (wheelchair ramp, grab bars, lowered
    /// counters) — § 3604(f)(3)(A) at TENANT expense.
    PhysicalModificationToDwelling,
    /// Policy exception (modify rules on visitor parking, quiet hours,
    /// recreational-facility use).
    PolicyException,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DisabilityNexusStatus {
    /// Disability and nexus to accommodation documented.
    DisabilityAndNexusDocumented,
    /// Disability obvious or known (visible disability, prior knowledge).
    DisabilityObviousOrKnown,
    /// Disability claimed but undocumented and not obvious.
    DisabilityClaimedNotDocumented,
    /// No claim of disability.
    NoClaimOfDisability,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LandlordResponseStatus {
    /// Granted the requested accommodation/modification.
    GrantedAccommodationOrModification,
    /// Engaged in cooperative interactive dialogue but proposed alternative.
    EngagedInInteractiveDialogueProposedAlternative,
    /// Outright denied without interactive dialogue.
    OutrightDeniedNoInteractiveDialogue,
    /// Imposed pet fee/deposit on ESA or service animal (per se violation).
    ImposedPetFeeOrDepositOnAssistanceAnimal,
    /// Required restoration of normal-wear modification at lease-end.
    RequiredRestorationOfNormalWearModification,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    NoClaimOfDisabilityNoAccommodationDuty,
    CompliantAccommodationGranted,
    CompliantInteractiveDialogueAlternativeOffered,
    EsaPost2026FederalReversalStateLawFloorMayStillApply,
    PetFeeOnAssistanceAnimalPerSeFhaViolation,
    RestorationDemandedForNormalWearModificationViolation,
    OutrightDenialFailureToEngageInteractiveDialogue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub jurisdiction: Jurisdiction,
    pub accommodation_request_type: AccommodationRequestType,
    pub disability_nexus_status: DisabilityNexusStatus,
    pub landlord_response_status: LandlordResponseStatus,
    pub tenant_actual_damages_cents: u64,
}

pub type RentalFairHousingReasonableAccommodationInput = Input;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub severity: Severity,
    pub estimated_landlord_exposure_cents: u64,
    pub note: String,
}

pub type RentalFairHousingReasonableAccommodationOutput = Output;
pub type RentalFairHousingReasonableAccommodationResult = Output;

const FHA_CIVIL_PENALTY_FIRST_VIOLATION_CENTS: u64 = 1_978_700;
const TYPICAL_EMOTIONAL_DISTRESS_AWARD_CENTS: u64 = 1_500_000;

#[must_use]
pub fn check(input: &Input) -> Output {
    if matches!(
        input.disability_nexus_status,
        DisabilityNexusStatus::NoClaimOfDisability
    ) {
        return Output {
            severity: Severity::NoClaimOfDisabilityNoAccommodationDuty,
            estimated_landlord_exposure_cents: 0,
            note: "No claim of disability — Fair Housing Act reasonable-accommodation duty \
                   not triggered. Landlord should preserve neutral lease enforcement; \
                   accommodation duty attaches when tenant requests modification due to \
                   actual or perceived disability per 42 U.S.C. § 3604(f)(3)(B)."
                .to_string(),
        };
    }

    if matches!(
        input.landlord_response_status,
        LandlordResponseStatus::ImposedPetFeeOrDepositOnAssistanceAnimal
    ) && matches!(
        input.accommodation_request_type,
        AccommodationRequestType::ServiceAnimalAdaStandard
            | AccommodationRequestType::EmotionalSupportAnimal
    ) {
        let exposure = input
            .tenant_actual_damages_cents
            .saturating_add(TYPICAL_EMOTIONAL_DISTRESS_AWARD_CENTS)
            .saturating_add(FHA_CIVIL_PENALTY_FIRST_VIOLATION_CENTS);
        return Output {
            severity: Severity::PetFeeOnAssistanceAnimalPerSeFhaViolation,
            estimated_landlord_exposure_cents: exposure,
            note: format!(
                "PER SE FHA VIOLATION. Imposing pet fee, pet deposit, or pet rent on a \
                 service animal or emotional support animal violates 42 U.S.C. § 3604(f)(3)(B) \
                 reasonable-accommodation duty regardless of jurisdiction. HUD/DOJ Joint \
                 Statement on Reasonable Accommodations (May 17, 2004) confirms pet-fee \
                 prohibition. Estimated exposure ${} = tenant actual damages (${}) + typical \
                 emotional-distress award (${}) + 42 U.S.C. § 3612(g)(3) civil penalty (${}) \
                 + attorney fees + injunctive relief.",
                exposure / 100,
                input.tenant_actual_damages_cents / 100,
                TYPICAL_EMOTIONAL_DISTRESS_AWARD_CENTS / 100,
                FHA_CIVIL_PENALTY_FIRST_VIOLATION_CENTS / 100
            ),
        };
    }

    if matches!(
        input.landlord_response_status,
        LandlordResponseStatus::RequiredRestorationOfNormalWearModification
    ) && matches!(
        input.accommodation_request_type,
        AccommodationRequestType::PhysicalModificationToDwelling
    ) {
        let exposure = input
            .tenant_actual_damages_cents
            .saturating_add(TYPICAL_EMOTIONAL_DISTRESS_AWARD_CENTS);
        return Output {
            severity: Severity::RestorationDemandedForNormalWearModificationViolation,
            estimated_landlord_exposure_cents: exposure,
            note: format!(
                "FHA VIOLATION. 42 U.S.C. § 3604(f)(3)(A) reasonable-modification provision \
                 bars landlord from requiring restoration of physical modifications when \
                 their continued presence is consistent with normal wear and tear. \
                 Restoration may be required only for modifications that would materially \
                 affect the next tenant's quiet enjoyment of the dwelling. HUD/DOJ Joint \
                 Statement on Reasonable Modifications (March 5, 2008). Estimated exposure \
                 ${} = tenant actual damages (${}) + emotional-distress baseline (${}) + \
                 attorney fees.",
                exposure / 100,
                input.tenant_actual_damages_cents / 100,
                TYPICAL_EMOTIONAL_DISTRESS_AWARD_CENTS / 100
            ),
        };
    }

    if matches!(
        input.landlord_response_status,
        LandlordResponseStatus::OutrightDeniedNoInteractiveDialogue
    ) {
        let exposure = input
            .tenant_actual_damages_cents
            .saturating_add(TYPICAL_EMOTIONAL_DISTRESS_AWARD_CENTS)
            .saturating_add(FHA_CIVIL_PENALTY_FIRST_VIOLATION_CENTS);
        return Output {
            severity: Severity::OutrightDenialFailureToEngageInteractiveDialogue,
            estimated_landlord_exposure_cents: exposure,
            note: format!(
                "FHA VIOLATION. Outright denial without engaging in the cooperative, \
                 interactive dialogue required by HUD/DOJ Joint Statement on Reasonable \
                 Accommodations (May 17, 2004). Landlord must engage in good-faith \
                 dialogue, evaluate the nexus between disability and requested \
                 accommodation, and consider less-burdensome alternatives before denying. \
                 NYC HRL (NYC Admin. Code § 8-107(5)) codifies the interactive-dialogue \
                 duty explicitly. Estimated exposure ${} = tenant actual damages (${}) + \
                 emotional-distress baseline (${}) + 42 U.S.C. § 3612(g)(3) civil penalty \
                 (${}) + attorney fees + injunctive relief + administrative complaint with \
                 HUD or state Fair Housing agency.",
                exposure / 100,
                input.tenant_actual_damages_cents / 100,
                TYPICAL_EMOTIONAL_DISTRESS_AWARD_CENTS / 100,
                FHA_CIVIL_PENALTY_FIRST_VIOLATION_CENTS / 100
            ),
        };
    }

    if matches!(
        input.jurisdiction,
        Jurisdiction::FederalFhaOnlyPost2026EsaReversal
    ) && matches!(
        input.accommodation_request_type,
        AccommodationRequestType::EmotionalSupportAnimal
    ) {
        return Output {
            severity: Severity::EsaPost2026FederalReversalStateLawFloorMayStillApply,
            estimated_landlord_exposure_cents: 0,
            note: "ESA accommodation request in federal-only jurisdiction post HUD's May 22, \
                   2026 reversal. HUD will no longer pursue complaints involving ESAs that \
                   have not been individually trained for disability-related work or tasks. \
                   HOWEVER: (a) state laws (CA FEHA, NY HRL, NJ LAD, MA ch. 151B) continue \
                   to provide higher floor — ESA accommodation may still be required under \
                   state law even where federal protection has narrowed; (b) prior-filed \
                   complaints retain their procedural posture; (c) HUD policy reversal does \
                   not amend the statute itself — judicial interpretation of § 3604(f)(3)(B) \
                   may diverge from HUD enforcement priorities. Document the analysis and \
                   consult counsel before refusing ESA accommodation."
                .to_string(),
        };
    }

    match input.landlord_response_status {
        LandlordResponseStatus::GrantedAccommodationOrModification => Output {
            severity: Severity::CompliantAccommodationGranted,
            estimated_landlord_exposure_cents: 0,
            note: format!(
                "Compliant: requested accommodation/modification granted. Document the \
                 disability-nexus analysis (HUD/DOJ Joint Statement on Reasonable \
                 Accommodations May 17, 2004 + HUD/DOJ Joint Statement on Reasonable \
                 Modifications March 5, 2008). {} Retain documentation for the longer of \
                 (a) lease term + statute of limitations or (b) 5 years.",
                state_law_floor_citation(input.jurisdiction)
            ),
        },
        LandlordResponseStatus::EngagedInInteractiveDialogueProposedAlternative => Output {
            severity: Severity::CompliantInteractiveDialogueAlternativeOffered,
            estimated_landlord_exposure_cents: 0,
            note: format!(
                "Compliant: engaged in cooperative interactive dialogue and proposed \
                 alternative accommodation. Document the dialogue, the alternatives \
                 considered, and the rationale for the alternative chosen. {} If tenant \
                 ultimately rejects the alternative, document the offer + rejection + \
                 reasonableness analysis.",
                state_law_floor_citation(input.jurisdiction)
            ),
        },
        _ => unreachable!("other response statuses handled above"),
    }
}

fn state_law_floor_citation(jurisdiction: Jurisdiction) -> &'static str {
    match jurisdiction {
        Jurisdiction::California => {
            "CA FEHA Cal. Gov. Code §§ 12927(c)(1) + 12955 provides state-law floor that \
             survives 2026 federal reversal; AB-468 effective Jan 1, 2022 requires 30-day \
             LMHP relationship before ESA letter."
        }
        Jurisdiction::NewYork => {
            "NY State HRL Exec. Law § 296(5)(c) + NYC HRL NYC Admin. Code § 8-107(5) \
             codify interactive-dialogue duty explicitly."
        }
        Jurisdiction::NewJersey => {
            "NJ LAD N.J.S.A. 10:5-12.4 reasonable accommodation duty; NJ DCR enforcement."
        }
        Jurisdiction::Massachusetts => {
            "MA Gen. L. ch. 151B § 4(7A) reasonable accommodation duty."
        }
        Jurisdiction::FederalFhaOnlyPost2026EsaReversal => {
            "Federal floor under 42 U.S.C. § 3604(f)(3)(B) + HUD enforcement post May 22, \
             2026 reversal. ESA-specific enforcement narrowed."
        }
        Jurisdiction::Default => {
            "Federal FHA floor under 42 U.S.C. § 3604(f)(3)(B) + state law may apply."
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Input {
        Input {
            jurisdiction: Jurisdiction::California,
            accommodation_request_type:
                AccommodationRequestType::EmotionalSupportAnimal,
            disability_nexus_status:
                DisabilityNexusStatus::DisabilityAndNexusDocumented,
            landlord_response_status:
                LandlordResponseStatus::GrantedAccommodationOrModification,
            tenant_actual_damages_cents: 5_000_00,
        }
    }

    #[test]
    fn no_claim_of_disability_no_accommodation_duty() {
        let mut input = base();
        input.disability_nexus_status = DisabilityNexusStatus::NoClaimOfDisability;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::NoClaimOfDisabilityNoAccommodationDuty
        );
        assert!(output.note.contains("§ 3604(f)(3)(B)"));
    }

    #[test]
    fn compliant_accommodation_granted() {
        let input = base();
        let output = check(&input);
        assert_eq!(output.severity, Severity::CompliantAccommodationGranted);
        assert!(output.note.contains("CA FEHA"));
        assert!(output.note.contains("AB-468"));
    }

    #[test]
    fn pet_fee_on_assistance_animal_per_se_violation() {
        let mut input = base();
        input.landlord_response_status =
            LandlordResponseStatus::ImposedPetFeeOrDepositOnAssistanceAnimal;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::PetFeeOnAssistanceAnimalPerSeFhaViolation
        );
        assert!(output.note.contains("PER SE"));
        assert!(output.note.contains("HUD/DOJ Joint Statement"));
        assert!(output.note.contains("$19787"));
    }

    #[test]
    fn pet_fee_on_service_animal_per_se_violation() {
        let mut input = base();
        input.accommodation_request_type =
            AccommodationRequestType::ServiceAnimalAdaStandard;
        input.landlord_response_status =
            LandlordResponseStatus::ImposedPetFeeOrDepositOnAssistanceAnimal;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::PetFeeOnAssistanceAnimalPerSeFhaViolation
        );
    }

    #[test]
    fn restoration_demanded_normal_wear_modification_violation() {
        let mut input = base();
        input.accommodation_request_type =
            AccommodationRequestType::PhysicalModificationToDwelling;
        input.landlord_response_status =
            LandlordResponseStatus::RequiredRestorationOfNormalWearModification;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::RestorationDemandedForNormalWearModificationViolation
        );
        assert!(output.note.contains("§ 3604(f)(3)(A)"));
        assert!(output.note.contains("March 5, 2008"));
    }

    #[test]
    fn outright_denial_no_interactive_dialogue_violation() {
        let mut input = base();
        input.landlord_response_status =
            LandlordResponseStatus::OutrightDeniedNoInteractiveDialogue;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::OutrightDenialFailureToEngageInteractiveDialogue
        );
        assert!(output.note.contains("interactive dialogue"));
        assert!(output.note.contains("§ 8-107(5)"));
    }

    #[test]
    fn engaged_in_interactive_dialogue_alternative_compliant() {
        let mut input = base();
        input.landlord_response_status =
            LandlordResponseStatus::EngagedInInteractiveDialogueProposedAlternative;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::CompliantInteractiveDialogueAlternativeOffered
        );
    }

    #[test]
    fn federal_only_esa_post_2026_reversal_state_floor_note() {
        let mut input = base();
        input.jurisdiction = Jurisdiction::FederalFhaOnlyPost2026EsaReversal;
        input.accommodation_request_type =
            AccommodationRequestType::EmotionalSupportAnimal;
        input.landlord_response_status =
            LandlordResponseStatus::EngagedInInteractiveDialogueProposedAlternative;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::EsaPost2026FederalReversalStateLawFloorMayStillApply
        );
        assert!(output.note.contains("May 22, 2026"));
        assert!(output.note.contains("higher floor"));
    }

    #[test]
    fn california_state_floor_survives_federal_reversal() {
        let mut input = base();
        input.jurisdiction = Jurisdiction::California;
        let output = check(&input);
        assert!(output.note.contains("survives 2026 federal reversal"));
    }

    #[test]
    fn new_york_interactive_dialogue_codified() {
        let mut input = base();
        input.jurisdiction = Jurisdiction::NewYork;
        let output = check(&input);
        assert!(output.note.contains("§ 296(5)(c)"));
        assert!(output.note.contains("§ 8-107(5)"));
    }

    #[test]
    fn new_jersey_lad_citation() {
        let mut input = base();
        input.jurisdiction = Jurisdiction::NewJersey;
        let output = check(&input);
        assert!(output.note.contains("N.J.S.A. 10:5-12.4"));
    }

    #[test]
    fn massachusetts_151b_4_7a_citation() {
        let mut input = base();
        input.jurisdiction = Jurisdiction::Massachusetts;
        let output = check(&input);
        assert!(output.note.contains("§ 4(7A)"));
    }

    #[test]
    fn service_animal_request_separately_protected() {
        let mut input = base();
        input.accommodation_request_type =
            AccommodationRequestType::ServiceAnimalAdaStandard;
        let output = check(&input);
        assert_eq!(output.severity, Severity::CompliantAccommodationGranted);
    }

    #[test]
    fn live_in_aide_request_compliant_when_granted() {
        let mut input = base();
        input.accommodation_request_type =
            AccommodationRequestType::LiveInAideCaregiverSupport;
        let output = check(&input);
        assert_eq!(output.severity, Severity::CompliantAccommodationGranted);
    }

    #[test]
    fn accessible_parking_compliant_when_granted() {
        let mut input = base();
        input.accommodation_request_type =
            AccommodationRequestType::AccessibleParkingSpaceAssignment;
        let output = check(&input);
        assert_eq!(output.severity, Severity::CompliantAccommodationGranted);
    }

    #[test]
    fn fha_civil_penalty_constant_pins_19787() {
        assert_eq!(FHA_CIVIL_PENALTY_FIRST_VIOLATION_CENTS, 1_978_700);
    }

    #[test]
    fn typical_emotional_distress_constant_pins_15000() {
        assert_eq!(TYPICAL_EMOTIONAL_DISTRESS_AWARD_CENTS, 1_500_000);
    }

    #[test]
    fn very_large_damages_no_overflow() {
        let mut input = base();
        input.landlord_response_status =
            LandlordResponseStatus::OutrightDeniedNoInteractiveDialogue;
        input.tenant_actual_damages_cents = u64::MAX;
        let output = check(&input);
        assert_eq!(output.estimated_landlord_exposure_cents, u64::MAX);
    }

    #[test]
    fn zero_damages_uses_baseline_emotional_distress_plus_penalty() {
        let mut input = base();
        input.landlord_response_status =
            LandlordResponseStatus::OutrightDeniedNoInteractiveDialogue;
        input.tenant_actual_damages_cents = 0;
        let output = check(&input);
        // $15,000 emotional distress + $19,787 civil penalty = $34,787
        assert_eq!(output.estimated_landlord_exposure_cents, 3_478_700);
    }
}
