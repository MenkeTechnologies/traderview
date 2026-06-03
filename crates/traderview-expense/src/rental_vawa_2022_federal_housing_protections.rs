//! Violence Against Women Act Reauthorization Act of 2022
//! (VAWA 2022) Federal Housing Protections Compliance Module
//! (34 U.S.C. § 12491).
//!
//! Pure-compute check for landlord / public housing agency /
//! owner / manager compliance with VAWA 2022 housing
//! protections under 34 U.S.C. § 12491. Covers anti-
//! discrimination, lease bifurcation, emergency transfer plan,
//! confidentiality, VAWA notice (HUD Form 5380), and
//! certification (HUD Form 5382) requirements across 16+
//! covered federally-assisted housing programs. Distinct from
//! state-law DV statutes covered elsewhere in this codebase
//! (rental_domestic_violence_lock_change_lease_termination,
//! tenant_domestic_violence_lease_termination, dv_termination)
//! because VAWA 2022 imposes parallel federal floor on HUD,
//! USDA, and LIHTC properties.
//!
//! Web research (verified 2026-06-03):
//! - **Violence Against Women Act Reauthorization Act of 2022**
//!   (Public Law 117-103; signed by President Biden on **March
//!   15, 2022**); housing rights subpart effective **October 1,
//!   2022**; codified at **34 U.S.C. § 12491** ([DOJ Civil
//!   Rights Division — VAWA 2022 Housing Rights Subpart](https://www.justice.gov/crt/violence-against-women-act-reauthorization-act-2022-vawa-2022-housing-rights-subpart);
//!   [HUD VAWA](https://www.hud.gov/vawa)).
//! - **HUD Federal Register Notice — January 4, 2023** (88 FR
//!   482): The Violence Against Women Act Reauthorization Act
//!   of 2022 — Overview of Applicability to HUD Programs
//!   ([Federal Register Jan 4, 2023](https://www.federalregister.gov/documents/2023/01/04/2022-28073/the-violence-against-women-act-reauthorization-act-of-2022-overview-of-applicability-to-hud-programs)).
//! - **Covered Housing Programs**: VAWA 2022 applies to **16+
//!   federally-assisted housing programs**: (1) Public Housing;
//!   (2) Section 8 Housing Choice Voucher (tenant-based); (3)
//!   Section 8 Project-Based Rental Assistance; (4) Section 8
//!   Moderate Rehabilitation; (5) Section 202 Supportive
//!   Housing for the Elderly; (6) Section 811 Supportive
//!   Housing for Persons with Disabilities; (7) HOPWA (Housing
//!   Opportunities for Persons With AIDS); (8) HOME Investment
//!   Partnerships Program; (9) Continuum of Care (CoC); (10)
//!   Emergency Solutions Grants (ESG); (11) McKinney-Vento
//!   Homeless Assistance Act Title IV; (12) LIHTC (Low Income
//!   Housing Tax Credit) under IRC § 42; (13) USDA Rural
//!   Development housing programs; (14) Section 221(d)(3)
//!   BMIR; (15) Section 236; (16) Section 1437f programs
//!   ([HUD Chart VAWA Covered Housing](https://files.hudexchange.info/resources/documents/Chart-VAWA-Covered-Housing.pdf)).
//! - **Anti-Discrimination Protection (34 USC § 12491(b)(1))**:
//!   applicant for assistance or tenant may NOT be denied
//!   admission, denied assistance, terminated from
//!   participation, or evicted from housing because the
//!   applicant or tenant is or has been a victim of domestic
//!   violence, dating violence, sexual assault, or stalking.
//! - **Lease Bifurcation (34 USC § 12491(b)(3))**: PHA or owner
//!   or manager of covered housing program may **bifurcate
//!   lease** to evict, remove, or terminate assistance to
//!   individual tenant who **engages in criminal activity
//!   directly relating to domestic violence, dating violence,
//!   sexual assault, or stalking against an affiliated
//!   individual or other individual**, without evicting,
//!   removing, terminating assistance to, or otherwise
//!   penalizing the **victim** of such criminal activity who
//!   is also a tenant or lawful occupant of the housing.
//! - **Emergency Transfer Plan (34 USC § 12491(e))**: every
//!   covered housing provider must adopt and make available an
//!   **emergency transfer plan**; victim has right to request
//!   emergency transfer to safe unit.
//! - **Documentation/Certification (34 USC § 12491(c)(3))**:
//!   victim may submit any of: (1) HUD Form 5382
//!   self-certification; (2) federal/state/tribal/local police
//!   record or court record; (3) documentation signed by a
//!   victim service provider, attorney, medical professional,
//!   or mental health professional from whom the victim has
//!   sought assistance; (4) any other statement or evidence
//!   provided by the victim or third party.
//! - **VAWA Notice (HUD Form 5380)**: must be provided at
//!   admission, when assistance is denied, and at termination
//!   notice; explains victim rights, certification options,
//!   confidentiality, and emergency transfer plan.
//! - **Confidentiality (34 USC § 12491(c)(4))**: all
//!   information submitted to housing provider must be kept
//!   confidential except as required by law, in eviction
//!   proceedings, or with written consent of victim.
//! - **Affiliated Individual Definition**: spouse, parent,
//!   brother, sister, child, dependent of victim, or any
//!   individual living in the household of the victim and
//!   related to the victim by blood, marriage, or other
//!   legally-recognized relationship.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const VAWA_2022_PL_NUMBER: u32 = 117_103;
pub const VAWA_2022_SIGNED_DATE_YEAR: u32 = 2022;
pub const VAWA_2022_SIGNED_DATE_MONTH: u32 = 3;
pub const VAWA_2022_SIGNED_DATE_DAY: u32 = 15;
pub const VAWA_2022_HOUSING_EFFECTIVE_DATE_YEAR: u32 = 2022;
pub const VAWA_2022_HOUSING_EFFECTIVE_DATE_MONTH: u32 = 10;
pub const VAWA_2022_HOUSING_EFFECTIVE_DATE_DAY: u32 = 1;
pub const VAWA_2022_HUD_FR_NOTICE_YEAR: u32 = 2023;
pub const VAWA_2022_HUD_FR_NOTICE_MONTH: u32 = 1;
pub const VAWA_2022_HUD_FR_NOTICE_DAY: u32 = 4;
pub const VAWA_2022_HUD_FR_VOLUME: u32 = 88;
pub const VAWA_2022_HUD_FR_PAGE: u32 = 482;
pub const VAWA_2022_COVERED_PROGRAMS_COUNT: u32 = 16;
pub const HUD_FORM_VAWA_NOTICE_NUMBER: u32 = 5_380;
pub const HUD_FORM_VAWA_CERTIFICATION_NUMBER: u32 = 5_382;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CoveredHousingProgram {
    PublicHousing,
    Section8HousingChoiceVoucherTenantBased,
    Section8ProjectBasedRentalAssistance,
    Section8ModerateRehabilitation,
    Section202SupportiveHousingElderly,
    Section811SupportiveHousingDisabled,
    Hopwa,
    HomeInvestmentPartnerships,
    ContinuumOfCare,
    EmergencySolutionsGrants,
    McKinneyVentoHomelessTitleIv,
    LihtcUnderSection42,
    UsdaRuralDevelopmentHousing,
    Section221d3Bmir,
    Section236,
    Section1437fOther,
    NotACoveredVawaHousingProgram,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VictimDocumentationType {
    HudForm5382SelfCertification,
    PoliceOrCourtRecord,
    VictimServiceProviderAttorneyMedicalSignedStatement,
    OtherStatementOrEvidence,
    NoDocumentationProvided,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LandlordAction {
    AdmissionDenial,
    AssistanceDenial,
    EvictionInitiated,
    TerminationOfParticipation,
    LeaseBifurcationAgainstPerpetrator,
    EmergencyTransferRequestProcessing,
    NoAdverseAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VictimBifurcationStatus {
    VictimRetainedAsTenantOrLawfulOccupant,
    VictimEvictedAlongsidePerpetrator,
    NoVictimPresent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Vawa2022Mode {
    NotApplicableNotACoveredVawaHousingProgram,
    CompliantSection12491B1AntiDiscriminationProtectionRespected,
    CompliantSection12491B3LeaseBifurcationAgainstPerpetratorVictimRetained,
    CompliantSection12491EEmergencyTransferPlanProvided,
    CompliantHudForm5380NoticeProvidedAtAdmissionDenialOrTermination,
    CompliantHudForm5382SelfCertificationAccepted,
    CompliantSection12491C4ConfidentialityMaintained,
    ViolationSection12491B1DiscriminationBasedOnVictimStatusOfDvOrSaOrStalking,
    ViolationSection12491B3LeaseBifurcationVictimImproperlyEvictedAlongsidePerpetrator,
    ViolationSection12491EEmergencyTransferPlanNotProvidedOrRequestIgnored,
    ViolationHudForm5380VawaNoticeNotProvided,
    ViolationVictimDocumentationRefusedBeyondHudForm5382SafeHarbor,
    ViolationSection12491C4ConfidentialityBreached,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub covered_housing_program: CoveredHousingProgram,
    pub victim_documentation_type: VictimDocumentationType,
    pub landlord_action: LandlordAction,
    pub victim_bifurcation_status: VictimBifurcationStatus,
    pub adverse_action_based_on_victim_status: bool,
    pub hud_form_5380_vawa_notice_provided: bool,
    pub hud_form_5382_self_certification_accepted: bool,
    pub emergency_transfer_plan_provided: bool,
    pub emergency_transfer_request_processed: bool,
    pub confidentiality_maintained: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Vawa2022Mode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type RentalVawa2022FederalHousingProtectionsInput = Input;
pub type RentalVawa2022FederalHousingProtectionsOutput = Output;
pub type RentalVawa2022FederalHousingProtectionsResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "Violence Against Women Act Reauthorization Act of 2022 (Public Law 117-103; signed by President Biden on March 15, 2022); housing rights subpart effective October 1, 2022; codified at 34 U.S.C. § 12491".to_string(),
        "HUD Federal Register Notice — January 4, 2023 (88 FR 482) — The Violence Against Women Act Reauthorization Act of 2022 Overview of Applicability to HUD Programs".to_string(),
        "Covered Housing Programs (16+) — Public Housing; Section 8 HCV (tenant-based); Section 8 PBRA; Section 8 Moderate Rehabilitation; Section 202 Supportive Housing Elderly; Section 811 Supportive Housing Disabled; HOPWA; HOME; CoC; ESG; McKinney-Vento Title IV; LIHTC (IRC § 42); USDA Rural Development; Section 221(d)(3) BMIR; Section 236; Section 1437f".to_string(),
        "34 U.S.C. § 12491(b)(1) — anti-discrimination; applicant or tenant may NOT be denied admission, denied assistance, terminated from participation, or evicted because applicant or tenant is or has been victim of domestic violence, dating violence, sexual assault, or stalking".to_string(),
        "34 U.S.C. § 12491(b)(3) — lease bifurcation; PHA/owner/manager may bifurcate lease to evict perpetrator who engaged in criminal activity directly relating to DV/DV/SA/stalking against affiliated individual or other individual, WITHOUT evicting victim who is also tenant or lawful occupant".to_string(),
        "34 U.S.C. § 12491(c)(3) — victim documentation; victim may submit any of (1) HUD Form 5382 self-certification; (2) federal/state/tribal/local police record or court record; (3) documentation signed by victim service provider, attorney, medical or mental health professional; (4) other statement or evidence".to_string(),
        "34 U.S.C. § 12491(c)(4) — confidentiality; all information submitted must be kept confidential except as required by law, in eviction proceedings, or with written consent of victim".to_string(),
        "34 U.S.C. § 12491(e) — emergency transfer plan; every covered housing provider must adopt and make available emergency transfer plan; victim has right to request emergency transfer to safe unit".to_string(),
        "HUD Form 5380 (Notice of Occupancy Rights under VAWA) — must be provided at admission, when assistance is denied, and at termination notice".to_string(),
        "HUD Form 5382 (Certification of Domestic Violence, Dating Violence, Sexual Assault, or Stalking, and Alternate Documentation) — primary victim self-certification form".to_string(),
        "Affiliated Individual Definition — spouse, parent, brother, sister, child, dependent of victim, or individual living in household of victim and related by blood, marriage, or other legally-recognized relationship".to_string(),
        "DOJ Civil Rights Division — VAWA 2022 Housing Rights Subpart primary federal enforcement page".to_string(),
        "HUD VAWA Resources — primary HUD landing page with regulatory guidance and implementation notices".to_string(),
        "HUD Exchange Chart of VAWA Covered Housing — comprehensive program crosswalk".to_string(),
        "WomensLaw.org VAWA Housing Protections — practitioner guide".to_string(),
    ];

    if input.covered_housing_program == CoveredHousingProgram::NotACoveredVawaHousingProgram {
        return Output {
            mode: Vawa2022Mode::NotApplicableNotACoveredVawaHousingProgram,
            statutory_basis: "VAWA 2022 — 34 U.S.C. § 12491 covered housing program list".to_string(),
            notes: "NOT APPLICABLE: property not within any of 16+ VAWA 2022 covered federally-assisted housing programs; private market rental subject to state-law DV protections only.".to_string(),
            citations,
        };
    }

    if input.adverse_action_based_on_victim_status
        && matches!(
            input.landlord_action,
            LandlordAction::AdmissionDenial
                | LandlordAction::AssistanceDenial
                | LandlordAction::EvictionInitiated
                | LandlordAction::TerminationOfParticipation
        )
    {
        return Output {
            mode: Vawa2022Mode::ViolationSection12491B1DiscriminationBasedOnVictimStatusOfDvOrSaOrStalking,
            statutory_basis: "34 U.S.C. § 12491(b)(1) — anti-discrimination prohibition".to_string(),
            notes: format!(
                "VIOLATION: adverse action ({:?}) based on victim's status as victim of domestic violence, dating violence, sexual assault, or stalking; § 12491(b)(1) prohibits such discrimination.",
                input.landlord_action
            ),
            citations,
        };
    }

    if input.landlord_action == LandlordAction::LeaseBifurcationAgainstPerpetrator
        && input.victim_bifurcation_status == VictimBifurcationStatus::VictimEvictedAlongsidePerpetrator
    {
        return Output {
            mode: Vawa2022Mode::ViolationSection12491B3LeaseBifurcationVictimImproperlyEvictedAlongsidePerpetrator,
            statutory_basis: "34 U.S.C. § 12491(b)(3) — lease bifurcation must preserve victim's tenancy".to_string(),
            notes: "VIOLATION: lease bifurcation procedure used to evict perpetrator but victim improperly evicted alongside perpetrator; § 12491(b)(3) requires victim to remain as tenant or lawful occupant.".to_string(),
            citations,
        };
    }

    if !input.hud_form_5380_vawa_notice_provided
        && matches!(
            input.landlord_action,
            LandlordAction::AdmissionDenial
                | LandlordAction::AssistanceDenial
                | LandlordAction::EvictionInitiated
                | LandlordAction::TerminationOfParticipation
        )
    {
        return Output {
            mode: Vawa2022Mode::ViolationHudForm5380VawaNoticeNotProvided,
            statutory_basis: "HUD Form 5380 (Notice of Occupancy Rights under VAWA) — required at admission, denial, or termination".to_string(),
            notes: format!(
                "VIOLATION: adverse action ({:?}) taken without providing HUD Form 5380 VAWA notice; notice must be provided at admission, when assistance is denied, and at termination notice.",
                input.landlord_action
            ),
            citations,
        };
    }

    if input.landlord_action == LandlordAction::EmergencyTransferRequestProcessing
        && (!input.emergency_transfer_plan_provided || !input.emergency_transfer_request_processed)
    {
        return Output {
            mode: Vawa2022Mode::ViolationSection12491EEmergencyTransferPlanNotProvidedOrRequestIgnored,
            statutory_basis: "34 U.S.C. § 12491(e) — emergency transfer plan and request processing".to_string(),
            notes: "VIOLATION: emergency transfer plan not provided OR victim's emergency transfer request ignored; § 12491(e) requires covered housing provider to adopt and make available emergency transfer plan and process victim requests.".to_string(),
            citations,
        };
    }

    if input.victim_documentation_type == VictimDocumentationType::NoDocumentationProvided
        && !matches!(input.landlord_action, LandlordAction::NoAdverseAction)
    {
        return Output {
            mode: Vawa2022Mode::ViolationVictimDocumentationRefusedBeyondHudForm5382SafeHarbor,
            statutory_basis: "34 U.S.C. § 12491(c)(3) — documentation safe harbor".to_string(),
            notes: "Note: when victim has not submitted any documentation, landlord may proceed with normal lease enforcement; but if victim submits ANY of HUD Form 5382, police/court record, victim service provider statement, or other evidence, landlord must accept.".to_string(),
            citations,
        };
    }

    if !input.confidentiality_maintained
        && input.victim_documentation_type != VictimDocumentationType::NoDocumentationProvided
    {
        return Output {
            mode: Vawa2022Mode::ViolationSection12491C4ConfidentialityBreached,
            statutory_basis: "34 U.S.C. § 12491(c)(4) — confidentiality requirement".to_string(),
            notes: "VIOLATION: confidentiality of victim documentation breached; § 12491(c)(4) requires all submitted information to be kept confidential except as required by law, in eviction proceedings, or with victim's written consent.".to_string(),
            citations,
        };
    }

    if input.landlord_action == LandlordAction::LeaseBifurcationAgainstPerpetrator
        && input.victim_bifurcation_status
            == VictimBifurcationStatus::VictimRetainedAsTenantOrLawfulOccupant
    {
        return Output {
            mode: Vawa2022Mode::CompliantSection12491B3LeaseBifurcationAgainstPerpetratorVictimRetained,
            statutory_basis: "34 U.S.C. § 12491(b)(3) — lease bifurcation against perpetrator with victim retained".to_string(),
            notes: "COMPLIANT: § 12491(b)(3) lease bifurcation used to evict perpetrator engaging in DV/DV/SA/stalking criminal activity; victim retained as tenant or lawful occupant of housing.".to_string(),
            citations,
        };
    }

    if input.landlord_action == LandlordAction::EmergencyTransferRequestProcessing
        && input.emergency_transfer_plan_provided
        && input.emergency_transfer_request_processed
    {
        return Output {
            mode: Vawa2022Mode::CompliantSection12491EEmergencyTransferPlanProvided,
            statutory_basis: "34 U.S.C. § 12491(e) — emergency transfer plan provided and request processed".to_string(),
            notes: "COMPLIANT: § 12491(e) emergency transfer plan provided to victim AND victim's emergency transfer request processed per plan procedures.".to_string(),
            citations,
        };
    }

    if input.victim_documentation_type == VictimDocumentationType::HudForm5382SelfCertification
        && input.hud_form_5382_self_certification_accepted
    {
        return Output {
            mode: Vawa2022Mode::CompliantHudForm5382SelfCertificationAccepted,
            statutory_basis: "34 U.S.C. § 12491(c)(3) — HUD Form 5382 self-certification documentation".to_string(),
            notes: "COMPLIANT: § 12491(c)(3) HUD Form 5382 self-certification accepted by landlord; victim documentation safe harbor satisfied.".to_string(),
            citations,
        };
    }

    if input.hud_form_5380_vawa_notice_provided {
        return Output {
            mode: Vawa2022Mode::CompliantHudForm5380NoticeProvidedAtAdmissionDenialOrTermination,
            statutory_basis: "HUD Form 5380 — Notice of Occupancy Rights under VAWA provided".to_string(),
            notes: "COMPLIANT: HUD Form 5380 VAWA Notice of Occupancy Rights provided at appropriate trigger event (admission, denial, or termination).".to_string(),
            citations,
        };
    }

    Output {
        mode: Vawa2022Mode::CompliantSection12491B1AntiDiscriminationProtectionRespected,
        statutory_basis: "34 U.S.C. § 12491(b)(1) — anti-discrimination protection respected".to_string(),
        notes: "COMPLIANT: no adverse action taken based on victim status; § 12491(b)(1) anti-discrimination protection respected.".to_string(),
        citations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_compliant_section_8_hcv_no_adverse_action() -> Input {
        Input {
            covered_housing_program: CoveredHousingProgram::Section8HousingChoiceVoucherTenantBased,
            victim_documentation_type: VictimDocumentationType::HudForm5382SelfCertification,
            landlord_action: LandlordAction::NoAdverseAction,
            victim_bifurcation_status: VictimBifurcationStatus::NoVictimPresent,
            adverse_action_based_on_victim_status: false,
            hud_form_5380_vawa_notice_provided: true,
            hud_form_5382_self_certification_accepted: true,
            emergency_transfer_plan_provided: true,
            emergency_transfer_request_processed: true,
            confidentiality_maintained: true,
        }
    }

    #[test]
    fn non_covered_program_not_applicable() {
        let input = Input {
            covered_housing_program: CoveredHousingProgram::NotACoveredVawaHousingProgram,
            ..baseline_compliant_section_8_hcv_no_adverse_action()
        };
        let result = check(&input);
        assert_eq!(result.mode, Vawa2022Mode::NotApplicableNotACoveredVawaHousingProgram);
    }

    #[test]
    fn anti_discrimination_violation() {
        let input = Input {
            landlord_action: LandlordAction::AdmissionDenial,
            adverse_action_based_on_victim_status: true,
            ..baseline_compliant_section_8_hcv_no_adverse_action()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Vawa2022Mode::ViolationSection12491B1DiscriminationBasedOnVictimStatusOfDvOrSaOrStalking
        );
    }

    #[test]
    fn lease_bifurcation_with_victim_retained_compliant() {
        let input = Input {
            landlord_action: LandlordAction::LeaseBifurcationAgainstPerpetrator,
            victim_bifurcation_status: VictimBifurcationStatus::VictimRetainedAsTenantOrLawfulOccupant,
            ..baseline_compliant_section_8_hcv_no_adverse_action()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Vawa2022Mode::CompliantSection12491B3LeaseBifurcationAgainstPerpetratorVictimRetained
        );
    }

    #[test]
    fn lease_bifurcation_victim_improperly_evicted_violation() {
        let input = Input {
            landlord_action: LandlordAction::LeaseBifurcationAgainstPerpetrator,
            victim_bifurcation_status: VictimBifurcationStatus::VictimEvictedAlongsidePerpetrator,
            ..baseline_compliant_section_8_hcv_no_adverse_action()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Vawa2022Mode::ViolationSection12491B3LeaseBifurcationVictimImproperlyEvictedAlongsidePerpetrator
        );
    }

    #[test]
    fn hud_form_5380_not_provided_at_termination_violation() {
        let input = Input {
            landlord_action: LandlordAction::EvictionInitiated,
            hud_form_5380_vawa_notice_provided: false,
            ..baseline_compliant_section_8_hcv_no_adverse_action()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Vawa2022Mode::ViolationHudForm5380VawaNoticeNotProvided
        );
    }

    #[test]
    fn emergency_transfer_request_compliant() {
        let input = Input {
            landlord_action: LandlordAction::EmergencyTransferRequestProcessing,
            ..baseline_compliant_section_8_hcv_no_adverse_action()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Vawa2022Mode::CompliantSection12491EEmergencyTransferPlanProvided
        );
    }

    #[test]
    fn emergency_transfer_request_ignored_violation() {
        let input = Input {
            landlord_action: LandlordAction::EmergencyTransferRequestProcessing,
            emergency_transfer_request_processed: false,
            ..baseline_compliant_section_8_hcv_no_adverse_action()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Vawa2022Mode::ViolationSection12491EEmergencyTransferPlanNotProvidedOrRequestIgnored
        );
    }

    #[test]
    fn emergency_transfer_plan_not_provided_violation() {
        let input = Input {
            landlord_action: LandlordAction::EmergencyTransferRequestProcessing,
            emergency_transfer_plan_provided: false,
            ..baseline_compliant_section_8_hcv_no_adverse_action()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Vawa2022Mode::ViolationSection12491EEmergencyTransferPlanNotProvidedOrRequestIgnored
        );
    }

    #[test]
    fn hud_form_5382_self_certification_compliant() {
        let result = check(&baseline_compliant_section_8_hcv_no_adverse_action());
        assert_eq!(
            result.mode,
            Vawa2022Mode::CompliantHudForm5382SelfCertificationAccepted
        );
    }

    #[test]
    fn confidentiality_breach_violation() {
        let input = Input {
            confidentiality_maintained: false,
            ..baseline_compliant_section_8_hcv_no_adverse_action()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Vawa2022Mode::ViolationSection12491C4ConfidentialityBreached
        );
    }

    #[test]
    fn lihtc_covered_compliant() {
        let input = Input {
            covered_housing_program: CoveredHousingProgram::LihtcUnderSection42,
            ..baseline_compliant_section_8_hcv_no_adverse_action()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Vawa2022Mode::CompliantHudForm5382SelfCertificationAccepted
        );
    }

    #[test]
    fn usda_rural_development_covered_compliant() {
        let input = Input {
            covered_housing_program: CoveredHousingProgram::UsdaRuralDevelopmentHousing,
            ..baseline_compliant_section_8_hcv_no_adverse_action()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Vawa2022Mode::CompliantHudForm5382SelfCertificationAccepted
        );
    }

    #[test]
    fn public_housing_with_anti_discrimination_compliant() {
        let input = Input {
            covered_housing_program: CoveredHousingProgram::PublicHousing,
            landlord_action: LandlordAction::AdmissionDenial,
            adverse_action_based_on_victim_status: false,
            ..baseline_compliant_section_8_hcv_no_adverse_action()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Vawa2022Mode::CompliantHudForm5382SelfCertificationAccepted
        );
    }

    #[test]
    fn citations_pin_vawa_2022_subsections_and_forms() {
        let result = check(&baseline_compliant_section_8_hcv_no_adverse_action());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("Violence Against Women Act Reauthorization Act of 2022"));
        assert!(joined.contains("Public Law 117-103"));
        assert!(joined.contains("March 15, 2022"));
        assert!(joined.contains("October 1, 2022"));
        assert!(joined.contains("34 U.S.C. § 12491"));
        assert!(joined.contains("88 FR 482"));
        assert!(joined.contains("January 4, 2023"));
        assert!(joined.contains("16+"));
        assert!(joined.contains("Public Housing"));
        assert!(joined.contains("Section 8 HCV"));
        assert!(joined.contains("Section 8 PBRA"));
        assert!(joined.contains("Section 202"));
        assert!(joined.contains("Section 811"));
        assert!(joined.contains("HOPWA"));
        assert!(joined.contains("HOME"));
        assert!(joined.contains("CoC"));
        assert!(joined.contains("ESG"));
        assert!(joined.contains("McKinney-Vento"));
        assert!(joined.contains("LIHTC"));
        assert!(joined.contains("USDA Rural Development"));
        assert!(joined.contains("§ 12491(b)(1)"));
        assert!(joined.contains("§ 12491(b)(3)"));
        assert!(joined.contains("§ 12491(c)(3)"));
        assert!(joined.contains("§ 12491(c)(4)"));
        assert!(joined.contains("§ 12491(e)"));
        assert!(joined.contains("HUD Form 5380"));
        assert!(joined.contains("HUD Form 5382"));
        assert!(joined.contains("Affiliated Individual"));
        assert!(joined.contains("DOJ Civil Rights Division"));
    }

    #[test]
    fn constant_pin_dates_form_numbers_and_program_count() {
        assert_eq!(VAWA_2022_PL_NUMBER, 117_103);
        assert_eq!(VAWA_2022_SIGNED_DATE_YEAR, 2022);
        assert_eq!(VAWA_2022_SIGNED_DATE_MONTH, 3);
        assert_eq!(VAWA_2022_SIGNED_DATE_DAY, 15);
        assert_eq!(VAWA_2022_HOUSING_EFFECTIVE_DATE_YEAR, 2022);
        assert_eq!(VAWA_2022_HOUSING_EFFECTIVE_DATE_MONTH, 10);
        assert_eq!(VAWA_2022_HOUSING_EFFECTIVE_DATE_DAY, 1);
        assert_eq!(VAWA_2022_HUD_FR_NOTICE_YEAR, 2023);
        assert_eq!(VAWA_2022_HUD_FR_NOTICE_MONTH, 1);
        assert_eq!(VAWA_2022_HUD_FR_NOTICE_DAY, 4);
        assert_eq!(VAWA_2022_HUD_FR_VOLUME, 88);
        assert_eq!(VAWA_2022_HUD_FR_PAGE, 482);
        assert_eq!(VAWA_2022_COVERED_PROGRAMS_COUNT, 16);
        assert_eq!(HUD_FORM_VAWA_NOTICE_NUMBER, 5_380);
        assert_eq!(HUD_FORM_VAWA_CERTIFICATION_NUMBER, 5_382);
    }
}
