//! Multi-State Drone Overflight + Aerial Imagery + Surveillance
//! Privacy Compliance Module.
//!
//! Pure-compute check for whether a landlord (or property manager,
//! commercial inspector, security service) has satisfied state-
//! specific drone overflight + image-capture privacy statutes
//! when operating a drone over residential real property. Trader-
//! landlord critical because drones are increasingly used for
//! facade / roof inspection, parking-lot security, tenant
//! surveillance, and aerial marketing photography — but state
//! statutes impose **per-image civil penalties (up to $50,000 in
//! California; up to $10,000 per disclosed image in Texas)** and
//! treble damages for violations.
//!
//! Web research (verified 2026-06-03):
//! - **California Civ. Code § 1708.8** (rewritten by AB 856 of
//!   2015; "anti-paparazzi drone law"): a person is liable for
//!   PHYSICAL INVASION OF PRIVACY when the person knowingly enters
//!   onto the land or into the airspace above the land of another
//!   person WITHOUT PERMISSION to capture any visual image, sound
//!   recording, or other physical impression of the plaintiff
//!   engaging in a PRIVATE, PERSONAL, OR FAMILIAL ACTIVITY and the
//!   invasion occurs in a manner offensive to a reasonable person.
//!   **Statutory civil fine $5,000 - $50,000 per violation** +
//!   actual damages + treble damages + disgorgement of sale
//!   proceeds + attorney fees. ([CA Legislative Information
//!   AB 856 Bill Text](https://leginfo.legislature.ca.gov/faces/billNavClient.xhtml?bill_id=201520160AB856);
//!   Coblentz Law — Paparazzi Lose Hobbyists Win on Drones; ABJ
//!   Drone Academy California Drone Laws.)
//! - **Texas Government Code Chapter 423** (Privacy of Captured
//!   Images Act, 2013; Texas Privacy Act): offense to use drone
//!   to capture image of individual OR privately owned real
//!   property WITH INTENT to conduct surveillance. Consent of
//!   property owner = full defense. Civil remedies: injunction +
//!   **$5,000 per single-episode capture** OR **$10,000 if images
//!   disclosed/displayed/distributed/used** + actual damages +
//!   court costs + reasonable attorney fees. Illegally obtained
//!   images NOT ADMISSIBLE as evidence in criminal/civil/
//!   administrative proceedings. 5th Circuit upheld
//!   constitutionality in 2023. ([Texas State Law Library —
//!   Drones Recording Laws](https://guides.sll.texas.gov/recording-laws/drones);
//!   Lloyd Gosselink Regulations on Images Captured by Drones;
//!   Texas Agriculture Law upheld 5th Cir.)
//! - **Florida Stat. § 934.50** (Freedom from Unwarranted
//!   Surveillance Act, amended 2015): person/state agency/
//!   political subdivision may NOT use drone with imaging device
//!   to record PRIVATELY OWNED REAL PROPERTY OR the **owner /
//!   tenant / occupant / invitee / licensee** of such property
//!   WITH INTENT to conduct surveillance in violation of person's
//!   REASONABLE EXPECTATION OF PRIVACY without WRITTEN CONSENT.
//!   Person presumed to have reasonable expectation of privacy on
//!   privately owned real property if not observable by persons
//!   located at ground level in a place where they have legal
//!   right to be — regardless of aerial drone visibility. Civil
//!   remedies: compensatory damages + injunctive relief.
//!   ([FindLaw Florida Stat § 934.50](https://codes.findlaw.com/fl/title-xlvii-criminal-procedure-and-corrections/fl-st-sect-934-50/);
//!   Florida Senate Statutes 2024 ch. 934.50.)
//! - **Federal FAA Part 107**: commercial drone use requires
//!   Remote Pilot Certificate; daylight operation; visual line-of-
//!   sight; maximum altitude **400 feet AGL** above ground level.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const CA_1708_8_MIN_FINE_DOLLARS: u64 = 5_000;
pub const CA_1708_8_MAX_FINE_DOLLARS: u64 = 50_000;
pub const CA_1708_8_TREBLE_MULTIPLIER: u64 = 3;
pub const TX_423_SINGLE_EPISODE_FINE_DOLLARS: u64 = 5_000;
pub const TX_423_DISCLOSED_DISPLAYED_FINE_DOLLARS: u64 = 10_000;
pub const TX_423_ENACTED_YEAR: u32 = 2013;
pub const FL_934_50_ENACTED_YEAR: u32 = 2013;
pub const FL_934_50_AMENDED_YEAR: u32 = 2015;
pub const FAA_PART_107_MAX_ALTITUDE_FEET_AGL: u32 = 400;
pub const AB_856_ENACTED_YEAR: u32 = 2015;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DroneJurisdiction {
    California1708_8,
    TexasChapter423PrivacyAct,
    Florida934_50FreedomFromUnwarrantedSurveillance,
    OtherStateCommonLawIntrusionUponSeclusion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FlightPurpose {
    CommercialPropertyInspectionFacadeOrRoof,
    TenantSurveillanceForLeaseEnforcement,
    ParkingLotSecurity,
    AerialMarketingPhotography,
    HobbyistRecreational,
    NewsGathering,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsentStatus {
    WrittenConsentFromOwnerAndTenant,
    WrittenConsentFromOwnerOnly,
    WrittenConsentFromTenantOnly,
    NoConsentObtained,
    OralConsentOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DroneOverflightMode {
    NotApplicableNoDroneFlightOrImageCaptured,
    NotApplicableConsentObtainedFromAllPropertyOwnersAndOccupants,
    NotApplicableHobbyistRecreationalFlightExempt,
    CompliantCommercialFlightFAAPart107CertifiedAndConsented,
    CompliantWrittenConsentObtainedPerStateLaw,
    ViolationCaliforniaCiv1708_8PhysicalInvasionOfPrivacyDrone,
    ViolationTexasChapter423CapturedImageWithoutConsent,
    ViolationFlorida934_50ImagingDeviceWithoutWrittenConsent,
    ViolationFAAPart107NoRemotePilotCertificate,
    ViolationCommercialFlightOutsideDaylightOrVisualLineOfSight,
    ViolationDroneFlightAboveLegalAltitude400FtAgl,
    ViolationTenantSurveillanceWithoutLeaseAuthorityOrConsent,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub jurisdiction: DroneJurisdiction,
    pub flight_purpose: FlightPurpose,
    pub drone_flight_performed: bool,
    pub image_captured_of_private_property_or_individual: bool,
    pub consent_status: ConsentStatus,
    pub flight_altitude_feet_agl: u32,
    pub faa_part_107_certified_pilot: bool,
    pub daylight_visual_line_of_sight_compliant: bool,
    pub intent_to_conduct_surveillance: bool,
    pub tenant_observable_from_ground_level_legal_position: bool,
    pub commercial_flight: bool,
    pub images_disclosed_displayed_or_distributed: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: DroneOverflightMode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type RentalDroneOverflightSurveillancePrivacyInput = Input;
pub type RentalDroneOverflightSurveillancePrivacyOutput = Output;
pub type RentalDroneOverflightSurveillancePrivacyResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "Cal. Civ. Code § 1708.8 (rewritten by AB 856 of 2015) — physical invasion of privacy: airspace entry without permission to capture image of plaintiff engaging in private/personal/familial activity offensive to reasonable person; $5,000-$50,000 statutory fine + treble damages + disgorgement + attorney fees".to_string(),
        "Tex. Gov. Code Chapter 423 (Privacy of Captured Images Act of 2013) — offense to use drone to capture image of individual or privately owned real property with INTENT to conduct surveillance; consent = full defense; $5,000/single-episode + $10,000/disclosed; 5th Cir. upheld 2023".to_string(),
        "Fla. Stat. § 934.50 (Freedom from Unwarranted Surveillance Act; 2013 enactment + 2015 amendment) — drone with imaging device may NOT record privately owned real property or owner/tenant/occupant/invitee/licensee with INTENT to surveil in violation of reasonable expectation of privacy without WRITTEN CONSENT".to_string(),
        "Fla. Stat. § 934.50(3)(b) — person presumed reasonable expectation of privacy on privately owned real property when not observable by persons at ground level in legal location, REGARDLESS of aerial drone visibility".to_string(),
        "Federal FAA Part 107 — commercial drone use: Remote Pilot Certificate required; daylight + visual line-of-sight; max altitude 400 feet AGL".to_string(),
        "Cal. Civ. Code § 1708.8 — illegally captured images may not be sold, transferred, or licensed; civil disgorgement of any proceeds from sale".to_string(),
        "Tex. Gov. Code Ch. 423 — illegally obtained images NOT ADMISSIBLE as evidence in criminal/civil/administrative proceedings".to_string(),
        "Common law alternative — intrusion-upon-seclusion tort (Restatement (Second) of Torts § 652B) applies in jurisdictions without specific drone statute".to_string(),
    ];

    if !input.drone_flight_performed || !input.image_captured_of_private_property_or_individual {
        return Output {
            mode: DroneOverflightMode::NotApplicableNoDroneFlightOrImageCaptured,
            statutory_basis: "No drone flight or no image captured".to_string(),
            notes: "No drone flight performed or no image captured of private property/individual; drone surveillance statutes not invoked.".to_string(),
            citations,
        };
    }

    if input.flight_purpose == FlightPurpose::HobbyistRecreational
        && !input.intent_to_conduct_surveillance
    {
        return Output {
            mode: DroneOverflightMode::NotApplicableHobbyistRecreationalFlightExempt,
            statutory_basis: "Hobbyist recreational flight without surveillance intent".to_string(),
            notes: "Hobbyist recreational flight without intent to conduct surveillance; FAA Part 107 exemption available; state surveillance statutes generally not triggered absent intent.".to_string(),
            citations,
        };
    }

    if input.consent_status == ConsentStatus::WrittenConsentFromOwnerAndTenant {
        return Output {
            mode: DroneOverflightMode::NotApplicableConsentObtainedFromAllPropertyOwnersAndOccupants,
            statutory_basis: "Written consent from both owner and tenant satisfies all state drone privacy statutes".to_string(),
            notes: "Written consent obtained from both property owner AND tenant satisfies CA § 1708.8 + TX Chapter 423 + FL § 934.50 consent requirements.".to_string(),
            citations,
        };
    }

    if input.commercial_flight && !input.faa_part_107_certified_pilot {
        return Output {
            mode: DroneOverflightMode::ViolationFAAPart107NoRemotePilotCertificate,
            statutory_basis: "FAA Part 107 — commercial drone operation requires Remote Pilot Certificate".to_string(),
            notes: "VIOLATION FAA Part 107: commercial drone flight requires Remote Pilot Certificate; pilot not certified.".to_string(),
            citations,
        };
    }

    if input.commercial_flight && !input.daylight_visual_line_of_sight_compliant {
        return Output {
            mode: DroneOverflightMode::ViolationCommercialFlightOutsideDaylightOrVisualLineOfSight,
            statutory_basis: "FAA Part 107 — daylight + visual line-of-sight required".to_string(),
            notes: "VIOLATION FAA Part 107: commercial flight outside daylight hours or beyond visual line-of-sight without waiver.".to_string(),
            citations,
        };
    }

    if input.flight_altitude_feet_agl > FAA_PART_107_MAX_ALTITUDE_FEET_AGL {
        return Output {
            mode: DroneOverflightMode::ViolationDroneFlightAboveLegalAltitude400FtAgl,
            statutory_basis: "FAA Part 107 — max altitude 400 feet AGL".to_string(),
            notes: format!(
                "VIOLATION FAA Part 107: flight altitude {} feet AGL exceeds 400-foot maximum.",
                input.flight_altitude_feet_agl
            ),
            citations,
        };
    }

    if input.flight_purpose == FlightPurpose::TenantSurveillanceForLeaseEnforcement
        && !matches!(
            input.consent_status,
            ConsentStatus::WrittenConsentFromOwnerAndTenant
                | ConsentStatus::WrittenConsentFromTenantOnly
        )
    {
        return Output {
            mode: DroneOverflightMode::ViolationTenantSurveillanceWithoutLeaseAuthorityOrConsent,
            statutory_basis: "Tenant surveillance requires lease authorization OR tenant written consent".to_string(),
            notes: format!(
                "VIOLATION: drone tenant surveillance for lease enforcement requires lease authorization OR written tenant consent. Consent status = {:?}.",
                input.consent_status
            ),
            citations,
        };
    }

    match input.jurisdiction {
        DroneJurisdiction::California1708_8 => {
            if input.intent_to_conduct_surveillance
                && input.consent_status == ConsentStatus::NoConsentObtained
            {
                return Output {
                    mode: DroneOverflightMode::ViolationCaliforniaCiv1708_8PhysicalInvasionOfPrivacyDrone,
                    statutory_basis: "Cal. Civ. Code § 1708.8 — airspace entry without permission to capture image".to_string(),
                    notes: "VIOLATION Cal. Civ. Code § 1708.8: drone entered airspace above private property without permission to capture image; $5,000-$50,000 statutory fine + treble damages.".to_string(),
                    citations,
                };
            }
        }
        DroneJurisdiction::TexasChapter423PrivacyAct => {
            if input.intent_to_conduct_surveillance
                && input.consent_status == ConsentStatus::NoConsentObtained
            {
                return Output {
                    mode: DroneOverflightMode::ViolationTexasChapter423CapturedImageWithoutConsent,
                    statutory_basis: "Tex. Gov. Code Chapter 423 — captured image with intent to surveil without consent".to_string(),
                    notes: format!(
                        "VIOLATION Tex. Gov. Code Chapter 423: drone captured image of private property with intent to surveil without consent; $5,000/episode + $10,000 if disclosed. Images disclosed = {}.",
                        input.images_disclosed_displayed_or_distributed
                    ),
                    citations,
                };
            }
        }
        DroneJurisdiction::Florida934_50FreedomFromUnwarrantedSurveillance => {
            if input.intent_to_conduct_surveillance
                && !input.tenant_observable_from_ground_level_legal_position
                && !matches!(
                    input.consent_status,
                    ConsentStatus::WrittenConsentFromOwnerAndTenant
                        | ConsentStatus::WrittenConsentFromOwnerOnly
                        | ConsentStatus::WrittenConsentFromTenantOnly
                )
            {
                return Output {
                    mode: DroneOverflightMode::ViolationFlorida934_50ImagingDeviceWithoutWrittenConsent,
                    statutory_basis: "Fla. Stat. § 934.50 — written consent required when reasonable expectation of privacy".to_string(),
                    notes: "VIOLATION Fla. Stat. § 934.50: tenant not observable from ground level in legal location; reasonable expectation of privacy presumed; written consent required but not obtained.".to_string(),
                    citations,
                };
            }
        }
        DroneJurisdiction::OtherStateCommonLawIntrusionUponSeclusion => {}
    }

    if matches!(
        input.consent_status,
        ConsentStatus::WrittenConsentFromOwnerOnly | ConsentStatus::WrittenConsentFromTenantOnly
    ) {
        return Output {
            mode: DroneOverflightMode::CompliantWrittenConsentObtainedPerStateLaw,
            statutory_basis: "Written consent satisfies state drone privacy statute".to_string(),
            notes: format!(
                "COMPLIANT: written consent obtained ({:?}); jurisdiction {:?} statute satisfied.",
                input.consent_status, input.jurisdiction
            ),
            citations,
        };
    }

    Output {
        mode: DroneOverflightMode::CompliantCommercialFlightFAAPart107CertifiedAndConsented,
        statutory_basis: "FAA Part 107 + state drone privacy statute satisfied".to_string(),
        notes: format!(
            "COMPLIANT: FAA Part 107-certified pilot; flight altitude {} ft AGL (≤ 400); daylight + VLOS; jurisdiction {:?} surveillance privacy requirements observed.",
            input.flight_altitude_feet_agl, input.jurisdiction
        ),
        citations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_california_violation() -> Input {
        Input {
            jurisdiction: DroneJurisdiction::California1708_8,
            flight_purpose: FlightPurpose::TenantSurveillanceForLeaseEnforcement,
            drone_flight_performed: true,
            image_captured_of_private_property_or_individual: true,
            consent_status: ConsentStatus::NoConsentObtained,
            flight_altitude_feet_agl: 200,
            faa_part_107_certified_pilot: true,
            daylight_visual_line_of_sight_compliant: true,
            intent_to_conduct_surveillance: true,
            tenant_observable_from_ground_level_legal_position: false,
            commercial_flight: true,
            images_disclosed_displayed_or_distributed: false,
        }
    }

    #[test]
    fn no_drone_flight_not_applicable() {
        let input = Input {
            drone_flight_performed: false,
            ..baseline_california_violation()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            DroneOverflightMode::NotApplicableNoDroneFlightOrImageCaptured
        );
    }

    #[test]
    fn no_image_captured_not_applicable() {
        let input = Input {
            image_captured_of_private_property_or_individual: false,
            ..baseline_california_violation()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            DroneOverflightMode::NotApplicableNoDroneFlightOrImageCaptured
        );
    }

    #[test]
    fn hobbyist_recreational_without_intent_exempt() {
        let input = Input {
            flight_purpose: FlightPurpose::HobbyistRecreational,
            intent_to_conduct_surveillance: false,
            ..baseline_california_violation()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            DroneOverflightMode::NotApplicableHobbyistRecreationalFlightExempt
        );
    }

    #[test]
    fn written_consent_from_owner_and_tenant_compliant() {
        let input = Input {
            consent_status: ConsentStatus::WrittenConsentFromOwnerAndTenant,
            ..baseline_california_violation()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            DroneOverflightMode::NotApplicableConsentObtainedFromAllPropertyOwnersAndOccupants
        );
    }

    #[test]
    fn california_tenant_surveillance_without_consent_two_violations() {
        let result = check(&baseline_california_violation());
        assert!(matches!(
            result.mode,
            DroneOverflightMode::ViolationCaliforniaCiv1708_8PhysicalInvasionOfPrivacyDrone
                | DroneOverflightMode::ViolationTenantSurveillanceWithoutLeaseAuthorityOrConsent
        ));
    }

    #[test]
    fn faa_part_107_no_pilot_certificate_violation() {
        let input = Input {
            faa_part_107_certified_pilot: false,
            flight_purpose: FlightPurpose::CommercialPropertyInspectionFacadeOrRoof,
            consent_status: ConsentStatus::WrittenConsentFromOwnerOnly,
            ..baseline_california_violation()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            DroneOverflightMode::ViolationFAAPart107NoRemotePilotCertificate
        );
    }

    #[test]
    fn faa_daylight_violation() {
        let input = Input {
            daylight_visual_line_of_sight_compliant: false,
            flight_purpose: FlightPurpose::CommercialPropertyInspectionFacadeOrRoof,
            consent_status: ConsentStatus::WrittenConsentFromOwnerOnly,
            ..baseline_california_violation()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            DroneOverflightMode::ViolationCommercialFlightOutsideDaylightOrVisualLineOfSight
        );
    }

    #[test]
    fn altitude_above_400_ft_violation() {
        let input = Input {
            flight_altitude_feet_agl: 401,
            flight_purpose: FlightPurpose::CommercialPropertyInspectionFacadeOrRoof,
            consent_status: ConsentStatus::WrittenConsentFromOwnerOnly,
            ..baseline_california_violation()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            DroneOverflightMode::ViolationDroneFlightAboveLegalAltitude400FtAgl
        );
    }

    #[test]
    fn altitude_at_exactly_400_ft_compliant() {
        let input = Input {
            flight_altitude_feet_agl: 400,
            flight_purpose: FlightPurpose::CommercialPropertyInspectionFacadeOrRoof,
            consent_status: ConsentStatus::WrittenConsentFromOwnerOnly,
            intent_to_conduct_surveillance: false,
            ..baseline_california_violation()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            DroneOverflightMode::CompliantWrittenConsentObtainedPerStateLaw
        );
    }

    #[test]
    fn tenant_surveillance_without_lease_authority_violation() {
        let input = Input {
            flight_purpose: FlightPurpose::TenantSurveillanceForLeaseEnforcement,
            consent_status: ConsentStatus::NoConsentObtained,
            ..baseline_california_violation()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            DroneOverflightMode::ViolationTenantSurveillanceWithoutLeaseAuthorityOrConsent
        );
    }

    #[test]
    fn texas_chapter_423_no_consent_violation() {
        let input = Input {
            jurisdiction: DroneJurisdiction::TexasChapter423PrivacyAct,
            flight_purpose: FlightPurpose::AerialMarketingPhotography,
            consent_status: ConsentStatus::NoConsentObtained,
            intent_to_conduct_surveillance: true,
            ..baseline_california_violation()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            DroneOverflightMode::ViolationTexasChapter423CapturedImageWithoutConsent
        );
    }

    #[test]
    fn texas_chapter_423_with_owner_consent_compliant() {
        let input = Input {
            jurisdiction: DroneJurisdiction::TexasChapter423PrivacyAct,
            flight_purpose: FlightPurpose::AerialMarketingPhotography,
            consent_status: ConsentStatus::WrittenConsentFromOwnerOnly,
            intent_to_conduct_surveillance: false,
            ..baseline_california_violation()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            DroneOverflightMode::CompliantWrittenConsentObtainedPerStateLaw
        );
    }

    #[test]
    fn florida_934_50_no_written_consent_violation() {
        let input = Input {
            jurisdiction: DroneJurisdiction::Florida934_50FreedomFromUnwarrantedSurveillance,
            flight_purpose: FlightPurpose::AerialMarketingPhotography,
            consent_status: ConsentStatus::NoConsentObtained,
            tenant_observable_from_ground_level_legal_position: false,
            intent_to_conduct_surveillance: true,
            ..baseline_california_violation()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            DroneOverflightMode::ViolationFlorida934_50ImagingDeviceWithoutWrittenConsent
        );
    }

    #[test]
    fn florida_oral_consent_does_not_satisfy_written_consent() {
        let input = Input {
            jurisdiction: DroneJurisdiction::Florida934_50FreedomFromUnwarrantedSurveillance,
            flight_purpose: FlightPurpose::AerialMarketingPhotography,
            consent_status: ConsentStatus::OralConsentOnly,
            tenant_observable_from_ground_level_legal_position: false,
            intent_to_conduct_surveillance: true,
            ..baseline_california_violation()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            DroneOverflightMode::ViolationFlorida934_50ImagingDeviceWithoutWrittenConsent
        );
    }

    #[test]
    fn florida_tenant_observable_from_ground_no_violation() {
        let input = Input {
            jurisdiction: DroneJurisdiction::Florida934_50FreedomFromUnwarrantedSurveillance,
            flight_purpose: FlightPurpose::AerialMarketingPhotography,
            consent_status: ConsentStatus::NoConsentObtained,
            tenant_observable_from_ground_level_legal_position: true,
            intent_to_conduct_surveillance: true,
            ..baseline_california_violation()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            DroneOverflightMode::CompliantCommercialFlightFAAPart107CertifiedAndConsented
        );
    }

    #[test]
    fn florida_with_written_consent_owner_compliant() {
        let input = Input {
            jurisdiction: DroneJurisdiction::Florida934_50FreedomFromUnwarrantedSurveillance,
            flight_purpose: FlightPurpose::CommercialPropertyInspectionFacadeOrRoof,
            consent_status: ConsentStatus::WrittenConsentFromOwnerOnly,
            tenant_observable_from_ground_level_legal_position: false,
            intent_to_conduct_surveillance: false,
            ..baseline_california_violation()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            DroneOverflightMode::CompliantWrittenConsentObtainedPerStateLaw
        );
    }

    #[test]
    fn other_state_common_law_intrusion_upon_seclusion_no_specific_statute() {
        let input = Input {
            jurisdiction: DroneJurisdiction::OtherStateCommonLawIntrusionUponSeclusion,
            flight_purpose: FlightPurpose::CommercialPropertyInspectionFacadeOrRoof,
            consent_status: ConsentStatus::WrittenConsentFromOwnerOnly,
            intent_to_conduct_surveillance: false,
            ..baseline_california_violation()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            DroneOverflightMode::CompliantWrittenConsentObtainedPerStateLaw
        );
    }

    #[test]
    fn citations_pin_statutes_and_faa_part_107() {
        let result = check(&baseline_california_violation());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("Cal. Civ. Code § 1708.8"));
        assert!(joined.contains("AB 856 of 2015"));
        assert!(joined.contains("$5,000-$50,000"));
        assert!(joined.contains("Tex. Gov. Code Chapter 423"));
        assert!(joined.contains("Privacy of Captured Images Act of 2013"));
        assert!(joined.contains("5th Cir."));
        assert!(joined.contains("Fla. Stat. § 934.50"));
        assert!(joined.contains("Freedom from Unwarranted Surveillance Act"));
        assert!(joined.contains("WRITTEN CONSENT"));
        assert!(joined.contains("FAA Part 107"));
        assert!(joined.contains("400 feet AGL"));
        assert!(joined.contains("Restatement (Second) of Torts § 652B"));
    }

    #[test]
    fn constant_pin_fines_and_thresholds() {
        assert_eq!(CA_1708_8_MIN_FINE_DOLLARS, 5_000);
        assert_eq!(CA_1708_8_MAX_FINE_DOLLARS, 50_000);
        assert_eq!(CA_1708_8_TREBLE_MULTIPLIER, 3);
        assert_eq!(TX_423_SINGLE_EPISODE_FINE_DOLLARS, 5_000);
        assert_eq!(TX_423_DISCLOSED_DISPLAYED_FINE_DOLLARS, 10_000);
        assert_eq!(TX_423_ENACTED_YEAR, 2013);
        assert_eq!(FL_934_50_ENACTED_YEAR, 2013);
        assert_eq!(FL_934_50_AMENDED_YEAR, 2015);
        assert_eq!(FAA_PART_107_MAX_ALTITUDE_FEET_AGL, 400);
        assert_eq!(AB_856_ENACTED_YEAR, 2015);
    }
}
