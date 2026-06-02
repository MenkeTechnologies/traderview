//! Tenant fire safety plan + emergency preparedness notice
//! disclosure compliance — when must a trader-landlord post
//! and distribute fire safety plans + emergency preparedness
//! notices to tenants of multi-unit residential properties?
//! Trader-landlord critical for any multifamily building
//! owner: failure to post / distribute required HPD-mandated
//! signage exposes owner to HPD violations + civil penalties
//! + tenant breach-of-habitability claims.
//!
//! Distinct from siblings `detector_requirements` (smoke + CO
//! detector hardware standards), `fire_sprinkler_disclosure`
//! (fire-suppression system disclosure), `water_heater_
//! earthquake_strap` (CA § 19211 seismic), and `landlord_
//! emergency_entry_notice` (post-emergency entry notice
//! requirements).
//!
//! **Three regimes**:
//!
//! **New York City — HMC § 27-2046 + Article 11 + HPD
//! Required Signs**:
//! - HPD Fire Safety Plan posted at building entrance.
//! - Fire Safety Plan mailed to tenants ANNUALLY.
//! - Emergency Preparedness Notice on INSIDE of all
//!   apartment entrance doors AND in lobby/common area (3+
//!   apartments).
//! - § 27-2046: smoke-detector duties for Class A and Class
//!   B multiple dwellings.
//! - Smoke Detecting Devices Notice at/near mailboxes (Class
//!   A multiple dwellings).
//! - Carbon Monoxide Detector requirement notice in common
//!   area.
//!
//! **California — Cal. Health & Safety Code § 13145 + § 17926
//! + HCD requirements**:
//! - Fire alarm system disclosure required for multi-unit
//!   buildings.
//! - Smoke detector requirements (1991 Smoke Detector Act).
//! - Carbon monoxide detector requirements (2010 Carbon
//!   Monoxide Poisoning Prevention Act).
//! - Emergency evacuation plan required for buildings with
//!   specific occupancy thresholds.
//!
//! **Default — IBC / IFC + state-specific habitability**:
//! - International Fire Code (IFC) § 403.10 Fire Safety
//!   Plans required in covered occupancies.
//! - State-specific habitability standards may impose
//!   additional requirements.
//!
//! Citations: NYC HMC § 27-2046 (Article 11 Protective
//! Devices and Fire Protection); NYC HPD Required Signs
//! Checklist; Cal. Health & Safety Code §§ 13145, 17926;
//! International Fire Code (IFC) § 403.10.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    NewYorkCity,
    California,
    Default,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TenantFireSafetyPlanDisclosureInput {
    pub regime: Regime,
    /// Number of apartment units in the building.
    pub apartment_unit_count: u32,
    /// Whether HPD Fire Safety Plan is posted at building
    /// entrance (NYC).
    pub fire_safety_plan_posted_at_entrance: bool,
    /// Whether Fire Safety Plan is mailed to tenants annually
    /// (NYC).
    pub fire_safety_plan_mailed_annually: bool,
    /// Whether Emergency Preparedness Notice is posted on
    /// inside of all apartment entrance doors (NYC).
    pub emergency_notice_on_apartment_doors: bool,
    /// Whether Emergency Preparedness Notice is posted in
    /// lobby or common area (NYC).
    pub emergency_notice_in_lobby: bool,
    /// Whether smoke detector notice is posted at/near
    /// mailboxes (NYC Class A multiple dwellings).
    pub smoke_detector_notice_at_mailboxes: bool,
    /// Whether CO detector notice is posted in common area
    /// (NYC).
    pub co_detector_notice_in_common_area: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TenantFireSafetyPlanDisclosureResult {
    pub compliant: bool,
    pub building_in_scope: bool,
    pub annual_mailing_required: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &TenantFireSafetyPlanDisclosureInput) -> TenantFireSafetyPlanDisclosureResult {
    match input.regime {
        Regime::NewYorkCity => check_nyc(input),
        Regime::California => check_ca(input),
        Regime::Default => check_default(input),
    }
}

fn check_nyc(
    input: &TenantFireSafetyPlanDisclosureInput,
) -> TenantFireSafetyPlanDisclosureResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "NYC HMC § 27-2046 (Article 11 Protective Devices and Fire Protection) — smoke detector duties for Class A and Class B multiple dwellings; specific HPD-approved notice format required at or near mailboxes"
            .to_string(),
        "NYC HPD Required Signs — HPD Fire Safety Plan must be posted at building entrance AND mailed to tenants ANNUALLY; Emergency Preparedness Notice on INSIDE of all apartment entrance doors AND in lobby/common area for 3+ apartment buildings"
            .to_string(),
        "NYC HMC + Article 11 — Carbon Monoxide Detector requirement notice must be posted in common area; Smoke Detecting Devices Notice at/near mailboxes for Class A multiple dwellings"
            .to_string(),
    ];

    let in_scope = input.apartment_unit_count >= 3;

    if in_scope {
        if !input.fire_safety_plan_posted_at_entrance {
            violations.push(
                "NYC HPD — HPD Fire Safety Plan must be posted at building entrance".to_string(),
            );
        }
        if !input.fire_safety_plan_mailed_annually {
            violations.push(
                "NYC HPD — Fire Safety Plan must be mailed to tenants ANNUALLY".to_string(),
            );
        }
        if !input.emergency_notice_on_apartment_doors {
            violations.push(
                "NYC HPD — Emergency Preparedness Notice must be posted on INSIDE of all apartment entrance doors (3+ apartment buildings)".to_string(),
            );
        }
        if !input.emergency_notice_in_lobby {
            violations.push(
                "NYC HPD — Emergency Preparedness Notice must be posted in lobby or common area (3+ apartment buildings)".to_string(),
            );
        }
        if !input.smoke_detector_notice_at_mailboxes {
            violations.push(
                "NYC HMC § 27-2046 — Smoke Detecting Devices Notice must be posted at or near mailboxes (Class A multiple dwellings)".to_string(),
            );
        }
        if !input.co_detector_notice_in_common_area {
            violations.push(
                "NYC HMC + Article 11 — Carbon Monoxide Detector requirement notice must be posted in common area".to_string(),
            );
        }
    }

    TenantFireSafetyPlanDisclosureResult {
        compliant: violations.is_empty(),
        building_in_scope: in_scope,
        annual_mailing_required: in_scope,
        violations,
        citation: "NYC HMC § 27-2046 (Article 11 Protective Devices and Fire Protection); NYC HPD Required Signs Checklist",
        notes,
    }
}

