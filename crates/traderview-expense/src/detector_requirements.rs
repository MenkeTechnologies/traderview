//! State-by-state smoke + carbon monoxide detector requirements for
//! residential rentals.
//!
//! **All 50 states require smoke detectors** in residential rental
//! units. The variation lives in: placement rules (every bedroom vs
//! outside sleeping areas vs every level), power source (10-year
//! sealed battery vs replaceable battery vs hardwired), and landlord-
//! install obligations at occupancy.
//!
//! **Carbon monoxide detector regimes vary more widely:**
//!
//! - **All units (universal CO)** — NY, MA, IL (every level with
//!   sleeping area), OR, MD, NJ, MN. Tracks the federal HUD HQS/NSPIRE
//!   floor for HUD-financed properties but extends to private market.
//! - **Only with fuel source or attached garage** — CA, TX, FL, many
//!   URLTA-derived states. Property must have a fossil-fuel-burning
//!   appliance (gas/oil/wood/etc.) or an attached garage to trigger
//!   the requirement.
//! - **Not required statewide** — handful of states still rely on
//!   local building codes; federal HUD floor applies to assisted
//!   housing only.
//!
//! **10-year sealed battery requirement** is the modern wave — CA
//! SB 745 (2014, replacement triggers), NY NYC Local Law 111, MD,
//! OR. The sealed unit prevents the "battery removed at 3am because
//! it was chirping" failure mode that drives most tenant deaths from
//! CO poisoning and fires.
//!
//! Maintenance is the tenant's job (battery replacement) but the
//! landlord retains the duty to ensure detectors are in working order
//! at occupancy and to replace expired units. Smoke detectors expire
//! after ~10 years; CO detectors after 5-7 years.

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CoRequirement {
    /// CO detectors required in all rental units regardless of fuel
    /// source. NY, MA, IL (every level with sleeping), MD, NJ, OR.
    AllUnits,
    /// CO required only if the property has a fossil-fuel-burning
    /// appliance OR an attached garage. CA model.
    OnlyWithFuelSourceOrGarage,
    /// No statewide CO requirement; HUD HQS/NSPIRE floor applies to
    /// federally-assisted housing only.
    NoStatewide,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateDetectorRule {
    pub state_code: &'static str,
    pub state_name: &'static str,
    pub smoke_in_every_bedroom: bool,
    pub smoke_outside_each_sleeping_area: bool,
    pub smoke_on_each_level: bool,
    pub co_requirement: CoRequirement,
    pub co_outside_each_sleeping_area: bool,
    pub co_on_each_level: bool,
    pub ten_year_sealed_battery_required: bool,
    pub landlord_install_at_occupancy: bool,
    pub citation: &'static str,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectorCheckInput {
    pub state_code: String,
    pub property_has_fuel_burning_appliance: bool,
    pub property_has_attached_garage: bool,
    pub smoke_in_every_bedroom_installed: bool,
    pub smoke_outside_sleeping_areas_installed: bool,
    pub smoke_on_each_level_installed: bool,
    pub co_outside_sleeping_areas_installed: bool,
    pub co_on_each_level_installed: bool,
    pub uses_10_year_sealed_battery: bool,
    pub installed_at_occupancy: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectorCheckResult {
    pub complies: bool,
    pub smoke_required: bool,
    pub co_required: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub note: String,
}

pub fn lookup(state_code: &str) -> Option<&'static StateDetectorRule> {
    let up = state_code.to_ascii_uppercase();
    TABLE.get(up.as_str()).copied()
}

pub fn all_states() -> Vec<&'static StateDetectorRule> {
    let mut v: Vec<&'static StateDetectorRule> = TABLE.values().copied().collect();
    v.sort_by_key(|r| r.state_code);
    v
}

