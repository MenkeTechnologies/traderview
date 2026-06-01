//! IRC § 25D — Residential Clean Energy Credit.
//!
//! § 25D provides a **30% nonrefundable credit** for qualifying clean
//! energy property installed at the taxpayer's residence. IRA 2022
//! extended and expanded the credit through 2032 with a step-down
//! schedule (30% / 30% / 30% / 26% / 22% / 0%). **OBBBA § 70426**
//! ACCELERATED the termination to expenditures made after
//! **December 31, 2025** — wiping out the 2026-2034 step-down years.
//!
//! **Qualifying property** (§ 25D(d)/(e)):
//! - Solar electric (rooftop PV) (§ 25D(d)(1)).
//! - Solar water heater (§ 25D(d)(2)).
//! - Fuel cell (§ 25D(d)(3); capped at $500/0.5kW capacity).
//! - Small wind energy property (§ 25D(d)(4)).
//! - Geothermal heat pump (§ 25D(d)(5)).
//! - Battery storage technology with capacity ≥ **3 kWh** (§ 25D(d)(6);
//!   added 2023 — stand-alone batteries now qualify).
//! - Biomass fuel property — TERMINATED end of 2022; not modeled here.
//!
//! **Residence requirement** (§ 25D(d)): the dwelling unit must be USED
//! AS A RESIDENCE by the taxpayer. Primary + secondary homes qualify
//! (including vacation homes). **Pure rentals never qualified** — a
//! landlord whose tenant lives in the unit but who does NOT himself
//! live there at any point during the year cannot claim § 25D. OBBBA
//! did not change this.
//!
//! **Nonrefundable + carryforward** (§ 25D(c) + § 26(a)): credit
//! limited to current-year tax liability; unused portion carries forward
//! INDEFINITELY to succeeding tax years and adds to the next year's
//! § 25D credit allowance.
//!
//! Citations: 26 U.S.C. § 25D (general); § 25D(a) (30% credit rate);
//! § 25D(c) (carryforward to succeeding tax year); § 25D(d) (qualifying
//! property + residence requirement); § 25D(d)(6) (battery storage 3 kWh
//! minimum, added 2023); § 25D(g) (sunset as accelerated by OBBBA
//! § 70426); IRS § 25D OBBBA FAQ (December 31, 2025 termination).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QualifyingProperty {
    SolarElectric,
    SolarWaterHeater,
    FuelCell,
    SmallWindEnergy,
    GeothermalHeatPump,
    BatteryStorage,
    /// Biomass fuel — terminated end of 2022. Modeled as never-eligible
    /// for any tax year covered by this module (2023+).
    BiomassFuel,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section25DInput {
    pub expenditure_year: u32,
    pub expenditure_month: u32,
    pub expenditure_day: u32,
    pub qualifying_property_cost_cents: i64,
    pub property_type: QualifyingProperty,
    /// kWh capacity for battery storage. § 25D(d)(6) requires ≥ 3 kWh.
    /// Ignored for non-battery property types.
    pub battery_capacity_kwh: u32,
    /// Whether the taxpayer USES the dwelling as a residence at any
    /// point during the year (§ 25D(d) residence requirement).
    pub taxpayer_resides_in_dwelling: bool,
    /// Current-year tax liability against which the credit can be applied.
    /// Excess carries forward.
    pub current_year_tax_liability_cents: i64,
    /// Unused credit carried forward from prior years per § 25D(c).
    pub prior_year_carryforward_cents: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section25DResult {
    pub credit_eligible: bool,
    pub expenditure_after_obbba_termination: bool,
    pub residence_requirement_met: bool,
    pub gross_credit_cents: i64,
    pub current_year_credit_used_cents: i64,
    pub carryforward_to_next_year_cents: i64,
    pub citation: &'static str,
    pub note: String,
}

pub fn compute(input: &Section25DInput) -> Section25DResult {
    let cost = input.qualifying_property_cost_cents.max(0);
    let tax_liability = input.current_year_tax_liability_cents.max(0);
    let carryforward_in = input.prior_year_carryforward_cents.max(0);

    let after_termination = is_after_2025_12_31(
        input.expenditure_year,
        input.expenditure_month,
        input.expenditure_day,
    );
    if after_termination {
        return Section25DResult {
            credit_eligible: false,
            expenditure_after_obbba_termination: true,
            residence_requirement_met: input.taxpayer_resides_in_dwelling,
            gross_credit_cents: 0,
            current_year_credit_used_cents: 0,
            carryforward_to_next_year_cents: carryforward_in,
            citation:
                "26 U.S.C. § 25D + OBBBA § 70426 — credit TERMINATED for expenditures made after 2025-12-31",
            note: format!(
                "Expenditure on {}-{:02}-{:02} after OBBBA § 70426 termination date 2025-12-31. No § 25D credit on this expenditure. Prior-year carryforward {} cents preserved.",
                input.expenditure_year, input.expenditure_month, input.expenditure_day, carryforward_in
            ),
        };
    }

    if !input.taxpayer_resides_in_dwelling {
        return Section25DResult {
            credit_eligible: false,
            expenditure_after_obbba_termination: false,
            residence_requirement_met: false,
            gross_credit_cents: 0,
            current_year_credit_used_cents: 0,
            carryforward_to_next_year_cents: carryforward_in,
            citation:
                "26 U.S.C. § 25D(d) — dwelling must be USED AS A RESIDENCE by the taxpayer (pure rentals never qualified)",
            note: "Taxpayer does not use the dwelling as a residence. Pure rental properties do not qualify for § 25D regardless of property type.".to_string(),
        };
    }

    if input.property_type == QualifyingProperty::BiomassFuel {
        return Section25DResult {
            credit_eligible: false,
            expenditure_after_obbba_termination: false,
            residence_requirement_met: true,
            gross_credit_cents: 0,
            current_year_credit_used_cents: 0,
            carryforward_to_next_year_cents: carryforward_in,
            citation:
                "26 U.S.C. § 25D(g) — biomass fuel property terminated end of 2022 (consult § 25C for biomass moving forward)",
            note: "Biomass fuel property was removed from § 25D end of 2022 and moved to § 25C (Energy Efficient Home Improvement Credit).".to_string(),
        };
    }

    // Battery-storage ≥ 3 kWh capacity test (§ 25D(d)(6)).
    if input.property_type == QualifyingProperty::BatteryStorage
        && input.battery_capacity_kwh < 3
    {
        return Section25DResult {
            credit_eligible: false,
            expenditure_after_obbba_termination: false,
            residence_requirement_met: true,
            gross_credit_cents: 0,
            current_year_credit_used_cents: 0,
            carryforward_to_next_year_cents: carryforward_in,
            citation:
                "26 U.S.C. § 25D(d)(6) — battery storage technology must have capacity of at least 3 kWh",
            note: format!(
                "Battery capacity {} kWh is below the § 25D(d)(6) 3 kWh minimum. No credit.",
                input.battery_capacity_kwh
            ),
        };
    }

    // 30% credit rate. § 25D(a). Pre-IRA the rate was stepping down
    // (26% / 22%); IRA fixed at 30% for 2022-2032; OBBBA terminates
    // 2025-12-31. Pre-2022 years not modeled here.
    let gross_credit = (cost as i128 * 30 / 100) as i64;
    let total_available = gross_credit + carryforward_in;
    // Nonrefundable: limit to current-year tax liability.
    let credit_used = total_available.min(tax_liability);
    let carryforward_out = total_available - credit_used;

    let note = format!(
        "Gross 30% credit = {} cents × 30% = {} cents. Prior-year carryforward = {} cents. Total available = {} cents. Current-year tax liability = {} cents. Credit used this year = {} cents. § 25D(c) carryforward to next year = {} cents.",
        cost,
        gross_credit,
        carryforward_in,
        total_available,
        tax_liability,
        credit_used,
        carryforward_out,
    );

    Section25DResult {
        credit_eligible: true,
        expenditure_after_obbba_termination: false,
        residence_requirement_met: true,
        gross_credit_cents: gross_credit,
        current_year_credit_used_cents: credit_used,
        carryforward_to_next_year_cents: carryforward_out,
        citation:
            "26 U.S.C. § 25D(a) — 30% residential clean energy credit + § 25D(c) indefinite carryforward (terminates 2025-12-31 per OBBBA § 70426)",
        note,
    }
}

fn is_after_2025_12_31(year: u32, month: u32, day: u32) -> bool {
    match year.cmp(&2025) {
        std::cmp::Ordering::Greater => true,
        std::cmp::Ordering::Less => false,
        std::cmp::Ordering::Equal => match month.cmp(&12) {
            std::cmp::Ordering::Greater => true,
            std::cmp::Ordering::Less => false,
            std::cmp::Ordering::Equal => day > 31,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        year: u32,
        month: u32,
        day: u32,
        cost: i64,
        prop: QualifyingProperty,
        battery_kwh: u32,
        residence: bool,
        tax_liability: i64,
        carryforward: i64,
    ) -> Section25DInput {
        Section25DInput {
            expenditure_year: year,
            expenditure_month: month,
            expenditure_day: day,
            qualifying_property_cost_cents: cost,
            property_type: prop,
            battery_capacity_kwh: battery_kwh,
            taxpayer_resides_in_dwelling: residence,
            current_year_tax_liability_cents: tax_liability,
            prior_year_carryforward_cents: carryforward,
        }
    }

    #[test]
    fn solar_panels_30_percent_credit() {
        // $30K solar → $9K credit, $100K tax liability → fully usable.
        let r = compute(&input(
            2024, 6, 1, 30_000_00, QualifyingProperty::SolarElectric, 0, true, 100_000_00, 0,
        ));
        assert!(r.credit_eligible);
        assert_eq!(r.gross_credit_cents, 9_000_00);
        assert_eq!(r.current_year_credit_used_cents, 9_000_00);
        assert_eq!(r.carryforward_to_next_year_cents, 0);
    }

    #[test]
    fn obbba_termination_after_2025_12_31() {
        let r = compute(&input(
            2026, 1, 1, 30_000_00, QualifyingProperty::SolarElectric, 0, true, 100_000_00, 0,
        ));
        assert!(!r.credit_eligible);
        assert!(r.expenditure_after_obbba_termination);
        assert!(r.citation.contains("OBBBA § 70426"));
        assert!(r.citation.contains("TERMINATED"));
    }

    #[test]
    fn at_2025_12_31_boundary_still_eligible() {
        let r = compute(&input(
            2025, 12, 31, 30_000_00, QualifyingProperty::SolarElectric, 0, true, 100_000_00, 0,
        ));
        assert!(r.credit_eligible);
        assert!(!r.expenditure_after_obbba_termination);
    }

    #[test]
    fn one_day_after_cutoff_terminated() {
        let r = compute(&input(
            2026, 1, 1, 30_000_00, QualifyingProperty::SolarElectric, 0, true, 100_000_00, 0,
        ));
        assert!(!r.credit_eligible);
    }

    #[test]
    fn pure_rental_no_residence_requirement_fail() {
        let r = compute(&input(
            2024, 6, 1, 30_000_00, QualifyingProperty::SolarElectric, 0, false, 100_000_00, 0,
        ));
        assert!(!r.credit_eligible);
        assert!(!r.residence_requirement_met);
        assert!(r.citation.contains("USED AS A RESIDENCE"));
        assert!(r.note.contains("Pure rental"));
    }

    #[test]
    fn biomass_fuel_no_longer_qualifying() {
        let r = compute(&input(
            2024, 6, 1, 5_000_00, QualifyingProperty::BiomassFuel, 0, true, 100_000_00, 0,
        ));
        assert!(!r.credit_eligible);
        assert!(r.citation.contains("biomass"));
        assert!(r.note.contains("§ 25C"));
    }

    #[test]
    fn battery_storage_above_3_kwh_qualifies() {
        let r = compute(&input(
            2024, 6, 1, 10_000_00, QualifyingProperty::BatteryStorage, 13, true, 50_000_00, 0,
        ));
        assert!(r.credit_eligible);
        assert_eq!(r.gross_credit_cents, 3_000_00);
    }

    #[test]
    fn battery_storage_at_3_kwh_boundary_qualifies() {
        let r = compute(&input(
            2024, 6, 1, 10_000_00, QualifyingProperty::BatteryStorage, 3, true, 50_000_00, 0,
        ));
        assert!(r.credit_eligible);
    }

    #[test]
    fn battery_storage_below_3_kwh_fails() {
        let r = compute(&input(
            2024, 6, 1, 10_000_00, QualifyingProperty::BatteryStorage, 2, true, 50_000_00, 0,
        ));
        assert!(!r.credit_eligible);
        assert!(r.citation.contains("§ 25D(d)(6)"));
        assert!(r.citation.contains("3 kWh"));
    }

    #[test]
    fn carryforward_when_credit_exceeds_tax_liability() {
        // $30K solar → $9K credit. Tax liability $5K → $4K carries forward.
        let r = compute(&input(
            2024, 6, 1, 30_000_00, QualifyingProperty::SolarElectric, 0, true, 5_000_00, 0,
        ));
        assert_eq!(r.gross_credit_cents, 9_000_00);
        assert_eq!(r.current_year_credit_used_cents, 5_000_00);
        assert_eq!(r.carryforward_to_next_year_cents, 4_000_00);
    }

    #[test]
    fn prior_year_carryforward_adds_to_current_year() {
        // $20K solar this year = $6K. Prior carryforward $3K. Total avail
        // $9K. Tax liability $10K → all $9K used; no remaining carryforward.
        let r = compute(&input(
            2024, 6, 1, 20_000_00, QualifyingProperty::SolarElectric, 0, true, 10_000_00, 3_000_00,
        ));
        assert_eq!(r.gross_credit_cents, 6_000_00);
        assert_eq!(r.current_year_credit_used_cents, 9_000_00);
        assert_eq!(r.carryforward_to_next_year_cents, 0);
    }

    #[test]
    fn carryforward_preserved_through_termination() {
        // 2026 post-termination expenditure: no new credit but prior
        // carryforward is preserved (use against current tax).
        let r = compute(&input(
            2026, 1, 1, 30_000_00, QualifyingProperty::SolarElectric, 0, true, 100_000_00, 5_000_00,
        ));
        assert!(!r.credit_eligible);
        // The compute function for termination returns the unused
        // carryforward as the next-year value — caller can claim it
        // against this year's tax via a separate carryforward line on
        // Form 5695 even though this expenditure produces no new credit.
        assert_eq!(r.carryforward_to_next_year_cents, 5_000_00);
    }

    #[test]
    fn solar_water_heater_qualifies() {
        let r = compute(&input(
            2024, 6, 1, 5_000_00, QualifyingProperty::SolarWaterHeater, 0, true, 50_000_00, 0,
        ));
        assert!(r.credit_eligible);
        assert_eq!(r.gross_credit_cents, 1_500_00);
    }

    #[test]
    fn geothermal_heat_pump_qualifies() {
        let r = compute(&input(
            2024, 6, 1, 20_000_00, QualifyingProperty::GeothermalHeatPump, 0, true, 100_000_00, 0,
        ));
        assert!(r.credit_eligible);
        assert_eq!(r.gross_credit_cents, 6_000_00);
    }

    #[test]
    fn small_wind_energy_qualifies() {
        let r = compute(&input(
            2024, 6, 1, 15_000_00, QualifyingProperty::SmallWindEnergy, 0, true, 100_000_00, 0,
        ));
        assert!(r.credit_eligible);
        assert_eq!(r.gross_credit_cents, 4_500_00);
    }

    #[test]
    fn fuel_cell_qualifies() {
        let r = compute(&input(
            2024, 6, 1, 10_000_00, QualifyingProperty::FuelCell, 0, true, 100_000_00, 0,
        ));
        assert!(r.credit_eligible);
        assert_eq!(r.gross_credit_cents, 3_000_00);
    }

    #[test]
    fn zero_cost_no_credit() {
        let r = compute(&input(
            2024, 6, 1, 0, QualifyingProperty::SolarElectric, 0, true, 100_000_00, 0,
        ));
        assert!(r.credit_eligible);
        assert_eq!(r.gross_credit_cents, 0);
    }

    #[test]
    fn negative_inputs_clamped() {
        let r = compute(&input(
            2024, 6, 1, -1000, QualifyingProperty::SolarElectric, 0, true, -1, -1,
        ));
        assert_eq!(r.gross_credit_cents, 0);
        assert_eq!(r.current_year_credit_used_cents, 0);
    }

    #[test]
    fn termination_check_precedence_over_residence() {
        // Post-termination + no-residence: termination check fires first.
        let r = compute(&input(
            2026, 1, 1, 30_000_00, QualifyingProperty::SolarElectric, 0, false, 100_000_00, 0,
        ));
        assert!(r.expenditure_after_obbba_termination);
        assert!(r.citation.contains("TERMINATED"));
    }

    #[test]
    fn citations_pin_correct_authorities() {
        let solar = compute(&input(
            2024, 6, 1, 30_000_00, QualifyingProperty::SolarElectric, 0, true, 100_000_00, 0,
        ));
        assert!(solar.citation.contains("§ 25D(a)"));
        assert!(solar.citation.contains("§ 25D(c)"));
        assert!(solar.citation.contains("OBBBA § 70426"));

        let post = compute(&input(
            2026, 1, 1, 30_000_00, QualifyingProperty::SolarElectric, 0, true, 100_000_00, 0,
        ));
        assert!(post.citation.contains("OBBBA § 70426"));

        let no_residence = compute(&input(
            2024, 6, 1, 30_000_00, QualifyingProperty::SolarElectric, 0, false, 100_000_00, 0,
        ));
        assert!(no_residence.citation.contains("§ 25D(d)"));

        let small_battery = compute(&input(
            2024, 6, 1, 5_000_00, QualifyingProperty::BatteryStorage, 1, true, 50_000_00, 0,
        ));
        assert!(small_battery.citation.contains("§ 25D(d)(6)"));
    }

    #[test]
    fn date_boundary_dec_30_31_jan_1() {
        let d30 = compute(&input(
            2025, 12, 30, 30_000_00, QualifyingProperty::SolarElectric, 0, true, 100_000_00, 0,
        ));
        let d31 = compute(&input(
            2025, 12, 31, 30_000_00, QualifyingProperty::SolarElectric, 0, true, 100_000_00, 0,
        ));
        let jan1 = compute(&input(
            2026, 1, 1, 30_000_00, QualifyingProperty::SolarElectric, 0, true, 100_000_00, 0,
        ));
        assert!(d30.credit_eligible);
        assert!(d31.credit_eligible);
        assert!(!jan1.credit_eligible);
    }

    #[test]
    fn large_solar_300k_full_credit_breakdown() {
        // $300K commercial-scale residential solar: $90K gross credit.
        // High-income trader with $200K tax liability → $90K fully used.
        let r = compute(&input(
            2024, 6, 1, 300_000_00, QualifyingProperty::SolarElectric, 0, true, 200_000_00, 0,
        ));
        assert_eq!(r.gross_credit_cents, 90_000_00);
        assert_eq!(r.current_year_credit_used_cents, 90_000_00);
        assert_eq!(r.carryforward_to_next_year_cents, 0);
    }

    #[test]
    fn ultra_large_credit_with_low_tax_creates_long_carryforward() {
        // $1M solar = $300K credit. $20K tax liability → $280K carryforward.
        let r = compute(&input(
            2024, 6, 1, 1_000_000_00, QualifyingProperty::SolarElectric, 0, true, 20_000_00, 0,
        ));
        assert_eq!(r.gross_credit_cents, 300_000_00);
        assert_eq!(r.current_year_credit_used_cents, 20_000_00);
        assert_eq!(r.carryforward_to_next_year_cents, 280_000_00);
    }
}
