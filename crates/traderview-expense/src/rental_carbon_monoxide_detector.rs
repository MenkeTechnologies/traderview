//! Rental property carbon monoxide (CO) detector compliance
//! — when a trader-landlord must install, maintain, and
//! certify CO alarms in dwelling units with fossil-fuel-
//! burning appliances or attached garages. Trader-landlord
//! operational concern: missing or non-functional CO alarms
//! create criminal-misdemeanor exposure (IL Class B), civil
//! penalties, and wrongful-death liability when CO poisoning
//! occurs. Distinct from siblings `rental_bed_bug_disclosure`
//! (lease disclosure), `rental_hot_water_temperature`
//! (habitability minimums), `tenant_fire_safety_plan_
//! disclosure` (fire safety), `rental_bedroom_egress_window`
//! (structural), `rental_gas_appliance_ban` (electrification).
//!
//! **Four regimes**:
//!
//! **California — SB 183 of 2010 (Cal. Health & Safety Code
//! §§ 13260-13263, Carbon Monoxide Poisoning Prevention
//! Act)**:
//! - Compliance deadlines: single-family with fossil-fuel
//!   appliance OR attached garage by July 1, 2011;
//!   multifamily dwellings by January 1, 2013.
//! - CO alarm installed and maintained outside each sleeping
//!   area AND on every level (including basements).
//! - State Fire Marshal-certified device required.
//! - Landlord liability: tenant may pursue actual damages
//!   plus statutory damages up to $100 per violation.
//!
//! **New York — Amanda's Law (NY Exec. Law § 378(5-a),
//! eff. February 22, 2010)**:
//! - CO alarm in EVERY one-and-two-family dwelling and
//!   multiple dwelling where carbon-monoxide source is
//!   present (fossil-fuel appliance OR attached garage).
//! - Alarm placed **within 15 feet** of each sleeping area
//!   entrance.
//! - UL 2034 listed alarm required.
//!
//! **Illinois — Carbon Monoxide Alarm Detector Act (430
//! ILCS 135/1 et seq., eff. January 1, 2007)**:
//! - CO detector **within 15 feet** of every sleeping room
//!   in dwelling unit with fossil-fuel-burning appliance OR
//!   attached garage.
//! - Failure to install is a **Class B misdemeanor**.
//! - UL 2034 listed alarm required.
//!
//! **Massachusetts — Nicole's Law (M.G.L. c. 148 § 26F½,
//! eff. March 31, 2006)**:
//! - Strictest among comparators: **interconnected
//!   (hardwired or wireless) CO alarms required on every
//!   level**.
//! - Within 10 feet of each bedroom door.
//! - **Certificate of compliance from local fire department
//!   required before selling or renting** the dwelling.
//! - UL 2034 listed alarm required.
//!
//! Citations: Cal. Health & Safety Code §§ 13260-13263
//! (SB 183 of 2010); NY Exec. Law § 378(5-a) (Amanda's
//! Law); 430 ILCS 135/1 et seq. (IL CO Alarm Detector Act);
//! M.G.L. c. 148 § 26F½ (MA Nicole's Law); UL 2034.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    California,
    NewYork,
    Illinois,
    Massachusetts,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PowerSource {
    /// Battery-only (not interconnected).
    BatteryOnly,
    /// Hardwired with battery backup, not interconnected.
    HardwiredNotInterconnected,
    /// Interconnected hardwired (Massachusetts strictest).
    InterconnectedHardwired,
    /// Interconnected wireless (Massachusetts strictest).
    InterconnectedWireless,
    /// No alarm installed.
    NoAlarm,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RentalCarbonMonoxideDetectorInput {
    pub regime: Regime,
    /// Whether dwelling has fossil-fuel-burning appliance OR
    /// attached garage (triggers CO alarm requirement).
    pub has_co_source: bool,
    /// Whether CO alarm is installed.
    pub alarm_installed: bool,
    /// Power source / interconnection type.
    pub power_source: PowerSource,
    /// Whether alarm is UL 2034 listed (NY + IL + MA).
    pub ul_2034_listed: bool,
    /// Whether alarm is State Fire Marshal certified (CA).
    pub ca_state_fire_marshal_certified: bool,
    /// Distance from sleeping area entrance in feet.
    pub distance_from_sleeping_area_feet: u32,
    /// Whether alarm is on every level of the dwelling.
    pub alarm_on_every_level: bool,
    /// Whether MA certificate of compliance was obtained from
    /// local fire department before renting.
    pub ma_fire_dept_certificate_obtained: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RentalCarbonMonoxideDetectorResult {
    pub installation_compliant: bool,
    pub distance_compliant: bool,
    pub certification_compliant: bool,
    pub interconnection_compliant: bool,
    pub fire_dept_certificate_compliant: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &RentalCarbonMonoxideDetectorInput) -> RentalCarbonMonoxideDetectorResult {
    match input.regime {
        Regime::California => check_ca(input),
        Regime::NewYork => check_ny(input),
        Regime::Illinois => check_il(input),
        Regime::Massachusetts => check_ma(input),
    }
}

fn check_ca(input: &RentalCarbonMonoxideDetectorInput) -> RentalCarbonMonoxideDetectorResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "Cal. Health & Safety Code §§ 13260-13263 (SB 183 of 2010, Carbon Monoxide Poisoning Prevention Act) — CO alarm required in all dwelling units intended for human occupancy with fossil-fuel-burning appliance or attached garage".to_string(),
        "Cal. Health & Safety Code § 13262 — CO alarm installed and maintained outside each sleeping area AND on every level of the dwelling unit including basements".to_string(),
        "Cal. Health & Safety Code § 13261 — State Fire Marshal-certified device required".to_string(),
        "Cal. SB 183 of 2010 — compliance deadlines: single-family with fossil-fuel appliance OR attached garage by July 1, 2011; multifamily dwellings by January 1, 2013".to_string(),
        "Cal. Health & Safety Code § 13263 — landlord liability: tenant may pursue actual damages plus statutory damages up to $100 per violation".to_string(),
    ];

    if input.has_co_source && !input.alarm_installed {
        violations.push(
            "Cal. Health & Safety Code §§ 13260-13263 (SB 183) — CO alarm required in dwelling with fossil-fuel-burning appliance or attached garage".to_string(),
        );
    }

    if input.has_co_source
        && input.alarm_installed
        && !input.ca_state_fire_marshal_certified
    {
        violations.push(
            "Cal. Health & Safety Code § 13261 — CO alarm must be State Fire Marshal certified".to_string(),
        );
    }

    if input.has_co_source && input.alarm_installed && !input.alarm_on_every_level {
        violations.push(
            "Cal. Health & Safety Code § 13262 — CO alarm required on every level of dwelling including basements".to_string(),
        );
    }

    RentalCarbonMonoxideDetectorResult {
        installation_compliant: !input.has_co_source || input.alarm_installed,
        distance_compliant: true,
        certification_compliant: input.ca_state_fire_marshal_certified || !input.has_co_source,
        interconnection_compliant: true,
        fire_dept_certificate_compliant: true,
        violations,
        citation: "Cal. Health & Safety Code §§ 13260-13263 (SB 183 of 2010)",
        notes,
    }
    .with_overall()
}

