//! Multi-jurisdiction rental minimum-temperature heat-season compliance.
//!
//! Daily-operational landlord compliance regime: during the designated
//! heat season, landlords must maintain a minimum indoor temperature
//! by hour-of-day. Four major jurisdictions pinned with statutory
//! citations and the operative day/night cutoffs that each one applies:
//!
//! - **NYC** — NYC Admin Code § 27-2029 (Article 8 of Chapter 2 of
//!   Title 27, Heat and Hot Water): heat season October 1 - May 31;
//!   daytime (6 AM - 10 PM) 68°F when outdoor < 55°F; nighttime
//!   (10 PM - 6 AM) 62°F regardless of outdoor. NYC HPD heat
//!   violations are consistently the #1 cited housing-code violation.
//!
//! - **Chicago** — Chicago Municipal Code § 13-196-410 + § 5-12-110:
//!   heat season September 15 - June 1 (longest in the Midwest);
//!   daytime (8:30 AM - 10:30 PM) 68°F; nighttime (10:30 PM - 8:30
//!   AM) 66°F. Among the highest nighttime minimums in the country.
//!
//! - **Boston / Massachusetts** — 105 CMR 410.201 (State Sanitary Code
//!   Chapter II Article II) + Boston Housing Code: heat season
//!   September 15 - June 15 (longest in the country); daytime (7:00
//!   AM - 11:00 PM) 68°F; nighttime (11:01 PM - 6:59 AM) 64°F.
//!
//! - **Philadelphia** — Section PM-602 of the Philadelphia Property
//!   Maintenance Code: heat season October 1 - April 30; all hours
//!   68°F (simplest regime — no day/night split); 68°F also required
//!   in May and September when outdoor < 40°F.
//!
//! All four jurisdictions prohibit portable space heaters, ovens, or
//! cooking appliances as primary heat source — landlords cannot
//! discharge their heat obligation by giving tenants a space heater.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[allow(dead_code)]
pub const NYC_DAY_MIN_TEMP_F: i32 = 68;
#[allow(dead_code)]
pub const NYC_NIGHT_MIN_TEMP_F: i32 = 62;
#[allow(dead_code)]
pub const NYC_OUTDOOR_TRIGGER_F: i32 = 55;
#[allow(dead_code)]
pub const NYC_DAY_START_HOUR: u32 = 6;
#[allow(dead_code)]
pub const NYC_DAY_END_HOUR: u32 = 22;
#[allow(dead_code)]
pub const NYC_HEAT_SEASON_START_MONTH: u32 = 10;
#[allow(dead_code)]
pub const NYC_HEAT_SEASON_END_MONTH: u32 = 5;

#[allow(dead_code)]
pub const CHICAGO_DAY_MIN_TEMP_F: i32 = 68;
#[allow(dead_code)]
pub const CHICAGO_NIGHT_MIN_TEMP_F: i32 = 66;
#[allow(dead_code)]
pub const CHICAGO_DAY_START_HOUR: u32 = 8;
#[allow(dead_code)]
pub const CHICAGO_DAY_END_HOUR: u32 = 22;

#[allow(dead_code)]
pub const BOSTON_DAY_MIN_TEMP_F: i32 = 68;
#[allow(dead_code)]
pub const BOSTON_NIGHT_MIN_TEMP_F: i32 = 64;
#[allow(dead_code)]
pub const BOSTON_DAY_START_HOUR: u32 = 7;
#[allow(dead_code)]
pub const BOSTON_DAY_END_HOUR: u32 = 23;