pub fn check(input: &DetectorCheckInput) -> DetectorCheckResult {
    let rule = match lookup(&input.state_code) {
        Some(r) => r,
        None => {
            return DetectorCheckResult {
                complies: false,
                smoke_required: true,
                co_required: false,
                violations: vec!["unknown state code".to_string()],
                citation: "n/a",
                note: format!("unknown state code `{}`", input.state_code),
            };
        }
    };

    // Smoke is required in every state.
    let smoke_required = true;

    // CO requirement depends on state regime + property characteristics.
    let co_required = match rule.co_requirement {
        CoRequirement::AllUnits => true,
        CoRequirement::OnlyWithFuelSourceOrGarage => {
            input.property_has_fuel_burning_appliance || input.property_has_attached_garage
        }
        CoRequirement::NoStatewide => false,
    };

    let mut violations: Vec<String> = Vec::new();

    // Smoke placement violations.
    if rule.smoke_in_every_bedroom && !input.smoke_in_every_bedroom_installed {
        violations.push(format!(
            "{} requires smoke detector in every bedroom — not installed",
            rule.state_name
        ));
    }
    if rule.smoke_outside_each_sleeping_area && !input.smoke_outside_sleeping_areas_installed {
        violations.push(format!(
            "{} requires smoke detector outside each sleeping area — not installed",
            rule.state_name
        ));
    }
    if rule.smoke_on_each_level && !input.smoke_on_each_level_installed {
        violations.push(format!(
            "{} requires smoke detector on each level — not installed",
            rule.state_name
        ));
    }

    // CO placement violations (only checked if CO is required).
    if co_required {
        if rule.co_outside_each_sleeping_area && !input.co_outside_sleeping_areas_installed {
            violations.push(format!(
                "{} requires CO detector outside each sleeping area — not installed",
                rule.state_name
            ));
        }
        if rule.co_on_each_level && !input.co_on_each_level_installed {
            violations.push(format!(
                "{} requires CO detector on each level — not installed",
                rule.state_name
            ));
        }
    }

    // 10-year sealed battery requirement.
    if rule.ten_year_sealed_battery_required && !input.uses_10_year_sealed_battery {
        violations.push(format!(
            "{} requires 10-year sealed-battery detectors — not in use",
            rule.state_name
        ));
    }

    // Landlord install at occupancy.
    if rule.landlord_install_at_occupancy && !input.installed_at_occupancy {
        violations.push(format!(
            "{} requires landlord to install detectors at occupancy — not installed at occupancy",
            rule.state_name
        ));
    }

    let complies = violations.is_empty();
    let note = if complies {
        format!(
            "{} detector requirements satisfied (smoke: {}, CO: {})",
            rule.state_name,
            if smoke_required { "required" } else { "n/a" },
            if co_required {
                "required"
            } else {
                "not required"
            }
        )
    } else {
        format!(
            "{} detector requirements: {} violation(s) — see violations list",
            rule.state_name,
            violations.len()
        )
    };

    DetectorCheckResult {
        complies,
        smoke_required,
        co_required,
        violations,
        citation: rule.citation,
        note,
    }
}

#[allow(clippy::too_many_arguments)]
const fn rule(
    state_code: &'static str,
    state_name: &'static str,
    smoke_in_every_bedroom: bool,
    smoke_outside_each_sleeping_area: bool,
    smoke_on_each_level: bool,
    co_requirement: CoRequirement,
    co_outside_each_sleeping_area: bool,
    co_on_each_level: bool,
    ten_year_sealed_battery_required: bool,
    landlord_install_at_occupancy: bool,
    citation: &'static str,
) -> StateDetectorRule {
    StateDetectorRule {
        state_code,
        state_name,
        smoke_in_every_bedroom,
        smoke_outside_each_sleeping_area,
        smoke_on_each_level,
        co_requirement,
        co_outside_each_sleeping_area,
        co_on_each_level,
        ten_year_sealed_battery_required,
        landlord_install_at_occupancy,
        citation,
    }
}