fn check_ny(input: &RentalCarbonMonoxideDetectorInput) -> RentalCarbonMonoxideDetectorResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "NY Exec. Law § 378(5-a) (Amanda's Law, effective February 22, 2010) — CO alarm required in every one-and-two-family dwelling and multiple dwelling where carbon-monoxide source is present".to_string(),
        "NY Exec. Law § 378(5-a) — alarm must be installed within 15 feet of each sleeping area entrance".to_string(),
        "NY Exec. Law § 378(5-a) — UL 2034 listed alarm required".to_string(),
        "Amanda's Law named for Amanda Hansen who died from CO poisoning at sleepover in 2009; statewide CO alarm mandate enacted in response".to_string(),
    ];

    if input.has_co_source && !input.alarm_installed {
        violations.push(
            "NY Exec. Law § 378(5-a) (Amanda's Law) — CO alarm required where CO source is present".to_string(),
        );
    }

    if input.has_co_source && input.alarm_installed && !input.ul_2034_listed {
        violations.push(
            "NY Exec. Law § 378(5-a) — UL 2034 listed CO alarm required".to_string(),
        );
    }

    let distance_compliant = !input.alarm_installed
        || input.distance_from_sleeping_area_feet <= 15;
    if input.has_co_source && input.alarm_installed && !distance_compliant {
        violations.push(
            "NY Exec. Law § 378(5-a) — CO alarm must be within 15 feet of each sleeping area entrance".to_string(),
        );
    }

    RentalCarbonMonoxideDetectorResult {
        installation_compliant: !input.has_co_source || input.alarm_installed,
        distance_compliant,
        certification_compliant: input.ul_2034_listed || !input.has_co_source,
        interconnection_compliant: true,
        fire_dept_certificate_compliant: true,
        violations,
        citation: "NY Exec. Law § 378(5-a) (Amanda's Law)",
        notes,
    }
    .with_overall()
}

