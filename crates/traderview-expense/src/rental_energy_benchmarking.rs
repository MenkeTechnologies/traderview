//! Rental property energy benchmarking + GHG emissions
//! disclosure compliance — when must a trader-landlord owning
//! a large multifamily building annually report energy/water
//! consumption AND comply with carbon emissions caps?
//! Trader-landlord critical for any large multifamily owner
//! in NYC + Boston + other jurisdictions with building-
//! emissions ordinances: failure to comply triggers
//! significant ongoing penalties (NYC $268/ton CO2e
//! exceedance + $2,000/year benchmarking; Boston BERDO 2.0
//! emissions penalties).
//!
//! Distinct from siblings `rental_property_registration`
//! (general landlord registration), `rental_gas_appliance_
//! ban` (new construction electrification mandate), and
//! `landlord_annual_rent_statement` (rent disclosure).
//!
//! **Three regimes**:
//!
//! **New York City — Local Law 84 of 2009 (Energy
//! Benchmarking) + Local Law 97 of 2019 (Carbon Emissions
//! Caps, Climate Mobilization Act)**:
//! - LL84: buildings > 25,000 sq ft OR groups > 100,000 sq
//!   ft on single lot must annually report energy + water
//!   consumption via ENERGY STAR Portfolio Manager.
//! - LL84 reporting deadline: **May 1st each year**.
//! - LL84 penalties: **$500 missed deadline + $500/quarter
//!   additional + up to $2,000/year max**.
//! - LL97: covered buildings must report annual GHG
//!   emissions to NYC DOB; emissions caps phased in 2024
//!   onward.
//! - LL97 penalty: **$268/metric ton CO2e** emitted over
//!   individual building cap in any given year.
//!
//! **Boston BERDO 2.0 (Building Emissions Reduction and
//! Disclosure Ordinance; eff. 2021, emissions standards
//! phased in 2025-2050)**:
//! - Annual energy + water consumption reporting required.
//! - GHG emissions standards by building class with
//!   declining limits through net-zero by 2050.
//! - Penalties: $300/metric ton CO2e above limits (parallel
//!   to LL97 but slightly higher per-ton penalty).
//!
//! **Default — other jurisdictions with local benchmarking
//! ordinances** (CA AB 802 + Seattle BAEDO + DC GBES +
//! Chicago Energy Benchmarking + Minneapolis):
//! - Most major U.S. cities have adopted similar
//!   benchmarking programs with size thresholds 25,000-
//!   50,000 sq ft.
//! - State-specific (CA AB 802 = 50,000 sq ft threshold).
//! - No federal mandate; verify local adoption.
//!
//! Citations: NYC Local Law 84 of 2009 (Energy Benchmarking);
//! NYC Local Law 97 of 2019 (Climate Mobilization Act);
//! Boston BERDO 2.0 (Building Emissions Reduction and
//! Disclosure Ordinance); CA AB 802 (Statewide Energy Use
//! Disclosure); Seattle BAEDO; DC GBES.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    NewYorkCity,
    BostonBerdo,
    Default,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RentalEnergyBenchmarkingInput {
    pub regime: Regime,
    /// Building gross floor area in square feet.
    pub building_sq_ft: u32,
    /// Whether building is part of a group of buildings on a
    /// single lot (for NYC > 100,000 sq ft aggregate threshold).
    pub group_buildings_aggregate_sq_ft: u32,
    /// Whether annual energy + water benchmarking report was
    /// submitted by deadline (May 1 for NYC).
    pub benchmarking_report_submitted_on_time: bool,
    /// Number of full quarters past the deadline if late.
    pub quarters_past_deadline: u32,
    /// Annual GHG emissions in metric tons CO2e (for NYC LL97
    /// + Boston BERDO).
    pub ghg_emissions_metric_tons_co2e: u32,
    /// Building's individual emissions cap in metric tons
    /// CO2e (for NYC LL97 + Boston BERDO).
    pub building_emissions_cap_metric_tons_co2e: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RentalEnergyBenchmarkingResult {
    pub compliant: bool,
    pub building_in_scope: bool,
    pub benchmarking_penalty_cents: i64,
    pub emissions_penalty_cents: i64,
    pub total_penalty_cents: i64,
    pub emissions_cap_exceeded_metric_tons: u32,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &RentalEnergyBenchmarkingInput) -> RentalEnergyBenchmarkingResult {
    match input.regime {
        Regime::NewYorkCity => check_nyc(input),
        Regime::BostonBerdo => check_boston(input),
        Regime::Default => check_default(input),
    }
}

fn check_nyc(input: &RentalEnergyBenchmarkingInput) -> RentalEnergyBenchmarkingResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "NYC Local Law 84 of 2009 — annual energy + water benchmarking required for buildings > 25,000 sq ft OR groups > 100,000 sq ft aggregate; ENERGY STAR Portfolio Manager submission deadline May 1st"
            .to_string(),
        "NYC LL84 penalties — $500 missed deadline + $500/quarter additional + up to $2,000/year max for ongoing noncompliance"
            .to_string(),
        "NYC Local Law 97 of 2019 (Climate Mobilization Act) — covered buildings must report annual GHG emissions to NYC DOB; emissions caps phased in 2024 onward; penalty $268/metric ton CO2e exceedance"
            .to_string(),
    ];

    let in_scope = input.building_sq_ft > 25_000 || input.group_buildings_aggregate_sq_ft > 100_000;

    let mut benchmarking_penalty: i64 = 0;
    if in_scope && !input.benchmarking_report_submitted_on_time {
        let base_penalty: i64 = 50_000;
        let quarterly_penalty: i64 = (input.quarters_past_deadline as i64).saturating_mul(50_000);
        let total = base_penalty.saturating_add(quarterly_penalty);
        let annual_cap: i64 = 200_000;
        benchmarking_penalty = total.min(annual_cap);
        violations.push(format!(
            "NYC LL84 — benchmarking report not submitted by May 1st; ${} penalty engaged ({} full quarters past deadline)",
            benchmarking_penalty / 100,
            input.quarters_past_deadline
        ));
    }

    let emissions_exceedance = input
        .ghg_emissions_metric_tons_co2e
        .saturating_sub(input.building_emissions_cap_metric_tons_co2e);

    let emissions_penalty: i64 = if in_scope && emissions_exceedance > 0 {
        let per_ton_cents: i64 = 26_800;
        (emissions_exceedance as i64).saturating_mul(per_ton_cents)
    } else {
        0
    };

    if in_scope && emissions_exceedance > 0 {
        violations.push(format!(
            "NYC LL97 — building exceeded emissions cap by {} metric tons CO2e; $268/ton penalty = ${}",
            emissions_exceedance,
            emissions_penalty / 100
        ));
    }

    let total = benchmarking_penalty.saturating_add(emissions_penalty);

    RentalEnergyBenchmarkingResult {
        compliant: violations.is_empty(),
        building_in_scope: in_scope,
        benchmarking_penalty_cents: benchmarking_penalty,
        emissions_penalty_cents: emissions_penalty,
        total_penalty_cents: total,
        emissions_cap_exceeded_metric_tons: emissions_exceedance,
        violations,
        citation: "NYC Local Law 84 of 2009; NYC Local Law 97 of 2019 (Climate Mobilization Act)",
        notes,
    }
}

