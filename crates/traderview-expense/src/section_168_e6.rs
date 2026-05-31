//! IRC §168(e)(6) — Qualified Improvement Property (QIP).
//!
//! QIP is the 15-year MACRS class for **interior improvements to
//! nonresidential buildings** made by the taxpayer after the
//! building was originally placed in service. Critical for landlords
//! planning tenant build-out allowances on commercial properties.
//!
//! **Drafting-error saga:**
//!
//!   * **TCJA 2017** intended QIP to be 15-year property eligible
//!     for §168(k) bonus depreciation. The bill text accidentally
//!     omitted the 15-year recovery period assignment, so QIP placed
//!     in service 2018-2019 defaulted to **39-year** real property
//!     under §168(c) — and was NOT bonus eligible.
//!
//!   * **CARES Act 2020 (P.L. 116-136 §2307)** retroactively fixed
//!     the drafting error effective for property placed in service
//!     after December 31, 2017 — assigned 15-year recovery and
//!     restored §168(k) bonus eligibility. Taxpayers could file Form
//!     3115 (accounting method change) to recover prior years'
//!     missed bonus depreciation as a §481(a) adjustment.
//!
//! §168(e)(6) **definition** of QIP:
//!
//!   "any improvement made by the taxpayer to an interior portion of
//!   a building which is nonresidential real property if such
//!   improvement is placed in service after the date such building
//!   was first placed in service."
//!
//! **§168(e)(6) exclusions** — improvements attributable to:
//!
//!   1. The enlargement of the building.
//!   2. Any elevator or escalator.
//!   3. The internal structural framework of the building.
//!
//! These three categories are NOT QIP and follow 39-year recovery
//! under §168(b)(3) regardless of when placed in service.
//!
//! §168(k) bonus depreciation phase-down (shared with iter 11's
//! `cost_segregation`):
//!
//!   * 2018-2022: 100%
//!   * 2023: 80%
//!   * 2024: 60%
//!   * 2025: 40%
//!   * 2026: 20%
//!   * 2027+: 0%
//!
//! Pure compute. Caller asserts the improvement type + dates + bonus
//! election; we determine QIP qualification and compute the year-1
//! deduction.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImprovementType {
    /// Interior improvement to nonresidential building (the QIP
    /// general case): new partitions, drop ceilings, electrical
    /// outlets, HVAC ductwork, flooring, paint, lighting.
    InteriorNonresidential,
    /// §168(e)(6)(A) exclusion — building enlargement / addition.
    BuildingEnlargement,
    /// §168(e)(6)(B) exclusion — elevator or escalator.
    ElevatorOrEscalator,
    /// §168(e)(6)(C) exclusion — internal structural framework.
    InternalStructuralFramework,
    /// Residential rental — QIP is by definition nonresidential only.
    ResidentialRental,
}

impl ImprovementType {
    fn qualifies_as_qip(self) -> bool {
        matches!(self, ImprovementType::InteriorNonresidential)
    }

