//! State landlord-identification / emergency-contact-information
//! disclosure compliance check. Addresses the upfront requirement that
//! a landlord disclose who they are, where they can be served, and
//! how the tenant reaches an emergency contact.
//!
//! California (Cal. Civ. Code § 1962) — landlord MUST disclose: (i)
//! name + telephone + STREET ADDRESS at which personal service may be
//! effected for each owner or owner's authorized agent for service of
//! process AND for receiving notices/demands; (ii) name + address of
//! entity to whom rent payments shall be made. Disclosure required
//! within 15 days of oral agreement OR within 15 days of written
//! agreement execution. Successor owner has 15 days to comply. STRICT
//! compliance — § 1962 compliance is effectively a JURISDICTIONAL
//! PREREQUISITE to an unlawful detainer action; defective disclosure
//! bars eviction.
//!
//! New Jersey (N.J.S.A. 46:8-27 through 46:8-37 "Landlord Identity
//! Law") — landlord must (a) REGISTER with municipal clerk within 30
//! days of creating tenancy; AND (b) supply registration info to each
//! tenant. Registration content includes: record owner name + address;
//! if corporate, registered agent + officers; if out-of-county owner,
//! in-county designee for service; managing agent name + address (if
//! any); EMERGENCY CONTACT — representative of owner or managing agent
//! who may be contacted in case of emergency. NJ-specific emergency-
//! contact requirement is the regression-critical distinguisher.
//!
//! Washington (RCW 59.18.060) — landlord must furnish in writing to
//! tenant: (i) name + address of person authorized to manage premises;
//! AND (ii) owner of premises OR person authorized to act for owner.
//! Simpler than CA + NJ.
//!
//! Default — no statewide statutory landlord-identification disclosure;
//! common-law right of tenant to identify landlord for purposes of
//! suit / service / rent payment.
//!
//! Citations: Cal. Civ. Code § 1962 (CA owner-identification + service-
//! address + 15-day deadline + § 1962 jurisdictional prerequisite to
//! unlawful detainer); N.J.S.A. 46:8-27 (NJ Landlord Identity Law);
//! N.J.S.A. 46:8-28 (registration content); N.J.S.A. 46:8-29 (30-day
//! deadline + emergency-contact requirement); RCW 59.18.060 (WA
//! landlord-identification disclosure).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    California,
    NewJersey,
    Washington,
    Default,
}

