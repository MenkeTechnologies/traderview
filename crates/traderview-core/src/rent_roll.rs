//! Rent roll — a multi-unit rental income summary. For each unit it records the
//! monthly rent, whether it is occupied, and the square footage; from these it
//! computes scheduled rent (all units), actual rent (occupied only), the vacancy
//! loss, physical occupancy (occupied units ÷ total) and economic occupancy
//! (actual ÷ scheduled rent), the annualized totals, and the average rent per
//! square foot. Distinct from the single-property NOI module — this rolls up a
//! whole building. Pure compute, not investment advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Unit {
    pub label: String,
    pub monthly_rent_usd: f64,
    #[serde(default)]
    pub occupied: bool,
    #[serde(default)]
    pub sqft: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RentRollInput {
    pub property_label: String,
    pub units: Vec<Unit>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct RentRollReport {
    pub unit_count: usize,
    pub occupied_units: usize,
    pub scheduled_monthly_usd: f64,
    pub actual_monthly_usd: f64,
    pub vacancy_loss_monthly_usd: f64,
    /// Occupied units ÷ total units, percent.
    pub physical_occupancy_pct: f64,
    /// Actual rent ÷ scheduled rent, percent.
    pub economic_occupancy_pct: f64,
    pub annual_scheduled_usd: f64,
    pub annual_actual_usd: f64,
    /// Scheduled monthly rent ÷ total square footage (0 if no sqft given).
    pub avg_rent_per_sqft_usd: f64,
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn round4(x: f64) -> f64 {
    (x * 10_000.0).round() / 10_000.0
}

pub fn generate(i: &RentRollInput) -> RentRollReport {
    let n = i.units.len();
    if n == 0 {
        return RentRollReport::default();
    }
    let scheduled: f64 = i.units.iter().map(|u| u.monthly_rent_usd).sum();
    let actual: f64 = i.units.iter().filter(|u| u.occupied).map(|u| u.monthly_rent_usd).sum();
    let occupied = i.units.iter().filter(|u| u.occupied).count();
    let total_sqft: f64 = i.units.iter().map(|u| u.sqft).sum();

    RentRollReport {
        unit_count: n,
        occupied_units: occupied,
        scheduled_monthly_usd: cents(scheduled),
        actual_monthly_usd: cents(actual),
        vacancy_loss_monthly_usd: cents(scheduled - actual),
        physical_occupancy_pct: round2(occupied as f64 / n as f64 * 100.0),
        economic_occupancy_pct: if scheduled > 0.0 { round2(actual / scheduled * 100.0) } else { 0.0 },
        annual_scheduled_usd: cents(scheduled * 12.0),
        annual_actual_usd: cents(actual * 12.0),
        avg_rent_per_sqft_usd: if total_sqft > 0.0 { round4(scheduled / total_sqft) } else { 0.0 },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    fn u(label: &str, rent: f64, occ: bool, sqft: f64) -> Unit {
        Unit { label: label.into(), monthly_rent_usd: rent, occupied: occ, sqft }
    }

    fn base() -> RentRollInput {
        RentRollInput {
            property_label: "Maple Court".into(),
            units: vec![
                u("A", 1500.0, true, 800.0),
                u("B", 1600.0, true, 850.0),
                u("C", 1400.0, false, 750.0),
                u("D", 1800.0, true, 1000.0),
            ],
        }
    }

    #[test]
    fn scheduled_actual_vacancy() {
        let d = generate(&base());
        assert_eq!(d.unit_count, 4);
        assert_eq!(d.occupied_units, 3);
        assert!(close(d.scheduled_monthly_usd, 6_300.0));
        assert!(close(d.actual_monthly_usd, 4_900.0));
        assert!(close(d.vacancy_loss_monthly_usd, 1_400.0));
    }

    #[test]
    fn physical_vs_economic_occupancy() {
        let d = generate(&base());
        assert!(close(d.physical_occupancy_pct, 75.0));
        assert!(close(d.economic_occupancy_pct, 77.78));
    }

    #[test]
    fn annualized_and_per_sqft() {
        let d = generate(&base());
        assert!(close(d.annual_scheduled_usd, 75_600.0));
        assert!(close(d.annual_actual_usd, 58_800.0));
        assert!(close(d.avg_rent_per_sqft_usd, 1.8529));
    }

    #[test]
    fn fully_occupied_equal_occupancy() {
        let mut inp = base();
        for unit in inp.units.iter_mut() {
            unit.occupied = true;
        }
        let d = generate(&inp);
        assert!(close(d.physical_occupancy_pct, 100.0));
        assert!(close(d.economic_occupancy_pct, 100.0));
        assert!(close(d.vacancy_loss_monthly_usd, 0.0));
    }

    #[test]
    fn empty_units_zero() {
        let d = generate(&RentRollInput { property_label: "X".into(), units: vec![] });
        assert_eq!(d.unit_count, 0);
        assert!(close(d.scheduled_monthly_usd, 0.0));
    }
}