fn check_boston(input: &RentalEnergyBenchmarkingInput) -> RentalEnergyBenchmarkingResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "Boston BERDO 2.0 (Building Emissions Reduction and Disclosure Ordinance, eff. 2021) — annual energy + water consumption reporting required for covered buildings"
            .to_string(),
        "Boston BERDO 2.0 — GHG emissions standards by building class with declining limits through net-zero by 2050; penalties $300/metric ton CO2e above limits"
            .to_string(),
    ];

    let in_scope = input.building_sq_ft >= 35_000;

    if in_scope && !input.benchmarking_report_submitted_on_time {
        violations.push(
            "Boston BERDO 2.0 — annual benchmarking report not submitted; reporting deadline missed".to_string(),
        );
    }

    let emissions_exceedance = input
        .ghg_emissions_metric_tons_co2e
        .saturating_sub(input.building_emissions_cap_metric_tons_co2e);

    let emissions_penalty: i64 = if in_scope && emissions_exceedance > 0 {
        let per_ton_cents: i64 = 30_000;
        (emissions_exceedance as i64).saturating_mul(per_ton_cents)
    } else {
        0
    };

    if in_scope && emissions_exceedance > 0 {
        violations.push(format!(
            "Boston BERDO 2.0 — building exceeded emissions limits by {} metric tons CO2e; $300/ton penalty = ${}",
            emissions_exceedance,
            emissions_penalty / 100
        ));
    }

    RentalEnergyBenchmarkingResult {
        compliant: violations.is_empty(),
        building_in_scope: in_scope,
        benchmarking_penalty_cents: 0,
        emissions_penalty_cents: emissions_penalty,
        total_penalty_cents: emissions_penalty,
        emissions_cap_exceeded_metric_tons: emissions_exceedance,
        violations,
        citation: "Boston BERDO 2.0 (Building Emissions Reduction and Disclosure Ordinance)",
        notes,
    }
}

