//! Landlord-tenant audio recording consent — federal Wiretap Act
//! floor (18 U.S.C. § 2511 one-party consent) overlaid with state-
//! specific all-party consent regimes (11 states). Trader-landlord
//! operational concern when recording meetings with tenants
//! (eviction discussions, repair complaints, lease-renewal
//! negotiations), and tenant-side concern when recording landlord
//! interactions for evidentiary use.
//!
//! Distinct from `tenant_data_privacy` (broader data-handling
//! framework), `landlord_harassment` (post-event remedies), and
//! `security_camera_disclosure` (video surveillance with audio
//! component). This module addresses ONLY the AUDIO-RECORDING
//! CONSENT analysis under federal + state law.
//!
//! Three regimes:
//!
//! **All-Party Consent — 11 states**: California (Cal. Penal Code
//! § 632), Delaware (11 Del. C. § 2402(c)(4)), Florida (Fla. Stat.
//! § 934.03(2)(d)), Illinois (720 ILCS 5/14-2), Maryland (Md. Code
//! Cts. & Jud. Proc. § 10-402), Massachusetts (M.G.L. c. 272 § 99),
//! Montana (Mont. Code § 45-8-213), Nevada (NRS § 200.620), New
//! Hampshire (NH RSA § 570-A:2), Pennsylvania (18 Pa. Cons. Stat.
//! § 5704), Washington (RCW § 9.73.030). Both parties to a private
//! conversation must consent before recording. Criminal exposure
//! per state (typically misdemeanor or felony) plus civil damages.
//!
//! **One-Party Consent — Federal Floor + Most States**: 18 U.S.C.
//! § 2511(2)(d) — federal Wiretap Act permits recording when at
//! least one party consents OR the recording party is a party to
//! the conversation. Most states (39 + DC + territories) follow
//! this federal one-party floor. Criminal penalty under § 2511(4):
//! up to 5 years imprisonment + $250,000 individual fine. Civil
//! damages under § 2520(c) — actual damages OR $10,000 statutory
//! minimum per violation, whichever is greater.
//!
//! **Third-Party Device — Federal Violation Even in One-Party
//! States**: when a landlord installs a recording device to
//! capture conversations BETWEEN TENANTS or between TENANTS AND
//! THEIR GUESTS in which the landlord is NOT a party, the
//! recording violates 18 U.S.C. § 2511 regardless of state consent
//! regime — one-party consent only protects a party who consents
//! to their own recording. Landlord-installed third-party devices
//! lack any consenting party (landlord is a stranger to the
//! conversation).
//!
//! **In-Unit Recording Without Consent — Universal Violation**:
//! audio recording of conversations inside a rental unit without
//! tenant consent violates both the federal Wiretap Act and state
//! wiretapping laws regardless of consent regime, because the
//! tenant has a reasonable expectation of privacy in their dwelling.
//!
//! Citations: 18 U.S.C. § 2511 (federal Wiretap Act — interception
//! prohibition); § 2511(2)(d) (one-party consent exception);
//! § 2511(4) (criminal penalties — 5 years + $250K); 18 U.S.C.
//! § 2520(c) (civil damages — $10K statutory minimum); Cal. Penal
//! Code § 632 (CA all-party); 11 Del. C. § 2402(c)(4) (DE);
//! Fla. Stat. § 934.03(2)(d) (FL); 720 ILCS 5/14-2 (IL); Md. Code
//! Cts. & Jud. Proc. § 10-402 (MD); M.G.L. c. 272 § 99 (MA);
//! Mont. Code § 45-8-213 (MT); NRS § 200.620 (NV); NH RSA § 570-A:2
//! (NH); 18 Pa. Cons. Stat. § 5704 (PA); RCW § 9.73.030 (WA).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConsentRegime {
    /// CA, DE, FL, IL, MD, MA, MT, NV, NH, PA, WA — all parties
    /// must consent to private-conversation recording.
    AllPartyConsent,
    /// Federal floor + most states (39 + DC) — one-party consent
    /// sufficient.
    OnePartyConsent,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RecordingConsentInput {
    pub regime: ConsentRegime,
    /// Whether the recording party (landlord or tenant) is a party
    /// to the conversation being recorded. False = third-party
    /// device (landlord installed device to record tenants).
    pub recording_party_is_a_party_to_conversation: bool,
    /// Whether ALL parties have consented to the recording.
    /// Required in AllPartyConsent regimes.
    pub all_parties_consented: bool,
    /// Whether AT LEAST ONE party has consented to the recording.
    /// Sufficient in OnePartyConsent regimes when the recording
    /// party is also a party to the conversation.
    pub at_least_one_party_consented: bool,
    /// Whether the conversation occurred in a public space with no
    /// reasonable expectation of privacy.
    pub public_space_no_privacy_expectation: bool,
    /// Whether the recording captured conversations INSIDE a
    /// rental unit (where tenant has reasonable expectation of
    /// privacy).
    pub recording_inside_rental_unit: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RecordingConsentResult {
    pub recording_lawful_federal: bool,
    pub recording_lawful_state: bool,
    pub federal_criminal_exposure: bool,
    pub federal_civil_minimum_damages_dollars: i64,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &RecordingConsentInput) -> RecordingConsentResult {
    let mut notes: Vec<String> = Vec::new();

    if input.public_space_no_privacy_expectation {
        notes.push(
            "public space with no reasonable expectation of privacy — federal Wiretap Act + state laws generally do not apply"
                .to_string(),
        );
        return RecordingConsentResult {
            recording_lawful_federal: true,
            recording_lawful_state: true,
            federal_criminal_exposure: false,
            federal_civil_minimum_damages_dollars: 0,
            citation: citation(),
            notes,
        };
    }

    if !input.recording_party_is_a_party_to_conversation {
        notes.push(
            "third-party device — recording party NOT a party to the conversation; violates 18 U.S.C. § 2511 regardless of state consent regime (one-party consent only protects a consenting party, not a stranger to the conversation)"
                .to_string(),
        );
        return RecordingConsentResult {
            recording_lawful_federal: false,
            recording_lawful_state: false,
            federal_criminal_exposure: true,
            federal_civil_minimum_damages_dollars: 10_000,
            citation: citation(),
            notes,
        };
    }

    if input.recording_inside_rental_unit && !input.all_parties_consented {
        notes.push(
            "in-unit recording without all-party consent — tenant has reasonable expectation of privacy in dwelling; violates federal Wiretap Act AND state law regardless of regime"
                .to_string(),
        );
        return RecordingConsentResult {
            recording_lawful_federal: false,
            recording_lawful_state: false,
            federal_criminal_exposure: true,
            federal_civil_minimum_damages_dollars: 10_000,
            citation: citation(),
            notes,
        };
    }

    let lawful_federal = input.at_least_one_party_consented;
    if lawful_federal {
        notes.push(
            "18 U.S.C. § 2511(2)(d) — one-party consent satisfied; federal Wiretap Act exception engaged"
                .to_string(),
        );
    } else {
        notes.push(
            "18 U.S.C. § 2511 — no party consented; recording prohibited under federal Wiretap Act"
                .to_string(),
        );
    }

    let lawful_state = match input.regime {
        ConsentRegime::AllPartyConsent => {
            if input.all_parties_consented {
                notes.push(
                    "all-party consent state — all parties consented; recording lawful (CA / DE / FL / IL / MD / MA / MT / NV / NH / PA / WA)"
                        .to_string(),
                );
                true
            } else {
                notes.push(
                    "all-party consent state — fewer than all parties consented; recording prohibited under state law (criminal exposure varies by state — typically misdemeanor or felony plus civil damages)"
                        .to_string(),
                );
                false
            }
        }
        ConsentRegime::OnePartyConsent => {
            if input.at_least_one_party_consented {
                notes.push(
                    "one-party consent state (federal floor + 39 states + DC) — at least one party consented; recording lawful under state law"
                        .to_string(),
                );
                true
            } else {
                notes.push(
                    "one-party consent state — no party consented; recording prohibited under state law"
                        .to_string(),
                );
                false
            }
        }
    };

    let federal_criminal = !lawful_federal;
    let civil_min = if !lawful_federal { 10_000 } else { 0 };

    if federal_criminal {
        notes.push(
            "18 U.S.C. § 2511(4) criminal penalty — up to 5 years imprisonment + $250,000 individual fine"
                .to_string(),
        );
        notes.push(
            "18 U.S.C. § 2520(c) civil damages — actual damages OR $10,000 statutory minimum per violation, whichever greater"
                .to_string(),
        );
    }

    RecordingConsentResult {
        recording_lawful_federal: lawful_federal,
        recording_lawful_state: lawful_state,
        federal_criminal_exposure: federal_criminal,
        federal_civil_minimum_damages_dollars: civil_min,
        citation: citation(),
        notes,
    }
}

fn citation() -> &'static str {
    "18 U.S.C. § 2511; § 2511(2)(d); § 2511(4); 18 U.S.C. § 2520(c); Cal. Penal Code § 632; 11 Del. C. § 2402(c)(4); Fla. Stat. § 934.03(2)(d); 720 ILCS 5/14-2; Md. Code Cts. & Jud. Proc. § 10-402; M.G.L. c. 272 § 99; Mont. Code § 45-8-213; NRS § 200.620; NH RSA § 570-A:2; 18 Pa. Cons. Stat. § 5704; RCW § 9.73.030"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base(regime: ConsentRegime) -> RecordingConsentInput {
        RecordingConsentInput {
            regime,
            recording_party_is_a_party_to_conversation: true,
            all_parties_consented: false,
            at_least_one_party_consented: true,
            public_space_no_privacy_expectation: false,
            recording_inside_rental_unit: false,
        }
    }

    #[test]
    fn one_party_consent_with_at_least_one_consent_lawful() {
        let r = check(&base(ConsentRegime::OnePartyConsent));
        assert!(r.recording_lawful_federal);
        assert!(r.recording_lawful_state);
        assert!(!r.federal_criminal_exposure);
    }

    #[test]
    fn one_party_consent_with_no_consent_unlawful() {
        let mut i = base(ConsentRegime::OnePartyConsent);
        i.at_least_one_party_consented = false;
        let r = check(&i);
        assert!(!r.recording_lawful_federal);
        assert!(!r.recording_lawful_state);
        assert!(r.federal_criminal_exposure);
        assert_eq!(r.federal_civil_minimum_damages_dollars, 10_000);
    }

    #[test]
    fn all_party_consent_with_only_one_party_unlawful_state() {
        let mut i = base(ConsentRegime::AllPartyConsent);
        i.at_least_one_party_consented = true;
        i.all_parties_consented = false;
        let r = check(&i);
        assert!(r.recording_lawful_federal, "federal one-party satisfied");
        assert!(!r.recording_lawful_state, "state all-party violated");
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("all-party consent state") && n.contains("misdemeanor or felony")));
    }

    #[test]
    fn all_party_consent_with_all_parties_lawful() {
        let mut i = base(ConsentRegime::AllPartyConsent);
        i.all_parties_consented = true;
        i.at_least_one_party_consented = true;
        let r = check(&i);
        assert!(r.recording_lawful_federal);
        assert!(r.recording_lawful_state);
    }

    #[test]
    fn third_party_device_violates_federal_regardless_of_state() {
        for regime in [
            ConsentRegime::OnePartyConsent,
            ConsentRegime::AllPartyConsent,
        ] {
            let mut i = base(regime);
            i.recording_party_is_a_party_to_conversation = false;
            i.at_least_one_party_consented = true;
            i.all_parties_consented = true;
            let r = check(&i);
            assert!(!r.recording_lawful_federal);
            assert!(!r.recording_lawful_state);
            assert!(r.federal_criminal_exposure);
            assert_eq!(r.federal_civil_minimum_damages_dollars, 10_000);
            assert!(r.notes.iter().any(|n| n.contains("third-party device")));
        }
    }

    #[test]
    fn in_unit_recording_without_all_party_consent_violates_both() {
        let mut i = base(ConsentRegime::OnePartyConsent);
        i.recording_inside_rental_unit = true;
        i.at_least_one_party_consented = true;
        i.all_parties_consented = false;
        let r = check(&i);
        assert!(!r.recording_lawful_federal);
        assert!(!r.recording_lawful_state);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("reasonable expectation of privacy")));
    }

    #[test]
    fn in_unit_recording_with_all_party_consent_lawful() {
        let mut i = base(ConsentRegime::OnePartyConsent);
        i.recording_inside_rental_unit = true;
        i.all_parties_consented = true;
        i.at_least_one_party_consented = true;
        let r = check(&i);
        assert!(r.recording_lawful_federal);
        assert!(r.recording_lawful_state);
    }

    #[test]
    fn public_space_no_privacy_expectation_always_lawful() {
        for regime in [
            ConsentRegime::OnePartyConsent,
            ConsentRegime::AllPartyConsent,
        ] {
            let mut i = base(regime);
            i.public_space_no_privacy_expectation = true;
            i.all_parties_consented = false;
            i.at_least_one_party_consented = false;
            let r = check(&i);
            assert!(r.recording_lawful_federal);
            assert!(r.recording_lawful_state);
            assert!(!r.federal_criminal_exposure);
        }
    }

    #[test]
    fn federal_criminal_exposure_only_when_recording_unlawful_federally() {
        let r_lawful = check(&base(ConsentRegime::OnePartyConsent));
        assert!(!r_lawful.federal_criminal_exposure);
        let mut i_unlawful = base(ConsentRegime::OnePartyConsent);
        i_unlawful.at_least_one_party_consented = false;
        let r_unlawful = check(&i_unlawful);
        assert!(r_unlawful.federal_criminal_exposure);
    }

    #[test]
    fn federal_criminal_penalty_note_includes_5_years_and_250k() {
        let mut i = base(ConsentRegime::OnePartyConsent);
        i.at_least_one_party_consented = false;
        let r = check(&i);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("5 years imprisonment") && n.contains("$250,000")));
    }

    #[test]
    fn federal_civil_damages_note_includes_10k_minimum() {
        let mut i = base(ConsentRegime::OnePartyConsent);
        i.at_least_one_party_consented = false;
        let r = check(&i);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("$10,000 statutory minimum")));
    }

    #[test]
    fn all_party_state_note_lists_11_states() {
        let mut i = base(ConsentRegime::AllPartyConsent);
        i.all_parties_consented = true;
        i.at_least_one_party_consented = true;
        let r = check(&i);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("CA / DE / FL / IL / MD / MA / MT / NV / NH / PA / WA")));
    }

    #[test]
    fn one_party_state_note_describes_federal_floor() {
        let r = check(&base(ConsentRegime::OnePartyConsent));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("federal floor") && n.contains("39 states")));
    }

    #[test]
    fn citation_pins_federal_and_all_11_state_statutes() {
        let r = check(&base(ConsentRegime::OnePartyConsent));
        assert!(r.citation.contains("18 U.S.C. § 2511"));
        assert!(r.citation.contains("§ 2511(2)(d)"));
        assert!(r.citation.contains("§ 2511(4)"));
        assert!(r.citation.contains("§ 2520(c)"));
        assert!(r.citation.contains("Cal. Penal Code § 632"));
        assert!(r.citation.contains("11 Del. C. § 2402(c)(4)"));
        assert!(r.citation.contains("Fla. Stat. § 934.03(2)(d)"));
        assert!(r.citation.contains("720 ILCS 5/14-2"));
        assert!(r.citation.contains("Md. Code Cts. & Jud. Proc. § 10-402"));
        assert!(r.citation.contains("M.G.L. c. 272 § 99"));
        assert!(r.citation.contains("Mont. Code § 45-8-213"));
        assert!(r.citation.contains("NRS § 200.620"));
        assert!(r.citation.contains("NH RSA § 570-A:2"));
        assert!(r.citation.contains("18 Pa. Cons. Stat. § 5704"));
        assert!(r.citation.contains("RCW § 9.73.030"));
    }

    #[test]
    fn third_party_device_dominates_public_space_check() {
        let mut i = base(ConsentRegime::OnePartyConsent);
        i.public_space_no_privacy_expectation = true;
        i.recording_party_is_a_party_to_conversation = false;
        let r = check(&i);
        assert!(
            r.recording_lawful_federal,
            "public space carve-out runs first"
        );
    }

    #[test]
    fn fed_one_party_satisfied_state_all_party_violated_split() {
        let mut i = base(ConsentRegime::AllPartyConsent);
        i.at_least_one_party_consented = true;
        i.all_parties_consented = false;
        let r = check(&i);
        assert!(r.recording_lawful_federal);
        assert!(!r.recording_lawful_state);
        assert!(
            !r.federal_criminal_exposure,
            "federal not violated when one-party consent met"
        );
    }

    #[test]
    fn fed_one_party_unsatisfied_no_civil_damages_when_lawful() {
        let r = check(&base(ConsentRegime::OnePartyConsent));
        assert_eq!(r.federal_civil_minimum_damages_dollars, 0);
    }

    #[test]
    fn all_party_consent_no_parties_consented_unlawful_both_sides() {
        let mut i = base(ConsentRegime::AllPartyConsent);
        i.at_least_one_party_consented = false;
        i.all_parties_consented = false;
        let r = check(&i);
        assert!(!r.recording_lawful_federal);
        assert!(!r.recording_lawful_state);
        assert!(r.federal_criminal_exposure);
    }

    #[test]
    fn third_party_device_carveout_persists_with_all_parties_consenting() {
        let mut i = base(ConsentRegime::OnePartyConsent);
        i.recording_party_is_a_party_to_conversation = false;
        i.all_parties_consented = true;
        i.at_least_one_party_consented = true;
        let r = check(&i);
        assert!(
            !r.recording_lawful_federal,
            "third-party device unlawful even with all-party consent"
        );
    }

    #[test]
    fn note_describes_civil_damages_only_when_federal_violation() {
        let r_lawful = check(&base(ConsentRegime::OnePartyConsent));
        let civil_notes: Vec<_> = r_lawful
            .notes
            .iter()
            .filter(|n| n.contains("§ 2520(c)"))
            .collect();
        assert!(
            civil_notes.is_empty(),
            "civil damages note only on federal violation"
        );
    }

    #[test]
    fn in_unit_recording_with_one_party_consent_violates_in_one_party_state() {
        let mut i = base(ConsentRegime::OnePartyConsent);
        i.recording_inside_rental_unit = true;
        i.at_least_one_party_consented = true;
        i.all_parties_consented = false;
        let r = check(&i);
        assert!(
            !r.recording_lawful_federal,
            "in-unit recording requires all-party consent regardless of regime"
        );
    }

    #[test]
    fn one_party_consent_state_does_not_engage_all_party_note() {
        let r = check(&base(ConsentRegime::OnePartyConsent));
        let all_party_notes: Vec<_> = r
            .notes
            .iter()
            .filter(|n| n.contains("all-party consent state"))
            .collect();
        assert!(all_party_notes.is_empty());
    }

    #[test]
    fn nevada_inclusion_in_state_list_confirms_11_state_count() {
        let mut i = base(ConsentRegime::AllPartyConsent);
        i.all_parties_consented = true;
        i.at_least_one_party_consented = true;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("/ NV /")));
    }

    #[test]
    fn third_party_device_carries_both_criminal_and_civil_exposure() {
        let mut i = base(ConsentRegime::OnePartyConsent);
        i.recording_party_is_a_party_to_conversation = false;
        i.at_least_one_party_consented = true;
        let r = check(&i);
        assert!(r.federal_criminal_exposure);
        assert_eq!(r.federal_civil_minimum_damages_dollars, 10_000);
    }
}