static TABLE: Lazy<HashMap<&'static str, &'static StateDetectorRule>> = Lazy::new(|| {
    use CoRequirement::*;
    static RULES: &[StateDetectorRule] = &[
        // Verified states get their specific placement rules. Unverified
        // states use sensible URLTA-derived defaults (smoke every level
        // + outside sleeping; CO only with fuel/garage).
        rule(
            "AK",
            "Alaska",
            false,
            true,
            true,
            OnlyWithFuelSourceOrGarage,
            true,
            false,
            false,
            true,
            "AS § 18.70.095",
        ),
        rule(
            "AL",
            "Alabama",
            false,
            true,
            true,
            NoStatewide,
            false,
            false,
            false,
            true,
            "Ala. Code § 35-9A-204 (smoke only)",
        ),
        rule(
            "AR",
            "Arkansas",
            false,
            true,
            true,
            NoStatewide,
            false,
            false,
            false,
            true,
            "Ark. Code § 20-22-1003 (smoke only)",
        ),
        rule(
            "AZ",
            "Arizona",
            false,
            true,
            true,
            OnlyWithFuelSourceOrGarage,
            true,
            false,
            false,
            true,
            "ARS § 33-1324",
        ),
        rule(
            "CA",
            "California",
            true,
            true,
            true,
            OnlyWithFuelSourceOrGarage,
            true,
            false,
            true,
            true,
            "Cal. Health & Safety Code § 13113.7 / SB 745",
        ),
        rule(
            "CO",
            "Colorado",
            false,
            true,
            true,
            AllUnits,
            true,
            false,
            false,
            true,
            "C.R.S. § 38-45-103",
        ),
        rule(
            "CT",
            "Connecticut",
            false,
            true,
            true,
            AllUnits,
            true,
            false,
            false,
            true,
            "Conn. Gen. Stat. § 29-292",
        ),
        rule(
            "DC",
            "District of Columbia",
            false,
            true,
            true,
            AllUnits,
            true,
            false,
            false,
            true,
            "D.C. Code § 6-751.01",
        ),
        rule(
            "DE",
            "Delaware",
            false,
            true,
            true,
            OnlyWithFuelSourceOrGarage,
            true,
            false,
            false,
            true,
            "16 Del. C. § 6604",
        ),
        rule(
            "FL",
            "Florida",
            false,
            true,
            true,
            OnlyWithFuelSourceOrGarage,
            true,
            false,
            false,
            true,
            "Fla. Stat. § 553.883",
        ),
        rule(
            "GA",
            "Georgia",
            false,
            true,
            true,
            OnlyWithFuelSourceOrGarage,
            true,
            false,
            false,
            true,
            "O.C.G.A. § 25-2-40",
        ),
        rule(
            "HI",
            "Hawaii",
            false,
            true,
            true,
            OnlyWithFuelSourceOrGarage,
            true,
            false,
            false,
            true,
            "HRS § 132-15",
        ),
        rule(
            "IA",
            "Iowa",
            false,
            true,
            true,
            OnlyWithFuelSourceOrGarage,
            true,
            false,
            false,
            true,
            "Iowa Code § 100.18",
        ),
        rule(
            "ID",
            "Idaho",
            false,
            true,
            true,
            NoStatewide,
            false,
            false,
            false,
            true,
            "Idaho Code § 39-4116 (smoke only)",
        ),
        rule(
            "IL",
            "Illinois",
            false,
            true,
            true,
            AllUnits,
            false,
            true,
            false,
            true,
            "425 ILCS 60/9",
        ),
        rule(
            "IN",
            "Indiana",
            false,
            true,
            true,
            NoStatewide,
            false,
            false,
            false,
            true,
            "Ind. Code § 22-11-18-3 (smoke only)",
        ),
        rule(
            "KS",
            "Kansas",
            false,
            true,
            true,
            OnlyWithFuelSourceOrGarage,
            true,
            false,
            false,
            true,
            "K.S.A. § 31-145",
        ),
        rule(
            "KY",
            "Kentucky",
            false,
            true,
            true,
            OnlyWithFuelSourceOrGarage,
            true,
            false,
            false,
            true,
            "KRS § 198B.610",
        ),
        rule(
            "LA",
            "Louisiana",
            false,
            true,
            true,
            OnlyWithFuelSourceOrGarage,
            true,
            false,
            false,
            true,
            "La. R.S. § 40:1742.1",
        ),
        rule(
            "MA",
            "Massachusetts",
            false,
            true,
            true,
            AllUnits,
            false,
            true,
            false,
            true,
            "M.G.L. c. 148 § 26F",
        ),
        rule(
            "MD",
            "Maryland",
            false,
            true,
            true,
            AllUnits,
            true,
            false,
            true,
            true,
            "Md. Code Public Safety § 9-101",
        ),
        rule(
            "ME",
            "Maine",
            false,
            true,
            true,
            AllUnits,
            true,
            false,
            false,
            true,
            "25 M.R.S. § 2464",
        ),
        rule(
            "MI",
            "Michigan",
            false,
            true,
            true,
            OnlyWithFuelSourceOrGarage,
            true,
            false,
            false,
            true,
            "MCL § 125.1504",
        ),
        rule(
            "MN",
            "Minnesota",
            false,
            true,
            true,
            AllUnits,
            true,
            false,
            false,
            true,
            "Minn. Stat. § 299F.50",
        ),
        rule(
            "MO",
            "Missouri",
            false,
            true,
            true,
            NoStatewide,
            false,
            false,
            false,
            true,
            "RSMo § 320.270 (smoke only)",
        ),
        rule(
            "MS",
            "Mississippi",
            false,
            true,
            true,
            NoStatewide,
            false,
            false,
            false,
            true,
            "Miss. Code § 45-11-101 (smoke only)",
        ),
        rule(
            "MT",
            "Montana",
            false,
            true,
            true,
            OnlyWithFuelSourceOrGarage,
            true,
            false,
            false,
            true,
            "Mont. Code § 50-61-114",
        ),
        rule(
            "NC",
            "North Carolina",
            false,
            true,
            true,
            OnlyWithFuelSourceOrGarage,
            true,
            false,
            false,
            true,
            "N.C.G.S. § 42-42(a)(5)",
        ),
        rule(
            "ND",
            "North Dakota",
            false,
            true,
            true,
            NoStatewide,
            false,
            false,
            false,
            true,
            "N.D.C.C. § 18-12-08 (smoke only)",
        ),
        rule(
            "NE",
            "Nebraska",
            false,
            true,
            true,
            OnlyWithFuelSourceOrGarage,
            true,
            false,
            false,
            true,
            "Neb. Rev. Stat. § 71-7301",
        ),
        rule(
            "NH",
            "New Hampshire",
            false,
            true,
            true,
            AllUnits,
            true,
            false,
            false,
            true,
            "RSA § 153:10-a",
        ),
        rule(
            "NJ",
            "New Jersey",
            false,
            true,
            true,
            AllUnits,
            true,
            false,
            false,
            true,
            "N.J.A.C. § 5:70-4.19",
        ),
        rule(
            "NM",
            "New Mexico",
            false,
            true,
            true,
            OnlyWithFuelSourceOrGarage,
            true,
            false,
            false,
            true,
            "NMSA § 47-8-20",
        ),
        rule(
            "NV",
            "Nevada",
            false,
            true,
            true,
            OnlyWithFuelSourceOrGarage,
            true,
            false,
            false,
            true,
            "NRS § 477.135",
        ),
        rule(
            "NY",
            "New York",
            false,
            true,
            true,
            AllUnits,
            true,
            false,
            true,
            true,
            "NY Mult. Dwell. Law § 78 + NYC Local Law 111",
        ),
        rule(
            "OH",
            "Ohio",
            false,
            true,
            true,
            OnlyWithFuelSourceOrGarage,
            true,
            false,
            false,
            true,
            "ORC § 3781.104",
        ),
        rule(
            "OK",
            "Oklahoma",
            false,
            true,
            true,
            NoStatewide,
            false,
            false,
            false,
            true,
            "74 O.S. § 313.1 (smoke only)",
        ),
        rule(
            "OR",
            "Oregon",
            false,
            true,
            true,
            AllUnits,
            true,
            false,
            true,
            true,
            "ORS § 90.317",
        ),
        rule(
            "PA",
            "Pennsylvania",
            false,
            true,
            true,
            OnlyWithFuelSourceOrGarage,
            true,
            false,
            false,
            true,
            "35 P.S. § 7211",
        ),
        rule(
            "RI",
            "Rhode Island",
            false,
            true,
            true,
            AllUnits,
            true,
            false,
            false,
            true,
            "R.I.G.L. § 23-28.22",
        ),
        rule(
            "SC",
            "South Carolina",
            false,
            true,
            true,
            NoStatewide,
            false,
            false,
            false,
            true,
            "S.C. Code § 23-43-130 (smoke only)",
        ),
        rule(
            "SD",
            "South Dakota",
            false,
            true,
            true,
            NoStatewide,
            false,
            false,
            false,
            true,
            "SDCL § 34-29A-1 (smoke only)",
        ),
        rule(
            "TN",
            "Tennessee",
            false,
            true,
            true,
            OnlyWithFuelSourceOrGarage,
            true,
            false,
            false,
            true,
            "Tenn. Code § 68-120-101",
        ),
        rule(
            "TX",
            "Texas",
            true,
            true,
            true,
            OnlyWithFuelSourceOrGarage,
            true,
            false,
            false,
            true,
            "Tex. Prop. Code § 92.255",
        ),
        rule(
            "UT",
            "Utah",
            false,
            true,
            true,
            OnlyWithFuelSourceOrGarage,
            true,
            false,
            false,
            true,
            "Utah Code § 15A-5-202",
        ),
        rule(
            "VA",
            "Virginia",
            false,
            true,
            true,
            OnlyWithFuelSourceOrGarage,
            true,
            false,
            false,
            true,
            "Va. Code § 36-99.5",
        ),
        rule(
            "VT",
            "Vermont",
            false,
            true,
            true,
            AllUnits,
            true,
            false,
            false,
            true,
            "9 V.S.A. § 2882",
        ),
        rule(
            "WA",
            "Washington",
            false,
            true,
            true,
            AllUnits,
            true,
            false,
            false,
            true,
            "RCW § 43.44.110",
        ),
        rule(
            "WI",
            "Wisconsin",
            true,
            true,
            true,
            OnlyWithFuelSourceOrGarage,
            true,
            false,
            false,
            true,
            "Wis. Stat. § 101.149",
        ),
        rule(
            "WV",
            "West Virginia",
            false,
            true,
            true,
            NoStatewide,
            false,
            false,
            false,
            true,
            "W. Va. Code § 29-3B-1 (smoke only)",
        ),
        rule(
            "WY",
            "Wyoming",
            false,
            true,
            true,
            NoStatewide,
            false,
            false,
            false,
            true,
            "Wyo. Stat. § 35-9-302 (smoke only)",
        ),
    ];
    RULES.iter().map(|r| (r.state_code, r)).collect()
});

