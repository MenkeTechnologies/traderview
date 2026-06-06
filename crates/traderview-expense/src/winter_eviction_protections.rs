//! State / municipal winter and weather-based eviction protection
//! compliance check. Addresses statutory restrictions on landlord ability
//! to evict during hazardous weather conditions or during specified
//! winter periods. Distinct from `snow_removal_responsibility` (landlord
//! duty to clear snow/ice) and `heat_requirements` (habitability heat
//! minimums).
//!
//! Three regimes:
//!
//! District of Columbia (DC Code § 42-3505.01(k)) — strongest statutory
//! weather-based restriction. No eviction may occur on any day when:
//! (a) NWS predicts at 8:00 AM that temperature at Reagan National
//! Airport will fall BELOW 32°F (0°C); (b) NWS predicts at 8:00 AM that
//! temperature will rise ABOVE 95°F (35°C); or (c) precipitation is
//! falling at the location of the rental unit. Applies to all housing
//! providers subject to DC's Rental Housing Act of 1985.
//!
//! Cook County, Illinois (Chicago + suburbs) — sheriff-discretionary
//! regime via Cook County Sheriff's policy. Sheriff will NOT execute
//! eviction orders when (a) temperature is 15°F or COLDER, OR (b)
//! extreme weather conditions (blizzard, storm, high winds) endanger
//! sheriff or tenant safety. Additionally, Cook County Sheriff
//! observes a holiday moratorium from December 19 through January 5
//! annually.
//!
//! Default — no statutory winter / weather-based eviction restriction.
//! Court may exercise equitable discretion to stay execution on an
//! individual case basis. Tenant may invoke common-law habitability if
//! no-heat conditions exist (covered by `heat_requirements`).
//!
//! Citations: DC Code § 42-3505.01(k)(1) (sub-freezing); § 42-3505.01(k)(2)
//! (precipitation); § 42-3505.01(k)(3) (extreme heat); Cook County
//! Sheriff Order on Eviction Execution (annual holiday moratorium +
//! 15°F cold-weather rule + extreme-weather discretion); state common-
//! law equitable stay doctrine.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    DistrictOfColumbia,
    CookCountyIllinois,
    Default,
}

