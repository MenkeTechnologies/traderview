//! Rental property swimming pool drain safety compliance —
//! when a trader-landlord operating a multifamily building
//! with pool or spa must comply with federal Virginia
//! Graeme Baker Pool and Spa Safety Act (VGB Act) drain
//! cover + anti-entrapment requirements. Trader-landlord
//! operational concern: multifamily rental pools are
//! treated as "public pools" under VGB Act; non-compliant
//! drain covers create per-incident strict liability +
//! CPSC civil penalty exposure + wrongful death tort
//! liability. Distinct from siblings `swimming_pool_safety`
//! (general pool fencing/barrier framework — already
//! shipped), `rental_carbon_monoxide_detector` (CO),
//! `rental_bedroom_egress_window` (structural).
//!
//! **Three regimes**:
//!
//! **Federal — Virginia Graeme Baker Pool and Spa Safety
//! Act of 2007 (VGB Act, Pub. L. 110-140 EISA Title 14,
//! eff. December 19, 2008; 15 USC §§ 8001-8008)**:
//! - Applies to ALL **public pools and spas** including
//!   **apartment complexes**, hotels, schools, community
//!   centers (treats multifamily rentals as public).
//! - All pools/spas must have **ASME/ANSI A112.19.8-2007
//!   drain covers** (now **ANSI/APSP/ICC-16** successor
//!   standard per 2019 CPSC rule).
//! - Single-drain pools/spas (other than unblockable
//!   drains) must employ at least ONE additional
//!   anti-entrapment safeguard: (1) separated drain
//!   systems; (2) safety vacuum release systems (SVRS);
//!   (3) suction-limiting vent systems; (4) gravity
//!   drainage systems; (5) automatic pump shutoff; or
//!   (6) equivalent system approved by CPSC.
//! - CPSC civil penalties up to **$120,000 per violation**
//!   per 15 USC § 2069 (CPSA penalty schedule).
//! - Private residential pools NOT covered by federal
//!   mandate.
//!
//! **California — VGB-incorporated + Cal. Health & Safety
//! Code § 116064.1 (Pool Safety Act + 2008 SB 442)**:
//! - Adopts VGB Act drain cover + anti-entrapment
//!   standards for public pools.
//! - Adds pool fence requirements under Cal. Health &
//!   Safety Code § 115922 (5-foot enclosure + self-
//!   latching gate).
//! - California Building Code Ch. 31B aquatic facility
//!   chapter incorporates ASME/APSP/ICC-16 by reference.
//! - Cal. SB 442 of 2017 strengthened residential pool
//!   safety (separate from VGB-covered public pools).
//! - Cal. Health & Safety Code § 116064.4 — CDPH (state
//!   health department) enforcement authority.
//!
//! **Florida — Florida Building Code § 454.2.17 + Florida
//! Statute § 514.0315 (Florida Public Swimming Pool and
//! Bathing Place Safety Act)**:
//! - Adopts VGB Act federal standards for public pools.
//! - Florida-specific drain cover inspection and
//!   recertification cycle.
//! - FL Department of Health enforcement under FL Statute
//!   § 514.0315.
//! - Florida Building Code § 454.2.17 requires
//!   compliance with ASME/APSP/ICC-16 successor standard.
//! - FL Pool Safety Act adds residential pool fencing
//!   requirements (separate from VGB-covered).
//!
//! Citations: Virginia Graeme Baker Pool and Spa Safety
//! Act of 2007 (Pub. L. 110-140 EISA Title 14); 15 USC §§
//! 8001-8008; 15 USC § 2069; ASME/ANSI A112.19.8-2007;
//! ANSI/APSP/ICC-16; Cal. Health & Safety Code § 116064.1
//! and § 115922 and § 116064.4 and California Building Code
//! Ch. 31B and FL Building Code § 454.2.17 and FL Statute §
//! 514.0315 and 16 CFR Part 1450 (CPSC pool safety rules).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    Federal,
    California,
    Florida,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SecondaryProtection {
    /// § 8003(c)(1)(A)(i) — separated drain systems
    /// (multiple drains to prevent single-drain entrapment).
    SeparatedDrainSystems,
    /// § 8003(c)(1)(A)(ii) — safety vacuum release system
    /// (SVRS).
    SafetyVacuumReleaseSystem,
    /// § 8003(c)(1)(A)(iii) — suction-limiting vent system.
    SuctionLimitingVentSystem,
    /// § 8003(c)(1)(A)(iv) — gravity drainage system.
    GravityDrainageSystem,
    /// § 8003(c)(1)(A)(v) — automatic pump shutoff.
    AutomaticPumpShutoff,
    /// § 8003(c)(1)(A)(vi) — equivalent system approved by
    /// CPSC.
    CpscApprovedEquivalentSystem,
    /// No secondary protection installed.
    None,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RentalSwimmingPoolDrainSafetyInput {
    pub regime: Regime,
    /// Whether pool/spa is at multifamily building (treated
    /// as public pool under VGB Act).
    pub multifamily_building: bool,
    /// Whether drain cover meets ASME/ANSI A112.19.8-2007
    /// or successor ANSI/APSP/ICC-16 standard.
    pub drain_cover_compliant: bool,
    /// Whether pool has single drain (other than unblockable
    /// drain) requiring secondary anti-entrapment protection.
    pub single_drain_requiring_secondary: bool,
    /// Type of secondary anti-entrapment protection (if any).
    pub secondary_protection: SecondaryProtection,
    /// Whether CA pool fence under § 115922 is installed
    /// (5-foot enclosure + self-latching gate).
    pub ca_pool_fence_compliant: bool,
    /// CPSC civil penalty assessed in cents (cap $120,000
    /// per violation under 15 USC § 2069).
    pub cpsc_penalty_assessed_cents: u64,
    /// Whether FL Department of Health drain cover
    /// recertification was conducted within required cycle.
    pub fl_doh_recertification_current: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RentalSwimmingPoolDrainSafetyResult {
    pub compliant: bool,
    pub vgb_act_applies: bool,
    pub drain_cover_compliant: bool,
    pub secondary_protection_adequate: bool,
    pub ca_pool_fence_compliant: bool,
    pub cpsc_penalty_within_cap: bool,
    pub fl_recertification_current: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &RentalSwimmingPoolDrainSafetyInput) -> RentalSwimmingPoolDrainSafetyResult {
    match input.regime {
        Regime::Federal => check_federal(input),
        Regime::California => check_ca(input),
        Regime::Florida => check_fl(input),
    }
}

fn check_federal(
    input: &RentalSwimmingPoolDrainSafetyInput,
) -> RentalSwimmingPoolDrainSafetyResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "Virginia Graeme Baker Pool and Spa Safety Act of 2007 (Pub. L. 110-140 EISA Title 14, eff. December 19, 2008; 15 USC §§ 8001-8008) — applies to ALL public pools and spas including apartment complexes, hotels, schools, community centers".to_string(),
        "15 USC § 8003(c) — all public pools/spas must have ASME/ANSI A112.19.8-2007 drain covers; ANSI/APSP/ICC-16 successor standard per 2019 CPSC rule".to_string(),
        "15 USC § 8003(c)(1)(A) — single-drain pools (other than unblockable drains) must employ at least ONE additional anti-entrapment safeguard: (i) separated drain systems; (ii) safety vacuum release system (SVRS); (iii) suction-limiting vent system; (iv) gravity drainage system; (v) automatic pump shutoff; (vi) equivalent system approved by CPSC".to_string(),
        "15 USC § 2069 — CPSC civil penalties up to $120,000 per violation under CPSA penalty schedule".to_string(),
        "Private residential pools NOT covered by federal VGB Act mandate".to_string(),
    ];

    let applies = input.multifamily_building;

    if applies && !input.drain_cover_compliant {
        violations.push(
            "15 USC § 8003(c) — drain cover must meet ASME/ANSI A112.19.8-2007 or ANSI/APSP/ICC-16 successor standard".to_string(),
        );
    }

    let secondary_adequate = !input.single_drain_requiring_secondary
        || !matches!(input.secondary_protection, SecondaryProtection::None);

    if applies && input.single_drain_requiring_secondary && !secondary_adequate {
        violations.push(
            "15 USC § 8003(c)(1)(A) — single-drain pool must employ at least one additional anti-entrapment safeguard (separated drain / SVRS / vent system / gravity drainage / automatic pump shutoff / CPSC-approved equivalent)".to_string(),
        );
    }

    const CPSC_PENALTY_CAP_CENTS: u64 = 12_000_000;
    let penalty_within_cap = input.cpsc_penalty_assessed_cents <= CPSC_PENALTY_CAP_CENTS;
    if !penalty_within_cap {
        violations.push(
            "15 USC § 2069 — CPSC civil penalty capped at $120,000 per violation".to_string(),
        );
    }

    RentalSwimmingPoolDrainSafetyResult {
        compliant: violations.is_empty(),
        vgb_act_applies: applies,
        drain_cover_compliant: input.drain_cover_compliant,
        secondary_protection_adequate: secondary_adequate,
        ca_pool_fence_compliant: true,
        cpsc_penalty_within_cap: penalty_within_cap,
        fl_recertification_current: true,
        violations,
        citation: "Virginia Graeme Baker Pool and Spa Safety Act of 2007 (Pub. L. 110-140); 15 USC §§ 8001-8008; 15 USC § 2069; ASME/ANSI A112.19.8-2007; ANSI/APSP/ICC-16; 16 CFR Part 1450",
        notes,
    }
}

