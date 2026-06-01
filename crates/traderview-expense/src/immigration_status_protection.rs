//! State immigration-status tenant protection landlord compliance check.
//!
//! Two states have enacted Immigrant Tenant Protection Acts that
//! prohibit landlords from using a tenant's immigration status as a
//! tool for harassment, retaliation, or eviction. California was first
//! (AB 291, 2017); Illinois followed (2019, Public Act 101-0439). The
//! federal Fair Housing Act bars national-origin discrimination but
//! does NOT specifically protect against immigration-status-related
//! threats — these state laws fill that gap.
//!
//! California (AB 291; Cal. Civ. Code §§ 1940.05, 1940.2, 1940.3,
//! 1942.5) — landlord may not disclose to any immigration authority,
//! law enforcement, or any local/state/federal agency information
//! relating to the immigration or citizenship status of any tenant,
//! occupant, or associated person, FOR THE PURPOSE of: harassment;
//! intimidation; retaliation for exercise of rights; influencing the
//! tenant to vacate; or recovering possession of the dwelling. Also
//! prohibits threats of such disclosure. Civil penalty up to $2,000
//! per violation; AG/DA may bring criminal charges. Judicial-warrant
//! carve-out: disclosure permitted when served with a warrant or
//! subpoena signed by a judge as part of a criminal investigation.
//!
//! Illinois (765 ILCS 755/ Immigrant Tenant Protection Act, eff.
//! 2019-08-23) — landlord may not evict or retaliate against a tenant
//! based on citizenship or immigration status; may not intimidate by
//! disclosing or threatening to disclose immigration status to any
//! person, entity, or immigration / law enforcement agency. Remedies:
//! actual damages + civil penalty up to $2,000 per violation +
//! reasonable attorney's fees + equitable relief.
//!
//! Default — no statewide immigration-tenant-protection statute. Fair
//! Housing Act (42 U.S.C. § 3604) bars national-origin discrimination
//! but does NOT specifically address immigration-status threats.
//!
//! Citations: Cal. Civ. Code § 1940.05 (definitions); § 1940.2 (immigration
//! disclosure prohibition); § 1940.3 (housing-application immigration-
//! inquiry prohibition); § 1942.5(g) (retaliation remedies); AB 291 (2017);
//! 765 ILCS 755/ (Illinois Immigrant Tenant Protection Act, 2019); IL Pub.
//! Act 101-0439; 42 U.S.C. § 3604 (FHA national-origin discrimination).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    CaliforniaAb291,
    Illinois,
    Default,
}