impl Regime {
    pub fn for_jurisdiction(state: &str, county: &str) -> Self {
        let st = state.trim().to_ascii_uppercase();
        let co = county.trim().to_ascii_lowercase();
        match (st.as_str(), co.as_str()) {
            ("DC", _) => Self::DistrictOfColumbia,
            ("IL", "cook") | ("IL", "cook county") => Self::CookCountyIllinois,
            _ => Self::Default,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct WinterEvictionInput {
    pub regime: Regime,
    /// NWS-predicted temperature at 8:00 AM at the relevant weather
    /// station (DC: Reagan National; Cook County: Chicago/O'Hare or
    /// Midway). Fahrenheit; can be negative.
    pub nws_predicted_temp_at_8am_f: i32,
    /// Whether precipitation is actively falling at the rental unit
    /// location at the time of attempted eviction (DC § 42-3505.01(k)(2)).
    pub precipitation_falling_at_unit: bool,
    /// Whether the eviction date falls within Cook County's annual
    /// holiday moratorium window (Dec 19 through Jan 5).
    pub within_cook_county_holiday_moratorium_window: bool,
    /// Whether extreme weather conditions (blizzard / storm / high
    /// winds) endanger sheriff or tenant safety. Drives Cook County
    /// extreme-weather discretion.
    pub extreme_weather_safety_threat: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RestrictionType {
    None,
    SubFreezing,
    ExtremeHeat,
    PrecipitationAtUnit,
    HolidayMoratorium,
    ExtremeWeatherSafetyThreat,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct WinterEvictionResult {
    pub regime: Regime,
    pub eviction_permitted: bool,
    pub restriction_type: RestrictionType,
    pub citation: &'static str,
    pub note: String,
}

pub fn check(input: &WinterEvictionInput) -> WinterEvictionResult {
    match input.regime {
        Regime::DistrictOfColumbia => dc_check(input),
        Regime::CookCountyIllinois => cook_county_check(input),
        Regime::Default => default_check(input),
    }
}

fn dc_check(input: &WinterEvictionInput) -> WinterEvictionResult {
    if input.nws_predicted_temp_at_8am_f < 32 {
        return WinterEvictionResult {
            regime: Regime::DistrictOfColumbia,
            eviction_permitted: false,
            restriction_type: RestrictionType::SubFreezing,
            citation: "DC Code § 42-3505.01(k)(1) — no eviction when NWS predicts at 8 AM that Reagan National Airport temperature will fall BELOW 32°F (0°C)",
            note: format!(
                "NWS-predicted temperature at 8 AM is {}°F (< 32°F). DC § 42-3505.01(k)(1) bars eviction today.",
                input.nws_predicted_temp_at_8am_f
            ),
        };
    }
    if input.nws_predicted_temp_at_8am_f > 95 {
        return WinterEvictionResult {
            regime: Regime::DistrictOfColumbia,
            eviction_permitted: false,
            restriction_type: RestrictionType::ExtremeHeat,
            citation: "DC Code § 42-3505.01(k)(3) — no eviction when NWS predicts at 8 AM that Reagan National Airport temperature will rise ABOVE 95°F (35°C)",
            note: format!(
                "NWS-predicted temperature at 8 AM is {}°F (> 95°F). DC § 42-3505.01(k)(3) bars eviction today.",
                input.nws_predicted_temp_at_8am_f
            ),
        };
    }
    if input.precipitation_falling_at_unit {
        return WinterEvictionResult {
            regime: Regime::DistrictOfColumbia,
            eviction_permitted: false,
            restriction_type: RestrictionType::PrecipitationAtUnit,
            citation: "DC Code § 42-3505.01(k)(2) — no eviction when precipitation is falling at the location of the rental unit",
            note: "Precipitation actively falling at rental unit; DC § 42-3505.01(k)(2) bars eviction today.".to_string(),
        };
    }
    WinterEvictionResult {
        regime: Regime::DistrictOfColumbia,
        eviction_permitted: true,
        restriction_type: RestrictionType::None,
        citation: "DC Code § 42-3505.01(k) — eviction permitted; weather conditions outside statutory restriction windows",
        note: format!(
            "Eviction permitted. Temperature {}°F is within 32-95°F window and no precipitation at unit.",
            input.nws_predicted_temp_at_8am_f
        ),
    }
}

fn cook_county_check(input: &WinterEvictionInput) -> WinterEvictionResult {
    if input.within_cook_county_holiday_moratorium_window {
        return WinterEvictionResult {
            regime: Regime::CookCountyIllinois,
            eviction_permitted: false,
            restriction_type: RestrictionType::HolidayMoratorium,
            citation: "Cook County Sheriff Order on Eviction Execution — annual holiday moratorium from December 19 through January 5",
            note: "Within Cook County Sheriff's annual holiday moratorium window (Dec 19 - Jan 5). Sheriff will not execute eviction orders.".to_string(),
        };
    }
    if input.nws_predicted_temp_at_8am_f <= 15 {
        return WinterEvictionResult {
            regime: Regime::CookCountyIllinois,
            eviction_permitted: false,
            restriction_type: RestrictionType::SubFreezing,
            citation: "Cook County Sheriff Order — sheriff will not execute eviction orders when temperature is 15°F or COLDER",
            note: format!(
                "Temperature {}°F is at or below the 15°F Cook County Sheriff threshold. Sheriff will not execute eviction.",
                input.nws_predicted_temp_at_8am_f
            ),
        };
    }
    if input.extreme_weather_safety_threat {
        return WinterEvictionResult {
            regime: Regime::CookCountyIllinois,
            eviction_permitted: false,
            restriction_type: RestrictionType::ExtremeWeatherSafetyThreat,
            citation: "Cook County Sheriff Order — sheriff will not execute eviction during extreme weather threatening sheriff or tenant safety (blizzard, storm, high winds)",
            note: "Extreme weather conditions endanger sheriff or tenant safety. Cook County Sheriff will not execute eviction today.".to_string(),
        };
    }
    WinterEvictionResult {
        regime: Regime::CookCountyIllinois,
        eviction_permitted: true,
        restriction_type: RestrictionType::None,
        citation: "Cook County Sheriff Order — eviction execution permitted; conditions outside restriction triggers",
        note: format!(
            "Cook County eviction permitted. Temperature {}°F > 15°F, no holiday moratorium, no extreme-weather threat.",
            input.nws_predicted_temp_at_8am_f
        ),
    }
}

fn default_check(_input: &WinterEvictionInput) -> WinterEvictionResult {
    WinterEvictionResult {
        regime: Regime::Default,
        eviction_permitted: true,
        restriction_type: RestrictionType::None,
        citation:
            "No statutory winter / weather-based eviction restriction identified; court may exercise equitable discretion on individual cases",
        note: "Default regime: no statutory winter/weather eviction protection. Court may stay execution on equitable basis. Tenant remedies for habitability (no-heat) covered separately under common law.".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        regime: Regime,
        temp: i32,
        precipitation: bool,
        holiday: bool,
        extreme_weather: bool,
    ) -> WinterEvictionInput {
        WinterEvictionInput {
            regime,
            nws_predicted_temp_at_8am_f: temp,
            precipitation_falling_at_unit: precipitation,
            within_cook_county_holiday_moratorium_window: holiday,
            extreme_weather_safety_threat: extreme_weather,
        }
    }

    #[test]
    fn dc_sub_freezing_31f_blocks_eviction() {
        let r = check(&input(Regime::DistrictOfColumbia, 31, false, false, false));
        assert!(!r.eviction_permitted);
        assert_eq!(r.restriction_type, RestrictionType::SubFreezing);
        assert!(r.citation.contains("§ 42-3505.01(k)(1)"));
    }

    #[test]
    fn dc_at_32f_boundary_eviction_permitted() {
        // Strictly BELOW 32°F triggers restriction — at exactly 32°F OK.
        let r = check(&input(Regime::DistrictOfColumbia, 32, false, false, false));
        assert!(r.eviction_permitted);
        assert_eq!(r.restriction_type, RestrictionType::None);
    }

    #[test]
    fn dc_extreme_heat_96f_blocks_eviction() {
        let r = check(&input(Regime::DistrictOfColumbia, 96, false, false, false));
        assert!(!r.eviction_permitted);
        assert_eq!(r.restriction_type, RestrictionType::ExtremeHeat);
        assert!(r.citation.contains("§ 42-3505.01(k)(3)"));
    }

    #[test]
    fn dc_at_95f_boundary_eviction_permitted() {
        // Strictly ABOVE 95°F triggers restriction.
        let r = check(&input(Regime::DistrictOfColumbia, 95, false, false, false));
        assert!(r.eviction_permitted);
    }

    #[test]
    fn dc_precipitation_blocks_eviction() {
        let r = check(&input(Regime::DistrictOfColumbia, 70, true, false, false));
        assert!(!r.eviction_permitted);
        assert_eq!(r.restriction_type, RestrictionType::PrecipitationAtUnit);
        assert!(r.citation.contains("§ 42-3505.01(k)(2)"));
    }

    #[test]
    fn dc_mild_weather_no_precipitation_eviction_permitted() {
        let r = check(&input(Regime::DistrictOfColumbia, 55, false, false, false));
        assert!(r.eviction_permitted);
        assert_eq!(r.restriction_type, RestrictionType::None);
    }

    #[test]
    fn cook_county_holiday_moratorium_blocks() {
        let r = check(&input(Regime::CookCountyIllinois, 50, false, true, false));
        assert!(!r.eviction_permitted);
        assert_eq!(r.restriction_type, RestrictionType::HolidayMoratorium);
        assert!(r.citation.contains("December 19 through January 5"));
    }

    #[test]
    fn cook_county_15f_or_colder_blocks() {
        let r = check(&input(Regime::CookCountyIllinois, 15, false, false, false));
        assert!(!r.eviction_permitted);
        assert_eq!(r.restriction_type, RestrictionType::SubFreezing);
        assert!(r.citation.contains("15°F or COLDER"));
    }

    #[test]
    fn cook_county_14f_blocks() {
        let r = check(&input(Regime::CookCountyIllinois, 14, false, false, false));
        assert!(!r.eviction_permitted);
        assert_eq!(r.restriction_type, RestrictionType::SubFreezing);
    }

    #[test]
    fn cook_county_16f_eviction_permitted() {
        // 16°F > 15°F threshold — eviction permitted.
        let r = check(&input(Regime::CookCountyIllinois, 16, false, false, false));
        assert!(r.eviction_permitted);
    }

    #[test]
    fn cook_county_extreme_weather_blocks() {
        let r = check(&input(Regime::CookCountyIllinois, 45, false, false, true));
        assert!(!r.eviction_permitted);
        assert_eq!(
            r.restriction_type,
            RestrictionType::ExtremeWeatherSafetyThreat
        );
    }

    #[test]
    fn cook_county_normal_conditions_permitted() {
        let r = check(&input(Regime::CookCountyIllinois, 45, false, false, false));
        assert!(r.eviction_permitted);
        assert_eq!(r.restriction_type, RestrictionType::None);
    }

    #[test]
    fn default_no_obligation() {
        let r = check(&input(Regime::Default, 0, true, true, true));
        // Default regime: no statutory restriction even with extreme inputs.
        assert!(r.eviction_permitted);
        assert!(r.citation.contains("No statutory winter"));
    }

    #[test]
    fn dc_extreme_cold_negative_temp_blocks() {
        let r = check(&input(Regime::DistrictOfColumbia, -10, false, false, false));
        assert!(!r.eviction_permitted);
        assert_eq!(r.restriction_type, RestrictionType::SubFreezing);
    }

    #[test]
    fn dc_temperature_priority_over_precipitation() {
        // Both cold AND precipitation. SubFreezing fires first.
        let r = check(&input(Regime::DistrictOfColumbia, 20, true, false, false));
        assert_eq!(r.restriction_type, RestrictionType::SubFreezing);
    }

    #[test]
    fn cook_county_holiday_priority_over_temperature() {
        // Within holiday window AND cold temp. Holiday fires first.
        let r = check(&input(Regime::CookCountyIllinois, 10, false, true, false));
        assert_eq!(r.restriction_type, RestrictionType::HolidayMoratorium);
    }

    #[test]
    fn jurisdiction_routing_dc_cook_default() {
        assert_eq!(
            Regime::for_jurisdiction("DC", "Washington"),
            Regime::DistrictOfColumbia
        );
        assert_eq!(
            Regime::for_jurisdiction("IL", "Cook"),
            Regime::CookCountyIllinois
        );
        assert_eq!(
            Regime::for_jurisdiction("IL", "Cook County"),
            Regime::CookCountyIllinois
        );
        assert_eq!(Regime::for_jurisdiction("IL", "DuPage"), Regime::Default);
        assert_eq!(Regime::for_jurisdiction("NY", "New York"), Regime::Default);
    }

    #[test]
    fn jurisdiction_routing_case_insensitive() {
        assert_eq!(
            Regime::for_jurisdiction("dc", "anywhere"),
            Regime::DistrictOfColumbia
        );
        assert_eq!(
            Regime::for_jurisdiction("il", "cook"),
            Regime::CookCountyIllinois
        );
    }

    #[test]
    fn only_dc_has_precipitation_restriction() {
        // Same precipitation scenario across regimes.
        let dc = check(&input(Regime::DistrictOfColumbia, 50, true, false, false));
        let cook = check(&input(Regime::CookCountyIllinois, 50, true, false, false));
        let d = check(&input(Regime::Default, 50, true, false, false));
        assert_eq!(dc.restriction_type, RestrictionType::PrecipitationAtUnit);
        assert!(cook.eviction_permitted);
        assert!(d.eviction_permitted);
    }

    #[test]
    fn only_dc_has_extreme_heat_restriction() {
        // 100°F scenario across regimes.
        let dc = check(&input(Regime::DistrictOfColumbia, 100, false, false, false));
        let cook = check(&input(Regime::CookCountyIllinois, 100, false, false, false));
        assert_eq!(dc.restriction_type, RestrictionType::ExtremeHeat);
        assert!(cook.eviction_permitted);
    }

    #[test]
    fn only_cook_county_has_holiday_moratorium() {
        // Same within-holiday scenario across regimes.
        let cook = check(&input(Regime::CookCountyIllinois, 50, false, true, false));
        let dc = check(&input(Regime::DistrictOfColumbia, 50, false, true, false));
        let d = check(&input(Regime::Default, 50, false, true, false));
        assert_eq!(cook.restriction_type, RestrictionType::HolidayMoratorium);
        // DC + Default ignore the holiday flag.
        assert!(dc.eviction_permitted);
        assert!(d.eviction_permitted);
    }

    #[test]
    fn dc_threshold_chronology_31_32_95_96() {
        // DC: 31°F blocks (sub-freezing), 32°F permitted, 95°F permitted,
        // 96°F blocks (extreme heat).
        let r31 = check(&input(Regime::DistrictOfColumbia, 31, false, false, false));
        let r32 = check(&input(Regime::DistrictOfColumbia, 32, false, false, false));
        let r95 = check(&input(Regime::DistrictOfColumbia, 95, false, false, false));
        let r96 = check(&input(Regime::DistrictOfColumbia, 96, false, false, false));
        assert!(!r31.eviction_permitted);
        assert!(r32.eviction_permitted);
        assert!(r95.eviction_permitted);
        assert!(!r96.eviction_permitted);
    }

    #[test]
    fn citations_pin_correct_authorities() {
        let dc_cold = check(&input(Regime::DistrictOfColumbia, 31, false, false, false));
        assert!(dc_cold.citation.contains("§ 42-3505.01(k)(1)"));

        let dc_heat = check(&input(Regime::DistrictOfColumbia, 96, false, false, false));
        assert!(dc_heat.citation.contains("§ 42-3505.01(k)(3)"));

        let dc_precip = check(&input(Regime::DistrictOfColumbia, 50, true, false, false));
        assert!(dc_precip.citation.contains("§ 42-3505.01(k)(2)"));

        let cook = check(&input(Regime::CookCountyIllinois, 10, false, false, false));
        assert!(cook.citation.contains("Cook County Sheriff Order"));
    }
}
