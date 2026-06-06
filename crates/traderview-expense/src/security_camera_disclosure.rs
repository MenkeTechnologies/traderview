//! State security-camera and surveillance landlord disclosure /
//! consent compliance check.
//!
//! Landlord installation of security cameras in residential rental
//! property is regulated by two overlapping bodies of law: (1) state
//! WIRETAP / EAVESDROPPING statutes governing AUDIO recording (1-party
//! vs 2-party consent), and (2) state PRIVACY statutes prohibiting
//! VIDEO surveillance of private living spaces. The two regimes
//! distinguish along the audio-consent axis: California + Illinois +
//! Washington + Massachusetts require ALL-PARTY (2-party) consent;
//! Texas + Federal Wiretap Act require only 1-party consent. Video
//! surveillance is universally barred from inside the unit but
//! permitted in common areas subject to reasonable-expectation-of-
//! privacy limits.
//!
//! California (Penal Code § 632) — 2-PARTY consent for audio recording
//! of confidential communications. Video allowed in common areas
//! (lobby, mailroom, laundry, entry/exit) but NEVER inside the tenant's
//! private unit. $2,500 per-violation civil penalty; criminal exposure
//! up to 1 year county jail or state prison. "Confidential
//! communication" includes any conversation where parties reasonably
//! expect privacy — small enclosed common spaces (laundry rooms, narrow
//! hallways) may qualify.
//!
//! New York (Civil Rights Law § 52-a + Penal Law § 250.00) — 1-PARTY
//! consent for audio (NY follows federal Wiretap Act standard). Video
//! allowed in lobbies, elevator cabs, building entries/exits, mailrooms,
//! parking garages, gyms, laundry rooms, rooftop areas. Hallway cameras
//! permitted when positioned to monitor corridors without capturing
//! apartment interiors when doors open. Civil Rights Law § 52-a creates
//! private right of action for backyard recreational video without
//! written consent.
//!
//! Texas (Penal Code § 16.02) — 1-PARTY consent for audio. Most
//! landlord-friendly regime. Video subject to general reasonable-
//! expectation-of-privacy standard.
//!
//! Default — federal Wiretap Act (1-party consent) + state privacy
//! laws vary; video in tenant's private unit always barred under
//! reasonable-expectation-of-privacy doctrine.
//!
//! Citations: Cal. Penal Code § 632 (audio recording, 2-party consent,
//! $2,500 per-violation); NY Civil Rights Law § 52-a (residential
//! video privacy + backyard); NY Penal Law § 250.00 (1-party audio
//! consent); Tex. Penal Code § 16.02 (wiretap, 1-party consent); 18
//! U.S.C. § 2511 (federal Wiretap Act, 1-party consent baseline).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    California,
    NewYork,
    Texas,
    Default,
}