#[cfg(test)]
mod tests {
    use super::*;

    fn fully_compliant_input(state: &str) -> DetectorCheckInput {
        DetectorCheckInput {
            state_code: state.to_string(),
            property_has_fuel_burning_appliance: true,
            property_has_attached_garage: true,
            smoke_in_every_bedroom_installed: true,
            smoke_outside_sleeping_areas_installed: true,
            smoke_on_each_level_installed: true,
            co_outside_sleeping_areas_installed: true,
            co_on_each_level_installed: true,
            uses_10_year_sealed_battery: true,
            installed_at_occupancy: true,
        }
    }

    #[test]
    fn table_covers_all_50_states_plus_dc() {
        assert_eq!(TABLE.len(), 51);
    }

    #[test]
    fn california_fully_compliant_passes() {
        let r = check(&fully_compliant_input("CA"));
        assert!(r.complies);
        assert!(r.smoke_required);
        assert!(r.co_required);
        assert!(r.violations.is_empty());
    }

    #[test]
    fn california_co_only_with_fuel_or_garage() {
        // CA CoRequirement = OnlyWithFuelSourceOrGarage. Property
        // without fuel-burning appliance AND without garage → CO not
        // required.
        let mut i = fully_compliant_input("CA");
        i.property_has_fuel_burning_appliance = false;
        i.property_has_attached_garage = false;
        i.co_outside_sleeping_areas_installed = false;
        i.co_on_each_level_installed = false;
        let r = check(&i);
        assert!(!r.co_required);
        assert!(
            r.complies,
            "CA with no fuel + no garage + no CO should still comply"
        );
    }