fn check_il(input: &RentalCarbonMonoxideDetectorInput) -> RentalCarbonMonoxideDetectorResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "430 ILCS 135/1 et seq. (Illinois Carbon Monoxide Alarm Detector Act, effective January 1, 2007) — CO detector required within 15 feet of every sleeping room in dwelling unit with fossil-fuel-burning appliance or attached garage".to_string(),
        "430 ILCS 135/15 — failure to install CO detector is a Class B misdemeanor".to_string(),
        "430 ILCS 135/10 — UL 2034 listed CO detector required".to_string(),
        "430 ILCS 135/5 — landlord responsible for installation; tenant responsible for routine maintenance and battery replacement (similar to smoke alarm framework)".to_string(),
    ];

    if input.has_co_source && !input.alarm_installed {
        violations.push(
            "430 ILCS 135/10 — CO detector required in dwelling with fossil-fuel-burning appliance or attached garage; failure constitutes Class B misdemeanor under 430 ILCS 135/15".to_string(),
        );
    }

    if input.has_co_source && input.alarm_installed && !input.ul_2034_listed {
        violations.push(
            "430 ILCS 135/10 — UL 2034 listed CO detector required".to_string(),
        );
    }

    let distance_compliant = !input.alarm_installed
        || input.distance_from_sleeping_area_feet <= 15;
    if input.has_co_source && input.alarm_installed && !distance_compliant {
        violations.push(
            "430 ILCS 135/10 — CO detector must be within 15 feet of every sleeping room".to_string(),
        );
    }

    RentalCarbonMonoxideDetectorResult {
        installation_compliant: !input.has_co_source || input.alarm_installed,
        distance_compliant,
        certification_compliant: input.ul_2034_listed || !input.has_co_source,
        interconnection_compliant: true,
        fire_dept_certificate_compliant: true,
        violations,
        citation: "430 ILCS 135/1 et seq. (Illinois Carbon Monoxide Alarm Detector Act)",
        notes,
    }
    .with_overall()
}

fn check_ma(input: &RentalCarbonMonoxideDetectorInput) -> RentalCarbonMonoxideDetectorResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "M.G.L. c. 148 § 26F½ (Nicole's Law, effective March 31, 2006) — interconnected (hardwired or wireless) CO alarms required on every level of dwelling".to_string(),
        "M.G.L. c. 148 § 26F½ — alarm placed within 10 feet of each bedroom door".to_string(),
        "M.G.L. c. 148 § 26F½ — certificate of compliance from local fire department REQUIRED BEFORE selling or renting dwelling (strictest among comparators)".to_string(),
        "M.G.L. c. 148 § 26F½ — UL 2034 listed CO alarm required".to_string(),
        "Nicole's Law named for Nicole Garofalo who died of CO poisoning in 2005; Massachusetts strictest CO alarm framework among comparators".to_string(),
    ];

    if input.has_co_source && !input.alarm_installed {
        violations.push(
            "M.G.L. c. 148 § 26F½ (Nicole's Law) — CO alarm required in dwelling with fossil-fuel-burning appliance or attached garage".to_string(),
        );
    }

    let interconnected = matches!(
        input.power_source,
        PowerSource::InterconnectedHardwired | PowerSource::InterconnectedWireless
    );
    if input.has_co_source && input.alarm_installed && !interconnected {
        violations.push(
            "M.G.L. c. 148 § 26F½ — alarms must be interconnected (hardwired or wireless) so activation of one triggers all".to_string(),
        );
    }

    if input.has_co_source && input.alarm_installed && !input.alarm_on_every_level {
        violations.push(
            "M.G.L. c. 148 § 26F½ — interconnected CO alarms required on every level of dwelling".to_string(),
        );
    }

    let distance_compliant = !input.alarm_installed
        || input.distance_from_sleeping_area_feet <= 10;
    if input.has_co_source && input.alarm_installed && !distance_compliant {
        violations.push(
            "M.G.L. c. 148 § 26F½ — CO alarm must be within 10 feet of each bedroom door (stricter than NY/IL 15-foot rule)".to_string(),
        );
    }

    if input.has_co_source && input.alarm_installed && !input.ul_2034_listed {
        violations.push(
            "M.G.L. c. 148 § 26F½ — UL 2034 listed CO alarm required".to_string(),
        );
    }

    if input.has_co_source && !input.ma_fire_dept_certificate_obtained {
        violations.push(
            "M.G.L. c. 148 § 26F½ — certificate of compliance from local fire department REQUIRED BEFORE selling or renting dwelling".to_string(),
        );
    }

    RentalCarbonMonoxideDetectorResult {
        installation_compliant: !input.has_co_source || input.alarm_installed,
        distance_compliant,
        certification_compliant: input.ul_2034_listed || !input.has_co_source,
        interconnection_compliant: interconnected || !input.has_co_source,
        fire_dept_certificate_compliant: input.ma_fire_dept_certificate_obtained
            || !input.has_co_source,
        violations,
        citation: "M.G.L. c. 148 § 26F½ (Nicole's Law)",
        notes,
    }
    .with_overall()
}

