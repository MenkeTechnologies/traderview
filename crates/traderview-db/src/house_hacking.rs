//! House hacking — buy a small multi-unit, live in one, rent the rest.
//!
//! The canonical wealth-building starter move: the tenants' rent offsets
//! (or erases) your housing cost, so the money that would have been rent
//! goes to principal and savings instead. This nets the rental income
//! against the full carrying cost (P&I + tax + insurance + maintenance +
//! HOA) to show what you actually pay to live there, compares it to
//! renting a comparable place, and reports the property's standalone cash
//! flow once you move out and rent every unit.
//!
//! Pure compute — the P&I uses the shared
//! [`crate::mortgage_amortization::monthly_payment`] so there's one
//! amortization formula in the codebase.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct HouseHackInput {
    pub home_price_usd: f64,
    pub down_payment_usd: f64,
    pub apr_pct: f64,
    pub term_months: u32,
    pub total_units: u32,
    pub owner_units: u32,
    /// Monthly rent collected per rented unit.
    pub rent_per_unit_usd: f64,
    pub monthly_tax_usd: f64,
    pub monthly_insurance_usd: f64,
    pub monthly_maintenance_usd: f64,
    pub monthly_hoa_usd: f64,
    /// What renting a comparable place for yourself would cost — the
    /// baseline the house hack is measured against.
    pub comparable_rent_usd: f64,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct HouseHackResult {
    pub principal_financed_usd: f64,
    pub monthly_pi_usd: f64,
    pub rented_units: u32,
    pub rental_income_usd: f64,
    /// P&I + tax + insurance + maintenance + HOA.
    pub total_housing_cost_usd: f64,
    /// Carrying cost net of rent — what you actually pay to live there.
    /// Negative means the tenants more than cover the place.
    pub net_housing_cost_usd: f64,
    /// `comparable_rent − net_housing_cost`: monthly saved vs renting.
    pub savings_vs_renting_usd: f64,
    /// Does one unit's rent already cover the mortgage P&I?
    pub rent_covers_pi: bool,
    /// Cash flow once you move out and rent every unit.
    pub full_rental_cash_flow_usd: f64,
}

pub fn compute(i: &HouseHackInput) -> HouseHackResult {
    let principal = (i.home_price_usd - i.down_payment_usd).max(0.0);
    let monthly_pi = crate::mortgage_amortization::monthly_payment(principal, i.apr_pct, i.term_months);

    let owner = i.owner_units.min(i.total_units);
    let rented = i.total_units.saturating_sub(owner);
    let rental_income = rented as f64 * i.rent_per_unit_usd;

    let operating = i.monthly_tax_usd
        + i.monthly_insurance_usd
        + i.monthly_maintenance_usd
        + i.monthly_hoa_usd;
    let total_housing_cost = monthly_pi + operating;
    let net_housing_cost = total_housing_cost - rental_income;

    let full_rental_income = i.total_units as f64 * i.rent_per_unit_usd;

    HouseHackResult {
        principal_financed_usd: principal,
        monthly_pi_usd: monthly_pi,
        rented_units: rented,
        rental_income_usd: rental_income,
        total_housing_cost_usd: total_housing_cost,
        net_housing_cost_usd: net_housing_cost,
        savings_vs_renting_usd: i.comparable_rent_usd - net_housing_cost,
        rent_covers_pi: rental_income >= monthly_pi,
        full_rental_cash_flow_usd: full_rental_income - total_housing_cost,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn duplex() -> HouseHackInput {
        HouseHackInput {
            home_price_usd: 400_000.0,
            down_payment_usd: 80_000.0,
            apr_pct: 6.0,
            term_months: 360,
            total_units: 2,
            owner_units: 1,
            rent_per_unit_usd: 1_500.0,
            monthly_tax_usd: 400.0,
            monthly_insurance_usd: 100.0,
            monthly_maintenance_usd: 200.0,
            monthly_hoa_usd: 0.0,
            comparable_rent_usd: 1_800.0,
        }
    }

    #[test]
    fn nets_rent_against_carrying_cost() {
        let r = compute(&duplex());
        assert!((r.principal_financed_usd - 320_000.0).abs() < 1e-6);
        assert_eq!(r.rented_units, 1);
        assert!((r.rental_income_usd - 1_500.0).abs() < 1e-9);
        // Identities the result must satisfy, independent of the exact P&I.
        assert!((r.total_housing_cost_usd - (r.monthly_pi_usd + 700.0)).abs() < 1e-6);
        assert!((r.net_housing_cost_usd - (r.total_housing_cost_usd - r.rental_income_usd)).abs() < 1e-6);
        assert!((r.savings_vs_renting_usd - (1_800.0 - r.net_housing_cost_usd)).abs() < 1e-6);
    }

    #[test]
    fn monthly_pi_matches_shared_amortization() {
        let r = compute(&duplex());
        let expect = crate::mortgage_amortization::monthly_payment(320_000.0, 6.0, 360);
        assert!((r.monthly_pi_usd - expect).abs() < 1e-9);
        // ~$1,918/mo on a $320k 30-yr at 6%.
        assert!((r.monthly_pi_usd - 1_918.0).abs() < 5.0, "got {}", r.monthly_pi_usd);
        assert!(!r.rent_covers_pi); // one unit's $1,500 < ~$1,918 P&I
    }

    #[test]
    fn fourplex_can_pay_you_to_live_there() {
        // 4 units, live in one, rent three at $1,500 = $4,500 income.
        let r = compute(&HouseHackInput {
            total_units: 4,
            ..duplex()
        });
        assert_eq!(r.rented_units, 3);
        assert!((r.rental_income_usd - 4_500.0).abs() < 1e-9);
        // Income > carrying cost → negative net housing cost (paid to live).
        assert!(r.net_housing_cost_usd < 0.0, "net {}", r.net_housing_cost_usd);
        assert!(r.rent_covers_pi);
    }

    #[test]
    fn full_rental_cash_flow_after_moving_out() {
        let r = compute(&duplex());
        // Both units rented = $3,000 less the total carrying cost.
        let expect = 3_000.0 - r.total_housing_cost_usd;
        assert!((r.full_rental_cash_flow_usd - expect).abs() < 1e-6);
    }

    #[test]
    fn owner_units_capped_at_total() {
        // Claiming to occupy more units than exist leaves zero rented.
        let r = compute(&HouseHackInput {
            total_units: 2,
            owner_units: 5,
            ..duplex()
        });
        assert_eq!(r.rented_units, 0);
        assert_eq!(r.rental_income_usd, 0.0);
    }
}
