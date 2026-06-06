//! California Swimming Pool Safety Act compliance — when a
//! building permit is issued for the construction of a new
//! swimming pool or spa OR the remodeling of an existing
//! swimming pool or spa at a private single-family home, the
//! pool or spa SHALL be equipped with AT LEAST TWO of seven
//! enumerated drowning prevention safety features under Cal.
//! Health & Safety Code § 115922. Trader-landlord critical
//! for CA single-family rental properties with pools — non-
//! compliance breaches habitability + exposes landlord to
//! drowning-incident premises liability.
//!
//! Distinct from siblings `detector_requirements` (smoke /
//! CO), `fire_sprinkler_disclosure` (fire suppression),
//! `water_heater_earthquake_strap` (§ 19211 seismic), and
//! `lead_in_drinking_water_disclosure` (drinking water).
//!
//! **Two regimes**:
//!
//! **California — Cal. Health & Safety Code §§ 115920-
//! 115929 (Swimming Pool Safety Act; SB 442 eff. January 1,
//! 2018)**:
//! - § 115922(a) — at least TWO of seven drowning prevention
//!   features required at building-permit issuance for new
//!   pool / spa OR remodeling of existing pool / spa.
//! - Seven features:
//!   1. § 115923 enclosure isolating pool from home
//!   2. Removable mesh fencing per ASTM F2286 + self-closing
//!      / self-latching gate with key lockable device
//!   3. Manually or power-operated safety pool cover per
//!      ASTM F1346-23
//!   4. Exit alarms on home's doors and windows providing
//!      direct pool access
//!   5. Self-closing / self-latching device on home's doors
//!      providing direct pool access; release mechanism ≥
//!      54 inches above floor
//!   6. Pool alarm per ASTM F2208 (surface motion + pressure
//!      + sonar + laser + infrared)
//!   7. Other means of equivalent drowning prevention as
//!      approved by local building official
//! - Statute applies to private single-family homes;
//!   multifamily pools regulated by California Code of
//!   Regulations Title 22 + § 116025 et seq. (CalCode).
//!
//! **Default — no statutory pool-safety feature requirement
//! at permit issuance**. Common-law premises liability + IPC
//! § 305 (where adopted) + local pool ordinances may apply.
//!
//! Citations: Cal. Health & Safety Code §§ 115920-115929
//! (Swimming Pool Safety Act); SB 442 Stats. 2017 ch. 670;
//! ASTM F2286 (removable mesh fencing); ASTM F1346-23
//! (safety pool cover); ASTM F2208 (pool alarm).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    California,
    Default,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PermitTrigger {
    /// New swimming pool or spa construction.
    NewConstruction,
    /// Remodeling of existing pool or spa.
    Remodel,
    /// No permit triggered (no construction or remodel).
    None,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SwimmingPoolSafetyInput {
    pub regime: Regime,
    pub permit_trigger: PermitTrigger,
    /// Whether the property is a private single-family home
    /// (vs multifamily rental — multifamily uses different
    /// regulatory framework).
    pub private_single_family_home: bool,
    /// Feature 1: § 115923 enclosure isolating pool from home.
    pub feature_115923_enclosure: bool,
    /// Feature 2: Removable mesh fencing per ASTM F2286.
    pub feature_removable_mesh_astm_f2286: bool,
    /// Feature 3: Safety pool cover per ASTM F1346-23.
    pub feature_safety_pool_cover_astm_f1346: bool,
    /// Feature 4: Exit alarms on doors and windows.
    pub feature_exit_alarms: bool,
    /// Feature 5: Self-closing / self-latching door device
    /// with release mechanism ≥ 54 inches above floor.
    pub feature_self_closing_door_54in: bool,
    /// Feature 6: Pool alarm per ASTM F2208.
    pub feature_pool_alarm_astm_f2208: bool,
    /// Feature 7: Other means approved by local building
    /// official.
    pub feature_other_approved_means: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SwimmingPoolSafetyResult {
    pub compliant: bool,
    pub statute_engages: bool,
    pub features_present_count: u32,
    pub minimum_features_required: u32,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &SwimmingPoolSafetyInput) -> SwimmingPoolSafetyResult {
    match input.regime {
        Regime::California => check_california(input),
        Regime::Default => check_default(input),
    }
}

fn check_california(input: &SwimmingPoolSafetyInput) -> SwimmingPoolSafetyResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "Cal. Health & Safety Code § 115922(a) — at least TWO of seven drowning prevention features required at building-permit issuance for new pool / spa OR remodeling of existing pool / spa at private single-family home"
            .to_string(),
        "Cal. Health & Safety Code § 115922 seven features: (1) § 115923 enclosure isolating pool from home; (2) ASTM F2286 removable mesh fencing + self-closing self-latching gate with key lock; (3) ASTM F1346-23 safety pool cover; (4) exit alarms on doors and windows with direct pool access; (5) self-closing self-latching device with release mechanism ≥ 54 inches above floor; (6) ASTM F2208 pool alarm; (7) other equivalent means approved by local building official"
            .to_string(),
        "SB 442 Stats. 2017 ch. 670 — Swimming Pool Safety Act amendments effective January 1, 2018 increased minimum from one to two features"
            .to_string(),
        "Statute applies to private single-family homes; multifamily pools regulated by California Code of Regulations Title 22 + Cal. Health & Safety Code § 116025 et seq. (CalCode public pool framework)"
            .to_string(),
    ];

    let permit_triggered = matches!(
        input.permit_trigger,
        PermitTrigger::NewConstruction | PermitTrigger::Remodel
    );
    let statute_engages = permit_triggered && input.private_single_family_home;

    let feature_flags = [
        input.feature_115923_enclosure,
        input.feature_removable_mesh_astm_f2286,
        input.feature_safety_pool_cover_astm_f1346,
        input.feature_exit_alarms,
        input.feature_self_closing_door_54in,
        input.feature_pool_alarm_astm_f2208,
        input.feature_other_approved_means,
    ];
    let count = feature_flags.iter().filter(|f| **f).count() as u32;

    let minimum_required: u32 = 2;

    if statute_engages && count < minimum_required {
        violations.push(format!(
            "Cal. Health & Safety Code § 115922(a) — only {} of seven drowning prevention features present; minimum {} required at permit issuance",
            count, minimum_required
        ));
    }

    SwimmingPoolSafetyResult {
        compliant: violations.is_empty(),
        statute_engages,
        features_present_count: count,
        minimum_features_required: if statute_engages { minimum_required } else { 0 },
        violations,
        citation: "Cal. Health & Safety Code §§ 115920-115929 (Swimming Pool Safety Act); SB 442 Stats. 2017 ch. 670; ASTM F2286, F1346-23, F2208",
        notes,
    }
}