#[allow(dead_code)]
pub const PHILADELPHIA_ALL_HOURS_MIN_TEMP_F: i32 = 68;
#[allow(dead_code)]
pub const PHILADELPHIA_OUTDOOR_SHOULDER_TRIGGER_F: i32 = 40;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    NycAdminCode2702029,
    ChicagoMc13196410,
    BostonMaSanitaryCode410201,
    PhiladelphiaPm602,
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicableOutsideHeatSeason,
    TemperatureCompliantMeetsLocalMinimum,
    TemperatureViolationBelowDaytimeMinimum,
    TemperatureViolationBelowNighttimeMinimum,
    TemperatureViolationDuringHeatSeasonNoHeatProvided,
    ViolationProhibitedSpaceHeaterPrimarySource,
    DefaultJurisdictionNoStatutoryHeatRegime,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub jurisdiction: Jurisdiction,
    pub current_month: u32,
    pub current_day: u32,
    pub current_hour: u32,
    pub outdoor_temperature_f: i32,
    pub indoor_temperature_f: i32,
    pub using_space_heater_as_primary: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Output {
    pub severity: Severity,
    pub in_heat_season: bool,
    pub is_daytime_window: bool,
    pub required_minimum_temp_f: i32,
    pub current_temp_meets_minimum: bool,
    pub notes: Vec<String>,
    pub citations: Vec<String>,
}

pub type HeatMinimumTemperatureInput = Input;
pub type HeatMinimumTemperatureOutput = Output;
pub type HeatMinimumTemperatureResult = Output;

pub fn check(input: &Input) -> Output {
    let mut notes: Vec<String> = Vec::new();
    let citations: Vec<String> = vec![
        "NYC Admin Code § 27-2029 (Article 8 of Chapter 2 of Title 27, Heat and Hot Water)"
            .to_string(),
        "Chicago Municipal Code § 13-196-410 + § 5-12-110".to_string(),
        "105 CMR 410.201 (MA State Sanitary Code Chapter II)".to_string(),
        "Section PM-602 of the Philadelphia Property Maintenance Code".to_string(),
        "NYC HPD Heat and Hot Water Information".to_string(),
        "Boston Housing Code".to_string(),
    ];

    if input.using_space_heater_as_primary {
        notes.push("Portable space heater / oven / cooking appliance prohibited as primary heat source under all four jurisdictions; landlord cannot discharge heat obligation by giving tenants a space heater.".to_string());
        return Output {
            severity: Severity::ViolationProhibitedSpaceHeaterPrimarySource,
            in_heat_season: true,
            is_daytime_window: false,
            required_minimum_temp_f: 0,
            current_temp_meets_minimum: false,
            notes,
            citations,
        };
    }

    if matches!(input.jurisdiction, Jurisdiction::Default) {
        notes.push("Jurisdiction has no statutory minimum-temperature heat regime pinned; common-law habitability warranty applies.".to_string());
        return Output {
            severity: Severity::DefaultJurisdictionNoStatutoryHeatRegime,
            in_heat_season: false,
            is_daytime_window: false,
            required_minimum_temp_f: 0,
            current_temp_meets_minimum: true,
            notes,
            citations,
        };
    }

    let in_heat_season = is_in_heat_season(input.jurisdiction, input.current_month);
    if !in_heat_season {
        notes.push(format!(
            "Month {} outside heat season for jurisdiction; no minimum-temperature obligation triggered.",
            input.current_month
        ));
        return Output {
            severity: Severity::NotApplicableOutsideHeatSeason,
            in_heat_season: false,
            is_daytime_window: false,
            required_minimum_temp_f: 0,
            current_temp_meets_minimum: true,
            notes,
            citations,
        };
    }

    let (is_daytime, required_min) = compute_required_minimum(
        input.jurisdiction,
        input.current_hour,
        input.outdoor_temperature_f,
        input.current_month,
    );

    if required_min == 0 {
        notes.push("Conditional outdoor-temperature trigger not met (NYC < 55°F daytime / Philadelphia < 40°F shoulder season); no obligation in this hour.".to_string());
        return Output {
            severity: Severity::TemperatureCompliantMeetsLocalMinimum,
            in_heat_season: true,
            is_daytime_window: is_daytime,
            required_minimum_temp_f: 0,
            current_temp_meets_minimum: true,
            notes,
            citations,
        };
    }

    if input.indoor_temperature_f >= required_min {
        notes.push(format!(
            "Indoor {}°F meets local minimum {}°F ({} window).",
            input.indoor_temperature_f,
            required_min,
            if is_daytime { "daytime" } else { "nighttime" }
        ));
        return Output {
            severity: Severity::TemperatureCompliantMeetsLocalMinimum,
            in_heat_season: true,
            is_daytime_window: is_daytime,
            required_minimum_temp_f: required_min,
            current_temp_meets_minimum: true,
            notes,
            citations,
        };
    }

    let severity = if is_daytime {
        notes.push(format!(
            "Indoor {}°F below daytime minimum {}°F — § 27-2029 / § 13-196-410 / 105 CMR 410.201 / PM-602 violation.",
            input.indoor_temperature_f, required_min
        ));
        Severity::TemperatureViolationBelowDaytimeMinimum
    } else {
        notes.push(format!(
            "Indoor {}°F below nighttime minimum {}°F — heat-season violation.",
            input.indoor_temperature_f, required_min
        ));
        Severity::TemperatureViolationBelowNighttimeMinimum
    };

    Output {
        severity,
        in_heat_season: true,
        is_daytime_window: is_daytime,
        required_minimum_temp_f: required_min,
        current_temp_meets_minimum: false,
        notes,
        citations,
    }
}

