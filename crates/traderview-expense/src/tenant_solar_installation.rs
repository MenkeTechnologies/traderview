//! State tenant right-to-install solar-energy-system compliance
//! check. Emerging area parallel to existing `ev_charger_installation`
//! (tenant right-to-charge electric-vehicle).
//!
//! Most state solar-rights laws cover homeowners + HOAs + condos
//! but not rental tenants directly. The push to extend protection
//! to renters is recent (2024–2026), with portable / plug-in solar
//! systems leading the way because they require no permanent
//! installation and minimal landlord coordination.
//!
//! Four regimes:
//!
//!   - **California** — Cal. Civ. Code § 714 (Solar Rights Act —
//!     restrictions on solar energy systems are "void and
//!     unenforceable") + § 714.1 (HOA common-interest development
//!     restrictions on rooftop solar). Tenant rental coverage is
//!     limited; § 714 applies most directly when tenant has
//!     exclusive use or control of an installation area (similar
//!     to OTARD's tenant-exclusive-use scope).
//!
//!   - **Colorado** — Colorado HB22-1020 (Customer Right To Use
//!     Energy, 2022) + 2026 House-passed plug-in solar legalization
//!     bill extending portable solar to renters and multifamily
//!     residents. Establishes regulatory framework for portable
//!     arrays without landlord prohibition.
//!
//!   - **NewJersey** — N.J.S.A. 45:22A-48.2 limits HOA authority
//!     over solar collectors on certain roofs (Planned Real Estate
//!     Development Full Disclosure Act). Tenant rental coverage
//!     limited; typically requires lease consent.
//!
//!   - **Default** — most other states have solar-rights laws but
//!     they apply to homeowners + HOAs only; tenant rentals
//!     require lease-based or landlord-consent installation.
//!
//! Three installation-type axes apply across regimes:
//!
//!   - **Plug-in portable** (balcony or window-mounted plug-in
//!     units; no permanent installation) — most permissive across
//!     regimes; Colorado specifically legalizes this for renters.
//!
//!   - **Roof-mounted** (permanent rooftop installation) — typical
//!     HOA / homeowner scope; requires landlord consent in rental
//!     context across most regimes.
//!
//!   - **Ground-mounted** (yard installation) — typically requires
//!     landlord consent for any tenant rental.
//!
//! Universal safety thresholds:
//!   - Installation must meet safety / electrical code.
//!   - Installation must not damage landlord property without
//!     restoration agreement.
//!
//! Citations: Cal. Civ. Code § 714 (Solar Rights Act — restrictions
//! void and unenforceable); Cal. Civ. Code § 714.1 (HOA common-
//! interest development rooftop solar); Cal. Civ. Code § 4600 +
//! § 4746 (CID common-area solar installation by member-owners);
//! Colorado HB22-1020 (Customer Right To Use Energy); Colorado
//! 2026 plug-in solar legalization for renters (pending);
//! N.J.S.A. 45:22A-48.2 (NJ HOA solar-collector limitation under
//! Planned Real Estate Development Full Disclosure Act); Mass.
//! G.L. c. 184 § 32–34 (Massachusetts solar easement framework).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    California,
    Colorado,
    NewJersey,
    Default,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InstallationType {
    /// Portable plug-in solar (balcony / window mount); no
    /// permanent installation.
    PlugInPortable,
    /// Permanent roof-mounted installation.
    RoofMounted,
    /// Ground-mounted yard installation.
    GroundMounted,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub regime: Regime,
    pub installation_type: InstallationType,
    /// Whether the tenant has exclusive use or control of the
    /// installation area (balcony, patio, exclusive-use roof
    /// section). Important for CA § 714 scope.
    pub tenant_owns_unit_or_has_exclusive_use_area: bool,
    /// Whether the landlord has consented to the installation.
    pub landlord_consent_obtained: bool,
    /// Whether the installation meets electrical / safety code.
    pub installation_meets_safety_code: bool,
    /// Whether the installation will not damage landlord property
    /// (or has a restoration agreement).
    pub installation_does_not_damage_property: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CheckResult {
    /// True if the regime provides statutory tenant protection
    /// for the installation type.
    pub statutory_tenant_protection: bool,
    /// True if the landlord may statutorily prohibit the
    /// installation (i.e., no tenant protection or tenant fails
    /// universal threshold).
    pub landlord_can_prohibit: bool,
    /// True if installation is compliant under the regime (statutory
    /// protection + universal safety thresholds met OR landlord
    /// consent obtained).
    pub installation_permitted: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Input) -> CheckResult {
    let mut violations: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    // Universal safety thresholds — failure of either prevents
    // protection.
    if !input.installation_meets_safety_code {
        violations.push(
            "Installation must meet electrical and safety code; deficient installation is not \
             protected regardless of regime."
                .to_string(),
        );
    }
    if !input.installation_does_not_damage_property {
        violations.push(
            "Installation must not damage landlord property without restoration agreement; \
             tenant must restore or pay damages."
                .to_string(),
        );
    }

    let universal_safety_ok =
        input.installation_meets_safety_code && input.installation_does_not_damage_property;

    let (statutory_protection, citation): (bool, &'static str) = match input.regime {
        Regime::California => {
            // CA § 714 applies most directly to plug-in portable
            // OR where tenant has exclusive-use area for roof
            // installation.
            let protected = matches!(input.installation_type, InstallationType::PlugInPortable)
                || (matches!(input.installation_type, InstallationType::RoofMounted)
                    && input.tenant_owns_unit_or_has_exclusive_use_area);
            (
                protected,
                "Cal. Civ. Code § 714 (Solar Rights Act — restrictions on solar energy systems \
                 are void and unenforceable); § 714.1 (HOA common-interest development rooftop \
                 solar); § 4600 + § 4746 (CID common-area solar installation by member-owners)",
            )
        }
        Regime::Colorado => {
            // Colorado 2026 bill specifically legalizes plug-in
            // portable solar for renters. HB22-1020 covers customer
            // right to use energy more broadly.
            let protected = matches!(input.installation_type, InstallationType::PlugInPortable);
            (
                protected,
                "Colorado HB22-1020 (Customer Right To Use Energy 2022); Colorado 2026 plug-in \
                 solar legalization bill extending portable solar to renters and multifamily \
                 residents — establishes regulatory framework for portable arrays without \
                 landlord prohibition",
            )
        }
        Regime::NewJersey => (
            // NJ § 45:22A-48.2 limits HOA restrictions; tenant
            // rental coverage limited. Plug-in portable typically
            // not statutorily protected for renters.
            false,
            "N.J.S.A. 45:22A-48.2 (Planned Real Estate Development Full Disclosure Act — limits \
             HOA authority over solar collectors on certain roofs); tenant rental coverage \
             limited; typically requires lease consent",
        ),
        Regime::Default => (
            false,
            "Most states with solar-rights laws cover homeowners + HOAs + condos; tenant \
             rentals require lease-based or landlord-consent installation",
        ),
    };

    let landlord_can_prohibit = !statutory_protection;

    if landlord_can_prohibit && !input.landlord_consent_obtained {
        violations.push(format!(
            "{:?} — installation type {:?} not statutorily protected for tenants in this \
             regime; landlord consent required and has not been obtained.",
            input.regime, input.installation_type,
        ));
    }

    let installation_permitted =
        universal_safety_ok && (statutory_protection || input.landlord_consent_obtained);

    // Installation-type notes.
    match input.installation_type {
        InstallationType::PlugInPortable => {
            notes.push(
                "Plug-in portable solar (balcony / window mount) is the most permissive \
                 installation type across regimes; no permanent installation required."
                    .to_string(),
            );
        }
        InstallationType::RoofMounted => {
            notes.push(
                "Permanent roof-mounted installation typically requires landlord consent in \
                 rental context across most regimes. CA § 714 may protect where tenant has \
                 exclusive use or control of installation area."
                    .to_string(),
            );
        }
        InstallationType::GroundMounted => {
            notes.push(
                "Ground-mounted yard installation typically requires landlord consent for any \
                 tenant rental."
                    .to_string(),
            );
        }
    }

    notes.push(
        "Sibling module ev_charger_installation addresses tenant right-to-charge for electric \
         vehicles — parallel emerging area with similar consent + safety + property-damage \
         analysis."
            .to_string(),
    );

    CheckResult {
        statutory_tenant_protection: statutory_protection,
        landlord_can_prohibit,
        installation_permitted,
        violations,
        citation,
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base(regime: Regime, installation: InstallationType) -> Input {
        Input {
            regime,
            installation_type: installation,
            tenant_owns_unit_or_has_exclusive_use_area: false,
            landlord_consent_obtained: false,
            installation_meets_safety_code: true,
            installation_does_not_damage_property: true,
        }
    }

    // ── California § 714 Solar Rights Act ──────────────────────

    #[test]
    fn california_plug_in_portable_statutorily_protected() {
        let r = check(&base(Regime::California, InstallationType::PlugInPortable));
        assert!(r.statutory_tenant_protection);
        assert!(!r.landlord_can_prohibit);
        assert!(r.installation_permitted);
        assert!(r.citation.contains("§ 714"));
        assert!(r.citation.contains("Solar Rights Act"));
    }

    #[test]
    fn california_roof_mounted_with_exclusive_use_protected() {
        let mut i = base(Regime::California, InstallationType::RoofMounted);
        i.tenant_owns_unit_or_has_exclusive_use_area = true;
        let r = check(&i);
        assert!(r.statutory_tenant_protection);
        assert!(r.installation_permitted);
    }

    #[test]
    fn california_roof_mounted_without_exclusive_use_not_protected() {
        let mut i = base(Regime::California, InstallationType::RoofMounted);
        i.tenant_owns_unit_or_has_exclusive_use_area = false;
        let r = check(&i);
        assert!(!r.statutory_tenant_protection);
        assert!(r.landlord_can_prohibit);
        // Without consent, no installation.
        assert!(!r.installation_permitted);
    }

    #[test]
    fn california_ground_mounted_requires_landlord_consent() {
        let mut i = base(Regime::California, InstallationType::GroundMounted);
        i.landlord_consent_obtained = false;
        let r = check(&i);
        assert!(!r.statutory_tenant_protection);
        assert!(!r.installation_permitted);
    }

    // ── Colorado plug-in solar legalization ────────────────────

    #[test]
    fn colorado_plug_in_portable_protected_for_renters() {
        let r = check(&base(Regime::Colorado, InstallationType::PlugInPortable));
        assert!(r.statutory_tenant_protection);
        assert!(r.installation_permitted);
        assert!(r.citation.contains("Colorado HB22-1020"));
        assert!(r.citation.contains("plug-in solar"));
    }

    #[test]
    fn colorado_roof_mounted_not_statutorily_protected_for_renters() {
        let r = check(&base(Regime::Colorado, InstallationType::RoofMounted));
        assert!(!r.statutory_tenant_protection);
    }

    #[test]
    fn colorado_ground_mounted_not_protected() {
        let r = check(&base(Regime::Colorado, InstallationType::GroundMounted));
        assert!(!r.statutory_tenant_protection);
    }

    // ── New Jersey § 45:22A-48.2 ────────────────────────────────

    #[test]
    fn new_jersey_hoa_focus_no_tenant_rental_protection() {
        let r = check(&base(Regime::NewJersey, InstallationType::PlugInPortable));
        assert!(!r.statutory_tenant_protection);
        assert!(r.citation.contains("45:22A-48.2"));
    }

    // ── Default ─────────────────────────────────────────────────

    #[test]
    fn default_no_tenant_rental_protection() {
        let r = check(&base(Regime::Default, InstallationType::PlugInPortable));
        assert!(!r.statutory_tenant_protection);
        assert!(r.landlord_can_prohibit);
        assert!(r.citation.contains("Most states"));
    }

    // ── Universal safety thresholds ────────────────────────────

    #[test]
    fn unsafe_installation_never_permitted_invariant() {
        // Across all regimes, unsafe installation is never permitted
        // — even where statutory protection would otherwise apply.
        for &regime in &[
            Regime::California,
            Regime::Colorado,
            Regime::NewJersey,
            Regime::Default,
        ] {
            for installation in [
                InstallationType::PlugInPortable,
                InstallationType::RoofMounted,
                InstallationType::GroundMounted,
            ] {
                let mut i = base(regime, installation);
                i.installation_meets_safety_code = false;
                i.tenant_owns_unit_or_has_exclusive_use_area = true;
                i.landlord_consent_obtained = true;
                let r = check(&i);
                assert!(
                    !r.installation_permitted,
                    "{:?} {:?}: unsafe installation must not be permitted",
                    regime, installation,
                );
                assert!(
                    r.violations.iter().any(|v| v.contains("safety code")),
                    "{:?} {:?}: safety-code violation must appear",
                    regime,
                    installation,
                );
            }
        }
    }

    #[test]
    fn property_damage_never_permitted_invariant() {
        for &regime in &[
            Regime::California,
            Regime::Colorado,
            Regime::NewJersey,
            Regime::Default,
        ] {
            let mut i = base(regime, InstallationType::PlugInPortable);
            i.installation_does_not_damage_property = false;
            i.landlord_consent_obtained = true;
            let r = check(&i);
            assert!(
                !r.installation_permitted,
                "{:?}: property-damaging installation must not be permitted",
                regime,
            );
            assert!(
                r.violations
                    .iter()
                    .any(|v| v.contains("damage landlord property")),
                "{:?}: property-damage violation must appear",
                regime,
            );
        }
    }

    // ── Landlord consent path ──────────────────────────────────

    #[test]
    fn landlord_consent_permits_otherwise_unprotected_installation() {
        let mut i = base(Regime::Default, InstallationType::RoofMounted);
        i.landlord_consent_obtained = true;
        let r = check(&i);
        assert!(r.installation_permitted);
        assert!(r.violations.is_empty());
    }

    #[test]
    fn missing_consent_violation_when_not_statutorily_protected() {
        let mut i = base(Regime::Default, InstallationType::PlugInPortable);
        i.landlord_consent_obtained = false;
        let r = check(&i);
        assert!(!r.installation_permitted);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("landlord consent required")));
    }

    // ── Regression-critical multi-regime invariants ────────────

    #[test]
    fn only_california_and_colorado_protect_plug_in_portable_invariant() {
        for &regime in &[Regime::California, Regime::Colorado] {
            let r = check(&base(regime, InstallationType::PlugInPortable));
            assert!(
                r.statutory_tenant_protection,
                "{:?}: must protect plug-in portable",
                regime,
            );
        }
        for &regime in &[Regime::NewJersey, Regime::Default] {
            let r = check(&base(regime, InstallationType::PlugInPortable));
            assert!(
                !r.statutory_tenant_protection,
                "{:?}: must not protect plug-in portable",
                regime,
            );
        }
    }

    #[test]
    fn only_california_protects_roof_mounted_with_exclusive_use_invariant() {
        let mut ca = base(Regime::California, InstallationType::RoofMounted);
        ca.tenant_owns_unit_or_has_exclusive_use_area = true;
        assert!(check(&ca).statutory_tenant_protection);

        for &regime in &[Regime::Colorado, Regime::NewJersey, Regime::Default] {
            let mut i = base(regime, InstallationType::RoofMounted);
            i.tenant_owns_unit_or_has_exclusive_use_area = true;
            assert!(
                !check(&i).statutory_tenant_protection,
                "{:?}: must not protect roof-mounted with exclusive use",
                regime,
            );
        }
    }

    #[test]
    fn ground_mounted_never_statutorily_protected_invariant() {
        for &regime in &[
            Regime::California,
            Regime::Colorado,
            Regime::NewJersey,
            Regime::Default,
        ] {
            let r = check(&base(regime, InstallationType::GroundMounted));
            assert!(
                !r.statutory_tenant_protection,
                "{:?}: ground-mounted must not be statutorily protected",
                regime,
            );
        }
    }

    #[test]
    fn installation_permitted_iff_safety_and_either_protected_or_consented_invariant() {
        // 8-cell truth table: (safety_ok, statutory_protected,
        // consent) → permitted.
        for safety_ok in [false, true] {
            for protected in [false, true] {
                for consent in [false, true] {
                    let mut i = base(Regime::Default, InstallationType::PlugInPortable);
                    i.installation_meets_safety_code = safety_ok;
                    i.installation_does_not_damage_property = safety_ok;
                    i.landlord_consent_obtained = consent;
                    // For protected=true case, use CA with plug-in
                    // which is statutorily protected.
                    if protected {
                        i.regime = Regime::California;
                    }
                    let r = check(&i);
                    let expected_permitted = safety_ok && (protected || consent);
                    assert_eq!(
                        r.installation_permitted, expected_permitted,
                        "safety={} protected={} consent={} expected={}",
                        safety_ok, protected, consent, expected_permitted,
                    );
                }
            }
        }
    }

    #[test]
    fn citation_pins_authority_per_regime() {
        assert!(
            check(&base(Regime::California, InstallationType::PlugInPortable))
                .citation
                .contains("§ 714")
        );
        assert!(
            check(&base(Regime::Colorado, InstallationType::PlugInPortable))
                .citation
                .contains("HB22-1020")
        );
        assert!(
            check(&base(Regime::NewJersey, InstallationType::PlugInPortable))
                .citation
                .contains("45:22A-48.2")
        );
        assert!(
            check(&base(Regime::Default, InstallationType::PlugInPortable))
                .citation
                .contains("Most states")
        );
    }

    #[test]
    fn sibling_module_note_present_across_all_combos() {
        for &regime in &[
            Regime::California,
            Regime::Colorado,
            Regime::NewJersey,
            Regime::Default,
        ] {
            for installation in [
                InstallationType::PlugInPortable,
                InstallationType::RoofMounted,
                InstallationType::GroundMounted,
            ] {
                let r = check(&base(regime, installation));
                assert!(
                    r.notes
                        .iter()
                        .any(|n| n.contains("ev_charger_installation")
                            && n.contains("right-to-charge")),
                    "{:?} {:?}: sibling-module note must be present",
                    regime,
                    installation,
                );
            }
        }
    }
}
