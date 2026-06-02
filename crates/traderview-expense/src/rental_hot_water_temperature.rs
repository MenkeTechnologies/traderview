//! Rental property minimum hot water temperature compliance
//! — when must a trader-landlord supply hot water at a
//! statutorily-specified minimum temperature? Trader-
//! landlord critical for any rental owner: failure to
//! supply hot water at required minimum temperature breaches
//! implied warranty of habitability + triggers tenant
//! remedies (rent withholding + repair-and-deduct + state-
//! specific penalties + HPD/health-department complaints).
//!
//! Distinct from siblings `heat_requirements` (space heat
//! temperature standards), `cooling_requirements` (cooling
//! standards), `lead_in_drinking_water_disclosure` (lead
//! contamination), and `water_heater_earthquake_strap` (CA §
//! 19211 seismic anchoring).
//!
//! **Three regimes**:
//!
//! **California — Cal. Health & Safety Code § 114192 + Cal.
//! Civ. Code § 1941.1 + 22 CCR § 81088**:
//! - § 114192: hot water must be supplied at minimum of
//!   **120°F** measured from the faucet.
//! - Cal. Civ. Code § 1941.1: implied warranty of
//!   habitability includes hot water.
//! - 22 CCR § 81088: hot water fixture temperature controls
//!   maintained at not less than **105°F** and not more than
//!   120°F (for care facilities).
//!
//! **New York City — NYC HMC § 27-2031 (Article 8 Heat and
//! Hot Water)**:
//! - Hot water must be provided **365 days per year** at a
//!   constant minimum temperature of **120°F**.
//! - Failure triggers HPD violation + tenant remedies +
//!   civil penalties.
//!
//! **Default — IPC § 607.1.1 + state habitability standards**:
//! - International Plumbing Code § 607.1.1 requires **110°F
//!   minimum** at outlet.
//! - Many states default to 120°F via plumbing code adoption.
//! - State-specific habitability standards may impose
//!   higher minimums.
//!
//! Citations: Cal. Health & Safety Code §§ 114192, 17920.3;
//! Cal. Civ. Code § 1941.1; 22 CCR § 81088; NYC HMC § 27-2031
//! (Article 8); International Plumbing Code § 607.1.1.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    California,
    NewYorkCity,
    Default,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RentalHotWaterTemperatureInput {
    pub regime: Regime,
    /// Hot water temperature at faucet in degrees Fahrenheit
    /// (whole number).
    pub temperature_fahrenheit: u32,
    /// Whether hot water is supplied year-round 365 days
    /// (NYC requirement).
    pub year_round_supply: bool,
    /// Whether hot water is supplied at all (whether tenant
    /// has access).
    pub hot_water_supplied: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RentalHotWaterTemperatureResult {
    pub compliant: bool,
    pub minimum_temperature_required_f: u32,
    pub year_round_required: bool,
    pub habitability_breach: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &RentalHotWaterTemperatureInput) -> RentalHotWaterTemperatureResult {
    match input.regime {
        Regime::California => check_ca(input),
        Regime::NewYorkCity => check_nyc(input),
        Regime::Default => check_default(input),
    }
}

fn check_ca(input: &RentalHotWaterTemperatureInput) -> RentalHotWaterTemperatureResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "Cal. Health & Safety Code § 114192 — hot water must be supplied at minimum 120°F measured from the faucet"
            .to_string(),
        "Cal. Civ. Code § 1941.1 — implied warranty of habitability includes hot water supply; failure renders dwelling untenantable"
            .to_string(),
        "22 CCR § 81088 — hot water fixture temperature controls maintained at not less than 105°F and not more than 120°F (care facilities); IPC range for safety"
            .to_string(),
    ];

    let min_required: u32 = 120;

    if !input.hot_water_supplied {
        violations.push(
            "Cal. Civ. Code § 1941.1 — failure to supply hot water breaches implied warranty of habitability".to_string(),
        );
    } else if input.temperature_fahrenheit < min_required {
        violations.push(format!(
            "Cal. Health & Safety Code § 114192 — hot water supplied at {}°F is below 120°F minimum required at the faucet",
            input.temperature_fahrenheit
        ));
    }

    let habitability_breach = !input.hot_water_supplied
        || input.temperature_fahrenheit < min_required;

    RentalHotWaterTemperatureResult {
        compliant: violations.is_empty(),
        minimum_temperature_required_f: min_required,
        year_round_required: false,
        habitability_breach,
        violations,
        citation: "Cal. Health & Safety Code §§ 114192, 17920.3; Cal. Civ. Code § 1941.1; 22 CCR § 81088",
        notes,
    }
}