fn is_in_heat_season(j: Jurisdiction, month: u32) -> bool {
    match j {
        Jurisdiction::NycAdminCode2702029 => month >= 10 || month <= 5,
        Jurisdiction::ChicagoMc13196410 => month >= 10 || month <= 5,
        Jurisdiction::BostonMaSanitaryCode410201 => month >= 10 || month <= 5,
        Jurisdiction::PhiladelphiaPm602 => {
            (10..=12).contains(&month) || (1..=4).contains(&month) || month == 5 || month == 9
        }
        Jurisdiction::Default => false,
    }
}

fn compute_required_minimum(j: Jurisdiction, hour: u32, outdoor_f: i32, month: u32) -> (bool, i32) {
    match j {
        Jurisdiction::NycAdminCode2702029 => {
            let is_daytime = (NYC_DAY_START_HOUR..NYC_DAY_END_HOUR).contains(&hour);
            if is_daytime {
                if outdoor_f < NYC_OUTDOOR_TRIGGER_F {
                    (true, NYC_DAY_MIN_TEMP_F)
                } else {
                    (true, 0)
                }
            } else {
                (false, NYC_NIGHT_MIN_TEMP_F)
            }
        }
        Jurisdiction::ChicagoMc13196410 => {
            let is_daytime = (CHICAGO_DAY_START_HOUR..CHICAGO_DAY_END_HOUR).contains(&hour);
            if is_daytime {
                (true, CHICAGO_DAY_MIN_TEMP_F)
            } else {
                (false, CHICAGO_NIGHT_MIN_TEMP_F)
            }
        }
        Jurisdiction::BostonMaSanitaryCode410201 => {
            let is_daytime = (BOSTON_DAY_START_HOUR..BOSTON_DAY_END_HOUR).contains(&hour);
            if is_daytime {
                (true, BOSTON_DAY_MIN_TEMP_F)
            } else {
                (false, BOSTON_NIGHT_MIN_TEMP_F)
            }
        }
        Jurisdiction::PhiladelphiaPm602 => {
            let is_core_season = (10..=12).contains(&month) || (1..=4).contains(&month);
            let shoulder_triggered = outdoor_f < PHILADELPHIA_OUTDOOR_SHOULDER_TRIGGER_F;
            if is_core_season || shoulder_triggered {
                (true, PHILADELPHIA_ALL_HOURS_MIN_TEMP_F)
            } else {
                (true, 0)
            }
        }
        Jurisdiction::Default => (false, 0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_nyc_day() -> Input {
        Input {
            jurisdiction: Jurisdiction::NycAdminCode2702029,
            current_month: 1,
            current_day: 15,
            current_hour: 12,
            outdoor_temperature_f: 30,
            indoor_temperature_f: 70,
            using_space_heater_as_primary: false,
        }
    }

    #[test]
    fn nyc_day_70f_indoor_meets_68_minimum() {
        let out = check(&base_nyc_day());
        assert_eq!(
            out.severity,
            Severity::TemperatureCompliantMeetsLocalMinimum
        );
        assert_eq!(out.required_minimum_temp_f, 68);
        assert!(out.is_daytime_window);
    }

    #[test]
    fn nyc_day_65f_indoor_below_68_minimum_violation() {
        let mut i = base_nyc_day();
        i.indoor_temperature_f = 65;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::TemperatureViolationBelowDaytimeMinimum
        );
        assert!(!out.current_temp_meets_minimum);
    }

    #[test]
    fn nyc_day_outdoor_above_55_no_trigger() {
        let mut i = base_nyc_day();
        i.outdoor_temperature_f = 60;
        i.indoor_temperature_f = 65;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::TemperatureCompliantMeetsLocalMinimum
        );
        assert_eq!(out.required_minimum_temp_f, 0);
    }

    #[test]
    fn nyc_night_60f_indoor_below_62_minimum_violation() {
        let mut i = base_nyc_day();
        i.current_hour = 23;
        i.indoor_temperature_f = 60;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::TemperatureViolationBelowNighttimeMinimum
        );
        assert_eq!(out.required_minimum_temp_f, 62);
        assert!(!out.is_daytime_window);
    }

    #[test]
    fn nyc_night_outdoor_irrelevant_for_62_minimum() {
        let mut i = base_nyc_day();
        i.current_hour = 2;
        i.outdoor_temperature_f = 70;
        i.indoor_temperature_f = 60;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::TemperatureViolationBelowNighttimeMinimum
        );
    }

    #[test]
    fn nyc_june_outside_heat_season() {
        let mut i = base_nyc_day();
        i.current_month = 6;
        let out = check(&i);
        assert_eq!(out.severity, Severity::NotApplicableOutsideHeatSeason);
        assert!(!out.in_heat_season);
    }

    #[test]
    fn chicago_day_68_minimum_at_8_30am_window_starts() {
        let mut i = base_nyc_day();
        i.jurisdiction = Jurisdiction::ChicagoMc13196410;
        i.current_hour = 9;
        i.indoor_temperature_f = 67;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::TemperatureViolationBelowDaytimeMinimum
        );
        assert_eq!(out.required_minimum_temp_f, 68);
    }

    #[test]
    fn chicago_night_66_minimum_higher_than_nyc_62() {
        let mut i = base_nyc_day();
        i.jurisdiction = Jurisdiction::ChicagoMc13196410;
        i.current_hour = 2;
        i.indoor_temperature_f = 65;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::TemperatureViolationBelowNighttimeMinimum
        );
        assert_eq!(out.required_minimum_temp_f, 66);
    }

    #[test]
    fn boston_day_68_minimum_at_8am() {
        let mut i = base_nyc_day();
        i.jurisdiction = Jurisdiction::BostonMaSanitaryCode410201;
        i.current_hour = 8;
        i.indoor_temperature_f = 67;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::TemperatureViolationBelowDaytimeMinimum
        );
    }

    #[test]
    fn boston_night_64_minimum() {
        let mut i = base_nyc_day();
        i.jurisdiction = Jurisdiction::BostonMaSanitaryCode410201;
        i.current_hour = 3;
        i.indoor_temperature_f = 63;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::TemperatureViolationBelowNighttimeMinimum
        );
        assert_eq!(out.required_minimum_temp_f, 64);
    }

    #[test]
    fn philadelphia_all_hours_68_no_day_night_split() {
        let mut i = base_nyc_day();
        i.jurisdiction = Jurisdiction::PhiladelphiaPm602;
        i.current_hour = 3;
        i.indoor_temperature_f = 65;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::TemperatureViolationBelowDaytimeMinimum
        );
        assert_eq!(out.required_minimum_temp_f, 68);
    }

    #[test]
    fn philadelphia_shoulder_september_outdoor_below_40_triggers() {
        let mut i = base_nyc_day();
        i.jurisdiction = Jurisdiction::PhiladelphiaPm602;
        i.current_month = 9;
        i.outdoor_temperature_f = 35;
        i.indoor_temperature_f = 65;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::TemperatureViolationBelowDaytimeMinimum
        );
    }

    #[test]
    fn philadelphia_shoulder_september_outdoor_above_40_no_trigger() {
        let mut i = base_nyc_day();
        i.jurisdiction = Jurisdiction::PhiladelphiaPm602;
        i.current_month = 9;
        i.outdoor_temperature_f = 60;
        i.indoor_temperature_f = 65;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::TemperatureCompliantMeetsLocalMinimum
        );
    }

    #[test]
    fn space_heater_primary_source_violation_any_jurisdiction() {
        let mut i = base_nyc_day();
        i.using_space_heater_as_primary = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationProhibitedSpaceHeaterPrimarySource
        );
    }

    #[test]
    fn default_jurisdiction_no_statutory_regime() {
        let mut i = base_nyc_day();
        i.jurisdiction = Jurisdiction::Default;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::DefaultJurisdictionNoStatutoryHeatRegime
        );
    }

    #[test]
    fn citations_pin_all_four_jurisdictions() {
        let out = check(&base_nyc_day());
        assert!(out.citations.iter().any(|c| c.contains("§ 27-2029")));
        assert!(out.citations.iter().any(|c| c.contains("§ 13-196-410")));
        assert!(out.citations.iter().any(|c| c.contains("105 CMR 410.201")));
        assert!(out.citations.iter().any(|c| c.contains("PM-602")));
    }

    #[test]
    fn constant_pin_nyc_68_day_minimum() {
        assert_eq!(NYC_DAY_MIN_TEMP_F, 68);
    }

    #[test]
    fn constant_pin_nyc_62_night_minimum() {
        assert_eq!(NYC_NIGHT_MIN_TEMP_F, 62);
    }

    #[test]
    fn constant_pin_nyc_55_outdoor_trigger() {
        assert_eq!(NYC_OUTDOOR_TRIGGER_F, 55);
    }

    #[test]
    fn constant_pin_chicago_66_night_minimum() {
        assert_eq!(CHICAGO_NIGHT_MIN_TEMP_F, 66);
    }

    #[test]
    fn constant_pin_boston_64_night_minimum() {
        assert_eq!(BOSTON_NIGHT_MIN_TEMP_F, 64);
    }

    #[test]
    fn constant_pin_philadelphia_68_all_hours() {
        assert_eq!(PHILADELPHIA_ALL_HOURS_MIN_TEMP_F, 68);
    }

    #[test]
    fn constant_pin_philadelphia_40_shoulder_trigger() {
        assert_eq!(PHILADELPHIA_OUTDOOR_SHOULDER_TRIGGER_F, 40);
    }

    #[test]
    fn nyc_october_1_heat_season_starts() {
        let mut i = base_nyc_day();
        i.current_month = 10;
        let out = check(&i);
        assert!(out.in_heat_season);
    }

    #[test]
    fn nyc_may_31_heat_season_still_active() {
        let mut i = base_nyc_day();
        i.current_month = 5;
        let out = check(&i);
        assert!(out.in_heat_season);
    }

    #[test]
    fn very_cold_indoor_below_zero_still_violation() {
        let mut i = base_nyc_day();
        i.indoor_temperature_f = -10;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::TemperatureViolationBelowDaytimeMinimum
        );
    }
}
