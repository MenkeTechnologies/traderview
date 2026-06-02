//! Tenant voting address protection and Address Confidentiality Program (ACP)
//! framework — covers landlord-disclosure restrictions when a tenant is
//! enrolled in a state ACP as a survivor of domestic violence, sexual assault,
//! stalking, or human trafficking. Distinct from sibling [[tenant_smart_lock_
//! biometric_consent]] (biometric privacy at unit access), [[security_camera_
//! disclosure]] (landlord-installed surveillance), [[tenant_data_privacy]]
//! (general tenant data handling), [[landlord_tenant_recording_consent]]
//! (audio recording consent).
//!
//! Trader-landlord critical because (1) ACP enrollment provides a state-
//! issued substitute address that legally REPLACES the tenant's residential
//! address on public records (voter registration, driver's license, school
//! enrollment, court records); (2) unauthorized disclosure of an ACP
//! participant's actual residential address by a landlord is a criminal
//! offense in most ACP states (typically misdemeanor, sometimes felony when
//! results in harm to participant); (3) civil damages plus attorney's fees
//! available to participant; (4) parallel non-ACP confidentiality duty
//! attaches when tenant discloses victim status to landlord; (5) voter-
//! registration challenger / opposition-research firm requests for tenant
//! addresses must be DENIED when tenant has invoked ACP or victim status.
//!
//! State frameworks:
//!
//! - **California** — Cal. Gov. Code § 6206-6210 Safe at Home Program;
//!   Cal. Civ. Code § 1946.7 lease termination right for survivors plus
//!   Cal. Civ. Code § 1161.3 unlawful detainer protection; Penal Code §
//!   273.6 violation of protective order. California requires proof of
//!   abuse for ACP eligibility.
//!
//! - **Massachusetts** — M.G.L. ch. 9A Address Confidentiality Program;
//!   M.G.L. ch. 209A Abuse Prevention; M.G.L. ch. 186 § 24-29 lease
//!   termination right for victims; M.G.L. ch. 151B § 4 fair housing
//!   protection.
//!
//! - **New York** — N.Y. Executive Law § 108 Address Confidentiality
//!   Program; N.Y. Real Property Law § 227-c lease termination right
//!   for victims; N.Y. Social Services Law § 459-a confidentiality.
//!
//! - **Washington** — RCW 40.24 Address Confidentiality Program; RCW
//!   59.18.575 lease termination right; RCW 59.18.585 prohibition on
//!   adverse action; Washington was first state to enact ACP (1991).
//!
//! - **Default** — federal Violence Against Women Act (VAWA) 42 U.S.C.
//!   § 14043e protections for federally-assisted housing; 24 C.F.R.
//!   § 5.2003 reasonable accommodation requirements; common-law privacy
//!   torts (intrusion upon seclusion, public disclosure of private facts).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    California,
    Massachusetts,
    NewYork,
    Washington,
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AcpStatus {
    /// Tenant enrolled in state Address Confidentiality Program with state-
    /// issued substitute address.
    EnrolledWithSubstituteAddress,
    /// Tenant disclosed victim status to landlord but not enrolled in ACP.
    VictimStatusDisclosedNotEnrolled,
    /// No ACP enrollment and no victim disclosure.
    NoAcpAndNoVictimDisclosure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DisclosureRequestType {
    /// Voter registration challenger / opposition research / partisan
    /// canvasser.
    VoterRegistrationChallenger,
    /// Law enforcement subpoena, court order, or warrant.
    LawEnforcementWithWarrantOrSubpoena,
    /// Law enforcement informal request without warrant.
    LawEnforcementWithoutWarrant,
    /// Process server attempting to serve unrelated lawsuit.
    ProcessServerCivilUnrelated,
    /// Debt collector, skip tracer, or commercial information broker.
    DebtCollectorOrSkipTracer,
    /// Family member or estranged spouse claiming relationship.
    FamilyMemberOrEstrangedSpouse,
    /// Routine landlord-business request (vendor, repair tech, post-tenancy
    /// reference).
    RoutineLandlordBusiness,
    /// No disclosure request pending.
    NoRequest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    CompliantConfidentialityMaintained,
    DisclosureToVoterChallengerProhibited,
    DisclosureToFamilyOrEstrangedSpouseProhibited,
    DisclosureToDebtCollectorRequiresAcpSubstitute,
    DisclosureToProcessServerRequiresAcpSubstitute,
    DisclosureToLawEnforcementWithoutWarrantProhibited,
    LawEnforcementWithValidWarrantCompliant,
    ConfidentialityBreachAcpParticipantCriminalLiability,
    ConfidentialityBreachNonAcpVictimTortLiability,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub jurisdiction: Jurisdiction,
    pub acp_status: AcpStatus,
    pub disclosure_request: DisclosureRequestType,
    pub landlord_disclosed_actual_address: bool,
    pub disclosure_caused_actual_harm_to_tenant: bool,
    pub landlord_acknowledged_acp_enrollment_in_writing: bool,
    pub annual_rent_cents: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub severity: Severity,
    pub recommended_actions: Vec<String>,
    pub annual_rent_at_risk_cents: u64,
    pub criminal_exposure_misdemeanor: bool,
    pub criminal_exposure_felony: bool,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub const WA_ACP_ENACTED_YEAR: i32 = 1991;
pub const FEDERAL_VAWA_TITLE: &str = "42 U.S.C. § 14043e";

pub fn check(input: &Input) -> Output {
    let mut actions: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();
    let severity: Severity;
    let mut criminal_misdemeanor = false;
    let mut criminal_felony = false;

    if matches!(
        input.acp_status,
        AcpStatus::NoAcpAndNoVictimDisclosure
    ) && matches!(input.disclosure_request, DisclosureRequestType::NoRequest)
    {
        notes.push(
            "No ACP enrollment, no victim status disclosure, no pending disclosure request; \
             framework inapplicable. Routine landlord-tenant confidentiality applies under \
             state lease law and common-law privacy torts."
                .to_string(),
        );
        return Output {
            severity: Severity::NotApplicable,
            recommended_actions: actions,
            annual_rent_at_risk_cents: 0,
            criminal_exposure_misdemeanor: false,
            criminal_exposure_felony: false,
            citation: "n/a",
            notes,
        };
    }

    let is_acp_participant = matches!(
        input.acp_status,
        AcpStatus::EnrolledWithSubstituteAddress
    );
    let is_victim_disclosed = matches!(
        input.acp_status,
        AcpStatus::VictimStatusDisclosedNotEnrolled
    );

    if input.landlord_disclosed_actual_address && is_acp_participant {
        if input.disclosure_caused_actual_harm_to_tenant {
            severity = Severity::ConfidentialityBreachAcpParticipantCriminalLiability;
            criminal_misdemeanor = true;
            criminal_felony = true;
            actions.push(
                "ACP participant's actual address disclosed AND disclosure caused actual harm \
                 to tenant — criminal liability under state ACP statute (CA Penal Code § 273.6, \
                 MA M.G.L. ch. 9A § 8, NY Executive Law § 108(5), WA RCW 40.24.090). MA + NY \
                 + WA classify intentional ACP-breach causing harm as FELONY; CA misdemeanor \
                 with possible felony upgrade. IMMEDIATE notification to state ACP \
                 administrator plus tenant plus law-enforcement coordination."
                    .to_string(),
            );
        } else {
            severity = Severity::ConfidentialityBreachAcpParticipantCriminalLiability;
            criminal_misdemeanor = true;
            actions.push(
                "ACP participant's actual address disclosed by landlord — misdemeanor under \
                 most state ACP statutes regardless of resulting harm. CA Penal Code § 273.6 \
                 + MA M.G.L. ch. 9A § 8 + NY Executive Law § 108(5) + WA RCW 40.24.090. \
                 Civil damages plus attorney's fees available to participant; notify state \
                 ACP administrator immediately."
                    .to_string(),
            );
        }
    } else if input.landlord_disclosed_actual_address && is_victim_disclosed {
        severity = Severity::ConfidentialityBreachNonAcpVictimTortLiability;
        actions.push(
            "Victim status disclosed to landlord but tenant not ACP-enrolled; unauthorized \
             address disclosure triggers common-law privacy tort liability (intrusion upon \
             seclusion, public disclosure of private facts) plus state-statutory \
             confidentiality duty per CA Civ. Code § 1946.7, MA M.G.L. ch. 186 § 24-29, NY \
             RPL § 227-c, WA RCW 59.18.575. Damages include emotional distress, relocation \
             costs, and security expenses."
                .to_string(),
        );
    } else if matches!(
        input.disclosure_request,
        DisclosureRequestType::VoterRegistrationChallenger
    ) && (is_acp_participant || is_victim_disclosed)
    {
        severity = Severity::DisclosureToVoterChallengerProhibited;
        actions.push(
            "Voter registration challenger or opposition-research firm requesting tenant \
             address — DENY disclosure for any ACP participant or disclosed-victim tenant. \
             Provide substitute address per ACP enrollment OR refuse disclosure entirely \
             citing tenant's protected status. ACP-issued substitute address is the LEGAL \
             address for voter registration per state ACP statute; no challenge can pierce."
                .to_string(),
        );
    } else if matches!(
        input.disclosure_request,
        DisclosureRequestType::FamilyMemberOrEstrangedSpouse
    ) && (is_acp_participant || is_victim_disclosed)
    {
        severity = Severity::DisclosureToFamilyOrEstrangedSpouseProhibited;
        actions.push(
            "Family member or estranged spouse requesting tenant address — DENY disclosure \
             regardless of claimed relationship. Estranged spouse / former partner is the \
             most common perpetrator pattern in DV and stalking cases; ACP enrollment plus \
             victim-status disclosure specifically protects against family-member \
             discovery. Refuse politely without confirming or denying tenancy."
                .to_string(),
        );
    } else if matches!(
        input.disclosure_request,
        DisclosureRequestType::DebtCollectorOrSkipTracer
    ) && is_acp_participant
    {
        severity = Severity::DisclosureToDebtCollectorRequiresAcpSubstitute;
        actions.push(
            "Debt collector or skip tracer requesting ACP participant address — provide \
             ACP-issued substitute address ONLY. Actual residential address remains \
             confidential. Debt collector required by FDCPA 15 U.S.C. § 1692c plus state \
             ACP statute to use substitute address for service of process and account \
             communications."
                .to_string(),
        );
    } else if matches!(
        input.disclosure_request,
        DisclosureRequestType::ProcessServerCivilUnrelated
    ) && is_acp_participant
    {
        severity = Severity::DisclosureToProcessServerRequiresAcpSubstitute;
        actions.push(
            "Process server requesting ACP participant address for service of unrelated \
             civil lawsuit — provide ACP-issued substitute address ONLY. Service deemed \
             valid at substitute address per state ACP statute; civil rules of procedure \
             accommodate ACP substitute service."
                .to_string(),
        );
    } else if matches!(
        input.disclosure_request,
        DisclosureRequestType::LawEnforcementWithoutWarrant
    ) && (is_acp_participant || is_victim_disclosed)
    {
        severity = Severity::DisclosureToLawEnforcementWithoutWarrantProhibited;
        actions.push(
            "Law enforcement informal request WITHOUT warrant or subpoena — DECLINE \
             disclosure absent warrant. Most state ACP statutes require a court order, \
             subpoena, or warrant for law-enforcement disclosure of ACP participant's actual \
             address. Document the request, the officer's identity, and the refusal in \
             writing; recommend law enforcement contact state ACP administrator for \
             expedited disclosure under ACP statute."
                .to_string(),
        );
    } else if matches!(
        input.disclosure_request,
        DisclosureRequestType::LawEnforcementWithWarrantOrSubpoena
    ) {
        severity = Severity::LawEnforcementWithValidWarrantCompliant;
        actions.push(
            "Law enforcement request WITH valid warrant, subpoena, or court order — \
             disclosure permitted per state ACP statute exception. Verify warrant / \
             subpoena facial validity, retain copy, document compliance in landlord file, \
             notify ACP administrator that disclosure occurred per warrant exception. \
             Provide notice to tenant unless gag order applies."
                .to_string(),
        );
    } else {
        severity = Severity::CompliantConfidentialityMaintained;
        actions.push(
            "Confidentiality maintained per state ACP statute and lease confidentiality \
             clause. Document acknowledgment of ACP enrollment in landlord file plus train \
             property-management staff on ACP non-disclosure protocols plus maintain \
             substitute address as the only address in all routine tenant records. Annual \
             refresher recommended."
                .to_string(),
        );
    }

    match input.jurisdiction {
        Jurisdiction::California => {
            notes.push(
                "Cal. Gov. Code § 6206-6210 Safe at Home Program governs CA ACP; \
                 administered by Secretary of State. Cal. Civ. Code § 1946.7 lease \
                 termination right for survivors; Cal. Civ. Code § 1161.3 unlawful detainer \
                 protection; Cal. Penal Code § 273.6 violation of protective order. CA \
                 requires proof of abuse documentation for ACP eligibility."
                    .to_string(),
            );
        }
        Jurisdiction::Massachusetts => {
            notes.push(
                "M.G.L. ch. 9A Address Confidentiality Program administered by Secretary \
                 of the Commonwealth. M.G.L. ch. 209A Abuse Prevention; M.G.L. ch. 186 § \
                 24-29 lease termination right for victims; M.G.L. ch. 151B § 4 fair \
                 housing protection. Intentional ACP-breach causing harm classified as \
                 FELONY under M.G.L. ch. 9A § 8."
                    .to_string(),
            );
        }
        Jurisdiction::NewYork => {
            notes.push(
                "N.Y. Executive Law § 108 Address Confidentiality Program administered by \
                 Department of State. N.Y. Real Property Law § 227-c lease termination \
                 right for victims; N.Y. Social Services Law § 459-a confidentiality. \
                 Reproductive health care services providers, employees, volunteers, \
                 patients, or their immediate family members also eligible (added post-\
                 Dobbs)."
                    .to_string(),
            );
        }
        Jurisdiction::Washington => {
            notes.push(format!(
                "RCW 40.24 Address Confidentiality Program — Washington was FIRST state to \
                 enact ACP in {}. RCW 59.18.575 lease termination right; RCW 59.18.585 \
                 prohibition on adverse action. WA ACP expanded to include Criminal Justice \
                 Affiliates, Election Officials, and Protected Health Care Workers \
                 targeted for threats or harassment.",
                WA_ACP_ENACTED_YEAR
            ));
        }
        Jurisdiction::Default => {
            notes.push(format!(
                "Federal Violence Against Women Act (VAWA) {} protections for federally-\
                 assisted housing; 24 C.F.R. § 5.2003 reasonable accommodation requirements; \
                 common-law privacy torts (intrusion upon seclusion, public disclosure of \
                 private facts). State ACP statutes operate as state-law overlay on federal \
                 VAWA baseline.",
                FEDERAL_VAWA_TITLE
            ));
        }
    }

    notes.push(
        "Coordination with [[tenant_smart_lock_biometric_consent]] (biometric privacy at \
         unit access — parallel privacy framework), [[security_camera_disclosure]] \
         (landlord-installed surveillance — distinct exposure pathway), [[tenant_data_\
         privacy]] (general tenant data handling), [[landlord_tenant_recording_consent]] \
         (audio recording consent), [[tenant_emotional_distress_damages]] (IIED claim for \
         malicious disclosure causing fear or actual harm), [[mid_tenancy_temporary_\
         relocation]] (emergency relocation if ACP confidentiality breached)."
            .to_string(),
    );

    let annual_rent_at_risk: u64 = match severity {
        Severity::ConfidentialityBreachAcpParticipantCriminalLiability => input.annual_rent_cents,
        Severity::ConfidentialityBreachNonAcpVictimTortLiability => input.annual_rent_cents,
        Severity::DisclosureToVoterChallengerProhibited
        | Severity::DisclosureToFamilyOrEstrangedSpouseProhibited
        | Severity::DisclosureToLawEnforcementWithoutWarrantProhibited => {
            input.annual_rent_cents.saturating_div(2)
        }
        _ => 0,
    };

    Output {
        severity,
        recommended_actions: actions,
        annual_rent_at_risk_cents: annual_rent_at_risk,
        criminal_exposure_misdemeanor: criminal_misdemeanor,
        criminal_exposure_felony: criminal_felony,
        citation: match input.jurisdiction {
            Jurisdiction::California => "Cal. Gov. Code § 6206-6210 + Cal. Civ. Code § 1946.7 + § 1161.3 + Penal § 273.6",
            Jurisdiction::Massachusetts => "M.G.L. ch. 9A + ch. 209A + ch. 186 § 24-29 + ch. 151B § 4",
            Jurisdiction::NewYork => "N.Y. Executive Law § 108 + RPL § 227-c + Social Services Law § 459-a",
            Jurisdiction::Washington => "RCW 40.24 + RCW 59.18.575 + 59.18.585",
            Jurisdiction::Default => "42 U.S.C. § 14043e VAWA + 24 C.F.R. § 5.2003 + common-law privacy torts",
        },
        notes,
    }
}

pub type TenantVotingAddressProtectionInput = Input;
pub type TenantVotingAddressProtectionResult = Output;

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> Input {
        Input {
            jurisdiction: Jurisdiction::California,
            acp_status: AcpStatus::EnrolledWithSubstituteAddress,
            disclosure_request: DisclosureRequestType::NoRequest,
            landlord_disclosed_actual_address: false,
            disclosure_caused_actual_harm_to_tenant: false,
            landlord_acknowledged_acp_enrollment_in_writing: true,
            annual_rent_cents: 36_000_00,
        }
    }

    #[test]
    fn no_acp_no_victim_no_request_not_applicable() {
        let mut i = baseline();
        i.acp_status = AcpStatus::NoAcpAndNoVictimDisclosure;
        i.disclosure_request = DisclosureRequestType::NoRequest;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::NotApplicable));
        assert_eq!(r.annual_rent_at_risk_cents, 0);
        assert!(!r.criminal_exposure_misdemeanor);
        assert!(!r.criminal_exposure_felony);
    }

    #[test]
    fn acp_participant_no_disclosure_compliant() {
        let i = baseline();
        let r = check(&i);
        assert!(matches!(r.severity, Severity::CompliantConfidentialityMaintained));
        assert_eq!(r.annual_rent_at_risk_cents, 0);
    }

    #[test]
    fn acp_participant_address_disclosed_criminal_misdemeanor() {
        let mut i = baseline();
        i.landlord_disclosed_actual_address = true;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::ConfidentialityBreachAcpParticipantCriminalLiability
        ));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents);
        assert!(r.criminal_exposure_misdemeanor);
        assert!(!r.criminal_exposure_felony);
    }

    #[test]
    fn acp_breach_with_harm_felony_upgrade() {
        let mut i = baseline();
        i.landlord_disclosed_actual_address = true;
        i.disclosure_caused_actual_harm_to_tenant = true;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::ConfidentialityBreachAcpParticipantCriminalLiability
        ));
        assert!(r.criminal_exposure_misdemeanor);
        assert!(r.criminal_exposure_felony);
        assert!(r.recommended_actions.iter().any(|a| a.contains("FELONY")));
    }

    #[test]
    fn non_acp_victim_disclosure_tort_liability() {
        let mut i = baseline();
        i.acp_status = AcpStatus::VictimStatusDisclosedNotEnrolled;
        i.landlord_disclosed_actual_address = true;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::ConfidentialityBreachNonAcpVictimTortLiability
        ));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents);
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("intrusion upon seclusion")));
    }

    #[test]
    fn voter_challenger_request_for_acp_prohibited() {
        let mut i = baseline();
        i.disclosure_request = DisclosureRequestType::VoterRegistrationChallenger;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::DisclosureToVoterChallengerProhibited));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents / 2);
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("substitute address")));
    }

    #[test]
    fn voter_challenger_for_non_acp_victim_disclosed_prohibited() {
        let mut i = baseline();
        i.acp_status = AcpStatus::VictimStatusDisclosedNotEnrolled;
        i.disclosure_request = DisclosureRequestType::VoterRegistrationChallenger;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::DisclosureToVoterChallengerProhibited));
    }

    #[test]
    fn family_member_request_prohibited() {
        let mut i = baseline();
        i.disclosure_request = DisclosureRequestType::FamilyMemberOrEstrangedSpouse;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::DisclosureToFamilyOrEstrangedSpouseProhibited
        ));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("Refuse politely")));
    }

    #[test]
    fn debt_collector_request_requires_acp_substitute() {
        let mut i = baseline();
        i.disclosure_request = DisclosureRequestType::DebtCollectorOrSkipTracer;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::DisclosureToDebtCollectorRequiresAcpSubstitute
        ));
        assert!(r.recommended_actions.iter().any(|a| a.contains("FDCPA 15 U.S.C. § 1692c")));
    }

    #[test]
    fn process_server_request_requires_acp_substitute() {
        let mut i = baseline();
        i.disclosure_request = DisclosureRequestType::ProcessServerCivilUnrelated;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::DisclosureToProcessServerRequiresAcpSubstitute
        ));
    }

    #[test]
    fn law_enforcement_no_warrant_prohibited() {
        let mut i = baseline();
        i.disclosure_request = DisclosureRequestType::LawEnforcementWithoutWarrant;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::DisclosureToLawEnforcementWithoutWarrantProhibited
        ));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("expedited disclosure")));
    }

    #[test]
    fn law_enforcement_with_warrant_compliant() {
        let mut i = baseline();
        i.disclosure_request = DisclosureRequestType::LawEnforcementWithWarrantOrSubpoena;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::LawEnforcementWithValidWarrantCompliant));
        assert_eq!(r.annual_rent_at_risk_cents, 0);
        assert!(r.recommended_actions.iter().any(|a| a.contains("warrant")));
    }

    #[test]
    fn ca_jurisdiction_pins_safe_at_home_program() {
        let i = baseline();
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("Safe at Home Program")));
        assert!(r.notes.iter().any(|n| n.contains("Cal. Gov. Code § 6206-6210")));
        assert!(r.notes.iter().any(|n| n.contains("Cal. Civ. Code § 1946.7")));
    }

    #[test]
    fn ma_jurisdiction_pins_mgl_ch_9a_felony() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Massachusetts;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("M.G.L. ch. 9A")));
        assert!(r.notes.iter().any(|n| n.contains("M.G.L. ch. 186 § 24-29")));
        assert!(r.notes.iter().any(|n| n.contains("FELONY")));
    }

    #[test]
    fn ny_jurisdiction_pins_executive_law_108_post_dobbs() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::NewYork;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("N.Y. Executive Law § 108")));
        assert!(r.notes.iter().any(|n| n.contains("Real Property Law § 227-c")));
        assert!(r.notes.iter().any(|n| n.contains("post-")));
        assert!(r.notes.iter().any(|n| n.contains("Dobbs")));
    }

    #[test]
    fn wa_jurisdiction_pins_first_state_1991() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Washington;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("RCW 40.24")));
        assert!(r.notes.iter().any(|n| n.contains("1991")));
        assert!(r.notes.iter().any(|n| n.contains("FIRST state")));
    }

    #[test]
    fn default_jurisdiction_pins_vawa_and_cfr() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Default;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("VAWA")));
        assert!(r.notes.iter().any(|n| n.contains("42 U.S.C. § 14043e")));
        assert!(r.notes.iter().any(|n| n.contains("24 C.F.R. § 5.2003")));
    }

    #[test]
    fn coordination_note_references_siblings() {
        let i = baseline();
        let r = check(&i);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("tenant_smart_lock_biometric_consent")));
        assert!(r.notes.iter().any(|n| n.contains("security_camera_disclosure")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("tenant_emotional_distress_damages")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("mid_tenancy_temporary_relocation")));
    }

    #[test]
    fn coordination_note_pinned_in_all_jurisdictions() {
        for j in [
            Jurisdiction::California,
            Jurisdiction::Massachusetts,
            Jurisdiction::NewYork,
            Jurisdiction::Washington,
            Jurisdiction::Default,
        ] {
            let mut i = baseline();
            i.jurisdiction = j;
            let r = check(&i);
            assert!(
                r.notes
                    .iter()
                    .any(|n| n.contains("tenant_smart_lock_biometric_consent")),
                "coordination missing for {j:?}"
            );
        }
    }

    #[test]
    fn wa_acp_enacted_year_pins_1991() {
        assert_eq!(WA_ACP_ENACTED_YEAR, 1991);
    }

    #[test]
    fn federal_vawa_title_pins_42_usc_14043e() {
        assert_eq!(FEDERAL_VAWA_TITLE, "42 U.S.C. § 14043e");
    }

    #[test]
    fn severity_priority_disclosure_breach_overrides_request() {
        let mut i = baseline();
        i.landlord_disclosed_actual_address = true;
        i.disclosure_request = DisclosureRequestType::VoterRegistrationChallenger;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::ConfidentialityBreachAcpParticipantCriminalLiability
        ));
    }

    #[test]
    fn zero_rent_zero_risk_does_not_panic() {
        let mut i = baseline();
        i.annual_rent_cents = 0;
        i.landlord_disclosed_actual_address = true;
        let r = check(&i);
        assert_eq!(r.annual_rent_at_risk_cents, 0);
    }

    #[test]
    fn citation_branch_for_each_jurisdiction() {
        let ca = check(&{ let mut i = baseline(); i.jurisdiction = Jurisdiction::California; i });
        let ma = check(&{ let mut i = baseline(); i.jurisdiction = Jurisdiction::Massachusetts; i });
        let ny = check(&{ let mut i = baseline(); i.jurisdiction = Jurisdiction::NewYork; i });
        let wa = check(&{ let mut i = baseline(); i.jurisdiction = Jurisdiction::Washington; i });
        let de = check(&{ let mut i = baseline(); i.jurisdiction = Jurisdiction::Default; i });
        assert!(ca.citation.contains("Cal. Gov. Code"));
        assert!(ma.citation.contains("M.G.L. ch. 9A"));
        assert!(ny.citation.contains("N.Y. Executive Law"));
        assert!(wa.citation.contains("RCW 40.24"));
        assert!(de.citation.contains("VAWA"));
    }

    #[test]
    fn non_acp_voter_challenger_no_severity_change() {
        let mut i = baseline();
        i.acp_status = AcpStatus::NoAcpAndNoVictimDisclosure;
        i.disclosure_request = DisclosureRequestType::VoterRegistrationChallenger;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::CompliantConfidentialityMaintained));
    }

    #[test]
    fn routine_landlord_business_no_severity_change() {
        let mut i = baseline();
        i.disclosure_request = DisclosureRequestType::RoutineLandlordBusiness;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::CompliantConfidentialityMaintained));
    }
}