fn check_ca(input: &TenantFireSafetyPlanDisclosureInput) -> TenantFireSafetyPlanDisclosureResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "Cal. Health & Safety Code § 13145 — fire alarm system disclosure required for multi-unit buildings"
            .to_string(),
        "Cal. Health & Safety Code § 17926 + 2010 Carbon Monoxide Poisoning Prevention Act + 1991 Smoke Detector Act — smoke and carbon monoxide detector requirements; emergency evacuation plan for buildings with specific occupancy thresholds"
            .to_string(),
    ];

    let in_scope = input.apartment_unit_count >= 3;

    if in_scope
        && !input.fire_safety_plan_posted_at_entrance
        && !input.emergency_notice_in_lobby
    {
        violations.push(
            "Cal. Health & Safety Code § 13145 — fire alarm system disclosure required at building entrance or in lobby/common area".to_string(),
        );
    }

    TenantFireSafetyPlanDisclosureResult {
        compliant: violations.is_empty(),
        building_in_scope: in_scope,
        annual_mailing_required: false,
        violations,
        citation: "Cal. Health & Safety Code §§ 13145, 17926; 2010 Carbon Monoxide Poisoning Prevention Act",
        notes,
    }
}

fn check_default(
    input: &TenantFireSafetyPlanDisclosureInput,
) -> TenantFireSafetyPlanDisclosureResult {
    let notes: Vec<String> = vec![
        "default rule — International Fire Code (IFC) § 403.10 Fire Safety Plans required in covered occupancies (typically high-rise + larger multifamily); state-specific habitability standards may impose additional requirements"
            .to_string(),
        "default rule — many states have local ordinances (e.g., Chicago RLTO + Boston Inspectional Services) imposing fire safety plan disclosure; verify local jurisdiction adoption"
            .to_string(),
    ];

    let in_scope = input.apartment_unit_count >= 4;

    TenantFireSafetyPlanDisclosureResult {
        compliant: true,
        building_in_scope: in_scope,
        annual_mailing_required: false,
        violations: Vec::new(),
        citation: "International Fire Code (IFC) § 403.10; state-specific habitability standards",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nyc_fully_compliant() -> TenantFireSafetyPlanDisclosureInput {
        TenantFireSafetyPlanDisclosureInput {
            regime: Regime::NewYorkCity,
            apartment_unit_count: 10,
            fire_safety_plan_posted_at_entrance: true,
            fire_safety_plan_mailed_annually: true,
            emergency_notice_on_apartment_doors: true,
            emergency_notice_in_lobby: true,
            smoke_detector_notice_at_mailboxes: true,
            co_detector_notice_in_common_area: true,
        }
    }

    fn ca_compliant() -> TenantFireSafetyPlanDisclosureInput {
        let mut i = nyc_fully_compliant();
        i.regime = Regime::California;
        i
    }

    fn default_base() -> TenantFireSafetyPlanDisclosureInput {
        let mut i = nyc_fully_compliant();
        i.regime = Regime::Default;
        i
    }

    #[test]
    fn nyc_fully_compliant_passes() {
        let r = check(&nyc_fully_compliant());
        assert!(r.compliant);
        assert!(r.building_in_scope);
        assert!(r.annual_mailing_required);
    }

    #[test]
    fn nyc_2_units_out_of_scope() {
        let mut i = nyc_fully_compliant();
        i.apartment_unit_count = 2;
        i.fire_safety_plan_posted_at_entrance = false;
        let r = check(&i);
        assert!(!r.building_in_scope);
        assert!(r.compliant);
    }

    #[test]
    fn nyc_3_unit_boundary_in_scope() {
        let mut i = nyc_fully_compliant();
        i.apartment_unit_count = 3;
        let r = check(&i);
        assert!(r.building_in_scope);
    }

    #[test]
    fn nyc_no_fire_safety_plan_at_entrance_violates() {
        let mut i = nyc_fully_compliant();
        i.fire_safety_plan_posted_at_entrance = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("HPD Fire Safety Plan") && v.contains("entrance")));
    }

    #[test]
    fn nyc_no_annual_mailing_violates() {
        let mut i = nyc_fully_compliant();
        i.fire_safety_plan_mailed_annually = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("mailed") && v.contains("ANNUALLY")));
    }

    #[test]
    fn nyc_no_apartment_door_notice_violates() {
        let mut i = nyc_fully_compliant();
        i.emergency_notice_on_apartment_doors = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("Emergency Preparedness Notice") && v.contains("INSIDE")));
    }

    #[test]
    fn nyc_no_lobby_notice_violates() {
        let mut i = nyc_fully_compliant();
        i.emergency_notice_in_lobby = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("Emergency Preparedness Notice") && v.contains("lobby")));
    }

    #[test]
    fn nyc_no_smoke_detector_mailbox_notice_violates() {
        let mut i = nyc_fully_compliant();
        i.smoke_detector_notice_at_mailboxes = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 27-2046") && v.contains("mailboxes")));
    }

    #[test]
    fn nyc_no_co_detector_notice_violates() {
        let mut i = nyc_fully_compliant();
        i.co_detector_notice_in_common_area = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("Carbon Monoxide") && v.contains("common area")));
    }

    #[test]
    fn nyc_all_6_violations_stack() {
        let mut i = nyc_fully_compliant();
        i.fire_safety_plan_posted_at_entrance = false;
        i.fire_safety_plan_mailed_annually = false;
        i.emergency_notice_on_apartment_doors = false;
        i.emergency_notice_in_lobby = false;
        i.smoke_detector_notice_at_mailboxes = false;
        i.co_detector_notice_in_common_area = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert_eq!(r.violations.len(), 6);
    }

    #[test]
    fn nyc_citation_pins_authorities() {
        let r = check(&nyc_fully_compliant());
        assert!(r.citation.contains("§ 27-2046"));
        assert!(r.citation.contains("Article 11"));
        assert!(r.citation.contains("HPD Required Signs"));
    }

    #[test]
    fn ca_compliant_passes() {
        let r = check(&ca_compliant());
        assert!(r.compliant);
        assert!(r.building_in_scope);
        assert!(!r.annual_mailing_required);
    }

    #[test]
    fn ca_no_disclosure_violates() {
        let mut i = ca_compliant();
        i.fire_safety_plan_posted_at_entrance = false;
        i.emergency_notice_in_lobby = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 13145")));
    }

    #[test]
    fn ca_partial_disclosure_at_lobby_satisfies() {
        let mut i = ca_compliant();
        i.fire_safety_plan_posted_at_entrance = false;
        i.emergency_notice_in_lobby = true;
        let r = check(&i);
        assert!(r.compliant);
    }

    #[test]
    fn ca_citation_pins_authorities() {
        let r = check(&ca_compliant());
        assert!(r.citation.contains("§§ 13145, 17926"));
        assert!(r.citation.contains("Carbon Monoxide Poisoning Prevention Act"));
    }

    #[test]
    fn ca_2_units_out_of_scope() {
        let mut i = ca_compliant();
        i.apartment_unit_count = 2;
        i.fire_safety_plan_posted_at_entrance = false;
        i.emergency_notice_in_lobby = false;
        let r = check(&i);
        assert!(!r.building_in_scope);
        assert!(r.compliant);
    }

    #[test]
    fn default_4_units_in_scope() {
        let mut i = default_base();
        i.apartment_unit_count = 4;
        let r = check(&i);
        assert!(r.building_in_scope);
        assert!(r.compliant);
    }

    #[test]
    fn default_3_units_out_of_scope() {
        let mut i = default_base();
        i.apartment_unit_count = 3;
        let r = check(&i);
        assert!(!r.building_in_scope);
        assert!(r.compliant);
    }

    #[test]
    fn default_no_violations_regardless_of_inputs() {
        let mut i = default_base();
        i.fire_safety_plan_posted_at_entrance = false;
        i.fire_safety_plan_mailed_annually = false;
        i.emergency_notice_on_apartment_doors = false;
        let r = check(&i);
        assert!(r.compliant);
    }

    #[test]
    fn default_citation_pins_ifc() {
        let r = check(&default_base());
        assert!(r.citation.contains("IFC"));
        assert!(r.citation.contains("§ 403.10"));
    }

    #[test]
    fn three_regimes_routed_correctly() {
        for regime in [Regime::NewYorkCity, Regime::California, Regime::Default] {
            let mut i = nyc_fully_compliant();
            i.regime = regime;
            let r = check(&i);
            assert!(!r.citation.is_empty());
        }
    }

    #[test]
    fn nyc_uniquely_requires_annual_mailing_invariant() {
        let r_nyc = check(&nyc_fully_compliant());
        assert!(r_nyc.annual_mailing_required);

        let r_ca = check(&ca_compliant());
        assert!(!r_ca.annual_mailing_required);

        let r_default = check(&default_base());
        assert!(!r_default.annual_mailing_required);
    }

    #[test]
    fn nyc_3_unit_threshold_lower_than_default_4_invariant() {
        let mut i_nyc = nyc_fully_compliant();
        i_nyc.apartment_unit_count = 3;
        let r_nyc = check(&i_nyc);
        assert!(r_nyc.building_in_scope);

        let mut i_default = default_base();
        i_default.apartment_unit_count = 3;
        let r_default = check(&i_default);
        assert!(!r_default.building_in_scope);
    }

    #[test]
    fn nyc_note_pins_3_plus_apartment_threshold() {
        let r = check(&nyc_fully_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("3+ apartment buildings")));
    }

    #[test]
    fn nyc_note_pins_annual_mailing() {
        let r = check(&nyc_fully_compliant());
        assert!(r.notes.iter().any(|n| n.contains("ANNUALLY")));
    }

    #[test]
    fn nyc_note_pins_co_detector_notice() {
        let r = check(&nyc_fully_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Carbon Monoxide") && n.contains("common area")));
    }
}