fn check_ca(input: &RentalSwimmingPoolDrainSafetyInput) -> RentalSwimmingPoolDrainSafetyResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "Cal. Health & Safety Code § 116064.1 (Pool Safety Act + 2008 SB 442) — adopts VGB Act drain cover and anti-entrapment standards for public pools".to_string(),
        "Cal. Health & Safety Code § 115922 — pool fence requirements (5-foot enclosure + self-latching gate)".to_string(),
        "California Building Code Ch. 31B — aquatic facility chapter incorporates ANSI/APSP/ICC-16 by reference".to_string(),
        "Cal. SB 442 of 2017 — strengthened residential pool safety (separate from VGB-covered public pools)".to_string(),
        "Cal. Health & Safety Code § 116064.4 — CDPH (state health department) enforcement authority".to_string(),
    ];

    let applies = input.multifamily_building;

    if applies && !input.drain_cover_compliant {
        violations.push(
            "Cal. Health & Safety Code § 116064.1 + California Building Code Ch. 31B — drain cover must meet ANSI/APSP/ICC-16 successor standard".to_string(),
        );
    }

    let secondary_adequate = !input.single_drain_requiring_secondary
        || !matches!(input.secondary_protection, SecondaryProtection::None);

    if applies && input.single_drain_requiring_secondary && !secondary_adequate {
        violations.push(
            "Cal. Health & Safety Code § 116064.1 (incorporating VGB § 8003(c)(1)(A)) — single-drain pool must employ at least one additional anti-entrapment safeguard".to_string(),
        );
    }

    if applies && !input.ca_pool_fence_compliant {
        violations.push(
            "Cal. Health & Safety Code § 115922 — pool fence required (5-foot enclosure + self-latching gate)".to_string(),
        );
    }

    RentalSwimmingPoolDrainSafetyResult {
        compliant: violations.is_empty(),
        vgb_act_applies: applies,
        drain_cover_compliant: input.drain_cover_compliant,
        secondary_protection_adequate: secondary_adequate,
        ca_pool_fence_compliant: input.ca_pool_fence_compliant,
        cpsc_penalty_within_cap: true,
        fl_recertification_current: true,
        violations,
        citation: "Cal. Health & Safety Code § 116064.1 + § 115922 + § 116064.4; California Building Code Ch. 31B; Cal. SB 442 of 2017; VGB Act incorporated",
        notes,
    }
}