impl Regime {
    pub fn for_state(state: &str) -> Self {
        match state.trim().to_ascii_uppercase().as_str() {
            "CA" => Self::California,
            "NJ" => Self::NewJersey,
            "WA" => Self::Washington,
            _ => Self::Default,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct LandlordIdentificationInput {
    pub regime: Regime,
    /// Whether landlord provided written disclosure to tenant of owner /
    /// manager identification.
    pub written_disclosure_provided: bool,
    /// Whether disclosure includes name of owner / authorized agent.
    pub disclosure_includes_name: bool,
    /// Whether disclosure includes street address for personal service.
    pub disclosure_includes_street_address: bool,
    /// CA-specific: whether disclosure includes phone number.
    pub disclosure_includes_phone: bool,
    /// NJ-specific: whether disclosure identifies an emergency contact
    /// (representative of owner or managing agent).
    pub disclosure_includes_emergency_contact: bool,
    /// NJ-specific: whether landlord registered with municipal clerk.
    pub registered_with_municipal_clerk: bool,
    /// Days since the tenancy was created. Drives statutory deadline
    /// test (15 days CA / 30 days NJ).
    pub days_since_tenancy_created: u32,
    /// CA-specific: whether the landlord is attempting an unlawful
    /// detainer action. Drives § 1962 jurisdictional-prerequisite
    /// consequence.
    pub attempting_unlawful_detainer: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ViolationType {
    None,
    MissingWrittenDisclosure,
    MissingName,
    MissingStreetAddress,
    MissingPhone,
    MissingEmergencyContact,
    NotRegisteredWithMunicipalClerk,
    LatePastStatutoryDeadline,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LandlordIdentificationResult {
    pub regime: Regime,
    pub statutory_deadline_days: u32,
    pub unlawful_detainer_jurisdictionally_barred: bool,
    pub violation: ViolationType,
    pub landlord_compliant: bool,
    pub citation: &'static str,
    pub note: String,
}

pub fn check(input: &LandlordIdentificationInput) -> LandlordIdentificationResult {
    match input.regime {
        Regime::California => ca_check(input),
        Regime::NewJersey => nj_check(input),
        Regime::Washington => wa_check(input),
        Regime::Default => default_check(input),
    }
}

fn ca_check(input: &LandlordIdentificationInput) -> LandlordIdentificationResult {
    // CA strict-compliance requirements + 15-day deadline.
    let deadline = 15;
    let attempting_ud = input.attempting_unlawful_detainer;

    if !input.written_disclosure_provided {
        return LandlordIdentificationResult {
            regime: Regime::California,
            statutory_deadline_days: deadline,
            unlawful_detainer_jurisdictionally_barred: attempting_ud,
            violation: ViolationType::MissingWrittenDisclosure,
            landlord_compliant: false,
            citation: "Cal. Civ. Code § 1962 — strict compliance required; § 1962 compliance is JURISDICTIONAL PREREQUISITE to unlawful detainer action",
            note: "Required § 1962 written disclosure not provided. Compliance is jurisdictionally required for unlawful detainer.".to_string(),
        };
    }
    if !input.disclosure_includes_name {
        return ca_violation(ViolationType::MissingName, attempting_ud, "Name of owner or owner's authorized agent not disclosed under § 1962.");
    }
    if !input.disclosure_includes_street_address {
        return ca_violation(ViolationType::MissingStreetAddress, attempting_ud, "Street address for personal service of process not disclosed under § 1962.");
    }
    if !input.disclosure_includes_phone {
        return ca_violation(ViolationType::MissingPhone, attempting_ud, "Phone number not disclosed; § 1962 requires name + phone + street address.");
    }
    if input.days_since_tenancy_created > deadline {
        return ca_violation(ViolationType::LatePastStatutoryDeadline, attempting_ud, &format!(
            "Disclosure provided {} days after tenancy creation; § 1962 requires within 15 days.",
            input.days_since_tenancy_created
        ));
    }
    LandlordIdentificationResult {
        regime: Regime::California,
        statutory_deadline_days: deadline,
        unlawful_detainer_jurisdictionally_barred: false,
        violation: ViolationType::None,
        landlord_compliant: true,
        citation: "Cal. Civ. Code § 1962 — owner-identification disclosure compliance OK",
        note: "California § 1962 owner-identification disclosure requirements satisfied.".to_string(),
    }
}

fn ca_violation(
    v: ViolationType,
    attempting_ud: bool,
    note_text: &str,
) -> LandlordIdentificationResult {
    LandlordIdentificationResult {
        regime: Regime::California,
        statutory_deadline_days: 15,
        unlawful_detainer_jurisdictionally_barred: attempting_ud,
        violation: v,
        landlord_compliant: false,
        citation: "Cal. Civ. Code § 1962 — strict compliance required; defective disclosure bars unlawful detainer action",
        note: note_text.to_string(),
    }
}

fn nj_check(input: &LandlordIdentificationInput) -> LandlordIdentificationResult {
    let deadline = 30;
    if !input.registered_with_municipal_clerk {
        return LandlordIdentificationResult {
            regime: Regime::NewJersey,
            statutory_deadline_days: deadline,
            unlawful_detainer_jurisdictionally_barred: false,
            violation: ViolationType::NotRegisteredWithMunicipalClerk,
            landlord_compliant: false,
            citation: "N.J.S.A. 46:8-28 — landlord must file Certificate of Registration with municipal clerk within 30 days of creating tenancy",
            note: "Required N.J.S.A. 46:8-28 municipal clerk registration not completed.".to_string(),
        };
    }
    if !input.written_disclosure_provided {
        return LandlordIdentificationResult {
            regime: Regime::NewJersey,
            statutory_deadline_days: deadline,
            unlawful_detainer_jurisdictionally_barred: false,
            violation: ViolationType::MissingWrittenDisclosure,
            landlord_compliant: false,
            citation: "N.J.S.A. 46:8-29 — landlord must supply registration information to each tenant",
            note: "Required tenant disclosure of registration information not provided.".to_string(),
        };
    }
    if !input.disclosure_includes_name || !input.disclosure_includes_street_address {
        return LandlordIdentificationResult {
            regime: Regime::NewJersey,
            statutory_deadline_days: deadline,
            unlawful_detainer_jurisdictionally_barred: false,
            violation: ViolationType::MissingName,
            landlord_compliant: false,
            citation: "N.J.S.A. 46:8-28 — disclosure must include record owner name + address",
            note: "Disclosure missing owner name or address.".to_string(),
        };
    }
    if !input.disclosure_includes_emergency_contact {
        return LandlordIdentificationResult {
            regime: Regime::NewJersey,
            statutory_deadline_days: deadline,
            unlawful_detainer_jurisdictionally_barred: false,
            violation: ViolationType::MissingEmergencyContact,
            landlord_compliant: false,
            citation: "N.J.S.A. 46:8-28 — disclosure must include emergency contact (representative of owner or managing agent who may be contacted in case of emergency)",
            note: "Disclosure missing the required emergency-contact representative (NJ-only requirement).".to_string(),
        };
    }
    if input.days_since_tenancy_created > deadline {
        return LandlordIdentificationResult {
            regime: Regime::NewJersey,
            statutory_deadline_days: deadline,
            unlawful_detainer_jurisdictionally_barred: false,
            violation: ViolationType::LatePastStatutoryDeadline,
            landlord_compliant: false,
            citation: "N.J.S.A. 46:8-28 — registration must be completed within 30 days of creating tenancy",
            note: format!(
                "Registration / disclosure {} days after tenancy creation exceeds 30-day deadline.",
                input.days_since_tenancy_created
            ),
        };
    }
    LandlordIdentificationResult {
        regime: Regime::NewJersey,
        statutory_deadline_days: deadline,
        unlawful_detainer_jurisdictionally_barred: false,
        violation: ViolationType::None,
        landlord_compliant: true,
        citation: "N.J.S.A. 46:8-27 to 46:8-37 (NJ Landlord Identity Law) — compliance OK",
        note: "New Jersey Landlord Identity Law requirements satisfied: registration + tenant disclosure + emergency contact.".to_string(),
    }
}

fn wa_check(input: &LandlordIdentificationInput) -> LandlordIdentificationResult {
    if !input.written_disclosure_provided {
        return LandlordIdentificationResult {
            regime: Regime::Washington,
            statutory_deadline_days: 0,
            unlawful_detainer_jurisdictionally_barred: false,
            violation: ViolationType::MissingWrittenDisclosure,
            landlord_compliant: false,
            citation: "RCW 59.18.060 — landlord must furnish in writing to tenant name + address of person authorized to manage premises and owner / authorized agent",
            note: "Required RCW 59.18.060 written landlord-identification disclosure not provided.".to_string(),
        };
    }
    if !input.disclosure_includes_name {
        return LandlordIdentificationResult {
            regime: Regime::Washington,
            statutory_deadline_days: 0,
            unlawful_detainer_jurisdictionally_barred: false,
            violation: ViolationType::MissingName,
            landlord_compliant: false,
            citation: "RCW 59.18.060 — disclosure must include name of person authorized to manage premises + owner / authorized agent",
            note: "Required name disclosure not provided.".to_string(),
        };
    }
    if !input.disclosure_includes_street_address {
        return LandlordIdentificationResult {
            regime: Regime::Washington,
            statutory_deadline_days: 0,
            unlawful_detainer_jurisdictionally_barred: false,
            violation: ViolationType::MissingStreetAddress,
            landlord_compliant: false,
            citation: "RCW 59.18.060 — disclosure must include address of manager + owner",
            note: "Required address disclosure not provided.".to_string(),
        };
    }
    LandlordIdentificationResult {
        regime: Regime::Washington,
        statutory_deadline_days: 0,
        unlawful_detainer_jurisdictionally_barred: false,
        violation: ViolationType::None,
        landlord_compliant: true,
        citation: "RCW 59.18.060 — compliance OK",
        note: "Washington landlord-identification disclosure requirements satisfied.".to_string(),
    }
}

fn default_check(_input: &LandlordIdentificationInput) -> LandlordIdentificationResult {
    LandlordIdentificationResult {
        regime: Regime::Default,
        statutory_deadline_days: 0,
        unlawful_detainer_jurisdictionally_barred: false,
        violation: ViolationType::None,
        landlord_compliant: true,
        citation:
            "No statewide landlord-identification statute identified — common-law tenant right to identify landlord for service and rent purposes",
        note: "Default regime: common-law right of tenant to identify landlord. No specific statutory disclosure requirement.".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        regime: Regime,
        written: bool,
        name: bool,
        address: bool,
        phone: bool,
        emergency: bool,
        registered: bool,
        days: u32,
        ud: bool,
    ) -> LandlordIdentificationInput {
        LandlordIdentificationInput {
            regime,
            written_disclosure_provided: written,
            disclosure_includes_name: name,
            disclosure_includes_street_address: address,
            disclosure_includes_phone: phone,
            disclosure_includes_emergency_contact: emergency,
            registered_with_municipal_clerk: registered,
            days_since_tenancy_created: days,
            attempting_unlawful_detainer: ud,
        }
    }

    #[test]
    fn ca_full_compliance() {
        let r = check(&input(
            Regime::California,
            true,
            true,
            true,
            true,
            false,
            false,
            10,
            false,
        ));
        assert_eq!(r.violation, ViolationType::None);
        assert!(r.landlord_compliant);
        assert!(!r.unlawful_detainer_jurisdictionally_barred);
    }

    #[test]
    fn ca_missing_written_disclosure_bars_unlawful_detainer() {
        let r = check(&input(
            Regime::California,
            false,
            false,
            false,
            false,
            false,
            false,
            10,
            true,
        ));
        assert_eq!(r.violation, ViolationType::MissingWrittenDisclosure);
        assert!(r.unlawful_detainer_jurisdictionally_barred);
        assert!(r.citation.contains("JURISDICTIONAL"));
    }

    #[test]
    fn ca_missing_phone_violation() {
        let r = check(&input(
            Regime::California,
            true,
            true,
            true,
            false, // no phone
            false,
            false,
            10,
            false,
        ));
        assert_eq!(r.violation, ViolationType::MissingPhone);
    }

    #[test]
    fn ca_missing_street_address_violation() {
        let r = check(&input(
            Regime::California,
            true,
            true,
            false,
            true,
            false,
            false,
            10,
            false,
        ));
        assert_eq!(r.violation, ViolationType::MissingStreetAddress);
    }

    #[test]
    fn ca_past_15_day_deadline_violation() {
        let r = check(&input(
            Regime::California,
            true,
            true,
            true,
            true,
            false,
            false,
            16,
            false,
        ));
        assert_eq!(r.violation, ViolationType::LatePastStatutoryDeadline);
        assert!(r.citation.contains("§ 1962"));
    }

    #[test]
    fn ca_at_15_day_boundary_compliant() {
        let r = check(&input(
            Regime::California,
            true,
            true,
            true,
            true,
            false,
            false,
            15,
            false,
        ));
        assert_eq!(r.violation, ViolationType::None);
    }

    #[test]
    fn ca_unlawful_detainer_flag_only_when_attempting() {
        let r_attempting = check(&input(
            Regime::California,
            false,
            false,
            false,
            false,
            false,
            false,
            10,
            true,
        ));
        let r_not_attempting = check(&input(
            Regime::California,
            false,
            false,
            false,
            false,
            false,
            false,
            10,
            false,
        ));
        assert!(r_attempting.unlawful_detainer_jurisdictionally_barred);
        assert!(!r_not_attempting.unlawful_detainer_jurisdictionally_barred);
    }

    #[test]
    fn nj_full_compliance() {
        let r = check(&input(
            Regime::NewJersey,
            true,
            true,
            true,
            false,
            true,
            true,
            20,
            false,
        ));
        assert_eq!(r.violation, ViolationType::None);
        assert!(r.landlord_compliant);
    }

    #[test]
    fn nj_not_registered_violation() {
        let r = check(&input(
            Regime::NewJersey,
            true,
            true,
            true,
            false,
            true,
            false,
            20,
            false,
        ));
        assert_eq!(r.violation, ViolationType::NotRegisteredWithMunicipalClerk);
        assert!(r.citation.contains("46:8-28"));
    }

    #[test]
    fn nj_missing_emergency_contact_violation() {
        let r = check(&input(
            Regime::NewJersey,
            true,
            true,
            true,
            false,
            false, // no emergency contact
            true,
            20,
            false,
        ));
        assert_eq!(r.violation, ViolationType::MissingEmergencyContact);
        assert!(r.citation.contains("emergency contact"));
    }

    #[test]
    fn nj_past_30_day_deadline_violation() {
        let r = check(&input(
            Regime::NewJersey,
            true,
            true,
            true,
            false,
            true,
            true,
            31,
            false,
        ));
        assert_eq!(r.violation, ViolationType::LatePastStatutoryDeadline);
    }

    #[test]
    fn nj_at_30_day_boundary_compliant() {
        let r = check(&input(
            Regime::NewJersey,
            true,
            true,
            true,
            false,
            true,
            true,
            30,
            false,
        ));
        assert_eq!(r.violation, ViolationType::None);
    }

    #[test]
    fn wa_full_compliance() {
        let r = check(&input(
            Regime::Washington,
            true,
            true,
            true,
            false,
            false,
            false,
            10,
            false,
        ));
        assert_eq!(r.violation, ViolationType::None);
        assert!(r.landlord_compliant);
    }

    #[test]
    fn wa_missing_disclosure_violation() {
        let r = check(&input(
            Regime::Washington,
            false,
            false,
            false,
            false,
            false,
            false,
            10,
            false,
        ));
        assert_eq!(r.violation, ViolationType::MissingWrittenDisclosure);
        assert!(r.citation.contains("RCW 59.18.060"));
    }

    #[test]
    fn wa_missing_name_violation() {
        let r = check(&input(
            Regime::Washington,
            true,
            false,
            true,
            false,
            false,
            false,
            10,
            false,
        ));
        assert_eq!(r.violation, ViolationType::MissingName);
    }

    #[test]
    fn wa_missing_address_violation() {
        let r = check(&input(
            Regime::Washington,
            true,
            true,
            false,
            false,
            false,
            false,
            10,
            false,
        ));
        assert_eq!(r.violation, ViolationType::MissingStreetAddress);
    }

    #[test]
    fn wa_no_phone_or_emergency_contact_requirement() {
        // WA does not require phone or emergency-contact disclosure.
        let r = check(&input(
            Regime::Washington,
            true,
            true,
            true,
            false,
            false,
            false,
            10,
            false,
        ));
        assert_eq!(r.violation, ViolationType::None);
    }

    #[test]
    fn default_no_obligation() {
        let r = check(&input(
            Regime::Default,
            false,
            false,
            false,
            false,
            false,
            false,
            100,
            true,
        ));
        assert!(r.landlord_compliant);
        assert!(r.citation.contains("common-law"));
    }

    #[test]
    fn state_routing_ca_nj_wa_default() {
        assert_eq!(Regime::for_state("CA"), Regime::California);
        assert_eq!(Regime::for_state("NJ"), Regime::NewJersey);
        assert_eq!(Regime::for_state("WA"), Regime::Washington);
        assert_eq!(Regime::for_state("TX"), Regime::Default);
        assert_eq!(Regime::for_state("NY"), Regime::Default);
    }

    #[test]
    fn state_routing_case_insensitive() {
        assert_eq!(Regime::for_state("ca"), Regime::California);
        assert_eq!(Regime::for_state("nj"), Regime::NewJersey);
        assert_eq!(Regime::for_state("wa"), Regime::Washington);
    }

    #[test]
    fn only_ca_has_unlawful_detainer_jurisdictional_consequence() {
        // Missing disclosure scenario across regimes with UD attempt.
        let ca = check(&input(
            Regime::California,
            false,
            false,
            false,
            false,
            false,
            false,
            10,
            true,
        ));
        let nj = check(&input(
            Regime::NewJersey,
            false,
            false,
            false,
            false,
            false,
            false,
            10,
            true,
        ));
        let wa = check(&input(
            Regime::Washington,
            false,
            false,
            false,
            false,
            false,
            false,
            10,
            true,
        ));
        assert!(ca.unlawful_detainer_jurisdictionally_barred);
        assert!(!nj.unlawful_detainer_jurisdictionally_barred);
        assert!(!wa.unlawful_detainer_jurisdictionally_barred);
    }

    #[test]
    fn only_nj_has_emergency_contact_requirement() {
        // Disclosure missing emergency contact across regimes.
        let nj = check(&input(
            Regime::NewJersey,
            true,
            true,
            true,
            false,
            false, // no emergency contact
            true,
            10,
            false,
        ));
        let ca = check(&input(
            Regime::California,
            true,
            true,
            true,
            true,
            false, // no emergency contact (CA doesn't require)
            false,
            10,
            false,
        ));
        let wa = check(&input(
            Regime::Washington,
            true,
            true,
            true,
            false,
            false,
            false,
            10,
            false,
        ));
        assert_eq!(nj.violation, ViolationType::MissingEmergencyContact);
        assert_eq!(ca.violation, ViolationType::None);
        assert_eq!(wa.violation, ViolationType::None);
    }

    #[test]
    fn only_nj_requires_municipal_clerk_registration() {
        // Not-registered scenario across regimes.
        let nj = check(&input(
            Regime::NewJersey,
            true,
            true,
            true,
            false,
            true,
            false,
            10,
            false,
        ));
        let ca = check(&input(
            Regime::California,
            true,
            true,
            true,
            true,
            false,
            false, // not registered (irrelevant)
            10,
            false,
        ));
        assert_eq!(nj.violation, ViolationType::NotRegisteredWithMunicipalClerk);
        assert_eq!(ca.violation, ViolationType::None);
    }

    #[test]
    fn only_ca_requires_phone_disclosure() {
        // Disclosure missing phone across regimes.
        let ca = check(&input(
            Regime::California,
            true,
            true,
            true,
            false,
            false,
            false,
            10,
            false,
        ));
        let nj = check(&input(
            Regime::NewJersey,
            true,
            true,
            true,
            false,
            true,
            true,
            10,
            false,
        ));
        let wa = check(&input(
            Regime::Washington,
            true,
            true,
            true,
            false,
            false,
            false,
            10,
            false,
        ));
        assert_eq!(ca.violation, ViolationType::MissingPhone);
        assert_eq!(nj.violation, ViolationType::None);
        assert_eq!(wa.violation, ViolationType::None);
    }

    #[test]
    fn citations_pin_correct_authorities() {
        let ca = check(&input(
            Regime::California,
            false,
            false,
            false,
            false,
            false,
            false,
            10,
            false,
        ));
        assert!(ca.citation.contains("§ 1962"));

        let nj = check(&input(
            Regime::NewJersey,
            true,
            true,
            true,
            false,
            true,
            true,
            10,
            false,
        ));
        assert!(nj.citation.contains("46:8"));

        let wa = check(&input(
            Regime::Washington,
            true,
            true,
            true,
            false,
            false,
            false,
            10,
            false,
        ));
        assert!(wa.citation.contains("RCW 59.18.060"));
    }
}