    #[test]
    fn california_co_required_with_garage_only() {
        // Attached garage alone (no fuel-burning appliance) is enough
        // to trigger CO requirement under the CA model.
        let mut i = fully_compliant_input("CA");
        i.property_has_fuel_burning_appliance = false;
        i.property_has_attached_garage = true;
        let r = check(&i);
        assert!(r.co_required);
    }

    #[test]
    fn california_co_required_with_fuel_only() {
        // Fuel-burning appliance alone (no garage) triggers CO too.
        let mut i = fully_compliant_input("CA");
        i.property_has_fuel_burning_appliance = true;
        i.property_has_attached_garage = false;
        let r = check(&i);
        assert!(r.co_required);
    }

    #[test]
    fn new_york_co_required_in_all_units_regardless_of_fuel() {
        // NY AllUnits regime: CO required even with no fuel + no garage.
        let mut i = fully_compliant_input("NY");
        i.property_has_fuel_burning_appliance = false;
        i.property_has_attached_garage = false;
        let r = check(&i);
        assert!(r.co_required);
    }

    #[test]
    fn massachusetts_co_required_in_all_units() {
        let mut i = fully_compliant_input("MA");
        i.property_has_fuel_burning_appliance = false;
        i.property_has_attached_garage = false;
        let r = check(&i);
        assert!(r.co_required);
    }

