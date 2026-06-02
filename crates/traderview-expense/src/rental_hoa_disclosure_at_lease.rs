//! Rental property HOA / Condominium Association disclosure
//! at lease signing compliance — when a trader-landlord
//! renting a unit governed by a Homeowners Association
//! (HOA) or Condominium Owners Association (COA) must
//! disclose association rules, CC&Rs, fee structure,
//! rental restrictions, and tenant-information sharing to
//! prospective tenants at lease execution and to the
//! association before/after lease execution. Trader-
//! landlord operational concern: undisclosed HOA rules
//! create breach of warranty of quiet enjoyment + tenant
//! rescission claim + per-incident assessment exposure
//! when HOA fines landlord for tenant rule violation.
//! Distinct from siblings `rental_application_denial_
//! disclosure` (screening), `tenant_data_privacy` (general
//! data handling), `rental_property_registration` (state/
//! municipal registration), `short_term_rental_conversion`
//! (STR restrictions).
//!
//! **Three regimes**:
//!
//! **California — Cal. Civ. Code § 4740 (Davis-Stirling
//! Common Interest Development Act) + § 4525 + § 1102 et
//! seq.**:
//! - Landlord must provide HOA with **tenant's name and
//!   contact information before lease execution** (§ 4740).
//! - Tenant must comply with HOA's CC&Rs and rules
//!   regardless of whether disclosed at lease.
//! - Landlord may **redact personal and financial
//!   information** from signed lease before sending to
//!   HOA — protects tenant financial privacy.
//! - HOA rental restrictions limited under Davis-Stirling
//!   — pre-acquisition rental prohibitions enforceable;
//!   post-acquisition prohibitions grandfathered for
//!   existing owners.
//! - § 1102 Transfer Disclosure Statement (TDS) required
//!   on property sale (separate from lease disclosure).
//!
//! **Florida — FL Statute § 720.401 + § 718.503**:
//! - **Disclosure Summary** required before contract
//!   execution for HOA-governed property purchase (§
//!   720.401).
//! - Disclosure notifies buyer of (1) HOA membership
//!   subject + (2) potential assessments + (3) failure to
//!   pay results in lien on property.
//! - FL Condominium Act § 718.503 — condominium
//!   association may make **written demand to tenant** to
//!   submit rental payments to association until
//!   delinquent balance paid (rent diversion remedy).
//! - HOA documents must be furnished to prospective
//!   tenant on request including CC&Rs + rules + fee
//!   structure.
//!
//! **Nevada — Nev. Rev. Stat. § 116 (Uniform Common-
//! Interest Ownership Act) + § 116.335 + § 118A**:
//! - NRS § 116.335 — association may not prohibit unit
//!   owner from renting or leasing UNLESS declaration
//!   prohibited at time of purchase.
//! - Association may not require approval to rent or lease
//!   unless required at time of purchase.
//! - Pre-acquisition rental restrictions grandfathered;
//!   post-acquisition prohibition unenforceable.
//! - NRS § 118A Landlord and Tenant: Dwellings — general
//!   landlord-tenant disclosure framework applies.
//!
//! Citations: Cal. Civ. Code § 4740 + § 4525 + § 1102 et
//! seq. (Davis-Stirling Common Interest Development Act);
//! FL Statute § 720.401 + § 718.503 (Condominium Act + HOA
//! Disclosure Summary); Nev. Rev. Stat. § 116 + § 116.335
//! (Uniform Common-Interest Ownership Act) + § 118A
//! Landlord and Tenant Dwellings.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    California,
    Florida,
    Nevada,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RentalHoaDisclosureAtLeaseInput {
    pub regime: Regime,
    /// Whether tenant name and contact information provided
    /// to HOA before lease execution (CA § 4740).
    pub tenant_info_provided_to_hoa: bool,
    /// Whether HOA Disclosure Summary provided to tenant
    /// (FL § 720.401).
    pub hoa_disclosure_summary_provided: bool,
    /// Whether HOA CC&Rs + rules + fee structure furnished
    /// to prospective tenant.
    pub ccrs_rules_furnished: bool,
    /// Whether HOA imposed rental prohibition at time of
    /// owner's purchase (CA + NV grandfathering trigger).
    pub rental_prohibition_at_purchase: bool,
    /// Whether HOA imposed rental prohibition AFTER owner's
    /// purchase (post-acquisition restriction — generally
    /// unenforceable).
    pub rental_prohibition_post_acquisition: bool,
    /// Whether owner attempted to lease despite pre-
    /// acquisition prohibition.
    pub owner_leased_despite_pre_acquisition_prohibition: bool,
    /// Whether condominium association made written demand
    /// for rent diversion under FL § 718.503.
    pub fl_rent_diversion_demand: bool,
    /// Whether sensitive tenant financial information was
    /// shared with HOA without redaction (CA § 4740
    /// redaction right).
    pub ca_sensitive_info_shared_without_redaction: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RentalHoaDisclosureAtLeaseResult {
    pub disclosure_compliant: bool,
    pub hoa_notification_required: bool,
    pub disclosure_summary_required: bool,
    pub rental_restriction_enforceable: bool,
    pub tenant_redaction_right_available: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &RentalHoaDisclosureAtLeaseInput) -> RentalHoaDisclosureAtLeaseResult {
    match input.regime {
        Regime::California => check_ca(input),
        Regime::Florida => check_fl(input),
        Regime::Nevada => check_nv(input),
    }
}

fn check_ca(input: &RentalHoaDisclosureAtLeaseInput) -> RentalHoaDisclosureAtLeaseResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "Cal. Civ. Code § 4740 (Davis-Stirling Common Interest Development Act) — landlord must provide HOA with tenant's name and contact information BEFORE lease execution".to_string(),
        "Cal. Civ. Code § 4740 — landlord may REDACT personal and financial information from signed lease before sending to HOA; protects tenant financial privacy".to_string(),
        "Cal. Civ. Code § 4525 — HOA must furnish governing documents to prospective buyer (and analogously to prospective tenant) on request".to_string(),
        "Davis-Stirling Act — HOA rental restrictions: pre-acquisition rental prohibitions enforceable; post-acquisition prohibitions grandfathered for existing owners".to_string(),
        "Cal. Civ. Code § 1102 et seq. — Transfer Disclosure Statement (TDS) required at sale (separate from lease disclosure obligation)".to_string(),
    ];

    if !input.tenant_info_provided_to_hoa {
        violations.push(
            "Cal. Civ. Code § 4740 — landlord must provide HOA with tenant's name and contact information BEFORE lease execution".to_string(),
        );
    }

    if input.ca_sensitive_info_shared_without_redaction {
        violations.push(
            "Cal. Civ. Code § 4740 — landlord may redact personal and financial information from signed lease before sending to HOA; failure to redact may breach tenant privacy".to_string(),
        );
    }

    let rental_restriction_enforceable = input.rental_prohibition_at_purchase
        && !input.rental_prohibition_post_acquisition;

    if input.rental_prohibition_at_purchase
        && input.owner_leased_despite_pre_acquisition_prohibition
    {
        violations.push(
            "Davis-Stirling Act — pre-acquisition rental prohibition enforceable; owner leased despite prohibition existing at time of purchase".to_string(),
        );
    }

    RentalHoaDisclosureAtLeaseResult {
        disclosure_compliant: violations.is_empty(),
        hoa_notification_required: true,
        disclosure_summary_required: false,
        rental_restriction_enforceable,
        tenant_redaction_right_available: true,
        violations,
        citation: "Cal. Civ. Code § 4740 + § 4525 + § 1102 et seq. (Davis-Stirling Common Interest Development Act)",
        notes,
    }
}

