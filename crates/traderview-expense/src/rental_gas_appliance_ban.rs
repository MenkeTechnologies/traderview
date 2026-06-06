//! Rental property gas appliance ban / electrification
//! mandate compliance — when must a trader-landlord building
//! new construction OR substantially renovating an existing
//! rental property comply with statutory bans on natural
//! gas / propane hookups + fossil-fuel-burning appliances?
//! Trader-landlord critical for any new-construction or
//! substantial-renovation project in NY (post-Jan 1, 2026
//! pending Second Circuit ruling) + CA (2025 Energy Code
//! eff. Jan 1, 2026). DOES NOT require existing landlords
//! to replace existing gas appliances; applies to new
//! construction.
//!
//! Distinct from siblings `cooling_requirements` (cooling
//! habitability standards), `heat_requirements` (heat
//! habitability standards), `detector_requirements` (CO +
//! smoke detector requirements), and `landlord_repair_
//! response_timeframe` (repair obligations).
//!
//! **Three regimes**:
//!
//! **New York — All-Electric Buildings Act (effective
//! January 1, 2026; enforcement stayed pending Second
//! Circuit ruling expected fall 2026)**:
//! - Bans fossil-fuel hookups in MOST new homes.
//! - Covers: natural gas mains + propane tanks + boilers +
//!   furnaces + tank/tankless water heaters + gas ranges +
//!   gas dryers + gas fireplaces.
//! - Covers: piping that supplies/distributes/delivers
//!   fossil fuel.
//! - Does NOT apply to existing buildings or existing
//!   appliances.
//! - State agreed to NOT enforce until Second Circuit
//!   resolves challenge.
//!
//! **California — 2025 Energy Code (effective January 1,
//! 2026; Title 24, Part 6)**:
//! - New construction permits require HEAT PUMPS for most
//!   space and water heating in new homes and some
//!   commercial buildings.
//! - Does NOT require existing landlords to replace
//!   existing gas appliances.
//! - SF considering ordinance for major-renovation
//!   electrification by 2027 (not yet enacted statewide).
//! - Berkeley's 2019 gas-hookup ban was enjoined by 9th
//!   Circuit in Cal. Restaurant Ass'n v. City of Berkeley
//!   (89 F.4th 1094 (9th Cir. 2023)) — federally preempted
//!   by Energy Policy and Conservation Act 42 USC § 6297;
//!   CA cities post-Berkeley generally pursue indirect
//!   bans via building/health code.
//!
//! **Default — federal law silent; locality-controlled**.
//! No federal gas ban; many local ordinances; substantial
//! status variation. Federal preemption status under EPCA
//! 42 USC § 6297 limits direct-ban approach.
//!
//! Citations: NY All-Electric Buildings Act, S4006C / A3006C
//! Part RR (2023); NY Stretch Code 2025; CA Title 24 Part 6
//! 2025 Energy Code (eff. Jan 1, 2026); Cal. Restaurant
//! Ass'n v. City of Berkeley, 89 F.4th 1094 (9th Cir. 2023)
//! (Berkeley ban enjoined); Energy Policy and Conservation
//! Act, 42 USC § 6297 (federal preemption).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    NewYork,
    California,
    Default,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProjectType {
    /// New construction (subject to gas ban / electrification
    /// mandate where applicable).
    NewConstruction,
    /// Substantial renovation (may trigger ban depending on
    /// jurisdiction).
    SubstantialRenovation,
    /// Replacement of existing gas appliance in existing
    /// building.
    AppliancePeplacement,
    /// Routine maintenance of existing appliance.
    Maintenance,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RentalGasApplianceBanInput {
    pub regime: Regime,
    pub project_type: ProjectType,
    /// Whether the project includes installation of a gas
    /// appliance (boiler / furnace / water heater / range /
    /// dryer / fireplace).
    pub gas_appliance_installation: bool,
    /// Whether the project includes installation of new
    /// fossil-fuel piping or hookup.
    pub fossil_fuel_piping_installation: bool,
    /// Whether the project includes installation of an
    /// electric heat pump (CA 2025 Energy Code requirement).
    pub heat_pump_installation: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RentalGasApplianceBanResult {
    pub project_compliant: bool,
    pub ban_engages: bool,
    pub electrification_required: bool,
    pub heat_pump_required: bool,
    pub enforcement_stayed: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &RentalGasApplianceBanInput) -> RentalGasApplianceBanResult {
    match input.regime {
        Regime::NewYork => check_ny(input),
        Regime::California => check_ca(input),
        Regime::Default => check_default(input),
    }
}

fn check_ny(input: &RentalGasApplianceBanInput) -> RentalGasApplianceBanResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "NY All-Electric Buildings Act (S4006C / A3006C Part RR, 2023) — effective January 1, 2026; bans fossil-fuel hookups in MOST new homes"
            .to_string(),
        "NY All-Electric Buildings Act — covers natural gas mains + propane tanks + boilers + furnaces + tank/tankless water heaters + gas ranges + gas dryers + gas fireplaces; covers piping supplying / distributing / delivering fossil fuel"
            .to_string(),
        "NY enforcement STAYED — state agreed to not enforce zero-emissions standard until Second Circuit U.S. Court of Appeals resolves challenge (expected fall 2026); ban does NOT apply to existing buildings or existing appliances"
            .to_string(),
    ];

    let ban_engages = matches!(input.project_type, ProjectType::NewConstruction);

    if ban_engages && input.gas_appliance_installation {
        violations.push(
            "NY All-Electric Buildings Act — gas appliance installation prohibited in new construction (subject to Second Circuit ruling pending)".to_string(),
        );
    }

    if ban_engages && input.fossil_fuel_piping_installation {
        violations.push(
            "NY All-Electric Buildings Act — fossil-fuel piping installation prohibited in new construction (subject to Second Circuit ruling pending)".to_string(),
        );
    }

    RentalGasApplianceBanResult {
        project_compliant: violations.is_empty(),
        ban_engages,
        electrification_required: ban_engages,
        heat_pump_required: false,
        enforcement_stayed: true,
        violations,
        citation: "NY All-Electric Buildings Act, S4006C / A3006C Part RR (2023)",
        notes,
    }
}