impl Regime {
    pub fn for_state(state: &str) -> Self {
        match state.trim().to_ascii_uppercase().as_str() {
            "CA" => Self::CaliforniaAb291,
            "IL" => Self::Illinois,
            _ => Self::Default,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProhibitedActType {
    /// Disclosed immigration status to ICE / law enforcement / other agency.
    DisclosureToAgency,
    /// Threatened to disclose immigration status to coerce tenant.
    ThreatToDisclose,
    /// Filed eviction action based on tenant's immigration status.
    EvictionBasedOnStatus,
    /// Retaliated against tenant for exercising rights, using immigration
    /// status as the lever.
    RetaliationBasedOnStatus,
    /// Asked about immigration status on the rental application — CA-only
    /// prohibition under § 1940.3.
    ApplicationInquiry,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ImmigrationProtectionInput {
    pub regime: Regime,
    pub act_type: ProhibitedActType,
    /// Whether the landlord was served with a judicial warrant or judge-
    /// signed subpoena as part of a criminal investigation. CA carve-out
    /// permits disclosure under such legal process.
    pub judicial_warrant_or_subpoena: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ViolationType {
    None,
    ProhibitedDisclosure,
    ProhibitedThreat,
    ProhibitedEviction,
    ProhibitedRetaliation,
    ProhibitedApplicationInquiry,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ImmigrationProtectionResult {
    pub regime: Regime,
    pub act_prohibited: bool,
    pub judicial_carve_out_applies: bool,
    pub max_civil_penalty_per_violation_cents: i64,
    pub criminal_prosecution_available: bool,
    pub actual_damages_recoverable: bool,
    pub attorney_fees_recoverable: bool,
    pub violation: ViolationType,
    pub landlord_compliant: bool,
    pub citation: &'static str,
    pub note: String,
}

pub fn check(input: &ImmigrationProtectionInput) -> ImmigrationProtectionResult {
    match input.regime {
        Regime::CaliforniaAb291 => ca_check(input),
        Regime::Illinois => il_check(input),
        Regime::Default => default_check(input),
    }
}

fn ca_check(input: &ImmigrationProtectionInput) -> ImmigrationProtectionResult {
    // CA carve-out: disclosure permitted only when responding to a
    // judicial warrant or judge-signed subpoena in a criminal
    // investigation. Other prohibited acts are NEVER excused by warrant.
    if input.act_type == ProhibitedActType::DisclosureToAgency
        && input.judicial_warrant_or_subpoena
    {
        return ImmigrationProtectionResult {
            regime: Regime::CaliforniaAb291,
            act_prohibited: false,
            judicial_carve_out_applies: true,
            max_civil_penalty_per_violation_cents: 200000,
            criminal_prosecution_available: true,
            actual_damages_recoverable: true,
            attorney_fees_recoverable: true,
            violation: ViolationType::None,
            landlord_compliant: true,
            citation:
                "Cal. Civ. Code § 1940.2 — disclosure permitted when made in response to a judicial warrant or judge-signed subpoena in a criminal investigation",
            note: "Disclosure made under valid judicial warrant or judge-signed subpoena — § 1940.2 carve-out applies. Landlord compliant.".to_string(),
        };
    }
    let (violation, citation) = match input.act_type {
        ProhibitedActType::DisclosureToAgency => (
            ViolationType::ProhibitedDisclosure,
            "Cal. Civ. Code § 1940.2 — disclosure of immigration status to any agency to harass, intimidate, retaliate, induce vacate, or recover possession is PROHIBITED",
        ),
        ProhibitedActType::ThreatToDisclose => (
            ViolationType::ProhibitedThreat,
            "Cal. Civ. Code § 1940.2 — threat to disclose immigration status is PROHIBITED (AB 291)",
        ),
        ProhibitedActType::EvictionBasedOnStatus => (
            ViolationType::ProhibitedEviction,
            "Cal. Civ. Code § 1942.5 — eviction based on immigration status is retaliatory and PROHIBITED",
        ),
        ProhibitedActType::RetaliationBasedOnStatus => (
            ViolationType::ProhibitedRetaliation,
            "Cal. Civ. Code § 1942.5 — retaliation using immigration status as lever is PROHIBITED",
        ),
        ProhibitedActType::ApplicationInquiry => (
            ViolationType::ProhibitedApplicationInquiry,
            "Cal. Civ. Code § 1940.3 — landlord may not make inquiry into immigration status on rental application (CA-only prong)",
        ),
    };
    ImmigrationProtectionResult {
        regime: Regime::CaliforniaAb291,
        act_prohibited: true,
        judicial_carve_out_applies: false,
        max_civil_penalty_per_violation_cents: 200000,
        criminal_prosecution_available: true,
        actual_damages_recoverable: true,
        attorney_fees_recoverable: true,
        violation,
        landlord_compliant: false,
        citation,
        note: format!(
            "Act {:?} is prohibited under CA AB 291. Civil penalty up to $2,000 per violation; AG/DA may pursue criminal charges; tenant may also recover actual damages and attorney's fees.",
            input.act_type
        ),
    }
}

fn il_check(input: &ImmigrationProtectionInput) -> ImmigrationProtectionResult {
    // Illinois does NOT have a parallel CA-style § 1940.3 application-
    // inquiry prong; only the disclosure / threat / eviction /
    // retaliation prongs.
    if input.act_type == ProhibitedActType::ApplicationInquiry {
        return ImmigrationProtectionResult {
            regime: Regime::Illinois,
            act_prohibited: false,
            judicial_carve_out_applies: false,
            max_civil_penalty_per_violation_cents: 200000,
            criminal_prosecution_available: false,
            actual_damages_recoverable: true,
            attorney_fees_recoverable: true,
            violation: ViolationType::None,
            landlord_compliant: true,
            citation:
                "765 ILCS 755/ — Illinois Immigrant Tenant Protection Act does NOT include a CA-style application-inquiry prong",
            note: "Illinois does not prohibit application-stage immigration inquiry. Other prongs (disclosure / threat / eviction / retaliation) are prohibited.".to_string(),
        };
    }
    let (violation, citation) = match input.act_type {
        ProhibitedActType::DisclosureToAgency => (
            ViolationType::ProhibitedDisclosure,
            "765 ILCS 755/ § 10 — disclosure of citizenship or immigration status to any person, entity, or immigration/law enforcement agency for intimidation is PROHIBITED",
        ),
        ProhibitedActType::ThreatToDisclose => (
            ViolationType::ProhibitedThreat,
            "765 ILCS 755/ § 10 — threat to disclose citizenship or immigration status is PROHIBITED",
        ),
        ProhibitedActType::EvictionBasedOnStatus => (
            ViolationType::ProhibitedEviction,
            "765 ILCS 755/ § 5 — eviction based on citizenship or immigration status is PROHIBITED",
        ),
        ProhibitedActType::RetaliationBasedOnStatus => (
            ViolationType::ProhibitedRetaliation,
            "765 ILCS 755/ § 5 — retaliation based on citizenship or immigration status is PROHIBITED",
        ),
        ProhibitedActType::ApplicationInquiry => unreachable!(),
    };
    ImmigrationProtectionResult {
        regime: Regime::Illinois,
        act_prohibited: true,
        judicial_carve_out_applies: false,
        max_civil_penalty_per_violation_cents: 200000,
        criminal_prosecution_available: false,
        actual_damages_recoverable: true,
        attorney_fees_recoverable: true,
        violation,
        landlord_compliant: false,
        citation,
        note: format!(
            "Act {:?} is prohibited under 765 ILCS 755/ (Illinois Immigrant Tenant Protection Act, eff. 2019-08-23). Civil penalty up to $2,000 per violation + actual damages + attorney fees + equitable relief.",
            input.act_type
        ),
    }
}

fn default_check(_input: &ImmigrationProtectionInput) -> ImmigrationProtectionResult {
    ImmigrationProtectionResult {
        regime: Regime::Default,
        act_prohibited: false,
        judicial_carve_out_applies: false,
        max_civil_penalty_per_violation_cents: 0,
        criminal_prosecution_available: false,
        actual_damages_recoverable: false,
        attorney_fees_recoverable: false,
        violation: ViolationType::None,
        landlord_compliant: true,
        citation:
            "No statewide immigrant-tenant-protection statute identified — 42 U.S.C. § 3604 (FHA) bars national-origin discrimination but does NOT specifically address immigration-status threats",
        note: "Default regime: no statewide statute. Federal Fair Housing Act may provide a national-origin-discrimination cause of action.".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(regime: Regime, act: ProhibitedActType, warrant: bool) -> ImmigrationProtectionInput {
        ImmigrationProtectionInput {
            regime,
            act_type: act,
            judicial_warrant_or_subpoena: warrant,
        }
    }

    #[test]
    fn ca_disclosure_to_ice_violation() {
        let r = check(&input(
            Regime::CaliforniaAb291,
            ProhibitedActType::DisclosureToAgency,
            false,
        ));
        assert_eq!(r.violation, ViolationType::ProhibitedDisclosure);
        assert!(!r.landlord_compliant);
        assert!(r.citation.contains("§ 1940.2"));
        assert_eq!(r.max_civil_penalty_per_violation_cents, 200000);
    }

    #[test]
    fn ca_judicial_warrant_carve_out_excuses_disclosure() {
        let r = check(&input(
            Regime::CaliforniaAb291,
            ProhibitedActType::DisclosureToAgency,
            true,
        ));
        assert!(r.judicial_carve_out_applies);
        assert!(r.landlord_compliant);
        assert_eq!(r.violation, ViolationType::None);
        assert!(r.citation.contains("judicial warrant"));
    }

    #[test]
    fn ca_threat_to_disclose_violation() {
        let r = check(&input(
            Regime::CaliforniaAb291,
            ProhibitedActType::ThreatToDisclose,
            false,
        ));
        assert_eq!(r.violation, ViolationType::ProhibitedThreat);
        assert!(r.citation.contains("AB 291"));
    }

    #[test]
    fn ca_threat_NOT_excused_by_warrant() {
        // Warrant only excuses DISCLOSURE, not threats.
        let r = check(&input(
            Regime::CaliforniaAb291,
            ProhibitedActType::ThreatToDisclose,
            true,
        ));
        assert_eq!(r.violation, ViolationType::ProhibitedThreat);
        assert!(!r.landlord_compliant);
    }

    #[test]
    fn ca_eviction_based_on_status_violation() {
        let r = check(&input(
            Regime::CaliforniaAb291,
            ProhibitedActType::EvictionBasedOnStatus,
            false,
        ));
        assert_eq!(r.violation, ViolationType::ProhibitedEviction);
        assert!(r.citation.contains("§ 1942.5"));
    }

    #[test]
    fn ca_retaliation_violation() {
        let r = check(&input(
            Regime::CaliforniaAb291,
            ProhibitedActType::RetaliationBasedOnStatus,
            false,
        ));
        assert_eq!(r.violation, ViolationType::ProhibitedRetaliation);
    }

    #[test]
    fn ca_application_inquiry_violation() {
        // CA-only prong under § 1940.3.
        let r = check(&input(
            Regime::CaliforniaAb291,
            ProhibitedActType::ApplicationInquiry,
            false,
        ));
        assert_eq!(r.violation, ViolationType::ProhibitedApplicationInquiry);
        assert!(r.citation.contains("§ 1940.3"));
        assert!(r.citation.contains("CA-only"));
    }

    #[test]
    fn ca_criminal_prosecution_available() {
        let r = check(&input(
            Regime::CaliforniaAb291,
            ProhibitedActType::DisclosureToAgency,
            false,
        ));
        assert!(r.criminal_prosecution_available);
    }

    #[test]
    fn il_disclosure_violation() {
        let r = check(&input(
            Regime::Illinois,
            ProhibitedActType::DisclosureToAgency,
            false,
        ));
        assert_eq!(r.violation, ViolationType::ProhibitedDisclosure);
        assert!(r.citation.contains("765 ILCS 755/"));
    }

    #[test]
    fn il_threat_violation() {
        let r = check(&input(
            Regime::Illinois,
            ProhibitedActType::ThreatToDisclose,
            false,
        ));
        assert_eq!(r.violation, ViolationType::ProhibitedThreat);
    }

    #[test]
    fn il_eviction_violation() {
        let r = check(&input(
            Regime::Illinois,
            ProhibitedActType::EvictionBasedOnStatus,
            false,
        ));
        assert_eq!(r.violation, ViolationType::ProhibitedEviction);
        assert!(r.citation.contains("§ 5"));
    }

    #[test]
    fn il_application_inquiry_NOT_prohibited() {
        // Illinois doesn't have a § 1940.3-equivalent application prong.
        let r = check(&input(
            Regime::Illinois,
            ProhibitedActType::ApplicationInquiry,
            false,
        ));
        assert_eq!(r.violation, ViolationType::None);
        assert!(r.landlord_compliant);
        assert!(r.citation.contains("does NOT include"));
    }

    #[test]
    fn il_no_judicial_warrant_carve_out() {
        // Illinois has no warrant carve-out — disclosure is always prohibited.
        let r = check(&input(
            Regime::Illinois,
            ProhibitedActType::DisclosureToAgency,
            true,
        ));
        assert_eq!(r.violation, ViolationType::ProhibitedDisclosure);
        assert!(!r.judicial_carve_out_applies);
    }

    #[test]
    fn il_no_criminal_prosecution() {
        let r = check(&input(
            Regime::Illinois,
            ProhibitedActType::DisclosureToAgency,
            false,
        ));
        assert!(!r.criminal_prosecution_available);
    }

    #[test]
    fn il_2000_civil_penalty() {
        let r = check(&input(
            Regime::Illinois,
            ProhibitedActType::DisclosureToAgency,
            false,
        ));
        assert_eq!(r.max_civil_penalty_per_violation_cents, 200000);
    }

    #[test]
    fn default_no_obligation() {
        let r = check(&input(
            Regime::Default,
            ProhibitedActType::DisclosureToAgency,
            false,
        ));
        assert_eq!(r.violation, ViolationType::None);
        assert!(r.landlord_compliant);
        assert!(r.citation.contains("FHA"));
    }

    #[test]
    fn state_routing_ca_il_default() {
        assert_eq!(Regime::for_state("CA"), Regime::CaliforniaAb291);
        assert_eq!(Regime::for_state("IL"), Regime::Illinois);
        assert_eq!(Regime::for_state("TX"), Regime::Default);
        assert_eq!(Regime::for_state("NY"), Regime::Default);
    }

    #[test]
    fn state_routing_case_insensitive() {
        assert_eq!(Regime::for_state("ca"), Regime::CaliforniaAb291);
        assert_eq!(Regime::for_state("il"), Regime::Illinois);
    }

    #[test]
    fn only_ca_has_application_inquiry_prong() {
        // Application inquiry is CA-only — IL and Default allow it.
        let ca = check(&input(
            Regime::CaliforniaAb291,
            ProhibitedActType::ApplicationInquiry,
            false,
        ));
        let il = check(&input(
            Regime::Illinois,
            ProhibitedActType::ApplicationInquiry,
            false,
        ));
        let d = check(&input(
            Regime::Default,
            ProhibitedActType::ApplicationInquiry,
            false,
        ));
        assert_eq!(ca.violation, ViolationType::ProhibitedApplicationInquiry);
        assert_eq!(il.violation, ViolationType::None);
        assert_eq!(d.violation, ViolationType::None);
    }

    #[test]
    fn only_ca_has_judicial_warrant_carve_out() {
        // Same warrant input across regimes — only CA's carve-out fires.
        let ca = check(&input(
            Regime::CaliforniaAb291,
            ProhibitedActType::DisclosureToAgency,
            true,
        ));
        let il = check(&input(
            Regime::Illinois,
            ProhibitedActType::DisclosureToAgency,
            true,
        ));
        assert!(ca.judicial_carve_out_applies);
        assert!(!il.judicial_carve_out_applies);
        // CA's disclosure is excused; IL's still violates.
        assert_eq!(ca.violation, ViolationType::None);
        assert_eq!(il.violation, ViolationType::ProhibitedDisclosure);
    }

    #[test]
    fn only_ca_has_criminal_prosecution() {
        let ca = check(&input(
            Regime::CaliforniaAb291,
            ProhibitedActType::DisclosureToAgency,
            false,
        ));
        let il = check(&input(
            Regime::Illinois,
            ProhibitedActType::DisclosureToAgency,
            false,
        ));
        let d = check(&input(
            Regime::Default,
            ProhibitedActType::DisclosureToAgency,
            false,
        ));
        assert!(ca.criminal_prosecution_available);
        assert!(!il.criminal_prosecution_available);
        assert!(!d.criminal_prosecution_available);
    }

    #[test]
    fn ca_and_il_both_2000_civil_penalty() {
        // Both regimes set the per-violation penalty at $2,000.
        let ca = check(&input(
            Regime::CaliforniaAb291,
            ProhibitedActType::DisclosureToAgency,
            false,
        ));
        let il = check(&input(
            Regime::Illinois,
            ProhibitedActType::DisclosureToAgency,
            false,
        ));
        assert_eq!(ca.max_civil_penalty_per_violation_cents, 200000);
        assert_eq!(il.max_civil_penalty_per_violation_cents, 200000);
    }

    #[test]
    fn citations_pin_correct_authorities() {
        let ca_d = check(&input(
            Regime::CaliforniaAb291,
            ProhibitedActType::DisclosureToAgency,
            false,
        ));
        assert!(ca_d.citation.contains("§ 1940.2"));

        let ca_a = check(&input(
            Regime::CaliforniaAb291,
            ProhibitedActType::ApplicationInquiry,
            false,
        ));
        assert!(ca_a.citation.contains("§ 1940.3"));

        let ca_e = check(&input(
            Regime::CaliforniaAb291,
            ProhibitedActType::EvictionBasedOnStatus,
            false,
        ));
        assert!(ca_e.citation.contains("§ 1942.5"));

        let il = check(&input(
            Regime::Illinois,
            ProhibitedActType::DisclosureToAgency,
            false,
        ));
        assert!(il.citation.contains("765 ILCS 755/"));
    }
}