fn check_fl(input: &RentalHoaDisclosureAtLeaseInput) -> RentalHoaDisclosureAtLeaseResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "FL Statute § 720.401 — HOA Disclosure Summary required before contract execution for HOA-governed property purchase".to_string(),
        "FL Statute § 720.401 — Disclosure Summary notifies buyer of (1) HOA membership subject + (2) potential assessments + (3) failure to pay results in lien on property".to_string(),
        "FL Statute § 718.503 (Condominium Act) — condominium association may make written demand to TENANT to submit rental payments to association until delinquent balance paid (rent diversion remedy)".to_string(),
        "Florida HOA documents must be furnished to prospective tenant on request including CC&Rs + rules + fee structure".to_string(),
        "FL Florida Realtors HOA/Condo Disclosure Statement separate from § 720.401 sale disclosure".to_string(),
    ];

    if !input.hoa_disclosure_summary_provided {
        violations.push(
            "FL Statute § 720.401 — HOA Disclosure Summary required for HOA-governed property; should be furnished to prospective tenant at lease signing".to_string(),
        );
    }

    if !input.ccrs_rules_furnished {
        violations.push(
            "Florida HOA framework — CC&Rs + rules + fee structure must be furnished to prospective tenant on request".to_string(),
        );
    }

    RentalHoaDisclosureAtLeaseResult {
        disclosure_compliant: violations.is_empty(),
        hoa_notification_required: false,
        disclosure_summary_required: true,
        rental_restriction_enforceable: true,
        tenant_redaction_right_available: false,
        violations,
        citation: "FL Statute § 720.401 + § 718.503 (Condominium Act + HOA Disclosure Summary)",
        notes,
    }
}

