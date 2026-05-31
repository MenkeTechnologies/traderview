//! State-by-state heat minimum temperature requirements for residential
//! rentals during heat season.
//!
//! Direct landlord liability obligation during winter months — failure
//! to provide adequate heat is one of the most common habitability
//! violations leading to rent withholding, code enforcement actions, and
//! in extreme cases criminal exposure (M.G.L. c. 186 § 14 felony for
//! willful interruption of heat).
//!
//! Three regimes:
//!
//! 1. **Specific statute / code** — concrete temperature + day/night
//!    split + heat season dates. NY NYC Heat Law, IL Chicago Heat
//!    Ordinance, MA M.G.L. c. 105 CMR 410, MN Minn. Stat. § 504B.161.
//!
//! 2. **Implied habitability covenant** — state law requires "habitable"
//!    premises without a specific temperature. Caller must apply local
//!    code or case law for the numeric threshold.
//!
//! 3. **No state statute** — neither statewide habitability nor heat-
//!    specific rules. Local ordinances may apply.
//!
//! **Boundary thresholds** that matter for compliance:
//!
//! - **Day vs night split**: NY (6am-10pm = day), MA (7am-11pm = day),
//!   Chicago (8:30am-10:30pm = day modeled at 9am-22pm). Many states
//!   have no split (single temperature 24/7).
//! - **Heat season**: NY (Oct 1 - May 31), MA (Sept 15 - Jun 14),
//!   Chicago (Sept 15 - Jun 1), MN (Oct 1 - Apr 30).
//! - **Outside-temperature trigger**: NY only requires the 68°F daytime
//!   minimum when outside temp drops below 55°F; many other states
//!   require continuous compliance during heat season regardless of
//!   outside temperature.