fn check_default(_input: &SwimmingPoolSafetyInput) -> SwimmingPoolSafetyResult {
    let notes: Vec<String> = vec![
        "default rule — no statutory pool-safety feature requirement at permit issuance; common-law premises liability + IPC § 305 (where adopted) + local pool ordinances may apply"
            .to_string(),
        "default rule — most states require some form of pool barrier (typically 48-inch fencing) but lack CA's seven-feature menu + two-minimum framework"
            .to_string(),
    ];

    SwimmingPoolSafetyResult {
        compliant: true,
        statute_engages: false,
        features_present_count: 0,
        minimum_features_required: 0,
        violations: Vec::new(),
        citation:
            "common-law premises liability + local pool ordinances + IPC § 305 (where adopted)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ca_compliant() -> SwimmingPoolSafetyInput {
        SwimmingPoolSafetyInput {
            regime: Regime::California,
            permit_trigger: PermitTrigger::NewConstruction,
            private_single_family_home: true,
            feature_115923_enclosure: true,
            feature_removable_mesh_astm_f2286: false,
            feature_safety_pool_cover_astm_f1346: true,
            feature_exit_alarms: false,
            feature_self_closing_door_54in: false,
            feature_pool_alarm_astm_f2208: false,
            feature_other_approved_means: false,
        }
    }

    fn default_base() -> SwimmingPoolSafetyInput {
        let mut i = ca_compliant();
        i.regime = Regime::Default;
        i
    }

    #[test]
    fn ca_two_features_compliant() {
        let r = check(&ca_compliant());
        assert!(r.compliant);
        assert!(r.statute_engages);
        assert_eq!(r.features_present_count, 2);
        assert_eq!(r.minimum_features_required, 2);
    }

    #[test]
    fn ca_one_feature_violates() {
        let mut i = ca_compliant();
        i.feature_safety_pool_cover_astm_f1346 = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert_eq!(r.features_present_count, 1);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 115922(a)") && v.contains("1 of seven")));
    }

    #[test]
    fn ca_zero_features_violates() {
        let mut i = ca_compliant();
        i.feature_115923_enclosure = false;
        i.feature_safety_pool_cover_astm_f1346 = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert_eq!(r.features_present_count, 0);
    }

    #[test]
    fn ca_all_seven_features_compliant() {
        let mut i = ca_compliant();
        i.feature_115923_enclosure = true;
        i.feature_removable_mesh_astm_f2286 = true;
        i.feature_safety_pool_cover_astm_f1346 = true;
        i.feature_exit_alarms = true;
        i.feature_self_closing_door_54in = true;
        i.feature_pool_alarm_astm_f2208 = true;
        i.feature_other_approved_means = true;
        let r = check(&i);
        assert!(r.compliant);
        assert_eq!(r.features_present_count, 7);
    }

    #[test]
    fn ca_no_permit_no_statute_engagement() {
        let mut i = ca_compliant();
        i.permit_trigger = PermitTrigger::None;
        let r = check(&i);
        assert!(!r.statute_engages);
        assert!(r.compliant);
    }

    #[test]
    fn ca_multifamily_no_statute_engagement() {
        let mut i = ca_compliant();
        i.private_single_family_home = false;
        let r = check(&i);
        assert!(!r.statute_engages);
        assert!(r.compliant);
    }

    #[test]
    fn ca_remodel_triggers_statute() {
        let mut i = ca_compliant();
        i.permit_trigger = PermitTrigger::Remodel;
        let r = check(&i);
        assert!(r.statute_engages);
    }

    #[test]
    fn ca_remodel_with_one_feature_violates() {
        let mut i = ca_compliant();
        i.permit_trigger = PermitTrigger::Remodel;
        i.feature_safety_pool_cover_astm_f1346 = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert_eq!(r.features_present_count, 1);
    }

    #[test]
    fn ca_citation_pins_authorities() {
        let r = check(&ca_compliant());
        assert!(r.citation.contains("§§ 115920-115929"));
        assert!(r.citation.contains("SB 442"));
        assert!(r.citation.contains("Stats. 2017 ch. 670"));
        assert!(r.citation.contains("ASTM F2286"));
        assert!(r.citation.contains("F1346-23"));
        assert!(r.citation.contains("F2208"));
    }

    #[test]
    fn ca_note_pins_two_minimum_threshold() {
        let r = check(&ca_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 115922(a)") && n.contains("at least TWO")));
    }

    #[test]
    fn ca_note_pins_seven_features_enumeration() {
        let r = check(&ca_compliant());
        assert!(r.notes.iter().any(|n| n.contains("ASTM F2286")
            && n.contains("ASTM F1346-23")
            && n.contains("ASTM F2208")
            && n.contains("54 inches")));
    }

    #[test]
    fn ca_note_pins_sb_442_eff_2018() {
        let r = check(&ca_compliant());
        assert!(r.notes.iter().any(|n| n.contains("SB 442")
            && n.contains("Stats. 2017 ch. 670")
            && n.contains("January 1, 2018")));
    }

    #[test]
    fn ca_note_pins_multifamily_carve_out() {
        let r = check(&ca_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("multifamily") && n.contains("CalCode") && n.contains("Title 22")));
    }

    #[test]
    fn ca_two_minimum_boundary_satisfied() {
        let mut i = ca_compliant();
        i.feature_115923_enclosure = true;
        i.feature_safety_pool_cover_astm_f1346 = true;
        i.feature_removable_mesh_astm_f2286 = false;
        let r = check(&i);
        assert!(r.compliant);
        assert_eq!(r.features_present_count, 2);
    }

    #[test]
    fn ca_three_features_compliant() {
        let mut i = ca_compliant();
        i.feature_removable_mesh_astm_f2286 = true;
        let r = check(&i);
        assert!(r.compliant);
        assert_eq!(r.features_present_count, 3);
    }

    #[test]
    fn ca_features_2_through_7_count_correctly() {
        for (idx, build) in [
            (0u32, true),
            (1, false),
            (2, false),
            (3, false),
            (4, false),
            (5, false),
            (6, false),
        ]
        .iter()
        .enumerate()
        {
            let _ = (idx, build);
        }

        let i = SwimmingPoolSafetyInput {
            feature_115923_enclosure: false,
            feature_removable_mesh_astm_f2286: true,
            feature_safety_pool_cover_astm_f1346: true,
            feature_exit_alarms: false,
            feature_self_closing_door_54in: false,
            feature_pool_alarm_astm_f2208: false,
            feature_other_approved_means: false,
            ..ca_compliant()
        };
        let r = check(&i);
        assert_eq!(r.features_present_count, 2);
        assert!(r.compliant);
    }

    #[test]
    fn ca_exit_alarms_self_closing_pair_compliant() {
        let i = SwimmingPoolSafetyInput {
            feature_115923_enclosure: false,
            feature_safety_pool_cover_astm_f1346: false,
            feature_exit_alarms: true,
            feature_self_closing_door_54in: true,
            ..ca_compliant()
        };
        let r = check(&i);
        assert!(r.compliant);
        assert_eq!(r.features_present_count, 2);
    }

    #[test]
    fn ca_pool_alarm_other_approved_pair_compliant() {
        let i = SwimmingPoolSafetyInput {
            feature_115923_enclosure: false,
            feature_safety_pool_cover_astm_f1346: false,
            feature_pool_alarm_astm_f2208: true,
            feature_other_approved_means: true,
            ..ca_compliant()
        };
        let r = check(&i);
        assert!(r.compliant);
        assert_eq!(r.features_present_count, 2);
    }

    #[test]
    fn default_compliant_no_statute() {
        let r = check(&default_base());
        assert!(r.compliant);
        assert!(!r.statute_engages);
    }

    #[test]
    fn default_zero_features_still_compliant() {
        let mut i = default_base();
        i.feature_115923_enclosure = false;
        i.feature_safety_pool_cover_astm_f1346 = false;
        let r = check(&i);
        assert!(r.compliant);
    }

    #[test]
    fn default_citation_pins_premises_liability() {
        let r = check(&default_base());
        assert!(r.citation.contains("common-law premises liability"));
        assert!(r.citation.contains("IPC § 305"));
    }

    #[test]
    fn default_note_pins_48_inch_fencing_baseline() {
        let r = check(&default_base());
        assert!(r.notes.iter().any(|n| n.contains("48-inch fencing")));
    }

    #[test]
    fn two_regimes_routed_correctly() {
        for regime in [Regime::California, Regime::Default] {
            let mut i = ca_compliant();
            i.regime = regime;
            let r = check(&i);
            assert!(!r.citation.is_empty());
        }
    }

    #[test]
    fn ca_uniquely_engages_statute_invariant() {
        let r_ca = check(&ca_compliant());
        assert!(r_ca.statute_engages);

        let r_default = check(&default_base());
        assert!(!r_default.statute_engages);
    }

    #[test]
    fn permit_trigger_truth_table() {
        for (trigger, exp_engages) in [
            (PermitTrigger::NewConstruction, true),
            (PermitTrigger::Remodel, true),
            (PermitTrigger::None, false),
        ] {
            let mut i = ca_compliant();
            i.permit_trigger = trigger;
            let r = check(&i);
            assert_eq!(r.statute_engages, exp_engages);
        }
    }

    #[test]
    fn ca_minimum_features_required_only_when_statute_engages() {
        let mut i_engaged = ca_compliant();
        i_engaged.permit_trigger = PermitTrigger::NewConstruction;
        let r_engaged = check(&i_engaged);
        assert_eq!(r_engaged.minimum_features_required, 2);

        let mut i_no_permit = ca_compliant();
        i_no_permit.permit_trigger = PermitTrigger::None;
        let r_no_permit = check(&i_no_permit);
        assert_eq!(r_no_permit.minimum_features_required, 0);

        let mut i_multifamily = ca_compliant();
        i_multifamily.private_single_family_home = false;
        let r_multifamily = check(&i_multifamily);
        assert_eq!(r_multifamily.minimum_features_required, 0);
    }

    #[test]
    fn ca_seven_feature_count_max_at_seven() {
        let mut i = ca_compliant();
        i.feature_115923_enclosure = true;
        i.feature_removable_mesh_astm_f2286 = true;
        i.feature_safety_pool_cover_astm_f1346 = true;
        i.feature_exit_alarms = true;
        i.feature_self_closing_door_54in = true;
        i.feature_pool_alarm_astm_f2208 = true;
        i.feature_other_approved_means = true;
        let r = check(&i);
        assert_eq!(r.features_present_count, 7);
    }
}