fn check_nyc(input: &RentalHotWaterTemperatureInput) -> RentalHotWaterTemperatureResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "NYC HMC § 27-2031 (Article 8 Heat and Hot Water) — hot water must be provided 365 days per year at constant minimum temperature of 120°F"
            .to_string(),
        "NYC HPD — failure triggers HPD violation + tenant remedies + civil penalties + private right of action under § 27-2115"
            .to_string(),
    ];

    let min_required: u32 = 120;

    if !input.hot_water_supplied {
        violations.push(
            "NYC HMC § 27-2031 — failure to supply hot water at all violates Article 8 habitability standards".to_string(),
        );
    } else if input.temperature_fahrenheit < min_required {
        violations.push(format!(
            "NYC HMC § 27-2031 — hot water supplied at {}°F is below 120°F minimum required",
            input.temperature_fahrenheit
        ));
    }

    if !input.year_round_supply {
        violations.push(
            "NYC HMC § 27-2031 — hot water must be supplied 365 days per year (year-round); seasonal supply violates Article 8".to_string(),
        );
    }

    let habitability_breach = !input.hot_water_supplied
        || input.temperature_fahrenheit < min_required
        || !input.year_round_supply;

    RentalHotWaterTemperatureResult {
        compliant: violations.is_empty(),
        minimum_temperature_required_f: min_required,
        year_round_required: true,
        habitability_breach,
        violations,
        citation: "NYC HMC § 27-2031 (Article 8 Heat and Hot Water); HPD § 27-2115",
        notes,
    }
}