use chrono::{Datelike, NaiveDate};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum HeatRegime {
    /// State has specific statutory or municipal-code temperature + day/
    /// night + heat-season rules.
    SpecificStatuteOrCode,
    /// Implied habitability covenant; no statutory temperature.
    HabitabilityImpliedNoSpecificTemp,
    /// No statewide rule. Local ordinances may apply.
    NoStateStatute,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateHeatRule {
    pub state_code: &'static str,
    pub state_name: &'static str,
    pub regime: HeatRegime,
    /// Minimum indoor temperature during DAYTIME hours (Fahrenheit).
    /// Zero when regime != SpecificStatuteOrCode.
    pub day_temp_f: u32,
    /// Minimum indoor temperature during NIGHTTIME hours. Equal to
    /// day_temp_f when state has no day/night split. Zero when regime
    /// != SpecificStatuteOrCode.
    pub night_temp_f: u32,
    /// Hour-of-day daytime window starts (24-hour, inclusive). NY = 6
    /// (6am), MA = 7 (7am). 0 when no day/night split.
    pub day_start_hour: u32,
    /// Hour-of-day nighttime window starts (24-hour). NY = 22 (10pm),
    /// MA = 23 (11pm). 24 (i.e., never) when no night window.
    pub night_start_hour: u32,
    pub heat_season_start_month: u32,
    pub heat_season_start_day: u32,
    pub heat_season_end_month: u32,
    pub heat_season_end_day: u32,
    /// If `Some(F)`, daytime heat requirement is triggered ONLY when
    /// outside temperature is below F°F. NY uses 55°F. Other states
    /// require continuous compliance regardless of outside temperature.
    pub outside_temp_trigger_f: Option<u32>,
    pub citation: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DayOrNight {
    Day,
    Night,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeatCheckInput {
    pub state_code: String,
    pub measurement_date: NaiveDate,
    /// 24-hour clock hour (0-23).
    pub measurement_hour: u32,
    pub indoor_temp_f: i32,
    pub outdoor_temp_f: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeatCheckResult {
    pub in_heat_season: bool,
    pub day_or_night: DayOrNight,
    pub minimum_required_temp_f: u32,
    /// True if the state's outside-temperature trigger gates the
    /// daytime requirement and outside temp is at/above trigger
    /// (no requirement).
    pub outside_trigger_disengages_requirement: bool,
    pub complies: bool,
    pub shortfall_degrees_f: i32,
    pub no_statute_in_state: bool,
    pub habitability_only: bool,
    pub citation: &'static str,
    pub note: String,
}

pub fn lookup(state_code: &str) -> Option<&'static StateHeatRule> {
    let up = state_code.to_ascii_uppercase();
    TABLE.get(up.as_str()).copied()
}

pub fn all_states() -> Vec<&'static StateHeatRule> {
    let mut v: Vec<&'static StateHeatRule> = TABLE.values().copied().collect();
    v.sort_by_key(|r| r.state_code);
    v
}

/// Heat season spans the calendar boundary (e.g., Oct 1 - May 31 wraps
/// past Dec 31). Returns true if `date` is within the season.
fn in_heat_season(rule: &StateHeatRule, date: NaiveDate) -> bool {
    let m = date.month();
    let d = date.day();
    let start_m = rule.heat_season_start_month;
    let start_d = rule.heat_season_start_day;
    let end_m = rule.heat_season_end_month;
    let end_d = rule.heat_season_end_day;

    // Season wraps if start_month > end_month.
    let wraps = start_m > end_m;
    let after_start = m > start_m || (m == start_m && d >= start_d);
    let before_end = m < end_m || (m == end_m && d <= end_d);
    if wraps {
        after_start || before_end
    } else {
        after_start && before_end
    }
}

fn is_daytime(rule: &StateHeatRule, hour: u32) -> bool {
    // Daytime is [day_start_hour, night_start_hour). Wrap-around handled
    // when night_start_hour < day_start_hour (rare; doesn't occur in
    // any modeled state but defensively coded).
    if rule.day_start_hour == 0 && rule.night_start_hour == 24 {
        true // no split → always "day"
    } else if rule.day_start_hour <= rule.night_start_hour {
        hour >= rule.day_start_hour && hour < rule.night_start_hour
    } else {
        hour >= rule.day_start_hour || hour < rule.night_start_hour
    }
}

pub fn check(input: &HeatCheckInput) -> HeatCheckResult {
    let rule = match lookup(&input.state_code) {
        Some(r) => r,
        None => {
            return HeatCheckResult {
                in_heat_season: false,
                day_or_night: DayOrNight::Day,
                minimum_required_temp_f: 0,
                outside_trigger_disengages_requirement: false,
                complies: false,
                shortfall_degrees_f: 0,
                no_statute_in_state: true,
                habitability_only: false,
                citation: "n/a",
                note: format!("unknown state code `{}`", input.state_code),
            };
        }
    };

    let no_statute = matches!(rule.regime, HeatRegime::NoStateStatute);
    let habitability_only =
        matches!(rule.regime, HeatRegime::HabitabilityImpliedNoSpecificTemp);

    if no_statute || habitability_only {
        return HeatCheckResult {
            in_heat_season: false,
            day_or_night: DayOrNight::Day,
            minimum_required_temp_f: 0,
            outside_trigger_disengages_requirement: false,
            complies: true,
            shortfall_degrees_f: 0,
            no_statute_in_state: no_statute,
            habitability_only,
            citation: rule.citation,
            note: if no_statute {
                format!(
                    "{}: no statewide heat-temperature statute — local ordinances or case law apply",
                    rule.state_name
                )
            } else {
                format!(
                    "{}: state requires habitable premises but no specific temperature — caller must apply local code or habitability case law",
                    rule.state_name
                )
            },
        };
    }

    let season = in_heat_season(rule, input.measurement_date);
    let day = is_daytime(rule, input.measurement_hour);
    let day_or_night = if day { DayOrNight::Day } else { DayOrNight::Night };

    if !season {
        return HeatCheckResult {
            in_heat_season: false,
            day_or_night,
            minimum_required_temp_f: 0,
            outside_trigger_disengages_requirement: false,
            complies: true,
            shortfall_degrees_f: 0,
            no_statute_in_state: false,
            habitability_only: false,
            citation: rule.citation,
            note: format!(
                "{}: {} is outside heat season ({}/{} – {}/{}); no statutory minimum",
                rule.state_name,
                input.measurement_date,
                rule.heat_season_start_month,
                rule.heat_season_start_day,
                rule.heat_season_end_month,
                rule.heat_season_end_day
            ),
        };
    }

    // Determine required minimum based on day/night.
    let raw_required = if day { rule.day_temp_f } else { rule.night_temp_f };

    // Outside-temperature trigger (NY model): daytime requirement gated
    // when outside is at/above the trigger.
    let outside_disengages = if day {
        rule.outside_temp_trigger_f
            .map(|t| input.outdoor_temp_f >= t as i32)
            .unwrap_or(false)
    } else {
        false // night requirement is unconditional regardless of outside
    };

    let effective_required = if outside_disengages { 0 } else { raw_required };
    let complies = input.indoor_temp_f as i64 >= effective_required as i64;
    let shortfall = (effective_required as i32 - input.indoor_temp_f).max(0);

    let note = if outside_disengages {
        format!(
            "{}: outside {}°F ≥ {}°F trigger — daytime heat requirement DISENGAGED; indoor {}°F",
            rule.state_name,
            input.outdoor_temp_f,
            rule.outside_temp_trigger_f.unwrap_or(0),
            input.indoor_temp_f
        )
    } else if complies {
        format!(
            "{}: {} window requires {}°F; indoor {}°F complies",
            rule.state_name,
            if day { "day" } else { "night" },
            effective_required,
            input.indoor_temp_f
        )
    } else {
        format!(
            "{}: {} window requires {}°F; indoor {}°F — {}°F short of minimum",
            rule.state_name,
            if day { "day" } else { "night" },
            effective_required,
            input.indoor_temp_f,
            shortfall
        )
    };

    HeatCheckResult {
        in_heat_season: true,
        day_or_night,
        minimum_required_temp_f: effective_required,
        outside_trigger_disengages_requirement: outside_disengages,
        complies,
        shortfall_degrees_f: shortfall,
        no_statute_in_state: false,
        habitability_only: false,
        citation: rule.citation,
        note,
    }
}

#[allow(clippy::too_many_arguments)]
const fn rule(
    state_code: &'static str,
    state_name: &'static str,
    regime: HeatRegime,
    day_temp_f: u32,
    night_temp_f: u32,
    day_start_hour: u32,
    night_start_hour: u32,
    heat_season_start_month: u32,
    heat_season_start_day: u32,
    heat_season_end_month: u32,
    heat_season_end_day: u32,
    outside_temp_trigger_f: Option<u32>,
    citation: &'static str,
) -> StateHeatRule {
    StateHeatRule {
        state_code,
        state_name,
        regime,
        day_temp_f,
        night_temp_f,
        day_start_hour,
        night_start_hour,
        heat_season_start_month,
        heat_season_start_day,
        heat_season_end_month,
        heat_season_end_day,
        outside_temp_trigger_f,
        citation,
    }
}

static TABLE: Lazy<HashMap<&'static str, &'static StateHeatRule>> = Lazy::new(|| {
    use HeatRegime::*;
    static RULES: &[StateHeatRule] = &[
        rule("AK", "Alaska", HabitabilityImpliedNoSpecificTemp, 0, 0, 0, 24, 0, 0, 0, 0, None, "AS § 34.03.100 (habitability)"),
        rule("AL", "Alabama", NoStateStatute, 0, 0, 0, 24, 0, 0, 0, 0, None, "no statewide statute"),
        rule("AR", "Arkansas", NoStateStatute, 0, 0, 0, 24, 0, 0, 0, 0, None, "no statewide statute"),
        rule("AZ", "Arizona", HabitabilityImpliedNoSpecificTemp, 0, 0, 0, 24, 0, 0, 0, 0, None, "A.R.S. § 33-1324 (habitability)"),
        rule(
            "CA",
            "California",
            SpecificStatuteOrCode,
            70, 70, 0, 24, 11, 1, 5, 31, None,
            "Cal. Code Regs. tit. 25 § 34 (70°F in living areas)",
        ),
        rule("CO", "Colorado", HabitabilityImpliedNoSpecificTemp, 0, 0, 0, 24, 0, 0, 0, 0, None, "C.R.S. § 38-12-505 (habitability)"),
        rule(
            "CT",
            "Connecticut",
            SpecificStatuteOrCode,
            65, 65, 0, 24, 10, 1, 5, 31, None,
            "Conn. Gen. Stat. § 47a-7 + Conn. Agencies Regs.",
        ),
        rule("DC", "District of Columbia", SpecificStatuteOrCode, 68, 65, 6, 23, 10, 1, 5, 1, None, "14 DCMR § 503"),
        rule("DE", "Delaware", HabitabilityImpliedNoSpecificTemp, 0, 0, 0, 24, 0, 0, 0, 0, None, "25 Del. C. § 5305 (habitability)"),
        rule("FL", "Florida", HabitabilityImpliedNoSpecificTemp, 0, 0, 0, 24, 0, 0, 0, 0, None, "Fla. Stat. § 83.51 (habitability)"),
        rule("GA", "Georgia", NoStateStatute, 0, 0, 0, 24, 0, 0, 0, 0, None, "no statewide statute"),
        rule("HI", "Hawaii", NoStateStatute, 0, 0, 0, 24, 0, 0, 0, 0, None, "no statewide statute (no heat needed)"),
        rule("IA", "Iowa", HabitabilityImpliedNoSpecificTemp, 0, 0, 0, 24, 0, 0, 0, 0, None, "Iowa Code § 562A.15 (habitability)"),
        rule("ID", "Idaho", NoStateStatute, 0, 0, 0, 24, 0, 0, 0, 0, None, "no statewide statute"),
        rule(
            "IL",
            "Illinois",
            SpecificStatuteOrCode,
            68, 66, 8, 22, 9, 15, 6, 1, None,
            "Chicago Municipal Code § 13-196-410 (Heat Ordinance)",
        ),
        rule("IN", "Indiana", HabitabilityImpliedNoSpecificTemp, 0, 0, 0, 24, 0, 0, 0, 0, None, "Ind. Code § 32-31-8-5 (habitability)"),
        rule("KS", "Kansas", HabitabilityImpliedNoSpecificTemp, 0, 0, 0, 24, 0, 0, 0, 0, None, "K.S.A. § 58-2553 (habitability)"),
        rule("KY", "Kentucky", HabitabilityImpliedNoSpecificTemp, 0, 0, 0, 24, 0, 0, 0, 0, None, "KRS § 383.595 (habitability)"),
        rule("LA", "Louisiana", NoStateStatute, 0, 0, 0, 24, 0, 0, 0, 0, None, "no statewide statute"),
        rule(
            "MA",
            "Massachusetts",
            SpecificStatuteOrCode,
            68, 64, 7, 23, 9, 16, 6, 14, None,
            "M.G.L. c. 105 § 410.201 (state sanitary code)",
        ),
        rule(
            "MD",
            "Maryland",
            SpecificStatuteOrCode,
            68, 65, 7, 23, 10, 1, 5, 1, None,
            "Md. Code Real Prop. § 8-211 + Baltimore Heat Code",
        ),
        rule("ME", "Maine", HabitabilityImpliedNoSpecificTemp, 0, 0, 0, 24, 0, 0, 0, 0, None, "14 M.R.S. § 6021 (habitability)"),
        rule("MI", "Michigan", HabitabilityImpliedNoSpecificTemp, 0, 0, 0, 24, 0, 0, 0, 0, None, "MCL § 554.139 (habitability)"),
        rule(
            "MN",
            "Minnesota",
            SpecificStatuteOrCode,
            68, 68, 0, 24, 10, 1, 4, 30, None,
            "Minn. Stat. § 504B.161",
        ),
        rule("MO", "Missouri", NoStateStatute, 0, 0, 0, 24, 0, 0, 0, 0, None, "no statewide statute"),
        rule("MS", "Mississippi", NoStateStatute, 0, 0, 0, 24, 0, 0, 0, 0, None, "no statewide statute"),
        rule("MT", "Montana", HabitabilityImpliedNoSpecificTemp, 0, 0, 0, 24, 0, 0, 0, 0, None, "Mont. Code § 70-24-303 (habitability)"),
        rule("NC", "North Carolina", HabitabilityImpliedNoSpecificTemp, 0, 0, 0, 24, 0, 0, 0, 0, None, "N.C.G.S. § 42-42 (habitability)"),
        rule("ND", "North Dakota", HabitabilityImpliedNoSpecificTemp, 0, 0, 0, 24, 0, 0, 0, 0, None, "N.D.C.C. § 47-16-13.1 (habitability)"),
        rule("NE", "Nebraska", HabitabilityImpliedNoSpecificTemp, 0, 0, 0, 24, 0, 0, 0, 0, None, "Neb. Rev. Stat. § 76-1419 (habitability)"),
        rule("NH", "New Hampshire", HabitabilityImpliedNoSpecificTemp, 0, 0, 0, 24, 0, 0, 0, 0, None, "RSA § 540-A (habitability)"),
        rule(
            "NJ",
            "New Jersey",
            SpecificStatuteOrCode,
            68, 65, 6, 23, 10, 1, 5, 1, None,
            "N.J.A.C. § 5:10-14.3",
        ),
        rule("NM", "New Mexico", HabitabilityImpliedNoSpecificTemp, 0, 0, 0, 24, 0, 0, 0, 0, None, "NMSA § 47-8-20 (habitability)"),
        rule("NV", "Nevada", HabitabilityImpliedNoSpecificTemp, 0, 0, 0, 24, 0, 0, 0, 0, None, "NRS § 118A.290 (habitability)"),
        rule(
            "NY",
            "New York",
            SpecificStatuteOrCode,
            68, 62, 6, 22, 10, 1, 5, 31, Some(55),
            "NYC Admin. Code § 27-2029 (Heat Law)",
        ),
        rule("OH", "Ohio", HabitabilityImpliedNoSpecificTemp, 0, 0, 0, 24, 0, 0, 0, 0, None, "ORC § 5321.04 (habitability)"),
        rule("OK", "Oklahoma", HabitabilityImpliedNoSpecificTemp, 0, 0, 0, 24, 0, 0, 0, 0, None, "41 O.S. § 118 (habitability)"),
        rule(
            "OR",
            "Oregon",
            SpecificStatuteOrCode,
            68, 60, 6, 22, 10, 1, 5, 1, None,
            "ORS § 90.320 + ORS § 446.250",
        ),
        rule("PA", "Pennsylvania", HabitabilityImpliedNoSpecificTemp, 0, 0, 0, 24, 0, 0, 0, 0, None, "Pugh v. Holmes (1979) habitability case law"),
        rule(
            "RI",
            "Rhode Island",
            SpecificStatuteOrCode,
            68, 64, 6, 23, 10, 1, 5, 1, None,
            "R.I. Gen. Laws § 45-24.3-8",
        ),
        rule("SC", "South Carolina", HabitabilityImpliedNoSpecificTemp, 0, 0, 0, 24, 0, 0, 0, 0, None, "S.C. Code § 27-40-440 (habitability)"),
        rule("SD", "South Dakota", NoStateStatute, 0, 0, 0, 24, 0, 0, 0, 0, None, "no statewide statute"),
        rule("TN", "Tennessee", HabitabilityImpliedNoSpecificTemp, 0, 0, 0, 24, 0, 0, 0, 0, None, "Tenn. Code § 66-28-304 (habitability)"),
        rule("TX", "Texas", HabitabilityImpliedNoSpecificTemp, 0, 0, 0, 24, 0, 0, 0, 0, None, "Tex. Prop. Code § 92.052 (habitability)"),
        rule("UT", "Utah", HabitabilityImpliedNoSpecificTemp, 0, 0, 0, 24, 0, 0, 0, 0, None, "Utah Code § 57-22-4 (habitability)"),
        rule(
            "VA",
            "Virginia",
            SpecificStatuteOrCode,
            68, 65, 6, 23, 10, 15, 5, 1, None,
            "Va. Code § 36-105 + Virginia Maintenance Code",
        ),
        rule("VT", "Vermont", HabitabilityImpliedNoSpecificTemp, 0, 0, 0, 24, 0, 0, 0, 0, None, "9 V.S.A. § 4457 (habitability)"),
        rule(
            "WA",
            "Washington",
            HabitabilityImpliedNoSpecificTemp,
            0, 0, 0, 24, 0, 0, 0, 0, None,
            "RCW § 59.18.060(8) (habitability)",
        ),
        rule(
            "WI",
            "Wisconsin",
            SpecificStatuteOrCode,
            67, 67, 0, 24, 10, 1, 4, 30, None,
            "Wis. Admin. Code § ATCP 134.04",
        ),
        rule("WV", "West Virginia", HabitabilityImpliedNoSpecificTemp, 0, 0, 0, 24, 0, 0, 0, 0, None, "W. Va. Code § 37-6-30 (habitability)"),
        rule("WY", "Wyoming", NoStateStatute, 0, 0, 0, 24, 0, 0, 0, 0, None, "no statewide statute"),
    ];
    RULES.iter().map(|r| (r.state_code, r)).collect()
});

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    fn input(state: &str, date: NaiveDate, hour: u32, indoor: i32, outdoor: i32) -> HeatCheckInput {
        HeatCheckInput {
            state_code: state.to_string(),
            measurement_date: date,
            measurement_hour: hour,
            indoor_temp_f: indoor,
            outdoor_temp_f: outdoor,
        }
    }

    #[test]
    fn table_covers_all_50_states_plus_dc() {
        assert_eq!(TABLE.len(), 51);
    }

    #[test]
    fn ny_day_68f_required_when_outside_below_55() {
        // NY: day = 68°F when outside < 55°F. Measurement 10am, indoor
        // 70°F, outside 40°F → complies.
        let r = check(&input("NY", d(2026, 1, 15), 10, 70, 40));
        assert!(r.in_heat_season);
        assert_eq!(r.day_or_night, DayOrNight::Day);
        assert_eq!(r.minimum_required_temp_f, 68);
        assert!(r.complies);
    }

    #[test]
    fn ny_day_outside_above_55_disengages_requirement() {
        // NY: day with outside ≥ 55°F → no heat requirement.
        let r = check(&input("NY", d(2026, 1, 15), 10, 60, 60));
        assert!(r.outside_trigger_disengages_requirement);
        assert_eq!(r.minimum_required_temp_f, 0);
        assert!(r.complies);
    }

    #[test]
    fn ny_night_62f_required_regardless_of_outside_temp() {
        // NY: night 10pm-6am = 62°F regardless of outside temp.
        let r = check(&input("NY", d(2026, 1, 15), 23, 60, 70)); // 11pm, indoor 60, outside warm
        assert_eq!(r.day_or_night, DayOrNight::Night);
        assert_eq!(r.minimum_required_temp_f, 62);
        assert!(!r.complies);
        assert_eq!(r.shortfall_degrees_f, 2);
    }

    #[test]
    fn ny_outside_heat_season_no_requirement() {
        // NY: heat season Oct 1 - May 31. June 1 is outside.
        let r = check(&input("NY", d(2026, 6, 1), 10, 50, 30));
        assert!(!r.in_heat_season);
        assert!(r.complies);
    }

    #[test]
    fn ny_heat_season_boundary_oct_1_in_season() {
        let r = check(&input("NY", d(2026, 10, 1), 10, 70, 40));
        assert!(r.in_heat_season);
    }

    #[test]
    fn ny_heat_season_boundary_sep_30_out_of_season() {
        let r = check(&input("NY", d(2026, 9, 30), 10, 50, 30));
        assert!(!r.in_heat_season);
    }

    #[test]
    fn ny_heat_season_wraps_past_jan_1() {
        // Jan 15 is in the Oct-May season (wraps year boundary).
        let r = check(&input("NY", d(2026, 1, 15), 10, 70, 40));
        assert!(r.in_heat_season);
    }

    #[test]
    fn ma_day_68_night_64_with_7am_11pm_split() {
        // MA day = 68°F, night = 64°F. Day window 7am-11pm.
        let day = check(&input("MA", d(2026, 1, 15), 10, 70, 30));
        assert!(day.complies);
        assert_eq!(day.minimum_required_temp_f, 68);

        let night = check(&input("MA", d(2026, 1, 15), 23, 65, 30));
        assert!(night.complies);
        assert_eq!(night.minimum_required_temp_f, 64);
    }

    #[test]
    fn ma_no_outside_temp_trigger_continuous_requirement() {
        // MA requires 68°F day continuously regardless of outside temp.
        // Distinguishes from NY which has the 55°F trigger.
        let r = check(&input("MA", d(2026, 1, 15), 10, 67, 70)); // warm outside
        assert!(!r.outside_trigger_disengages_requirement);
        assert!(!r.complies);
        assert_eq!(r.shortfall_degrees_f, 1);
    }

    #[test]
    fn mn_single_temp_no_day_night_split() {
        // MN requires 68°F day or night (no split).
        let mid_day = check(&input("MN", d(2026, 1, 15), 12, 67, 0));
        assert!(!mid_day.complies);
        let mid_night = check(&input("MN", d(2026, 1, 15), 3, 67, 0));
        assert!(!mid_night.complies);
        assert_eq!(mid_day.minimum_required_temp_f, mid_night.minimum_required_temp_f);
    }

    #[test]
    fn ct_65f_minimum_no_day_night_split() {
        let r = check(&input("CT", d(2026, 1, 15), 10, 65, 30));
        assert!(r.complies);
        assert_eq!(r.minimum_required_temp_f, 65);

        let fail = check(&input("CT", d(2026, 1, 15), 10, 64, 30));
        assert!(!fail.complies);
    }

    #[test]
    fn habitability_only_states_return_no_specific_requirement() {
        // States with HabitabilityImpliedNoSpecificTemp regime — no
        // statutory numeric threshold. Result has habitability_only=true
        // and complies=true (deferring to local code).
        for code in ["TX", "FL", "WA", "OH", "PA", "NM", "MT"] {
            let r = check(&input(code, d(2026, 1, 15), 10, 50, 30));
            assert!(r.habitability_only, "{code} should be habitability-only");
            assert!(!r.no_statute_in_state);
            assert!(r.complies);
        }
    }

    #[test]
    fn no_statute_states_return_no_requirement() {
        // States with no statewide heat statute at all.
        for code in ["AL", "AR", "GA", "MO", "MS", "WY", "ID", "LA", "SD"] {
            let r = check(&input(code, d(2026, 1, 15), 10, 30, 0));
            assert!(r.no_statute_in_state, "{code} should be no-statute");
            assert!(r.complies);
        }
    }

    #[test]
    fn hawaii_no_statewide_statute() {
        // Hawaii listed as NoStateStatute (warm climate, no heat needed).
        let r = check(&input("HI", d(2026, 1, 15), 10, 65, 70));
        assert!(r.no_statute_in_state);
    }

    #[test]
    fn ma_heat_season_september_16_in_season() {
        // MA season Sept 16 - Jun 14. Sept 16 = first day, in season.
        let r = check(&input("MA", d(2026, 9, 16), 10, 70, 40));
        assert!(r.in_heat_season);
    }

    #[test]
    fn ma_heat_season_september_15_out_of_season() {
        let r = check(&input("MA", d(2026, 9, 15), 10, 50, 30));
        assert!(!r.in_heat_season);
    }

    #[test]
    fn unknown_state_handled() {
        let r = check(&input("ZZ", d(2026, 1, 15), 10, 70, 40));
        assert!(r.no_statute_in_state);
        assert!(r.note.contains("unknown state code"));
    }

    #[test]
    fn lookup_case_insensitive() {
        assert!(lookup("NY").is_some());
        assert!(lookup("ny").is_some());
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
    fn specific_statute_states_pinned() {
        // NY, MA, IL, MN, CT, NJ, OR, DC, RI, MD, VA, WI, CA have
        // SpecificStatuteOrCode regime.
        for code in ["NY", "MA", "IL", "MN", "CT", "NJ", "OR", "DC", "RI", "MD", "VA", "WI", "CA"] {
            let r = lookup(code).unwrap();
            assert!(
                matches!(r.regime, HeatRegime::SpecificStatuteOrCode),
                "{code} should be SpecificStatuteOrCode"
            );
        }
    }

    #[test]
    fn ny_is_only_state_with_outside_trigger() {
        // NY is the only state with an outside-temp trigger (55°F).
        // Every other state requires continuous compliance during
        // heat season.
        let ny = lookup("NY").unwrap();
        assert!(ny.outside_temp_trigger_f.is_some());
        for r in TABLE.values() {
            if r.state_code != "NY" {
                assert!(
                    r.outside_temp_trigger_f.is_none(),
                    "{} should not have outside trigger",
                    r.state_code
                );
            }
        }
    }

    #[test]
    fn shortfall_calculated_correctly() {
        // Indoor 60, required 68 → shortfall 8.
        let r = check(&input("NY", d(2026, 1, 15), 10, 60, 30));
        assert_eq!(r.shortfall_degrees_f, 8);
    }

    #[test]
    fn ny_day_night_boundary_at_22_hour() {
        // NY day = [6, 22). At hour 22 (10pm) = night.
        let r22 = check(&input("NY", d(2026, 1, 15), 22, 65, 30));
        assert_eq!(r22.day_or_night, DayOrNight::Night);
        let r21 = check(&input("NY", d(2026, 1, 15), 21, 65, 30));
        assert_eq!(r21.day_or_night, DayOrNight::Day);
    }

    #[test]
    fn night_window_outside_trigger_does_not_apply() {
        // Even though NY has outside trigger for DAY, the night
        // requirement (62°F) applies unconditionally.
        let r = check(&input("NY", d(2026, 1, 15), 2, 60, 80)); // 2am, warm
        assert_eq!(r.day_or_night, DayOrNight::Night);
        assert!(!r.outside_trigger_disengages_requirement);
        assert_eq!(r.minimum_required_temp_f, 62);
        assert!(!r.complies);
    }
}