fn check_default(_input: &RentalEnergyBenchmarkingInput) -> RentalEnergyBenchmarkingResult {
    let notes: Vec<String> = vec![
        "default rule — no federal benchmarking mandate; verify local jurisdiction adoption (CA AB 802 = 50,000 sq ft threshold; Seattle BAEDO; DC GBES; Chicago Energy Benchmarking; Minneapolis)"
            .to_string(),
        "default rule — most major U.S. cities have adopted benchmarking programs with size thresholds 25,000-50,000 sq ft; emissions caps less common than NYC + Boston"
            .to_string(),
    ];

    RentalEnergyBenchmarkingResult {
        compliant: true,
        building_in_scope: false,
        benchmarking_penalty_cents: 0,
        emissions_penalty_cents: 0,
        total_penalty_cents: 0,
        emissions_cap_exceeded_metric_tons: 0,
        violations: Vec::new(),
        citation: "no federal mandate; verify local jurisdiction (CA AB 802; Seattle BAEDO; DC GBES; Chicago)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nyc_compliant_base() -> RentalEnergyBenchmarkingInput {
        RentalEnergyBenchmarkingInput {
            regime: Regime::NewYorkCity,
            building_sq_ft: 50_000,
            group_buildings_aggregate_sq_ft: 0,
            benchmarking_report_submitted_on_time: true,
            quarters_past_deadline: 0,
            ghg_emissions_metric_tons_co2e: 100,
            building_emissions_cap_metric_tons_co2e: 200,
        }
    }

    fn boston_compliant_base() -> RentalEnergyBenchmarkingInput {
        let mut i = nyc_compliant_base();
        i.regime = Regime::BostonBerdo;
        i
    }

    fn default_base() -> RentalEnergyBenchmarkingInput {
        let mut i = nyc_compliant_base();
        i.regime = Regime::Default;
        i
    }

    #[test]
    fn nyc_compliant_passes() {
        let r = check(&nyc_compliant_base());
        assert!(r.compliant);
        assert!(r.building_in_scope);
        assert_eq!(r.total_penalty_cents, 0);
    }

    #[test]
    fn nyc_under_25000_sqft_not_in_scope() {
        let mut i = nyc_compliant_base();
        i.building_sq_ft = 25_000;
        i.benchmarking_report_submitted_on_time = false;
        let r = check(&i);
        assert!(!r.building_in_scope);
        assert!(r.compliant);
    }

    #[test]
    fn nyc_at_25001_sqft_in_scope() {
        let mut i = nyc_compliant_base();
        i.building_sq_ft = 25_001;
        let r = check(&i);
        assert!(r.building_in_scope);
    }

    #[test]
    fn nyc_group_over_100000_sqft_in_scope() {
        let mut i = nyc_compliant_base();
        i.building_sq_ft = 10_000;
        i.group_buildings_aggregate_sq_ft = 100_001;
        let r = check(&i);
        assert!(r.building_in_scope);
    }

    #[test]
    fn nyc_ll84_missed_deadline_500_dollar_penalty() {
        let mut i = nyc_compliant_base();
        i.benchmarking_report_submitted_on_time = false;
        i.quarters_past_deadline = 0;
        let r = check(&i);
        assert_eq!(r.benchmarking_penalty_cents, 50_000);
        assert!(!r.compliant);
    }

    #[test]
    fn nyc_ll84_one_quarter_past_1000_dollar_penalty() {
        let mut i = nyc_compliant_base();
        i.benchmarking_report_submitted_on_time = false;
        i.quarters_past_deadline = 1;
        let r = check(&i);
        assert_eq!(r.benchmarking_penalty_cents, 100_000);
    }

    #[test]
    fn nyc_ll84_three_quarters_past_2000_dollar_penalty() {
        let mut i = nyc_compliant_base();
        i.benchmarking_report_submitted_on_time = false;
        i.quarters_past_deadline = 3;
        let r = check(&i);
        assert_eq!(r.benchmarking_penalty_cents, 200_000);
    }

    #[test]
    fn nyc_ll84_capped_at_2000_dollar_annual_max() {
        let mut i = nyc_compliant_base();
        i.benchmarking_report_submitted_on_time = false;
        i.quarters_past_deadline = 100;
        let r = check(&i);
        assert_eq!(r.benchmarking_penalty_cents, 200_000);
    }

    #[test]
    fn nyc_ll97_emissions_under_cap_no_penalty() {
        let mut i = nyc_compliant_base();
        i.ghg_emissions_metric_tons_co2e = 150;
        i.building_emissions_cap_metric_tons_co2e = 200;
        let r = check(&i);
        assert_eq!(r.emissions_penalty_cents, 0);
        assert_eq!(r.emissions_cap_exceeded_metric_tons, 0);
    }

    #[test]
    fn nyc_ll97_emissions_at_cap_boundary_no_penalty() {
        let mut i = nyc_compliant_base();
        i.ghg_emissions_metric_tons_co2e = 200;
        i.building_emissions_cap_metric_tons_co2e = 200;
        let r = check(&i);
        assert_eq!(r.emissions_penalty_cents, 0);
        assert_eq!(r.emissions_cap_exceeded_metric_tons, 0);
    }

    #[test]
    fn nyc_ll97_emissions_1_ton_over_268_dollar_penalty() {
        let mut i = nyc_compliant_base();
        i.ghg_emissions_metric_tons_co2e = 201;
        i.building_emissions_cap_metric_tons_co2e = 200;
        let r = check(&i);
        assert_eq!(r.emissions_penalty_cents, 26_800);
        assert_eq!(r.emissions_cap_exceeded_metric_tons, 1);
    }

    #[test]
    fn nyc_ll97_emissions_100_tons_over_26800_dollar_penalty() {
        let mut i = nyc_compliant_base();
        i.ghg_emissions_metric_tons_co2e = 300;
        i.building_emissions_cap_metric_tons_co2e = 200;
        let r = check(&i);
        assert_eq!(r.emissions_penalty_cents, 2_680_000);
        assert_eq!(r.emissions_cap_exceeded_metric_tons, 100);
    }

    #[test]
    fn nyc_combined_ll84_and_ll97_penalties_stack() {
        let mut i = nyc_compliant_base();
        i.benchmarking_report_submitted_on_time = false;
        i.quarters_past_deadline = 1;
        i.ghg_emissions_metric_tons_co2e = 300;
        i.building_emissions_cap_metric_tons_co2e = 200;
        let r = check(&i);
        assert_eq!(r.total_penalty_cents, 100_000 + 2_680_000);
        assert_eq!(r.violations.len(), 2);
    }

    #[test]
    fn boston_compliant_passes() {
        let r = check(&boston_compliant_base());
        assert!(r.compliant);
        assert!(r.building_in_scope);
    }

    #[test]
    fn boston_under_35000_sqft_not_in_scope() {
        let mut i = boston_compliant_base();
        i.building_sq_ft = 34_999;
        i.benchmarking_report_submitted_on_time = false;
        i.ghg_emissions_metric_tons_co2e = 1000;
        i.building_emissions_cap_metric_tons_co2e = 0;
        let r = check(&i);
        assert!(!r.building_in_scope);
        assert!(r.compliant);
    }

    #[test]
    fn boston_at_35000_sqft_in_scope() {
        let mut i = boston_compliant_base();
        i.building_sq_ft = 35_000;
        let r = check(&i);
        assert!(r.building_in_scope);
    }

    #[test]
    fn boston_emissions_over_cap_300_dollar_per_ton() {
        let mut i = boston_compliant_base();
        i.ghg_emissions_metric_tons_co2e = 300;
        i.building_emissions_cap_metric_tons_co2e = 200;
        let r = check(&i);
        assert_eq!(r.emissions_penalty_cents, 100 * 30_000);
        assert_eq!(r.emissions_cap_exceeded_metric_tons, 100);
    }

    #[test]
    fn boston_missed_deadline_violates() {
        let mut i = boston_compliant_base();
        i.benchmarking_report_submitted_on_time = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("BERDO 2.0") && v.contains("reporting deadline")));
    }

    #[test]
    fn default_no_violations_regardless() {
        let mut i = default_base();
        i.benchmarking_report_submitted_on_time = false;
        i.ghg_emissions_metric_tons_co2e = 10_000;
        i.building_emissions_cap_metric_tons_co2e = 0;
        let r = check(&i);
        assert!(r.compliant);
        assert!(!r.building_in_scope);
    }

    #[test]
    fn nyc_citation_pins_ll84_ll97() {
        let r = check(&nyc_compliant_base());
        assert!(r.citation.contains("Local Law 84 of 2009"));
        assert!(r.citation.contains("Local Law 97 of 2019"));
        assert!(r.citation.contains("Climate Mobilization Act"));
    }

    #[test]
    fn boston_citation_pins_berdo() {
        let r = check(&boston_compliant_base());
        assert!(r.citation.contains("BERDO 2.0"));
    }

    #[test]
    fn default_citation_pins_ca_ab_802() {
        let r = check(&default_base());
        assert!(r.citation.contains("CA AB 802"));
        assert!(r.citation.contains("Seattle BAEDO"));
    }

    #[test]
    fn nyc_note_pins_268_per_ton_penalty() {
        let r = check(&nyc_compliant_base());
        assert!(r.notes.iter().any(|n| n.contains("$268/metric ton CO2e")));
    }

    #[test]
    fn nyc_note_pins_may_1st_deadline() {
        let r = check(&nyc_compliant_base());
        assert!(r.notes.iter().any(|n| n.contains("May 1st")));
    }

    #[test]
    fn boston_note_pins_300_per_ton_penalty() {
        let r = check(&boston_compliant_base());
        assert!(r.notes.iter().any(|n| n.contains("$300/metric ton")));
    }

    #[test]
    fn three_regimes_routed_correctly() {
        for regime in [Regime::NewYorkCity, Regime::BostonBerdo, Regime::Default] {
            let mut i = nyc_compliant_base();
            i.regime = regime;
            let r = check(&i);
            assert!(!r.citation.is_empty());
        }
    }

    #[test]
    fn nyc_unique_25000_sqft_threshold_vs_boston_35000_invariant() {
        let mut i_nyc = nyc_compliant_base();
        i_nyc.building_sq_ft = 30_000;
        let r_nyc = check(&i_nyc);
        assert!(r_nyc.building_in_scope);

        let mut i_boston = boston_compliant_base();
        i_boston.building_sq_ft = 30_000;
        let r_boston = check(&i_boston);
        assert!(!r_boston.building_in_scope);
    }

    #[test]
    fn boston_uniquely_higher_per_ton_penalty_invariant() {
        let mut i_nyc = nyc_compliant_base();
        i_nyc.ghg_emissions_metric_tons_co2e = 200;
        i_nyc.building_emissions_cap_metric_tons_co2e = 100;
        let r_nyc = check(&i_nyc);
        assert_eq!(r_nyc.emissions_penalty_cents, 100 * 26_800);

        let mut i_boston = boston_compliant_base();
        i_boston.ghg_emissions_metric_tons_co2e = 200;
        i_boston.building_emissions_cap_metric_tons_co2e = 100;
        let r_boston = check(&i_boston);
        assert_eq!(r_boston.emissions_penalty_cents, 100 * 30_000);
        assert!(r_boston.emissions_penalty_cents > r_nyc.emissions_penalty_cents);
    }

    #[test]
    fn defensive_emissions_under_cap_zero_penalty() {
        let mut i = nyc_compliant_base();
        i.ghg_emissions_metric_tons_co2e = 0;
        i.building_emissions_cap_metric_tons_co2e = 1000;
        let r = check(&i);
        assert_eq!(r.emissions_penalty_cents, 0);
    }

    #[test]
    fn defensive_emissions_overflow_saturating() {
        let mut i = nyc_compliant_base();
        i.ghg_emissions_metric_tons_co2e = u32::MAX;
        i.building_emissions_cap_metric_tons_co2e = 0;
        let r = check(&i);
        assert!(r.emissions_penalty_cents > 0);
    }
}