fn check_ca(input: &RentalGasApplianceBanInput) -> RentalGasApplianceBanResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "CA 2025 Energy Code (Title 24 Part 6) — effective January 1, 2026; new construction permits require HEAT PUMPS for most space and water heating in new homes and some commercial buildings"
            .to_string(),
        "CA 2025 Energy Code — does NOT require existing landlords to replace existing gas appliances; SF considering ordinance for major-renovation electrification by 2027 (not yet enacted statewide)"
            .to_string(),
        "Cal. Restaurant Ass'n v. City of Berkeley, 89 F.4th 1094 (9th Cir. 2023) — Berkeley's 2019 gas-hookup ban was ENJOINED as federally preempted by Energy Policy and Conservation Act (42 USC § 6297); CA cities post-Berkeley pursue indirect bans via building/health code"
            .to_string(),
    ];

    let heat_pump_required = matches!(input.project_type, ProjectType::NewConstruction);

    if heat_pump_required && input.gas_appliance_installation && !input.heat_pump_installation {
        violations.push(
            "CA Title 24 Part 6 (2025 Energy Code) — new construction permits require heat pumps for most space and water heating; gas appliance installed without heat pump alternative".to_string(),
        );
    }

    RentalGasApplianceBanResult {
        project_compliant: violations.is_empty(),
        ban_engages: false,
        electrification_required: false,
        heat_pump_required,
        enforcement_stayed: false,
        violations,
        citation: "CA Title 24 Part 6 (2025 Energy Code); Cal. Restaurant Ass'n v. City of Berkeley, 89 F.4th 1094 (9th Cir. 2023); 42 USC § 6297 (EPCA preemption)",
        notes,
    }
}