fn check_fl(input: &RentalSwimmingPoolDrainSafetyInput) -> RentalSwimmingPoolDrainSafetyResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "Florida Building Code § 454.2.17 + Florida Statute § 514.0315 (Florida Public Swimming Pool and Bathing Place Safety Act) — adopts VGB Act federal standards for public pools".to_string(),
        "FL Statute § 514.0315 — FL Department of Health enforcement of public pool/spa safety standards".to_string(),
        "Florida Building Code § 454.2.17 — requires compliance with ASME/APSP/ICC-16 successor standard".to_string(),
        "Florida-specific drain cover inspection and recertification cycle (typically every 5 years)".to_string(),
        "FL Pool Safety Act adds residential pool fencing requirements (separate from VGB-covered)".to_string(),
    ];

    let applies = input.multifamily_building;

    if applies && !input.drain_cover_compliant {
        violations.push(
            "Florida Building Code § 454.2.17 — drain cover must meet ANSI/APSP/ICC-16 successor standard".to_string(),
        );
    }

    let secondary_adequate = !input.single_drain_requiring_secondary
        || !matches!(input.secondary_protection, SecondaryProtection::None);

    if applies && input.single_drain_requiring_secondary && !secondary_adequate {
        violations.push(
            "FL Statute § 514.0315 (incorporating VGB § 8003(c)(1)(A)) — single-drain pool must employ at least one additional anti-entrapment safeguard".to_string(),
        );
    }

    if applies && !input.fl_doh_recertification_current {
        violations.push(
            "FL Statute § 514.0315 + FL Building Code § 454.2.17 — FL Department of Health drain cover recertification required within statutory cycle".to_string(),
        );
    }

    RentalSwimmingPoolDrainSafetyResult {
        compliant: violations.is_empty(),
        vgb_act_applies: applies,
        drain_cover_compliant: input.drain_cover_compliant,
        secondary_protection_adequate: secondary_adequate,
        ca_pool_fence_compliant: true,
        cpsc_penalty_within_cap: true,
        fl_recertification_current: input.fl_doh_recertification_current,
        violations,
        citation: "Florida Building Code § 454.2.17; FL Statute § 514.0315; ANSI/APSP/ICC-16; VGB Act incorporated",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn federal_clean() -> RentalSwimmingPoolDrainSafetyInput {
        RentalSwimmingPoolDrainSafetyInput {
            regime: Regime::Federal,
            multifamily_building: true,
            drain_cover_compliant: true,
            single_drain_requiring_secondary: false,
            secondary_protection: SecondaryProtection::None,
            ca_pool_fence_compliant: true,
            cpsc_penalty_assessed_cents: 0,
            fl_doh_recertification_current: true,
        }
    }

    fn ca_clean() -> RentalSwimmingPoolDrainSafetyInput {
        let mut i = federal_clean();
        i.regime = Regime::California;
        i
    }

    fn fl_clean() -> RentalSwimmingPoolDrainSafetyInput {
        let mut i = federal_clean();
        i.regime = Regime::Florida;
        i
    }

    #[test]
    fn federal_clean_compliant() {
        let r = check(&federal_clean());
        assert!(r.compliant);
        assert!(r.vgb_act_applies);
    }

    #[test]
    fn federal_private_residential_not_covered() {
        let mut i = federal_clean();
        i.multifamily_building = false;
        i.drain_cover_compliant = false;
        let r = check(&i);
        assert!(r.compliant);
        assert!(!r.vgb_act_applies);
    }

    #[test]
    fn federal_non_compliant_drain_cover_violation() {
        let mut i = federal_clean();
        i.drain_cover_compliant = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("ASME/ANSI A112.19.8-2007")));
    }

    #[test]
    fn federal_single_drain_with_secondary_compliant() {
        let mut i = federal_clean();
        i.single_drain_requiring_secondary = true;
        i.secondary_protection = SecondaryProtection::SafetyVacuumReleaseSystem;
        let r = check(&i);
        assert!(r.compliant);
        assert!(r.secondary_protection_adequate);
    }

    #[test]
    fn federal_single_drain_without_secondary_violation() {
        let mut i = federal_clean();
        i.single_drain_requiring_secondary = true;
        i.secondary_protection = SecondaryProtection::None;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(!r.secondary_protection_adequate);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 8003(c)(1)(A)") && v.contains("SVRS")));
    }

    #[test]
    fn federal_separated_drain_systems_adequate() {
        let mut i = federal_clean();
        i.single_drain_requiring_secondary = true;
        i.secondary_protection = SecondaryProtection::SeparatedDrainSystems;
        let r = check(&i);
        assert!(r.secondary_protection_adequate);
    }

    #[test]
    fn federal_gravity_drainage_system_adequate() {
        let mut i = federal_clean();
        i.single_drain_requiring_secondary = true;
        i.secondary_protection = SecondaryProtection::GravityDrainageSystem;
        let r = check(&i);
        assert!(r.secondary_protection_adequate);
    }

    #[test]
    fn federal_automatic_pump_shutoff_adequate() {
        let mut i = federal_clean();
        i.single_drain_requiring_secondary = true;
        i.secondary_protection = SecondaryProtection::AutomaticPumpShutoff;
        let r = check(&i);
        assert!(r.secondary_protection_adequate);
    }

    #[test]
    fn federal_cpsc_approved_equivalent_adequate() {
        let mut i = federal_clean();
        i.single_drain_requiring_secondary = true;
        i.secondary_protection = SecondaryProtection::CpscApprovedEquivalentSystem;
        let r = check(&i);
        assert!(r.secondary_protection_adequate);
    }

    #[test]
    fn federal_cpsc_penalty_at_120k_cap_compliant() {
        let mut i = federal_clean();
        i.cpsc_penalty_assessed_cents = 12_000_000;
        let r = check(&i);
        assert!(r.cpsc_penalty_within_cap);
    }

    #[test]
    fn federal_cpsc_penalty_above_120k_cap_violation() {
        let mut i = federal_clean();
        i.cpsc_penalty_assessed_cents = 12_000_001;
        let r = check(&i);
        assert!(!r.cpsc_penalty_within_cap);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 2069") && v.contains("$120,000")));
    }

    #[test]
    fn ca_clean_compliant() {
        let r = check(&ca_clean());
        assert!(r.compliant);
        assert!(r.vgb_act_applies);
    }

    #[test]
    fn ca_no_pool_fence_violation() {
        let mut i = ca_clean();
        i.ca_pool_fence_compliant = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 115922") && v.contains("5-foot")));
    }

    #[test]
    fn ca_non_compliant_drain_cover_violation() {
        let mut i = ca_clean();
        i.drain_cover_compliant = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 116064.1") && v.contains("ANSI/APSP/ICC-16")));
    }

    #[test]
    fn fl_clean_compliant() {
        let r = check(&fl_clean());
        assert!(r.compliant);
        assert!(r.vgb_act_applies);
    }

    #[test]
    fn fl_no_recertification_violation() {
        let mut i = fl_clean();
        i.fl_doh_recertification_current = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 514.0315") && v.contains("recertification")));
    }

    #[test]
    fn fl_non_compliant_drain_cover_violation() {
        let mut i = fl_clean();
        i.drain_cover_compliant = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 454.2.17")));
    }

    #[test]
    fn citation_pins_federal_authority() {
        let r = check(&federal_clean());
        assert!(r.citation.contains("Pub. L. 110-140"));
        assert!(r.citation.contains("§§ 8001-8008"));
        assert!(r.citation.contains("§ 2069"));
        assert!(r.citation.contains("A112.19.8-2007"));
        assert!(r.citation.contains("ANSI/APSP/ICC-16"));
    }

    #[test]
    fn citation_pins_ca_authority() {
        let r = check(&ca_clean());
        assert!(r.citation.contains("§ 116064.1"));
        assert!(r.citation.contains("§ 115922"));
        assert!(r.citation.contains("Ch. 31B"));
        assert!(r.citation.contains("SB 442"));
    }

    #[test]
    fn citation_pins_fl_authority() {
        let r = check(&fl_clean());
        assert!(r.citation.contains("§ 454.2.17"));
        assert!(r.citation.contains("§ 514.0315"));
    }

    #[test]
    fn note_pins_vgb_2008_effective_date() {
        let r = check(&federal_clean());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("December 19, 2008") && n.contains("Pub. L. 110-140")));
    }

    #[test]
    fn note_pins_apartment_complexes_in_scope() {
        let r = check(&federal_clean());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("apartment complexes")));
    }

    #[test]
    fn note_pins_six_secondary_protection_options() {
        let r = check(&federal_clean());
        assert!(r.notes.iter().any(|n| n.contains("separated drain")
            && n.contains("SVRS")
            && n.contains("vent system")
            && n.contains("gravity drainage")
            && n.contains("automatic pump shutoff")
            && n.contains("equivalent")));
    }

    #[test]
    fn note_pins_120k_cpsc_penalty_cap() {
        let r = check(&federal_clean());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("$120,000") && n.contains("§ 2069")));
    }

    #[test]
    fn note_pins_ca_5_foot_fence() {
        let r = check(&ca_clean());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("5-foot") && n.contains("§ 115922")));
    }

    #[test]
    fn note_pins_fl_recertification_cycle() {
        let r = check(&fl_clean());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("recertification cycle") || n.contains("5 years")));
    }

    #[test]
    fn ca_uniquely_requires_5_foot_pool_fence_invariant() {
        let mut i_fed = federal_clean();
        i_fed.ca_pool_fence_compliant = false;
        let r_fed = check(&i_fed);
        assert!(r_fed.compliant);

        let mut i_ca = ca_clean();
        i_ca.ca_pool_fence_compliant = false;
        let r_ca = check(&i_ca);
        assert!(!r_ca.compliant);
    }

    #[test]
    fn fl_uniquely_requires_doh_recertification_invariant() {
        let mut i_fed = federal_clean();
        i_fed.fl_doh_recertification_current = false;
        let r_fed = check(&i_fed);
        assert!(r_fed.compliant);

        let mut i_fl = fl_clean();
        i_fl.fl_doh_recertification_current = false;
        let r_fl = check(&i_fl);
        assert!(!r_fl.compliant);
    }

    #[test]
    fn secondary_protection_truth_table_seven_cells() {
        for (protection, exp_adequate) in [
            (SecondaryProtection::SeparatedDrainSystems, true),
            (SecondaryProtection::SafetyVacuumReleaseSystem, true),
            (SecondaryProtection::SuctionLimitingVentSystem, true),
            (SecondaryProtection::GravityDrainageSystem, true),
            (SecondaryProtection::AutomaticPumpShutoff, true),
            (SecondaryProtection::CpscApprovedEquivalentSystem, true),
            (SecondaryProtection::None, false),
        ] {
            let mut i = federal_clean();
            i.single_drain_requiring_secondary = true;
            i.secondary_protection = protection;
            let r = check(&i);
            assert_eq!(
                r.secondary_protection_adequate, exp_adequate,
                "protection={:?} expected adequate={}",
                protection, exp_adequate
            );
        }
    }

    #[test]
    fn private_residential_pool_not_covered_invariant_across_regimes() {
        for regime in [Regime::Federal, Regime::California, Regime::Florida] {
            let mut i = match regime {
                Regime::Federal => federal_clean(),
                Regime::California => ca_clean(),
                Regime::Florida => fl_clean(),
            };
            i.multifamily_building = false;
            i.drain_cover_compliant = false;
            let r = check(&i);
            assert!(r.compliant);
            assert!(!r.vgb_act_applies);
        }
    }

    #[test]
    fn multiple_federal_violations_stack() {
        let mut i = federal_clean();
        i.drain_cover_compliant = false;
        i.single_drain_requiring_secondary = true;
        i.secondary_protection = SecondaryProtection::None;
        i.cpsc_penalty_assessed_cents = 20_000_000;
        let r = check(&i);
        assert_eq!(r.violations.len(), 3);
    }
}