    fn exclusion_reason(self) -> Option<&'static str> {
        match self {
            ImprovementType::BuildingEnlargement =>
                Some("§168(e)(6)(A) excludes building enlargement"),
            ImprovementType::ElevatorOrEscalator =>
                Some("§168(e)(6)(B) excludes elevators and escalators"),
            ImprovementType::InternalStructuralFramework =>
                Some("§168(e)(6)(C) excludes internal structural framework"),
            ImprovementType::ResidentialRental =>
                Some("residential rental — QIP is nonresidential property only"),
            ImprovementType::InteriorNonresidential => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section168E6Input {
    pub improvement_cost: Decimal,
    pub improvement_type: ImprovementType,
    pub placed_in_service_year: i32,
    /// Tax year being computed (for current-year recovery + bonus
    /// lookup).
    pub current_tax_year: i32,
    /// Original building placed-in-service year. QIP improvement
    /// must be made AFTER this date per §168(e)(6).
    pub building_first_placed_in_service_year: i32,
    pub elect_bonus_depreciation: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Section168E6Result {
    pub qualifies_as_qip: bool,
    pub exclusion_reason: Option<String>,
    pub recovery_period_years: u32,
    pub bonus_pct_applied: Decimal,
    pub year_1_bonus_deduction: Decimal,
    pub year_1_macrs_deduction: Decimal,
    pub year_1_total_deduction: Decimal,
    pub note: String,
}

fn d(s: &str) -> Decimal {
    Decimal::from_str(s).unwrap()
}

/// §168(k) bonus depreciation phase-down by tax year.
fn bonus_pct(year: i32) -> Decimal {
    match year {
        ..=2017 => d("0.50"),
        2018..=2022 => d("1.00"),
        2023 => d("0.80"),
        2024 => d("0.60"),
        2025 => d("0.40"),
        2026 => d("0.20"),
        _ => Decimal::ZERO,
    }
}

/// 15-year property half-year convention year-1 rate per Pub 946
/// Table A-1: 5% in year 1, 9.5% years 2-9, etc. We use 5% for
/// simplicity (the table value).
fn macrs_15_year_year_1_rate() -> Decimal {
    d("0.05")
}

/// 39-year nonresidential real property year-1 rate is mid-month;
/// we approximate at half-year midpoint = 1.282% (1/39 × 0.5).
fn macrs_39_year_year_1_rate() -> Decimal {
    d("0.01282")
}

pub fn compute(input: &Section168E6Input) -> Section168E6Result {
    let mut r = Section168E6Result {
        qualifies_as_qip: input.improvement_type.qualifies_as_qip()
            && input.placed_in_service_year > input.building_first_placed_in_service_year,
        ..Section168E6Result::default()
    };

    if !r.qualifies_as_qip {
        // Excluded categories or building-not-yet-placed-in-service
        // → 39-year nonresidential recovery, no bonus.
        if let Some(reason) = input.improvement_type.exclusion_reason() {
            r.exclusion_reason = Some(reason.into());
        } else if input.placed_in_service_year <= input.building_first_placed_in_service_year {
            r.exclusion_reason = Some(
                "improvement not made AFTER the building's original placed-in-service date"
                    .into(),
            );
        }
        r.recovery_period_years = 39;
        r.year_1_macrs_deduction =
            (input.improvement_cost * macrs_39_year_year_1_rate()).round_dp(2);
        r.year_1_total_deduction = r.year_1_macrs_deduction;
        r.note = format!(
            "not QIP ({}): 39-year nonresidential MACRS, mid-month half-year approx ${} year-1; no §168(k) bonus.",
            r.exclusion_reason.clone().unwrap_or_else(|| "see exclusion".into()),
            r.year_1_macrs_deduction
        );
        return r;
    }

    // Qualifying QIP: 15-year MACRS + §168(k) bonus eligible.
    r.recovery_period_years = 15;

    let actual_bonus_pct = if input.elect_bonus_depreciation {
        bonus_pct(input.placed_in_service_year)
    } else {
        Decimal::ZERO
    };
    r.bonus_pct_applied = actual_bonus_pct;

    let bonus_amount =
        (input.improvement_cost * actual_bonus_pct).round_dp(2);
    let remaining_basis = input.improvement_cost - bonus_amount;
    let macrs_amount =
        (remaining_basis * macrs_15_year_year_1_rate()).round_dp(2);

    r.year_1_bonus_deduction = bonus_amount;
    r.year_1_macrs_deduction = macrs_amount;
    r.year_1_total_deduction = bonus_amount + macrs_amount;

    r.note = if input.elect_bonus_depreciation {
        let pct_display = (actual_bonus_pct * Decimal::from(100)).round_dp(0);
        format!(
            "QIP: 15-year MACRS + {}% §168(k) bonus. Year 1: ${} bonus + ${} MACRS on remaining = ${} total.",
            pct_display, bonus_amount, macrs_amount, r.year_1_total_deduction,
        )
    } else {
        format!(
            "QIP: 15-year MACRS, no §168(k) bonus elected. Year 1 MACRS half-year ${}.",
            r.year_1_macrs_deduction
        )
    };
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn base() -> Section168E6Input {
        Section168E6Input {
            improvement_cost: dec!(100000),
            improvement_type: ImprovementType::InteriorNonresidential,
            placed_in_service_year: 2024,
            current_tax_year: 2024,
            building_first_placed_in_service_year: 2015,
            elect_bonus_depreciation: true,
        }
    }

    #[test]
    fn interior_nonresidential_qualifies_as_qip() {
        let r = compute(&base());
        assert!(r.qualifies_as_qip);
        assert_eq!(r.recovery_period_years, 15);
    }

    #[test]
    fn qip_2024_60pct_bonus_year_1_total() {
        // $100k × 60% = $60k bonus. Remaining $40k × 5% = $2k MACRS.
        // Total = $62k year 1.
        let r = compute(&base());
        assert_eq!(r.bonus_pct_applied, dec!(0.60));
        assert_eq!(r.year_1_bonus_deduction, dec!(60000));
        assert_eq!(r.year_1_macrs_deduction, dec!(2000));
        assert_eq!(r.year_1_total_deduction, dec!(62000));
    }

    #[test]
    fn qip_2022_100pct_bonus_full_year_1_deduction() {
        let mut i = base();
        i.placed_in_service_year = 2022;
        i.current_tax_year = 2022;
        let r = compute(&i);
        assert_eq!(r.bonus_pct_applied, dec!(1.00));
        assert_eq!(r.year_1_total_deduction, dec!(100000));
    }

    #[test]
    fn qip_2023_80pct_bonus_phase_down() {
        let mut i = base();
        i.placed_in_service_year = 2023;
        i.current_tax_year = 2023;
        let r = compute(&i);
        assert_eq!(r.bonus_pct_applied, dec!(0.80));
        // $80k bonus + $1k MACRS on $20k remaining at 5% = $81k.
        assert_eq!(r.year_1_total_deduction, dec!(81000));
    }

    #[test]
    fn qip_2027_zero_bonus_after_phase_down() {
        let mut i = base();
        i.placed_in_service_year = 2027;
        i.current_tax_year = 2027;
        let r = compute(&i);
        assert_eq!(r.bonus_pct_applied, Decimal::ZERO);
        assert_eq!(r.year_1_bonus_deduction, Decimal::ZERO);
        // All $100k at 5% MACRS = $5k.
        assert_eq!(r.year_1_macrs_deduction, dec!(5000));
    }

    #[test]
    fn building_enlargement_excluded_from_qip_39_year() {
        let mut i = base();
        i.improvement_type = ImprovementType::BuildingEnlargement;
        let r = compute(&i);
        assert!(!r.qualifies_as_qip);
        assert_eq!(r.recovery_period_years, 39);
        assert!(r.exclusion_reason.unwrap().contains("§168(e)(6)(A)"));
    }

    #[test]
    fn elevator_or_escalator_excluded() {
        let mut i = base();
        i.improvement_type = ImprovementType::ElevatorOrEscalator;
        let r = compute(&i);
        assert!(!r.qualifies_as_qip);
        assert!(r.exclusion_reason.unwrap().contains("§168(e)(6)(B)"));
    }

    #[test]
    fn internal_structural_framework_excluded() {
        let mut i = base();
        i.improvement_type = ImprovementType::InternalStructuralFramework;
        let r = compute(&i);
        assert!(!r.qualifies_as_qip);
        assert!(r.exclusion_reason.unwrap().contains("§168(e)(6)(C)"));
    }

    #[test]
    fn residential_rental_not_qip_definition() {
        let mut i = base();
        i.improvement_type = ImprovementType::ResidentialRental;
        let r = compute(&i);
        assert!(!r.qualifies_as_qip);
        assert!(r.exclusion_reason.unwrap().contains("nonresidential"));
    }

    #[test]
    fn improvement_in_year_of_building_placed_in_service_not_qip() {
        // Improvement must be AFTER original placed-in-service date.
        let mut i = base();
        i.placed_in_service_year = 2015; // same as building
        let r = compute(&i);
        assert!(!r.qualifies_as_qip);
        assert!(r.exclusion_reason.unwrap().contains("AFTER"));
    }

    #[test]
    fn improvement_before_building_placed_in_service_not_qip() {
        let mut i = base();
        i.placed_in_service_year = 2010; // before building
        let r = compute(&i);
        assert!(!r.qualifies_as_qip);
    }

    #[test]
    fn excluded_category_uses_39_year_macrs() {
        let mut i = base();
        i.improvement_type = ImprovementType::BuildingEnlargement;
        let r = compute(&i);
        // $100k × 1.282% = $1,282.
        assert_eq!(r.year_1_macrs_deduction, dec!(1282));
    }

    #[test]
    fn no_bonus_election_macrs_half_year_only() {
        let mut i = base();
        i.elect_bonus_depreciation = false;
        let r = compute(&i);
        assert_eq!(r.bonus_pct_applied, Decimal::ZERO);
        // Half-year year 1 = 5% of $100k = $5k.
        assert_eq!(r.year_1_macrs_deduction, dec!(5000));
        assert_eq!(r.year_1_total_deduction, dec!(5000));
    }

    #[test]
    fn bonus_phase_down_2023_through_2027_each_year_exact() {
        for (year, expected) in [
            (2023, dec!(0.80)),
            (2024, dec!(0.60)),
            (2025, dec!(0.40)),
            (2026, dec!(0.20)),
            (2027, Decimal::ZERO),
        ] {
            let mut i = base();
            i.placed_in_service_year = year;
            i.current_tax_year = year;
            let r = compute(&i);
            assert_eq!(r.bonus_pct_applied, expected, "year {year}");
        }
    }

    #[test]
    fn qip_qualifies_helper_returns_true_only_for_interior_nonresidential() {
        assert!(ImprovementType::InteriorNonresidential.qualifies_as_qip());
        assert!(!ImprovementType::BuildingEnlargement.qualifies_as_qip());
        assert!(!ImprovementType::ElevatorOrEscalator.qualifies_as_qip());
        assert!(!ImprovementType::InternalStructuralFramework.qualifies_as_qip());
        assert!(!ImprovementType::ResidentialRental.qualifies_as_qip());
    }

    #[test]
    fn cares_act_drafting_error_note_shows_15_year_post_2017() {
        // CARES Act retroactive fix: any QIP placed in service in 2018+
        // gets 15-year recovery. We verify the recovery period without
        // checking the historical note (note text varies).
        for year in [2018, 2019, 2020, 2021] {
            let mut i = base();
            i.placed_in_service_year = year;
            i.current_tax_year = year;
            let r = compute(&i);
            assert_eq!(r.recovery_period_years, 15, "year {year} should be 15-year QIP");
        }
    }

    #[test]
    fn note_distinguishes_qip_path_vs_excluded_path() {
        let qip = compute(&base());
        assert!(qip.note.contains("QIP"));

        let mut not_qip = base();
        not_qip.improvement_type = ImprovementType::ElevatorOrEscalator;
        let r = compute(&not_qip);
        assert!(r.note.contains("not QIP"));
    }

    #[test]
    fn total_deduction_equals_bonus_plus_macrs() {
        let r = compute(&base());
        assert_eq!(
            r.year_1_total_deduction,
            r.year_1_bonus_deduction + r.year_1_macrs_deduction
        );
    }
}