impl RentalCarbonMonoxideDetectorResult {
    fn with_overall(self) -> Self {
        let overall_compliant = self.installation_compliant
            && self.distance_compliant
            && self.certification_compliant
            && self.interconnection_compliant
            && self.fire_dept_certificate_compliant
            && self.violations.is_empty();
        Self {
            installation_compliant: overall_compliant,
            distance_compliant: self.distance_compliant,
            certification_compliant: self.certification_compliant,
            interconnection_compliant: self.interconnection_compliant,
            fire_dept_certificate_compliant: self.fire_dept_certificate_compliant,
            violations: self.violations,
            citation: self.citation,
            notes: self.notes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ca_clean() -> RentalCarbonMonoxideDetectorInput {
        RentalCarbonMonoxideDetectorInput {
            regime: Regime::California,
            has_co_source: true,
            alarm_installed: true,
            power_source: PowerSource::BatteryOnly,
            ul_2034_listed: false,
            ca_state_fire_marshal_certified: true,
            distance_from_sleeping_area_feet: 10,
            alarm_on_every_level: true,
            ma_fire_dept_certificate_obtained: false,
        }
    }

    fn ny_clean() -> RentalCarbonMonoxideDetectorInput {
        let mut i = ca_clean();
        i.regime = Regime::NewYork;
        i.ca_state_fire_marshal_certified = false;
        i.ul_2034_listed = true;
        i.distance_from_sleeping_area_feet = 15;
        i
    }

    fn il_clean() -> RentalCarbonMonoxideDetectorInput {
        let mut i = ny_clean();
        i.regime = Regime::Illinois;
        i
    }

    fn ma_clean() -> RentalCarbonMonoxideDetectorInput {
        let mut i = ca_clean();
        i.regime = Regime::Massachusetts;
        i.ca_state_fire_marshal_certified = false;
        i.ul_2034_listed = true;
        i.power_source = PowerSource::InterconnectedHardwired;
        i.distance_from_sleeping_area_feet = 10;
        i.ma_fire_dept_certificate_obtained = true;
        i
    }

    #[test]
    fn ca_clean_compliant() {
        let r = check(&ca_clean());
        assert!(r.installation_compliant);
        assert!(r.violations.is_empty());
    }

    #[test]
    fn ca_no_alarm_with_co_source_violation() {
        let mut i = ca_clean();
        i.alarm_installed = false;
        let r = check(&i);
        assert!(!r.installation_compliant);
        assert!(r.violations.iter().any(|v| v.contains("SB 183")));
    }

    #[test]
    fn ca_no_co_source_no_alarm_compliant() {
        let mut i = ca_clean();
        i.has_co_source = false;
        i.alarm_installed = false;
        let r = check(&i);
        assert!(r.installation_compliant);
    }

    #[test]
    fn ca_missing_state_fire_marshal_certification_violation() {
        let mut i = ca_clean();
        i.ca_state_fire_marshal_certified = false;
        let r = check(&i);
        assert!(!r.installation_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 13261") && v.contains("State Fire Marshal")));
    }

    #[test]
    fn ca_not_on_every_level_violation() {
        let mut i = ca_clean();
        i.alarm_on_every_level = false;
        let r = check(&i);
        assert!(!r.installation_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 13262") && v.contains("every level")));
    }

    #[test]
    fn ny_clean_compliant() {
        let r = check(&ny_clean());
        assert!(r.installation_compliant);
    }

    #[test]
    fn ny_no_alarm_with_co_source_violation() {
        let mut i = ny_clean();
        i.alarm_installed = false;
        let r = check(&i);
        assert!(!r.installation_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 378(5-a)") && v.contains("Amanda")));
    }

    #[test]
    fn ny_not_ul_2034_listed_violation() {
        let mut i = ny_clean();
        i.ul_2034_listed = false;
        let r = check(&i);
        assert!(!r.installation_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("UL 2034")));
    }

    #[test]
    fn ny_15_foot_boundary_compliant() {
        let mut i = ny_clean();
        i.distance_from_sleeping_area_feet = 15;
        let r = check(&i);
        assert!(r.distance_compliant);
    }

    #[test]
    fn ny_16_foot_distance_violation() {
        let mut i = ny_clean();
        i.distance_from_sleeping_area_feet = 16;
        let r = check(&i);
        assert!(!r.distance_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("15 feet")));
    }

    #[test]
    fn il_clean_compliant() {
        let r = check(&il_clean());
        assert!(r.installation_compliant);
    }

    #[test]
    fn il_no_alarm_class_b_misdemeanor_violation() {
        let mut i = il_clean();
        i.alarm_installed = false;
        let r = check(&i);
        assert!(!r.installation_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("430 ILCS") && v.contains("Class B misdemeanor")));
    }

    #[test]
    fn il_15_foot_boundary_compliant() {
        let mut i = il_clean();
        i.distance_from_sleeping_area_feet = 15;
        let r = check(&i);
        assert!(r.distance_compliant);
    }

    #[test]
    fn il_16_foot_distance_violation() {
        let mut i = il_clean();
        i.distance_from_sleeping_area_feet = 16;
        let r = check(&i);
        assert!(!r.distance_compliant);
    }

    #[test]
    fn ma_clean_compliant() {
        let r = check(&ma_clean());
        assert!(r.installation_compliant);
        assert!(r.fire_dept_certificate_compliant);
        assert!(r.interconnection_compliant);
    }

    #[test]
    fn ma_battery_only_not_interconnected_violation() {
        let mut i = ma_clean();
        i.power_source = PowerSource::BatteryOnly;
        let r = check(&i);
        assert!(!r.interconnection_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("interconnected")));
    }

    #[test]
    fn ma_hardwired_not_interconnected_violation() {
        let mut i = ma_clean();
        i.power_source = PowerSource::HardwiredNotInterconnected;
        let r = check(&i);
        assert!(!r.interconnection_compliant);
    }

    #[test]
    fn ma_interconnected_wireless_compliant() {
        let mut i = ma_clean();
        i.power_source = PowerSource::InterconnectedWireless;
        let r = check(&i);
        assert!(r.interconnection_compliant);
    }

    #[test]
    fn ma_10_foot_boundary_compliant() {
        let mut i = ma_clean();
        i.distance_from_sleeping_area_feet = 10;
        let r = check(&i);
        assert!(r.distance_compliant);
    }

    #[test]
    fn ma_11_foot_violation() {
        let mut i = ma_clean();
        i.distance_from_sleeping_area_feet = 11;
        let r = check(&i);
        assert!(!r.distance_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("10 feet") && v.contains("bedroom door")));
    }