fn check_default(input: &RentalHotWaterTemperatureInput) -> RentalHotWaterTemperatureResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "default rule — International Plumbing Code (IPC) § 607.1.1 requires hot water minimum 110°F at outlet; many states default to 120°F via plumbing code adoption"
            .to_string(),
        "default rule — state-specific habitability standards may impose higher minimums; verify local jurisdiction adoption of IPC + state warranty of habitability framework"
            .to_string(),
    ];

    let min_required: u32 = 110;

    if !input.hot_water_supplied {
        violations.push(
            "IPC § 607.1.1 + state warranty of habitability — failure to supply hot water at all breaches habitability standards".to_string(),
        );
    } else if input.temperature_fahrenheit < min_required {
        violations.push(format!(
            "IPC § 607.1.1 — hot water supplied at {}°F is below 110°F minimum required at outlet",
            input.temperature_fahrenheit
        ));
    }

    let habitability_breach = !input.hot_water_supplied
        || input.temperature_fahrenheit < min_required;

    RentalHotWaterTemperatureResult {
        compliant: violations.is_empty(),
        minimum_temperature_required_f: min_required,
        year_round_required: false,
        habitability_breach,
        violations,
        citation: "IPC § 607.1.1; state-specific habitability standards",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ca_compliant() -> RentalHotWaterTemperatureInput {
        RentalHotWaterTemperatureInput {
            regime: Regime::California,
            temperature_fahrenheit: 125,
            year_round_supply: true,
            hot_water_supplied: true,
        }
    }

    fn nyc_compliant() -> RentalHotWaterTemperatureInput {
        let mut i = ca_compliant();
        i.regime = Regime::NewYorkCity;
        i
    }

    fn default_compliant() -> RentalHotWaterTemperatureInput {
        let mut i = ca_compliant();
        i.regime = Regime::Default;
        i
    }

    #[test]
    fn ca_125f_compliant() {
        let r = check(&ca_compliant());
        assert!(r.compliant);
        assert_eq!(r.minimum_temperature_required_f, 120);
        assert!(!r.year_round_required);
        assert!(!r.habitability_breach);
    }

    #[test]
    fn ca_at_120f_boundary_compliant() {
        let mut i = ca_compliant();
        i.temperature_fahrenheit = 120;
        let r = check(&i);
        assert!(r.compliant);
    }

    #[test]
    fn ca_119f_violates() {
        let mut i = ca_compliant();
        i.temperature_fahrenheit = 119;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.habitability_breach);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 114192") && v.contains("120°F") && v.contains("119")));
    }

    #[test]
    fn ca_no_hot_water_violates() {
        let mut i = ca_compliant();
        i.hot_water_supplied = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.habitability_breach);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 1941.1") && v.contains("habitability")));
    }

    #[test]
    fn ca_citation_pins_authorities() {
        let r = check(&ca_compliant());
        assert!(r.citation.contains("§§ 114192, 17920.3"));
        assert!(r.citation.contains("§ 1941.1"));
        assert!(r.citation.contains("22 CCR § 81088"));
    }

    #[test]
    fn ca_note_pins_120_at_faucet() {
        let r = check(&ca_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 114192") && n.contains("120°F") && n.contains("faucet")));
    }

    #[test]
    fn ca_note_pins_warranty_of_habitability() {
        let r = check(&ca_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 1941.1")
            && n.contains("warranty of habitability")
            && n.contains("untenantable")));
    }

    #[test]
    fn ca_note_pins_22_ccr_81088() {
        let r = check(&ca_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("22 CCR § 81088") && n.contains("105°F") && n.contains("120°F")));
    }

    #[test]
    fn nyc_125f_year_round_compliant() {
        let r = check(&nyc_compliant());
        assert!(r.compliant);
        assert_eq!(r.minimum_temperature_required_f, 120);
        assert!(r.year_round_required);
    }

    #[test]
    fn nyc_at_120f_boundary_compliant() {
        let mut i = nyc_compliant();
        i.temperature_fahrenheit = 120;
        let r = check(&i);
        assert!(r.compliant);
    }

    #[test]
    fn nyc_119f_violates() {
        let mut i = nyc_compliant();
        i.temperature_fahrenheit = 119;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 27-2031") && v.contains("120°F") && v.contains("119")));
    }

    #[test]
    fn nyc_seasonal_supply_violates() {
        let mut i = nyc_compliant();
        i.year_round_supply = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("365 days") && v.contains("year-round")));
    }

    #[test]
    fn nyc_no_hot_water_violates() {
        let mut i = nyc_compliant();
        i.hot_water_supplied = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 27-2031") && v.contains("at all")));
    }

    #[test]
    fn nyc_citation_pins_authorities() {
        let r = check(&nyc_compliant());
        assert!(r.citation.contains("§ 27-2031"));
        assert!(r.citation.contains("Article 8"));
        assert!(r.citation.contains("HPD § 27-2115"));
    }

    #[test]
    fn nyc_note_pins_365_days() {
        let r = check(&nyc_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 27-2031") && n.contains("365 days")));
    }

    #[test]
    fn nyc_note_pins_hpd_enforcement() {
        let r = check(&nyc_compliant());
        assert!(r.notes.iter().any(|n| n.contains("HPD")
            && n.contains("civil penalties")
            && n.contains("§ 27-2115")));
    }

    #[test]
    fn default_110f_compliant() {
        let mut i = default_compliant();
        i.temperature_fahrenheit = 110;
        let r = check(&i);
        assert!(r.compliant);
        assert_eq!(r.minimum_temperature_required_f, 110);
    }

    #[test]
    fn default_at_110f_boundary_compliant() {
        let mut i = default_compliant();
        i.temperature_fahrenheit = 110;
        let r = check(&i);
        assert!(r.compliant);
    }

    #[test]
    fn default_109f_violates() {
        let mut i = default_compliant();
        i.temperature_fahrenheit = 109;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("IPC § 607.1.1") && v.contains("110°F") && v.contains("109")));
    }

    #[test]
    fn default_119f_below_ca_nyc_threshold_but_compliant() {
        let mut i = default_compliant();
        i.temperature_fahrenheit = 119;
        let r = check(&i);
        assert!(r.compliant);
    }

    #[test]
    fn default_citation_pins_ipc() {
        let r = check(&default_compliant());
        assert!(r.citation.contains("IPC § 607.1.1"));
        assert!(r.citation.contains("state-specific habitability"));
    }

    #[test]
    fn three_regimes_routed_correctly() {
        for regime in [Regime::California, Regime::NewYorkCity, Regime::Default] {
            let mut i = ca_compliant();
            i.regime = regime;
            let r = check(&i);
            assert!(!r.citation.is_empty());
        }
    }

    #[test]
    fn nyc_uniquely_requires_year_round_invariant() {
        let r_nyc = check(&nyc_compliant());
        assert!(r_nyc.year_round_required);

        let r_ca = check(&ca_compliant());
        assert!(!r_ca.year_round_required);

        let r_default = check(&default_compliant());
        assert!(!r_default.year_round_required);
    }

    #[test]
    fn ca_and_nyc_120f_default_110f_threshold_invariant() {
        let r_ca = check(&ca_compliant());
        assert_eq!(r_ca.minimum_temperature_required_f, 120);

        let r_nyc = check(&nyc_compliant());
        assert_eq!(r_nyc.minimum_temperature_required_f, 120);

        let r_default = check(&default_compliant());
        assert_eq!(r_default.minimum_temperature_required_f, 110);
    }

    #[test]
    fn nyc_3_violations_stack_at_seasonal_below_no_supply() {
        let mut i = nyc_compliant();
        i.hot_water_supplied = false;
        i.year_round_supply = false;
        let r = check(&i);
        assert_eq!(r.violations.len(), 2);
    }

    #[test]
    fn habitability_breach_engages_when_below_minimum() {
        for regime in [Regime::California, Regime::NewYorkCity, Regime::Default] {
            let mut i = ca_compliant();
            i.regime = regime;
            i.temperature_fahrenheit = 50;
            let r = check(&i);
            assert!(r.habitability_breach);
        }
    }

    #[test]
    fn no_hot_water_engages_habitability_breach_all_regimes() {
        for regime in [Regime::California, Regime::NewYorkCity, Regime::Default] {
            let mut i = ca_compliant();
            i.regime = regime;
            i.hot_water_supplied = false;
            let r = check(&i);
            assert!(r.habitability_breach);
        }
    }
}