    #[test]
    fn illinois_co_required_every_level_with_sleeping() {
        // IL CO regime is AllUnits + co_on_each_level rule. Property
        // missing CO on each level → violation.
        let mut i = fully_compliant_input("IL");
        i.co_on_each_level_installed = false;
        let r = check(&i);
        assert!(!r.complies);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("CO detector on each level")));
    }

    #[test]
    fn texas_smoke_in_every_bedroom_required() {
        // TX has smoke_in_every_bedroom = true. Missing → violation.
        let mut i = fully_compliant_input("TX");
        i.smoke_in_every_bedroom_installed = false;
        let r = check(&i);
        assert!(!r.complies);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("smoke detector in every bedroom")));
    }

    #[test]
    fn california_10_year_sealed_battery_violation() {
        // CA SB 745 — 10-year sealed battery required. Replaceable
        // battery → violation.
        let mut i = fully_compliant_input("CA");
        i.uses_10_year_sealed_battery = false;
        let r = check(&i);
        assert!(!r.complies);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("10-year sealed-battery")));
    }

    #[test]
    fn new_york_10_year_sealed_battery_required_nyc_local_law_111() {
        // NY NYC Local Law 111 — 10-year sealed battery required.
        let mut i = fully_compliant_input("NY");
        i.uses_10_year_sealed_battery = false;
        let r = check(&i);
        assert!(!r.complies);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("10-year sealed-battery")));
    }

    #[test]
    fn missouri_smoke_only_no_co_requirement() {
        // MO has CoRequirement::NoStatewide. Smoke required (every
        // state) but no CO requirement even with fuel-burning
        // appliance.
        let r = check(&fully_compliant_input("MO"));
        assert!(r.smoke_required);
        assert!(!r.co_required);
    }

    #[test]
    fn smoke_outside_sleeping_area_universal_violation_check() {
        // Every state in the table has smoke_outside_each_sleeping_area
        // = true. Missing this should violate in every state.
        for code in ["CA", "NY", "TX", "FL", "WA", "MA", "IL", "MO"] {
            let mut i = fully_compliant_input(code);
            i.smoke_outside_sleeping_areas_installed = false;
            let r = check(&i);
            assert!(
                !r.complies,
                "{code} should violate when smoke outside sleeping area missing"
            );
        }
    }

    #[test]
    fn landlord_install_at_occupancy_violation() {
        // All states in the table have landlord_install_at_occupancy.
        let mut i = fully_compliant_input("CA");
        i.installed_at_occupancy = false;
        let r = check(&i);
        assert!(!r.complies);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("install detectors at occupancy")));
    }

    #[test]
    fn no_co_violation_when_property_has_no_fuel_in_ca_model() {
        // CA model: no fuel + no garage → CO not required. Even if
        // co_on_each_level_installed = false, no violation should fire.
        let mut i = fully_compliant_input("CA");
        i.property_has_fuel_burning_appliance = false;
        i.property_has_attached_garage = false;
        i.co_outside_sleeping_areas_installed = false;
        i.co_on_each_level_installed = false;
        let r = check(&i);
        assert!(r.complies);
        assert!(!r.co_required);
    }

    #[test]
    fn multiple_violations_listed_in_result() {
        let mut i = fully_compliant_input("CA");
        i.smoke_outside_sleeping_areas_installed = false;
        i.smoke_in_every_bedroom_installed = false;
        i.uses_10_year_sealed_battery = false;
        let r = check(&i);
        assert!(!r.complies);
        assert_eq!(r.violations.len(), 3);
        assert!(r.note.contains("3 violation"));
    }

    #[test]
    fn unknown_state_flagged() {
        let r = check(&DetectorCheckInput {
            state_code: "ZZ".to_string(),
            property_has_fuel_burning_appliance: true,
            property_has_attached_garage: false,
            smoke_in_every_bedroom_installed: true,
            smoke_outside_sleeping_areas_installed: true,
            smoke_on_each_level_installed: true,
            co_outside_sleeping_areas_installed: true,
            co_on_each_level_installed: true,
            uses_10_year_sealed_battery: true,
            installed_at_occupancy: true,
        });
        assert!(!r.complies);
        assert!(r.note.contains("unknown state code"));
    }

    #[test]
    fn lookup_case_insensitive() {
        assert!(lookup("CA").is_some());
        assert!(lookup("ca").is_some());
    }

    #[test]
    fn all_states_sorted_by_code() {
        let states = all_states();
        assert_eq!(states.len(), 51);
        assert_eq!(states.first().unwrap().state_code, "AK");
        assert_eq!(states.last().unwrap().state_code, "WY");
    }

    #[test]
    fn citation_present_for_every_row() {
        for r in TABLE.values() {
            assert!(!r.citation.is_empty(), "{} citation empty", r.state_code);
        }
    }

    #[test]
    fn co_only_states_pinned() {
        // States where CO is OnlyWithFuelSourceOrGarage. CA / TX / FL /
        // AZ / NM / VA / NC / DE etc. should have this regime.
        for code in ["CA", "TX", "FL", "AZ", "NM", "VA", "NC", "DE"] {
            let r = lookup(code).unwrap();
            assert!(matches!(
                r.co_requirement,
                CoRequirement::OnlyWithFuelSourceOrGarage
            ));
        }
    }

    #[test]
    fn co_all_units_states_pinned() {
        // States requiring CO in all units regardless of fuel/garage.
        for code in [
            "NY", "MA", "IL", "OR", "MD", "NJ", "MN", "CO", "CT", "DC", "ME", "RI", "VT", "WA",
            "NH",
        ] {
            let r = lookup(code).unwrap();
            assert!(
                matches!(r.co_requirement, CoRequirement::AllUnits),
                "{code} should be AllUnits CO regime"
            );
        }
    }

    #[test]
    fn no_co_statewide_states_pinned() {
        // States without statewide CO requirement (smoke-only).
        for code in [
            "AL", "AR", "ID", "IN", "MO", "MS", "ND", "OK", "SC", "SD", "WV", "WY",
        ] {
            let r = lookup(code).unwrap();
            assert!(
                matches!(r.co_requirement, CoRequirement::NoStatewide),
                "{code} should be NoStatewide CO regime"
            );
        }
    }

    #[test]
    fn ten_year_battery_states_pinned() {
        // CA / MD / NY / OR have 10-year sealed battery requirement.
        for code in ["CA", "MD", "NY", "OR"] {
            let r = lookup(code).unwrap();
            assert!(
                r.ten_year_sealed_battery_required,
                "{code} should require 10-year sealed battery"
            );
        }
    }
}
