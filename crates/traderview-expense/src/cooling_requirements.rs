//! State + municipal landlord cooling / maximum-indoor-temperature
//! / AC-installation compliance check.
//!
//! Companion to `heat_requirements` (which addresses winter minimum
//! temperature) — this module addresses summer maximum temperature
//! and the emerging AC-installation mandates. Driven by recent
//! extreme-heat events that pushed AG offices, city councils, and
//! state legislatures to extend habitability law beyond winter
//! heating.
//!
//! Six regimes diverge along five axes:
//!
//!   1. **Maximum-temperature standard** (Phoenix 82°F habitable
//!      rooms / 86°F if evaporative cooler; Dallas indoor ≤ outdoor
//!      − 20°F; NYC bedrooms 78°F when outdoor > 82°F; CA implied
//!      warranty no statutory max; default none).
//!   2. **Statutory cure period for broken cooling** (Arizona 5
//!      days after written notice; Texas 7 days; NYC 60-day
//!      install-request response; CA "reasonable time" under
//!      implied warranty; default none).
//!   3. **Effective-season window** (Phoenix year-round; Dallas
//!      April 1 to November 1; NYC June 15 to September 15;
//!      others year-round if any).
//!   4. **Coverage** (Phoenix all habitable rooms including
//!      bathrooms; Dallas all rentals; NYC bedrooms only).
//!   5. **AC-installation mandate** (NYC Cool Homes for All —
//!      tenant request starts 2028-03-01, enforcement begins
//!      2030-01-01, 60-day landlord response window; all other
//!      regimes only require landlord to MAINTAIN existing
//!      cooling, not INSTALL new cooling).
//!
//! Six regimes:
//!
//!   - **ArizonaPhoenix** — A.R.S. § 33-1364 (cooling as essential
//!     service if part of unit) + Phoenix City Code (82°F max
//!     habitable; 86°F max if evaporative cooler) + AZ AG Mayes
//!     2024–2026 enforcement guidance + AZ SB 1608 (2026 proposed
//!     statewide minimum/maximum indoor temperature standards).
//!
//!   - **ArizonaTucson** — A.R.S. § 33-1364 + Tucson City Code
//!     parallel 82°F cap.
//!
//!   - **Dallas** — Dallas City Code Chapter 27 refrigerated-air
//!     requirement: indoor ≤ outdoor − 20°F during April 1 to
//!     November 1; Tex. Prop. Code § 92.052 7-day repair window for
//!     conditions materially affecting health and safety.
//!
//!   - **NYCCoolHomes** — N.Y.C. Admin. Code Cool Homes for All
//!     (Int 0994-2024): tenant request starts 2028-03-01,
//!     enforcement begins 2030-01-01; bedrooms must be ≤ 78°F when
//!     outdoor temperature exceeds 82°F during June 15 to September
//!     15; 60-day landlord response window for installation
//!     requests; covers both market-rate and rent-stabilized units.
//!
//!   - **California** — Cal. Civ. Code § 1941.1 implied warranty of
//!     habitability covers existing cooling systems (landlord must
//!     repair) but does NOT impose statewide maximum temperature or
//!     install mandate.
//!
//!   - **Default** — no statewide cooling requirement; landlord
//!     must maintain existing cooling as "essential service" under
//!     state implied warranty of habitability where applicable.
//!
//! Citations: A.R.S. § 33-1364 (Arizona essential-service rule);
//! Phoenix City Code § 39-5 (max temperature); Dallas City Code
//! Chapter 27 (refrigerated air); Tex. Prop. Code § 92.052
//! (7-day repair); N.Y.C. Admin. Code Cool Homes for All (Int
//! 0994-2024); Cal. Civ. Code § 1941.1 (CA implied warranty).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    ArizonaPhoenix,
    ArizonaTucson,
    Dallas,
    NYCCoolHomes,
    California,
    Default,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub regime: Regime,
    /// Indoor temperature measured in habitable room (°F).
    pub measured_temperature_f: i32,
    /// Outdoor temperature at time of measurement (°F). Used by
    /// Dallas (relative cap) and NYC (trigger threshold).
    pub outdoor_temperature_f: i32,
    /// True if the unit's cooling system is an evaporative cooler
    /// (Arizona regimes apply a higher 86°F max for evaporative
    /// systems versus the 82°F cap for refrigerated AC).
    pub is_evaporative_cooler: bool,
    /// Days elapsed since the tenant gave written notice of the
    /// cooling failure to the landlord.
    pub days_since_written_notice: u32,
    /// Whether a cooling system exists in the unit. If false, no
    /// "maintain existing cooling" duty arises; install mandate
    /// depends on regime.
    pub cooling_system_in_unit: bool,
    /// Days elapsed since the tenant requested AC INSTALLATION
    /// (relevant only under the NYC Cool Homes for All install
    /// mandate).
    pub days_since_tenant_request_for_install: u32,
    /// Calendar day of year (1–366). Used to determine whether the
    /// measurement falls within the regime's effective-season
    /// window (Dallas April 1 to November 1; NYC June 15 to
    /// September 15).
    pub day_of_year: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CheckResult {
    /// Statutory maximum indoor temperature in °F. None where the
    /// regime does not set a statutory cap.
    pub maximum_temperature_threshold_f: Option<i32>,
    /// Whether the measured temperature is within the threshold.
    /// None where the regime has no threshold or measurement is
    /// outside the effective season.
    pub compliant_with_temperature: Option<bool>,
    /// Statutory cure period in days for broken cooling. None
    /// where no statutory cure period applies.
    pub statutory_cure_period_days: Option<u32>,
    /// Whether the days-since-written-notice exceeds the cure
    /// period. False if no cure period applies.
    pub cure_period_expired: bool,
    /// Whether the regime imposes an AC-INSTALLATION mandate
    /// (only true for NYC Cool Homes for All).
    pub install_mandate: bool,
    /// Landlord's response window for an installation request
    /// (days). None where no install mandate applies.
    pub install_response_window_days: Option<u32>,
    /// Whether the install response is overdue. False where no
    /// install mandate applies or no request has been pending.
    pub install_response_overdue: bool,
    /// Whether the measurement falls inside the regime's effective
    /// season window.
    pub in_effective_season: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

/// Phoenix (and other AZ city codes) — refrigerated AC max.
pub const ARIZONA_REFRIGERATED_MAX_F: i32 = 82;
/// Phoenix — evaporative cooler max (higher than refrigerated).
pub const ARIZONA_EVAPORATIVE_MAX_F: i32 = 86;
/// Dallas — required differential below outdoor temperature.
pub const DALLAS_DIFFERENTIAL_F: i32 = 20;
/// NYC Cool Homes for All — bedroom max when outdoor > 82°F.
pub const NYC_BEDROOM_MAX_F: i32 = 78;
/// NYC trigger — outdoor temperature above which the bedroom max
/// applies.
pub const NYC_OUTDOOR_TRIGGER_F: i32 = 82;
/// NYC install-request response window.
pub const NYC_INSTALL_RESPONSE_WINDOW_DAYS: u32 = 60;
/// Arizona — written-notice cure period for broken cooling.
pub const ARIZONA_CURE_DAYS: u32 = 5;
/// Texas (Dallas) — written-notice cure period for repair.
pub const TEXAS_CURE_DAYS: u32 = 7;

/// Dallas refrigerated-air season — April 1 to November 1.
pub const DALLAS_SEASON_START_DOY: u32 = 91;
pub const DALLAS_SEASON_END_DOY: u32 = 305;
/// NYC Cool Homes for All season — June 15 to September 15.
pub const NYC_SEASON_START_DOY: u32 = 166;
pub const NYC_SEASON_END_DOY: u32 = 258;

pub fn check(input: &Input) -> CheckResult {
    let mut violations: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    let (
        threshold_f,
        cure_period_days,
        install_mandate,
        install_window_days,
        in_season,
        citation,
    ): (Option<i32>, Option<u32>, bool, Option<u32>, bool, &'static str) = match input.regime {
        Regime::ArizonaPhoenix | Regime::ArizonaTucson => {
            let threshold = if input.is_evaporative_cooler {
                ARIZONA_EVAPORATIVE_MAX_F
            } else {
                ARIZONA_REFRIGERATED_MAX_F
            };
            (
                Some(threshold),
                Some(ARIZONA_CURE_DAYS),
                false,
                None,
                true,
                "A.R.S. § 33-1364 (cooling as essential service if part of unit; 5-day cure \
                 period after written notice); Phoenix / Tucson City Code maximum-temperature \
                 standards (82°F refrigerated AC; 86°F evaporative cooler in habitable rooms); \
                 AZ AG Mayes enforcement guidance; AZ SB 1608 (2026 proposed statewide \
                 indoor-temperature standards)",
            )
        }
        Regime::Dallas => {
            let in_season = input.day_of_year >= DALLAS_SEASON_START_DOY
                && input.day_of_year <= DALLAS_SEASON_END_DOY;
            let threshold = if in_season {
                Some(input.outdoor_temperature_f.saturating_sub(DALLAS_DIFFERENTIAL_F))
            } else {
                None
            };
            (
                threshold,
                Some(TEXAS_CURE_DAYS),
                false,
                None,
                in_season,
                "Dallas City Code Chapter 27 (refrigerated air; indoor ≤ outdoor − 20°F during \
                 April 1 to November 1); Tex. Prop. Code § 92.052 (7-day repair window for \
                 conditions materially affecting health and safety)",
            )
        }
        Regime::NYCCoolHomes => {
            let in_season = input.day_of_year >= NYC_SEASON_START_DOY
                && input.day_of_year <= NYC_SEASON_END_DOY;
            let trigger_met = input.outdoor_temperature_f > NYC_OUTDOOR_TRIGGER_F;
            let threshold = if in_season && trigger_met {
                Some(NYC_BEDROOM_MAX_F)
            } else {
                None
            };
            (
                threshold,
                None,
                true,
                Some(NYC_INSTALL_RESPONSE_WINDOW_DAYS),
                in_season,
                "N.Y.C. Admin. Code Cool Homes for All (Int 0994-2024) — tenant request begins \
                 2028-03-01; enforcement begins 2030-01-01; bedrooms ≤ 78°F when outdoor > \
                 82°F during June 15 to September 15; 60-day landlord response window for \
                 installation requests; covers market-rate AND rent-stabilized units",
            )
        }
        Regime::California => (
            None,
            None,
            false,
            None,
            true,
            "Cal. Civ. Code § 1941.1 (implied warranty of habitability covers existing cooling; \
             no statewide maximum-temperature cap or install mandate; landlord must repair \
             existing cooling within reasonable time)",
        ),
        Regime::Default => (
            None,
            None,
            false,
            None,
            true,
            "No statewide cooling requirement; landlord must maintain existing cooling as \
             essential service under state implied warranty of habitability where applicable",
        ),
    };

    let compliant_with_temperature = threshold_f.map(|t| input.measured_temperature_f <= t);

    if let (Some(threshold), Some(false)) = (threshold_f, compliant_with_temperature) {
        violations.push(format!(
            "Measured indoor temperature {}°F exceeds the regime's maximum of {}°F (outdoor {}°F).",
            input.measured_temperature_f, threshold, input.outdoor_temperature_f,
        ));
    }

    let cure_period_expired = match cure_period_days {
        Some(days)
            if input.cooling_system_in_unit
                && compliant_with_temperature == Some(false)
                && input.days_since_written_notice > days =>
        {
            violations.push(format!(
                "Cooling failure not repaired within statutory cure period: notice given {} \
                 days ago; statutory cure period is {} days.",
                input.days_since_written_notice, days,
            ));
            true
        }
        _ => false,
    };

    let install_response_overdue = match install_window_days {
        Some(days)
            if !input.cooling_system_in_unit
                && input.days_since_tenant_request_for_install > days =>
        {
            violations.push(format!(
                "NYC Cool Homes for All installation request not fulfilled within statutory \
                 60-day response window: request made {} days ago.",
                input.days_since_tenant_request_for_install,
            ));
            true
        }
        _ => false,
    };

    if matches!(input.regime, Regime::Dallas) && !in_season {
        notes.push(format!(
            "Dallas refrigerated-air requirement applies April 1 to November 1 (DOY {}–{}); \
             measurement on DOY {} is outside the effective season — no statutory maximum \
             applies.",
            DALLAS_SEASON_START_DOY, DALLAS_SEASON_END_DOY, input.day_of_year,
        ));
    }

    if matches!(input.regime, Regime::NYCCoolHomes) {
        if !in_season {
            notes.push(format!(
                "NYC Cool Homes for All bedroom-temperature cap applies June 15 to September \
                 15 (DOY {}–{}); measurement on DOY {} is outside the effective season — no \
                 statutory bedroom maximum applies.",
                NYC_SEASON_START_DOY, NYC_SEASON_END_DOY, input.day_of_year,
            ));
        } else if input.outdoor_temperature_f <= NYC_OUTDOOR_TRIGGER_F {
            notes.push(format!(
                "NYC Cool Homes for All bedroom-temperature cap triggers only when outdoor \
                 temperature exceeds 82°F; current outdoor temperature {}°F does not trigger \
                 the bedroom max.",
                input.outdoor_temperature_f,
            ));
        }
    }

    notes.push(
        "Sibling to heat_requirements (winter minimum temperature). This module addresses the \
         summer maximum-temperature axis and emerging AC-installation mandates."
            .to_string(),
    );

    CheckResult {
        maximum_temperature_threshold_f: threshold_f,
        compliant_with_temperature,
        statutory_cure_period_days: cure_period_days,
        cure_period_expired,
        install_mandate,
        install_response_window_days: install_window_days,
        install_response_overdue,
        in_effective_season: in_season,
        violations,
        citation,
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base(regime: Regime) -> Input {
        Input {
            regime,
            measured_temperature_f: 75,
            outdoor_temperature_f: 95,
            is_evaporative_cooler: false,
            days_since_written_notice: 0,
            cooling_system_in_unit: true,
            days_since_tenant_request_for_install: 0,
            day_of_year: 200, // mid-July — inside all seasons
        }
    }

    // ── Arizona Phoenix — 82°F refrigerated, 86°F evaporative ──

    #[test]
    fn phoenix_refrigerated_at_82_boundary_compliant() {
        let mut i = base(Regime::ArizonaPhoenix);
        i.measured_temperature_f = 82;
        let r = check(&i);
        assert_eq!(r.maximum_temperature_threshold_f, Some(82));
        assert_eq!(r.compliant_with_temperature, Some(true));
        assert!(r.violations.is_empty());
    }

    #[test]
    fn phoenix_refrigerated_above_82_violation() {
        let mut i = base(Regime::ArizonaPhoenix);
        i.measured_temperature_f = 85;
        let r = check(&i);
        assert_eq!(r.compliant_with_temperature, Some(false));
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("85") && v.contains("82")));
        assert!(r.citation.contains("A.R.S. § 33-1364"));
    }

    #[test]
    fn phoenix_evaporative_at_86_boundary_compliant() {
        let mut i = base(Regime::ArizonaPhoenix);
        i.is_evaporative_cooler = true;
        i.measured_temperature_f = 86;
        let r = check(&i);
        assert_eq!(r.maximum_temperature_threshold_f, Some(86));
        assert_eq!(r.compliant_with_temperature, Some(true));
    }

    #[test]
    fn phoenix_evaporative_above_86_violation() {
        let mut i = base(Regime::ArizonaPhoenix);
        i.is_evaporative_cooler = true;
        i.measured_temperature_f = 87;
        let r = check(&i);
        assert_eq!(r.compliant_with_temperature, Some(false));
    }

    #[test]
    fn phoenix_5_day_cure_period_violation_on_day_6() {
        let mut i = base(Regime::ArizonaPhoenix);
        i.measured_temperature_f = 90;
        i.days_since_written_notice = 6;
        let r = check(&i);
        assert!(r.cure_period_expired);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("cure period") && v.contains("5")));
    }

    #[test]
    fn phoenix_at_5_day_cure_boundary_not_yet_expired() {
        let mut i = base(Regime::ArizonaPhoenix);
        i.measured_temperature_f = 90;
        i.days_since_written_notice = 5;
        let r = check(&i);
        assert!(!r.cure_period_expired);
    }

    // ── Arizona Tucson mirrors Phoenix ────────────────────────

    #[test]
    fn tucson_mirrors_phoenix_82_max() {
        let mut i = base(Regime::ArizonaTucson);
        i.measured_temperature_f = 83;
        let r = check(&i);
        assert_eq!(r.maximum_temperature_threshold_f, Some(82));
        assert_eq!(r.compliant_with_temperature, Some(false));
    }

    // ── Dallas refrigerated-air rule (relative to outdoor) ────

    #[test]
    fn dallas_109_outdoor_89_indoor_at_boundary_compliant() {
        let mut i = base(Regime::Dallas);
        i.outdoor_temperature_f = 109;
        i.measured_temperature_f = 89;
        let r = check(&i);
        assert_eq!(r.maximum_temperature_threshold_f, Some(89));
        assert_eq!(r.compliant_with_temperature, Some(true));
    }

    #[test]
    fn dallas_109_outdoor_91_indoor_violation() {
        let mut i = base(Regime::Dallas);
        i.outdoor_temperature_f = 109;
        i.measured_temperature_f = 91;
        let r = check(&i);
        assert_eq!(r.maximum_temperature_threshold_f, Some(89));
        assert_eq!(r.compliant_with_temperature, Some(false));
    }

    #[test]
    fn dallas_outside_season_no_requirement() {
        let mut i = base(Regime::Dallas);
        i.day_of_year = 50; // mid-February
        i.measured_temperature_f = 100;
        i.outdoor_temperature_f = 95;
        let r = check(&i);
        assert!(!r.in_effective_season);
        assert_eq!(r.maximum_temperature_threshold_f, None);
        assert_eq!(r.compliant_with_temperature, None);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("outside the effective season")));
    }

    #[test]
    fn dallas_7_day_cure_period() {
        let mut i = base(Regime::Dallas);
        i.outdoor_temperature_f = 109;
        i.measured_temperature_f = 95;
        i.days_since_written_notice = 8;
        let r = check(&i);
        assert!(r.cure_period_expired);
        assert!(r.violations.iter().any(|v| v.contains("7")));
    }

    #[test]
    fn dallas_at_april_1_season_start_boundary_in_season() {
        let mut i = base(Regime::Dallas);
        i.day_of_year = DALLAS_SEASON_START_DOY;
        let r = check(&i);
        assert!(r.in_effective_season);
    }

    #[test]
    fn dallas_at_november_1_season_end_boundary_in_season() {
        let mut i = base(Regime::Dallas);
        i.day_of_year = DALLAS_SEASON_END_DOY;
        let r = check(&i);
        assert!(r.in_effective_season);
    }

    // ── NYC Cool Homes for All ────────────────────────────────

    #[test]
    fn nyc_in_season_outdoor_above_82_bedroom_max_78() {
        let mut i = base(Regime::NYCCoolHomes);
        i.outdoor_temperature_f = 90;
        i.measured_temperature_f = 79;
        let r = check(&i);
        assert!(r.in_effective_season);
        assert_eq!(r.maximum_temperature_threshold_f, Some(78));
        assert_eq!(r.compliant_with_temperature, Some(false));
        assert!(r.citation.contains("Cool Homes for All"));
    }

    #[test]
    fn nyc_in_season_outdoor_at_82_does_not_trigger_max() {
        // Outdoor exactly 82°F does NOT exceed; trigger needs > 82.
        let mut i = base(Regime::NYCCoolHomes);
        i.outdoor_temperature_f = 82;
        i.measured_temperature_f = 85;
        let r = check(&i);
        assert_eq!(r.maximum_temperature_threshold_f, None);
        assert_eq!(r.compliant_with_temperature, None);
        assert!(r.notes.iter().any(|n| n.contains("does not trigger")));
    }

    #[test]
    fn nyc_outside_season_no_requirement() {
        let mut i = base(Regime::NYCCoolHomes);
        i.day_of_year = 100; // April — before June 15
        i.outdoor_temperature_f = 95;
        i.measured_temperature_f = 90;
        let r = check(&i);
        assert!(!r.in_effective_season);
        assert_eq!(r.maximum_temperature_threshold_f, None);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("outside the effective season")));
    }

    #[test]
    fn nyc_install_mandate_request_within_60_days_not_overdue() {
        let mut i = base(Regime::NYCCoolHomes);
        i.cooling_system_in_unit = false;
        i.days_since_tenant_request_for_install = 30;
        let r = check(&i);
        assert!(r.install_mandate);
        assert_eq!(r.install_response_window_days, Some(60));
        assert!(!r.install_response_overdue);
    }

    #[test]
    fn nyc_install_mandate_request_past_60_days_overdue() {
        let mut i = base(Regime::NYCCoolHomes);
        i.cooling_system_in_unit = false;
        i.days_since_tenant_request_for_install = 61;
        let r = check(&i);
        assert!(r.install_response_overdue);
        assert!(r.violations.iter().any(|v| v.contains("60-day")));
    }

    #[test]
    fn nyc_install_at_60_day_boundary_not_overdue() {
        let mut i = base(Regime::NYCCoolHomes);
        i.cooling_system_in_unit = false;
        i.days_since_tenant_request_for_install = 60;
        let r = check(&i);
        assert!(!r.install_response_overdue);
    }

    // ── California implied warranty ───────────────────────────

    #[test]
    fn california_no_statutory_max_returns_none() {
        let r = check(&base(Regime::California));
        assert_eq!(r.maximum_temperature_threshold_f, None);
        assert_eq!(r.compliant_with_temperature, None);
        assert!(!r.install_mandate);
        assert!(r.citation.contains("§ 1941.1"));
    }

    // ── Default ──────────────────────────────────────────────

    #[test]
    fn default_no_statutory_requirement() {
        let r = check(&base(Regime::Default));
        assert_eq!(r.maximum_temperature_threshold_f, None);
        assert!(!r.install_mandate);
        assert_eq!(r.statutory_cure_period_days, None);
    }

    // ── Regression-critical multi-regime invariants ──────────

    #[test]
    fn only_nyc_imposes_install_mandate_6_regime_invariant() {
        let nyc = check(&base(Regime::NYCCoolHomes));
        assert!(nyc.install_mandate);
        for &regime in &[
            Regime::ArizonaPhoenix,
            Regime::ArizonaTucson,
            Regime::Dallas,
            Regime::California,
            Regime::Default,
        ] {
            let r = check(&base(regime));
            assert!(
                !r.install_mandate,
                "{:?}: must NOT impose install mandate",
                regime,
            );
        }
    }

    #[test]
    fn only_dallas_uses_relative_to_outdoor_threshold_invariant() {
        // Same indoor + outdoor across regimes; Dallas threshold
        // changes with outdoor temperature; others don't.
        let mut dallas_low = base(Regime::Dallas);
        dallas_low.outdoor_temperature_f = 85;
        let mut dallas_high = base(Regime::Dallas);
        dallas_high.outdoor_temperature_f = 110;
        assert_ne!(
            check(&dallas_low).maximum_temperature_threshold_f,
            check(&dallas_high).maximum_temperature_threshold_f,
            "Dallas threshold must vary with outdoor temperature",
        );

        let mut phx_low = base(Regime::ArizonaPhoenix);
        phx_low.outdoor_temperature_f = 85;
        let mut phx_high = base(Regime::ArizonaPhoenix);
        phx_high.outdoor_temperature_f = 110;
        assert_eq!(
            check(&phx_low).maximum_temperature_threshold_f,
            check(&phx_high).maximum_temperature_threshold_f,
            "Phoenix threshold must be constant across outdoor temperatures",
        );
    }

    #[test]
    fn only_arizona_regimes_have_evaporative_cooler_carve_out_invariant() {
        // AZ-Phoenix + AZ-Tucson: evaporative bumps threshold to
        // 86°F. Other regimes ignore the flag.
        for &regime in &[Regime::ArizonaPhoenix, Regime::ArizonaTucson] {
            let std = base(regime);
            let mut evap = base(regime);
            evap.is_evaporative_cooler = true;
            assert_eq!(check(&std).maximum_temperature_threshold_f, Some(82));
            assert_eq!(check(&evap).maximum_temperature_threshold_f, Some(86));
        }
        // Dallas + NYC + CA + Default: evap flag has no effect.
        for &regime in &[
            Regime::Dallas,
            Regime::NYCCoolHomes,
            Regime::California,
            Regime::Default,
        ] {
            let std = base(regime);
            let mut evap = base(regime);
            evap.is_evaporative_cooler = true;
            assert_eq!(
                check(&std).maximum_temperature_threshold_f,
                check(&evap).maximum_temperature_threshold_f,
                "{:?}: evap flag must be a no-op",
                regime,
            );
        }
    }

    #[test]
    fn cure_period_only_arizona_5_dallas_7_invariant() {
        assert_eq!(
            check(&base(Regime::ArizonaPhoenix)).statutory_cure_period_days,
            Some(5)
        );
        assert_eq!(
            check(&base(Regime::ArizonaTucson)).statutory_cure_period_days,
            Some(5)
        );
        assert_eq!(
            check(&base(Regime::Dallas)).statutory_cure_period_days,
            Some(7)
        );
        for &regime in &[Regime::NYCCoolHomes, Regime::California, Regime::Default] {
            assert_eq!(check(&base(regime)).statutory_cure_period_days, None);
        }
    }

    #[test]
    fn citation_pins_authority_per_regime() {
        assert!(check(&base(Regime::ArizonaPhoenix))
            .citation
            .contains("A.R.S. § 33-1364"));
        assert!(check(&base(Regime::Dallas))
            .citation
            .contains("Dallas City Code Chapter 27"));
        assert!(check(&base(Regime::NYCCoolHomes))
            .citation
            .contains("Int 0994-2024"));
        assert!(check(&base(Regime::California))
            .citation
            .contains("§ 1941.1"));
    }

    #[test]
    fn sibling_module_note_present_across_all_regimes() {
        for &regime in &[
            Regime::ArizonaPhoenix,
            Regime::ArizonaTucson,
            Regime::Dallas,
            Regime::NYCCoolHomes,
            Regime::California,
            Regime::Default,
        ] {
            let r = check(&base(regime));
            assert!(
                r.notes
                    .iter()
                    .any(|n| n.contains("heat_requirements") && n.contains("summer")),
                "{:?}: sibling-module heat_requirements note must be present",
                regime,
            );
        }
    }

    #[test]
    fn no_cooling_in_unit_no_repair_duty_outside_install_mandate() {
        // Where no system is in the unit and no install mandate
        // applies, there is no cure-period violation regardless of
        // notice age.
        let mut i = base(Regime::ArizonaPhoenix);
        i.cooling_system_in_unit = false;
        i.measured_temperature_f = 100;
        i.days_since_written_notice = 365;
        let r = check(&i);
        assert!(!r.cure_period_expired);
    }
}