fn check_default(_input: &RentalGasApplianceBanInput) -> RentalGasApplianceBanResult {
    let notes: Vec<String> = vec![
        "default rule — federal law silent on gas appliance bans; locality-controlled with substantial status variation"
            .to_string(),
        "Energy Policy and Conservation Act (42 USC § 6297) — federally preempts state/local laws that establish energy efficiency / use standards different from federal standards; limits direct-ban approach (per Cal. Restaurant Ass'n v. City of Berkeley, 89 F.4th 1094, 9th Cir. 2023)"
            .to_string(),
    ];

    RentalGasApplianceBanResult {
        project_compliant: true,
        ban_engages: false,
        electrification_required: false,
        heat_pump_required: false,
        enforcement_stayed: false,
        violations: Vec::new(),
        citation: "federal law silent; 42 USC § 6297 (EPCA preemption); locality-controlled",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ny_new_construction_gas() -> RentalGasApplianceBanInput {
        RentalGasApplianceBanInput {
            regime: Regime::NewYork,
            project_type: ProjectType::NewConstruction,
            gas_appliance_installation: true,
            fossil_fuel_piping_installation: true,
            heat_pump_installation: false,
        }
    }

    fn ca_new_construction_heat_pump() -> RentalGasApplianceBanInput {
        RentalGasApplianceBanInput {
            regime: Regime::California,
            project_type: ProjectType::NewConstruction,
            gas_appliance_installation: false,
            fossil_fuel_piping_installation: false,
            heat_pump_installation: true,
        }
    }

    fn default_base() -> RentalGasApplianceBanInput {
        let mut i = ny_new_construction_gas();
        i.regime = Regime::Default;
        i
    }

    #[test]
    fn ny_new_construction_gas_appliance_violates() {
        let r = check(&ny_new_construction_gas());
        assert!(!r.project_compliant);
        assert!(r.ban_engages);
        assert!(r.electrification_required);
        assert!(r.enforcement_stayed);
    }

    #[test]
    fn ny_new_construction_all_electric_compliant() {
        let mut i = ny_new_construction_gas();
        i.gas_appliance_installation = false;
        i.fossil_fuel_piping_installation = false;
        let r = check(&i);
        assert!(r.project_compliant);
        assert!(r.ban_engages);
        assert!(r.electrification_required);
    }

    #[test]
    fn ny_existing_building_appliance_replacement_compliant() {
        let mut i = ny_new_construction_gas();
        i.project_type = ProjectType::AppliancePeplacement;
        let r = check(&i);
        assert!(r.project_compliant);
        assert!(!r.ban_engages);
    }

    #[test]
    fn ny_substantial_renovation_does_not_engage_ban() {
        let mut i = ny_new_construction_gas();
        i.project_type = ProjectType::SubstantialRenovation;
        let r = check(&i);
        assert!(r.project_compliant);
        assert!(!r.ban_engages);
    }

    #[test]
    fn ny_maintenance_compliant() {
        let mut i = ny_new_construction_gas();
        i.project_type = ProjectType::Maintenance;
        let r = check(&i);
        assert!(r.project_compliant);
        assert!(!r.ban_engages);
    }

    #[test]
    fn ny_gas_appliance_violation_message() {
        let r = check(&ny_new_construction_gas());
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("All-Electric Buildings Act") && v.contains("gas appliance")));
    }

    #[test]
    fn ny_fossil_fuel_piping_violation_message() {
        let r = check(&ny_new_construction_gas());
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("All-Electric Buildings Act") && v.contains("piping")));
    }

    #[test]
    fn ny_enforcement_stayed_flag() {
        let r = check(&ny_new_construction_gas());
        assert!(r.enforcement_stayed);
    }

    #[test]
    fn ny_citation_pins_act() {
        let r = check(&ny_new_construction_gas());
        assert!(r.citation.contains("All-Electric Buildings Act"));
        assert!(r.citation.contains("S4006C"));
        assert!(r.citation.contains("A3006C"));
        assert!(r.citation.contains("Part RR"));
    }

    #[test]
    fn ny_note_pins_january_1_2026() {
        let r = check(&ny_new_construction_gas());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("January 1, 2026") && n.contains("MOST new homes")));
    }

    #[test]
    fn ny_note_pins_appliance_list() {
        let r = check(&ny_new_construction_gas());
        assert!(r.notes.iter().any(|n| n.contains("boilers")
            && n.contains("furnaces")
            && n.contains("water heaters")
            && n.contains("gas ranges")
            && n.contains("gas dryers")
            && n.contains("gas fireplaces")));
    }

    #[test]
    fn ny_note_pins_second_circuit_stay() {
        let r = check(&ny_new_construction_gas());
        assert!(r.notes.iter().any(|n| n.contains("Second Circuit")
            && n.contains("STAYED")
            && n.contains("fall 2026")));
    }

    #[test]
    fn ca_new_construction_heat_pump_compliant() {
        let r = check(&ca_new_construction_heat_pump());
        assert!(r.project_compliant);
        assert!(r.heat_pump_required);
        assert!(!r.electrification_required);
    }

    #[test]
    fn ca_new_construction_gas_without_heat_pump_violates() {
        let mut i = ca_new_construction_heat_pump();
        i.gas_appliance_installation = true;
        i.heat_pump_installation = false;
        let r = check(&i);
        assert!(!r.project_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("Title 24 Part 6") && v.contains("heat pumps")));
    }

    #[test]
    fn ca_existing_landlord_no_retroactive_replacement() {
        let mut i = ca_new_construction_heat_pump();
        i.project_type = ProjectType::AppliancePeplacement;
        i.gas_appliance_installation = true;
        i.heat_pump_installation = false;
        let r = check(&i);
        assert!(r.project_compliant);
        assert!(!r.heat_pump_required);
    }

    #[test]
    fn ca_citation_pins_title_24() {
        let r = check(&ca_new_construction_heat_pump());
        assert!(r.citation.contains("Title 24 Part 6"));
        assert!(r.citation.contains("2025 Energy Code"));
        assert!(r.citation.contains("89 F.4th 1094"));
        assert!(r.citation.contains("§ 6297"));
    }

    #[test]
    fn ca_note_pins_berkeley_injunction() {
        let r = check(&ca_new_construction_heat_pump());
        assert!(r.notes.iter().any(|n| n.contains("Berkeley")
            && n.contains("ENJOINED")
            && n.contains("Energy Policy and Conservation Act")
            && n.contains("§ 6297")));
    }

    #[test]
    fn ca_note_pins_2027_sf_ordinance() {
        let r = check(&ca_new_construction_heat_pump());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("SF") && n.contains("2027") && n.contains("major-renovation")));
    }

    #[test]
    fn default_no_ban_compliant() {
        let r = check(&default_base());
        assert!(r.project_compliant);
        assert!(!r.ban_engages);
        assert!(!r.heat_pump_required);
        assert!(!r.enforcement_stayed);
    }

    #[test]
    fn default_citation_pins_epca_preemption() {
        let r = check(&default_base());
        assert!(r.citation.contains("federal law silent"));
        assert!(r.citation.contains("§ 6297"));
    }

    #[test]
    fn default_note_pins_epca_preemption() {
        let r = check(&default_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Energy Policy and Conservation Act") && n.contains("§ 6297")));
    }

    #[test]
    fn three_regimes_routed_correctly() {
        for regime in [Regime::NewYork, Regime::California, Regime::Default] {
            let mut i = ny_new_construction_gas();
            i.regime = regime;
            let r = check(&i);
            assert!(!r.citation.is_empty());
        }
    }

    #[test]
    fn ny_uniquely_engages_ban_invariant() {
        let r_ny = check(&ny_new_construction_gas());
        assert!(r_ny.ban_engages);

        let r_ca = check(&ca_new_construction_heat_pump());
        assert!(!r_ca.ban_engages);

        let r_default = check(&default_base());
        assert!(!r_default.ban_engages);
    }

    #[test]
    fn ca_uniquely_requires_heat_pump_invariant() {
        let r_ca = check(&ca_new_construction_heat_pump());
        assert!(r_ca.heat_pump_required);

        let r_ny = check(&ny_new_construction_gas());
        assert!(!r_ny.heat_pump_required);
    }

    #[test]
    fn ny_uniquely_enforcement_stayed_invariant() {
        let r_ny = check(&ny_new_construction_gas());
        assert!(r_ny.enforcement_stayed);

        let r_ca = check(&ca_new_construction_heat_pump());
        assert!(!r_ca.enforcement_stayed);

        let r_default = check(&default_base());
        assert!(!r_default.enforcement_stayed);
    }

    #[test]
    fn project_type_truth_table_ny() {
        for (project, exp_ban) in [
            (ProjectType::NewConstruction, true),
            (ProjectType::SubstantialRenovation, false),
            (ProjectType::AppliancePeplacement, false),
            (ProjectType::Maintenance, false),
        ] {
            let mut i = ny_new_construction_gas();
            i.project_type = project;
            let r = check(&i);
            assert_eq!(r.ban_engages, exp_ban);
        }
    }

    #[test]
    fn project_type_truth_table_ca_heat_pump_requirement() {
        for (project, exp_heat_pump) in [
            (ProjectType::NewConstruction, true),
            (ProjectType::SubstantialRenovation, false),
            (ProjectType::AppliancePeplacement, false),
            (ProjectType::Maintenance, false),
        ] {
            let mut i = ca_new_construction_heat_pump();
            i.project_type = project;
            let r = check(&i);
            assert_eq!(r.heat_pump_required, exp_heat_pump);
        }
    }

    #[test]
    fn ny_new_construction_2_violations_stack() {
        let r = check(&ny_new_construction_gas());
        assert_eq!(r.violations.len(), 2);
    }

    #[test]
    fn existing_appliance_replacement_compliant_in_all_regimes() {
        for regime in [Regime::NewYork, Regime::California, Regime::Default] {
            let mut i = ny_new_construction_gas();
            i.regime = regime;
            i.project_type = ProjectType::AppliancePeplacement;
            let r = check(&i);
            assert!(r.project_compliant);
        }
    }
}
