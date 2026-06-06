//! Tenant clothesline / drying-rack right under California
//! Civil Code § 1940.20 (eff. January 1, 2016) — landlord
//! may not impose an outright ban on a tenant's use of a
//! clothesline or drying rack in the tenant's PRIVATE AREA,
//! subject to six conditions. Trader-landlord critical for
//! CA rental owners managing multifamily properties: an
//! outright ban (lease clause or eviction notice) violates §
//! 1940.20. Distinct from siblings `tenant_solar_
//! installation` (rooftop solar panels), `ev_charger_
//! installation` (vehicle charging), `tenant_organizing`
//! (tenant association formation), and `flag_display_right`
//! (federal Freedom to Display the American Flag Act).
//!
//! **Two regimes**:
//!
//! **California — Cal. Civ. Code § 1940.20**:
//! - § 1940.20(a)(1) — "clothesline" includes cord / rope /
//!   wire from which laundered items may be hung to dry or
//!   air; a balcony / railing / awning / other part of a
//!   structure or building does NOT qualify as a clothesline.
//! - § 1940.20(a)(2) — "drying rack" means an apparatus from
//!   which laundered items may be hung; balcony / railing /
//!   awning / other part of structure or building does NOT
//!   qualify as a drying rack.
//! - § 1940.20(a)(3) — "private area" means outdoor area OR
//!   area in tenant's premises enclosed by wall or fence with
//!   access from a door of the premises.
//! - § 1940.20(b) — tenant may use clothesline or drying rack
//!   in private area if six conditions are met:
//!   1. Will not interfere with maintenance of rental property
//!   2. Will not create health or safety hazard
//!   3. Will not block doorways or interfere with walkways or
//!      utility service equipment
//!   4. Tenant seeks landlord's consent before affixing
//!      clothesline to building
//!   5. Use does not violate reasonable time or location
//!      restrictions imposed by landlord
//!   6. (effective Jan 1, 2016 — AB 1448 Stats. 2015 ch. 415)
//!
//! **Default — no specific tenant clothesline/drying right**.
//! Lease provisions controlling; common-law quiet-enjoyment
//! review. Solar-access/HOA right-to-dry laws exist in 20+
//! states for HOMEOWNERS but rental-specific protections are
//! rare outside CA.
//!
//! Citations: Cal. Civ. Code § 1940.20 (clothesline / drying-
//! rack tenant rights); Cal. Civ. Code § 4750.10 (HOA
//! companion right-to-dry); AB 1448, Stats. 2015 ch. 415.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    California,
    Default,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DryingDevice {
    /// Cord / rope / wire clothesline (qualifying device).
    Clothesline,
    /// Apparatus drying rack (qualifying device).
    DryingRack,
    /// Balcony railing used as drying surface (does NOT
    /// qualify as clothesline or drying rack under §
    /// 1940.20(a)).
    BalconyRailing,
    /// Awning used as drying surface (does NOT qualify).
    Awning,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InstallationLocation {
    /// Outdoor area on tenant premises.
    PrivateOutdoorArea,
    /// Indoor area enclosed by wall/fence with door access.
    EnclosedPrivateArea,
    /// Common area shared with other tenants (NOT private).
    CommonArea,
    /// Affixed to building structure (requires landlord
    /// consent under § 1940.20(b)(4)).
    AffixedToBuilding,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TenantClotheslineDryingRightInput {
    pub regime: Regime,
    pub device: DryingDevice,
    pub location: InstallationLocation,
    /// Whether the device interferes with property maintenance.
    pub interferes_with_property_maintenance: bool,
    /// Whether the device creates a health or safety hazard.
    pub creates_health_or_safety_hazard: bool,
    /// Whether the device blocks doorways or interferes with
    /// walkways or utility service equipment.
    pub blocks_doorways_or_walkways_or_utilities: bool,
    /// Whether tenant sought landlord consent (required if
    /// affixed to building under § 1940.20(b)(4)).
    pub tenant_sought_landlord_consent: bool,
    /// Whether use violates reasonable time or location
    /// restrictions imposed by landlord.
    pub violates_reasonable_time_location_restrictions: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TenantClotheslineDryingRightResult {
    pub right_protected: bool,
    pub qualifies_as_clothesline_or_rack: bool,
    pub is_private_area: bool,
    pub consent_required: bool,
    pub consent_satisfied: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &TenantClotheslineDryingRightInput) -> TenantClotheslineDryingRightResult {
    match input.regime {
        Regime::California => check_california(input),
        Regime::Default => check_default(input),
    }
}

fn check_california(
    input: &TenantClotheslineDryingRightInput,
) -> TenantClotheslineDryingRightResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "Cal. Civ. Code § 1940.20(a)(1) — 'clothesline' includes cord / rope / wire from which laundered items may be hung; balcony / railing / awning / other part of structure or building does NOT qualify"
            .to_string(),
        "Cal. Civ. Code § 1940.20(a)(3) — 'private area' means outdoor area OR area in tenant's premises enclosed by wall or fence with access from a door of the premises"
            .to_string(),
        "Cal. Civ. Code § 1940.20(b) six conditions: (1) no interference with maintenance; (2) no health/safety hazard; (3) no blocking doorways/walkways/utility equipment; (4) tenant consent if affixed to building; (5) reasonable time/location restrictions; (6) eff. January 1, 2016 (AB 1448 Stats. 2015 ch. 415)"
            .to_string(),
        "Cal. Civ. Code § 4750.10 — companion HOA right-to-dry provision; ban in CC&Rs is void"
            .to_string(),
    ];

    let qualifies = matches!(
        input.device,
        DryingDevice::Clothesline | DryingDevice::DryingRack
    );

    if !qualifies {
        violations.push(format!(
            "Cal. Civ. Code § 1940.20(a) — device does not qualify as clothesline or drying rack ({:?} excluded)",
            input.device
        ));
    }

    let is_private = matches!(
        input.location,
        InstallationLocation::PrivateOutdoorArea
            | InstallationLocation::EnclosedPrivateArea
            | InstallationLocation::AffixedToBuilding
    );

    if !is_private {
        violations.push(
            "Cal. Civ. Code § 1940.20(a)(3) — location not in tenant's private area (common areas excluded)".to_string(),
        );
    }

    if input.interferes_with_property_maintenance {
        violations.push(
            "Cal. Civ. Code § 1940.20(b)(1) — device interferes with maintenance of rental property".to_string(),
        );
    }

    if input.creates_health_or_safety_hazard {
        violations.push(
            "Cal. Civ. Code § 1940.20(b)(2) — device creates health or safety hazard".to_string(),
        );
    }

    if input.blocks_doorways_or_walkways_or_utilities {
        violations.push(
            "Cal. Civ. Code § 1940.20(b)(3) — device blocks doorways or interferes with walkways or utility service equipment".to_string(),
        );
    }

    let consent_required = matches!(input.location, InstallationLocation::AffixedToBuilding);
    let consent_satisfied = !consent_required || input.tenant_sought_landlord_consent;

    if consent_required && !input.tenant_sought_landlord_consent {
        violations.push(
            "Cal. Civ. Code § 1940.20(b)(4) — tenant did not seek landlord's consent before affixing clothesline to building".to_string(),
        );
    }

    if input.violates_reasonable_time_location_restrictions {
        violations.push(
            "Cal. Civ. Code § 1940.20(b)(5) — use violates reasonable time or location restrictions imposed by landlord".to_string(),
        );
    }

    TenantClotheslineDryingRightResult {
        right_protected: violations.is_empty() && qualifies && is_private,
        qualifies_as_clothesline_or_rack: qualifies,
        is_private_area: is_private,
        consent_required,
        consent_satisfied,
        violations,
        citation: "Cal. Civ. Code §§ 1940.20, 4750.10; AB 1448 Stats. 2015 ch. 415",
        notes,
    }
}

fn check_default(_input: &TenantClotheslineDryingRightInput) -> TenantClotheslineDryingRightResult {
    let notes: Vec<String> = vec![
        "default rule — no specific tenant clothesline / drying-rack right under state statute; lease provisions controlling; common-law quiet-enjoyment review may apply"
            .to_string(),
        "default rule — solar-access / HOA right-to-dry laws exist in 20+ states for HOMEOWNERS (FL, CO, MD, ME, NV, etc.) but rental-specific protections are rare outside CA"
            .to_string(),
    ];

    TenantClotheslineDryingRightResult {
        right_protected: false,
        qualifies_as_clothesline_or_rack: false,
        is_private_area: false,
        consent_required: false,
        consent_satisfied: true,
        violations: Vec::new(),
        citation: "lease provisions + common-law quiet enjoyment",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ca_protected() -> TenantClotheslineDryingRightInput {
        TenantClotheslineDryingRightInput {
            regime: Regime::California,
            device: DryingDevice::Clothesline,
            location: InstallationLocation::PrivateOutdoorArea,
            interferes_with_property_maintenance: false,
            creates_health_or_safety_hazard: false,
            blocks_doorways_or_walkways_or_utilities: false,
            tenant_sought_landlord_consent: true,
            violates_reasonable_time_location_restrictions: false,
        }
    }

    fn default_base() -> TenantClotheslineDryingRightInput {
        let mut i = ca_protected();
        i.regime = Regime::Default;
        i
    }

    #[test]
    fn ca_clothesline_private_outdoor_protected() {
        let r = check(&ca_protected());
        assert!(r.right_protected);
        assert!(r.qualifies_as_clothesline_or_rack);
        assert!(r.is_private_area);
        assert!(!r.consent_required);
        assert!(r.consent_satisfied);
        assert!(r.violations.is_empty());
    }

    #[test]
    fn ca_drying_rack_protected() {
        let mut i = ca_protected();
        i.device = DryingDevice::DryingRack;
        let r = check(&i);
        assert!(r.right_protected);
    }

    #[test]
    fn ca_balcony_railing_does_not_qualify() {
        let mut i = ca_protected();
        i.device = DryingDevice::BalconyRailing;
        let r = check(&i);
        assert!(!r.right_protected);
        assert!(!r.qualifies_as_clothesline_or_rack);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 1940.20(a)") && v.contains("does not qualify")));
    }

    #[test]
    fn ca_awning_does_not_qualify() {
        let mut i = ca_protected();
        i.device = DryingDevice::Awning;
        let r = check(&i);
        assert!(!r.right_protected);
        assert!(!r.qualifies_as_clothesline_or_rack);
    }

    #[test]
    fn ca_common_area_not_private() {
        let mut i = ca_protected();
        i.location = InstallationLocation::CommonArea;
        let r = check(&i);
        assert!(!r.is_private_area);
        assert!(!r.right_protected);
        assert!(r.violations.iter().any(|v| v.contains("§ 1940.20(a)(3)")));
    }

    #[test]
    fn ca_enclosed_private_area_protected() {
        let mut i = ca_protected();
        i.location = InstallationLocation::EnclosedPrivateArea;
        let r = check(&i);
        assert!(r.right_protected);
        assert!(r.is_private_area);
    }

    #[test]
    fn ca_affixed_to_building_requires_consent() {
        let mut i = ca_protected();
        i.location = InstallationLocation::AffixedToBuilding;
        i.tenant_sought_landlord_consent = false;
        let r = check(&i);
        assert!(r.consent_required);
        assert!(!r.consent_satisfied);
        assert!(!r.right_protected);
        assert!(r.violations.iter().any(|v| v.contains("§ 1940.20(b)(4)")));
    }

    #[test]
    fn ca_affixed_to_building_with_consent_protected() {
        let mut i = ca_protected();
        i.location = InstallationLocation::AffixedToBuilding;
        i.tenant_sought_landlord_consent = true;
        let r = check(&i);
        assert!(r.consent_required);
        assert!(r.consent_satisfied);
        assert!(r.right_protected);
    }

    #[test]
    fn ca_property_maintenance_interference_violates() {
        let mut i = ca_protected();
        i.interferes_with_property_maintenance = true;
        let r = check(&i);
        assert!(!r.right_protected);
        assert!(r.violations.iter().any(|v| v.contains("§ 1940.20(b)(1)")));
    }

    #[test]
    fn ca_health_safety_hazard_violates() {
        let mut i = ca_protected();
        i.creates_health_or_safety_hazard = true;
        let r = check(&i);
        assert!(!r.right_protected);
        assert!(r.violations.iter().any(|v| v.contains("§ 1940.20(b)(2)")));
    }

    #[test]
    fn ca_blocks_doorway_violates() {
        let mut i = ca_protected();
        i.blocks_doorways_or_walkways_or_utilities = true;
        let r = check(&i);
        assert!(!r.right_protected);
        assert!(r.violations.iter().any(|v| v.contains("§ 1940.20(b)(3)")));
    }

    #[test]
    fn ca_reasonable_restriction_violation() {
        let mut i = ca_protected();
        i.violates_reasonable_time_location_restrictions = true;
        let r = check(&i);
        assert!(!r.right_protected);
        assert!(r.violations.iter().any(|v| v.contains("§ 1940.20(b)(5)")));
    }

    #[test]
    fn ca_citation_pins_authorities() {
        let r = check(&ca_protected());
        assert!(r.citation.contains("§§ 1940.20"));
        assert!(r.citation.contains("4750.10"));
        assert!(r.citation.contains("AB 1448"));
        assert!(r.citation.contains("Stats. 2015 ch. 415"));
    }

    #[test]
    fn ca_note_pins_definitions() {
        let r = check(&ca_protected());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 1940.20(a)(1)") && n.contains("cord")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 1940.20(a)(3)") && n.contains("outdoor area")));
    }

    #[test]
    fn ca_note_pins_six_conditions() {
        let r = check(&ca_protected());
        assert!(r.notes.iter().any(|n| n.contains("§ 1940.20(b)")
            && n.contains("six conditions")
            && n.contains("January 1, 2016")
            && n.contains("AB 1448")));
    }

    #[test]
    fn ca_note_pins_hoa_companion() {
        let r = check(&ca_protected());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 4750.10") && n.contains("HOA")));
    }

    #[test]
    fn default_no_protection() {
        let r = check(&default_base());
        assert!(!r.right_protected);
    }

    #[test]
    fn default_citation_pins_lease_and_common_law() {
        let r = check(&default_base());
        assert!(r.citation.contains("lease provisions"));
        assert!(r.citation.contains("common-law quiet enjoyment"));
    }

    #[test]
    fn default_note_pins_homeowner_state_laws() {
        let r = check(&default_base());
        assert!(r.notes.iter().any(|n| n.contains("FL")
            && n.contains("CO")
            && n.contains("MD")
            && n.contains("ME")
            && n.contains("NV")
            && n.contains("HOMEOWNERS")));
    }

    #[test]
    fn two_regimes_routed_correctly() {
        for regime in [Regime::California, Regime::Default] {
            let mut i = ca_protected();
            i.regime = regime;
            let r = check(&i);
            assert!(!r.citation.is_empty());
        }
    }

    #[test]
    fn ca_uniquely_protects_clothesline_invariant() {
        let r_ca = check(&ca_protected());
        assert!(r_ca.right_protected);

        let r_default = check(&default_base());
        assert!(!r_default.right_protected);
    }

    #[test]
    fn ca_five_violation_stack_when_all_5_conditions_failed() {
        let mut i = ca_protected();
        i.interferes_with_property_maintenance = true;
        i.creates_health_or_safety_hazard = true;
        i.blocks_doorways_or_walkways_or_utilities = true;
        i.location = InstallationLocation::AffixedToBuilding;
        i.tenant_sought_landlord_consent = false;
        i.violates_reasonable_time_location_restrictions = true;
        let r = check(&i);
        assert!(!r.right_protected);
        assert_eq!(r.violations.len(), 5);
    }

    #[test]
    fn device_truth_table() {
        for (device, exp_qualifies) in [
            (DryingDevice::Clothesline, true),
            (DryingDevice::DryingRack, true),
            (DryingDevice::BalconyRailing, false),
            (DryingDevice::Awning, false),
        ] {
            let mut i = ca_protected();
            i.device = device;
            let r = check(&i);
            assert_eq!(r.qualifies_as_clothesline_or_rack, exp_qualifies);
        }
    }

    #[test]
    fn location_truth_table() {
        for (loc, exp_private) in [
            (InstallationLocation::PrivateOutdoorArea, true),
            (InstallationLocation::EnclosedPrivateArea, true),
            (InstallationLocation::AffixedToBuilding, true),
            (InstallationLocation::CommonArea, false),
        ] {
            let mut i = ca_protected();
            i.location = loc;
            i.tenant_sought_landlord_consent = true;
            let r = check(&i);
            assert_eq!(r.is_private_area, exp_private);
        }
    }

    #[test]
    fn consent_only_required_when_affixed_to_building_invariant() {
        for (loc, exp_consent_required) in [
            (InstallationLocation::PrivateOutdoorArea, false),
            (InstallationLocation::EnclosedPrivateArea, false),
            (InstallationLocation::AffixedToBuilding, true),
            (InstallationLocation::CommonArea, false),
        ] {
            let mut i = ca_protected();
            i.location = loc;
            let r = check(&i);
            assert_eq!(r.consent_required, exp_consent_required);
        }
    }

    #[test]
    fn ca_balcony_railing_in_private_area_still_does_not_qualify() {
        let mut i = ca_protected();
        i.device = DryingDevice::BalconyRailing;
        i.location = InstallationLocation::PrivateOutdoorArea;
        let r = check(&i);
        assert!(!r.right_protected);
        assert!(r.is_private_area);
        assert!(!r.qualifies_as_clothesline_or_rack);
    }

    #[test]
    fn ca_clothesline_in_common_area_violates_private_area_gate() {
        let mut i = ca_protected();
        i.device = DryingDevice::Clothesline;
        i.location = InstallationLocation::CommonArea;
        let r = check(&i);
        assert!(!r.right_protected);
        assert!(r.qualifies_as_clothesline_or_rack);
        assert!(!r.is_private_area);
    }

    #[test]
    fn default_no_consent_required() {
        let r = check(&default_base());
        assert!(!r.consent_required);
        assert!(r.consent_satisfied);
    }
}
