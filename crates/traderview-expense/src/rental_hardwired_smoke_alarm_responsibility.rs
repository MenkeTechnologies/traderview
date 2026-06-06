//! Multi-jurisdictional rental property HARDWIRED SMOKE
//! ALARM installation, maintenance, and operability
//! compliance framework. When a landlord rents a property,
//! what smoke alarm technology (hardwired with battery
//! backup vs 10-year sealed battery), installation
//! locations (every bedroom + hallway outside sleeping
//! areas + each floor including basement), and post-
//! tenancy verification duties apply, and what failure-
//! mode liabilities expose landlord after a fire injury
//! or fatality?
//!
//! Distinct from sibling modules: rental_carbon_monoxide_
//! detector (CO alarm), rental_chimney_fireplace_
//! inspection_disclosure (iter 471), rental_fire_
//! extinguisher_requirement (iter 473), rental_window_
//! blind_cord_safety (iter 469), rental_bedroom_egress_
//! window (structural), rental_smoke_free_housing_
//! disclosure (smoking policy not alarms), tenant_fire_
//! safety_plan_disclosure.
//!
//! Three-jurisdiction framework:
//!
//! 1. CALIFORNIA (most prescriptive) — Cal. Health &
//!    Safety Code § 13113.7 + § 13113.8 + Cal. Civ. Code
//!    § 1941.1(a)(7) implied warranty of habitability:
//!    - 2014 SEALED-BATTERY REQUIREMENT: all battery-
//!      operated smoke alarms must have a non-replaceable
//!      battery with at least 10-YEAR LIFESPAN per State
//!      Fire Marshal regulations
//!    - JANUARY 1, 2016 HARDWIRED MANDATE: rental
//!      property owners must install additional smoke
//!      alarms compliant with current building standards
//!      (HARDWIRED into electrical system with BATTERY
//!      BACKUP) when adding new alarms; existing
//!      operable alarms need not be replaced
//!      preemptively
//!    - LANDLORD OPERABILITY DUTY: ensure smoke alarms
//!      are operable when a NEW TENANCY is created;
//!      tenants must notify landlord of any inoperable
//!      alarms; landlord must correct reported
//!      deficiencies
//!
//! 2. MASSACHUSETTS — 527 C.M.R. 1.00 Comprehensive Fire
//!    Safety Code + M.G.L. c. 148 § 26F:
//!    - PHOTOELECTRIC TECHNOLOGY MANDATE in bedrooms
//!      and adjacent sleeping areas (per 2017 state
//!      amendments)
//!    - 10-YEAR SEALED-BATTERY requirement for new
//!      installations
//!    - Hardwired with battery backup required in new
//!      construction and substantial renovation
//!
//! 3. DEFAULT — NFPA 72 (National Fire Alarm and
//!    Signaling Code) detector placement standards
//!    referenced by most state and local fire codes;
//!    common-law implied warranty of habitability per
//!    Hilder v. St. Peter, 478 A.2d 202 (Vt. 1984);
//!    Green v. Superior Court, 10 Cal. 3d 616 (1974);
//!    tort negligence + premises liability for fire
//!    injury where smoke-alarm failure causally
//!    contributed.
//!
//! NFPA 72 standard placement requirements:
//! 1. INSIDE EVERY BEDROOM AND SLEEPING AREA
//! 2. OUTSIDE EACH SEPARATE SLEEPING AREA in the
//!    immediate vicinity of bedrooms (hallway)
//! 3. ON EACH ADDITIONAL STORY of the dwelling
//!    including basements (but not crawl spaces or
//!    unfinished attics)
//! 4. INTERCONNECTED so all alarms activate when any
//!    one detects smoke (typical hardwired installation)
//! 5. AT LEAST 4 INCHES from any wall when ceiling-
//!    mounted; AT LEAST 4 INCHES from ceiling when
//!    wall-mounted; NOT WITHIN 36 INCHES of HVAC
//!    supply registers
//!
//! Smoke alarm types modeled:
//! 1. HARDWIRED with BATTERY BACKUP — required in new
//!    construction and substantial renovation in most
//!    jurisdictions
//! 2. 10-YEAR SEALED-BATTERY — required in California
//!    since 2014 for all battery-operated alarms;
//!    eliminates battery-removal vandalism + dead-battery
//!    failures
//! 3. REPLACEABLE-BATTERY — pre-2014 California legacy;
//!    rolling replacement at end of life
//! 4. PHOTOELECTRIC — required in MA bedrooms; better at
//!    detecting slow smoldering fires
//! 5. IONIZATION — better at detecting flaming fires;
//!    legacy technology with high false-alarm rate from
//!    cooking
//! 6. DUAL-SENSOR — combines photoelectric + ionization;
//!    best detection across fire types
//!
//! Universal failure-mode liability framework:
//! 1. ALARM INOPERABLE AT START OF TENANCY → Cal. Health
//!    & Safety Code § 13113.7 violation + habitability
//!    breach (Hilder v. St. Peter)
//! 2. TENANT REPORTED INOPERABLE ALARM NOT CORRECTED →
//!    constructive eviction + retaliation exposure
//!    (landlord_retaliation_damages)
//! 3. NO ALARM IN BEDROOM OR HALLWAY → NFPA 72 placement
//!    violation + fire-injury negligence per se
//! 4. FIRE INJURY OR FATALITY DURING TENANCY → tort
//!    negligence + wrongful death + IIED parallel to
//!    tenant_emotional_distress_damages iter 453;
//!    settlements routinely exceed $1M-$5M with non-
//!    functioning alarm
//! 5. BATTERY-REMOVAL FAILURE (replaceable-battery
//!    alarms where tenant removed battery) → defense
//!    available if landlord had 10-year sealed-battery
//!    or hardwired with backup; weaker defense with
//!    replaceable-battery design
//!
//! Trader-landlord critical because (1) fire-injury
//! settlement values with non-functioning smoke alarm
//! routinely exceed $1M-$5M; (2) California's 2014
//! sealed-battery mandate + 2016 hardwired mandate
//! create rolling-replacement obligation as legacy
//! alarms fail; (3) operability verification at new
//! tenancy is the single most important compliance
//! task; (4) NFPA 72 interconnected alarms are
//! significantly more protective than standalone units
//! and increasingly mandatory; (5) photoelectric vs
//! ionization debate matters — photoelectric detects
//! smoldering fires faster, ionization detects flaming
//! fires faster, dual-sensor is best practice; (6) MA
//! tenant-occupancy bedroom requirement is photoelectric-
//! specific; CA does not mandate specific sensor type.
//!
//! Authority: Cal. Health & Safety Code § 13113.7;
//! Cal. Health & Safety Code § 13113.8; Cal. Civ. Code
//! § 1941.1(a)(7); 24 C.C.R. § 1100.0 et seq. (California
//! Building Code); 527 C.M.R. 1.00 (Massachusetts
//! Comprehensive Fire Safety Code); M.G.L. c. 148 § 26F
//! (Massachusetts); 25 M.R.S. § 2464 (Maine); NFPA 72
//! (National Fire Alarm and Signaling Code); Hilder v.
//! St. Peter, 478 A.2d 202 (Vt. 1984); Green v. Superior
//! Court, 10 Cal. 3d 616 (1974); California State Fire
//! Marshal 2014 regulations on 10-year sealed battery.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    California,
    Massachusetts,
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AlarmTechnology {
    HardwiredWithBatteryBackup,
    TenYearSealedBattery,
    ReplaceableBattery,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SensorType {
    Photoelectric,
    Ionization,
    DualSensor,
    Unknown,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub jurisdiction: Jurisdiction,
    pub alarm_technology: AlarmTechnology,
    pub sensor_type: SensorType,
    pub installed_in_every_bedroom: bool,
    pub installed_in_hallway_outside_sleeping_areas: bool,
    pub installed_on_each_floor_including_basement: bool,
    pub interconnected_alarms: bool,
    pub operable_at_start_of_tenancy: bool,
    pub tenant_reported_inoperable_alarm: bool,
    pub tenant_report_corrected: bool,
    pub fire_injury_or_fatality_during_tenancy: bool,
    pub property_built_or_renovated_after_2016: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NoAlarmInstalled,
    Compliant,
    PlacementViolation,
    OperabilityViolation,
    TechnologyDowngradeRisk,
    FireInjuryEvent,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub severity: Severity,
    pub jurisdiction_specific_actions: Vec<String>,
    pub notes: Vec<String>,
}

pub type RentalHardwiredSmokeAlarmResponsibilityInput = Input;
pub type RentalHardwiredSmokeAlarmResponsibilityResult = Output;

pub fn check(input: &Input) -> Output {
    let notes: Vec<String> = vec![
        "Three-jurisdiction framework: California (most prescriptive — Cal. Health & Safety Code § 13113.7 + § 13113.8 + Cal. Civ. Code § 1941.1(a)(7) implied warranty + 2014 STATE FIRE MARSHAL 10-YEAR SEALED-BATTERY mandate + JANUARY 1, 2016 HARDWIRED-WITH-BATTERY-BACKUP mandate for newly installed alarms + landlord operability duty at new tenancy + tenant notification and landlord correction duty); Massachusetts (527 C.M.R. 1.00 Comprehensive Fire Safety Code + M.G.L. c. 148 § 26F — PHOTOELECTRIC mandate in bedrooms and adjacent sleeping areas + 10-year sealed-battery for new + hardwired-with-backup in new construction); Default (NFPA 72 National Fire Alarm and Signaling Code referenced by most state/local fire codes + common-law implied warranty of habitability per Hilder v. St. Peter, 478 A.2d 202 (Vt. 1984) + tort negligence + premises liability).".to_string(),
        "NFPA 72 placement requirements: (1) INSIDE EVERY BEDROOM and sleeping area; (2) OUTSIDE EACH SEPARATE SLEEPING AREA in immediate vicinity of bedrooms (hallway); (3) ON EACH ADDITIONAL STORY of the dwelling including basements (NOT crawl spaces or unfinished attics); (4) INTERCONNECTED so all alarms activate when any one detects smoke (typical hardwired installation); (5) AT LEAST 4 INCHES from wall when ceiling-mounted + AT LEAST 4 INCHES from ceiling when wall-mounted + NOT WITHIN 36 INCHES of HVAC supply registers.".to_string(),
        "Six smoke alarm types modeled: HARDWIRED with BATTERY BACKUP (required in new construction and substantial renovation in most jurisdictions); 10-YEAR SEALED-BATTERY (California mandate since 2014 — eliminates battery-removal vandalism + dead-battery failures); REPLACEABLE-BATTERY (pre-2014 California legacy; rolling replacement at end of life); PHOTOELECTRIC (MA bedroom mandate; better for slow smoldering fires); IONIZATION (better for flaming fires; high cooking false-alarm rate); DUAL-SENSOR (best detection across fire types).".to_string(),
        "Five universal failure-mode liabilities: (1) alarm inoperable at start of tenancy → Cal. Health & Safety Code § 13113.7 violation + habitability breach; (2) tenant reported inoperable alarm not corrected → constructive eviction + retaliation (landlord_retaliation_damages); (3) no alarm in bedroom or hallway → NFPA 72 placement violation + fire-injury negligence per se; (4) fire injury/fatality during tenancy → tort negligence + wrongful death + IIED parallel to tenant_emotional_distress_damages iter 453 ($1M-$5M settlements routine); (5) battery-removal failure on replaceable-battery design → weaker defense than 10-year sealed-battery or hardwired-with-backup.".to_string(),
        "Companion modules: rental_carbon_monoxide_detector, rental_chimney_fireplace_inspection_disclosure (iter 471), rental_fire_extinguisher_requirement (iter 473), rental_window_blind_cord_safety (iter 469), rental_bedroom_egress_window, tenant_fire_safety_plan_disclosure, landlord_retaliation_damages, tenant_emotional_distress_damages.".to_string(),
    ];
    let mut actions: Vec<String> = Vec::new();

    if input.fire_injury_or_fatality_during_tenancy {
        actions.push("Fire injury or fatality reported: engage emergency services + counsel; preserve evidence including alarm units and battery test logs; tort negligence + wrongful death + IIED exposure parallel to tenant_emotional_distress_damages iter 453; $1M-$5M settlement routine with non-functioning alarm.".to_string());
    }

    if matches!(input.alarm_technology, AlarmTechnology::None) {
        actions.push("NO smoke alarm installed: per-se violation in every modeled jurisdiction; immediate installation required to satisfy Cal. Health & Safety Code § 13113.7 / 527 C.M.R. 1.00 / NFPA 72 + common-law habitability.".to_string());
    }

    let placement_violation = !input.installed_in_every_bedroom
        || !input.installed_in_hallway_outside_sleeping_areas
        || !input.installed_on_each_floor_including_basement;

    if placement_violation && !matches!(input.alarm_technology, AlarmTechnology::None) {
        let mut missing: Vec<&str> = Vec::new();
        if !input.installed_in_every_bedroom {
            missing.push("every bedroom");
        }
        if !input.installed_in_hallway_outside_sleeping_areas {
            missing.push("hallway outside sleeping areas");
        }
        if !input.installed_on_each_floor_including_basement {
            missing.push("each floor including basement");
        }
        actions.push(format!(
            "NFPA 72 placement violation: alarms missing from {} — install additional alarms to satisfy placement standard.",
            missing.join(" + ")
        ));
    }

    if !input.operable_at_start_of_tenancy {
        actions.push("Alarm not operable at start of tenancy: violates Cal. Health & Safety Code § 13113.7 landlord operability duty + Cal. Civ. Code § 1941.1(a)(7) implied warranty of habitability + analogous state statutes; test and replace before tenant occupancy.".to_string());
    }

    if input.tenant_reported_inoperable_alarm && !input.tenant_report_corrected {
        actions.push("Tenant reported inoperable alarm NOT corrected: violates landlord-correction duty under Cal. Health & Safety Code § 13113.7 + habitability breach (Hilder v. St. Peter) + retaliation exposure under landlord_retaliation_damages.".to_string());
    }

    let downgrade_risk = matches!(input.alarm_technology, AlarmTechnology::ReplaceableBattery)
        && matches!(input.jurisdiction, Jurisdiction::California);

    if downgrade_risk {
        actions.push("California legacy replaceable-battery alarm: post-2014 State Fire Marshal mandate requires 10-year sealed-battery for new battery-operated installations; existing operable alarms need not be replaced preemptively but should be upgraded at rolling end-of-life for liability mitigation; consider hardwired-with-backup for new installations after January 1, 2016 building standard.".to_string());
    }

    let post_2016_hardwired_required = matches!(input.jurisdiction, Jurisdiction::California)
        && input.property_built_or_renovated_after_2016
        && !matches!(
            input.alarm_technology,
            AlarmTechnology::HardwiredWithBatteryBackup
        );

    if post_2016_hardwired_required {
        actions.push("California post-January-1-2016 property: building standard requires HARDWIRED-WITH-BATTERY-BACKUP smoke alarms in newly constructed or substantially renovated dwellings; battery-only alarms insufficient.".to_string());
    }

    if matches!(input.jurisdiction, Jurisdiction::Massachusetts)
        && !matches!(
            input.sensor_type,
            SensorType::Photoelectric | SensorType::DualSensor
        )
        && input.installed_in_every_bedroom
    {
        actions.push("Massachusetts bedroom photoelectric mandate: M.G.L. c. 148 § 26F + 527 C.M.R. 1.00 require photoelectric (or dual-sensor) technology in bedrooms and adjacent sleeping areas; ionization-only insufficient.".to_string());
    }

    if !input.interconnected_alarms {
        actions.push("Alarms NOT interconnected: NFPA 72 best-practice interconnection (alarm in one location triggers all others) significantly improves life-safety; required in new hardwired installations; consider upgrade for liability mitigation.".to_string());
    }

    match input.jurisdiction {
        Jurisdiction::California => {
            actions.push("California: Cal. Health & Safety Code § 13113.7 (landlord operability duty + correction duty) + § 13113.8 + Cal. Civ. Code § 1941.1(a)(7) implied warranty; 2014 State Fire Marshal 10-year sealed-battery mandate + January 1, 2016 hardwired-with-battery-backup mandate for new installations.".to_string());
        }
        Jurisdiction::Massachusetts => {
            actions.push("Massachusetts: 527 C.M.R. 1.00 Comprehensive Fire Safety Code + M.G.L. c. 148 § 26F — photoelectric technology mandate in bedrooms + 10-year sealed-battery for new + hardwired-with-backup in new construction.".to_string());
        }
        Jurisdiction::Default => {
            actions.push("Default jurisdiction: NFPA 72 National Fire Alarm and Signaling Code referenced by most state/local fire codes + common-law implied warranty per Hilder v. St. Peter, 478 A.2d 202 (Vt. 1984) + Cal. Civ. Code § 1941.1; tort negligence + premises liability for fire-injury claims.".to_string());
        }
    }

    let severity = if input.fire_injury_or_fatality_during_tenancy {
        Severity::FireInjuryEvent
    } else if matches!(input.alarm_technology, AlarmTechnology::None) {
        Severity::NoAlarmInstalled
    } else if !input.operable_at_start_of_tenancy
        || (input.tenant_reported_inoperable_alarm && !input.tenant_report_corrected)
    {
        Severity::OperabilityViolation
    } else if placement_violation || post_2016_hardwired_required {
        Severity::PlacementViolation
    } else if downgrade_risk {
        Severity::TechnologyDowngradeRisk
    } else {
        Severity::Compliant
    };

    Output {
        severity,
        jurisdiction_specific_actions: actions,
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> Input {
        Input {
            jurisdiction: Jurisdiction::California,
            alarm_technology: AlarmTechnology::HardwiredWithBatteryBackup,
            sensor_type: SensorType::DualSensor,
            installed_in_every_bedroom: true,
            installed_in_hallway_outside_sleeping_areas: true,
            installed_on_each_floor_including_basement: true,
            interconnected_alarms: true,
            operable_at_start_of_tenancy: true,
            tenant_reported_inoperable_alarm: false,
            tenant_report_corrected: false,
            fire_injury_or_fatality_during_tenancy: false,
            property_built_or_renovated_after_2016: true,
        }
    }

    #[test]
    fn ca_compliant_baseline() {
        let i = baseline();
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn no_alarm_installed_top_violation() {
        let mut i = baseline();
        i.alarm_technology = AlarmTechnology::None;
        let out = check(&i);
        assert_eq!(out.severity, Severity::NoAlarmInstalled);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("NO smoke alarm installed"));
    }

    #[test]
    fn fire_injury_event_top_severity() {
        let mut i = baseline();
        i.fire_injury_or_fatality_during_tenancy = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::FireInjuryEvent);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("$1M-$5M settlement"));
        assert!(joined.contains("tenant_emotional_distress_damages"));
    }

    #[test]
    fn inoperable_at_start_of_tenancy_operability_violation() {
        let mut i = baseline();
        i.operable_at_start_of_tenancy = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::OperabilityViolation);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("§ 13113.7"));
        assert!(joined.contains("§ 1941.1(a)(7)"));
    }

    #[test]
    fn tenant_reported_inoperable_not_corrected_violation() {
        let mut i = baseline();
        i.tenant_reported_inoperable_alarm = true;
        i.tenant_report_corrected = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::OperabilityViolation);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("landlord_retaliation_damages"));
    }

    #[test]
    fn tenant_reported_inoperable_corrected_compliant() {
        let mut i = baseline();
        i.tenant_reported_inoperable_alarm = true;
        i.tenant_report_corrected = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn missing_bedroom_alarm_placement_violation() {
        let mut i = baseline();
        i.installed_in_every_bedroom = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::PlacementViolation);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("every bedroom"));
        assert!(joined.contains("NFPA 72"));
    }

    #[test]
    fn missing_hallway_alarm_placement_violation() {
        let mut i = baseline();
        i.installed_in_hallway_outside_sleeping_areas = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::PlacementViolation);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("hallway outside sleeping areas"));
    }

    #[test]
    fn missing_floor_alarm_placement_violation() {
        let mut i = baseline();
        i.installed_on_each_floor_including_basement = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::PlacementViolation);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("each floor including basement"));
    }

    #[test]
    fn ca_replaceable_battery_legacy_downgrade_risk() {
        let mut i = baseline();
        i.alarm_technology = AlarmTechnology::ReplaceableBattery;
        i.property_built_or_renovated_after_2016 = false; // pre-2016 — no hardwired mandate
        let out = check(&i);
        assert_eq!(out.severity, Severity::TechnologyDowngradeRisk);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("post-2014 State Fire Marshal"));
        assert!(joined.contains("10-year sealed-battery"));
    }

    #[test]
    fn ca_post_2016_property_must_be_hardwired() {
        let mut i = baseline();
        i.alarm_technology = AlarmTechnology::TenYearSealedBattery;
        // post-2016 California property
        let out = check(&i);
        assert_eq!(out.severity, Severity::PlacementViolation);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("post-January-1-2016"));
        assert!(joined.contains("HARDWIRED-WITH-BATTERY-BACKUP"));
    }

    #[test]
    fn ca_pre_2016_property_sealed_battery_ok() {
        let mut i = baseline();
        i.alarm_technology = AlarmTechnology::TenYearSealedBattery;
        i.property_built_or_renovated_after_2016 = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn ca_ten_year_sealed_battery_no_downgrade_risk() {
        let mut i = baseline();
        i.alarm_technology = AlarmTechnology::TenYearSealedBattery;
        i.property_built_or_renovated_after_2016 = false;
        let out = check(&i);
        // 10-year sealed battery is current; not legacy replaceable
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn ma_bedroom_photoelectric_mandate_violation() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Massachusetts;
        i.sensor_type = SensorType::Ionization;
        let out = check(&i);
        // Note added but compliant on other fronts
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("Massachusetts bedroom photoelectric mandate"));
        assert!(joined.contains("ionization-only insufficient"));
    }

    #[test]
    fn ma_bedroom_dual_sensor_no_photoelectric_warning() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Massachusetts;
        i.sensor_type = SensorType::DualSensor;
        let out = check(&i);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(!joined.contains("Massachusetts bedroom photoelectric mandate"));
    }

    #[test]
    fn non_interconnected_alarms_warning() {
        let mut i = baseline();
        i.interconnected_alarms = false;
        let out = check(&i);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("NOT interconnected"));
        assert!(joined.contains("NFPA 72 best-practice"));
    }

    #[test]
    fn ca_jurisdiction_cites_13113_7() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::California;
        let out = check(&i);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("§ 13113.7"));
        assert!(joined.contains("§ 13113.8"));
        assert!(joined.contains("§ 1941.1(a)(7)"));
        assert!(joined.contains("2014 State Fire Marshal"));
        assert!(joined.contains("January 1, 2016"));
    }

    #[test]
    fn ma_jurisdiction_cites_527_cmr_and_26f() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Massachusetts;
        let out = check(&i);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("527 C.M.R. 1.00"));
        assert!(joined.contains("§ 26F"));
    }

    #[test]
    fn default_jurisdiction_common_law_habitability() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Default;
        let out = check(&i);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("NFPA 72 National Fire Alarm"));
        assert!(joined.contains("Hilder v. St. Peter"));
        assert!(joined.contains("§ 1941.1"));
    }

    #[test]
    fn severity_priority_fire_injury_above_no_alarm_above_operability() {
        let mut i = baseline();
        i.fire_injury_or_fatality_during_tenancy = true;
        i.alarm_technology = AlarmTechnology::None;
        i.operable_at_start_of_tenancy = false;
        let out = check(&i);
        // Fire injury wins
        assert_eq!(out.severity, Severity::FireInjuryEvent);
    }

    #[test]
    fn severity_no_alarm_above_operability_above_placement() {
        let mut i = baseline();
        i.alarm_technology = AlarmTechnology::None;
        i.operable_at_start_of_tenancy = false;
        i.installed_in_every_bedroom = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::NoAlarmInstalled);
    }

    #[test]
    fn severity_operability_above_placement() {
        let mut i = baseline();
        i.operable_at_start_of_tenancy = false;
        i.installed_in_every_bedroom = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::OperabilityViolation);
    }

    #[test]
    fn severity_placement_above_downgrade_risk() {
        let mut i = baseline();
        i.installed_in_every_bedroom = false;
        i.alarm_technology = AlarmTechnology::ReplaceableBattery;
        i.property_built_or_renovated_after_2016 = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::PlacementViolation);
    }

    #[test]
    fn citation_pins_all_authorities() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("§ 13113.7"));
        assert!(joined.contains("§ 13113.8"));
        assert!(joined.contains("§ 1941.1(a)(7)"));
        assert!(joined.contains("527 C.M.R. 1.00"));
        assert!(joined.contains("§ 26F"));
        assert!(joined.contains("NFPA 72"));
        assert!(joined.contains("Hilder v. St. Peter"));
        assert!(joined.contains("478 A.2d 202"));
        assert!(joined.contains("2014"));
        assert!(joined.contains("JANUARY 1, 2016"));
    }

    #[test]
    fn note_pins_three_jurisdiction_framework() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("California (most prescriptive"));
        assert!(joined.contains("Massachusetts"));
        assert!(joined.contains("Default"));
    }

    #[test]
    fn note_pins_nfpa_72_five_placement_requirements() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("INSIDE EVERY BEDROOM"));
        assert!(joined.contains("OUTSIDE EACH SEPARATE SLEEPING AREA"));
        assert!(joined.contains("ON EACH ADDITIONAL STORY"));
        assert!(joined.contains("INTERCONNECTED"));
        assert!(joined.contains("4 INCHES"));
        assert!(joined.contains("36 INCHES"));
    }

    #[test]
    fn note_pins_six_alarm_types() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("HARDWIRED with BATTERY BACKUP"));
        assert!(joined.contains("10-YEAR SEALED-BATTERY"));
        assert!(joined.contains("REPLACEABLE-BATTERY"));
        assert!(joined.contains("PHOTOELECTRIC"));
        assert!(joined.contains("IONIZATION"));
        assert!(joined.contains("DUAL-SENSOR"));
    }

    #[test]
    fn note_pins_five_failure_modes() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("inoperable at start of tenancy"));
        assert!(joined.contains("tenant reported inoperable"));
        assert!(joined.contains("no alarm in bedroom"));
        assert!(joined.contains("fire injury/fatality"));
        assert!(joined.contains("battery-removal failure"));
    }

    #[test]
    fn note_pins_companion_modules() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("rental_carbon_monoxide_detector"));
        assert!(joined.contains("rental_chimney_fireplace_inspection_disclosure"));
        assert!(joined.contains("rental_fire_extinguisher_requirement"));
        assert!(joined.contains("tenant_emotional_distress_damages"));
    }

    #[test]
    fn jurisdiction_truth_table_three_cells() {
        let ca = check(&Input {
            jurisdiction: Jurisdiction::California,
            ..baseline()
        });
        let ma = check(&Input {
            jurisdiction: Jurisdiction::Massachusetts,
            ..baseline()
        });
        let de = check(&Input {
            jurisdiction: Jurisdiction::Default,
            ..baseline()
        });
        assert_eq!(ca.severity, Severity::Compliant);
        assert_eq!(ma.severity, Severity::Compliant);
        assert_eq!(de.severity, Severity::Compliant);
    }

    #[test]
    fn ma_uniquely_photoelectric_invariant() {
        // Same fact: Ionization sensor in bedrooms
        let ma = check(&Input {
            jurisdiction: Jurisdiction::Massachusetts,
            sensor_type: SensorType::Ionization,
            ..baseline()
        });
        let ca = check(&Input {
            jurisdiction: Jurisdiction::California,
            sensor_type: SensorType::Ionization,
            ..baseline()
        });
        let joined_ma = ma.jurisdiction_specific_actions.join(" ");
        let joined_ca = ca.jurisdiction_specific_actions.join(" ");
        assert!(joined_ma.contains("Massachusetts bedroom photoelectric mandate"));
        assert!(!joined_ca.contains("Massachusetts bedroom photoelectric mandate"));
    }

    #[test]
    fn ca_uniquely_2016_hardwired_invariant() {
        // Same fact: TenYearSealedBattery in post-2016 property
        let ca = check(&Input {
            jurisdiction: Jurisdiction::California,
            alarm_technology: AlarmTechnology::TenYearSealedBattery,
            property_built_or_renovated_after_2016: true,
            ..baseline()
        });
        let ma = check(&Input {
            jurisdiction: Jurisdiction::Massachusetts,
            alarm_technology: AlarmTechnology::TenYearSealedBattery,
            property_built_or_renovated_after_2016: true,
            ..baseline()
        });
        // CA triggers placement violation; MA does not have post-2016 hardwired rule
        assert_eq!(ca.severity, Severity::PlacementViolation);
        assert_eq!(ma.severity, Severity::Compliant);
    }

    #[test]
    fn multiple_violations_stack_in_actions() {
        let mut i = baseline();
        i.installed_in_every_bedroom = false;
        i.installed_in_hallway_outside_sleeping_areas = false;
        i.installed_on_each_floor_including_basement = false;
        i.interconnected_alarms = false;
        let out = check(&i);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("every bedroom"));
        assert!(joined.contains("hallway outside sleeping areas"));
        assert!(joined.contains("each floor including basement"));
        assert!(joined.contains("NOT interconnected"));
    }
}