impl Regime {
    pub fn for_state(state: &str) -> Self {
        match state.trim().to_ascii_uppercase().as_str() {
            "CA" => Self::California,
            "NY" => Self::NewYork,
            "TX" => Self::Texas,
            _ => Self::Default,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordingType {
    /// Video-only surveillance in common area (lobby, laundry, mailroom).
    VideoInCommonArea,
    /// Video AND audio in common area.
    AudioVideoInCommonArea,
    /// Audio-only recording in common area.
    AudioInCommonArea,
    /// Any recording (video or audio) inside the tenant's private unit.
    /// Universally barred under reasonable-expectation-of-privacy
    /// doctrine.
    InsideTenantPrivateUnit,
    /// Video recording of the tenant's backyard. NY Civil Rights Law §
    /// 52-a creates private right of action for backyard recreational
    /// video without written consent.
    BackyardVideo,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SecurityCameraInput {
    pub regime: Regime,
    pub recording_type: RecordingType,
    /// Whether ALL parties to the recorded communication / scene have
    /// given consent. For 2-party-consent states this is the relevant
    /// gate. For 1-party-consent states, landlord can record audio
    /// where landlord is a party.
    pub all_party_consent_obtained: bool,
    /// Whether AT LEAST ONE party (typically the landlord) has consented.
    /// Relevant for 1-party-consent states.
    pub one_party_consent_obtained: bool,
    /// Whether the location involves a reasonable expectation of privacy
    /// (small enclosed common spaces like laundry rooms, narrow halls).
    pub reasonable_expectation_of_privacy: bool,
    /// Whether the lease discloses the surveillance setup.
    pub written_lease_disclosure: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsentStandard {
    OneParty,
    TwoParty,
    /// Prohibited regardless of consent (e.g., inside private unit).
    ProhibitedRegardless,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ViolationType {
    None,
    AudioWithoutRequiredConsent,
    VideoInsidePrivateUnit,
    BackyardVideoWithoutWrittenConsent,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SecurityCameraResult {
    pub regime: Regime,
    pub audio_consent_standard: ConsentStandard,
    pub recording_permitted: bool,
    pub per_violation_civil_penalty_cents: i64,
    pub criminal_exposure: bool,
    pub violation: ViolationType,
    pub landlord_compliant: bool,
    pub citation: &'static str,
    pub note: String,
}

pub fn check(input: &SecurityCameraInput) -> SecurityCameraResult {
    let (audio_standard, audio_civil_penalty, criminal): (ConsentStandard, i64, bool) =
        match input.regime {
            Regime::California => (ConsentStandard::TwoParty, 250000, true),
            Regime::NewYork => (ConsentStandard::OneParty, 0, false),
            Regime::Texas => (ConsentStandard::OneParty, 0, false),
            Regime::Default => (ConsentStandard::OneParty, 0, false),
        };

    // Universal bar: recording inside the tenant's private unit is never
    // permitted regardless of consent.
    if input.recording_type == RecordingType::InsideTenantPrivateUnit {
        return SecurityCameraResult {
            regime: input.regime,
            audio_consent_standard: ConsentStandard::ProhibitedRegardless,
            recording_permitted: false,
            per_violation_civil_penalty_cents: audio_civil_penalty,
            criminal_exposure: criminal,
            violation: ViolationType::VideoInsidePrivateUnit,
            landlord_compliant: false,
            citation: "Reasonable-expectation-of-privacy doctrine — recording inside tenant's private living unit is universally barred regardless of consent",
            note: "Recording (video or audio) inside the tenant's private living unit is barred under universal reasonable-expectation-of-privacy doctrine. Consent does not cure.".to_string(),
        };
    }

    // NY-specific backyard video.
    if input.recording_type == RecordingType::BackyardVideo
        && input.regime == Regime::NewYork
        && !input.written_lease_disclosure
    {
        return SecurityCameraResult {
            regime: Regime::NewYork,
            audio_consent_standard: audio_standard,
            recording_permitted: false,
            per_violation_civil_penalty_cents: 0,
            criminal_exposure: false,
            violation: ViolationType::BackyardVideoWithoutWrittenConsent,
            landlord_compliant: false,
            citation: "NY Civil Rights Law § 52-a — private right of action for video taping recreational activities in backyard of residential real property without written consent of owner and/or tenant",
            note: "Backyard video without written consent of owner/tenant violates NY Civil Rights Law § 52-a — tenant has private right of action for damages.".to_string(),
        };
    }

    // Audio recording: check consent standard.
    let involves_audio = matches!(
        input.recording_type,
        RecordingType::AudioInCommonArea | RecordingType::AudioVideoInCommonArea
    );
    if involves_audio {
        let consent_satisfied = match audio_standard {
            ConsentStandard::TwoParty => input.all_party_consent_obtained,
            ConsentStandard::OneParty => input.one_party_consent_obtained,
            ConsentStandard::ProhibitedRegardless => false,
        };
        // For 2-party states, the audio rule applies most strongly when
        // there's a reasonable expectation of privacy (CA Penal Code §
        // 632 "confidential communications" requirement).
        let actually_requires_consent =
            input.regime != Regime::California || input.reasonable_expectation_of_privacy;
        if actually_requires_consent && !consent_satisfied {
            return SecurityCameraResult {
                regime: input.regime,
                audio_consent_standard: audio_standard,
                recording_permitted: false,
                per_violation_civil_penalty_cents: audio_civil_penalty,
                criminal_exposure: criminal,
                violation: ViolationType::AudioWithoutRequiredConsent,
                landlord_compliant: false,
                citation: match input.regime {
                    Regime::California => {
                        "Cal. Penal Code § 632 — audio recording of confidential communications requires ALL-PARTY consent; $2,500 per-violation civil penalty + criminal exposure"
                    }
                    Regime::NewYork => {
                        "NY Penal Law § 250.00 — audio recording requires at least ONE-PARTY consent"
                    }
                    Regime::Texas => "Tex. Penal Code § 16.02 — audio recording requires at least ONE-PARTY consent",
                    Regime::Default => {
                        "18 U.S.C. § 2511 (federal Wiretap Act) — audio recording requires at least ONE-PARTY consent baseline"
                    }
                },
                note: format!(
                    "Audio recording in {:?} requires {} consent which was not obtained.",
                    input.recording_type,
                    match audio_standard {
                        ConsentStandard::TwoParty => "ALL-PARTY",
                        ConsentStandard::OneParty => "ONE-PARTY",
                        ConsentStandard::ProhibitedRegardless => "no",
                    }
                ),
            };
        }
    }

    // Compliant.
    SecurityCameraResult {
        regime: input.regime,
        audio_consent_standard: audio_standard,
        recording_permitted: true,
        per_violation_civil_penalty_cents: audio_civil_penalty,
        criminal_exposure: criminal,
        violation: ViolationType::None,
        landlord_compliant: true,
        citation: match input.regime {
            Regime::California => "Cal. Penal Code § 632 — video in common areas permitted; audio requires ALL-PARTY consent when confidential",
            Regime::NewYork => "NY Civil Rights Law § 52-a + NY Penal Law § 250.00 — video in common areas permitted; audio requires ONE-PARTY consent",
            Regime::Texas => "Tex. Penal Code § 16.02 — landlord-friendly: video in common areas permitted; audio requires ONE-PARTY consent",
            Regime::Default => "18 U.S.C. § 2511 (federal baseline) — video in common areas permitted; audio requires ONE-PARTY consent",
        },
        note: format!(
            "Recording {:?} complies with applicable surveillance rules. {} consent satisfied.",
            input.recording_type,
            match audio_standard {
                ConsentStandard::TwoParty => "ALL-PARTY",
                ConsentStandard::OneParty => "ONE-PARTY",
                ConsentStandard::ProhibitedRegardless => "no",
            }
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        regime: Regime,
        recording: RecordingType,
        all_party: bool,
        one_party: bool,
        reasonable_privacy: bool,
        written_disclosure: bool,
    ) -> SecurityCameraInput {
        SecurityCameraInput {
            regime,
            recording_type: recording,
            all_party_consent_obtained: all_party,
            one_party_consent_obtained: one_party,
            reasonable_expectation_of_privacy: reasonable_privacy,
            written_lease_disclosure: written_disclosure,
        }
    }

    #[test]
    fn ca_video_only_common_area_permitted() {
        let r = check(&input(
            Regime::California,
            RecordingType::VideoInCommonArea,
            false,
            false,
            false,
            false,
        ));
        assert!(r.recording_permitted);
        assert_eq!(r.violation, ViolationType::None);
        assert_eq!(r.audio_consent_standard, ConsentStandard::TwoParty);
    }

    #[test]
    fn ca_audio_in_common_area_with_reasonable_privacy_requires_2party() {
        let r = check(&input(
            Regime::California,
            RecordingType::AudioInCommonArea,
            false,
            true,
            true,
            false,
        ));
        // 1-party consent insufficient for CA when reasonable privacy.
        assert_eq!(r.violation, ViolationType::AudioWithoutRequiredConsent);
        assert!(r.citation.contains("§ 632"));
        assert!(r.citation.contains("$2,500"));
        assert_eq!(r.per_violation_civil_penalty_cents, 2_500_00);
        assert!(r.criminal_exposure);
    }

    #[test]
    fn ca_audio_no_reasonable_privacy_no_violation() {
        // CA § 632 applies only to "confidential communications" — no
        // reasonable expectation = no § 632 violation.
        let r = check(&input(
            Regime::California,
            RecordingType::AudioInCommonArea,
            false,
            true,
            false,
            false,
        ));
        assert_eq!(r.violation, ViolationType::None);
    }

    #[test]
    fn ca_audio_with_all_party_consent_compliant() {
        let r = check(&input(
            Regime::California,
            RecordingType::AudioInCommonArea,
            true,
            true,
            true,
            false,
        ));
        assert!(r.recording_permitted);
    }

    #[test]
    fn ny_audio_with_one_party_consent_sufficient() {
        let r = check(&input(
            Regime::NewYork,
            RecordingType::AudioInCommonArea,
            false,
            true,
            true,
            false,
        ));
        assert!(r.recording_permitted);
        assert_eq!(r.audio_consent_standard, ConsentStandard::OneParty);
    }

    #[test]
    fn ny_audio_without_any_consent_violation() {
        let r = check(&input(
            Regime::NewYork,
            RecordingType::AudioInCommonArea,
            false,
            false,
            true,
            false,
        ));
        assert_eq!(r.violation, ViolationType::AudioWithoutRequiredConsent);
        assert!(r.citation.contains("§ 250.00"));
    }

    #[test]
    fn ny_backyard_video_without_consent_violation() {
        let r = check(&input(
            Regime::NewYork,
            RecordingType::BackyardVideo,
            false,
            false,
            true,
            false,
        ));
        assert_eq!(
            r.violation,
            ViolationType::BackyardVideoWithoutWrittenConsent
        );
        assert!(r.citation.contains("§ 52-a"));
    }

    #[test]
    fn ny_backyard_video_with_consent_permitted() {
        let r = check(&input(
            Regime::NewYork,
            RecordingType::BackyardVideo,
            false,
            false,
            true,
            true,
        ));
        assert!(r.recording_permitted);
    }

    #[test]
    fn tx_one_party_consent_sufficient() {
        let r = check(&input(
            Regime::Texas,
            RecordingType::AudioInCommonArea,
            false,
            true,
            true,
            false,
        ));
        assert!(r.recording_permitted);
        assert!(r.citation.contains("§ 16.02"));
    }

    #[test]
    fn tx_audio_without_any_consent_violation() {
        let r = check(&input(
            Regime::Texas,
            RecordingType::AudioInCommonArea,
            false,
            false,
            true,
            false,
        ));
        assert_eq!(r.violation, ViolationType::AudioWithoutRequiredConsent);
    }

    #[test]
    fn default_one_party_baseline() {
        let r = check(&input(
            Regime::Default,
            RecordingType::AudioInCommonArea,
            false,
            true,
            true,
            false,
        ));
        assert!(r.recording_permitted);
        assert!(r.citation.contains("§ 2511"));
        assert!(r.citation.contains("federal"));
    }

    #[test]
    fn video_inside_private_unit_universally_barred_ca() {
        let r = check(&input(
            Regime::California,
            RecordingType::InsideTenantPrivateUnit,
            true,
            true,
            true,
            true,
        ));
        assert!(!r.recording_permitted);
        assert_eq!(r.violation, ViolationType::VideoInsidePrivateUnit);
        assert_eq!(
            r.audio_consent_standard,
            ConsentStandard::ProhibitedRegardless
        );
    }

    #[test]
    fn video_inside_private_unit_universally_barred_default() {
        let r = check(&input(
            Regime::Default,
            RecordingType::InsideTenantPrivateUnit,
            true,
            true,
            true,
            true,
        ));
        assert!(!r.recording_permitted);
    }

    #[test]
    fn ca_audio_video_in_common_area_with_2_party_consent_compliant() {
        let r = check(&input(
            Regime::California,
            RecordingType::AudioVideoInCommonArea,
            true,
            true,
            true,
            false,
        ));
        assert!(r.recording_permitted);
    }

    #[test]
    fn state_routing_ca_ny_tx_default() {
        assert_eq!(Regime::for_state("CA"), Regime::California);
        assert_eq!(Regime::for_state("NY"), Regime::NewYork);
        assert_eq!(Regime::for_state("TX"), Regime::Texas);
        assert_eq!(Regime::for_state("WA"), Regime::Default);
        assert_eq!(Regime::for_state("IL"), Regime::Default);
    }

    #[test]
    fn state_routing_case_insensitive() {
        assert_eq!(Regime::for_state("ca"), Regime::California);
        assert_eq!(Regime::for_state("Ny"), Regime::NewYork);
    }

    #[test]
    fn only_ca_has_two_party_audio_consent() {
        // Same audio + 1-party-consent scenario across regimes.
        let ca = check(&input(
            Regime::California,
            RecordingType::AudioInCommonArea,
            false,
            true,
            true,
            false,
        ));
        let ny = check(&input(
            Regime::NewYork,
            RecordingType::AudioInCommonArea,
            false,
            true,
            true,
            false,
        ));
        let tx = check(&input(
            Regime::Texas,
            RecordingType::AudioInCommonArea,
            false,
            true,
            true,
            false,
        ));
        assert_eq!(ca.violation, ViolationType::AudioWithoutRequiredConsent);
        assert_eq!(ny.violation, ViolationType::None);
        assert_eq!(tx.violation, ViolationType::None);
    }

    #[test]
    fn only_ca_has_2500_per_violation_civil_penalty() {
        let ca = check(&input(
            Regime::California,
            RecordingType::AudioInCommonArea,
            false,
            true,
            true,
            false,
        ));
        let ny = check(&input(
            Regime::NewYork,
            RecordingType::AudioInCommonArea,
            false,
            false,
            true,
            false,
        ));
        let tx = check(&input(
            Regime::Texas,
            RecordingType::AudioInCommonArea,
            false,
            false,
            true,
            false,
        ));
        assert_eq!(ca.per_violation_civil_penalty_cents, 2_500_00);
        assert_eq!(ny.per_violation_civil_penalty_cents, 0);
        assert_eq!(tx.per_violation_civil_penalty_cents, 0);
    }

    #[test]
    fn only_ca_has_criminal_exposure() {
        let ca = check(&input(
            Regime::California,
            RecordingType::AudioInCommonArea,
            false,
            true,
            true,
            false,
        ));
        let ny = check(&input(
            Regime::NewYork,
            RecordingType::AudioInCommonArea,
            false,
            true,
            true,
            false,
        ));
        assert!(ca.criminal_exposure);
        assert!(!ny.criminal_exposure);
    }

    #[test]
    fn only_ny_has_backyard_video_private_right_of_action() {
        // Same backyard-video-without-consent scenario across regimes.
        let ny = check(&input(
            Regime::NewYork,
            RecordingType::BackyardVideo,
            false,
            false,
            true,
            false,
        ));
        let ca = check(&input(
            Regime::California,
            RecordingType::BackyardVideo,
            false,
            false,
            true,
            false,
        ));
        let tx = check(&input(
            Regime::Texas,
            RecordingType::BackyardVideo,
            false,
            false,
            true,
            false,
        ));
        assert_eq!(
            ny.violation,
            ViolationType::BackyardVideoWithoutWrittenConsent
        );
        // Other states don't have the backyard-specific right of action.
        assert_eq!(ca.violation, ViolationType::None);
        assert_eq!(tx.violation, ViolationType::None);
    }

    #[test]
    fn citations_pin_correct_authorities() {
        let ca = check(&input(
            Regime::California,
            RecordingType::AudioInCommonArea,
            false,
            true,
            true,
            false,
        ));
        assert!(ca.citation.contains("§ 632"));

        let ny = check(&input(
            Regime::NewYork,
            RecordingType::AudioInCommonArea,
            false,
            false,
            true,
            false,
        ));
        assert!(ny.citation.contains("§ 250.00"));

        let tx = check(&input(
            Regime::Texas,
            RecordingType::AudioInCommonArea,
            false,
            false,
            true,
            false,
        ));
        assert!(tx.citation.contains("§ 16.02"));

        let backyard = check(&input(
            Regime::NewYork,
            RecordingType::BackyardVideo,
            false,
            false,
            true,
            false,
        ));
        assert!(backyard.citation.contains("§ 52-a"));
    }
}