    #[test]
    fn ma_missing_fire_dept_certificate_violation() {
        let mut i = ma_clean();
        i.ma_fire_dept_certificate_obtained = false;
        let r = check(&i);
        assert!(!r.fire_dept_certificate_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("certificate of compliance") && v.contains("fire department")));
    }

    #[test]
    fn ma_strictest_10_foot_distance_invariant() {
        let mut i_ny = ny_clean();
        i_ny.distance_from_sleeping_area_feet = 11;
        let r_ny = check(&i_ny);
        assert!(r_ny.distance_compliant);

        let mut i_ma = ma_clean();
        i_ma.distance_from_sleeping_area_feet = 11;
        let r_ma = check(&i_ma);
        assert!(!r_ma.distance_compliant);
    }

    #[test]
    fn ma_uniquely_requires_interconnection_invariant() {
        let mut i_ca = ca_clean();
        i_ca.power_source = PowerSource::BatteryOnly;
        let r_ca = check(&i_ca);
        assert!(r_ca.interconnection_compliant);

        let mut i_ma = ma_clean();
        i_ma.power_source = PowerSource::BatteryOnly;
        let r_ma = check(&i_ma);
        assert!(!r_ma.interconnection_compliant);
    }

    #[test]
    fn ma_uniquely_requires_fire_dept_certificate_invariant() {
        let mut i_ny = ny_clean();
        i_ny.ma_fire_dept_certificate_obtained = false;
        let r_ny = check(&i_ny);
        assert!(r_ny.fire_dept_certificate_compliant);

        let mut i_ma = ma_clean();
        i_ma.ma_fire_dept_certificate_obtained = false;
        let r_ma = check(&i_ma);
        assert!(!r_ma.fire_dept_certificate_compliant);
    }

    #[test]
    fn citation_pins_ca_authority() {
        let r = check(&ca_clean());
        assert!(r.citation.contains("§§ 13260-13263"));
        assert!(r.citation.contains("SB 183"));
    }

    #[test]
    fn citation_pins_ny_authority() {
        let r = check(&ny_clean());
        assert!(r.citation.contains("§ 378(5-a)"));
        assert!(r.citation.contains("Amanda"));
    }

    #[test]
    fn citation_pins_il_authority() {
        let r = check(&il_clean());
        assert!(r.citation.contains("430 ILCS"));
        assert!(r.citation.contains("Carbon Monoxide"));
    }

    #[test]
    fn citation_pins_ma_authority() {
        let r = check(&ma_clean());
        assert!(r.citation.contains("c. 148 § 26F"));
        assert!(r.citation.contains("Nicole"));
    }

    #[test]
    fn note_pins_ca_july_2011_january_2013_deadlines() {
        let r = check(&ca_clean());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("July 1, 2011") && n.contains("January 1, 2013")));
    }

    #[test]
    fn note_pins_ny_amanda_hansen_origin() {
        let r = check(&ny_clean());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Amanda Hansen") && n.contains("CO poisoning")));
    }

    #[test]
    fn note_pins_il_class_b_misdemeanor() {
        let r = check(&il_clean());
        assert!(r.notes.iter().any(|n| n.contains("Class B misdemeanor")));
    }

    #[test]
    fn note_pins_ma_nicole_garofalo_origin() {
        let r = check(&ma_clean());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Nicole Garofalo") && n.contains("strictest")));
    }

    #[test]
    fn no_co_source_no_violations_all_regimes() {
        for regime in [
            Regime::California,
            Regime::NewYork,
            Regime::Illinois,
            Regime::Massachusetts,
        ] {
            let mut i = match regime {
                Regime::California => ca_clean(),
                Regime::NewYork => ny_clean(),
                Regime::Illinois => il_clean(),
                Regime::Massachusetts => ma_clean(),
            };
            i.has_co_source = false;
            i.alarm_installed = false;
            let r = check(&i);
            assert!(r.installation_compliant);
        }
    }

    #[test]
    fn power_source_truth_table_for_ma() {
        for (ps, exp_interconnected) in [
            (PowerSource::BatteryOnly, false),
            (PowerSource::HardwiredNotInterconnected, false),
            (PowerSource::InterconnectedHardwired, true),
            (PowerSource::InterconnectedWireless, true),
        ] {
            let mut i = ma_clean();
            i.power_source = ps;
            let r = check(&i);
            assert_eq!(
                r.interconnection_compliant, exp_interconnected,
                "ps={:?} expected interconnected={}",
                ps, exp_interconnected
            );
        }
    }

    #[test]
    fn multiple_ma_violations_stack() {
        let mut i = ma_clean();
        i.power_source = PowerSource::BatteryOnly;
        i.distance_from_sleeping_area_feet = 20;
        i.ma_fire_dept_certificate_obtained = false;
        i.ul_2034_listed = false;
        i.alarm_on_every_level = false;
        let r = check(&i);
        assert!(r.violations.len() >= 4);
    }
}