fn check_nv(input: &RentalHoaDisclosureAtLeaseInput) -> RentalHoaDisclosureAtLeaseResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "Nev. Rev. Stat. § 116 (Uniform Common-Interest Ownership Act) — comprehensive HOA legal framework".to_string(),
        "Nev. Rev. Stat. § 116.335 — association may NOT prohibit unit owner from renting or leasing UNLESS declaration prohibited at TIME OF PURCHASE".to_string(),
        "NRS § 116.335 — association may not require approval to rent or lease unless required at time of purchase".to_string(),
        "Pre-acquisition rental restrictions grandfathered; post-acquisition prohibition unenforceable against owner".to_string(),
        "Nev. Rev. Stat. § 118A — Landlord and Tenant Dwellings — general landlord-tenant disclosure framework applies in addition to HOA Chapter 116".to_string(),
    ];

    let rental_restriction_enforceable = input.rental_prohibition_at_purchase
        && !input.rental_prohibition_post_acquisition;

    if input.rental_prohibition_post_acquisition && !input.rental_prohibition_at_purchase {
        violations.push(
            "Nev. Rev. Stat. § 116.335 — association may not impose rental prohibition AFTER purchase; post-acquisition prohibition unenforceable against existing owner".to_string(),
        );
    }

    if input.rental_prohibition_at_purchase
        && input.owner_leased_despite_pre_acquisition_prohibition
    {
        violations.push(
            "Nev. Rev. Stat. § 116.335 — pre-acquisition rental prohibition enforceable; owner leased despite prohibition existing at time of purchase".to_string(),
        );
    }

    RentalHoaDisclosureAtLeaseResult {
        disclosure_compliant: violations.is_empty(),
        hoa_notification_required: false,
        disclosure_summary_required: false,
        rental_restriction_enforceable,
        tenant_redaction_right_available: false,
        violations,
        citation: "Nev. Rev. Stat. § 116 + § 116.335 (Uniform Common-Interest Ownership Act) + § 118A Landlord and Tenant Dwellings",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ca_clean() -> RentalHoaDisclosureAtLeaseInput {
        RentalHoaDisclosureAtLeaseInput {
            regime: Regime::California,
            tenant_info_provided_to_hoa: true,
            hoa_disclosure_summary_provided: false,
            ccrs_rules_furnished: false,
            rental_prohibition_at_purchase: false,
            rental_prohibition_post_acquisition: false,
            owner_leased_despite_pre_acquisition_prohibition: false,
            fl_rent_diversion_demand: false,
            ca_sensitive_info_shared_without_redaction: false,
        }
    }

    fn fl_clean() -> RentalHoaDisclosureAtLeaseInput {
        let mut i = ca_clean();
        i.regime = Regime::Florida;
        i.hoa_disclosure_summary_provided = true;
        i.ccrs_rules_furnished = true;
        i
    }

    fn nv_clean() -> RentalHoaDisclosureAtLeaseInput {
        let mut i = ca_clean();
        i.regime = Regime::Nevada;
        i
    }

    #[test]
    fn ca_clean_compliant() {
        let r = check(&ca_clean());
        assert!(r.disclosure_compliant);
        assert!(r.hoa_notification_required);
        assert!(r.tenant_redaction_right_available);
    }

    #[test]
    fn ca_no_tenant_info_to_hoa_violation() {
        let mut i = ca_clean();
        i.tenant_info_provided_to_hoa = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 4740") && v.contains("name and contact information")));
    }

    #[test]
    fn ca_sensitive_info_shared_without_redaction_violation() {
        let mut i = ca_clean();
        i.ca_sensitive_info_shared_without_redaction = true;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 4740") && v.contains("redact")));
    }

    #[test]
    fn ca_pre_acquisition_rental_prohibition_enforceable() {
        let mut i = ca_clean();
        i.rental_prohibition_at_purchase = true;
        let r = check(&i);
        assert!(r.rental_restriction_enforceable);
    }

    #[test]
    fn ca_post_acquisition_rental_prohibition_not_enforceable() {
        let mut i = ca_clean();
        i.rental_prohibition_post_acquisition = true;
        let r = check(&i);
        assert!(!r.rental_restriction_enforceable);
    }

    #[test]
    fn ca_owner_leased_despite_pre_acquisition_prohibition_violation() {
        let mut i = ca_clean();
        i.rental_prohibition_at_purchase = true;
        i.owner_leased_despite_pre_acquisition_prohibition = true;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("Davis-Stirling") && v.contains("pre-acquisition")));
    }

    #[test]
    fn fl_clean_compliant() {
        let r = check(&fl_clean());
        assert!(r.disclosure_compliant);
        assert!(r.disclosure_summary_required);
    }

    #[test]
    fn fl_no_disclosure_summary_violation() {
        let mut i = fl_clean();
        i.hoa_disclosure_summary_provided = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 720.401") && v.contains("HOA Disclosure Summary")));
    }

    #[test]
    fn fl_no_ccrs_rules_furnished_violation() {
        let mut i = fl_clean();
        i.ccrs_rules_furnished = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("CC&Rs") && v.contains("fee structure")));
    }

    #[test]
    fn nv_clean_compliant() {
        let r = check(&nv_clean());
        assert!(r.disclosure_compliant);
    }

    #[test]
    fn nv_post_acquisition_prohibition_unenforceable_violation() {
        let mut i = nv_clean();
        i.rental_prohibition_post_acquisition = true;
        i.rental_prohibition_at_purchase = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 116.335") && v.contains("AFTER purchase")));
    }

    #[test]
    fn nv_owner_leased_despite_pre_acquisition_prohibition_violation() {
        let mut i = nv_clean();
        i.rental_prohibition_at_purchase = true;
        i.owner_leased_despite_pre_acquisition_prohibition = true;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 116.335") && v.contains("pre-acquisition")));
    }

    #[test]
    fn nv_pre_acquisition_only_enforceable() {
        let mut i = nv_clean();
        i.rental_prohibition_at_purchase = true;
        i.rental_prohibition_post_acquisition = false;
        let r = check(&i);
        assert!(r.rental_restriction_enforceable);
    }

    #[test]
    fn citation_pins_ca_authority() {
        let r = check(&ca_clean());
        assert!(r.citation.contains("§ 4740"));
        assert!(r.citation.contains("§ 4525"));
        assert!(r.citation.contains("§ 1102"));
        assert!(r.citation.contains("Davis-Stirling"));
    }

    #[test]
    fn citation_pins_fl_authority() {
        let r = check(&fl_clean());
        assert!(r.citation.contains("§ 720.401"));
        assert!(r.citation.contains("§ 718.503"));
        assert!(r.citation.contains("Condominium Act"));
    }

    #[test]
    fn citation_pins_nv_authority() {
        let r = check(&nv_clean());
        assert!(r.citation.contains("§ 116"));
        assert!(r.citation.contains("§ 116.335"));
        assert!(r.citation.contains("§ 118A"));
        assert!(r.citation.contains("Common-Interest Ownership"));
    }

    #[test]
    fn note_pins_ca_tenant_info_requirement() {
        let r = check(&ca_clean());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 4740") && n.contains("BEFORE lease execution")));
    }

    #[test]
    fn note_pins_ca_redaction_right() {
        let r = check(&ca_clean());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("REDACT")
                && n.contains("financial privacy")));
    }

    #[test]
    fn note_pins_fl_disclosure_summary_three_elements() {
        let r = check(&fl_clean());
        assert!(r.notes.iter().any(|n| n.contains("§ 720.401")
            && n.contains("HOA membership")
            && n.contains("assessments")
            && n.contains("lien on property")));
    }

    #[test]
    fn note_pins_fl_condominium_rent_diversion() {
        let r = check(&fl_clean());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 718.503") && n.contains("rental payments")));
    }

    #[test]
    fn note_pins_nv_116_335_pre_acquisition_grandfathering() {
        let r = check(&nv_clean());
        assert!(r.notes.iter().any(|n| n.contains("§ 116.335")
            && n.contains("TIME OF PURCHASE")));
    }

    #[test]
    fn note_pins_nv_118a_landlord_tenant_framework() {
        let r = check(&nv_clean());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 118A") && n.contains("Landlord and Tenant")));
    }

    #[test]
    fn ca_uniquely_requires_tenant_info_to_hoa_invariant() {
        let mut i_ca = ca_clean();
        i_ca.tenant_info_provided_to_hoa = false;
        let r_ca = check(&i_ca);
        assert!(!r_ca.disclosure_compliant);

        let mut i_fl = fl_clean();
        i_fl.tenant_info_provided_to_hoa = false;
        let r_fl = check(&i_fl);
        assert!(r_fl.disclosure_compliant);
    }

    #[test]
    fn fl_uniquely_requires_disclosure_summary_invariant() {
        let mut i_fl = fl_clean();
        i_fl.hoa_disclosure_summary_provided = false;
        let r_fl = check(&i_fl);
        assert!(!r_fl.disclosure_compliant);

        let mut i_ca = ca_clean();
        i_ca.hoa_disclosure_summary_provided = false;
        let r_ca = check(&i_ca);
        assert!(r_ca.disclosure_compliant);
    }

    #[test]
    fn nv_uniquely_prohibits_post_acquisition_restriction_invariant() {
        let mut i_nv = nv_clean();
        i_nv.rental_prohibition_post_acquisition = true;
        i_nv.rental_prohibition_at_purchase = false;
        let r_nv = check(&i_nv);
        assert!(!r_nv.disclosure_compliant);

        let mut i_fl = fl_clean();
        i_fl.rental_prohibition_post_acquisition = true;
        i_fl.rental_prohibition_at_purchase = false;
        let r_fl = check(&i_fl);
        assert!(r_fl.disclosure_compliant);
    }

    #[test]
    fn pre_acquisition_rental_prohibition_invariant_across_ca_nv() {
        for regime in [Regime::California, Regime::Nevada] {
            let mut i = match regime {
                Regime::California => ca_clean(),
                Regime::Nevada => nv_clean(),
                _ => ca_clean(),
            };
            i.rental_prohibition_at_purchase = true;
            i.owner_leased_despite_pre_acquisition_prohibition = true;
            let r = check(&i);
            assert!(!r.disclosure_compliant);
        }
    }

    #[test]
    fn ca_redaction_right_uniquely_available_invariant() {
        let r_ca = check(&ca_clean());
        let r_fl = check(&fl_clean());
        let r_nv = check(&nv_clean());
        assert!(r_ca.tenant_redaction_right_available);
        assert!(!r_fl.tenant_redaction_right_available);
        assert!(!r_nv.tenant_redaction_right_available);
    }

    #[test]
    fn multiple_ca_violations_stack() {
        let mut i = ca_clean();
        i.tenant_info_provided_to_hoa = false;
        i.ca_sensitive_info_shared_without_redaction = true;
        i.rental_prohibition_at_purchase = true;
        i.owner_leased_despite_pre_acquisition_prohibition = true;
        let r = check(&i);
        assert_eq!(r.violations.len(), 3);
    }
}
